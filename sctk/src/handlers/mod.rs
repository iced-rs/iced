// handlers
pub mod compositor;
pub mod data_device;
pub mod output;
pub mod seat;
pub mod shell;

use sctk::{
    delegate_registry, delegate_shm,
    output::OutputState,
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::SeatState,
    shm::{Shm, ShmHandler},
};
use std::fmt::Debug;

use crate::event_loop::state::SctkState;

impl<T: Debug> ShmHandler for SctkState<T> {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm_state
    }
}

impl<T: Debug> ProvidesRegistryState for SctkState<T>
where
    T: 'static,
{
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState,];
}

delegate_shm!(@<T: 'static + Debug> SctkState<T>);
delegate_registry!(@<T: 'static + Debug> SctkState<T>);
