#[cfg_attr(target_arch = "wasm32", path = "web.rs")]
#[cfg_attr(not(target_arch = "wasm32"), path = "winit.rs")]
mod platform;
pub use platform::*;

pub struct Instance<A : Application> {
    pub platform : platform::Platform,
    instance : A,
}

impl<A : Application + 'static> Instance<A> {
    pub fn new(application : A) -> Self {
        let instance = application;
        Self{platform: platform::Platform::new(&instance), instance}
    }
    pub fn run(self) { self.platform.run(self.instance); }
}
