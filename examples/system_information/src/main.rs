use iced::{
    executor, system, Application, Column, Command, Container, Element, Length,
    Settings, Text,
};

use bytesize::ByteSize;

pub fn main() -> iced::Result {
    Example::run(Settings::default())
}

enum Example {
    Loading,
    Loaded { information: system::Information },
    Unsupported,
}

#[derive(Debug)]
enum Message {
    InformationReceived(Option<system::Information>),
}

impl Application for Example {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self::Loading,
            system::information(Message::InformationReceived),
        )
    }

    fn title(&self) -> String {
        String::from("System Information - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::InformationReceived(information) => {
                if let Some(information) = information {
                    *self = Self::Loaded { information };
                } else {
                    *self = Self::Unsupported;
                }
            }
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        let content: Element<Message> = match self {
            Example::Loading => Text::new("Loading...").size(40).into(),
            Example::Loaded { information } => {
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
                    "Memory (total): {}",
                    format!(
                        "{} kb ({})",
                        information.memory_total, memory_readable
                    )
                ));

                let graphics_adapter = Text::new(format!(
                    "Graphics adapter: {}",
                    information.graphics_adapter
                ));

                let graphics_backend = Text::new(format!(
                    "Graphics backend: {}",
                    information.graphics_backend
                ));

                Column::with_children(vec![
                    system_name.into(),
                    system_kernel.into(),
                    system_version.into(),
                    cpu_brand.into(),
                    cpu_cores.into(),
                    memory_total.into(),
                    graphics_adapter.into(),
                    graphics_backend.into(),
                ])
                .into()
            }
            Example::Unsupported => Text::new("Unsupported!").size(20).into(),
        };

        Container::new(content)
            .center_x()
            .center_y()
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
