use crate::core::alignment;
use crate::core::event;
use crate::core::layout;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::text::{Paragraph, Span};
use crate::core::widget::text::{
    self, Catalog, LineHeight, Shaping, Style, StyleFn,
};
use crate::core::widget::tree::{self, Tree};
use crate::core::{
    self, Clipboard, Color, Element, Event, Layout, Length, Pixels, Rectangle,
    Shell, Size, Widget,
};

use std::borrow::Cow;

/// A bunch of [`Rich`] text.
#[allow(missing_debug_implementations)]
pub struct Rich<
    'a,
    Message,
    Link = (),
    Theme = crate::Theme,
    Renderer = crate::Renderer,
> where
    Link: Clone + 'static,
    Theme: Catalog,
    Renderer: core::text::Renderer,
{
    spans: Cow<'a, [Span<'a, Link, Renderer::Font>]>,
    size: Option<Pixels>,
    line_height: LineHeight,
    width: Length,
    height: Length,
    font: Option<Renderer::Font>,
    align_x: alignment::Horizontal,
    align_y: alignment::Vertical,
    class: Theme::Class<'a>,
    on_link: Option<Box<dyn Fn(Link) -> Message + 'a>>,
}

impl<'a, Message, Link, Theme, Renderer>
    Rich<'a, Message, Link, Theme, Renderer>
where
    Link: Clone + 'static,
    Theme: Catalog,
    Renderer: core::text::Renderer,
{
    /// Creates a new empty [`Rich`] text.
    pub fn new() -> Self {
        Self {
            spans: Cow::default(),
            size: None,
            line_height: LineHeight::default(),
            width: Length::Shrink,
            height: Length::Shrink,
            font: None,
            align_x: alignment::Horizontal::Left,
            align_y: alignment::Vertical::Top,
            class: Theme::default(),
            on_link: None,
        }
    }

    /// Creates a new [`Rich`] text with the given text spans.
    pub fn with_spans(
        spans: impl Into<Cow<'a, [Span<'a, Link, Renderer::Font>]>>,
    ) -> Self {
        Self {
            spans: spans.into(),
            ..Self::new()
        }
    }

    /// Sets the default size of the [`Rich`] text.
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = Some(size.into());
        self
    }

    /// Sets the defualt [`LineHeight`] of the [`Rich`] text.
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
    pub fn align_x(
        mut self,
        alignment: impl Into<alignment::Horizontal>,
    ) -> Self {
        self.align_x = alignment.into();
        self
    }

    /// Sets the [`alignment::Vertical`] of the [`Rich`] text.
    pub fn align_y(
        mut self,
        alignment: impl Into<alignment::Vertical>,
    ) -> Self {
        self.align_y = alignment.into();
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

    /// Sets the message handler for link clicks on the [`Rich`] text.
    pub fn on_link(mut self, on_link: impl Fn(Link) -> Message + 'a) -> Self {
        self.on_link = Some(Box::new(on_link));
        self
    }

    /// Sets the default style class of the [`Rich`] text.
    #[cfg(feature = "advanced")]
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    /// Adds a new text [`Span`] to the [`Rich`] text.
    pub fn push(
        mut self,
        span: impl Into<Span<'a, Link, Renderer::Font>>,
    ) -> Self {
        self.spans.to_mut().push(span.into());
        self
    }
}

impl<'a, Message, Link, Theme, Renderer> Default
    for Rich<'a, Message, Link, Theme, Renderer>
where
    Link: Clone + 'static,
    Theme: Catalog,
    Renderer: core::text::Renderer,
{
    fn default() -> Self {
        Self::new()
    }
}

struct State<Link, P: Paragraph> {
    spans: Vec<Span<'static, Link, P::Font>>,
    span_pressed: Option<usize>,
    paragraph: P,
}

impl<'a, Message, Link, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Rich<'a, Message, Link, Theme, Renderer>
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

    fn layout(
        &self,
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
            self.spans.as_ref(),
            self.line_height,
            self.size,
            self.font,
            self.align_x,
            self.align_y,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        defaults: &renderer::Style,
        layout: Layout<'_>,
        _cursor_position: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree
            .state
            .downcast_ref::<State<Link, Renderer::Paragraph>>();

        let style = theme.style(&self.class);

        text::draw(
            renderer,
            defaults,
            layout,
            &state.paragraph,
            style,
            viewport,
        );
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        let Some(on_link_click) = self.on_link.as_ref() else {
            return event::Status::Ignored;
        };

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(position) = cursor.position_in(layout.bounds()) {
                    let state = tree
                        .state
                        .downcast_mut::<State<Link, Renderer::Paragraph>>();

                    if let Some(span) = state.paragraph.hit_span(position) {
                        state.span_pressed = Some(span);

                        return event::Status::Captured;
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let state = tree
                    .state
                    .downcast_mut::<State<Link, Renderer::Paragraph>>();

                if let Some(span_pressed) = state.span_pressed {
                    state.span_pressed = None;

                    if let Some(position) = cursor.position_in(layout.bounds())
                    {
                        match state.paragraph.hit_span(position) {
                            Some(span) if span == span_pressed => {
                                if let Some(link) = state
                                    .spans
                                    .get(span)
                                    .and_then(|span| span.link.clone())
                                {
                                    shell.publish(on_link_click(link));
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }

        event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if self.on_link.is_none() {
            return mouse::Interaction::None;
        }

        if let Some(position) = cursor.position_in(layout.bounds()) {
            let state = tree
                .state
                .downcast_ref::<State<Link, Renderer::Paragraph>>();

            if let Some(span) = state
                .paragraph
                .hit_span(position)
                .and_then(|span| state.spans.get(span))
            {
                if span.link.is_some() {
                    return mouse::Interaction::Pointer;
                }
            }
        }

        mouse::Interaction::None
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
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
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
            horizontal_alignment,
            vertical_alignment,
            shaping: Shaping::Advanced,
        };

        if state.spans != spans {
            state.paragraph =
                Renderer::Paragraph::with_spans(text_with_spans());
            state.spans = spans.iter().cloned().map(Span::to_static).collect();
        } else {
            match state.paragraph.compare(core::Text {
                content: (),
                bounds,
                size,
                line_height,
                font,
                horizontal_alignment,
                vertical_alignment,
                shaping: Shaping::Advanced,
            }) {
                core::text::Difference::None => {}
                core::text::Difference::Bounds => {
                    state.paragraph.resize(bounds);
                }
                core::text::Difference::Shape => {
                    state.paragraph =
                        Renderer::Paragraph::with_spans(text_with_spans());
                }
            }
        }

        state.paragraph.min_bounds()
    })
}

impl<'a, Message, Link, Theme, Renderer>
    FromIterator<Span<'a, Link, Renderer::Font>>
    for Rich<'a, Message, Link, Theme, Renderer>
where
    Link: Clone + 'static,
    Theme: Catalog,
    Renderer: core::text::Renderer,
{
    fn from_iter<T: IntoIterator<Item = Span<'a, Link, Renderer::Font>>>(
        spans: T,
    ) -> Self {
        Self {
            spans: spans.into_iter().collect(),
            ..Self::new()
        }
    }
}

impl<'a, Message, Link, Theme, Renderer>
    From<Rich<'a, Message, Link, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Link: Clone + 'static,
    Theme: Catalog + 'a,
    Renderer: core::text::Renderer + 'a,
{
    fn from(
        text: Rich<'a, Message, Link, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(text)
    }
}
