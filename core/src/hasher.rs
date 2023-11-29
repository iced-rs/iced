/// The hasher used to compare layouts.
#[allow(missing_debug_implementations)] // Doesn't really make sense to have debug on the hasher state anyways.
#[derive(Default)]
pub struct Hasher(xxhash_rust::xxh3::Xxh3);

impl core::hash::Hasher for Hasher {
    fn write(&mut self, bytes: &[u8]) {
        self.0.write(bytes);
    }

    fn finish(&self) -> u64 {
        self.0.finish()
    }
}
