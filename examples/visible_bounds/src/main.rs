use iced::event::{self, Event};
use iced::mouse;
use iced::widget::{
    column, container, horizontal_space, row, scrollable, selector, text,
    vertical_space,
};
use iced::window;
use iced::{
    Center, Color, Element, Fill, Font, Point, Rectangle, Subscription, Task,
    Theme,
};

pub fn main() -> iced::Result {
    iced::application(Example::default, Example::update, Example::view)
        .subscription(Example::subscription)
        .theme(Theme::Dark)
        .run()
}

#[derive(Default)]
struct Example {
    mouse_position: Option<Point>,
    outer_bounds: Option<Rectangle>,
    inner_bounds: Option<Rectangle>,
}

#[derive(Debug, Clone)]
enum Message {
    MouseMoved(Point),
    WindowResized,
    Scrolled,
    OuterFound(Option<selector::Match>),
    InnerFound(Option<selector::Match>),
}

impl Example {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::MouseMoved(position) => {
                self.mouse_position = Some(position);

                Task::none()
            }
            Message::Scrolled | Message::WindowResized => Task::batch(vec![
                selector::find_by_id(OUTER_CONTAINER).map(Message::OuterFound),
                selector::find_by_id(INNER_CONTAINER).map(Message::InnerFound),
            ]),
            Message::OuterFound(outer) => {
                self.outer_bounds =
                    outer.as_ref().and_then(selector::Bounded::visible_bounds);

                Task::none()
            }
            Message::InnerFound(inner) => {
                self.inner_bounds =
                    inner.as_ref().and_then(selector::Bounded::visible_bounds);

                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
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
                        .id(OUTER_CONTAINER)
                        .padding(40)
                        .style(container::rounded_box),
                    vertical_space().height(400),
                    scrollable(
                        column![
                            text("Scroll me!"),
                            vertical_space().height(400),
                            container(text("I am the inner container!"))
                                .id(INNER_CONTAINER)
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

const OUTER_CONTAINER: &str = "outer";
const INNER_CONTAINER: &str = "inner";
