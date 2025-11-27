use crate::core;

use guillotiere::{AtlasAllocator, Size};

pub struct Allocator {
    raw: AtlasAllocator,
    allocations: usize,
}

impl Allocator {
    const PADDING: u32 = 1;

    pub fn new(size: u32) -> Allocator {
        let raw = AtlasAllocator::new(Size::new(size as i32, size as i32));

        Allocator {
            raw,
            allocations: 0,
        }
    }

    pub fn allocate(&mut self, width: u32, height: u32) -> Option<Region> {
        let size = self.raw.size();

        let padded_width = width + Self::PADDING * 2;
        let padded_height = height + Self::PADDING * 2;

        let pad_width = padded_width as i32 <= size.width;
        let pad_height = padded_height as i32 <= size.height;

        let mut allocation = self.raw.allocate(Size::new(
            if pad_width { padded_width } else { width } as i32,
            if pad_height { padded_height } else { height } as i32,
        ))?;

        if pad_width {
            allocation.rectangle.min.x += Self::PADDING as i32;
            allocation.rectangle.max.x -= Self::PADDING as i32;
        }

        if pad_height {
            allocation.rectangle.min.y += Self::PADDING as i32;
            allocation.rectangle.max.y -= Self::PADDING as i32;
        }

        self.allocations += 1;

        Some(Region {
            allocation,
            padding: core::Size::new(
                if pad_width { Self::PADDING } else { 0 },
                if pad_height { Self::PADDING } else { 0 },
            ),
        })
    }

    pub fn deallocate(&mut self, region: &Region) {
        self.raw.deallocate(region.allocation.id);

        self.allocations = self.allocations.saturating_sub(1);
    }

    pub fn is_empty(&self) -> bool {
        self.allocations == 0
    }

    pub fn allocations(&self) -> usize {
        self.allocations
    }
}

pub struct Region {
    allocation: guillotiere::Allocation,
    padding: core::Size<u32>,
}

impl Region {
    pub fn position(&self) -> (u32, u32) {
        let rectangle = &self.allocation.rectangle;

        (rectangle.min.x as u32, rectangle.min.y as u32)
    }

    pub fn size(&self) -> core::Size<u32> {
        let size = self.allocation.rectangle.size();

        core::Size::new(size.width as u32, size.height as u32)
    }

    pub fn padding(&self) -> crate::core::Size<u32> {
        self.padding
    }
}

impl std::fmt::Debug for Allocator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Allocator")
    }
}

impl std::fmt::Debug for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Region")
            .field("id", &self.allocation.id)
            .field("rectangle", &self.allocation.rectangle)
            .finish()
    }
}
