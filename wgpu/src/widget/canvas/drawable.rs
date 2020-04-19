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

impl<'a> Drawable for dyn Fn(&mut Frame) + 'a {
    fn draw(&self, frame: &mut Frame) {
        self(frame)
    }
}

impl<T> Drawable for &T
where
    T: Drawable,
{
    fn draw(&self, frame: &mut Frame) {
        T::draw(self, frame)
    }
}
