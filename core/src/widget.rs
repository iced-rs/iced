//! Create custom widgets and operate on them.
pub mod operation;
pub mod text;
pub mod tree;

pub use crate::id::Id;
pub use operation::{Operation, OperationOutputWrapper};
pub use text::Text;
pub use tree::Tree;

use crate::event::{self, Event};
use crate::layout::{self, Layout};
use crate::mouse;
use crate::overlay;
use crate::renderer;
use crate::{Clipboard, Length, Point, Rectangle, Shell};

/// A component that displays information and allows interaction.
///
/// If you want to build your own widgets, you will need to implement this
/// trait.
///
/// # Examples
/// The repository has some [examples] showcasing how to implement a custom
/// widget:
///
/// - [`bezier_tool`], a Paint-like tool for drawing Bézier curves using
/// [`lyon`].
/// - [`custom_widget`], a demonstration of how to build a custom widget that
/// draws a circle.
/// - [`geometry`], a custom widget showcasing how to draw geometry with the
/// `Mesh2D` primitive in [`iced_wgpu`].
///
/// [examples]: https://github.com/iced-rs/iced/tree/0.9/examples
/// [`bezier_tool`]: https://github.com/iced-rs/iced/tree/0.9/examples/bezier_tool
/// [`custom_widget`]: https://github.com/iced-rs/iced/tree/0.9/examples/custom_widget
/// [`geometry`]: https://github.com/iced-rs/iced/tree/0.9/examples/geometry
/// [`lyon`]: https://github.com/nical/lyon
/// [`iced_wgpu`]: https://github.com/iced-rs/iced/tree/0.9/wgpu
pub trait Widget<Message, Renderer>
where
    Renderer: crate::Renderer,
{
    /// Returns the width of the [`Widget`].
    fn width(&self) -> Length;

    /// Returns the height of the [`Widget`].
    fn height(&self) -> Length;

    /// Returns the [`layout::Node`] of the [`Widget`].
    ///
    /// This [`layout::Node`] is used by the runtime to compute the [`Layout`] of the
    /// user interface.
    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node;

    /// Draws the [`Widget`] using the associated `Renderer`.
    fn draw(
        &self,
        state: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    );

    /// Returns the [`Tag`] of the [`Widget`].
    ///
    /// [`Tag`]: tree::Tag
    fn tag(&self) -> tree::Tag {
        tree::Tag::stateless()
    }

    /// Returns the [`State`] of the [`Widget`].
    ///
    /// [`State`]: tree::State
    fn state(&self) -> tree::State {
        tree::State::None
    }

    /// Returns the state [`Tree`] of the children of the [`Widget`].
    fn children(&self) -> Vec<Tree> {
        Vec::new()
    }

    /// Reconciliates the [`Widget`] with the provided [`Tree`].
    fn diff(&self, _tree: &mut Tree) {}

    /// Applies an [`Operation`] to the [`Widget`].
    fn operate(
        &self,
        _state: &mut Tree,
        _layout: Layout<'_>,
        _renderer: &Renderer,
        _operation: &mut dyn Operation<OperationOutputWrapper<Message>>,
    ) {
    }

    /// Processes a runtime [`Event`].
    ///
    /// By default, it does nothing.
    fn on_event(
        &mut self,
        _state: &mut Tree,
        _event: Event,
        _layout: Layout<'_>,
        _cursor_position: Point,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        _shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        event::Status::Ignored
    }

    /// Returns the current [`mouse::Interaction`] of the [`Widget`].
    ///
    /// By default, it returns [`mouse::Interaction::Idle`].
    fn mouse_interaction(
        &self,
        _state: &Tree,
        _layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse::Interaction::Idle
    }

    /// Returns the overlay of the [`Widget`], if there is any.
    fn overlay<'a>(
        &'a mut self,
        _state: &'a mut Tree,
        _layout: Layout<'_>,
        _renderer: &Renderer,
    ) -> Option<overlay::Element<'a, Message, Renderer>> {
        None
    }

    #[cfg(feature = "a11y")]
    /// get the a11y nodes for the widget and its children
    fn a11y_nodes(
        &self,
        _layout: Layout<'_>,
        _state: &Tree,
        _cursor_position: Point,
    ) -> iced_accessibility::A11yTree {
        iced_accessibility::A11yTree::default()
    }

    /// Returns the id of the widget
    fn id(&self) -> Option<Id> {
        None
    }
}
