//! `wide.inf` and `model.inf` through the test shell's scalar views
//! (`tests/shell/out/main.wasm`), against exact `u128` and `u64` arithmetic,
//! traps included.
//!
//! The 11 methods build no value of 2^80 or more, so they leave most of
//! wide.inf's carries, borrows and traps unexercised, and no obligation
//! reaches them either (`src/main.inf` lists those paths). Here every
//! operation runs on the limb grid (each limb 0, 1, 2^31, 2^32 - 2 or
//! 2^32 - 1, all pairs), on seeded random values of every width, and on named
//! probes, one per carry, borrow and trap path. A `w_` view takes its U128
//! operands as limbs, least significant first, and returns one limb of a U128
//! result, chosen by its last argument; an `m_` view keeps model.inf's own
//! signature, whose 64-bit operands are written high limb first.

mod vm;

use std::collections::HashSet;
use std::fmt::Debug;

use stellar_amm_math_tests::Trap;
use stellar_amm_math_tests::rng::Rng;
use stellar_amm_math_tests::source::read_repo_file;
use vm::Vm;
use wasmi::{Engine, ExternType, Module, ValType, WasmParams};

const SHELL: &str = "tests/shell/out/main.wasm";

const M: u32 = u32::MAX;
const H: u32 = 1 << 31;

/// A U128's limbs, least significant first.
type Limbs = [u32; 4];

fn value(limbs: Limbs) -> u128 {
    limbs.iter().rev().fold(0, |acc, &limb| (acc << 32) | u128::from(limb))
}

fn limbs(value: u128) -> Limbs {
    std::array::from_fn(|k| u32::try_from((value >> (32 * k)) & u128::from(M)).expect("one limb"))
}

/// A 64-bit operand of model.inf, `hi * 2^32 + lo`.
fn join(hi: u32, lo: u32) -> u64 {
    (u64::from(hi) << 32) | u64::from(lo)
}

/// The operands `(hi, lo)` of the 64-bit value `n`.
fn split(n: u64) -> (u32, u32) {
    let [lo, hi, _, _] = limbs(u128::from(n));
    (hi, lo)
}

// ---- the views ------------------------------------------------------------

/// Every view the test shell exports, with its number of `i32` parameters
/// and its result. Each is a view some test below calls; nothing checks
/// that, so a line here is the reviewer's record of it.
const VIEWS: [(&str, usize, ValType); 19] = [
    ("w_from_u32", 2, ValType::I32),
    ("w_mul32", 3, ValType::I32),
    ("w_add", 9, ValType::I32),
    ("w_sub", 9, ValType::I32),
    ("w_mul_by", 6, ValType::I32),
    ("w_eq", 8, ValType::I32),
    ("w_lt", 8, ValType::I32),
    ("w_le", 8, ValType::I32),
    ("w_shr32", 5, ValType::I32),
    ("w_shl1_or", 6, ValType::I32),
    ("w_quotient_fits", 8, ValType::I32),
    ("w_div_to_u32", 8, ValType::I32),
    ("w_isqrt", 4, ValType::I32),
    ("m_widen", 1, ValType::I64),
    ("m_mul_lo", 2, ValType::I32),
    ("m_mul_hi", 2, ValType::I32),
    ("m_mul_by_limb", 4, ValType::I32),
    ("m_div64", 3, ValType::I32),
    ("m_sqrt64", 2, ValType::I32),
];

/// The test shell exports exactly [`VIEWS`], and no other function: a view
/// added to `tests/shell/src/main.inf` fails here until it has a line in the
/// list. (wasmi lists exports by name, so this compares them as sorted
/// lists.)
#[test]
fn the_shell_exports_the_tested_views() {
    let engine = Engine::default();
    let module = Module::new(&engine, &read_repo_file(SHELL)[..]).expect("the test shell validates");
    let signature = |name: &str, params: &[ValType], results: &[ValType]| format!("{name}{params:?} -> {results:?}");
    let mut exported: Vec<String> = module
        .exports()
        .filter_map(|export| {
            let ExternType::Func(func) = export.ty() else { return None };
            Some(signature(export.name(), func.params(), func.results()))
        })
        .collect();
    let mut listed: Vec<String> =
        VIEWS.iter().map(|&(name, arity, result)| signature(name, &vec![ValType::I32; arity], &[result])).collect();
    exported.sort();
    listed.sort();
    assert_eq!(exported, listed, "the functions {SHELL} exports, and VIEWS");
}

