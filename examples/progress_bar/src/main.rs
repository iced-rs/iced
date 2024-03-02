use iced::widget::{checkbox, column, progress_bar, slider};
use iced::{Element, Sandbox, Settings};

pub fn main() -> iced::Result {
    Progress::run(Settings::default())
}

#[derive(Default)]
struct Progress {
    value: f32,
    buffer: f32,
    reverse: bool,
    vertical: bool,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    ValueChanged(f32),
    BufferChanged(f32),
    Reverse(bool),
    Vertical(bool),
}

impl Sandbox for Progress {
    type Message = Message;

    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        String::from("A simple Progressbar")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ValueChanged(x) => self.value = x,
            Message::BufferChanged(x) => self.buffer = x,
            Message::Reverse(f) => self.reverse = f,
            Message::Vertical(f) => self.vertical = f,
        }
    }

    fn view(&self) -> Element<Message> {
        column![
            checkbox("Vertical", self.vertical).on_toggle(Message::Vertical),
            checkbox("Reverse", self.reverse).on_toggle(Message::Reverse),
            progress_bar(0.0..=100.0, self.value)
                .buffer(self.buffer)
                .vertical_f(self.vertical)
                .reverse_f(self.reverse),
            slider(0.0..=100.0, self.value, Message::ValueChanged).step(0.01),
            slider(0.0..=100.0, self.buffer, Message::BufferChanged).step(0.01),
        ]
        .padding(20)
        .spacing(8)
        .into()
    }
}
