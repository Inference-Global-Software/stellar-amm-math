//! The cost ledger's measurement: the workload ([`crate::workload`]) run on
//! `out/main.wasm` under the in-process Soroban host, as the JSON document the
//! `cost` binary prints, and the reader of the lock `bench/snapshot.sh` takes
//! the host version from.
//!
//! The document has sorted keys and integers only:
//!
//! ```text
//! {"code_size":..,"cost":{<method>:{<case>:{"delta_instructions":..,
//!  "instructions":..,"mem_bytes":..,"wasm_insns":..}}},"interface_sha256":..,
//!  "wasm_sha256":..,"workload":..,"workload_sha256":..}
//! ```
//!
//! [`MEASUREMENT`] states how each case is measured, and the workload's hash
//! covers it. Each case runs in an `Env` of its own that holds only this
//! contract (any other contract registered in an `Env` changes the costs
//! measured in it), right after a `minimum_liquidity()` call, the baseline.
//! `instructions` and `mem_bytes` are the host's `resources()` for the call,
//! and `delta_instructions` its instructions less the baseline's.
//! `wasm_insns` is not an instruction count: it is the wasmi fuel the call
//! consumed (the host's `WasmInsnExec` count), at the weights soroban-env-host
//! calibrates (`load_calibrated_fuel_costs`): 1 per instruction, 2 per load,
//! 1 per store, 3 per global access, 67 per call, and 1 per 8 locals
//! (parameters included) of each function called. A block, loop or `if` is
//! charged when it is entered, so an instruction a branch skips can count.
//! Calls are 30% to 49% of every case's fuel on this module but the
//! baseline's (its 77 is its one call and 10 more), so a toolchain that
//! inlines a helper lowers the fuel by about 67 per call it removes while
//! executing nearly the same instructions. The host meters deterministically,
//! so two runs print the same bytes.
//!
//! What the numbers carry, under the host this crate pins (a test below
//! checks it on every run):
//! - `mem_bytes` is host memory, the VM instance plus 32 bytes per argument,
//!   so it moves only with the module's structure or the host, never with
//!   what the contract executes;
//! - `delta_instructions` is 4 instructions per unit of Wasm fuel above the
//!   baseline's, plus a fixed host term per method (its name symbol and its
//!   arguments), the same for every case of the method;
//! - an overhead a toolchain adds to every export's entry cancels in
//!   `delta_instructions`, and shows only in `wasm_insns` and in the absolute
//!   `instructions`;
//! - so does the VM instantiation every call pays, most of the baseline's
//!   instructions, which grows with the module: a larger module shows in the
//!   absolute `instructions` and `mem_bytes` and in the row's `wasm_size` and
//!   `code_size`, not in `delta_instructions`.
//!
//! `code_size` is the length of the module's code section, `interface_sha256`
//! the sha256 of its `contractspecv0` section, the contract's interface. The
//! fee estimate is left out: for these calls it is `instructions` at 7
//! stroops per 10,000, rounded up (68 to 189 stroops), so it adds nothing but
//! the rounding.

use std::collections::BTreeMap;
use std::fmt;

use soroban_sdk::xdr::ContractCostType;

use crate::host::{Deployment, contract};
use crate::method::{Method, render_call};
use crate::source::read_repo_file;
use crate::workload::{self, Case};
use crate::{sections, sha256_hex};

/// How each case of [`measure_cost`] is measured, as part of the workload's
/// identity ([`workload::describe`]). Change it whenever the measurement
/// (the private `measure_case`) changes what or how it measures: the
/// workload's hash then changes, and the `cost` binary refuses to run until
/// the change is listed as a new workload.
pub const MEASUREMENT: &str = "measurement 1: each case in a fresh Env holding only this contract, right after \
                               a minimum_liquidity() call, the baseline; instructions and mem_bytes from \
                               resources(), wasm_insns from the WasmInsnExec tracker, delta_instructions = \
                               instructions - the baseline's instructions";