struct Shell(Vm);

impl Shell {
    fn load() -> Shell {
        Shell(Vm::load(SHELL))
    }

    /// A U128-valued view, called once per limb: the value, or a trap when
    /// every limb's call traps.
    fn wide<P>(&mut self, name: &str, params: impl Fn(u32) -> P) -> Result<u128, Trap>
    where
        P: WasmParams + Debug + Copy,
    {
        let results: [Result<u32, Trap>; 4] = [0, 1, 2, 3].map(|k| self.0.call(name, params(k)));
        if results.iter().all(Result::is_err) {
            return Err(Trap);
        }
        let limbs = results
            .map(|limb| limb.unwrap_or_else(|_| panic!("{name}{:?}: some limbs trap, some do not", params(0))));
        Ok(value(limbs))
    }

    fn w_from_u32(&mut self, x: u32) -> Result<u128, Trap> {
        self.wide("w_from_u32", |k| (x, k))
    }
    fn w_mul32(&mut self, a: u32, b: u32) -> Result<u128, Trap> {
        self.wide("w_mul32", |k| (a, b, k))
    }
    fn w_add(&mut self, a: u128, b: u128) -> Result<u128, Trap> {
        let ([a0, a1, a2, a3], [b0, b1, b2, b3]) = (limbs(a), limbs(b));
        self.wide("w_add", |k| (a0, a1, a2, a3, b0, b1, b2, b3, k))
    }
    fn w_sub(&mut self, a: u128, b: u128) -> Result<u128, Trap> {
        let ([a0, a1, a2, a3], [b0, b1, b2, b3]) = (limbs(a), limbs(b));
        self.wide("w_sub", |k| (a0, a1, a2, a3, b0, b1, b2, b3, k))
    }
    fn w_mul_by(&mut self, a: u128, m: u32) -> Result<u128, Trap> {
        let [a0, a1, a2, a3] = limbs(a);
        self.wide("w_mul_by", |k| (a0, a1, a2, a3, m, k))
    }
    fn w_shr32(&mut self, a: u128) -> Result<u128, Trap> {
        let [a0, a1, a2, a3] = limbs(a);
        self.wide("w_shr32", |k| (a0, a1, a2, a3, k))
    }
    fn w_shl1_or(&mut self, a: u128, bit: u32) -> Result<u128, Trap> {
        let [a0, a1, a2, a3] = limbs(a);
        self.wide("w_shl1_or", |k| (a0, a1, a2, a3, bit, k))
    }
    fn compare(&mut self, name: &str, a: u128, b: u128) -> Result<bool, Trap> {
        let ([a0, a1, a2, a3], [b0, b1, b2, b3]) = (limbs(a), limbs(b));
        self.0.call_bool(name, (a0, a1, a2, a3, b0, b1, b2, b3))
    }
    fn w_div_to_u32(&mut self, n: u128, d: u128) -> Result<u32, Trap> {
        let ([n0, n1, n2, n3], [d0, d1, d2, d3]) = (limbs(n), limbs(d));
        self.0.call("w_div_to_u32", (n0, n1, n2, n3, d0, d1, d2, d3))
    }
    fn w_isqrt(&mut self, n: u128) -> Result<u32, Trap> {
        let [n0, n1, n2, n3] = limbs(n);
        self.0.call("w_isqrt", (n0, n1, n2, n3))
    }
}

// ---- the exact model ------------------------------------------------------

fn fits_u32_quotient(n: u128, d: u128) -> bool {
    (n >> 32) < d
}

fn model_shl1_or(a: u128, bit: u32) -> Option<u128> {
    (a >> 127 == 0).then(|| (a << 1) | u128::from(bit))
}

fn model_div_to_u32(n: u128, d: u128) -> Option<u32> {
    fits_u32_quotient(n, d).then(|| u32::try_from(n / d).expect("the quotient fits"))
}

fn model_isqrt(n: u128) -> Option<u32> {
    u64::try_from(n).ok().map(|n| u32::try_from(n.isqrt()).expect("the root of a u64 fits a u32"))
}

// ---- comparing ------------------------------------------------------------

/// A result as a failure message shows it: a U128 as its limbs, least
/// significant first, like the operands.
trait Shown {
    fn shown(&self) -> String;
}

