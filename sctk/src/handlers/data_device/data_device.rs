use std::fmt::Debug;

use sctk::{
    data_device_manager::{
        data_device::{DataDevice, DataDeviceDataExt, DataDeviceHandler},
        data_offer::DragOffer,
    },
    delegate_data_device,
    reexports::client::{Connection, QueueHandle},
};

use crate::{
    event_loop::state::{SctkDragOffer, SctkSelectionOffer, SctkState},
    sctk_event::{DndOfferEvent, SctkEvent},
};

impl<T> DataDeviceHandler for SctkState<T> {
    fn enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        data_device: DataDevice,
    ) {
        let mime_types = data_device.drag_mime_types();
        let drag_offer = data_device.drag_offer().unwrap();
        self.dnd_offer = Some(SctkDragOffer {
            dropped: false,
            offer: drag_offer.clone(),
            cur_read: None,
        });
        self.sctk_events.push(SctkEvent::DndOffer {
            event: DndOfferEvent::Enter {
                mime_types,
                x: drag_offer.x,
                y: drag_offer.y,
            },
            surface: drag_offer.surface.clone(),
        });
    }

    fn leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        data_device: DataDevice,
    ) {
        // ASHLEY TODO the dnd_offer should be removed when the leave event is received
        // but for now it is not if the offer was previously dropped.
        // It seems that leave events are received even for offers which have
        // been accepted and need to be read.
        if let Some(dnd_offer) = self.dnd_offer.take() {
            if dnd_offer.dropped {
                self.dnd_offer = Some(dnd_offer);
                return;
            }

            self.sctk_events.push(SctkEvent::DndOffer {
                event: DndOfferEvent::Leave,
                surface: dnd_offer.offer.surface.clone(),
            });
        }
    }

    fn motion(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        data_device: DataDevice,
    ) {
        let offer = data_device.drag_offer();
        // if the offer is not the same as the current one, ignore the leave event
        if offer.as_ref() != self.dnd_offer.as_ref().map(|o| &o.offer) {
            return;
        }
        let DragOffer { x, y, surface, .. } = data_device.drag_offer().unwrap();
        self.sctk_events.push(SctkEvent::DndOffer {
            event: DndOfferEvent::Motion { x, y },
            surface: surface.clone(),
        });
    }

    fn selection(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        data_device: DataDevice,
    ) {
        if let Some(offer) = data_device.selection_offer() {
            self.sctk_events.push(SctkEvent::SelectionOffer(
                crate::sctk_event::SelectionOfferEvent::Offer(
                    data_device.selection_mime_types(),
                ),
            ));
            self.selection_offer = Some(SctkSelectionOffer {
                offer: offer.clone(),
                cur_read: None,
            });
        }
    }

    fn drop_performed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        data_device: DataDevice,
    ) {
        if let Some(offer) = data_device.drag_offer() {
            if let Some(dnd_offer) = self.dnd_offer.as_mut() {
                if offer != dnd_offer.offer {
                    return;
                }
                dnd_offer.dropped = true;
            }
            self.sctk_events.push(SctkEvent::DndOffer {
                event: DndOfferEvent::DropPerformed,
                surface: offer.surface.clone(),
            });
        }
    }
}

delegate_data_device!(@<T: 'static + Debug> SctkState<T>);
