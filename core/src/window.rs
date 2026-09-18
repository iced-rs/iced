//! Build window-based GUI applications.
pub mod icon;
pub mod screenshot;
pub mod settings;

mod direction;
mod event;
mod id;
mod level;
mod mode;
mod position;
mod redraw_request;
mod user_attention;

pub use direction::Direction;
pub use event::Event;
pub use icon::Icon;
pub use id::Id;
pub use level::Level;
pub use mode::Mode;
pub use position::Position;
pub use redraw_request::RedrawRequest;
pub use screenshot::Screenshot;
pub use settings::Settings;
pub use user_attention::UserAttention;

use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle,
};
use std::fmt::Debug;

pub use raw_window_handle;

/// A window managed by iced.
///
/// It implements both [`HasWindowHandle`] and [`HasDisplayHandle`].
pub trait Window: HasWindowHandle + HasDisplayHandle + Debug {}

impl<T> Window for T where T: HasWindowHandle + HasDisplayHandle + Debug {}

/// A headless window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Headless;

impl HasWindowHandle for Headless {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        Err(HandleError::NotSupported)
    }
}

impl HasDisplayHandle for Headless {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        Err(HandleError::NotSupported)
    }
}
