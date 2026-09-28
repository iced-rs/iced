mod download;

use download::download;

use iced::task;
use iced::widget::{button, center, column, progress_bar, text};
use iced::{Center, Function, Right, Task, Widget};

pub fn main() -> iced::Result {
    iced::application(Example::default, Example::update, Example::view).run()
}

#[derive(Debug)]
struct Example {
    downloads: Vec<Download>,
    last_id: usize,
}

#[derive(Debug, Clone)]
pub enum Message {
    Add,
    Download(usize),
    DownloadUpdated(usize, Update),
}

impl Example {
    fn new() -> Self {
        Self {
            downloads: vec![Download::new(0)],
            last_id: 0,
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Add => {
                self.last_id += 1;

                self.downloads.push(Download::new(self.last_id));

                Task::none()
            }
            Message::Download(index) => {
                let Some(download) = self.downloads.get_mut(index) else {
                    return Task::none();
                };

                let task = download.start();

                task.map(Message::DownloadUpdated.with(index))
            }
            Message::DownloadUpdated(id, update) => {
                if let Some(download) = self.downloads.iter_mut().find(|download| download.id == id)
                {
                    download.update(update);
                }

                Task::none()
            }
        }
    }

    fn view(&self) -> impl Widget<Message> {
        let downloads = column![
            column(self.downloads.iter().map(Download::view)).spacing(20),
            button("Add another download")
                .on_press(Message::Add)
                .padding(10)
        ]
        .spacing(20)
        .align_x(Right);

        center(downloads).padding(20)
    }
}

impl Default for Example {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct Download {
    id: usize,
    state: State,
}

#[derive(Debug, Clone)]
pub enum Update {
    Downloading(download::Progress),
    Finished(Result<(), download::Error>),
}

#[derive(Debug)]
enum State {
    Idle,
    Downloading { progress: f32, _task: task::Handle },
    Finished,
    Errored,
}

impl Download {
    pub fn new(id: usize) -> Self {
        Download {
            id,
            state: State::Idle,
        }
    }

    pub fn start(&mut self) -> Task<Update> {
        match self.state {
            State::Idle | State::Finished | State::Errored => {
                let (task, handle) = Task::sip(
                    download(
                        "https://huggingface.co/\
                        mattshumer/Reflection-Llama-3.1-70B/\
                        resolve/main/model-00001-of-00162.safetensors",
                    ),
                    Update::Downloading,
                    Update::Finished,
                )
                .abortable();

                self.state = State::Downloading {
                    progress: 0.0,
                    _task: handle.abort_on_drop(),
                };

                task
            }
            State::Downloading { .. } => Task::none(),
        }
    }

    pub fn update(&mut self, update: Update) {
        if let State::Downloading { progress, .. } = &mut self.state {
            match update {
                Update::Downloading(new_progress) => {
                    *progress = new_progress.percent;
                }
                Update::Finished(result) => {
                    self.state = if result.is_ok() {
                        State::Finished
                    } else {
                        State::Errored
                    };
                }
            }
        }
    }

    pub fn view(&self) -> impl Widget<Message> {
        let current_progress = match &self.state {
            State::Idle => 0.0,
            State::Downloading { progress, .. } => *progress,
            State::Finished => 100.0,
            State::Errored => 0.0,
        };

        let progress_bar = progress_bar(0.0..=100.0, current_progress);

        let control = match &self.state {
            State::Idle => button("Start the download!")
                .on_press(Message::Download(self.id))
                .boxed(),
            State::Finished => column!["Download finished!", button("Start again")]
                .spacing(10)
                .align_x(Center)
                .boxed(),
            State::Downloading { .. } => text!("Downloading... {current_progress:.2}%").boxed(),
            State::Errored => column![
                "Something went wrong :(",
                button("Try again").on_press(Message::Download(self.id)),
            ]
            .spacing(10)
            .align_x(Center)
            .boxed(),
        };

        column![progress_bar, control]
            .spacing(10)
            .padding(10)
            .align_x(Center)
    }
}
