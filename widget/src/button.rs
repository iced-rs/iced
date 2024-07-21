//! Allow your users to perform actions by pressing a button.
use lilt::{Animated, FloatRepresentable};
use std::time::Instant;

use crate::core::animations::AnimationDuration;
use crate::core::border::{self, Border};
use crate::core::event::{self, Event};
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::theme::palette::{self, deviate};
use crate::core::touch;
use crate::core::widget::tree::{self, Tree};
use crate::core::widget::Operation;
use crate::core::window;
use crate::core::{
    Background, Clipboard, Color, Element, Layout, Length, Padding, Rectangle,
    Shadow, Shell, Size, Theme, Vector, Widget,
};

/// A generic widget that produces a message when pressed.
///
/// ```no_run
/// # type Button<'a, Message> = iced_widget::Button<'a, Message>;
/// #
/// #[derive(Clone)]
/// enum Message {
///     ButtonPressed,
/// }
///
/// let button = Button::new("Press me!").on_press(Message::ButtonPressed);
/// ```
///
/// If a [`Button::on_press`] handler is not set, the resulting [`Button`] will
/// be disabled:
///
/// ```
/// # type Button<'a, Message> = iced_widget::Button<'a, Message>;
/// #
/// #[derive(Clone)]
/// enum Message {
///     ButtonPressed,
/// }
///
/// fn disabled_button<'a>() -> Button<'a, Message> {
///     Button::new("I'm disabled!")
/// }
///
/// fn enabled_button<'a>() -> Button<'a, Message> {
///     disabled_button().on_press(Message::ButtonPressed)
/// }
/// ```
#[allow(missing_debug_implementations)]
pub struct Button<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    Renderer: crate::core::Renderer,
    Theme: Catalog,
{
    content: Element<'a, Message, Theme, Renderer>,
    on_press: Option<OnPress<'a, Message>>,
    width: Length,
    height: Length,
    padding: Padding,
    clip: bool,
    class: Theme::Class<'a>,
    animation_forward_duration: AnimationDuration,
    animation_backward_duration: AnimationDuration,
}

enum OnPress<'a, Message> {
    Direct(Message),
    Closure(Box<dyn Fn() -> Message + 'a>),
}

impl<'a, Message: Clone> OnPress<'a, Message> {
    fn get(&self) -> Message {
        match self {
            OnPress::Direct(message) => message.clone(),
            OnPress::Closure(f) => f(),
        }
    }
}

