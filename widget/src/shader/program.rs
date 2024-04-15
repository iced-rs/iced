use crate::core::event;
use crate::core::mouse;
use crate::core::{Rectangle, Shell};
use crate::renderer::wgpu::Primitive;
use crate::shader;

/// The state and logic of a [`Shader`] widget.
///
/// A [`Program`] can mutate the internal state of a [`Shader`] widget
/// and produce messages for an application.
///
/// [`Shader`]: crate::Shader
pub trait Program<Message> {
    /// The internal state of the [`Program`].
    type State: Default + 'static;

    /// The type of primitive this [`Program`] can draw.
    type Primitive: Primitive + 'static;

    /// Update the internal [`State`] of the [`Program`]. This can be used to reflect state changes
    /// based on mouse & other events. You can use the [`Shell`] to publish messages, request a
    /// redraw for the window, etc.
    ///
    /// By default, this method does and returns nothing.
    ///
    /// [`State`]: Self::State
    fn update(
        &self,
        _state: &mut Self::State,
        _event: shader::Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
        _shell: &mut Shell<'_, Message>,
    ) -> (event::Status, Option<Message>) {
        (event::Status::Ignored, None)
    }

    /// Draws the [`Primitive`].
    ///
    /// [`Primitive`]: Self::Primitive
    fn draw(
        &self,
        state: &Self::State,
        cursor: mouse::Cursor,
        bounds: Rectangle,
    ) -> Self::Primitive;

    /// Returns the current mouse interaction of the [`Program`].
    ///
    /// The interaction returned will be in effect even if the cursor position is out of
    /// bounds of the [`Shader`]'s program.
    ///
    /// [`Shader`]: crate::Shader
    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        mouse::Interaction::default()
    }
}
