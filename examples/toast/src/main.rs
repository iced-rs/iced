use iced::event::{self, Event};
use iced::keyboard;
use iced::keyboard::key;
use iced::time::seconds;
use iced::widget::{
    Id, button, center, column, operation, pick_list, right, row, slider, stack, text, text_input,
};
use iced::{Center, Fill, Fit, Subscription, Task, Widget};

use iced::widget::toast::{self, Status, Toast};

pub fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .run()
}

struct App {
    toasts: Vec<(Id, Draft)>,
    editing: Draft,
    timeout_secs: u64,
}

#[derive(Debug, Clone, Default)]
struct Draft {
    title: String,
    body: String,
    status: Status,
}

#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
enum Message {
    Add,
    Close(Id),
    Title(String),
    Body(String),
    Status(Status),
    Timeout(f64),
    Event(Event),
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let toast = Draft {
            title: "Example Toast".into(),
            body: "Add more toasts in the form below!".into(),
            status: Status::Primary,
        };

        let id = Id::unique();
        let timeout_secs = toast::DEFAULT_TIMEOUT;

        (
            Self {
                toasts: vec![(id.clone(), toast)],
                editing: Draft::default(),
                timeout_secs,
            },
            Self::expire(id, timeout_secs),
        )
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::Event)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Add => {
                if !self.editing.title.is_empty() && !self.editing.body.is_empty() {
                    let toast = std::mem::take(&mut self.editing);
                    let id = Id::unique();

                    self.toasts.push((id.clone(), toast));

                    Self::expire(id, self.timeout_secs)
                } else {
                    Task::none()
                }
            }
            Message::Close(id) => {
                self.toasts.retain(|(toast_id, _)| toast_id != &id);
                Task::none()
            }
            Message::Title(title) => {
                self.editing.title = title;
                Task::none()
            }
            Message::Body(body) => {
                self.editing.body = body;
                Task::none()
            }
            Message::Status(status) => {
                self.editing.status = status;
                Task::none()
            }
            Message::Timeout(timeout) => {
                self.timeout_secs = timeout as u64;
                Task::none()
            }
            Message::Event(Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::Tab),
                modifiers,
                ..
            })) if modifiers.shift() => operation::focus_previous(),
            Message::Event(Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::Tab),
                ..
            })) => operation::focus_next(),
            Message::Event(_) => Task::none(),
        }
    }

    /// Schedules the expiration of the toast with the given `id`.
    fn expire(id: Id, timeout_secs: u64) -> Task<Message> {
        Task::perform(tokio::time::sleep(seconds(timeout_secs)), move |_| {
            Message::Close(id)
        })
    }

    fn view(&self) -> impl Widget<Message> {
        fn subtitle<'a>(
            title: &'a str,
            content: impl Widget<Message> + 'a,
        ) -> impl Widget<Message> {
            column![text(title).size(14), content].spacing(5)
        }

        let add_toast = button("Add Toast").on_press_maybe(
            (!self.editing.body.is_empty() && !self.editing.title.is_empty())
                .then_some(Message::Add),
        );

        let content = center(
            column![
                subtitle(
                    "Title",
                    text_input("", &self.editing.title)
                        .on_input(Message::Title)
                        .on_submit(Message::Add)
                ),
                subtitle(
                    "Message",
                    text_input("", &self.editing.body)
                        .on_input(Message::Body)
                        .on_submit(Message::Add)
                ),
                subtitle(
                    "Status",
                    pick_list(
                        Some(self.editing.status),
                        toast::Status::ALL,
                        toast::Status::to_string
                    )
                    .on_select(Message::Status)
                    .width(Fill)
                ),
                subtitle(
                    "Timeout",
                    row![
                        text!("{:0>2} sec", self.timeout_secs),
                        slider(1.0..=30.0, self.timeout_secs as f64, Message::Timeout).step(1.0)
                    ]
                    .spacing(5)
                ),
                column![add_toast].align_x(Center)
            ]
            .spacing(10)
            .width(Fit.max(200)),
        );

        let toasts = self.toasts.iter().map(|(id, draft)| {
            Toast::new(draft.title.as_str(), draft.body.as_str())
                .status(draft.status)
                .on_close(Message::Close(id.clone()))
                .width(Fit.max(200))
                .view()
        });

        stack![content, right(column(toasts).spacing(10))]
    }
}
