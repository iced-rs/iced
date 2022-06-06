//! Pure versions of the widgets.

/// A container that distributes its contents vertically.
pub type Column<'a, Message, Theme = crate::Theme> =
    iced_pure::widget::Column<'a, Message, crate::Renderer<Theme>>;

/// A container that distributes its contents horizontally.
pub type Row<'a, Message, Theme = crate::Theme> =
    iced_pure::widget::Row<'a, Message, crate::Renderer<Theme>>;

/// A paragraph of text.
pub type Text<Theme = crate::Theme> =
    iced_pure::widget::Text<crate::Renderer<Theme>>;

pub mod button {
    //! Allow your users to perform actions by pressing a button.
    pub use iced_pure::widget::button::{Appearance, StyleSheet};

    /// A widget that produces a message when clicked.
    pub type Button<'a, Message, Theme = crate::Theme> =
        iced_pure::widget::Button<'a, Message, crate::Renderer<Theme>>;
}

pub mod checkbox {
    //! Show toggle controls using checkboxes.
    pub use iced_pure::widget::checkbox::{Appearance, StyleSheet};

    /// A box that can be checked.
    pub type Checkbox<'a, Message, Theme> =
        iced_native::widget::Checkbox<'a, Message, crate::Renderer<Theme>>;
}

pub mod container {
    //! Decorate content and apply alignment.
    pub use iced_pure::widget::container::{Style, StyleSheet};

    /// An element decorating some content.
    pub type Container<'a, Message, Theme = crate::Theme> =
        iced_pure::widget::Container<'a, Message, crate::Renderer<Theme>>;
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
    pub use iced_pure::widget::pane_grid::{
        Axis, Configuration, Direction, DragEvent, Line, Node, Pane,
        ResizeEvent, Split, State, StyleSheet,
    };

    /// A collection of panes distributed using either vertical or horizontal splits
    /// to completely fill the space available.
    ///
    /// [![Pane grid - Iced](https://thumbs.gfycat.com/MixedFlatJellyfish-small.gif)](https://gfycat.com/mixedflatjellyfish)
    pub type PaneGrid<'a, Message, Theme> =
        iced_pure::widget::PaneGrid<'a, Message, crate::Renderer<Theme>>;

    /// The content of a [`Pane`].
    pub type Content<'a, Message, Theme> =
        iced_pure::widget::pane_grid::Content<
            'a,
            Message,
            crate::Renderer<Theme>,
        >;

    /// The title bar of a [`Pane`].
    pub type TitleBar<'a, Message, Theme> =
        iced_pure::widget::pane_grid::TitleBar<
            'a,
            Message,
            crate::Renderer<Theme>,
        >;
}

pub mod pick_list {
    //! Display a dropdown list of selectable values.
    pub use iced_pure::overlay::menu::Style as Menu;
    pub use iced_pure::widget::pick_list::{Style, StyleSheet};

    /// A widget allowing the selection of a single value from a list of options.
    pub type PickList<'a, T, Message, Theme> =
        iced_pure::widget::PickList<'a, T, Message, crate::Renderer<Theme>>;
}

pub mod radio {
    //! Create choices using radio buttons.
    pub use iced_pure::widget::radio::{Appearance, StyleSheet};

    /// A circular button representing a choice.
    pub type Radio<Message, Theme> =
        iced_pure::widget::Radio<Message, crate::Renderer<Theme>>;
}

pub mod scrollable {
    //! Navigate an endless amount of content with a scrollbar.
    pub use iced_pure::widget::scrollable::{Scrollbar, Scroller, StyleSheet};

    /// A widget that can vertically display an infinite amount of content
    /// with a scrollbar.
    pub type Scrollable<'a, Message, Theme> =
        iced_pure::widget::Scrollable<'a, Message, crate::Renderer<Theme>>;
}

pub mod toggler {
    //! Show toggle controls using togglers.
    pub use iced_pure::widget::toggler::{Appearance, StyleSheet};

    /// A toggler widget.
    pub type Toggler<'a, Message, Theme> =
        iced_pure::widget::Toggler<'a, Message, crate::Renderer<Theme>>;
}

pub mod text_input {
    //! Display fields that can be filled with text.
    use crate::Renderer;

    pub use iced_pure::widget::text_input::{Appearance, StyleSheet};

    /// A field that can be filled with text.
    pub type TextInput<'a, Message, Theme> =
        iced_pure::widget::TextInput<'a, Message, Renderer<Theme>>;
}

pub mod tooltip {
    //! Display a widget over another.
    pub use iced_pure::widget::tooltip::Position;

    /// A widget allowing the selection of a single value from a list of options.
    pub type Tooltip<'a, Message, Theme> =
        iced_pure::widget::Tooltip<'a, Message, crate::Renderer<Theme>>;
}

pub use iced_pure::widget::progress_bar;
pub use iced_pure::widget::rule;
pub use iced_pure::widget::slider;
pub use iced_pure::widget::Space;

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
pub use text_input::TextInput;
pub use toggler::Toggler;
pub use tooltip::Tooltip;

#[cfg(feature = "canvas")]
pub use iced_graphics::widget::pure::canvas;

#[cfg(feature = "qr_code")]
pub use iced_graphics::widget::pure::qr_code;

#[cfg(feature = "image")]
pub mod image {
    //! Display images in your user interface.
    pub use iced_native::image::Handle;

    /// A frame that displays an image.
    pub type Image = iced_pure::widget::Image<Handle>;
}

#[cfg(feature = "svg")]
pub use iced_pure::widget::svg;

#[cfg(feature = "canvas")]
pub use canvas::Canvas;

#[cfg(feature = "qr_code")]
pub use qr_code::QRCode;

#[cfg(feature = "image")]
pub use image::Image;

#[cfg(feature = "svg")]
pub use svg::Svg;
