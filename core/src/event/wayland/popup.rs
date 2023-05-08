/// popup events
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PopupEvent {
    /// Done
    Done,
    /// repositioned,
    Configured {
        /// x position
        x: i32,
        /// y position
        y: i32,
        /// width
        width: u32,
        /// height
        height: u32,
    },
    /// popup focused
    Focused,
    /// popup unfocused
    Unfocused,
}
