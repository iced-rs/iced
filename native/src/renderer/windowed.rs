use crate::{Color, MouseCursor};

use raw_window_handle::HasRawWindowHandle;

/// A renderer that can target windows.
pub trait Windowed: super::Renderer + Sized {
    type Settings: Default;

    /// The type of target.
    type Target: Target<Renderer = Self>;

    /// Creates a new [`Windowed`] renderer.
    ///
    /// [`Windowed`]: trait.Windowed.html
    fn new(settings: Self::Settings) -> Self;

    /// Performs the drawing operations described in the output on the given
    /// target.
    ///
    /// The overlay can be a bunch of debug text logs. It should be rendered on
    /// top of the GUI on most scenarios.
    fn draw<T: AsRef<str>>(
        &mut self,
        clear_color: Color,
        output: &Self::Output,
        overlay: &[T],
        target: &mut Self::Target,
    ) -> MouseCursor;
}

/// A rendering target.
pub trait Target {
    /// The renderer of this target.
    type Renderer;

    /// Creates a new rendering [`Target`] from the given window handle, width,
    /// height and dpi factor.
    ///
    /// [`Target`]: trait.Target.html
    fn new<W: HasRawWindowHandle>(
        window: &W,
        width: u16,
        height: u16,
        dpi: f32,
        renderer: &Self::Renderer,
    ) -> Self;

    /// Resizes the current [`Target`].
    ///
    /// [`Target`]: trait.Target.html
    fn resize(
        &mut self,
        width: u16,
        height: u16,
        dpi: f32,
        renderer: &Self::Renderer,
    );
}
