use iced::multi_window::{self, Application};
use iced::widget::{button, column, container, scrollable, text, text_input};
use iced::{
    executor, window, Alignment, Command, Element, Length, Settings, Theme,
};
use std::collections::HashMap;

fn main() -> iced::Result {
    Example::run(Settings::default())
}

#[derive(Default)]
struct Example {
    windows_count: usize,
    windows: HashMap<window::Id, Window>,
}

struct Window {
    id: window::Id,
    title: String,
    scale_input: String,
    current_scale: f64,
}

#[derive(Debug, Clone)]
enum Message {
    ScaleInputChanged(window::Id, String),
    ScaleChanged(window::Id, String),
    TitleChanged(window::Id, String),
    CloseWindow(window::Id),
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
                windows_count: 0,
                windows: HashMap::from([(
                    window::Id::MAIN,
                    Window::new(window::Id::MAIN),
                )]),
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
            }
            Message::ScaleChanged(id, scale) => {
                let window =
                    self.windows.get_mut(&id).expect("Window not found!");

                window.current_scale = scale
                    .parse::<f64>()
                    .unwrap_or(window.current_scale)
                    .clamp(0.5, 5.0);
            }
            Message::TitleChanged(id, title) => {
                let window =
                    self.windows.get_mut(&id).expect("Window not found.");

                window.title = title;
            }
            Message::CloseWindow(id) => {
                return window::close(id);
            }
            Message::NewWindow => {
                self.windows_count += 1;
                let id = window::Id::new(self.windows_count);
                self.windows.insert(id, Window::new(id));

                return window::spawn(id, window::Settings::default());
            }
        }

        Command::none()
    }

    fn view(&self, window: window::Id) -> Element<Message> {
        let window = self
            .windows
            .get(&window)
            .map(|window| window.view())
            .unwrap();

        container(window)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn scale_factor(&self, window: window::Id) -> f64 {
        self.windows
            .get(&window)
            .map(|window| window.current_scale)
            .unwrap_or(1.0)
    }

    fn close_requested(&self, window: window::Id) -> Self::Message {
        Message::CloseWindow(window)
    }
}

impl Window {
    fn new(id: window::Id) -> Self {
        Self {
            id,
            title: "Window".to_string(),
            scale_input: "1.0".to_string(),
            current_scale: 1.0,
        }
    }

    fn view(&self) -> Element<Message> {
        window_view(self.id, &self.scale_input, &self.title)
    }
}

fn window_view<'a>(
    id: window::Id,
    scale_input: &'a str,
    title: &'a str,
) -> Element<'a, Message> {
    let scale_input = column![
        text("Window scale factor:"),
        text_input("Window Scale", scale_input, move |msg| {
            Message::ScaleInputChanged(id, msg)
        })
        .on_submit(Message::ScaleChanged(id, scale_input.to_string()))
    ];

    let title_input = column![
        text("Window title:"),
        text_input("Window Title", title, move |msg| {
            Message::TitleChanged(id, msg)
        })
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
