//! Access the native system.
use crate::command::{self, Command};
pub use iced_native::system::*;

/// Query for available system information.
///
/// Returns `None` if not using the `sysinfo` feature flag.
pub fn information<Message>(
    f: impl Fn(Option<Information>) -> Message + 'static,
) -> Command<Message> {
    Command::single(command::Action::System(Action::QueryInformation(
        Box::new(f),
    )))
}