impl<'a, Message, Theme, Renderer> Button<'a, Message, Theme, Renderer>
where
    Renderer: crate::core::Renderer,
    Theme: Catalog,
{
    /// Creates a new [`Button`] with the given content.
    pub fn new(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        let content = content.into();
        let size = content.as_widget().size_hint();

        Button {
            content,
            on_press: None,
            width: size.width.fluid(),
            height: size.height.fluid(),
            padding: DEFAULT_PADDING,
            clip: false,
            class: Theme::default(),
            animation_forward_duration: AnimationDuration::new(75.),
            animation_backward_duration: AnimationDuration::new(200.),
        }
    }

    /// Sets the width of the [`Button`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Button`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the [`Padding`] of the [`Button`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed.
    ///
    /// Unless `on_press` is called, the [`Button`] will be disabled.
    pub fn on_press(mut self, on_press: Message) -> Self {
        self.on_press = Some(OnPress::Direct(on_press));
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed.
    ///
    /// This is analogous to [`Button::on_press`], but using a closure to produce
    /// the message.
    ///
    /// This closure will only be called when the [`Button`] is actually pressed and,
    /// therefore, this method is useful to reduce overhead if creating the resulting
    /// message is slow.
    pub fn on_press_with(
        mut self,
        on_press: impl Fn() -> Message + 'a,
    ) -> Self {
        self.on_press = Some(OnPress::Closure(Box::new(on_press)));
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed,
    /// if `Some`.
    ///
    /// If `None`, the [`Button`] will be disabled.
    pub fn on_press_maybe(mut self, on_press: Option<Message>) -> Self {
        self.on_press = on_press.map(OnPress::Direct);
        self
    }

    /// Sets whether the contents of the [`Button`] should be clipped on
    /// overflow.
    pub fn clip(mut self, clip: bool) -> Self {
        self.clip = clip;
        self
    }

    /// Sets the style of the [`Button`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`Button`].
    #[cfg(feature = "advanced")]
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    /// Sets the [`AnimationDuration`] forward duration (in milliseconds) of the [`Button`].
    pub fn animation_forward_duration(
        mut self,
        animation_duration_ms: f32,
    ) -> Self {
        self.animation_forward_duration.set(animation_duration_ms);
        self
    }

    /// Sets the [`AnimationDuration`] backward duration (in milliseconds) of the [`Button`].
    pub fn animation_backward_duration(
        mut self,
        animation_duration_ms: f32,
    ) -> Self {
        self.animation_backward_duration.set(animation_duration_ms);
        self
    }

    /// Enables or disables [`AnimationDuration`] transition on [`Pressed`] and [`Hover`] states.
    pub fn set_animations_enabled(mut self, enable: bool) -> Self {
        if enable {
            self.animation_forward_duration.enable();
            self.animation_backward_duration.enable();
        } else {
            self.animation_forward_duration.disable();
            self.animation_backward_duration.disable();
        }
        self
    }
}

#[derive(Debug, Clone)]
struct State {
    is_pressed: bool,
    style_animation: Animated<AnimationTarget, Instant>,
}

impl State {
    /// This check is meant to fix cases when we get a tainted state from another
    /// ['Button'] widget by finding impossible cases.
    fn is_animation_state_tainted(
        &self,
        animate_value: f32,
        is_mouse_over: bool,
    ) -> bool {
        if !is_mouse_over {
            match self.style_animation.value {
                AnimationTarget::Hovered => {
                    animate_value == AnimationTarget::Pressed.float_value()
                }
                AnimationTarget::Pressed => {
                    animate_value == AnimationTarget::Hovered.float_value()
                }
                AnimationTarget::Active => false,
            }
        } else {
            false
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Button<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + crate::core::Renderer,
    Theme: Catalog,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        // The animations start backwards so that it can do its first transition by switching to forward
        let state = State {
            style_animation: Animated::new(AnimationTarget::Active)
                .duration(self.animation_forward_duration.get())
                .asymmetric_duration(self.animation_backward_duration.get())
                .easing(lilt::Easing::Linear),
            is_pressed: false,
        };
        tree::State::new(state)
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
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
        layout::padded(
            limits,
            self.width,
            self.height,
            self.padding,
            |limits| {
                self.content.as_widget().layout(
                    &mut tree.children[0],
                    renderer,
                    limits,
                )
            },
        )
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<()>,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.content.as_widget().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        });
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
        if let event::Status::Captured = self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event.clone(),
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        ) {
            return event::Status::Captured;
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if self.on_press.is_some() {
                    let bounds = layout.bounds();

                    if cursor.is_over(bounds) {
                        let state = tree.state.downcast_mut::<State>();

                        state.is_pressed = true;
                        if self.animation_forward_duration.enabled() {
                            state.style_animation.transition(
                                AnimationTarget::Pressed,
                                Instant::now(),
                            );
                        } else {
                            state.style_animation.transition_instantaneous(
                                AnimationTarget::Pressed,
                                Instant::now(),
                            );
                        }
                        shell.request_redraw(window::RedrawRequest::NextFrame);
                        return event::Status::Captured;
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. }) => {
                if let Some(on_press) = self.on_press.as_ref().map(OnPress::get)
                {
                    let state = tree.state.downcast_mut::<State>();
                    let bounds = layout.bounds();
                    let is_mouse_over = cursor.is_over(bounds);

                    if state.is_pressed {
                        state.is_pressed = false;
                        if is_mouse_over {
                            if self.animation_backward_duration.enabled() {
                                state.style_animation.transition(
                                    AnimationTarget::Hovered,
                                    Instant::now(),
                                );
                            } else {
                                state.style_animation.transition_instantaneous(
                                    AnimationTarget::Hovered,
                                    Instant::now(),
                                );
                            }
                            shell.publish(on_press);
                        } else if self.animation_backward_duration.enabled() {
                            state.style_animation.transition(
                                AnimationTarget::Active,
                                Instant::now(),
                            );
                        } else {
                            state.style_animation.transition_instantaneous(
                                AnimationTarget::Hovered,
                                Instant::now(),
                            );
                        }
                        shell.request_redraw(window::RedrawRequest::NextFrame);

                        return event::Status::Captured;
                    }
                }
            }
            Event::Touch(touch::Event::FingerLost { .. }) => {
                let state = tree.state.downcast_mut::<State>();

                state.is_pressed = false;
            }
            Event::Window(window::Event::RedrawRequested(now)) => {
                let state = tree.state.downcast_mut::<State>();
                let bounds = layout.bounds();
                let is_mouse_over = cursor.is_over(bounds);
                let animated_value = state
                    .style_animation
                    .animate(|target| target.float_value(), now);

                // Reset animation on tainted state
                if state
                    .is_animation_state_tainted(animated_value, is_mouse_over)
                {
                    state
                        .style_animation
                        .transition_instantaneous(AnimationTarget::Active, now);
                }

                if state.style_animation.in_progress(now) {
                    shell.request_redraw(window::RedrawRequest::NextFrame);
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { position: _ }) => {
                let state = tree.state.downcast_mut::<State>();
                let bounds = layout.bounds();
                let is_mouse_over = cursor.is_over(bounds);
                let style_animation = &mut state.style_animation;
                if !state.is_pressed
                    && hover_animation_on_cursor_move(
                        is_mouse_over,
                        style_animation,
                        Instant::now(),
                        &self.animation_forward_duration,
                        &self.animation_backward_duration,
                    )
                {
                    shell.request_redraw(window::RedrawRequest::NextFrame);
                }
            }
            _ => {}
        }

        event::Status::Ignored
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let content_layout = layout.children().next().unwrap();
        let is_mouse_over = cursor.is_over(bounds);

        let status = if self.on_press.is_none() {
            Status::Disabled
        } else {
            let state = tree.state.downcast_ref::<State>();
            if is_mouse_over
                || state.is_pressed
                || state.style_animation.in_progress(Instant::now())
            {
                let now = Instant::now();
                if state.is_pressed {
                    Status::Pressed {
                        animation_progress: state
                            .style_animation
                            .animate(|target| target.float_value(), now),
                    }
                } else {
                    Status::Hovered {
                        animation_progress: state
                            .style_animation
                            .animate(|target| target.float_value(), now),
                    }
                }
            } else {
                Status::Active
            }
        };

        let style = theme.style(&self.class, status);

        if style.background.is_some()
            || style.border.width > 0.0
            || style.shadow.color.a > 0.0
        {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: style.border,
                    shadow: style.shadow,
                },
                style
                    .background
                    .unwrap_or(Background::Color(Color::TRANSPARENT)),
            );
        }

        let viewport = if self.clip {
            bounds.intersection(viewport).unwrap_or(*viewport)
        } else {
            *viewport
        };

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            &renderer::Style {
                text_color: style.text_color,
            },
            content_layout,
            cursor,
            &viewport,
        );
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let is_mouse_over = cursor.is_over(layout.bounds());

        if is_mouse_over && self.on_press.is_some() {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
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
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<Button<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: crate::core::Renderer + 'a,
{
    fn from(button: Button<'a, Message, Theme, Renderer>) -> Self {
        Self::new(button)
    }
}

/// The default [`Padding`] of a [`Button`].
pub(crate) const DEFAULT_PADDING: Padding = Padding {
    top: 5.0,
    bottom: 5.0,
    right: 10.0,
    left: 10.0,
};

/// The possible status of a [`Button`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Status {
    /// The [`Button`] can be pressed.
    Active,
    /// The [`Button`] can be pressed and it is being hovered
    Hovered {
        /// Current progress of the [`AnimationTime`] transition
        animation_progress: f32,
    },
    /// The [`Button`] is being pressed
    Pressed {
        /// Current progress of the [`AnimationTime`] transition
        animation_progress: f32,
    },
    /// The [`Button`] cannot be pressed.
    Disabled,
}

