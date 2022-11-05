use crate::image::atlas;

use iced_graphics::image;
use iced_graphics::Size;

#[derive(Debug)]
pub enum Entry {
    Contiguous(atlas::Allocation),
    Fragmented {
        size: Size<u32>,
        fragments: Vec<Fragment>,
    },
}

impl image::storage::Entry for Entry {
    fn size(&self) -> Size<u32> {
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
