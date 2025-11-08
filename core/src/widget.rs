//! Create custom widgets and operate on them.
pub mod operation;
pub mod text;
pub mod tree;

mod id;

pub use id::Id;
pub use operation::Operation;
pub use text::Text;
pub use tree::Tree;

use crate::layout::{self, Layout};
use crate::mouse;
use crate::overlay;
use crate::renderer;
use crate::{Clipboard, Event, Length, Rectangle, Shell, Size, Vector};

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
///   [`lyon`].
/// - [`custom_widget`], a demonstration of how to build a custom widget that
///   draws a circle.
/// - [`geometry`], a custom widget showcasing how to draw geometry with the
///   `Mesh2D` primitive in [`iced_wgpu`].
///
/// [examples]: https://github.com/iced-rs/iced/tree/0.13/examples
/// [`bezier_tool`]: https://github.com/iced-rs/iced/tree/0.13/examples/bezier_tool
/// [`custom_widget`]: https://github.com/iced-rs/iced/tree/0.13/examples/custom_widget
/// [`geometry`]: https://github.com/iced-rs/iced/tree/0.13/examples/geometry
/// [`lyon`]: https://github.com/nical/lyon
/// [`iced_wgpu`]: https://github.com/iced-rs/iced/tree/0.13/wgpu
pub trait Widget<Message, Theme, Renderer>
where
    Renderer: crate::Renderer,
{
    /// Returns the [`Size`] of the [`Widget`] in lengths.
    fn size(&self) -> Size<Length>;

    /// Returns a [`Size`] hint for laying out the [`Widget`].
    ///
    /// This hint may be used by some widget containers to adjust their sizing strategy
    /// during construction.
    fn size_hint(&self) -> Size<Length> {
        self.size()
    }

    /// Returns the [`layout::Node`] of the [`Widget`].
    ///
    /// This [`layout::Node`] is used by the runtime to compute the [`Layout`] of the
    /// user interface.
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node;

    /// Draws the [`Widget`] using the associated `Renderer`.
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
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

    /// Reconciles the [`Widget`] with the provided [`Tree`].
    fn diff(&self, tree: &mut Tree) {
        tree.children.clear();
    }

    /// Applies an [`Operation`] to the [`Widget`].
    fn operate(
        &mut self,
        _state: &mut Tree,
        _layout: Layout<'_>,
        _renderer: &Renderer,
        _operation: &mut dyn Operation,
    ) {
    }

    /// Processes a runtime [`Event`].
    ///
    /// By default, it does nothing.
    fn update(
        &mut self,
        _state: &mut Tree,
        _event: &Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        _shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
    }

    /// Returns the current [`mouse::Interaction`] of the [`Widget`].
    ///
    /// By default, it returns [`mouse::Interaction::Idle`].
    fn mouse_interaction(
        &self,
        _state: &Tree,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse::Interaction::None
    }

    /// Returns the overlay of the [`Widget`], if there is any.
    fn overlay<'a>(
        &'a mut self,
        _state: &'a mut Tree,
        _layout: Layout<'a>,
        _renderer: &Renderer,
        _viewport: &Rectangle,
        _translation: Vector,
    ) -> Option<overlay::Element<'a, Message, Theme, Renderer>> {
        None
    }

    /// Returns accessibility information for this [`Widget`].
    ///
    /// By default, returns `None`, making the widget invisible to
    /// accessibility tools. Widgets should override this to provide
    /// meaningful accessibility information.
    ///
    /// # Return Value
    /// - `Some(node)`: This widget should appear in the accessibility tree
    /// - `None`: This widget is transparent to accessibility (layout-only)
    ///
    /// # When to return `None`
    /// Return `None` for:
    /// - Pure layout containers (Container, Column, Row)
    /// - Spacing widgets (Space)
    /// - Decorative elements with no semantic meaning
    ///
    /// # When to return `Some`
    /// Return `Some(node)` for:
    /// - Interactive widgets (Button, TextInput, Checkbox)
    /// - Informational content (Text, Image with alt text)
    /// - Semantic containers (List, Table, Group)
    ///
    /// # Example
    /// ```
    /// use iced_core::accessibility::{AccessibilityNode, Role};
    /// use iced_core::widget::{Tree, Widget};
    /// use iced_core::{Layout, Rectangle};
    ///
    /// # struct MyButton;
    /// # impl<Message, Theme, Renderer: iced_core::Renderer> Widget<Message, Theme, Renderer> for MyButton {
    /// #     fn size(&self) -> iced_core::Size<iced_core::Length> { iced_core::Size::new(iced_core::Length::Shrink, iced_core::Length::Shrink) }
    /// #     fn layout(&mut self, _: &mut Tree, _: &Renderer, _: &iced_core::layout::Limits) -> iced_core::layout::Node {
    /// #         iced_core::layout::Node::new(iced_core::Size::ZERO)
    /// #     }
    /// #     fn draw(&self, _: &Tree, _: &mut Renderer, _: &Theme, _: &iced_core::renderer::Style, _: Layout<'_>, _: iced_core::mouse::Cursor, _: &Rectangle) {}
    /// fn accessibility(
    ///     &self,
    ///     _state: &Tree,
    ///     layout: Layout<'_>,
    /// ) -> Option<AccessibilityNode> {
    ///     Some(
    ///         AccessibilityNode::new(layout.bounds())
    ///             .role(Role::Button)
    ///             .label("Click me")
    ///             .focusable(true)
    ///     )
    /// }
    /// # }
    /// ```
    fn accessibility(
        &self,
        _state: &Tree,
        _layout: Layout<'_>,
    ) -> Option<crate::accessibility::AccessibilityNode> {
        None
    }
}
