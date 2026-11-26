use crate::core::alignment;
use crate::core::border;
use crate::core::layout;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::text::{Paragraph, Span, Target};
use crate::core::widget;
use crate::core::widget::operation;
use crate::core::widget::text::{
    self, Alignment, Catalog, Ellipsis, LineHeight, Shaping, Style, StyleFn, Wrapping,
};
use crate::core::widget::tree::{self, Tree};
use crate::core::window;
use crate::core::{
    self, Border, Color, Event, Font, Layout, Length, Pixels, Point, Rectangle, Shell, Size,
    Vector, Widget,
};

/// A bunch of [`Rich`] text.
pub struct Rich<'a, Link, Message, Theme = crate::Theme>
where
    Link: Clone + 'static,
    Theme: Catalog,
{
    spans: Box<dyn AsRef<[Span<'a, Link>]> + 'a>,
    size: Option<Pixels>,
    line_height: Option<LineHeight>,
    width: Length,
    height: Length,
    font: Option<Font>,
    align_x: Alignment,
    align_y: alignment::Vertical,
    wrapping: Wrapping,
    ellipsis: Ellipsis,
    class: Theme::Class<'a>,
    on_link_click: Option<Box<dyn Fn(Link) -> Message + 'a>>,
    selectable: bool,
    hovered_link: Option<usize>,
    is_hovered: bool,
}

impl<'a, Link, Message, Theme> Rich<'a, Link, Message, Theme>
where
    Link: Clone + 'static,
    Theme: Catalog,
{
    /// Creates a new empty [`Rich`] text.
    pub fn new() -> Self {
        Self {
            spans: Box::new([]),
            size: None,
            line_height: None,
            width: Length::Fit,
            height: Length::Fit,
            font: None,
            align_x: Alignment::Default,
            align_y: alignment::Vertical::Top,
            wrapping: Wrapping::default(),
            ellipsis: Ellipsis::default(),
            class: Theme::default(),
            on_link_click: None,
            selectable: false,
            hovered_link: None,
            is_hovered: false,
        }
    }

    /// Creates a new [`Rich`] text with the given text spans.
    pub fn with_spans(spans: impl AsRef<[Span<'a, Link>]> + 'a) -> Self {
        Self {
            spans: Box::new(spans),
            ..Self::new()
        }
    }

    /// Sets the default size of the [`Rich`] text.
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = Some(size.into());
        self
    }

    /// Sets the default [`LineHeight`] of the [`Rich`] text.
    pub fn line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
        self.line_height = Some(line_height.into());
        self
    }

    /// Sets the default font of the [`Rich`] text.
    pub fn font(mut self, font: impl Into<Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    /// Sets the width of the [`Rich`] text boundaries.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Rich`] text boundaries.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Centers the [`Rich`] text, both horizontally and vertically.
    pub fn center(self) -> Self {
        self.align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
    }

    /// Sets the [`alignment::Horizontal`] of the [`Rich`] text.
    pub fn align_x(mut self, alignment: impl Into<Alignment>) -> Self {
        self.align_x = alignment.into();
        self
    }

    /// Sets the [`alignment::Vertical`] of the [`Rich`] text.
    pub fn align_y(mut self, alignment: impl Into<alignment::Vertical>) -> Self {
        self.align_y = alignment.into();
        self
    }

    /// Sets the [`Wrapping`] strategy of the [`Rich`] text.
    pub fn wrapping(mut self, wrapping: Wrapping) -> Self {
        self.wrapping = wrapping;
        self
    }

    /// Sets the [`Ellipsis`] strategy of the [`Rich`] text.
    pub fn ellipsis(mut self, ellipsis: Ellipsis) -> Self {
        self.ellipsis = ellipsis;
        self
    }

    /// Sets the message that will be produced when a link of the [`Rich`] text
    /// is clicked.
    ///
    /// If the spans of the [`Rich`] text contain no links, you may need to call
    /// this method with `on_link_click(never)` in order for the compiler to infer
    /// the proper `Link` generic type.
    pub fn on_link_click(mut self, on_link_click: impl Fn(Link) -> Message + 'a) -> Self {
        self.on_link_click = Some(Box::new(on_link_click));
        self
    }

    /// Sets whether the [`Rich`] text can be selected.
    ///
    /// By default, it is `false`.
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    /// Sets the default style of the [`Rich`] text.
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the default [`Color`] of the [`Rich`] text.
    pub fn color(self, color: impl Into<Color>) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.color_maybe(Some(color))
    }

    /// Sets the default [`Color`] of the [`Rich`] text, if `Some`.
    pub fn color_maybe(self, color: Option<impl Into<Color>>) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        let color = color.map(Into::into);

        self.style(move |_theme| Style {
            color,
            selection: None,
        })
    }

    /// Sets the default style class of the [`Rich`] text.
    #[cfg(feature = "advanced")]
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

