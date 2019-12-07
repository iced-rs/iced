pub use iced_winit::{
    Align, Background, Color, Command, Font, HorizontalAlignment, Length,
    Palette, VerticalAlignment,
};

pub mod widget {
    //! Display information and interactive controls in your application.
    //!
    //! # Re-exports
    //! For convenience, the contents of this module are available at the root
    //! module. Therefore, you can directly type:
    //!
    //! ```
    //! use iced::{button, Button};
    //! ```
    //!
    //! # Stateful widgets
    //! Some widgets need to keep track of __local state__.
    //!
    //! These widgets have their own module with a `State` type. For instance, a
    //! [`TextInput`] has some [`text_input::State`].
    //!
    //! [`TextInput`]: text_input/struct.TextInput.html
    //! [`text_input::State`]: text_input/struct.State.html
    pub mod button {
        //! Allow your users to perform actions by pressing a button.
        //!
        //! A [`Button`] has some local [`State`].
        //!
        //! [`Button`]: type.Button.html
        //! [`State`]: struct.State.html

        /// A widget that produces a message when clicked.
        ///
        /// This is an alias of an `iced_native` button with a default
        /// `Renderer` and `Style`.
        pub type Button<'a, Message> = iced_winit::Button<
            'a,
            Message,
            iced_wgpu::Renderer,
            <iced_wgpu::Renderer as iced_winit::button::Renderer>::WidgetStyle,
        >;

        pub use iced_winit::button::State;
    }

    pub mod scrollable {
        //! Navigate an endless amount of content with a scrollbar.

        /// A widget that can vertically display an infinite amount of content
        /// with a scrollbar.
        ///
        /// This is an alias of an `iced_native` scrollable with a default
        /// `Renderer`.
        pub type Scrollable<'a, Message> =
            iced_winit::Scrollable<'a, Message, iced_wgpu::Renderer>;

        pub use iced_winit::scrollable::State;
    }

    pub mod text_input {
        //! Ask for information using text fields.
        //!
        //! A [`TextInput`] has some local [`State`].
        //!
        //! [`TextInput`]: type.TextInput.html
        //! [`State`]: struct.State.html

        /// This is an alias of an `iced_native` text input with a default
        /// `Style`.
        pub type TextInput<'a, Message> = iced_winit::TextInput<
            'a,
            Message,
            <iced_wgpu::Renderer as iced_winit::text_input::Renderer>::WidgetStyle,
        >;

        pub use iced_winit::text_input::State;
    }

    pub mod slider {
        //! Display an interactive selector of a single value from a range of
        //! values.
        //!
        //! A [`Slider`] has some local [`State`].
        //!
        //! [`Slider`]: type.Slider.html
        //! [`State`]: struct.State.html

        /// This is an alias of an `iced_native` slider with a default
        /// `Style`.
        pub type Slider<'a, Message> = iced_winit::Slider<
            'a,
            Message,
            <iced_wgpu::Renderer as iced_winit::slider::Renderer>::WidgetStyle,
        >;

        pub use iced_winit::slider::State;
    }

    pub mod checkbox {
        //! Display a box that can be checked and an associated label with some
        //! text.

        /// This is an alias of an `iced_native` checkbox with a default
        /// `Style`.
        pub type Checkbox<Message> = iced_winit::Checkbox<
            Message,
            <iced_wgpu::Renderer as iced_winit::checkbox::Renderer>::WidgetStyle,
        >;
    }

    pub mod text {
        //! Display a text snippet.

        /// This is an alias of an `iced_native` text with a default
        /// `Style`.
        pub type Text = iced_winit::Text<
            <iced_wgpu::Renderer as iced_winit::text::Renderer>::WidgetStyle,
        >;
    }

    pub mod radio {
        //! Display an option button that can be turned on or off and an
        //! associated label with some text.

        /// This is an alias of an `iced_native` radio with a default
        /// `Style`.
        pub type Radio<Message> = iced_winit::Radio<
            Message,
            <iced_wgpu::Renderer as iced_winit::radio::Renderer>::WidgetStyle,
        >;
    }

    pub mod image {
        //! Display images in your user interface.
        pub use iced_winit::image::{Handle, Image};
    }

    #[doc(no_inline)]
    pub use {
        button::Button, checkbox::Checkbox, image::Image, radio::Radio,
        scrollable::Scrollable, slider::Slider, text::Text,
        text_input::TextInput,
    };

    /// A container that distributes its contents vertically.
    ///
    /// This is an alias of an `iced_native` column with a default `Renderer`.
    pub type Column<'a, Message> =
        iced_winit::Column<'a, Message, iced_wgpu::Renderer>;

    /// A container that distributes its contents horizontally.
    ///
    /// This is an alias of an `iced_native` row with a default `Renderer`.
    pub type Row<'a, Message> =
        iced_winit::Row<'a, Message, iced_wgpu::Renderer>;

    /// An element decorating some content.
    ///
    /// This is an alias of an `iced_native` container with a default
    /// `Renderer`.
    pub type Container<'a, Message> =
        iced_winit::Container<'a, Message, iced_wgpu::Renderer>;
}

pub mod style {
    //! Provides styling for your widgets.

    pub use iced_wgpu::*;
}

#[doc(no_inline)]
pub use widget::*;

#[doc(no_inline)]
pub use style::*;

/// A generic widget.
///
/// This is an alias of an `iced_native` element with a default `Renderer`.
pub type Element<'a, Message> =
    iced_winit::Element<'a, Message, iced_wgpu::Renderer>;
