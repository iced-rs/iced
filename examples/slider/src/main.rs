use iced::widget::{column, container, iced, slider, text, vertical_slider};
use iced::{Center, Element, Fill};

pub fn main() -> iced::Result {
    iced::run("Slider - Iced", Slider::update, Slider::view)
}

#[derive(Debug, Clone)]
pub enum Message {
    SliderChanged(u8),
}

pub struct Slider {
    value: u8,
}

impl Slider {
    fn new() -> Self {
        Slider { value: 50 }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::SliderChanged(value) => {
                self.value = value;
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let h_slider = container(
            slider(1..=100, self.value, Message::SliderChanged)
                .default(50)
                .shift_step(5),
        )
        .width(250);

        let v_slider = container(
            vertical_slider(1..=100, self.value, Message::SliderChanged)
                .default(50)
                .shift_step(5),
        )
        .height(200);

        let text = text(self.value);

        column![v_slider, h_slider, text, iced(self.value as f32),]
            .width(Fill)
            .align_x(Center)
            .spacing(20)
            .padding(20)
            .into()
    }
}

impl Default for Slider {
    fn default() -> Self {
        Self::new()
    }
}