impl<'a, Link, Message, Theme> Default for Rich<'a, Link, Message, Theme>
where
    Link: Clone + 'a,
    Theme: Catalog,
{
    fn default() -> Self {
        Self::new()
    }
}

struct State<Link, P: Paragraph> {
    spans: Vec<Span<'static, Link>>,
    span_pressed: Option<usize>,
    paragraph: P,
}

impl<Link, Message, Theme> widget::Meta for Rich<'_, Link, Message, Theme>
where
    Link: Clone + 'static,
    Theme: Catalog,
{
}

impl<Link, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Rich<'_, Link, Message, Theme>
where
    Link: Clone + 'static,
    Theme: Catalog,
    Renderer: core::text::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Link, Renderer::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::<Link, _> {
            spans: Vec::new(),
            span_pressed: None,
            paragraph: Renderer::Paragraph::default(),
        })
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) {
        tree.size = layout(
            tree.state
                .downcast_mut::<State<Link, Renderer::Paragraph>>(),
            renderer,
            limits,
            self.width,
            self.height,
            self.spans.as_ref().as_ref(),
            self.line_height,
            self.size,
            self.font,
            self.align_x,
            self.align_y,
            self.wrapping,
            self.ellipsis,
        );
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        defaults: &renderer::Style,
        layout: Layout,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        if !layout.bounds().intersects(viewport) {
            return;
        }

        let state = tree
            .state
            .downcast_ref::<State<Link, Renderer::Paragraph>>();

        let style = theme.style(&self.class);
        let translation = layout.position() - Point::ORIGIN;
        let mut has_decorations = self.hovered_link.is_some();

        for (index, span) in self.spans.as_ref().as_ref().iter().enumerate() {
            if let Some(highlight) = span.highlight {
                let regions = state.paragraph.span_bounds(index);

                for (i, bounds) in regions.iter().enumerate() {
                    let starts = i == 0;
                    let ends = i + 1 == regions.len();

                    // The horizontal padding belongs to the start and end
                    // of the span, not to each of its lines
                    let left = if starts { span.padding.left } else { 0.0 };
                    let right = if ends { span.padding.right } else { 0.0 };

                    let highlight_bounds = Rectangle::new(
                        bounds.position() + translation - Vector::new(left, span.padding.top),
                        bounds.size() + Size::new(left + right, span.padding.y()),
                    );

                    let Some(clipped_bounds) = highlight_bounds.intersection(viewport) else {
                        continue;
                    };

                    let radius = border::Radius {
                        top_left: highlight.border.radius.top_left * f32::from(starts),
                        bottom_left: highlight.border.radius.bottom_left * f32::from(starts),
                        top_right: highlight.border.radius.top_right * f32::from(ends),
                        bottom_right: highlight.border.radius.bottom_right * f32::from(ends),
                    };

                    let draw_highlight = |renderer: &mut Renderer, quad_bounds| {
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: quad_bounds,
                                border: Border {
                                    radius,
                                    ..highlight.border
                                },
                                ..Default::default()
                            },
                            highlight.background,
                        );
                    };

                    if clipped_bounds == highlight_bounds
                        || (highlight.border.width == 0.0 && radius == border::Radius::default())
                    {
                        draw_highlight(renderer, clipped_bounds);
                    } else {
                        renderer.with_layer(*viewport, |renderer| {
                            draw_highlight(renderer, highlight_bounds);
                        });
                    }
                }
            }

            has_decorations |= span.underline || span.strikethrough;
        }

        text::draw(
            renderer,
            defaults,
            layout.bounds(),
            &state.paragraph,
            style,
            theme.selection(),
            viewport,
        );

        if !has_decorations {
            return;
        }

        for (index, span) in self.spans.as_ref().as_ref().iter().enumerate() {
            let is_hovered_link = self.on_link_click.is_some() && Some(index) == self.hovered_link;

            if span.underline || span.strikethrough || is_hovered_link {
                let regions = state.paragraph.span_bounds(index);
                let size = span.size.or(self.size).unwrap_or(renderer.text_size());

                let line_height = span
                    .line_height
                    .or(self.line_height)
                    .unwrap_or_else(|| renderer.line_height())
                    .to_absolute(size);

                let color = span.color.or(style.color).unwrap_or(defaults.text_color);

                let baseline =
                    translation + Vector::new(0.0, size.0 + (line_height.0 - size.0) / 2.0);

                let fill_decoration = |renderer: &mut Renderer, bounds: Rectangle| {
                    if let Some(clipped) = bounds.intersection(viewport) {
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: clipped,
                                ..Default::default()
                            },
                            color,
                        );
                    }
                };

                if span.underline || is_hovered_link {
                    for bounds in &regions {
                        fill_decoration(
                            renderer,
                            Rectangle::new(
                                bounds.position() + baseline,
                                Size::new(bounds.width, 1.0),
                            ),
                        );
                    }
                }

                if span.strikethrough {
                    for bounds in &regions {
                        fill_decoration(
                            renderer,
                            Rectangle::new(
                                bounds.position() + baseline - Vector::new(0.0, size.0 / 2.0),
                                Size::new(bounds.width, 1.0),
                            ),
                        );
                    }
                }
            }
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Window(window::Event::RedrawRequested(_)) => {
                let Some(position) = cursor.position_in(layout.bounds()) else {
                    self.is_hovered = false;
                    self.hovered_link = None;
                    return;
                };

                let state = tree
                    .state
                    .downcast_ref::<State<Link, Renderer::Paragraph>>();

                self.is_hovered = state.paragraph.hit_glyph(position);

                if self.on_link_click.is_none() {
                    return;
                };

                let was_hovered = self.hovered_link.is_some();

                self.hovered_link = state.paragraph.hit_span(position).and_then(|span| {
                    if self.spans.as_ref().as_ref().get(span)?.link.is_some() {
                        Some(span)
                    } else {
                        None
                    }
                });

                if was_hovered != self.hovered_link.is_some() && !matches!(event, Event::Window(_))
                {
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let state = tree
                    .state
                    .downcast_mut::<State<Link, Renderer::Paragraph>>();

                if self.hovered_link.is_some() {
                    state.span_pressed = self.hovered_link;
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let Some(on_link_clicked) = &self.on_link_click else {
                    return;
                };

                let state = tree
                    .state
                    .downcast_mut::<State<Link, Renderer::Paragraph>>();

                match state.span_pressed {
                    Some(span) if Some(span) == self.hovered_link => {
                        if let Some(link) = self
                            .spans
                            .as_ref()
                            .as_ref()
                            .get(span)
                            .and_then(|span| span.link.clone())
                        {
                            shell.publish(on_link_clicked(link));
                        }
                    }
                    _ => {}
                }

                state.span_pressed = None;
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        _layout: Layout,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if self.hovered_link.is_some() {
            mouse::Interaction::Pointer
        } else if self.selectable && self.is_hovered {
            mouse::Interaction::Text
        } else {
            mouse::Interaction::None
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout,
        _viewport: &Rectangle,
        _renderer: &Renderer,
        operation: &mut dyn core::widget::Operation,
    ) {
        struct Operand<'a, Link, P: Paragraph> {
            pub state: &'a mut State<Link, P>,
            pub layout: Layout,
            pub selectable: bool,
        }

        impl<Link, P: Paragraph> operation::Text for Operand<'_, Link, P> {
            fn text(&self) -> core::text::Fragment<'_> {
                self.state
                    .spans
                    .iter()
                    .fold(String::new(), |mut text, next| {
                        text.push_str(&next.text);
                        text
                    })
                    .into()
            }

            fn select(&mut self, start: Point, end: Point, target: Target) {
                if !self.selectable {
                    return;
                }

                let translation = self.layout.position() - Point::ORIGIN;

                self.state
                    .paragraph
                    .select(start - translation, end - translation, target);
            }

            fn select_all(&mut self) {
                self.state.paragraph.select_all();
            }

            fn deselect(&mut self) {
                self.state.paragraph.deselect();
            }

            fn copy(&mut self) -> Option<String> {
                self.state.paragraph.copy()
            }
        }

        let state = tree
            .state
            .downcast_mut::<State<Link, Renderer::Paragraph>>();

        operation.text(
            None,
            layout.bounds(),
            &mut Operand {
                state,
                layout,
                selectable: self.selectable,
            },
        );
    }
}

