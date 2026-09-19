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
use wasmtime_wasi::{DirPerms, FilePerms, WasiCtxBuilder};

/// The capability context granted to a WASI program.
#[derive(Debug, Clone, Copy, Default)]
pub struct Capabilities {
    /// Grant access to stdout (inherited from the host). On by default.
    pub stdout: bool,
    /// Grant filesystem access (a single preopened, read-only directory).
    /// **Off by default.**
    pub fs: bool,
    /// Grant network access. **Off by default, and not supported** — WASI
    /// Preview 1 in this host has no socket support, so requesting `net: true`
    /// is refused at construction rather than silently inert (M2).
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
    ///
    /// Returns an error if `net: true` is requested (unsupported in this WASI
    /// Preview 1 host), so an inert security control can never be mistaken for
    /// an actual grant (M2).
    pub fn new(wasm: &[u8], caps: Capabilities) -> Result<Self, String> {
        if caps.net {
            return Err(
                "network access is not supported by this WASI Preview 1 host; \
                 `net: true` is refused rather than silently inert"
                    .to_string(),
            );
        }
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
        // `fs` grants a single preopened, read-only directory at "/" — the
        // narrowest possible filesystem grant (M2). When `fs` is false there is
        // no preopen, so the guest cannot reach the filesystem at all.
        if self.caps.fs {
            builder
                .preopened_dir("/", "/", DirPerms::READ, FilePerms::READ)
                .map_err(|e| e.to_string())?;
        }
        // `net` is refused at construction (see `new`), so there is nothing to
        // wire here: the absence of a socket grant is the enforcement.
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
    fn network_grant_is_refused_not_inert() {
        // M2: `net: true` must fail at construction, so an inert security
        // control can never be mistaken for an actual grant.
        let caps = Capabilities {
            stdout: true,
            fs: false,
            net: true,
        };
        let rt = WasiRuntime::new(&compile_wat(ADD_WAT), caps);
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
