//! The fixed workload of the cost ledger: the calls `src/bin/cost.rs`
//! measures and `bench/history.jsonl` records.
//!
//! Every method has a "typical" case, taken from Uniswap V2's reference values
//! where one exists, and a "worst" case: the input with the most Wasm fuel
//! (`wasm_insns`, see [`crate::ledger`]) that a search of the edge grid, the
//! seeded differential inputs and a hill climb from the best of them found
//! under Inference v0.0.6. It is not a proven maximum. minimum_liquidity takes no arguments; its one call is
//! the baseline every other cost is measured against.
//!
//! A workload is the cases and how each is measured
//! ([`crate::ledger::MEASUREMENT`]), and a row of the ledger compares with
//! another only when both measured the same workload. [`WORKLOADS`] lists
//! every workload the ledger has measured, and [`CASES`] with the current
//! measurement is its last entry.

use crate::ledger::MEASUREMENT;
use crate::method::{Method, Value};

/// Every workload the ledger has measured, oldest first: its id and the
/// sha256 of its description ([`describe`]).
///
/// Append-only. A changed [`CASES`] or [`MEASUREMENT`] is a new workload, and
/// takes a new last entry with a new id; the rows already in
/// `bench/history.jsonl` keep naming theirs. The `cost` binary refuses to run
/// unless [`describe`] hashes to the last entry ([`check`]), and the tests
/// refuse a repeated id and a ledger row whose workload is not listed with
/// the hash it records.
pub const WORKLOADS: &[(&str, &str)] =
    &[("pair-math-1", "abbe4bcc41f9f24194c573e15b50565719d04a242e3c20004a16ccb772a3219b")];

/// The id of the current workload: [`CASES`], measured as [`MEASUREMENT`]
/// states.
pub const WORKLOAD_ID: &str = WORKLOADS[WORKLOADS.len() - 1].0;

/// The sha256 of [`describe`] that [`WORKLOAD_ID`] names.
pub const WORKLOAD_SHA256: &str = WORKLOADS[WORKLOADS.len() - 1].1;

/// One measured call, and the value it must return.
pub struct Case {
    pub method: Method,
    pub name: &'static str,
    pub args: &'static [u32],
    pub returns: Value,
}

const M: u32 = u32::MAX;

pub const CASES: &[Case] = &[
    case(Method::MinimumLiquidity, "typical", &[], Value::U32(1000)),
    case(Method::MinimumLiquidity, "worst", &[], Value::U32(1000)),
    case(Method::GetAmountOut, "typical", &[1000, 100_000, 100_000, 30], Value::U32(987)),
    case(Method::GetAmountOut, "worst", &[M, M, M, 0], Value::U32(2_147_483_647)),
    case(Method::GetAmountIn, "typical", &[987, 100_000, 100_000, 30], Value::U32(1000)),
    case(Method::GetAmountIn, "worst", &[774_720_652, 457_148_923, 2_738_052_370, 9580], Value::U32(M)),
    case(Method::Quote, "typical", &[100, 1000, 3000], Value::U32(300)),
    case(Method::Quote, "worst", &[4_068_246_060, 2_773_968_643, 2_928_560_472], Value::U32(M)),
    case(Method::MintInitial, "typical", &[1_000_000, 4_000_000], Value::U32(1_999_000)),
    case(Method::MintInitial, "worst", &[M - 1, M], Value::U32(4_294_966_294)),
    case(Method::MintProportional, "typical", &[1000, 4000, 1_000_000, 4_000_000, 2_000_000], Value::U32(2000)),
    case(Method::MintProportional, "worst", &[M, M, M, M, M], Value::U32(M)),
    case(Method::BurnShare, "typical", &[1000, 1_000_000, 2_000_000], Value::U32(500)),
    case(Method::BurnShare, "worst", &[M, M, M], Value::U32(M)),
    case(Method::KHolds, "typical", &[1000, 987, 100_000, 100_000], Value::Bool(true)),
    case(Method::KHolds, "worst", &[999, 0, M - 1, M - 1], Value::Bool(true)),
    case(Method::KHoldsWithFee, "typical", &[1000, 987, 100_000, 100_000, 30], Value::Bool(true)),
    case(Method::KHoldsWithFee, "worst", &[1, 0, M, M, 1], Value::Bool(true)),
    case(Method::MulDiv, "typical", &[1000, 2000, 3], Value::U32(666_666)),
    case(Method::MulDiv, "worst", &[3_227_299_360, 3_908_205_981, 2_936_681_421], Value::U32(M)),
    case(Method::MulDivUp, "typical", &[1000, 2000, 3], Value::U32(666_667)),
    case(Method::MulDivUp, "worst", &[M, M, M], Value::U32(M)),
];

const fn case(method: Method, name: &'static str, args: &'static [u32], returns: Value) -> Case {
    Case { method, name, args, returns }
}

/// The workload as text: [`MEASUREMENT`] on the first line, then one
/// `method case (args) -> value` line per case.
#[must_use]
pub fn describe() -> String {
    let cases = CASES.iter().map(|case| {
        let args: Vec<String> = case.args.iter().map(u32::to_string).collect();
        format!("{} {} ({}) -> {:?}\n", case.method.name(), case.name, args.join(", "), case.returns)
    });
    std::iter::once(format!("{MEASUREMENT}\n")).chain(cases).collect()
}

/// The sha256 of [`describe`].
#[must_use]
pub fn workload_sha256() -> String {
    crate::sha256_hex(describe().as_bytes())
}

/// Whether [`CASES`] and [`MEASUREMENT`] are the workload [`WORKLOAD_ID`]
/// names.
///
/// # Errors
///
/// When either changed and [`WORKLOADS`] did not follow; the message says
/// what to add.
pub fn check() -> Result<(), String> {
    let actual = workload_sha256();
    if actual == WORKLOAD_SHA256 {
        return Ok(());
    }
    Err(format!(
        "workload::CASES or ledger::MEASUREMENT changed (sha256 {actual}), but the workload id is still \
         {WORKLOAD_ID}, which names {WORKLOAD_SHA256}: append the new workload to workload::WORKLOADS under \
         a new id"
    ))
}
