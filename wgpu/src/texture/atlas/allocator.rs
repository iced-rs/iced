use guillotiere::{AtlasAllocator, Size};

pub struct Allocator {
    raw: AtlasAllocator,
    size: u32,
}

impl Allocator {
    const PADDING: u32 = 1;

    pub fn new(size: u32) -> Allocator {
        let raw = AtlasAllocator::new(Size::new(size as i32, size as i32));

        Allocator { raw, size }
    }

    pub fn allocate(&mut self, width: u32, height: u32) -> Option<Region> {
        let padding = (
            if width + Self::PADDING * 2 < self.size {
                Self::PADDING
            } else {
                0
            },
            if height + Self::PADDING * 2 < self.size {
                Self::PADDING
            } else {
                0
            },
        );

        let allocation = self.raw.allocate(Size::new(
            (width + padding.0 * 2) as i32,
            (height + padding.1 * 2) as i32,
        ))?;

        Some(Region {
            allocation,
            padding,
        })
    }

    pub fn deallocate(&mut self, region: Region) {
        self.raw.deallocate(region.allocation.id);
    }
}

pub struct Region {
    allocation: guillotiere::Allocation,
    padding: (u32, u32),
}

impl Region {
    pub fn position(&self) -> (u32, u32) {
        let rectangle = &self.allocation.rectangle;

        (
            rectangle.min.x as u32 + self.padding.0,
            rectangle.min.y as u32 + self.padding.1,
        )
    }

    pub fn size(&self) -> (u32, u32) {
        let size = self.allocation.rectangle.size();

        (
            size.width as u32 - self.padding.0 * 2,
            size.height as u32 - self.padding.1 * 2,
        )
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
            .field("padding", &self.padding)
            .finish()
    }
}
