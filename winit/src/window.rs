//! Interact with the window of your application.
use crate::command::{self, Command};
pub use iced_native::screenshot::Screenshot;
use iced_native::window;
pub use window::Event;

/// Resizes the window to the given logical dimensions.
pub fn resize<Message>(width: u32, height: u32) -> Command<Message> {
    Command::single(command::Action::Window(window::Action::Resize {
        width,
        height,
    }))
}

/// Moves a window to the given logical coordinates.
pub fn move_to<Message>(x: i32, y: i32) -> Command<Message> {
    Command::single(command::Action::Window(window::Action::Move { x, y }))
}

///Take a [`Screenshot`] and call the provided screen_cap function to present the [`Screenshot`]
///back to the [`Application`]
///
///
///
///
///[`Application`]: crate::application::Application
pub fn take_screenshot<Message>(
    screen_cap: Box<dyn Fn(Option<Screenshot>) -> Message>,
) -> Command<Message> {
    Command::single(command::Action::Window(window::Action::TakeScreenshot(
        screen_cap,
    )))
}
