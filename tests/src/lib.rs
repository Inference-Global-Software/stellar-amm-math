//! Host-side tests and the cost ledger of the stellar-amm-math contract.
//!
//! The library holds what the test targets and the `cost` binary share: an
//! independent oracle of Uniswap V2's formulas ([`oracle`]), the contract's
//! methods as data ([`method`]), the inputs every differential test runs
//! ([`vectors`], drawn with [`rng`]) and the hand-picked rows ([`rows`]), the
//! Soroban harness ([`host`]), the cost workload ([`workload`]) and its
//! measurement ([`ledger`]), a reader of a module's sections ([`sections`]),
//! and readers of the repository's files ([`source`]).
//!
//! # Running
//!
//! This crate lives in `tests/`, apart from the Inference project at the
//! repository root, and runs from there:
//!
//! - `cargo test --locked --no-fail-fast` runs every target on the committed
//!   modules: `contract` on `out/main.wasm` under the Soroban host, `wasm32`
//!   and `specs` on `proofs/main.wasm`, `wide` (and `specs`) on
//!   `tests/shell/out/main.wasm`, and `repo` on the files that must agree.
//!   Without `--no-fail-fast`, a failing target stops the run before the
//!   targets after it. One test is ignored unless asked for: `repo`'s check
//!   that the ledger has a row for the committed build. `cargo test --locked
//!   --no-fail-fast -- --include-ignored` runs it too, the command for CI and
//!   before a commit.
//! - `cargo run --locked --release --bin cost` prints the ledger's
//!   measurement of `out/main.wasm` as JSON.
//! - `bench/snapshot.sh` rebuilds the modules, runs both, and records a row
//!   of `bench/history.jsonl`, the row that ignored test requires.
//!
//! `infs build` (at the root and in `tests/shell/`) writes the two `out/`
//! modules, and `proofs/generate.sh` the proof build.

pub mod host;
pub mod ledger;
pub mod method;
pub mod oracle;
pub mod rng;
pub mod rows;
pub mod sections;
pub mod source;
pub mod vectors;
pub mod workload;

use sha2::{Digest, Sha256};

/// A call that the contract refused. Every refusal and every overflow is the
/// same WebAssembly trap (`unreachable`), and a `try_` caller gets the same
/// error for each ([`host::TRAPPED`]), so a caller learns no more than this.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Trap;

/// The sha256 of `bytes` in lowercase hex, as `sha256sum` prints it.
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
