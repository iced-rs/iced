use iced::{
    canvas, executor, Application, Canvas, Color, Command, Element, Length,
    Point, Settings,
};

pub fn main() {
    Clock::run(Settings::default())
}

struct Clock {
    now: LocalTime,
    clock: canvas::layer::Cached<LocalTime>,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Tick(chrono::DateTime<chrono::Local>),
}

impl Application for Clock {
    type Executor = executor::Default;
    type Message = Message;

    fn new() -> (Self, Command<Message>) {
        let now: LocalTime = chrono::Local::now().into();

        (
            Clock {
                now,
                clock: canvas::layer::Cached::new(),
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
                let now = local_time.into();

                if now != self.now {
                    self.now = now;

                    self.clock.clear();
                }
            }
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        Canvas::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .push(self.clock.with(&self.now))
            .into()
    }
}

#[derive(Debug, PartialEq, Eq)]
struct LocalTime {
    hour: u32,
    minute: u32,
    second: u32,
}

impl From<chrono::DateTime<chrono::Local>> for LocalTime {
    fn from(date_time: chrono::DateTime<chrono::Local>) -> LocalTime {
        use chrono::Timelike;

        LocalTime {
            hour: date_time.hour(),
            minute: date_time.minute(),
            second: date_time.second(),
        }
    }
}

impl canvas::layer::Drawable for LocalTime {
    fn draw(&self, frame: &mut canvas::Frame) {
        let center = frame.center();
        let radius = frame.width().min(frame.height()) as f32 / 2.0;

        let mut path = canvas::Path::new();

        path.arc(canvas::path::Arc {
            center,
            radius,
            start_angle: 0.0,
            end_angle: 360.0 * 2.0 * std::f32::consts::PI,
        });

        frame.fill(
            path,
            canvas::Fill::Color(Color::from_rgb8(0x12, 0x93, 0xD8)),
        );

        fn draw_handle(
            n: u32,
            total: u32,
            length: f32,
            path: &mut canvas::Path,
        ) {
            let turns = n as f32 / total as f32;
            let t = 2.0 * std::f32::consts::PI * (turns - 0.25);

            let x = length * t.cos();
            let y = length * t.sin();

            path.line_to(Point::new(x, y));
        }

        let mut path = canvas::Path::new();

        path.move_to(center);
        draw_handle(self.hour, 12, 0.6 * radius, &mut path);

        path.move_to(center);
        draw_handle(self.minute, 60, 0.9 * radius, &mut path);

        frame.stroke(
            path,
            canvas::Stroke {
                width: 4.0,
                color: Color::WHITE,
                line_cap: canvas::LineCap::Round,
                ..canvas::Stroke::default()
            },
        );

        let mut path = canvas::Path::new();

        path.move_to(center);
        draw_handle(self.second, 60, 0.9 * radius, &mut path);

        frame.stroke(
            path,
            canvas::Stroke {
                width: 2.0,
                color: Color::WHITE,
                line_cap: canvas::LineCap::Round,
                ..canvas::Stroke::default()
            },
        );
    }
}
