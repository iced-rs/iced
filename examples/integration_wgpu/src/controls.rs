use iced_wgpu::Renderer;
use iced_winit::widget::slider::{self, Slider};
use iced_winit::widget::text_input::{self, TextInput};
use iced_winit::widget::{Column, Row, Text};
use iced_winit::{Alignment, Color, Command, Element, Length, Program};

pub struct Controls {
    background_color: Color,
    text: String,
    sliders: [slider::State; 3],
    text_input: text_input::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    BackgroundColorChanged(Color),
    TextChanged(String),
}

impl Controls {
    pub fn new() -> Controls {
        Controls {
            background_color: Color::BLACK,
            text: Default::default(),
            sliders: Default::default(),
            text_input: Default::default(),
        }
    }

    pub fn background_color(&self) -> Color {
        self.background_color
    }
}

impl Program for Controls {
    type Renderer = Renderer;
    type Message = Message;

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::BackgroundColorChanged(color) => {
                self.background_color = color;
            }
            Message::TextChanged(text) => {
                self.text = text;
            }
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message, Renderer> {
        let [r, g, b] = &mut self.sliders;
        let t = &mut self.text_input;
        let background_color = self.background_color;
        let text = &self.text;

        let sliders = Row::new()
            .width(Length::Units(500))
            .spacing(20)
            .push(
                Slider::new(r, 0.0..=1.0, background_color.r, move |r| {
                    Message::BackgroundColorChanged(Color {
                        r,
                        ..background_color
                    })
                })
                .step(0.01),
            )
            .push(
                Slider::new(g, 0.0..=1.0, background_color.g, move |g| {
                    Message::BackgroundColorChanged(Color {
                        g,
                        ..background_color
                    })
                })
                .step(0.01),
            )
            .push(
                Slider::new(b, 0.0..=1.0, background_color.b, move |b| {
                    Message::BackgroundColorChanged(Color {
                        b,
                        ..background_color
                    })
                })
                .step(0.01),
            );

        Row::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(Alignment::End)
            .push(
                Column::new()
                    .width(Length::Fill)
                    .align_items(Alignment::End)
                    .push(
                        Column::new()
                            .padding(10)
                            .spacing(10)
                            .push(
                                Text::new("Background color")
                                    .color(Color::WHITE),
                            )
                            .push(sliders)
                            .push(
                                Text::new(format!("{:?}", background_color))
                                    .size(14)
                                    .color(Color::WHITE),
                            )
                            .push(TextInput::new(
                                t,
                                "Placeholder",
                                text,
                                move |text| Message::TextChanged(text),
                            )),
                    ),
            )
            .into()
    }
}
