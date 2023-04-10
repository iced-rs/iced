//! Allow your users to perform actions by pressing a button.
//!
//! A [`Button`] has some local [`State`].

use crate::core::event::{self, Event};
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::touch;
use crate::core::widget::tree::{self, Tree};
use crate::core::widget::Operation;
use crate::core::{
    Background, Clipboard, Color, Element, Layout, Length, Padding, Point,
    Rectangle, Shell, Vector, Widget,
};
use crate::core::window;

use iced_style::animation::{AnimationDirection, Hover};
pub use iced_style::button::{Appearance, StyleSheet};

/// A generic widget that produces a message when pressed.
///
/// ```no_run
/// # type Button<'a, Message> =
/// #     iced_widget::Button<'a, Message, iced_widget::renderer::Renderer<iced_widget::style::Theme>>;
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
/// # type Button<'a, Message> =
/// #     iced_widget::Button<'a, Message, iced_widget::renderer::Renderer<iced_widget::style::Theme>>;
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
pub struct Button<'a, Message, Renderer = crate::Renderer>
where
    Renderer: crate::core::Renderer,
    Renderer::Theme: StyleSheet,
{
    content: Element<'a, Message, Renderer>,
    on_press: Option<Message>,
    width: Length,
    height: Length,
    padding: Padding,
    style: <Renderer::Theme as StyleSheet>::Style,
    /// Animation duration in milliseconds
    animation_duration_ms: u16,
}

impl<'a, Message, Renderer> Button<'a, Message, Renderer>
where
    Renderer: crate::core::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// Creates a new [`Button`] with the given content.
    pub fn new(content: impl Into<Element<'a, Message, Renderer>>) -> Self {
        Button {
            content: content.into(),
            on_press: None,
            width: Length::Shrink,
            height: Length::Shrink,
            padding: Padding::new(5.0),
            style: <Renderer::Theme as StyleSheet>::Style::default(),
            animation_duration_ms: 150,
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
    pub fn on_press(mut self, msg: Message) -> Self {
        self.on_press = Some(msg);
        self
    }

    /// Sets the style variant of this [`Button`].
    pub fn style(
        mut self,
        style: <Renderer::Theme as StyleSheet>::Style,
    ) -> Self {
        self.style = style;
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Button<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + crate::core::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content))
    }

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout(
            renderer,
            limits,
            self.width,
            self.height,
            self.padding,
            |renderer, limits| {
                self.content.as_widget().layout(renderer, limits)
            },
        )
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<Message>,
    ) {
        operation.container(None, &mut |operation| {
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
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        if let event::Status::Captured = self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event.clone(),
            layout.children().next().unwrap(),
            cursor_position,
            renderer,
            clipboard,
            shell,
        ) {
            return event::Status::Captured;
        }

        update(
            event,
            layout,
            cursor_position,
            shell,
            &self.on_press,
            || tree.state.downcast_mut::<State>(),
            self.animation_duration_ms,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let content_layout = layout.children().next().unwrap();

        let styling = draw(
            renderer,
            bounds,
            self.on_press.is_some(),
            theme,
            &self.style,
            || tree.state.downcast_ref::<State>(),
        );

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            &renderer::Style {
                text_color: styling.text_color,
            },
            content_layout,
            cursor_position,
            &bounds,
        );
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse_interaction(layout, cursor_position, self.on_press.is_some())
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
        )
    }
}

impl<'a, Message, Renderer> From<Button<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: crate::core::Renderer + 'a,
    Renderer::Theme: StyleSheet,
{
    fn from(button: Button<'a, Message, Renderer>) -> Self {
        Self::new(button)
    }
}

/// The local state of a [`Button`].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct State {
    is_pressed: bool,
    is_hovered: Option<Hover>,
}

impl State {
    /// Creates a new [`State`].
    pub fn new() -> State {
        State::default()
    }
}

