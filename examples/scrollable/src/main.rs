use iced::executor;
use iced::widget::{
    button, column, container, horizontal_rule, progress_bar, radio,
    scrollable, text, vertical_space, Row,
};
use iced::{Application, Command, Element, Length, Settings, Theme};

pub fn main() -> iced::Result {
    ScrollableDemo::run(Settings::default())
}

struct ScrollableDemo {
    theme: Theme,
    variants: Vec<Variant>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum ThemeType {
    Light,
    Dark,
}

#[derive(Debug, Clone)]
enum Message {
    ThemeChanged(ThemeType),
    ScrollToTop(usize),
    ScrollToBottom(usize),
    Scrolled(usize, f32),
}

impl Application for ScrollableDemo {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            ScrollableDemo {
                theme: Default::default(),
                variants: Variant::all(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Scrollable - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ThemeChanged(theme) => {
                self.theme = match theme {
                    ThemeType::Light => Theme::Light,
                    ThemeType::Dark => Theme::Dark,
                };

                Command::none()
            }
            Message::ScrollToTop(i) => {
                if let Some(variant) = self.variants.get_mut(i) {
                    variant.latest_offset = 0.0;

                    scrollable::snap_to(Variant::id(i), 0.0)
                } else {
                    Command::none()
                }
            }
            Message::ScrollToBottom(i) => {
                if let Some(variant) = self.variants.get_mut(i) {
                    variant.latest_offset = 1.0;

                    scrollable::snap_to(Variant::id(i), 1.0)
                } else {
                    Command::none()
                }
            }
            Message::Scrolled(i, offset) => {
                if let Some(variant) = self.variants.get_mut(i) {
                    variant.latest_offset = offset;
                }

                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let ScrollableDemo { variants, .. } = self;

        let choose_theme = [ThemeType::Light, ThemeType::Dark].iter().fold(
            column!["Choose a theme:"].spacing(10),
            |column, option| {
                column.push(radio(
                    format!("{:?}", option),
                    *option,
                    Some(*option),
                    Message::ThemeChanged,
                ))
            },
        );

        let scrollable_row = Row::with_children(
            variants
                .iter()
                .enumerate()
                .map(|(i, variant)| {
                    let mut contents = column![
                        variant.title,
                        button("Scroll to bottom",)
                            .width(Length::Fill)
                            .padding(10)
                            .on_press(Message::ScrollToBottom(i)),
                    ]
                    .padding(10)
                    .spacing(10)
                    .width(Length::Fill);

                    if let Some(scrollbar_width) = variant.scrollbar_width {
                        contents = contents.push(text(format!(
                            "scrollbar_width: {:?}",
                            scrollbar_width
                        )));
                    }

                    if let Some(scrollbar_margin) = variant.scrollbar_margin {
                        contents = contents.push(text(format!(
                            "scrollbar_margin: {:?}",
                            scrollbar_margin
                        )));
                    }

                    if let Some(scroller_width) = variant.scroller_width {
                        contents = contents.push(text(format!(
                            "scroller_width: {:?}",
                            scroller_width
                        )));
                    }

                    contents = contents
                        .push(vertical_space(Length::Units(100)))
                        .push(
                            "Some content that should wrap within the \
                            scrollable. Let's output a lot of short words, so \
                            that we'll make sure to see how wrapping works \
                            with these scrollbars.",
                        )
                        .push(vertical_space(Length::Units(1200)))
                        .push("Middle")
                        .push(vertical_space(Length::Units(1200)))
                        .push("The End.")
                        .push(
                            button("Scroll to top")
                                .width(Length::Fill)
                                .padding(10)
                                .on_press(Message::ScrollToTop(i)),
                        );

                    let mut scrollable = scrollable(contents)
                        .id(Variant::id(i))
                        .height(Length::Fill)
                        .on_scroll(move |offset| Message::Scrolled(i, offset));

                    if let Some(scrollbar_width) = variant.scrollbar_width {
                        scrollable =
                            scrollable.scrollbar_width(scrollbar_width);
                    }

                    if let Some(scrollbar_margin) = variant.scrollbar_margin {
                        scrollable =
                            scrollable.scrollbar_margin(scrollbar_margin);
                    }

                    if let Some(scroller_width) = variant.scroller_width {
                        scrollable = scrollable.scroller_width(scroller_width);
                    }

                    column![
                        scrollable,
                        progress_bar(0.0..=1.0, variant.latest_offset,)
                    ]
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .spacing(10)
                    .into()
                })
                .collect(),
        )
        .spacing(20)
        .width(Length::Fill)
        .height(Length::Fill);

        let content =
            column![choose_theme, horizontal_rule(20), scrollable_row]
                .spacing(20)
                .padding(20);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}

/// A version of a scrollable
struct Variant {
    title: &'static str,
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
                scrollbar_width: None,
                scrollbar_margin: None,
                scroller_width: None,
                latest_offset: 0.0,
            },
            Self {
                title: "Slimmed & Margin",
                scrollbar_width: Some(4),
                scrollbar_margin: Some(3),
                scroller_width: Some(4),
                latest_offset: 0.0,
            },
            Self {
                title: "Wide Scroller",
                scrollbar_width: Some(4),
                scrollbar_margin: None,
                scroller_width: Some(10),
                latest_offset: 0.0,
            },
            Self {
                title: "Narrow Scroller",
                scrollbar_width: Some(10),
                scrollbar_margin: None,
                scroller_width: Some(4),
                latest_offset: 0.0,
            },
        ]
    }

    pub fn id(i: usize) -> scrollable::Id {
        scrollable::Id::new(format!("scrollable-{}", i))
    }
}
