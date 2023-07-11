use iced::advanced::graphics::core::keyboard;
use iced::alignment::Horizontal;
use iced::widget::{blur, button, column, container, row, slider, svg, text};
use iced::{
    event, executor, subscription, window, Application, Command, Element,
    Event, Length, Settings, Subscription,
};
use iced::{Alignment, Theme};
use std::time::Instant;

pub fn main() -> iced::Result {
    env_logger::builder().format_timestamp(None).init();

    Tiger::run(Settings::default())
}

#[derive(Debug)]
struct Tiger {
    blur: u16,
    last: Instant,
    curr: Content,
}

impl Default for Tiger {
    fn default() -> Self {
        Self {
            blur: 0,
            last: Instant::now(),
            curr: Content::default(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Blurred(u16),
    IncrementBlur,
    DecrementBlur,
    Tick(Instant),
}

impl Application for Tiger {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Tiger::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("SVG - Iced")
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Message> {
        match message {
            Message::Blurred(value) => {
                self.blur = value;
            }
            Message::IncrementBlur => {
                self.blur += 1;
            }
            Message::DecrementBlur => {
                self.blur = if self.blur == 0 { 0 } else { self.blur - 1 }
            }
            Message::Tick(now) => {
                let duration = (now - self.last).as_secs();

                if duration > 5 {
                    self.last = now;
                    self.curr.next();
                }
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let slider = row![
            text(self.blur),
            slider(0..=100, self.blur, Message::Blurred).width(200),
        ]
        .spacing(20);

        container(
            column![
                container(blur(self.blur, self.curr.view()))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                    .center_y(),
                container(slider).width(Length::Fill).center_x(),
            ]
            .spacing(20)
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .center_x()
        .center_y()
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        let tick = window::frames().map(Message::Tick);
        let keyboard =
            subscription::events_with(|event, status| match (event, status) {
                (
                    Event::Keyboard(keyboard::Event::KeyPressed {
                        key_code: keyboard::KeyCode::Right,
                        ..
                    }),
                    event::Status::Ignored,
                ) => Some(Message::IncrementBlur),
                (
                    Event::Keyboard(keyboard::Event::KeyPressed {
                        key_code: keyboard::KeyCode::Left,
                        ..
                    }),
                    event::Status::Ignored,
                ) => Some(Message::DecrementBlur),
                _ => None,
            });

        Subscription::batch(vec![tick, keyboard])
    }
}

#[derive(Default, Debug)]
enum Content {
    #[default]
    Image,
    Text,
    Layout,
}

impl Content {
    pub fn view<'a>(&self) -> Element<'a, Message> {
        match self {
            Content::Image => img_example(),
            Content::Text => text_example(),
            Content::Layout => layout_example(),
        }
    }

    pub fn next(&mut self) {
        *self = match self {
            Content::Image => Content::Text,
            Content::Text => Content::Layout,
            Content::Layout => Content::Image,
        }
    }
}

fn img_example<'a>() -> Element<'a, Message> {
    let handle = svg::Handle::from_path(format!(
        "{}/resources/tiger.svg",
        env!("CARGO_MANIFEST_DIR")
    ));

    let tiger = svg(handle).width(Length::Fill).height(Length::Fill);

    container(tiger)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .into()
}

fn text_example<'a>() -> Element<'a, Message> {
    container(
        text(r#"        Lorem ipsum dolor sit amet, consectetur adipiscing elit.
        Suspendisse ut ligula sem. Integer hendrerit ullamcorper porttitor.
        Vivamus sollicitudin eget ex eget viverra. Morbi maximus et ipsum sed consequat.
        Sed id augue lectus. Duis non nunc eget dui auctor condimentum.
        Sed elit nunc, porta in quam at, aliquam eleifend metus.

        Proin venenatis varius lorem in rutrum. Integer nec erat sodales, vestibulum nunc non, varius ipsum.
        Sed viverra fermentum sapien, at vestibulum orci convallis ac.
        Vivamus accumsan finibus est, pellentesque egestas leo faucibus sit amet.
        Pellentesque risus lectus, consectetur eu accumsan non, tincidunt sed nunc.
        Proin hendrerit erat vitae laoreet gravida. Ut ut bibendum quam.
        Proin rutrum orci ac felis porttitor eleifend.
        Vivamus turpis lorem, pellentesque pharetra ultrices et, mattis sed tellus.
        Sed ligula enim, facilisis et risus a, egestas vestibulum dui.
        Duis blandit luctus lorem, nec tincidunt orci luctus id.
        Vivamus hendrerit lectus non sem auctor sodales quis pellentesque tortor.
        Aenean ac interdum leo, eget auctor eros. Quisque gravida libero eu urna fermentum blandit.
        Nulla sit amet ornare ante. Mauris vulputate, lectus vitae euismod sollicitudin,
        massa nibh egestas orci, sit amet elementum mi elit a eros."#)
    )
        .padding(40)
        .into()
}

fn layout_example<'a>() -> Element<'a, Message> {
    let some_buttons =
        row![button("Button 1"), button("Button 2"), button("Button 3")]
            .spacing(20);

    let some_text = text("Some text!")
        .size(42)
        .width(Length::Fill)
        .horizontal_alignment(Horizontal::Center);

    let some_columns = row![
        column![button("Button 4"), text("Text!"), button("Button 5"),]
            .align_items(Alignment::Center)
            .spacing(10),
        column![button("Button 6"), text("Text!"), button("Button 7"),]
            .align_items(Alignment::Center)
            .spacing(10),
        column![button("Button 8"), text("Text!"), button("Button 9"),]
            .align_items(Alignment::Center)
            .spacing(10),
    ]
    .spacing(20);

    container(
        column![some_buttons, some_text, some_columns]
            .spacing(40)
            .align_items(Alignment::Center),
    )
    .padding(20)
    .width(400)
    .into()
}
