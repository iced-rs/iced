use iced_winit::{button, Align, Button, Column, Element, Application, Settings, Text, Renderer, Program, Command};

pub fn main() {
    Counter::run(Settings::default()).unwrap()
}

#[derive(Default)]
struct Counter {
    value: i32,
    increment_button: button::State,
    decrement_button: button::State,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    IncrementPressed,
    DecrementPressed,
}

impl Application for Counter {
    type Flags = ();

    fn new(flags: Self::Flags) -> (Self, Command<Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Counter with winit - Iced")
    }
}

impl Program for Counter {
    type Renderer = Renderer;
    type Message = Message;

    fn update(&mut self, message: Message) {
        match message {
            Message::IncrementPressed => {
                self.value += 1;
            }
            Message::DecrementPressed => {
                self.value -= 1;
            }
        }
    }

    fn view(&mut self) -> Element<Message, Self::Renderer> {
        Column::new()
            .padding(20)
            .align_items(Align::Center)
            .push(
                Button::new(&mut self.increment_button, Text::new("Increment"))
                    .on_press(Message::IncrementPressed),
            )
            .push(Text::new(self.value.to_string()).size(50))
            .push(
                Button::new(&mut self.decrement_button, Text::new("Decrement"))
                    .on_press(Message::DecrementPressed),
            )
            .into()
    }
}
