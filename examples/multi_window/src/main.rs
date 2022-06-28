use iced::multi_window::Application;
use iced::pure::{button, column, text, Element};
use iced::{window, Alignment, Command, Settings};

pub fn main() -> iced::Result {
    Counter::run(Settings::default())
}

struct Counter {
    value: i32,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    IncrementPressed,
    DecrementPressed,
}

impl Application for Counter {
    type Flags = ();
    type Executor = iced::executor::Default;
    type Message = Message;

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (Self { value: 0 }, Command::none())
    }

    fn title(&self) -> String {
        String::from("MultiWindow - Iced")
    }

    fn windows(&self) -> Vec<(window::Id, iced::window::Settings)> {
        todo!()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::IncrementPressed => {
                self.value += 1;
            }
            Message::DecrementPressed => {
                self.value -= 1;
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        column()
            .padding(20)
            .align_items(Alignment::Center)
            .push(button("Increment").on_press(Message::IncrementPressed))
            .push(text(self.value.to_string()).size(50))
            .push(button("Decrement").on_press(Message::DecrementPressed))
            .into()
    }
}
