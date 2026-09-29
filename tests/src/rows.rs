//! Hand-picked inputs, each with the result it must give: Uniswap V2's
//! reference values, the boundaries where a result stops fitting or steps,
//! and the trap matrix, one refusal per row of the refusal table in
//! `src/main.inf`. `contract.rs` runs every set on the Soroban contract and
//! `wasm32.rs` on the proof module ([`mismatches`]), and the tests below
//! check the oracle against each row.

use crate::Trap;
use crate::method::{Method, Value, render_call};
use crate::oracle::Refusal;
use crate::vectors::EXACT_QUOTIENT_FEES;

const M: u32 = u32::MAX;

/// An input and the result it must give.
pub struct Row {
    pub method: Method,
    pub args: Vec<u32>,
    pub expected: Result<Value, Refusal>,
}

fn row(method: Method, args: &[u32], expected: Result<Value, Refusal>) -> Row {
    Row { method, args: args.to_vec(), expected }
}

fn returns(method: Method, args: &[u32], value: u32) -> Row {
    row(method, args, Ok(Value::U32(value)))
}

/// A set of rows, built on demand.
pub type RowSet = fn() -> Vec<Row>;

/// Every set, by name.
pub const SETS: [(&str, RowSet); 7] = [
    ("v2_reference", v2_reference),
    ("get_amount_in_at_the_largest_result", get_amount_in_at_the_largest_result),
    ("mul_div_up_where_only_the_ceiling_overflows", mul_div_up_where_only_the_ceiling_overflows),
    ("mint_initial_at_the_threshold_and_at_squares", mint_initial_at_the_threshold_and_at_squares),
    ("burn_share_at_one_unit", burn_share_at_one_unit),
    ("k_checks_at_equality", k_checks_at_equality),
    ("trap_matrix", trap_matrix),
];

/// Every row of every set on which `call` does not give the row's result (its
/// value, or a trap where the row refuses), one line each.
pub fn mismatches(mut call: impl FnMut(Method, &[u32]) -> Result<Value, Trap>) -> Vec<String> {
    let mut found = Vec::new();
    for (set, rows) in SETS {
        for row in rows() {
            let actual = call(row.method, &row.args);
            if actual != row.expected.map_err(|_| Trap) {
                let shown = render_call(row.method, &row.args);
                found.push(format!("{set}: {shown} gives {actual:?}, the row {:?}", row.expected));
            }
        }
    }
    found
}

