//! The inputs every differential test runs a method on, so that the Soroban
//! contract, the proof module and the oracle all see the same cases.
//!
//! For each method, in a fixed order: every combination of [`EDGES`] (with
//! [`FEES`] in the `fee_bps` slot), then seeded random inputs, then directed
//! inputs placed on and around each boundary where the result stops fitting,
//! rounds differently or flips, which random sampling rarely lands on.

use crate::method::{Method, Value};
use crate::oracle::Refusal;
use crate::rng::Rng;

const M: u32 = u32::MAX;
const D: u64 = 10_000;

/// Amounts and reserves: zero and one, the minimum-liquidity threshold, the fee
/// denominator, the 16-bit and 31-bit boundaries, and the top of `u32`.
pub const EDGES: [u32; 14] = [
    0,
    1,
    2,
    999,
    1000,
    1001,
    9999,
    10_000,
    65_535,
    65_536,
    (1 << 31) - 1,
    1 << 31,
    M - 1,
    M,
];

/// `fee_bps`: no fee, the smallest, a common one, V2's, the largest accepted and
/// the first refused.
pub const FEES: [u32; 6] = [0, 1, 25, 30, 9999, 10_000];

/// Random inputs per method.
pub const RANDOM_CASES: usize = 20_000;

/// Rounds of the directed generators per method; each round draws fresh anchors.
pub const DIRECTED_ROUNDS: usize = 300;

/// Calls `visit` with every input the differential tests use for `method`.
pub fn for_each_case(method: Method, mut visit: impl FnMut(&[u32])) {
    for_each_sourced_case(method, |_, args| visit(args));
}

/// Where an input comes from: the edge grid, the seeded random draws, or a
/// directed generator. `tests/specs.rs` draws from the same three.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Edges,
    Random,
    Directed,
}

impl Source {
    pub const ALL: [Source; 3] = [Source::Edges, Source::Random, Source::Directed];
}

/// [`for_each_case`], with the source of each input.
fn for_each_sourced_case(method: Method, mut visit: impl FnMut(Source, &[u32])) {
    for_each_edge_case(method, |args| visit(Source::Edges, args));
    let mut rng = Rng::new(seed(method));
    for _ in 0..RANDOM_CASES {
        visit(Source::Random, &random_case(method, &mut rng));
    }
    for _ in 0..DIRECTED_ROUNDS {
        for case in directed_cases(method, &mut rng) {
            visit(Source::Directed, &case);
        }
    }
}

/// Each method draws from its own stream, so adding cases to one method leaves
/// the others' inputs unchanged.
fn seed(method: Method) -> u64 {
    let index = Method::ALL.iter().position(|&m| m == method).expect("a listed method");
    0x5eed_a3a0_0000_0000 + index as u64
}

/// Every combination of edge values, one per parameter: [`FEES`] in the
/// `fee_bps` slot, [`EDGES`] in the others.
pub fn for_each_edge_case(method: Method, visit: impl FnMut(&[u32])) {
    let fees: Vec<usize> = method.fee_param().into_iter().collect();
    for_each_edge_input(method.arity(), &fees, visit);
}

/// Every combination of `arity` edge values, [`FEES`] at the positions in
/// `fees` and [`EDGES`] elsewhere, the last position varying fastest.
pub fn for_each_edge_input(arity: usize, fees: &[usize], mut visit: impl FnMut(&[u32])) {
    let domain = |i: usize| if fees.contains(&i) { &FEES[..] } else { &EDGES[..] };
    let mut digits = vec![0; arity];
    let mut args: Vec<u32> = (0..arity).map(|i| domain(i)[0]).collect();
    loop {
        visit(&args);
        let mut i = arity;
        loop {
            if i == 0 {
                return;
            }
            i -= 1;
            digits[i] += 1;
            if digits[i] < domain(i).len() {
                args[i] = domain(i)[digits[i]];
                break;
            }
            digits[i] = 0;
            args[i] = domain(i)[0];
        }
    }
}

