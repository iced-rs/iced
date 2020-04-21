use crate::canvas::Frame;

/// A type that can be drawn on a [`Frame`].
///
/// [`Frame`]: struct.Frame.html
pub trait Drawable {
    /// Draws the [`Drawable`] on the given [`Frame`].
    ///
    /// [`Drawable`]: trait.Drawable.html
    /// [`Frame`]: struct.Frame.html
    fn draw(&self, frame: &mut Frame);
}
