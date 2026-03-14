use crate::core::alignment;
use crate::core::keyboard;
use crate::core::layout;
use crate::core::mouse::{self, click};
use crate::core::renderer;
use crate::core::text::{Hit, Paragraph, Span};
use crate::core::widget::text::{
    self, Alignment, Catalog, Ellipsis, LineHeight, Shaping, Style, StyleFn, Wrapping,
};
use crate::core::widget::tree::{self, Tree};
use crate::core::{
    self, Color, Element, Event, Layout, Length, Pixels, Point, Rectangle, Shell, Size, Vector,
    Widget, clipboard,
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
    /// Selection range (start_char, end_char)
    selection: Option<(usize, usize)>,
    /// Selection highlight color
    selection_color: Option<Color>,
    /// Callback when selection changes (legacy, for single-paragraph)
    on_selection_change: Option<Box<dyn Fn(Option<(usize, usize)>) -> Message + 'a>>,
    /// Callback when selection starts (mouse pressed)
    on_selection_start: Option<Box<dyn Fn(usize) -> Message + 'a>>,
    /// Callback when selection extends (mouse moved while selecting)
    on_selection_drag: Option<Box<dyn Fn(usize) -> Message + 'a>>,
    /// Callback when selection ends (mouse released)
    on_selection_end: Option<Box<dyn Fn() -> Message + 'a>>,
    /// Whether selection is enabled
    selectable: bool,
    /// Global selection active (for cross-paragraph coordination)
    /// When true, emit drag events even if this widget didn't start selection
    global_selecting: bool,
    /// For edge handling: (paragraph_index, total_paragraphs)
    paragraph_info: Option<(usize, usize)>,
    /// Letter spacing in pixels (applied to all spans)
    letter_spacing: Option<Pixels>,
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
            selection: None,
            selection_color: None,
            on_selection_change: None,
            on_selection_start: None,
            on_selection_drag: None,
            on_selection_end: None,
            selectable: false,
            global_selecting: false,
            paragraph_info: None,
            letter_spacing: None,
        }
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

    /// Sets the selection range for highlighting.
    ///
    /// The range is specified as (start_char, end_char) indices into the text content.
    pub fn selection(mut self, range: Option<(usize, usize)>) -> Self {
        self.selection = range;
        self
    }

    /// Sets the color for selection highlighting.
    pub fn selection_color(mut self, color: impl Into<Color>) -> Self {
        self.selection_color = Some(color.into());
        self
    }

    /// Enables text selection with mouse interaction (single-paragraph mode).
    ///
    /// When enabled, clicking and dragging will select text, and the callback
    /// will be invoked with the new selection range.
    pub fn on_selection_change(
        mut self,
        callback: impl Fn(Option<(usize, usize)>) -> Message + 'a,
    ) -> Self {
        self.on_selection_change = Some(Box::new(callback));
        self.selectable = true;
        self
    }

    /// Sets callback for when selection starts (mouse pressed).
    /// Used for cross-paragraph selection coordination.
    pub fn on_selection_start(mut self, callback: impl Fn(usize) -> Message + 'a) -> Self {
        self.on_selection_start = Some(Box::new(callback));
        self.selectable = true;
        self
    }

    /// Sets callback for when selection is dragged (mouse moved while selecting).
    /// Used for cross-paragraph selection coordination.
    pub fn on_selection_drag(mut self, callback: impl Fn(usize) -> Message + 'a) -> Self {
        self.on_selection_drag = Some(Box::new(callback));
        self.selectable = true;
        self
    }

    /// Sets callback for when selection ends (mouse released).
    /// Used for cross-paragraph selection coordination.
    pub fn on_selection_end(mut self, callback: impl Fn() -> Message + 'a) -> Self {
        self.on_selection_end = Some(Box::new(callback));
        self
    }

    /// Sets global selecting state for cross-paragraph selection.
    /// When true, this widget will emit drag events even if it didn't start the selection.
    pub fn global_selecting(mut self, selecting: bool) -> Self {
        self.global_selecting = selecting;
        self
    }

    /// Sets paragraph info for edge handling in cross-paragraph selection.
    /// (paragraph_index, total_paragraphs)
    pub fn paragraph_info(mut self, index: usize, total: usize) -> Self {
        self.paragraph_info = Some((index, total));
        self
    }

    /// Sets the letter spacing of the [`Rich`] text in pixels.
    pub fn letter_spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.letter_spacing = Some(spacing.into());
        self
    }

    /// Optionally sets the letter spacing of the [`Rich`] text.
    pub fn letter_spacing_maybe(mut self, spacing: Option<Pixels>) -> Self {
        self.letter_spacing = spacing;
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

        self.style(move |_theme| Style { color })
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

struct State<Link, P: Paragraph> {
    spans: Vec<Span<'static, Link, P::Font>>,
    span_pressed: Option<usize>,
    paragraph: P,
    /// Selection anchor (start point when dragging)
    selection_anchor: Option<usize>,
    /// Whether currently dragging a selection
    is_selecting: bool,
    /// Last click for double/triple click detection
    last_click: Option<click::Click>,
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
            span_pressed: None,
            paragraph: Renderer::Paragraph::default(),
            selection_anchor: None,
            is_selecting: false,
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
            self.letter_spacing,
        )
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

        // Draw selection highlight
        if let Some((start, end)) = self.selection {
            let start = start.min(end);
            let end = start.max(end);

            let selection_color = self
                .selection_color
                .unwrap_or(Color::from_rgba(0.3, 0.5, 0.9, 0.35));

            // Use the same anchor calculation as text::draw to account for alignment
            let anchor = layout.bounds().anchor(
                state.paragraph.min_bounds(),
                state.paragraph.align_x(),
                state.paragraph.align_y(),
            );
            let selection_bounds = state.paragraph.selection_bounds(start, end);

            for bounds in selection_bounds {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle::new(
                            bounds.position() + (anchor - Point::ORIGIN),
                            bounds.size(),
                        ),
                        ..Default::default()
                    },
                    selection_color,
                );
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
        let state = tree
            .state
            .downcast_mut::<State<Link, Renderer::Paragraph>>();
        let bounds = layout.bounds();

        // Handle link hovering
        let was_hovered = self.hovered_link.is_some();
        if let Some(position) = cursor.position_in(bounds) {
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

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                // Handle link press
                if self.hovered_link.is_some() && self.on_link_click.is_some() {
                    state.span_pressed = self.hovered_link;
                    shell.capture_event();
                    return;
                }

                // Handle selection start
                if self.selectable
                    && let Some(position) = cursor.position_in(bounds)
                {
                    let click_obj =
                        click::Click::new(position, mouse::Button::Left, state.last_click);

                    let total_len: usize = self
                        .spans
                        .as_ref()
                        .as_ref()
                        .iter()
                        .map(|s| s.text.len())
                        .sum();

                    if let Some(Hit::CharOffset(offset)) = state.paragraph.hit_test(position) {
                        match click_obj.kind() {
                            click::Kind::Single => {
                                // Single click: start selection at cursor
                                state.selection_anchor = Some(offset);
                                state.is_selecting = true;

                                if let Some(on_start) = &self.on_selection_start {
                                    shell.publish(on_start(offset));
                                }
                                if let Some(on_change) = &self.on_selection_change {
                                    shell.publish(on_change(Some((offset, offset))));
                                }
                            }
                            click::Kind::Double => {
                                // Double click: select word
                                let spans = self.spans.as_ref().as_ref();
                                let (start, end) = find_word_bounds_in_spans(spans, offset);
                                state.selection_anchor = Some(start);
                                state.is_selecting = false; // Word selected, not dragging

                                if let Some(on_start) = &self.on_selection_start {
                                    shell.publish(on_start(start));
                                }
                                if let Some(on_drag) = &self.on_selection_drag {
                                    shell.publish(on_drag(end));
                                }
                                if let Some(on_end) = &self.on_selection_end {
                                    shell.publish(on_end());
                                }
                                if let Some(on_change) = &self.on_selection_change {
                                    shell.publish(on_change(Some((start, end))));
                                }
                            }
                            click::Kind::Triple => {
                                // Triple click: select entire paragraph
                                state.selection_anchor = Some(0);
                                state.is_selecting = false;

                                if let Some(on_start) = &self.on_selection_start {
                                    shell.publish(on_start(0));
                                }
                                if let Some(on_drag) = &self.on_selection_drag {
                                    shell.publish(on_drag(total_len));
                                }
                                if let Some(on_end) = &self.on_selection_end {
                                    shell.publish(on_end());
                                }
                                if let Some(on_change) = &self.on_selection_change {
                                    shell.publish(on_change(Some((0, total_len))));
                                }
                            }
                        }
                        shell.capture_event();
                    }

                    state.last_click = Some(click_obj);
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                // Handle selection drag
                if !self.selectable {
                    return;
                }

                let total_len: usize = self
                    .spans
                    .as_ref()
                    .as_ref()
                    .iter()
                    .map(|s| s.text.len())
                    .sum();

                // Expand detection bounds to include spacing around paragraph
                const VERTICAL_PADDING: f32 = 20.0; // Generous padding for gap detection

                let mouse_in_bounds = cursor.position_in(bounds).is_some();

                // Check if mouse is in expanded vertical range (same X, expanded Y)
                let mouse_in_expanded = if let Some(pos) = cursor.position() {
                    pos.x >= bounds.x
                        && pos.x <= bounds.x + bounds.width
                        && pos.y >= bounds.y - VERTICAL_PADDING
                        && pos.y <= bounds.y + bounds.height + VERTICAL_PADDING
                } else {
                    false
                };

                // Check if this is first or last paragraph (for edge handling)
                let (is_first, is_last) = self
                    .paragraph_info
                    .map(|(idx, total)| (idx == 0, idx + 1 >= total))
                    .unwrap_or((false, false));

                // For global selection, respond if:
                // 1. Mouse is in our expanded bounds (includes spacing), OR
                // 2. We're the first paragraph and mouse is above all content, OR
                // 3. We're the last paragraph and mouse is below all content
                if self.global_selecting {
                    let should_respond = if mouse_in_expanded {
                        true
                    } else if let Some(cursor_pos) = cursor.position() {
                        (is_first && cursor_pos.y < bounds.y - VERTICAL_PADDING)
                            || (is_last
                                && cursor_pos.y > bounds.y + bounds.height + VERTICAL_PADDING)
                    } else {
                        false
                    };

                    if should_respond {
                        let offset = if let Some(position) = cursor.position_in(bounds) {
                            // Mouse in actual bounds - use hit test
                            if let Some(Hit::CharOffset(off)) = state.paragraph.hit_test(position) {
                                off
                            } else {
                                let para_bounds = state.paragraph.min_bounds();
                                if position.x > para_bounds.width {
                                    total_len
                                } else {
                                    0
                                }
                            }
                        } else if let Some(cursor_pos) = cursor.position() {
                            // Mouse in expanded bounds but not actual - clamp to edge
                            if cursor_pos.y < bounds.y {
                                0 // Above this paragraph
                            } else if cursor_pos.y > bounds.y + bounds.height {
                                total_len // Below this paragraph
                            } else if cursor_pos.x < bounds.x {
                                0
                            } else {
                                total_len
                            }
                        } else {
                            0
                        };

                        if let Some(on_drag) = &self.on_selection_drag {
                            shell.publish(on_drag(offset));
                        }
                        shell.request_redraw();
                    }
                }

                // If this widget owns the selection, handle edge cases when mouse leaves
                // BUT only if global_selecting is false OR mouse is still in our bounds
                // (otherwise another paragraph will handle it)
                if state.is_selecting {
                    // If global selection is active and mouse is NOT in our bounds,
                    // let the other paragraphs handle it - don't emit from here
                    if self.global_selecting && !mouse_in_bounds {
                        // Only emit if we're first/last paragraph and mouse is outside ALL paragraphs
                        let cursor_pos = cursor.position();
                        let should_handle_edge = cursor_pos
                            .map(|pos| {
                                (is_first && pos.y < bounds.y)
                                    || (is_last && pos.y > bounds.y + bounds.height)
                            })
                            .unwrap_or(false);

                        if !should_handle_edge {
                            return; // Let another paragraph handle it
                        }
                    }

                    let offset = if let Some(position) = cursor.position_in(bounds) {
                        if let Some(Hit::CharOffset(off)) = state.paragraph.hit_test(position) {
                            Some(off)
                        } else {
                            let para_bounds = state.paragraph.min_bounds();
                            Some(if position.x > para_bounds.width {
                                total_len
                            } else {
                                0
                            })
                        }
                    } else {
                        cursor.position().map(|cursor_pos| {
                            if cursor_pos.y < bounds.y || cursor_pos.x < bounds.x {
                                0
                            } else {
                                total_len
                            }
                        })
                    };

                    if let Some(offset) = offset {
                        if let Some(on_drag) = &self.on_selection_drag {
                            shell.publish(on_drag(offset));
                        }
                        if let Some(anchor) = state.selection_anchor
                            && let Some(on_change) = &self.on_selection_change
                        {
                            shell.publish(on_change(Some((anchor, offset))));
                        }
                        shell.request_redraw();
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                // Handle link release
                if let Some(span) = state.span_pressed.take()
                    && Some(span) == self.hovered_link
                    && let Some(on_link_clicked) = &self.on_link_click
                    && let Some(link) = self
                        .spans
                        .as_ref()
                        .as_ref()
                        .get(span)
                        .and_then(|span| span.link.clone())
                {
                    shell.publish(on_link_clicked(link));
                }

                // End selection drag - handle both local and global selection
                if self.selectable {
                    if state.is_selecting {
                        state.is_selecting = false;
                        if let Some(on_end) = &self.on_selection_end {
                            shell.publish(on_end());
                        }
                    } else if self.global_selecting {
                        // Global selection ended in this widget
                        if let Some(on_end) = &self.on_selection_end {
                            shell.publish(on_end());
                        }
                    }
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Character(c),
                modifiers,
                ..
            }) if self.selectable && modifiers.command() && c.as_str() == "c" => {
                // Copy selection to clipboard
                if let Some((start, end)) = self.selection {
                    let text = extract_text_from_spans(
                        self.spans.as_ref().as_ref(),
                        start.min(end),
                        start.max(end),
                    );
                    if !text.is_empty() {
                        shell.write_clipboard(clipboard::Content::Text(text));
                    }
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
    letter_spacing: Option<Pixels>,
) -> layout::Node
where
    Link: Clone,
    Renderer: core::text::Renderer,
{
    layout::sized(limits, width, height, |limits| {
        let bounds = limits.max();

        let size = size.unwrap_or_else(|| renderer.default_size());
        let font = font.unwrap_or_else(|| renderer.default_font());

        // Use Basic shaping for pure ASCII content, Advanced for complex text
        let shaping = if spans.iter().all(|span| span.text.is_ascii()) {
            Shaping::Basic
        } else {
            Shaping::Advanced
        };

        let text_with_spans = || core::Text {
            content: spans,
            bounds,
            size,
            line_height,
            font,
            align_x,
            align_y,
            shaping,
            wrapping,
            ellipsis,
            hint_factor: renderer.scale_factor(),
            letter_spacing: letter_spacing.map(|p| p.0),
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
                shaping,
                wrapping,
                ellipsis,
                hint_factor: renderer.scale_factor(),
                letter_spacing: letter_spacing.map(|p| p.0),
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

/// Extract text content from spans within a given byte range.
fn extract_text_from_spans<Link, Font>(
    spans: &[Span<'_, Link, Font>],
    start: usize,
    end: usize,
) -> String {
    let mut result = String::new();
    let mut offset = 0usize;

    for span in spans {
        let span_len = span.text.len();
        let span_end = offset + span_len;

        if span_end <= start {
            offset = span_end;
            continue;
        }
        if offset >= end {
            break;
        }

        let local_start = start.saturating_sub(offset);
        let local_end = (end - offset).min(span_len);

        if local_start < span_len && local_start < local_end {
            result.push_str(&span.text[local_start..local_end]);
        }

        offset = span_end;
    }

    result
}

/// Find word boundaries around a given character offset in spans.
/// Returns (start, end) byte offsets of the word.
fn find_word_bounds_in_spans<Link, Font>(
    spans: &[Span<'_, Link, Font>],
    offset: usize,
) -> (usize, usize) {
    // Build combined text and find boundaries
    let text: String = spans.iter().map(|s| s.text.as_ref()).collect();

    if text.is_empty() {
        return (0, 0);
    }

    let offset = offset.min(text.len());
    let bytes = text.as_bytes();

    // Find start of word (scan backwards)
    let mut start = offset;
    while start > 0 {
        let prev = start - 1;
        if !is_word_char(bytes[prev]) {
            break;
        }
        start = prev;
    }

    // Find end of word (scan forwards)
    let mut end = offset;
    while end < bytes.len() {
        if !is_word_char(bytes[end]) {
            break;
        }
        end += 1;
    }

    // If we're on whitespace/punctuation, select just that character
    if start == end && offset < text.len() {
        end = offset + 1;
    }

    (start, end)
}

/// Check if a byte is part of a "word" (alphanumeric or underscore).
fn is_word_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}