/// Values Uniswap V2 itself gives at its 0.3% fee (`fee_bps = 30`), and
/// values derived by hand from its formulas, at that fee and at others (with
/// `10000 - fee_bps` over 10000 in place of 997 over 1000), and for mul_div
/// and mul_div_up from v3-core's FullMath, which they mirror. Several are
/// chosen so that a plausible misreading of a formula gives another value
/// (the comments say which). The contract and the oracle could share a
/// misreading; these rows come from V2 and FullMath, not from either.
#[must_use]
pub fn v2_reference() -> Vec<Row> {
    use Method::{
        BurnShare, GetAmountIn, GetAmountOut, KHolds, KHoldsWithFee, MinimumLiquidity, MintInitial,
        MintProportional, MulDiv, MulDivUp, Quote,
    };
    let yes = Ok(Value::Bool(true));
    vec![
        // The values the spec's v2_reference_vectors claims.
        returns(MinimumLiquidity, &[], 1000),
        returns(GetAmountOut, &[1000, 100_000, 100_000, 30], 987),
        returns(GetAmountIn, &[987, 100_000, 100_000, 30], 1000),
        returns(GetAmountOut, &[1, 1_000_000, 1000, 30], 0),
        returns(MintInitial, &[1_000_000, 4_000_000], 1_999_000),
        returns(Quote, &[100, 1000, 3000], 300),
        returns(MulDiv, &[M, M, M], M),
        returns(MulDivUp, &[M, M, M], M),
        // v2-periphery's router tests (test/UniswapV2Router02.spec.ts, describe
        // 'UniswapV2Router02', run against UniswapV2Router02):
        // getAmountOut(2, 100, 100) = 1, getAmountIn(1, 100, 100) = 2,
        // quote(1, 100, 200) = 2 and quote(2, 200, 100) = 1.
        returns(GetAmountOut, &[2, 100, 100, 30], 1),
        returns(GetAmountIn, &[1, 100, 100, 30], 2),
        returns(Quote, &[1, 100, 200], 2),
        returns(Quote, &[2, 200, 100], 1),
        // 997e6 / 1997 = 499248.8: the denominator is Ri*1000 + a*997.
        // (Ri + a)*1000 in its place gives 498500, a fee taken from the
        // floor of a*997/1000 gives 0.
        returns(GetAmountOut, &[1, 1, 1_000_000, 30], 499_248),
        // An input above the reserve is ordinary V2 input:
        // 2*997*1000 / (1000 + 2*997) = 665.99 and
        // 1000*997*1000 / (1000 + 1000*997) = 998.99.
        returns(GetAmountOut, &[2, 1, 1000, 30], 665),
        returns(GetAmountOut, &[1000, 1, 1000, 30], 998),
        // The fee in basis points, G = 10000 - fee_bps in place of 997.
        // 3*9999*4 / (10000 + 3*9999) = 2.9999, where fee 0 gives 3 exactly:
        // even 1 bps costs a unit here.
        returns(GetAmountOut, &[3, 1, 4, 1], 2),
        // 1e4*9975*1e4 / (1e8 + 1e4*9975) = 4993.7; a fee rounded down to
        // whole per mille, (1000 - fee_bps / 10) over 1000 (G = 9980), gives
        // 4994, and fee_bps read as per mille (G = 9750) gives 4936.
        returns(GetAmountOut, &[10_000, 10_000, 10_000, 25], 4993),
        // G = 1: 1e6*1e9 / (10000 + 1e6) = 990099009.9; rounded down to 999
        // per mille (G = 10) gives 999000999.
        returns(GetAmountOut, &[1_000_000, 1, 1_000_000_000, 9999], 990_099_009),
        // At fee 0 one unit buys one unit of (1, 2), exactly, and so
        // get_amount_in's +1 overpays by one: 1*1 / (1*1) + 1 = 2.
        returns(GetAmountOut, &[1, 1, 2, 0], 1),
        returns(GetAmountIn, &[1, 1, 2, 0], 2),
        // quote rounds down, and may return 0: V2 has no `result > 0` check.
        // Its amount may exceed the reserve: 3*5 / 2 = 7.5.
        returns(Quote, &[2, 3, 5], 3),
        returns(Quote, &[1, 3, 2], 0),
        returns(Quote, &[3, 2, 5], 7),
        // The quoted swap passes both k checks, and one unit more fails the
        // fee check.
        row(KHolds, &[1000, 987, 100_000, 100_000], yes),
        row(KHoldsWithFee, &[1000, 987, 100_000, 100_000, 30], yes),
        row(KHoldsWithFee, &[1000, 988, 100_000, 100_000, 30], Ok(Value::Bool(false))),
        // The whole supply burns the whole balance.
        returns(BurnShare, &[2_000_000, 4_000_000, 2_000_000], 4_000_000),
        // mint's Math.min(amount0 * S / R0, amount1 * S / R1): equal shares,
        // the smaller share on either side, and one share too large for a
        // u32 (V2 computes it in uint256 and takes the other).
        returns(MintProportional, &[1000, 4000, 1_000_000, 4_000_000, 2_000_000], 2000),
        returns(MintProportional, &[2000, 4000, 1_000_000, 4_000_000, 2_000_000], 2000),
        returns(MintProportional, &[1000, 8000, 1_000_000, 4_000_000, 2_000_000], 2000),
        returns(MintProportional, &[M, 1, 1, 1, M], M),
        returns(MintProportional, &[1, M, 1, 1, M], M),
        returns(MulDiv, &[1000, 2000, 3], 666_666),
        returns(MulDivUp, &[1000, 2000, 3], 666_667),
        // v3-core's FullMath: mulDiv of a zero product is 0, and
        // mulDivRoundingUp adds one only where mulmod(a, b, d) > 0, so a zero
        // product is 0 both ways. The ceiling idiom (a*b - 1) / d + 1 traps
        // at a*b = 0 instead.
        returns(MulDiv, &[0, M, 7], 0),
        returns(MulDivUp, &[0, M, 7], 0),
        returns(MulDivUp, &[M, 0, 1], 0),
        // A product below d, but not zero, floors to 0 and rounds up to 1:
        // mulmod(1, 1, 2) = 1 > 0.
        returns(MulDiv, &[1, 1, 2], 0),
        returns(MulDivUp, &[1, 1, 2], 1),
    ]
}

