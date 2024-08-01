use iced::widget::{button, center, column, text};
use iced::{system, Element, Task};

pub fn main() -> iced::Result {
    iced::application(
        "System Information - Iced",
        Example::update,
        Example::view,
    )
    .run()
}

#[derive(Default)]
#[allow(clippy::large_enum_variant)]
enum Example {
    #[default]
    Loading,
    Loaded {
        information: system::Information,
    },
}

#[derive(Clone, Debug)]
#[allow(clippy::large_enum_variant)]
enum Message {
    InformationReceived(system::Information),
    Refresh,
}

impl Example {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Refresh => {
                *self = Self::Loading;

                return system::fetch_information()
                    .map(Message::InformationReceived);
            }
            Message::InformationReceived(information) => {
                *self = Self::Loaded { information };
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<Message> {
        use bytesize::ByteSize;

        let content: Element<_> = match self {
            Example::Loading => text("Loading...").size(40).into(),
            Example::Loaded { information } => {
                let system_name = text!(
                    "System name: {}",
                    information
                        .system_name
                        .as_ref()
                        .unwrap_or(&"unknown".to_string())
                );

                let system_kernel = text!(
                    "System kernel: {}",
                    information
                        .system_kernel
                        .as_ref()
                        .unwrap_or(&"unknown".to_string())
                );

                let system_version = text!(
                    "System version: {}",
                    information
                        .system_version
                        .as_ref()
                        .unwrap_or(&"unknown".to_string())
                );

                let system_short_version = text!(
                    "System short version: {}",
                    information
                        .system_short_version
                        .as_ref()
                        .unwrap_or(&"unknown".to_string())
                );

                let cpu_brand =
                    text!("Processor brand: {}", information.cpu_brand);

                let cpu_cores = text!(
                    "Processor cores: {}",
                    information
                        .cpu_cores
                        .map_or("unknown".to_string(), |cores| cores
                            .to_string())
                );

                let memory_readable =
                    ByteSize::b(information.memory_total).to_string();

                let memory_total = text!(
                    "Memory (total): {} bytes ({memory_readable})",
                    information.memory_total,
                );

                let memory_text = if let Some(memory_used) =
                    information.memory_used
                {
                    let memory_readable = ByteSize::b(memory_used).to_string();

                    format!("{memory_used} bytes ({memory_readable})")
                } else {
                    String::from("None")
                };

                let memory_used = text!("Memory (used): {memory_text}");

                let graphics_adapter =
                    text!("Graphics adapter: {}", information.graphics_adapter);

                let graphics_backend =
                    text!("Graphics backend: {}", information.graphics_backend);

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

        center(content).into()
    }
}
