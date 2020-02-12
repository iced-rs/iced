use iced_native::Color;

#[derive(Debug, Clone, Copy)]
pub struct Stroke {
    pub color: Color,
    pub width: f32,
    pub line_cap: LineCap,
    pub line_join: LineJoin,
}

impl Default for Stroke {
    fn default() -> Stroke {
        Stroke {
            color: Color::BLACK,
            width: 1.0,
            line_cap: LineCap::default(),
            line_join: LineJoin::default(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LineCap {
    Butt,
    Square,
    Round,
}

impl Default for LineCap {
    fn default() -> LineCap {
        LineCap::Butt
    }
}

impl From<LineCap> for lyon::tessellation::LineCap {
    fn from(line_cap: LineCap) -> lyon::tessellation::LineCap {
        match line_cap {
            LineCap::Butt => lyon::tessellation::LineCap::Butt,
            LineCap::Square => lyon::tessellation::LineCap::Square,
            LineCap::Round => lyon::tessellation::LineCap::Round,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LineJoin {
    Miter,
    Round,
    Bevel,
}

impl Default for LineJoin {
    fn default() -> LineJoin {
        LineJoin::Miter
    }
}

impl From<LineJoin> for lyon::tessellation::LineJoin {
    fn from(line_join: LineJoin) -> lyon::tessellation::LineJoin {
        match line_join {
            LineJoin::Miter => lyon::tessellation::LineJoin::Miter,
            LineJoin::Round => lyon::tessellation::LineJoin::Round,
            LineJoin::Bevel => lyon::tessellation::LineJoin::Bevel,
        }
    }
}
