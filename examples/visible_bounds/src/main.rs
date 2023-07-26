use iced::executor;
use iced::mouse;
use iced::subscription::{self, Subscription};
use iced::theme::{self, Theme};
use iced::widget::{
    column, container, horizontal_space, row, scrollable, text, vertical_space,
};
use iced::window;
use iced::{
    Alignment, Application, Color, Command, Element, Event, Font, Length,
    Point, Rectangle, Settings,
};

pub fn main() -> iced::Result {
    Example::run(Settings::default())
}

struct Example {
    mouse_position: Option<Point>,
    outer_bounds: Option<Rectangle>,
    inner_bounds: Option<Rectangle>,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    MouseMoved(Point),
    WindowResized,
    Scrolled(scrollable::Viewport),
    OuterBoundsFetched(Option<Rectangle>),
    InnerBoundsFetched(Option<Rectangle>),
}

impl Application for Example {
    type Message = Message;
    type Theme = Theme;
    type Flags = ();
    type Executor = executor::Default;

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                mouse_position: None,
                outer_bounds: None,
                inner_bounds: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Visible bounds - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::MouseMoved(position) => {
                self.mouse_position = Some(position);

                Command::none()
            }
            Message::Scrolled(_) | Message::WindowResized => {
                Command::batch(vec![
                    container::visible_bounds(OUTER_CONTAINER.clone())
                        .map(Message::OuterBoundsFetched),
                    container::visible_bounds(INNER_CONTAINER.clone())
                        .map(Message::InnerBoundsFetched),
                ])
            }
            Message::OuterBoundsFetched(outer_bounds) => {
                self.outer_bounds = outer_bounds;

                Command::none()
            }
            Message::InnerBoundsFetched(inner_bounds) => {
                self.inner_bounds = inner_bounds;

                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let data_row = |label, value, color| {
            row![
                text(label),
                horizontal_space(Length::Fill),
                text(value).font(Font::MONOSPACE).size(14).style(color),
            ]
            .height(40)
            .align_items(Alignment::Center)
        };

        let view_bounds = |label, bounds: Option<Rectangle>| {
            data_row(
                label,
                match bounds {
                    Some(bounds) => format!("{:?}", bounds),
                    None => "not visible".to_string(),
                },
                if bounds
                    .zip(self.mouse_position)
                    .map(|(bounds, mouse_position)| {
                        bounds.contains(mouse_position)
                    })
                    .unwrap_or_default()
                {
                    Color {
                        g: 1.0,
                        ..Color::BLACK
                    }
                    .into()
                } else {
                    theme::Text::Default
                },
            )
        };

        column![
            data_row(
                "Mouse position",
                match self.mouse_position {
                    Some(Point { x, y }) => format!("({x}, {y})"),
                    None => "unknown".to_string(),
                },
                theme::Text::Default,
            ),
            view_bounds("Outer container", self.outer_bounds),
            view_bounds("Inner container", self.inner_bounds),
            scrollable(
                column![
                    text("Scroll me!"),
                    vertical_space(400),
                    container(text("I am the outer container!"))
                        .id(OUTER_CONTAINER.clone())
                        .padding(40)
                        .style(theme::Container::Box),
                    vertical_space(400),
                    scrollable(
                        column![
                            text("Scroll me!"),
                            vertical_space(400),
                            container(text("I am the inner container!"))
                                .id(INNER_CONTAINER.clone())
                                .padding(40)
                                .style(theme::Container::Box),
                            vertical_space(400)
                        ]
                        .padding(20)
                    )
                    .on_scroll(Message::Scrolled)
                    .width(Length::Fill)
                    .height(300),
                ]
                .padding(20)
            )
            .on_scroll(Message::Scrolled)
            .width(Length::Fill)
            .height(300),
        ]
        .spacing(10)
        .padding(20)
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        subscription::events_with(|event, _| match event {
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                Some(Message::MouseMoved(position))
            }
            Event::Window(window::Event::Resized { .. }) => {
                Some(Message::WindowResized)
            }
            _ => None,
        })
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

use once_cell::sync::Lazy;

static OUTER_CONTAINER: Lazy<container::Id> =
    Lazy::new(|| container::Id::new("outer"));
static INNER_CONTAINER: Lazy<container::Id> =
    Lazy::new(|| container::Id::new("inner"));
