//! The deployable contract, `out/main.wasm`, under the Soroban host, against
//! the oracle: its interface against the method table, the hand-picked rows
//! (V2's reference values, the boundaries where a result stops fitting or
//! flips, the trap matrix), every method on the shared differential inputs,
//! and what the host does around refusals, malformed arguments and the
//! instruction budget.
//!
//! A `try_` call returns the same error for every refusal ([`TRAPPED`]), so
//! here a trap that is not a refusal (an unguarded division by zero, an
//! access out of bounds) looks like one. `wasm32.rs` tells them apart on
//! `proofs/main.wasm`, a separate build, and that covers this module because
//! the two builds share their code: this module's functions are the proof
//! build's first functions, byte for byte, followed by one `<method>$val`
//! wrapper per method that decodes the arguments and encodes the result
//! (`the_two_builds_share_every_function_but_the_wrappers`).

mod reused;

use std::any::Any;
use std::fmt::Debug;
use std::io::Cursor;
use std::panic::{AssertUnwindSafe, catch_unwind};

use reused::{Reused, TRAPS};
use soroban_sdk::xdr::{Limited, Limits, ReadXdr, ScErrorCode, ScErrorType, ScSpecEntry, ScSpecTypeDef};
use soroban_sdk::{Env, Error, IntoVal, Symbol, Val, vec};
use stellar_amm_math_tests::Trap;
use stellar_amm_math_tests::host::{Deployment, TRAPPED, contract, settle, test_env};
use stellar_amm_math_tests::method::{Method, Value, render_call};
use stellar_amm_math_tests::rows;
use stellar_amm_math_tests::sections;
use stellar_amm_math_tests::source::read_repo_file;
use stellar_amm_math_tests::vectors::{self, Coverage};
use stellar_amm_math_tests::workload;
use wasmi::TrapCode;

const M: u32 = u32::MAX;

/// The error the VM reports for a trap, which a plain call panics with.
const VM_TRAPPED: Error = Error::from_type_and_code(ScErrorType::WasmVm, ScErrorCode::InvalidAction);

/// The error the VM reports for a call with the wrong number of arguments.
const VM_WRONG_ARGUMENT_COUNT: Error = Error::from_type_and_code(ScErrorType::WasmVm, ScErrorCode::UnexpectedSize);

/// The instructions one invocation may use on mainnet (soroban-sdk 28's
/// snapshot of the network settings).
const MAINNET_INSTRUCTION_LIMIT: i64 = 400_000_000;

/// A value as out/main.wasm's exports take and return it under wasmi: the
/// payload of its `Val`.
fn val_payload(value: Val) -> i64 {
    value.get_payload().cast_signed()
}

/// A `u32` as a `U32Val` payload.
fn u32_val(x: u32) -> i64 {
    val_payload(Val::from_u32(x).to_val())
}

/// The text of a panic's payload.
fn panic_message(payload: &(dyn Any + Send)) -> &str {
    payload
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| payload.downcast_ref::<&str>().copied())
        .unwrap_or_default()
}

/// Requires `call` to panic with `error`, which the host escalates as a
/// message that starts `HostError: <error>`. The rest of the message is the
/// host's diagnostic event log, and is not matched: the errors its events
/// name are not always the one the call failed with.
fn assert_panics_with<R: Debug>(error: Error, what: &str, call: impl FnOnce() -> R) {
    let payload = catch_unwind(AssertUnwindSafe(call)).expect_err(&format!("{what} returned"));
    let message = panic_message(payload.as_ref());
    let host_error = format!("HostError: {error:?}");
    assert!(message.starts_with(&host_error), "{what} did not panic with {host_error}: {message}");
}

