use crate::alignment;
use crate::image;
use crate::renderer::{self, Renderer};
use crate::svg;
use crate::text::{self, Text};
use crate::{
    Background, Color, Font, Pixels, Point, Radians, Rectangle, Size,
    Transformation,
};

impl Renderer for () {
    fn start_layer(&mut self, _bounds: Rectangle) {}

    fn end_layer(&mut self) {}

    fn start_transformation(&mut self, _transformation: Transformation) {}

    fn end_transformation(&mut self) {}

    fn clear(&mut self) {}

    fn fill_quad(
        &mut self,
        _quad: renderer::Quad,
        _background: impl Into<Background>,
    ) {
    }
}

impl text::Renderer for () {
    type Font = Font;
    type Paragraph = ();
    type Editor = ();

    const ICON_FONT: Font = Font::DEFAULT;
    const CHECKMARK_ICON: char = '0';
    const ARROW_DOWN_ICON: char = '0';

    fn default_font(&self) -> Self::Font {
        Font::default()
    }

    fn default_size(&self) -> Pixels {
        Pixels(16.0)
    }

    fn fill_paragraph(
        &mut self,
        _paragraph: &Self::Paragraph,
        _position: Point,
        _color: Color,
        _clip_bounds: Rectangle,
    ) {
    }

    fn fill_editor(
        &mut self,
        _editor: &Self::Editor,
        _position: Point,
        _color: Color,
        _clip_bounds: Rectangle,
    ) {
    }

    fn fill_text(
        &mut self,
        _paragraph: Text,
        _position: Point,
        _color: Color,
        _clip_bounds: Rectangle,
    ) {
    }
}

impl text::Paragraph for () {
    type Font = Font;

    fn with_text(_text: Text<&str>) -> Self {}

    fn resize(&mut self, _new_bounds: Size) {}

    fn compare(&self, _text: Text<&str>) -> text::Difference {
        text::Difference::None
    }

    fn horizontal_alignment(&self) -> alignment::Horizontal {
        alignment::Horizontal::Left
    }

    fn vertical_alignment(&self) -> alignment::Vertical {
        alignment::Vertical::Top
    }

    fn grapheme_position(&self, _line: usize, _index: usize) -> Option<Point> {
        None
    }

    fn min_bounds(&self) -> Size {
        Size::ZERO
    }

    fn hit_test(&self, _point: Point) -> Option<text::Hit> {
        None
    }
}

impl text::Editor for () {
    type Font = Font;

    fn with_text(_text: &str) -> Self {}

    fn cursor(&self) -> text::editor::Cursor {
        text::editor::Cursor::Caret(Point::ORIGIN)
    }

    fn cursor_position(&self) -> (usize, usize) {
        (0, 0)
    }

    fn selection(&self) -> Option<String> {
        None
    }

    fn line(&self, _index: usize) -> Option<&str> {
        None
    }

    fn line_count(&self) -> usize {
        0
    }

    fn perform(&mut self, _action: text::editor::Action) {}

    fn bounds(&self) -> Size {
        Size::ZERO
    }

    fn min_bounds(&self) -> Size {
        Size::ZERO
    }

    fn update(
        &mut self,
        _new_bounds: Size,
        _new_font: Self::Font,
        _new_size: Pixels,
        _new_line_height: text::LineHeight,
        _new_highlighter: &mut impl text::Highlighter,
    ) {
    }

    fn highlight<H: text::Highlighter>(
        &mut self,
        _font: Self::Font,
        _highlighter: &mut H,
        _format_highlight: impl Fn(
            &H::Highlight,
        ) -> text::highlighter::Format<Self::Font>,
    ) {
    }
}

impl image::Renderer for () {
    type Handle = ();

    fn measure_image(&self, _handle: &Self::Handle) -> Size<u32> {
        Size::default()
    }

    fn draw_image(
        &mut self,
        _handle: Self::Handle,
        _filter_method: image::FilterMethod,
        _bounds: Rectangle,
        _rotation: Radians,
        _opacity: f32,
    ) {
    }
}

impl svg::Renderer for () {
    fn measure_svg(&self, _handle: &svg::Handle) -> Size<u32> {
        Size::default()
    }

    fn draw_svg(
        &mut self,
        _handle: svg::Handle,
        _color: Option<Color>,
        _bounds: Rectangle,
        _rotation: Radians,
        _opacity: f32,
    ) {
    }
}
