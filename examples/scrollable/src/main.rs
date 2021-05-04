mod style;

use iced::{
    button, scrollable, Button, Column, Container, Element, Length, Radio, Row,
    Rule, Sandbox, Scrollable, Settings, Space, Text,
};

pub fn main() -> iced::Result {
    ScrollableDemo::run(Settings::default())
}

struct ScrollableDemo {
    theme: style::Theme,
    variants: Vec<Variant>,
}

#[derive(Debug, Clone)]
enum Message {
    ThemeChanged(style::Theme),
}

impl Sandbox for ScrollableDemo {
    type Message = Message;

    fn new() -> Self {
        ScrollableDemo {
            theme: Default::default(),
            variants: Variant::all(),
        }
    }

    fn title(&self) -> String {
        String::from("Scrollable - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ThemeChanged(theme) => self.theme = theme,
        }
    }

    fn view(&mut self) -> Element<Message> {
        let ScrollableDemo {
            theme, variants, ..
        } = self;

        let choose_theme = style::Theme::ALL.iter().fold(
            Column::new().spacing(10).push(Text::new("Choose a theme:")),
            |column, option| {
                column.push(
                    Radio::new(
                        *option,
                        &format!("{:?}", option),
                        Some(*theme),
                        Message::ThemeChanged,
                    )
                    .style(*theme),
                )
            },
        );

        let scrollable_row = Row::with_children(
            variants
                .iter_mut()
                .map(|variant| {
                    let mut scrollable =
                        Scrollable::new(&mut variant.scrollable)
                            .padding(10)
                            .spacing(10)
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .style(*theme)
                            .push(Text::new(variant.title));

                    if let Some(scrollbar_width) = variant.scrollbar_width {
                        scrollable = scrollable
                            .scrollbar_width(scrollbar_width)
                            .push(Text::new(format!(
                                "scrollbar_width: {:?}",
                                scrollbar_width
                            )));
                    }

                    if let Some(scrollbar_margin) = variant.scrollbar_margin {
                        scrollable = scrollable
                            .scrollbar_margin(scrollbar_margin)
                            .push(Text::new(format!(
                                "scrollbar_margin: {:?}",
                                scrollbar_margin
                            )));
                    }

                    if let Some(scroller_width) = variant.scroller_width {
                        scrollable = scrollable
                            .scroller_width(scroller_width)
                            .push(Text::new(format!(
                                "scroller_width: {:?}",
                                scroller_width
                            )));
                    }

                    scrollable = scrollable
                        .push(Space::with_height(Length::Units(100)))
                        .push(Text::new(
                            "Some content that should wrap within the \
                            scrollable. Let's output a lot of short words, so \
                            that we'll make sure to see how wrapping works \
                            with these scrollbars.",
                        ))
                        .push(Space::with_height(Length::Units(1200)))
                        .push(Text::new("Middle"))
                        .push(Space::with_height(Length::Units(1200)))
                        .push(
                            Button::new(
                                &mut variant.button,
                                Text::new("I am a button"),
                            )
                            .width(Length::Fill)
                            .padding(10),
                        )
                        .push(Text::new("The End."));

                    Container::new(scrollable)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .style(*theme)
                        .into()
                })
                .collect(),
        )
        .spacing(20)
        .width(Length::Fill)
        .height(Length::Fill);

        let content = Column::new()
            .spacing(20)
            .padding(20)
            .push(choose_theme)
            .push(Rule::horizontal(20).style(self.theme))
            .push(scrollable_row);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(self.theme)
            .into()
    }
}

/// A version of a scrollable
struct Variant {
    title: &'static str,
    scrollable: scrollable::State,
    button: button::State,
    scrollbar_width: Option<u16>,
    scrollbar_margin: Option<u16>,
    scroller_width: Option<u16>,
}

impl Variant {
    pub fn all() -> Vec<Self> {
        vec![
            Self {
                title: "Default Scrollbar",
                scrollable: scrollable::State::new(),
                button: button::State::new(),
                scrollbar_width: None,
                scrollbar_margin: None,
                scroller_width: None,
            },
            Self {
                title: "Slimmed & Margin",
                scrollable: scrollable::State::new(),
                button: button::State::new(),
                scrollbar_width: Some(4),
                scrollbar_margin: Some(3),
                scroller_width: Some(4),
            },
            Self {
                title: "Wide Scroller",
                scrollable: scrollable::State::new(),
                button: button::State::new(),
                scrollbar_width: Some(4),
                scrollbar_margin: None,
                scroller_width: Some(10),
            },
            Self {
                title: "Narrow Scroller",
                scrollable: scrollable::State::new(),
                button: button::State::new(),
                scrollbar_width: Some(10),
                scrollbar_margin: None,
                scroller_width: Some(4),
            },
        ]
    }
}