/// Amounts of other types than `u32`, as `Val`s whose payload holds 100 in
/// its upper 32 bits, where a `U32Val` holds its value. A contract that
/// skipped its check of the argument's type would read each as the amount
/// 100, and quote(100, 1000, 3000) returns 300: where they trap, that check
/// is what refuses them.
fn mistyped_amounts(env: &Env) -> [(&'static str, Val); 3] {
    let amounts = [
        ("an i32 amount", 100_i32.into_val(env)),
        ("a u64 amount", (100_u64 << 24).into_val(env)),
        ("an i64 amount", (100_i64 << 24).into_val(env)),
    ];
    for (what, amount) in amounts {
        assert_eq!(val_payload(amount) >> 32, 100, "{what}, {amount:?}: the upper half of its payload");
    }
    amounts
}

/// The Soroban way to test a contract: an `Env`, the contract registered in
/// it, and the typed client `contractimport!` generates from the module's
/// interface. The plain functions return the value; the `try_` functions
/// return the value in `Ok(Ok(..))` and a refusal as [`TRAPPED`].
#[test]
fn typed_client_returns_values_and_try_calls_return_the_refusal() {
    let env = test_env();
    let id = env.register(contract::WASM, ());
    let client = contract::Client::new(&env, &id);
    assert_eq!(client.get_amount_out(&1000, &100_000, &100_000, &30), 987);
    assert!(client.k_holds_with_fee(&1000, &987, &100_000, &100_000, &30));
    assert_eq!(client.try_quote(&100, &1000, &3000), Ok(Ok(300)));
    assert_eq!(client.try_quote(&0, &1000, &3000), Err(Ok(TRAPPED)));
}

/// A plain call panics on a refusal, with the error the VM reports for the
/// trap, not the one a `try_` call returns.
#[test]
fn the_plain_client_panics_on_a_refusal() {
    let contract = Deployment::deploy();
    assert_panics_with(VM_TRAPPED, "quote(0, 1000, 3000)", || contract.client().quote(&0, &1000, &3000));
}

/// The premise of this file's first paragraph: out/main.wasm's functions are
/// the proof build's first functions, byte for byte, followed by one
/// `<method>$val` wrapper per method. So a trap `wasm32.rs` sees in a method
/// of the proof module is the trap this module's method raises. (Neither
/// module imports a function, so body `i` of each is function `i`.)
#[test]
fn the_two_builds_share_every_function_but_the_wrappers() {
    fn bodies<'a>(path: &str, module: &'a [u8]) -> Vec<&'a [u8]> {
        sections::function_bodies(module).unwrap_or_else(|message| panic!("{path}: {message}"))
    }
    let proof_module = read_repo_file("proofs/main.wasm");
    let (contract, proof) = (bodies("out/main.wasm", contract::WASM), bodies("proofs/main.wasm", &proof_module));
    let wrappers = Method::ALL.len();
    let shared = contract.len().checked_sub(wrappers).unwrap_or_else(|| {
        panic!("out/main.wasm defines {} functions, fewer than its {wrappers} wrappers", contract.len())
    });
    assert!(
        proof.len() >= shared,
        "proofs/main.wasm defines {} functions, fewer than the {shared} it shares with out/main.wasm",
        proof.len()
    );
    if let Some(i) = (0..shared).find(|&i| contract[i] != proof[i]) {
        panic!(
            "function {i} of out/main.wasm differs from function {i} of proofs/main.wasm, the first of the {shared} \
             the two builds must share: wasm32.rs's trap checks on the proof module do not cover this module's code \
             (compare the two with `wasm-tools print`)"
        );
    }
}

/// `method::Method` is the contract's interface, as the `contractspecv0`
/// section of `out/main.wasm` declares it for `contractimport!`, the CLI and
/// every other client: the same functions in the same order, each parameter
/// a `u32` under the name [`Method::params`] gives it (its CLI flag), and a
/// `bool` or a `u32` result. A method added to the contract, or a parameter
/// renamed, fails here until the table, and with it every differential test,
/// follows.
#[test]
fn the_method_table_is_the_contracts_interface() {
    let interface = sections::interface(contract::WASM).unwrap_or_else(|message| panic!("out/main.wasm: {message}"));
    let mut reader = Limited::new(Cursor::new(interface), Limits::none());
    let declared: Vec<String> = ScSpecEntry::read_xdr_iter(&mut reader)
        .map(|entry| match entry.expect("a well-formed interface entry") {
            ScSpecEntry::FunctionV0(function) => {
                let params: Vec<String> = function
                    .inputs
                    .iter()
                    .map(|input| format!("{}: {:?}", input.name.to_utf8_string_lossy(), input.type_))
                    .collect();
                let results: Vec<String> = function.outputs.iter().map(|result| format!("{result:?}")).collect();
                format!("{}({}) -> {}", function.name.to_utf8_string_lossy(), params.join(", "), results.join(", "))
            }
            other => panic!("the interface declares more than functions: {other:?}"),
        })
        .collect();
    let listed: Vec<String> = Method::ALL
        .iter()
        .map(|method| {
            let params: Vec<String> =
                method.params().iter().map(|name| format!("{name}: {:?}", ScSpecTypeDef::U32)).collect();
            let result = if method.returns_bool() { ScSpecTypeDef::Bool } else { ScSpecTypeDef::U32 };
            format!("{}({}) -> {result:?}", method.name(), params.join(", "))
        })
        .collect();
    assert_eq!(declared, listed, "the functions out/main.wasm declares, and method::Method");
}

