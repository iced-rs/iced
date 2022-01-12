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
#[cfg(not(target_arch = "wasm32"))]
mod platform {
    pub use crate::renderer::widget::{
        button, checkbox, container, pane_grid, pick_list, progress_bar, radio,
        rule, scrollable, slider, text_input, toggler, tooltip, Column, Row,
        Space, Text,
    };

    #[cfg(any(feature = "canvas", feature = "glow_canvas"))]
    #[cfg_attr(
        docsrs,
        doc(cfg(any(feature = "canvas", feature = "glow_canvas")))
    )]
    pub use crate::renderer::widget::canvas;

    #[cfg(any(feature = "qr_code", feature = "glow_qr_code"))]
    #[cfg_attr(
        docsrs,
        doc(cfg(any(feature = "qr_code", feature = "glow_qr_code")))
    )]
    pub use crate::renderer::widget::qr_code;

    #[cfg_attr(docsrs, doc(cfg(feature = "image")))]
    pub mod image {
        //! Display images in your user interface.
        pub use crate::runtime::image::Handle;
        pub use crate::runtime::widget::image::viewer;
        pub use crate::runtime::widget::image::{Image, Viewer};
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "svg")))]
    pub mod svg {
        //! Display vector graphics in your user interface.
        pub use crate::runtime::svg::Handle;
        pub use crate::runtime::widget::svg::Svg;
    }

    #[doc(no_inline)]
    pub use {
        button::Button, checkbox::Checkbox, container::Container, image::Image,
        pane_grid::PaneGrid, pick_list::PickList, progress_bar::ProgressBar,
        radio::Radio, rule::Rule, scrollable::Scrollable, slider::Slider,
        svg::Svg, text_input::TextInput, toggler::Toggler, tooltip::Tooltip,
    };

    #[cfg(any(feature = "canvas", feature = "glow_canvas"))]
    #[doc(no_inline)]
    pub use canvas::Canvas;

    #[cfg(any(feature = "qr_code", feature = "glow_qr_code"))]
    #[doc(no_inline)]
    pub use qr_code::QRCode;
}

#[cfg(target_arch = "wasm32")]
mod platform {
    pub use iced_web::widget::*;
}

pub use platform::*;
