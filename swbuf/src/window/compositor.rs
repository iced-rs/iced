use crate::{Backend, surface::Surface};

use iced_graphics::{
    Color, Error, Viewport,
    compositor::{self, Information, SurfaceError},
};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::marker::PhantomData;

/// A window graphics backend for iced powered by `glow`.
pub struct Compositor<Theme> {
    theme: PhantomData<Theme>,
}

/// A graphics compositor that can draw to windows.
impl<Theme> compositor::Compositor for Compositor<Theme> {
    /// The settings of the backend.
    type Settings = crate::Settings;

    /// The iced renderer of the backend.
    type Renderer = crate::Renderer<Theme>;

    /// The surface of the backend.
    type Surface = Surface;

    /// Creates a new [`Compositor`].
    fn new<W: HasRawWindowHandle + HasRawDisplayHandle>(
        settings: Self::Settings,
        compatible_window: Option<&W>,
    ) -> Result<(Self, Self::Renderer), Error> {
        let compositor = Self {
            theme: PhantomData,
        };

        let renderer = Self::Renderer::new(Backend::new());

        Ok((compositor, renderer))
    }

    /// Crates a new [`Surface`] for the given window.
    ///
    /// [`Surface`]: Self::Surface
    fn create_surface<W: HasRawWindowHandle + HasRawDisplayHandle>(
        &mut self,
        window: &W,
    ) -> Self::Surface {
        Self::Surface::new(window)
    }

    /// Configures a new [`Surface`] with the given dimensions.
    ///
    /// [`Surface`]: Self::Surface
    fn configure_surface(
        &mut self,
        surface: &mut Self::Surface,
        width: u32,
        height: u32,
    ) {
        surface.configure(width, height);
    }

    /// Returns [`Information`] used by this [`Compositor`].
    fn fetch_information(&self) -> Information {
        todo!("Compositor::fetch_information");
    }

    /// Presents the [`Renderer`] primitives to the next frame of the given [`Surface`].
    ///
    /// [`Renderer`]: Self::Renderer
    /// [`Surface`]: Self::Surface
    fn present<T: AsRef<str>>(
        &mut self,
        renderer: &mut Self::Renderer,
        surface: &mut Self::Surface,
        viewport: &Viewport,
        background: Color,
        overlay: &[T],
    ) -> Result<(), SurfaceError> {
        surface.present(renderer, background);
        Ok(())
    }
}
