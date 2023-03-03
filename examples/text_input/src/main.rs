use crate::Message::{StartTimer, TextEditModeChange};
use iced::widget::{button, column, container, row, text, text_input};
use iced::{
    executor, window, Application, Command, Element, Length, Renderer,
    Settings, Theme,
};
use tokio::time::{sleep, Duration};

fn main() -> iced::Result {
    let settings = Settings {
        window: window::Settings {
            size: (700, 100),
            ..window::Settings::default()
        },
        ..Settings::default()
    };

    Example::run(settings)
}

#[derive(Default)]
struct Example {
    data: String,
    text_edit_enabled: bool,
}

#[derive(Debug, Clone)]
enum Message {
    StartTimer,
    TextEditModeChange,
    TextInputChanged(String),
}

impl Application for Example {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        "TextInput example".into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::TextEditModeChange => {
                self.text_edit_enabled = !self.text_edit_enabled;
                Command::none()
            }
            Message::TextInputChanged(updated_text) => {
                self.data = updated_text;
                Command::none()
            }
            StartTimer => {
                let timer_f = async {
                    sleep(Duration::from_secs(3)).await;
                };
                Command::perform(timer_f, |_| TextEditModeChange)
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        let placeholder = if self.text_edit_enabled {
            "Enabled TextEdit"
        } else {
            "Disabled TextEdit"
        };

        let mut txt_input = text_input(placeholder, &self.data);

        if self.text_edit_enabled {
            txt_input = txt_input.on_change(Message::TextInputChanged);
        }

        let btn = button("Enable/Disable").on_press(StartTimer);
        let label = text(
            "The mode will be changed after 3s when the button is pressed",
        );

        let content = row![txt_input, btn].spacing(10);
        let content = column![content, label].spacing(10);

        container(content)
            .width(Length::Shrink)
            .height(Length::Shrink)
            .padding(20)
            .into()
    }
}
