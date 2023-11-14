mod camera;
mod cubes;
mod pipeline;
mod primitive;

use crate::camera::Camera;
use crate::cubes::Cubes;
use crate::pipeline::Pipeline;

use iced::executor;
use iced::time::Instant;
use iced::widget::{
    checkbox, column, container, row, shader, slider, text, vertical_space,
};
use iced::window;
use iced::{
    Alignment, Application, Color, Command, Element, Length, Renderer,
    Subscription, Theme,
};

fn main() -> iced::Result {
    IcedCubes::run(iced::Settings::default())
}

struct IcedCubes {
    start: Instant,
    cubes: Cubes,
    num_cubes_slider: u32,
}

impl Default for IcedCubes {
    fn default() -> Self {
        Self {
            start: Instant::now(),
            cubes: Cubes::new(),
            num_cubes_slider: cubes::MAX,
        }
    }
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
        (IcedCubes::default(), Command::none())
    }

    fn title(&self) -> String {
        "Iced Cubes".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::CubeAmountChanged(num) => {
                self.num_cubes_slider = num;
                self.cubes.adjust_num_cubes(num);
            }
            Message::CubeSizeChanged(size) => {
                self.cubes.size = size;
            }
            Message::Tick(time) => {
                self.cubes.update(time - self.start);
            }
            Message::ShowDepthBuffer(show) => {
                self.cubes.show_depth_buffer = show;
            }
            Message::LightColorChanged(color) => {
                self.cubes.light_color = color;
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        let top_controls = row![
            control(
                "Amount",
                slider(
                    1..=cubes::MAX,
                    self.num_cubes_slider,
                    Message::CubeAmountChanged
                )
                .width(100)
            ),
            control(
                "Size",
                slider(0.1..=0.25, self.cubes.size, Message::CubeSizeChanged)
                    .step(0.01)
                    .width(100),
            ),
            checkbox(
                "Show Depth Buffer",
                self.cubes.show_depth_buffer,
                Message::ShowDepthBuffer
            ),
        ]
        .spacing(40);

        let bottom_controls = row![
            control(
                "R",
                slider(0.0..=1.0, self.cubes.light_color.r, move |r| {
                    Message::LightColorChanged(Color {
                        r,
                        ..self.cubes.light_color
                    })
                })
                .step(0.01)
                .width(100)
            ),
            control(
                "G",
                slider(0.0..=1.0, self.cubes.light_color.g, move |g| {
                    Message::LightColorChanged(Color {
                        g,
                        ..self.cubes.light_color
                    })
                })
                .step(0.01)
                .width(100)
            ),
            control(
                "B",
                slider(0.0..=1.0, self.cubes.light_color.b, move |b| {
                    Message::LightColorChanged(Color {
                        b,
                        ..self.cubes.light_color
                    })
                })
                .step(0.01)
                .width(100)
            )
        ]
        .spacing(40);

        let controls = column![top_controls, bottom_controls,]
            .spacing(10)
            .align_items(Alignment::Center);

        let shader =
            shader(&self.cubes).width(Length::Fill).height(Length::Fill);

        container(
            column![shader, controls, vertical_space(20),]
                .spacing(40)
                .align_items(Alignment::Center),
        )
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
