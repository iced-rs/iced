use iced::executor;
use iced::wayland::layer_surface::{get_layer_surface, set_size};
use iced::wayland::SurfaceIdWrapper;
use iced::widget::canvas::{
    stroke, Cache, Cursor, Geometry, LineCap, Path, Stroke,
};
use iced::widget::{canvas, container};
use iced::{
    sctk_settings::InitialSurface, Application, Color, Command, Element,
    Length, Point, Rectangle, Settings, Subscription, Theme, Vector,
};
use iced_native::command::platform_specific::wayland::layer_surface::SctkLayerSurfaceSettings;
use iced_native::window::Id;
use sctk::shell::layer::Anchor;

pub fn main() -> iced::Result {
    Clock::run(Settings {
        antialiasing: true,
        initial_surface: InitialSurface::LayerSurface(
            SctkLayerSurfaceSettings {
                size: (None, Some(200)),
                anchor: Anchor::LEFT.union(Anchor::RIGHT).union(Anchor::TOP),
                exclusive_zone: 200,
                ..Default::default()
            },
        ),
        ..Settings::default()
    })
}

struct Clock {
    now: time::OffsetDateTime,
    clock: Cache,
    count: u32,
    to_destroy: Id,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Tick(time::OffsetDateTime),
}

impl Application for Clock {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let to_destroy = Id::new(10);
        (
            Clock {
                now: time::OffsetDateTime::now_local()
                    .unwrap_or_else(|_| time::OffsetDateTime::now_utc()),
                clock: Default::default(),
                count: 0,
                to_destroy,
            },
            get_layer_surface(SctkLayerSurfaceSettings {
                // XXX id must be unique!
                id: to_destroy,
                size: (None, Some(100)),
                anchor: Anchor::LEFT.union(Anchor::RIGHT).union(Anchor::BOTTOM),
                exclusive_zone: 100,
                ..Default::default()
            }),
        )
    }

    fn title(&self) -> String {
        String::from("Clock - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Tick(local_time) => {
                let now = local_time;

                if now != self.now {
                    self.now = now;
                    self.clock.clear();
                }
                // destroy the second layer surface after counting to 10.
                self.count += 1;
                if self.count == 10 {
                    println!("time to remove the bottom clock!");
                    return set_size::<Message>(
                        self.to_destroy,
                        None,
                        Some(200),
                    );
                }
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(std::time::Duration::from_millis(500)).map(|_| {
            Message::Tick(
                time::OffsetDateTime::now_local()
                    .unwrap_or_else(|_| time::OffsetDateTime::now_utc()),
            )
        })
    }

    fn view(
        &self,
        _id: SurfaceIdWrapper,
    ) -> Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let canvas = canvas(self as &Self)
            .width(Length::Fill)
            .height(Length::Fill);

        container(canvas)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .into()
    }

    fn close_requested(&self, _id: SurfaceIdWrapper) -> Self::Message {
        unimplemented!()
    }
}

impl<Message> canvas::Program<Message> for Clock {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let clock = self.clock.draw(bounds.size(), |frame| {
            let center = frame.center();
            let radius = frame.width().min(frame.height()) / 2.0;

            let background = Path::circle(center, radius);
            frame.fill(&background, Color::from_rgb8(0x12, 0x93, 0xD8));

            let short_hand =
                Path::line(Point::ORIGIN, Point::new(0.0, -0.5 * radius));

            let long_hand =
                Path::line(Point::ORIGIN, Point::new(0.0, -0.8 * radius));

            let width = radius / 100.0;

            let thin_stroke = || -> Stroke {
                Stroke {
                    width,
                    style: stroke::Style::Solid(Color::WHITE),
                    line_cap: LineCap::Round,
                    ..Stroke::default()
                }
            };

            let wide_stroke = || -> Stroke {
                Stroke {
                    width: width * 3.0,
                    style: stroke::Style::Solid(Color::WHITE),
                    line_cap: LineCap::Round,
                    ..Stroke::default()
                }
            };

            frame.translate(Vector::new(center.x, center.y));

            frame.with_save(|frame| {
                frame.rotate(hand_rotation(self.now.hour(), 12));
                frame.stroke(&short_hand, wide_stroke());
            });

            frame.with_save(|frame| {
                frame.rotate(hand_rotation(self.now.minute(), 60));
                frame.stroke(&long_hand, wide_stroke());
            });

            frame.with_save(|frame| {
                frame.rotate(hand_rotation(self.now.second(), 60));
                frame.stroke(&long_hand, thin_stroke());
            })
        });

        vec![clock]
    }
}

fn hand_rotation(n: u8, total: u8) -> f32 {
    let turns = n as f32 / total as f32;

    2.0 * std::f32::consts::PI * turns
}
