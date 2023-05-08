use crate::{
    event_loop::state::SctkState,
    sctk_event::{KeyboardEventVariant, SctkEvent},
};

use sctk::{
    delegate_keyboard, reexports::client::Proxy,
    seat::keyboard::KeyboardHandler,
};
use std::fmt::Debug;

impl<T: Debug> KeyboardHandler for SctkState<T> {
    fn enter(
        &mut self,
        _conn: &sctk::reexports::client::Connection,
        _qh: &sctk::reexports::client::QueueHandle<Self>,
        keyboard: &sctk::reexports::client::protocol::wl_keyboard::WlKeyboard,
        surface: &sctk::reexports::client::protocol::wl_surface::WlSurface,
        _serial: u32,
        _raw: &[u32],
        _keysyms: &[u32],
    ) {
        let (i, mut is_active, seat) = {
            let (i, is_active, my_seat) =
                match self.seats.iter_mut().enumerate().find_map(|(i, s)| {
                    if s.kbd.as_ref() == Some(keyboard) {
                        Some((i, s))
                    } else {
                        None
                    }
                }) {
                    Some((i, s)) => (i, i == 0, s),
                    None => return,
                };
            my_seat.kbd_focus.replace(surface.clone());

            let seat = my_seat.seat.clone();
            (i, is_active, seat)
        };

        // TODO Ashley: thoroughly test this
        // swap the active seat to be the current seat if the current "active" seat is not focused on the application anyway
        if !is_active && self.seats[0].kbd_focus.is_none() {
            is_active = true;
            self.seats.swap(0, i);
        }

        if is_active {
            self.sctk_events.push(SctkEvent::KeyboardEvent {
                variant: KeyboardEventVariant::Enter(surface.clone()),
                kbd_id: keyboard.clone(),
                seat_id: seat,
            })
        }
    }

    fn leave(
        &mut self,
        _conn: &sctk::reexports::client::Connection,
        _qh: &sctk::reexports::client::QueueHandle<Self>,
        keyboard: &sctk::reexports::client::protocol::wl_keyboard::WlKeyboard,
        surface: &sctk::reexports::client::protocol::wl_surface::WlSurface,
        _serial: u32,
    ) {
        let (is_active, seat, kbd) = {
            let (is_active, my_seat) =
                match self.seats.iter_mut().enumerate().find_map(|(i, s)| {
                    if s.kbd.as_ref() == Some(keyboard) {
                        Some((i, s))
                    } else {
                        None
                    }
                }) {
                    Some((i, s)) => (i == 0, s),
                    None => return,
                };
            let seat = my_seat.seat.clone();
            let kbd = keyboard.clone();
            my_seat.kbd_focus.take();
            (is_active, seat, kbd)
        };

        if is_active {
            self.sctk_events.push(SctkEvent::KeyboardEvent {
                variant: KeyboardEventVariant::Leave(surface.clone()),
                kbd_id: kbd,
                seat_id: seat,
            });
            // if there is another seat with a keyboard focused on a surface make that the new active seat
            if let Some(i) =
                self.seats.iter().position(|s| s.kbd_focus.is_some())
            {
                self.seats.swap(0, i);
                let s = &self.seats[0];
                self.sctk_events.push(SctkEvent::KeyboardEvent {
                    variant: KeyboardEventVariant::Enter(
                        s.kbd_focus.clone().unwrap(),
                    ),
                    kbd_id: s.kbd.clone().unwrap(),
                    seat_id: s.seat.clone(),
                })
            }
        }
    }

    fn press_key(
        &mut self,
        _conn: &sctk::reexports::client::Connection,
        _qh: &sctk::reexports::client::QueueHandle<Self>,
        keyboard: &sctk::reexports::client::protocol::wl_keyboard::WlKeyboard,
        serial: u32,
        event: sctk::seat::keyboard::KeyEvent,
    ) {
        let (is_active, my_seat) =
            match self.seats.iter_mut().enumerate().find_map(|(i, s)| {
                if s.kbd.as_ref() == Some(keyboard) {
                    Some((i, s))
                } else {
                    None
                }
            }) {
                Some((i, s)) => (i == 0, s),
                None => return,
            };
        let seat_id = my_seat.seat.clone();
        let kbd_id = keyboard.clone();
        my_seat.last_kbd_press.replace((event.clone(), serial));
        if is_active {
            self.sctk_events.push(SctkEvent::KeyboardEvent {
                variant: KeyboardEventVariant::Press(event),
                kbd_id,
                seat_id,
            });
        }
    }

    fn release_key(
        &mut self,
        _conn: &sctk::reexports::client::Connection,
        _qh: &sctk::reexports::client::QueueHandle<Self>,
        keyboard: &sctk::reexports::client::protocol::wl_keyboard::WlKeyboard,
        _serial: u32,
        event: sctk::seat::keyboard::KeyEvent,
    ) {
        let (is_active, my_seat) =
            match self.seats.iter_mut().enumerate().find_map(|(i, s)| {
                if s.kbd.as_ref() == Some(keyboard) {
                    Some((i, s))
                } else {
                    None
                }
            }) {
                Some((i, s)) => (i == 0, s),
                None => return,
            };
        let seat_id = my_seat.seat.clone();
        let kbd_id = keyboard.clone();

        if is_active {
            self.sctk_events.push(SctkEvent::KeyboardEvent {
                variant: KeyboardEventVariant::Release(event),
                kbd_id,
                seat_id,
            });
        }
    }

    fn update_modifiers(
        &mut self,
        _conn: &sctk::reexports::client::Connection,
        _qh: &sctk::reexports::client::QueueHandle<Self>,
        keyboard: &sctk::reexports::client::protocol::wl_keyboard::WlKeyboard,
        _serial: u32,
        modifiers: sctk::seat::keyboard::Modifiers,
    ) {
        let (is_active, my_seat) =
            match self.seats.iter_mut().enumerate().find_map(|(i, s)| {
                if s.kbd.as_ref() == Some(keyboard) {
                    Some((i, s))
                } else {
                    None
                }
            }) {
                Some((i, s)) => (i == 0, s),
                None => return,
            };
        let seat_id = my_seat.seat.clone();
        let kbd_id = keyboard.clone();

        if is_active {
            self.sctk_events.push(SctkEvent::KeyboardEvent {
                variant: KeyboardEventVariant::Modifiers(modifiers),
                kbd_id,
                seat_id,
            })
        }
    }
}

delegate_keyboard!(@<T: 'static + Debug> SctkState<T>);
