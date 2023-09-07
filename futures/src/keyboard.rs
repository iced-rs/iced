//! Listen to keyboard events.
use crate::core;
use crate::core::keyboard::{Event, KeyCode, Modifiers};
use crate::subscription::{self, Subscription};
use crate::MaybeSend;

/// Listens to keyboard key presses and calls the given function
/// map them into actual messages.
///
/// If the function returns `None`, the key press will be simply
/// ignored.
pub fn on_key_press<Message>(
    f: fn(KeyCode, Modifiers) -> Option<Message>,
) -> Subscription<Message>
where
    Message: MaybeSend + 'static,
{
    #[derive(Hash)]
    struct OnKeyPress;

    subscription::filter_map((OnKeyPress, f), move |event, status| {
        match (event, status) {
            (
                core::Event::Keyboard(Event::KeyPressed {
                    key_code,
                    modifiers,
                }),
                core::event::Status::Ignored,
            ) => f(key_code, modifiers),
            _ => None,
        }
    })
}

/// Listens to keyboard key releases and calls the given function
/// map them into actual messages.
///
/// If the function returns `None`, the key release will be simply
/// ignored.
pub fn on_key_release<Message>(
    f: fn(KeyCode, Modifiers) -> Option<Message>,
) -> Subscription<Message>
where
    Message: MaybeSend + 'static,
{
    #[derive(Hash)]
    struct OnKeyRelease;

    subscription::filter_map((OnKeyRelease, f), move |event, status| {
        match (event, status) {
            (
                core::Event::Keyboard(Event::KeyReleased {
                    key_code,
                    modifiers,
                }),
                core::event::Status::Ignored,
            ) => f(key_code, modifiers),
            _ => None,
        }
    })
}
