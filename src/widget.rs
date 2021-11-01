//! Display information and interactive controls in your application.
pub use iced_native::widget::helpers::*;

pub use iced_native::{column, row};

/// A container that distributes its contents vertically.
pub type Column<'a, Message, Renderer = crate::Renderer> =
    iced_native::widget::Column<'a, Message, Renderer>;

/// A container that distributes its contents horizontally.
pub type Row<'a, Message, Renderer = crate::Renderer> =
    iced_native::widget::Row<'a, Message, Renderer>;

pub mod text {
    //! Write some text for your users to read.
    pub use iced_native::widget::text::{Appearance, StyleSheet};

    /// A paragraph of text.
    pub type Text<'a, Renderer = crate::Renderer> =
        iced_native::widget::Text<'a, Renderer>;
}

pub mod button {
    //! Allow your users to perform actions by pressing a button.
    pub use iced_native::widget::button::{Appearance, StyleSheet};

    /// A widget that produces a message when clicked.
    pub type Button<'a, Message, Renderer = crate::Renderer> =
        iced_native::widget::Button<'a, Message, Renderer>;
}

pub mod checkbox {
    //! Show toggle controls using checkboxes.
    pub use iced_native::widget::checkbox::{Appearance, StyleSheet};

    /// A box that can be checked.
    pub type Checkbox<'a, Message, Renderer = crate::Renderer> =
        iced_native::widget::Checkbox<'a, Message, Renderer>;
}

pub mod container {
    //! Decorate content and apply alignment.
    pub use iced_native::widget::container::{Appearance, StyleSheet};

    /// An element decorating some content.
    pub type Container<'a, Message, Renderer = crate::Renderer> =
        iced_native::widget::Container<'a, Message, Renderer>;
}

pub mod pane_grid {
    //! Let your users split regions of your application and organize layout dynamically.
    //!
    //! [![Pane grid - Iced](https://thumbs.gfycat.com/MixedFlatJellyfish-small.gif)](https://gfycat.com/mixedflatjellyfish)
    //!
    //! # Example
    //! The [`pane_grid` example] showcases how to use a [`PaneGrid`] with resizing,
    //! drag and drop, and hotkey support.
    //!
    //! [`pane_grid` example]: https://github.com/iced-rs/iced/tree/0.4/examples/pane_grid
    pub use iced_native::widget::pane_grid::{
        Axis, Configuration, Direction, DragEvent, Line, Node, Pane,
        ResizeEvent, Split, State, StyleSheet,
    };

    /// A collection of panes distributed using either vertical or horizontal splits
    /// to completely fill the space available.
    ///
    /// [![Pane grid - Iced](https://thumbs.gfycat.com/MixedFlatJellyfish-small.gif)](https://gfycat.com/mixedflatjellyfish)
    pub type PaneGrid<'a, Message, Renderer = crate::Renderer> =
        iced_native::widget::PaneGrid<'a, Message, Renderer>;

    /// The content of a [`Pane`].
    pub type Content<'a, Message, Renderer = crate::Renderer> =
        iced_native::widget::pane_grid::Content<'a, Message, Renderer>;

    /// The title bar of a [`Pane`].
    pub type TitleBar<'a, Message, Renderer = crate::Renderer> =
        iced_native::widget::pane_grid::TitleBar<'a, Message, Renderer>;
}

pub mod pick_list {
    //! Display a dropdown list of selectable values.
    pub use iced_native::widget::pick_list::{Appearance, StyleSheet};

    /// A widget allowing the selection of a single value from a list of options.
    pub type PickList<'a, T, Message, Renderer = crate::Renderer> =
        iced_native::widget::PickList<'a, T, Message, Renderer>;
}

pub mod radio {
    //! Create choices using radio buttons.
    pub use iced_native::widget::radio::{Appearance, StyleSheet};

    /// A circular button representing a choice.
    pub type Radio<Message, Renderer = crate::Renderer> =
        iced_native::widget::Radio<Message, Renderer>;
}

pub mod scrollable {
    //! Navigate an endless amount of content with a scrollbar.
    pub use iced_native::widget::scrollable::{
        snap_to, style::Scrollbar, style::Scroller, Id, StyleSheet,
    };

