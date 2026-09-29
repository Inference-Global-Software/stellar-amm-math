//! A hand mirror of every spec function: `spec PairMath` in `src/main.inf`
//! and `spec WideArith` in `src/model.inf`.
//!
//! The proof build translates each spec function into a Rocq obligation that
//! ends in Admitted, and a false claim compiles as cleanly as a true one. Each
//! test here, named after its spec function, evaluates the claim on concrete
//! inputs, calling the compiled functions: the 11 methods in `proofs/main.wasm`
//! (the module the obligations describe) and model.inf's views in the test
//! shell (the proof module does not export them).
//!
//! The reading is the strict one `src/main.inf` gives above `spec PairMath`:
//! - `let a: u32 = @;` draws an input; a `let` of a term is substituted.
//! - An `assume` filters an input out only where its condition is false. A
//!   condition whose own term does not denote (a call that traps, a division
//!   by zero) filters nothing: the claim must hold there too.
//! - A claim over a term that does not denote is a counterexample, like a
//!   false claim. A bare call claims that the call returns.
//! - Arithmetic in a spec body wraps: `u32` operations here are `wrapping_*`,
//!   and `u64` values are `Wrapping<u64>`.
//!
//! Inputs: the cross product of the edge values, seeded random inputs, and
//! directed witnesses. Each test declares the boundary of every assumption,
//! each `u32` guard such as `a > 0`, `x < ro` or `f < 10000` included, and of
//! every branch and claim that flips, and checks that its inputs reach each
//! exactly: at its last point inside and its first point outside (or, where
//! those are no inputs at all, at the nearest points that are). A mirror that
//! assumes more than its spec function, `a > 1` for `a > 0`, never reaches
//! the claim at `a = 1`, and fails there. Each test also checks that each
//! source of inputs reached the claim, that its random inputs are many,
//! varied and often past the assumptions, and that its directed inputs alone
//! reach the claim at each point their generator places them on.
//!
//! A mirror is a hand translation, and checks the spec function it was
//! written from. The mirror list at the end of this file pins each spec
//! function's body as it read when its mirror was last reviewed: an edited
//! claim or assumption fails `every_mirror_follows_its_spec_function` until
//! the mirror has been reviewed against it and the pin updated. A rebuilt
//! proof and a new ledger row do not stand in for that review.

mod vm;

use std::collections::HashSet;
use std::num::Wrapping;

use stellar_amm_math_tests::rng::Rng;
use stellar_amm_math_tests::source::{SPEC_BLOCKS, read_repo_text, review_pin, spec_functions};
use stellar_amm_math_tests::vectors::{
    EDGES, EXACT_QUOTIENT_FEES, Source, accepted_fee, clamp, factor, for_each_edge_input, near, positive,
    random_fee,
};
use vm::Vm;

type W64 = Wrapping<u64>;

const M: u32 = u32::MAX;
const D: W64 = Wrapping(10_000);
const U64_MAX: W64 = Wrapping(u64::MAX);
const U64_MAX_OVER_D: W64 = Wrapping(1_844_674_407_370_955);
const MAX_U: W64 = Wrapping(429_496);
const MAX_Q: W64 = Wrapping(4_294_967_294);

const RANDOM_INPUTS: usize = 5000;
const DIRECTED_ROUNDS: usize = 200;

/// A term that does not denote: a call that traps, or a division or
/// remainder by zero.
#[derive(Debug)]
struct Undefined(String);

type Term<T> = Result<T, Undefined>;

/// How a mirror decided one input.
enum Verdict {
    /// An assumption filtered the input out.
    Outside,
    Holds,
    Violated(&'static str),
}

/// `assume { assert(condition); }`: the input is outside where the condition
/// is false, and a condition that does not denote filters nothing.
macro_rules! assume {
    ($condition:expr) => {
        if matches!(evaluate(|| Ok($condition)), Ok(false)) {
            return Ok(Verdict::Outside);
        }
    };
}

/// `assert(condition);`: false is a counterexample, and so (through `?`) is
/// a term that does not denote.
macro_rules! claim {
    ($condition:expr) => {
        if !($condition) {
            return Ok(Verdict::Violated(stringify!($condition)));
        }
    };
}

/// A condition's value, or the term in it that does not denote (from `?`).
fn evaluate(condition: impl FnOnce() -> Term<bool>) -> Term<bool> {
    condition()
}

fn div(a: W64, b: W64) -> Term<W64> {
    a.0.checked_div(b.0).map(Wrapping).ok_or_else(|| Undefined(format!("{a} / 0")))
}

fn rem(a: W64, b: W64) -> Term<W64> {
    a.0.checked_rem(b.0).map(Wrapping).ok_or_else(|| Undefined(format!("{a} % 0")))
}

fn sub32(a: u32, b: u32) -> u32 {
    a.wrapping_sub(b)
}

fn add32(a: u32, b: u32) -> u32 {
    a.wrapping_add(b)
}

/// The functions the obligations call.
struct Callee {
    proof: Vm,
    shell: Vm,
}

impl Callee {
    fn load() -> Callee {
        Callee { proof: Vm::load("proofs/main.wasm"), shell: Vm::load("tests/shell/out/main.wasm") }
    }

    fn method<P>(&mut self, name: &str, params: P) -> Term<u32>
    where
        P: wasmi::WasmParams + std::fmt::Debug + Copy,
    {
        self.proof.call(name, params).map_err(|_| Undefined(format!("{name}{params:?} traps")))
    }

    fn predicate<P>(&mut self, name: &str, params: P) -> Term<bool>
    where
        P: wasmi::WasmParams + std::fmt::Debug + Copy,
    {
        self.proof.call_bool(name, params).map_err(|_| Undefined(format!("{name}{params:?} traps")))
    }

    fn view<P>(&mut self, name: &str, params: P) -> Term<u32>
    where
        P: wasmi::WasmParams + std::fmt::Debug + Copy,
    {
        self.shell.call(name, params).map_err(|_| Undefined(format!("{name}{params:?} traps")))
    }

    fn minimum_liquidity(&mut self) -> Term<u32> {
        self.method("minimum_liquidity", ())
    }
    fn get_amount_out(&mut self, a: u32, ri: u32, ro: u32, f: u32) -> Term<u32> {
        self.method("get_amount_out", (a, ri, ro, f))
    }
    fn get_amount_in(&mut self, x: u32, ri: u32, ro: u32, f: u32) -> Term<u32> {
        self.method("get_amount_in", (x, ri, ro, f))
    }
    fn quote(&mut self, a: u32, ra: u32, rb: u32) -> Term<u32> {
        self.method("quote", (a, ra, rb))
    }
    fn mint_initial(&mut self, a0: u32, a1: u32) -> Term<u32> {
        self.method("mint_initial", (a0, a1))
    }
    fn mint_proportional(&mut self, a0: u32, a1: u32, r0: u32, r1: u32, s: u32) -> Term<u32> {
        self.method("mint_proportional", (a0, a1, r0, r1, s))
    }
    fn burn_share(&mut self, l: u32, b: u32, s: u32) -> Term<u32> {
        self.method("burn_share", (l, b, s))
    }
    fn k_holds(&mut self, a: u32, o: u32, ri: u32, ro: u32) -> Term<bool> {
        self.predicate("k_holds", (a, o, ri, ro))
    }
    fn k_holds_with_fee(&mut self, a: u32, o: u32, ri: u32, ro: u32, f: u32) -> Term<bool> {
        self.predicate("k_holds_with_fee", (a, o, ri, ro, f))
    }
    fn mul_div(&mut self, a: u32, b: u32, d: u32) -> Term<u32> {
        self.method("mul_div", (a, b, d))
    }
    fn mul_div_up(&mut self, a: u32, b: u32, d: u32) -> Term<u32> {
        self.method("mul_div_up", (a, b, d))
    }

