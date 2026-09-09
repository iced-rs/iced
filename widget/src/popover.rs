//! A popover is a floating piece of content that appears over some element
//! when the element is **clicked**.
//!
//! Unlike a [`tooltip`], a popover is not shown on hover. It only appears when
//! the base element is clicked, and it disappears as soon as the user clicks
//! anywhere outside of the popover's bounds. Clicking the base again while it
//! is open will also dismiss it.
//!
//! The base of the popover is drawn and behaves like a [`button`]: it has a
//! background, border, shadow, and text color, and it reflects its current
//! [`Status`] (including [`Status::Opened`] while the popover is shown).
//!
//! [`tooltip`]: crate::tooltip::Tooltip
//! [`button`]: crate::button::Button
//!
//! # Example
//! ```no_run
//! # mod iced { pub mod widget { pub use iced_widget::*; } }
//! # pub type State = ();
//! # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
//! use iced::widget::{container, popover};
//!
//! enum Message {
//!     // ...
//! }
//!
//! fn view(_state: &State) -> Element<'_, Message> {
//!     popover(
//!         "Click me to display the popover!",
//!         container("This is the popover contents!")
//!             .padding(10)
//!             .style(container::rounded_box),
//!         popover::Position::Bottom,
//!     )
//!     .into()
//! }
//! ```
use crate::core::border::{self, Border};
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::text;
use crate::core::theme::palette;
use crate::core::touch;
use crate::core::widget::{self, Widget};
use crate::core::window;
use crate::core::{
    Background, Color, Element, Event, Length, Padding, Pixels, Point, Rectangle, Shadow, Shell,
    Size, Theme, Vector,
};

/// A floating piece of content that appears over another element when it is
/// clicked.
///
/// The popover only appears when the base element is clicked, and it
/// disappears when the user clicks outside of the popover's bounds.
///
/// The base of the popover is rendered and behaves like a [`Button`]: it is
/// styled with a [`Style`] that reacts to its [`Status`] (including
/// [`Status::Opened`] while the popover is shown).
///
/// [`Button`]: crate::button::Button
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } }
/// # pub type State = ();
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// use iced::widget::{container, popover};
///
/// enum Message {
///     // ...
/// }
///
/// fn view(_state: &State) -> Element<'_, Message> {
///     popover(
///         "Click me to display the popover!",
///         container("This is the popover contents!")
///             .padding(10)
///             .style(container::rounded_box),
///         popover::Position::Bottom,
///     )
///     .into()
/// }
/// ```
pub struct Popover<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    content: Element<'a, Message, Theme, Renderer>,
    popover: Element<'a, Message, Theme, Renderer>,
    position: Position,
    gap: f32,
    padding: f32,
    snap_within_viewport: bool,
    class: <Theme as Catalog>::Class<'a>,
    status: Option<Status>,
}

