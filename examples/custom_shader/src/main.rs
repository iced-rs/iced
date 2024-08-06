mod scene;

use scene::Scene;

use iced::time::Instant;
use iced::widget::shader::wgpu;
use iced::widget::{center, checkbox, column, row, shader, slider, text};
use iced::window;
use iced::{Center, Color, Element, Fill, Subscription};

fn main() -> iced::Result {
    iced::application(
        "Custom Shader - Iced",
        IcedCubes::update,
        IcedCubes::view,
    )
    .subscription(IcedCubes::subscription)
    .run()
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

impl IcedCubes {
    fn new() -> Self {
        Self {
            start: Instant::now(),
            scene: Scene::new(),
        }
    }

    fn update(&mut self, message: Message) {
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
    }

    fn view(&self) -> Element<'_, Message> {
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
            checkbox("Show Depth Buffer", self.scene.show_depth_buffer)
                .on_toggle(Message::ShowDepthBuffer),
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
            .align_x(Center);

        let shader = shader(&self.scene).width(Fill).height(Fill);

        center(column![shader, controls].align_x(Center)).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        window::frames().map(Message::Tick)
    }
}

impl Default for IcedCubes {
    fn default() -> Self {
        Self::new()
    }
}

fn control<'a>(
    label: &'static str,
    control: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    row![text(label), control.into()].spacing(10).into()
}
