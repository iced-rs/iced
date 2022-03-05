use crate::Style;
use iced::{
    button, container, progress_bar, radio, rule, scrollable, slider,
    text_input, Background, Color, Vector,
};

pub const BACKGROUND: Color = Color::from_rgb(0.99, 0.73, 0.0);

const TEXT_COLOR: Color = Color::from_rgb(1.0, 0.0, 0.0);

const SURFACE: Color = Color::from_rgb(0.7, 1.0, 0.2);

const ACCENT: Color = Color::from_rgb(1.0, 1.0, 0.0);

pub const ACTIVE: Color = Color::BLACK;

const HOVERED: Color = Color::from_rgb(0.7, 1.0, 0.2);

const SCROLLBAR: Color = Color::from_rgb(0.0, 1.0, 1.0);

const SCROLLER: Color = Color::from_rgb(0.0, 0.0, 1.0);

pub struct Container;

impl container::StyleSheet for Container {
    fn style(&self) -> container::Style {
        container::Style {
            background: Color {
                a: 0.99,
                ..BACKGROUND
            }
            .into(),
            text_color: TEXT_COLOR.into(),
            ..container::Style::default()
        }
    }
}

pub struct Radio;

impl radio::StyleSheet for Radio {
    fn active(&self) -> radio::Style {
        radio::Style {
            background: SURFACE.into(),
            dot_color: ACTIVE,
            border_width: 1.0,
            border_color: ACTIVE,
            text_color: Color::from_rgb(0.0, 0.0, 1.0).into(),
        }
    }

    fn hovered(&self) -> radio::Style {
        radio::Style {
            background: Color { a: 0.5, ..SURFACE }.into(),
            ..self.active()
        }
    }
}

pub struct Scrollable;

impl scrollable::StyleSheet for Scrollable {
    fn active(&self) -> scrollable::Style {
        scrollable::Style {
            background: Color {
                a: 0.8,
                ..SCROLLBAR
            }
            .into(),
            border_radius: 2.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            scroller: scrollable::Scroller {
                color: Color { a: 0.7, ..SCROLLER },
                border_radius: 2.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
        }
    }

    fn hovered(&self) -> scrollable::Style {
        let active = self.active();

        scrollable::Style {
            background: SCROLLBAR.into(),
            scroller: scrollable::Scroller {
                color: SCROLLER,
                ..active.scroller
            },
            ..active
        }
    }

    fn dragging(&self) -> scrollable::Style {
        let hovered = self.hovered();

        scrollable::Style {
            scroller: scrollable::Scroller {
                color: ACCENT,
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
            color: SURFACE,
            width: 2,
            radius: 1.0,
            fill_mode: rule::FillMode::Percent(30.0),
        }
    }
}

pub struct Button;

impl button::StyleSheet for Button {
    fn active(&self) -> button::Style {
        button::Style {
            shadow_offset: Vector::new(0.0, 0.0),
            background: Some(Background::Color([0.0, 0.0, 1.0].into())),
            border_radius: 2.0,
            border_width: 1.0,
            border_color: [0.7, 0.7, 0.7].into(),
        }
    }

    fn hovered(&self) -> button::Style {
        button::Style {
            background: HOVERED.into(),
            ..self.active()
        }
    }

    fn pressed(&self) -> button::Style {
        button::Style {
            border_width: 1.0,
            border_color: Color::from_rgb(0.5, 0.2, 0.6),
            ..self.hovered()
        }
    }
}

pub struct TextInput;

impl text_input::StyleSheet for TextInput {
    fn active(&self) -> text_input::Style {
        text_input::Style {
            background: Color::from_rgb(0.03, 0.09, 0.73).into(),
            border_radius: 5.0,
            border_width: 1.0,
            border_color: Color::from_rgb(0.7, 0.7, 0.7),
        }
    }

    fn focused(&self) -> text_input::Style {
        text_input::Style {
            border_color: Color::from_rgb(0.5, 0.5, 0.5),
            ..self.active()
        }
    }

    fn placeholder_color(&self) -> Color {
        Color::from_rgb(0.99, 0.67, 0.11)
    }

    fn value_color(&self) -> Color {
        Color::from_rgb(0.0, 1.0, 0.0)
    }

    fn selection_color(&self) -> Color {
        Color::from_rgb(1.0, 0.0, 0.0)
    }
}

pub struct Slider;

impl slider::StyleSheet for Slider {
    fn active(&self) -> slider::Style {
        slider::Style {
            rail_colors: (ACTIVE, Color { a: 0.1, ..ACTIVE }),
            handle: slider::Handle {
                shape: slider::HandleShape::Rectangle {
                    width: 30,
                    border_radius: 50.0,
                },
                color: ACTIVE,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
        }
    }

    fn hovered(&self) -> slider::Style {
        let active = self.active();

        slider::Style {
            handle: slider::Handle {
                color: HOVERED,
                ..active.handle
            },
            ..active
        }
    }

    fn dragging(&self) -> slider::Style {
        let active = self.active();

        slider::Style {
            handle: slider::Handle {
                color: Color::from_rgb(0.85, 0.85, 0.85),
                ..active.handle
            },
            ..active
        }
    }
}

pub struct ProgressBar;

impl progress_bar::StyleSheet for ProgressBar {
    fn style(&self) -> progress_bar::Style {
        progress_bar::Style {
            background: Color::from_rgb(0.43, 0.2, 0.92).into(),
            bar: Color::from_rgb(0.14, 0.79, 0.75).into(),
            border_radius: 50.0,
        }
    }
}

pub fn get_style() -> Style {
    Style {
        text_color: TEXT_COLOR,
        button_style_sheet: Button.into(),
        container_style_sheet: Container.into(),
        radio_style_sheet: Radio.into(),
        rule_style_sheet: Rule.into(),
        scrollable_style_sheet: Scrollable.into(),
        text_input_style_sheet: TextInput.into(),
        slider_style_sheet: Slider.into(),
        progress_bar_style_sheet: ProgressBar.into(),
        ..Default::default()
    }
}