    fn widen(&mut self, x: u32) -> Term<W64> {
        self.shell.call("m_widen", x).map(Wrapping).map_err(|_| Undefined(format!("widen({x}) traps")))
    }
    fn mul_lo(&mut self, a: u32, b: u32) -> Term<u32> {
        self.view("m_mul_lo", (a, b))
    }
    fn mul_hi(&mut self, a: u32, b: u32) -> Term<u32> {
        self.view("m_mul_hi", (a, b))
    }
    fn mul_by_limb(&mut self, hi: u32, lo: u32, m: u32, k: u32) -> Term<u32> {
        self.view("m_mul_by_limb", (hi, lo, m, k))
    }
    fn div64(&mut self, hi: u32, lo: u32, d: u32) -> Term<u32> {
        self.view("m_div64", (hi, lo, d))
    }
    fn sqrt64(&mut self, hi: u32, lo: u32) -> Term<u32> {
        self.view("m_sqrt64", (hi, lo))
    }
}

// ---- the runner -----------------------------------------------------------

/// A boundary the inputs must reach exactly from both sides: `distance` is at
/// most 0 on one side and positive on the other, and `None` where it does not
/// apply. Some input must lie at `nearest.0` and reach the claim, and some
/// input at `nearest.1`, which must also reach the claim unless the boundary
/// is an assumption's, whose outside it filters out.
struct Boundary<const N: usize> {
    name: &'static str,
    distance: Distance<N>,
    filters_outside: bool,
    nearest: (i128, i128),
}

type Distance<const N: usize> = fn([u32; N]) -> Option<i128>;

/// An assumption's edge: inside at most 0, filtered out above.
fn assumption<const N: usize>(name: &'static str, distance: Distance<N>) -> Boundary<N> {
    Boundary { name, distance, filters_outside: true, nearest: (0, 1) }
}

/// Where a branch turns or a claimed value flips: both sides reach the claim.
fn branch<const N: usize>(name: &'static str, distance: Distance<N>) -> Boundary<N> {
    Boundary { name, distance, filters_outside: false, nearest: (0, 1) }
}

impl<const N: usize> Boundary<N> {
    /// A boundary whose own points, at distances 0 and 1, are no inputs: the
    /// inputs must reach the nearest ones that are.
    fn reached_at(self, inside: i128, outside: i128) -> Boundary<N> {
        assert!(inside <= 0 && outside > 0, "`{}`: {inside} is not inside, or {outside} not outside", self.name);
        Boundary { nearest: (inside, outside), ..self }
    }
}

/// A point a directed generator places inputs on: its name, and whether an
/// input is on it. Some directed input on it must reach the claim.
type Target<const N: usize> = (&'static str, fn([u32; N]) -> bool);

/// Where a spec function's inputs come from, besides the edge values and
/// random draws every test gets, and what they must reach.
struct Inputs<const N: usize> {
    /// The positions that are fees, which take `vectors::FEES` and fee-sized
    /// values.
    fees: &'static [usize],
    directed: fn(&mut Rng) -> Vec<[u32; N]>,
    /// The points `directed` places its inputs on. The edge grid and the
    /// random inputs would hide a generator that no longer lands on them, so
    /// the directed inputs must reach the claim at each on their own.
    targets: Vec<Target<N>>,
    boundaries: Vec<Boundary<N>>,
}

/// How many of one source's inputs ran, and how many reached the claim.
#[derive(Default)]
struct Reached {
    inputs: usize,
    inside: usize,
}

/// What each source of inputs reached.
#[derive(Default)]
struct Reach {
    edges: Reached,
    random: Reached,
    directed: Reached,
}

impl Reach {
    fn of(&mut self, source: Source) -> &mut Reached {
        match source {
            Source::Edges => &mut self.edges,
            Source::Random => &mut self.random,
            Source::Directed => &mut self.directed,
        }
    }
}

/// The least random inputs a test runs, and the least of them that must be
/// distinct: a random part cut down or collapsed onto a few inputs fails.
const MIN_RANDOM_INPUTS: usize = 2500;

/// The least share of the random inputs, one in this many, that must reach
/// the claim. The mirror whose assumptions filter the most lets 1165 of its
/// 5000 through.
const MIN_RANDOM_INSIDE: usize = 10;

/// Runs the mirror of spec function `spec` on every input. Fails with the
/// first counterexamples; when a source of inputs never gets past the
/// assumptions; when the inputs miss a side of a boundary; when no directed
/// input reaches the claim at one of the directed targets; or when the random
/// inputs are too few, too alike, or too rarely past the assumptions.
fn check<const N: usize>(spec: &str, inputs: &Inputs<N>, mirror: impl Fn(&mut Callee, [u32; N]) -> Term<Verdict>) {
    let mut callee = Callee::load();
    let mut counterexamples = Vec::new();
    let mut reach = Reach::default();
    let mut sides = vec![[false; 2]; inputs.boundaries.len()];
    let mut on_target = vec![false; inputs.targets.len()];
    let mut distinct_random = HashSet::new();
    let mut run = |args: [u32; N], source: Source| {
        reach.of(source).inputs += 1;
        if source == Source::Random {
            distinct_random.insert(args);
        }
        let verdict = mirror(&mut callee, args);
        let held = matches!(verdict, Ok(Verdict::Holds));
        match verdict {
            Ok(Verdict::Outside) => {}
            Ok(Verdict::Holds) => reach.of(source).inside += 1,
            Ok(Verdict::Violated(claim)) => counterexamples.push(format!("{args:?}: `{claim}` is false")),
            Err(Undefined(what)) => counterexamples.push(format!("{args:?}: {what}")),
        }
        for (boundary, side) in inputs.boundaries.iter().zip(&mut sides) {
            let distance = (boundary.distance)(args);
            if distance == Some(boundary.nearest.0) && held {
                side[0] = true;
            }
            if distance == Some(boundary.nearest.1) && (held || boundary.filters_outside) {
                side[1] = true;
            }
        }
        if source == Source::Directed && held {
            for (hit, (_, on)) in on_target.iter_mut().zip(&inputs.targets) {
                *hit |= on(args);
            }
        }
    };

    for_each_edge_input(N, inputs.fees, |args| run(args.try_into().expect("N values"), Source::Edges));
    let mut rng = Rng::new(seed(spec));
    for _ in 0..RANDOM_INPUTS {
        let args = std::array::from_fn(|i| {
            if inputs.fees.contains(&i) { random_fee(&mut rng) } else { rng.mixed_u32(&EDGES) }
        });
        run(args, Source::Random);
    }
    for _ in 0..DIRECTED_ROUNDS {
        for args in (inputs.directed)(&mut rng) {
            run(args, Source::Directed);
        }
    }

    assert!(
        counterexamples.is_empty(),
        "{spec}: {} counterexamples, the first: {:#?}",
        counterexamples.len(),
        &counterexamples[..counterexamples.len().min(5)]
    );
    for source in Source::ALL {
        let r = reach.of(source);
        assert!(r.inside > 0, "{spec}: none of the {} {source:?} inputs got past the assumptions", r.inputs);
    }
    for (boundary, side) in inputs.boundaries.iter().zip(&sides) {
        let (inside, outside) = boundary.nearest;
        assert!(side[0], "{spec}: no input that reaches the claim is at distance {inside} of `{}`", boundary.name);
        assert!(side[1], "{spec}: no input is at distance {outside} of `{}`, just past it", boundary.name);
    }
    for (hit, (target, _)) in on_target.iter().zip(&inputs.targets) {
        assert!(hit, "{spec}: no directed input reaches the claim at {target}");
    }
    let (drawn, inside, distinct) = (reach.random.inputs, reach.random.inside, distinct_random.len());
    assert!(drawn >= MIN_RANDOM_INPUTS, "{spec}: {drawn} random inputs");
    assert!(distinct >= MIN_RANDOM_INPUTS, "{spec}: {distinct} of {drawn} random inputs distinct");
    assert!(inside * MIN_RANDOM_INSIDE >= drawn, "{spec}: {inside} of {drawn} random inputs got past the assumptions");
}

/// A claim about fixed values (a body with no `@`): it must hold.
fn check_fixed(spec: &str, mirror: impl Fn(&mut Callee) -> Term<Verdict>) {
    match mirror(&mut Callee::load()) {
        Ok(Verdict::Holds) => {}
        Ok(Verdict::Outside) => unreachable!("{spec} has no assumptions"),
        Ok(Verdict::Violated(claim)) => panic!("{spec}: `{claim}` is false"),
        Err(Undefined(what)) => panic!("{spec}: {what}"),
    }
}

/// Each spec function draws from its own stream (FNV-1a of its name).
fn seed(spec: &str) -> u64 {
    spec.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3))
}

fn wide(x: u32) -> i128 {
    i128::from(x)
}

// ---- boundaries, in exact integer arithmetic ------------------------------

const DI: i128 = 10_000;

/// get_amount_in's quotient `floor(Ri * x * D / ((Ro - x) * G))`.
fn amount_in_quotient([x, ri, ro, f]: [u32; 4]) -> Option<i128> {
    (x > 0 && x < ro && f < 10_000).then(|| wide(ri) * wide(x) * DI / (wide(ro - x) * (DI - wide(f))))
}

/// The output get_amount_out quotes.
fn amount_out([a, ri, ro, f]: [u32; 4]) -> Option<i128> {
    let g = DI - wide(f);
    (a > 0 && ri > 0 && ro > 0 && f < 10_000).then(|| wide(a) * g * wide(ro) / (wide(ri) * DI + wide(a) * g))
}

/// How far `a * b` is past `d * 2^32 - 1`, the last product whose quotient by
/// `d` fits a u32.
fn past_fit(a: u32, b: u32, d: u32) -> i128 {
    wide(a) * wide(b) - ((wide(d) << 32) - 1)
}

/// A share `floor(a * S / R)` of mint_proportional, where `R > 0`.
fn share(a: u32, s: u32, r: u32) -> Option<i128> {
    (r > 0).then(|| wide(a) * wide(s) / wide(r))
}

// ---- the u32 guards -------------------------------------------------------

/// The guards of a get_amount_out claim over `[a, ri, ro, f]`,
/// `a > 0 && ri > 0 && ro > 0 && f < 10000`, as assumption boundaries. Each
/// u32 guard of every mirror is one: a mirror that assumed more than its spec
/// function (`a > 1` for `a > 0`) would never reach the claim at the guard's
/// last point inside (`a = 1`), and fails there.
fn amount_out_guards() -> Vec<Boundary<4>> {
    vec![
        assumption("a > 0", |[a, ..]| Some(1 - wide(a))),
        assumption("ri > 0", |[_, ri, ..]| Some(1 - wide(ri))),
        assumption("ro > 0", |[_, _, ro, _]| Some(1 - wide(ro))),
        assumption("f < 10000", |[.., f]| Some(wide(f) - 9999)),
    ]
}

/// The guards of a get_amount_in claim over `[x, ri, ro, f]`,
/// `x > 0 && ri > 0 && ro > 0 && x < ro && f < 10000`. No `x` lies between 0
/// and `ro = 1`, so `ro > 0` is reached at `ro = 2`.
fn amount_in_guards() -> Vec<Boundary<4>> {
    vec![
        assumption("x > 0", |[x, ..]| Some(1 - wide(x))),
        assumption("ri > 0", |[_, ri, ..]| Some(1 - wide(ri))),
        assumption("ro > 0", |[_, _, ro, _]| Some(1 - wide(ro))).reached_at(-1, 1),
        assumption("x < ro", |[x, _, ro, _]| Some(wide(x) - wide(ro) + 1)),
        assumption("f < 10000", |[.., f]| Some(wide(f) - 9999)),
    ]
}

/// The guards of a fee-adjusted k claim over `[a, o, ri, ro, f]`,
/// `f < 10000 && ri > 0 && o < ro`.
fn k_guards() -> Vec<Boundary<5>> {
    vec![
        assumption("f < 10000", |[.., f]| Some(wide(f) - 9999)),
        assumption("ri > 0", |[_, _, ri, _, _]| Some(1 - wide(ri))),
        assumption("o < ro", |[_, o, _, ro, _]| Some(wide(o) - wide(ro) + 1)),
    ]
}

/// The guards of a mul_div claim over `[a, b, d]`, `d > 0`.
fn product_guards() -> Vec<Boundary<3>> {
    vec![assumption("d > 0", |[.., d]| Some(1 - wide(d)))]
}

// ---- spec PairMath --------------------------------------------------------

const FEE_LAST: &[usize] = &[3];

/// Directed get_amount_out inputs: huge inputs against tiny reserves, where
/// the output reaches `reserve_out - 1` or `reserve_out - 2`, and exact
/// divisions.
fn get_amount_out_directed(rng: &mut Rng) -> Vec<[u32; 4]> {
    let ro = rng.range_u32(2, 5000);
    let x = rng.range_u32(1, M / 2);
    vec![
        [M, 1, ro, 0],
        [M, 1, ro, accepted_fee(rng)],
        [M, 1, rng.range_u32(429_498, 858_993), 9999],
        [x, x, x, 0],
        [x, x, 2 * x, 0],
        [positive(rng), positive(rng), positive(rng), accepted_fee(rng)],
    ]
}

/// Where [`get_amount_out_directed`] places its inputs.
fn get_amount_out_targets() -> Vec<Target<4>> {
    vec![
        ("an output of reserve_out - 1", |args| amount_out(args) == Some(wide(args[2]) - 1)),
        ("an output of reserve_out - 2 at fee 9999", |args| {
            args[3] == 9999 && amount_out(args) == Some(wide(args[2]) - 2)
        }),
        ("an exact division at fee 0", |[a, ri, ro, f]| {
            f == 0 && a > 0 && (wide(a) * wide(ro)) % (wide(ri) + wide(a)) == 0
        }),
    ]
}

#[test]
fn get_amount_out_returns_below_reserve() {
    let inputs = Inputs {
        fees: FEE_LAST,
        directed: get_amount_out_directed,
        targets: get_amount_out_targets(),
        boundaries: amount_out_guards(),
    };
    check("get_amount_out_returns_below_reserve", &inputs, |c, [a, ri, ro, f]| {
        assume!(a > 0 && ri > 0 && ro > 0 && f < 10_000);
        c.get_amount_out(a, ri, ro, f)?;
        claim!(c.get_amount_out(a, ri, ro, f)? < ro);
        Ok(Verdict::Holds)
    });
}

