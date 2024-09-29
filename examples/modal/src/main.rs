use iced::event::{self, Event};
use iced::keyboard;
use iced::keyboard::key;
use iced::widget::{
    self, button, center, column, container, horizontal_space, mouse_area,
    opaque, pick_list, row, stack, text, text_input,
};
use iced::{Bottom, Color, Element, Fill, Subscription, Task};

use std::fmt;

pub fn main() -> iced::Result {
    iced::application("Modal - Iced", App::update, App::view)
        .subscription(App::subscription)
        .run()
}

#[derive(Default)]
struct App {
    show_modal: bool,
    email: String,
    password: String,
    plan: Plan,
}

#[derive(Debug, Clone)]
enum Message {
    ShowModal,
    HideModal,
    Email(String),
    Password(String),
    Plan(Plan),
    Submit,
    Event(Event),
}

impl App {
    fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::Event)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ShowModal => {
                self.show_modal = true;
                widget::focus_next()
            }
            Message::HideModal => {
                self.hide_modal();
                Task::none()
            }
            Message::Email(email) => {
                self.email = email;
                Task::none()
            }
            Message::Password(password) => {
                self.password = password;
                Task::none()
            }
            Message::Plan(plan) => {
                self.plan = plan;
                Task::none()
            }
            Message::Submit => {
                if !self.email.is_empty() && !self.password.is_empty() {
                    self.hide_modal();
                }

                Task::none()
            }
            Message::Event(event) => match event {
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(key::Named::Tab),
                    modifiers,
                    ..
                }) => {
                    if modifiers.shift() {
                        widget::focus_previous()
                    } else {
                        widget::focus_next()
                    }
                }
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(key::Named::Escape),
                    ..
                }) => {
                    self.hide_modal();
                    Task::none()
                }
                _ => Task::none(),
            },
        }
    }

    fn view(&self) -> Element<Message> {
        let content = container(
            column![
                row![text("Top Left"), horizontal_space(), text("Top Right")]
                    .height(Fill),
                center(button(text("Show Modal")).on_press(Message::ShowModal)),
                row![
                    text("Bottom Left"),
                    horizontal_space(),
                    text("Bottom Right")
                ]
                .align_y(Bottom)
                .height(Fill),
            ]
            .height(Fill),
        )
        .padding(10);

        if self.show_modal {
            let signup = container(
                column![
                    text("Sign Up").size(24),
                    column![
                        column![
                            text("Email").size(12),
                            text_input("abc@123.com", &self.email,)
                                .on_input(Message::Email)
                                .on_submit(Message::Submit)
                                .padding(5),
                        ]
                        .spacing(5),
                        column![
                            text("Password").size(12),
                            text_input("", &self.password)
                                .on_input(Message::Password)
                                .on_submit(Message::Submit)
                                .secure(true)
                                .padding(5),
                        ]
                        .spacing(5),
                        column![
                            text("Plan").size(12),
                            pick_list(
                                Plan::ALL,
                                Some(self.plan),
                                Message::Plan
                            )
                            .padding(5),
                        ]
                        .spacing(5),
                        button(text("Submit")).on_press(Message::HideModal),
                    ]
                    .spacing(10)
                ]
                .spacing(20),
            )
            .width(300)
            .padding(10)
            .style(container::rounded_box);

            modal(content, signup, Message::HideModal)
        } else {
            content.into()
        }
    }
}

impl App {
    fn hide_modal(&mut self) {
        self.show_modal = false;
        self.email.clear();
        self.password.clear();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Plan {
    #[default]
    Basic,
    Pro,
    Enterprise,
}

impl Plan {
    pub const ALL: &'static [Self] =
        &[Self::Basic, Self::Pro, Self::Enterprise];
}

impl fmt::Display for Plan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Plan::Basic => "Basic",
            Plan::Pro => "Pro",
            Plan::Enterprise => "Enterprise",
        }
        .fmt(f)
    }
}

fn modal<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
    on_blur: Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    stack![
        base.into(),
        opaque(
            mouse_area(center(opaque(content)).style(|_theme| {
                container::Style {
                    background: Some(
                        Color {
                            a: 0.8,
                            ..Color::BLACK
                        }
                        .into(),
                    ),
                    ..container::Style::default()
                }
            }))
            .on_press(on_blur)
        )
    ]
    .into()
}
