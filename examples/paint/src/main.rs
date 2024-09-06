#![allow(dead_code, unused_imports)]

use iced::{
    alignment::{Horizontal, Vertical},
    color,
    theme::palette,
    widget::{
        button, column, container, row, text, tooltip, vertical_rule,
        vertical_slider, vertical_space, Column, Container,
    },
    Color, Element, Font, Length, Theme,
};

const ICON_FONT: Font = Font::with_name("paint-icons");

fn main() -> iced::Result {
    iced::application("Iced Paint", Paint::update, Paint::view)
        .theme(|_| Theme::TokyoNight)
        .antialiasing(true)
        .font(include_bytes!("../fonts/paint-icons.ttf").as_slice())
        .run()
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
enum PaintColor {
    #[default]
    Black,
    White,
    Grey,
    Ivory,
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Indigo,
    Violet,
    Rose,
    Cyan,
    Fuchsia,
    Empty,
    Custom(Color),
}

impl PaintColor {
    const ALL: [PaintColor; 14] = [
        Self::White,
        Self::Black,
        Self::Grey,
        Self::Ivory,
        Self::Red,
        Self::Orange,
        Self::Yellow,
        Self::Green,
        Self::Blue,
        Self::Indigo,
        Self::Violet,
        Self::Fuchsia,
        Self::Rose,
        Self::Cyan,
    ];
}

impl From<PaintColor> for Color {
    fn from(value: PaintColor) -> Self {
        match value {
            PaintColor::Black => color!(0, 0, 0),
            PaintColor::White => color!(255, 255, 255),
            PaintColor::Grey => color!(71, 85, 105),
            PaintColor::Ivory => color!(240, 234, 214),
            PaintColor::Red => color!(255, 0, 0),
            PaintColor::Green => color!(0, 255, 0),
            PaintColor::Blue => color!(0, 0, 255),
            PaintColor::Orange => color!(234, 88, 12),
            PaintColor::Yellow => color!(234, 179, 8),
            PaintColor::Indigo => color!(79, 70, 229),
            PaintColor::Violet => color!(124, 58, 237),
            PaintColor::Rose => color!(225, 29, 72),
            PaintColor::Cyan => color!(8, 145, 178),
            PaintColor::Fuchsia => color!(192, 38, 211),
            PaintColor::Empty => color!(115, 115, 115),
            PaintColor::Custom(color) => color,
        }
    }
}

impl From<Color> for PaintColor {
    fn from(value: Color) -> Self {
        PaintColor::Custom(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Shapes {
    Line,
    Bezier,
    Rectangle,
    Circle,
    Triangle,
    Bestagon,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
enum Tool {
    Pencil,
    Eraser,
    Text,
    #[default]
    Brush,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Action {
    Tool(Tool),
    Select,
    Shape(Shapes),
}

impl Action {
    fn has_opacity(&self) -> bool {
        match self {
            Self::Select => false,
            Self::Shape(_) => true,
            Self::Tool(Tool::Eraser) => false,
            Self::Tool(_) => true,
        }
    }

    fn has_scale(&self) -> bool {
        if let Self::Tool(_) = self {
            true
        } else {
            false
        }
    }
}

impl Default for Action {
    fn default() -> Self {
        Self::Tool(Tool::default())
    }
}

#[derive(Debug, Clone)]
enum Message {
    Selector,
    Tool(Tool),
    Shape(Shapes),
    Color(PaintColor),
    Clear,
    Opacity(f32),
    Scale(f32),
    None,
}

#[derive(Debug)]
struct Paint {
    action: Action,
    color: PaintColor,
    palette: [PaintColor; 18],
    opacity: f32,
    scale: f32,
}

impl Default for Paint {
    fn default() -> Self {
        let palette = [
            PaintColor::White,
            PaintColor::Black,
            PaintColor::Grey,
            PaintColor::Ivory,
            PaintColor::Red,
            PaintColor::Orange,
            PaintColor::Yellow,
            PaintColor::Green,
            PaintColor::Blue,
            PaintColor::Indigo,
            PaintColor::Violet,
            PaintColor::Fuchsia,
            PaintColor::Rose,
            PaintColor::Cyan,
            PaintColor::Empty,
            PaintColor::Empty,
            PaintColor::Empty,
            PaintColor::Empty,
        ];

        Self {
            palette,
            action: Action::default(),
            color: PaintColor::default(),
            opacity: 1.0,
            scale: 1.0,
        }
    }
}

impl Paint {
    fn side_panel(&self) -> Container<'_, Message> {
        let clear = button("Clear")
            .on_press(Message::Clear)
            .style(styles::toolbar_btn);

        let opacity = {
            let slider =
                vertical_slider(0.0..=1.0, self.opacity, Message::Opacity)
                    .default(1.0)
                    .step(0.05)
                    .shift_step(0.1);

            let desc = text("Opacity").size(15.0);

            tooltip(slider, desc, tooltip::Position::Bottom).gap(8.0)
        };

        let scale = {
            let slider = vertical_slider(0.0..=3.0, self.scale, Message::Scale)
                .default(1.0)
                .step(0.1)
                .shift_step(0.1);

            let desc = text("Scale");

            tooltip(slider, desc, tooltip::Position::Bottom).gap(8.0)
        };

        let mut controls = row!().spacing(10);

        if self.action.has_opacity() {
            controls = controls.push(opacity);
        }

        if self.action.has_scale() {
            controls = controls.push(scale);
        }

        let mut content = column!(clear, controls,)
            .padding([8, 3])
            .align_x(Horizontal::Center);

        if self.action.has_scale() || self.action.has_opacity() {
            content = content.spacing(20.0)
        }

        let content =
            container(content).max_height(400.0).style(styles::controls);

        container(content)
            .padding([2, 10])
            .align_y(Vertical::Center)
            .align_x(Horizontal::Center)
            .height(Length::Fill)
    }

    fn colors(&self) -> Column<'_, Message> {
        let description = text("Colors");

        let colors = {
            let mut rw1 = row!().spacing(15);
            let mut rw2 = row!().spacing(15);
            let mut rw3 = row!().spacing(15);

            let colors = self
                .palette
                .iter()
                .map(|color| match color {
                    PaintColor::Empty => (*color, Message::None),
                    _ => (*color, Message::Color(*color)),
                })
                .enumerate();

            for (idx, (color, msg)) in colors {
                let btn = button("").width(20).height(20).on_press(msg).style(
                    move |_, status| styles::color_btn(color.into(), status),
                );

                match idx / 6 {
                    0 => rw1 = rw1.push(btn),
                    1 => rw2 = rw2.push(btn),
                    _ => rw3 = rw3.push(btn),
                }
            }

            column!(rw1, rw2, rw3).spacing(5)
        };

        let current = button("")
            .width(35)
            .height(35)
            .on_press(Message::None)
            .style(|_, status| styles::color_btn(self.color.into(), status));

        let colors =
            row!(current, colors).align_y(Vertical::Center).spacing(10);

        column!(colors, vertical_space(), description)
            .align_x(Horizontal::Center)
            .height(Length::Fill)
    }

    fn toolbar(&self) -> Container<'_, Message> {
        let selector = {
            let icon = text('\u{E847}').size(40.0).font(ICON_FONT);

            let btn = button(icon)
                .on_press(Message::Selector)
                .padding([2, 6])
                .style(styles::toolbar_btn);

            let description = text("Selection");

            column!(btn, vertical_space(), description)
                .align_x(Horizontal::Center)
                .width(75)
                .height(Length::Fill)
        };

        let tools = {
            let tool_btn = |code: char, message: Message| {
                let icon = text(code).font(ICON_FONT);

                button(icon).on_press(message).style(styles::toolbar_btn)
            };

            let rw1 = row!(
                tool_btn('\u{E800}', Message::Tool(Tool::Pencil)),
                tool_btn('\u{F12D}', Message::Tool(Tool::Eraser))
            );
            let rw2 = row!(
                tool_btn('\u{E801}', Message::Tool(Tool::Text)),
                tool_btn('\u{F1FC}', Message::Tool(Tool::Brush))
            );

            let description = text("Tools");

            let tools = column!(rw1, rw2);

            column!(tools, vertical_space(), description)
                .align_x(Horizontal::Center)
                .height(Length::Fill)
        };

        let shapes = {
            let shape_btn = |code: char, msg: Message| {
                let icon = text(code).font(ICON_FONT);

                button(icon).on_press(msg).style(styles::toolbar_btn)
            };

            let rw1 = row!(
                shape_btn('\u{E802}', Message::Shape(Shapes::Line)),
                shape_btn('\u{E803}', Message::Shape(Shapes::Bezier)),
                shape_btn('\u{E804}', Message::Shape(Shapes::Triangle)),
            );

            let rw2 = row!(
                shape_btn('\u{E805}', Message::Shape(Shapes::Rectangle)),
                shape_btn('\u{E806}', Message::Shape(Shapes::Circle)),
                shape_btn('\u{E807}', Message::Shape(Shapes::Bestagon)),
            );

            let description = text("Shapes");

            let shapes = column!(rw1, rw2);

            column!(shapes, vertical_space(), description)
                .align_x(Horizontal::Center)
                .height(Length::Fill)
        };

        container(
            row!(
                selector,
                vertical_rule(2),
                tools,
                vertical_rule(2),
                shapes,
                vertical_rule(2),
                self.colors()
            )
            .width(Length::Fill)
            .height(Length::Fixed(110.0))
            .spacing(10.0)
            .padding([5, 8])
            .align_y(Vertical::Center),
        )
        .style(styles::toolbar)
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Selector => self.action = Action::Select,
            Message::Tool(tool) => self.action = Action::Tool(tool),
            Message::Shape(shape) => self.action = Action::Shape(shape),
            Message::Color(color) => self.color = color,
            Message::Clear => {}
            Message::Opacity(opacity) => self.opacity = opacity,
            Message::Scale(scale) => self.scale = scale,
            Message::None => {}
        }
    }

    fn view(&self) -> Element<Message> {
        let stage = row!(self.side_panel());

        let content = column!(self.toolbar(), stage);

        container(content).into()
    }
}

mod styles {
    use iced::{widget, Background, Border, Color, Theme};

    pub fn toolbar(theme: &Theme) -> widget::container::Style {
        let background = theme.extended_palette().background.weak;

        widget::container::Style {
            background: Some(Background::Color(background.color)),
            text_color: Some(background.text),
            ..Default::default()
        }
    }

    pub fn controls(theme: &Theme) -> widget::container::Style {
        widget::container::Style {
            border: Border {
                radius: 5.0.into(),
                ..Default::default()
            },
            ..toolbar(theme)
        }
    }

    pub fn toolbar_btn(
        theme: &Theme,
        status: widget::button::Status,
    ) -> widget::button::Style {
        match status {
            widget::button::Status::Hovered => {
                let background = theme.extended_palette().background.strong;

                widget::button::Style {
                    background: Some(Background::Color(background.color)),
                    border: Border {
                        radius: 5.0.into(),
                        ..Default::default()
                    },
                    text_color: background.text,
                    ..Default::default()
                }
            }
            _ => {
                let background = theme.extended_palette().background.weak;

                widget::button::Style {
                    background: Some(Background::Color(background.color)),
                    border: Border {
                        radius: 5.0.into(),
                        ..Default::default()
                    },
                    text_color: background.text,
                    ..Default::default()
                }
            }
        }
    }

    pub fn color_btn(
        color: Color,
        status: widget::button::Status,
    ) -> widget::button::Style {
        let background = color;

        match status {
            widget::button::Status::Hovered => widget::button::Style {
                background: Some(Background::Color(background)),
                border: Border {
                    width: 0.0,
                    radius: 100.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            _ => widget::button::Style {
                background: Some(Background::Color(background)),
                border: Border {
                    width: 0.5,
                    radius: 100.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }
}
