//! A compositor is responsible for initializing a renderer and managing window
//! surfaces.
use crate::core::Color;
use crate::futures::{MaybeSend, MaybeSync};
use crate::{Error, Settings, Viewport};

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use thiserror::Error;

use std::borrow::Cow;

/// A graphics compositor that can draw to windows.
pub trait Compositor: Sized {
    /// The iced renderer of the backend.
    type Renderer;

    /// The surface of the backend.
    type Surface;

    /// Creates a new [`Compositor`].
    fn new<W: Window + Clone>(
        settings: Settings,
        compatible_window: W,
    ) -> impl Future<Output = Result<Self, Error>> {
        Self::with_backend(settings, compatible_window, None)
    }

    /// Creates a new [`Compositor`] with a backend preference.
    ///
    /// If the backend does not match the preference, it will return
    /// [`Error::GraphicsAdapterNotFound`].
    fn with_backend<W: Window + Clone>(
        _settings: Settings,
        _compatible_window: W,
        _backend: Option<&str>,
    ) -> impl Future<Output = Result<Self, Error>>;

    /// Creates a [`Self::Renderer`] for the [`Compositor`].
    fn create_renderer(&self) -> Self::Renderer;

    /// Crates a new [`Surface`] for the given window.
    ///
    /// [`Surface`]: Self::Surface
    fn create_surface<W: Window + Clone>(
        &mut self,
        window: W,
        width: u32,
        height: u32,
    ) -> Self::Surface;

    /// Configures a new [`Surface`] with the given dimensions.
    ///
    /// [`Surface`]: Self::Surface
    fn configure_surface(
        &mut self,
        surface: &mut Self::Surface,
        width: u32,
        height: u32,
    );

    /// Returns [`Information`] used by this [`Compositor`].
    fn fetch_information(&self) -> Information;

    /// Loads a font from its bytes.
    fn load_font(&mut self, font: Cow<'static, [u8]>) {
        crate::text::font_system()
            .write()
            .expect("Write to font system")
            .load_font(font);
    }

    /// Presents the [`Renderer`] primitives to the next frame of the given [`Surface`].
    ///
    /// [`Renderer`]: Self::Renderer
    /// [`Surface`]: Self::Surface
    fn present(
        &mut self,
        renderer: &mut Self::Renderer,
        surface: &mut Self::Surface,
        viewport: &Viewport,
        background_color: Color,
        on_pre_present: impl FnOnce(),
    ) -> Result<(), SurfaceError>;

    /// Screenshots the current [`Renderer`] primitives to an offscreen texture, and returns the bytes of
    /// the texture ordered as `RGBA` in the `sRGB` color space.
    ///
    /// [`Renderer`]: Self::Renderer
    fn screenshot(
        &mut self,
        renderer: &mut Self::Renderer,
        viewport: &Viewport,
        background_color: Color,
    ) -> Vec<u8>;
}

/// A window that can be used in a [`Compositor`].
///
/// This is just a convenient super trait of the `raw-window-handle`
/// traits.
pub trait Window:
    HasWindowHandle + HasDisplayHandle + MaybeSend + MaybeSync + 'static
{
}

impl<T> Window for T where
    T: HasWindowHandle + HasDisplayHandle + MaybeSend + MaybeSync + 'static
{
}

/// Defines the default compositor of a renderer.
pub trait Default {
    /// The compositor of the renderer.
    type Compositor: Compositor<Renderer = Self>;
}

/// Result of an unsuccessful call to [`Compositor::present`].
#[derive(Clone, PartialEq, Eq, Debug, Error)]
pub enum SurfaceError {
    /// A timeout was encountered while trying to acquire the next frame.
    #[error("A timeout was encountered while trying to acquire the next frame")]
    Timeout,
    /// The underlying surface has changed, and therefore the surface must be updated.
    #[error(
        "The underlying surface has changed, and therefore the surface must be updated."
    )]
    Outdated,
    /// The swap chain has been lost and needs to be recreated.
    #[error("The surface has been lost and needs to be recreated")]
    Lost,
    /// There is no more memory left to allocate a new frame.
    #[error("There is no more memory left to allocate a new frame")]
    OutOfMemory,
    /// Acquiring a texture failed with a generic error.
    #[error("Acquiring a texture failed with a generic error")]
    Other,
}

/// Contains information about the graphics (e.g. graphics adapter, graphics backend).
#[derive(Debug)]
pub struct Information {
    /// Contains the graphics adapter.
    pub adapter: String,
    /// Contains the graphics backend.
    pub backend: String,
}

#[cfg(debug_assertions)]
impl Compositor for () {
    type Renderer = ();
    type Surface = ();

    async fn with_backend<W: Window + Clone>(
        _settings: Settings,
        _compatible_window: W,
        _preferred_backend: Option<&str>,
    ) -> Result<Self, Error> {
        Ok(())
    }

    fn create_renderer(&self) -> Self::Renderer {}

    fn create_surface<W: Window + Clone>(
        &mut self,
        _window: W,
        _width: u32,
        _height: u32,
    ) -> Self::Surface {
    }

    fn configure_surface(
        &mut self,
        _surface: &mut Self::Surface,
        _width: u32,
        _height: u32,
    ) {
    }

    fn load_font(&mut self, _font: Cow<'static, [u8]>) {}

    fn fetch_information(&self) -> Information {
        Information {
            adapter: String::from("Null Renderer"),
            backend: String::from("Null"),
        }
    }

    fn present(
        &mut self,
        _renderer: &mut Self::Renderer,
        _surface: &mut Self::Surface,
        _viewport: &Viewport,
        _background_color: Color,
        _on_pre_present: impl FnOnce(),
    ) -> Result<(), SurfaceError> {
        Ok(())
    }

    fn screenshot(
        &mut self,
        _renderer: &mut Self::Renderer,
        _viewport: &Viewport,
        _background_color: Color,
    ) -> Vec<u8> {
        vec![]
    }
}

#[cfg(debug_assertions)]
impl Default for () {
    type Compositor = ();
}
