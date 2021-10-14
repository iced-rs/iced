//! Build and show dropdown menus.
use crate::backend::{self, Backend};
use crate::Renderer;

use iced_native::overlay;

pub use iced_style::menu::Style;

impl<B> overlay::menu::Renderer for Renderer<B>
where
    B: Backend + backend::Text,
{
    type Style = Style;
}
