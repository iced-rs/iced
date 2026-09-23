use iced::event::{self, Event};
use iced::keyboard;
use iced::keyboard::key;
use iced::time::seconds;
use iced::widget::{
    Id, button, center, column, container, operation, pick_list, right, row, rule, slider, space,
    stack, text, text_input,
};
use iced::{Center, Element, Fill, Fit, Subscription, Task};

use toast::{Status, Toast};

pub fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .run()
}

struct App {
    toasts: Vec<(Id, Toast)>,
    editing: Toast,
    timeout_secs: u64,
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
        let toast = Toast {
            title: "Example Toast".into(),
            body: "Add more toasts in the form below!".into(),
            status: Status::Primary,
        };

        let id = Id::unique();
        let timeout_secs = toast::DEFAULT_TIMEOUT;

        (
            Self {
                toasts: vec![(id.clone(), toast)],
                editing: Toast::default(),
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

    fn view(&self) -> Element<'_, Message> {
        fn subtitle<'a>(
            title: &'a str,
            content: impl Into<Element<'a, Message>>,
        ) -> Element<'a, Message> {
            column![text(title).size(14), content.into()]
                .spacing(5)
                .into()
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

        let toasts = self.toasts.iter().map(|(id, toast)| {
            container(
                column![
                    container(
                        row![
                            text(toast.title.as_str()),
                            space::horizontal(),
                            button("X")
                                .on_press_with(|| Message::Close(id.clone()))
                                .padding(5),
                        ]
                        .align_y(Center)
                    )
                    .width(Fill)
                    .padding(10)
                    .style(match toast.status {
                        Status::Primary => container::primary,
                        Status::Secondary => container::secondary,
                        Status::Success => container::success,
                        Status::Danger => container::danger,
                        Status::Warning => container::warning,
                    }),
                    rule::horizontal(1),
                    text(toast.body.as_str())
                ]
                .spacing(10)
                .padding(10),
            )
            .style(container::rounded_box)
            .width(Fit.max(200))
            .into()
        });

        stack![content, right(column(toasts).spacing(10))].into()
    }
}

mod toast {
    use std::fmt;

    pub const DEFAULT_TIMEOUT: u64 = 5;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub enum Status {
        #[default]
        Primary,
        Secondary,
        Success,
        Danger,
        Warning,
    }

    impl Status {
        pub const ALL: &'static [Self] = &[
            Self::Primary,
            Self::Secondary,
            Self::Success,
            Self::Danger,
            Self::Warning,
        ];
    }

    impl fmt::Display for Status {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Status::Primary => "Primary",
                Status::Secondary => "Secondary",
                Status::Success => "Success",
                Status::Danger => "Danger",
                Status::Warning => "Warning",
            }
            .fmt(f)
        }
    }

    #[derive(Debug, Clone, Default)]
    pub struct Toast {
        pub title: String,
        pub body: String,
        pub status: Status,
    }
}
