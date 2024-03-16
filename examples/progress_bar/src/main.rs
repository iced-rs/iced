use iced::widget::{column, progress_bar, slider};
use iced::Element;

pub fn main() -> iced::Result {
    iced::run("Progress Bar - Iced", Progress::update, Progress::view)
}

#[derive(Default)]
struct Progress {
    value: f32,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    SliderChanged(f32),
}

impl Progress {
    fn update(&mut self, message: Message) {
        match message {
            Message::SliderChanged(x) => self.value = x,
        }
    }

    fn view(&self) -> Element<Message> {
        column![
            progress_bar(0.0..=100.0, self.value),
            slider(0.0..=100.0, self.value, Message::SliderChanged).step(0.01)
        ]
        .padding(20)
        .into()
    }
}
