use iced::widget::{button, column, container, text};
use iced::{
    executor, system, Application, Command, Element, Length, Settings, Theme,
};

use bytesize::ByteSize;

pub fn main() -> iced::Result {
    Example::run(Settings::default())
}

#[allow(clippy::large_enum_variant)]
enum Example {
    Loading,
    Loaded { information: system::Information },
}

#[derive(Clone, Debug)]
#[allow(clippy::large_enum_variant)]
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
                *self = Self::Loaded { information };
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let content: Element<_> = match self {
            Example::Loading => text("Loading...").size(40).into(),
            Example::Loaded { information } => {
                let system_name = text(format!(
                    "System name: {}",
                    information
                        .system_name
                        .as_ref()
                        .unwrap_or(&"unknown".to_string())
                ));

                let system_kernel = text(format!(
                    "System kernel: {}",
                    information
                        .system_kernel
                        .as_ref()
                        .unwrap_or(&"unknown".to_string())
                ));

                let system_version = text(format!(
                    "System version: {}",
                    information
                        .system_version
                        .as_ref()
                        .unwrap_or(&"unknown".to_string())
                ));

                let system_short_version = text(format!(
                    "System short version: {}",
                    information
                        .system_short_version
                        .as_ref()
                        .unwrap_or(&"unknown".to_string())
                ));

                let cpu_brand =
                    text(format!("Processor brand: {}", information.cpu_brand));

                let cpu_cores = text(format!(
                    "Processor cores: {}",
                    information
                        .cpu_cores
                        .map_or("unknown".to_string(), |cores| cores
                            .to_string())
                ));

                let memory_readable =
                    ByteSize::kb(information.memory_total).to_string();

                let memory_total = text(format!(
                    "Memory (total): {} kb ({})",
                    information.memory_total, memory_readable
                ));

                let memory_text = if let Some(memory_used) =
                    information.memory_used
                {
                    let memory_readable = ByteSize::kb(memory_used).to_string();

                    format!("{memory_used} kb ({memory_readable})")
                } else {
                    String::from("None")
                };

                let memory_used = text(format!("Memory (used): {memory_text}"));

                let graphics_adapter = text(format!(
                    "Graphics adapter: {}",
                    information.graphics_adapter
                ));

                let graphics_backend = text(format!(
                    "Graphics backend: {}",
                    information.graphics_backend
                ));

                column![
                    system_name.size(30),
                    system_kernel.size(30),
                    system_version.size(30),
                    system_short_version.size(30),
                    cpu_brand.size(30),
                    cpu_cores.size(30),
                    memory_total.size(30),
                    memory_used.size(30),
                    graphics_adapter.size(30),
                    graphics_backend.size(30),
                    button("Refresh").on_press(Message::Refresh)
                ]
                .spacing(10)
                .into()
            }
        };

        container(content)
            .center_x()
            .center_y()
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
