use crate::{event_loop::state::SctkState, sctk_event::SctkEvent};
use sctk::{
    delegate_pointer,
    reexports::client::Proxy,
    seat::pointer::{PointerEventKind, PointerHandler},
};
use std::fmt::Debug;

impl<T: Debug> PointerHandler for SctkState<T> {
    fn pointer_frame(
        &mut self,
        _conn: &sctk::reexports::client::Connection,
        _qh: &sctk::reexports::client::QueueHandle<Self>,
        pointer: &sctk::reexports::client::protocol::wl_pointer::WlPointer,
        events: &[sctk::seat::pointer::PointerEvent],
    ) {
        let (is_active, my_seat) =
            match self.seats.iter_mut().enumerate().find_map(|(i, s)| {
                if s.ptr.as_ref() == Some(pointer) {
                    Some((i, s))
                } else {
                    None
                }
            }) {
                Some((i, s)) => (i == 0, s),
                None => return,
            };

        // track events, but only forward for the active seat
        for e in events {
            if is_active {
                self.sctk_events.push(SctkEvent::PointerEvent {
                    variant: e.clone(),
                    ptr_id: pointer.clone(),
                    seat_id: my_seat.seat.clone(),
                });
            }
            match e.kind {
                PointerEventKind::Enter { .. } => {
                    my_seat.ptr_focus.replace(e.surface.clone());
                }
                PointerEventKind::Leave { .. } => {
                    my_seat.ptr_focus.take();
                }
                PointerEventKind::Press {
                    time,
                    button,
                    serial,
                } => {
                    my_seat.last_ptr_press.replace((time, button, serial));
                }
                // TODO revisit events that ought to be handled and change internal state
                _ => {}
            }
        }
    }
}

delegate_pointer!(@<T: 'static + Debug> SctkState<T>);
