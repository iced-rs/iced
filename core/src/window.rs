//! Build window-based GUI applications.
pub mod icon;

mod event;
mod mode;
mod redraw_request;
mod user_attention;

pub use event::Event;
pub use icon::Icon;
pub use mode::Mode;
pub use redraw_request::RedrawRequest;
pub use user_attention::UserAttention;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// The identifier of a generic window.
pub struct Id(pub u128);
