use crate::{
    input::{keyboard, mouse, ButtonState},
    layout, Element, Event, Hasher, Layout, Length, Point, Rectangle, Size,
    Widget,
};

pub use iced_core::{text_input::State, TextInput};

impl<'a, Message, Renderer> Widget<Message, Renderer> for TextInput<'a, Message>
where
    Renderer: self::Renderer,
    Message: Clone + std::fmt::Debug,
{
    fn width(&self) -> Length {
        self.width
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let padding = self.padding as f32;
        let text_size = self.size.unwrap_or(renderer.default_size());

        let limits = limits
            .pad(padding)
            .width(self.width)
            .height(Length::Units(text_size));

        let mut text = layout::Node::new(limits.resolve(Size::ZERO));
        text.bounds.x = padding;
        text.bounds.y = padding;

        layout::Node::with_children(text.size().pad(padding), vec![text])
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
        _renderer: &Renderer,
    ) {
        match event {
            Event::Mouse(mouse::Event::Input {
                button: mouse::Button::Left,
                state: ButtonState::Pressed,
            }) => {
                self.state.is_focused =
                    layout.bounds().contains(cursor_position);

                if self.state.cursor_position(&self.value) == 0 {
                    self.state.move_cursor_to_end(&self.value);
                }
            }
            Event::Keyboard(keyboard::Event::CharacterReceived(c))
                if self.state.is_focused && !c.is_control() =>
            {
                let cursor_position = self.state.cursor_position(&self.value);

                self.value.insert(cursor_position, c);
                self.state.move_cursor_right(&self.value);

                let message = (self.on_change)(self.value.to_string());
                messages.push(message);
            }
            Event::Keyboard(keyboard::Event::Input {
                key_code,
                state: ButtonState::Pressed,
            }) if self.state.is_focused => match key_code {
                keyboard::KeyCode::Enter => {
                    if let Some(on_submit) = self.on_submit.clone() {
                        messages.push(on_submit);
                    }
                }
                keyboard::KeyCode::Backspace => {
                    let cursor_position =
                        self.state.cursor_position(&self.value);

                    if cursor_position > 0 {
                        self.state.move_cursor_left(&self.value);

                        let _ = self.value.remove(cursor_position - 1);

                        let message = (self.on_change)(self.value.to_string());
                        messages.push(message);
                    }
                }
                keyboard::KeyCode::Delete => {
                    let cursor_position =
                        self.state.cursor_position(&self.value);

                    if cursor_position < self.value.len() {
                        let _ = self.value.remove(cursor_position);

                        let message = (self.on_change)(self.value.to_string());
                        messages.push(message);
                    }
                }
                keyboard::KeyCode::Left => {
                    self.state.move_cursor_left(&self.value);
                }
                keyboard::KeyCode::Right => {
                    self.state.move_cursor_right(&self.value);
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Renderer::Output {
        let bounds = layout.bounds();
        let text_bounds = layout.children().next().unwrap().bounds();

        renderer.draw(&self, bounds, text_bounds, cursor_position)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::any::TypeId;
        use std::hash::Hash;

        TypeId::of::<TextInput<'static, ()>>().hash(state);

        self.width.hash(state);
        self.max_width.hash(state);
        self.padding.hash(state);
        self.size.hash(state);
    }
}

pub trait Renderer: crate::Renderer + Sized {
    fn default_size(&self) -> u16;

    fn draw<Message>(
        &mut self,
        text_input: &TextInput<'_, Message>,
        bounds: Rectangle,
        text_bounds: Rectangle,
        cursor_position: Point,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<TextInput<'a, Message>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'static + self::Renderer,
    Message: 'static + Clone + std::fmt::Debug,
{
    fn from(
        text_input: TextInput<'a, Message>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(text_input)
    }
}
