pub use iced_widget::*;

use crate::core::Font;
use crate::program;

pub fn monospace<'a, Renderer>(
    fragment: impl text::IntoFragment<'a>,
) -> Text<'a, Theme, Renderer>
where
    Renderer: program::Renderer + 'a,
{
    text(fragment).font(Font::MONOSPACE)
}
