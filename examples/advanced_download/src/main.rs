use iced::{
    button, scrollable, window, Align, Application, Button, Color, Column,
    Command, Container, Element, HorizontalAlignment, Length, Row, Scrollable,
    Settings, Subscription, Text,
};

use crate::download::{Download, Progress};

mod download;

const FILE_URL: &str = "https://speed.hetzner.de/100MB.bin";

enum Example {
    Loading,
    Loaded(State),
}

#[derive(Debug, Clone, Default)]
struct Task {
    name: String,
    url: String,
    button: button::State,
    state: TaskState,
}

#[derive(Debug, Clone, Default)]
struct State {
    tasks: Vec<Task>,
    downloading_urls: Vec<String>,
    scrollable: scrollable::State,
}

#[derive(Debug, Clone)]
enum Message {
    Loaded(State),
    StartDownload(String),
    CancelDownload(String),
    DownloadProgressed((String, Progress)),
}

#[derive(Debug, Clone)]
enum TaskState {
    Idle,
    Downloading(f32),
    Finished,
    Error,
}

impl Default for TaskState {
    fn default() -> Self {
        Self::Idle
    }
}

impl Task {
    fn view(&mut self) -> Element<Message> {
        Row::new()
            .spacing(20)
            .push(Text::new(&self.name))
            .push(match self.state {
                TaskState::Idle => {
                    Button::new(&mut self.button, Text::new("Download"))
                        .on_press(Message::StartDownload(
                            (*self.url).to_string(),
                        ))
                }
                TaskState::Downloading(progress) => Button::new(
                    &mut self.button,
                    Text::new(format!("{:.2}%", progress)),
                )
                .on_press(Message::CancelDownload((*self.url).to_string())),
                TaskState::Finished => {
                    Button::new(&mut self.button, Text::new("Downloaded"))
                }
                TaskState::Error => {
                    Button::new(&mut self.button, Text::new("Errored"))
                        .on_press(Message::StartDownload(
                            (*self.url).to_string(),
                        ))
                }
            })
            .into()
    }
}

impl Example {
    async fn load_data() -> State {
        let tasks: Vec<_> = (1..=10)
            .map(|n| Task {
                name: format!("File {:0>2}", n),
                url: format!("{}?{}", FILE_URL, n),
                ..Task::default()
            })
            .collect();
        State {
            tasks,
            ..State::default()
        }
    }
}

impl Application for Example {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Example, Command<Message>) {
        (
            Example::Loading,
            Command::perform(Example::load_data(), Message::Loaded),
        )
    }

    fn title(&self) -> String {
        "Advanced Download".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match self {
            Example::Loading => {
                if let Message::Loaded(state) = message {
                    *self = Example::Loaded(state)
                }
            }
            Example::Loaded(State {
                tasks,
                downloading_urls,
                ..
            }) => match message {
                Message::StartDownload(url) => {
                    if let Some(task) = tasks.iter_mut().find(|t| t.url == url)
                    {
                        downloading_urls.push(url);
                        task.state = TaskState::Downloading(0f32);
                    }
                }
                Message::DownloadProgressed((url, progress)) => {
                    if let Some(task) = tasks.iter_mut().find(|t| t.url == url)
                    {
                        if let TaskState::Downloading(p) = &mut task.state {
                            match progress {
                                Progress::Started => *p = 0.0,
                                Progress::Advanced(percentage) => {
                                    *p = percentage
                                }
                                Progress::Finished(_bytes) => {
                                    if let Some(position) = downloading_urls
                                        .iter()
                                        .position(|u| u == &url)
                                    {
                                        downloading_urls.remove(position);
                                        task.state = TaskState::Finished;
                                    }
                                }
                                Progress::Errored => {
                                    task.state = TaskState::Error;
                                }
                            }
                        }
                    }
                }
                Message::CancelDownload(url) => {
                    if let Some(task) = tasks.iter_mut().find(|t| t.url == url)
                    {
                        downloading_urls.remove(
                            downloading_urls
                                .iter()
                                .position(|u| u == &url)
                                .unwrap(),
                        );
                        task.state = TaskState::Idle;
                    }
                }
                _ => {}
            },
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        match self {
            Example::Loading => Subscription::none(),
            Example::Loaded(State {
                downloading_urls, ..
            }) => Subscription::batch(downloading_urls.iter().map(|url| {
                Subscription::from_recipe(Download {
                    url: url.to_string(),
                })
                .map(Message::DownloadProgressed)
            })),
        }
    }

    fn view(&mut self) -> Element<Self::Message> {
        match self {
            Example::Loading => Container::new(
                Column::new()
                    .padding(80)
                    .spacing(20)
                    .push(
                        Text::new("Advanced Download")
                            .width(Length::Fill)
                            .size(40)
                            .horizontal_alignment(HorizontalAlignment::Center)
                            .color(Color::from_rgb8(16, 93, 208)),
                    )
                    .push(
                        Text::new("Loading...")
                            .width(Length::Fill)
                            .size(28)
                            .horizontal_alignment(HorizontalAlignment::Center)
                            .color(Color::from_rgb(0.3, 0.3, 0.3)),
                    ),
            )
            .width(Length::Fill)
            .center_x()
            .into(),
            Example::Loaded(State {
                tasks, scrollable, ..
            }) => {
                let list = tasks.iter_mut().fold(
                    Column::new()
                        .width(Length::Fill)
                        .spacing(20)
                        .align_items(Align::Center),
                    |column, task| column.push(task.view()),
                );
                Container::new(
                    Scrollable::new(scrollable).padding(40).spacing(40).align_items(Align::Center)
                        .push(Text::new("Download multiple files asynchronously, click again to cancel the download task.")
                            .horizontal_alignment(HorizontalAlignment::Center)
                            .width(Length::Units(400)))
                        .push(list),
                )
                    .width(Length::Fill)
                    .center_x()
                    .into()
            }
        }
    }
}

fn main() {
    Example::run(Settings {
        window: window::Settings {
            size: (800, 600),
            resizable: true,
            decorations: true,
        },
        flags: (),
        default_font: None,
        antialiasing: false,
    });
}