impl Shown for u128 {
    fn shown(&self) -> String {
        format!("{:?}", limbs(*self))
    }
}

impl Shown for u64 {
    fn shown(&self) -> String {
        self.to_string()
    }
}

impl Shown for u32 {
    fn shown(&self) -> String {
        self.to_string()
    }
}

impl Shown for bool {
    fn shown(&self) -> String {
        self.to_string()
    }
}

fn shown<T: Shown>(result: &Result<T, Trap>) -> String {
    result.as_ref().map_or_else(|_| "a trap".to_owned(), Shown::shown)
}

/// The cases of one operation: how many ran, trapped and differed, and the
/// first differences.
struct Tally {
    op: &'static str,
    cases: usize,
    traps: usize,
    differing: usize,
    first: Vec<String>,
}

impl Tally {
    fn new(op: &'static str) -> Tally {
        Tally { op, cases: 0, traps: 0, differing: 0, first: Vec::new() }
    }

    /// The view's result must be the model's value, or a trap where the model
    /// has none.
    fn check<T: PartialEq + Shown>(&mut self, operands: impl Debug, actual: Result<T, Trap>, expected: Option<T>) {
        self.cases += 1;
        self.traps += usize::from(actual.is_err());
        let expected = expected.ok_or(Trap);
        if actual != expected {
            self.differing += 1;
            if self.first.len() < 10 {
                let (actual, expected) = (shown(&actual), shown(&expected));
                self.first.push(format!("{}{operands:?}: the shell gives {actual}, exactly {expected}", self.op));
            }
        }
    }

    /// A named probe: one path, reported with its label.
    fn probe<T: PartialEq + Shown>(
        &mut self,
        label: &str,
        operands: impl Debug,
        actual: Result<T, Trap>,
        expected: Option<T>,
    ) {
        let before = self.first.len();
        self.check(&operands, actual, expected);
        if let Some(difference) = self.first.get_mut(before) {
            *difference = format!("{label}: {difference}");
        }
    }

    /// Fails on any difference, or when the cases never trapped although
    /// `can_trap`, or always trapped.
    fn finish(self, can_trap: bool) {
        let Tally { op, cases, traps, differing, first } = self;
        assert!(differing == 0, "{op}: {differing} of {cases} cases differ, the first: {first:#?}");
        assert!(traps < cases, "{op}: every case trapped");
        assert_eq!(traps > 0, can_trap, "{op}: {traps} of {cases} cases trapped");
    }
}

// ---- operands -------------------------------------------------------------

/// Limb values: zero, one, the top bit, and the two largest values.
const GRID_LIMBS: [u32; 5] = [0, 1, H, M - 1, M];

/// Limb values for the unary operations, with a few more bit patterns.
const WIDE_LIMBS: [u32; 7] = [0, 1, 2, H - 1, H, M - 1, M];

/// Every U128 whose limbs are all drawn from `edges`.
fn grid(edges: &[u32]) -> Vec<u128> {
    let mut values = Vec::new();
    for &a in edges {
        for &b in edges {
            for &c in edges {
                for &d in edges {
                    values.push(value([a, b, c, d]));
                }
            }
        }
    }
    values
}

/// A value with zero to four live limbs, each an edge or uniform, so that
/// short values and full ones both appear.
fn random_value(rng: &mut Rng) -> u128 {
    let live = usize::try_from(rng.below(5)).expect("below 5");
    let mut limbs = [0; 4];
    for limb in limbs.iter_mut().take(live) {
        *limb = random_limb(rng);
    }
    value(limbs)
}

fn random_limb(rng: &mut Rng) -> u32 {
    if rng.below(2) == 0 { rng.pick(&WIDE_LIMBS) } else { rng.next_u32() }
}

fn random_u128(rng: &mut Rng) -> u128 {
    (u128::from(rng.next_u64()) << 64) | u128::from(rng.next_u64())
}

const RANDOM_CASES: usize = 20_000;

/// Random operand pairs: independent values, and the second one equal to the
/// first, one away from it, or placed so that the quotient sits at the fit
/// boundary.
fn random_pairs(seed: u64) -> Vec<(u128, u128)> {
    let mut rng = Rng::new(seed);
    (0..RANDOM_CASES)
        .map(|_| {
            let a = random_value(&mut rng);
            let b = match rng.below(8) {
                0 => a,
                1 => a.wrapping_add(1),
                2 => a.wrapping_sub(1),
                3 => (a >> 32) + u128::from(rng.below(3)),
                _ => random_value(&mut rng),
            };
            (a, b)
        })
        .collect()
}