#[test]
fn get_amount_out_is_largest_feasible() {
    let mut boundaries = amount_out_guards();
    boundaries.push(branch("o + 1 < ro", |args| amount_out(args).map(|o| o + 2 - wide(args[2]))));
    let targets = get_amount_out_targets();
    let inputs = Inputs { fees: FEE_LAST, directed: get_amount_out_directed, targets, boundaries };
    check("get_amount_out_is_largest_feasible", &inputs, |c, [a, ri, ro, f]| {
        assume!(a > 0 && ri > 0 && ro > 0 && f < 10_000);
        let o = c.get_amount_out(a, ri, ro, f)?;
        claim!(c.k_holds_with_fee(a, o, ri, ro, f)?);
        if add32(o, 1) < ro {
            claim!(!c.k_holds_with_fee(a, add32(o, 1), ri, ro, f)?);
        }
        Ok(Verdict::Holds)
    });
}

#[test]
fn get_amount_out_is_exact() {
    /// Reserves at the 16-bit split the formula uses.
    fn directed(rng: &mut Rng) -> Vec<[u32; 4]> {
        let mut inputs = get_amount_out_directed(rng);
        for ro in [1, 65_535, 65_536, 65_537, 131_072, M - 65_535, M] {
            inputs.push([M, 1, ro, 0]);
            inputs.push([positive(rng), positive(rng), ro, accepted_fee(rng)]);
        }
        inputs
    }
    let mut targets = get_amount_out_targets();
    targets.push(("a reserve_out of 2^32 - 2^16, the last multiple of 2^16", |[.., ro, _]| ro == M - 65_535));
    let inputs = Inputs { fees: FEE_LAST, directed, targets, boundaries: amount_out_guards() };
    check("get_amount_out_is_exact", &inputs, |c, [a, ri, ro, f]| {
        assume!(a > 0 && ri > 0 && ro > 0 && f < 10_000);
        let p = c.widen(a)? * c.widen(sub32(10_000, f))?;
        let den = c.widen(ri)? * D + p;
        let rh = c.widen(ro)? >> 16;
        let rl = c.widen(ro)? & Wrapping(65_535);
        let ph = p * rh;
        let pl = p * rl;
        let expected = div(ph, den)? * Wrapping(65_536) + div(rem(ph, den)? * Wrapping(65_536) + pl, den)?;
        let out = c.get_amount_out(a, ri, ro, f)?;
        claim!(c.widen(out)? == expected);
        Ok(Verdict::Holds)
    });
}

#[test]
fn get_amount_out_matches_v2_formula() {
    /// The envelope's edge in `a`, `floor(floor((2^64 - 1) / Ro) / G)`, and
    /// the amounts next to it. At most reserves `a * G` stays below
    /// `floor((2^64 - 1) / Ro)` even at the edge, where `<` and `<=` agree; at
    /// the reserves of [`DIVIDING`] G divides it, and the edge has `a * G`
    /// equal to it.
    fn directed(rng: &mut Rng) -> Vec<[u32; 4]> {
        let f = accepted_fee(rng);
        let g = 10_000 - u64::from(f);
        let ro = clamp(i128::from((1_u64 << 32) / g) + i128::from(rng.below(1 << 30))).max(1);
        let a = u64::MAX / u64::from(ro) / g;
        let mut inputs: Vec<[u32; 4]> = near(i128::from(a)).map(|a| [a.max(1), positive(rng), ro, f]).to_vec();
        for (ro, f) in DIVIDING {
            let a = u64::MAX / u64::from(ro) / (10_000 - u64::from(f));
            inputs.push([u32::try_from(a).expect("the edge fits a u32"), positive(rng), ro, f]);
        }
        inputs
    }
    /// Reserves and fees where G divides `floor((2^64 - 1) / Ro)`, and the
    /// edge `a` fits a u32: G = 10000, 9970 and 2.
    const DIVIDING: [(u32, u32); 3] = [(429_665, 0), (448_571, 30), (2_147_483_649, 9998)];
    fn past_envelope([a, _, ro, f]: [u32; 4]) -> Option<i128> {
        let envelope = |ro: u32, f: u32| i128::from(u64::MAX / u64::from(ro) / (10_000 - u64::from(f)));
        (a > 0 && ro > 0 && f < 10_000).then(|| wide(a) - envelope(ro, f))
    }
    let mut boundaries = amount_out_guards();
    boundaries.push(assumption("a * G <= floor((2^64 - 1) / Ro)", past_envelope));
    let targets: Vec<Target<4>> = vec![
        ("a at floor(floor((2^64 - 1) / Ro) / G)", |args| past_envelope(args) == Some(0)),
        ("a * G = floor((2^64 - 1) / Ro)", |[a, _, ro, f]| {
            ro > 0 && f < 10_000 && wide(a) * (DI - wide(f)) == i128::from(u64::MAX / u64::from(ro))
        }),
    ];
    let inputs = Inputs { fees: FEE_LAST, directed, targets, boundaries };
    check("get_amount_out_matches_v2_formula", &inputs, |c, [a, ri, ro, f]| {
        assume!(a > 0 && ri > 0 && ro > 0 && f < 10_000);
        assume!(c.widen(a)? * c.widen(sub32(10_000, f))? <= div(U64_MAX, c.widen(ro)?)?);
        let g = c.widen(sub32(10_000, f))?;
        let expected = div(c.widen(a)? * g * c.widen(ro)?, c.widen(ri)? * D + c.widen(a)? * g)?;
        let out = c.get_amount_out(a, ri, ro, f)?;
        claim!(c.widen(out)? == expected);
        Ok(Verdict::Holds)
    });
}

#[test]
fn get_amount_out_never_falls_as_amount_in_rises() {
    fn directed(rng: &mut Rng) -> Vec<[u32; 5]> {
        let (a, ri, ro, f) = (positive(rng), positive(rng), positive(rng), accepted_fee(rng));
        vec![[a, a, ri, ro, f], [a, a.saturating_add(1), ri, ro, f], [1, M, ri, ro, f], [a, M, ri, ro, f]]
    }
    let targets: Vec<Target<5>> = vec![
        ("a2 = a1 + 1, the next amount", |[a1, a2, ..]| wide(a2) == wide(a1) + 1),
        ("a1 = 1 and a2 = 2^32 - 1, the widest step", |[a1, a2, ..]| a1 == 1 && a2 == M),
    ];
    let boundaries: Vec<Boundary<5>> = vec![
        assumption("a1 > 0", |[a1, ..]| Some(1 - wide(a1))),
        assumption("a1 <= a2", |[a1, a2, ..]| Some(wide(a1) - wide(a2))),
        assumption("ri > 0", |[_, _, ri, _, _]| Some(1 - wide(ri))),
        assumption("ro > 0", |[.., ro, _]| Some(1 - wide(ro))),
        assumption("f < 10000", |[.., f]| Some(wide(f) - 9999)),
    ];
    let inputs = Inputs { fees: &[4], directed, targets, boundaries };
    check("get_amount_out_never_falls_as_amount_in_rises", &inputs, |c, [a1, a2, ri, ro, f]| {
        assume!(a1 > 0 && a1 <= a2 && ri > 0 && ro > 0 && f < 10_000);
        claim!(c.get_amount_out(a1, ri, ro, f)? <= c.get_amount_out(a2, ri, ro, f)?);
        Ok(Verdict::Holds)
    });
}

#[test]
fn get_amount_out_never_rises_with_fee() {
    fn directed(rng: &mut Rng) -> Vec<[u32; 5]> {
        let (a, ri, ro, f) = (positive(rng), positive(rng), positive(rng), accepted_fee(rng));
        let next = (f + 1).min(9999);
        [(f, f), (f, next), (0, 9999), (f, 9999), (f, 10_000)].map(|(f1, f2)| [a, ri, ro, f1, f2]).to_vec()
    }
    let targets: Vec<Target<5>> = vec![
        ("f2 = f1 + 1, the next fee", |[.., f1, f2]| wide(f2) == wide(f1) + 1),
        ("f1 = 0 and f2 = 9999, the widest step", |[.., f1, f2]| f1 == 0 && f2 == 9999),
    ];
    let boundaries: Vec<Boundary<5>> = vec![
        assumption("a > 0", |[a, ..]| Some(1 - wide(a))),
        assumption("ri > 0", |[_, ri, ..]| Some(1 - wide(ri))),
        assumption("ro > 0", |[_, _, ro, _, _]| Some(1 - wide(ro))),
        assumption("f1 <= f2", |[.., f1, f2]| Some(wide(f1) - wide(f2))),
        assumption("f2 < 10000", |[.., f2]| Some(wide(f2) - 9999)),
    ];
    let inputs = Inputs { fees: &[3, 4], directed, targets, boundaries };
    check("get_amount_out_never_rises_with_fee", &inputs, |c, [a, ri, ro, f1, f2]| {
        assume!(a > 0 && ri > 0 && ro > 0 && f1 <= f2 && f2 < 10_000);
        claim!(c.get_amount_out(a, ri, ro, f2)? <= c.get_amount_out(a, ri, ro, f1)?);
        Ok(Verdict::Holds)
    });
}

/// get_amount_in's quotient in the split form its claims use:
/// `(u, u * D + (v * D) / q)` for `n = Ri * x = u * q + v` and
/// `q = (Ro - x) * G`.
fn amount_in_split(c: &mut Callee, x: u32, ri: u32, ro: u32, f: u32) -> Term<(W64, W64)> {
    let n = c.widen(ri)? * c.widen(x)?;
    let q = c.widen(sub32(ro, x))? * c.widen(sub32(10_000, f))?;
    let u = div(n, q)?;
    let v = rem(n, q)?;
    Ok((u, u * D + div(v * D, q)?))
}

/// Directed get_amount_in inputs: quotients of exactly 1 to 3 (where the
/// result crosses 2) and `2^32 - 4` to `2^32 - 1` (where it stops fitting),
/// and random neighbourhoods of those and of `u = 429496`.
fn get_amount_in_directed(rng: &mut Rng) -> Vec<[u32; 4]> {
    let mut inputs = Vec::new();
    let f = rng.pick(&EXACT_QUOTIENT_FEES);
    let ratio = 1 + 10_000 / (10_000 - f);
    let x = rng.range_u32(1, M / ratio);
    for ri in [1, 2, 3, M - 3, M - 2, M - 1, M] {
        inputs.push([x, ri, x * ratio, f]);
    }
    let ro = clamp(wide(positive(rng)) + 1).max(2);
    let x = rng.range_u32(1, ro - 1);
    let f = accepted_fee(rng);
    let q = wide(ro - x) * (DI - wide(f));
    for target in [wide(M) - 2, wide(M) - 1, wide(M), 429_496 * DI, 429_497 * DI] {
        for ri in near(target * q / (wide(x) * DI)) {
            inputs.push([x, ri, ro, f]);
        }
    }
    inputs
}

