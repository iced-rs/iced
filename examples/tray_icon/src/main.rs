use iced::{
    Element, Event, Subscription, Task, event, keyboard, tray_icon,
    widget::text,
};

pub fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .subscription(App::subscription)
        .tray_icon(tray_icon::Settings {
            title: Some("Iced".into()),
            icon: None,
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
    last_menu_clicked_id: String,
}

impl App {
    fn new() -> Self {
        Self {
            last_menu_clicked_id: "".into(),
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Event(Event::TrayIcon(event)) => match event {
                tray_icon::Event::MenuItemClicked { id } => {
                    self.last_menu_clicked_id = id;
                }
                _ => {}
            },
            _ => {}
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        text(self.last_menu_clicked_id.clone()).into()
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