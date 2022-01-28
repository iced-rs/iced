//! A `wasm-bindgein-futures` backend.

/// A `wasm-bindgen-futures` executor.
#[derive(Debug)]
pub struct Executor;

impl crate::Executor for Executor {
    fn new() -> Result<Self, futures::io::Error> {
        Ok(Self)
    }

    fn spawn(&self, future: impl futures::Future<Output = ()> + 'static) {
        wasm_bindgen_futures::spawn_local(future);
    }
}
