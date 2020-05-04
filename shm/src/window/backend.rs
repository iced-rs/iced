use smithay_client_toolkit::shm;
use crate::{window::SwapChain, Renderer, Settings};
use iced_native::MouseCursor;

#[derive(derive_more::Deref, derive_more::DerefMut)] struct MemPool(shm::MemPool);
impl std::fmt::Debug for MemPool { fn fmt(&self, _: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> { unimplemented!() } }

/// A window graphics backend for iced
#[derive(Debug)]
pub struct Backend {
    pool: MemPool,
}

use smithay_client_toolkit::{environment::{Environment, GlobalHandler}, reexports::client::protocol::wl_shm::WlShm};

/// iced_native::window::Backend with 'new' env argument to pass &' Environment<:GlobalHandler<WlShm>>
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
    fn create_surface<W: iced_native::window::HasRawWindowHandle>(&mut self, window: &W) -> Self::Surface;
    ///
    fn create_swap_chain(&mut self, surface: &Self::Surface, width: u32, height: u32) -> Self::SwapChain;
    ///
    fn draw(&mut self, renderer: &mut Self::Renderer, swap_chain: &mut Self::SwapChain, output: &<Self::Renderer as iced_native::Renderer>::Output,
                                            scale_factor: f64, overlay: &[impl AsRef<str>]) -> MouseCursor;
}

impl /*iced_native::window::*/ShmBackend for Backend {
    type Settings = Settings;
    type Renderer = Renderer;
    type Surface = ();//RawWindowHandle;
    type SwapChain = SwapChain;

    fn new<E:GlobalHandler<WlShm>>(env: &Environment<E>, arguments: Settings) -> (Backend, Renderer) {
        let renderer = Renderer::new(&mut (), arguments);
        (Backend {pool: MemPool(env.create_simple_pool(|_|{}).unwrap())}, renderer)
    }

    fn create_surface<W: iced_native::window::HasRawWindowHandle>(&mut self, _: &W) -> Self::Surface { /*window.raw_window_handle()*/ }

    fn create_swap_chain(
        &mut self,
        surface: &Self::Surface,
        width: u32,
        height: u32,
    ) -> SwapChain {
        let stride = width*4;
        self.pool.resize((height*stride) as usize).unwrap();
        SwapChain::new(surface, width, height)
    }

    fn draw(
        &mut self,
        renderer: &mut Self::Renderer,
        swap_chain: &mut SwapChain,
        output: &<Self::Renderer as iced_native::Renderer>::Output,
        scale_factor: f64,
        overlay: &[impl AsRef<str>],
    ) -> MouseCursor {
        let (_frame, viewport) = swap_chain.next_frame().expect("Next frame");
        use framework::widget::{Target,WHITE};
        let mut frame = Target::from_bytes(self.pool.mmap(), viewport.dimensions().into());
        frame.set(|_| WHITE);
        renderer.draw(
            crate::Target {
                texture: &mut frame,
                viewport,
            },
            output,
            scale_factor,
            overlay,
        )
    }
}
