use crate::Action;
use crate::canvas::mouse;
use crate::canvas::{Event, Geometry};
use crate::core::{Mouse, Rectangle};
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
        _mouse: Mouse,
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
        mouse: Mouse,
    ) -> Vec<Geometry<Renderer>>;

    /// Returns the current [`mouse::Cursor`] of the [`Program`].
    ///
    /// The cursor returned will be in effect even if the mouse position
    /// is out of bounds of the [`Canvas`].
    ///
    /// [`Canvas`]: crate::Canvas
    fn mouse_cursor(
        &self,
        _state: &Self::State,
        _bounds: Rectangle,
        _mouse: Mouse,
    ) -> mouse::Cursor {
        mouse::Cursor::default()
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
        mouse: Mouse,
    ) -> Option<Action<Message>> {
        T::update(self, state, event, bounds, mouse)
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        mouse: Mouse,
    ) -> Vec<Geometry<Renderer>> {
        T::draw(self, state, renderer, theme, bounds, mouse)
    }

    fn mouse_cursor(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        mouse: Mouse,
    ) -> mouse::Cursor {
        T::mouse_cursor(self, state, bounds, mouse)
    }
}