#[must_use]
pub fn random_case(method: Method, rng: &mut Rng) -> Vec<u32> {
    (0..method.arity())
        .map(|i| if method.fee_param() == Some(i) { random_fee(rng) } else { rng.mixed_u32(&EDGES) })
        .collect()
}

/// A fee: one of [`FEES`] half the time, any value up to the first one
/// refused three times in eight, and a value above that, up to `u32::MAX`,
/// once in eight.
pub fn random_fee(rng: &mut Rng) -> u32 {
    match rng.below(8) {
        0 => rng.mixed_u32(&EDGES).max(10_001),
        1..=4 => rng.pick(&FEES),
        _ => rng.range_u32(0, 10_000),
    }
}

/// A fee the methods accept.
pub fn accepted_fee(rng: &mut Rng) -> u32 {
    random_fee(rng) % 10_000
}

/// A positive amount or reserve, of any scale.
pub fn positive(rng: &mut Rng) -> u32 {
    rng.mixed_u32(&EDGES).max(1)
}

/// `value` clamped to `0..=u32::MAX`.
///
/// # Panics
///
/// Never: a value clamped to that range fits a `u32`.
#[must_use]
pub fn clamp(value: i128) -> u32 {
    u32::try_from(value.clamp(0, i128::from(M))).expect("clamped")
}

/// `value` and its neighbours up to two away, clamped to `u32`.
#[must_use]
pub fn near(value: i128) -> [u32; 5] {
    [-2, -1, 0, 1, 2].map(|delta| clamp(value + delta))
}

/// A factorisation `a * b = target` with `a <= 200` and both factors in `u32`.
#[must_use]
pub fn factor(target: u128) -> Option<(u32, u32)> {
    (1..=200_u32).find_map(|a| {
        let b = u32::try_from(target / u128::from(a)).ok()?;
        target.is_multiple_of(u128::from(a)).then_some((a, b))
    })
}

/// Directed inputs for `method`, drawn around fresh random anchors.
pub fn directed_cases(method: Method, rng: &mut Rng) -> Vec<Vec<u32>> {
    match method {
        Method::MinimumLiquidity => Vec::new(),
        Method::GetAmountOut => get_amount_out_edges(rng),
        Method::GetAmountIn => get_amount_in_quotient_edges(rng),
        Method::Quote => quote_fit_edges(rng),
        Method::MintInitial => mint_initial_root_edges(rng),
        Method::MintProportional => mint_proportional_share_edges(rng),
        Method::BurnShare => burn_share_unit_edges(rng),
        Method::KHolds => k_equality_edges(rng).into_iter().map(without_fee).collect(),
        Method::KHoldsWithFee => k_equality_edges(rng),
        Method::MulDiv | Method::MulDivUp => product_fit_edges(rng),
    }
}

/// A k_holds_with_fee input as a k_holds input.
fn without_fee(mut case: Vec<u32>) -> Vec<u32> {
    case.pop();
    case
}

/// The output's approach to the reserve: a huge input against a tiny reserve
/// buys `reserve_out - 1`, and `(x, x, 2x, 0)` divides exactly.
fn get_amount_out_edges(rng: &mut Rng) -> Vec<Vec<u32>> {
    let ro = positive(rng);
    let x = rng.range_u32(1, M / 2);
    vec![
        vec![M, 1, ro, 0],
        vec![M, 1, ro, accepted_fee(rng)],
        vec![M, 1, ro, 9999],
        vec![x, x, 2 * x, 0],
        vec![positive(rng), positive(rng), positive(rng), accepted_fee(rng)],
    ]
}

/// Fees at which `(Ro - x) * (D - f) = x * D` for `Ro = x * (1 + D / (D - f))`,
/// so that get_amount_in's quotient `Ri * x * D / ((Ro - x) * (D - f))` is
/// exactly `Ri`.
pub const EXACT_QUOTIENT_FEES: [u32; 7] = [0, 5000, 7500, 8000, 9000, 9900, 9999];