/// The edge of the assumption every get_amount_in claim makes: the result,
/// its quotient plus one, fits a u32.
fn amount_in_fits() -> Boundary<4> {
    assumption("floor(Ri * x * D / Q) <= 2^32 - 2", |args| {
        amount_in_quotient(args).map(|quotient| quotient - (wide(M) - 1))
    })
}

/// `u = floor(Ri * x / Q)` of get_amount_in's split form.
fn amount_in_u([x, ri, ro, f]: [u32; 4]) -> Option<i128> {
    (x > 0 && x < ro && f < 10_000).then(|| wide(ri) * wide(x) / (wide(ro - x) * (DI - wide(f))))
}

/// The assumptions of the claims over get_amount_in's split form: the
/// guards, and the edges of the split form's two conjuncts, `u <= 429496` and
/// `floor_nd_q <= 2^32 - 2`.
fn get_amount_in_boundaries() -> Vec<Boundary<4>> {
    let u_fits = assumption("u = floor(Ri * x / Q) <= 429496", |args| amount_in_u(args).map(|u| u - 429_496));
    let mut boundaries = amount_in_guards();
    boundaries.extend([u_fits, amount_in_fits()]);
    boundaries
}

/// Where [`get_amount_in_directed`] places its inputs that reach the claims.
fn get_amount_in_targets() -> Vec<Target<4>> {
    vec![
        ("the quotient 2, whose result 3 is the first above 2", |args| amount_in_quotient(args) == Some(2)),
        ("the quotient 2^32 - 2, whose result 2^32 - 1 is the largest", |args| {
            amount_in_quotient(args) == Some(wide(M) - 1)
        }),
        ("u = 429496, the largest the split form admits", |args| amount_in_u(args) == Some(429_496)),
    ]
}

#[test]
fn get_amount_in_buys_the_target() {
    let inputs = Inputs {
        fees: FEE_LAST,
        directed: get_amount_in_directed,
        targets: get_amount_in_targets(),
        boundaries: get_amount_in_boundaries(),
    };
    check("get_amount_in_buys_the_target", &inputs, |c, [x, ri, ro, f]| {
        assume!(x > 0 && ri > 0 && ro > 0 && x < ro && f < 10_000);
        assume!({
            let (u, floor_nd_q) = amount_in_split(c, x, ri, ro, f)?;
            u <= MAX_U && floor_nd_q <= MAX_Q
        });
        let i = c.get_amount_in(x, ri, ro, f)?;
        claim!(c.get_amount_out(i, ri, ro, f)? >= x);
        claim!(c.k_holds_with_fee(i, x, ri, ro, f)?);
        Ok(Verdict::Holds)
    });
}

#[test]
fn get_amount_in_overpays_by_at_most_one() {
    let mut boundaries = get_amount_in_boundaries();
    boundaries.push(branch("i > 2", |args| amount_in_quotient(args).map(|quotient| 3 - (quotient + 1))));
    let targets = get_amount_in_targets();
    let inputs = Inputs { fees: FEE_LAST, directed: get_amount_in_directed, targets, boundaries };
    check("get_amount_in_overpays_by_at_most_one", &inputs, |c, [x, ri, ro, f]| {
        assume!(x > 0 && ri > 0 && ro > 0 && x < ro && f < 10_000);
        assume!({
            let (u, floor_nd_q) = amount_in_split(c, x, ri, ro, f)?;
            u <= MAX_U && floor_nd_q <= MAX_Q
        });
        let i = c.get_amount_in(x, ri, ro, f)?;
        if i > 2 {
            claim!(c.get_amount_out(sub32(i, 2), ri, ro, f)? < x);
        }
        Ok(Verdict::Holds)
    });
}

#[test]
fn get_amount_in_is_exact() {
    let inputs = Inputs {
        fees: FEE_LAST,
        directed: get_amount_in_directed,
        targets: get_amount_in_targets(),
        boundaries: get_amount_in_boundaries(),
    };
    check("get_amount_in_is_exact", &inputs, |c, [x, ri, ro, f]| {
        assume!(x > 0 && ri > 0 && ro > 0 && x < ro && f < 10_000);
        assume!({
            let (u, floor_nd_q) = amount_in_split(c, x, ri, ro, f)?;
            u <= MAX_U && floor_nd_q <= MAX_Q
        });
        let (_, floor_nd_q) = amount_in_split(c, x, ri, ro, f)?;
        let i = c.get_amount_in(x, ri, ro, f)?;
        claim!(c.widen(i)? == floor_nd_q + Wrapping(1));
        Ok(Verdict::Holds)
    });
}

/// Factor pairs `(p, q)`, both below 2^32, of `floor((2^64 - 1) / D) + d` for
/// `d = -2`, `-1` and `2`. The bound itself and `d = 1` have no such pair, so
/// these are the reachable products nearest it on either side.
const ENVELOPE_PAIRS: [(u32, u32); 6] = [
    (508_971, 3_624_321_243),
    (1_526_913, 1_208_107_081),
    (14_081_531, 130_999_563),
    (1_815_959, 1_015_812_806),
    (3_631_918, 507_906_403),
    (19_349_881, 95_332_597),
];

/// Whether `n` is a product of two `u32` values.
fn is_u32_product(n: u64) -> bool {
    (n.div_ceil(u64::from(M)).max(1)..=n.isqrt()).any(|p| n.is_multiple_of(p))
}

#[test]
fn envelope_pairs_are_the_nearest_products() {
    let bound = U64_MAX_OVER_D.0;
    for (p, q) in ENVELOPE_PAIRS {
        let product = u64::from(p) * u64::from(q);
        assert!([bound - 2, bound - 1, bound + 2].contains(&product), "{p} * {q} = {product}");
    }
    assert!(!is_u32_product(bound), "the bound {bound} is a product of two u32 values");
    assert!(!is_u32_product(bound + 1), "the bound + 1, {}, is a product of two u32 values", bound + 1);
    assert!(is_u32_product(bound - 1), "the bound - 1, {}, is no product of two u32 values", bound - 1);
    assert!(is_u32_product(bound + 2), "the bound + 2, {}, is no product of two u32 values", bound + 2);
}

#[test]
fn get_amount_in_matches_v2_formula() {
    fn directed(rng: &mut Rng) -> Vec<[u32; 4]> {
        let mut inputs = get_amount_in_directed(rng);
        for (p, q) in ENVELOPE_PAIRS {
            inputs.push([p, q, p.saturating_add(positive(rng)), accepted_fee(rng)]);
            inputs.push([q, p, q.saturating_add(positive(rng)), accepted_fee(rng)]);
        }
        inputs
    }
    // The bound and one past it are no products of two u32 values (see
    // ENVELOPE_PAIRS), so the nearest inputs lie one inside and two past it.
    let product_fits = assumption("Ri * x <= floor((2^64 - 1) / D)", |[x, ri, _, _]| {
        Some(wide(ri) * wide(x) - i128::from(U64_MAX_OVER_D.0))
    })
    .reached_at(-1, 2);
    let mut boundaries = amount_in_guards();
    boundaries.extend([product_fits, amount_in_fits()]);
    let mut targets = get_amount_in_targets();
    targets.push(("Ri * x = floor((2^64 - 1) / D) - 1, the largest product below the bound", |[x, ri, ..]| {
        wide(ri) * wide(x) == i128::from(U64_MAX_OVER_D.0) - 1
    }));
    let inputs = Inputs { fees: FEE_LAST, directed, targets, boundaries };
    check("get_amount_in_matches_v2_formula", &inputs, |c, [x, ri, ro, f]| {
        assume!(x > 0 && ri > 0 && ro > 0 && x < ro && f < 10_000);
        assume!(c.widen(ri)? * c.widen(x)? <= U64_MAX_OVER_D);
        assume!({
            let num = c.widen(ri)? * c.widen(x)? * D;
            let q = c.widen(sub32(ro, x))? * c.widen(sub32(10_000, f))?;
            div(num, q)? <= MAX_Q
        });
        let num = c.widen(ri)? * c.widen(x)? * D;
        let q = c.widen(sub32(ro, x))? * c.widen(sub32(10_000, f))?;
        let i = c.get_amount_in(x, ri, ro, f)?;
        claim!(c.widen(i)? == div(num, q)? + Wrapping(1));
        Ok(Verdict::Holds)
    });
}

#[test]
fn quote_is_exact() {
    /// `a * Rb` at `Ra * 2^32 - 1` and `Ra * 2^32`, near and exactly.
    fn directed(rng: &mut Rng) -> Vec<[u32; 3]> {
        let (ra, a) = (positive(rng), positive(rng));
        let limit = u128::from(ra) << 32;
        let rb = i128::try_from((limit - 1) / u128::from(a)).expect("below 2^64");
        let mut inputs = near(rb).map(|rb| [a, ra, rb]).to_vec();
        for target in [limit - 1, limit] {
            if let Some((a, rb)) = factor(target) {
                inputs.push([a, ra, rb]);
            }
        }
        inputs
    }
    let boundaries: Vec<Boundary<3>> = vec![
        assumption("a > 0", |[a, ..]| Some(1 - wide(a))),
        assumption("ra > 0", |[_, ra, _]| Some(1 - wide(ra))),
        assumption("rb > 0", |[.., rb]| Some(1 - wide(rb))),
        assumption("a * Rb < Ra * 2^32", |[a, ra, rb]| Some(past_fit(a, rb, ra))),
    ];
    let targets: Vec<Target<3>> =
        vec![("a * Rb = Ra * 2^32 - 1, the largest product that fits", |[a, ra, rb]| past_fit(a, rb, ra) == 0)];
    let inputs = Inputs { fees: &[], directed, targets, boundaries };
    check("quote_is_exact", &inputs, |c, [a, ra, rb]| {
        assume!(a > 0 && ra > 0 && rb > 0);
        assume!(c.widen(a)? * c.widen(rb)? < (c.widen(ra)? << 32));
        let quoted = c.quote(a, ra, rb)?;
        claim!(c.widen(quoted)? == div(c.widen(a)? * c.widen(rb)?, c.widen(ra)?)?);
        Ok(Verdict::Holds)
    });
}

