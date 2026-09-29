//! The cost ledger's measurement: runs the fixed workload
//! (`workload::CASES`) on `out/main.wasm` under the in-process Soroban host
//! and prints one JSON document to stdout, which `bench/snapshot.sh` records.
//! `stellar_amm_math_tests::ledger` describes the document and how each cost
//! is measured.

use std::process::ExitCode;

use stellar_amm_math_tests::source::read_repo_file;
use stellar_amm_math_tests::{ledger, sha256_hex, workload};

fn main() -> ExitCode {
    eprintln!(
        "cost: workload {} ({}) on out/main.wasm, sha256 {}",
        workload::WORKLOAD_ID,
        workload::WORKLOAD_SHA256,
        sha256_hex(&read_repo_file("out/main.wasm"))
    );
    match ledger::report() {
        Ok(report) => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("cost: {message}");
            ExitCode::FAILURE
        }
    }
}
