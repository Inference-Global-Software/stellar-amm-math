# Uniswap V2 AMM math — a Stellar smart contract in Inference

The pricing core of a [Uniswap V2](https://github.com/Uniswap/v2-core)
constant-product pair — swap quotes, liquidity minting and burning, and the
constant-product (`k`) check — written in the
[Inference](https://github.com/Inferara/inference) language and compiled to a
Stellar (Soroban) smart contract with the Inference toolchain
[v0.0.6](https://github.com/Inferara/inference/releases/tag/v0.0.6). Its results
equal V2's for every input it accepts, with the fee in basis points.

This repository is the reference for Inference's `stellar` target. It covers
building a contract with `infs`, testing it on the Soroban host from Rust,
deploying and calling it with the `stellar` CLI, and stating its properties
as Rocq proof obligations. It also doubles as a fixed workload for the
toolchain. Every compiler release gets a ledger row with the contract's
bytes and its deterministic host cost, in
[`bench/history.jsonl`](bench/history.jsonl).

**Live on testnet:**
[`CB4WEDZCFTWGFMLCBVYDNS35M6UR67GIEAS47QPZNFMYCQTZECGJ4WZH`](https://stellar.expert/explorer/testnet/contract/CB4WEDZCFTWGFMLCBVYDNS35M6UR67GIEAS47QPZNFMYCQTZECGJ4WZH)
([record](deployments/testnet.json)).

```console
$ docker compose run --rm cli-net keys generate me --network testnet --fund
$ docker compose run --rm cli-net contract invoke --network testnet --source me \
    --id CB4WEDZCFTWGFMLCBVYDNS35M6UR67GIEAS47QPZNFMYCQTZECGJ4WZH \
    -- get_amount_out --amount_in 1000 --reserve_in 100000 --reserve_out 100000 --fee_bps 30
ℹ️  Simulation identified as read-only. Send by rerunning with `--send=yes`.
987
```

## What the contract shows about the Stellar target

| Target feature (v0.0.6) | How this contract uses it |
|---|---|
| **`[build] target = "stellar"`** ([Inferara/inference#324](https://github.com/Inferara/inference/issues/324)) | `infs build` turns every `pub fn` of `src/main.inf` into a contract method, wrapped in Soroban's value ABI and stamped with `contractenvmetav0` (protocol 20). No SDK and no macros are involved. |
| **`contractspecv0` / `contractmetav0`** ([#466](https://github.com/Inferara/inference/issues/466)) | The CLI reads a typed interface, and each parameter name becomes a flag (`--amount_in`). Rust code imports the contract with `soroban_sdk::contractimport!`, exactly like an SDK contract. |
| **Only `u32`, `i32` and `bool` at the boundary** | Amounts and reserves are `u32`. A wider or compound value crosses as a host object, which the target cannot build yet ([#472](https://github.com/Inferara/inference/issues/472)). |
| **No host imports**, so no storage, events or auth | The contract is stateless: a pool keeps the state and delegates the math ([docs/integration.md](docs/integration.md)). |
| **Arithmetic traps on overflow by default** ([#316](https://github.com/Inferara/inference/issues/316)) | Every `+`, `-` and `*` is checked. `wrapping(...)` marks the ten intentional modular additions and subtractions, on limbs 0–2 of the 128-bit `add` and `sub`; the top limb stays checked, so a 128-bit overflow traps. |
| **`assert`** | Each of V2's `require`s is an `assert`, and a failed one traps. |
| **Module hierarchy** ([#63](https://github.com/Inferara/inference/issues/63)) | Three files: `main.inf` (the methods), `wide.inf` (128-bit arithmetic) and `model.inf` (the bridge for the specs). |
| **`spec` blocks → Rocq** | There are 32 `forall` obligations. The `wasm32` build carries them, and the `stellar` build deploys the same executable code. |

## The interface

`stellar contract info interface --wasm out/main.wasm` (and the deployed
contract) reports:

```rust
pub trait Contract {
    fn minimum_liquidity(env) -> u32;
    fn get_amount_out(env, amount_in: u32, reserve_in: u32, reserve_out: u32, fee_bps: u32) -> u32;
    fn get_amount_in(env, amount_out: u32, reserve_in: u32, reserve_out: u32, fee_bps: u32) -> u32;
    fn quote(env, amount_a: u32, reserve_a: u32, reserve_b: u32) -> u32;
    fn mint_initial(env, amount0: u32, amount1: u32) -> u32;
    fn mint_proportional(env, amount0: u32, amount1: u32, reserve0: u32, reserve1: u32, total_supply: u32) -> u32;
    fn burn_share(env, liquidity: u32, balance: u32, total_supply: u32) -> u32;
    fn k_holds(env, amount_in: u32, amount_out: u32, reserve_in: u32, reserve_out: u32) -> bool;
    fn k_holds_with_fee(env, amount_in: u32, amount_out: u32, reserve_in: u32, reserve_out: u32, fee_bps: u32) -> bool;
    fn mul_div(env, a: u32, b: u32, denominator: u32) -> u32;
    fn mul_div_up(env, a: u32, b: u32, denominator: u32) -> u32;
}
```

(Abridged: each `env` is a `soroban_sdk::Env`. The CLI's exact output is
[`e2e/interface.txt`](e2e/interface.txt), which the end-to-end test compares
byte for byte.)

The formulas, their rounding, and the V2 `require` each refusal mirrors are
in the header of [`src/main.inf`](src/main.inf). At `fee_bps = 30`, the
formulas reproduce V2's 997/1000 exactly.

Every refusal reaches the caller as the same WebAssembly trap
(`Error(WasmVm, InvalidAction)`, "UnreachableCodeReached"), and a
`try_` call sees `Error(Context, InvalidAction)`.

## Why the amounts are u32, and how the math stays exact

Only `u32`, `i32` and `bool` cross the v0.0.6 Stellar boundary, so every
amount is a `u32`: at most 4,294,967,295 smallest units, or about 429 whole
tokens at Stellar's 7 decimals. V2's formulas need wider intermediates: `a·(10000−fee)·Ro` alone
reaches about 2^77.3. The language has no numeric conversions and no type
wider than 64 bits, so [`src/wide.inf`](src/wide.inf) builds a 128-bit
integer from four `u32` limbs. Products come from 16-bit column products.
Quotients come from restoring division that is guarded by an assert, so no
native division instruction ever divides by zero. `isqrt` finds its result
bit by bit. The `u32` bound therefore applies to arguments and results only,
and every result is exact.

## Layout

```
Inference.toml          [build] target = "stellar"
src/main.inf            the 11 methods, the refusal table, spec PairMath (25 obligations)
src/wide.inf            128-bit unsigned arithmetic over u32 limbs
src/model.inf           the scalar bridge the specs are written against, spec WideArith (7)
out/main.wasm           the deployable contract (9,409 bytes), committed
tests/                  Rust crate: the host tests, the V2 oracle, the cost binary
tests/shell/            a wasm32 build exporting the wide and model helpers for the tests
proofs/                 main.v (Rocq) and main.wasm (the module it describes); generate.sh, check.sh
e2e/                    interface.txt and vectors.tsv (120 CLI calls) for scripts/e2e-local.sh
bench/                  the toolchain ledger: history.jsonl, RESULTS.md, snapshot.sh, kept modules
compose.yaml, docker/   local network, pinned stellar CLI and the two Rocq checkers
scripts/                e2e-local.sh, deploy-testnet.sh, check-testnet.sh, Rocq staging helpers
deployments/            testnet.json, the live deployment
docs/integration.md     how a pool calls these methods, and what it must check itself
```

## Build, test, deploy

You need the Inference toolchain [v0.0.6](https://github.com/Inferara/inference/releases/tag/v0.0.6)
or newer (`infs` and `infc`), Rust 1.91 or newer for the tests, and Docker with
Compose v2 for everything that talks to a network or to Rocq. Nothing else is
installed locally: the Stellar network, the `stellar` CLI and Coq run in
containers.

```bash
infs build                                          # out/main.wasm
(cd tests/shell && infs build)                      # the test shell
(cd tests && cargo test --locked -- --include-ignored)
bash scripts/e2e-local.sh                           # local network: deploy, call 120 vectors, tear down
```

`infs build` prints the methods it exported:

```
Stellar contract: minimum_liquidity/0, get_amount_out/4, get_amount_in/4, quote/3, mint_initial/2, mint_proportional/5, burn_share/3, k_holds/4, k_holds_with_fee/5, mul_div/3, mul_div_up/3; env protocol 20; 9409 bytes
```

To work against a local network by hand:

```bash
docker compose up -d --wait stellar
docker compose run --rm cli keys generate alice --network local --fund
docker compose run --rm cli contract deploy --wasm out/main.wasm --source alice --network local
docker compose run --rm cli contract invoke --id <ID> --source alice --network local -- quote --amount_a 100 --reserve_a 1000 --reserve_b 3000
docker compose --profile '*' down -v
```

`scripts/deploy-testnet.sh` deploys to testnet with a friendbot-funded key
and writes `deployments/testnet.json`. `scripts/check-testnet.sh` checks that
the recorded contract is still there and holds `out/main.wasm`, and that it
answers the reference swap. CI runs the check weekly, because testnet is reset
from time to time.

## What the tests check

`tests/` is a standalone Rust crate on soroban-sdk 28. It imports the
contract with `contractimport!`, the same way any Soroban project imports a
deployed contract. The suite has 138 tests:

- **`contract.rs`** runs every method on the in-process Soroban host against
  an independent V2 oracle in `u128`. The inputs are:
  - V2's own reference values;
  - the edge cross product;
  - seeded random inputs;
  - directed boundaries;
  - one trap per refusal-table row.
- **`wasm32.rs`** runs the proof module (`proofs/main.wasm`) under wasmi on
  the same inputs.
- **`wide.rs`** runs the 128-bit helpers through `tests/shell` against
  `u128`, including every carry and borrow path. The contract's methods never
  reach the high limbs, so a carry-dropping mutant survives every
  contract-level test.
- **`specs.rs`** evaluates each of the 32 spec obligations numerically at the
  boundaries of its assumptions. A proof build accepts a false obligation as
  readily as a true one.
- **`repo.rs`** checks the repository's invariants: the shell's copies of
  `src/`, the refusal table, the Rocq ordinals, the ledger, and the byte
  identity of the proof module's and the contract's executable code.

`scripts/e2e-local.sh` then deploys the contract to a real local network and
makes 120 calls through the CLI.

## Proofs: prove the wasm32 build, deploy the Stellar one

The `stellar` target refuses proof mode, so `proofs/generate.sh` builds the
same source for `wasm32` with `infc -v`. `proofs/main.v` states the module
and the obligations as Rocq theorems (`valid_main`, `valid_main__PairMath`,
`valid_main__model_WideArith`). `proofs/main.wasm` is the module they
describe. Its 30 executable function bodies are byte-identical to the
deployed contract's, and `tests/contract.rs` checks this. The Stellar build
adds only the value-ABI wrappers.

```bash
bash proofs/generate.sh --check           # the committed proofs are what src/ builds
bash proofs/check.sh stub --self-test     # type-check against the public signature stub
bash proofs/check.sh real --self-test     # against the real library (maintainers only, see below)
```

The checks establish that `main.v` is well formed and states every expected
obligation. They do not prove the obligations: the theorems end in `Admitted`
until a proof discharges them. The `real` tier uses the wasm-verifier
library, which is not public. Stage it from a clone with
`scripts/stage-wasm-verifier.sh`, and read `proofs/check.sh` for how to
remove every copy afterwards. CI runs only the `stub` tier.

## Cost ledger and toolchain tracking

`bench/snapshot.sh` rebuilds everything with a given toolchain and measures a
fixed workload on the in-process host. It records the instructions each call
costs above a same-contract baseline, and the Wasm fuel it burns. Both are
deterministic. It then appends a row to `bench/history.jsonl` and keeps the
module in `bench/modules/`.

At v0.0.6 (`4deba32`), with typical inputs:

| method | Δ instructions | Wasm fuel |
|---|---:|---:|
| get_amount_out | 72,194 | 18,118 |
| get_amount_in | 71,246 | 17,881 |
| quote | 58,908 | 14,910 |
| mint_initial | 62,734 | 15,757 |
| mint_proportional | 124,724 | 31,248 |
| burn_share | 61,138 | 15,356 |
| k_holds | 11,144 | 2,967 |
| k_holds_with_fee | 29,276 | 7,386 |
| mul_div / mul_div_up | 62,308 / 64,138 | 15,760 / 16,106 |

A call's whole cost, VM instantiation included, is 107,675 to 269,943
instructions, under a thousandth of a transaction's limit.
[`bench/RESULTS.md`](bench/RESULTS.md) has the worst cases and the method.

[`canary.yml`](.github/workflows/canary.yml) builds with two toolchains on
every push, on every pull request and weekly:

- **the ledger's release:** the committed modules and proofs must be exactly
  what it builds, and the host cost must equal the row's;
- **the latest release:** its summary reports what changed.

## Limitations

- **Amounts are `u32`.** Real Soroban token amounts are `i128`, and they
  wait on host-object support in the target ([#324](https://github.com/Inferara/inference/issues/324),
  [#472](https://github.com/Inferara/inference/issues/472)).
- **No state.** The contract cannot hold reserves, mint liquidity tokens or
  move funds. [docs/integration.md](docs/integration.md) describes what a
  pool keeps and checks, and which parts of V2 (the protocol fee, two-sided
  swaps, the price accumulators) have no method here.
- **Refusals are indistinguishable.** Every failure is the same trap; the
  refusal table in `src/main.inf` maps each one to its cause.
- **Spec-only code is deployed too.** The compiler has no dead-code
  elimination, so `src/model.inf` ships in the deployed contract (830 bytes).
- **The obligations are stated, not proved.** The theorems end in `Admitted`.
- **Comparing structs.** `==` on a struct compares addresses in executable
  code ([#384](https://github.com/Inferara/inference/issues/384)), so
  `wide.inf` compares limbs with `eq`.

## License

Apache-2.0 (see [LICENSE](LICENSE)).
