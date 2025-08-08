use std::path::Path;

use iced::{
    Alignment::Center,
    Element, Event,
    Length::{Fill, Fixed},
    Subscription, Task, Theme, border, event, keyboard, tray_icon,
    widget::{center, column, container, scrollable, text},
};

pub fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .subscription(App::subscription)
        .tray_icon(tray_icon::Settings {
            title: Some("Iced".into()),
            icon: Some(load_icon(Path::new(&format!(
                "{}/logo.png",
                env!("CARGO_MANIFEST_DIR")
            )))),
            tooltip: Some("Iced".to_string()),
            menu_items: Some(vec![
                tray_icon::MenuItem::Text {
                    id: "Hello".into(),
                    text: "Hello".into(),
                    enabled: true,
                    accelerator: Some(tray_icon::Accelerator(
                        keyboard::key::Code::KeyC,
                        keyboard::Modifiers::SHIFT,
                    )),
                },
                tray_icon::MenuItem::Check {
                    id: "Checkbox".into(),
                    text: "Checked".into(),
                    enabled: true,
                    checked: true,
                    accelerator: None,
                },
                tray_icon::MenuItem::Predefined {
                    predefined_type: tray_icon::PredefinedMenuItem::CloseWindow,
                    alternate_text: None,
                },
            ]),
        })
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    Event(Event),
}

struct App {
    menu_events: Vec<String>,
}

impl App {
    fn new() -> Self {
        Self {
            menu_events: Vec::new(),
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Event(Event::TrayIcon(event)) => match event {
                tray_icon::Event::MenuItemClicked { id } => {
                    self.menu_events.push(format!("Menu Item Clicked: {}", id));
                }
                tray_icon::Event::MouseEntered { .. } => {
                    self.menu_events.push("Mouse Entered".into());
                }
                tray_icon::Event::MouseExited { .. } => {
                    self.menu_events.push("Mouse Exited".into());
                }
                _ => {}
            },
            _ => {}
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let events =
            container(column(self.menu_events.iter().map(|e| text(e).into())))
                .style(|theme: &Theme| {
                    let palette = theme.extended_palette();

                    container::Style::default().border(
                        border::color(palette.background.strong.color).width(4),
                    )
                })
                .padding(4);
        let content = column![
            text("Tray Icon Events"),
            scrollable(events).height(Fill).width(Fixed(400.0))
        ]
        .width(Fill)
        .align_x(Center)
        .spacing(10);
        center(content).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::Event)
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn load_icon(path: &Path) -> iced::tray_icon::Icon {
    println!("{:?}", path);
    let (icon_rgba, icon_size) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, (width, height))
    };
    iced::tray_icon::Icon {
        rgba: icon_rgba,
        size: icon_size.into(),
    }
}
