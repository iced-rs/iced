use url::Url;

use iced::{
    button, scrollable, Align, Application, Button, Column, Command, Container,
    Element, HorizontalAlignment, Length, ProgressBar, Row, Scrollable,
    Settings, Space, Subscription, Text,
};

use crate::download::{Download, Progress};

mod download;

pub fn main() -> iced::Result {
    Example::run(Settings::default())
}

const FILE_URL: &str = "https://speed.hetzner.de/100MB.bin";

enum Example {
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
    scrollable: scrollable::State,
}

#[derive(Debug, Clone)]
struct Message {
    url: String,
    download_message: DownloadMessage,
}

impl Message {
    fn from(url: String, download_message: DownloadMessage) -> Self {
        Message {
            url,
            download_message,
        }
    }
}

#[derive(Debug, Clone)]
enum DownloadMessage {
    StartDownload,
    CancelDownload,
    DownloadProgressed(Progress),
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
        let download_text = |text| {
            Text::new(text)
                .size(15)
                .horizontal_alignment(HorizontalAlignment::Center)
        };
        Row::new()
            .spacing(20)
            .align_items(Align::Center)
            .push(Text::new(&self.name))
            .push(match self.state {
                TaskState::Downloading(progress) => {
                    ProgressBar::new(0.0..=100.0, progress)
                        .width(Length::Units(150))
                        .height(Length::Units(18))
                        .into()
                }
                _ => {
                    let element: Element<_> =
                        Space::new(Length::Units(150), Length::Shrink).into();
                    element
                }
            })
            .push(
                match self.state {
                    TaskState::Idle => {
                        Button::new(&mut self.button, download_text("Download"))
                            .on_press(Message::from(
                                (*self.url).to_string(),
                                DownloadMessage::StartDownload,
                            ))
                    }
                    TaskState::Downloading(progress) => Button::new(
                        &mut self.button,
                        download_text(&format!("{:.2}%", progress)),
                    )
                    .on_press(Message::from(
                        (*self.url).to_string(),
                        DownloadMessage::CancelDownload,
                    )),
                    TaskState::Finished => Button::new(
                        &mut self.button,
                        download_text("Downloaded"),
                    ),
                    TaskState::Error => {
                        Button::new(&mut self.button, download_text("Errored"))
                            .on_press(Message::from(
                                (*self.url).to_string(),
                                DownloadMessage::StartDownload,
                            ))
                    }
                }
                .width(Length::Units(85)),
            )
            .into()
    }
}

impl Application for Example {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Example, Command<Message>) {
        let tasks: Vec<_> = (1..=10)
            .map(|n| Task {
                name: format!("File {:0>2}", n),
                url: format!("{}?{}", FILE_URL, n),
                ..Task::default()
            })
            .collect();

        (
            Example::Loaded(State {
                tasks,
                ..State::default()
            }),
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Download progress - Iced")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match self {
            Example::Loaded(State { tasks, .. }) => {
                let url = message.url;
                if let Some(task) = tasks.iter_mut().find(|t| t.url == url) {
                    match message.download_message {
                        DownloadMessage::StartDownload => {
                            task.state = TaskState::Downloading(0f32);
                        }
                        DownloadMessage::DownloadProgressed(progress) => {
                            if let TaskState::Downloading(p) = &mut task.state {
                                match progress {
                                    Progress::Started => *p = 0.0,
                                    Progress::Advanced(percentage) => {
                                        *p = percentage
                                    }
                                    Progress::Finished(_bytes) => {
                                        task.state = TaskState::Finished;
                                    }
                                    Progress::Errored => {
                                        task.state = TaskState::Error;
                                    }
                                }
                            }
                        }
                        DownloadMessage::CancelDownload => {
                            task.state = TaskState::Idle;
                        }
                    }
                }
            }
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        match self {
            Example::Loaded(State { tasks, .. }) => {
                Subscription::batch(tasks.iter().map(|task| file(&task.url)))
            }
        }
    }

    fn view(&mut self) -> Element<Self::Message> {
        match self {
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

// Just a little utility function
fn file<T: ToString>(url: T) -> iced::Subscription<Message> {
    iced::Subscription::from_recipe(Download {
        url: Url::parse(&url.to_string()).unwrap(),
    })
    .map(|(url, progress)| {
        Message::from(
            url.as_str().to_string(),
            DownloadMessage::DownloadProgressed(progress),
        )
    })
}
