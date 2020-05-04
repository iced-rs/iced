use crate::canvas::{Cursor, Event, Geometry};
use iced_native::{mouse, Rectangle};

/// The state and logic of a [`Canvas`].
///
/// A [`Program`] can mutate internal state and produce messages for an
/// application.
///
/// [`Canvas`]: struct.Canvas.html
/// [`Program`]: trait.Program.html
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
    /// [`Program`]: trait.Program.html
    /// [`Canvas`]: struct.Canvas.html
    /// [`Event`]: enum.Event.html
    fn update(
        &mut self,
        _event: Event,
        _bounds: Rectangle,
        _cursor: Cursor,
    ) -> Option<Message> {
        None
    }

    /// Draws the state of the [`Program`], producing a bunch of [`Geometry`].
    ///
    /// [`Geometry`] can be easily generated with a [`Frame`] or stored in a
    /// [`Cache`].
    ///
    /// [`Program`]: trait.Program.html
    /// [`Geometry`]: struct.Geometry.html
    /// [`Frame`]: struct.Frame.html
    /// [`Cache`]: struct.Cache.html
    fn draw(&self, bounds: Rectangle, cursor: Cursor) -> Vec<Geometry>;

    /// Returns the current mouse interaction of the [`Program`].
    ///
    /// The interaction returned will be in effect even if the cursor position
    /// is out of bounds of the program's [`Canvas`].
    ///
    /// [`Program`]: trait.Program.html
    /// [`Canvas`]: struct.Canvas.html
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
    ) -> Option<Message> {
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
