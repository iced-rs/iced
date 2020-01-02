use iced::{
    settings::Window, slider, Background, Color, Column, Element, Sandbox,
    Settings, Slider,
};
use iced_winit::Progressbar;

pub fn main() {
    Progress::run(Settings {
        window: Window {
            size: (700, 300),
            resizable: true,
            decorations: true,
        },
    })
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
            Message::SliderChanged(x) => self.value = x,
        }
    }

    fn view(&mut self) -> Element<Message> {
        Column::new()
            .padding(20)
            .push(
                Progressbar::new(0.0..=100.0, self.value)
                    .background(Background::Color(Color::from_rgb(
                        0.6, 0.6, 0.6,
                    )))
                    .active_color(Color::from_rgb(0.0, 0.95, 0.0)),
            )
            .push(Slider::new(
                &mut self.progressbar_slider,
                0.0..=100.0,
                self.value,
                Message::SliderChanged,
            ))
            .into()
    }
}