/// The document the `cost` binary prints.
///
/// # Errors
///
/// When the workload table is not the one its id names, when `out/main.wasm`
/// is not the module this build embeds, when the module has no code or no
/// interface section, or when a case traps or returns another value than it
/// records.
pub fn report() -> Result<Json, String> {
    workload::check()?;
    if read_repo_file("out/main.wasm") != contract::WASM {
        return Err("out/main.wasm is not the module this binary embeds; rebuild it (cargo run)".to_owned());
    }
    let code_size = sections::code(contract::WASM)?.len();
    let interface = sections::interface(contract::WASM)?;
    Ok(Json::Object(BTreeMap::from([
        ("code_size", Json::Int(i64::try_from(code_size).map_err(|_| "an oversized code section")?)),
        ("cost", measure_cost()?),
        ("interface_sha256", Json::Str(Plain::new(&sha256_hex(interface))?)),
        ("wasm_sha256", Json::Str(Plain::new(&sha256_hex(contract::WASM))?)),
        ("workload", Json::Str(Plain::new(workload::WORKLOAD_ID)?)),
        ("workload_sha256", Json::Str(Plain::new(workload::WORKLOAD_SHA256)?)),
    ])))
}

/// `method -> case -> costs` for every case of the workload: the `cost`
/// object of [`report`].
///
/// # Errors
///
/// When a case traps or returns another value than it records.
pub fn measure_cost() -> Result<Json, String> {
    let mut cost: BTreeMap<&str, BTreeMap<&str, Json>> = BTreeMap::new();
    for case in workload::CASES {
        cost.entry(case.method.name()).or_default().insert(case.name, measure_case(case)?.to_json());
    }
    Ok(Json::Object(cost.into_iter().map(|(method, cases)| (method, Json::Object(cases))).collect()))
}

/// What one case costs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Costs {
    instructions: i64,
    mem_bytes: i64,
    wasm_insns: i64,
    delta_instructions: i64,
}

impl Costs {
    fn to_json(self) -> Json {
        Json::Object(BTreeMap::from([
            ("delta_instructions", Json::Int(self.delta_instructions)),
            ("instructions", Json::Int(self.instructions)),
            ("mem_bytes", Json::Int(self.mem_bytes)),
            ("wasm_insns", Json::Int(self.wasm_insns)),
        ]))
    }
}

/// Measures `case` as [`MEASUREMENT`] states.
fn measure_case(case: &Case) -> Result<Costs, String> {
    let contract = Deployment::deploy();
    let call = render_call(case.method, case.args);
    contract.call(Method::MinimumLiquidity, &[]).map_err(|_| "the baseline call trapped")?;
    let baseline = contract.env().cost_estimate().resources().instructions;
    let returned = contract.call(case.method, case.args).map_err(|_| format!("{call} trapped"))?;
    if returned != case.returns {
        return Err(format!("{call} returned {returned:?}, not {:?}", case.returns));
    }
    let estimate = contract.env().cost_estimate();
    let resources = estimate.resources();
    let wasm_insns = estimate.budget().tracker(ContractCostType::WasmInsnExec).iterations;
    Ok(Costs {
        instructions: resources.instructions,
        mem_bytes: resources.mem_bytes,
        wasm_insns: i64::try_from(wasm_insns).map_err(|_| "too many Wasm instructions")?,
        delta_instructions: resources.instructions - baseline,
    })
}

/// The JSON [`report`] builds: objects with sorted keys, integers and
/// [`Plain`] strings. It prints as `jq -c` prints it, so a row of the ledger
/// holds the printed `cost` object verbatim.
pub enum Json {
    Int(i64),
    Str(Plain),
    Object(BTreeMap<&'static str, Json>),
}

/// A string that JSON prints as itself between quotes: ASCII letters, digits,
/// `-`, `_` and `.`, as ids and hex digests are.
pub struct Plain(String);

impl Plain {
    /// # Errors
    ///
    /// When `text` holds any other character, which JSON would escape.
    pub fn new(text: &str) -> Result<Plain, String> {
        if text.chars().all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c)) {
            Ok(Plain(text.to_owned()))
        } else {
            Err(format!("{text:?} needs escaping in JSON"))
        }
    }
}

