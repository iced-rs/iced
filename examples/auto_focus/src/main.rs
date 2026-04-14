use iced::event::{self, Event};
use iced::keyboard;
use iced::keyboard::key;
use iced::widget::operation;
use iced::widget::{auto_focus, button, center, column, container, row, text, text_input};
use iced::{Element, Subscription, Task};

/// Demonstrates the `auto_focus` widget wrapper.
///
/// - Wrap any focusable widget with `auto_focus(...)` to mark it as the
///   preferred focus target.
/// - The widget automatically focuses on first mount — no manual `Task`.
/// - Supply `.key(page)` so auto-focus re-triggers on page transitions
///   even when the tree structure is identical.
pub fn main() -> iced::Result {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .format_timestamp_millis()
        .init();

    log::info!("Starting auto_focus example");

    iced::application(App::default, App::update, App::view)
        .title("Auto Focus")
        .subscription(App::subscription)
        .run()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum Page {
    #[default]
    Login,
    Search,
    Profile,
}

#[derive(Default)]
struct App {
    page: Page,
    // Login fields
    username: String,
    password: String,
    // Search fields
    query: String,
    filter: String,
    // Profile fields
    name: String,
    email: String,
    bio: String,
}

#[derive(Debug, Clone)]
enum Message {
    GoTo(Page),
    Username(String),
    Password(String),
    Query(String),
    Filter(String),
    Name(String),
    Email(String),
    Bio(String),
    Event(Event),
}

impl App {
    fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::Event)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::GoTo(page) => {
                log::debug!("[App] GoTo({:?})", page);
                self.page = page;
                Task::none()
            }
            Message::Username(v) => {
                self.username = v;
                Task::none()
            }
            Message::Password(v) => {
                self.password = v;
                Task::none()
            }
            Message::Query(v) => {
                self.query = v;
                Task::none()
            }
            Message::Filter(v) => {
                self.filter = v;
                Task::none()
            }
            Message::Name(v) => {
                self.name = v;
                Task::none()
            }
            Message::Email(v) => {
                self.email = v;
                Task::none()
            }
            Message::Bio(v) => {
                self.bio = v;
                Task::none()
            }
            Message::Event(Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::Tab),
                modifiers,
                ..
            })) => {
                if modifiers.shift() {
                    operation::focus_previous()
                } else {
                    operation::focus_next()
                }
            }
            Message::Event(_) => Task::none(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let nav = row![
            nav_button("Login", Page::Login, self.page),
            nav_button("Search", Page::Search, self.page),
            nav_button("Profile", Page::Profile, self.page),
        ]
        .spacing(8);

        let page_content: Element<'_, Message> = match self.page {
            Page::Login => self.login_page(),
            Page::Search => self.search_page(),
            Page::Profile => self.profile_page(),
        };

        center(
            column![
                text("auto_focus example").size(24),
                text("Switch pages — the marked field gets focus automatically.").size(14),
                nav,
                container(page_content)
                    .padding(20)
                    .width(400)
                    .style(container::rounded_box),
            ]
            .spacing(16)
            .align_x(iced::Center),
        )
        .padding(40)
        .into()
    }

    /// Login page — auto-focus on the username field (first input).
    fn login_page(&self) -> Element<'_, Message> {
        column![
            text("Login").size(18),
            text("Username (auto-focused):").size(12),
            auto_focus(text_input("Enter username", &self.username).on_input(Message::Username))
                .key(self.page),
            text("Password:").size(12),
            text_input("Enter password", &self.password)
                .on_input(Message::Password)
                .secure(true),
        ]
        .spacing(8)
        .into()
    }

    /// Search page — auto-focus on the search query field (skips the
    /// filter field even though it appears after in the tree).
    fn search_page(&self) -> Element<'_, Message> {
        column![
            text("Search").size(18),
            text("Filter (not auto-focused):").size(12),
            text_input("Optional filter", &self.filter).on_input(Message::Filter),
            text("Search query (auto-focused):").size(12),
            auto_focus(text_input("Type your search...", &self.query).on_input(Message::Query))
                .key(self.page),
        ]
        .spacing(8)
        .into()
    }

    /// Profile page — auto-focus on the name field, skipping email and bio.
    fn profile_page(&self) -> Element<'_, Message> {
        column![
            text("Profile").size(18),
            text("Email:").size(12),
            text_input("email@example.com", &self.email).on_input(Message::Email),
            text("Display Name (auto-focused):").size(12),
            auto_focus(text_input("Your name", &self.name).on_input(Message::Name)).key(self.page),
            text("Bio:").size(12),
            text_input("Tell us about yourself", &self.bio).on_input(Message::Bio),
        ]
        .spacing(8)
        .into()
    }
}

fn nav_button(label: &str, target: Page, current: Page) -> Element<'_, Message> {
    let btn = button(text(label));

    if current == target {
        btn.into()
    } else {
        btn.on_press(Message::GoTo(target)).into()
    }
}
