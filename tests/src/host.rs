//! The deployable contract, `out/main.wasm`, under the in-process Soroban host.

use std::fmt::Debug;

use soroban_sdk::testutils::EnvTestConfig;
use soroban_sdk::xdr::{ScErrorCode, ScErrorType};
use soroban_sdk::{Env, Error, InvokeError};

use crate::Trap;
use crate::method::{Method, Value};
use crate::source::assert_built_here;

/// The typed client of `out/main.wasm`. The module is embedded with
/// `include_bytes!`, so Cargo rebuilds whatever uses it when the file changes.
pub mod contract {
    soroban_sdk::contractimport!(file = "../out/main.wasm");
}

/// The error a `try_` call returns for every refusal: the host's `try_call`
/// narrows each recoverable error that is not a contract error to it.
///
/// `soroban_sdk::Error` is the error type of the generated `try_` functions.
/// soroban-sdk hides it from its documentation (it is documented as
/// `soroban_env_common::Error`), and its own example for
/// `Env::try_as_contract` builds one with `from_type_and_code`, as here.
///
/// It is not the error the VM reports. A trap (a failed assert, a checked
/// overflow, or the contract's own check of an argument's type, each a wasm
/// `unreachable`) is `Error(WasmVm, InvalidAction)`, and a wrong argument
/// count is `Error(WasmVm, UnexpectedSize)`, refused before any Wasm runs.
/// Those are the errors the plain client panics with (`contract.rs` pins
/// them). The CLI and the host's diagnostic events show them too, with a
/// reason (`VM call trapped: UnreachableCodeReached`) that is not part of the
/// error. Budget exhaustion is not recoverable: it panics through a `try_`
/// call instead of returning.
pub const TRAPPED: Error = Error::from_type_and_code(ScErrorType::Context, ScErrorCode::InvalidAction);

/// A test environment that writes nothing to disk. `Env::default()` would save
/// a ledger snapshot into `test_snapshots/` whenever an `Env` drops.
///
/// # Panics
///
/// When this binary embeds another checkout's contract
/// ([`assert_built_here`]).
#[must_use]
pub fn test_env() -> Env {
    assert_built_here();
    Env::new_with_config(EnvTestConfig { capture_snapshot_at_drop: false })
}

/// `out/main.wasm` deployed in an `Env` of its own. Any other contract in the
/// same `Env` would change the metered costs, even without being called.
/// (`contract::Contract` is another thing: the interface trait
/// `contractimport!` generates, from which it builds [`contract::Client`].)
pub struct Deployment {
    client: contract::Client<'static>,
}

impl Deployment {
    /// A fresh [`test_env`] with `out/main.wasm` registered in it.
    #[must_use]
    pub fn deploy() -> Deployment {
        let env = test_env();
        let id = env.register(contract::WASM, ());
        Deployment { client: contract::Client::new(&env, &id) }
    }

    /// The typed client of the deployed contract.
    #[must_use]
    pub fn client(&self) -> &contract::Client<'static> {
        &self.client
    }

    /// The `Env` the contract is deployed in, whose budget meters its calls.
    #[must_use]
    pub fn env(&self) -> &Env {
        &self.client.env
    }

    /// Calls `method` through its `try_` client function.
    ///
    /// # Panics
    ///
    /// When `args` does not have one value per parameter, or the call fails in
    /// any way but the contract trap ([`TRAPPED`]).
    pub fn call(&self, method: Method, args: &[u32]) -> Result<Value, Trap> {
        use Value::{Bool, U32};
        let c = &self.client;
        match (method, args) {
            (Method::MinimumLiquidity, []) => settle(c.try_minimum_liquidity()).map(U32),
            (Method::GetAmountOut, [a, ri, ro, f]) => settle(c.try_get_amount_out(a, ri, ro, f)).map(U32),
            (Method::GetAmountIn, [x, ri, ro, f]) => settle(c.try_get_amount_in(x, ri, ro, f)).map(U32),
            (Method::Quote, [a, ra, rb]) => settle(c.try_quote(a, ra, rb)).map(U32),
            (Method::MintInitial, [a0, a1]) => settle(c.try_mint_initial(a0, a1)).map(U32),
            (Method::MintProportional, [a0, a1, r0, r1, s]) => {
                settle(c.try_mint_proportional(a0, a1, r0, r1, s)).map(U32)
            }
            (Method::BurnShare, [l, b, s]) => settle(c.try_burn_share(l, b, s)).map(U32),
            (Method::KHolds, [a, o, ri, ro]) => settle(c.try_k_holds(a, o, ri, ro)).map(Bool),
            (Method::KHoldsWithFee, [a, o, ri, ro, f]) => {
                settle(c.try_k_holds_with_fee(a, o, ri, ro, f)).map(Bool)
            }
            (Method::MulDiv, [a, b, d]) => settle(c.try_mul_div(a, b, d)).map(U32),
            (Method::MulDivUp, [a, b, d]) => settle(c.try_mul_div_up(a, b, d)).map(U32),
            _ => panic!("{} takes {} arguments, got {args:?}", method.name(), method.arity()),
        }
    }
}

/// A `try_` result as a value or a trap. Anything else (a conversion error, an
/// error other than [`TRAPPED`], an `InvokeError`) is a harness failure.
///
/// # Panics
///
/// On any outcome but a value or [`TRAPPED`].
pub fn settle<T: Debug, E: Debug>(result: Result<Result<T, E>, Result<Error, InvokeError>>) -> Result<T, Trap> {
    match result {
        Ok(Ok(value)) => Ok(value),
        Err(Ok(error)) if error == TRAPPED => Err(Trap),
        other => panic!("expected a value or {TRAPPED:?}, got {other:?}"),
    }
}
