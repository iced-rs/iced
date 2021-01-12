mod style;

use iced::{
    scrollable, scrollable::Axis, Column, Container, Element, Length, Radio, Row, Rule, Sandbox,
    Scrollable, Settings, Space, Text,
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
                    let mut content = Column::new();
                    content = content.push(Text::new(variant.title));

                    if let Some(scrollbar_width) = variant.scrollbar_width {
                        content = content.push(Text::new(format!(
                                "scrollbar_width: {:?}",
                                scrollbar_width
                            )));
                    }

                    if let Some(scrollbar_margin) = variant.scrollbar_margin {
                        content = content.push(Text::new(format!(
                                "scrollbar_margin: {:?}",
                                scrollbar_margin
                            )));
                    }

                    if let Some(scroller_width) = variant.scroller_width {
                        content = content.push(Text::new(format!(
                                "scroller_width: {:?}",
                                scroller_width
                            )));
                    }

                    if let Some(inner_variant) = &mut variant.inner_variant {
                        let iv = &mut (**inner_variant);
                        content = content
                            .push(Scrollable::new(&mut iv.state,
                                    Row::new()
                                    .padding(10)
                                    .spacing(20)
                                    .push(Text::new(iv.title))
                                    .push(Text::new(
                                        "Some content that should NOT wrap within the top scrollable \
                                        because it is in a horizontal scrollable. Let's output a lot \
                                        of short words, so that we'll make sure to see how that it \
                                        works as expected with this horizontal scrollbar.",
                                    ))).axis(Axis::Horizontal)
                                .width(Length::Fill)
                                .height(Length::Shrink)
                                .style(*theme));
                    }

                    content = content
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
                        .push(Text::new("The End."));

                    let content_container = Container::new(content).padding(10);

                    let mut scrollable = Scrollable::new(&mut variant.state, content_container)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(*theme);

                    if let Some(scrollbar_width) = variant.scrollbar_width {
                        scrollable = scrollable
                            .scrollbar_width(scrollbar_width);
                    }

                    if let Some(scrollbar_margin) = variant.scrollbar_margin {
                        scrollable = scrollable
                            .scrollbar_margin(scrollbar_margin);
                    }

                    if let Some(scroller_width) = variant.scroller_width {
                        scrollable = scrollable
                            .scroller_width(scroller_width);
                    }

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
    state: scrollable::State,
    scrollbar_width: Option<u16>,
    scrollbar_margin: Option<u16>,
    scroller_width: Option<u16>,
    inner_variant: Option<Box<Variant>>,
}

impl Variant {
    pub fn all() -> Vec<Self> {
        vec![
            Self {
                title: "Default Scrollbar",
                state: scrollable::State::new(),
                scrollbar_width: None,
                scrollbar_margin: None,
                scroller_width: None,
                inner_variant: Some(Box::new(Self {
                    title: "Horizontal Scrollbar",
                    state: scrollable::State::new(),
                    scrollbar_width: None,
                    scrollbar_margin: None,
                    scroller_width: None,
                    inner_variant: None,
                })),
            },
            Self {
                title: "Slimmed & Margin",
                state: scrollable::State::new(),
                scrollbar_width: Some(4),
                scrollbar_margin: Some(3),
                scroller_width: Some(4),
                inner_variant: None,
            },
            Self {
                title: "Wide Scroller",
                state: scrollable::State::new(),
                scrollbar_width: Some(4),
                scrollbar_margin: None,
                scroller_width: Some(10),
                inner_variant: None,
            },
            Self {
                title: "Narrow Scroller",
                state: scrollable::State::new(),
                scrollbar_width: Some(10),
                scrollbar_margin: None,
                scroller_width: Some(4),
                inner_variant: None,
            },
        ]
    }
}
