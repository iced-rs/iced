mod style;

use iced::{
    button, scrollable, Button, Column, Container, Element, Length,
    ProgressBar, Radio, Row, Rule, Sandbox, Scrollable, Settings, Space, Text,
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
    ScrollToTop(usize),
    ScrollToBottom(usize),
    Scrolled(usize, f32),
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
            Message::ScrollToTop(i) => {
                if let Some(variant) = self.variants.get_mut(i) {
                    variant.scrollable.snap_to(0.0);

                    variant.latest_offset = 0.0;
                }
            }
            Message::ScrollToBottom(i) => {
                if let Some(variant) = self.variants.get_mut(i) {
                    variant.scrollable.snap_to(1.0);

                    variant.latest_offset = 1.0;
                }
            }
            Message::Scrolled(i, offset) => {
                if let Some(variant) = self.variants.get_mut(i) {
                    variant.latest_offset = offset;
                }
            }
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
                        format!("{:?}", option),
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
                .enumerate()
                .map(|(i, variant)| {
                    let mut scrollable =
                        Scrollable::new(&mut variant.scrollable)
                            .padding(10)
                            .spacing(10)
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .on_scroll(move |offset| {
                                Message::Scrolled(i, offset)
                            })
                            .style(*theme)
                            .push(Text::new(variant.title))
                            .push(
                                Button::new(
                                    &mut variant.scroll_to_bottom,
                                    Text::new("Scroll to bottom"),
                                )
                                .width(Length::Fill)
                                .padding(10)
                                .on_press(Message::ScrollToBottom(i)),
                            );

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
                        .push(Text::new("The End."))
                        .push(
                            Button::new(
                                &mut variant.scroll_to_top,
                                Text::new("Scroll to top"),
                            )
                            .width(Length::Fill)
                            .padding(10)
                            .on_press(Message::ScrollToTop(i)),
                        );

                    Column::new()
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .spacing(10)
                        .push(
                            Container::new(scrollable)
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .style(*theme),
                        )
                        .push(ProgressBar::new(
                            0.0..=1.0,
                            variant.latest_offset,
                        ))
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
    scroll_to_top: button::State,
    scroll_to_bottom: button::State,
    scrollbar_width: Option<u16>,
    scrollbar_margin: Option<u16>,
    scroller_width: Option<u16>,
    latest_offset: f32,
}

impl Variant {
    pub fn all() -> Vec<Self> {
        vec![
            Self {
                title: "Default Scrollbar",
                scrollable: scrollable::State::new(),
                scroll_to_top: button::State::new(),
                scroll_to_bottom: button::State::new(),
                scrollbar_width: None,
                scrollbar_margin: None,
                scroller_width: None,
                latest_offset: 0.0,
            },
            Self {
                title: "Slimmed & Margin",
                scrollable: scrollable::State::new(),
                scroll_to_top: button::State::new(),
                scroll_to_bottom: button::State::new(),
                scrollbar_width: Some(4),
                scrollbar_margin: Some(3),
                scroller_width: Some(4),
                latest_offset: 0.0,
            },
            Self {
                title: "Wide Scroller",
                scrollable: scrollable::State::new(),
                scroll_to_top: button::State::new(),
                scroll_to_bottom: button::State::new(),
                scrollbar_width: Some(4),
                scrollbar_margin: None,
                scroller_width: Some(10),
                latest_offset: 0.0,
            },
            Self {
                title: "Narrow Scroller",
                scrollable: scrollable::State::new(),
                scroll_to_top: button::State::new(),
                scroll_to_bottom: button::State::new(),
                scrollbar_width: Some(10),
                scrollbar_margin: None,
                scroller_width: Some(4),
                latest_offset: 0.0,
            },
        ]
    }
}