/// Processes the given [`Event`] and updates the [`State`] of a [`Button`]
/// accordingly.
pub fn update<'a, Message: Clone>(
    event: Event,
    layout: Layout<'_>,
    cursor_position: Point,
    shell: &mut Shell<'_, Message>,
    on_press: &Option<Message>,
    state: impl FnOnce() -> &'a mut State,
    animation_duration_ms: u16,
) -> event::Status {
    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerPressed { .. }) => {
            if on_press.is_some() {
                let bounds = layout.bounds();

                if bounds.contains(cursor_position) {
                    let state = state();

                    state.is_pressed = true;

                    return event::Status::Captured;
                }
            }
        }
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerLifted { .. }) => {
            if let Some(on_press) = on_press.clone() {
                let state = state();

                if state.is_pressed {
                    state.is_pressed = false;

                    let bounds = layout.bounds();

                    if bounds.contains(cursor_position) {
                        shell.publish(on_press);
                    }

                    return event::Status::Captured;
                }
            }
        }
        Event::Touch(touch::Event::FingerLost { .. }) => {
            let state = state();

            state.is_pressed = false;
        }
        Event::Window(window::Event::RedrawRequested(now)) => {
            let state = state();

            if let Some(hover) = &mut state.is_hovered {
                if animation_duration_ms == 0 || hover.animation_progress >= 1.0
                {
                    hover.animation_progress = 1.0;
                }
                if hover.animation_progress == 0.0
                    && hover.direction == AnimationDirection::Backward
                {
                    state.is_hovered = None;
                } else {
                    match hover.direction {
                        AnimationDirection::Forward => {
                            hover.animation_progress = (hover.initial_progress
                                + (((now - hover.started_at).as_millis()
                                    as f64)
                                    / (animation_duration_ms as f64))
                                    as f32)
                                .clamp(0.0, 1.0);
                        }
                        AnimationDirection::Backward => {
                            hover.animation_progress = (hover.initial_progress
                                - (((now - hover.started_at).as_millis()
                                    as f64)
                                    / (animation_duration_ms as f64))
                                    as f32)
                                .clamp(0.0, 1.0);
                        }
                    }
                }
                shell.request_redraw(window::RedrawRequest::NextFrame);
            }
        }
        Event::Mouse(mouse::Event::CursorMoved { position }) => {
            let state = state();
            let bounds = layout.bounds();
            let is_mouse_over = bounds.contains(position);

            if is_mouse_over {
                if let Some(hover) = &mut state.is_hovered {
                    if hover.direction == AnimationDirection::Backward {
                        hover.initial_progress = hover.animation_progress;
                        hover.direction = AnimationDirection::Forward;
                        hover.started_at = std::time::Instant::now();
                    }
                }
                if state.is_hovered.is_none() {
                    state.is_hovered = Some(Hover {
                        direction: AnimationDirection::Forward,
                        started_at: std::time::Instant::now(),
                        animation_progress: 0.0,
                        initial_progress: 0.0,
                    });
                }
                shell.request_redraw(window::RedrawRequest::NextFrame);
            } else if let Some(hover) = &mut state.is_hovered {
                if hover.direction == AnimationDirection::Forward {
                    hover.initial_progress = hover.animation_progress;
                    hover.direction = AnimationDirection::Backward;
                    hover.started_at = std::time::Instant::now();
                    shell.request_redraw(window::RedrawRequest::NextFrame);
                }
            }
        }
        _ => {}
    }

    event::Status::Ignored
}

/// Draws a [`Button`].
pub fn draw<'a, Renderer: crate::core::Renderer>(
    renderer: &mut Renderer,
    bounds: Rectangle,
    is_enabled: bool,
    style_sheet: &dyn StyleSheet<
        Style = <Renderer::Theme as StyleSheet>::Style,
    >,
    style: &<Renderer::Theme as StyleSheet>::Style,
    state: impl FnOnce() -> &'a State,
) -> Appearance
where
    Renderer::Theme: StyleSheet,
{
    let state = state();

    let styling = if !is_enabled {
        style_sheet.disabled(style)
    } else if let Some(hover) = state.is_hovered {
        if state.is_pressed {
            style_sheet.pressed(style)
        } else {
            style_sheet.hovered(style, Some(hover))
        }
    } else {
        style_sheet.active(style)
    };

    if styling.background.is_some() || styling.border_width > 0.0 {
        if styling.shadow_offset != Vector::default() {
            // TODO: Implement proper shadow support
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: bounds.x + styling.shadow_offset.x,
                        y: bounds.y + styling.shadow_offset.y,
                        ..bounds
                    },
                    border_radius: styling.border_radius.into(),
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                },
                Background::Color([0.0, 0.0, 0.0, 0.5].into()),
            );
        }

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border_radius: styling.border_radius.into(),
                border_width: styling.border_width,
                border_color: styling.border_color,
            },
            styling
                .background
                .unwrap_or(Background::Color(Color::TRANSPARENT)),
        );
    }

    styling
}

/// Computes the layout of a [`Button`].
pub fn layout<Renderer>(
    renderer: &Renderer,
    limits: &layout::Limits,
    width: Length,
    height: Length,
    padding: Padding,
    layout_content: impl FnOnce(&Renderer, &layout::Limits) -> layout::Node,
) -> layout::Node {
    let limits = limits.width(width).height(height);

    let mut content = layout_content(renderer, &limits.pad(padding));
    let padding = padding.fit(content.size(), limits.max());
    let size = limits.pad(padding).resolve(content.size()).pad(padding);

    content.move_to(Point::new(padding.left, padding.top));

    layout::Node::with_children(size, vec![content])
}

/// Returns the [`mouse::Interaction`] of a [`Button`].
pub fn mouse_interaction(
    layout: Layout<'_>,
    cursor_position: Point,
    is_enabled: bool,
) -> mouse::Interaction {
    let is_mouse_over = layout.bounds().contains(cursor_position);

    if is_mouse_over && is_enabled {
        mouse::Interaction::Pointer
    } else {
        mouse::Interaction::default()
    }
}
