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
pub mod radio;
pub mod row;
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
pub use radio::Radio;
#[doc(no_inline)]
pub use row::Row;
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

use crate::{layout, Clipboard, Event, Hasher, Layout, Length, Point};

/// A component that displays information and allows interaction.
///
/// If you want to build your own widgets, you will need to implement this
/// trait.
///
/// [`Widget`]: trait.Widget.html
/// [`Element`]: ../struct.Element.html
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
    ///
    /// By default, it does nothing.
    ///
    /// [`Event`]: ../enum.Event.html
    /// [`Widget`]: trait.Widget.html
    /// [`Layout`]: ../layout/struct.Layout.html
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
}
