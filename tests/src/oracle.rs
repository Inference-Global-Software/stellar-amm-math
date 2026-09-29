//! Uniswap V2's formulas in `u128`, as an oracle for the contract.
//!
//! Written from UniswapV2Library (getAmountOut, getAmountIn, quote),
//! UniswapV2Pair (mint, burn, the 'UniswapV2: K' check) and v3-core's FullMath
//! (mulDiv, mulDivRoundingUp), at a fee denominator of 10000 instead of 1000.
//! Every product of two or three `u32` operands fits a `u128`, so each formula
//! is evaluated exactly as V2 writes it, with no limbs. Each function refuses
//! exactly where the contract traps, and names the row of the refusal table in
//! `src/main.inf` that refuses.

/// The fee denominator: fees are in basis points.
const D: u128 = 10_000;

/// The liquidity the first deposit locks for good (V2's MINIMUM_LIQUIDITY).
pub const MINIMUM_LIQUIDITY: u32 = 1000;

/// A row of the refusal table in `src/main.inf`: the condition that failed,
/// with the V2 require it mirrors. The contract cannot tell them apart (every
/// refusal is the same trap); only this oracle names them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// `amount_in > 0` in get_amount_out: INSUFFICIENT_INPUT_AMOUNT.
    ZeroAmountIn,
    /// `amount_out > 0` in get_amount_in: INSUFFICIENT_OUTPUT_AMOUNT.
    ZeroAmountOut,
    /// `amount_a > 0` in quote: INSUFFICIENT_AMOUNT.
    ZeroAmountA,
    /// Both reserves > 0 (the swap and quote methods, where it is
    /// INSUFFICIENT_LIQUIDITY, and mint_proportional, a zero divisor in V2),
    /// or `reserve_in > 0` in the k checks (swap's INSUFFICIENT_LIQUIDITY).
    ZeroReserve,
    /// `amount_out < reserve_out` in get_amount_in (ds-math-sub-underflow, or a
    /// zero divisor at equality) and in the k checks (K when amount_out is the
    /// output side's balance change, INSUFFICIENT_LIQUIDITY when it is the
    /// requested amount).
    AmountOutNotBelowReserve,
    /// `fee_bps < 10000` wherever a fee is taken; V2's fee is a constant.
    FeeNotBelowDenominator,
    /// The result fits a u32: a uint256 in V2 (get_amount_in, quote), and
    /// FullMath's `require(denominator > prod1)` and its rounding-up require
    /// (mul_div, mul_div_up).
    ResultDoesNotFit,
    /// `floor(sqrt(a0*a1)) >= 1000` in mint_initial: ds-math-sub-underflow.
    RootBelowMinimum,
    /// `result > 0` in mint_initial and mint_proportional:
    /// INSUFFICIENT_LIQUIDITY_MINTED.
    NothingMinted,
    /// `total_supply > 0` in mint_proportional (V2 then uses the mint_initial
    /// formula) and burn_share (a zero divisor in V2).
    ZeroSupply,
    /// At least one share of mint_proportional fits a u32.
    NoShareFits,
    /// `liquidity <= total_supply` in burn_share.
    LiquidityAboveSupply,
    /// `result > 0` in burn_share: INSUFFICIENT_LIQUIDITY_BURNED.
    NothingBurned,
    /// `denominator > 0` in mul_div and mul_div_up: FullMath's first require.
    ZeroDenominator,
}

impl Refusal {
    pub const ALL: [Refusal; 14] = [
        Refusal::ZeroAmountIn,
        Refusal::ZeroAmountOut,
        Refusal::ZeroAmountA,
        Refusal::ZeroReserve,
        Refusal::AmountOutNotBelowReserve,
        Refusal::FeeNotBelowDenominator,
        Refusal::ResultDoesNotFit,
        Refusal::RootBelowMinimum,
        Refusal::NothingMinted,
        Refusal::ZeroSupply,
        Refusal::NoShareFits,
        Refusal::LiquidityAboveSupply,
        Refusal::NothingBurned,
        Refusal::ZeroDenominator,
    ];
}

