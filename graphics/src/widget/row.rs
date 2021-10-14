use crate::Renderer;

/// A container that distributes its contents horizontally.
pub type Row<'a, Message, Backend> =
    iced_native::Row<'a, Message, Renderer<Backend>>;