/// get_amount_in's result is its quotient plus one, so the largest result,
/// `2^32 - 1`, comes from the quotient `2^32 - 2`; at `2^32 - 1` the checked
/// +1 overflows, and from `2^32` the quotient itself does not fit.
#[must_use]
pub fn get_amount_in_at_the_largest_result() -> Vec<Row> {
    let mut rows = Vec::new();
    for f in EXACT_QUOTIENT_FEES {
        // Ro = x * (1 + D / G) makes the quotient exactly Ri.
        let ratio = 1 + 10_000 / (10_000 - f);
        for x in [1, 1000, M / ratio] {
            let args = |ri| [x, ri, x * ratio, f];
            rows.push(returns(Method::GetAmountIn, &args(M - 2), M - 1));
            rows.push(returns(Method::GetAmountIn, &args(M - 1), M));
            rows.push(row(Method::GetAmountIn, &args(M), Err(Refusal::ResultDoesNotFit)));
        }
    }
    // At x = 2, Ro = 3 and fee 0 the quotient is 2 * Ri.
    rows.push(returns(Method::GetAmountIn, &[2, (1 << 31) - 1, 3, 0], M));
    rows.push(row(Method::GetAmountIn, &[2, 1 << 31, 3, 0], Err(Refusal::ResultDoesNotFit)));
    rows
}

/// mul_div_up's floor fits up to `a * b = d * 2^32 - 1`, but its ceiling only
/// up to `d * (2^32 - 1)`: in between, mul_div returns and mul_div_up traps.
///
/// # Panics
///
/// When a hand-picked product is not in that gap.
#[must_use]
pub fn mul_div_up_where_only_the_ceiling_overflows() -> Vec<Row> {
    let too_large = Err(Refusal::ResultDoesNotFit);
    let both = |value| (Ok(Value::U32(value)), Ok(Value::U32(value)));
    let only_floor = (Ok(Value::U32(M)), too_large);
    let neither = (too_large, too_large);
    let mut pairs = vec![
        // 7 * 1227133513 = 2 * (2^32 - 1) + 1.
        ([7, 1_227_133_513, 2], only_floor),
        ([7, 1_227_133_512, 2], both(M - 3)),
    ];
    for d in [1, 2, 3, 10_000, 65_537, M - 1, M] {
        pairs.push(([d, M, d], both(M)));
        if let Some(twice) = d.checked_mul(2) {
            pairs.push(([twice, (1 << 31) - 1, d], both(M - 1)));
            pairs.push(([twice, 1 << 31, d], neither));
        }
    }
    // More products d * (2^32 - 1) + 1: the floor is 2^32 - 1, the remainder 1.
    for (a, b, d) in [(101, 127_573_286, 3), (479, 44_832_644, 5), (11, 2_733_161_006, 7), (439, 978_352_459, 100)] {
        let product = u64::from(a) * u64::from(b);
        let (floor, remainder) = (product / u64::from(d), product % u64::from(d));
        assert!(floor == u64::from(M) && remainder > 0, "{a} * {b} / {d} is not in the gap");
        pairs.push(([a, b, d], only_floor));
    }
    pairs
        .into_iter()
        .flat_map(|(args, (floor, ceiling))| [row(Method::MulDiv, &args, floor), row(Method::MulDivUp, &args, ceiling)])
        .collect()
}