impl<'a, Message, Theme, Renderer> Popover<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    /// The default padding of a [`Popover`] drawn by this renderer.
    const DEFAULT_PADDING: f32 = 5.0;

    /// Creates a new [`Popover`].
    ///
    /// The `content` is the base element that, when clicked, will show the
    /// `popover`. The base is rendered like a [`Button`].
    ///
    /// [`Button`]: crate::button::Button
    /// [`Popover`]: struct.Popover.html
    pub fn new(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
        popover: impl Into<Element<'a, Message, Theme, Renderer>>,
        position: Position,
    ) -> Self {
        Popover {
            content: content.into(),
            popover: popover.into(),
            position,
            gap: 0.0,
            padding: Self::DEFAULT_PADDING,
            snap_within_viewport: true,
            class: <Theme as Catalog>::default(),
            status: None,
        }
    }

    /// Sets the gap between the content and its [`Popover`].
    pub fn gap(mut self, gap: impl Into<Pixels>) -> Self {
        self.gap = gap.into().0;
        self
    }

    /// Sets the padding of the [`Popover`].
    pub fn padding(mut self, padding: impl Into<Pixels>) -> Self {
        self.padding = padding.into().0;
        self
    }

    /// Sets whether the [`Popover`] is snapped within the viewport.
    pub fn snap_within_viewport(mut self, snap: bool) -> Self {
        self.snap_within_viewport = snap;
        self
    }

    /// Sets the style of the base of the [`Popover`].
    ///
    /// The base is rendered like a [`Button`], so the style function receives
    /// the current [`Status`] of the base (including [`Status::Opened`] while
    /// the popover is shown).
    ///
    /// [`Button`]: crate::button::Button
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        <Theme as Catalog>::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the base of the [`Popover`].
    #[cfg(feature = "advanced")]
    #[must_use]
    pub fn class(mut self, class: impl Into<<Theme as Catalog>::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Popover<'_, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    fn diff(&mut self, tree: &mut widget::Tree) {
        tree.diff_children(&mut [self.content.as_widget_mut(), self.popover.as_widget_mut()]);
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State::default())
    }

    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<State>()
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        // The base is laid out like a [`Button`]: its content is wrapped in
        // the default button padding.
        //
        // [`Button`]: crate::button::Button
        layout::padded(
            limits,
            Length::Fit,
            Length::Fit,
            crate::button::DEFAULT_PADDING,
            |limits| {
                self.content
                    .as_widget_mut()
                    .layout(&mut tree.children[0], renderer, limits)
            },
        )
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();

        // The base behaves like a [`Button`]: it tracks whether it is pressed
        // and toggles the popover when it is released (over the base), just as
        // a button only fires its message on release.
        //
        // [`Button`]: crate::button::Button
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. })
                if cursor.is_over(bounds) =>
            {
                state.is_pressed = true;

                shell.capture_event();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. }) => {
                if state.is_pressed {
                    state.is_pressed = false;

                    if cursor.is_over(bounds) {
                        // Release over the base: toggle the popover. Clicking
                        // it opens it, and clicking it again (while it is
                        // open) dismisses it.
                        state.is_open = !state.is_open;

                        shell.invalidate_layout();
                        shell.request_redraw();
                    }

                    shell.capture_event();
                }
            }
            Event::Touch(touch::Event::FingerLost { .. }) => {
                state.is_pressed = false;
            }
            _ => {}
        }

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            shell,
            viewport,
        );

        // Compute the current status of the base and request a redraw when it
        // changes.
        let current_status = if state.is_open {
            Status::Opened
        } else if cursor.is_over(bounds) {
            if state.is_pressed {
                Status::Pressed
            } else {
                Status::Hovered
            }
        } else {
            Status::Active
        };

        if self.status != Some(current_status) {
            self.status = Some(current_status);

            if !matches!(event, Event::Window(window::Event::RedrawRequested(_))) {
                shell.request_redraw();
            }
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let content_layout = layout.children().next().unwrap();
        let status = self.status.unwrap_or(Status::Active);
        let style = <Theme as Catalog>::style(theme, &self.class, status);

        if style.background.is_some() || style.border.width > 0.0 || style.shadow.color.a > 0.0 {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: style.border,
                    shadow: style.shadow,
                    snap: style.snap,
                },
                style
                    .background
                    .unwrap_or(Background::Color(Color::TRANSPARENT)),
            );
        }

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            &renderer::Style {
                text_color: style.text_color,
            },
            content_layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let is_open = tree.state.downcast_ref::<State>().is_open;

        let (base, rest) = tree.children.split_at_mut(1);

        let content = self.content.as_widget_mut().overlay(
            &mut base[0],
            layout,
            renderer,
            viewport,
            translation,
        );

        let popover = if is_open {
            Some(overlay::Element::new(Box::new(Overlay {
                position: layout.position() + translation,
                popover: &mut self.popover,
                popover_tree: &mut rest[0],
                state: &mut tree.state,
                content_bounds: layout.bounds(),
                snap_within_viewport: self.snap_within_viewport,
                positioning: self.position,
                gap: self.gap,
                padding: self.padding,
                viewport: *viewport,
            })))
        } else {
            None
        };

        let mut children = Vec::new();
        if let Some(content) = content {
            children.push(content);
        }
        if let Some(popover) = popover {
            children.push(popover);
        }

        (!children.is_empty()).then(|| overlay::Group::with_children(children).overlay())
    }
}