/// The [`AnimationTarget`] represents, through its ['FloatRepresentable`]
/// implementation the ratio of color mixing between the base and hover colors.
#[derive(Debug, Clone)]
enum AnimationTarget {
    Active,
    Hovered,
    Pressed,
}

impl FloatRepresentable for AnimationTarget {
    fn float_value(&self) -> f32 {
        match self {
            AnimationTarget::Active => 0.0,
            AnimationTarget::Hovered => 0.5,
            AnimationTarget::Pressed => 1.0,
        }
    }
}

/// The style of a button.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The [`Background`] of the button.
    pub background: Option<Background>,
    /// The text [`Color`] of the button.
    pub text_color: Color,
    /// The [`Border`] of the buton.
    pub border: Border,
    /// The [`Shadow`] of the butoon.
    pub shadow: Shadow,
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
        }
    }
}

/// The theme catalog of a [`Button`].
pub trait Catalog {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}

/// A styling function for a [`Button`].
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

/// Mix the base with the hover color according to the [`Button`] state.
#[inline(always)]
fn status_style(status: Status, base: palette::Pair) -> Style {
    match status {
        Status::Active => styled(base),
        Status::Pressed {
            animation_progress: progress,
        } => Style {
            background: Some(Background::Color(deviate(
                base.color,
                progress * 0.2,
            ))),
            ..styled(base)
        },
        Status::Hovered {
            animation_progress: progress,
        } => Style {
            background: Some(Background::Color(deviate(
                base.color,
                progress * 0.2,
            ))),
            ..styled(base)
        },
        Status::Disabled => disabled(styled(base)),
    }
}