/// get_amount_in with its quotient at the largest that fits after the +1
/// (`2^32 - 2`), at `2^32 - 1` (where the +1 overflows), and at `2^32` (where
/// the quotient itself does not fit), exactly and in random neighbourhoods.
fn get_amount_in_quotient_edges(rng: &mut Rng) -> Vec<Vec<u32>> {
    let mut cases = Vec::new();
    let f = rng.pick(&EXACT_QUOTIENT_FEES);
    let ratio = 1 + D / (D - u64::from(f));
    let x = rng.range_u32(1, u32::try_from(u64::from(M) / ratio).expect("fits"));
    let ro = u32::try_from(u64::from(x) * ratio).expect("x is bounded by the ratio");
    for ri in [M - 3, M - 2, M - 1, M] {
        cases.push(vec![x, ri, ro, f]);
    }
    // At x = 2, Ro = 3 and fee 0 the quotient is 2 * Ri, so Ri = 2^31 makes
    // it 2^32, the first quotient that does not fit.
    for ri in near(1 << 31) {
        cases.push(vec![2, ri, 3, 0]);
    }
    // A random numerator scaled so that the quotient lands near each target.
    let ro = clamp(i128::from(positive(rng)) + 1).max(2);
    let x = rng.range_u32(1, ro - 1);
    let f = accepted_fee(rng);
    let q = u128::from(ro - x) * u128::from(D - u64::from(f));
    for target in [u128::from(M) - 1, u128::from(M), u128::from(M) + 1] {
        let ri = target * q / (u128::from(x) * u128::from(D));
        for ri in near(i128::try_from(ri).expect("below 2^100")) {
            cases.push(vec![x, ri, ro, f]);
        }
    }
    cases
}

/// quote with `a * Rb` around `Ra * 2^32`, where the result stops fitting.
fn quote_fit_edges(rng: &mut Rng) -> Vec<Vec<u32>> {
    let mut cases = Vec::new();
    let ra = positive(rng);
    let a = positive(rng);
    let limit = u64::from(ra) << 32;
    for rb in near(i128::from((limit - 1) / u64::from(a))) {
        cases.push(vec![a, ra, rb]);
    }
    for target in [limit - 1, limit] {
        if let Some((a, rb)) = factor(u128::from(target)) {
            cases.push(vec![a, ra, rb]);
        }
    }
    cases
}

/// mint_initial with `a0 * a1` around `1001^2` (the first product that mints)
/// and around perfect squares, where the root steps.
fn mint_initial_root_edges(rng: &mut Rng) -> Vec<Vec<u32>> {
    let mut cases = vec![
        vec![1001, 1001],
        vec![1000, 1002],
        vec![1, 1_002_000],
        vec![1, 1_002_001],
        vec![1, 1_002_002],
        vec![1000, 1000],
        vec![65_536, 65_536],
        vec![M, M],
    ];
    let root = if rng.below(4) == 0 { 1001 } else { i128::from(positive(rng)) };
    let a0 = positive(rng);
    let square = root * root;
    for target in [square - 1, square, square + 2 * root, square + 2 * root + 1] {
        for a1 in near((target + i128::from(a0) - 1) / i128::from(a0)) {
            cases.push(vec![a0, a1]);
        }
    }
    cases
}

/// mint_proportional with each share at its fit boundary (`2^32 - 1` and
/// `2^32`), in both orders, and with the two shares tied.
fn mint_proportional_share_edges(rng: &mut Rng) -> Vec<Vec<u32>> {
    let mut cases = Vec::new();
    let (r0, r1, s) = (positive(rng), positive(rng), positive(rng));
    let a0 = u64::from(r0) * u64::from(M) / u64::from(s);
    let a1 = (u64::from(r1) << 32) / u64::from(s);
    for x0 in near(i128::from(a0)) {
        for x1 in near(i128::from(a1)) {
            cases.push(vec![x0, x1, r0, r1, s]);
            cases.push(vec![x1, x0, r1, r0, s]);
        }
    }
    let a = positive(rng);
    cases.push(vec![a, a, r0, r0, s]);
    cases.push(vec![a, a.saturating_add(1), r0, r0, s]);
    // The smallest share at zero and one: a0 * S around R0.
    for x0 in near(i128::from(u64::from(r0).div_ceil(u64::from(s)))) {
        cases.push(vec![x0, M, r0, 1, s]);
    }
    cases
}