impl<'a, Message, Theme, Renderer> From<Popover<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: text::Renderer + 'a,
{
    fn from(
        popover: Popover<'a, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(popover)
    }
}

/// Returns whether the given [`Event`] is a press that should open or dismiss
/// the popover.
fn is_press(event: &Event) -> bool {
    matches!(
        event,
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. })
    )
}

/// The position of the popover.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Position {
    /// The popover will appear on the top of the widget.
    Top,
    /// The popover will appear on the bottom of the widget.
    #[default]
    Bottom,
    /// The popover will appear on the left of the widget.
    Left,
    /// The popover will appear on the right of the widget.
    Right,
}

/// The internal state of a [`Popover`].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct State {
    /// Whether the popover is currently shown.
    is_open: bool,
    /// Whether the base (button) is currently pressed.
    is_pressed: bool,
}

struct Overlay<'a, 'b, Message, Theme, Renderer>
where
    Renderer: text::Renderer,
{
    position: Point,
    popover: &'b mut Element<'a, Message, Theme, Renderer>,
    popover_tree: &'b mut widget::Tree,
    state: &'b mut widget::tree::State,
    content_bounds: Rectangle,
    snap_within_viewport: bool,
    positioning: Position,
    gap: f32,
    padding: f32,
    viewport: Rectangle,
}

impl<Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for Overlay<'_, '_, Message, Theme, Renderer>
where
    Renderer: text::Renderer,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let viewport = Rectangle::with_size(bounds);

        let popover_layout = self.popover.as_widget_mut().layout(
            self.popover_tree,
            renderer,
            &layout::Limits::new(
                Size::ZERO,
                if self.snap_within_viewport {
                    viewport.size()
                } else {
                    Size::INFINITE
                },
            )
            .shrink(Padding::new(self.padding)),
        );

        let popover_bounds = popover_layout.bounds();
        let x_center = self.position.x + (self.content_bounds.width - popover_bounds.width) / 2.0;
        let y_center = self.position.y + (self.content_bounds.height - popover_bounds.height) / 2.0;

        let mut popover_rect = {
            let offset = match self.positioning {
                Position::Top => Vector::new(
                    x_center,
                    self.position.y - popover_bounds.height - self.gap - self.padding,
                ),
                Position::Bottom => Vector::new(
                    x_center,
                    self.position.y + self.content_bounds.height + self.gap + self.padding,
                ),
                Position::Left => Vector::new(
                    self.position.x - popover_bounds.width - self.gap - self.padding,
                    y_center,
                ),
                Position::Right => Vector::new(
                    self.position.x + self.content_bounds.width + self.gap + self.padding,
                    y_center,
                ),
            };

            Rectangle {
                x: offset.x - self.padding,
                y: offset.y - self.padding,
                width: popover_bounds.width + self.padding * 2.0,
                height: popover_bounds.height + self.padding * 2.0,
            }
        };

        if self.snap_within_viewport {
            if popover_rect.x < viewport.x {
                popover_rect.x = viewport.x;
            } else if viewport.x + viewport.width < popover_rect.x + popover_rect.width {
                popover_rect.x = viewport.x + viewport.width - popover_rect.width;
            }

            if popover_rect.y < viewport.y {
                popover_rect.y = viewport.y;
            } else if viewport.y + viewport.height < popover_rect.y + popover_rect.height {
                popover_rect.y = viewport.y + viewport.height - popover_rect.height;
            }
        }

        layout::Node::with_children(
            popover_rect.size(),
            vec![popover_layout.translate(Vector::new(self.padding, self.padding))],
        )
        .translate(Vector::new(popover_rect.x, popover_rect.y))
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
    ) {
        let cursor_position = cursor.position();

        let is_inside = cursor_position.is_some_and(|position| layout.bounds().contains(position));

        let is_over_base =
            cursor_position.is_some_and(|position| self.content_bounds.contains(position));

        if is_press(event) && !is_inside {
            if is_over_base {
                // The base is responsible for toggling itself, so let the
                // event reach it.
                return;
            }

            // The user clicked outside of the popover: dismiss it.
            self.state.downcast_mut::<State>().is_open = false;
            shell.invalidate_layout();
            shell.request_redraw();
            return;
        }

        // Forward the event to the popover contents so interactive elements
        // inside the popover keep working.
        if is_inside || !matches!(event, Event::Mouse(_) | Event::Touch(_)) {
            let Some(popover_layout) = layout.children().next() else {
                return;
            };

            self.popover.as_widget_mut().update(
                self.popover_tree,
                event,
                popover_layout,
                cursor,
                renderer,
                shell,
                &self.viewport,
            );
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
    ) {
        // The popover content is drawn directly; users are responsible for
        // styling it (e.g. by wrapping it in a `container`).
        self.popover.as_widget().draw(
            self.popover_tree,
            renderer,
            theme,
            inherited_style,
            layout.children().next().unwrap(),
            cursor_position,
            &Rectangle::with_size(Size::INFINITE),
        );
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        if !cursor.is_over(layout.bounds()) {
            return mouse::Interaction::None;
        }

        self.popover.as_widget().mouse_interaction(
            self.popover_tree,
            layout.children().next().unwrap(),
            cursor,
            &Rectangle::with_size(Size::INFINITE),
            renderer,
        )
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.popover.as_widget_mut().operate(
            self.popover_tree,
            layout.children().next().unwrap(),
            renderer,
            operation,
        );
    }

    /// Draws the popover on top of other overlays.
    fn index(&self) -> f32 {
        2.0
    }
}