/// Every hand-picked row ([`rows::SETS`]): the contract gives the row's
/// value, or traps where the row refuses. (`rows` checks the oracle against
/// the same rows.)
#[test]
fn hand_picked_rows() {
    let contract = Deployment::deploy();
    let differing = rows::mismatches(|method, args| contract.call(method, args));
    assert!(differing.is_empty(), "{} rows differ on out/main.wasm:\n{}", differing.len(), differing.join("\n"));
}

/// Every method on the shared differential inputs
/// ([`vectors::for_each_case`]): the contract returns the oracle's value, and
/// traps exactly where the oracle refuses. No call that returns uses a
/// thousandth of the mainnet instruction limit, so budget exhaustion, which a
/// `try_` call does not return (`budget_exhaustion_escapes_try_calls`), is
/// out of reach.
mod differential {
    use super::*;

    fn agrees_with_the_oracle(method: Method) {
        let contract = Deployment::deploy();
        let mut coverage = Coverage::default();
        let mut costliest = (0, Vec::new());
        vectors::for_each_case(method, |args| {
            let expected = method.oracle(args);
            let actual = contract.call(method, args);
            assert_eq!(actual, expected.map_err(|_| Trap), "{} (the oracle: {expected:?})", render_call(method, args));
            if actual.is_ok() {
                let instructions = contract.env().cost_estimate().resources().instructions;
                if instructions > costliest.0 {
                    costliest = (instructions, args.to_vec());
                }
            }
            coverage.record(expected);
        });
        coverage.assert_complete(method);
        let (instructions, args) = costliest;
        assert!(
            instructions < MAINNET_INSTRUCTION_LIMIT / 1000,
            "{} uses {instructions} instructions",
            render_call(method, &args)
        );
    }

    #[test]
    fn minimum_liquidity() {
        agrees_with_the_oracle(Method::MinimumLiquidity);
    }

    #[test]
    fn get_amount_out() {
        agrees_with_the_oracle(Method::GetAmountOut);
    }

    #[test]
    fn get_amount_in() {
        agrees_with_the_oracle(Method::GetAmountIn);
    }

    #[test]
    fn quote() {
        agrees_with_the_oracle(Method::Quote);
    }

    #[test]
    fn mint_initial() {
        agrees_with_the_oracle(Method::MintInitial);
    }

    #[test]
    fn mint_proportional() {
        agrees_with_the_oracle(Method::MintProportional);
    }

    #[test]
    fn burn_share() {
        agrees_with_the_oracle(Method::BurnShare);
    }

    #[test]
    fn k_holds() {
        agrees_with_the_oracle(Method::KHolds);
    }

    #[test]
    fn k_holds_with_fee() {
        agrees_with_the_oracle(Method::KHoldsWithFee);
    }

    #[test]
    fn mul_div() {
        agrees_with_the_oracle(Method::MulDiv);
    }

    #[test]
    fn mul_div_up() {
        agrees_with_the_oracle(Method::MulDivUp);
    }
}

/// Arguments that do not match the interface are refused, and a `try_` call
/// cannot tell them from a refusal of the contract's own: it returns
/// [`TRAPPED`] for each. A plain call panics with the host's own error, which
/// tells a wrong number of arguments apart (`Error(WasmVm, UnexpectedSize)`,
/// refused before any Wasm runs) but not a value of another type: that traps
/// in the contract's own check of the argument, like any refusal
/// (`Error(WasmVm, InvalidAction)`), and the next test shows that trap on the
/// module itself. The CLI and the host's diagnostic events add a reason, `VM
/// call trapped: UnreachableCodeReached` and `VM call failed:
/// Func(MismatchingParameterLen)`, which is not part of the error and is not
/// checked here. The typed client cannot make such a call, so these go
/// through the untyped `invoke_contract`, with the arguments as raw `Val`s.
#[test]
fn malformed_arguments_are_refused() {
    let env = test_env();
    let id = env.register(contract::WASM, ());
    let quote = Symbol::new(&env, "quote");
    let (a, ra, rb): (Val, Val, Val) = (100_u32.into_val(&env), 1000_u32.into_val(&env), 3000_u32.into_val(&env));
    assert_eq!(settle(env.try_invoke_contract::<u32, Error>(&id, &quote, vec![&env, a, ra, rb])), Ok(300));
    let mistyped = mistyped_amounts(&env).map(|(what, amount)| (what, vec![&env, amount, ra, rb], VM_TRAPPED));
    let malformed = [
        // A bool's payload is 0 or 1, whose upper half reads as the amount 0,
        // which quote refuses on its own: this row shows only that some check
        // refuses a bool.
        ("a bool amount", vec![&env, true.into_val(&env), ra, rb], VM_TRAPPED),
        ("two arguments", vec![&env, a, ra], VM_WRONG_ARGUMENT_COUNT),
        ("four arguments", vec![&env, a, ra, rb, rb], VM_WRONG_ARGUMENT_COUNT),
    ];
    for (what, args, vm_error) in mistyped.into_iter().chain(malformed) {
        let tried = settle(env.try_invoke_contract::<u32, Error>(&id, &quote, args.clone()));
        assert_eq!(tried, Err(Trap), "a try_ call of quote with {what}");
        let plain = format!("a plain call of quote with {what}");
        assert_panics_with(vm_error, &plain, || env.invoke_contract::<u32>(&id, &quote, args));
    }
}

