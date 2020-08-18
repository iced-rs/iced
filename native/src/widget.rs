//! Use the built-in widgets or create your own.
//!
//! # Built-in widgets
//! Every built-in drawable widget has its own module with a `Renderer` trait
//! that must be implemented by a [renderer] before being able to use it as
//! a [`Widget`].
//!
//! # Custom widgets
//! If you want to implement a custom widget, you simply need to implement the
//! [`Widget`] trait. You can use the API of the built-in widgets as a guide or
//! source of inspiration.
//!
//! # Re-exports
//! For convenience, the contents of this module are available at the root
//! module. Therefore, you can directly type:
//!
//! ```
//! use iced_native::{button, Button, Widget};
//! ```
//!
//! [`Widget`]: trait.Widget.html
//! [renderer]: ../renderer/index.html
pub mod button;
pub mod checkbox;
pub mod column;
pub mod container;
pub mod image;
pub mod pane_grid;
pub mod pick_list;
pub mod progress_bar;
pub mod radio;
pub mod row;
pub mod rule;
pub mod scrollable;
pub mod slider;
pub mod space;
pub mod svg;
pub mod text;
pub mod text_input;

#[doc(no_inline)]
pub use button::Button;
#[doc(no_inline)]
pub use checkbox::Checkbox;
#[doc(no_inline)]
pub use column::Column;
#[doc(no_inline)]
pub use container::Container;
#[doc(no_inline)]
pub use image::Image;
#[doc(no_inline)]
pub use pane_grid::PaneGrid;
#[doc(no_inline)]
pub use pick_list::PickList;
#[doc(no_inline)]
pub use progress_bar::ProgressBar;
#[doc(no_inline)]
pub use radio::Radio;
#[doc(no_inline)]
pub use row::Row;
#[doc(no_inline)]
pub use rule::Rule;
#[doc(no_inline)]
pub use scrollable::Scrollable;
#[doc(no_inline)]
pub use slider::Slider;
#[doc(no_inline)]
pub use space::Space;
#[doc(no_inline)]
pub use svg::Svg;
#[doc(no_inline)]
pub use text::Text;
#[doc(no_inline)]
pub use text_input::TextInput;

use crate::{
    layout, overlay, Clipboard, Event, Hasher, Layout, Length, Point, Rectangle,
};

/// A component that displays information and allows interaction.
///
/// If you want to build your own widgets, you will need to implement this
/// trait.
///
/// [`Widget`]: trait.Widget.html
/// [`Element`]: ../struct.Element.html
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
/// [examples]: https://github.com/hecrj/iced/tree/0.1/examples
/// [`bezier_tool`]: https://github.com/hecrj/iced/tree/0.1/examples/bezier_tool
/// [`custom_widget`]: https://github.com/hecrj/iced/tree/0.1/examples/custom_widget
/// [`geometry`]: https://github.com/hecrj/iced/tree/0.1/examples/geometry
/// [`lyon`]: https://github.com/nical/lyon
/// [`iced_wgpu`]: https://github.com/hecrj/iced/tree/0.1/wgpu
pub trait Widget<Message, Renderer>
where
    Renderer: crate::Renderer,
{
    /// Returns the width of the [`Widget`].
    ///
    /// [`Widget`]: trait.Widget.html
    fn width(&self) -> Length;

    /// Returns the height of the [`Widget`].
    ///
    /// [`Widget`]: trait.Widget.html
    fn height(&self) -> Length;

    /// Returns the [`Node`] of the [`Widget`].
    ///
    /// This [`Node`] is used by the runtime to compute the [`Layout`] of the
    /// user interface.
    ///
    /// [`Node`]: ../layout/struct.Node.html
    /// [`Widget`]: trait.Widget.html
    /// [`Layout`]: ../layout/struct.Layout.html
    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node;

    /// Draws the [`Widget`] using the associated `Renderer`.
    ///
    /// [`Widget`]: trait.Widget.html
    fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) -> Renderer::Output;

    /// Computes the _layout_ hash of the [`Widget`].
    ///
    /// The produced hash is used by the runtime to decide if the [`Layout`]
    /// needs to be recomputed between frames. Therefore, to ensure maximum
    /// efficiency, the hash should only be affected by the properties of the
    /// [`Widget`] that can affect layouting.
    ///
    /// For example, the [`Text`] widget does not hash its color property, as
    /// its value cannot affect the overall [`Layout`] of the user interface.
    ///
    /// [`Widget`]: trait.Widget.html
    /// [`Layout`]: ../layout/struct.Layout.html
    /// [`Text`]: text/struct.Text.html
    fn hash_layout(&self, state: &mut Hasher);

    /// Processes a runtime [`Event`].
    ///
    /// It receives:
    ///   * an [`Event`] describing user interaction
    ///   * the computed [`Layout`] of the [`Widget`]
    ///   * the current cursor position
    ///   * a mutable `Message` list, allowing the [`Widget`] to produce
    ///   new messages based on user interaction.
    ///   * the `Renderer`
    ///   * a [`Clipboard`], if available
    ///
    /// By default, it does nothing.
    ///
    /// [`Event`]: ../enum.Event.html
    /// [`Widget`]: trait.Widget.html
    /// [`Layout`]: ../layout/struct.Layout.html
    /// [`Clipboard`]: ../trait.Clipboard.html
    fn on_event(
        &mut self,
        _event: Event,
        _layout: Layout<'_>,
        _cursor_position: Point,
        _messages: &mut Vec<Message>,
        _renderer: &Renderer,
        _clipboard: Option<&dyn Clipboard>,
    ) {
    }

    /// Returns the overlay of the [`Element`], if there is any.
    ///
    /// [`Element`]: struct.Element.html
    fn overlay(
        &mut self,
        _layout: Layout<'_>,
    ) -> Option<overlay::Element<'_, Message, Renderer>> {
        None
    }
}
