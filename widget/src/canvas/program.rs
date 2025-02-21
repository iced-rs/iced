use crate::Action;
use crate::canvas::mouse;
use crate::canvas::{Event, Geometry};
use crate::core::Rectangle;
use crate::graphics::geometry;

/// The state and logic of a [`Canvas`].
///
/// A [`Program`] can mutate internal state and produce messages for an
/// application.
///
/// [`Canvas`]: crate::Canvas
pub trait Program<Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    Renderer: geometry::Renderer,
{
    /// The internal state mutated by the [`Program`].
    type State: Default + 'static;

    /// Updates the [`State`](Self::State) of the [`Program`].
    ///
    /// When a [`Program`] is used in a [`Canvas`], the runtime will call this
    /// method for each [`Event`].
    ///
    /// This method can optionally return an [`Action`] to either notify an
    /// application of any meaningful interactions, capture the event, or
    /// request a redraw.
    ///
    /// By default, this method does and returns nothing.
    ///
    /// [`Canvas`]: crate::Canvas
    fn update(
        &self,
        _state: &mut Self::State,
        _event: &Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        None
    }

    /// Draws the state of the [`Program`], producing a bunch of [`Geometry`].
    ///
    /// [`Geometry`] can be easily generated with a [`Frame`] or stored in a
    /// [`Cache`].
    ///
    /// [`Geometry`]: crate::canvas::Geometry
    /// [`Frame`]: crate::canvas::Frame
    /// [`Cache`]: crate::canvas::Cache
    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry<Renderer>>;

    /// Returns the current mouse interaction of the [`Program`].
    ///
    /// The interaction returned will be in effect even if the cursor position
    /// is out of bounds of the program's [`Canvas`].
    ///
    /// [`Canvas`]: crate::Canvas
    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        mouse::Interaction::default()
    }
}

impl<Message, Theme, Renderer, T> Program<Message, Theme, Renderer> for &T
where
    Renderer: geometry::Renderer,
    T: Program<Message, Theme, Renderer>,
{
    type State = T::State;

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        T::update(self, state, event, bounds, cursor)
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry<Renderer>> {
        T::draw(self, state, renderer, theme, bounds, cursor)
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        T::mouse_interaction(self, state, bounds, cursor)
    }
}
