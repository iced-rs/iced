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
}

impl Sandbox for Slider {
    type Message = Message;

    fn new() -> Slider {
        Slider { slider_value: 50 }
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

        let h_slider =
            container(slider(0..=100, value, Message::SliderChanged))
                .width(Length::Units(250));

        let v_slider =
            container(vertical_slider(0..=100, value, Message::SliderChanged))
                .height(Length::Units(200));

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
