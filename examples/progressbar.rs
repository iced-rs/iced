use iced::{slider, Column, Element, Sandbox, Settings, Slider};
use iced_winit::Progressbar;

pub fn main() {
    Progress::run(Settings::default())
}

#[derive(Default)]
struct Progress {
    value: f32,
    progressbar_slider: slider::State,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    SliderChanged(f32),
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
            Message::SliderChanged(x) => {
                self.value = x;
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        Column::new()
            .padding(20)
            .push(Progressbar::new(0.0..=100.0, self.value))
            .padding(20)
            .push(Slider::new(
                &mut self.progressbar_slider,
                0.0..=100.0,
                self.value,
                Message::SliderChanged,
            ))
            .into()
    }
}
