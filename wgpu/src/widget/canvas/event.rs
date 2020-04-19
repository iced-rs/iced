use iced_native::input::mouse;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    Mouse(mouse::Event),
}