/// burn_share with `liquidity * balance` around `total_supply`, where the
/// result steps from zero to one.
fn burn_share_unit_edges(rng: &mut Rng) -> Vec<Vec<u32>> {
    let s = positive(rng);
    let l = rng.range_u32(1, s);
    let b = u64::from(s).div_ceil(u64::from(l));
    let mut cases: Vec<Vec<u32>> = near(i128::from(b)).iter().map(|&b| vec![l, b, s]).collect();
    cases.extend([vec![s, M, s], vec![s, 1, s], vec![s.saturating_add(1), 1, s], vec![M, M, M]]);
    cases
}

/// The k checks at their equality, `a * (Ro - o) * G = Ri * o * D`, and one
/// unit of reserve either side. The last argument is `fee_bps`.
fn k_equality_edges(rng: &mut Rng) -> Vec<Vec<u32>> {
    let mut cases = Vec::new();
    // At fee 0: a = o and Ro = o + Ri.
    let o = rng.range_u32(1, 1 << 20);
    let ri = rng.range_u32(2, 1 << 20);
    for r in [ri - 1, ri, ri + 1] {
        cases.push(vec![o, o, r, o + ri, 0]);
    }
    // At a fee: a = o * D * c and Ri = c * (Ro - o) * G.
    let (o, c, span) = (rng.range_u32(1, 400), rng.range_u32(1, 10), rng.range_u32(1, 40));
    let f = accepted_fee(rng);
    let a = o * 10_000 * c;
    let ri = c * span * (10_000 - f);
    for r in [ri - 1, ri, ri + 1] {
        cases.push(vec![a, o, r, o + span, f]);
    }
    // Around the output get_amount_out quotes.
    let (a, ri, ro, f) = (positive(rng), positive(rng), positive(rng), accepted_fee(rng));
    let g = u128::from(D - u64::from(f));
    let quoted = u128::from(a) * g * u128::from(ro) / (u128::from(ri) * u128::from(D) + u128::from(a) * g);
    for o in near(i128::try_from(quoted).expect("below 2^32")) {
        cases.push(vec![a, o, ri, ro, f]);
    }
    cases
}

/// mul_div and mul_div_up with `a * b` around `d * (2^32 - 1)` and `d * 2^32`:
/// the floor stops fitting at `d * 2^32`, and the ceiling one step earlier,
/// just above `d * (2^32 - 1)`.
fn product_fit_edges(rng: &mut Rng) -> Vec<Vec<u32>> {
    let mut cases = vec![vec![7, 1_227_133_513, 2], vec![7, 1_227_133_512, 2], vec![7, 1_227_133_514, 2]];
    let d = positive(rng);
    let a = positive(rng);
    let (d64, m64) = (u64::from(d), u64::from(M));
    let targets = [d64 * m64 - 1, d64 * m64, d64 * m64 + 1, d64 * m64 + d64 - 1, (d64 << 32) - 1, d64 << 32];
    for target in targets {
        for b in near(i128::from(target / u64::from(a))) {
            cases.push(vec![a, b, d]);
        }
        if let Some((a, b)) = factor(u128::from(target)) {
            cases.push(vec![a, b, d]);
        }
    }
    if let Some(twice) = d.checked_mul(2) {
        cases.push(vec![twice, 1 << 31, d]);
        cases.push(vec![twice, (1 << 31) - 1, d]);
    }
    cases
}

/// What the oracle did over a set of cases, so that a differential test can
/// show it compared values, both booleans, and every refusal of the method's
/// rows of the refusal table, not one kind of outcome only.
#[derive(Default)]
pub struct Coverage {
    values: usize,
    trues: usize,
    falses: usize,
    refusals: Vec<Refusal>,
}

impl Coverage {
    pub fn record(&mut self, expected: Result<Value, Refusal>) {
        match expected {
            Ok(value) => {
                self.values += 1;
                self.trues += usize::from(value == Value::Bool(true));
                self.falses += usize::from(value == Value::Bool(false));
            }
            Err(refusal) => {
                if !self.refusals.contains(&refusal) {
                    self.refusals.push(refusal);
                }
            }
        }
    }