/// V2's `require(condition, message)`.
fn require(condition: bool, refusal: Refusal) -> Result<(), Refusal> {
    if condition { Ok(()) } else { Err(refusal) }
}

/// The result as a u32, or [`Refusal::ResultDoesNotFit`].
fn fit(value: u128) -> Result<u32, Refusal> {
    u32::try_from(value).map_err(|_| Refusal::ResultDoesNotFit)
}

/// The share of an input that counts after the fee: 997 in V2, scaled by ten.
fn gamma(fee_bps: u32) -> u128 {
    D - u128::from(fee_bps)
}

#[must_use]
pub fn minimum_liquidity() -> u32 {
    MINIMUM_LIQUIDITY
}

/// UniswapV2Library.getAmountOut:
/// `amountIn * 997 * reserveOut / (reserveIn * 1000 + amountIn * 997)`.
pub fn get_amount_out(
    amount_in: u32,
    reserve_in: u32,
    reserve_out: u32,
    fee_bps: u32,
) -> Result<u32, Refusal> {
    require(amount_in > 0, Refusal::ZeroAmountIn)?;
    require(reserve_in > 0 && reserve_out > 0, Refusal::ZeroReserve)?;
    require(u128::from(fee_bps) < D, Refusal::FeeNotBelowDenominator)?;
    let amount_in_with_fee = u128::from(amount_in) * gamma(fee_bps);
    let numerator = amount_in_with_fee * u128::from(reserve_out);
    let denominator = u128::from(reserve_in) * D + amount_in_with_fee;
    fit(numerator / denominator)
}

/// UniswapV2Library.getAmountIn:
/// `reserveIn * amountOut * 1000 / ((reserveOut - amountOut) * 997) + 1`.
pub fn get_amount_in(
    amount_out: u32,
    reserve_in: u32,
    reserve_out: u32,
    fee_bps: u32,
) -> Result<u32, Refusal> {
    require(amount_out > 0, Refusal::ZeroAmountOut)?;
    require(reserve_in > 0 && reserve_out > 0, Refusal::ZeroReserve)?;
    require(amount_out < reserve_out, Refusal::AmountOutNotBelowReserve)?;
    require(u128::from(fee_bps) < D, Refusal::FeeNotBelowDenominator)?;
    let numerator = u128::from(reserve_in) * u128::from(amount_out) * D;
    let denominator = u128::from(reserve_out - amount_out) * gamma(fee_bps);
    fit(numerator / denominator + 1)
}

/// UniswapV2Library.quote: `amountA * reserveB / reserveA`.
pub fn quote(amount_a: u32, reserve_a: u32, reserve_b: u32) -> Result<u32, Refusal> {
    require(amount_a > 0, Refusal::ZeroAmountA)?;
    require(reserve_a > 0 && reserve_b > 0, Refusal::ZeroReserve)?;
    fit(u128::from(amount_a) * u128::from(reserve_b) / u128::from(reserve_a))
}

/// UniswapV2Pair.mint at zero supply:
/// `Math.sqrt(amount0 * amount1) - MINIMUM_LIQUIDITY`, which must be positive.
///
/// # Panics
///
/// Never: the root of a product of two `u32` values fits a `u32`.
pub fn mint_initial(amount0: u32, amount1: u32) -> Result<u32, Refusal> {
    let root = (u128::from(amount0) * u128::from(amount1)).isqrt();
    let root = fit(root).expect("the root of a product of two u32 values fits a u32");
    let minted = root.checked_sub(MINIMUM_LIQUIDITY).ok_or(Refusal::RootBelowMinimum)?;
    require(minted > 0, Refusal::NothingMinted)?;
    Ok(minted)
}

