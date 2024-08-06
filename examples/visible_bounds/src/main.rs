use iced::event::{self, Event};
use iced::mouse;
use iced::widget::{
    column, container, horizontal_space, row, scrollable, text, vertical_space,
};
use iced::window;
use iced::{
    Center, Color, Element, Fill, Font, Point, Rectangle, Subscription, Task,
    Theme,
};

pub fn main() -> iced::Result {
    iced::application("Visible Bounds - Iced", Example::update, Example::view)
        .subscription(Example::subscription)
        .theme(|_| Theme::Dark)
        .run()
}

#[derive(Default)]
struct Example {
    mouse_position: Option<Point>,
    outer_bounds: Option<Rectangle>,
    inner_bounds: Option<Rectangle>,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    MouseMoved(Point),
    WindowResized,
    Scrolled,
    OuterBoundsFetched(Option<Rectangle>),
    InnerBoundsFetched(Option<Rectangle>),
}

impl Example {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::MouseMoved(position) => {
                self.mouse_position = Some(position);

                Task::none()
            }
            Message::Scrolled | Message::WindowResized => Task::batch(vec![
                container::visible_bounds(OUTER_CONTAINER.clone())
                    .map(Message::OuterBoundsFetched),
                container::visible_bounds(INNER_CONTAINER.clone())
                    .map(Message::InnerBoundsFetched),
            ]),
            Message::OuterBoundsFetched(outer_bounds) => {
                self.outer_bounds = outer_bounds;

                Task::none()
            }
            Message::InnerBoundsFetched(inner_bounds) => {
                self.inner_bounds = inner_bounds;

                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let data_row = |label, value, color| {
            row![
                text(label),
                horizontal_space(),
                text(value)
                    .font(Font::MONOSPACE)
                    .size(14)
                    .color_maybe(color),
            ]
            .height(40)
            .align_y(Center)
        };

        let view_bounds = |label, bounds: Option<Rectangle>| {
            data_row(
                label,
                match bounds {
                    Some(bounds) => format!("{bounds:?}"),
                    None => "not visible".to_string(),
                },
                if bounds
                    .zip(self.mouse_position)
                    .map(|(bounds, mouse_position)| {
                        bounds.contains(mouse_position)
                    })
                    .unwrap_or_default()
                {
                    Some(Color {
                        g: 1.0,
                        ..Color::BLACK
                    })
                } else {
                    None
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
                None,
            ),
            view_bounds("Outer container", self.outer_bounds),
            view_bounds("Inner container", self.inner_bounds),
            scrollable(
                column![
                    text("Scroll me!"),
                    vertical_space().height(400),
                    container(text("I am the outer container!"))
                        .id(OUTER_CONTAINER.clone())
                        .padding(40)
                        .style(container::rounded_box),
                    vertical_space().height(400),
                    scrollable(
                        column![
                            text("Scroll me!"),
                            vertical_space().height(400),
                            container(text("I am the inner container!"))
                                .id(INNER_CONTAINER.clone())
                                .padding(40)
                                .style(container::rounded_box),
                            vertical_space().height(400),
                        ]
                        .padding(20)
                    )
                    .on_scroll(|_| Message::Scrolled)
                    .width(Fill)
                    .height(300),
                ]
                .padding(20)
            )
            .on_scroll(|_| Message::Scrolled)
            .width(Fill)
            .height(300),
        ]
        .spacing(10)
        .padding(20)
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, _status, _window| match event {
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                Some(Message::MouseMoved(position))
            }
            Event::Window(window::Event::Resized { .. }) => {
                Some(Message::WindowResized)
            }
            _ => None,
        })
    }
}

use once_cell::sync::Lazy;

static OUTER_CONTAINER: Lazy<container::Id> =
    Lazy::new(|| container::Id::new("outer"));
static INNER_CONTAINER: Lazy<container::Id> =
    Lazy::new(|| container::Id::new("inner"));