fn layout<Link, Renderer>(
    state: &mut State<Link, Renderer::Paragraph>,
    renderer: &Renderer,
    limits: &layout::Limits,
    width: Length,
    height: Length,
    spans: &[Span<'_, Link>],
    line_height: Option<LineHeight>,
    size: Option<Pixels>,
    font: Option<Font>,
    align_x: Alignment,
    align_y: alignment::Vertical,
    wrapping: Wrapping,
    ellipsis: Ellipsis,
) -> Size
where
    Link: Clone,
    Renderer: core::text::Renderer,
{
    layout::sized(limits, width, height, |limits| {
        let bounds = limits.bounds();

        let size = size.unwrap_or_else(|| renderer.text_size());
        let font = font.unwrap_or_else(|| renderer.font());
        let line_height = line_height.unwrap_or_else(|| renderer.line_height());

        let text_with_spans = || core::Text {
            content: spans,
            bounds,
            size,
            line_height,
            font,
            align_x,
            align_y,
            shaping: Shaping::Advanced,
            wrapping,
            ellipsis,
            hint_factor: renderer.hint_factor(),
        };

        if state.spans != spans {
            state.paragraph = Renderer::Paragraph::with_spans(text_with_spans());
            state.spans = spans.iter().cloned().map(Span::to_static).collect();
        } else {
            match state.paragraph.compare(core::Text {
                content: (),
                bounds,
                size,
                line_height,
                font,
                align_x,
                align_y,
                shaping: Shaping::Advanced,
                wrapping,
                ellipsis,
                hint_factor: renderer.hint_factor(),
            }) {
                core::text::Difference::None => {}
                core::text::Difference::Bounds => {
                    state.paragraph.resize(bounds);
                }
                core::text::Difference::Shape => {
                    state.paragraph = Renderer::Paragraph::with_spans(text_with_spans());
                }
            }
        }

        state.paragraph.min_bounds()
    })
}

impl<'a, Link, Message, Theme> FromIterator<Span<'a, Link>> for Rich<'a, Link, Message, Theme>
where
    Link: Clone + 'a,
    Theme: Catalog,
{
    fn from_iter<T: IntoIterator<Item = Span<'a, Link>>>(spans: T) -> Self {
        Self::with_spans(spans.into_iter().collect::<Vec<_>>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::alignment;
    use crate::core::image;
    use crate::core::layout;
    use crate::core::mouse;
    use crate::core::renderer;
    use crate::core::text::{self, Text};
    use crate::core::widget::Tree;
    use crate::core::{
        Background, Border, Color, Font, Layout, Pixels, Point, Rectangle, Size, Transformation,
        Widget,
    };

    #[derive(Debug, PartialEq)]
    enum DrawOp {
        StartLayer(Rectangle),
        EndLayer,
        Quad(Rectangle),
        ParagraphClip(Rectangle),
    }

    #[derive(Default)]
    struct RecordingRenderer {
        ops: Vec<DrawOp>,
    }

    impl core::Renderer for RecordingRenderer {
        fn start_layer(&mut self, bounds: Rectangle) {
            self.ops.push(DrawOp::StartLayer(bounds));
        }

        fn end_layer(&mut self) {
            self.ops.push(DrawOp::EndLayer);
        }

        fn start_transformation(&mut self, _transformation: Transformation) {}

        fn end_transformation(&mut self) {}

        fn fill_quad(&mut self, quad: renderer::Quad, _background: impl Into<Background>) {
            self.ops.push(DrawOp::Quad(quad.bounds));
        }

        fn allocate_image(
            &self,
            _handle: &image::Handle,
            _callback: impl FnOnce(Result<image::Allocation, image::Error>) + Send + 'static,
        ) {
        }

        fn hint(&mut self, _scale: renderer::Scale) {}

        fn scale(&self) -> Option<renderer::Scale> {
            None
        }

        fn reset(&mut self, _new_bounds: Rectangle) {
            self.ops.clear();
        }

        fn settings(&self) -> renderer::Settings {
            renderer::Settings::default()
        }
    }

    impl core::text::Renderer for RecordingRenderer {
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
            _text: &Self::Paragraph,
            _position: Point,
            _color: Color,
            clip_bounds: Rectangle,
        ) {
            self.ops.push(DrawOp::ParagraphClip(clip_bounds));
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
            _text: Text<String>,
            _position: Point,
            _color: Color,
            _clip_bounds: Rectangle,
        ) {
        }
    }

    #[derive(Debug, Clone, Default)]
    struct MockParagraph {
        span_bounds: Vec<Vec<Rectangle>>,
        bounds: Size,
    }

    impl text::Paragraph for MockParagraph {
        fn with_text(_text: Text<&str>) -> Self {
            Self::default()
        }

        fn with_spans<Link>(text: Text<&[Span<'_, Link>]>) -> Self {
            let mut x = 0.0;
            let mut span_bounds = Vec::with_capacity(text.content.len());

            for span in text.content {
                let width = span.text.len() as f32 * 10.0;
                span_bounds.push(vec![Rectangle::new(
                    Point::new(x, 0.0),
                    Size::new(width, 20.0),
                )]);
                x += width;
            }

            Self {
                span_bounds,
                bounds: Size::new(x, 20.0),
            }
        }

        fn resize(&mut self, _new_bounds: Size) {}

        fn compare(&self, _text: Text<()>) -> text::Difference {
            text::Difference::Shape
        }

        fn hint_factor(&self) -> Option<f32> {
            None
        }

        fn size(&self) -> Pixels {
            Pixels(16.0)
        }

        fn font(&self) -> Font {
            Font::DEFAULT
        }

        fn line_height(&self) -> LineHeight {
            LineHeight::default()
        }

        fn align_x(&self) -> Alignment {
            Alignment::Default
        }

        fn align_y(&self) -> alignment::Vertical {
            alignment::Vertical::Top
        }

        fn wrapping(&self) -> Wrapping {
            Wrapping::default()
        }

        fn ellipsis(&self) -> Ellipsis {
            Ellipsis::default()
        }

        fn shaping(&self) -> Shaping {
            Shaping::default()
        }

        fn bounds(&self) -> Size {
            self.bounds
        }

        fn min_bounds(&self) -> Size {
            self.bounds
        }

        fn hit_test(&self, _point: Point) -> Option<text::Hit> {
            None
        }

        fn hit_span(&self, _point: Point) -> Option<usize> {
            None
        }

        fn span_bounds(&self, index: usize) -> Vec<Rectangle> {
            self.span_bounds.get(index).cloned().unwrap_or_default()
        }

        fn select(&mut self, _start: Point, _end: Point, _target: text::Target) {}
        fn select_all(&mut self) {}
        fn deselect(&mut self) {}

        fn selection(&self) -> &[Rectangle] {
            &[]
        }

        fn copy(&mut self) -> Option<String> {
            None
        }
    }

    fn draw_rich<Link: Clone + 'static>(
        mut rich: Rich<'_, Link, (), crate::Theme>,
        viewport: Rectangle,
    ) -> Vec<DrawOp> {
        let mut renderer = RecordingRenderer::default();
        let theme = crate::Theme::Dark;
        let mut tree = Tree::new::<(), crate::Theme, RecordingRenderer>(&rich);

        <Rich<'_, Link, (), crate::Theme> as Widget<(), crate::Theme, RecordingRenderer>>::layout(
            &mut rich,
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(500.0, 500.0)),
        );

        let layout = Layout::new(Size::new(500.0, 20.0));

        <Rich<'_, Link, (), crate::Theme> as Widget<(), crate::Theme, RecordingRenderer>>::draw(
            &rich,
            &tree,
            &mut renderer,
            &theme,
            &renderer::Style::default(),
            layout,
            mouse::Cursor::Unavailable,
            &viewport,
        );

        renderer.ops
    }

    #[test]
    fn draw_clips_underlines_and_strikethroughs_to_viewport() {
        let spans = [
            Span::<()>::new("12345").underline(true).strikethrough(true), // x: 0..50
            Span::new("123456").underline(true).strikethrough(true), // x: 50..110 (partially clipped)
            Span::new("12345").underline(true).strikethrough(true), // x: 110..160 (outside viewport)
        ];

        let viewport = Rectangle::new(Point::ORIGIN, Size::new(80.0, 25.0));
        let ops = draw_rich(Rich::with_spans(spans), viewport);

        assert_eq!(ops[0], DrawOp::ParagraphClip(viewport));

        let quads: Vec<Rectangle> = ops
            .into_iter()
            .filter_map(|op| match op {
                DrawOp::Quad(bounds) => Some(bounds),
                _ => None,
            })
            .collect();

        // Span 0: underline (0..50), strikethrough (0..50)
        // Span 1: underline clipped (50..80), strikethrough clipped (50..80)
        // Span 2: outside viewport -> skipped
        assert_eq!(quads.len(), 4);
        assert_eq!(quads[0].x, 0.0);
        assert_eq!(quads[0].width, 50.0);
        assert_eq!(quads[1].x, 0.0);
        assert_eq!(quads[1].width, 50.0);
        assert_eq!(quads[2].x, 50.0);
        assert_eq!(quads[2].width, 30.0);
        assert_eq!(quads[3].x, 50.0);
        assert_eq!(quads[3].width, 30.0);

        for quad in quads {
            assert!(quad.x >= viewport.x);
            assert!(quad.x + quad.width <= viewport.x + viewport.width);
            assert!(quad.y >= viewport.y);
            assert!(quad.y + quad.height <= viewport.y + viewport.height);
        }
    }

    #[test]
    fn draw_clips_hovered_link_underline_vertically() {
        let mut rich: Rich<'_, (), (), crate::Theme> =
            Rich::with_spans([Span::new("12345678").link(())]).on_link_click(|()| ());
        rich.hovered_link = Some(0);

        // Viewport ends at y=18.0, above the underline at baseline y=18.4..19.4
        let viewport = Rectangle::new(Point::ORIGIN, Size::new(100.0, 18.0));
        let ops = draw_rich(rich, viewport);

        assert!(!ops.iter().any(|op| matches!(op, DrawOp::Quad(_))));
    }

    #[test]
    fn draw_clips_highlights_with_and_without_borders() {
        let flat_highlight = Span::<()>::new("12345678").background(Color::WHITE); // x: 0..80
        let rounded_highlight = Span::<()>::new("123456") // x: 80..140
            .background(Color::BLACK)
            .border(Border::default().rounded(4.0));
        let outside_highlight = Span::<()>::new("12345").background(Color::WHITE); // x: 140..190

        let viewport = Rectangle::new(Point::ORIGIN, Size::new(100.0, 20.0));
        let ops = draw_rich(
            Rich::with_spans([flat_highlight, rounded_highlight, outside_highlight]),
            viewport,
        );

        assert_eq!(
            ops,
            vec![
                DrawOp::Quad(Rectangle::new(Point::ORIGIN, Size::new(80.0, 20.0))),
                DrawOp::StartLayer(viewport),
                DrawOp::Quad(Rectangle::new(Point::new(80.0, 0.0), Size::new(60.0, 20.0))),
                DrawOp::EndLayer,
                DrawOp::ParagraphClip(viewport),
            ]
        );
    }
}
