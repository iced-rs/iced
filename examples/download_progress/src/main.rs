use iced::{
    button, executor, Align, Application, Button, Column, Command, Container,
    Element, Length, ProgressBar, Settings, Subscription, Text,
};

mod downloader;

pub fn main() {
    Downloader::run(Settings::default())
}

#[derive(Debug, Default)]
struct Downloader {
    // Whether to start the download or not.
    enabled: bool,
    // The current percentage of the download
    current_progress: u64,

    btn_state: button::State,
}

#[derive(Debug)]
pub enum Message {
    DownloadUpdate(downloader::DownloadMessage),
    Interaction(Interaction),
}

// For explanation of why we use an Interaction enum see here:
// https://github.com/hecrj/iced/pull/155#issuecomment-573523405
#[derive(Debug, Clone)]
pub enum Interaction {
    // User pressed the button to start the download
    StartDownload,
}

impl Application for Downloader {
    type Executor = executor::Default;
    type Message = Message;

    fn new() -> (Downloader, Command<Message>) {
        (Downloader::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Download Progress - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Interaction(action) => match action {
                Interaction::StartDownload => {
                    self.enabled = true;
                }
            },
            Message::DownloadUpdate(update) => match update {
                downloader::DownloadMessage::Downloading(percentage) => {
                    self.current_progress = percentage;
                }
                downloader::DownloadMessage::Done => {
                    self.current_progress = 100;
                    self.enabled = false;
                }
                _ => {}
            },
        };

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.enabled {
            downloader::file("https://speed.hetzner.de/100MB.bin")
                .map(Message::DownloadUpdate)
        } else {
            Subscription::none()
        }
    }

    fn view(&mut self) -> Element<Message> {
        // Construct widgets

        let toggle_text = match self.enabled {
            true => "Downloading...",
            false => "Start the download!",
        };

        let toggle: Element<Interaction> =
            Button::new(&mut self.btn_state, Text::new(toggle_text))
                .on_press(Interaction::StartDownload)
                .into();

        let progress_bar =
            ProgressBar::new(0.0..=100.0, self.current_progress as f32);

        let progress_text = &match self.enabled {
            true => format!("Downloading {}%", self.current_progress),
            false => "Ready to rock!".into(),
        };

        // Construct layout
        let content = Column::new()
            .align_items(Align::Center)
            .spacing(20)
            .padding(20)
            .push(Text::new(progress_text))
            .push(progress_bar)
            .push(toggle.map(Message::Interaction));

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
