use iced::event;
use iced::executor;
use iced::multi_window::{self, Application};
use iced::widget::{button, column, container, scrollable, text, text_input};
use iced::window;
use iced::{
    Alignment, Command, Element, Length, Point, Settings, Subscription, Theme,
    Vector,
};

use std::collections::HashMap;

fn main() -> iced::Result {
    Example::run(Settings::default())
}

#[derive(Default)]
struct Example {
    windows: HashMap<window::Id, Window>,
    next_window_pos: window::Position,
}

#[derive(Debug)]
struct Window {
    title: String,
    scale_input: String,
    current_scale: f64,
    theme: Theme,
    input_id: iced::widget::text_input::Id,
}

#[derive(Debug, Clone)]
enum Message {
    ScaleInputChanged(window::Id, String),
    ScaleChanged(window::Id, String),
    TitleChanged(window::Id, String),
    CloseWindow(window::Id),
    WindowOpened(window::Id, Option<Point>),
    WindowClosed(window::Id),
    NewWindow,
}

impl multi_window::Application for Example {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Example {
                windows: HashMap::from([(window::Id::MAIN, Window::new(1))]),
                next_window_pos: window::Position::Default,
            },
            Command::none(),
        )
    }

    fn title(&self, window: window::Id) -> String {
        self.windows
            .get(&window)
            .map(|window| window.title.clone())
            .unwrap_or("Example".to_string())
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ScaleInputChanged(id, scale) => {
                let window =
                    self.windows.get_mut(&id).expect("Window not found!");
                window.scale_input = scale;

                Command::none()
            }
            Message::ScaleChanged(id, scale) => {
                let window =
                    self.windows.get_mut(&id).expect("Window not found!");

                window.current_scale = scale
                    .parse::<f64>()
                    .unwrap_or(window.current_scale)
                    .clamp(0.5, 5.0);

                Command::none()
            }
            Message::TitleChanged(id, title) => {
                let window =
                    self.windows.get_mut(&id).expect("Window not found.");

                window.title = title;

                Command::none()
            }
            Message::CloseWindow(id) => window::close(id),
            Message::WindowClosed(id) => {
                self.windows.remove(&id);
                Command::none()
            }
            Message::WindowOpened(id, position) => {
                if let Some(position) = position {
                    self.next_window_pos = window::Position::Specific(
                        position + Vector::new(20.0, 20.0),
                    );
                }

                if let Some(window) = self.windows.get(&id) {
                    text_input::focus(window.input_id.clone())
                } else {
                    Command::none()
                }
            }
            Message::NewWindow => {
                let count = self.windows.len() + 1;

                let (id, spawn_window) = window::spawn(window::Settings {
                    position: self.next_window_pos,
                    exit_on_close_request: count % 2 == 0,
                    ..Default::default()
                });

                self.windows.insert(id, Window::new(count));

                spawn_window
            }
        }
    }

    fn view(&self, window: window::Id) -> Element<Message> {
        let content = self.windows.get(&window).unwrap().view(window);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn theme(&self, window: window::Id) -> Self::Theme {
        self.windows.get(&window).unwrap().theme.clone()
    }

    fn scale_factor(&self, window: window::Id) -> f64 {
        self.windows
            .get(&window)
            .map(|window| window.current_scale)
            .unwrap_or(1.0)
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        event::listen_with(|event, _| {
            if let iced::Event::Window(id, window_event) = event {
                match window_event {
                    window::Event::CloseRequested => {
                        Some(Message::CloseWindow(id))
                    }
                    window::Event::Opened { position, .. } => {
                        Some(Message::WindowOpened(id, position))
                    }
                    window::Event::Closed => Some(Message::WindowClosed(id)),
                    _ => None,
                }
            } else {
                None
            }
        })
    }
}

impl Window {
    fn new(count: usize) -> Self {
        Self {
            title: format!("Window_{}", count),
            scale_input: "1.0".to_string(),
            current_scale: 1.0,
            theme: if count % 2 == 0 {
                Theme::Light
            } else {
                Theme::Dark
            },
            input_id: text_input::Id::unique(),
        }
    }

    fn view(&self, id: window::Id) -> Element<Message> {
        let scale_input = column![
            text("Window scale factor:"),
            text_input("Window Scale", &self.scale_input)
                .on_input(move |msg| { Message::ScaleInputChanged(id, msg) })
                .on_submit(Message::ScaleChanged(
                    id,
                    self.scale_input.to_string()
                ))
        ];

        let title_input = column![
            text("Window title:"),
            text_input("Window Title", &self.title)
                .on_input(move |msg| { Message::TitleChanged(id, msg) })
                .id(self.input_id.clone())
        ];

        let new_window_button =
            button(text("New Window")).on_press(Message::NewWindow);

        let content = scrollable(
            column![scale_input, title_input, new_window_button]
                .spacing(50)
                .width(Length::Fill)
                .align_items(Alignment::Center),
        );

        container(content).width(200).center_x().into()
    }
}
