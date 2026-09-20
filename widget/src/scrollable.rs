//! Scrollables let users navigate an endless amount of content with a scrollbar.
//!
//! # Example
//! ```no_run
//! # mod iced { pub mod widget { pub use iced_widget::*; } }
//! # pub type State = ();
//! # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
//! use iced::widget::{column, scrollable, space};
//!
//! enum Message {
//!     // ...
//! }
//!
//! fn view(state: &State) -> Element<'_, Message> {
//!     scrollable(column![
//!         "Scroll me!",
//!         space().height(3000),
//!         "You did it!",
//!     ]).into()
//! }
//! ```
use crate::container;
use crate::core::alignment;
use crate::core::border::{self, Border};
use crate::core::keyboard;
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::text;
use crate::core::time::{Duration, Instant};
use crate::core::touch;
use crate::core::widget;
use crate::core::widget::operation::{self, Operation};
use crate::core::widget::tree::{self, Tree};
use crate::core::window;
use crate::core::{
    self, Background, Color, Element, Event, InputMethod, Layout, Length, Padding, Pixels, Point,
    Rectangle, Shadow, Shell, Size, Theme, Vector, Widget,
};

pub use operation::scrollable::{AbsoluteOffset, RelativeOffset};

/// The distance (in logical pixels) scrolled per wheel line.
///
/// Chromium scrolls a fixed 120 CSS pixels per classic wheel notch,
/// independent of the page's line height: the OS delta is normalized to
/// 120 units (`ui::MouseWheelEvent::kWheelDelta`) and passed through 1:1.
///
/// This value assumes the platform reports one line per notch (e.g. X11).
/// On platforms that report three lines per notch, `40.0` (Chromium's
/// `cc::kPixelsPerLineStep`) is the equivalent value.
const WHEEL_PX_PER_LINE: f32 = 120.0;

