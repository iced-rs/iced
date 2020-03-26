//! This example showcases a native custom widget that shows an alert on top of everything
mod alert {
    // For now, to implement a custom native widget you will need to add
    // `iced_native` and `iced_wgpu` to your dependencies.
    //
    // Then, you simply need to define your widget type and implement the
    // `iced_native::Widget` trait with the `iced_wgpu::Renderer`.
    //
    // Of course, you can choose to make the implementation renderer-agnostic,
    // if you wish to, by creating your own `Renderer` trait, which could be
    // implemented by `iced_wgpu` and other renderers.
    use iced_native::{
        layout, Color, Depth, Element, Font, Hasher, HorizontalAlignment,
        Layout, Length, MouseCursor, Point, Rectangle, Size, VerticalAlignment,
        Widget,
    };
    use iced_wgpu::{Defaults, Item, Primitive, Renderer};

    pub struct Alert {
        message: String,
    }

    impl Alert {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }

    impl<Message> Widget<Message, Renderer> for Alert {
        fn width(&self) -> Length {
            Length::Shrink
        }

        fn height(&self) -> Length {
            Length::Shrink
        }

        fn layout(
            &self,
            _renderer: &Renderer,
            _limits: &layout::Limits,
        ) -> layout::Node {
            layout::Node::new(Size::ZERO)
        }

        fn hash_layout(&self, state: &mut Hasher) {
            use std::hash::Hash;

            self.message.hash(state);
        }

        fn draw(
            &self,
            _renderer: &mut Renderer,
            _defaults: &Defaults,
            _layout: Layout<'_>,
            _cursor_position: Point,
        ) -> (Item, MouseCursor) {
            use std::f32;

            (
                (
                    Primitive::Text {
                        content: self.message.clone(),
                        color: Color::from_rgb(0.4, 0.4, 0.4).into(),
                        font: Font::Default,
                        bounds: Rectangle {
                            width: f32::INFINITY,
                            height: 30.,
                            x: 10.,
                            y: 10.,
                        },
                        size: f32::from(30.),
                        horizontal_alignment: HorizontalAlignment::Left,
                        vertical_alignment: VerticalAlignment::Center,
                    },
                    Depth::Below,
                ),
                MouseCursor::Text,
            )
        }
    }

    impl<'a, Message> Into<Element<'a, Message, Renderer>> for Alert {
        fn into(self) -> Element<'a, Message, Renderer> {
            Element::new(self)
        }
    }
}

use alert::Alert;
use iced::{
    button, keyboard, pane_grid, scrollable, Align, Button, Column, Container,
    Element, HorizontalAlignment, Length, PaneGrid, Sandbox, Scrollable,
    Settings, Text,
};

pub fn main() {
    Example::run(Settings::default())
}

struct Example {
    panes: pane_grid::State<Content>,
    panes_created: usize,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Split(pane_grid::Axis, pane_grid::Pane),
    SplitFocused(pane_grid::Axis),
    FocusAdjacent(pane_grid::Direction),
    Dragged(pane_grid::DragEvent),
    Resized(pane_grid::ResizeEvent),
    Close(pane_grid::Pane),
    CloseFocused,
}

impl Sandbox for Example {
    type Message = Message;

    fn new() -> Self {
        let (panes, _) = pane_grid::State::new(Content::new(0));

        Example {
            panes,
            panes_created: 1,
        }
    }

