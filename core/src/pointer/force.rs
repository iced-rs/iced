/// The force of a touch or tablet interaction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Force {
    /// A force calibrated by the platform.
    Calibrated {
        /// The applied force, where `1.0` represents an average touch.
        force: f64,

        /// The maximum force supported by the device.
        max_possible_force: f64,
    },

    /// A force normalized to the range from `0.0` to `1.0`.
    Normalized(f64),
}
