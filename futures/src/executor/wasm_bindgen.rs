use crate::Executor;

#[derive(Debug)]
pub struct WasmBindgen;

impl Executor for WasmBindgen {
    fn new() -> Result<Self, futures::io::Error> {
        Ok(Self)
    }

    fn spawn(
        &self,
        future: impl futures::Future<Output = ()> + Send + 'static,
    ) {
        wasm_bindgen_futures::spawn_local(future);
    }
}
