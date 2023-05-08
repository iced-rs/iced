use crate::{
    event_loop::{state::SctkSeat, state::SctkState},
    sctk_event::{KeyboardEventVariant, SctkEvent, SeatEventVariant},
};
use iced_runtime::keyboard::Modifiers;
use sctk::{
    delegate_seat,
    reexports::client::{protocol::wl_keyboard::WlKeyboard, Proxy},
    seat::SeatHandler,
};
use std::fmt::Debug;

impl<T: Debug> SeatHandler for SctkState<T>
where
    T: 'static,
{
    fn seat_state(&mut self) -> &mut sctk::seat::SeatState {
        &mut self.seat_state
    }

    fn new_seat(
        &mut self,
        _conn: &sctk::reexports::client::Connection,
        qh: &sctk::reexports::client::QueueHandle<Self>,
        seat: sctk::reexports::client::protocol::wl_seat::WlSeat,
    ) {
        self.sctk_events.push(SctkEvent::SeatEvent {
            variant: SeatEventVariant::New,
            id: seat.clone(),
        });
        let data_device =
            self.data_device_manager_state.get_data_device(qh, &seat);
        self.seats.push(SctkSeat {
            seat,
            kbd: None,
            ptr: None,
            touch: None,
            data_device,
            modifiers: Modifiers::default(),
            kbd_focus: None,
            ptr_focus: None,
            last_ptr_press: None,
            last_kbd_press: None,
        });
    }

    fn new_capability(
        &mut self,
        _conn: &sctk::reexports::client::Connection,
        qh: &sctk::reexports::client::QueueHandle<Self>,
        seat: sctk::reexports::client::protocol::wl_seat::WlSeat,
        capability: sctk::seat::Capability,
    ) {
        let my_seat = match self.seats.iter_mut().find(|s| s.seat == seat) {
            Some(s) => s,
            None => {
                self.seats.push(SctkSeat {
                    seat: seat.clone(),
                    kbd: None,
                    ptr: None,
                    touch: None,
                    data_device: self
                        .data_device_manager_state
                        .get_data_device(qh, &seat),
                    modifiers: Modifiers::default(),
                    kbd_focus: None,
                    ptr_focus: None,
                    last_ptr_press: None,
                    last_kbd_press: None,
                });
                self.seats.last_mut().unwrap()
            }
        };
        // TODO data device
        match capability {
            sctk::seat::Capability::Keyboard => {
                let seat_clone = seat.clone();
                if let Ok(kbd) = self.seat_state.get_keyboard_with_repeat(
                    qh,
                    &seat,
                    None,
                    self.loop_handle.clone(),
                    Box::new(move |state, kbd: &WlKeyboard, e| {
                        state.sctk_events.push(SctkEvent::KeyboardEvent {
                            variant: KeyboardEventVariant::Repeat(e),
                            kbd_id: kbd.clone(),
                            seat_id: seat_clone.clone(),
                        });
                    }),
                ) {
                    self.sctk_events.push(SctkEvent::SeatEvent {
                        variant: SeatEventVariant::NewCapability(
                            capability,
                            kbd.id(),
                        ),
                        id: seat.clone(),
                    });
                    my_seat.kbd.replace(kbd);
                }
            }
            sctk::seat::Capability::Pointer => {
                if let Ok(ptr) = self.seat_state.get_pointer(qh, &seat) {
                    self.sctk_events.push(SctkEvent::SeatEvent {
                        variant: SeatEventVariant::NewCapability(
                            capability,
                            ptr.id(),
                        ),
                        id: seat.clone(),
                    });
                    my_seat.ptr.replace(ptr);
                }
            }
            sctk::seat::Capability::Touch => {
                // TODO touch
            }
            _ => unimplemented!(),
        }
    }

    fn remove_capability(
        &mut self,
        _conn: &sctk::reexports::client::Connection,
        _qh: &sctk::reexports::client::QueueHandle<Self>,
        seat: sctk::reexports::client::protocol::wl_seat::WlSeat,
        capability: sctk::seat::Capability,
    ) {
        let my_seat = match self.seats.iter_mut().find(|s| s.seat == seat) {
            Some(s) => s,
            None => return,
        };

        // TODO data device
        match capability {
            // TODO use repeating kbd?
            sctk::seat::Capability::Keyboard => {
                if let Some(kbd) = my_seat.kbd.take() {
                    self.sctk_events.push(SctkEvent::SeatEvent {
                        variant: SeatEventVariant::RemoveCapability(
                            capability,
                            kbd.id(),
                        ),
                        id: seat.clone(),
                    });
                }
            }
            sctk::seat::Capability::Pointer => {
                if let Some(ptr) = my_seat.ptr.take() {
                    self.sctk_events.push(SctkEvent::SeatEvent {
                        variant: SeatEventVariant::RemoveCapability(
                            capability,
                            ptr.id(),
                        ),
                        id: seat.clone(),
                    });
                }
            }
            sctk::seat::Capability::Touch => {
                // TODO touch
                // my_seat.touch = self.seat_state.get_touch(qh, &seat).ok();
            }
            _ => unimplemented!(),
        }
    }

    fn remove_seat(
        &mut self,
        _conn: &sctk::reexports::client::Connection,
        _qh: &sctk::reexports::client::QueueHandle<Self>,
        seat: sctk::reexports::client::protocol::wl_seat::WlSeat,
    ) {
        self.sctk_events.push(SctkEvent::SeatEvent {
            variant: SeatEventVariant::Remove,
            id: seat.clone(),
        });
        if let Some(i) = self.seats.iter().position(|s| s.seat == seat) {
            self.seats.remove(i);
        }
    }
}

delegate_seat!(@<T: 'static + Debug> SctkState<T>);