/// The seeded random operands the tests below draw are many and varied: a
/// random part cut to nothing, or a generator collapsed onto a few values,
/// would leave each operation to its grid and its probes, and fail here.
#[test]
fn random_operands_are_many_and_varied() {
    let pairs = random_pairs(2);
    let distinct: HashSet<&(u128, u128)> = pairs.iter().collect();
    assert!(pairs.len() >= 10_000, "{} random pairs", pairs.len());
    assert!(distinct.len() * 3 >= pairs.len() * 2, "{} of {} random pairs distinct", distinct.len(), pairs.len());
    let mut rng = Rng::new(1);
    let values: Vec<u128> = (0..RANDOM_CASES).map(|_| random_value(&mut rng)).collect();
    let distinct: HashSet<&u128> = values.iter().collect();
    assert!(values.len() >= 10_000, "{} random values", values.len());
    assert!(distinct.len() * 2 >= values.len(), "{} of {} random values distinct", distinct.len(), values.len());
    let limbs: HashSet<u32> = (0..RANDOM_CASES).map(|_| random_limb(&mut rng)).collect();
    assert!(limbs.len() * 3 >= RANDOM_CASES, "{} of {RANDOM_CASES} random limbs distinct", limbs.len());
}

// ---- wide.inf -------------------------------------------------------------

#[test]
fn from_u32_and_mul32() {
    let mut shell = Shell::load();
    let mut from_u32 = Tally::new("from_u32");
    let mut mul32 = Tally::new("mul32");
    let columns = [
        0, 1, 2, 0xffff, 0x1_0000, 0x1_0001, 0x1_ffff, 0x7fff_ffff, 0x8000_0000, 0xffff_0000, 0xffff_0001, 0xfffe_ffff,
        M - 1, M,
    ];
    let mut rng = Rng::new(1);
    let randoms: Vec<u32> = (0..200).map(|_| rng.next_u32()).collect();
    for &a in columns.iter().chain(&randoms) {
        from_u32.check(a, shell.w_from_u32(a), Some(u128::from(a)));
        for &b in &columns {
            mul32.check((a, b), shell.w_mul32(a, b), Some(u128::from(a) * u128::from(b)));
        }
    }
    mul32.probe("MAX * MAX", (M, M), shell.w_mul32(M, M), Some(u128::from(M) * u128::from(M)));
    from_u32.finish(false);
    mul32.finish(false);
}

/// A view's limb index selects limb 3 for every value from 3 on.
#[test]
fn a_limb_index_from_3_on_selects_limb_3() {
    let mut shell = Shell::load();
    for k in [3, 4, 1000, M] {
        let limb = shell.0.call::<(u32, u32, u32, u32, u32), u32>("w_shr32", (1, 2, 3, 4, k));
        assert_eq!(limb, Ok(0), "w_shr32(1, 2, 3, 4, {k})");
        let limb = shell.0.call::<(u32, u32, u32, u32, u32, u32), u32>("w_mul_by", (0, 0, 0, 7, 1, k));
        assert_eq!(limb, Ok(7), "w_mul_by(0, 0, 0, 7, 1, {k})");
    }
}

/// Named add probes: one per carry, and the two overflows.
const ADD_PROBES: [(&str, Limbs, Limbs); 8] = [
    ("limb-0 carry", [M, 0, 0, 0], [1, 0, 0, 0]),
    ("limb-1 carry from the limb sum", [0, M, 0, 0], [0, 1, 0, 0]),
    ("limb-1 carry from the incoming carry", [M, M, 0, 0], [1, 0, 0, 0]),
    ("limb-2 carry from the limb sum", [0, 0, M, 0], [0, 0, 1, 0]),
    ("limb-2 carry from the incoming carry, into the top limb", [M, M, M, 0], [1, 0, 0, 0]),
    ("a carry chain with every limb sum wrapping", [M, M, M, 5], [M, M, M, 7]),
    ("2^128 through the carry chain traps", [M, M, M, M], [1, 0, 0, 0]),
    ("top-limb overflow traps", [0, 0, 0, M], [0, 0, 0, 1]),
];

