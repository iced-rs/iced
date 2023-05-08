use crate::{event_loop::state::SctkState, sctk_event::SctkEvent};
use sctk::{delegate_output, output::OutputHandler, reexports::client::Proxy};
use std::fmt::Debug;

impl<T: Debug> OutputHandler for SctkState<T> {
    fn output_state(&mut self) -> &mut sctk::output::OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &sctk::reexports::client::Connection,
        _qh: &sctk::reexports::client::QueueHandle<Self>,
        output: sctk::reexports::client::protocol::wl_output::WlOutput,
    ) {
        self.sctk_events.push(SctkEvent::NewOutput {
            id: output.clone(),
            info: self.output_state.info(&output),
        });
        self.outputs.push(output);
    }

    fn update_output(
        &mut self,
        _conn: &sctk::reexports::client::Connection,
        _qh: &sctk::reexports::client::QueueHandle<Self>,
        output: sctk::reexports::client::protocol::wl_output::WlOutput,
    ) {
        if let Some(info) = self.output_state.info(&output) {
            self.sctk_events.push(SctkEvent::UpdateOutput {
                id: output.clone(),
                info,
            });
        }
    }

    fn output_destroyed(
        &mut self,
        _conn: &sctk::reexports::client::Connection,
        _qh: &sctk::reexports::client::QueueHandle<Self>,
        output: sctk::reexports::client::protocol::wl_output::WlOutput,
    ) {
        self.sctk_events.push(SctkEvent::RemovedOutput(output));
        // TODO clean up any layer surfaces on this output?
    }
}

delegate_output!(@<T: 'static + Debug> SctkState<T>);
