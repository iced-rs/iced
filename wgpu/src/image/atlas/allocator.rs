use guillotiere::{AtlasAllocator, Size};

pub struct Allocator {
    raw: AtlasAllocator,
    allocations: usize,
}

impl Allocator {
    pub fn new(size: u32) -> Allocator {
        let raw = AtlasAllocator::new(Size::new(size as i32, size as i32));

        Allocator {
            raw,
            allocations: 0,
        }
    }

    pub fn allocate(&mut self, width: u32, height: u32) -> Option<Region> {
        let allocation =
            self.raw.allocate(Size::new(width as i32, height as i32))?;

        self.allocations += 1;

        Some(Region { allocation })
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
}

impl Region {
    pub fn position(&self) -> (u32, u32) {
        let rectangle = &self.allocation.rectangle;

        (rectangle.min.x as u32, rectangle.min.y as u32)
    }

    pub fn size(&self) -> crate::core::Size<u32> {
        let size = self.allocation.rectangle.size();

        crate::core::Size::new(size.width as u32, size.height as u32)
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
