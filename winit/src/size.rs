pub struct Size {
    physical: winit::dpi::PhysicalSize<u32>,
    logical: winit::dpi::LogicalSize<f64>,
    scale_factor: f64,
}

impl Size {
    pub fn new(
        physical: winit::dpi::PhysicalSize<u32>,
        scale_factor: f64,
    ) -> Size {
        Size {
            logical: physical.to_logical(scale_factor),
            physical,
            scale_factor,
        }
    }

    pub fn physical(&self) -> winit::dpi::PhysicalSize<u32> {
        self.physical
    }

    pub fn logical(&self) -> winit::dpi::LogicalSize<f64> {
        self.logical
    }

    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }
}
