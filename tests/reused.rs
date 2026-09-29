//! One instance of a module under wasmi, every call made on it as it is:
//! nothing restores `__stack_pointer` after a trap, as nothing would on a
//! host that reused one instance for every invocation.
//!
//! Shared by `contract.rs`, on `out/main.wasm`, and `wasm32.rs`, on the proof
//! module (`mod reused;`).

use std::fmt::Debug;

use stellar_amm_math_tests::source::read_repo_file;
use wasmi::{Engine, Global, Instance, Linker, Module, Store, TrapCode, WasmParams, WasmResults};

/// More trapped calls than one reused instance of either module survives:
/// [`Reused::leaks_a_frame_per_trap`] requires its stack to run out before
/// this many. So a module that still returns after this many traps got a
/// fresh instance per call (the Soroban host's) or a restored stack pointer
/// (`vm::Vm`'s).
pub const TRAPS: u32 = 2000;

pub struct Reused {
    path: &'static str,
    store: Store<()>,
    instance: Instance,
    stack_pointer: Global,
}

impl Reused {
    /// Instantiates the module at `path`, from the repository root.
    pub fn load(path: &'static str) -> Reused {
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
        Reused { path, store, instance, stack_pointer }
    }

    /// Calls the export `name`: its result, or the code of the trap it
    /// failed with.
    ///
    /// # Panics
    ///
    /// When the export is missing or has another signature, or when the call
    /// fails without a trap.
    pub fn call<P: WasmParams, R: WasmResults>(&mut self, name: &str, params: P) -> Result<R, TrapCode> {
        let func = self
            .instance
            .get_typed_func::<P, R>(&self.store, name)
            .unwrap_or_else(|err| panic!("{}: export {name}: {err}", self.path));
        func.call(&mut self.store, params).map_err(|err| {
            err.as_trap_code().unwrap_or_else(|| panic!("{}: {name} failed without a trap: {err}", self.path))
        })
    }

    fn stack_pointer(&self) -> i32 {
        self.stack_pointer.get(&self.store).i32().expect("__stack_pointer is an i32")
    }

    /// Why a harness must restore `__stack_pointer` after a trap, and a host
    /// instantiate the module for every invocation. A trap unwinds past the
    /// function epilogue, so the frame the call reserved on the shadow stack
    /// is never given back.
    ///
    /// On this instance, the call of `name` with `returning.0` returns
    /// `returning.1` and gives its frame back. Then each call with `refused`
    /// traps with `unreachable` (an assert that refuses it) and leaves the
    /// stack pointer a frame lower, until a call's frame no longer fits the
    /// stack and it fails with an out-of-bounds memory access instead: fewer
    /// than [`TRAPS`] traps in, with the whole stack leaked.
    ///
    /// # Panics
    ///
    /// When any of that does not hold.
    pub fn leaks_a_frame_per_trap<P, R>(&mut self, name: &str, returning: (P, R), refused: P)
    where
        P: WasmParams + Debug + Copy,
        R: WasmResults + PartialEq + Debug,
    {
        let path = self.path;
        let top = self.stack_pointer();
        let (params, expected) = returning;
        assert_eq!(self.call::<P, R>(name, params), Ok(expected), "{path}: {name}{params:?}");
        assert_eq!(self.stack_pointer(), top, "{path}: {name}{params:?} did not give its frame back");

        let first = self.call::<P, R>(name, refused);
        assert_eq!(first, Err(TrapCode::UnreachableCodeReached), "{path}: the refused {name}{refused:?}");
        let frame = top - self.stack_pointer();
        assert!(frame > 0, "{path}: a trapped {name} left the stack pointer at {top}");

        let mut traps: u32 = 1;
        let exhausted = loop {
            match self.call::<P, R>(name, refused) {
                Err(TrapCode::UnreachableCodeReached) => traps += 1,
                other => break other,
            }
            assert!(traps < TRAPS, "{path}: {TRAPS} traps of {frame} bytes each, and the stack still has room");
        };
        assert_eq!(exhausted, Err(TrapCode::MemoryOutOfBounds), "{path}: after {traps} traps of {frame} bytes each");
        let (leaked, top) = (i64::from(traps) * i64::from(frame), i64::from(top));
        let whole_stack = leaked <= top && top - leaked < i64::from(frame);
        assert!(whole_stack, "{path}: {traps} traps leaked {leaked} of {top} bytes");
    }
}