#[test]
fn add() {
    let mut shell = Shell::load();
    let mut tally = Tally::new("add");
    for (label, a, b) in ADD_PROBES {
        tally.probe(label, (a, b), shell.w_add(value(a), value(b)), value(a).checked_add(value(b)));
    }
    let values = grid(&GRID_LIMBS);
    for &a in &values {
        for &b in &values {
            tally.check((limbs(a), limbs(b)), shell.w_add(a, b), a.checked_add(b));
        }
    }
    for (a, b) in random_pairs(2) {
        tally.check((limbs(a), limbs(b)), shell.w_add(a, b), a.checked_add(b));
    }
    tally.finish(true);
}

/// Named sub probes: one per borrow, and the two underflows.
const SUB_PROBES: [(&str, Limbs, Limbs); 7] = [
    ("limb-0 borrow", [0, 1, 0, 0], [1, 0, 0, 0]),
    ("limb-1 borrow from the limb difference", [0, 0, 1, 0], [0, 1, 0, 0]),
    ("limb-1 borrow from the incoming borrow", [0, 0, 1, 0], [1, 0, 0, 0]),
    ("limb-2 borrow from the limb difference", [0, 0, 0, 1], [0, 0, 1, 0]),
    ("limb-2 borrow from the incoming borrow, into the top limb", [0, 0, 0, 1], [1, 0, 0, 0]),
    ("a negative result through the borrow chain traps", [0, 0, 0, 0], [1, 0, 0, 0]),
    ("a negative result at the top limb traps", [0, 0, 0, 1], [0, 0, 0, 2]),
];

#[test]
fn sub() {
    let mut shell = Shell::load();
    let mut tally = Tally::new("sub");
    for (label, a, b) in SUB_PROBES {
        tally.probe(label, (a, b), shell.w_sub(value(a), value(b)), value(a).checked_sub(value(b)));
    }
    let values = grid(&GRID_LIMBS);
    for &a in &values {
        for &b in &values {
            tally.check((limbs(a), limbs(b)), shell.w_sub(a, b), a.checked_sub(b));
        }
    }
    for (a, b) in random_pairs(3) {
        tally.check((limbs(a), limbs(b)), shell.w_sub(a, b), a.checked_sub(b));
    }
    tally.finish(true);
}

/// Named mul_by probes: both ways a product overflows, the largest product
/// that does not, and full carries.
const MUL_BY_PROBES: [(&str, Limbs, u32); 5] = [
    ("the top partial product needs a fifth limb", [0, 0, 0, H], 2),
    ("overflow through the adds, the top partial product in range", [M, M, M, 1_431_655_765], 3),
    ("the largest product below 2^128", [M, M, M, 1_431_655_764], 3),
    ("every partial product carries", [M, M, M, 0], M),
    ("by zero", [M, M, M, M], 0),
];

#[test]
fn mul_by() {
    let mut shell = Shell::load();
    let mut tally = Tally::new("mul_by");
    for (label, a, m) in MUL_BY_PROBES {
        tally.probe(label, (a, m), shell.w_mul_by(value(a), m), value(a).checked_mul(u128::from(m)));
    }
    let multipliers = [0, 1, 2, 3, 10_000, 0xffff, 0x1_0000, H - 1, H, M - 1, M];
    for a in grid(&WIDE_LIMBS) {
        for m in multipliers {
            tally.check((limbs(a), m), shell.w_mul_by(a, m), a.checked_mul(u128::from(m)));
        }
    }
    let mut rng = Rng::new(4);
    for _ in 0..RANDOM_CASES {
        let (a, m) = (random_value(&mut rng), random_limb(&mut rng));
        tally.check((limbs(a), m), shell.w_mul_by(a, m), a.checked_mul(u128::from(m)));
    }
    tally.finish(true);
}