/// A value of another type is refused by a trap of the contract's own, the
/// `unreachable` of its check of the argument's type, which is all that
/// stands between [`mistyped_amounts`] and a result: out/main.wasm under
/// wasmi (it imports nothing), called with `Val` payloads. This tries quote's
/// amount, one of the 36 arguments the `<method>$val` wrappers check.
#[test]
fn a_value_of_another_type_traps_in_the_contracts_own_check() {
    let env = test_env();
    let mut contract = Reused::load("out/main.wasm");
    let (ra, rb) = (u32_val(1000), u32_val(3000));
    assert_eq!(contract.call::<_, i64>("quote", (u32_val(100), ra, rb)), Ok(u32_val(300)), "quote(100, 1000, 3000)");
    for (what, amount) in mistyped_amounts(&env) {
        let called = contract.call::<_, i64>("quote", (val_payload(amount), ra, rb));
        let called = called.map(|result| Val::from_payload(result.cast_unsigned()));
        assert!(matches!(called, Err(TrapCode::UnreachableCodeReached)), "quote with {what}, {amount:?}: {called:?}");
    }
}

/// Why the host must instantiate the contract for every invocation
/// ([`Reused::leaks_a_frame_per_trap`]): out/main.wasm's get_amount_out
/// reserves its frame before the assert that refuses amount_in = 0, so on one
/// reused instance each such trap leaks a frame until the stack runs out.
/// Shown on out/main.wasm itself under wasmi, with its arguments as `U32Val`s.
#[test]
fn a_trap_leaks_the_contracts_frame_on_a_reused_instance() {
    let quoted = ((u32_val(1000), u32_val(100_000), u32_val(100_000), u32_val(30)), u32_val(987));
    let refused = (u32_val(0), u32_val(1), u32_val(1), u32_val(0));
    Reused::load("out/main.wasm").leaks_a_frame_per_trap("get_amount_out", quoted, refused);
}

/// The host instantiates the module for every invocation (the test above
/// shows why it must, and that a reused instance would fail before [`TRAPS`]
/// traps), so [`TRAPS`] traps later a call still returns.
#[test]
fn every_call_gets_a_fresh_instance() {
    let contract = Deployment::deploy();
    for i in 1..=TRAPS {
        assert_eq!(contract.call(Method::GetAmountOut, &[0, 1, 1, 0]), Err(Trap), "trap {i} of {TRAPS}");
    }
    let quoted = contract.call(Method::GetAmountOut, &[1000, 100_000, 100_000, 30]);
    assert_eq!(quoted, Ok(Value::U32(987)), "get_amount_out(1000, 100000, 100000, 30) after {TRAPS} traps");
}

/// Running out of budget is not a trap: the host panics through a `try_`
/// call instead of returning an error. No call of this contract comes near
/// the limit (the differential tests check every call that returns).
#[test]
fn budget_exhaustion_escapes_try_calls() {
    let contract = Deployment::deploy();
    contract.env().cost_estimate().budget().reset_limits(100_000, 100_000_000);
    let exceeded = Error::from_type_and_code(ScErrorType::Budget, ScErrorCode::ExceededLimit);
    let call = "try_get_amount_out(2^32 - 1, 2^32 - 1, 2^32 - 1, 0) on a budget of 100000 instructions";
    assert_panics_with(exceeded, call, || contract.client().try_get_amount_out(&M, &M, &M, &0));
}

#[test]
fn workload_cases_return_their_recorded_values() {
    for case in workload::CASES {
        let call = render_call(case.method, case.args);
        assert_eq!(case.method.oracle(case.args), Ok(case.returns), "oracle: {call}");
        assert_eq!(Deployment::deploy().call(case.method, case.args), Ok(case.returns), "contract: {call}");
    }
    for method in Method::ALL {
        let names: Vec<&str> =
            workload::CASES.iter().filter(|case| case.method == method).map(|case| case.name).collect();
        assert_eq!(names, ["typical", "worst"], "{}", method.name());
    }
}