/// The possible status of a [`Popover`] base.
///
/// The base of a [`Popover`] is rendered like a [`Button`], and its [`Style`]
/// reacts to these statuses. Unlike a [`Button`], a [`Popover`] base can also
/// be in the [`Status::Opened`] state, which indicates the popover is currently
/// shown.
///
/// [`Button`]: crate::button::Button
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The [`Popover`] can be opened.
    Active,
    /// The [`Popover`] can be opened and it is being hovered.
    Hovered,
    /// The [`Popover`] is being pressed.
    Pressed,
    /// The [`Popover`] is open.
    Opened,
}

/// The style of a [`Popover`] base.
///
/// If not specified with [`Popover::style`], the theme will provide the style.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The [`Background`] of the base.
    pub background: Option<Background>,
    /// The text [`Color`] of the base.
    pub text_color: Color,
    /// The [`Border`] of the base.
    pub border: Border,
    /// The [`Shadow`] of the base.
    pub shadow: Shadow,
    /// Whether the base should be snapped to the pixel grid.
    pub snap: bool,
}

impl Style {
    /// Updates the [`Style`] with the given [`Background`].
    pub fn with_background(self, background: impl Into<Background>) -> Self {
        Self {
            background: Some(background.into()),
            ..self
        }
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            background: None,
            text_color: Color::BLACK,
            border: Border::default(),
            shadow: Shadow::default(),
            snap: renderer::CRISP,
        }
    }
}

/// The theme catalog of a [`Popover`].
///
/// All themes that can be used with [`Popover`] must implement this trait.
pub trait Catalog {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}

/// A styling function for a [`Popover`] base.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(primary)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

/// A primary popover base; denoting a main action.
pub fn primary(theme: &Theme, status: Status) -> Style {
    let palette = theme.palette();
    let base = styled(palette.primary.base);

    match status {
        Status::Active | Status::Pressed | Status::Opened => base,
        Status::Hovered => Style {
            background: Some(Background::Color(palette.primary.strong.color)),
            ..base
        },
    }
}

fn styled(pair: palette::Pair) -> Style {
    Style {
        background: Some(Background::Color(pair.color)),
        text_color: pair.text,
        border: border::rounded(2),
        ..Style::default()
    }
}
