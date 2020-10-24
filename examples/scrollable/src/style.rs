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
