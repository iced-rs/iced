use std::collections::hash_map::DefaultHasher;

/// The hasher used to compare subscriptions.
#[derive(Debug)]
pub struct Hasher(DefaultHasher);

impl Default for Hasher {
    fn default() -> Self {
        Hasher(DefaultHasher::default())
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
