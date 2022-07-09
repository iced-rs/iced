use iced::{
    button, executor, system, Application, Button, Column, Command, Container,
    Element, Length, Settings, Text, Theme,
};

use bytesize::ByteSize;

pub fn main() -> iced::Result {
    Example::run(Settings::default())
}

enum Example {
    Loading,
    Loaded {
        information: system::Information,
        refresh_button: button::State,
    },
}

#[derive(Clone, Debug)]
enum Message {
    InformationReceived(system::Information),
    Refresh,
}

impl Application for Example {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self::Loading,
            system::fetch_information(Message::InformationReceived),
        )
    }

    fn title(&self) -> String {
        String::from("System Information - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Refresh => {
                *self = Self::Loading;

                return system::fetch_information(Message::InformationReceived);
            }
            Message::InformationReceived(information) => {
                let refresh_button = button::State::new();
                *self = Self::Loaded {
                    information,
                    refresh_button,
                };
            }
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        let content: Element<Message> = match self {
            Example::Loading => Text::new("Loading...").size(40).into(),
            Example::Loaded {
                information,
                refresh_button,
            } => {
                let system_name = Text::new(format!(
                    "System name: {}",
                    information
                        .system_name
                        .as_ref()
                        .unwrap_or(&"unknown".to_string())
                ));

                let system_kernel = Text::new(format!(
                    "System kernel: {}",
                    information
                        .system_kernel
                        .as_ref()
                        .unwrap_or(&"unknown".to_string())
                ));

                let system_version = Text::new(format!(
                    "System version: {}",
                    information
                        .system_version
                        .as_ref()
                        .unwrap_or(&"unknown".to_string())
                ));

                let cpu_brand = Text::new(format!(
                    "Processor brand: {}",
                    information.cpu_brand
                ));

                let cpu_cores = Text::new(format!(
                    "Processor cores: {}",
                    information
                        .cpu_cores
                        .map_or("unknown".to_string(), |cores| cores
                            .to_string())
                ));

                let memory_readable =
                    ByteSize::kb(information.memory_total).to_string();

                let memory_total = Text::new(format!(
                    "Memory (total): {} kb ({})",
                    information.memory_total, memory_readable
                ));

                let memory_text = if let Some(memory_used) =
                    information.memory_used
                {
                    let memory_readable = ByteSize::kb(memory_used).to_string();

                    format!("{} kb ({})", memory_used, memory_readable)
                } else {
                    String::from("None")
                };

                let memory_used =
                    Text::new(format!("Memory (used): {}", memory_text));

                let graphics_adapter = Text::new(format!(
                    "Graphics adapter: {}",
                    information.graphics_adapter
                ));

                let graphics_backend = Text::new(format!(
                    "Graphics backend: {}",
                    information.graphics_backend
                ));

                Column::with_children(vec![
                    system_name.size(30).into(),
                    system_kernel.size(30).into(),
                    system_version.size(30).into(),
                    cpu_brand.size(30).into(),
                    cpu_cores.size(30).into(),
                    memory_total.size(30).into(),
                    memory_used.size(30).into(),
                    graphics_adapter.size(30).into(),
                    graphics_backend.size(30).into(),
                    Button::new(refresh_button, Text::new("Refresh"))
                        .on_press(Message::Refresh)
                        .into(),
                ])
                .spacing(10)
                .into()
            }
        };

        Container::new(content)
            .center_x()
            .center_y()
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