#[test]
fn comparisons() {
    let mut shell = Shell::load();
    let mut eq = Tally::new("eq");
    let mut lt = Tally::new("lt");
    let mut le = Tally::new("le");
    for k in 0..4 {
        let one_at_k = 1_u128 << (32 * k);
        lt.probe(&format!("decided at limb {k}"), k, shell.compare("w_lt", 0, one_at_k), Some(true));
        le.probe(&format!("decided at limb {k}, reversed"), k, shell.compare("w_le", one_at_k, 0), Some(false));
        eq.probe(&format!("differs only at limb {k}"), k, shell.compare("w_eq", 0, one_at_k), Some(false));
    }
    let same = value([5, 6, 7, 8]);
    eq.probe("equal values", same, shell.compare("w_eq", same, same), Some(true));
    le.probe("equal values", same, shell.compare("w_le", same, same), Some(true));
    let values = grid(&GRID_LIMBS);
    let pairs = values.iter().flat_map(|&a| values.iter().map(move |&b| (a, b))).chain(random_pairs(5));
    for (a, b) in pairs {
        let operands = (limbs(a), limbs(b));
        eq.check(operands, shell.compare("w_eq", a, b), Some(a == b));
        lt.check(operands, shell.compare("w_lt", a, b), Some(a < b));
        le.check(operands, shell.compare("w_le", a, b), Some(a <= b));
    }
    eq.finish(false);
    lt.finish(false);
    le.finish(false);
}

#[test]
fn shr32_and_shl1_or() {
    let mut shell = Shell::load();
    let mut shr32 = Tally::new("shr32");
    let mut shl1_or = Tally::new("shl1_or");
    let moves = value([1, 2, 3, 4]);
    shr32.probe("moves every limb down", limbs(moves), shell.w_shr32(moves), Some(moves >> 32));
    let crossing = value([H, H, H, 0]);
    let shifted = shell.w_shl1_or(crossing, 1);
    shl1_or.probe("bits cross every limb boundary", (limbs(crossing), 1), shifted, model_shl1_or(crossing, 1));
    let top = value([0, 0, 0, H]);
    shl1_or.probe("a set top bit traps", (limbs(top), 0), shell.w_shl1_or(top, 0), None);
    let mut rng = Rng::new(6);
    let random = (0..RANDOM_CASES).map(|_| random_value(&mut rng));
    let values: Vec<u128> = grid(&WIDE_LIMBS).into_iter().chain(random).collect();
    for a in values {
        shr32.check(limbs(a), shell.w_shr32(a), Some(a >> 32));
        // A bit above 1 is OR-ed into limb 0 as it stands.
        for bit in [0, 1, 2, 3, H, M] {
            shl1_or.check((limbs(a), bit), shell.w_shl1_or(a, bit), model_shl1_or(a, bit));
        }
    }
    shr32.finish(false);
    shl1_or.finish(true);
}

/// Divisors of every width, at and around powers of two.
fn divisors(rng: &mut Rng) -> Vec<u128> {
    let mut divisors = vec![
        1,
        2,
        3,
        u128::from(M),
        1 << 32,
        (1 << 32) + 1,
        1 << 63,
        u128::from(u64::MAX),
        1 << 64,
        (1 << 95) + 12_345,
        (1 << 96) - 1,
        1 << 96,
        (1 << 127) - 1,
        1 << 127,
        (1 << 127) + 1,
        u128::MAX,
        // The largest denominator of get_amount_out: (2^32 - 1) * (10000 + 9999).
        u128::from(M) * 19_999,
    ];
    for _ in 0..200 {
        let bits = 1 + rng.below(128);
        divisors.push((random_u128(rng) >> (128 - bits)) | 1);
    }
    divisors
}

