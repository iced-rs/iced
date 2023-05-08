use crate::sctk_event::ActionRequestEvent;
use iced_accessibility::{accesskit, accesskit_unix};
use sctk::reexports::client::protocol::wl_surface::WlSurface;
use sctk::reexports::client::Proxy;
use std::{
    num::NonZeroU128,
    sync::{Arc, Mutex},
};

pub enum A11yWrapper {
    Enabled,
    Event(ActionRequestEvent),
}

pub struct IcedSctkAdapter {
    pub(crate) id: NonZeroU128,
    pub(crate) adapter: accesskit_unix::Adapter,
}

pub struct IcedSctkActionHandler {
    pub(crate) wl_surface: WlSurface,
    pub(crate) event_list: Arc<Mutex<Vec<A11yWrapper>>>,
}
impl accesskit::ActionHandler for IcedSctkActionHandler {
    fn do_action(&self, request: accesskit::ActionRequest) {
        let mut event_list = self.event_list.lock().unwrap();
        event_list.push(A11yWrapper::Event(
            crate::sctk_event::ActionRequestEvent {
                request,
                surface_id: self.wl_surface.id(),
            },
        ));
    }
}
