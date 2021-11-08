use iced::{Container, Element, Length, Sandbox, Settings};
use numeric_input::NumericInput;

pub fn main() -> iced::Result {
    Component::run(Settings::default())
}

#[derive(Default)]
struct Component {
    numeric_input: numeric_input::State,
    value: Option<u32>,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    NumericInputChanged(Option<u32>),
}

impl Sandbox for Component {
    type Message = Message;

    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        String::from("Component - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::NumericInputChanged(value) => {
                self.value = value;
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        Container::new(NumericInput::new(
            &mut self.numeric_input,
            self.value,
            Message::NumericInputChanged,
        ))
        .padding(20)
        .height(Length::Fill)
        .center_y()
        .into()
    }
}

mod numeric_input {
    use iced_lazy::component::{self, Component};
    use iced_native::alignment::{self, Alignment};
    use iced_native::text;
    use iced_native::widget::button::{self, Button};
    use iced_native::widget::text_input::{self, TextInput};
    use iced_native::widget::{Row, Text};
    use iced_native::{Element, Length};

    pub struct NumericInput<'a, Message> {
        state: &'a mut State,
        value: Option<u32>,
        on_change: Box<dyn Fn(Option<u32>) -> Message>,
    }

    #[derive(Default)]
    pub struct State {
        input: text_input::State,
        decrement_button: button::State,
        increment_button: button::State,
    }

    #[derive(Debug, Clone)]
    pub enum Event {
        InputChanged(String),
        IncrementPressed,
        DecrementPressed,
    }

    impl<'a, Message> NumericInput<'a, Message> {
        pub fn new(
            state: &'a mut State,
            value: Option<u32>,
            on_change: impl Fn(Option<u32>) -> Message + 'static,
        ) -> Self {
            Self {
                state,
                value,
                on_change: Box::new(on_change),
            }
        }
    }

    impl<'a, Message, Renderer> Component<Message, Renderer>
        for NumericInput<'a, Message>
    where
        Renderer: 'a + text::Renderer,
    {
        type Event = Event;

        fn update(&mut self, event: Event) -> Option<Message> {
            match event {
                Event::IncrementPressed => Some((self.on_change)(Some(
                    self.value.unwrap_or_default().saturating_add(1),
                ))),
                Event::DecrementPressed => Some((self.on_change)(Some(
                    self.value.unwrap_or_default().saturating_sub(1),
                ))),
                Event::InputChanged(value) => {
                    if value.is_empty() {
                        Some((self.on_change)(None))
                    } else {
                        value
                            .parse()
                            .ok()
                            .map(Some)
                            .map(self.on_change.as_ref())
                    }
                }
            }
        }

        fn view(&mut self) -> Element<Event, Renderer> {
            let button = |state, label, on_press| {
                Button::new(
                    state,
                    Text::new(label)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .horizontal_alignment(alignment::Horizontal::Center)
                        .vertical_alignment(alignment::Vertical::Center),
                )
                .width(Length::Units(50))
                .on_press(on_press)
            };

            Row::with_children(vec![
                button(
                    &mut self.state.decrement_button,
                    "-",
                    Event::DecrementPressed,
                )
                .into(),
                TextInput::new(
                    &mut self.state.input,
                    "Type a number",
                    self.value
                        .as_ref()
                        .map(u32::to_string)
                        .as_ref()
                        .map(String::as_str)
                        .unwrap_or(""),
                    Event::InputChanged,
                )
                .padding(10)
                .into(),
                button(
                    &mut self.state.increment_button,
                    "+",
                    Event::IncrementPressed,
                )
                .into(),
            ])
            .align_items(Alignment::Fill)
            .spacing(10)
            .into()
        }
    }

    impl<'a, Message, Renderer> From<NumericInput<'a, Message>>
        for Element<'a, Message, Renderer>
    where
        Message: 'a,
        Renderer: text::Renderer + 'a,
    {
        fn from(numeric_input: NumericInput<'a, Message>) -> Self {
            component::view(numeric_input)
        }
    }
}
