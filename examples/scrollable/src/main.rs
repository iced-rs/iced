use iced::{
    scrollable, Column, Container, Element, Length, Radio, Row, Rule, Sandbox,
    Scrollable, Settings, Space, Text,
};

pub fn main() -> iced::Result {
    ScrollableDemo::run(Settings::default())
}

struct ScrollableDemo {
    theme: style::Theme,
    scroll: Vec<scrollable::State>,
    config: Vec<Config>,
}

#[derive(Debug, Clone)]
enum Message {
    ThemeChanged(style::Theme),
}

/// Contains configuration for a single scrollbar
struct Config {
    top_content: String,
    scrollbar_width: Option<u16>,
    scrollbar_margin: Option<u16>,
    scroller_width: Option<u16>,
}

fn get_configs() -> Vec<Config> {
    vec![
        Config {
            top_content: "Default Scrollbar".into(),
            scrollbar_width: None,
            scrollbar_margin: None,
            scroller_width: None,
        },
        Config {
            top_content: "Slimmed & Margin".into(),
            scrollbar_width: Some(4),
            scrollbar_margin: Some(3),
            scroller_width: Some(4),
        },
        Config {
            top_content: "Wide Scroller".into(),
            scrollbar_width: Some(4),
            scrollbar_margin: None,
            scroller_width: Some(10),
        },
        Config {
            top_content: "Narrow Scroller".into(),
            scrollbar_width: Some(10),
            scrollbar_margin: None,
            scroller_width: Some(4),
        },
    ]
}

impl Sandbox for ScrollableDemo {
    type Message = Message;

    fn new() -> Self {
        let config = get_configs();
        ScrollableDemo {
            theme: Default::default(),
            scroll: vec![scrollable::State::default(); config.len()],
            config,
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
        let choose_theme = style::Theme::ALL.iter().fold(
            Column::new().spacing(10).push(Text::new("Choose a theme:")),
            |column, theme| {
                column.push(
                    Radio::new(
                        *theme,
                        &format!("{:?}", theme),
                        Some(self.theme),
                        Message::ThemeChanged,
                    )
                    .style(self.theme),
                )
            },
        );

        let ScrollableDemo { scroll, theme, .. } = self;

        let scrollable_row = Row::with_children(
            scroll
                .iter_mut()
                .zip(self.config.iter())
                .map(|(state, config)| {
                    let mut scrollable = Scrollable::new(state)
                        .padding(10)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .style(*theme)
                        .push(Text::new(config.top_content.clone()));

                    if let Some(scrollbar_width) = config.scrollbar_width {
                        scrollable = scrollable
                            .scrollbar_width(scrollbar_width)
                            .push(Text::new(format!(
                                "scrollbar_width: {:?}",
                                scrollbar_width
                            )));
                    }
                    if let Some(scrollbar_margin) = config.scrollbar_margin {
                        scrollable = scrollable
                            .scrollbar_margin(scrollbar_margin)
                            .push(Text::new(format!(
                                "scrollbar_margin: {:?}",
                                scrollbar_margin
                            )));
                    }
                    if let Some(scroller_width) = config.scroller_width {
                        scrollable = scrollable
                            .scroller_width(scroller_width)
                            .push(Text::new(format!(
                                "scroller_width: {:?}",
                                scroller_width
                            )));
                    }

                    scrollable = scrollable
                        .push(Space::with_height(Length::Units(100)))
                        .push(Text::new("Some content that should wrap within the scrollable. Let's output a lot of short words, so that we'll make sure to see how wrapping works with these scrollbars."))
                        .push(Space::with_height(Length::Units(1200)))
                        .push(Text::new("Middle"))
                        .push(Space::with_height(Length::Units(1200)))
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

mod style {
    use iced::{container, radio, rule, scrollable};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Theme {
        Light,
        Dark,
    }

    impl Theme {
        pub const ALL: [Theme; 2] = [Theme::Light, Theme::Dark];
    }

    impl Default for Theme {
        fn default() -> Theme {
            Theme::Light
        }
    }

    impl From<Theme> for Box<dyn container::StyleSheet> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Light => Default::default(),
                Theme::Dark => dark::Container.into(),
            }
        }
    }

    impl From<Theme> for Box<dyn radio::StyleSheet> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Light => Default::default(),
                Theme::Dark => dark::Radio.into(),
            }
        }
    }

