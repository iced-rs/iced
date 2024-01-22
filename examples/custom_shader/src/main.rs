mod scene;

use scene::Scene;

use iced::executor;
use iced::time::Instant;
use iced::widget::shader::wgpu;
use iced::widget::{checkbox, column, container, row, shader, slider, text};
use iced::window;
use iced::{
    Alignment, Application, Color, Command, Element, Length, Subscription,
    Theme,
};

fn main() -> iced::Result {
    IcedCubes::run(iced::Settings::default())
}

struct IcedCubes {
    start: Instant,
    scene: Scene,
}

#[derive(Debug, Clone)]
enum Message {
    CubeAmountChanged(u32),
    CubeSizeChanged(f32),
    Tick(Instant),
    ShowDepthBuffer(bool),
    LightColorChanged(Color),
}

impl Application for IcedCubes {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                start: Instant::now(),
                scene: Scene::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Iced Cubes".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::CubeAmountChanged(amount) => {
                self.scene.change_amount(amount);
            }
            Message::CubeSizeChanged(size) => {
                self.scene.size = size;
            }
            Message::Tick(time) => {
                self.scene.update(time - self.start);
            }
            Message::ShowDepthBuffer(show) => {
                self.scene.show_depth_buffer = show;
            }
            Message::LightColorChanged(color) => {
                self.scene.light_color = color;
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let top_controls = row![
            control(
                "Amount",
                slider(
                    1..=scene::MAX,
                    self.scene.cubes.len() as u32,
                    Message::CubeAmountChanged
                )
                .width(100)
            ),
            control(
                "Size",
                slider(0.1..=0.25, self.scene.size, Message::CubeSizeChanged)
                    .step(0.01)
                    .width(100),
            ),
            checkbox(
                "Show Depth Buffer",
                self.scene.show_depth_buffer,
                Message::ShowDepthBuffer
            ),
        ]
        .spacing(40);

        let bottom_controls = row![
            control(
                "R",
                slider(0.0..=1.0, self.scene.light_color.r, move |r| {
                    Message::LightColorChanged(Color {
                        r,
                        ..self.scene.light_color
                    })
                })
                .step(0.01)
                .width(100)
            ),
            control(
                "G",
                slider(0.0..=1.0, self.scene.light_color.g, move |g| {
                    Message::LightColorChanged(Color {
                        g,
                        ..self.scene.light_color
                    })
                })
                .step(0.01)
                .width(100)
            ),
            control(
                "B",
                slider(0.0..=1.0, self.scene.light_color.b, move |b| {
                    Message::LightColorChanged(Color {
                        b,
                        ..self.scene.light_color
                    })
                })
                .step(0.01)
                .width(100)
            )
        ]
        .spacing(40);

        let controls = column![top_controls, bottom_controls,]
            .spacing(10)
            .padding(20)
            .align_items(Alignment::Center);

        let shader =
            shader(&self.scene).width(Length::Fill).height(Length::Fill);

        container(column![shader, controls].align_items(Alignment::Center))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        window::frames().map(Message::Tick)
    }
}

fn control<'a>(
    label: &'static str,
    control: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    row![text(label), control.into()].spacing(10).into()
}
