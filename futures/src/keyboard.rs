//! Listen to keyboard events.
use crate::MaybeSend;
use crate::core;
use crate::core::event;
use crate::core::keyboard::{Event, Key, Modifiers};
use crate::subscription::{self, Subscription};

/// Listens to keyboard key presses and calls the given function
/// to map them into actual messages.
///
/// If the function returns `None`, the key press will be simply
/// ignored.
pub fn on_key_press<Message>(
    f: fn(Key, Modifiers) -> Option<Message>,
) -> Subscription<Message>
where
    Message: MaybeSend + 'static,
{
    #[derive(Hash)]
    struct OnKeyPress;

    subscription::filter_map((OnKeyPress, f), move |event| match event {
        subscription::Event::Interaction {
            event:
                core::Event::Keyboard(Event::KeyPressed { key, modifiers, .. }),
            status: event::Status::Ignored,
            ..
        } => f(key, modifiers),
        _ => None,
    })
}

/// Listens to keyboard key releases and calls the given function
/// to map them into actual messages.
///
/// If the function returns `None`, the key release will be simply
/// ignored.
pub fn on_key_release<Message>(
    f: fn(Key, Modifiers) -> Option<Message>,
) -> Subscription<Message>
where
    Message: MaybeSend + 'static,
{
    #[derive(Hash)]
    struct OnKeyRelease;

    subscription::filter_map((OnKeyRelease, f), move |event| match event {
        subscription::Event::Interaction {
            event:
                core::Event::Keyboard(Event::KeyReleased { key, modifiers, .. }),
            status: event::Status::Ignored,
            ..
        } => f(key, modifiers),
        _ => None,
    })
}