    fn title(&self) -> String {
        String::from("Pane grid - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Split(axis, pane) => {
                let _ = self.panes.split(
                    axis,
                    &pane,
                    Content::new(self.panes_created),
                );

                self.panes_created += 1;
            }
            Message::SplitFocused(axis) => {
                if let Some(pane) = self.panes.active() {
                    let _ = self.panes.split(
                        axis,
                        &pane,
                        Content::new(self.panes_created),
                    );

                    self.panes_created += 1;
                }
            }
            Message::FocusAdjacent(direction) => {
                if let Some(pane) = self.panes.active() {
                    if let Some(adjacent) =
                        self.panes.adjacent(&pane, direction)
                    {
                        self.panes.focus(&adjacent);
                    }
                }
            }
            Message::Resized(pane_grid::ResizeEvent { split, ratio }) => {
                self.panes.resize(&split, ratio);
            }
            Message::Dragged(pane_grid::DragEvent::Dropped {
                pane,
                target,
            }) => {
                self.panes.swap(&pane, &target);
            }
            Message::Dragged(_) => {}
            Message::Close(pane) => {
                let _ = self.panes.close(&pane);
            }
            Message::CloseFocused => {
                if let Some(pane) = self.panes.active() {
                    let _ = self.panes.close(&pane);
                }
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let total_panes = self.panes.len();

        let pane_grid =
            PaneGrid::new(&mut self.panes, |pane, content, focus| {
                content.view(pane, focus, total_panes)
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(10)
            .on_drag(Message::Dragged)
            .on_resize(Message::Resized)
            .on_key_press(handle_hotkey);

        Column::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .push(Alert::new(String::from("I'm an alert")))
            .push(pane_grid)
            .into()
    }
}

fn handle_hotkey(event: pane_grid::KeyPressEvent) -> Option<Message> {
    use keyboard::KeyCode;
    use pane_grid::{Axis, Direction};

    let direction = match event.key_code {
        KeyCode::Up => Some(Direction::Up),
        KeyCode::Down => Some(Direction::Down),
        KeyCode::Left => Some(Direction::Left),
        KeyCode::Right => Some(Direction::Right),
        _ => None,
    };

    match event.key_code {
        KeyCode::V => Some(Message::SplitFocused(Axis::Vertical)),
        KeyCode::H => Some(Message::SplitFocused(Axis::Horizontal)),
        KeyCode::W => Some(Message::CloseFocused),
        _ => direction.map(Message::FocusAdjacent),
    }
}

struct Content {
    id: usize,
    scroll: scrollable::State,
    split_horizontally: button::State,
    split_vertically: button::State,
    close: button::State,
}

impl Content {
    fn new(id: usize) -> Self {
        Content {
            id,
            scroll: scrollable::State::new(),
            split_horizontally: button::State::new(),
            split_vertically: button::State::new(),
            close: button::State::new(),
        }
    }
    fn view(
        &mut self,
        pane: pane_grid::Pane,
        focus: Option<pane_grid::Focus>,
        total_panes: usize,
    ) -> Element<Message> {
        let Content {
            id,
            scroll,
            split_horizontally,
            split_vertically,
            close,
        } = self;

        let button = |state, label, message, style| {
            Button::new(
                state,
                Text::new(label)
                    .width(Length::Fill)
                    .horizontal_alignment(HorizontalAlignment::Center)
                    .size(16),
            )
            .width(Length::Fill)
            .padding(8)
            .on_press(message)
            .style(style)
        };

        let mut controls = Column::new()
            .spacing(5)
            .max_width(150)
            .push(button(
                split_horizontally,
                "Split horizontally",
                Message::Split(pane_grid::Axis::Horizontal, pane),
                style::Button::Primary,
            ))
            .push(button(
                split_vertically,
                "Split vertically",
                Message::Split(pane_grid::Axis::Vertical, pane),
                style::Button::Primary,
            ));

        if total_panes > 1 {
            controls = controls.push(button(
                close,
                "Close",
                Message::Close(pane),
                style::Button::Destructive,
            ));
        }

        let content = Scrollable::new(scroll)
            .width(Length::Fill)
            .spacing(10)
            .align_items(Align::Center)
            .push(Text::new(format!("Pane {}", id)).size(30))
            .push(controls);

        Container::new(Column::new().padding(5).push(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_y()
            .style(style::Pane {
                is_focused: focus.is_some(),
            })
            .into()
    }
}

mod style {
    use iced::{button, container, Background, Color, Vector};

    const SURFACE: Color = Color::from_rgb(
        0xF2 as f32 / 255.0,
        0xF3 as f32 / 255.0,
        0xF5 as f32 / 255.0,
    );

    const ACTIVE: Color = Color::from_rgb(
        0x72 as f32 / 255.0,
        0x89 as f32 / 255.0,
        0xDA as f32 / 255.0,
    );

    const HOVERED: Color = Color::from_rgb(
        0x67 as f32 / 255.0,
        0x7B as f32 / 255.0,
        0xC4 as f32 / 255.0,
    );

    pub struct Pane {
        pub is_focused: bool,
    }

    impl container::StyleSheet for Pane {
        fn style(&self) -> container::Style {
            container::Style {
                background: Some(Background::Color(SURFACE)),
                border_width: 2,
                border_color: Color {
                    a: if self.is_focused { 1.0 } else { 0.3 },
                    ..Color::BLACK
                },
                ..Default::default()
            }
        }
    }

    pub enum Button {
        Primary,
        Destructive,
    }

    impl button::StyleSheet for Button {
        fn active(&self) -> button::Style {
            let (background, text_color) = match self {
                Button::Primary => (Some(ACTIVE), Color::WHITE),
                Button::Destructive => {
                    (None, Color::from_rgb8(0xFF, 0x47, 0x47))
                }
            };

            button::Style {
                text_color,
                background: background.map(Background::Color),
                border_radius: 5,
                shadow_offset: Vector::new(0.0, 0.0),
                ..button::Style::default()
            }
        }

        fn hovered(&self) -> button::Style {
            let active = self.active();

            let background = match self {
                Button::Primary => Some(HOVERED),
                Button::Destructive => Some(Color {
                    a: 0.2,
                    ..active.text_color
                }),
            };

            button::Style {
                background: background.map(Background::Color),
                ..active
            }
        }
    }
}
