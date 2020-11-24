use iced::{
    button, executor, Align, Application, Button, Column, Command, Container,
    Element, Length, ProgressBar, Settings, Subscription, Text,
};

mod download;

pub fn main() -> iced::Result {
    Example::run(Settings::default())
}

#[derive(Debug)]
enum Example {
    Idle { button: button::State },
    Downloading { progress: f32 },
    Finished { button: button::State },
    Errored { button: button::State },
}

#[derive(Debug, Clone)]
pub enum Message {
    Download,
    DownloadProgressed(download::Progress),
}

impl Application for Example {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Example, Command<Message>) {
        (
            Example::Idle {
                button: button::State::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Download progress - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Download => match self {
                Example::Idle { .. }
                | Example::Finished { .. }
                | Example::Errored { .. } => {
                    *self = Example::Downloading { progress: 0.0 };
                }
                _ => {}
            },
            Message::DownloadProgressed(message) => match self {
                Example::Downloading { progress } => match message {
                    download::Progress::Started => {
                        *progress = 0.0;
                    }
                    download::Progress::Advanced(percentage) => {
                        *progress = percentage;
                    }
                    download::Progress::Finished => {
                        *self = Example::Finished {
                            button: button::State::new(),
                        }
                    }
                    download::Progress::Errored => {
                        *self = Example::Errored {
                            button: button::State::new(),
                        };
                    }
                },
                _ => {}
            },
        };

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        match self {
            Example::Downloading { .. } => {
                download::file("https://speed.hetzner.de/100MB.bin")
                    .map(Message::DownloadProgressed)
            }
            _ => Subscription::none(),
        }
    }

    fn view(&mut self) -> Element<Message> {
        let current_progress = match self {
            Example::Idle { .. } => 0.0,
            Example::Downloading { progress } => *progress,
            Example::Finished { .. } => 100.0,
            Example::Errored { .. } => 0.0,
        };

        let progress_bar = ProgressBar::new(0.0..=100.0, current_progress);

        let control: Element<_> = match self {
            Example::Idle { button } => {
                Button::new(button, Text::new("Start the download!"))
                    .on_press(Message::Download)
                    .into()
            }
            Example::Finished { button } => Column::new()
                .spacing(10)
                .align_items(Align::Center)
                .push(Text::new("Download finished!"))
                .push(
                    Button::new(button, Text::new("Start again"))
                        .on_press(Message::Download),
                )
                .into(),
            Example::Downloading { .. } => {
                Text::new(format!("Downloading... {:.2}%", current_progress))
                    .into()
            }
            Example::Errored { button } => Column::new()
                .spacing(10)
                .align_items(Align::Center)
                .push(Text::new("Something went wrong :("))
                .push(
                    Button::new(button, Text::new("Try again"))
                        .on_press(Message::Download),
                )
                .into(),
        };

        let content = Column::new()
            .spacing(10)
            .padding(10)
            .align_items(Align::Center)
            .push(progress_bar)
            .push(control);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
