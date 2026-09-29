//! A plain wasm32 module under wasmi: the proof module or the test shell.
//!
//! Shared by the test targets that run them (`mod vm;`).
//!
//! In the functions these modules export, every refusal is an `assert` or a
//! checked `+`, `-` or `*`, and both compile to `unreachable`. Inference's `/`
//! and `%` are not guarded: a zero divisor raises `IntegerDivisionByZero`.
//! Any trap code but `UnreachableCodeReached` therefore means a division or a
//! memory access that no assert guards, and fails the test with its code. A
//! Soroban `try_` caller cannot tell such a trap from a refusal (the host
//! narrows both to the same error), so `contract.rs` cannot see it: this
//! check is where it shows.
//!
//! A trap unwinds past the function epilogue, so the frame the call reserved
//! on the shadow stack is never given back and the exported `__stack_pointer`
//! stays lowered (`reused.rs` shows an instance that is not restored running
//! out of stack after about a thousand traps). [`Vm`] restores it after every
//! trap, and checks that every call that returns leaves it where it was.

use std::fmt::Debug;

use stellar_amm_math_tests::Trap;
use stellar_amm_math_tests::source::read_repo_file;
use wasmi::{Engine, Global, Instance, Linker, Module, Store, TrapCode, Val, WasmParams, WasmResults};

pub struct Vm {
    path: &'static str,
    store: Store<()>,
    instance: Instance,
    stack_pointer: Global,
    stack_top: i32,
}

impl Vm {
    /// Instantiates the module at `path`, from the repository root.
    pub fn load(path: &'static str) -> Vm {
        let engine = Engine::default();
        let module = Module::new(&engine, &read_repo_file(path)[..])
            .unwrap_or_else(|err| panic!("{path} does not validate: {err}"));
        let mut store = Store::new(&engine, ());
        let instance = Linker::<()>::new(&engine)
            .instantiate_and_start(&mut store, &module)
            .unwrap_or_else(|err| panic!("{path} does not instantiate: {err}"));
        let stack_pointer = instance
            .get_global(&store, "__stack_pointer")
            .unwrap_or_else(|| panic!("{path} exports no __stack_pointer"));
        let stack_top = stack_pointer.get(&store).i32().expect("__stack_pointer is an i32");
        Vm { path, store, instance, stack_pointer, stack_top }
    }

    /// Calls the export `name`: its result, or [`Trap`] when it traps with
    /// UnreachableCodeReached.
    ///
    /// # Panics
    ///
    /// When the export is missing or has another signature, when the call
    /// traps with any other code, or when a call that returns moves the stack
    /// pointer.
    pub fn call<P, R>(&mut self, name: &str, params: P) -> Result<R, Trap>
    where
        P: WasmParams + Debug + Copy,
        R: WasmResults,
    {
        let func = self
            .instance
            .get_typed_func::<P, R>(&self.store, name)
            .unwrap_or_else(|err| panic!("{}: export {name}: {err}", self.path));
        match func.call(&mut self.store, params) {
            Ok(result) => {
                let stack_pointer = self.stack_pointer.get(&self.store).i32();
                assert_eq!(
                    stack_pointer,
                    Some(self.stack_top),
                    "{}: {name}{params:?} returned with __stack_pointer moved",
                    self.path,
                );
                Ok(result)
            }
            Err(err) => {
                assert_eq!(
                    err.as_trap_code(),
                    Some(TrapCode::UnreachableCodeReached),
                    "{}: {name}{params:?} failed with another trap: {err}",
                    self.path,
                );
                self.stack_pointer
                    .set(&mut self.store, Val::I32(self.stack_top))
                    .expect("__stack_pointer is a mutable i32");
                Err(Trap)
            }
        }
    }

    /// [`Vm::call`] for an export that returns a `bool`, an `i32` that is 0
    /// or 1.
    ///
    /// # Panics
    ///
    /// As [`Vm::call`] does, and when the result is any other `i32`.
    pub fn call_bool<P>(&mut self, name: &str, params: P) -> Result<bool, Trap>
    where
        P: WasmParams + Debug + Copy,
    {
        match self.call::<P, i32>(name, params)? {
            0 => Ok(false),
            1 => Ok(true),
            other => panic!("{}: {name}{params:?} returned {other}, not a bool", self.path),
        }
    }
}