/// UniswapV2Pair.mint at a positive supply:
/// `Math.min(amount0 * totalSupply / reserve0, amount1 * totalSupply / reserve1)`.
pub fn mint_proportional(
    amount0: u32,
    amount1: u32,
    reserve0: u32,
    reserve1: u32,
    total_supply: u32,
) -> Result<u32, Refusal> {
    require(reserve0 > 0 && reserve1 > 0, Refusal::ZeroReserve)?;
    require(total_supply > 0, Refusal::ZeroSupply)?;
    let supply = u128::from(total_supply);
    let share0 = u128::from(amount0) * supply / u128::from(reserve0);
    let share1 = u128::from(amount1) * supply / u128::from(reserve1);
    let minted = u32::try_from(share0.min(share1)).map_err(|_| Refusal::NoShareFits)?;
    require(minted > 0, Refusal::NothingMinted)?;
    Ok(minted)
}

/// UniswapV2Pair.burn, for one token: `liquidity * balance / totalSupply`.
///
/// # Panics
///
/// Never: with `liquidity <= total_supply` the share is at most the balance.
pub fn burn_share(liquidity: u32, balance: u32, total_supply: u32) -> Result<u32, Refusal> {
    require(total_supply > 0, Refusal::ZeroSupply)?;
    require(liquidity <= total_supply, Refusal::LiquidityAboveSupply)?;
    let amount = u128::from(liquidity) * u128::from(balance) / u128::from(total_supply);
    let amount = fit(amount).expect("liquidity <= total_supply keeps the share within the balance");
    require(amount > 0, Refusal::NothingBurned)?;
    Ok(amount)
}

/// UniswapV2Pair.swap's 'UniswapV2: K' check at fee 0, with the input on one
/// side and the output on the other: `(Ri + a) * (Ro - o) >= Ri * Ro`.
pub fn k_holds(
    amount_in: u32,
    amount_out: u32,
    reserve_in: u32,
    reserve_out: u32,
) -> Result<bool, Refusal> {
    require(reserve_in > 0, Refusal::ZeroReserve)?;
    require(amount_out < reserve_out, Refusal::AmountOutNotBelowReserve)?;
    let balance_in = u128::from(reserve_in) + u128::from(amount_in);
    let balance_out = u128::from(reserve_out - amount_out);
    Ok(balance_in * balance_out >= u128::from(reserve_in) * u128::from(reserve_out))
}

/// UniswapV2Pair.swap's 'UniswapV2: K' check:
/// `balance0Adjusted * balance1Adjusted >= reserve0 * reserve1 * 1000**2`,
/// where `balanceAdjusted = balance * 1000 - amountIn * 3`. The output side
/// takes no input, so its adjusted balance is `(Ro - o) * D`; one factor of D
/// cancels from both sides.
pub fn k_holds_with_fee(
    amount_in: u32,
    amount_out: u32,
    reserve_in: u32,
    reserve_out: u32,
    fee_bps: u32,
) -> Result<bool, Refusal> {
    require(u128::from(fee_bps) < D, Refusal::FeeNotBelowDenominator)?;
    require(reserve_in > 0, Refusal::ZeroReserve)?;
    require(amount_out < reserve_out, Refusal::AmountOutNotBelowReserve)?;
    let balance_in = u128::from(reserve_in) + u128::from(amount_in);
    let balance_in_adjusted = balance_in * D - u128::from(amount_in) * u128::from(fee_bps);
    let balance_out = u128::from(reserve_out - amount_out);
    let reserves = u128::from(reserve_in) * u128::from(reserve_out);
    Ok(balance_in_adjusted * balance_out >= reserves * D)
}

/// FullMath.mulDiv: `a * b / denominator`, rounding down.
pub fn mul_div(a: u32, b: u32, denominator: u32) -> Result<u32, Refusal> {
    require(denominator > 0, Refusal::ZeroDenominator)?;
    fit(u128::from(a) * u128::from(b) / u128::from(denominator))
}

