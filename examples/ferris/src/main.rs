use iced::widget::{column, container, image, pick_list, row, slider, text};
use iced::{
    Alignment, Color, ContentFit, Degrees, Element, Length, Rotation, Theme,
};

pub fn main() -> iced::Result {
    iced::program("Ferris - Iced", Image::update, Image::view)
        .theme(|_| Theme::TokyoNight)
        .run()
}

struct Image {
    width: f32,
    rotation: Rotation,
    content_fit: ContentFit,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    WidthChanged(f32),
    RotationStrategyChanged(RotationStrategy),
    RotationChanged(Degrees),
    ContentFitChanged(ContentFit),
}

impl Image {
    fn update(&mut self, message: Message) {
        match message {
            Message::WidthChanged(width) => {
                self.width = width;
            }
            Message::RotationStrategyChanged(strategy) => {
                self.rotation = match strategy {
                    RotationStrategy::Floating => {
                        Rotation::Floating(self.rotation.radians())
                    }
                    RotationStrategy::Solid => {
                        Rotation::Solid(self.rotation.radians())
                    }
                };
            }
            Message::RotationChanged(rotation) => {
                self.rotation = match self.rotation {
                    Rotation::Floating(_) => {
                        Rotation::Floating(rotation.into())
                    }
                    Rotation::Solid(_) => Rotation::Solid(rotation.into()),
                };
            }
            Message::ContentFitChanged(content_fit) => {
                self.content_fit = content_fit;
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let i_am_ferris = container(
            column![
                "Hello!",
                Element::from(
                    image(format!(
                        "{}/../tour/images/ferris.png",
                        env!("CARGO_MANIFEST_DIR")
                    ))
                    .width(self.width)
                    .content_fit(self.content_fit)
                    .rotation(self.rotation)
                )
                .explain(Color::WHITE),
                "I am Ferris!"
            ]
            .spacing(20)
            .align_items(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y();

        let sizing = row![
            pick_list(
                [
                    ContentFit::Contain,
                    ContentFit::Cover,
                    ContentFit::Fill,
                    ContentFit::None,
                    ContentFit::ScaleDown
                ],
                Some(self.content_fit),
                Message::ContentFitChanged
            )
            .width(Length::Fill),
            column![
                slider(100.0..=500.0, self.width, Message::WidthChanged),
                text(format!("Width: {}px", self.width))
                    .size(14)
                    .line_height(1.0)
            ]
            .spacing(5)
            .align_items(Alignment::Center)
        ]
        .spacing(10);

        let rotation = row![
            pick_list(
                [RotationStrategy::Floating, RotationStrategy::Solid],
                Some(match self.rotation {
                    Rotation::Floating(_) => RotationStrategy::Floating,
                    Rotation::Solid(_) => RotationStrategy::Solid,
                }),
                Message::RotationStrategyChanged,
            )
            .width(Length::Fill),
            column![
                slider(
                    Degrees::RANGE,
                    self.rotation.degrees(),
                    Message::RotationChanged
                ),
                text(format!(
                    "Rotation: {:.0}°",
                    f32::from(self.rotation.degrees())
                ))
                .size(14)
                .line_height(1.0)
            ]
            .spacing(5)
            .align_items(Alignment::Center)
        ]
        .spacing(10);

        container(column![i_am_ferris, sizing, rotation].spacing(10))
            .padding(10)
            .into()
    }
}

impl Default for Image {
    fn default() -> Self {
        Self {
            width: 300.0,
            rotation: Rotation::default(),
            content_fit: ContentFit::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RotationStrategy {
    Floating,
    Solid,
}

impl std::fmt::Display for RotationStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Floating => "Floating",
            Self::Solid => "Solid",
        })
    }
}
