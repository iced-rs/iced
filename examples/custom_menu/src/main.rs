mod menu;

use iced::{Element, Subscription, application, widget::text};
use menu::AppMenu;
use muda::Menu;

pub fn main() -> iced::Result {
    application(App::new, App::update, App::view)
        .title("Custom menu")
        .subscription(App::sub)
        .run()
}

struct App {
    menu: AppMenu,
    text: String,
}

impl App {
    fn new() -> Self {
        let menu_bar = Menu::new();
        let menu = menu::AppMenu::new(menu_bar);
        menu.init();
        Self {
            menu,
            text: "Custom menu".to_string(),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::MenuPressed(id) => {
                self.text = if self.menu.custom_item.id() == id {
                    format!("Custom item pressed!")
                } else {
                    format!("Menu {id} pressed!")
                };
            }
        }
    }

    fn view(&self) -> Element<Message> {
        text(self.text.clone()).into()
    }

    fn sub(&self) -> Subscription<Message> {
        iced::event::listen_with(|ev, _, _| match ev {
            iced::Event::Menu(id) => {
                return Some(Message::MenuPressed(id));
            }
            _ => None,
        })
    }
}

#[derive(Debug, Clone)]
enum Message {
    MenuPressed(String),
}
