//! Bivdi WASI runtime — `D-004` native application format.
//!
//! Executes WASI components inside the Bivdi runtime. This makes the decided
//! "WASI components as the native application format" concrete: a WASI module
//! is loaded into a sandboxed engine and invoked through typed functions, never
//! through a shell or a global filesystem.
//!
//! # Provisional
//!
//! - Uses Wasmtime's engine. The full Bivdi ABI (RFC 0002 proposes WIT + the
//!   component model) is **not** wired here yet — that depends on accepting
//!   RFC 0002. This crate demonstrates the execution model only.
//! - No ambient authority: the engine grants no filesystem/network by default.

use wasmtime::{Engine, Func, Linker, Module, Store};

/// A compiled, runnable WASI module hosted in a Bivdi sandbox.
pub struct WasiRuntime {
    engine: Engine,
    linker: Linker<()>,
    module: Module,
}

impl WasiRuntime {
    /// Compile a WASI module (`.wasm` bytes) into a runnable instance.
    pub fn new(wasm: &[u8]) -> Result<Self, String> {
        let engine = Engine::default();
        let module = Module::new(&engine, wasm).map_err(|e| e.to_string())?;
        let linker = Linker::new(&engine);
        Ok(Self {
            engine,
            linker,
            module,
        })
    }

    /// The exported names of the compiled module.
    pub fn exports(&self) -> Vec<String> {
        self.module
            .exports()
            .map(|e| e.name().to_string())
            .collect()
    }

    /// Instantiate the module and call a `(i32, i32) -> i32` exported function
    /// by name (e.g. `add`). Per-invocation isolation: a fresh store each call.
    pub fn call_i32_i32(&mut self, name: &str, a: i32, b: i32) -> Result<i32, String> {
        let mut store = Store::new(&self.engine, ());
        let instance = self
            .linker
            .instantiate(&mut store, &self.module)
            .map_err(|e| e.to_string())?;
        let func: Func = instance
            .get_func(&mut store, name)
            .ok_or_else(|| format!("no exported function `{name}`"))?;
        let mut results = [wasmtime::Val::I32(0)];
        func.call(&mut store, &[a.into(), b.into()], &mut results)
            .map_err(|e| e.to_string())?;
        Ok(results[0].unwrap_i32())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A self-contained module: `(i32, i32) -> i32` returning a + b.
    const ADD_WAT: &str =
        "(module (func (export \"add\") (param i32 i32) (result i32) local.get 0 local.get 1 i32.add))";

    fn compile_wat(wat: &str) -> Vec<u8> {
        wat::parse_str(wat).unwrap()
    }

    #[test]
    fn compiles_an_empty_module() {
        let rt = WasiRuntime::new(b"\0asm\x01\0\0\0");
        assert!(rt.is_ok());
    }

    #[test]
    fn rejects_garbage() {
        let rt = WasiRuntime::new(b"not wasm");
        assert!(rt.is_err());
    }

    #[test]
    fn runs_an_add_function() {
        let wasm = compile_wat(ADD_WAT);
        let mut rt = WasiRuntime::new(&wasm).unwrap();
        assert!(rt.exports().contains(&"add".to_string()));
        assert_eq!(rt.call_i32_i32("add", 2, 3).unwrap(), 5);
    }
}
