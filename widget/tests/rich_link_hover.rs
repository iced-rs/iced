//! Regression test: moving the cursor directly from one link to an
//! adjacent link must request a redraw, so the underline of the previously
//! hovered link is removed.
//!
//! A redraw used to be requested only when hover *began or ended*
//! (`Option::is_some` changed), so a `Some(0) -> Some(1)` transition left
//! the stale underline on screen.

use iced_widget::core::alignment;
use iced_widget::core::layout::{Layout, Limits};
use iced_widget::core::mouse;
use iced_widget::core::renderer::{self, Renderer, Scale, Settings};
use iced_widget::core::shell::{Bus, Waker};
use iced_widget::core::text::{
    self, Alignment, Difference, Ellipsis, Hit, LineHeight, Paragraph, Shaping, Span, Target, Text,
    Wrapping,
};
use iced_widget::core::widget::Tree;
use iced_widget::core::window::{Headless, RedrawRequest};
use iced_widget::core::{
    Background, Color, Event, Font, Pixels, Point, Rectangle, Shell, Size, Theme, Transformation,
    Widget,
};
use iced_widget::span;
use iced_widget::text::Rich;

/// A paragraph whose spans sit on two consecutive lines of 20px each:
/// span 0 (a link) on line 0, span 1 (a newline) with no glyph, span 2 (a
/// link) on line 1. `hit_span` therefore jumps straight from span 0 to span 2
/// when the cursor moves down, without ever hitting span 1.
#[derive(Default)]
struct MockParagraph;

impl Paragraph for MockParagraph {
    fn with_text(_text: Text<&str>) -> Self {
        Self
    }

    fn with_spans<Link>(_text: Text<&[Span<'_, Link>]>) -> Self {
        Self
    }

    fn resize(&mut self, _new_bounds: Size) {}

    fn compare(&self, _text: Text<()>) -> Difference {
        Difference::None
    }

    fn size(&self) -> Pixels {
        Pixels(16.0)
    }

    fn hint_factor(&self) -> Option<f32> {
        None
    }

    fn font(&self) -> Font {
        Font::DEFAULT
    }

    fn line_height(&self) -> LineHeight {
        LineHeight::Relative(1.25)
    }

    fn align_x(&self) -> Alignment {
        Alignment::Default
    }

    fn align_y(&self) -> alignment::Vertical {
        alignment::Vertical::Top
    }

    fn wrapping(&self) -> Wrapping {
        Wrapping::Word
    }

    fn ellipsis(&self) -> Ellipsis {
        Ellipsis::None
    }

    fn shaping(&self) -> Shaping {
        Shaping::Advanced
    }

    fn bounds(&self) -> Size {
        Size::new(200.0, 40.0)
    }

    fn min_bounds(&self) -> Size {
        Size::new(200.0, 40.0)
    }

    fn hit_test(&self, _point: Point) -> Option<Hit> {
        None
    }

    fn hit_span(&self, point: Point) -> Option<usize> {
        if point.y < 20.0 { Some(0) } else { Some(2) }
    }

    fn span_bounds(&self, index: usize) -> Vec<Rectangle> {
        let y = if index == 0 { 0.0 } else { 20.0 };
        vec![Rectangle::new(Point::new(0.0, y), Size::new(200.0, 20.0))]
    }

    fn select(&mut self, _start: Point, _end: Point, _target: Target) {}

    fn select_all(&mut self) {}

    fn deselect(&mut self) {}

    fn selection(&self) -> &[Rectangle] {
        &[]
    }

    fn copy(&mut self) -> Option<String> {
        None
    }
}

struct MockRenderer;

impl Renderer for MockRenderer {
    fn start_layer(&mut self, _bounds: Rectangle) {}
    fn end_layer(&mut self) {}
    fn start_transformation(&mut self, _transformation: Transformation) {}
    fn end_transformation(&mut self) {}
    fn fill_quad(&mut self, _quad: renderer::Quad, _background: impl Into<Background>) {}
    fn allocate_image(
        &self,
        handle: &iced_widget::core::image::Handle,
        callback: impl FnOnce(
            Result<iced_widget::core::image::Allocation, iced_widget::core::image::Error>,
        ) + Send
        + 'static,
    ) {
        #[allow(unsafe_code)]
        callback(Ok(unsafe {
            iced_widget::core::image::allocate(handle, Size::new(1, 1))
        }));
    }
    fn hint(&mut self, _scale: Scale) {}
    fn scale(&self) -> Option<Scale> {
        None
    }
    fn reset(&mut self, _new_bounds: Rectangle) {}
    fn settings(&self) -> Settings {
        Settings::default()
    }
}

impl text::Renderer for MockRenderer {
    type Paragraph = MockParagraph;
    type Editor = ();

