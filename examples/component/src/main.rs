use iced::widget::center;
use iced::Element;

use numeric_input::numeric_input;

pub fn main() -> iced::Result {
    iced::run("Component - Iced", Component::update, Component::view)
}

#[derive(Default)]
struct Component {
    value: Option<u32>,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    NumericInputChanged(Option<u32>),
}

impl Component {
    fn update(&mut self, message: Message) {
        match message {
            Message::NumericInputChanged(value) => {
                self.value = value;
            }
        }
    }

    fn view(&self) -> Element<Message> {
        center(numeric_input(self.value, Message::NumericInputChanged))
            .padding(20)
            .into()
    }
}

mod numeric_input {
    use iced::widget::{button, component, row, text, text_input, Component};
    use iced::{Center, Element, Fill, Length, Size};

    pub struct NumericInput<Message> {
        value: Option<u32>,
        on_change: Box<dyn Fn(Option<u32>) -> Message>,
    }

    pub fn numeric_input<Message>(
        value: Option<u32>,
        on_change: impl Fn(Option<u32>) -> Message + 'static,
    ) -> NumericInput<Message> {
        NumericInput::new(value, on_change)
    }

    #[derive(Debug, Clone)]
    pub enum Event {
        InputChanged(String),
        IncrementPressed,
        DecrementPressed,
    }

    impl<Message> NumericInput<Message> {
        pub fn new(
            value: Option<u32>,
            on_change: impl Fn(Option<u32>) -> Message + 'static,
        ) -> Self {
            Self {
                value,
                on_change: Box::new(on_change),
            }
        }
    }

    impl<Message, Theme> Component<Message, Theme> for NumericInput<Message>
    where
        Theme: text::Catalog + button::Catalog + text_input::Catalog + 'static,
    {
        type State = ();
        type Event = Event;

        fn update(
            &mut self,
            _state: &mut Self::State,
            event: Event,
        ) -> Option<Message> {
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

        fn view(&self, _state: &Self::State) -> Element<'_, Event, Theme> {
            let button = |label, on_press| {
                button(text(label).width(Fill).height(Fill).center())
                    .width(40)
                    .height(40)
                    .on_press(on_press)
            };

            row![
                button("-", Event::DecrementPressed),
                text_input(
                    "Type a number",
                    self.value
                        .as_ref()
                        .map(u32::to_string)
                        .as_deref()
                        .unwrap_or(""),
                )
                .on_input(Event::InputChanged)
                .padding(10),
                button("+", Event::IncrementPressed),
            ]
            .align_y(Center)
            .spacing(10)
            .into()
        }

        fn size_hint(&self) -> Size<Length> {
            Size {
                width: Length::Fill,
                height: Length::Shrink,
            }
        }
    }

    impl<'a, Message, Theme> From<NumericInput<Message>>
        for Element<'a, Message, Theme>
    where
        Theme: text::Catalog + button::Catalog + text_input::Catalog + 'static,
        Message: 'a,
    {
        fn from(numeric_input: NumericInput<Message>) -> Self {
            component(numeric_input)
        }
    }
}