#[test]
fn mint_initial_is_floor_sqrt_minus_minimum() {
    /// Products at 1001^2 and around perfect squares, where the root steps.
    fn directed(rng: &mut Rng) -> Vec<[u32; 2]> {
        let mut inputs = vec![[M, M], [1001, 1001], [1000, 1002], [1, 1_002_001], [1, 1_002_000], [7, 143_143]];
        let root = if rng.below(4) == 0 { 1001 } else { wide(positive(rng)) };
        let a0 = positive(rng);
        for target in [root * root - 1, root * root, root * root + 2 * root, root * root + 2 * root + 1] {
            for a1 in near((target + wide(a0) - 1) / wide(a0)) {
                inputs.push([a0, a1]);
            }
        }
        let s = rng.range_u32(1001, M - 1);
        inputs.extend([[s, s], [s - 1, s + 1]]);
        inputs
    }
    let targets: Vec<Target<2>> = vec![
        ("a0 * a1 = 1001^2, the first product that mints", |[a0, a1]| wide(a0) * wide(a1) == 1_002_001),
        ("a product one below a perfect square, where the root steps", |[a0, a1]| {
            let next = u64::from(a0) * u64::from(a1) + 1;
            next.isqrt().pow(2) == next
        }),
    ];
    let inputs = Inputs {
        fees: &[],
        directed,
        targets,
        boundaries: vec![assumption("a0 * a1 >= 1001^2", |[a0, a1]| Some(1_002_001 - wide(a0) * wide(a1)))],
    };
    check("mint_initial_is_floor_sqrt_minus_minimum", &inputs, |c, [a0, a1]| {
        assume!(c.widen(a0)? * c.widen(a1)? >= Wrapping(1_002_001));
        let n = c.widen(a0)? * c.widen(a1)?;
        let (minted, locked) = (c.mint_initial(a0, a1)?, c.minimum_liquidity()?);
        let root = c.widen(minted)? + c.widen(locked)?;
        claim!(root * root <= n && n - root * root <= Wrapping(2) * root);
        Ok(Verdict::Holds)
    });
}

#[test]
fn mint_initial_is_exact_on_equal_amounts() {
    fn directed(rng: &mut Rng) -> Vec<[u32; 1]> {
        [999, 1000, 1001, 1002, 65_535, 65_536, 65_537, M - 1, M, rng.next_u32()].map(|x| [x]).to_vec()
    }
    let targets: Vec<Target<1>> =
        vec![("x = 1001, the first amount the claim covers", |[x]| x == 1001), ("x = 2^32 - 1", |[x]| x == M)];
    let boundaries = vec![assumption("x > 1000", |[x]| Some(1001 - wide(x)))];
    let inputs = Inputs { fees: &[], directed, targets, boundaries };
    check("mint_initial_is_exact_on_equal_amounts", &inputs, |c, [x]| {
        assume!(x > 1000);
        claim!(c.mint_initial(x, x)? == sub32(x, c.minimum_liquidity()?));
        Ok(Verdict::Holds)
    });
}

/// Directed mint_proportional inputs: each share at its fit boundary (a
/// quotient of `2^32 - 1` against `2^32`), near and exactly, in both orders;
/// each share at zero against one; and tied shares.
fn mint_proportional_directed(rng: &mut Rng) -> Vec<[u32; 5]> {
    let mut inputs = Vec::new();
    let (r0, r1, s) = (positive(rng), positive(rng), positive(rng));
    let fit0 = ((wide(r0) << 32) - 1) / wide(s);
    let fit1 = ((wide(r1) << 32) - 1) / wide(s);
    for a0 in near(fit0) {
        for a1 in near(fit1 + 1) {
            inputs.push([a0, a1, r0, r1, s]);
            inputs.push([a1, a0, r1, r0, s]);
        }
        for a1 in near(fit1) {
            inputs.push([a0, a1, r0, r1, s]);
        }
    }
    let (r0, r1) = (rng.range_u32(2, 1 << 16), rng.range_u32(2, 1 << 16));
    for n0 in [(u128::from(r0) << 32) - 1, u128::from(r0) << 32] {
        if let Some((s, a0)) = factor(n0) {
            inputs.push([a0, r1.saturating_mul(5), r0, r1, s]);
            inputs.push([r1.saturating_mul(5), a0, r1, r0, s]);
        }
    }
    inputs.extend([[r0, r1, r0, r1, 1], [r0 - 1, r1, r0, r1, 1], [r0, r1 - 1, r0, r1, 1]]);
    let a = positive(rng);
    inputs.extend([[a, a, r0, r0, s], [a, a.saturating_add(1), r0, r0, s], [M, 1, 1, 1, M], [1, M, 1, 1, M]]);
    inputs
}