#[test]
fn quotient_fits_and_div_to_u32() {
    let mut shell = Shell::load();
    let mut fits = Tally::new("quotient_fits");
    let mut div = Tally::new("div_to_u32");
    let mut both = |shell: &mut Shell, n: u128, d: u128, label: Option<&str>| {
        let operands = (limbs(n), limbs(d));
        let (actual_fits, actual_div) = (shell.compare("w_quotient_fits", n, d), shell.w_div_to_u32(n, d));
        if let Some(label) = label {
            fits.probe(label, operands, actual_fits, Some(fits_u32_quotient(n, d)));
            div.probe(label, operands, actual_div, model_div_to_u32(n, d));
        } else {
            fits.check(operands, actual_fits, Some(fits_u32_quotient(n, d)));
            div.check(operands, actual_div, model_div_to_u32(n, d));
        }
    };
    let probes: [(&str, u128, u128); 8] = [
        ("n = d * 2^32 - 1, the largest fitting quotient", value([M, 6, 0, 0]), 7),
        ("n = d * 2^32, the first quotient that does not fit", value([0, 7, 0, 0]), 7),
        ("a zero denominator", 5, 0),
        ("0 / 0", 0, 0),
        ("the largest quotient with a 2^127 divisor", u128::MAX, 1 << 127),
        ("n = d * 2^32 - 1 at d = 2^94", (1 << 126) - 1, 1 << 94),
        ("a quotient of 2^32", value([0, 0, 0, 1]), value([0, 0, 1, 0])),
        ("the remainder near 2^127", u128::MAX, u128::MAX),
    ];
    for (label, n, d) in probes {
        both(&mut shell, n, d, Some(label));
    }
    let values = grid(&GRID_LIMBS);
    for &n in &values {
        for &d in &values {
            both(&mut shell, n, d, None);
        }
    }
    for (n, d) in random_pairs(7) {
        both(&mut shell, n, d, None);
    }
    let mut rng = Rng::new(8);
    for d in divisors(&mut rng) {
        for q in [0, 1, 2, u128::from(H) - 1, u128::from(H), u128::from(M) - 1, u128::from(M)] {
            for r in [0, 1, d - 1, d / 2] {
                if let Some(n) = q.checked_mul(d).and_then(|qd| qd.checked_add(r)) {
                    both(&mut shell, n, d, None);
                }
            }
        }
        // The fit boundary: d * 2^32 - 1 fits, d * 2^32 does not.
        if let Some(limit) = d.checked_mul(1 << 32) {
            for n in [limit - 1, limit, limit.saturating_add(1)] {
                both(&mut shell, n, d, None);
            }
        }
        for (n, d) in [(u128::MAX, d), (d, 0), (0, d)] {
            both(&mut shell, n, d, None);
        }
    }
    fits.finish(false);
    div.finish(true);
}

#[test]
fn isqrt() {
    let mut shell = Shell::load();
    let mut tally = Tally::new("isqrt");
    let top = u128::from(M);
    let probes: [(&str, u128); 5] = [
        ("2^64 - 1", u128::from(u64::MAX)),
        ("(2^32 - 1)^2", top * top),
        ("(2^32 - 1)^2 - 1", top * top - 1),
        ("limb 2 set traps", value([0, 0, 1, 0])),
        ("limb 3 set traps", value([0, 0, 0, 1])),
    ];
    for (label, n) in probes {
        tally.probe(label, limbs(n), shell.w_isqrt(n), model_isqrt(n));
    }
    let mut rng = Rng::new(9);
    let h = u128::from(H);
    let mut roots: Vec<u128> =
        vec![0, 1, 2, 3, 1000, 1001, 0xffff, 0x1_0000, 0x1_0001, h - 1, h, h + 1, top - 2, top - 1, top];
    roots.extend((0..400).map(|_| u128::from(rng.next_u32())));
    // Perfect squares, one either side, and the last value with the same root.
    for s in roots {
        let square = s * s;
        let last = square + 2 * s;
        let around = [square.checked_sub(1), Some(square), Some(square + 1), Some(last), Some(last + 1)];
        for n in around.into_iter().flatten() {
            tally.check(limbs(n), shell.w_isqrt(n), model_isqrt(n));
        }
    }
    for n in [1 << 64, (1 << 64) + 1, 1 << 96, u128::MAX] {
        tally.check(limbs(n), shell.w_isqrt(n), model_isqrt(n));
    }
    for n in grid(&WIDE_LIMBS).into_iter().chain((0..RANDOM_CASES).map(|_| random_value(&mut rng))) {
        tally.check(limbs(n), shell.w_isqrt(n), model_isqrt(n));
    }
    tally.finish(true);
}

// ---- model.inf ------------------------------------------------------------

/// Operand values for the 64-bit views.
const VIEW_EDGES: [u32; 9] = [0, 1, 2, 0xffff, 0x1_0000, H - 1, H, M - 1, M];

#[test]
fn widen_mul_lo_and_mul_hi() {
    let mut shell = Shell::load();
    let mut widen = Tally::new("widen");
    let mut mul_lo = Tally::new("mul_lo");
    let mut mul_hi = Tally::new("mul_hi");
    let mut rng = Rng::new(10);
    for (label, x) in [("widen: 2^32 - 1", M), ("widen: 2^31", H)] {
        widen.probe(label, x, shell.0.call::<u32, u64>("m_widen", x), Some(u64::from(x)));
    }
    let values: Vec<u32> = VIEW_EDGES.into_iter().chain((0..2000).map(|_| rng.next_u32())).collect();
    for &a in &values {
        widen.check(a, shell.0.call::<u32, u64>("m_widen", a), Some(u64::from(a)));
        for &b in &VIEW_EDGES {
            let (hi, lo) = split(u64::from(a) * u64::from(b));
            mul_lo.check((a, b), shell.0.call::<(u32, u32), u32>("m_mul_lo", (a, b)), Some(lo));
            mul_hi.check((a, b), shell.0.call::<(u32, u32), u32>("m_mul_hi", (a, b)), Some(hi));
        }
    }
    widen.finish(false);
    mul_lo.finish(false);
    mul_hi.finish(false);
}