    impl From<Theme> for Box<dyn scrollable::StyleSheet> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Light => Default::default(),
                Theme::Dark => dark::Scrollable.into(),
            }
        }
    }

    impl From<Theme> for Box<dyn rule::StyleSheet> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Light => Default::default(),
                Theme::Dark => dark::Rule.into(),
            }
        }
    }

    mod dark {
        use iced::{container, radio, rule, scrollable, Color};

        const BACKGROUND: Color = Color::from_rgb(
            0x0f as f32 / 255.0,
            0x14 as f32 / 255.0,
            0x19 as f32 / 255.0,
        );

        const LIGHTER_BACKGROUND: Color = Color::from_rgb(
            0x14 as f32 / 255.0,
            0x19 as f32 / 255.0,
            0x1f as f32 / 255.0,
        );

        const YELLOW: Color = Color::from_rgb(
            0xff as f32 / 255.0,
            0xb4 as f32 / 255.0,
            0x54 as f32 / 255.0,
        );

        const CYAN: Color = Color::from_rgb(
            0x39 as f32 / 255.0,
            0xaf as f32 / 255.0,
            0xd7 as f32 / 255.0,
        );

        const CYAN_LIGHT: Color = Color::from_rgb(
            0x5d as f32 / 255.0,
            0xb7 as f32 / 255.0,
            0xd5 as f32 / 255.0,
        );

        const ORANGE: Color = Color::from_rgb(
            0xff as f32 / 255.0,
            0x77 as f32 / 255.0,
            0x33 as f32 / 255.0,
        );

        const ORANGE_DARK: Color = Color::from_rgb(
            0xe6 as f32 / 255.0,
            0x5b as f32 / 255.0,
            0x16 as f32 / 255.0,
        );

        pub struct Container;

        impl container::StyleSheet for Container {
            fn style(&self) -> container::Style {
                container::Style {
                    background: Color {
                        a: 0.99,
                        ..BACKGROUND
                    }
                    .into(),
                    text_color: Color::WHITE.into(),
                    ..container::Style::default()
                }
            }
        }

        pub struct Radio;

        impl radio::StyleSheet for Radio {
            fn active(&self) -> radio::Style {
                radio::Style {
                    background: BACKGROUND.into(),
                    dot_color: CYAN,
                    border_width: 1,
                    border_color: CYAN,
                }
            }

            fn hovered(&self) -> radio::Style {
                radio::Style {
                    background: LIGHTER_BACKGROUND.into(),
                    ..self.active()
                }
            }
        }

        pub struct Scrollable;

        impl scrollable::StyleSheet for Scrollable {
            fn active(&self) -> scrollable::Scrollbar {
                scrollable::Scrollbar {
                    background: CYAN.into(),
                    border_radius: 2,
                    border_width: 0,
                    border_color: Color::TRANSPARENT,
                    scroller: scrollable::Scroller {
                        color: YELLOW,
                        border_radius: 2,
                        border_width: 0,
                        border_color: Color::TRANSPARENT,
                    },
                }
            }

            fn hovered(&self) -> scrollable::Scrollbar {
                let active = self.active();

                scrollable::Scrollbar {
                    background: CYAN_LIGHT.into(),
                    scroller: scrollable::Scroller {
                        color: ORANGE,
                        ..active.scroller
                    },
                    ..active
                }
            }

            fn dragging(&self) -> scrollable::Scrollbar {
                let hovered = self.hovered();

                scrollable::Scrollbar {
                    scroller: scrollable::Scroller {
                        color: ORANGE_DARK,
                        ..hovered.scroller
                    },
                    ..hovered
                }
            }
        }

        pub struct Rule;

        impl rule::StyleSheet for Rule {
            fn style(&self) -> rule::Style {
                rule::Style {
                    color: CYAN,
                    width: 2,
                    radius: 1,
                    fill_mode: rule::FillMode::Percent(15.0),
                }
            }
        }
    }
}
