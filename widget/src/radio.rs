//! Create choices using radio buttons.
use crate::core::alignment;
use crate::core::event::{self, Event};
use crate::core::layout;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::text;
use crate::core::touch;
use crate::core::widget::{self, Tree};
use crate::core::window;
use crate::core::{
    Alignment, Clipboard, Color, Element, Layout, Length, Pixels, Point,
    Rectangle, Shell, Widget,
};
use crate::{Row, Text};

use iced_style::animation::{AnimationEffect, HoverPressedAnimation};
pub use iced_style::radio::{Appearance, StyleSheet};

/// A circular button representing a choice.
///
/// # Example
/// ```no_run
/// # type Radio<Message> =
/// #     iced_widget::Radio<Message, iced_widget::renderer::Renderer<iced_widget::style::Theme>>;
/// #
/// # use iced_widget::column;
/// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// pub enum Choice {
///     A,
///     B,
///     C,
///     All,
/// }
///
/// #[derive(Debug, Clone, Copy)]
/// pub enum Message {
///     RadioSelected(Choice),
/// }
///
/// let selected_choice = Some(Choice::A);
///
/// let a = Radio::new(
///     "A",
///     Choice::A,
///     selected_choice,
///     Message::RadioSelected,
/// );
///
/// let b = Radio::new(
///     "B",
///     Choice::B,
///     selected_choice,
///     Message::RadioSelected,
/// );
///
/// let c = Radio::new(
///     "C",
///     Choice::C,
///     selected_choice,
///     Message::RadioSelected,
/// );
///
/// let all = Radio::new(
///     "All of the above",
///     Choice::All,
///     selected_choice,
///     Message::RadioSelected
/// );
///
/// let content = column![a, b, c, all];
/// ```
#[allow(missing_debug_implementations)]
pub struct Radio<Message, Renderer = crate::Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    is_selected: bool,
    on_click: Message,
    label: String,
    width: Length,
    size: f32,
    spacing: f32,
    text_size: Option<f32>,
    text_line_height: text::LineHeight,
    text_shaping: text::Shaping,
    font: Option<Renderer::Font>,
    style: <Renderer::Theme as StyleSheet>::Style,
    animation_duration_ms: u16,
    hover_animation_effect: AnimationEffect,
}

impl<Message, Renderer> Radio<Message, Renderer>
where
    Message: Clone,
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// The default size of a [`Radio`] button.
    pub const DEFAULT_SIZE: f32 = 28.0;

    /// The default spacing of a [`Radio`] button.
    pub const DEFAULT_SPACING: f32 = 15.0;

    /// Creates a new [`Radio`] button.
    ///
    /// It expects:
    ///   * the value related to the [`Radio`] button
    ///   * the label of the [`Radio`] button
    ///   * the current selected value
    ///   * a function that will be called when the [`Radio`] is selected. It
    ///   receives the value of the radio and must produce a `Message`.
    pub fn new<F, V>(
        label: impl Into<String>,
        value: V,
        selected: Option<V>,
        f: F,
    ) -> Self
    where
        V: Eq + Copy,
        F: FnOnce(V) -> Message,
    {
        Radio {
            is_selected: Some(value) == selected,
            on_click: f(value),
            label: label.into(),
            width: Length::Shrink,
            size: Self::DEFAULT_SIZE,
            spacing: Self::DEFAULT_SPACING, //15
            text_size: None,
            text_line_height: text::LineHeight::default(),
            text_shaping: text::Shaping::Basic,
            font: None,
            style: Default::default(),
            animation_duration_ms: 250,
            hover_animation_effect: AnimationEffect::Fade,
        }
    }

    /// Sets the size of the [`Radio`] button.
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = size.into().0;
        self
    }

    /// Sets the width of the [`Radio`] button.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the spacing between the [`Radio`] button and the text.
    pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.spacing = spacing.into().0;
        self
    }

    /// Sets the text size of the [`Radio`] button.
    pub fn text_size(mut self, text_size: impl Into<Pixels>) -> Self {
        self.text_size = Some(text_size.into().0);
        self
    }

    /// Sets the text [`LineHeight`] of the [`Radio`] button.
    pub fn text_line_height(
        mut self,
        line_height: impl Into<text::LineHeight>,
    ) -> Self {
        self.text_line_height = line_height.into();
        self
    }

    /// Sets the [`text::Shaping`] strategy of the [`Radio`] button.
    pub fn text_shaping(mut self, shaping: text::Shaping) -> Self {
        self.text_shaping = shaping;
        self
    }

    /// Sets the text font of the [`Radio`] button.
    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    /// Sets the style of the [`Radio`] button.
    pub fn style(
        mut self,
        style: impl Into<<Renderer::Theme as StyleSheet>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }
}

