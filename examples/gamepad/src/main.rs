use std::collections::HashMap;

use iced::{
    Alignment::*,
    Element, Length, Subscription,
    border::Radius,
    color, gamepad,
    widget::{Row, button, center, column, container, pin, progress_bar, row, space, stack, text},
};

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .run()
}

struct App {
    connected_gamepads: HashMap<gamepad::Id, bool>,
    focused_gamepad: Option<gamepad::Id>,
    pressed_buttons: HashMap<gamepad::Id, HashMap<gamepad::Button, f32>>,
    thumbsticks: HashMap<gamepad::Id, Thumbsticks>,
}

#[derive(Debug, Clone)]
struct Thumbsticks {
    left: Thumbstick,
    right: Thumbstick,
}

#[derive(Debug, Clone)]
struct Thumbstick {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone)]
enum Message {
    FocusGamepad(gamepad::Id),
    GamepadEvent(gamepad::Event),
}

impl App {
    fn new() -> Self {
        Self {
            connected_gamepads: HashMap::new(),
            focused_gamepad: None,
            pressed_buttons: HashMap::new(),
            thumbsticks: HashMap::new(),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::GamepadEvent(event) => match event {
                gamepad::Event::Connected(id) => {
                    if self.focused_gamepad.is_none() {
                        self.focused_gamepad = Some(id);
                    }

                    self.connected_gamepads.insert(id, true);
                }
                gamepad::Event::Disconnected(id) => {
                    self.connected_gamepads.insert(id, false);
                }
                gamepad::Event::ButtonPressed { id, button, .. } => {
                    self.update_button(id, button, 1.0);
                }
                gamepad::Event::ButtonChanged { id, button, value } => {
                    self.update_button(id, button, value);
                }
                gamepad::Event::AxisChanged {
                    id,
                    thumbstick,
                    axis,
                    value,
                } => {
                    self.thumbsticks.entry(id).or_insert_with(Thumbsticks::new);
                    let thumbsticks = self.thumbsticks.get_mut(&id).unwrap();
                    thumbsticks.update(thumbstick, axis, value);
                }
                _ => {}
            },
            Message::FocusGamepad(id) => {
                self.focused_gamepad = Some(id);
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let tabs = self
            .connected_gamepads
            .iter()
            .map(|(gamepad, active)| {
                button(
                    row![
                        container(space()).width(5).height(5).style(if *active {
                            container::success
                        } else {
                            container::danger
                        }),
                        text!("Gamepad {gamepad}")
                    ]
                    .align_y(Center)
                    .spacing(5),
                )
                .on_press_maybe(active.then(|| Message::FocusGamepad(*gamepad)))
                .style(if *active {
                    button::secondary
                } else {
                    button::subtle
                })
                .into()
            })
            .collect::<Row<'_, Message>>()
            .align_y(End)
            .width(Length::Fill);

        if let Some(gamepad) = self.focused_gamepad {
            const BUTTON_SIZE: f32 = 32.0;

            let face_buttons = {
                use gamepad::Button;

                column![
                    row![
                        space::horizontal(),
                        center(space()).style(self.button_style(gamepad, Button::North)),
                        space::horizontal(),
                    ],
                    row![
                        center(space()).style(self.button_style(gamepad, Button::West)),
                        space::horizontal(),
                        center(space()).style(self.button_style(gamepad, Button::East)),
                    ],
                    row![
                        space::horizontal(),
                        center(space()).style(self.button_style(gamepad, Button::South)),
                        space::horizontal(),
                    ],
                ]
                .width(BUTTON_SIZE * 3.0)
                .height(BUTTON_SIZE * 3.0)
            };

            let dpad_buttons = {
                use gamepad::Button;

                column![
                    row![
                        space::horizontal(),
                        center(space()).style(self.dpad_style(gamepad, Button::DPadUp)),
                        space::horizontal(),
                    ],
                    row![
                        center(space()).style(self.dpad_style(gamepad, Button::DPadLeft)),
                        center(space()).style(|theme| {
                            let mut style = container::bordered_box(theme);
                            style.border = style.border.rounded(0);
                            style
                        }),
                        center(space()).style(self.dpad_style(gamepad, Button::DPadRight)),
                    ],
                    row![
                        space::horizontal(),
                        center(space()).style(self.dpad_style(gamepad, Button::DPadDown)),
                        space::horizontal(),
                    ],
                ]
                .width(BUTTON_SIZE * 3.0)
                .height(BUTTON_SIZE * 3.0)
            };

            let triggers = {
                let lt = self.button_value(gamepad, gamepad::Button::LeftTrigger);
                let rt = self.button_value(gamepad, gamepad::Button::RightTrigger);

                let left = progress_bar(0.0..=1.0, lt).girth(10);
                let right = progress_bar(0.0..=1.0, rt).girth(10);

                let shoulders = {
                    let lb = column![
                        "Left bumper",
                        container(space())
                            .width(BUTTON_SIZE)
                            .height(BUTTON_SIZE / 2.0)
                            .style(self.button_style(gamepad, gamepad::Button::LeftShoulder))
                    ]
                    .spacing(10)
                    .align_x(Center);

                    let rb = column![
                        "Right bumper",
                        container(space())
                            .width(BUTTON_SIZE)
                            .height(BUTTON_SIZE / 2.0)
                            .style(self.button_style(gamepad, gamepad::Button::RightShoulder))
                    ]
                    .spacing(10)
                    .align_x(Center);

                    row![lb, rb].spacing(10)
                };

                column![
                    shoulders,
                    column![
                        column![
                            row!["Left trigger", space::horizontal(), text!("{lt:.3}")],
                            left
                        ]
                        .spacing(5)
                        .align_x(Center),
                        column![
                            row!["Right trigger", space::horizontal(), text!("{rt:.3}")],
                            right
                        ]
                        .spacing(5)
                        .align_x(Center),
                    ]
                    .spacing(20),
                ]
                .spacing(10)
                .align_x(Center)
            };

            let thumbsticks = {
                let (left, left_pressed) = (
                    self.thumbstick(gamepad, gamepad::Thumbstick::Left),
                    self.is_button_pressed(gamepad, gamepad::Button::LeftThumb),
                );
                let (right, right_pressed) = (
                    self.thumbstick(gamepad, gamepad::Thumbstick::Right),
                    self.is_button_pressed(gamepad, gamepad::Button::RightThumb),
                );

                column![
                    text!("Thumbsticks").size(24),
                    row![
                        column!["Left", self.thumbstick_graph(left, left_pressed)]
                            .spacing(10)
                            .align_x(Center),
                        column!["Right", self.thumbstick_graph(right, right_pressed)]
                            .spacing(10)
                            .align_x(Center),
                    ]
                    .spacing(20)
                ]
                .spacing(10)
                .align_x(Center)
            };

            let select_and_start = {
                row![
                    column![
                        "Select",
                        container(space())
                            .width(BUTTON_SIZE)
                            .height(BUTTON_SIZE / 2.0)
                            .style(self.button_style(gamepad, gamepad::Button::Select))
                    ]
                    .spacing(10)
                    .align_x(Center),
                    column![
                        "Start",
                        container(space())
                            .width(BUTTON_SIZE)
                            .height(BUTTON_SIZE / 2.0)
                            .style(self.button_style(gamepad, gamepad::Button::Start))
                    ]
                    .spacing(10)
                    .align_x(Center)
                ]
                .spacing(20)
            };

            column![
                tabs,
                container(
                    column![
                        row![
                            column!["D-PAD", dpad_buttons].spacing(10).align_x(Center),
                            triggers,
                            column!["Face buttons", face_buttons]
                                .spacing(10)
                                .align_x(Center),
                        ]
                        .align_y(Center)
                        .spacing(40),
                        thumbsticks,
                        select_and_start,
                    ]
                    .align_x(Center)
                )
                .padding(10)
                .style(|theme| {
                    let mut style = container::bordered_box(theme);
                    style.border = style.border.rounded(0);
                    style.background(color!(0x000000, 0.0))
                })
            ]
            .padding(10)
            .align_x(Center)
            .into()
        } else {
            center("Connect a gamepad to view demo.").into()
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::gamepad::listen().map(Message::GamepadEvent)
    }

    fn thumbstick(&self, id: gamepad::Id, thumbstick: gamepad::Thumbstick) -> Thumbstick {
        if let Some(thumbsticks) = self.thumbsticks.get(&id) {
            thumbsticks.get(thumbstick).clone()
        } else {
            Thumbstick::new()
        }
    }

    fn button_value(&self, id: gamepad::Id, button: gamepad::Button) -> f32 {
        if let Some(gamepad) = self.pressed_buttons.get(&id)
            && let Some(value) = gamepad.get(&button)
        {
            *value
        } else {
            0.0
        }
    }

    fn is_button_pressed(&self, id: gamepad::Id, button: gamepad::Button) -> bool {
        self.button_value(id, button) > 0.0
    }

    fn update_button(&mut self, id: gamepad::Id, button: gamepad::Button, value: f32) {
        self.pressed_buttons.entry(id).or_default();

        let pressed_buttons = self.pressed_buttons.get_mut(&id).unwrap();
        pressed_buttons.insert(button, value);
    }

    fn button_style(
        &self,
        gamepad: gamepad::Id,
        button: gamepad::Button,
    ) -> impl Fn(&iced::Theme) -> container::Style {
        move |theme: &iced::Theme| {
            let mut style = if self.is_button_pressed(gamepad, button.clone()) {
                container::primary(theme)
            } else {
                container::bordered_box(theme)
            };

            style.border = style.border.rounded(500);
            style
        }
    }

    fn dpad_style(
        &self,
        gamepad: gamepad::Id,
        button: gamepad::Button,
    ) -> impl Fn(&iced::Theme) -> container::Style {
        move |theme: &iced::Theme| {
            let mut style = if self.is_button_pressed(gamepad, button.clone()) {
                container::primary(theme)
            } else {
                container::bordered_box(theme)
            };

            let mut radius = Radius::new(0);
            radius = match button {
                gamepad::Button::DPadUp => radius.top(10),
                gamepad::Button::DPadDown => radius.bottom(10),
                gamepad::Button::DPadLeft => radius.left(10),
                gamepad::Button::DPadRight => radius.right(10),
                _ => radius,
            };

            style.border = style.border.rounded(radius);
            style
        }
    }

    fn thumbstick_graph(&self, thumbstick: Thumbstick, pressed: bool) -> Element<'_, Message> {
        const BOX_SIZE: f32 = 256.0;

        // map -1.0 to 0, 1.0 to BOX_SIZE, and interpolate inbetween
        let map_range = |value: f32| ((value + 1.0) * BOX_SIZE) / 2.0;

        let point = pin(container(space())
            .width(10)
            .height(10)
            .style(move |_theme| {
                let mut style = container::Style::default();
                style.border = style.border.rounded(100);
                style.background(if pressed {
                    color!(0x00ff00)
                } else {
                    color!(0xff0000)
                })
            }))
        .x(map_range(thumbstick.x) - 5.0)
        .y(map_range(-thumbstick.y) - 5.0)
        .width(BOX_SIZE)
        .height(BOX_SIZE);

        let container = container(space())
            .width(BOX_SIZE)
            .height(BOX_SIZE)
            .style(|theme| {
                let mut style = container::bordered_box(theme);
                style.border = style.border.rounded(0);
                style
            });

        row![
            row![
                "y",
                progress_bar(-1.0..=1.0, thumbstick.y)
                    .vertical(true)
                    .girth(10)
                    .length(BOX_SIZE)
            ]
            .spacing(5)
            .align_y(Center),
            column![
                stack![container, point].clip(true),
                column![
                    progress_bar(-1.0..=1.0, thumbstick.x)
                        .girth(10)
                        .length(BOX_SIZE),
                    "x"
                ]
                .spacing(5)
                .align_x(Center)
            ]
            .spacing(3),
        ]
        .spacing(3)
        .into()
    }
}

impl Thumbsticks {
    fn new() -> Self {
        Self {
            left: Thumbstick::new(),
            right: Thumbstick::new(),
        }
    }

    fn update(&mut self, thumbstick: gamepad::Thumbstick, axis: gamepad::Axis, value: f32) {
        match thumbstick {
            gamepad::Thumbstick::Left => self.left.update(axis, value),
            gamepad::Thumbstick::Right => self.right.update(axis, value),
        }
    }

    fn get(&self, thumbstick: gamepad::Thumbstick) -> &Thumbstick {
        match thumbstick {
            gamepad::Thumbstick::Left => &self.left,
            gamepad::Thumbstick::Right => &self.right,
        }
    }
}

impl Thumbstick {
    fn new() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    fn update(&mut self, axis: gamepad::Axis, value: f32) {
        match axis {
            gamepad::Axis::Horizontal => self.x = value,
            gamepad::Axis::Vertical => self.y = value,
        }
    }
}