    const ICON_FONT: Font = Font::DEFAULT;
    const CHECKMARK_ICON: char = '0';
    const ARROW_DOWN_ICON: char = '0';
    const SCROLL_UP_ICON: char = '0';
    const SCROLL_DOWN_ICON: char = '0';
    const SCROLL_LEFT_ICON: char = '0';
    const SCROLL_RIGHT_ICON: char = '0';
    const ICED_LOGO: char = '0';

    fn fill_paragraph(
        &mut self,
        _text: &MockParagraph,
        _position: Point,
        _color: Color,
        _clip_bounds: Rectangle,
    ) {
    }

    fn fill_editor(
        &mut self,
        _editor: &(),
        _position: Point,
        _color: Color,
        _clip_bounds: Rectangle,
    ) {
    }

    fn fill_text(
        &mut self,
        _text: Text<String>,
        _position: Point,
        _color: Color,
        _clip_bounds: Rectangle,
    ) {
    }
}

type TestWidget<'a> = Rich<'a, String, (), Theme>;

fn link_widget() -> TestWidget<'static> {
    let spans: Vec<Span<'static, String>> = vec![
        span("https://example.com").link("https://example.com".to_string()),
        span("\n"),
        span("https://google.com").link("https://google.com".to_string()),
    ];

    Rich::with_spans(spans).on_link_click(|_link: String| ())
}

struct Harness {
    widget: TestWidget<'static>,
    tree: Tree,
    layout: Layout,
    renderer: MockRenderer,
    window: Headless,
}

impl Harness {
    fn new() -> Self {
        let mut widget = link_widget();
        let mut tree = Tree::new::<(), Theme, MockRenderer>(&widget);
        let renderer = MockRenderer;
        let limits = Limits::new(Size::ZERO, Size::new(400.0, 100.0));

        widget.layout(&mut tree, &renderer, &limits);

        let layout = Layout::new(tree.size);

        Self {
            widget,
            tree,
            layout,
            renderer,
            window: Headless,
        }
    }

    /// Feeds a cursor move to the widget and returns whether a redraw was
    /// requested.
    fn move_cursor(&mut self, position: Option<Point>) -> bool {
        let cursor = match position {
            Some(p) => mouse::Cursor::Available(p),
            None => mouse::Cursor::Unavailable,
        };
        let event = Event::Mouse(mouse::Event::CursorMoved {
            position: cursor.position().unwrap_or(Point::ORIGIN),
        });

        let mut bus: Bus<()> = Bus::default();
        let mut shell = Shell::new(&self.window, Waker::noop(), &mut bus);
        let viewport = Rectangle::with_size(Size::new(1000.0, 1000.0));

        self.widget.update(
            &mut self.tree,
            &event,
            self.layout,
            cursor,
            &self.renderer,
            &mut shell,
            &viewport,
        );

        shell.redraw_request() == RedrawRequest::NextFrame
    }

    fn is_pointer(&self, position: Option<Point>) -> bool {
        let cursor = match position {
            Some(p) => mouse::Cursor::Available(p),
            None => mouse::Cursor::Unavailable,
        };
        let viewport = Rectangle::with_size(Size::new(1000.0, 1000.0));

        self.widget
            .mouse_interaction(&self.tree, self.layout, cursor, &viewport, &self.renderer)
            == mouse::Interaction::Pointer
    }
}

#[test]
fn hovering_adjacent_link_requests_redraw() {
    let mut harness = Harness::new();

    // Hover the first link: hover begins, a redraw is requested.
    assert!(harness.move_cursor(Some(Point::new(10.0, 10.0))));
    assert!(harness.is_pointer(Some(Point::new(10.0, 10.0))));

    // Move straight down onto the second link (still inside the widget
    // bounds, never hitting the non-link newline span): the hovered link
    // changes from Some(0) to Some(2), so a redraw must be requested to
    // remove the stale underline from the first link.
    assert!(
        harness.move_cursor(Some(Point::new(10.0, 30.0))),
        "moving directly from one link to an adjacent link must request a redraw"
    );
    assert!(harness.is_pointer(Some(Point::new(10.0, 30.0))));

    // Staying on the same link must not request a redraw.
    assert!(!harness.move_cursor(Some(Point::new(12.0, 31.0))));

    // Leaving the widget entirely clears the hovered link.
    assert!(!harness.move_cursor(None));
    assert!(!harness.is_pointer(None));
}
