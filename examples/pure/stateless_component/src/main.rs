use iced::pure::{button, text};
use iced::pure::{Element, Sandbox};
use iced::Settings;

use custom_text::custom_text;

pub fn main() -> iced::Result {
    Mode::run(Settings::default())
}

enum Mode {
    Standard,
    Custom,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    SwitchToComponent,
}

impl Sandbox for Mode {
    type Message = Message;

    fn new() -> Self {
        Self::Standard
    }

    fn title(&self) -> String {
        String::from("Stateless Component Bug")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::SwitchToComponent => *self = Mode::Custom,
        }
    }

    fn view(&self) -> Element<Message> {
        match self {
            Self::Standard => button(text("Click to Panic"))
                .on_press(Message::SwitchToComponent)
                .into(),
            Self::Custom => button(custom_text("..")).into(),
        }
    }
}

mod custom_text {
    use iced::pure::text;
    use iced_lazy::pure::{self, Component};
    use iced_native::text;
    use iced_pure::Element;

    pub struct CustomText<'a> {
        text: &'a str,
    }

    pub fn custom_text<'a>(text: &'a str) -> CustomText<'a> {
        CustomText { text }
    }

    #[derive(Debug, Clone)]
    pub enum Event {}

    impl<'a> CustomText<'a> {
        pub fn new(text: &'a str) -> Self {
            Self { text }
        }
    }

    impl<'a, Message, Renderer> Component<Message, Renderer> for CustomText<'a>
    where
        Renderer: text::Renderer + 'static,
    {
        type State = ();
        type Event = Event;

        fn update(
            &mut self,
            _state: &mut Self::State,
            _event: Event,
        ) -> Option<Message> {
            None
        }

        fn view(&self, _state: &Self::State) -> Element<Event, Renderer> {
            text(self.text).into()
        }
    }

    impl<'a, Message, Renderer> From<CustomText<'a>>
        for Element<'a, Message, Renderer>
    where
        Message: 'a,
        Renderer: 'static + text::Renderer,
    {
        fn from(x: CustomText<'a>) -> Self {
            pure::component(x)
        }
    }
}