/// mint_initial mints from `a0 * a1 = 1001^2` on (a root of 1001 less the
/// 1000 locked units), and its result steps exactly at perfect squares.
#[must_use]
pub fn mint_initial_at_the_threshold_and_at_squares() -> Vec<Row> {
    let mint = |args: [u32; 2], expected: Result<u32, Refusal>| {
        row(Method::MintInitial, &args, expected.map(Value::U32))
    };
    let mut rows = vec![
        mint([1001, 1001], Ok(1)),
        mint([7, 143_143], Ok(1)),
        mint([1, 1_002_001], Ok(1)),
        mint([1, 1_002_000], Err(Refusal::NothingMinted)),
        mint([1, 1_002_002], Ok(1)),
        mint([1000, 1000], Err(Refusal::NothingMinted)),
        mint([999, 1000], Err(Refusal::RootBelowMinimum)),
        mint([0, M], Err(Refusal::RootBelowMinimum)),
        mint([1, 1_004_003], Ok(1)),
        mint([1, 1_004_004], Ok(2)),
    ];
    for s in [1002, 4096, 65_535, 65_536, 65_537, 1 << 31, M - 1] {
        rows.push(mint([s, s], Ok(s - 1000)));
        // (s - 1) * (s + 1) = s^2 - 1, whose root is s - 1.
        rows.push(mint([s - 1, s + 1], Ok(s - 1001)));
    }
    rows.push(mint([M, M], Ok(M - 1000)));
    rows
}

/// burn_share returns one unit from `liquidity * balance = total_supply` on,
/// and traps one below it.
#[must_use]
pub fn burn_share_at_one_unit() -> Vec<Row> {
    let burn = |args: [u32; 3], expected: Result<u32, Refusal>| row(Method::BurnShare, &args, expected.map(Value::U32));
    vec![
        burn([1000, 1000, 1_000_000], Ok(1)),
        burn([1000, 1000, 1_000_001], Err(Refusal::NothingBurned)),
        burn([1, M, M], Ok(1)),
        burn([1, M - 1, M], Err(Refusal::NothingBurned)),
        burn([M, M, M], Ok(M)),
        burn([M - 1, M, M], Ok(M - 1)),
    ]
}

/// Both k checks hold at equality, and one unit of reserve_in more breaks it.
#[must_use]
pub fn k_checks_at_equality() -> Vec<Row> {
    let (holds, fails) = (Ok(Value::Bool(true)), Ok(Value::Bool(false)));
    let mut rows = Vec::new();
    // a = o and Ro = o + Ri: (Ri + a) * (Ro - o) = Ri * Ro exactly.
    for (o, ri) in [(1, 1), (2, 3), (1000, 99_000), (65_536, 65_536), (1 << 20, M - (1 << 20))] {
        let ro = o + ri;
        rows.push(row(Method::KHolds, &[o, o, ri, ro], holds));
        rows.push(row(Method::KHolds, &[o, o, ri + 1, ro], fails));
    }
    // a = o * D * c and Ri = c * (Ro - o) * G: the fee-adjusted sides are equal.
    for (o, c, span, f) in [(1, 1, 1, 30), (7, 3, 11, 0), (400, 10, 40, 9999), (13, 2, 5, 25)] {
        let (a, ri) = (o * 10_000 * c, c * span * (10_000 - f));
        rows.push(row(Method::KHoldsWithFee, &[a, o, ri, o + span, f], holds));
        rows.push(row(Method::KHoldsWithFee, &[a, o, ri + 1, o + span, f], fails));
    }
    rows
}

