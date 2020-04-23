pub use seat::{
    SeatListener,
    pointer::ThemedPointer as Pointer,
};
use smithay_client_toolkit::{
    Environment,
    seat::{pointer::Event, keyboard::Event},
    reeexports::client::{DispatchData, protocol::{pointer::WlPointer, keyboard::WlKeyboard}},
};

pub fn listener<DispatchData>(
    env: &Environment,
    pointer: impl FnMut(pointer::Event, WlPointer, DispatchData) + 'static,
    keyboard: impl FnMut(keyboard::Event, WlKeyboard, DispatchData) + 'static
) -> seat::SeatListener {
    use {
        reexports::client::protocol::wl_pointer as pointer,
        seat::{
            keyboard::{map_keyboard, RepeatKind},
            pointer::{ThemeManager, ThemeSpec},
        },
    };

    let theme_manager = ThemeManager::init(
        ThemeSpec::System,
        env.require_global(),
        env.require_global(),
    );

    env.listen_for_seats(move |seat, seat_data, mut state| {
        if seat_data.has_pointer {
            assert!(pointer.is_none());
            let pointer = &mut state.get().unwrap().get().unwrap();
            *pointer = Some(theme_manager.theme_pointer_with_impl(&seat, pointer).unwrap());
        }
        if seat_data.has_keyboard {
            let (_, _) = map_keyboard(&seat, None, RepeatKind::System, keyboard).unwrap();
        }
    })
}
