//! Provide visual feedback to your users when performing a slow task.

use crate::{
    Element, Hasher, Layout, MouseCursor, Node, Point, Rectangle, Style, Widget,
};

use std::hash::Hash;

/// A bar that is filled based on an amount of progress.
///
/// It implements [`Widget`] when the associated `Renderer` implements the
/// [`progress_bar::Renderer`] trait.
///
/// [`Widget`]: ../trait.Widget.html
/// [`progress_bar::Renderer`]: trait.Renderer.html
///
/// # Example
///
/// ```
/// use iced::ProgressBar;
///
/// let progress = 0.75;
///
/// ProgressBar::new(progress);
/// ```
#[derive(Debug)]
pub struct ProgressBar {
    progress: f32,
    style: Style,
}

impl ProgressBar {
    /// Creates a new [`ProgressBar`] filled based on the given amount of
    /// progress.
    ///
    /// The progress should be in the `0.0..=1.0` range. `0` meaning no work
    /// done, and `1` meaning work finished.
    ///
    /// [`ProgressBar`]: struct.ProgressBar.html
    pub fn new(progress: f32) -> Self {
        ProgressBar {
            progress,
            style: Style::default().fill_width(),
        }
    }

    /// Sets the width of the [`ProgressBar`] in pixels.
    ///
    /// [`ProgressBar`]: struct.ProgressBar.html
    pub fn width(mut self, width: u16) -> Self {
        self.style = self.style.width(width);
        self
    }
}

impl<Message, Renderer> Widget<Message, Renderer> for ProgressBar
where
    Renderer: self::Renderer,
{
    fn node(&self, _renderer: &Renderer) -> Node {
        Node::new(self.style.height(50))
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        layout: Layout<'_>,
        _cursor_position: Point,
    ) -> MouseCursor {
        renderer.draw(layout.bounds(), self.progress);

        MouseCursor::OutOfBounds
    }

    fn hash_layout(&self, state: &mut Hasher) {
        self.style.hash(state);
    }
}

/// The renderer of a [`ProgressBar`].
///
/// Your [renderer] will need to implement this trait before being able to use
/// a [`ProgressBar`] in your user interface.
///
/// [`ProgressBar`]: struct.ProgressBar.html
/// [renderer]: ../../renderer/index.html
pub trait Renderer {
    /// Draws a [`ProgressBar`].
    ///
    /// It receives:
    ///   * the bounds of the [`ProgressBar`]
    ///   * the current progress of the [`ProgressBar`], in the `0.0..=1.0`
    ///   range.
    ///
    /// [`ProgressBar`]: struct.ProgressBar.html
    fn draw(&mut self, bounds: Rectangle, progress: f32);
}

impl<'a, Message, Renderer> From<ProgressBar> for Element<'a, Message, Renderer>
where
    Renderer: self::Renderer,
{
    fn from(progress_bar: ProgressBar) -> Element<'a, Message, Renderer> {
        Element::new(progress_bar)
    }
}
