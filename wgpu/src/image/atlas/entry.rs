use crate::image::atlas;
use iced_graphics::image::TextureStoreEntry;

#[derive(Debug)]
pub enum Entry {
    Contiguous(atlas::Allocation),
    Fragmented {
        size: (u32, u32),
        fragments: Vec<Fragment>,
    },
}

impl TextureStoreEntry for Entry {
    fn size(&self) -> (u32, u32) {
        match self {
            Entry::Contiguous(allocation) => allocation.size(),
            Entry::Fragmented { size, .. } => *size,
        }
    }
}

#[derive(Debug)]
pub struct Fragment {
    pub position: (u32, u32),
    pub allocation: atlas::Allocation,
}