    /// # Panics
    ///
    /// When the cases missed a kind of outcome `method` has, or the oracle
    /// refused one with a row that is not the method's.
    pub fn assert_complete(&self, method: Method) {
        let name = method.name();
        assert!(self.values > 0, "{name}: no case returned a value");
        if method.returns_bool() {
            assert!(self.trues > 0 && self.falses > 0, "{name}: the cases never returned both true and false");
        }
        for refusal in method.refusals() {
            assert!(self.refusals.contains(refusal), "{name}: no case was refused with {refusal:?}");
        }
        for refusal in &self.refusals {
            assert!(method.refusals().contains(refusal), "{name}: refused with {refusal:?}, not a row of its own");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The edge grid keeps the values on each side of the guards and of the
    /// fit boundaries the methods have: without them the grid would still
    /// cover its own cross product, and only the rows and the directed inputs
    /// would reach those points.
    #[test]
    fn the_edge_grid_keeps_the_boundary_values() {
        let edges = [
            (0, "zero, refused as an amount or a reserve"),
            (1, "the smallest amount or reserve accepted"),
            (2, "one past it"),
            (999, "one below the minimum liquidity"),
            (1000, "the minimum liquidity"),
            (1001, "one above it, the smallest root that mints"),
            (9999, "one below the fee denominator"),
            (10_000, "the fee denominator"),
            (65_535, "the largest 16-bit value"),
            (65_536, "2^16"),
            ((1 << 31) - 1, "the largest 31-bit value"),
            (1 << 31, "2^31"),
            (M - 1, "one below the top of u32"),
            (M, "the top of u32"),
        ];
        for (edge, what) in edges {
            assert!(EDGES.contains(&edge), "EDGES lost {edge}, {what}");
        }
        let fees = [
            (0, "no fee"),
            (1, "the smallest fee"),
            (25, "a common fee"),
            (30, "V2's fee"),
            (9999, "the largest fee accepted"),
            (10_000, "the first fee refused"),
        ];
        for (fee, what) in fees {
            assert!(FEES.contains(&fee), "FEES lost {fee}, {what}");
        }
    }

    #[test]
    fn edge_cases_cover_the_cross_product() {
        let mut count = 0;
        for_each_edge_case(Method::GetAmountOut, |_| count += 1);
        assert_eq!(count, EDGES.len().pow(3) * FEES.len());
        let mut calls = Vec::new();
        for_each_edge_case(Method::MinimumLiquidity, |args| calls.push(args.to_vec()));
        assert_eq!(calls, vec![Vec::<u32>::new()]);
    }

    #[test]
    fn every_case_has_one_value_per_parameter() {
        for method in Method::ALL {
            let mut rng = Rng::new(1);
            for case in directed_cases(method, &mut rng) {
                assert_eq!(case.len(), method.arity(), "{}: {case:?}", method.name());
            }
            assert_eq!(random_case(method, &mut rng).len(), method.arity(), "{}", method.name());
        }
    }

    /// The seeded random inputs are there and varied: every method with
    /// parameters draws at least 10,000, nearly all distinct, and the oracle
    /// both returns values and refuses among them, the fee methods also for a
    /// fee above the denominator. A random part cut to nothing, or collapsed
    /// onto a few inputs, would leave the differential tests to the edge grid
    /// and the directed inputs, and fail here.
    #[test]
    fn random_inputs_are_many_and_varied() {
        for method in Method::ALL.into_iter().filter(|method| method.arity() > 0) {
            let name = method.name();
            let (mut drawn, mut values, mut refused) = (0, 0, 0);
            let (mut distinct, mut above) = (std::collections::HashSet::new(), 0);
            for_each_sourced_case(method, |source, args| {
                if source != Source::Random {
                    return;
                }
                drawn += 1;
                distinct.insert(args.to_vec());
                if method.oracle(args).is_ok() { values += 1 } else { refused += 1 }
                above += usize::from(method.fee_param().is_some_and(|i| args[i] > 10_000));
            });
            assert!(drawn >= 10_000, "{name}: {drawn} random inputs");
            assert!(distinct.len() * 10 >= drawn * 9, "{name}: {} of {drawn} random inputs distinct", distinct.len());
            assert!(
                values > 0 && refused > 0,
                "{name}: of {drawn} random inputs, {values} returned and {refused} were refused"
            );
            if method.fee_param().is_some() {
                assert!(above >= drawn / 10, "{name}: {above} of {drawn} random inputs have a fee above 10000");
            }
        }
    }

    #[test]
    fn exact_quotient_fees_make_the_quotient_the_reserve() {
        for f in EXACT_QUOTIENT_FEES {
            let ratio = 1 + D / (D - u64::from(f));
            assert_eq!((ratio - 1) * (D - u64::from(f)), D, "fee {f}");
        }
    }

    /// A point of a boundary that a method's directed inputs are placed on:
    /// its name, and whether an input, widened to `u128`, is on it.
    type Target = (&'static str, fn(&[u128]) -> bool);

    const TOP: u128 = M as u128;
    const WIDE_D: u128 = D as u128;

    /// get_amount_in's quotient, before the +1, where it has one.
    fn get_amount_in_quotient(args: &[u128]) -> Option<u128> {
        let &[x, ri, ro, f] = args else { return None };
        (x > 0 && ri > 0 && x < ro && f < WIDE_D).then(|| ri * x * WIDE_D / ((ro - x) * (WIDE_D - f)))
    }

    /// `numerator / denominator`, where the denominator is not zero.
    fn quotient(numerator: u128, denominator: u128) -> Option<u128> {
        numerator.checked_div(denominator)
    }

    /// mint_proportional's smaller share, where both reserves and the supply
    /// are positive.
    fn smaller_share(args: &[u128]) -> Option<u128> {
        let &[a0, a1, r0, r1, s] = args else { return None };
        if s == 0 {
            return None;
        }
        Some(quotient(a0 * s, r0)?.min(quotient(a1 * s, r1)?))
    }

    /// burn_share's `(liquidity, balance, the smallest balance that burns a
    /// unit)`, where the supply is positive and holds the liquidity.
    fn burn_step(args: &[u128]) -> Option<(u128, u128)> {
        let &[l, b, s] = args else { return None };
        (l > 0 && l <= s).then(|| (b, s.div_ceil(l)))
    }

    /// The two sides of the fee-adjusted k check, `((Ri + a) * D - a * f) *
    /// (Ro - o)` and `Ri * Ro * D`, at the reserve `ri`.
    fn k_sides(args: &[u128], ri: u128) -> Option<(u128, u128)> {
        let (a, o, ro, f) = match *args {
            [a, o, _, ro] => (a, o, ro, 0),
            [a, o, _, ro, f] => (a, o, ro, f),
            _ => return None,
        };
        (ri > 0 && o < ro && f < WIDE_D).then(|| (((ri + a) * WIDE_D - a * f) * (ro - o), ri * ro * WIDE_D))
    }

    /// On the k equality at the input's own reserve.
    fn k_equal(args: &[u128]) -> bool {
        k_sides(args, args[2]).is_some_and(|(left, right)| left == right)
    }

    /// One unit of reserve above a k equality.
    fn k_one_above_equality(args: &[u128]) -> bool {
        args[2] > 0 && k_sides(args, args[2] - 1).is_some_and(|(left, right)| left == right)
    }

    fn product_floor(args: &[u128]) -> Option<u128> {
        let &[a, b, d] = args else { return None };
        quotient(a * b, d)
    }

    fn product_ceiling(args: &[u128]) -> Option<u128> {
        let &[a, b, d] = args else { return None };
        (d > 0).then(|| (a * b).div_ceil(d))
    }

    fn targets(method: Method) -> Vec<Target> {
        match method {
            Method::MinimumLiquidity => vec![],
            Method::GetAmountOut => vec![
                ("an output of reserve_out - 1", |args| {
                    let &[a, ri, ro, f] = args else { return false };
                    let g = WIDE_D.saturating_sub(f);
                    a > 0 && ri > 0 && ro > 1 && g > 0 && quotient(a * g * ro, ri * WIDE_D + a * g) == Some(ro - 1)
                }),
                ("an exact division at fee 0", |args| {
                    let &[a, ri, ro, f] = args else { return false };
                    f == 0 && a > 0 && ri > 0 && ro > 0 && (a * ro).is_multiple_of(ri + a)
                }),
            ],
            Method::GetAmountIn => vec![
                ("the quotient 2^32 - 2, the largest result", |args| get_amount_in_quotient(args) == Some(TOP - 1)),
                ("the quotient 2^32 - 1, where the +1 overflows", |args| get_amount_in_quotient(args) == Some(TOP)),
                ("the quotient 2^32, the first that does not fit", |args| {
                    get_amount_in_quotient(args) == Some(TOP + 1)
                }),
            ],
            Method::Quote => vec![
                ("the result 2^32 - 1, the largest", |args| {
                    args[0] > 0 && quotient(args[0] * args[2], args[1]) == Some(TOP)
                }),
                ("the result 2^32, the first that does not fit", |args| {
                    args[0] > 0 && quotient(args[0] * args[2], args[1]) == Some(TOP + 1)
                }),
            ],
            Method::MintInitial => vec![
                ("a0 * a1 = 1001^2 - 1, the last that mints nothing", |args| args[0] * args[1] == 1001 * 1001 - 1),
                ("a0 * a1 = 1001^2, the first that mints", |args| args[0] * args[1] == 1001 * 1001),
            ],
            Method::MintProportional => vec![
                ("the smaller share 0, which mints nothing", |args| smaller_share(args) == Some(0)),
                ("the smaller share 1", |args| smaller_share(args) == Some(1)),
                ("the smaller share 2^32 - 1, the largest", |args| smaller_share(args) == Some(TOP)),
                ("the smaller share 2^32, the first that does not fit", |args| smaller_share(args) == Some(TOP + 1)),
            ],
            Method::BurnShare => vec![
                ("the smallest balance that burns a unit", |args| {
                    burn_step(args).is_some_and(|(balance, step)| balance == step)
                }),
                ("one unit of balance less, which burns nothing", |args| {
                    burn_step(args).is_some_and(|(balance, step)| balance + 1 == step)
                }),
            ],
            Method::KHolds | Method::KHoldsWithFee => vec![
                ("the k equality", k_equal),
                ("one unit of reserve above the k equality", k_one_above_equality),
            ],
            Method::MulDiv => vec![
                ("the floor 2^32 - 1, the largest", |args| product_floor(args) == Some(TOP)),
                ("the floor 2^32, the first that does not fit", |args| product_floor(args) == Some(TOP + 1)),
            ],
            Method::MulDivUp => vec![
                ("the ceiling 2^32 - 1, the largest", |args| product_ceiling(args) == Some(TOP)),
                ("the ceiling 2^32 over the floor 2^32 - 1", |args| {
                    product_ceiling(args) == Some(TOP + 1) && product_floor(args) == Some(TOP)
                }),
            ],
        }
    }

    /// The directed inputs, on their own, reach every boundary point they are
    /// placed on: the edge grid and the random inputs would otherwise hide a
    /// directed generator that no longer lands where its doc says.
    #[test]
    fn directed_inputs_reach_their_boundaries() {
        for method in Method::ALL {
            let targets = targets(method);
            let mut reached = vec![false; targets.len()];
            for_each_sourced_case(method, |source, args| {
                if source == Source::Directed {
                    let args: Vec<u128> = args.iter().map(|&arg| u128::from(arg)).collect();
                    for (reached, (_, on)) in reached.iter_mut().zip(&targets) {
                        *reached |= on(&args);
                    }
                }
            });
            for (reached, (target, _)) in reached.iter().zip(&targets) {
                assert!(reached, "{}: no directed input is on {target}", method.name());
            }
        }
    }
}
