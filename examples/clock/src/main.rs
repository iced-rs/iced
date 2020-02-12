use iced::{
    canvas, executor, Application, Canvas, Color, Command, Container, Element,
    Length, Point, Settings, Subscription, Vector,
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

    fn subscription(&self) -> Subscription<Message> {
        time::every(std::time::Duration::from_millis(1000)).map(Message::Tick)
    }

    fn view(&mut self) -> Element<Message> {
        let canvas = Canvas::new()
            .width(Length::Units(400))
            .height(Length::Units(400))
            .push(self.clock.with(&self.now));

        Container::new(canvas)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
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
        let offset = Vector::new(center.x, center.y);

        let path = canvas::Path::new(|path| {
            path.arc(canvas::path::Arc {
                center,
                radius,
                start_angle: 0.0,
                end_angle: 360.0 * 2.0 * std::f32::consts::PI,
            })
        });

        frame.fill(
            &path,
            canvas::Fill::Color(Color::from_rgb8(0x12, 0x93, 0xD8)),
        );

        fn draw_handle(
            n: u32,
            total: u32,
            length: f32,
            offset: Vector,
            path: &mut canvas::path::Builder,
        ) {
            let turns = n as f32 / total as f32;
            let t = 2.0 * std::f32::consts::PI * (turns - 0.25);

            let x = length * t.cos();
            let y = length * t.sin();

            path.line_to(Point::new(x, y) + offset);
        }

        let path = canvas::Path::new(|path| {
            path.move_to(center);
            draw_handle(self.hour, 12, 0.5 * radius, offset, path);

            path.move_to(center);
            draw_handle(self.minute, 60, 0.8 * radius, offset, path)
        });

        frame.stroke(
            &path,
            canvas::Stroke {
                width: 6.0,
                color: Color::WHITE,
                line_cap: canvas::LineCap::Round,
                ..canvas::Stroke::default()
            },
        );

        let path = canvas::Path::new(|path| {
            path.move_to(center);
            draw_handle(self.second, 60, 0.8 * radius, offset, path)
        });

        frame.stroke(
            &path,
            canvas::Stroke {
                width: 3.0,
                color: Color::WHITE,
                line_cap: canvas::LineCap::Round,
                ..canvas::Stroke::default()
            },
        );
    }
}

mod time {
    use iced::futures;

    pub fn every(
        duration: std::time::Duration,
    ) -> iced::Subscription<chrono::DateTime<chrono::Local>> {
        iced::Subscription::from_recipe(Every(duration))
    }

    struct Every(std::time::Duration);

    impl<H, I> iced_native::subscription::Recipe<H, I> for Every
    where
        H: std::hash::Hasher,
    {
        type Output = chrono::DateTime<chrono::Local>;

        fn hash(&self, state: &mut H) {
            use std::hash::Hash;

            std::any::TypeId::of::<Self>().hash(state);
            self.0.hash(state);
        }

        fn stream(
            self: Box<Self>,
            _input: futures::stream::BoxStream<'static, I>,
        ) -> futures::stream::BoxStream<'static, Self::Output> {
            use futures::stream::StreamExt;

            async_std::stream::interval(self.0)
                .map(|_| chrono::Local::now())
                .boxed()
        }
    }
}
