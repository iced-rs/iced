use iced::alignment;
use iced::mouse;
use iced::time;
use iced::widget::canvas::{stroke, Cache, Geometry, LineCap, Path, Stroke};
use iced::widget::{canvas, container};
use iced::{
    Degrees, Element, Fill, Font, Point, Rectangle, Renderer, Subscription,
    Theme, Vector,
};

pub fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::application("Clock - Iced", Clock::update, Clock::view)
        .subscription(Clock::subscription)
        .theme(Clock::theme)
        .antialiasing(true)
        .run()
}

struct Clock {
    now: chrono::DateTime<chrono::Local>,
    clock: Cache,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Tick(chrono::DateTime<chrono::Local>),
}

impl Clock {
    fn update(&mut self, message: Message) {
        match message {
            Message::Tick(local_time) => {
                let now = local_time;

                if now != self.now {
                    self.now = now;
                    self.clock.clear();
                }
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let canvas = canvas(self as &Self).width(Fill).height(Fill);

        container(canvas).padding(20).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(time::Duration::from_millis(500))
            .map(|_| Message::Tick(chrono::offset::Local::now()))
    }

    fn theme(&self) -> Theme {
        Theme::ALL[(self.now.timestamp() as usize / 10) % Theme::ALL.len()]
            .clone()
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self {
            now: chrono::offset::Local::now(),
            clock: Cache::default(),
        }
    }
}

impl<Message> canvas::Program<Message> for Clock {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        use chrono::Timelike;

        let clock = self.clock.draw(renderer, bounds.size(), |frame| {
            let palette = theme.extended_palette();

            let center = frame.center();
            let radius = frame.width().min(frame.height()) / 2.0;

            let background = Path::circle(center, radius);
            frame.fill(&background, palette.secondary.strong.color);

            let short_hand =
                Path::line(Point::ORIGIN, Point::new(0.0, -0.5 * radius));

            let long_hand =
                Path::line(Point::ORIGIN, Point::new(0.0, -0.8 * radius));

            let width = radius / 100.0;

            let thin_stroke = || -> Stroke {
                Stroke {
                    width,
                    style: stroke::Style::Solid(palette.secondary.strong.text),
                    line_cap: LineCap::Round,
                    ..Stroke::default()
                }
            };

            let wide_stroke = || -> Stroke {
                Stroke {
                    width: width * 3.0,
                    style: stroke::Style::Solid(palette.secondary.strong.text),
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
                let rotation = hand_rotation(self.now.second(), 60);

                frame.rotate(rotation);
                frame.stroke(&long_hand, thin_stroke());

                let rotate_factor = if rotation < 180.0 { 1.0 } else { -1.0 };

                frame.rotate(Degrees(-90.0 * rotate_factor));
                frame.fill_text(canvas::Text {
                    content: theme.to_string(),
                    size: (radius / 15.0).into(),
                    position: Point::new(
                        (0.78 * radius) * rotate_factor,
                        -width * 2.0,
                    ),
                    color: palette.secondary.strong.text,
                    horizontal_alignment: if rotate_factor > 0.0 {
                        alignment::Horizontal::Right
                    } else {
                        alignment::Horizontal::Left
                    },
                    vertical_alignment: alignment::Vertical::Bottom,
                    font: Font::MONOSPACE,
                    ..canvas::Text::default()
                });
            });
        });

        vec![clock]
    }
}

fn hand_rotation(n: u32, total: u32) -> Degrees {
    let turns = n as f32 / total as f32;

    Degrees(360.0 * turns)
}
