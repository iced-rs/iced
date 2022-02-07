//! Custom hasher implementations

/// The hasher used to compare layouts.
#[derive(Debug)]
pub struct Hasher(twox_hash::XxHash64);

impl Default for Hasher {
    fn default() -> Self {
        Hasher(twox_hash::XxHash64::default())
    }
}

impl core::hash::Hasher for Hasher {
    fn write(&mut self, bytes: &[u8]) {
        self.0.write(bytes)
    }

    fn finish(&self) -> u64 {
        self.0.finish()
    }
}
