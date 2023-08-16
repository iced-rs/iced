use bitflags::bitflags;

bitflags! {
    /// Specifies buttons of a window
    pub struct WindowButtons: u32 {
        /// The close button
        const CLOSE  = 1 << 0;
        /// The minimize button
        const MINIMIZE  = 1 << 1;
        /// The maximuze button
        const MAXIMIZE  = 1 << 2;
    }
}
