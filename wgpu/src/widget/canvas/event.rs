use iced_native::mouse;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    Mouse(mouse::Event),
}
