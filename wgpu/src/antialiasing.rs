/// An antialiasing strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Antialiasing {
    /// Multisample AA with 2 samples
    MSAAx2,
    /// Multisample AA with 4 samples
    MSAAx4,
    /// Multisample AA with 8 samples
    MSAAx8,
    /// Multisample AA with 16 samples
    MSAAx16,
}

impl Antialiasing {
    /// Returns the amount of samples of the [`Antialiasing`].
    pub fn sample_count(self) -> u32 {
        match self {
            Antialiasing::MSAAx2 => 2,
            Antialiasing::MSAAx4 => 4,
            Antialiasing::MSAAx8 => 8,
            Antialiasing::MSAAx16 => 16,
        }
    }
}
