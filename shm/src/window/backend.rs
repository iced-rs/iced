use crate::{window::SwapChain, Renderer, Settings};
use iced_native::mouse::Interaction;
use smithay_client_toolkit::shm;
#[derive(derive_more::Deref, derive_more::DerefMut)] struct MemPool(shm::MemPool);
impl std::fmt::Debug for MemPool { fn fmt(&self, _: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> { unimplemented!() } }

/// A window graphics backend for iced
#[derive(Debug)]
pub struct Backend {
    pool: MemPool,
}

use smithay_client_toolkit::{environment::{Environment, GlobalHandler}, reexports::client::protocol::wl_shm::WlShm};
use smithay_client_toolkit::reexports::client::{protocol::wl_surface::WlSurface};

impl /*iced_native::window::*/super::ShmBackend for Backend {
    type Settings = Settings;
    type Renderer = Renderer;
    type Surface = WlSurface;
    type SwapChain = SwapChain;

    fn new<E:GlobalHandler<WlShm>>(env: &Environment<E>, arguments: Settings) -> (Backend, Renderer) {
        let renderer = Renderer::new(&mut (), arguments);
        (Backend {pool: MemPool(env.create_simple_pool(|_|{}).unwrap())}, renderer)
    }

    #[cfg(feature="wayland-client/use_system_lib")]
    fn create_surface<W: iced_native::window::HasRawWindowHandle>(&mut self, window: &W) -> Self::Surface {
        use raw_window_handle::{RawWindowHandle::Wayland, unix::WaylandHandle};
        if let Wayland(WaylandHandle{surface, ..}) = window.raw_window_handle() {
            use smithay_client_toolkit::reexports::client::{Proxy, sys::client::wl_proxy};
            #[allow(unsafe_code)] unsafe{Proxy::from_c_ptr(surface as *mut wl_proxy)}.into()
        } else {
            unreachable!()
        }
    }

    fn create_swap_chain(
        &mut self,
        surface: &Self::Surface,
        width: u32,
        height: u32,
    ) -> SwapChain {
        let stride = width*4;
        self.pool.resize((height*stride) as usize).unwrap();
        log::trace!("Pool size: {}x{}", width, height);
        SwapChain::new(surface.clone(), width, height)
    }

    fn draw(
        &mut self,
        renderer: &mut Self::Renderer,
        swap_chain: &mut SwapChain,
        output: &<Self::Renderer as iced_native::Renderer>::Output,
        scale_factor: f64,
        overlay: &[impl AsRef<str>],
    ) -> Interaction{
        let (surface, viewport) = swap_chain.next_frame().expect("Next frame");
        use framework::widget::{Target, WHITE};
        let size : framework::size2 = viewport.dimensions().into();
        let stride = size.x*4;
        let mut frame = Target::from_bytes(&mut self.pool.mmap()[..(size.y*stride) as usize], size); // Pool never shrinks
        frame.set(|_| WHITE);
        let cursor = renderer.draw(
            crate::Target {
                texture: &mut frame,
                viewport,
            },
            output,
            scale_factor,
            overlay,
        );
        let stride = size.x*4;
        let buffer = self.pool.buffer(0, size.x as i32, size.y as i32, stride as i32, shm::Format::Argb8888);
        surface.attach(Some(&buffer), 0, 0);
        surface.damage_buffer(0, 0, size.x as i32, size.y as i32);
        surface.commit();
        log::trace!("Frame {:?}", size);
        cursor
    }
}