/// A primary button; denoting a main action.
pub fn primary(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let base = palette.primary.strong;

    status_style(status, base)
}

/// A secondary button; denoting a complementary action.
pub fn secondary(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let base = palette.secondary.base;

    status_style(status, base)
}

/// A success button; denoting a good outcome.
pub fn success(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let base = palette.success.base;

    status_style(status, base)
}

/// A danger button; denoting a destructive action.
pub fn danger(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let base = palette.danger.base;

    status_style(status, base)
}

/// A text button; useful for links.
pub fn text(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();

    let base = Style {
        text_color: palette.background.base.text,
        ..Style::default()
    };

    match status {
        Status::Active => base,
        Status::Pressed {
            animation_progress: progress,
        } => Style {
            text_color: palette
                .background
                .base
                .text
                .scale_alpha(0.8 + 0.2 * progress),
            ..base
        },
        Status::Hovered {
            animation_progress: progress,
        } => Style {
            text_color: palette
                .background
                .base
                .text
                .scale_alpha(1.0 - 0.2 * progress),
            ..base
        },
        Status::Disabled => disabled(base),
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

fn disabled(style: Style) -> Style {
    Style {
        background: style
            .background
            .map(|background| background.scale_alpha(0.5)),
        text_color: style.text_color.scale_alpha(0.5),
        ..style
    }
}

/// Update an animation for a hover effect when the cursor moves
fn hover_animation_on_cursor_move(
    is_mouse_over: bool,
    style_animation: &mut Animated<AnimationTarget, Instant>,
    now: Instant,
    forward_animation_duration: &AnimationDuration,
    backward_animation_duration: &AnimationDuration,
) -> bool {
    if is_mouse_over {
        if !style_animation.in_progress(now) {
            if forward_animation_duration.enabled() {
                style_animation.transition(AnimationTarget::Hovered, now);
            } else {
                style_animation
                    .transition_instantaneous(AnimationTarget::Hovered, now);
            }
        }
        true
    } else {
        // Change the animation's "direction" if it's still going back
        // to the "Active" state
        if matches!(style_animation.value, AnimationTarget::Hovered)
            || matches!(style_animation.value, AnimationTarget::Pressed)
        {
            if backward_animation_duration.enabled() {
                style_animation.transition(AnimationTarget::Active, now);
            } else {
                style_animation
                    .transition_instantaneous(AnimationTarget::Active, now);
            }
            true
        } else {
            false
        }
    }
}
