use iced::time::Instant;
use iced::widget::{
    center, checkbox, column, container, image, pick_list, row, slider, text,
};
use iced::window;
use iced::{
    Bottom, Center, Color, ContentFit, Degrees, Element, Fill, Radians,
    Rotation, Subscription, Theme,
};

pub fn main() -> iced::Result {
    iced::application("Ferris - Iced", Image::update, Image::view)
        .subscription(Image::subscription)
        .theme(|_| Theme::TokyoNight)
        .run()
}

struct Image {
    width: f32,
    opacity: f32,
    rotation: Rotation,
    content_fit: ContentFit,
    spin: bool,
    last_tick: Instant,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    WidthChanged(f32),
    OpacityChanged(f32),
    RotationStrategyChanged(RotationStrategy),
    RotationChanged(Degrees),
    ContentFitChanged(ContentFit),
    SpinToggled(bool),
    RedrawRequested(Instant),
}

impl Image {
    fn update(&mut self, message: Message) {
        match message {
            Message::WidthChanged(width) => {
                self.width = width;
            }
            Message::OpacityChanged(opacity) => {
                self.opacity = opacity;
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
            Message::SpinToggled(spin) => {
                self.spin = spin;
                self.last_tick = Instant::now();
            }
            Message::RedrawRequested(now) => {
                const ROTATION_SPEED: Degrees = Degrees(360.0);

                let delta = (now - self.last_tick).as_millis() as f32 / 1_000.0;

                *self.rotation.radians_mut() = (self.rotation.radians()
                    + ROTATION_SPEED * delta)
                    % (2.0 * Radians::PI);

                self.last_tick = now;
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.spin {
            window::frames().map(Message::RedrawRequested)
        } else {
            Subscription::none()
        }
    }

    fn view(&self) -> Element<Message> {
        let i_am_ferris = column![
            "Hello!",
            Element::from(
                image(format!(
                    "{}/../tour/images/ferris.png",
                    env!("CARGO_MANIFEST_DIR")
                ))
                .width(self.width)
                .content_fit(self.content_fit)
                .rotation(self.rotation)
                .opacity(self.opacity)
            )
            .explain(Color::WHITE),
            "I am Ferris!"
        ]
        .spacing(20)
        .align_x(Center);

        let fit = row![
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
            .width(Fill),
            pick_list(
                [RotationStrategy::Floating, RotationStrategy::Solid],
                Some(match self.rotation {
                    Rotation::Floating(_) => RotationStrategy::Floating,
                    Rotation::Solid(_) => RotationStrategy::Solid,
                }),
                Message::RotationStrategyChanged,
            )
            .width(Fill),
        ]
        .spacing(10)
        .align_y(Bottom);

        let properties = row![
            with_value(
                slider(100.0..=500.0, self.width, Message::WidthChanged),
                format!("Width: {}px", self.width)
            ),
            with_value(
                slider(0.0..=1.0, self.opacity, Message::OpacityChanged)
                    .step(0.01),
                format!("Opacity: {:.2}", self.opacity)
            ),
            with_value(
                row![
                    slider(
                        Degrees::RANGE,
                        self.rotation.degrees(),
                        Message::RotationChanged
                    ),
                    checkbox("Spin!", self.spin)
                        .text_size(12)
                        .on_toggle(Message::SpinToggled)
                        .size(12)
                ]
                .spacing(10)
                .align_y(Center),
                format!("Rotation: {:.0}°", f32::from(self.rotation.degrees()))
            )
        ]
        .spacing(10)
        .align_y(Bottom);

        container(column![fit, center(i_am_ferris), properties].spacing(10))
            .padding(10)
            .into()
    }
}

impl Default for Image {
    fn default() -> Self {
        Self {
            width: 300.0,
            opacity: 1.0,
            rotation: Rotation::default(),
            content_fit: ContentFit::default(),
            spin: false,
            last_tick: Instant::now(),
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

fn with_value<'a>(
    control: impl Into<Element<'a, Message>>,
    value: String,
) -> Element<'a, Message> {
    column![control.into(), text(value).size(12).line_height(1.0)]
        .spacing(2)
        .align_x(Center)
        .into()
}
