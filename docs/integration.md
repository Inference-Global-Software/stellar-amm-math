# Building a pool on stellar-amm-math

The contract is the pricing core of a Uniswap V2 pair and nothing else: it
holds no state, moves no tokens and checks no authorization. A pool that uses
it keeps its own reserves, liquidity-token balances and transfers, and asks
this contract for every number V2's `UniswapV2Library` and `UniswapV2Pair`
would compute. This page says what those numbers mean, what the pool must
still check itself, and which parts of V2 the contract does not provide.

`src/main.inf` is the authority: its header gives the formulas, and its
`Refusals` table maps every trap to the V2 require it mirrors. Nothing here
overrides them.

## Calling the contract

From another Soroban contract, or from Rust tests, the contract's
`contractspecv0` section gives a typed client, exactly as for a contract
written with soroban-sdk:

```rust
mod amm {
    // The path is relative to the importing crate's Cargo.toml.
    soroban_sdk::contractimport!(file = "../out/main.wasm");
}

let amm = amm::Client::new(&env, &amm_contract_id);
let out = amm.get_amount_out(&amount_in, &reserve_in, &reserve_out, &30);
```

`tests/src/host.rs` imports the contract this way, and every host test goes
through that client. From the command line, each parameter name is a flag:

```bash
stellar contract invoke --id <CONTRACT_ID> --source <KEY> --network testnet \
  -- get_amount_out --amount_in 1000 --reserve_in 100000 --reserve_out 100000 --fee_bps 30
```

Every refusal is a trap. A plain client call panics, and a `try_` call returns
`Err(Ok(Error(Context, InvalidAction)))` whatever the reason: a failed
precondition, an overflow and a malformed argument look the same to the
caller. The `Refusals` table is the only map from a failure back to its input,
so a pool that wants its own error codes checks the preconditions before it
calls.

Amounts are `u32`: the v0.0.6 Stellar target carries only `u32`, `i32` and
`bool` across the contract boundary. A pool that stores token amounts as
`i128` converts each one with `u32::try_from` and refuses what does not fit:
at 7 decimals, about 429 whole tokens per argument.

## The k checks take balance changes

`k_holds_with_fee` is V2's `UniswapV2: K` check,
`balance0Adjusted * balance1Adjusted >= reserve0 * reserve1 * 1000^2`, for a
swap with its input on one side and its output on the other, at D = 10000 in
place of 1000 and with one factor of D cancelled from both sides; `k_holds` is
the same check at fee 0. V2 reads both sides from the pool's balances after
the transfers, so both amounts are balance changes:

- `amount_in` is V2's `amountIn`, the input side's balance minus its stored
  reserve;
- `amount_out` is the output side's stored reserve minus its balance.

That equals V2's `amount1Out` only when the transfer out takes exactly
`amount1Out` from the pool and nothing pays into the output side during the
swap. When the output side's balance ends above `reserve_out - amount1Out`
(V2's `amount1In > 0`: a flash swap repaid, even partly, in the borrowed
token, a positive rebase, a transfer in during the callback), V2 charges the
fee on that inflow and neither argument reproduces its check:
`reserve - balance` returns true where V2 reverts with `K`, and the requested
amount can return false where V2 succeeds. The pool must refuse such a swap:
test `balance_out > reserve_out - amount1Out` before calling the check. A pool
that passes the requested amount instead, with a token that takes more from
the pool than it delivers (a sender-charged fee, a rebase during the swap),
gets true where V2 reverts with `UniswapV2: K`.

## Liquidity: the locked minimum and the supply

The 1000 units `mint_initial` withholds are the `MINIMUM_LIQUIDITY` that V2
mints to `address(0)` and locks for good; `minimum_liquidity()` returns it.
V2's `totalSupply` counts them, so after the first deposit the pool records
`total_supply = mint_initial(...) + minimum_liquidity()`, and that is the
supply `mint_proportional` and `burn_share` must receive. A pool that records
`mint_initial(...)` alone lets the first provider burn the whole pool.

`balance` in `burn_share` is the pool's current balance of the token, as V2's
burn uses (`balance0` and `balance1`, not the stored reserves), so a transfer
the pool has not yet synced is paid out pro rata too; `mint_proportional`,
like V2's mint, divides by the stored reserves. V2 requires both tokens'
amounts to be positive: call `burn_share` once per token, and either trap
refuses the burn.

## What the pool checks itself

These V2 requires guard state or amounts only the pool has, so the contract
has no row for them (the `Refusals` table lists them after the table):

- A swap that pays nothing out (`UniswapV2: INSUFFICIENT_OUTPUT_AMOUNT`): the
  k checks return true for `amount_out = 0`.
- A balance above what the pool can pass back in (`UniswapV2: OVERFLOW`, which
  V2's `_update` raises in swap, mint, burn and sync): every balance the pool
  stores must stay at most 4294967295, and neither k check tests
  `reserve_in + amount_in`.
- The requested `amount1Out` against the reserve before the transfer
  (`UniswapV2: INSUFFICIENT_LIQUIDITY`), and mint's
  `amount0 = balance0 - reserve0` (`ds-math-sub-underflow`).
- Re-entrancy, recipients and transfers (`LOCKED`, `INVALID_TO`,
  `TRANSFER_FAILED`).

## Not provided

Three parts of V2's pair have no method here; a pool built on this contract
implements or refuses them itself.

- **The protocol fee** (`_mintFee`). With it switched on, V2's mint and burn
  first mint `floor(S*(rootK - rootKLast) / (5*rootK + rootKLast))` to
  `feeTo` when `kLast != 0` and `rootK > rootKLast`, where
  `rootK = floor(sqrt(R0*R1))`, `rootKLast = floor(sqrt(kLast))`, and `kLast`
  is `R0*R1` after the last mint or burn made while the fee was on. V2 records
  `kLast` only then, and resets it to 0 at the first mint or burn after the
  fee is switched off. The mint or burn then passes the increased supply to
  `mint_proportional` or `burn_share`. The methods here cannot compute that
  fee in general: `mint_initial(R0, R1) + minimum_liquidity()` gives `rootK`
  only when `R0*R1 >= 1001^2`, `5*rootK + rootKLast` can exceed a `u32`, and
  `kLast` is a 64-bit product, so a pool that charges the fee needs its own
  wide arithmetic.
- **Swaps that are not one-sided.** The k checks take one input side and one
  output side. A swap that pays in on its output side (V2's flash swap repaid
  in the borrowed token), or that pays in or out on both sides, has no check
  here: the pool refuses it or computes V2's two-sided check itself. The test
  `balance_out > reserve_out - amount1Out` above detects a swap that pays in
  on its output side.
- **The price accumulators** (`price0CumulativeLast`, `price1CumulativeLast`)
  that V2's `_update` maintains.
