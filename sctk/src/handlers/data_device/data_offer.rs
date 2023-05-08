use sctk::{
    data_device_manager::data_offer::{
        DataDeviceOffer, DataOfferHandler, DragOffer,
    },
    delegate_data_offer,
    reexports::client::{
        protocol::wl_data_device_manager::DndAction, Connection, QueueHandle,
    },
};
use std::fmt::Debug;

use crate::event_loop::state::SctkState;

impl<T> DataOfferHandler for SctkState<T> {
    fn offer(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _offer: &mut DataDeviceOffer,
        _mime_type: String,
    ) {
    }

    fn source_actions(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        offer: &mut DragOffer,
        actions: DndAction,
    ) {
        if self.dnd_offer.as_ref().map(|o| o.offer.inner() == offer.inner()).unwrap_or(false)
        {
            self.sctk_events
                .push(crate::sctk_event::SctkEvent::DndOffer {
                    event: crate::sctk_event::DndOfferEvent::SourceActions(
                        actions,
                    ),
                    surface: offer.surface.clone(),
                });
        }
    }

    fn selected_action(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        offer: &mut DragOffer,
        actions: DndAction,
    ) {
        if self.dnd_offer.as_ref().map(|o| o.offer.inner() == offer.inner()).unwrap_or(false)
        {
            self.sctk_events
                .push(crate::sctk_event::SctkEvent::DndOffer {
                    event: crate::sctk_event::DndOfferEvent::SelectedAction(
                        actions,
                    ),
                    surface: offer.surface.clone(),
                });
        }
    }
}

delegate_data_offer!(@<T: 'static + Debug> SctkState<T>);
