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
    use crate::runtime;
    pub use runtime::Space;

    // iced_shm | iced_wgpu
    pub use crate::renderer::widget::{
        button::{self, Button}, checkbox::{self, Checkbox}, container::{self, Container}, pane_grid::{self, PaneGrid}, progress_bar::{self, ProgressBar}, radio::{self, Radio},
        scrollable::{self, Scrollable}, slider::{self, Slider}, text_input::{self, TextInput}, Text,
    };

    #[cfg(feature = "canvas")]
    #[cfg_attr(docsrs, doc(cfg(feature = "canvas")))]
    pub use crate::renderer::widget::canvas;

    #[cfg_attr(docsrs, doc(cfg(feature = "image")))]
    pub mod image {
        //! Display images in your user interface.
        #[cfg(feature = "image")]
        pub use crate::runtime::image::{Handle, Image};
    }
    #[cfg(feature = "image")] pub use image::Image;

    #[cfg_attr(docsrs, doc(cfg(feature = "svg")))]
    pub mod svg {
        //! Display vector graphics in your user interface.
        #[cfg(feature = "svg")]
        pub use crate::runtime::svg::{Handle, Svg};
    }
    #[cfg(feature = "svg")] pub use svg::Svg;

    #[cfg(feature = "canvas")]
    #[doc(no_inline)]
    pub use canvas::Canvas;

    /// A container that distributes its contents vertically.
    ///
    /// This is an alias of an `iced_native` column with a default `Renderer`.
    pub type Column<'a, Message> = runtime::Column<'a, Message, crate::renderer::Renderer>;

    /// A container that distributes its contents horizontally.
    ///
    /// This is an alias of an `iced_native` row with a default `Renderer`.
    pub type Row<'a, Message> = runtime::Row<'a, Message, crate::renderer::Renderer>;
}

#[cfg(target_arch = "wasm32")]
mod platform {
    pub use iced_web::widget::*;
}

pub use platform::*;
