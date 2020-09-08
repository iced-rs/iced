use iced::{
    canvas::{self, Cache, Canvas, Cursor, Geometry, LineCap, Path, Stroke},
    executor, time, Application, Color, Command, Container, Element, Length,
    Point, Rectangle, Settings, Subscription, Vector,
};

pub fn main() -> iced::Result {
    Clock::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}

struct Clock {
    now: chrono::DateTime<chrono::Local>,
    clock: Cache,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Tick(chrono::DateTime<chrono::Local>),
}

impl Application for Clock {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Clock {
                now: chrono::Local::now(),
                clock: Default::default(),
            },
            Command::none(),
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
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(std::time::Duration::from_millis(500))
            .map(|_| Message::Tick(chrono::Local::now()))
    }

    fn view(&mut self) -> Element<Message> {
        let canvas = Canvas::new(self)
            .width(Length::Units(400))
            .height(Length::Units(400));

        Container::new(canvas)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .center_x()
            .center_y()
            .into()
    }
}

impl canvas::Program<Message> for Clock {
    fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        use chrono::Timelike;

        let clock = self.clock.draw(bounds.size(), |frame| {
            let center = frame.center();
            let radius = frame.width().min(frame.height()) / 2.0;

            let background = Path::circle(center, radius);
            frame.fill(&background, Color::from_rgb8(0x12, 0x93, 0xD8));

            let short_hand =
                Path::line(Point::ORIGIN, Point::new(0.0, -0.5 * radius));

            let long_hand =
                Path::line(Point::ORIGIN, Point::new(0.0, -0.8 * radius));

            let thin_stroke = Stroke {
                width: radius / 100.0,
                color: Color::WHITE,
                line_cap: LineCap::Round,
                ..Stroke::default()
            };

            let wide_stroke = Stroke {
                width: thin_stroke.width * 3.0,
                ..thin_stroke
            };

            frame.translate(Vector::new(center.x, center.y));

            frame.with_save(|frame| {
                frame.rotate(hand_rotation(self.now.hour(), 12));
                frame.stroke(&short_hand, wide_stroke);
            });

            frame.with_save(|frame| {
                frame.rotate(hand_rotation(self.now.minute(), 60));
                frame.stroke(&long_hand, wide_stroke);
            });

            frame.with_save(|frame| {
                frame.rotate(hand_rotation(self.now.second(), 60));
                frame.stroke(&long_hand, thin_stroke);
            })
        });

        vec![clock]
    }
}

fn hand_rotation(n: u32, total: u32) -> f32 {
    let turns = n as f32 / total as f32;

    2.0 * std::f32::consts::PI * turns
}
