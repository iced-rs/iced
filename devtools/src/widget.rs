mod recorder;

pub use iced_widget::*;
pub use recorder::Recorder;

use crate::core::Element;

pub fn recorder<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Recorder<'a, Message, Theme, Renderer> {
    Recorder::new(content)
}
