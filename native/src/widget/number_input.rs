//! Display fields that can only be filled with numeric type.
//!
//! A [`NumberInput`] has some local [`State`].
use crate::{
    column, container,
    event::{self, Event},
    keyboard,
    layout::{Limits, Node},
    mouse, row,
    text_input::{self, cursor, Value},
    Align, Clipboard, Column, Container, Element, Hasher, Layout, Length,
    Point, Rectangle, Row, Size, Text, TextInput, Widget,
};
use num_traits::{Num, NumAssignOps};
use std::fmt::Display;
use std::str::FromStr;

/// A field that can only be filled with numeric type.
///
/// # Example
/// ```
/// # use iced_native::{number_input, renderer::Null};
/// #
/// # pub type NumberInput<'a, Message> = iced_native::NumberInput<'a, Message, Null>;
/// #[derive(Debug, Clone)]
/// enum Message {
///     NumberInputChanged(u32),
/// }
///
/// let mut state = number_input::State::new();
/// let value = 12;
/// let max = 1275;
///
/// let input = NumberInput::new(
///     &mut state,
///     value,
///     max,
///     Message::NumberInputChanged,
/// )
/// .step(2);
/// ```
#[allow(missing_debug_implementations)]
pub struct NumberInput<'a, T, Message, Renderer: self::Renderer> {
    state: &'a mut ModifierState,
    value: T,
    step: T,
    bounds: (T, T),
    padding: u16,
    size: Option<u16>,
    content: TextInput<'a, Message, Renderer>,
    on_change: Box<dyn Fn(T) -> Message>,
    style: <Renderer as self::Renderer>::Style,
    font: Renderer::Font,
}

