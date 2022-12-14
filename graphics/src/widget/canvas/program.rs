use crate::text;
use crate::widget::canvas::event::{self, Event};
use crate::widget::canvas::mouse;
use crate::widget::canvas::{Cursor, Geometry};
use crate::Rectangle;

/// The state and logic of a [`Canvas`].
///
/// A [`Program`] can mutate internal state and produce messages for an
/// application.
///
/// [`Canvas`]: crate::widget::Canvas
pub trait Program<Message, Theme = iced_native::Theme> {
    /// The internal state mutated by the [`Program`].
    type State: Default + 'static;

    /// Updates the [`State`](Self::State) of the [`Program`].
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
        &self,
        _state: &mut Self::State,
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
    fn draw(
        &self,
        state: &Self::State,
        theme: &Theme,
        text_cache: &text::Cache,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Vec<Geometry>;

    /// Returns the current mouse interaction of the [`Program`].
    ///
    /// The interaction returned will be in effect even if the cursor position
    /// is out of bounds of the program's [`Canvas`].
    ///
    /// [`Canvas`]: crate::widget::Canvas
    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: Rectangle,
        _cursor: Cursor,
    ) -> mouse::Interaction {
        mouse::Interaction::default()
    }
}

impl<Message, Theme, T> Program<Message, Theme> for &T
where
    T: Program<Message, Theme>,
{
    type State = T::State;

    fn update(
        &self,
        state: &mut Self::State,
        event: Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (event::Status, Option<Message>) {
        T::update(self, state, event, bounds, cursor)
    }

    fn draw(
        &self,
        state: &Self::State,
        theme: &Theme,
        text_cache: &text::Cache,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Vec<Geometry> {
        T::draw(self, state, theme, text_cache, bounds, cursor)
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> mouse::Interaction {
        T::mouse_interaction(self, state, bounds, cursor)
    }
}
