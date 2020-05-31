use iced::{
    text_input, Application, Column, Command, Container, Element,
    HorizontalAlignment, Length, Settings, Text, TextInput,
};

struct State {
    message1: String,
    message2: String,
    message3: String,

    // Local Element States
    left_input_state: text_input::State,
    center_input_state: text_input::State,
    right_input_state: text_input::State,
}

#[derive(Debug, Clone)]
enum Event {
    OnInputLeftChange(String),
    OnInputCenterChange(String),
    OnInputRightChange(String),
}

impl Application for State {
    type Executor = iced::executor::Default;
    type Message = Event;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Event>) {
        (
            State {
                message1: "".to_owned(),
                message2: "".to_owned(),
                message3: "".to_owned(),
                left_input_state: text_input::State::focused(),
                center_input_state: text_input::State::focused(),
                right_input_state: text_input::State::focused(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Hello World Example".to_owned()
    }

    fn update(&mut self, event: Event) -> Command<Event> {
        match event {
            Event::OnInputLeftChange(message) => {
                self.message1 = message;
            }
            Event::OnInputCenterChange(message) => {
                self.message2 = message;
            }
            Event::OnInputRightChange(message) => {
                self.message3 = message;
            }
        };

        Command::none()
    }

    fn view(&mut self) -> Element<Event> {
        let input_left = TextInput::new(
            &mut self.left_input_state,
            "Type a Message",
            &self.message1,
            Event::OnInputLeftChange,
        )
        .padding(15)
        .size(30)
        .horizontal_alignment(HorizontalAlignment::Left);

        let input_center = TextInput::new(
            &mut self.center_input_state,
            "Type a Message",
            &self.message2,
            Event::OnInputCenterChange,
        )
        .padding(15)
        .size(30)
        .horizontal_alignment(HorizontalAlignment::Center);

        let input_right = TextInput::new(
            &mut self.right_input_state,
            "Type a Message",
            &self.message3,
            Event::OnInputRightChange,
        )
        .padding(15)
        .size(30)
        .horizontal_alignment(HorizontalAlignment::Right);

        let text1 = Text::new(self.message1.to_string())
            .width(Length::Fill)
            .size(16)
            .color([0.5, 0.5, 0.5])
            .horizontal_alignment(HorizontalAlignment::Center);

        let text2 = Text::new(self.message2.to_string())
            .width(Length::Fill)
            .size(16)
            .color([0.5, 0.5, 0.5])
            .horizontal_alignment(HorizontalAlignment::Center);
        let text3 = Text::new(self.message3.to_string())
            .width(Length::Fill)
            .size(16)
            .color([0.5, 0.5, 0.5])
            .horizontal_alignment(HorizontalAlignment::Center);

        let column = Column::new()
            .push(input_left)
            .push(text1)
            .push(input_center)
            .push(text2)
            .push(input_right)
            .push(text3);

        Container::new(column)
            .padding(100)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn main() {
    log4rs::init_file(
        "examples/text_input_aligned/log4rs.yml",
        Default::default(),
    )
    .unwrap();

    State::run(Settings::default());
}