impl fmt::Display for Json {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Json::Int(n) => write!(f, "{n}"),
            Json::Str(Plain(s)) => write!(f, "\"{s}\""),
            Json::Object(fields) => {
                f.write_str("{")?;
                for (i, (key, value)) in fields.iter().enumerate() {
                    if i > 0 {
                        f.write_str(",")?;
                    }
                    write!(f, "\"{key}\":{value}")?;
                }
                f.write_str("}")
            }
        }
    }
}

/// The version `Cargo.lock` pins for `package`, or `None` unless it lists the
/// package exactly once.
#[must_use]
pub fn lock_version<'a>(lock: &'a str, package: &str) -> Option<&'a str> {
    let name = format!("name = \"{package}\"");
    let mut versions = lock
        .lines()
        .zip(lock.lines().skip(1))
        .filter(|&(line, _)| line == name)
        .map(|(_, next)| next.strip_prefix("version = \"")?.strip_suffix('"'));
    let version = versions.next()??;
    versions.next().is_none().then_some(version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prints_sorted_compact_json() {
        let json = Json::Object(BTreeMap::from([
            ("b", Json::Int(-1)),
            ("a", Json::Object(BTreeMap::from([("y", Json::Str(Plain::new("x-1.a_B").unwrap()))]))),
        ]));
        assert_eq!(json.to_string(), r#"{"a":{"y":"x-1.a_B"},"b":-1}"#);
    }

    #[test]
    fn refuses_strings_that_need_escaping() {
        for text in ["a\"b", "a\\b", "a b", "a\nb", "é"] {
            assert!(Plain::new(text).is_err(), "{text:?}");
        }
    }

    #[test]
    fn reads_the_one_locked_version() {
        let lock = "[[package]]\nname = \"a\"\nversion = \"1.0.0\"\n\n[[package]]\nname = \"b\"\nversion = \"2.1.0\"\n";
        assert_eq!(lock_version(lock, "b"), Some("2.1.0"));
        assert_eq!(lock_version(lock, "c"), None);
        let twice = format!("{lock}\n[[package]]\nname = \"b\"\nversion = \"3.0.0\"\n");
        assert_eq!(lock_version(&twice, "b"), None);
        assert_eq!(lock_version("name = \"b\"\n", "b"), None);
    }

    /// What the module doc says the numbers carry, measured: `mem_bytes` is
    /// the baseline's plus 32 per argument, and `delta_instructions` less 4
    /// per unit of fuel above the baseline's is the same for every case of a
    /// method.
    #[test]
    fn the_costs_decompose_as_documented() {
        let costs: Vec<(&Case, Costs)> = workload::CASES
            .iter()
            .map(|case| (case, measure_case(case).unwrap_or_else(|message| panic!("{message}"))))
            .collect();
        let baseline = costs.iter().find(|(case, _)| case.method == Method::MinimumLiquidity).expect("a baseline").1;
        assert_eq!(baseline.delta_instructions, 0, "the baseline against itself");
        let mut host_terms: BTreeMap<&str, i64> = BTreeMap::new();
        for (case, cost) in &costs {
            let call = render_call(case.method, case.args);
            let arguments = i64::try_from(case.args.len()).expect("a few arguments");
            assert_eq!(cost.mem_bytes, baseline.mem_bytes + 32 * arguments, "mem_bytes of {call}");
            let host_term = cost.delta_instructions - 4 * (cost.wasm_insns - baseline.wasm_insns);
            let first = *host_terms.entry(case.method.name()).or_insert(host_term);
            assert_eq!(host_term, first, "the host term of {call}, and of its method's first case");
        }
    }
}
