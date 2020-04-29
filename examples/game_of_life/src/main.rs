//! This example showcases an interactive version of the Game of Life, invented
//! by John Conway. It leverages a `Canvas` together with other widgets.
mod style;
mod time;

use grid::Grid;
use iced::{
    button::{self, Button},
    executor,
    slider::{self, Slider},
    Align, Application, Column, Command, Container, Element, Length, Row,
    Settings, Subscription, Text,
};
use std::time::{Duration, Instant};

pub fn main() {
    GameOfLife::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}

#[derive(Default)]
struct GameOfLife {
    grid: Grid,
    is_playing: bool,
    speed: u64,
    next_speed: Option<u64>,
    toggle_button: button::State,
    next_button: button::State,
    clear_button: button::State,
    speed_slider: slider::State,
}

#[derive(Debug, Clone)]
enum Message {
    Grid(grid::Message),
    Tick(Instant),
    Toggle,
    Next,
    Clear,
    SpeedChanged(f32),
}

impl Application for GameOfLife {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self {
                speed: 1,
                ..Self::default()
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Game of Life - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Grid(message) => {
                self.grid.update(message);
            }
            Message::Tick(_) | Message::Next => {
                self.grid.tick();

                if let Some(speed) = self.next_speed.take() {
                    self.speed = speed;
                }
            }
            Message::Toggle => {
                self.is_playing = !self.is_playing;
            }
            Message::Clear => {
                self.grid = Grid::default();
            }
            Message::SpeedChanged(speed) => {
                if self.is_playing {
                    self.next_speed = Some(speed.round() as u64);
                } else {
                    self.speed = speed.round() as u64;
                }
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.is_playing {
            time::every(Duration::from_millis(1000 / self.speed))
                .map(Message::Tick)
        } else {
            Subscription::none()
        }
    }

    fn view(&mut self) -> Element<Message> {
        let playback_controls = Row::new()
            .spacing(10)
            .push(
                Button::new(
                    &mut self.toggle_button,
                    Text::new(if self.is_playing { "Pause" } else { "Play" }),
                )
                .on_press(Message::Toggle)
                .style(style::Button),
            )
            .push(
                Button::new(&mut self.next_button, Text::new("Next"))
                    .on_press(Message::Next)
                    .style(style::Button),
            )
            .push(
                Button::new(&mut self.clear_button, Text::new("Clear"))
                    .on_press(Message::Clear)
                    .style(style::Button),
            );

        let selected_speed = self.next_speed.unwrap_or(self.speed);
        let speed_controls = Row::new()
            .spacing(10)
            .push(
                Slider::new(
                    &mut self.speed_slider,
                    1.0..=20.0,
                    selected_speed as f32,
                    Message::SpeedChanged,
                )
                .width(Length::Units(200))
                .style(style::Slider),
            )
            .push(Text::new(format!("x{}", selected_speed)).size(16))
            .align_items(Align::Center);

        let controls = Row::new()
            .spacing(20)
            .push(playback_controls)
            .push(speed_controls);

        let content = Column::new()
            .spacing(10)
            .padding(10)
            .align_items(Align::Center)
            .push(self.grid.view().map(Message::Grid))
            .push(controls);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(style::Container)
            .into()
    }
}

mod grid {
    use iced::{
        canvas::{self, Canvas, Cursor, Event, Frame, Geometry, Path},
        mouse, ButtonState, Color, Element, Length, MouseCursor, Point,
        Rectangle, Size, Vector,
    };

    const SIZE: usize = 32;

    #[derive(Default)]
    pub struct Grid {
        cells: [[Cell; SIZE]; SIZE],
        mouse_pressed: bool,
        cache: canvas::Cache,
    }

    impl Grid {
        pub fn tick(&mut self) {
            let mut populated_neighbors: [[usize; SIZE]; SIZE] =
                [[0; SIZE]; SIZE];

            for (i, row) in self.cells.iter().enumerate() {
                for (j, _) in row.iter().enumerate() {
                    populated_neighbors[i][j] = self.populated_neighbors(i, j);
                }
            }

            for (i, row) in populated_neighbors.iter().enumerate() {
                for (j, amount) in row.iter().enumerate() {
                    let is_populated = self.cells[i][j] == Cell::Populated;

                    self.cells[i][j] = match amount {
                        2 if is_populated => Cell::Populated,
                        3 => Cell::Populated,
                        _ => Cell::Unpopulated,
                    };
                }
            }

            self.cache.clear()
        }

        pub fn update(&mut self, message: Message) {
            match message {
                Message::Populate { i, j } => {
                    self.cells[i][j] = Cell::Populated;
                    self.cache.clear()
                }
            }
        }

        pub fn view<'a>(&'a mut self) -> Element<'a, Message> {
            Canvas::new(self)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }

        fn populated_neighbors(&self, row: usize, column: usize) -> usize {
            use itertools::Itertools;

            let rows = row.saturating_sub(1)..=row + 1;
            let columns = column.saturating_sub(1)..=column + 1;

            let is_inside_bounds = |i: usize, j: usize| i < SIZE && j < SIZE;
            let is_neighbor = |i: usize, j: usize| i != row || j != column;

            let is_populated =
                |i: usize, j: usize| self.cells[i][j] == Cell::Populated;

            rows.cartesian_product(columns)
                .filter(|&(i, j)| {
                    is_inside_bounds(i, j)
                        && is_neighbor(i, j)
                        && is_populated(i, j)
                })
                .count()
        }

        fn region(&self, size: Size) -> Rectangle {
            let side = size.width.min(size.height);

            Rectangle {
                x: (size.width - side) / 2.0,
                y: (size.height - side) / 2.0,
                width: side,
                height: side,
            }
        }

        fn cell_at(
            &self,
            region: Rectangle,
            position: Point,
        ) -> Option<(usize, usize)> {
            if region.contains(position) {
                let cell_size = region.width / SIZE as f32;

                let i = ((position.y - region.y) / cell_size).ceil() as usize;
                let j = ((position.x - region.x) / cell_size).ceil() as usize;

                Some((i.saturating_sub(1), j.saturating_sub(1)))
            } else {
                None
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Cell {
        Unpopulated,
        Populated,
    }

    impl Default for Cell {
        fn default() -> Cell {
            Cell::Unpopulated
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub enum Message {
        Populate { i: usize, j: usize },
    }

    impl<'a> canvas::Program<Message> for Grid {
        fn update(
            &mut self,
            event: Event,
            bounds: Rectangle,
            cursor: Cursor,
        ) -> Option<Message> {
            if let Event::Mouse(mouse::Event::Input {
                button: mouse::Button::Left,
                state,
            }) = event
            {
                self.mouse_pressed = state == ButtonState::Pressed;
            }

            let cursor_position = cursor.position_in(&bounds)?;

            let region = self.region(bounds.size());
            let (i, j) = self.cell_at(region, cursor_position)?;

            let populate = if self.cells[i][j] != Cell::Populated {
                Some(Message::Populate { i, j })
            } else {
                None
            };

            match event {
                Event::Mouse(mouse::Event::Input {
                    button: mouse::Button::Left,
                    ..
                }) if self.mouse_pressed => populate,
                Event::Mouse(mouse::Event::CursorMoved { .. })
                    if self.mouse_pressed =>
                {
                    populate
                }
                _ => None,
            }
        }

        fn draw(&self, bounds: Rectangle, cursor: Cursor) -> Vec<Geometry> {
            let region = self.region(bounds.size());
            let cell_size = Size::new(1.0, 1.0);

            let life = self.cache.draw(bounds.size(), |frame| {
                let background =
                    Path::rectangle(region.position(), region.size());
                frame.fill(
                    &background,
                    Color::from_rgb(
                        0x40 as f32 / 255.0,
                        0x44 as f32 / 255.0,
                        0x4B as f32 / 255.0,
                    ),
                );

                frame.with_save(|frame| {
                    frame.translate(Vector::new(region.x, region.y));
                    frame.scale(region.width / SIZE as f32);

                    let cells = Path::new(|p| {
                        for (i, row) in self.cells.iter().enumerate() {
                            for (j, cell) in row.iter().enumerate() {
                                if *cell == Cell::Populated {
                                    p.rectangle(
                                        Point::new(j as f32, i as f32),
                                        cell_size,
                                    );
                                }
                            }
                        }
                    });
                    frame.fill(&cells, Color::WHITE);
                });
            });

            let hovered_cell = {
                let mut frame = Frame::new(bounds.size());

                frame.translate(Vector::new(region.x, region.y));
                frame.scale(region.width / SIZE as f32);

                if let Some(cursor_position) = cursor.position_in(&bounds) {
                    if let Some((i, j)) = self.cell_at(region, cursor_position)
                    {
                        let interaction = Path::rectangle(
                            Point::new(j as f32, i as f32),
                            cell_size,
                        );

                        frame.fill(
                            &interaction,
                            Color {
                                a: 0.5,
                                ..Color::BLACK
                            },
                        );
                    }
                }

                frame.into_geometry()
            };

            vec![life, hovered_cell]
        }

        fn mouse_cursor(
            &self,
            bounds: Rectangle,
            cursor: Cursor,
        ) -> MouseCursor {
            let region = self.region(bounds.size());

            match cursor.position_in(&bounds) {
                Some(position) if region.contains(position) => {
                    MouseCursor::Crosshair
                }
                _ => MouseCursor::default(),
            }
        }
    }
}
