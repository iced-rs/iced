use iced::{
    button, executor, Alignment, Application, Button, Column, Command,
    Container, Element, Length, ProgressBar, Settings, Subscription, Text,
};

mod download;

pub fn main() -> iced::Result {
    Example::run(Settings::default())
}

#[derive(Debug)]
struct Example {
    downloads: Vec<Download>,
    last_id: usize,
    add: button::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    Add,
    Download(usize),
    DownloadProgressed((usize, download::Progress)),
}

impl Application for Example {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Example, Command<Message>) {
        (
            Example {
                downloads: vec![Download::new(0)],
                last_id: 0,
                add: button::State::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Download progress - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Add => {
                self.last_id = self.last_id + 1;

                self.downloads.push(Download::new(self.last_id));
            }
            Message::Download(index) => {
                if let Some(download) = self.downloads.get_mut(index) {
                    download.start();
                }
            }
            Message::DownloadProgressed((id, progress)) => {
                if let Some(download) =
                    self.downloads.iter_mut().find(|download| download.id == id)
                {
                    download.progress(progress);
                }
            }
        };

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(self.downloads.iter().map(Download::subscription))
    }

    fn view(&mut self) -> Element<Message> {
        let downloads = self
            .downloads
            .iter_mut()
            .fold(Column::new().spacing(20), |column, download| {
                column.push(download.view())
            })
            .push(
                Button::new(&mut self.add, Text::new("Add another download"))
                    .on_press(Message::Add)
                    .padding(10),
            )
            .align_items(Alignment::End);

        Container::new(downloads)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .padding(20)
            .into()
    }
}

#[derive(Debug)]
struct Download {
    id: usize,
    state: State,
}

#[derive(Debug)]
enum State {
    Idle { button: button::State },
    Downloading { progress: f32 },
    Finished { button: button::State },
    Errored { button: button::State },
}

impl Download {
    pub fn new(id: usize) -> Self {
        Download {
            id,
            state: State::Idle {
                button: button::State::new(),
            },
        }
    }

    pub fn start(&mut self) {
        match self.state {
            State::Idle { .. }
            | State::Finished { .. }
            | State::Errored { .. } => {
                self.state = State::Downloading { progress: 0.0 };
            }
            _ => {}
        }
    }

    pub fn progress(&mut self, new_progress: download::Progress) {
        match &mut self.state {
            State::Downloading { progress } => match new_progress {
                download::Progress::Started => {
                    *progress = 0.0;
                }
                download::Progress::Advanced(percentage) => {
                    *progress = percentage;
                }
                download::Progress::Finished => {
                    self.state = State::Finished {
                        button: button::State::new(),
                    }
                }
                download::Progress::Errored => {
                    self.state = State::Errored {
                        button: button::State::new(),
                    };
                }
            },
            _ => {}
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        match self.state {
            State::Downloading { .. } => {
                download::file(self.id, "https://speed.hetzner.de/100MB.bin?")
                    .map(Message::DownloadProgressed)
            }
            _ => Subscription::none(),
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        let current_progress = match &self.state {
            State::Idle { .. } => 0.0,
            State::Downloading { progress } => *progress,
            State::Finished { .. } => 100.0,
            State::Errored { .. } => 0.0,
        };

        let progress_bar = ProgressBar::new(0.0..=100.0, current_progress);

        let control: Element<_> = match &mut self.state {
            State::Idle { button } => {
                Button::new(button, Text::new("Start the download!"))
                    .on_press(Message::Download(self.id))
                    .into()
            }
            State::Finished { button } => Column::new()
                .spacing(10)
                .align_items(Alignment::Center)
                .push(Text::new("Download finished!"))
                .push(
                    Button::new(button, Text::new("Start again"))
                        .on_press(Message::Download(self.id)),
                )
                .into(),
            State::Downloading { .. } => {
                Text::new(format!("Downloading... {:.2}%", current_progress))
                    .into()
            }
            State::Errored { button } => Column::new()
                .spacing(10)
                .align_items(Alignment::Center)
                .push(Text::new("Something went wrong :("))
                .push(
                    Button::new(button, Text::new("Try again"))
                        .on_press(Message::Download(self.id)),
                )
                .into(),
        };

        Column::new()
            .spacing(10)
            .padding(10)
            .align_items(Alignment::Center)
            .push(progress_bar)
            .push(control)
            .into()
    }
}
