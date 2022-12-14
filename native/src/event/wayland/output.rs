/// output events
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputEvent {
    /// created output
    Created {
        /// make of the output
        make: String,
        /// model of the output
        model: String,
    },
    /// removed output
    Removed {
        /// make of the output
        make: String,
        /// model of the output
        model: String,
    },
    /// name of the output
    Name(String),
    /// logical size of the output
    LogicalSize(u32, u32),
    /// logical position of the output
    LogicalPosition(u32, u32),
}
