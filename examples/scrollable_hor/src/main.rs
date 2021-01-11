mod style;

use iced::{
    scrollable_hor, Column, Container, Element, Length, Radio, Rule, Sandbox,
    ScrollableHor, Settings, Space, Text,
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

        let scrollable_column = Column::with_children(
            variants
                .iter_mut()
                .map(|variant| {
                    let mut scrollable = ScrollableHor::new(&mut variant.state)
                        .padding(10)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .style(*theme)
                        .push(Text::new(variant.title));

                    if let Some(scrollbar_height) = variant.scrollbar_height {
                        scrollable = scrollable
                            .scrollbar_height(scrollbar_height)
                            .push(Text::new(format!(
                                "scrollbar_height: {:?}",
                                scrollbar_height
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

                    if let Some(scroller_width) = variant.scroller_height {
                        scrollable = scrollable
                            .scroller_height(scroller_width)
                            .push(Text::new(format!(
                                "scroller_height: {:?}",
                                scroller_width
                            )));
                    }

                    scrollable = scrollable
                        .push(Space::with_width(Length::Units(100)))
                        .push(Text::new(
                            "Some content that should not wrap within the \
                            scrollable. Let's output a lot of short words, so \
                            that we'll make sure to see how wrapping works \
                            with these scrollbars.",
                        ))
                        .push(Space::with_width(Length::Units(1200)))
                        .push(Text::new("Middle"))
                        .push(Space::with_width(Length::Units(1200)))
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
            .push(scrollable_column);

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
    state: scrollable_hor::State,
    scrollbar_height: Option<u16>,
    scrollbar_margin: Option<u16>,
    scroller_height: Option<u16>,
}

impl Variant {
    pub fn all() -> Vec<Self> {
        vec![
            Self {
                title: "Default Scrollbar",
                state: scrollable_hor::State::new(),
                scrollbar_height: None,
                scrollbar_margin: None,
                scroller_height: None,
            },
            Self {
                title: "Slimmed & Margin",
                state: scrollable_hor::State::new(),
                scrollbar_height: Some(4),
                scrollbar_margin: Some(3),
                scroller_height: Some(4),
            },
            Self {
                title: "Wide Scroller",
                state: scrollable_hor::State::new(),
                scrollbar_height: Some(4),
                scrollbar_margin: None,
                scroller_height: Some(10),
            },
            Self {
                title: "Narrow Scroller",
                state: scrollable_hor::State::new(),
                scrollbar_height: Some(10),
                scrollbar_margin: None,
                scroller_height: Some(4),
            },
        ]
    }
}
