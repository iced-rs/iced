use sctk::reexports::client::protocol::wl_seat::WlSeat;

/// seat events
/// Only one seat can interact with an iced_sctk application at a time, but many may interact with the application over the lifetime of the application
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeatEvent {
    /// A new seat is interacting with the application
    Enter,
    /// A seat is not interacting with the application anymore
    Leave,
}
