//! Display rendering results on windows.
use smithay_client_toolkit::{environment::{Environment, GlobalHandler}, reexports::client::protocol::wl_shm::WlShm};
use iced_native::mouse::Interaction;

/// iced_native::window::Backend:
/// 'new' env argument to pass &' Environment<:GlobalHandler<WlShm>>
/// Feature gate 'create_surface'
pub trait ShmBackend: Sized {
    ///
    type Settings;
    ///
    type Renderer: iced_native::Renderer;
    ///
    type Surface;
    ///
    type SwapChain;
    ///
    fn new<E:GlobalHandler<WlShm>>(env: &Environment<E>, settings: Self::Settings) -> (Self, Self::Renderer);
    ///
    #[cfg(feature="wayland-client/use_system_lib")]
    fn create_surface<W: iced_native::window::HasRawWindowHandle>(&mut self, window: &W) -> Self::Surface;
    ///
    fn create_swap_chain(&mut self, surface: &Self::Surface, width: u32, height: u32) -> Self::SwapChain;
    ///
    fn draw(&mut self, renderer: &mut Self::Renderer, swap_chain: &mut Self::SwapChain, output: &<Self::Renderer as iced_native::Renderer>::Output,
                                            scale_factor: f64, overlay: &[impl AsRef<str>]) -> Interaction;
}

mod backend;
mod swap_chain;

pub use backend::{Backend};
pub use swap_chain::SwapChain;
