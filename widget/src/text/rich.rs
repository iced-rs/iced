use crate::core::alignment;
use crate::core::keyboard;
use crate::core::layout;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::text::{Paragraph, Span};
use crate::core::widget::operation::Selectable;
use crate::core::widget::text::{
    self, Alignment, Catalog, Ellipsis, LineHeight, Shaping, Style, StyleFn, Wrapping,
};
use crate::core::widget::tree::{self, Tree};
use crate::core::{
    self, Color, Element, Event, Layout, Length, Pixels, Point, Rectangle, Shell, Size, Vector,
    Widget,
};

/// A bunch of [`Rich`] text.
pub struct Rich<'a, Link, Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    Link: Clone + 'static,
    Theme: Catalog,
    Renderer: core::text::Renderer,
{
    spans: Box<dyn AsRef<[Span<'a, Link, Renderer::Font>]> + 'a>,
    size: Option<Pixels>,
    line_height: LineHeight,
    width: Length,
    height: Length,
    font: Option<Renderer::Font>,
    align_x: Alignment,
    align_y: alignment::Vertical,
    wrapping: Wrapping,
    ellipsis: Ellipsis,
    class: Theme::Class<'a>,
    hovered_link: Option<usize>,
    on_link_click: Option<Box<dyn Fn(Link) -> Message + 'a>>,
    selectable: bool,
}

impl<'a, Link, Message, Theme, Renderer> Rich<'a, Link, Message, Theme, Renderer>
where
    Link: Clone + 'static,
    Theme: Catalog,
    Renderer: core::text::Renderer,
    Renderer::Font: 'a,
{
    /// Creates a new empty [`Rich`] text.
    pub fn new() -> Self {
        Self {
            spans: Box::new([]),
            size: None,
            line_height: LineHeight::default(),
            width: Length::Shrink,
            height: Length::Shrink,
            font: None,
            align_x: Alignment::Default,
            align_y: alignment::Vertical::Top,
            wrapping: Wrapping::default(),
            ellipsis: Ellipsis::default(),
            class: Theme::default(),
            hovered_link: None,
            on_link_click: None,
            selectable: false,
        }
    }

    /// Allows the user to drag-select text inside the [`Rich`] and
    /// copy it with `Ctrl+C` while focused. Off by default.
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    /// Creates a new [`Rich`] text with the given text spans.
    pub fn with_spans(spans: impl AsRef<[Span<'a, Link, Renderer::Font>]> + 'a) -> Self {
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
        self.line_height = line_height.into();
        self
    }

    /// Sets the default font of the [`Rich`] text.
    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
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

        // Inherit `selection` (and any other field) from the theme's
        // default class so a per-widget color override doesn't silently
        // disable the selection highlight.
        self.style(move |theme: &Theme| Style {
            color,
            ..theme.style(&<Theme as Catalog>::default())
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

impl<'a, Link, Message, Theme, Renderer> Default for Rich<'a, Link, Message, Theme, Renderer>
where
    Link: Clone + 'a,
    Theme: Catalog,
    Renderer: core::text::Renderer,
    Renderer::Font: 'a,
{
    fn default() -> Self {
        Self::new()
    }
}

/// The internal state of a [`Rich`] widget. Implements the
/// [`Selectable`] operation hook so coordinator widgets like
/// [`selectable_group`] can read and write the selection without
/// touching this concrete type.
///
/// [`Selectable`]: core::widget::operation::Selectable
/// [`selectable_group`]: crate::selectable_group
struct State<Link, P: Paragraph> {
    spans: Vec<Span<'static, Link, P::Font>>,
    /// Cached concatenation of every span's text, kept in sync with
    /// `spans` during layout. Lets the keyboard navigation helpers
    /// walk codepoints / words without allocating a fresh `String`
    /// per keystroke.
    text: String,
    span_pressed: Option<usize>,
    paragraph: P,
    selection: Option<(usize, usize)>,
    selecting: bool,
    focused: bool,
    externally_managed: bool,
    /// Most recent left-click; chained into `mouse::Click::new` so
    /// repeated presses within iced's threshold escalate Single →
    /// Double → Triple.
    last_click: Option<mouse::Click>,
}

impl<Link, P: Paragraph> core::widget::operation::Selectable for State<Link, P> {
    fn selection(&self) -> Option<(usize, usize)> {
        self.selection
    }

    fn set_selection(&mut self, range: Option<(usize, usize)>) {
        self.selection = range;
    }

    fn text(&self) -> &str {
        &self.text
    }

    fn byte_position(&self, byte: usize) -> Option<Point> {
        self.paragraph.byte_position(byte)
    }

    fn hit_test(&self, point: Point) -> Option<usize> {
        self.paragraph.hit_test(point).map(core::text::Hit::cursor)
    }

    fn visual_line_height(&self) -> Option<f32> {
        self.paragraph.visual_line_height()
    }

    fn min_bounds_height(&self) -> f32 {
        self.paragraph.min_bounds().height
    }

    fn bounds_width(&self) -> f32 {
        self.paragraph.bounds().width
    }

    fn set_externally_managed(&mut self, value: bool) {
        self.externally_managed = value;
    }

    /// Override: rich text needs to walk per-span to extract selection
    /// content, since `self.text` is a flat concatenation that doesn't
    /// preserve span boundaries — but the public API has to return
    /// what the user *sees* and that's the per-span text.
    fn selection_text(&self, start: usize, end: usize) -> String {
        collect_selection(&self.spans, start, end)
    }
}

impl<Link, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Rich<'_, Link, Message, Theme, Renderer>
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
            text: String::new(),
            span_pressed: None,
            paragraph: Renderer::Paragraph::default(),
            selection: None,
            selecting: false,
            focused: false,
            externally_managed: false,
            last_click: None,
        })
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout(
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
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn core::widget::Operation,
    ) {
        if !self.selectable {
            return;
        }
        let state = tree
            .state
            .downcast_mut::<State<Link, Renderer::Paragraph>>();
        operation.selectable(None, layout.bounds(), state);
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        defaults: &renderer::Style,
        layout: Layout<'_>,
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

        if self.selectable
            && let Some((a, b)) = state.selection
        {
            let (start, end) = if a <= b { (a, b) } else { (b, a) };
            if start < end && style.selection.a > 0.0 {
                let anchor = layout.bounds().anchor(
                    state.paragraph.min_bounds(),
                    state.paragraph.align_x(),
                    state.paragraph.align_y(),
                );
                let translation = anchor - Point::ORIGIN;
                for bounds in state.paragraph.selection_bounds(start, end) {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: bounds + translation,
                            ..Default::default()
                        },
                        style.selection,
                    );
                }
            }
        }

        for (index, span) in self.spans.as_ref().as_ref().iter().enumerate() {
            let is_hovered_link = self.on_link_click.is_some() && Some(index) == self.hovered_link;

            if span.highlight.is_some() || span.underline || span.strikethrough || is_hovered_link {
                let translation = layout.position() - Point::ORIGIN;
                let regions = state.paragraph.span_bounds(index);

                if let Some(highlight) = span.highlight {
                    for bounds in &regions {
                        let bounds = Rectangle::new(
                            bounds.position() - Vector::new(span.padding.left, span.padding.top),
                            bounds.size() + Size::new(span.padding.x(), span.padding.y()),
                        );

                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: bounds + translation,
                                border: highlight.border,
                                ..Default::default()
                            },
                            highlight.background,
                        );
                    }
                }

                if span.underline || span.strikethrough || is_hovered_link {
                    let size = span.size.or(self.size).unwrap_or(renderer.default_size());

                    let line_height = span
                        .line_height
                        .unwrap_or(self.line_height)
                        .to_absolute(size);

                    let color = span.color.or(style.color).unwrap_or(defaults.text_color);

                    let baseline =
                        translation + Vector::new(0.0, size.0 + (line_height.0 - size.0) / 2.0);

                    if span.underline || is_hovered_link {
                        for bounds in &regions {
                            renderer.fill_quad(
                                renderer::Quad {
                                    bounds: Rectangle::new(
                                        bounds.position() + baseline
                                            - Vector::new(0.0, size.0 * 0.08),
                                        Size::new(bounds.width, 1.0),
                                    ),
                                    ..Default::default()
                                },
                                color,
                            );
                        }
                    }

                    if span.strikethrough {
                        for bounds in &regions {
                            renderer.fill_quad(
                                renderer::Quad {
                                    bounds: Rectangle::new(
                                        bounds.position() + baseline
                                            - Vector::new(0.0, size.0 / 2.0),
                                        Size::new(bounds.width, 1.0),
                                    ),
                                    ..Default::default()
                                },
                                color,
                            );
                        }
                    }
                }
            }
        }

        text::draw(
            renderer,
            defaults,
            layout.bounds(),
            &state.paragraph,
            style,
            viewport,
        );
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        // Bail entirely when neither feature is enabled — keeps the
        // hot path for plain decorative `rich_text` allocation-free.
        if self.on_link_click.is_none() && !self.selectable {
            return;
        }

        let was_hovered = self.hovered_link.is_some();
        let cursor_in_bounds = cursor.position_in(layout.bounds());

        if self.on_link_click.is_some() {
            if let Some(position) = cursor_in_bounds {
                let state = tree
                    .state
                    .downcast_ref::<State<Link, Renderer::Paragraph>>();

                self.hovered_link = state.paragraph.hit_span(position).and_then(|span| {
                    if self.spans.as_ref().as_ref().get(span)?.link.is_some() {
                        Some(span)
                    } else {
                        None
                    }
                });
            } else {
                self.hovered_link = None;
            }

            if was_hovered != self.hovered_link.is_some() {
                shell.request_redraw();
            }
        }

        let externally_managed = tree
            .state
            .downcast_ref::<State<Link, Renderer::Paragraph>>()
            .externally_managed;
        let selectable_self = self.selectable && !externally_managed;

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let state = tree
                    .state
                    .downcast_mut::<State<Link, Renderer::Paragraph>>();

                if self.hovered_link.is_some() {
                    state.span_pressed = self.hovered_link;
                    shell.capture_event();
                } else if selectable_self
                    && let Some(position) = cursor_in_bounds
                    && let Some(hit) = state.paragraph.hit_test(position)
                {
                    let cursor_at = hit.cursor();
                    let click =
                        mouse::Click::new(position, mouse::Button::Left, state.last_click);

                    match click.kind() {
                        mouse::click::Kind::Single => {
                            state.selection = Some((cursor_at, cursor_at));
                            state.selecting = true;
                        }
                        mouse::click::Kind::Double => {
                            let start = state.step_byte_word(cursor_at, -1);
                            let end = state.step_byte_word(cursor_at, 1);
                            state.selection = Some((start, end));
                            state.selecting = false;
                        }
                        mouse::click::Kind::Triple => {
                            let len = state.text_len();
                            let start = state.line_edge_byte(cursor_at, -1).unwrap_or(0);
                            let end = state.line_edge_byte(cursor_at, 1).unwrap_or(len);
                            state.selection = Some((start, end));
                            state.selecting = false;
                        }
                    }

                    state.last_click = Some(click);
                    state.focused = true;
                    shell.capture_event();
                    shell.request_redraw();
                } else if selectable_self && (state.selection.take().is_some() || state.focused) {
                    // Press outside this widget's text drops focus, so
                    // siblings can self-clear on the same event.
                    state.focused = false;
                    state.last_click = None;
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) if selectable_self => {
                let state = tree
                    .state
                    .downcast_mut::<State<Link, Renderer::Paragraph>>();
                if state.selecting
                    && let Some(position) = cursor_in_bounds
                    && let Some(hit) = state.paragraph.hit_test(position)
                {
                    let new_focus = hit.cursor();
                    if let Some((anchor, focus)) = state.selection
                        && focus != new_focus
                    {
                        state.selection = Some((anchor, new_focus));
                        shell.request_redraw();
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let state = tree
                    .state
                    .downcast_mut::<State<Link, Renderer::Paragraph>>();

                match state.span_pressed {
                    Some(span) if Some(span) == self.hovered_link => {
                        if let Some(on_link_clicked) = &self.on_link_click
                            && let Some(link) = self
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

                if selectable_self && state.selecting {
                    state.selecting = false;

                    if let Some((a, b)) = state.selection
                        && a == b
                    {
                        state.selection = None;
                    }
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Character(c),
                modifiers,
                ..
            }) if selectable_self && modifiers.command() && matches!(c.as_str(), "c" | "C") => {
                let state = tree
                    .state
                    .downcast_ref::<State<Link, Renderer::Paragraph>>();
                if state.focused
                    && let Some((a, b)) = state.selection
                {
                    let (start, end) = if a <= b { (a, b) } else { (b, a) };
                    if start < end {
                        let extracted = collect_selection(self.spans.as_ref().as_ref(), start, end);
                        if !extracted.is_empty() {
                            shell.write_clipboard(core::clipboard::Content::Text(extracted));
                            shell.capture_event();
                        }
                    }
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Character(c),
                modifiers,
                ..
            }) if selectable_self && modifiers.command() && matches!(c.as_str(), "a" | "A") => {
                let state = tree
                    .state
                    .downcast_mut::<State<Link, Renderer::Paragraph>>();
                if state.focused {
                    let len = state.text_len();
                    if len > 0 {
                        state.selection = Some((0, len));
                        state.selecting = false;
                        shell.capture_event();
                        shell.request_redraw();
                    }
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(named),
                modifiers,
                ..
            }) if selectable_self => {
                let state = tree
                    .state
                    .downcast_mut::<State<Link, Renderer::Paragraph>>();
                if !state.focused {
                    return;
                }
                if matches!(named, keyboard::key::Named::Escape) {
                    if state.selection.take().is_some() {
                        shell.capture_event();
                        shell.request_redraw();
                    }
                    return;
                }

                let len = state.text_len();
                let (anchor, focus) = state
                    .selection
                    .unwrap_or((focus_default(*named, len), focus_default(*named, len)));

                let new_focus: Option<usize> = match named {
                    keyboard::key::Named::ArrowLeft if modifiers.command() => {
                        Some(state.step_byte_word(focus, -1))
                    }
                    keyboard::key::Named::ArrowRight if modifiers.command() => {
                        Some(state.step_byte_word(focus, 1))
                    }
                    keyboard::key::Named::ArrowLeft => Some(state.step_byte(focus, -1)),
                    keyboard::key::Named::ArrowRight => Some(state.step_byte(focus, 1)),
                    keyboard::key::Named::ArrowUp => state
                        .step_byte_line(focus, -1)
                        .filter(|&b| b != focus)
                        .or(Some(0)),
                    keyboard::key::Named::ArrowDown => state
                        .step_byte_line(focus, 1)
                        .filter(|&b| b != focus)
                        .or(Some(len)),
                    keyboard::key::Named::Home if modifiers.command() => Some(0),
                    keyboard::key::Named::End if modifiers.command() => Some(len),
                    keyboard::key::Named::Home => state.line_edge_byte(focus, -1).or(Some(0)),
                    keyboard::key::Named::End => state.line_edge_byte(focus, 1).or(Some(len)),
                    _ => return,
                };

                if let Some(new_focus) = new_focus
                    && new_focus != focus
                {
                    // With Shift: extend selection (anchor stays).
                    // Without Shift: collapse to a caret at the new
                    // focus, mirroring how `text_input` moves a
                    // non-extending cursor.
                    let new_anchor = if modifiers.shift() { anchor } else { new_focus };
                    state.selection = Some((new_anchor, new_focus));
                    shell.capture_event();
                    shell.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if self.hovered_link.is_some() {
            mouse::Interaction::Pointer
        } else if self.selectable && cursor.is_over(layout.bounds()) {
            mouse::Interaction::Text
        } else {
            mouse::Interaction::None
        }
    }
}

fn layout<Link, Renderer>(
    state: &mut State<Link, Renderer::Paragraph>,
    renderer: &Renderer,
    limits: &layout::Limits,
    width: Length,
    height: Length,
    spans: &[Span<'_, Link, Renderer::Font>],
    line_height: LineHeight,
    size: Option<Pixels>,
    font: Option<Renderer::Font>,
    align_x: Alignment,
    align_y: alignment::Vertical,
    wrapping: Wrapping,
    ellipsis: Ellipsis,
) -> layout::Node
where
    Link: Clone,
    Renderer: core::text::Renderer,
{
    layout::sized(limits, width, height, |limits| {
        let bounds = limits.max();

        let size = size.unwrap_or_else(|| renderer.default_size());
        let font = font.unwrap_or_else(|| renderer.default_font());

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
            hint_factor: renderer.scale_factor(),
        };

        if state.spans != spans {
            state.paragraph = Renderer::Paragraph::with_spans(text_with_spans());
            state.spans = spans.iter().cloned().map(Span::to_static).collect();
            state.text = state.spans.iter().map(|s| s.text.as_ref()).collect();
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
                hint_factor: renderer.scale_factor(),
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

/// Default focus byte when no selection exists yet — keys that go
/// rightward start from `0`, keys that go leftward start from the end.
fn focus_default(named: keyboard::key::Named, len: usize) -> usize {
    use keyboard::key::Named;
    match named {
        Named::ArrowLeft | Named::ArrowUp | Named::Home => len,
        Named::ArrowRight | Named::ArrowDown | Named::End => 0,
        _ => 0,
    }
}

fn collect_selection<Link, Font>(
    spans: &[Span<'_, Link, Font>],
    start: usize,
    end: usize,
) -> String {
    let mut out = String::new();
    let mut cursor = 0usize;
    for span in spans {
        let text = span.text.as_ref();
        let len = text.len();
        let span_end = cursor + len;
        if span_end <= start {
            cursor = span_end;
            continue;
        }
        if cursor >= end {
            break;
        }
        let local_start = floor_char_boundary(text, start.saturating_sub(cursor));
        let local_end = floor_char_boundary(text, (end - cursor).min(len));
        if local_start < local_end {
            out.push_str(&text[local_start..local_end]);
        }
        cursor = span_end;
    }
    out
}

fn floor_char_boundary(s: &str, mut idx: usize) -> usize {
    if idx >= s.len() {
        return s.len();
    }
    while idx > 0 && !s.is_char_boundary(idx) {
        idx -= 1;
    }
    idx
}

impl<'a, Link, Message, Theme, Renderer> FromIterator<Span<'a, Link, Renderer::Font>>
    for Rich<'a, Link, Message, Theme, Renderer>
where
    Link: Clone + 'a,
    Theme: Catalog,
    Renderer: core::text::Renderer,
    Renderer::Font: 'a,
{
    fn from_iter<T: IntoIterator<Item = Span<'a, Link, Renderer::Font>>>(spans: T) -> Self {
        Self::with_spans(spans.into_iter().collect::<Vec<_>>())
    }
}

impl<'a, Link, Message, Theme, Renderer> From<Rich<'a, Link, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Link: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: core::text::Renderer + 'a,
{
    fn from(
        text: Rich<'a, Link, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(text)
    }
}