impl<Message, Renderer> Widget<Message, Renderer> for Radio<Message, Renderer>
where
    Message: Clone,
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet + crate::text::StyleSheet,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State::new(
            HoverPressedAnimation::new(self.hover_animation_effect),
            HoverPressedAnimation::new(self.hover_animation_effect),
        ))
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        Row::<(), Renderer>::new()
            .width(self.width)
            .spacing(self.spacing)
            .align_items(Alignment::Center)
            .push(Row::new().width(self.size).height(self.size))
            .push(
                Text::new(&self.label)
                    .width(self.width)
                    .size(
                        self.text_size
                            .unwrap_or_else(|| renderer.default_size()),
                    )
                    .line_height(self.text_line_height)
                    .shaping(self.text_shaping),
            )
            .layout(renderer, limits)
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if layout.bounds().contains(cursor_position) {
                    let state = tree.state.downcast_mut::<State>();

                    state.hovered_animation.reset();
                    state.pressed_animation.on_activate();
                    shell.request_redraw(window::RedrawRequest::NextFrame);

                    shell.publish(self.on_click.clone());

                    return event::Status::Captured;
                }
            }
            Event::Window(window::Event::RedrawRequested(now)) => {
                let state = tree.state.downcast_mut::<State>();

                if self.is_selected && state.pressed_animation.is_running() {
                    if state.pressed_animation.on_redraw_request_update(
                        self.animation_duration_ms,
                        now,
                    ) {
                        shell.request_redraw(window::RedrawRequest::NextFrame);
                    }
                } else if state
                    .hovered_animation
                    .on_redraw_request_update(self.animation_duration_ms, now)
                {
                    shell.request_redraw(window::RedrawRequest::NextFrame);
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                let state = tree.state.downcast_mut::<State>();
                let bounds = layout.bounds();
                let is_mouse_over = bounds.contains(position);

                if !self.is_selected
                    && state
                        .hovered_animation
                        .on_cursor_moved_update(is_mouse_over)
                {
                    shell.request_redraw(window::RedrawRequest::NextFrame);
                }
            }
            _ => {}
        }

        event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        _state: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if layout.bounds().contains(cursor_position) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let style_sheet: &dyn StyleSheet<
            Style = <Renderer::Theme as StyleSheet>::Style,
        > = theme;
        let bounds = layout.bounds();
        let is_mouse_over = bounds.contains(cursor_position);

        let mut children = layout.children();

        let custom_style =
            if is_mouse_over || state.hovered_animation.is_running() {
                if self.is_selected || state.pressed_animation.is_running() {
                    style_sheet.active(
                        &self.style,
                        self.is_selected,
                        &state.pressed_animation,
                    )
                } else {
                    style_sheet.hovered(
                        &self.style,
                        self.is_selected,
                        &state.hovered_animation,
                    )
                }
            } else {
                theme.active(
                    &self.style,
                    self.is_selected,
                    &state.pressed_animation,
                )
            };

        {
            let layout = children.next().unwrap();
            let bounds = layout.bounds();

            let size = bounds.width;
            let dot_size = size / 2.0;

            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border_radius: (size / 2.0).into(),
                    border_width: custom_style.border_width,
                    border_color: custom_style.border_color,
                },
                custom_style.background,
            );

            if self.is_selected {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x + dot_size / 2.0,
                            y: bounds.y + dot_size / 2.0,
                            width: bounds.width - dot_size,
                            height: bounds.height - dot_size,
                        },
                        border_radius: (dot_size / 2.0).into(),
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
                    },
                    custom_style.dot_color,
                );
            }
        }

        {
            let label_layout = children.next().unwrap();

            crate::text::draw(
                renderer,
                style,
                label_layout,
                &self.label,
                self.text_size,
                self.text_line_height,
                self.font,
                crate::text::Appearance {
                    color: custom_style.text_color,
                },
                alignment::Horizontal::Left,
                alignment::Vertical::Center,
                self.text_shaping,
            );
        }
    }
}

/// The local state of a [`Radio`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct State {
    hovered_animation: HoverPressedAnimation,
    pressed_animation: HoverPressedAnimation,
}

impl State {
    /// Creates a new [`State`].
    pub fn new(
        hovered_animation: HoverPressedAnimation,
        pressed_animation: HoverPressedAnimation,
    ) -> State {
        State {
            hovered_animation,
            pressed_animation,
        }
    }
}

impl<'a, Message, Renderer> From<Radio<Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + text::Renderer,
    Renderer::Theme: StyleSheet + crate::text::StyleSheet,
{
    fn from(radio: Radio<Message, Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(radio)
    }
}