    /// A widget that can vertically display an infinite amount of content
    /// with a scrollbar.
    pub type Scrollable<'a, Message, Renderer = crate::Renderer> =
        iced_native::widget::Scrollable<'a, Message, Renderer>;
}

pub mod toggler {
    //! Show toggle controls using togglers.
    pub use iced_native::widget::toggler::{Appearance, StyleSheet};

    /// A toggler widget.
    pub type Toggler<'a, Message, Renderer = crate::Renderer> =
        iced_native::widget::Toggler<'a, Message, Renderer>;
}

pub mod text_input {
    //! Display fields that can be filled with text.
    pub use iced_native::widget::text_input::{
        focus, Appearance, Id, StyleSheet,
    };

    /// A field that can be filled with text.
    pub type TextInput<'a, Message, Renderer = crate::Renderer> =
        iced_native::widget::TextInput<'a, Message, Renderer>;
}

pub mod tooltip {
    //! Display a widget over another.
    pub use iced_native::widget::tooltip::Position;

    /// A widget allowing the selection of a single value from a list of options.
    pub type Tooltip<'a, Message, Renderer = crate::Renderer> =
        iced_native::widget::Tooltip<'a, Message, Renderer>;
}

pub use iced_native::widget::progress_bar;
pub use iced_native::widget::rule;
pub use iced_native::widget::slider;
pub use iced_native::widget::Space;

pub use button::Button;
pub use checkbox::Checkbox;
pub use container::Container;
pub use pane_grid::PaneGrid;
pub use pick_list::PickList;
pub use progress_bar::ProgressBar;
pub use radio::Radio;
pub use rule::Rule;
pub use scrollable::Scrollable;
pub use slider::Slider;
pub use text::Text;
pub use text_input::TextInput;
pub use toggler::Toggler;
pub use tooltip::Tooltip;

#[cfg(feature = "canvas")]
#[cfg_attr(docsrs, doc(cfg(feature = "canvas")))]
pub use iced_graphics::widget::canvas;

#[cfg(feature = "canvas")]
#[cfg_attr(docsrs, doc(cfg(feature = "canvas")))]
/// Creates a new [`Canvas`].
pub fn canvas<P, Message, Theme>(program: P) -> Canvas<Message, Theme, P>
where
    P: canvas::Program<Message, Theme>,
{
    Canvas::new(program)
}

#[cfg(feature = "image")]
#[cfg_attr(docsrs, doc(cfg(feature = "image")))]
pub mod image {
    //! Display images in your user interface.
    pub use iced_native::image::Handle;

    /// A frame that displays an image.
    pub type Image = iced_native::widget::Image<Handle>;

    pub use iced_native::widget::image::viewer;
    pub use viewer::Viewer;
}

#[cfg(feature = "qr_code")]
#[cfg_attr(docsrs, doc(cfg(feature = "qr_code")))]
pub use iced_graphics::widget::qr_code;

#[cfg(feature = "svg")]
#[cfg_attr(docsrs, doc(cfg(feature = "svg")))]
pub mod svg {
    //! Display vector graphics in your application.
    pub use iced_native::svg::Handle;
    pub use iced_native::widget::Svg;
}

#[cfg(feature = "canvas")]
#[cfg_attr(docsrs, doc(cfg(feature = "canvas")))]
pub use canvas::Canvas;

#[cfg(feature = "image")]
#[cfg_attr(docsrs, doc(cfg(feature = "image")))]
pub use image::Image;

#[cfg(feature = "qr_code")]
#[cfg_attr(docsrs, doc(cfg(feature = "qr_code")))]
pub use qr_code::QRCode;

#[cfg(feature = "svg")]
#[cfg_attr(docsrs, doc(cfg(feature = "svg")))]
pub use svg::Svg;

use crate::Command;
use iced_native::widget::operation;

/// Focuses the previous focusable widget.
pub fn focus_previous<Message>() -> Command<Message>
where
    Message: 'static,
{
    Command::widget(operation::focusable::focus_previous())
}

/// Focuses the next focusable widget.
pub fn focus_next<Message>() -> Command<Message>
where
    Message: 'static,
{
    Command::widget(operation::focusable::focus_next())
}
