use crate::Renderer;

pub use iced_native::widget::responsive::State;

pub type Responsive<'a, Message> =
    iced_native::widget::Responsive<'a, Message, Renderer>;
