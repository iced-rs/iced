use guillotiere::{AtlasAllocator, Size};

pub struct Allocator {
    raw: AtlasAllocator,
}

impl Allocator {
    pub fn new(size: u32) -> Allocator {
        let raw = AtlasAllocator::new(Size::new(size as i32, size as i32));

        Allocator { raw }
    }

    pub fn allocate(&mut self, width: u32, height: u32) -> Option<Region> {
        let allocation = self
            .raw
            .allocate(Size::new(width as i32 + 2, height as i32 + 2))?;

        Some(Region(allocation))
    }

    pub fn deallocate(&mut self, region: Region) {
        self.raw.deallocate(region.0.id);
    }
}

pub struct Region(guillotiere::Allocation);

impl Region {
    pub fn position(&self) -> (u32, u32) {
        let rectangle = &self.0.rectangle;

        (rectangle.min.x as u32 + 1, rectangle.min.y as u32 + 1)
    }

    pub fn size(&self) -> (u32, u32) {
        let size = self.0.rectangle.size();

        (size.width as u32 - 2, size.height as u32 - 2)
    }
}

impl std::fmt::Debug for Allocator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Allocator")
    }
}

impl std::fmt::Debug for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Region {{ id: {:?}, rectangle: {:?} }}",
            self.0.id, self.0.rectangle
        )
    }
}
