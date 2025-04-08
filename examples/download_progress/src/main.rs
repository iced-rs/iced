mod download;

use download::download;

use iced::task;
use iced::widget::{Column, button, center, column, progress_bar, text};
use iced::{Center, Element, Function, Right, Task};

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
                if let Some(download) =
                    self.downloads.iter_mut().find(|download| download.id == id)
                {
                    download.update(update);
                }

                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let downloads =
            Column::with_children(self.downloads.iter().map(Download::view))
                .push(
                    button("Add another download")
                        .on_press(Message::Add)
                        .padding(10),
                )
                .spacing(20)
                .align_x(Right);

        center(downloads).padding(20).into()
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

    pub fn view(&self) -> Element<Message> {
        let current_progress = match &self.state {
            State::Idle => 0.0,
            State::Downloading { progress, .. } => *progress,
            State::Finished => 100.0,
            State::Errored => 0.0,
        };

        let progress_bar = progress_bar(0.0..=100.0, current_progress);

        let control: Element<_> = match &self.state {
            State::Idle => button("Start the download!")
                .on_press(Message::Download(self.id))
                .into(),
            State::Finished => {
                column!["Download finished!", button("Start again")]
                    .spacing(10)
                    .align_x(Center)
                    .into()
            }
            State::Downloading { .. } => {
                text!("Downloading... {current_progress:.2}%").into()
            }
            State::Errored => column![
                "Something went wrong :(",
                button("Try again").on_press(Message::Download(self.id)),
            ]
            .spacing(10)
            .align_x(Center)
            .into(),
        };

        Column::new()
            .spacing(10)
            .padding(10)
            .align_x(Center)
            .push(progress_bar)
            .push(control)
            .into()
    }
}
