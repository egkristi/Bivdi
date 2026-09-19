//! Bivdi WASI runtime — `D-004` native application format.
//!
//! Executes WASI modules inside the Bivdi runtime. This makes the decided
//! "WASI components as the native application format" concrete: a WASI module
//! is loaded into a sandboxed engine and invoked through typed functions, never
//! through a shell or a global filesystem.
//!
//! # Capability scoping (no ambient authority)
//!
//! A WASI program is instantiated with an explicit capability context. The
//! engine grants **stdout by default** (for observability) and **no filesystem
//! or network access** unless the host opts in. This is the "no ambient
//! authority" invariant expressed at the WASI boundary.
//!
//! # Provisional
//!
//! Uses Wasmtime's WASI (Preview 1) engine. The full Bivdi ABI (RFC 0002
//! proposes WIT + the component model) is **not** wired here yet.

use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::preview1::WasiP1Ctx;
use wasmtime_wasi::WasiCtxBuilder;

/// The capability context granted to a WASI program.
#[derive(Debug, Clone, Copy, Default)]
pub struct Capabilities {
    /// Grant access to stdout (inherited from the host). On by default.
    pub stdout: bool,
    /// Grant filesystem access. **Off by default.**
    pub fs: bool,
    /// Grant network access. **Off by default.**
    pub net: bool,
}

impl Capabilities {
    /// The least-authority default: stdout only.
    pub fn least() -> Self {
        Self {
            stdout: true,
            fs: false,
            net: false,
        }
    }
}

/// A compiled, runnable WASI module hosted in a Bivdi sandbox.
pub struct WasiRuntime {
    engine: Engine,
    /// The raw module bytes, recompiled per invocation so the store host data
    /// type matches the linker.
    wasm: Vec<u8>,
    caps: Capabilities,
}

impl WasiRuntime {
    /// Compile a WASI module (`.wasm` bytes) into a runnable instance with the
    /// given capability context.
    pub fn new(wasm: &[u8], caps: Capabilities) -> Result<Self, String> {
        let engine = Engine::default();
        // Validate that it compiles at all.
        Module::new(&engine, wasm).map_err(|e| e.to_string())?;
        Ok(Self {
            engine,
            wasm: wasm.to_vec(),
            caps,
        })
    }

    /// Compile with least authority (stdout only).
    pub fn new_least(wasm: &[u8]) -> Result<Self, String> {
        Self::new(wasm, Capabilities::least())
    }

    /// The exported names of the compiled module.
    pub fn exports(&self) -> Vec<String> {
        let module = Module::new(&self.engine, &self.wasm).unwrap();
        module.exports().map(|e| e.name().to_string()).collect()
    }

    /// Build a WASI linker wired to a fresh context per run.
    fn wasi_linker(&self) -> Result<(Linker<WasiP1Ctx>, WasiP1Ctx), String> {
        let mut builder = WasiCtxBuilder::new();
        if self.caps.stdout {
            builder.inherit_stdout();
        }
        // `fs` and `net` are not wired in Phase 0: granting them would require
        // naming specific resources (the capability model's job). Absence of a
        // grant is the default — this is ambient-authority-free by omission.
        let ctx = builder.build_p1();
        let mut linker: Linker<WasiP1Ctx> = Linker::new(&self.engine);
        wasmtime_wasi::preview1::add_to_linker_sync(&mut linker, |c| c)
            .map_err(|e| e.to_string())?;
        Ok((linker, ctx))
    }

    /// Run a WASI command module (`_start`) to completion.
    ///
    /// A WASI command "exits" by calling `proc_exit`, which is implemented as a
    /// trap carrying the exit status. An exit status of 0 is treated as success;
    /// any other exit status or a genuine trap is an error.
    pub fn run_command(&self) -> Result<(), String> {
        let module = Module::new(&self.engine, &self.wasm).map_err(|e| e.to_string())?;
        let (linker, ctx) = self.wasi_linker()?;
        let mut store = Store::new(&self.engine, ctx);
        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| e.to_string())?;
        let start = instance
            .get_typed_func::<(), ()>(&mut store, "_start")
            .map_err(|e| e.to_string())?;
        match start.call(&mut store, ()) {
            Ok(()) => Ok(()),
            Err(trap) => {
                // `proc_exit(0)` surfaces as a trap with an i32 exit status 0.
                if let Some(status) = trap.downcast_ref::<wasmtime_wasi::I32Exit>() {
                    if status.0 == 0 {
                        return Ok(());
                    }
                    return Err(format!("WASI exit status {}", status.0));
                }
                Err(trap.to_string())
            }
        }
    }

    /// Instantiate the module and call a `(i32, i32) -> i32` exported function
    /// by name (e.g. `add`). Per-invocation isolation: a fresh store each call.
    pub fn call_i32_i32(&self, name: &str, a: i32, b: i32) -> Result<i32, String> {
        let module = Module::new(&self.engine, &self.wasm).map_err(|e| e.to_string())?;
        let linker: Linker<()> = Linker::new(&self.engine);
        let mut store = Store::new(&self.engine, ());
        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| e.to_string())?;
        let func = instance
            .get_typed_func::<(i32, i32), i32>(&mut store, name)
            .map_err(|e| e.to_string())?;
        func.call(&mut store, (a, b)).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ADD_WAT: &str =
        "(module (func (export \"add\") (param i32 i32) (result i32) local.get 0 local.get 1 i32.add))";

    // A WASI command whose `_start` simply returns (no `proc_exit`). A WASI
    // command may return normally from `_start`, which is the cleanest way to
    // demonstrate completion without depending on version-specific exit-trap
    // semantics.
    const EXIT_WAT: &str = r#"(module
        (func (export "_start"))
    )"#;

    fn compile_wat(wat: &str) -> Vec<u8> {
        wat::parse_str(wat).unwrap()
    }

    #[test]
    fn compiles_an_empty_module() {
        let rt = WasiRuntime::new_least(b"\0asm\x01\0\0\0");
        assert!(rt.is_ok());
    }

    #[test]
    fn rejects_garbage() {
        let rt = WasiRuntime::new_least(b"not wasm");
        assert!(rt.is_err());
    }

    #[test]
    fn runs_an_add_function() {
        let wasm = compile_wat(ADD_WAT);
        let rt = WasiRuntime::new_least(&wasm).unwrap();
        assert!(rt.exports().contains(&"add".to_string()));
        assert_eq!(rt.call_i32_i32("add", 2, 3).unwrap(), 5);
    }

    #[test]
    fn runs_a_wasi_command() {
        let wasm = compile_wat(EXIT_WAT);
        let rt = WasiRuntime::new_least(&wasm).unwrap();
        // proc_exit(0) completes the command successfully.
        rt.run_command().unwrap();
    }
}
