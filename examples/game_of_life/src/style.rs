use iced::{container, pick_list, Background, Color};

const BACKGROUND: Color = Color::from_rgb(
    0x2F as f32 / 255.0,
    0x31 as f32 / 255.0,
    0x36 as f32 / 255.0,
);

pub struct Container;

impl container::StyleSheet for Container {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(Color::from_rgb8(
                0x36, 0x39, 0x3F,
            ))),
            text_color: Some(Color::WHITE),
            ..container::Style::default()
        }
    }
}

pub struct PickList;

impl pick_list::StyleSheet for PickList {
    fn menu(&self) -> pick_list::Menu {
        pick_list::Menu {
            text_color: Color::WHITE,
            background: BACKGROUND.into(),
            border_width: 1.0,
            border_color: Color {
                a: 0.7,
                ..Color::BLACK
            },
            selected_background: Color {
                a: 0.5,
                ..Color::BLACK
            }
            .into(),
            selected_text_color: Color::WHITE,
        }
    }

    fn active(&self) -> pick_list::Style {
        pick_list::Style {
            text_color: Color::WHITE,
            background: BACKGROUND.into(),
            border_width: 1.0,
            border_color: Color {
                a: 0.6,
                ..Color::BLACK
            },
            border_radius: 2.0,
            icon_size: 0.5,
            ..pick_list::Style::default()
        }
    }

    fn hovered(&self) -> pick_list::Style {
        let active = self.active();

        pick_list::Style {
            border_color: Color {
                a: 0.9,
                ..Color::BLACK
            },
            ..active
        }
    }
}