/// A widget that can vertically display an infinite amount of content with a
/// scrollbar.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } }
/// # pub type State = ();
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// use iced::widget::{column, scrollable, space};
///
/// enum Message {
///     // ...
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     scrollable(column![
///         "Scroll me!",
///         space().height(3000),
///         "You did it!",
///     ]).into()
/// }
/// ```
pub struct Scrollable<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    Theme: Catalog,
    Renderer: core::Renderer,
{
    id: Option<widget::Id>,
    width: Length,
    height: Length,
    direction: Direction,
    auto_scroll: bool,
    smooth_scroll: bool,
    content: Element<'a, Message, Theme, Renderer>,
    on_scroll: Option<Box<dyn Fn(Viewport) -> Option<Message> + 'a>>,
    class: Theme::Class<'a>,
}

impl<'a, Message, Theme, Renderer> Scrollable<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    /// Creates a new vertical [`Scrollable`].
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self::with_direction(content, Direction::default())
    }

    /// Creates a new [`Scrollable`] with the given [`Direction`].
    pub fn with_direction(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
        direction: impl Into<Direction>,
    ) -> Self {
        Scrollable {
            id: None,
            width: Length::Fit,
            height: Length::Fit,
            direction: direction.into(),
            auto_scroll: false,
            smooth_scroll: true,
            content: content.into(),
            on_scroll: None,
            class: Theme::default(),
        }
    }

    /// Makes the [`Scrollable`] scroll horizontally, with default [`Scrollbar`] settings.
    pub fn horizontal(self) -> Self {
        self.direction(Direction::Horizontal(Scrollbar::default()))
    }

    /// Sets the [`Direction`] of the [`Scrollable`].
    pub fn direction(mut self, direction: impl Into<Direction>) -> Self {
        self.direction = direction.into();
        self
    }

    /// Sets the [`widget::Id`] of the [`Scrollable`].
    pub fn id(mut self, id: impl Into<widget::Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the width of the [`Scrollable`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Scrollable`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets a handler to call when the [`Scrollable`] is scrolled.
    ///
    /// The function takes the [`Viewport`] of the [`Scrollable`]
    pub fn on_scroll<T>(mut self, f: impl Fn(Viewport) -> T + 'a) -> Self
    where
        T: Into<Option<Message>>,
    {
        self.on_scroll = Some(Box::new(move |viewport| f(viewport).into()));
        self
    }

    /// Anchors the vertical [`Scrollable`] direction to the top.
    pub fn anchor_top(self) -> Self {
        self.anchor_y(Anchor::Start)
    }

    /// Anchors the vertical [`Scrollable`] direction to the bottom.
    pub fn anchor_bottom(self) -> Self {
        self.anchor_y(Anchor::End)
    }

    /// Anchors the horizontal [`Scrollable`] direction to the left.
    pub fn anchor_left(self) -> Self {
        self.anchor_x(Anchor::Start)
    }

    /// Anchors the horizontal [`Scrollable`] direction to the right.
    pub fn anchor_right(self) -> Self {
        self.anchor_x(Anchor::End)
    }

    /// Sets the [`Anchor`] of the horizontal direction of the [`Scrollable`], if applicable.
    pub fn anchor_x(mut self, alignment: Anchor) -> Self {
        match &mut self.direction {
            Direction::Horizontal(horizontal) | Direction::Both { horizontal, .. } => {
                horizontal.alignment = alignment;
            }
            Direction::Vertical { .. } => {}
        }

        self
    }

    /// Sets the [`Anchor`] of the vertical direction of the [`Scrollable`], if applicable.
    pub fn anchor_y(mut self, alignment: Anchor) -> Self {
        match &mut self.direction {
            Direction::Vertical(vertical) | Direction::Both { vertical, .. } => {
                vertical.alignment = alignment;
            }
            Direction::Horizontal { .. } => {}
        }

        self
    }

    /// Embeds the [`Scrollbar`] into the [`Scrollable`], instead of floating on top of the
    /// content.
    ///
    /// The `spacing` provided will be used as space between the [`Scrollbar`] and the contents
    /// of the [`Scrollable`].
    pub fn spacing(mut self, new_spacing: impl Into<Pixels>) -> Self {
        match &mut self.direction {
            Direction::Horizontal(scrollbar) | Direction::Vertical(scrollbar) => {
                scrollbar.spacing = Some(new_spacing.into().0);
            }
            Direction::Both { .. } => {}
        }

        self
    }

    /// Adds padding at the ends of the [`Scrollbar`]s of the [`Scrollable`].
    ///
    /// The `padding` provided will be used as space at the top and bottom of a
    /// vertical [`Scrollbar`], and at the left and right ends of a horizontal
    /// [`Scrollbar`], when they are visible.
    ///
    /// Unlike [`Self::spacing`], the padding does not affect the layout of the
    /// [`Scrollable`].
    pub fn padding(mut self, new_padding: impl Into<Pixels>) -> Self {
        let padding = new_padding.into().0;

        match &mut self.direction {
            Direction::Horizontal(scrollbar) | Direction::Vertical(scrollbar) => {
                scrollbar.padding = padding;
            }
            Direction::Both {
                horizontal,
                vertical,
            } => {
                horizontal.padding = padding;
                vertical.padding = padding;
            }
        }

        self
    }

    /// Sets whether the user should be allowed to auto-scroll the [`Scrollable`]
    /// with the middle mouse button.
    ///
    /// By default, it is disabled.
    pub fn auto_scroll(mut self, auto_scroll: bool) -> Self {
        self.auto_scroll = auto_scroll;
        self
    }

    /// Sets whether wheel scrolling should be smoothed out over time, instead of
    /// moving the [`Scrollable`] immediately.
    ///
    /// When enabled, discrete scrolls (e.g. from a mouse wheel) move a target
    /// scroll offset — carrying the momentum of the wheel notches — and the
    /// [`Scrollable`] eases towards it over a few frames. High-precision
    /// scrolls (e.g. from a touchpad), which are already smooth, are always
    /// applied immediately.
    ///
    /// By default, it is enabled.
    pub fn smooth_scroll(mut self, smooth_scroll: bool) -> Self {
        self.smooth_scroll = smooth_scroll;
        self
    }

    /// Sets the style of this [`Scrollable`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`Scrollable`].
    #[cfg(feature = "advanced")]
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

/// The direction of [`Scrollable`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    /// Vertical scrolling
    Vertical(Scrollbar),
    /// Horizontal scrolling
    Horizontal(Scrollbar),
    /// Both vertical and horizontal scrolling
    Both {
        /// The properties of the vertical scrollbar.
        vertical: Scrollbar,
        /// The properties of the horizontal scrollbar.
        horizontal: Scrollbar,
    },
}

impl Direction {
    /// Returns the horizontal [`Scrollbar`], if any.
    pub fn horizontal(&self) -> Option<&Scrollbar> {
        match self {
            Self::Horizontal(scrollbar) => Some(scrollbar),
            Self::Both { horizontal, .. } => Some(horizontal),
            Self::Vertical(_) => None,
        }
    }

    /// Returns the vertical [`Scrollbar`], if any.
    pub fn vertical(&self) -> Option<&Scrollbar> {
        match self {
            Self::Vertical(scrollbar) => Some(scrollbar),
            Self::Both { vertical, .. } => Some(vertical),
            Self::Horizontal(_) => None,
        }
    }

    fn align(&self, delta: Vector) -> Vector {
        let horizontal_alignment = self.horizontal().map(|p| p.alignment).unwrap_or_default();

        let vertical_alignment = self.vertical().map(|p| p.alignment).unwrap_or_default();

        let align = |alignment: Anchor, delta: f32| match alignment {
            Anchor::Start => delta,
            Anchor::End => -delta,
        };

        Vector::new(
            align(horizontal_alignment, delta.x),
            align(vertical_alignment, delta.y),
        )
    }
}

impl Default for Direction {
    fn default() -> Self {
        Self::Vertical(Scrollbar::default())
    }
}

/// A scrollbar within a [`Scrollable`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scrollbar {
    width: f32,
    margin: f32,
    scroller_width: f32,
    alignment: Anchor,
    spacing: Option<f32>,
    padding: f32,
}

impl Default for Scrollbar {
    fn default() -> Self {
        Self {
            width: 10.0,
            margin: 0.0,
            scroller_width: 10.0,
            alignment: Anchor::Start,
            spacing: None,
            padding: 0.0,
        }
    }
}

impl Scrollbar {
    /// Creates new [`Scrollbar`] for use in a [`Scrollable`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a [`Scrollbar`] with zero width to allow a [`Scrollable`] to scroll without a visible
    /// scroller.
    pub fn hidden() -> Self {
        Self::default().width(0).scroller_width(0)
    }

    /// Sets the scrollbar width of the [`Scrollbar`] .
    pub fn width(mut self, width: impl Into<Pixels>) -> Self {
        self.width = width.into().0.max(0.0);
        self
    }

    /// Sets the scrollbar margin of the [`Scrollbar`] .
    pub fn margin(mut self, margin: impl Into<Pixels>) -> Self {
        self.margin = margin.into().0;
        self
    }

    /// Sets the scroller width of the [`Scrollbar`] .
    pub fn scroller_width(mut self, scroller_width: impl Into<Pixels>) -> Self {
        self.scroller_width = scroller_width.into().0.max(0.0);
        self
    }

    /// Sets the [`Anchor`] of the [`Scrollbar`] .
    pub fn anchor(mut self, alignment: Anchor) -> Self {
        self.alignment = alignment;
        self
    }

    /// Sets whether the [`Scrollbar`] should be embedded in the [`Scrollable`], using
    /// the given spacing between itself and the contents.
    ///
    /// An embedded [`Scrollbar`] will always be displayed, will take layout space,
    /// and will not float over the contents.
    pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.spacing = Some(spacing.into().0);
        self
    }

    /// Sets the padding of the [`Scrollbar`].
    ///
    /// The padding is added at the top and bottom of the scrollbar (or at the
    /// left and right ends for a horizontal [`Scrollbar`]) when it is visible.
    ///
    /// Unlike [`Self::margin`] and [`Self::spacing`], the padding does not
    /// affect the layout of the [`Scrollable`].
    pub fn padding(mut self, padding: impl Into<Pixels>) -> Self {
        self.padding = padding.into().0;
        self
    }
}

/// The anchor of the scroller of the [`Scrollable`] relative to its [`Viewport`]
/// on a given axis.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Anchor {
    /// Scroller is anchored to the start of the [`Viewport`].
    #[default]
    Start,
    /// Content is aligned to the end of the [`Viewport`].
    End,
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Scrollable<'_, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new())
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_mut(&mut self.content));

        let state = tree.state.downcast_mut::<State>();

        if state.last_id != self.id {
            *state = State {
                last_id: self.id.clone(),
                ..State::default()
            };
        }

        let size = self.content.as_widget().size();

        self.width = self.width.stack(size.width);
        self.height = self.height.stack(size.height);
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
        let mut layout = |right_padding, bottom_padding| {
            let is_horizontal = self.direction.horizontal().is_some();
            let is_vertical = self.direction.vertical().is_some();

            layout::padded(
                limits,
                self.width,
                self.height,
                Padding {
                    right: right_padding,
                    bottom: bottom_padding,
                    ..Padding::ZERO
                },
                |limits| {
                    let child_limits = layout::Limits::with_flags(
                        limits.min,
                        limits.max,
                        limits.compression,
                        Size::new(
                            limits.infinite.width || is_horizontal,
                            limits.infinite.height || is_vertical,
                        ),
                    );

                    self.content.as_widget_mut().layout(
                        &mut tree.children[0],
                        renderer,
                        &child_limits,
                    )
                },
            )
        };

        match self.direction {
            Direction::Vertical(Scrollbar {
                width,
                margin,
                spacing: Some(spacing),
                ..
            })
            | Direction::Horizontal(Scrollbar {
                width,
                margin,
                spacing: Some(spacing),
                ..
            }) => {
                let is_vertical = matches!(self.direction, Direction::Vertical(_));

                let padding = width + margin * 2.0 + spacing;
                let state = tree.state.downcast_mut::<State>();

                let status_quo = layout(
                    if is_vertical && state.is_scrollbar_visible {
                        padding
                    } else {
                        0.0
                    },
                    if !is_vertical && state.is_scrollbar_visible {
                        padding
                    } else {
                        0.0
                    },
                );

                let is_scrollbar_visible = if is_vertical {
                    status_quo.children()[0].size().height > status_quo.size().height
                } else {
                    status_quo.children()[0].size().width > status_quo.size().width
                };

                if state.is_scrollbar_visible == is_scrollbar_visible {
                    status_quo
                } else {
                    log::trace!("Scrollbar status quo has changed");
                    state.is_scrollbar_visible = is_scrollbar_visible;

                    layout(
                        if is_vertical && state.is_scrollbar_visible {
                            padding
                        } else {
                            0.0
                        },
                        if !is_vertical && state.is_scrollbar_visible {
                            padding
                        } else {
                            0.0
                        },
                    )
                }
            }
            _ => layout(0.0, 0.0),
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        viewport: &Rectangle,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let state = tree.state.downcast_mut::<State>();

        let bounds = layout.bounds();
        let content_layout = layout.children().next().unwrap();
        let content_bounds = content_layout.bounds();
        let translation = state.last_translation;
        let viewport = viewport.intersection(&bounds).unwrap_or_default() + translation;

        operation.scrollable(self.id.as_ref(), bounds, content_bounds, translation, state);
        operation.container(self.id.as_ref(), content_bounds, &viewport);

        operation.traverse(&mut |operation| {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                content_layout,
                &viewport,
                renderer,
                operation,
            );
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        const AUTOSCROLL_DEADZONE: f32 = 20.0;
        const AUTOSCROLL_SMOOTHNESS: f32 = 1.5;

        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();
        let cursor_over_scrollable = cursor.position_over(bounds);

        let content = layout.children().next().unwrap();
        let content_bounds = content.bounds();

        let mut translation = state.last_translation;
        let scrollbars = Scrollbars::new(translation, self.direction, bounds, content_bounds);

        let (mouse_over_y_scrollbar, mouse_over_x_scrollbar) = scrollbars.is_mouse_over(cursor);

        let last_offsets = (state.offset_x, state.offset_y);

        if let Some(last_scrolled) = state.last_scrolled {
            let clear_transaction = match event {
                Event::Mouse(
                    mouse::Event::ButtonPressed(_)
                    | mouse::Event::ButtonReleased(_)
                    | mouse::Event::CursorLeft,
                ) => true,
                Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                    last_scrolled.elapsed() > Duration::from_millis(100)
                }
                _ => last_scrolled.elapsed() > Duration::from_millis(1500),
            };

            if clear_transaction {
                state.last_scrolled = None;
            }
        }

        let mut update = || {
            if let Event::Window(window::Event::RedrawRequested(now)) = event {
                // Step the smooth scrolling animation, if any;
                // `last_frame` guards against stepping twice for the
                // same instant
                if state.target.is_some()
                    && state.last_frame != Some(*now)
                    && state.step(*now, bounds, content_bounds)
                {
                    let _ = notify_scroll(state, &self.on_scroll, bounds, content_bounds, shell);
                }

                if state.target.is_some() {
                    shell.request_redraw();
                } else if let Interaction::AutoScrolling {
                    origin,
                    current,
                    last_frame,
                } = state.interaction
                {
                    if last_frame == Some(*now) {
                        shell.request_redraw();
                    } else {
                        state.interaction = Interaction::AutoScrolling {
                            origin,
                            current,
                            last_frame: None,
                        };

                        let mut delta = current - origin;

                        if delta.x.abs() < AUTOSCROLL_DEADZONE {
                            delta.x = 0.0;
                        }

                        if delta.y.abs() < AUTOSCROLL_DEADZONE {
                            delta.y = 0.0;
                        }

                        if delta.x != 0.0 || delta.y != 0.0 {
                            let time_delta = if let Some(last_frame) = last_frame {
                                *now - last_frame
                            } else {
                                Duration::ZERO
                            };

                            let scroll_factor = time_delta.as_secs_f32();

                            state.scroll(
                                self.direction.align(Vector::new(
                                    delta.x.signum()
                                        * delta.x.abs().powf(AUTOSCROLL_SMOOTHNESS)
                                        * scroll_factor,
                                    delta.y.signum()
                                        * delta.y.abs().powf(AUTOSCROLL_SMOOTHNESS)
                                        * scroll_factor,
                                )),
                                bounds,
                                content_bounds,
                            );

                            let has_scrolled = notify_scroll(
                                state,
                                &self.on_scroll,
                                bounds,
                                content_bounds,
                                shell,
                            );

                            if has_scrolled || time_delta.is_zero() {
                                state.interaction = Interaction::AutoScrolling {
                                    origin,
                                    current,
                                    last_frame: Some(*now),
                                };

                                shell.request_redraw();
                            }
                        } else {
                            let _ = notify_viewport(
                                state,
                                &self.on_scroll,
                                bounds,
                                content_bounds,
                                shell,
                            );
                        }
                    }
                } else {
                    let _ = notify_viewport(state, &self.on_scroll, bounds, content_bounds, shell);
                }

                translation = state.translation(self.direction, bounds, content_bounds);
                state.last_translation = translation;
            }

            if let Some(scroller_grabbed_at) = state.y_scroller_grabbed_at() {
                match event {
                    Event::Mouse(mouse::Event::CursorMoved { .. })
                    | Event::Touch(touch::Event::FingerMoved { .. }) => {
                        if let Some(scrollbar) = scrollbars.y {
                            let Some(cursor_position) = cursor.observe().position() else {
                                return;
                            };

                            state.scroll_y_to(
                                scrollbar.scroll_percentage_y(scroller_grabbed_at, cursor_position),
                                bounds,
                                content_bounds,
                            );

                            let _ = notify_scroll(
                                state,
                                &self.on_scroll,
                                bounds,
                                content_bounds,
                                shell,
                            );

                            shell.capture_event();
                        }
                    }
                    _ => {}
                }
            } else if mouse_over_y_scrollbar {
                match event {
                    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                    | Event::Touch(touch::Event::FingerPressed { .. }) => {
                        let Some(cursor_position) = cursor.position() else {
                            return;
                        };

                        if let (Some(scroller_grabbed_at), Some(scrollbar)) =
                            (scrollbars.grab_y_scroller(cursor_position), scrollbars.y)
                        {
                            state.scroll_y_to(
                                scrollbar.scroll_percentage_y(scroller_grabbed_at, cursor_position),
                                bounds,
                                content_bounds,
                            );

                            state.interaction = Interaction::YScrollerGrabbed(scroller_grabbed_at);

                            let _ = notify_scroll(
                                state,
                                &self.on_scroll,
                                bounds,
                                content_bounds,
                                shell,
                            );
                        }

                        shell.capture_event();
                    }
                    _ => {}
                }
            }

            if let Some(scroller_grabbed_at) = state.x_scroller_grabbed_at() {
                match event {
                    Event::Mouse(mouse::Event::CursorMoved { .. })
                    | Event::Touch(touch::Event::FingerMoved { .. }) => {
                        let Some(cursor_position) = cursor.observe().position() else {
                            return;
                        };

                        if let Some(scrollbar) = scrollbars.x {
                            state.scroll_x_to(
                                scrollbar.scroll_percentage_x(scroller_grabbed_at, cursor_position),
                                bounds,
                                content_bounds,
                            );

                            let _ = notify_scroll(
                                state,
                                &self.on_scroll,
                                bounds,
                                content_bounds,
                                shell,
                            );
                        }

                        shell.capture_event();
                    }
                    _ => {}
                }
            } else if mouse_over_x_scrollbar {
                match event {
                    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                    | Event::Touch(touch::Event::FingerPressed { .. }) => {
                        let Some(cursor_position) = cursor.position() else {
                            return;
                        };

                        if let (Some(scroller_grabbed_at), Some(scrollbar)) =
                            (scrollbars.grab_x_scroller(cursor_position), scrollbars.x)
                        {
                            state.scroll_x_to(
                                scrollbar.scroll_percentage_x(scroller_grabbed_at, cursor_position),
                                bounds,
                                content_bounds,
                            );

                            state.interaction = Interaction::XScrollerGrabbed(scroller_grabbed_at);

                            let _ = notify_scroll(
                                state,
                                &self.on_scroll,
                                bounds,
                                content_bounds,
                                shell,
                            );

                            shell.capture_event();
                        }
                    }
                    _ => {}
                }
            }

            if matches!(state.interaction, Interaction::AutoScrolling { .. })
                && matches!(
                    event,
                    Event::Mouse(
                        mouse::Event::ButtonPressed(_) | mouse::Event::WheelScrolled { .. }
                    ) | Event::Touch(_)
                        | Event::Keyboard(_)
                )
            {
                state.interaction = Interaction::None;
                shell.capture_event();
                shell.invalidate_layout();
                shell.request_redraw();
                return;
            }

            if state.last_scrolled.is_none()
                || !matches!(event, Event::Mouse(mouse::Event::WheelScrolled { .. }))
            {
                let cursor = match cursor_over_scrollable {
                    Some(cursor_position)
                        if !(mouse_over_x_scrollbar
                            || mouse_over_y_scrollbar
                            || state.scrollers_grabbed()) =>
                    {
                        mouse::Cursor::Available(cursor_position + translation)
                    }
                    _ => cursor.obstruct() + translation,
                };

                let had_input_method = shell.input_method().is_enabled();

                self.content.as_widget_mut().update(
                    &mut tree.children[0],
                    event,
                    content,
                    cursor,
                    renderer,
                    shell,
                    &Rectangle {
                        y: bounds.y + translation.y,
                        x: bounds.x + translation.x,
                        ..bounds
                    },
                );

                if !had_input_method
                    && let InputMethod::Enabled { cursor, .. } = shell.input_method_mut()
                {
                    *cursor -= translation;
                }
            };

            if matches!(
                event,
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                    | Event::Touch(
                        touch::Event::FingerLifted { .. } | touch::Event::FingerLost { .. }
                    )
            ) {
                state.interaction = Interaction::None;
                return;
            }

            if shell.is_event_captured() {
                return;
            }

            match event {
                Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                    if !cursor.land().is_over(bounds) {
                        return;
                    }

                    let (delta, is_lines) = match *delta {
                        mouse::ScrollDelta::Lines { x, y } => {
                            let is_shift_pressed = state.keyboard_modifiers.shift();

                            // macOS automatically inverts the axes when Shift is pressed
                            let (x, y) = if cfg!(target_os = "macos") && is_shift_pressed {
                                (y, x)
                            } else {
                                (x, y)
                            };

                            let movement = if !is_shift_pressed {
                                Vector::new(x, y)
                            } else {
                                Vector::new(y, x)
                            };

                            (-movement * WHEEL_PX_PER_LINE, true)
                        }
                        // Pixel deltas (e.g. from high-precision touchpads) are
                        // already smooth, so scrolling them immediately avoids
                        // double-smoothing them
                        mouse::ScrollDelta::Pixels { x, y } => (-Vector::new(x, y), false),
                    };

                    let delta = self.direction.align(delta);

                    if self.smooth_scroll && is_lines {
                        state.scroll_smoothly(
                            delta,
                            bounds,
                            content_bounds,
                            Instant::now()
                                - Duration::from_secs_f32(State::SMOOTH_SCROLL_FRAME_DELAY),
                        );
                    } else {
                        state.scroll(delta, bounds, content_bounds);
                    }

                    let has_scrolled =
                        notify_scroll(state, &self.on_scroll, bounds, content_bounds, shell);

                    let in_transaction = state.last_scrolled.is_some() || state.target.is_some();

                    if has_scrolled || in_transaction {
                        shell.capture_event();
                    }

                    if state.target.is_some() {
                        shell.request_redraw();
                    }
                }
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Middle))
                    if self.auto_scroll && matches!(state.interaction, Interaction::None) =>
                {
                    let Some(origin) = cursor_over_scrollable else {
                        return;
                    };

                    state.interaction = Interaction::AutoScrolling {
                        origin,
                        current: origin,
                        last_frame: None,
                    };

                    shell.capture_event();
                    shell.invalidate_layout();
                    shell.request_redraw();
                }
                Event::Touch(event)
                    if matches!(state.interaction, Interaction::TouchScrolling(_))
                        || (!mouse_over_y_scrollbar && !mouse_over_x_scrollbar) =>
                {
                    match event {
                        touch::Event::FingerPressed { .. } => {
                            let Some(position) = cursor_over_scrollable else {
                                return;
                            };

                            state.interaction = Interaction::TouchScrolling(position);
                        }
                        touch::Event::FingerMoved { .. } => {
                            let Interaction::TouchScrolling(scroll_box_touched_at) =
                                state.interaction
                            else {
                                return;
                            };

                            let Some(cursor_position) = cursor.position() else {
                                return;
                            };

                            let delta = Vector::new(
                                scroll_box_touched_at.x - cursor_position.x,
                                scroll_box_touched_at.y - cursor_position.y,
                            );

                            state.scroll(self.direction.align(delta), bounds, content_bounds);

                            state.interaction = Interaction::TouchScrolling(cursor_position);

                            // TODO: bubble up touch movements if not consumed.
                            let _ = notify_scroll(
                                state,
                                &self.on_scroll,
                                bounds,
                                content_bounds,
                                shell,
                            );
                        }
                        _ => {}
                    }

                    shell.capture_event();
                }
                Event::Mouse(mouse::Event::CursorMoved { position }) => {
                    if let Interaction::AutoScrolling {
                        origin, last_frame, ..
                    } = state.interaction
                    {
                        let delta = *position - origin;

                        state.interaction = Interaction::AutoScrolling {
                            origin,
                            current: *position,
                            last_frame,
                        };

                        if (delta.x.abs() >= AUTOSCROLL_DEADZONE
                            || delta.y.abs() >= AUTOSCROLL_DEADZONE)
                            && last_frame.is_none()
                        {
                            shell.request_redraw();
                        }
                    }
                }
                Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                    state.keyboard_modifiers = *modifiers;
                }
                _ => {}
            }
        };

        update();

        let status = if state.scrollers_grabbed() {
            Status::Dragged {
                is_horizontal_scrollbar_dragged: state.x_scroller_grabbed_at().is_some(),
                is_vertical_scrollbar_dragged: state.y_scroller_grabbed_at().is_some(),
                is_horizontal_scrollbar_disabled: scrollbars.is_x_disabled(),
                is_vertical_scrollbar_disabled: scrollbars.is_y_disabled(),
            }
        } else if cursor_over_scrollable.is_some() {
            Status::Hovered {
                is_horizontal_scrollbar_hovered: mouse_over_x_scrollbar,
                is_vertical_scrollbar_hovered: mouse_over_y_scrollbar,
                is_horizontal_scrollbar_disabled: scrollbars.is_x_disabled(),
                is_vertical_scrollbar_disabled: scrollbars.is_y_disabled(),
            }
        } else {
            Status::Active {
                is_horizontal_scrollbar_disabled: scrollbars.is_x_disabled(),
                is_vertical_scrollbar_disabled: scrollbars.is_y_disabled(),
            }
        };

        if let Event::Window(window::Event::RedrawRequested(_now)) = event {
            state.last_status = Some(status);
        }

        if last_offsets != (state.offset_x, state.offset_y)
            || state
                .last_status
                .is_some_and(|last_status| last_status != status)
        {
            shell.request_redraw();
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        defaults: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();

        let Some(viewport) = viewport.intersection(&bounds) else {
            return;
        };

        let content_layout = layout.children().next().unwrap();
        let content_bounds = content_layout.bounds();

        let translation = state.last_translation;
        let scrollbars = Scrollbars::new(translation, self.direction, bounds, content_bounds);
        let cursor_over_scrollable = cursor.position_over(bounds);
        let (mouse_over_y_scrollbar, mouse_over_x_scrollbar) = scrollbars.is_mouse_over(cursor);

        let cursor = match cursor_over_scrollable {
            Some(cursor_position) if !(mouse_over_x_scrollbar || mouse_over_y_scrollbar) => {
                mouse::Cursor::Available(cursor_position + translation)
            }
            _ => cursor.obstruct() + translation,
        };

        let style = theme.style(
            &self.class,
            state.last_status.unwrap_or(Status::Active {
                is_horizontal_scrollbar_disabled: false,
                is_vertical_scrollbar_disabled: false,
            }),
        );

        container::draw_background(renderer, &style.container, layout.bounds());

        // Draw inner content
        if scrollbars.active() {
            renderer.with_layer(viewport, |renderer| {
                renderer.with_translation(
                    -translation.hint(renderer.hint_factor().unwrap_or(1.0)),
                    |renderer| {
                        self.content.as_widget().draw(
                            &tree.children[0],
                            renderer,
                            theme,
                            defaults,
                            content_layout,
                            cursor,
                            &(viewport + translation),
                        );
                    },
                );
            });

            let draw_scrollbar =
                |renderer: &mut Renderer, style: Rail, scrollbar: &internals::Scrollbar| {
                    if scrollbar.bounds.width > 0.0
                        && scrollbar.bounds.height > 0.0
                        && (style.background.is_some()
                            || (style.border.color != Color::TRANSPARENT
                                && style.border.width > 0.0))
                    {
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: scrollbar.bounds,
                                border: style.border,
                                ..renderer::Quad::default()
                            },
                            style
                                .background
                                .unwrap_or(Background::Color(Color::TRANSPARENT)),
                        );
                    }

                    if let Some(scroller) = scrollbar.scroller
                        && scroller.bounds.width > 0.0
                        && scroller.bounds.height > 0.0
                        && (style.scroller.background != Background::Color(Color::TRANSPARENT)
                            || (style.scroller.border.color != Color::TRANSPARENT
                                && style.scroller.border.width > 0.0))
                    {
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: scroller.bounds,
                                border: style.scroller.border,
                                ..renderer::Quad::default()
                            },
                            style.scroller.background,
                        );
                    }
                };

            let has_floating_scrollbar = scrollbars.is_any_floating();

            if has_floating_scrollbar {
                renderer.start_layer(viewport);
            }

            if let Some(scrollbar) = scrollbars.y {
                draw_scrollbar(renderer, style.vertical_rail, &scrollbar);
            }

            if let Some(scrollbar) = scrollbars.x {
                draw_scrollbar(renderer, style.horizontal_rail, &scrollbar);
            }

            if let (Some(x), Some(y)) = (scrollbars.x, scrollbars.y) {
                let background = style.gap.or(style.container.background);

                if let Some(background) = background {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: Rectangle {
                                x: y.bounds.x,
                                y: x.bounds.y,
                                width: y.bounds.width,
                                height: x.bounds.height,
                            },
                            ..renderer::Quad::default()
                        },
                        background,
                    );
                }
            }

            if has_floating_scrollbar {
                renderer.end_layer();
            }
        } else {
            self.content.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                defaults,
                content_layout,
                cursor,
                &(viewport + translation),
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<State>();

        if state.scrollers_grabbed() {
            return mouse::Interaction::Idle;
        }

        let Some(viewport) = viewport.intersection(&bounds) else {
            return mouse::Interaction::None;
        };

        let cursor_over_scrollable = cursor.position_over(bounds);
        let content_layout = layout.children().next().unwrap();
        let content_bounds = content_layout.bounds();

        let translation = state.last_translation;
        let scrollbars = Scrollbars::new(translation, self.direction, bounds, content_bounds);
        let (mouse_over_y_scrollbar, mouse_over_x_scrollbar) = scrollbars.is_mouse_over(cursor);

        let cursor = match cursor_over_scrollable {
            Some(cursor_position) if !(mouse_over_x_scrollbar || mouse_over_y_scrollbar) => {
                mouse::Cursor::Available(cursor_position + translation)
            }
            _ => cursor.obstruct() + translation,
        };

        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            content_layout,
            cursor,
            &(viewport + translation),
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
        window: Size,
    ) -> Vec<overlay::Element<'b, Message, Theme, Renderer>> {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let content_layout = layout.children().next().unwrap();
        let content_bounds = content_layout.bounds();
        let viewport = viewport.intersection(&bounds).unwrap_or(*viewport);

        let offset = state.last_translation;

        let overlay = self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            &(viewport + offset),
            translation - offset,
            window,
        );

        let icon = if let Interaction::AutoScrolling { origin, .. } = state.interaction {
            let scrollbars = Scrollbars::new(offset, self.direction, bounds, content_bounds);

            Some(overlay::Element::new(Box::new(AutoScrollIcon {
                origin,
                vertical: scrollbars.y.is_some(),
                horizontal: scrollbars.x.is_some(),
                class: &self.class,
            })))
        } else {
            None
        };

        overlay.into_iter().chain(icon).collect()
    }
}

struct AutoScrollIcon<'a, Class> {
    origin: Point,
    vertical: bool,
    horizontal: bool,
    class: &'a Class,
}

impl<Class> AutoScrollIcon<'_, Class> {
    const SIZE: f32 = 40.0;
    const DOT: f32 = Self::SIZE / 10.0;
    const PADDING: f32 = Self::SIZE / 10.0;
}

impl<Message, Theme, Renderer> core::Overlay<Message, Theme, Renderer>
    for AutoScrollIcon<'_, Theme::Class<'_>>
where
    Renderer: text::Renderer,
    Theme: Catalog,
{
    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        _cursor: mouse::Cursor,
    ) {
        let bounds = Rectangle::new(
            self.origin - Vector::new(Self::SIZE, Self::SIZE) / 2.0,
            Size::new(Self::SIZE, Self::SIZE),
        );

        let style = theme
            .style(
                self.class,
                Status::Active {
                    is_horizontal_scrollbar_disabled: false,
                    is_vertical_scrollbar_disabled: false,
                },
            )
            .auto_scroll;

        renderer.with_layer(bounds, |renderer| {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: style.border,
                    shadow: style.shadow,
                    snap: false,
                },
                style.background,
            );

            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle::new(
                        bounds.center() - Vector::new(Self::DOT, Self::DOT) / 2.0,
                        Size::new(Self::DOT, Self::DOT),
                    ),
                    border: border::rounded(bounds.width),
                    snap: false,
                    ..renderer::Quad::default()
                },
                style.icon,
            );

            let arrow = core::Text {
                content: String::new(),
                bounds: bounds.size(),
                size: Pixels::from(12),
                line_height: text::LineHeight::from(1.0),
                font: Renderer::ICON_FONT,
                align_x: text::Alignment::Center,
                align_y: alignment::Vertical::Center,
                shaping: text::Shaping::Basic,
                wrapping: text::Wrapping::None,
                ellipsis: text::Ellipsis::None,
                hint_factor: None,
            };

            if self.vertical {
                renderer.fill_text(
                    core::Text {
                        content: Renderer::SCROLL_UP_ICON.to_string(),
                        align_y: alignment::Vertical::Top,
                        ..arrow
                    },
                    Point::new(bounds.center_x(), bounds.y + Self::PADDING),
                    style.icon,
                    bounds,
                );

                renderer.fill_text(
                    core::Text {
                        content: Renderer::SCROLL_DOWN_ICON.to_string(),
                        align_y: alignment::Vertical::Bottom,
                        ..arrow
                    },
                    Point::new(
                        bounds.center_x(),
                        bounds.y + bounds.height - Self::PADDING - 0.5,
                    ),
                    style.icon,
                    bounds,
                );
            }

            if self.horizontal {
                renderer.fill_text(
                    core::Text {
                        content: Renderer::SCROLL_LEFT_ICON.to_string(),
                        align_x: text::Alignment::Left,
                        ..arrow
                    },
                    Point::new(bounds.x + Self::PADDING + 1.0, bounds.center_y() + 1.0),
                    style.icon,
                    bounds,
                );

                renderer.fill_text(
                    core::Text {
                        content: Renderer::SCROLL_RIGHT_ICON.to_string(),
                        align_x: text::Alignment::Right,
                        ..arrow
                    },
                    Point::new(
                        bounds.x + bounds.width - Self::PADDING - 1.0,
                        bounds.center_y() + 1.0,
                    ),
                    style.icon,
                    bounds,
                );
            }
        });
    }

    fn index(&self) -> f32 {
        f32::MAX
    }
}

impl<'a, Message, Theme, Renderer> From<Scrollable<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a + Catalog,
    Renderer: 'a + text::Renderer,
{
    fn from(
        text_input: Scrollable<'a, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(text_input)
    }
}

fn notify_scroll<Message>(
    state: &mut State,
    on_scroll: &Option<Box<dyn Fn(Viewport) -> Option<Message> + '_>>,
    bounds: Rectangle,
    content_bounds: Rectangle,
    shell: &mut Shell<'_, Message>,
) -> bool {
    if notify_viewport(state, on_scroll, bounds, content_bounds, shell) {
        state.last_scrolled = Some(Instant::now());

        true
    } else {
        false
    }
}

fn notify_viewport<Message>(
    state: &mut State,
    on_scroll: &Option<Box<dyn Fn(Viewport) -> Option<Message> + '_>>,
    bounds: Rectangle,
    content_bounds: Rectangle,
    shell: &mut Shell<'_, Message>,
) -> bool {
    if content_bounds.width <= bounds.width && content_bounds.height <= bounds.height {
        return false;
    }

    let viewport = Viewport {
        offset_x: state.offset_x,
        offset_y: state.offset_y,
        bounds,
        content_bounds,
        is_animating: state.target.is_some(),
    };

    // Don't publish redundant viewports to shell
    if let Some(last_notified) = state.last_notified {
        let last_relative_offset = last_notified.relative_offset();
        let current_relative_offset = viewport.relative_offset();

        let last_absolute_offset = last_notified.absolute_offset();
        let current_absolute_offset = viewport.absolute_offset();

        let unchanged =
            |a: f32, b: f32| (a - b).abs() <= f32::EPSILON || (a.is_nan() && b.is_nan());

        if last_notified.bounds == bounds
            && last_notified.content_bounds == content_bounds
            && unchanged(last_relative_offset.x, current_relative_offset.x)
            && unchanged(last_relative_offset.y, current_relative_offset.y)
            && unchanged(last_absolute_offset.x, current_absolute_offset.x)
            && unchanged(last_absolute_offset.y, current_absolute_offset.y)
        {
            return false;
        }
    }

    state.last_notified = Some(viewport);

    if let Some(on_scroll) = on_scroll
        && let Some(message) = on_scroll(viewport)
    {
        shell.publish(message);
    }

    true
}

#[derive(Debug, Clone)]
struct State {
    offset_y: Offset,
    offset_x: Offset,
    target: Option<Vector>,
    segment_start: Point,
    segment_started: Option<Instant>,
    segment_duration: f32,
    segment_slope: f32,
    last_frame: Option<Instant>,
    last_translation: Vector,
    interaction: Interaction,
    keyboard_modifiers: keyboard::Modifiers,
    last_notified: Option<Viewport>,
    last_scrolled: Option<Instant>,
    is_scrollbar_visible: bool,
    last_status: Option<Status>,
    last_id: Option<widget::Id>,
}

#[derive(Debug, Clone, Copy)]
enum Interaction {
    None,
    YScrollerGrabbed(f32),
    XScrollerGrabbed(f32),
    TouchScrolling(Point),
    AutoScrolling {
        origin: Point,
        current: Point,
        last_frame: Option<Instant>,
    },
}

impl Default for State {
    fn default() -> Self {
        Self {
            offset_y: Offset::Absolute(0.0),
            offset_x: Offset::Absolute(0.0),
            target: None,
            segment_start: Point::ORIGIN,
            segment_started: None,
            segment_duration: 0.0,
            segment_slope: 0.0,
            last_frame: None,
            last_translation: Vector::ZERO,
            interaction: Interaction::None,
            keyboard_modifiers: keyboard::Modifiers::default(),
            last_notified: None,
            last_scrolled: None,
            is_scrollbar_visible: true,
            last_status: None,
            last_id: None,
        }
    }
}

impl operation::Scrollable for State {
    fn snap_to(&mut self, offset: RelativeOffset<Option<f32>>) {
        State::snap_to(self, offset);
    }

    fn scroll_to(&mut self, offset: AbsoluteOffset<Option<f32>>) {
        State::scroll_to(self, offset);
    }

    fn scroll_by(&mut self, offset: AbsoluteOffset, bounds: Rectangle, content_bounds: Rectangle) {
        State::scroll_by(self, offset, bounds, content_bounds);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Offset {
    Absolute(f32),
    Relative(f32),
}

impl Offset {
    fn absolute(self, viewport: f32, content: f32) -> f32 {
        match self {
            Offset::Absolute(absolute) => absolute.min((content - viewport).max(0.0)),
            Offset::Relative(percentage) => ((content - viewport) * percentage).max(0.0),
        }
    }

    fn translation(self, viewport: f32, content: f32, alignment: Anchor) -> f32 {
        let offset = self.absolute(viewport, content);

        match alignment {
            Anchor::Start => offset,
            Anchor::End => ((content - viewport).max(0.0) - offset).max(0.0),
        }
    }
}

/// The current [`Viewport`] of the [`Scrollable`].
#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    offset_x: Offset,
    offset_y: Offset,
    bounds: Rectangle,
    content_bounds: Rectangle,
    is_animating: bool,
}

impl Viewport {
    /// Returns the [`AbsoluteOffset`] of the current [`Viewport`].
    pub fn absolute_offset(&self) -> AbsoluteOffset {
        let x = self
            .offset_x
            .absolute(self.bounds.width, self.content_bounds.width);
        let y = self
            .offset_y
            .absolute(self.bounds.height, self.content_bounds.height);

        AbsoluteOffset { x, y }
    }

    /// Returns the [`AbsoluteOffset`] of the current [`Viewport`], but with its
    /// alignment reversed.
    ///
    /// This method can be useful to switch the alignment of a [`Scrollable`]
    /// while maintaining its scrolling position.
    pub fn absolute_offset_reversed(&self) -> AbsoluteOffset {
        let AbsoluteOffset { x, y } = self.absolute_offset();

        AbsoluteOffset {
            x: (self.content_bounds.width - self.bounds.width).max(0.0) - x,
            y: (self.content_bounds.height - self.bounds.height).max(0.0) - y,
        }
    }

    /// Returns the [`RelativeOffset`] of the current [`Viewport`].
    pub fn relative_offset(&self) -> RelativeOffset {
        let AbsoluteOffset { x, y } = self.absolute_offset();

        let x = x / (self.content_bounds.width - self.bounds.width);
        let y = y / (self.content_bounds.height - self.bounds.height);

        RelativeOffset { x, y }
    }

    /// Returns the bounds of the current [`Viewport`].
    pub fn bounds(&self) -> Rectangle {
        self.bounds
    }

    /// Returns the content bounds of the current [`Viewport`].
    pub fn content_bounds(&self) -> Rectangle {
        self.content_bounds
    }

    /// Returns whether a smooth scrolling animation is in progress.
    pub fn is_animating(&self) -> bool {
        self.is_animating
    }
}

impl State {
    // The smooth scrolling behavior below (the easing curve, the animation
    // durations, and the velocity-preserving retargeting) is derived from
    // the Chromium project's wheel scroll animation
    // (`cc/animation/scroll_offset_animation_curve.{h,cc}`), which is
    // licensed under the BSD 3-Clause license:
    // <https://chromium.googlesource.com/chromium/src/+/main/LICENSE>

    /// The control points of the smooth scrolling easing curve.
    ///
    /// This is the standard ease-in-out cubic bezier, as used by Chromium for
    /// wheel scrolling: the scroll starts from rest, peaks mid-way, and
    /// settles on the target at rest.
    const SMOOTH_SCROLL_BEZIER_X1: f32 = 0.42;
    const SMOOTH_SCROLL_BEZIER_X2: f32 = 0.58;

    /// The divisor of the smooth scrolling animation duration, matching
    /// Chromium's.
    const SMOOTH_SCROLL_DURATION_DIVISOR: f32 = 60.0;

    /// The delay (in seconds) between a wheel event and the next drawn frame.
    ///
    /// Wheel events are processed between frames, and the smooth scrolling
    /// animation is only stepped when the next frame is drawn. Starting the
    /// animation one nominal frame *before* the event, where a nominal frame
    /// is one unit of `SMOOTH_SCROLL_DURATION_DIVISOR`, ensures that the
    /// first drawn frame already shows some progress, instead of repeating
    /// the previous one.
    ///
    /// A full frame is used rather than the mean delay of half a frame: it
    /// guarantees that the first drawn frame visibly moves, even when the
    /// event arrives right after a frame. The animation then settles one
    /// frame early, which is imperceptible since the curve ends at rest.
    const SMOOTH_SCROLL_FRAME_DELAY: f32 = 1.0 / Self::SMOOTH_SCROLL_DURATION_DIVISOR;

    /// The distances (in pixels) at which the smooth scrolling duration ramp
    /// starts and ends.
    const SMOOTH_SCROLL_DURATION_RAMP_START: f32 = WHEEL_PX_PER_LINE;
    const SMOOTH_SCROLL_DURATION_RAMP_END: f32 = 480.0;

    /// The shortest and longest smooth scrolling animation durations, in
    /// `SMOOTH_SCROLL_DURATION_DIVISOR` units, matching Chromium's.
    ///
    /// The duration is *inversely* proportional to the distance within these
    /// bounds: short scrolls get a longer (softer) animation, while long
    /// scrolls get a shorter (snappier) one.
    const SMOOTH_SCROLL_DURATION_MIN: f32 = 6.0;
    const SMOOTH_SCROLL_DURATION_MAX: f32 = 12.0;

    /// The factor applied to the time it would take to cover the new distance
    /// at the current velocity when retargeting a running animation, matching
    /// Chromium's.
    ///
    /// Bounding the new duration by this keeps a fast scroll from "rubber
    /// banding" when a small new delta is added at high velocity.
    const SMOOTH_SCROLL_RETARGET_VELOCITY_BOUND: f32 = 2.5;

    /// The clamp for the initial slope of a retargeted animation, matching
    /// Chromium's.
    const SMOOTH_SCROLL_SLOPE_CLAMP: f32 = 1000.0;

    fn new() -> Self {
        State::default()
    }

    fn scroll(&mut self, delta: Vector<f32>, bounds: Rectangle, content_bounds: Rectangle) {
        self.cancel();

        if bounds.height < content_bounds.height {
            self.offset_y = Offset::Absolute(
                (self.offset_y.absolute(bounds.height, content_bounds.height) + delta.y)
                    .clamp(0.0, content_bounds.height - bounds.height),
            );
        }

        if bounds.width < content_bounds.width {
            self.offset_x = Offset::Absolute(
                (self.offset_x.absolute(bounds.width, content_bounds.width) + delta.x)
                    .clamp(0.0, content_bounds.width - bounds.width),
            );
        }
    }

    /// Moves the *target* scroll offset by `delta`, for smooth scrolling.
    ///
    /// The target is then eased towards on each frame, via [`State::step`],
    /// with an ease-in-out animation whose duration depends on the distance:
    /// short scrolls get a longer (softer) animation, while long scrolls get a
    /// shorter (snappier) one. When a new delta arrives while an animation is
    /// running, the animation is retargeted from the current position and
    /// velocity, so that continuous scrolling flows instead of restarting.
    fn scroll_smoothly(
        &mut self,
        delta: Vector<f32>,
        bounds: Rectangle,
        content_bounds: Rectangle,
        now: Instant,
    ) {
        // Materialize any snapped (relative) offsets before animating from them
        self.unsnap(bounds, content_bounds);

        let current = Point::new(
            self.offset_x.absolute(bounds.width, content_bounds.width),
            self.offset_y.absolute(bounds.height, content_bounds.height),
        );

        // Accumulate onto the pending target, if any, so that quick wheel
        // movements do not lose their (not yet scrolled) distance
        let target = match self.target {
            Some(target) => Vector::new(
                Self::clamp_offset(target.x + delta.x, bounds.width, content_bounds.width),
                Self::clamp_offset(target.y + delta.y, bounds.height, content_bounds.height),
            ),
            None => Vector::new(
                Self::clamp_offset(current.x + delta.x, bounds.width, content_bounds.width),
                Self::clamp_offset(current.y + delta.y, bounds.height, content_bounds.height),
            ),
        };

        // Nothing to animate: the content fits, or we're already at the target
        if target.x == current.x && target.y == current.y {
            self.target = None;
            self.last_frame = None;
            return;
        }

        let Some(ongoing) = self.target else {
            // A new scroll run: start a fresh segment from rest at the
            // current position
            let distance = (target.x - current.x)
                .abs()
                .max((target.y - current.y).abs());

            self.target = Some(target);
            self.segment_start = current;
            self.segment_started = Some(now);
            self.segment_duration = Self::smooth_scroll_duration(distance);
            self.segment_slope = 0.0;
            self.last_frame = Some(now);
            return;
        };

        // The delta was clamped away: the target is unchanged, so keep the
        // running segment as is
        if ongoing == target {
            return;
        }

        // Retarget the running segment from the current position, preserving
        // the current velocity
        let start = self.animated_position(now);
        let velocity = self.animated_velocity(now);
        let new = Vector::new(target.x - start.x, target.y - start.y);

        // The signed dimension with the largest magnitude, like Chromium's
        let max_dimension = if new.x.abs() > new.y.abs() {
            new.x
        } else {
            new.y
        };

        // The new duration, bounded so that a small delta added at high
        // velocity does not "rubber band"; a bound with the wrong sign means
        // the new delta is against the current motion, and does not apply
        let mut duration = Self::smooth_scroll_duration(max_dimension.abs());
        if velocity.abs() > 0.01 {
            let bound = Self::SMOOTH_SCROLL_RETARGET_VELOCITY_BOUND * max_dimension / velocity;

            if bound > 0.0 {
                duration = duration.min(bound);
            }
        }

        if max_dimension.abs() < 0.01 || duration < 0.01 {
            // The new target is right on top of us: end the animation now
            self.offset_x = Offset::Absolute(target.x);
            self.offset_y = Offset::Absolute(target.y);
            self.target = None;
            self.last_frame = None;
            return;
        }

        // Adjust the initial slope of the new segment so that it starts with
        // the current velocity
        let slope = (velocity * (duration / max_dimension)).clamp(
            -Self::SMOOTH_SCROLL_SLOPE_CLAMP,
            Self::SMOOTH_SCROLL_SLOPE_CLAMP,
        );

        self.target = Some(target);
        self.segment_start = start;
        self.segment_started = Some(now);
        self.segment_duration = duration;
        self.segment_slope = slope;
        self.last_frame = Some(now);
    }

    /// Steps the smooth scrolling animation forward, towards the target
    /// offset.
    ///
    /// Returns `true` if the animation is still in progress.
    fn step(&mut self, now: Instant, bounds: Rectangle, content_bounds: Rectangle) -> bool {
        let Some(target) = self.target else {
            return false;
        };
        let Some(started) = self.segment_started else {
            return false;
        };

        // Clamp the target in case the bounds changed while the animation is
        // running
        let target = Vector::new(
            Self::clamp_offset(target.x, bounds.width, content_bounds.width),
            Self::clamp_offset(target.y, bounds.height, content_bounds.height),
        );

        let t = (now - started).as_secs_f32();
        let progress = (t / self.segment_duration).clamp(0.0, 1.0);

        if progress >= 1.0 {
            // Settled exactly on the target
            self.offset_x = Offset::Absolute(target.x);
            self.offset_y = Offset::Absolute(target.y);
            self.target = None;
            self.last_frame = None;
            return false;
        }

        let bez = Self::smooth_scroll_progress(progress, self.segment_slope);
        self.offset_x =
            Offset::Absolute(self.segment_start.x + (target.x - self.segment_start.x) * bez);
        self.offset_y =
            Offset::Absolute(self.segment_start.y + (target.y - self.segment_start.y) * bez);

        self.last_frame = Some(now);
        true
    }

    /// Cancels any in-progress smooth scrolling animation, keeping the current
    /// offsets.
    ///
    /// This ensures that direct manipulation (scrollbar drags, touch scrolling,
    /// auto-scrolling, programmatic scrolling) always takes priority over a
    /// pending wheel animation.
    fn cancel(&mut self) {
        self.target = None;
        self.last_frame = None;
    }

    /// Clamps an absolute scroll offset to the range in which the given content
    /// can be scrolled inside the given viewport.
    fn clamp_offset(offset: f32, viewport: f32, content: f32) -> f32 {
        if content > viewport {
            offset.clamp(0.0, content - viewport)
        } else {
            0.0
        }
    }

    /// The animated position at `now`, while a segment is running.
    fn animated_position(&self, now: Instant) -> Point {
        let started = self.segment_started.expect("a segment is running");
        let target = self.target.expect("a segment is running");

        let t = (now - started).as_secs_f32();
        let progress = (t / self.segment_duration).clamp(0.0, 1.0);
        let bez = Self::smooth_scroll_progress(progress, self.segment_slope);

        Point::new(
            self.segment_start.x + (target.x - self.segment_start.x) * bez,
            self.segment_start.y + (target.y - self.segment_start.y) * bez,
        )
    }

    /// The animated velocity at `now`, in pixels per second along the
    /// segment's largest dimension, while a segment is running.
    fn animated_velocity(&self, now: Instant) -> f32 {
        let started = self.segment_started.expect("a segment is running");
        let target = self.target.expect("a segment is running");

        let t = (now - started).as_secs_f32();
        let progress = (t / self.segment_duration).clamp(0.0, 1.0);

        if progress >= 1.0 {
            return 0.0;
        }

        let dx = target.x - self.segment_start.x;
        let dy = target.y - self.segment_start.y;
        let max_dimension = if dx.abs() > dy.abs() { dx } else { dy };

        Self::smooth_scroll_curve_slope(progress, self.segment_slope) * max_dimension
            / self.segment_duration
    }

    /// The duration (in seconds) of a smooth scrolling segment covering the
    /// given distance (in pixels).
    ///
    /// The duration is inversely proportional to the distance within a ramp:
    /// short scrolls get a longer (softer) animation, while long scrolls get a
    /// shorter (snappier) one.
    fn smooth_scroll_duration(distance: f32) -> f32 {
        let slope = (Self::SMOOTH_SCROLL_DURATION_MIN - Self::SMOOTH_SCROLL_DURATION_MAX)
            / (Self::SMOOTH_SCROLL_DURATION_RAMP_END - Self::SMOOTH_SCROLL_DURATION_RAMP_START);
        let offset =
            Self::SMOOTH_SCROLL_DURATION_MAX - Self::SMOOTH_SCROLL_DURATION_RAMP_START * slope;

        (offset + distance * slope).clamp(
            Self::SMOOTH_SCROLL_DURATION_MIN,
            Self::SMOOTH_SCROLL_DURATION_MAX,
        ) / Self::SMOOTH_SCROLL_DURATION_DIVISOR
    }

    /// The progress of the smooth scrolling easing curve at the given time
    /// progress (in `[0, 1]`), with the given initial slope.
    ///
    /// The curve is an ease-in-out cubic bezier whose initial control point is
    /// scaled by `slope` (a slope of `0.0` is the plain ease-in-out curve).
    fn smooth_scroll_progress(time: f32, slope: f32) -> f32 {
        if time <= 0.0 {
            return 0.0;
        }

        if time >= 1.0 {
            return 1.0;
        }

        let s = Self::bezier_solve(time);
        let y1 = Self::SMOOTH_SCROLL_BEZIER_X1 * slope;
        let os = 1.0 - s;

        // y(s), with y2 = 1
        3.0 * y1 * s * os * os + 3.0 * s * s * os + s * s * s
    }

    /// The slope of the smooth scrolling easing curve (dy/dx) at the given
    /// time progress (in `[0, 1]`), with the given initial slope.
    fn smooth_scroll_curve_slope(time: f32, slope: f32) -> f32 {
        let s = if time <= 0.0 {
            0.0
        } else if time >= 1.0 {
            1.0
        } else {
            Self::bezier_solve(time)
        };

        let x1 = Self::SMOOTH_SCROLL_BEZIER_X1;
        let x2 = Self::SMOOTH_SCROLL_BEZIER_X2;
        let y1 = x1 * slope;
        let os = 1.0 - s;

        // x'(s) and y'(s), with y2 = 1
        let x_prime =
            3.0 * x1 * os * (1.0 - 3.0 * s) + 3.0 * x2 * s * (2.0 - 3.0 * s) + 3.0 * s * s;
        let y_prime = 3.0 * y1 * os * (1.0 - 3.0 * s) + 3.0 * s * (2.0 - 3.0 * s) + 3.0 * s * s;

        y_prime / x_prime
    }

    /// Solves `x(s) = time` for `s` in `[0, 1]`, for the smooth scrolling
    /// bezier's x-axis.
    ///
    /// The x-axis is strictly increasing for the control points used here, so
    /// bisection converges unconditionally.
    fn bezier_solve(time: f32) -> f32 {
        let x1 = Self::SMOOTH_SCROLL_BEZIER_X1;
        let x2 = Self::SMOOTH_SCROLL_BEZIER_X2;

        let x = |s: f32| {
            let os = 1.0 - s;
            3.0 * x1 * s * os * os + 3.0 * x2 * s * s * os + s * s * s
        };

        let mut low = 0.0;
        let mut high = 1.0;

        for _ in 0..40 {
            let mid = (low + high) / 2.0;

            if x(mid) < time {
                low = mid;
            } else {
                high = mid;
            }
        }

        (low + high) / 2.0
    }

    fn scroll_y_to(&mut self, percentage: f32, bounds: Rectangle, content_bounds: Rectangle) {
        self.cancel();
        self.offset_y = Offset::Relative(percentage.clamp(0.0, 1.0));
        self.unsnap(bounds, content_bounds);
    }

    fn scroll_x_to(&mut self, percentage: f32, bounds: Rectangle, content_bounds: Rectangle) {
        self.cancel();
        self.offset_x = Offset::Relative(percentage.clamp(0.0, 1.0));
        self.unsnap(bounds, content_bounds);
    }

    fn snap_to(&mut self, offset: RelativeOffset<Option<f32>>) {
        self.cancel();

        if let Some(x) = offset.x {
            self.offset_x = Offset::Relative(x.clamp(0.0, 1.0));
        }

        if let Some(y) = offset.y {
            self.offset_y = Offset::Relative(y.clamp(0.0, 1.0));
        }
    }

    fn scroll_to(&mut self, offset: AbsoluteOffset<Option<f32>>) {
        self.cancel();

        if let Some(x) = offset.x {
            self.offset_x = Offset::Absolute(x.max(0.0));
        }

        if let Some(y) = offset.y {
            self.offset_y = Offset::Absolute(y.max(0.0));
        }
    }

    /// Scroll by the provided [`AbsoluteOffset`].
    fn scroll_by(&mut self, offset: AbsoluteOffset, bounds: Rectangle, content_bounds: Rectangle) {
        self.scroll(Vector::new(offset.x, offset.y), bounds, content_bounds);
    }

    /// Unsnaps the current scroll position, if snapped, given the bounds of the
    /// [`Scrollable`] and its contents.
    fn unsnap(&mut self, bounds: Rectangle, content_bounds: Rectangle) {
        self.offset_x =
            Offset::Absolute(self.offset_x.absolute(bounds.width, content_bounds.width));
        self.offset_y =
            Offset::Absolute(self.offset_y.absolute(bounds.height, content_bounds.height));
    }

    /// Returns the scrolling translation of the [`State`], given a [`Direction`],
    /// the bounds of the [`Scrollable`] and its contents.
    fn translation(
        &self,
        direction: Direction,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) -> Vector {
        Vector::new(
            if let Some(horizontal) = direction.horizontal() {
                self.offset_x
                    .translation(bounds.width, content_bounds.width, horizontal.alignment)
            } else {
                0.0
            },
            if let Some(vertical) = direction.vertical() {
                self.offset_y
                    .translation(bounds.height, content_bounds.height, vertical.alignment)
            } else {
                0.0
            },
        )
    }

    fn scrollers_grabbed(&self) -> bool {
        matches!(
            self.interaction,
            Interaction::YScrollerGrabbed(_) | Interaction::XScrollerGrabbed(_),
        )
    }

    pub fn y_scroller_grabbed_at(&self) -> Option<f32> {
        let Interaction::YScrollerGrabbed(at) = self.interaction else {
            return None;
        };

        Some(at)
    }

    pub fn x_scroller_grabbed_at(&self) -> Option<f32> {
        let Interaction::XScrollerGrabbed(at) = self.interaction else {
            return None;
        };

        Some(at)
    }
}

#[derive(Debug)]
/// State of both [`Scrollbar`]s.
struct Scrollbars {
    y: Option<internals::Scrollbar>,
    x: Option<internals::Scrollbar>,
}

impl Scrollbars {
    /// Create y and/or x scrollbar(s) if content is overflowing the [`Scrollable`] bounds.
    fn new(
        translation: Vector,
        direction: Direction,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) -> Self {
        let show_scrollbar_x = direction
            .horizontal()
            .filter(|_scrollbar| content_bounds.width > bounds.width);

        let show_scrollbar_y = direction
            .vertical()
            .filter(|_scrollbar| content_bounds.height > bounds.height);

        let y_scrollbar = if let Some(vertical) = show_scrollbar_y {
            let Scrollbar {
                width,
                margin,
                scroller_width,
                spacing,
                padding,
                ..
            } = *vertical;

            // Adjust the height of the vertical scrollbar if the horizontal scrollbar
            // is present
            let x_scrollbar_height =
                show_scrollbar_x.map_or(0.0, |h| h.width.max(h.scroller_width) + h.margin);

            let total_scrollbar_width = width.max(scroller_width) + 2.0 * margin;

            // The padding is purely visual: it shrinks the top and bottom of the
            // scrollbar without affecting the layout
            let scrollbar_height = (bounds.height - x_scrollbar_height - 2.0 * padding).max(0.0);

            // Total bounds of the scrollbar + margin + scroller width
            let total_scrollbar_bounds = Rectangle {
                x: bounds.x + bounds.width - total_scrollbar_width,
                y: bounds.y + padding,
                width: total_scrollbar_width,
                height: scrollbar_height,
            };

            // Bounds of just the scrollbar
            let scrollbar_bounds = Rectangle {
                x: bounds.x + bounds.width - total_scrollbar_width / 2.0 - width / 2.0,
                y: bounds.y + padding,
                width,
                height: scrollbar_height,
            };

            let ratio = bounds.height / content_bounds.height;

            let scroller = if ratio >= 1.0 {
                None
            } else {
                // min height for easier grabbing with super tall content
                let scroller_height = (scrollbar_bounds.height * ratio).max(2.0);
                let scroller_offset =
                    translation.y * ratio * scrollbar_bounds.height / bounds.height;

                let scroller_bounds = Rectangle {
                    x: bounds.x + bounds.width - total_scrollbar_width / 2.0 - scroller_width / 2.0,
                    y: (scrollbar_bounds.y + scroller_offset).max(0.0),
                    width: scroller_width,
                    height: scroller_height,
                };

                Some(internals::Scroller {
                    bounds: scroller_bounds,
                })
            };

            Some(internals::Scrollbar {
                total_bounds: total_scrollbar_bounds,
                bounds: scrollbar_bounds,
                scroller,
                alignment: vertical.alignment,
                disabled: content_bounds.height <= bounds.height,
                floating: spacing.is_none(),
            })
        } else {
            None
        };

        let x_scrollbar = if let Some(horizontal) = show_scrollbar_x {
            let Scrollbar {
                width,
                margin,
                scroller_width,
                spacing,
                padding,
                ..
            } = *horizontal;

            // Need to adjust the width of the horizontal scrollbar if the vertical scrollbar
            // is present
            let scrollbar_y_width =
                y_scrollbar.map_or(0.0, |scrollbar| scrollbar.total_bounds.width);

            let total_scrollbar_height = width.max(scroller_width) + 2.0 * margin;

            // The padding is purely visual: it shrinks the left and right ends of
            // the scrollbar without affecting the layout
            let scrollbar_width = (bounds.width - scrollbar_y_width - 2.0 * padding).max(0.0);

            // Total bounds of the scrollbar + margin + scroller width
            let total_scrollbar_bounds = Rectangle {
                x: bounds.x + padding,
                y: bounds.y + bounds.height - total_scrollbar_height,
                width: scrollbar_width,
                height: total_scrollbar_height,
            };

            // Bounds of just the scrollbar
            let scrollbar_bounds = Rectangle {
                x: bounds.x + padding,
                y: bounds.y + bounds.height - total_scrollbar_height / 2.0 - width / 2.0,
                width: scrollbar_width,
                height: width,
            };

            let ratio = bounds.width / content_bounds.width;

            let scroller = if ratio >= 1.0 {
                None
            } else {
                // min width for easier grabbing with extra wide content
                let scroller_length = (scrollbar_bounds.width * ratio).max(2.0);
                let scroller_offset = translation.x * ratio * scrollbar_bounds.width / bounds.width;

                let scroller_bounds = Rectangle {
                    x: (scrollbar_bounds.x + scroller_offset).max(0.0),
                    y: bounds.y + bounds.height
                        - total_scrollbar_height / 2.0
                        - scroller_width / 2.0,
                    width: scroller_length,
                    height: scroller_width,
                };

                Some(internals::Scroller {
                    bounds: scroller_bounds,
                })
            };

            Some(internals::Scrollbar {
                total_bounds: total_scrollbar_bounds,
                bounds: scrollbar_bounds,
                scroller,
                alignment: horizontal.alignment,
                disabled: content_bounds.width <= bounds.width,
                floating: spacing.is_none(),
            })
        } else {
            None
        };

        Self {
            y: y_scrollbar,
            x: x_scrollbar,
        }
    }

    fn is_mouse_over(&self, cursor: mouse::Cursor) -> (bool, bool) {
        if let Some(cursor_position) = cursor.position() {
            (
                self.y
                    .as_ref()
                    .map(|scrollbar| scrollbar.is_mouse_over(cursor_position))
                    .unwrap_or(false),
                self.x
                    .as_ref()
                    .map(|scrollbar| scrollbar.is_mouse_over(cursor_position))
                    .unwrap_or(false),
            )
        } else {
            (false, false)
        }
    }

    fn is_y_disabled(&self) -> bool {
        self.y.map(|y| y.disabled).unwrap_or(false)
    }

    fn is_x_disabled(&self) -> bool {
        self.x.map(|x| x.disabled).unwrap_or(false)
    }

    fn grab_y_scroller(&self, cursor_position: Point) -> Option<f32> {
        let scrollbar = self.y?;
        let scroller = scrollbar.scroller?;

        if scrollbar.total_bounds.contains(cursor_position) {
            Some(if scroller.bounds.contains(cursor_position) {
                (cursor_position.y - scroller.bounds.y) / scroller.bounds.height
            } else {
                0.5
            })
        } else {
            None
        }
    }

    fn grab_x_scroller(&self, cursor_position: Point) -> Option<f32> {
        let scrollbar = self.x?;
        let scroller = scrollbar.scroller?;

        if scrollbar.total_bounds.contains(cursor_position) {
            Some(if scroller.bounds.contains(cursor_position) {
                (cursor_position.x - scroller.bounds.x) / scroller.bounds.width
            } else {
                0.5
            })
        } else {
            None
        }
    }

    fn is_any_floating(&self) -> bool {
        self.y.is_some_and(|scrollbar| scrollbar.floating)
            || self.x.is_some_and(|scrollbar| scrollbar.floating)
    }

    fn active(&self) -> bool {
        self.y.is_some() || self.x.is_some()
    }
}

pub(super) mod internals {
    use crate::core::{Point, Rectangle};

    use super::Anchor;

    #[derive(Debug, Copy, Clone)]
    pub struct Scrollbar {
        pub total_bounds: Rectangle,
        pub bounds: Rectangle,
        pub scroller: Option<Scroller>,
        pub alignment: Anchor,
        pub disabled: bool,
        pub floating: bool,
    }

    impl Scrollbar {
        /// Returns whether the mouse is over the scrollbar or not.
        pub fn is_mouse_over(&self, cursor_position: Point) -> bool {
            self.total_bounds.contains(cursor_position)
        }

        /// Returns the y-axis scrolled percentage from the cursor position.
        pub fn scroll_percentage_y(&self, grabbed_at: f32, cursor_position: Point) -> f32 {
            if let Some(scroller) = self.scroller {
                let percentage =
                    (cursor_position.y - self.bounds.y - scroller.bounds.height * grabbed_at)
                        / (self.bounds.height - scroller.bounds.height);

                match self.alignment {
                    Anchor::Start => percentage,
                    Anchor::End => 1.0 - percentage,
                }
            } else {
                0.0
            }
        }

        /// Returns the x-axis scrolled percentage from the cursor position.
        pub fn scroll_percentage_x(&self, grabbed_at: f32, cursor_position: Point) -> f32 {
            if let Some(scroller) = self.scroller {
                let percentage =
                    (cursor_position.x - self.bounds.x - scroller.bounds.width * grabbed_at)
                        / (self.bounds.width - scroller.bounds.width);

                match self.alignment {
                    Anchor::Start => percentage,
                    Anchor::End => 1.0 - percentage,
                }
            } else {
                0.0
            }
        }
    }

    /// The handle of a [`Scrollbar`].
    #[derive(Debug, Clone, Copy)]
    pub struct Scroller {
        /// The bounds of the [`Scroller`].
        pub bounds: Rectangle,
    }
}

/// The possible status of a [`Scrollable`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The [`Scrollable`] can be interacted with.
    Active {
        /// Whether or not the horizontal scrollbar is disabled meaning the content isn't overflowing.
        is_horizontal_scrollbar_disabled: bool,
        /// Whether or not the vertical scrollbar is disabled meaning the content isn't overflowing.
        is_vertical_scrollbar_disabled: bool,
    },
    /// The [`Scrollable`] is being hovered.
    Hovered {
        /// Indicates if the horizontal scrollbar is being hovered.
        is_horizontal_scrollbar_hovered: bool,
        /// Indicates if the vertical scrollbar is being hovered.
        is_vertical_scrollbar_hovered: bool,
        /// Whether or not the horizontal scrollbar is disabled meaning the content isn't overflowing.
        is_horizontal_scrollbar_disabled: bool,
        /// Whether or not the vertical scrollbar is disabled meaning the content isn't overflowing.
        is_vertical_scrollbar_disabled: bool,
    },
    /// The [`Scrollable`] is being dragged.
    Dragged {
        /// Indicates if the horizontal scrollbar is being dragged.
        is_horizontal_scrollbar_dragged: bool,
        /// Indicates if the vertical scrollbar is being dragged.
        is_vertical_scrollbar_dragged: bool,
        /// Whether or not the horizontal scrollbar is disabled meaning the content isn't overflowing.
        is_horizontal_scrollbar_disabled: bool,
        /// Whether or not the vertical scrollbar is disabled meaning the content isn't overflowing.
        is_vertical_scrollbar_disabled: bool,
    },
}

/// The appearance of a scrollable.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The [`container::Style`] of a scrollable.
    pub container: container::Style,
    /// The vertical [`Rail`] appearance.
    pub vertical_rail: Rail,
    /// The horizontal [`Rail`] appearance.
    pub horizontal_rail: Rail,
    /// The [`Background`] of the gap between a horizontal and vertical scrollbar.
    pub gap: Option<Background>,
    /// The appearance of the [`AutoScroll`] overlay.
    pub auto_scroll: AutoScroll,
}

/// The appearance of the scrollbar of a scrollable.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rail {
    /// The [`Background`] of a scrollbar.
    pub background: Option<Background>,
    /// The [`Border`] of a scrollbar.
    pub border: Border,
    /// The appearance of the [`Scroller`] of a scrollbar.
    pub scroller: Scroller,
}

/// The appearance of the scroller of a scrollable.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scroller {
    /// The [`Background`] of the scroller.
    pub background: Background,
    /// The [`Border`] of the scroller.
    pub border: Border,
}

/// The appearance of the autoscroll overlay of a scrollable.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AutoScroll {
    /// The [`Background`] of the [`AutoScroll`] overlay.
    pub background: Background,
    /// The [`Border`] of the [`AutoScroll`] overlay.
    pub border: Border,
    /// Thje [`Shadow`] of the [`AutoScroll`] overlay.
    pub shadow: Shadow,
    /// The [`Color`] for the arrow icons of the [`AutoScroll`] overlay.
    pub icon: Color,
}

/// The theme catalog of a [`Scrollable`].
pub trait Catalog {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}

/// A styling function for a [`Scrollable`].
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

/// The default style of a [`Scrollable`].
pub fn default(theme: &Theme, status: Status) -> Style {
    let palette = theme.palette();

    let scrollbar = Rail {
        background: Some(palette.background.weak.color.into()),
        border: border::rounded(2),
        scroller: Scroller {
            background: palette.background.strongest.color.into(),
            border: border::rounded(2),
        },
    };

    let auto_scroll = AutoScroll {
        background: palette.background.base.color.scale_alpha(0.9).into(),
        border: border::rounded(u32::MAX)
            .width(1)
            .color(palette.background.base.text.scale_alpha(0.8)),
        shadow: Shadow {
            color: Color::BLACK.scale_alpha(0.7),
            offset: Vector::ZERO,
            blur_radius: 2.0,
        },
        icon: palette.background.base.text.scale_alpha(0.8),
    };

    match status {
        Status::Active { .. } => Style {
            container: container::Style::default(),
            vertical_rail: scrollbar,
            horizontal_rail: scrollbar,
            gap: None,
            auto_scroll,
        },
        Status::Hovered {
            is_horizontal_scrollbar_hovered,
            is_vertical_scrollbar_hovered,
            ..
        } => {
            let hovered_scrollbar = Rail {
                scroller: Scroller {
                    background: palette.primary.strong.color.into(),
                    ..scrollbar.scroller
                },
                ..scrollbar
            };

            Style {
                container: container::Style::default(),
                vertical_rail: if is_vertical_scrollbar_hovered {
                    hovered_scrollbar
                } else {
                    scrollbar
                },
                horizontal_rail: if is_horizontal_scrollbar_hovered {
                    hovered_scrollbar
                } else {
                    scrollbar
                },
                gap: None,
                auto_scroll,
            }
        }
        Status::Dragged {
            is_horizontal_scrollbar_dragged,
            is_vertical_scrollbar_dragged,
            ..
        } => {
            let dragged_scrollbar = Rail {
                scroller: Scroller {
                    background: palette.primary.base.color.into(),
                    ..scrollbar.scroller
                },
                ..scrollbar
            };

            Style {
                container: container::Style::default(),
                vertical_rail: if is_vertical_scrollbar_dragged {
                    dragged_scrollbar
                } else {
                    scrollbar
                },
                horizontal_rail: if is_horizontal_scrollbar_dragged {
                    dragged_scrollbar
                } else {
                    scrollbar
                },
                gap: None,
                auto_scroll,
            }
        }
    }
}