/// The guards on the reserves, where each share stops fitting, as `fit`
/// makes it (an assumption of mint_proportional_is_min_share, a branch of
/// mint_proportional_one_share_fits), and where each share becomes positive.
/// The guard `s > 0` is each mirror's own.
fn mint_proportional_boundaries(fit: fn(&'static str, Distance<5>) -> Boundary<5>) -> Vec<Boundary<5>> {
    vec![
        assumption("r0 > 0", |[_, _, r0, _, _]| Some(1 - wide(r0))),
        assumption("r1 > 0", |[.., r1, _]| Some(1 - wide(r1))),
        fit("a0 * S < R0 * 2^32", |[a0, _, r0, _, s]| Some(past_fit(a0, s, r0))),
        fit("a1 * S < R1 * 2^32", |[_, a1, _, r1, s]| Some(past_fit(a1, s, r1))),
        assumption("a0 * S >= R0", |[a0, _, r0, _, s]| Some(wide(r0) - wide(a0) * wide(s))),
        assumption("a1 * S >= R1", |[_, a1, _, r1, s]| Some(wide(r1) - wide(a1) * wide(s))),
    ]
}

#[test]
fn mint_proportional_is_min_share() {
    let mut boundaries = mint_proportional_boundaries(assumption);
    boundaries.push(assumption("s > 0", |[.., s]| Some(1 - wide(s))));
    let targets: Vec<Target<5>> = vec![
        ("a share of 2^32 - 1, the largest that fits", |[a0, a1, r0, r1, s]| {
            share(a0, s, r0) == Some(wide(M)) || share(a1, s, r1) == Some(wide(M))
        }),
        ("a share of 1, at a0 * S = R0", |[a0, _, r0, _, s]| wide(a0) * wide(s) == wide(r0)),
    ];
    let inputs = Inputs { fees: &[], directed: mint_proportional_directed, targets, boundaries };
    check("mint_proportional_is_min_share", &inputs, |c, [a0, a1, r0, r1, s]| {
        assume!(r0 > 0 && r1 > 0 && s > 0);
        assume!(c.widen(a0)? * c.widen(s)? < (c.widen(r0)? << 32) && c.widen(a1)? * c.widen(s)? < (c.widen(r1)? << 32));
        assume!(c.widen(a0)? * c.widen(s)? >= c.widen(r0)? && c.widen(a1)? * c.widen(s)? >= c.widen(r1)?);
        let share0 = div(c.widen(a0)? * c.widen(s)?, c.widen(r0)?)?;
        let share1 = div(c.widen(a1)? * c.widen(s)?, c.widen(r1)?)?;
        let minted = c.mint_proportional(a0, a1, r0, r1, s)?;
        let liquidity = c.widen(minted)?;
        claim!(liquidity <= share0 && liquidity <= share1);
        claim!(liquidity == share0 || liquidity == share1);
        Ok(Verdict::Holds)
    });
}

/// At `S = 1` both shares fit (`a * S < 2^32 <= R * 2^32`), so no input
/// there reaches the claim, and `s > 0` is reached at `S = 2`.
#[test]
fn mint_proportional_one_share_fits() {
    let mut boundaries = mint_proportional_boundaries(branch);
    boundaries.push(assumption::<5>("s > 0", |[.., s]| Some(1 - wide(s))).reached_at(-1, 1));
    let targets: Vec<Target<5>> = vec![
        ("a share of 2^32 - 1, the largest that fits", |[a0, a1, r0, r1, s]| {
            share(a0, s, r0) == Some(wide(M)) || share(a1, s, r1) == Some(wide(M))
        }),
        ("a share of 2^32, the first that does not fit", |[a0, a1, r0, r1, s]| {
            share(a0, s, r0) == Some(wide(M) + 1) || share(a1, s, r1) == Some(wide(M) + 1)
        }),
    ];
    let inputs = Inputs { fees: &[], directed: mint_proportional_directed, targets, boundaries };
    check("mint_proportional_one_share_fits", &inputs, |c, [a0, a1, r0, r1, s]| {
        assume!(r0 > 0 && r1 > 0 && s > 0);
        let fits0 = c.widen(a0)? * c.widen(s)? < (c.widen(r0)? << 32);
        assume!(fits0 != (c.widen(a1)? * c.widen(s)? < (c.widen(r1)? << 32)));
        assume!(c.widen(a0)? * c.widen(s)? >= c.widen(r0)? && c.widen(a1)? * c.widen(s)? >= c.widen(r1)?);
        let minted = c.mint_proportional(a0, a1, r0, r1, s)?;
        let liquidity = c.widen(minted)?;
        if fits0 {
            claim!(liquidity == div(c.widen(a0)? * c.widen(s)?, c.widen(r0)?)?);
        } else {
            claim!(liquidity == div(c.widen(a1)? * c.widen(s)?, c.widen(r1)?)?);
        }
        Ok(Verdict::Holds)
    });
}

#[test]
fn burn_share_is_exact() {
    /// `liquidity * balance` at `total_supply` and one either side, and whole
    /// or excess liquidity.
    fn directed(rng: &mut Rng) -> Vec<[u32; 3]> {
        let s = positive(rng);
        let l = rng.range_u32(1, s);
        let mut inputs = near(i128::from(u64::from(s).div_ceil(u64::from(l)))).map(|b| [l, b, s]).to_vec();
        let c = rng.range_u32(2, 1 << 20);
        for l in [1, 2] {
            inputs.extend([[l, c, l * c], [l, c - 1, l * c], [l, c + 1, l * c]]);
        }
        inputs.extend([[s, M, s], [s, 1, s], [s.saturating_add(1), 1, s], [M, M, M], [M - 1, M, M]]);
        inputs
    }
    let boundaries: Vec<Boundary<3>> = vec![
        assumption("s > 0", |[.., s]| Some(1 - wide(s))),
        assumption("l <= s", |[l, _, s]| Some(wide(l) - wide(s))),
        assumption("l * B >= S", |[l, b, s]| Some(wide(s) - wide(l) * wide(b))),
    ];
    let targets: Vec<Target<3>> = vec![
        ("l * B = S, the smallest product that burns a unit", |[l, b, s]| wide(l) * wide(b) == wide(s)),
        ("l = S, the whole supply", |[l, _, s]| l == s),
    ];
    let inputs = Inputs { fees: &[], directed, targets, boundaries };
    check("burn_share_is_exact", &inputs, |c, [l, b, s]| {
        assume!(s > 0 && l <= s && c.widen(l)? * c.widen(b)? >= c.widen(s)?);
        let out = c.burn_share(l, b, s)?;
        claim!(c.widen(out)? == div(c.widen(l)? * c.widen(b)?, c.widen(s)?)?);
        claim!(out <= b);
        Ok(Verdict::Holds)
    });
}

#[test]
fn burn_share_of_whole_supply_is_balance() {
    fn directed(rng: &mut Rng) -> Vec<[u32; 2]> {
        vec![[0, 0], [1, 0], [0, 1], [1, 1], [M, 1], [1, M], [M, M], [positive(rng), positive(rng)]]
    }
    let targets: Vec<Target<2>> = vec![("a balance and a supply of 2^32 - 1", |[b, s]| b == M && s == M)];
    let boundaries = vec![
        assumption("s > 0", |[_, s]| Some(1 - wide(s))),
        assumption("b > 0", |[b, _]| Some(1 - wide(b))),
    ];
    let inputs = Inputs { fees: &[], directed, targets, boundaries };
    check("burn_share_of_whole_supply_is_balance", &inputs, |c, [b, s]| {
        assume!(s > 0 && b > 0);
        claim!(c.burn_share(s, b, s)? == b);
        Ok(Verdict::Holds)
    });
}

/// The largest `Ri` at which the k check holds, `floor(a * (Ro - o) * G / (o * D))`.
fn largest_reserve_in(a: u32, o: u32, ro: u32, f: u32) -> Option<i128> {
    (o > 0 && o < ro && f < 10_000).then(|| wide(a) * wide(ro - o) * (DI - wide(f)) / (wide(o) * DI))
}

/// Where the fee-adjusted k check turns from true to false as `Ri` grows.
fn fee_k_check_flips() -> Boundary<5> {
    branch("a * (Ro - o) * G >= Ri * o * D", |[a, o, ri, ro, f]| {
        largest_reserve_in(a, o, ro, f).map(|largest| wide(ri) - largest)
    })
}

/// Directed k-check inputs, the fee last: `Ri` at the largest value at which
/// the check holds and around it, the output get_amount_out quotes and around
/// it, and outputs at the reserve.
fn k_directed(rng: &mut Rng) -> Vec<[u32; 5]> {
    let mut inputs = Vec::new();
    let o = rng.range_u32(1, 1 << 20);
    let ro = o.saturating_add(positive(rng));
    let a = positive(rng);
    let f = accepted_fee(rng);
    for fee in [0, f] {
        if let Some(largest) = largest_reserve_in(a, o, ro, fee) {
            for ri in near(largest) {
                inputs.push([a, o, ri, ro, fee]);
            }
        }
    }
    let (a, ri, ro, f) = (positive(rng), positive(rng), positive(rng), accepted_fee(rng));
    if let Some(quoted) = amount_out([a, ri, ro, f]) {
        for o in near(quoted) {
            inputs.push([a, o, ri, ro, f]);
        }
    }
    let ro = rng.range_u32(2, M);
    inputs.extend([[a, ro - 1, positive(rng), ro, 9999], [a, 0, positive(rng), ro, f], [a, ro, positive(rng), ro, f]]);
    inputs
}

/// Where [`k_directed`] places its inputs that reach the claims.
fn k_targets() -> Vec<Target<5>> {
    vec![
        ("Ri at the largest the check holds for", |[a, o, ri, ro, f]| {
            largest_reserve_in(a, o, ro, f) == Some(wide(ri))
        }),
        ("the output get_amount_out quotes", |[a, o, ri, ro, f]| amount_out([a, ri, ro, f]) == Some(wide(o))),
        ("an output of reserve_out - 1 at fee 9999", |[_, o, _, ro, f]| f == 9999 && wide(o) == wide(ro) - 1),
    ]
}

#[test]
fn k_holds_is_exact() {
    fn directed(rng: &mut Rng) -> Vec<[u32; 4]> {
        k_directed(rng).into_iter().map(|[a, o, ri, ro, _]| [a, o, ri, ro]).collect()
    }
    let boundaries = vec![
        assumption("ri > 0", |[_, _, ri, _]| Some(1 - wide(ri))),
        assumption("o < ro", |[_, o, _, ro]| Some(wide(o) - wide(ro) + 1)),
        branch("a * (Ro - o) >= Ri * o", |[a, o, ri, ro]| {
            largest_reserve_in(a, o, ro, 0).map(|largest| wide(ri) - largest)
        }),
    ];
    let targets: Vec<Target<4>> = vec![
        ("Ri at the largest the check holds for", |[a, o, ri, ro]| {
            largest_reserve_in(a, o, ro, 0) == Some(wide(ri))
        }),
        ("an output of reserve_out - 1", |[_, o, _, ro]| wide(o) == wide(ro) - 1),
    ];
    let inputs = Inputs { fees: &[], directed, targets, boundaries };
    check("k_holds_is_exact", &inputs, |c, [a, o, ri, ro]| {
        assume!(ri > 0 && o < ro);
        claim!(c.k_holds(a, o, ri, ro)? == (c.widen(a)? * c.widen(sub32(ro, o))? >= c.widen(ri)? * c.widen(o)?));
        Ok(Verdict::Holds)
    });
}

#[test]
fn k_holds_with_fee_is_exact() {
    let mut boundaries = k_guards();
    boundaries.push(fee_k_check_flips());
    let inputs = Inputs { fees: &[4], directed: k_directed, targets: k_targets(), boundaries };
    check("k_holds_with_fee_is_exact", &inputs, |c, [a, o, ri, ro, f]| {
        assume!(f < 10_000 && ri > 0 && o < ro);
        let lhs = c.widen(a)? * c.widen(sub32(ro, o))?;
        let rhs = c.widen(ri)? * c.widen(o)?;
        let g = c.widen(sub32(10_000, f))?;
        let u = div(lhs, D)?;
        let v = rem(lhs, D)?;
        let floor_lhs_g_d = u * g + div(v * g, D)?;
        claim!(c.k_holds_with_fee(a, o, ri, ro, f)? == (floor_lhs_g_d >= rhs));
        Ok(Verdict::Holds)
    });
}

#[test]
fn k_holds_with_fee_matches_v2_formula() {
    /// `Ri * Ro` on both sides of `floor((2^64 - 1) / D)`, and `adj * (Ro - o)`
    /// at `2^64 - 1`: at fee 9999, `adj = Ri * D + a`, so each unit of `a` is
    /// one unit of `adj`.
    fn directed(rng: &mut Rng) -> Vec<[u32; 5]> {
        let mut inputs = k_directed(rng);
        for (p, q) in ENVELOPE_PAIRS {
            let (a, f) = (rng.range_u32(0, 1 << 20), accepted_fee(rng));
            let o = rng.range_u32(0, q.min(1_000_000) - 1);
            inputs.extend([[a, o, p, q, f], [a, o % p, q, p, f]]);
        }
        let span = rng.range_u32(1 << 31, M);
        let limit = u64::MAX / u64::from(span);
        let ri = u32::try_from((limit - u64::from(M)).div_ceil(10_000)).expect("small") + rng.range_u32(0, 3);
        for a in near(i128::from(limit) - wide(ri) * DI) {
            inputs.push([a, 0, ri, span, 9999]);
        }
        inputs
    }
    fn past_adjusted([a, o, ri, ro, f]: [u32; 5]) -> Option<i128> {
        let adj = (wide(ri) + wide(a)) * DI - wide(a) * wide(f);
        (o < ro && f < 10_000).then(|| adj - i128::from(u64::MAX / u64::from(ro - o)))
    }
    let mut boundaries = k_guards();
    boundaries.extend([
        assumption("Ri * Ro <= floor((2^64 - 1) / D)", |[_, _, ri, ro, _]| {
            Some(wide(ri) * wide(ro) - i128::from(U64_MAX_OVER_D.0))
        })
        .reached_at(-1, 2),
        assumption("adj <= floor((2^64 - 1) / (Ro - o))", past_adjusted),
        fee_k_check_flips(),
    ]);
    let mut targets = k_targets();
    targets.push(("adj * (Ro - o) at 2^64 - 1", |args| past_adjusted(args) == Some(0)));
    let inputs = Inputs { fees: &[4], directed, targets, boundaries };
    check("k_holds_with_fee_matches_v2_formula", &inputs, |c, [a, o, ri, ro, f]| {
        assume!(f < 10_000 && ri > 0 && o < ro);
        assume!({
            let adj = (c.widen(ri)? + c.widen(a)?) * D - c.widen(a)? * c.widen(f)?;
            adj <= div(U64_MAX, c.widen(sub32(ro, o))?)? && c.widen(ri)? * c.widen(ro)? <= U64_MAX_OVER_D
        });
        let adj = (c.widen(ri)? + c.widen(a)?) * D - c.widen(a)? * c.widen(f)?;
        let v2_holds = adj * c.widen(sub32(ro, o))? >= c.widen(ri)? * c.widen(ro)? * D;
        claim!(c.k_holds_with_fee(a, o, ri, ro, f)? == v2_holds);
        Ok(Verdict::Holds)
    });
}

#[test]
fn get_amount_out_passes_both_k_checks() {
    fn directed(rng: &mut Rng) -> Vec<[u32; 4]> {
        let mut inputs = get_amount_out_directed(rng);
        inputs.extend([[M, 1, M, 0], [M, M, M, 9999], [1, M, M, 0]]);
        inputs
    }
    let (targets, boundaries) = (get_amount_out_targets(), amount_out_guards());
    let inputs = Inputs { fees: FEE_LAST, directed, targets, boundaries };
    check("get_amount_out_passes_both_k_checks", &inputs, |c, [a, ri, ro, f]| {
        assume!(a > 0 && ri > 0 && ro > 0 && f < 10_000);
        let o = c.get_amount_out(a, ri, ro, f)?;
        claim!(c.k_holds(a, o, ri, ro)?);
        claim!(c.k_holds_with_fee(a, o, ri, ro, f)?);
        Ok(Verdict::Holds)
    });
}

#[test]
fn k_checks_refuse_unpaid_withdrawal() {
    fn directed(rng: &mut Rng) -> Vec<[u32; 4]> {
        let ro = rng.range_u32(2, M);
        vec![
            [1, positive(rng), ro, accepted_fee(rng)],
            [ro - 1, positive(rng), ro, accepted_fee(rng)],
            [1, 1, M, 9999],
            [M - 1, M, M, 0],
            [0, 1, ro, 0],
            [ro, 1, ro, 0],
        ]
    }
    let targets: Vec<Target<4>> =
        vec![("o = reserve_out - 1, the largest output below it", |[o, _, ro, _]| wide(o) == wide(ro) - 1)];
    let boundaries: Vec<Boundary<4>> = vec![
        assumption("ri > 0", |[_, ri, ..]| Some(1 - wide(ri))),
        assumption("o > 0", |[o, ..]| Some(1 - wide(o))),
        assumption("o < ro", |[o, _, ro, _]| Some(wide(o) - wide(ro) + 1)),
        assumption("f < 10000", |[.., f]| Some(wide(f) - 9999)),
    ];
    let inputs = Inputs { fees: FEE_LAST, directed, targets, boundaries };
    check("k_checks_refuse_unpaid_withdrawal", &inputs, |c, [o, ri, ro, f]| {
        assume!(ri > 0 && o > 0 && o < ro && f < 10_000);
        claim!(!c.k_holds(0, o, ri, ro)?);
        claim!(!c.k_holds_with_fee(0, o, ri, ro, f)?);
        Ok(Verdict::Holds)
    });
}

/// Directed mul_div inputs: `a * b` near and exactly at `d * 2^32 - d`,
/// `d * 2^32 - d + 1`, `d * 2^32 - 1` and `d * 2^32`, where the ceiling and
/// then the floor stop fitting.
fn product_directed(rng: &mut Rng) -> Vec<[u32; 3]> {
    let mut inputs = vec![[7, 1_227_133_512, 2], [7, 1_227_133_513, 2], [7, 1_227_133_514, 2]];
    let a = positive(rng);
    for d in [positive(rng), rng.range_u32(2, 1 << 20)] {
        let top = u128::from(d) << 32;
        for target in [top - u128::from(d), top - u128::from(d) + 1, top - 1, top] {
            for b in near(i128::try_from(target / u128::from(a)).expect("below 2^64")) {
                inputs.push([a, b, d]);
            }
            if let Some((a, b)) = factor(target) {
                inputs.push([a, b, d]);
            }
        }
    }
    inputs
}

#[test]
fn mul_div_is_floor() {
    let mut boundaries = product_guards();
    boundaries.push(assumption("a * b < d * 2^32", |[a, b, d]| Some(past_fit(a, b, d))));
    let targets: Vec<Target<3>> =
        vec![("a * b = d * 2^32 - 1, the largest whose floor fits", |[a, b, d]| past_fit(a, b, d) == 0)];
    let inputs = Inputs { fees: &[], directed: product_directed, targets, boundaries };
    check("mul_div_is_floor", &inputs, |c, [a, b, d]| {
        assume!(d > 0 && c.widen(a)? * c.widen(b)? < (c.widen(d)? << 32));
        let quotient = c.mul_div(a, b, d)?;
        claim!(c.widen(quotient)? == div(c.widen(a)? * c.widen(b)?, c.widen(d)?)?);
        Ok(Verdict::Holds)
    });
}

#[test]
fn mul_div_up_is_ceiling() {
    let mut boundaries = product_guards();
    boundaries.push(assumption("a * b + d - 1 < d * 2^32", |[a, b, d]| {
        Some(wide(a) * wide(b) + wide(d) - (wide(d) << 32))
    }));
    let targets: Vec<Target<3>> = vec![("a * b = d * 2^32 - d, the largest whose ceiling fits", |[a, b, d]| {
        wide(a) * wide(b) == (wide(d) << 32) - wide(d)
    })];
    let inputs = Inputs { fees: &[], directed: product_directed, targets, boundaries };
    check("mul_div_up_is_ceiling", &inputs, |c, [a, b, d]| {
        assume!(d > 0 && c.widen(a)? * c.widen(b)? + c.widen(d)? - Wrapping(1) < (c.widen(d)? << 32));
        let expected = div(c.widen(a)? * c.widen(b)? + c.widen(d)? - Wrapping(1), c.widen(d)?)?;
        let quotient = c.mul_div_up(a, b, d)?;
        claim!(c.widen(quotient)? == expected);
        Ok(Verdict::Holds)
    });
}

#[test]
fn v2_reference_vectors() {
    check_fixed("v2_reference_vectors", |c| {
        claim!(c.minimum_liquidity()? == 1000);
        claim!(c.get_amount_out(1000, 100_000, 100_000, 30)? == 987);
        claim!(c.get_amount_in(987, 100_000, 100_000, 30)? == 1000);
        claim!(c.get_amount_out(1, 1_000_000, 1000, 30)? == 0);
        claim!(c.mint_initial(1_000_000, 4_000_000)? == 1_999_000);
        claim!(c.quote(100, 1000, 3000)? == 300);
        claim!(c.mul_div(M, M, M)? == M);
        claim!(c.mul_div_up(M, M, M)? == M);
        Ok(Verdict::Holds)
    });
}

// ---- spec WideArith -------------------------------------------------------

#[test]
fn widen_is_zero_at_zero() {
    check_fixed("widen_is_zero_at_zero", |c| {
        claim!(c.widen(0)? == Wrapping(0));
        Ok(Verdict::Holds)
    });
}

/// Single inputs at the bit and half-word boundaries and at the top of `u32`.
fn u32_directed(rng: &mut Rng) -> Vec<[u32; 1]> {
    [0, 1, 2, 65_535, 65_536, (1 << 31) - 1, 1 << 31, M - 3, M - 2, M - 1, M, rng.next_u32()].map(|x| [x]).to_vec()
}

/// Where [`u32_directed`] places its inputs.
fn u32_targets() -> Vec<Target<1>> {
    vec![("x = 2^32 - 2, the last below the top", |[x]| x == M - 1), ("x = 2^32 - 3", |[x]| x == M - 2)]
}

#[test]
fn widen_is_one_more_at_x_plus_one() {
    let boundaries = vec![assumption("x < 4294967295", |[x]| Some(wide(x) - (wide(M) - 1)))];
    let inputs = Inputs { fees: &[], directed: u32_directed, targets: u32_targets(), boundaries };
    check("widen_is_one_more_at_x_plus_one", &inputs, |c, [x]| {
        assume!(x < 4_294_967_295);
        claim!(c.widen(add32(x, 1))? == c.widen(x)? + Wrapping(1));
        Ok(Verdict::Holds)
    });
}

#[test]
fn widen_is_at_most_u32_max() {
    let inputs = Inputs { fees: &[], directed: u32_directed, targets: u32_targets(), boundaries: vec![] };
    check("widen_is_at_most_u32_max", &inputs, |c, [x]| {
        claim!(c.widen(x)? <= Wrapping(4_294_967_295));
        Ok(Verdict::Holds)
    });
}

#[test]
fn mul32_is_exact() {
    /// Pairs at mul32's 16-bit column boundaries.
    fn directed(rng: &mut Rng) -> Vec<[u32; 2]> {
        let edges = [0, 1, 0xffff, 0x1_0000, 0x1_0001, 0x1_ffff, 0x7fff_ffff, 0x8000_0000, 0xffff_0000, 0xfffe_ffff, M];
        vec![[rng.pick(&edges), rng.pick(&edges)], [rng.next_u32(), M], [M, rng.next_u32()]]
    }
    let targets: Vec<Target<2>> = vec![("a factor of 2^32 - 1", |[a, b]| a == M || b == M)];
    let inputs = Inputs { fees: &[], directed, targets, boundaries: vec![] };
    check("mul32_is_exact", &inputs, |c, [a, b]| {
        let (hi, lo) = (c.mul_hi(a, b)?, c.mul_lo(a, b)?);
        let product = (c.widen(hi)? << 32) | c.widen(lo)?;
        claim!(product == c.widen(a)? * c.widen(b)?);
        Ok(Verdict::Holds)
    });
}

#[test]
fn mul_by_is_exact_on_64_bits() {
    fn directed(rng: &mut Rng) -> Vec<[u32; 3]> {
        vec![[M, M, M], [M, 0, M], [0, M, M], [M, M, 2], [1 << 31, 0, 2], [rng.next_u32(), rng.next_u32(), M]]
    }
    let targets: Vec<Target<3>> = vec![("hi, lo and m all 2^32 - 1", |args| args == [M, M, M])];
    let inputs = Inputs { fees: &[], directed, targets, boundaries: vec![] };
    check("mul_by_is_exact_on_64_bits", &inputs, |c, [hi, lo, m]| {
        let n = (c.widen(hi)? << 32) | c.widen(lo)?;
        let (l0, l1, l2) = (c.mul_by_limb(hi, lo, m, 0)?, c.mul_by_limb(hi, lo, m, 1)?, c.mul_by_limb(hi, lo, m, 2)?);
        let (l0, l1, l2) = (c.widen(l0)?, c.widen(l1)?, c.widen(l2)?);
        claim!(((l1 << 32) | l0) == n * c.widen(m)?);
        claim!(l2 == (c.widen(hi)? * c.widen(m)? + ((c.widen(lo)? * c.widen(m)?) >> 32)) >> 32);
        claim!(c.mul_by_limb(hi, lo, m, 3)? == 0);
        Ok(Verdict::Holds)
    });
}

#[test]
fn div64_is_floor() {
    fn directed(rng: &mut Rng) -> Vec<[u32; 3]> {
        let (d, lo) = (positive(rng), rng.next_u32());
        vec![[d - 1, lo, d], [d - 1, M, d], [d, lo, d], [0, lo, d], [0, M, 1], [M - 1, M, M], [M, M, M], [0, 0, 0]]
    }
    let targets: Vec<Target<3>> =
        vec![("hi = d - 1, the largest high word below d", |[hi, _, d]| wide(hi) == wide(d) - 1)];
    let boundaries = vec![assumption("hi < d", |[hi, _, d]| Some(wide(hi) - wide(d) + 1))];
    let inputs = Inputs { fees: &[], directed, targets, boundaries };
    check("div64_is_floor", &inputs, |c, [hi, lo, d]| {
        assume!(hi < d);
        let n = (c.widen(hi)? << 32) | c.widen(lo)?;
        let quotient = c.div64(hi, lo, d)?;
        let qd = c.widen(quotient)? * c.widen(d)?;
        claim!(qd <= n && n - qd < c.widen(d)?);
        Ok(Verdict::Holds)
    });
}

#[test]
fn sqrt64_is_floor() {
    /// Perfect squares, one either side, and the last value with the same root.
    fn directed(rng: &mut Rng) -> Vec<[u32; 2]> {
        let s = u128::from(rng.next_u32());
        let mut inputs = vec![[M, M], [0, 0]];
        for n in [s * s, (s * s).saturating_sub(1), s * s + 1, s * s + 2 * s, s * s + 2 * s + 1] {
            if let Ok(n) = u64::try_from(n) {
                inputs.push([n >> 32, n & u64::from(M)].map(|half| u32::try_from(half).expect("a half")));
            }
        }
        inputs
    }
    /// The distance to the nearest perfect square, where the root steps.
    fn past_square([hi, lo]: [u32; 2]) -> Option<i128> {
        let n = i128::from((u64::from(hi) << 32) | u64::from(lo));
        let below = i128::from(u64::try_from(n).expect("a u64").isqrt());
        let (low, high) = (below * below, (below + 1) * (below + 1));
        Some(if n - low <= high - n { n - low } else { n - high })
    }
    let targets: Vec<Target<2>> = vec![
        ("a perfect square", |args| past_square(args) == Some(0)),
        ("one above a perfect square", |args| past_square(args) == Some(1)),
    ];
    let inputs = Inputs { fees: &[], directed, targets, boundaries: vec![branch("a perfect square", past_square)] };
    check("sqrt64_is_floor", &inputs, |c, [hi, lo]| {
        let n = (c.widen(hi)? << 32) | c.widen(lo)?;
        let root = c.sqrt64(hi, lo)?;
        let s = c.widen(root)?;
        claim!(s * s <= n && n - s * s <= Wrapping(2) * s);
        Ok(Verdict::Holds)
    });
}

// ---- the runner's own checks ----------------------------------------------

fn one_input(_: &mut Rng) -> Vec<[u32; 1]> {
    vec![[1]]
}

#[test]
#[should_panic(expected = "counterexamples")]
fn runner_reports_a_false_claim() {
    let inputs = Inputs { fees: &[], directed: one_input, targets: vec![], boundaries: vec![] };
    check("runner_reports_a_false_claim", &inputs, |_, [x]| {
        claim!(x < 1000);
        Ok(Verdict::Holds)
    });
}

/// An assumption whose term does not denote filters nothing, so the false
/// claim after it is reached.
#[test]
#[should_panic(expected = "counterexamples")]
fn runner_lets_an_undefined_assumption_filter_nothing() {
    let inputs = Inputs { fees: &[], directed: one_input, targets: vec![], boundaries: vec![] };
    check("runner_lets_an_undefined_assumption_filter_nothing", &inputs, |_, [x]| {
        assume!(div(Wrapping(u64::from(x)), Wrapping(0))? == Wrapping(0));
        claim!(false);
        Ok(Verdict::Holds)
    });
}

#[test]
#[should_panic(expected = "no input that reaches the claim is at distance 0")]
fn runner_requires_both_sides_of_a_boundary() {
    let boundaries = vec![assumption("x <= 5", |[x]| Some(wide(x) - 5))];
    let inputs = Inputs { fees: &[], directed: one_input, targets: vec![], boundaries };
    check("runner_requires_both_sides_of_a_boundary", &inputs, |_, [x]| {
        assume!(x <= 1);
        Ok(Verdict::Holds)
    });
}

/// A boundary is reached at its own points, not merely near them: inputs one
/// inside and two past it do not do. Neither the edge values nor this test's
/// seeded draws come within two of 3000000000.
#[test]
#[should_panic(expected = "no input that reaches the claim is at distance 0")]
fn runner_requires_the_boundary_point_itself() {
    fn around(_: &mut Rng) -> Vec<[u32; 1]> {
        vec![[2_999_999_999], [3_000_000_002]]
    }
    let boundary = assumption("x <= 3000000000", |[x]| Some(wide(x) - 3_000_000_000));
    let inputs = Inputs { fees: &[], directed: around, targets: vec![], boundaries: vec![boundary] };
    check("runner_requires_the_boundary_point_itself", &inputs, |_, [x]| {
        assume!(x <= 3_000_000_000);
        Ok(Verdict::Holds)
    });
}

/// A target counts only where a directed input reaches the claim on it: the
/// edge grid holds `x = 2`, and that does not do.
#[test]
#[should_panic(expected = "no directed input reaches the claim at x = 2")]
fn runner_requires_the_directed_inputs_on_their_targets() {
    let inputs = Inputs { fees: &[], directed: one_input, targets: vec![("x = 2", |[x]| x == 2)], boundaries: vec![] };
    check("runner_requires_the_directed_inputs_on_their_targets", &inputs, |_, _| Ok(Verdict::Holds));
}

/// Nor does a directed input on the target that the assumptions filter out,
/// while another directed input gets past them.
#[test]
#[should_panic(expected = "no directed input reaches the claim at x = 2")]
fn runner_requires_the_directed_inputs_on_their_targets_to_reach_the_claim() {
    fn on_and_off_target(_: &mut Rng) -> Vec<[u32; 1]> {
        vec![[2], [7]]
    }
    let targets: Vec<Target<1>> = vec![("x = 2", |[x]| x == 2)];
    let inputs = Inputs { fees: &[], directed: on_and_off_target, targets, boundaries: vec![] };
    check("runner_requires_the_directed_inputs_on_their_targets_to_reach_the_claim", &inputs, |_, [x]| {
        assume!(x != 2);
        Ok(Verdict::Holds)
    });
}

// ---- the mirror list ------------------------------------------------------

/// Mirror functions, each with the pin ([`review_pin`]) of the spec function
/// body it was last reviewed against. A listed name without a function does
/// not compile.
macro_rules! mirrors {
    ($($name:ident => $pin:literal),* $(,)?) => {{
        $(let _: fn() = $name;)*
        vec![$((stringify!($name), $pin)),*]
    }};
}

/// The mirrors of the spec block `block`, in declaration order, with their
/// pins. After reviewing a mirror against an edited spec function, set its
/// pin to the one `every_mirror_follows_its_spec_function` reports.
fn mirror_list(block: &str) -> Vec<(&'static str, &'static str)> {
    match block {
        "PairMath" => mirrors![
            get_amount_out_returns_below_reserve => "5f9d06b977c5a8f1",
            get_amount_out_is_largest_feasible => "659372ffe0a4581a",
            get_amount_out_is_exact => "034c60e44bb8d521",
            get_amount_out_matches_v2_formula => "c2e83c7259030b9f",
            get_amount_out_never_falls_as_amount_in_rises => "8c5a00aceee28b7c",
            get_amount_out_never_rises_with_fee => "05ee81755bdb2994",
            get_amount_in_buys_the_target => "f9719d9aee14ad7e",
            get_amount_in_overpays_by_at_most_one => "20c0ba1b8a9445f6",
            get_amount_in_is_exact => "c893272ba51b23b8",
            get_amount_in_matches_v2_formula => "6d9edb101bbf16c6",
            quote_is_exact => "2e42cd6ca302d052",
            mint_initial_is_floor_sqrt_minus_minimum => "2381aacd3c9725fb",
            mint_initial_is_exact_on_equal_amounts => "a62d0c616bc5b64c",
            mint_proportional_is_min_share => "a1327f2c89b0bfbf",
            mint_proportional_one_share_fits => "1cb9a5beb3f36ced",
            burn_share_is_exact => "0b189ef4089213d3",
            burn_share_of_whole_supply_is_balance => "3afb1843f9686815",
            k_holds_is_exact => "3bb2118ac1d9d093",
            k_holds_with_fee_is_exact => "06ae0aee524d1e7e",
            k_holds_with_fee_matches_v2_formula => "0069327026c90614",
            get_amount_out_passes_both_k_checks => "bc4df4d49074219d",
            k_checks_refuse_unpaid_withdrawal => "c3ac1563826689fd",
            mul_div_is_floor => "cf6ae799c60406c6",
            mul_div_up_is_ceiling => "c615c54c1ab2afba",
            v2_reference_vectors => "43af83266dc0e94a",
        ],
        "WideArith" => mirrors![
            widen_is_zero_at_zero => "f496255c31481b6f",
            widen_is_one_more_at_x_plus_one => "5cf384a068a2ef5c",
            widen_is_at_most_u32_max => "f8b0ab28d48ae8d4",
            mul32_is_exact => "d41878331139062f",
            mul_by_is_exact_on_64_bits => "5c093a07727a4aad",
            div64_is_floor => "721d8e5acd7d84a7",
            sqrt64_is_floor => "3d853bef3708e90c",
        ],
        other => panic!("spec {other} has no mirrors here"),
    }
}

/// Every spec function has a mirror of its name, in declaration order, and
/// every mirror a spec function: a new, renamed or reordered spec function
/// (a parameterless `forall`, the one form `source::spec_functions` accepts)
/// fails here until its mirror follows. Each mirror is a plain `#[test]` of
/// this file: a mirror that lost the attribute would never run, and one with
/// another attribute (`#[ignore]`, `#[should_panic]`, a `#[cfg]`) would not
/// run or would pass by failing.
#[test]
fn every_spec_function_has_a_mirror() {
    let this_file = include_str!("specs.rs").replace("\r\n", "\n");
    for block in SPEC_BLOCKS {
        let declared: Vec<String> =
            spec_functions(&read_repo_text(block.file), block.name).into_iter().map(|f| f.name).collect();
        let mirrored: Vec<&str> = mirror_list(block.name).into_iter().map(|(name, _)| name).collect();
        assert_eq!(
            declared, mirrored,
            "the spec functions of `spec {}` in {}, and their mirrors here",
            block.name, block.file
        );
        for name in mirrored {
            let attributes = mirror_attributes(&this_file, name).unwrap_or_else(|message| panic!("{message}"));
            assert!(attributes.is_empty(), "the mirror `fn {name}` has attributes besides #[test]: {attributes:?}");
        }
    }
}

/// Each mirror was reviewed against its spec function as the function reads
/// now: the pin next to it in [`mirror_list`] is the pin of the function's
/// body. An edited claim, a dropped assumption, or any other change to what a
/// spec function states fails here until its mirror is reviewed against the
/// new body and the pin follows; a comment or the layout can change freely.
#[test]
fn every_mirror_follows_its_spec_function() {
    let mut changed = Vec::new();
    for block in SPEC_BLOCKS {
        let functions = spec_functions(&read_repo_text(block.file), block.name);
        for (function, (name, pin)) in functions.iter().zip(mirror_list(block.name)) {
            let now = review_pin(&function.body);
            if function.name != name {
                changed.push(format!("`fn {}` in {}: its mirror is listed as {name}", function.name, block.file));
            } else if now != pin {
                changed.push(format!("{name} in {}: pinned {pin}, now {now}", block.file));
            }
        }
    }
    assert!(
        changed.is_empty(),
        "spec functions changed since their mirrors were reviewed: review each mirror in tests/specs.rs \
         against its spec function, then update its pin in mirror_list:\n{}",
        changed.join("\n")
    );
}

/// The attributes a mirror has besides `#[test]`: every line that starts one
/// (`#[`, however indented) from the end of the item before the mirror to its
/// `#[test]` line, blank lines and comments included.
///
/// # Errors
///
/// When `fn <name>()` is not marked `#[test]` on the line above it.
fn mirror_attributes<'a>(file: &'a str, name: &str) -> Result<Vec<&'a str>, String> {
    let header = format!("\n#[test]\nfn {name}() {{\n");
    let at = file
        .find(&header)
        .ok_or_else(|| format!("the mirror `fn {name}` is not marked #[test] on the line above it, so it never runs"))?;
    let before = file[..at].lines().rev().take_while(|line| !ends_an_item(line));
    Ok(before.map(str::trim).filter(|line| line.starts_with("#[")).collect())
}

