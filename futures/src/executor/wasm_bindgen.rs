use crate::Executor;

/// A `wasm-bindgen-futures` runtime.
#[derive(Debug)]
pub struct WasmBindgen;

impl Executor for WasmBindgen {
    fn new() -> Result<Self, futures::io::Error> {
        Ok(Self)
    }

    fn spawn(&self, future: impl futures::Future<Output = ()> + 'static) {
        wasm_bindgen_futures::spawn_local(future);
    }
}
