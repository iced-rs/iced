//! Access the native system.
use crate::command::{self, Command};
use iced_native::system;

/// Query for available system information.
///
/// Returns `None` if not using the `sysinfo` feature flag.
pub fn information<Message>(
    f: impl Fn(Option<system::Information>) -> Message + 'static,
) -> Command<Message> {
    Command::single(command::Action::System(system::Action::QueryInformation(
        Box::new(f),
    )))
}