impl<'a, T, Message, Renderer> NumberInput<'a, T, Message, Renderer>
where
    T: Num + NumAssignOps + PartialOrd + Display + FromStr + Copy,
    Message: Clone,
    Renderer: self::Renderer,
{
    /// Creates a new [`NumberInput`].
    ///
    /// It expects:
    /// - some [`State`]
    /// - the current value
    /// - the max value
    /// - a function that produces a message when the [`NumberInput`] changes
    pub fn new<F>(state: &'a mut State, value: T, max: T, on_changed: F) -> Self
    where
        F: 'static + Fn(T) -> Message + Copy,
        T: 'static,
    {
        let State {
            input_state,
            mod_state,
        } = state;
        let padding = <Renderer as self::Renderer>::DEFAULT_PADDING;
        let convert_to_num = move |s: String| {
            on_changed(T::from_str(&s).unwrap_or(if s.is_empty() {
                T::zero()
            } else {
                value
            }))
        };

        Self {
            state: mod_state,
            value,
            step: T::one(),
            bounds: (T::zero(), max),
            padding,
            size: None,
            content: TextInput::new(
                input_state,
                "",
                format!("{}", value).as_str(),
                convert_to_num,
            )
            .padding(padding)
            .width(Length::Units(127)),
            on_change: Box::new(on_changed),
            style: <Renderer as self::Renderer>::Style::default(),
            font: Default::default(),
        }
    }

    /// Sets the step of the [`NumberInput`].
    pub fn step(mut self, step: T) -> Self {
        self.step = step;
        self
    }

    /// Sets the minimum value of the [`NumberInput`].
    pub fn min(mut self, min: T) -> Self {
        if min < self.bounds.1 {
            self.bounds.0 = min;
        }
        self
    }

    /// Sets the maximum value of the [`NumberInput`].
    pub fn max(mut self, max: T) -> Self {
        if max > self.bounds.0 {
            self.bounds.1 = max;
        }
        self
    }

    /// Sets the minimum & maximum value (bound) of the [`NumberInput`].
    pub fn bounds(mut self, bounds: (T, T)) -> Self {
        if bounds.0 < bounds.1 {
            self.bounds = bounds;
        }
        self
    }

    /// Sets the [ `Font`] of the [`Text`].
    ///
    /// [`Font`]: crate::widget::text::Renderer::Font
    /// [`Text`]: crate::widget::Text
    pub fn font(mut self, font: Renderer::Font) -> Self {
        self.font = font;
        self.content = self.content.font(font);
        self
    }

    /// Sets the width of the [`NumberInput`].
    pub fn width(mut self, width: Length) -> Self {
        self.content = self.content.width(width);
        self
    }

    /// Sets the maximum width of the [`NumberInput`].
    pub fn max_width(mut self, max_width: u32) -> Self {
        self.content = self.content.max_width(max_width);
        self
    }

    /// Sets the padding of the [`NumberInput`].
    pub fn padding(mut self, units: u16) -> Self {
        self.padding = units;
        self.content = self.content.padding(units);
        self
    }

    /// Sets the text size of the [`NumberInput`].
    pub fn size(mut self, size: u16) -> Self {
        self.size = Some(size);
        self.content = self.content.size(size);
        self
    }

    /// Sets the message that should be produced when the [`NumberInput`] is
    /// focused and the enter key is pressed.
    pub fn on_submit(mut self, message: Message) -> Self {
        self.content = self.content.on_submit(message);
        self
    }

    /// Sets the style of the [`NumberInput`].
    pub fn style(
        mut self,
        style: impl Into<<Renderer as self::Renderer>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }

    /// Sets the input style of the [`NumberInput`].
    pub fn input_style(
        mut self,
        style: impl Into<<Renderer as text_input::Renderer>::Style>,
    ) -> Self {
        self.content = self.content.style(style.into());
        self
    }

    fn decrease_val(&mut self, messages: &mut Vec<Message>) {
        if self.value > self.bounds.0 {
            let new_val = self.value - self.step;
            self.value = if new_val > self.bounds.0 {
                new_val
            } else {
                self.bounds.0
            };
            messages.push((self.on_change)(self.value));
        }
    }

    fn increase_val(&mut self, messages: &mut Vec<Message>) {
        if self.value < self.bounds.1 {
            let new_val = self.value + self.step;
            self.value = if new_val < self.bounds.1 {
                new_val
            } else {
                self.bounds.1
            };
            messages.push((self.on_change)(self.value));
        }
    }
}

impl<'a, T, Message, Renderer> Widget<Message, Renderer>
    for NumberInput<'a, T, Message, Renderer>
where
    T: Num + NumAssignOps + PartialOrd + Display + FromStr + ToString + Copy,
    Message: Clone,
    Renderer:
        self::Renderer + container::Renderer + column::Renderer + row::Renderer,
{
    fn width(&self) -> Length {
        Widget::<Message, Renderer>::width(&self.content)
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(&self, renderer: &Renderer, limits: &Limits) -> Node {
        let padding = f32::from(self.padding);
        let limits = limits
            .width(self.width())
            .height(Length::Shrink)
            .pad(padding);
        let content = self.content.layout(renderer, &limits.loose());
        let txt_size = self.size.unwrap_or(renderer.default_size());
        let icon_size = txt_size * 3 / 4;
        let btn_mod = |c| {
            Container::<(), Renderer>::new(
                Text::new(format!(" {} ", c)).size(icon_size),
            )
            .center_y()
            .center_x()
        };
        let mut modifier = if self.padding < Renderer::DEFAULT_PADDING {
            Row::<(), Renderer>::new()
                .spacing(1)
                .width(Length::Shrink)
                .push(btn_mod('+'))
                .push(btn_mod('-'))
                .layout(renderer, &limits.loose())
        } else {
            Column::<(), Renderer>::new()
                .spacing(1)
                .width(Length::Shrink)
                .push(btn_mod('▲'))
                .push(btn_mod('▼'))
                .layout(renderer, &limits.loose())
        };
        let intrinsic = Size::new(
            content.size().width - 3.0,
            content.size().height.max(modifier.size().height),
        );
        modifier.align(Align::End, Align::Center, intrinsic);
        let size = limits.resolve(intrinsic);
        Node::with_children(size, vec![content, modifier])
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) -> Renderer::Output {
        let bounds = layout.bounds();
        let mut children = layout.children();
        let content_layout = children.next().unwrap();
        let mut mod_children = children.next().unwrap().children();
        let inc_bounds = mod_children.next().unwrap().bounds();
        let dec_bounds = mod_children.next().unwrap().bounds();
        let is_mouse_over = bounds.contains(cursor_position);
        let content =
            self.content
                .draw(renderer, content_layout, cursor_position, None);
        let is_decrease_disabled = self.value <= self.bounds.0;
        let is_increase_disabled = self.value >= self.bounds.1;

        self::Renderer::draw(
            renderer,
            cursor_position,
            &self.state,
            inc_bounds,
            dec_bounds,
            is_mouse_over,
            is_decrease_disabled,
            is_increase_disabled,
            content,
            &self.style,
            self.font,
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash;
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.padding.hash(state);
        self.size.hash(state);
        self.content.hash_layout(state);
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
        renderer: &Renderer,
        clipboard: Option<&dyn Clipboard>,
    ) -> event::Status {
        let mut children = layout.children();
        let content = children.next().unwrap();
        let mut mod_children = children.next().unwrap().children();
        let inc_bounds = mod_children.next().unwrap().bounds();
        let dec_bounds = mod_children.next().unwrap().bounds();
        let mouse_over_inc = inc_bounds.contains(cursor_position);
        let mouse_over_dec = dec_bounds.contains(cursor_position);

        if layout.bounds().contains(cursor_position) {
            if mouse_over_inc || mouse_over_dec {
                let mut event_status = event::Status::Captured;
                match event {
                    Event::Mouse(mouse::Event::ButtonPressed(
                        mouse::Button::Left,
                    )) => {
                        if mouse_over_dec {
                            self.state.decrease_pressed = true;
                            self.decrease_val(messages);
                        } else if mouse_over_inc {
                            self.state.increase_pressed = true;
                            self.increase_val(messages);
                        } else {
                            event_status = event::Status::Ignored;
                        }
                    }
                    Event::Mouse(mouse::Event::ButtonReleased(
                        mouse::Button::Left,
                    )) => {
                        if mouse_over_dec {
                            self.state.decrease_pressed = false;
                        } else if mouse_over_inc {
                            self.state.increase_pressed = false;
                        } else {
                            event_status = event::Status::Ignored;
                        }
                    }
                    _ => event_status = event::Status::Ignored,
                }
                event_status
            } else {
                match event {
                    Event::Keyboard(keyboard::Event::CharacterReceived(c))
                        if self.content.state().is_focused()
                            && c.is_numeric() =>
                    {
                        let mut new_val = self.value.to_string();
                        match self
                            .content
                            .state()
                            .cursor()
                            .state(&Value::new(&new_val))
                        {
                            cursor::State::Index(idx) => new_val.insert(idx, c),
                            cursor::State::Selection { start, end } => new_val
                                .replace_range(start..end, &c.to_string()),
                        }

                        match T::from_str(&new_val) {
                            Ok(val) => {
                                if (self.bounds.0..=self.bounds.1)
                                    .contains(&val)
                                {
                                    self.content.on_event(
                                        event.clone(),
                                        content,
                                        cursor_position,
                                        messages,
                                        renderer,
                                        clipboard,
                                    )
                                } else {
                                    event::Status::Ignored
                                }
                            }
                            Err(_) => event::Status::Ignored,
                        }
                    }
                    Event::Keyboard(keyboard::Event::KeyPressed {
                        key_code,
                        ..
                    }) if self.content.state().is_focused() => {
                        match key_code {
                            keyboard::KeyCode::Up => {
                                self.increase_val(messages);
                                // self.content.state().move_cursor_to_end();
                                event::Status::Captured
                            }
                            keyboard::KeyCode::Down => {
                                self.decrease_val(messages);
                                // self.content.state().move_cursor_to_end();
                                event::Status::Captured
                            }
                            _ => self.content.on_event(
                                event.clone(),
                                content,
                                cursor_position,
                                messages,
                                renderer,
                                clipboard,
                            ),
                        }
                    }
                    _ => self.content.on_event(
                        event.clone(),
                        content,
                        cursor_position,
                        messages,
                        renderer,
                        clipboard,
                    ),
                }
            }
        } else {
            event::Status::Ignored
        }
    }
}

/// The state of a [`NumberInput`].
#[derive(Default, Clone, Debug)]
pub struct State {
    input_state: text_input::State,
    mod_state: ModifierState,
}

impl State {
    /// Creates a new [`State`], representing an unfocused [`NumberInput`].
    pub fn new() -> Self {
        Self::default()
    }
}

/// The modifier state of a [`NumberInput`].
#[derive(Default, Clone, Debug)]
pub struct ModifierState {
    /// The state of decrease button on a [`NumberInput`].
    pub decrease_pressed: bool,
    /// The state of increase button on a [`NumberInput`].
    pub increase_pressed: bool,
}

/// The renderer of a [`NumberInput`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`NumberInput`] in your user interface.
///
/// [renderer]: crate::renderer
pub trait Renderer: text_input::Renderer {
    /// The style supported by this renderer.
    type Style: Default;

    /// The default padding of a [`NumberInput`].
    const DEFAULT_PADDING: u16;

    /// Draws a [`NumberInput`].
    fn draw(
        &mut self,
        cursor_position: Point,
        state: &ModifierState,
        inc_bounds: Rectangle,
        dec_bounds: Rectangle,
        is_mouse_over: bool,
        is_decrease_disabled: bool,
        is_increase_disabled: bool,
        content: Self::Output,
        style: &<Self as self::Renderer>::Style,
        font: Self::Font,
    ) -> Self::Output;
}

impl<'a, T, Message, Renderer> From<NumberInput<'a, T, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    T: 'a + Num + NumAssignOps + PartialOrd + Display + FromStr + Copy,
    Message: 'a + Clone,
    Renderer: 'a
        + self::Renderer
        + container::Renderer
        + column::Renderer
        + row::Renderer,
{
    fn from(num_input: NumberInput<'a, T, Message, Renderer>) -> Self {
        Element::new(num_input)
    }
}