#[test]
fn mul_by_limb() {
    let mut shell = Shell::load();
    let mut tally = Tally::new("mul_by_limb");
    let max_squared = limbs(u128::from(join(M, M)) * u128::from(M));
    let limb_2 = shell.0.call::<(u32, u32, u32, u32), u32>("m_mul_by_limb", (M, M, M, 2));
    tally.probe("mul_by_limb: limb 2 of MAX:MAX * MAX", (M, M, M, 2), limb_2, Some(max_squared[2]));
    let mut check = |shell: &mut Shell, hi: u32, lo: u32, m: u32| {
        let product = limbs(u128::from(join(hi, lo)) * u128::from(m));
        // Limb k for k = 0, 1 or 2, and limb 3 for any other k.
        for k in 0..5 {
            let actual = shell.0.call::<(u32, u32, u32, u32), u32>("m_mul_by_limb", (hi, lo, m, k));
            tally.check((hi, lo, m, k), actual, Some(product[k.min(3) as usize]));
        }
    };
    for hi in VIEW_EDGES {
        for lo in VIEW_EDGES {
            for m in VIEW_EDGES {
                check(&mut shell, hi, lo, m);
            }
        }
    }
    let mut rng = Rng::new(11);
    for _ in 0..RANDOM_CASES / 4 {
        check(&mut shell, random_limb(&mut rng), random_limb(&mut rng), random_limb(&mut rng));
    }
    tally.finish(false);
}

#[test]
fn div64_and_sqrt64() {
    let mut shell = Shell::load();
    let mut div64 = Tally::new("div64");
    let mut sqrt64 = Tally::new("sqrt64");
    let largest = u32::try_from(join(6, M) / 7).expect("6 < 7 keeps the quotient in u32");
    for (label, (hi, lo, d), expected) in
        [("div64: hi = d - 1 (largest quotient)", (6, M, 7), Some(largest)), ("div64: hi = d traps", (7, 0, 7), None)]
    {
        div64.probe(label, (hi, lo, d), shell.0.call::<(u32, u32, u32), u32>("m_div64", (hi, lo, d)), expected);
    }
    let mut check = |shell: &mut Shell, hi: u32, lo: u32, d: u32| {
        let n = join(hi, lo);
        let expected = (hi < d).then(|| u32::try_from(n / u64::from(d)).expect("hi < d keeps the quotient in u32"));
        div64.check((hi, lo, d), shell.0.call::<(u32, u32, u32), u32>("m_div64", (hi, lo, d)), expected);
        let root = u32::try_from(n.isqrt()).expect("fits");
        sqrt64.check((hi, lo), shell.0.call::<(u32, u32), u32>("m_sqrt64", (hi, lo)), Some(root));
    };
    for hi in VIEW_EDGES {
        for lo in VIEW_EDGES {
            // hi < d is the whole domain: d = hi traps, d = hi + 1 is the largest quotient.
            for d in [hi, hi.saturating_add(1), hi.saturating_add(2), M, 0] {
                check(&mut shell, hi, lo, d);
            }
            for d in VIEW_EDGES {
                check(&mut shell, hi, lo, d);
            }
        }
    }
    let mut rng = Rng::new(12);
    for _ in 0..RANDOM_CASES / 4 {
        let (hi, lo) = (random_limb(&mut rng), random_limb(&mut rng));
        let d = if rng.below(3) == 0 { hi.saturating_add(1) } else { random_limb(&mut rng) };
        check(&mut shell, hi, lo, d);
    }
    for _ in 0..400 {
        let s = u64::from(rng.next_u32());
        for n in [s * s, (s * s).saturating_sub(1), s * s + 2 * s] {
            let (hi, lo) = split(n);
            check(&mut shell, hi, lo, 1);
        }
    }
    div64.finish(true);
    sqrt64.finish(false);
}
