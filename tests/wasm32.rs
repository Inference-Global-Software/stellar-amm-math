//! `proofs/main.wasm`, the wasm32 module the Rocq obligations describe, under
//! wasmi: on the hand-picked rows and on the same differential inputs as the
//! contract, every method returns what the Soroban contract returns and what
//! the oracle returns, and traps where they trap. The proof module exports
//! the methods with their plain wasm32 signatures: a `u32` is an `i32`, and a
//! `bool` an `i32` that is 0 or 1.
//!
//! A trap that is not a refusal (an unguarded division by zero, an access
//! out of bounds) shows here and not in `contract.rs`: the Soroban host
//! narrows every trap to the same error, while [`Vm`] fails on any trap code
//! but `UnreachableCodeReached`.

mod reused;
mod vm;

use std::fmt::Debug;

use reused::{Reused, TRAPS};
use stellar_amm_math_tests::Trap;
use stellar_amm_math_tests::host::Deployment;
use stellar_amm_math_tests::method::{Method, Value, render_call};
use stellar_amm_math_tests::rows;
use stellar_amm_math_tests::source::read_repo_file;
use stellar_amm_math_tests::vectors::{self, Coverage};
use vm::Vm;
use wasmi::{Engine, ExternType, Module, ValType, WasmParams};

const PROOF_MODULE: &str = "proofs/main.wasm";

/// Calls `method` in the proof module.
fn call(vm: &mut Vm, method: Method, args: &[u32]) -> Result<Value, Trap> {
    match *args {
        [] => call_with(vm, method, ()),
        [a, b] => call_with(vm, method, (a, b)),
        [a, b, c] => call_with(vm, method, (a, b, c)),
        [a, b, c, d] => call_with(vm, method, (a, b, c, d)),
        [a, b, c, d, e] => call_with(vm, method, (a, b, c, d, e)),
        _ => panic!("no method takes {} arguments", args.len()),
    }
}

fn call_with<P: WasmParams + Debug + Copy>(vm: &mut Vm, method: Method, params: P) -> Result<Value, Trap> {
    if method.returns_bool() {
        vm.call_bool(method.name(), params).map(Value::Bool)
    } else {
        vm.call(method.name(), params).map(Value::U32)
    }
}

/// The proof module exports the contract's methods and no other function,
/// each taking one `i32` per parameter and returning one `i32`. (wasmi lists
/// exports by name, so this compares them as sorted lists.)
#[test]
fn the_proof_module_exports_the_methods() {
    let engine = Engine::default();
    let module = Module::new(&engine, &read_repo_file(PROOF_MODULE)[..]).expect("the proof module validates");
    let signature = |name: &str, params: &[ValType], results: &[ValType]| format!("{name}{params:?} -> {results:?}");
    let mut exported: Vec<String> = module
        .exports()
        .filter_map(|export| {
            let ExternType::Func(func) = export.ty() else { return None };
            Some(signature(export.name(), func.params(), func.results()))
        })
        .collect();
    let mut listed: Vec<String> = Method::ALL
        .iter()
        .map(|method| signature(method.name(), &vec![ValType::I32; method.arity()], &[ValType::I32]))
        .collect();
    exported.sort();
    listed.sort();
    assert_eq!(exported, listed, "the functions {PROOF_MODULE} exports, and method::Method");
}

/// Every hand-picked row ([`rows::SETS`]): the proof module gives the row's
/// value, or traps where the row refuses.
#[test]
fn hand_picked_rows() {
    let mut vm = Vm::load(PROOF_MODULE);
    let differing = rows::mismatches(|method, args| call(&mut vm, method, args));
    assert!(differing.is_empty(), "{} rows differ on {PROOF_MODULE}:\n{}", differing.len(), differing.join("\n"));
}

/// Every method on the shared differential inputs
/// ([`vectors::for_each_case`]): the proof module, the Soroban contract and
/// the oracle agree.
mod differential {
    use super::*;

    fn agrees_with_the_contract_and_the_oracle(method: Method) {
        let mut vm = Vm::load(PROOF_MODULE);
        let contract = Deployment::deploy();
        let mut coverage = Coverage::default();
        vectors::for_each_case(method, |args| {
            let expected = method.oracle(args);
            let proof = call(&mut vm, method, args);
            let call = render_call(method, args);
            assert_eq!(proof, contract.call(method, args), "{call}: the proof module and the contract");
            assert_eq!(proof, expected.map_err(|_| Trap), "{call}: the proof module and the oracle ({expected:?})");
            coverage.record(expected);
        });
        coverage.assert_complete(method);
    }

    #[test]
    fn minimum_liquidity() {
        agrees_with_the_contract_and_the_oracle(Method::MinimumLiquidity);
    }

    #[test]
    fn get_amount_out() {
        agrees_with_the_contract_and_the_oracle(Method::GetAmountOut);
    }

    #[test]
    fn get_amount_in() {
        agrees_with_the_contract_and_the_oracle(Method::GetAmountIn);
    }

    #[test]
    fn quote() {
        agrees_with_the_contract_and_the_oracle(Method::Quote);
    }

    #[test]
    fn mint_initial() {
        agrees_with_the_contract_and_the_oracle(Method::MintInitial);
    }

    #[test]
    fn mint_proportional() {
        agrees_with_the_contract_and_the_oracle(Method::MintProportional);
    }

    #[test]
    fn burn_share() {
        agrees_with_the_contract_and_the_oracle(Method::BurnShare);
    }

    #[test]
    fn k_holds() {
        agrees_with_the_contract_and_the_oracle(Method::KHolds);
    }

    #[test]
    fn k_holds_with_fee() {
        agrees_with_the_contract_and_the_oracle(Method::KHoldsWithFee);
    }

    #[test]
    fn mul_div() {
        agrees_with_the_contract_and_the_oracle(Method::MulDiv);
    }

    #[test]
    fn mul_div_up() {
        agrees_with_the_contract_and_the_oracle(Method::MulDivUp);
    }
}

/// Why the harness restores `__stack_pointer` after a trap: on one reused
/// instance of the proof module, each trapped call leaks the frame it
/// reserved until the stack runs out ([`Reused::leaks_a_frame_per_trap`]).
/// [`Vm`] restores it, so there [`TRAPS`] traps are harmless, as they are
/// under the Soroban host, which instantiates per invocation (`contract.rs`:
/// every_call_gets_a_fresh_instance).
#[test]
fn a_reused_instance_leaks_a_frame_per_trap() {
    let quoted = ((1000_u32, 100_000_u32, 100_000_u32, 30_u32), 987_u32);
    Reused::load(PROOF_MODULE).leaks_a_frame_per_trap("get_amount_out", quoted, (0, 1, 1, 0));

    let mut vm = Vm::load(PROOF_MODULE);
    for i in 1..=TRAPS {
        assert_eq!(call(&mut vm, Method::GetAmountOut, &[0, 1, 1, 0]), Err(Trap), "trap {i} of {TRAPS} on Vm");
    }
    let quoted = call(&mut vm, Method::GetAmountOut, &[1000, 100_000, 100_000, 30]);
    assert_eq!(quoted, Ok(Value::U32(987)), "get_amount_out(1000, 100000, 100000, 30) after {TRAPS} traps on Vm");
}