/// FullMath.mulDivRoundingUp: mulDiv, plus one when `mulmod(a, b, d) > 0`.
pub fn mul_div_up(a: u32, b: u32, denominator: u32) -> Result<u32, Refusal> {
    require(denominator > 0, Refusal::ZeroDenominator)?;
    let product = u128::from(a) * u128::from(b);
    let denominator = u128::from(denominator);
    let floor = product / denominator;
    let rounded_up = if product % denominator > 0 { floor + 1 } else { floor };
    fit(rounded_up)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::method::{Method, Value, render_call};
    use crate::rows;
    use crate::vectors;

    /// The values Uniswap V2 itself produces at its 0.3% fee, and the ones
    /// derived by hand from its formulas.
    #[test]
    fn reproduces_v2_at_thirty_basis_points() {
        assert_eq!(get_amount_out(1000, 100_000, 100_000, 30), Ok(987));
        assert_eq!(get_amount_in(987, 100_000, 100_000, 30), Ok(1000));
        assert_eq!(get_amount_out(1, 1_000_000, 1000, 30), Ok(0));
        assert_eq!(mint_initial(1_000_000, 4_000_000), Ok(1_999_000));
        assert_eq!(quote(100, 1000, 3000), Ok(300));
        for row in rows::v2_reference() {
            let call = render_call(row.method, &row.args);
            assert_eq!(row.method.oracle(&row.args), row.expected, "{call}");
        }
    }

    /// Uniswap V2's own code for the three methods whose fee the oracle
    /// scales (997 over 1000 in V2, `10000 - fee_bps` over 10000 here), with
    /// its literal constants and its requires, each require returning the
    /// refusal the table names for it.
    mod v2 {
        use super::super::{Refusal, require};

        /// UniswapV2Library.getAmountOut.
        pub fn get_amount_out(amount_in: u128, reserve_in: u128, reserve_out: u128) -> Result<u128, Refusal> {
            require(amount_in > 0, Refusal::ZeroAmountIn)?;
            require(reserve_in > 0 && reserve_out > 0, Refusal::ZeroReserve)?;
            let amount_in_with_fee = amount_in * 997;
            Ok(amount_in_with_fee * reserve_out / (reserve_in * 1000 + amount_in_with_fee))
        }

        /// UniswapV2Library.getAmountIn, where `reserveOut.sub(amountOut)`
        /// reverts below zero and the division reverts at zero.
        pub fn get_amount_in(amount_out: u128, reserve_in: u128, reserve_out: u128) -> Result<u128, Refusal> {
            require(amount_out > 0, Refusal::ZeroAmountOut)?;
            require(reserve_in > 0 && reserve_out > 0, Refusal::ZeroReserve)?;
            let numerator = reserve_in * amount_out * 1000;
            let denominator = reserve_out.checked_sub(amount_out).ok_or(Refusal::AmountOutNotBelowReserve)? * 997;
            let quotient = numerator.checked_div(denominator).ok_or(Refusal::AmountOutNotBelowReserve)?;
            Ok(quotient + 1)
        }

        /// UniswapV2Pair.swap's `amount0Out < _reserve0 && amount1Out <
        /// _reserve1` (the input side's amount out is 0, so its reserve must be
        /// positive) and its K check, uncancelled:
        /// `(balance * 1000 - amountIn * 3) * (Ro - o) * 1000 >= Ri * Ro * 1000^2`.
        /// swap also requires a positive input and output, which the contract's
        /// k checks leave to their caller (they return a bool there), so those
        /// requires are not here.
        pub fn k_holds(
            amount_in: u128,
            amount_out: u128,
            reserve_in: u128,
            reserve_out: u128,
        ) -> Result<bool, Refusal> {
            require(reserve_in > 0, Refusal::ZeroReserve)?;
            require(amount_out < reserve_out, Refusal::AmountOutNotBelowReserve)?;
            let balance_in_adjusted = (reserve_in + amount_in) * 1000 - amount_in * 3;
            let balance_out_adjusted = (reserve_out - amount_out) * 1000;
            Ok(balance_in_adjusted * balance_out_adjusted >= reserve_in * reserve_out * 1000 * 1000)
        }
    }

    /// Runs `method`'s differential inputs through `both`, which gives the
    /// oracle's result at fee 30 and V2's own, and requires the two equal: the
    /// same value, or the same refusal. Equality both ways means the oracle
    /// refuses no more than V2 does, as well as computing V2's values, and
    /// shows the fee scale cancels. The inputs must reach every refusal the
    /// method has but the fee's.
    fn agrees_with_v2(method: Method, mut both: impl FnMut(&[u32]) -> [Result<Value, Refusal>; 2]) {
        let mut values = 0;
        let mut refused = Vec::new();
        vectors::for_each_case(method, |args| {
            let [oracle, v2] = both(args);
            assert_eq!(oracle, v2, "{} at fee 30: the oracle, and V2's own code", render_call(method, args));
            match oracle {
                Ok(_) => values += 1,
                Err(refusal) if !refused.contains(&refusal) => refused.push(refusal),
                Err(_) => {}
            }
        });
        let name = method.name();
        assert!(values > 10_000, "{name}: only {values} inputs returned a value");
        for refusal in method.refusals().iter().filter(|&&refusal| refusal != Refusal::FeeNotBelowDenominator) {
            assert!(refused.contains(refusal), "{name}: no input was refused with {refusal:?}");
        }
    }

    #[test]
    fn get_amount_out_agrees_with_v2() {
        agrees_with_v2(Method::GetAmountOut, |args| {
            let &[a, ri, ro, _] = args else { panic!("get_amount_out takes 4 arguments, got {args:?}") };
            let v2 = v2::get_amount_out(a.into(), ri.into(), ro.into()).and_then(fit);
            [get_amount_out(a, ri, ro, 30).map(Value::U32), v2.map(Value::U32)]
        });
    }

    #[test]
    fn get_amount_in_agrees_with_v2() {
        agrees_with_v2(Method::GetAmountIn, |args| {
            let &[x, ri, ro, _] = args else { panic!("get_amount_in takes 4 arguments, got {args:?}") };
            let v2 = v2::get_amount_in(x.into(), ri.into(), ro.into()).and_then(fit);
            [get_amount_in(x, ri, ro, 30).map(Value::U32), v2.map(Value::U32)]
        });
    }

    #[test]
    fn k_holds_with_fee_agrees_with_v2() {
        agrees_with_v2(Method::KHoldsWithFee, |args| {
            let &[a, o, ri, ro, _] = args else { panic!("k_holds_with_fee takes 5 arguments, got {args:?}") };
            let v2 = v2::k_holds(a.into(), o.into(), ri.into(), ro.into());
            [k_holds_with_fee(a, o, ri, ro, 30).map(Value::Bool), v2.map(Value::Bool)]
        });
    }

    #[test]
    fn rounds_as_documented() {
        assert_eq!(mul_div(7, 3, 2), Ok(10));
        assert_eq!(mul_div_up(7, 3, 2), Ok(11));
        assert_eq!(mul_div_up(6, 3, 2), Ok(9));
        // floor(7 * 1227133513 / 2) = 2^32 - 1 fits; the ceiling does not.
        assert_eq!(mul_div(7, 1_227_133_513, 2), Ok(u32::MAX));
        assert_eq!(mul_div_up(7, 1_227_133_513, 2), Err(Refusal::ResultDoesNotFit));
    }

    #[test]
    fn names_the_refusing_row() {
        assert_eq!(mint_initial(1000, 999), Err(Refusal::RootBelowMinimum));
        assert_eq!(mint_initial(1000, 1000), Err(Refusal::NothingMinted));
        assert_eq!(mint_initial(1001, 1001), Ok(1));
        assert_eq!(mint_proportional(u32::MAX, u32::MAX, 1, 1, u32::MAX), Err(Refusal::NoShareFits));
        assert_eq!(mint_proportional(1, 1, 2, 2, 1), Err(Refusal::NothingMinted));
        assert_eq!(burn_share(2, 1, 1), Err(Refusal::LiquidityAboveSupply));
        assert_eq!(burn_share(1, 1, 2), Err(Refusal::NothingBurned));
        assert_eq!(get_amount_in(1, 1, 1, 0), Err(Refusal::AmountOutNotBelowReserve));
    }
}
