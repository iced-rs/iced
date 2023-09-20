use iced::widget::{column, container, slider, text, vertical_slider};
use iced::{Element, Length, Sandbox, Settings};

pub fn main() -> iced::Result {
    Slider::run(Settings::default())
}

#[derive(Debug, Clone)]
pub enum Message {
    SliderChanged(u8),
}

pub struct Slider {
    slider_value: u8,
    slider_default: u8,
    slider_step: u8,
    slider_step_fine: u8,
}

impl Sandbox for Slider {
    type Message = Message;

    fn new() -> Slider {
        Slider {
            slider_value: 50,
            slider_default: 50,
            slider_step: 5,
            slider_step_fine: 1,
        }
    }

    fn title(&self) -> String {
        String::from("Slider - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::SliderChanged(value) => {
                self.slider_value = value;
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let value = self.slider_value;
        let default = self.slider_default;
        let step = self.slider_step;
        let step_fine = self.slider_step_fine;

        let h_slider = container(
            slider(0..=100, value, Message::SliderChanged)
                .default(default)
                .step(step)
                .step_fine(step_fine),
        )
        .width(250);

        let v_slider = container(
            vertical_slider(0..=100, value, Message::SliderChanged)
                .default(default)
                .step(step)
                .step_fine(step_fine),
        )
        .height(200);

        let text = text(format!("{value}"));

        container(
            column![
                container(v_slider).width(Length::Fill).center_x(),
                container(h_slider).width(Length::Fill).center_x(),
                container(text).width(Length::Fill).center_x(),
            ]
            .spacing(25),
        )
        .height(Length::Fill)
        .width(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }
}
