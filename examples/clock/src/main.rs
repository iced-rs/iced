use iced::{
    canvas, executor, Application, Canvas, Color, Command, Container, Element,
    Length, Point, Settings, Subscription, Vector,
};

pub fn main() {
    Clock::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}

struct Clock {
    now: LocalTime,
    clock: canvas::layer::Cache<LocalTime>,
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
                now: chrono::Local::now().into(),
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
        time::every(std::time::Duration::from_millis(500)).map(Message::Tick)
    }

    fn view(&mut self) -> Element<Message> {
        let canvas = Canvas::new()
            .width(Length::Units(400))
            .height(Length::Units(400))
            .push(self.clock.with(&self.now));

        Container::new(canvas)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
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

impl canvas::Drawable for LocalTime {
    fn draw(&self, frame: &mut canvas::Frame) {
        use canvas::Path;

        let center = frame.center();
        let radius = frame.width().min(frame.height()) / 2.0;

        let clock = Path::circle(center, radius);
        frame.fill(&clock, Color::from_rgb8(0x12, 0x93, 0xD8));

        let short_hand =
            Path::line(Point::ORIGIN, Point::new(0.0, -0.5 * radius));

        let long_hand =
            Path::line(Point::ORIGIN, Point::new(0.0, -0.8 * radius));

        let thin_stroke = canvas::Stroke {
            width: radius / 100.0,
            color: Color::WHITE,
            line_cap: canvas::LineCap::Round,
            ..canvas::Stroke::default()
        };

        let wide_stroke = canvas::Stroke {
            width: thin_stroke.width * 3.0,
            ..thin_stroke
        };

        frame.translate(Vector::new(center.x, center.y));

        frame.with_save(|frame| {
            frame.rotate(hand_rotation(self.hour, 12));
            frame.stroke(&short_hand, wide_stroke);
        });

        frame.with_save(|frame| {
            frame.rotate(hand_rotation(self.minute, 60));
            frame.stroke(&long_hand, wide_stroke);
        });

        frame.with_save(|frame| {
            frame.rotate(hand_rotation(self.second, 60));
            frame.stroke(&long_hand, thin_stroke);
        });
    }
}

fn hand_rotation(n: u32, total: u32) -> f32 {
    let turns = n as f32 / total as f32;

    2.0 * std::f32::consts::PI * turns
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
