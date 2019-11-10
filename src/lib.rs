#[cfg_attr(target_arch = "wasm32", path = "web.rs")]
#[cfg_attr(not(target_arch = "wasm32"), path = "winit.rs")]
mod platform;
pub use platform::*;

pub struct Instance<A : Application> {
    pub platform : platform::Platform,
    application : A,
}

impl<A : Application + 'static> Instance<A> {
    pub fn new(application : A) -> Self { Self{platform: platform::Platform::new(&application), application} }
    pub fn run(self) -> Result<(), platform::Error> { self.platform.run(self.application) }
}
