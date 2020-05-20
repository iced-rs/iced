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
    pub use iced_glow::widget::{
        button, checkbox, container, pane_grid, progress_bar, radio,
        scrollable, slider, text_input, Column, Row, Space, Text,
    };

    #[cfg(feature = "canvas")]
    #[cfg_attr(docsrs, doc(cfg(feature = "canvas")))]
    pub use iced_glow::widget::canvas;

    #[cfg_attr(docsrs, doc(cfg(feature = "image")))]
    pub mod image {
        //! Display images in your user interface.
        pub use iced_glutin::image::{Handle, Image};
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "svg")))]
    pub mod svg {
        //! Display vector graphics in your user interface.
        pub use iced_glutin::svg::{Handle, Svg};
    }

    #[doc(no_inline)]
    pub use {
        button::Button, checkbox::Checkbox, container::Container, image::Image,
        pane_grid::PaneGrid, progress_bar::ProgressBar, radio::Radio,
        scrollable::Scrollable, slider::Slider, svg::Svg,
        text_input::TextInput,
    };

    #[cfg(feature = "canvas")]
    #[doc(no_inline)]
    pub use canvas::Canvas;
}

#[cfg(target_arch = "wasm32")]
mod platform {
    pub use iced_web::widget::*;
}

pub use platform::*;
