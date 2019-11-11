use crate::{Length, Size};

#[derive(Debug, Clone, Copy)]
pub struct Limits {
    min: Size,
    max: Size,
    fill: Size,
}

impl Limits {
    pub const NONE: Limits = Limits {
        min: Size::ZERO,
        max: Size::INFINITY,
        fill: Size::INFINITY,
    };

    pub fn new(min: Size, max: Size) -> Limits {
        Limits {
            min,
            max,
            fill: Size::INFINITY,
        }
    }

    pub fn min(&self) -> Size {
        self.min
    }

    pub fn max(&self) -> Size {
        self.max
    }

    pub fn width(mut self, width: Length) -> Limits {
        match width {
            Length::Shrink => {
                self.fill.width = self.min.width;
            }
            Length::Fill => {
                self.fill.width = self.fill.width.min(self.max.width);
            }
            Length::Units(units) => {
                let new_width =
                    (units as f32).min(self.max.width).max(self.min.width);

                self.min.width = new_width;
                self.max.width = new_width;
                self.fill.width = new_width;
            }
        }

        self
    }

    pub fn height(mut self, height: Length) -> Limits {
        match height {
            Length::Shrink => {
                self.fill.height = self.min.height;
            }
            Length::Fill => {
                self.fill.height = self.fill.height.min(self.max.height);
            }
            Length::Units(units) => {
                let new_height =
                    (units as f32).min(self.max.height).max(self.min.height);

                self.min.height = new_height;
                self.max.height = new_height;
                self.fill.height = new_height;
            }
        }

        self
    }

    pub fn min_width(mut self, min_width: u32) -> Limits {
        self.min.width =
            self.min.width.max(min_width as f32).min(self.max.width);

        self
    }

    pub fn max_width(mut self, max_width: u32) -> Limits {
        self.max.width =
            self.max.width.min(max_width as f32).max(self.min.width);

        self
    }

    pub fn max_height(mut self, max_height: u32) -> Limits {
        self.max.height =
            self.max.height.min(max_height as f32).max(self.min.height);

        self
    }

    pub fn pad(&self, padding: f32) -> Limits {
        self.shrink(Size::new(padding * 2.0, padding * 2.0))
    }

    pub fn shrink(&self, size: Size) -> Limits {
        let min = Size::new(
            (self.min().width - size.width).max(0.0),
            (self.min().height - size.height).max(0.0),
        );

        let max = Size::new(
            (self.max().width - size.width).max(0.0),
            (self.max().height - size.height).max(0.0),
        );

        let fill = Size::new(
            (self.fill.width - size.width).max(0.0),
            (self.fill.height - size.height).max(0.0),
        );

        Limits { min, max, fill }
    }

    pub fn loose(&self) -> Limits {
        Limits {
            min: Size::ZERO,
            max: self.max,
            fill: self.fill,
        }
    }

    pub fn resolve(&self, intrinsic_size: Size) -> Size {
        Size::new(
            intrinsic_size
                .width
                .min(self.max.width)
                .max(self.fill.width),
            intrinsic_size
                .height
                .min(self.max.height)
                .max(self.fill.height),
        )
    }
}
