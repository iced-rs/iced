use iced::{
    text::HorizontalAlignment, text_input, Align, Application, Color, Column,
    Element, Justify, Length, Text, TextInput,
};

pub fn main() {
    Todos::default().run()
}

#[derive(Default)]
struct Todos {
    input: text_input::State,
    input_value: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    CreateTask,
}

impl Application for Todos {
    type Message = Message;

    fn update(&mut self, message: Message) {
        match message {
            Message::InputChanged(value) => {
                self.input_value = value;
            }
            Message::CreateTask => {}
        }
    }

    fn view(&mut self) -> Element<Message> {
        let title = Text::new("todos")
            .size(100)
            .color(GRAY)
            .horizontal_alignment(HorizontalAlignment::Center);

        let input = TextInput::new(
            &mut self.input,
            "What needs to be done?",
            &self.input_value,
            Message::InputChanged,
        )
        .padding(15)
        .size(30)
        .on_submit(Message::CreateTask);

        Column::new()
            .max_width(Length::Units(800))
            .height(Length::Fill)
            .align_self(Align::Center)
            .justify_content(Justify::Center)
            .spacing(20)
            .padding(20)
            .push(title)
            .push(input)
            .into()
    }
}

// Colors
const GRAY: Color = Color {
    r: 0.5,
    g: 0.5,
    b: 0.5,
    a: 1.0,
};
