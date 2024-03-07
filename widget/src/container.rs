//! Decorate content and apply alignment.
use crate::core::alignment::{self, Alignment};
use crate::core::event::{self, Event};
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget::tree::{self, Tree};
use crate::core::widget::{self, Operation};
use crate::core::{
    Background, Border, Clipboard, Color, Element, Layout, Length, Padding,
    Pixels, Point, Rectangle, Shadow, Shell, Size, Theme, Vector, Widget,
};
use crate::runtime::Command;

/// An element decorating some content.
///
/// It is normally used for alignment purposes.
#[allow(missing_debug_implementations)]
pub struct Container<
    'a,
    Message,
    Theme = crate::Theme,
    Renderer = crate::Renderer,
> where
    Renderer: crate::core::Renderer,
{
    id: Option<Id>,
    padding: Padding,
    width: Length,
    height: Length,
    max_width: f32,
    max_height: f32,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
    clip: bool,
    content: Element<'a, Message, Theme, Renderer>,
    style: Style<Theme>,
}

impl<'a, Message, Theme, Renderer> Container<'a, Message, Theme, Renderer>
where
    Renderer: crate::core::Renderer,
{
    /// Creates a [`Container`] with the given content.
    pub fn new(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self
    where
        Theme: DefaultStyle,
    {
        Self::with_style(content, Theme::default_style())
    }

    /// Creates a [`Container`] with the given content and style.
    pub fn with_style(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
        style: fn(&Theme, Status) -> Appearance,
    ) -> Self {
        let content = content.into();
        let size = content.as_widget().size_hint();

        Container {
            id: None,
            padding: Padding::ZERO,
            width: size.width.fluid(),
            height: size.height.fluid(),
            max_width: f32::INFINITY,
            max_height: f32::INFINITY,
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Top,
            clip: false,
            content,
            style,
        }
    }

    /// Sets the [`Id`] of the [`Container`].
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the [`Padding`] of the [`Container`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the width of the [`Container`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Container`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the maximum width of the [`Container`].
    pub fn max_width(mut self, max_width: impl Into<Pixels>) -> Self {
        self.max_width = max_width.into().0;
        self
    }

    /// Sets the maximum height of the [`Container`].
    pub fn max_height(mut self, max_height: impl Into<Pixels>) -> Self {
        self.max_height = max_height.into().0;
        self
    }

    /// Sets the content alignment for the horizontal axis of the [`Container`].
    pub fn align_x(mut self, alignment: alignment::Horizontal) -> Self {
        self.horizontal_alignment = alignment;
        self
    }

    /// Sets the content alignment for the vertical axis of the [`Container`].
    pub fn align_y(mut self, alignment: alignment::Vertical) -> Self {
        self.vertical_alignment = alignment;
        self
    }

    /// Centers the contents in the horizontal axis of the [`Container`].
    pub fn center_x(mut self) -> Self {
        self.horizontal_alignment = alignment::Horizontal::Center;
        self
    }

    /// Centers the contents in the vertical axis of the [`Container`].
    pub fn center_y(mut self) -> Self {
        self.vertical_alignment = alignment::Vertical::Center;
        self
    }

    /// Sets the style of the [`Container`].
    pub fn style(mut self, style: fn(&Theme, Status) -> Appearance) -> Self {
        self.style = style;
        self
    }

    /// Sets whether the contents of the [`Container`] should be clipped on
    /// overflow.
    pub fn clip(mut self, clip: bool) -> Self {
        self.clip = clip;
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Container<'a, Message, Theme, Renderer>
where
    Renderer: crate::core::Renderer,
{
    fn tag(&self) -> tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> tree::State {
        self.content.as_widget().state()
    }

    fn children(&self) -> Vec<Tree> {
        self.content.as_widget().children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.content.as_widget().diff(tree);
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
            limits,
            self.width,
            self.height,
            self.max_width,
            self.max_height,
            self.padding,
            self.horizontal_alignment,
            self.vertical_alignment,
            |limits| self.content.as_widget().layout(tree, renderer, limits),
        )
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<Message>,
    ) {
        operation.container(
            self.id.as_ref().map(|id| &id.0),
            layout.bounds(),
            &mut |operation| {
                self.content.as_widget().operate(
                    tree,
                    layout.children().next().unwrap(),
                    renderer,
                    operation,
                );
            },
        );
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        self.content.as_widget_mut().on_event(
            tree,
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        )
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            tree,
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        let status = if cursor.is_over(bounds) {
            Status::Hovered
        } else {
            Status::Idle
        };

        let style = (self.style)(theme, status);

        if let Some(clipped_viewport) = bounds.intersection(viewport) {
            draw_background(renderer, &style, bounds);

            self.content.as_widget().draw(
                tree,
                renderer,
                theme,
                &renderer::Style {
                    text_color: style
                        .text_color
                        .unwrap_or(renderer_style.text_color),
                },
                layout.children().next().unwrap(),
                cursor,
                if self.clip {
                    &clipped_viewport
                } else {
                    viewport
                },
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            tree,
            layout.children().next().unwrap(),
            renderer,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<Container<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: 'a + crate::core::Renderer,
{
    fn from(
        column: Container<'a, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(column)
    }
}

/// Computes the layout of a [`Container`].
pub fn layout(
    limits: &layout::Limits,
    width: Length,
    height: Length,
    max_width: f32,
    max_height: f32,
    padding: Padding,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
    layout_content: impl FnOnce(&layout::Limits) -> layout::Node,
) -> layout::Node {
    layout::positioned(
        &limits.max_width(max_width).max_height(max_height),
        width,
        height,
        padding,
        |limits| layout_content(&limits.loose()),
        |content, size| {
            content.align(
                Alignment::from(horizontal_alignment),
                Alignment::from(vertical_alignment),
                size,
            )
        },
    )
}

/// Draws the background of a [`Container`] given its [`Appearance`] and its `bounds`.
pub fn draw_background<Renderer>(
    renderer: &mut Renderer,
    appearance: &Appearance,
    bounds: Rectangle,
) where
    Renderer: crate::core::Renderer,
{
    if appearance.background.is_some()
        || appearance.border.width > 0.0
        || appearance.shadow.color.a > 0.0
    {
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: appearance.border,
                shadow: appearance.shadow,
            },
            appearance
                .background
                .unwrap_or(Background::Color(Color::TRANSPARENT)),
        );
    }
}

/// The identifier of a [`Container`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id(widget::Id);

impl Id {
    /// Creates a custom [`Id`].
    pub fn new(id: impl Into<std::borrow::Cow<'static, str>>) -> Self {
        Self(widget::Id::new(id))
    }

    /// Creates a unique [`Id`].
    ///
    /// This function produces a different [`Id`] every time it is called.
    pub fn unique() -> Self {
        Self(widget::Id::unique())
    }
}

impl From<Id> for widget::Id {
    fn from(id: Id) -> Self {
        id.0
    }
}

/// Produces a [`Command`] that queries the visible screen bounds of the
/// [`Container`] with the given [`Id`].
pub fn visible_bounds(id: Id) -> Command<Option<Rectangle>> {
    struct VisibleBounds {
        target: widget::Id,
        depth: usize,
        scrollables: Vec<(Vector, Rectangle, usize)>,
        bounds: Option<Rectangle>,
    }

    impl Operation<Option<Rectangle>> for VisibleBounds {
        fn scrollable(
            &mut self,
            _state: &mut dyn widget::operation::Scrollable,
            _id: Option<&widget::Id>,
            bounds: Rectangle,
            translation: Vector,
        ) {
            match self.scrollables.last() {
                Some((last_translation, last_viewport, _depth)) => {
                    let viewport = last_viewport
                        .intersection(&(bounds - *last_translation))
                        .unwrap_or(Rectangle::new(Point::ORIGIN, Size::ZERO));

                    self.scrollables.push((
                        translation + *last_translation,
                        viewport,
                        self.depth,
                    ));
                }
                None => {
                    self.scrollables.push((translation, bounds, self.depth));
                }
            }
        }

        fn container(
            &mut self,
            id: Option<&widget::Id>,
            bounds: Rectangle,
            operate_on_children: &mut dyn FnMut(
                &mut dyn Operation<Option<Rectangle>>,
            ),
        ) {
            if self.bounds.is_some() {
                return;
            }

            if id == Some(&self.target) {
                match self.scrollables.last() {
                    Some((translation, viewport, _)) => {
                        self.bounds =
                            viewport.intersection(&(bounds - *translation));
                    }
                    None => {
                        self.bounds = Some(bounds);
                    }
                }

                return;
            }

            self.depth += 1;

            operate_on_children(self);

            self.depth -= 1;

            match self.scrollables.last() {
                Some((_, _, depth)) if self.depth == *depth => {
                    let _ = self.scrollables.pop();
                }
                _ => {}
            }
        }

        fn finish(&self) -> widget::operation::Outcome<Option<Rectangle>> {
            widget::operation::Outcome::Some(self.bounds)
        }
    }

    Command::widget(VisibleBounds {
        target: id.into(),
        depth: 0,
        scrollables: Vec::new(),
        bounds: None,
    })
}

/// The appearance of a container.
#[derive(Debug, Clone, Copy, Default)]
pub struct Appearance {
    /// The text [`Color`] of the container.
    pub text_color: Option<Color>,
    /// The [`Background`] of the container.
    pub background: Option<Background>,
    /// The [`Border`] of the container.
    pub border: Border,
    /// The [`Shadow`] of the container.
    pub shadow: Shadow,
}

impl Appearance {
    /// Derives a new [`Appearance`] with a border of the given [`Color`] and
    /// `width`.
    pub fn with_border(
        self,
        color: impl Into<Color>,
        width: impl Into<Pixels>,
    ) -> Self {
        Self {
            border: Border {
                color: color.into(),
                width: width.into().0,
                ..Border::default()
            },
            ..self
        }
    }

    /// Derives a new [`Appearance`] with the given [`Background`].
    pub fn with_background(self, background: impl Into<Background>) -> Self {
        Self {
            background: Some(background.into()),
            ..self
        }
    }
}

/// The possible status of a [`Container`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The [`Container`] is idle.
    Idle,
    /// The [`Container`] is being hovered.
    Hovered,
}

/// The style of a [`Container`].
pub type Style<Theme> = fn(&Theme, Status) -> Appearance;

/// The default style of a [`Container`].
pub trait DefaultStyle {
    /// Returns the default style of a [`Container`].
    fn default_style() -> Style<Self>;
}

impl DefaultStyle for Theme {
    fn default_style() -> Style<Self> {
        transparent
    }
}

impl DefaultStyle for Appearance {
    fn default_style() -> Style<Self> {
        |appearance, _status| *appearance
    }
}

/// A transparent [`Container`].
pub fn transparent<Theme>(_theme: &Theme, _status: Status) -> Appearance {
    Appearance::default()
}

/// A rounded [`Container`] with a background.
pub fn box_(theme: &Theme, _status: Status) -> Appearance {
    let palette = theme.extended_palette();

    Appearance {
        background: Some(palette.background.weak.color.into()),
        border: Border::with_radius(2),
        ..Appearance::default()
    }
}

/// A bordered [`Container`] with a background.
pub fn bordered_box(theme: &Theme, _status: Status) -> Appearance {
    let palette = theme.extended_palette();

    Appearance {
        background: Some(palette.background.weak.color.into()),
        border: Border {
            width: 1.0,
            radius: 0.0.into(),
            color: palette.background.strong.color,
        },
        ..Appearance::default()
    }
}
