use iced::pure::container;
use iced::pure::{Element, Sandbox};
use iced::{Length, Settings};

use counter::counter;

pub fn main() -> iced::Result {
    Component::run(Settings::default())
}

#[derive(Default)]
struct Component;

#[derive(Debug, Clone, Copy)]
enum Message {}

impl Sandbox for Component {
    type Message = Message;

    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        String::from("Component - Iced")
    }

    fn update(&mut self, _message: Message) {}

    fn view(&self) -> Element<Message> {
        container(counter())
            .padding(20)
            .height(Length::Fill)
            .center_y()
            .into()
    }
}

mod counter {
    use iced::pure::{column, text};
    use iced_lazy::pure::{self, Component};
    use iced_native::text;
    use iced_pure::Element;

    use custom_button::custom_button;

    pub struct Counter;

    pub fn counter() -> Counter {
        Counter::new()
    }

    #[derive(Debug, Clone)]
    pub enum Event {
        IncrementPressed,
    }

    impl Counter {
        pub fn new() -> Self {
            Self {}
        }
    }

    #[derive(Default)]
    pub struct State {
        count: u32,
    }

    impl<Message, Renderer> Component<Message, Renderer> for Counter
    where
        Renderer: text::Renderer + 'static,
    {
        type State = State;
        type Event = Event;

        fn update(
            &mut self,
            state: &mut Self::State,
            event: Event,
        ) -> Option<Message> {
            match event {
                Event::IncrementPressed => {
                    state.count += 1;

                    None
                }
            }
        }

        fn view(&self, state: &Self::State) -> Element<Event, Renderer> {
            column()
                .push(text(&state.count.to_string()))
                .push(custom_button("Increment", Event::IncrementPressed))
                .into()
        }
    }

    impl<'a, Message, Renderer> From<Counter> for Element<'a, Message, Renderer>
    where
        Message: 'a,
        Renderer: 'static + text::Renderer,
    {
        fn from(counter: Counter) -> Self {
            pure::component(counter)
        }
    }

    mod custom_button {
        use iced::pure::{button, text};
        use iced_lazy::pure::{self, Component};
        use iced_native::text;
        use iced_pure::Element;

        pub struct CustomButton<'a, Message> {
            text: &'a str,
            on_press: Message,
        }

        pub fn custom_button<'a, Message>(
            text: &'a str,
            on_press: Message,
        ) -> CustomButton<'a, Message> {
            CustomButton { text, on_press }
        }

        #[derive(Clone)]
        pub enum Event {
            Press,
        }

        impl<'a, Message, Renderer> Component<Message, Renderer>
            for CustomButton<'a, Message>
        where
            Message: Clone,
            Renderer: iced_native::Renderer + text::Renderer + 'static,
        {
            type State = ();
            type Event = Event;

            fn update(
                &mut self,
                _state: &mut Self::State,
                event: Event,
            ) -> Option<Message> {
                match event {
                    Event::Press => Some(self.on_press.clone()),
                }
            }

            fn view(&self, _state: &Self::State) -> Element<Event, Renderer> {
                button(text(self.text)).on_press(Event::Press).into()
            }
        }

        impl<'a, Message, Renderer> From<CustomButton<'a, Message>>
            for Element<'a, Message, Renderer>
        where
            Message: Clone + 'a,
            Renderer: 'static + text::Renderer,
        {
            fn from(custom_button: CustomButton<'a, Message>) -> Self {
                pure::component(custom_button)
            }
        }
    }
}
