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

use canvas::{Painting, State};

const ICON_FONT: Font = Font::with_name("paint-icons");

fn main() -> iced::Result {
    iced::application("Iced Paint", Paint::update, Paint::view)
        .theme(|_| Theme::TokyoNight)
        .antialiasing(true)
        .font(include_bytes!("../fonts/paint-icons.ttf").as_slice())
        .run()
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PaintColor {
    Black(f32),
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
        Self::Black(1.0),
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

    fn opacity(&mut self, opacity: f32) -> Self {
        match self {
            Self::Black(_) => Self::Black(opacity),
            _ => *self,
        }
    }
}

impl Default for PaintColor {
    fn default() -> Self {
        Self::Black(1.0)
    }
}

impl From<PaintColor> for Color {
    fn from(value: PaintColor) -> Self {
        match value {
            PaintColor::Black(alpha) => color!(0, 0, 0, alpha),
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
        self != &Self::Select
    }
}

impl Default for Action {
    fn default() -> Self {
        Self::Tool(Tool::default())
    }
}

#[derive(Debug, Clone)]
enum Message {
    Action(Action),
    Color(PaintColor),
    Clear,
    Opacity(f32),
    Scale(f32),
    AddPainting(Painting),
    None,
}

#[derive(Debug)]
struct Paint {
    action: Action,
    color: PaintColor,
    palette: [PaintColor; 18],
    opacity: f32,
    scale: f32,
    drawings: Vec<Painting>,
    canvas: State,
}

impl Default for Paint {
    fn default() -> Self {
        let opacity = 1.0;
        let scale = 1.0;
        let color = PaintColor::default();

        let palette = [
            PaintColor::White,
            PaintColor::Black(opacity),
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

        let mut canvas = State::default();
        canvas.scale(scale);
        canvas.color(color.into());

        Self {
            palette,
            action: Action::default(),
            color,
            opacity,
            scale,
            drawings: Vec::default(),
            canvas,
        }
    }
}

impl Paint {
    fn side_panel(&self) -> Container<'_, Message> {
        let clear = button("Clear")
            .on_press(Message::Clear)
            .style(|theme, status| styles::toolbar_btn(theme, status, false));

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
                .on_press(Message::Action(Action::Select))
                .padding([2, 6])
                .style(|theme, status| {
                    styles::toolbar_btn(
                        theme,
                        status,
                        self.action == Action::Select,
                    )
                });

            let description = text("Selection");

            column!(btn, vertical_space(), description)
                .align_x(Horizontal::Center)
                .width(75)
                .height(Length::Fill)
        };

        let tools = {
            let tool_btn = |code: char, message: Message, tool: Tool| {
                let icon = text(code).font(ICON_FONT);

                button(icon).on_press(message).style(move |theme, status| {
                    styles::toolbar_btn(
                        theme,
                        status,
                        self.action == Action::Tool(tool),
                    )
                })
            };

            let rw1 = row!(
                tool_btn(
                    '\u{E800}',
                    Message::Action(Action::Tool(Tool::Pencil)),
                    Tool::Pencil
                ),
                tool_btn(
                    '\u{F12D}',
                    Message::Action(Action::Tool(Tool::Eraser)),
                    Tool::Eraser
                )
            )
            .spacing(2.5);

            let rw2 = row!(
                tool_btn(
                    '\u{E801}',
                    Message::Action(Action::Tool(Tool::Text)),
                    Tool::Text
                ),
                tool_btn(
                    '\u{F1FC}',
                    Message::Action(Action::Tool(Tool::Brush)),
                    Tool::Brush
                )
            )
            .spacing(2.5);

            let description = text("Tools");

            let tools = column!(rw1, rw2).spacing(2.5);

            column!(tools, vertical_space(), description)
                .align_x(Horizontal::Center)
                .height(Length::Fill)
        };

        let shapes = {
            let shape_btn = |code: char, msg: Message, shape: Shapes| {
                let icon = text(code).font(ICON_FONT);

                button(icon).on_press(msg).style(move |theme, status| {
                    styles::toolbar_btn(
                        theme,
                        status,
                        self.action == Action::Shape(shape),
                    )
                })
            };

            let rw1 = row!(
                shape_btn(
                    '\u{E802}',
                    Message::Action(Action::Shape(Shapes::Line)),
                    Shapes::Line
                ),
                shape_btn(
                    '\u{E803}',
                    Message::Action(Action::Shape(Shapes::Bezier)),
                    Shapes::Bezier
                ),
                shape_btn(
                    '\u{E804}',
                    Message::Action(Action::Shape(Shapes::Triangle)),
                    Shapes::Triangle
                ),
            )
            .spacing(2.5);

            let rw2 = row!(
                shape_btn(
                    '\u{E805}',
                    Message::Action(Action::Shape(Shapes::Rectangle)),
                    Shapes::Rectangle
                ),
                shape_btn(
                    '\u{E806}',
                    Message::Action(Action::Shape(Shapes::Circle)),
                    Shapes::Circle
                ),
                shape_btn(
                    '\u{E807}',
                    Message::Action(Action::Shape(Shapes::Bestagon)),
                    Shapes::Bestagon
                ),
            )
            .spacing(2.5);

            let description = text("Shapes");

            let shapes = column!(rw1, rw2).spacing(2.5);

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
            Message::Action(action) => {
                self.action = action;
                self.canvas.action(action);
            }
            Message::Color(color) => {
                self.color = color;
                self.canvas.color(self.color.opacity(self.opacity).into());
            }
            Message::Clear => {
                self.drawings.clear();
                self.canvas.redraw()
            }
            Message::Opacity(opacity) => {
                self.opacity = opacity;
                self.canvas.color(self.color.opacity(self.opacity).into());
            }
            Message::Scale(scale) => {
                self.scale = scale;
                self.canvas.scale(scale);
            }
            Message::AddPainting(painting) => {
                self.drawings.push(painting);
                self.canvas.redraw();
            }
            Message::None => {}
        }
    }

    fn view(&self) -> Element<Message> {
        let stage = row!(
            self.side_panel(),
            self.canvas.view(&self.drawings).map(Message::AddPainting)
        )
        .width(Length::Fill)
        .spacing(10.0)
        .padding([6, 6]);

        let content = column!(self.toolbar(), stage);

        container(content).into()
    }
}

mod canvas {

    use iced::{
        advanced::graphics::core::SmolStr,
        color, mouse,
        widget::canvas::{
            self,
            event::{self, Event},
            stroke, Canvas, Frame, Geometry, LineDash, Path, Stroke, Text,
        },
        Color, Element, Fill, Point, Rectangle, Renderer, Size, Theme,
    };

    use super::{Action, Shapes, Tool};

    const TEXT_LEFT_PADDING: f32 = 0.005;
    const TEXT_TOP_PADDING: f32 = 0.005;
    const SHAPE_DEFAULT_THICKNESS: f32 = 3.0;

    #[derive(Default, Debug)]
    pub struct State {
        cache: canvas::Cache,
        current_action: Action,
        color: Color,
        scale: f32,
    }

    impl State {
        pub fn redraw(&mut self) {
            self.cache.clear()
        }

        pub fn action(&mut self, action: Action) {
            self.current_action = action;
        }

        pub fn color(&mut self, color: Color) {
            self.color = color;
        }

        pub fn scale(&mut self, scale: f32) {
            self.scale = scale;
        }

        pub fn view<'a>(
            &'a self,
            paintings: &'a [Painting],
        ) -> Element<'a, Painting> {
            Canvas::new(PaintingCanvas {
                state: &self,
                paintings,
            })
            .width(Fill)
            .height(Fill)
            .into()
        }
    }

    struct PaintingCanvas<'a> {
        state: &'a State,
        paintings: &'a [Painting],
    }

    impl<'a> canvas::Program<Painting> for PaintingCanvas<'a> {
        type State = Option<Pending>;

        fn update(
            &self,
            state: &mut Self::State,
            event: Event,
            bounds: Rectangle,
            cursor: mouse::Cursor,
        ) -> (event::Status, Option<Painting>) {
            match (cursor.position_in(bounds), state.clone()) {
                (
                    Some(cursor_position),
                    Some(Pending::Text(TextPending::Typing {
                        from,
                        to,
                        text: mut state_text,
                    })),
                ) if self.state.current_action == Action::Tool(Tool::Text) => {
                    match event {
                        Event::Keyboard(
                            iced::keyboard::Event::KeyPressed {
                                text: Some(new_text),
                                ..
                            },
                        ) => {
                            if &new_text == "\u{8}" {
                                state_text.pop();
                            } else {
                                state_text.push_str(&new_text);
                            }

                            state.replace(Pending::Text(TextPending::Typing {
                                from,
                                to,
                                text: state_text,
                            }));

                            return (event::Status::Captured, None);
                        }
                        Event::Mouse(mouse::Event::ButtonPressed(
                            mouse::Button::Left,
                        )) => {
                            let bounds = Rectangle::new(
                                from,
                                Size::new(to.x - from.x, to.y - from.y),
                            );
                            if !bounds.contains(cursor_position) {
                                let painting = Painting::Text {
                                    top_left: from,
                                    bottom_right: to,
                                    text: state_text.clone(),
                                    color: self.state.color,
                                    scale: self.state.scale,
                                };

                                state.take();

                                if bounds.area() == 0.0 {
                                    return (event::Status::Captured, None);
                                }

                                return (
                                    event::Status::Captured,
                                    Some(painting),
                                );
                            }
                        }

                        _ => {}
                    }
                }

                (
                    _,
                    Some(Pending::Text(TextPending::Typing {
                        text: mut state_text,
                        from,
                        to,
                    })),
                ) => match event {
                    Event::Keyboard(iced::keyboard::Event::KeyPressed {
                        text: Some(new_text),
                        ..
                    }) => {
                        state_text.push_str(&new_text);

                        state.replace(Pending::Text(TextPending::Typing {
                            from,
                            to,
                            text: state_text,
                        }));

                        return (event::Status::Captured, None);
                    }
                    _ => {}
                },

                (
                    Some(cursor_position),
                    Some(Pending::FreeForm(prev_points)),
                ) => match event {
                    Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                        let updated = {
                            let mut points = prev_points;

                            if points.len() <= 1 {
                                points.push(cursor_position);
                            } else {
                                match points.pop() {
                                    Some(prev) => {
                                        if prev.x == cursor_position.x {
                                            points.push(cursor_position);
                                        } else if prev.y == cursor_position.y {
                                            points.push(cursor_position);
                                        } else {
                                            points.push(prev);
                                            points.push(cursor_position)
                                        }
                                    }
                                    None => points.push(cursor_position),
                                };
                            }

                            Pending::FreeForm(points)
                        };

                        state.replace(updated);

                        return (event::Status::Captured, None);
                    }

                    Event::Mouse(mouse::Event::ButtonReleased(
                        mouse::Button::Left,
                    )) => {
                        let painting = Painting::new_freeform(
                            self.state.current_action,
                            prev_points.clone(),
                            self.state.color,
                            self.state.scale,
                        );

                        state.take();

                        return (event::Status::Captured, painting);
                    }
                    _ => {}
                },

                (Some(cursor_position), _unused_state) => match event {
                    Event::Mouse(mouse::Event::ButtonReleased(
                        mouse::Button::Left,
                    )) if self.state.current_action
                        == Action::Tool(Tool::Text) =>
                    {
                        match state {
                            Some(Pending::Text(TextPending::One { from })) => {
                                let (from, to) =
                                    orient_points(*from, cursor_position);
                                let typing =
                                    Pending::Text(TextPending::Typing {
                                        from,
                                        to,
                                        text: String::default(),
                                    });

                                state.replace(typing);
                                return (event::Status::Captured, None);
                            }
                            Some(_) => {
                                panic!("Drawing while typing tool is selected")
                            }
                            None => {}
                        }
                    }

                    Event::Mouse(mouse::Event::ButtonReleased(
                        mouse::Button::Left,
                    )) if self.state.current_action
                        == Action::Shape(Shapes::Bezier) =>
                    {
                        match state {
                            Some(Pending::One { from }) => {
                                let pending = Pending::Two {
                                    from: *from,
                                    to: cursor_position,
                                };

                                state.replace(pending);
                                return (event::Status::Captured, None);
                            }
                            Some(Pending::Text(_)) => {
                                panic!("Typing while bezier tool is selected")
                            }
                            _ => {}
                        }
                    }

                    Event::Mouse(mouse::Event::ButtonReleased(
                        mouse::Button::Left,
                    )) => match state {
                        Some(Pending::One { from }) => {
                            let bounds = Rectangle::new(
                                *from,
                                Size::new(
                                    cursor_position.x - from.x,
                                    cursor_position.y - from.y,
                                ),
                            );

                            let painting = Painting::new(
                                self.state.current_action,
                                *from,
                                cursor_position,
                                self.state.color,
                                self.state.scale,
                            );
                            state.take();

                            if bounds.area() == 0.0 {
                                return (event::Status::Captured, None);
                            }

                            return (event::Status::Captured, Some(painting));
                        }
                        Some(Pending::Two { from, .. }) => {
                            let bounds = Rectangle::new(
                                *from,
                                Size::new(
                                    cursor_position.x - from.x,
                                    cursor_position.y - from.y,
                                ),
                            );

                            let painting = Painting::new(
                                self.state.current_action,
                                *from,
                                cursor_position,
                                self.state.color,
                                self.state.scale,
                            );
                            state.take();

                            if bounds.area() == 0.0 {
                                return (event::Status::Captured, None);
                            }

                            return (event::Status::Captured, Some(painting));
                        }
                        Some(Pending::FreeForm(_points)) => {}
                        Some(Pending::Text(_)) => {
                            panic!("Typing when text tool not selected")
                        }

                        None => {}
                    },

                    Event::Mouse(mouse::Event::ButtonPressed(
                        mouse::Button::Left,
                    )) => match state {
                        Some(Pending::Two { from, to })
                            if self.state.current_action
                                == Action::Shape(Shapes::Bezier) =>
                        {
                            let painting = Painting::Bezier {
                                from: *from,
                                to: *to,
                                control: cursor_position,
                                scale: self.state.scale,
                                color: self.state.color,
                            };
                            state.take();

                            return (event::Status::Captured, Some(painting));
                        }
                        Some(Pending::Text(TextPending::Typing {
                            from,
                            to,
                            text,
                        })) if self.state.current_action
                            == Action::Tool(Tool::Text) =>
                        {
                            let bounds = Rectangle::new(
                                *from,
                                Size::new(to.x - from.x, to.y - from.y),
                            );
                            if !bounds.contains(cursor_position) {
                                let painting = Painting::Text {
                                    top_left: *from,
                                    bottom_right: *to,
                                    text: text.clone(),
                                    color: self.state.color,
                                    scale: self.state.scale,
                                };

                                state.take();

                                if bounds.area() == 0.0 {
                                    return (event::Status::Captured, None);
                                }

                                return (
                                    event::Status::Captured,
                                    Some(painting),
                                );
                            }
                        }
                        Some(_) => {}
                        None => {
                            let pending = match self.state.current_action {
                                Action::Tool(Tool::Text) => {
                                    Pending::Text(TextPending::One {
                                        from: cursor_position,
                                    })
                                }
                                Action::Tool(Tool::Brush)
                                | Action::Tool(Tool::Pencil) => {
                                    Pending::FreeForm(vec![cursor_position])
                                }
                                _ => Pending::One {
                                    from: cursor_position,
                                },
                            };

                            state.replace(pending);

                            return (event::Status::Captured, None);
                        }
                    },

                    _ => {}
                },
                _ => {}
            };

            return (event::Status::Ignored, None);
        }

        fn draw(
            &self,
            state: &Self::State,
            renderer: &Renderer,
            theme: &Theme,
            bounds: Rectangle,
            cursor: iced::advanced::mouse::Cursor,
        ) -> Vec<Geometry<Renderer>> {
            let content =
                self.state.cache.draw(renderer, bounds.size(), |frame| {
                    frame.fill_rectangle(
                        Point::ORIGIN,
                        frame.size(),
                        color!(240, 234, 214),
                    );

                    Painting::draw_all(self.paintings, frame, bounds, theme);
                });

            if let Some(pending) = state {
                vec![
                    content,
                    pending.draw(
                        renderer,
                        bounds,
                        cursor,
                        self.state.current_action,
                        self.state.color,
                        self.state.scale,
                    ),
                ]
            } else {
                vec![content]
            }
        }

        fn mouse_interaction(
            &self,
            state: &Self::State,
            bounds: Rectangle,
            cursor: mouse::Cursor,
        ) -> mouse::Interaction {
            match state {
                Some(Pending::Text(TextPending::One { .. }))
                    if cursor.is_over(bounds) =>
                {
                    mouse::Interaction::Text
                }
                Some(_) | None if cursor.is_over(bounds) => {
                    mouse::Interaction::Crosshair
                }

                _ => mouse::Interaction::default(),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum Painting {
        FreeForm {
            points: Vec<Point>,
            is_pencil: bool,
            color: Color,
            scale: f32,
        },
        Text {
            top_left: Point,
            bottom_right: Point,
            text: String,
            color: Color,
            scale: f32,
        },
        Line {
            from: Point,
            to: Point,
            color: Color,
            scale: f32,
        },
        Bezier {
            from: Point,
            to: Point,
            control: Point,
            color: Color,
            scale: f32,
        },
        Rectangle {
            top_left: Point,
            bottom_right: Point,
            color: Color,
            scale: f32,
        },
        Circle {
            center: Point,
            radius: Point,
            color: Color,
            scale: f32,
        },
        Triangle {
            top: Point,
            right: Point,
            color: Color,
            scale: f32,
        },
        Bestagon {
            top: Point,
            top_right: Point,
            color: Color,
            scale: f32,
        },
        Eraser {
            scale: f32,
        },
        Select {
            top_left: Point,
            bottom_right: Point,
            color: Color,
            scale: f32,
        },
    }

    impl Painting {
        fn new(
            action: Action,
            from: Point,
            to: Point,
            color: Color,
            scale: f32,
        ) -> Self {
            match action {
                Action::Tool(Tool::Text) => Self::Text {
                    top_left: from,
                    bottom_right: to,
                    text: String::from("Text painting here invalid"),
                    color,
                    scale,
                },
                Action::Tool(Tool::Brush) => Self::FreeForm {
                    points: vec![from, to],
                    is_pencil: false,
                    color,
                    scale,
                },
                Action::Tool(Tool::Pencil) => Self::FreeForm {
                    points: vec![from, to],
                    is_pencil: true,
                    color,
                    scale,
                },
                Action::Tool(Tool::Eraser) => Self::Eraser { scale },
                Action::Select => Self::Select {
                    top_left: from,
                    bottom_right: to,
                    color,
                    scale,
                },
                Action::Shape(Shapes::Rectangle) => Self::Rectangle {
                    top_left: from,
                    bottom_right: to,
                    color,
                    scale,
                },
                Action::Shape(Shapes::Line) => Self::Line {
                    from,
                    to,
                    color,
                    scale,
                },
                Action::Shape(Shapes::Triangle) => Self::Triangle {
                    top: from,
                    right: to,
                    color,
                    scale,
                },
                Action::Shape(Shapes::Circle) => Self::Circle {
                    center: from,
                    radius: to,
                    color,
                    scale,
                },
                Action::Shape(Shapes::Bestagon) => Self::Bestagon {
                    top: from,
                    top_right: to,
                    color,
                    scale,
                },
                Action::Shape(Shapes::Bezier) => Self::Bezier {
                    from,
                    to,
                    control: to,
                    color,
                    scale,
                },
            }
        }

        fn new_freeform(
            action: Action,
            points: Vec<Point>,
            color: Color,
            scale: f32,
        ) -> Option<Self> {
            match action {
                Action::Tool(Tool::Pencil) => Some(Self::FreeForm {
                    points,
                    color,
                    scale,
                    is_pencil: true,
                }),
                Action::Tool(Tool::Brush) => Some(Self::FreeForm {
                    points,
                    color,
                    scale,
                    is_pencil: false,
                }),
                _ => None,
            }
        }

        fn draw_all(
            paintings: &[Self],
            frame: &mut Frame,
            bounds: Rectangle,
            _theme: &Theme,
        ) {
            for painting in paintings.iter() {
                match painting {
                    Painting::Text {
                        top_left,
                        text,
                        color,
                        scale,
                        ..
                    } => Painting::draw_text(
                        frame,
                        bounds,
                        text.clone(),
                        *top_left,
                        *color,
                        *scale,
                    ),
                    Painting::Bezier {
                        from,
                        to,
                        control,
                        color,
                        scale,
                    } => Painting::draw_bezier(
                        frame, *from, *to, *control, *color, *scale,
                    ),
                    Painting::Line {
                        from,
                        to,
                        color,
                        scale,
                    } => Painting::draw_line(frame, *from, *to, *color, *scale),

                    Painting::Rectangle {
                        top_left,
                        bottom_right,
                        color,
                        scale,
                    } => Painting::draw_rect(
                        frame,
                        *top_left,
                        *bottom_right,
                        *color,
                        *scale,
                    ),
                    Painting::Circle {
                        center,
                        radius,
                        color,
                        scale,
                    } => Painting::draw_circle(
                        frame, *center, *radius, *color, *scale,
                    ),

                    Painting::Triangle {
                        top,
                        right,
                        color,
                        scale,
                    } => Painting::draw_triangle(
                        frame, *top, *right, *color, *scale,
                    ),

                    Painting::Bestagon {
                        top,
                        top_right,
                        color,
                        scale,
                    } => Painting::draw_bestagon(
                        frame, *top, *top_right, *color, *scale,
                    ),

                    Painting::FreeForm {
                        points,
                        color,
                        scale,
                        is_pencil,
                    } => Painting::draw_freeform(
                        frame, points, *color, *scale, *is_pencil,
                    ),

                    _ => {}
                }
            }
        }

        fn draw_text(
            frame: &mut Frame,
            bounds: Rectangle,
            text: String,
            top_left: Point,
            color: Color,
            scale: f32,
        ) {
            if text.is_empty() || bounds.area() == 0.0 {
                return;
            }

            let size = (16.0 * scale.max(0.1)).into();

            let position = {
                let left = bounds.width * TEXT_LEFT_PADDING;
                let top = bounds.height * TEXT_TOP_PADDING;

                Point::new(top_left.x + left, top_left.y + top)
            };

            let text = Text {
                content: text.clone(),
                position,
                color,
                size,
                ..Default::default()
            };

            frame.fill_text(text)
        }

        fn draw_bezier(
            frame: &mut Frame,
            from: Point,
            to: Point,
            control: Point,
            color: Color,
            scale: f32,
        ) {
            let curve = Path::new(|builder| {
                builder.move_to(from);
                builder.quadratic_curve_to(control, to)
            });

            frame.stroke(
                &curve,
                Stroke::default()
                    .with_width(SHAPE_DEFAULT_THICKNESS * scale)
                    .with_color(color),
            )
        }

        fn draw_line(
            frame: &mut Frame,
            from: Point,
            to: Point,
            color: Color,
            scale: f32,
        ) {
            let line = Path::line(from, to);

            frame.stroke(
                &line,
                Stroke::default()
                    .with_color(color)
                    .with_width(SHAPE_DEFAULT_THICKNESS * scale),
            )
        }

        fn draw_rect(
            frame: &mut Frame,
            from: Point,
            to: Point,
            color: Color,
            scale: f32,
        ) {
            let (from, to) = orient_points(from, to);

            let size = Size::new(to.x - from.x, to.y - from.y);

            let rect = Path::rectangle(from, size);

            frame.stroke(
                &rect,
                Stroke::default()
                    .with_width(SHAPE_DEFAULT_THICKNESS * scale)
                    .with_color(color),
            )
        }

        fn draw_circle(
            frame: &mut Frame,
            center: Point,
            to: Point,
            color: Color,
            scale: f32,
        ) {
            let (center, to) = orient_points(center, to);

            let radius = center.distance(to);

            let cirlce = Path::circle(center, radius);

            frame.stroke(
                &cirlce,
                Stroke::default()
                    .with_width(SHAPE_DEFAULT_THICKNESS * scale)
                    .with_color(color),
            )
        }

        fn draw_triangle(
            frame: &mut Frame,
            top: Point,
            right: Point,
            color: Color,
            scale: f32,
        ) {
            let scale = SHAPE_DEFAULT_THICKNESS * scale;
            let triangle = Path::new(|builder| {
                let left_x = (right.x - top.x) * 2.0;
                let left = Point::new(right.x - left_x, right.y);

                builder.move_to(top);
                builder.line_to(right);
                builder.line_to(left);
                builder.line_to(top);
            });

            frame.stroke(
                &triangle,
                Stroke::default().with_color(color).with_width(scale),
            );
        }

        fn draw_bestagon(
            frame: &mut Frame,
            top: Point,
            right: Point,
            color: Color,
            scale: f32,
        ) {
            let scale = SHAPE_DEFAULT_THICKNESS * scale;

            let bestagon = Path::new(|builder| {
                let x_diff = right.x - top.x;
                let y_diff = right.y - top.y;

                builder.move_to(top);
                builder.line_to(right);
                builder.line_to(Point::new(right.x, right.y + y_diff));
                builder.line_to(Point::new(
                    right.x - x_diff,
                    right.y + (y_diff * 2.0),
                ));
                builder.line_to(Point::new(
                    right.x - (x_diff * 2.0),
                    right.y + y_diff,
                ));
                builder.line_to(Point::new(right.x - (x_diff * 2.0), right.y));

                builder.line_to(top);
            });

            frame.stroke(
                &bestagon,
                Stroke::default().with_color(color).with_width(scale),
            );
        }

        fn draw_freeform(
            frame: &mut Frame,
            points: &[Point],
            color: Color,
            scale: f32,
            is_pencil: bool,
        ) {
            let scale = if is_pencil {
                1.5 * scale
            } else {
                SHAPE_DEFAULT_THICKNESS * scale
            };

            let stroke = if is_pencil {
                Stroke {
                    width: scale,
                    style: stroke::Style::Solid(color),
                    ..Default::default()
                }
            } else {
                Stroke {
                    width: scale,
                    line_cap: stroke::LineCap::Round,
                    style: stroke::Style::Solid(color),
                    ..Default::default()
                }
            };

            let freeform = Path::new(|builder| {
                for (idx, point) in points.iter().enumerate() {
                    let point = *point;
                    if idx == 0 {
                        builder.move_to(point);
                    }

                    builder.line_to(point);
                }
            });

            frame.stroke(&freeform, stroke);
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    enum Pending {
        Text(TextPending),
        FreeForm(Vec<Point>),
        One { from: Point },
        Two { from: Point, to: Point },
    }

    impl Pending {
        fn draw(
            &self,
            renderer: &Renderer,
            bounds: Rectangle,
            cursor: mouse::Cursor,
            action: Action,
            color: Color,
            scale: f32,
        ) -> Geometry {
            let mut frame = Frame::new(renderer, bounds.size());

            match action {
                Action::Tool(Tool::Text) => match self {
                    Self::Text(text) => {
                        text.draw(&mut frame, bounds, cursor, color, scale)
                    }
                    _ => {}
                },

                Action::Shape(Shapes::Bezier) => match self {
                    Self::One { from } => {
                        if let Some(to) = cursor.position_in(bounds) {
                            Painting::draw_line(
                                &mut frame, *from, to, color, scale,
                            )
                        }
                    }
                    Self::Two { from, to } => {
                        if let Some(control) = cursor.position_in(bounds) {
                            Painting::draw_bezier(
                                &mut frame, *from, *to, control, color, scale,
                            )
                        }
                    }
                    _ => {}
                },

                Action::Shape(Shapes::Line) => match self {
                    Self::One { from } => {
                        if let Some(to) = cursor.position_in(bounds) {
                            Painting::draw_line(
                                &mut frame, *from, to, color, scale,
                            )
                        }
                    }
                    Self::Two { from, to } => Painting::draw_line(
                        &mut frame, *from, *to, color, scale,
                    ),
                    _ => {}
                },

                Action::Shape(Shapes::Rectangle) => match self {
                    Self::One { from } => {
                        if let Some(cursor_position) =
                            cursor.position_in(bounds)
                        {
                            Painting::draw_rect(
                                &mut frame,
                                *from,
                                cursor_position,
                                color,
                                scale,
                            )
                        }
                    }
                    Self::Two { from, to } => Painting::draw_rect(
                        &mut frame, *from, *to, color, scale,
                    ),
                    _ => {}
                },

                Action::Shape(Shapes::Circle) => match self {
                    Self::One { from } => {
                        if let Some(cursor_position) =
                            cursor.position_in(bounds)
                        {
                            Painting::draw_circle(
                                &mut frame,
                                *from,
                                cursor_position,
                                color,
                                scale,
                            )
                        }
                    }
                    Self::Two { from, to } => Painting::draw_circle(
                        &mut frame, *from, *to, color, scale,
                    ),
                    _ => {}
                },

                Action::Shape(Shapes::Triangle) => match self {
                    Self::One { from } => {
                        if let Some(cursor_position) =
                            cursor.position_in(bounds)
                        {
                            Painting::draw_triangle(
                                &mut frame,
                                *from,
                                cursor_position,
                                color,
                                scale,
                            )
                        }
                    }
                    Self::Two { from, to } => Painting::draw_triangle(
                        &mut frame, *from, *to, color, scale,
                    ),
                    _ => {}
                },

                Action::Shape(Shapes::Bestagon) => match self {
                    Self::One { from } => {
                        if let Some(cursor_position) =
                            cursor.position_in(bounds)
                        {
                            Painting::draw_bestagon(
                                &mut frame,
                                *from,
                                cursor_position,
                                color,
                                scale,
                            )
                        }
                    }
                    Self::Two { from, to } => Painting::draw_bestagon(
                        &mut frame, *from, *to, color, scale,
                    ),
                    _ => {}
                },

                Action::Tool(Tool::Brush) => match self {
                    Self::FreeForm(points) => Painting::draw_freeform(
                        &mut frame, points, color, scale, false,
                    ),

                    _ => {}
                },

                Action::Tool(Tool::Pencil) => match self {
                    Self::FreeForm(points) => Painting::draw_freeform(
                        &mut frame, points, color, scale, true,
                    ),

                    _ => {}
                },
                _ => {}
            }

            frame.into_geometry()
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    enum TextPending {
        One {
            from: Point,
        },
        Typing {
            from: Point,
            to: Point,
            text: String,
        },
    }

    impl TextPending {
        fn draw(
            &self,
            frame: &mut Frame,
            bounds: Rectangle,
            cursor: mouse::Cursor,
            color: Color,
            scale: f32,
        ) {
            let line_dash = LineDash {
                offset: 0,
                segments: &[4.0, 0.0, 4.0],
            };

            let stroke = Stroke {
                line_dash,
                style: stroke::Style::Solid(color),
                width: 2.0,
                ..Default::default()
            };

            match self {
                Self::One { from } => {
                    if let Some(cursor_position) = cursor.position_in(bounds) {
                        let size = Size::new(
                            cursor_position.x - from.x,
                            cursor_position.y - from.y,
                        );
                        let rect = Path::rectangle(*from, size);
                        frame.stroke(&rect, stroke);
                    }
                }
                Self::Typing { from, to, text } => {
                    let size = Size::new(to.x - from.x, to.y - from.y);
                    let rect = Path::rectangle(*from, size);
                    frame.stroke(&rect, stroke);

                    Painting::draw_text(
                        frame,
                        bounds,
                        text.clone(),
                        *from,
                        color,
                        scale,
                    );
                }
            }
        }
    }

    /// Determines the top left and bottom right points
    fn orient_points(iden: Point, other: Point) -> (Point, Point) {
        if other.y <= iden.y {
            let top_left = Point::new(f32::min(iden.x, other.x), other.y);
            let bottom_right = Point::new(f32::max(iden.x, other.x), iden.y);
            (top_left, bottom_right)
        } else {
            let top_left = Point::new(f32::min(iden.x, other.x), iden.y);
            let bottom_right = Point::new(f32::max(iden.x, other.x), other.y);
            (top_left, bottom_right)
        }
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
        selected: bool,
    ) -> widget::button::Style {
        let background = match status {
            widget::button::Status::Hovered => {
                theme.extended_palette().background.strong
            }
            _status if selected => theme.extended_palette().background.strong,
            _ => theme.extended_palette().background.weak,
        };

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
