use crate::canvas::event::{self, Event};
use crate::canvas::{Cursor, Geometry};
use iced_native::{mouse, Rectangle};

/// The state and logic of a [`Canvas`].
///
/// A [`Program`] can mutate internal state and produce messages for an
/// application.
///
/// [`Canvas`]: crate::widget::Canvas
pub trait Program<Message> {
    /// Updates the state of the [`Program`].
    ///
    /// When a [`Program`] is used in a [`Canvas`], the runtime will call this
    /// method for each [`Event`].
    ///
    /// This method can optionally return a `Message` to notify an application
    /// of any meaningful interactions.
    ///
    /// By default, this method does and returns nothing.
    ///
    /// [`Canvas`]: crate::widget::Canvas
    fn update(
        &mut self,
        _event: Event,
        _bounds: Rectangle,
        _cursor: Cursor,
    ) -> (event::Status, Option<Message>) {
        (event::Status::Ignored, None)
    }

    /// Draws the state of the [`Program`], producing a bunch of [`Geometry`].
    ///
    /// [`Geometry`] can be easily generated with a [`Frame`] or stored in a
    /// [`Cache`].
    ///
    /// [`Frame`]: crate::widget::canvas::Frame
    /// [`Cache`]: crate::widget::canvas::Cache
    fn draw(&self, bounds: Rectangle, cursor: Cursor) -> Vec<Geometry>;

    /// Returns the current mouse interaction of the [`Program`].
    ///
    /// The interaction returned will be in effect even if the cursor position
    /// is out of bounds of the program's [`Canvas`].
    ///
    /// [`Canvas`]: crate::widget::Canvas
    fn mouse_interaction(
        &self,
        _bounds: Rectangle,
        _cursor: Cursor,
    ) -> mouse::Interaction {
        mouse::Interaction::default()
    }
}

impl<T, Message> Program<Message> for &mut T
where
    T: Program<Message>,
{
    fn update(
        &mut self,
        event: Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (event::Status, Option<Message>) {
        T::update(self, event, bounds, cursor)
    }

    fn draw(&self, bounds: Rectangle, cursor: Cursor) -> Vec<Geometry> {
        T::draw(self, bounds, cursor)
    }

    fn mouse_interaction(
        &self,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> mouse::Interaction {
        T::mouse_interaction(self, bounds, cursor)
    }
}
