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
#[cfg(not(target_arch = "wasm32"))]
mod platform {
    pub use iced_wgpu::widget::*;

    pub mod image {
        //! Display images in your user interface.
        pub use iced_winit::image::{Handle, Image};
    }

    pub mod svg {
        //! Display vector graphics in your user interface.
        pub use iced_winit::svg::{Handle, Svg};
    }

    pub use iced_winit::Text;

    #[doc(no_inline)]
    pub use {
        button::Button, checkbox::Checkbox, container::Container, image::Image,
        progress_bar::ProgressBar, radio::Radio, scrollable::Scrollable,
        slider::Slider, svg::Svg, text_input::TextInput,
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

    /// A container that distributes its contents horizontally and vertically.
    ///
    /// This is an alias of an `iced_native` table with a default `Renderer`.
    pub type Table<'a, Message> =
        iced_winit::Table<'a, Message, iced_wgpu::Renderer>;
}

#[cfg(target_arch = "wasm32")]
mod platform {
    pub use iced_web::widget::*;
}

pub use platform::*;