/// One row per row of the refusal table in `src/main.inf`
/// ([`Method::refusals`]), and more where a row has more than one way to
/// fail. Each refusal is the same trap to the contract's caller; the oracle
/// names which row refused. The k checks refuse a swap by returning false,
/// not by trapping, and accept a zero output. A fee is refused at the
/// denominator and above it, where the other arguments alone would be
/// accepted (k_holds_with_fee has no `10000 - fee_bps` that would trap on
/// its own).
#[must_use]
pub fn trap_matrix() -> Vec<Row> {
    use Method::{
        BurnShare, GetAmountIn, GetAmountOut, KHolds, KHoldsWithFee, MintInitial, MintProportional, MulDiv,
        MulDivUp, Quote,
    };
    use Refusal::{
        AmountOutNotBelowReserve, FeeNotBelowDenominator, LiquidityAboveSupply, NoShareFits, NothingBurned,
        NothingMinted, ResultDoesNotFit, RootBelowMinimum, ZeroAmountA, ZeroAmountIn, ZeroAmountOut,
        ZeroDenominator, ZeroReserve, ZeroSupply,
    };
    let refused = |method, args: &[u32], refusal| row(method, args, Err(refusal));
    let (holds, fails) = (Ok(Value::Bool(true)), Ok(Value::Bool(false)));
    vec![
        refused(GetAmountOut, &[0, 100_000, 100_000, 30], ZeroAmountIn),
        refused(GetAmountOut, &[1000, 0, 100_000, 30], ZeroReserve),
        refused(GetAmountOut, &[1000, 100_000, 0, 30], ZeroReserve),
        refused(GetAmountOut, &[1000, 100_000, 100_000, 10_000], FeeNotBelowDenominator),
        refused(GetAmountOut, &[1000, 100_000, 100_000, 10_001], FeeNotBelowDenominator),
        refused(GetAmountOut, &[1000, 100_000, 100_000, M], FeeNotBelowDenominator),
        refused(GetAmountIn, &[0, 100_000, 100_000, 30], ZeroAmountOut),
        refused(GetAmountIn, &[987, 0, 100_000, 30], ZeroReserve),
        refused(GetAmountIn, &[987, 100_000, 0, 30], ZeroReserve),
        refused(GetAmountIn, &[100_000, 100_000, 100_000, 30], AmountOutNotBelowReserve),
        refused(GetAmountIn, &[100_001, 100_000, 100_000, 30], AmountOutNotBelowReserve),
        refused(GetAmountIn, &[987, 100_000, 100_000, 10_000], FeeNotBelowDenominator),
        refused(GetAmountIn, &[987, 100_000, 100_000, 10_001], FeeNotBelowDenominator),
        refused(GetAmountIn, &[987, 100_000, 100_000, M], FeeNotBelowDenominator),
        refused(GetAmountIn, &[1, M, 2, 0], ResultDoesNotFit),
        refused(GetAmountIn, &[2, 1 << 31, 3, 0], ResultDoesNotFit),
        refused(Quote, &[0, 1000, 3000], ZeroAmountA),
        refused(Quote, &[100, 0, 3000], ZeroReserve),
        refused(Quote, &[100, 1000, 0], ZeroReserve),
        refused(Quote, &[M, 1, 2], ResultDoesNotFit),
        refused(MintInitial, &[999, 1000], RootBelowMinimum),
        refused(MintInitial, &[1000, 1000], NothingMinted),
        refused(MintProportional, &[1000, 4000, 0, 4_000_000, 2_000_000], ZeroReserve),
        refused(MintProportional, &[1000, 4000, 1_000_000, 0, 2_000_000], ZeroReserve),
        refused(MintProportional, &[1000, 4000, 1_000_000, 4_000_000, 0], ZeroSupply),
        refused(MintProportional, &[M, M, 1, 1, M], NoShareFits),
        refused(MintProportional, &[1, 1, 1_000_000, 1_000_000, 1000], NothingMinted),
        refused(BurnShare, &[1000, 1_000_000, 0], ZeroSupply),
        refused(BurnShare, &[2_000_001, 1_000_000, 2_000_000], LiquidityAboveSupply),
        refused(BurnShare, &[1, 1, 2_000_000], NothingBurned),
        refused(KHolds, &[1000, 987, 0, 100_000], ZeroReserve),
        refused(KHolds, &[1000, 100_000, 100_000, 100_000], AmountOutNotBelowReserve),
        refused(KHolds, &[1000, 100_001, 100_000, 100_000], AmountOutNotBelowReserve),
        row(KHolds, &[0, 1, 100_000, 100_000], fails),
        row(KHolds, &[0, 0, 100_000, 100_000], holds),
        refused(KHoldsWithFee, &[1000, 987, 100_000, 100_000, 10_000], FeeNotBelowDenominator),
        refused(KHoldsWithFee, &[1000, 987, 100_000, 100_000, M], FeeNotBelowDenominator),
        refused(KHoldsWithFee, &[0, 0, 100_000, 100_000, 10_001], FeeNotBelowDenominator),
        refused(KHoldsWithFee, &[0, 0, 100_000, 100_000, 20_000], FeeNotBelowDenominator),
        refused(KHoldsWithFee, &[0, 0, 100_000, 100_000, M], FeeNotBelowDenominator),
        refused(KHoldsWithFee, &[1000, 987, 0, 100_000, 30], ZeroReserve),
        refused(KHoldsWithFee, &[1000, 100_000, 100_000, 100_000, 30], AmountOutNotBelowReserve),
        row(KHoldsWithFee, &[1000, 988, 100_000, 100_000, 30], fails),
        row(KHoldsWithFee, &[0, 1, 100_000, 100_000, 30], fails),
        row(KHoldsWithFee, &[0, 0, 100_000, 100_000, 30], holds),
        refused(MulDiv, &[1000, 2000, 0], ZeroDenominator),
        refused(MulDiv, &[M, M, 1], ResultDoesNotFit),
        refused(MulDivUp, &[1000, 2000, 0], ZeroDenominator),
        refused(MulDivUp, &[M, M, 1], ResultDoesNotFit),
        refused(MulDivUp, &[7, 1_227_133_513, 2], ResultDoesNotFit),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every row gives the oracle's result.
    #[test]
    fn the_oracle_gives_every_row() {
        for (set, rows) in SETS {
            for row in rows() {
                let call = crate::method::render_call(row.method, &row.args);
                assert_eq!(row.method.oracle(&row.args), row.expected, "{set}: {call}");
            }
        }
    }

    /// The trap matrix has a row for each (method, refusal) row of the refusal
    /// table, and no refusal that is not the method's own; each k check also
    /// has a row where it returns false.
    #[test]
    fn the_trap_matrix_covers_the_refusal_table() {
        let matrix = trap_matrix();
        for method in Method::ALL {
            let refused: Vec<Refusal> =
                matrix.iter().filter(|row| row.method == method).filter_map(|row| row.expected.err()).collect();
            for refusal in method.refusals() {
                assert!(refused.contains(refusal), "no row refuses {} with {refusal:?}", method.name());
            }
            for refusal in &refused {
                assert!(method.refusals().contains(refusal), "{} is not refused with {refusal:?}", method.name());
            }
            if method.returns_bool() {
                let fails = matrix.iter().any(|row| row.method == method && row.expected == Ok(Value::Bool(false)));
                assert!(fails, "no row of {} returns false", method.name());
            }
        }
        for refusal in Refusal::ALL {
            assert!(Method::ALL.iter().any(|method| method.refusals().contains(&refusal)), "{refusal:?} is no row");
        }
    }
}
