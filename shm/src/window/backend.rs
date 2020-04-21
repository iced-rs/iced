use crate::{window::SwapChain, Renderer, Settings, Target};

use iced_native::MouseCursor;

/// A window graphics backend for iced
#[derive(Debug)]
pub struct Backend {}

impl iced_sctk::window_ext::NoHasRawWindowHandleBackend for Backend {
    /// Creates a new [`Surface`] for the given window.
    ///
    /// [`Surface`]: #associatedtype.Surface
    fn create_surface<W>(&mut self, _window: &W) -> Self::Surface {
        ()
    }
}

impl iced_native::window::Backend for Backend {
    type Settings = Settings;
    type Renderer = Renderer;
    type Surface = ();
    type SwapChain = SwapChain;

    fn new(settings: Self::Settings) -> (Backend, Renderer) {
        let renderer = Renderer::new(&mut (), settings);
        (Backend {}, renderer)
    }

    fn create_surface<W: iced_native::window::HasRawWindowHandle>(
        &mut self,
        _window: &W,
    ) -> Self::Surface {
        ()
    }

    fn create_swap_chain(
        &mut self,
        surface: &Self::Surface,
        width: u32,
        height: u32,
    ) -> SwapChain {
        SwapChain::new(surface, width, height)
    }

    fn draw<T: AsRef<str>>(
        &mut self,
        renderer: &mut Self::Renderer,
        swap_chain: &mut SwapChain,
        output: &<Self::Renderer as iced_native::Renderer>::Output,
        scale_factor: f64,
        overlay: &[T],
    ) -> MouseCursor {
        let (frame, viewport) = swap_chain.next_frame().expect("Next frame");
        // TODO: Clear white
        let mouse_cursor = renderer.draw(
            Target {
                texture: &frame,
                viewport,
            },
            output,
            scale_factor,
            overlay,
        );
        mouse_cursor
    }
}