/// Whether `line` ends a top-level item: code at column 0 that ends in `}`
/// or `;`.
fn ends_an_item(line: &str) -> bool {
    let code = !line.starts_with(char::is_whitespace) && !line.starts_with("//");
    code && (line.ends_with('}') || line.ends_with(';'))
}

/// The check finds an attribute however the lines above the mirror are laid
/// out, and none of the item before it.
#[test]
fn mirror_check_finds_attributes_in_any_layout() {
    let layouts: [(&str, &[&str]); 6] = [
        ("}\n\n/// Doc.\n#[test]\nfn m() {\n", &[]),
        ("#[test]\nfn before() {\n}\n#[test]\nfn m() {\n", &[]),
        ("}\n\n    #[ignore]\n#[test]\nfn m() {\n", &["#[ignore]"]),
        ("}\n#[ignore]\n\n\n#[test]\nfn m() {\n", &["#[ignore]"]),
        ("};\n#[should_panic]\n/// A doc line that ends in a semicolon;\n#[test]\nfn m() {\n", &["#[should_panic]"]),
        ("}\n#[cfg(\n    any()\n)]\n#[test]\nfn m() {\n", &["#[cfg("]),
    ];
    for (file, expected) in layouts {
        assert_eq!(mirror_attributes(file, "m").as_deref(), Ok(expected), "{file:?}");
    }
    for file in ["}\nfn m() {\n", "}\n#[ignore] #[test]\nfn m() {\n", "}\n#[test]\n/// Doc.\nfn m() {\n"] {
        assert!(mirror_attributes(file, "m").is_err(), "{file:?}: a mirror not marked #[test] above it was accepted");
    }
}
