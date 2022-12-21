/// output events
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputEvent {
    /// created output
    Created,
    /// removed output
    Removed,
    /// Make and Model
    MakeAndModel {
        /// make of the output
        make: String,
        /// model of the output
        model: String,
    },
    /// name of the output
    Name(Option<String>),
    /// logical size of the output
    LogicalSize(Option<(i32, i32)>),
    /// logical position of the output
    LogicalPosition(Option<(i32, i32)>),
}
