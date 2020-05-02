//! This example showcases an interactive version of the Game of Life, invented
//! by John Conway. It leverages a `Canvas` together with other widgets.
mod style;

use grid::Grid;
use iced::{
    button::{self, Button},
    executor,
    slider::{self, Slider},
    time, Align, Application, Column, Command, Container, Element, Length, Row,
    Settings, Subscription, Text,
};
use std::time::{Duration, Instant};

pub fn main() {
    GameOfLife::run(Settings::default())
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
                    1.0..=100.0,
                    selected_speed as f32,
                    Message::SpeedChanged,
                )
                .width(Length::Units(200))
                .style(style::Slider),
            )
            .push(Text::new(format!("x{}", selected_speed)).size(16))
            .align_items(Align::Center);

        let controls = Row::new()
            .padding(10)
            .spacing(20)
            .push(playback_controls)
            .push(speed_controls);

        let content = Column::new()
            .spacing(10)
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
        canvas::{self, Cache, Canvas, Cursor, Event, Frame, Geometry, Path},
        mouse, Color, Element, Length, Point, Rectangle, Size, Vector,
    };
    use rustc_hash::{FxHashMap, FxHashSet};

    pub struct Grid {
        life: Life,
        interaction: Interaction,
        cache: Cache,
        translation: Vector,
        scaling: f32,
    }

    #[derive(Debug, Clone, Copy)]
    pub enum Message {
        Populate(Cell),
    }

    impl Default for Grid {
        fn default() -> Self {
            Self {
                life: Life::default(),
                interaction: Interaction::None,
                cache: Cache::default(),
                translation: Vector::default(),
                scaling: 1.0,
            }
        }
    }

    impl Grid {
        const MIN_SCALING: f32 = 0.1;
        const MAX_SCALING: f32 = 2.0;

        pub fn tick(&mut self) {
            self.life.tick();
            self.cache.clear()
        }

        pub fn update(&mut self, message: Message) {
            match message {
                Message::Populate(cell) => {
                    self.life.populate(cell);
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

        pub fn visible_region(&self, size: Size) -> Rectangle {
            let width = size.width / self.scaling;
            let height = size.height / self.scaling;

            Rectangle {
                x: -self.translation.x - width / 2.0,
                y: -self.translation.y - height / 2.0,
                width,
                height,
            }
        }

        pub fn project(&self, position: Point, size: Size) -> Point {
            let region = self.visible_region(size);

            Point::new(
                position.x / self.scaling + region.x,
                position.y / self.scaling + region.y,
            )
        }
    }

    impl<'a> canvas::Program<Message> for Grid {
        fn update(
            &mut self,
            event: Event,
            bounds: Rectangle,
            cursor: Cursor,
        ) -> Option<Message> {
            if let Event::Mouse(mouse::Event::ButtonReleased(_)) = event {
                self.interaction = Interaction::None;
            }

            let cursor_position = cursor.position_in(&bounds)?;
            let cell = Cell::at(self.project(cursor_position, bounds.size()));

            let populate = if self.life.contains(&cell) {
                None
            } else {
                Some(Message::Populate(cell))
            };

            match event {
                Event::Mouse(mouse_event) => match mouse_event {
                    mouse::Event::ButtonPressed(button) => match button {
                        mouse::Button::Left => {
                            self.interaction = Interaction::Drawing;

                            populate
                        }
                        mouse::Button::Right => {
                            self.interaction = Interaction::Panning {
                                translation: self.translation,
                                start: cursor_position,
                            };

                            None
                        }
                        _ => None,
                    },
                    mouse::Event::CursorMoved { .. } => {
                        match self.interaction {
                            Interaction::Drawing => populate,
                            Interaction::Panning { translation, start } => {
                                self.translation = translation
                                    + (cursor_position - start)
                                        * (1.0 / self.scaling);

                                self.cache.clear();

                                None
                            }
                            _ => None,
                        }
                    }
                    mouse::Event::WheelScrolled { delta } => match delta {
                        mouse::ScrollDelta::Lines { y, .. }
                        | mouse::ScrollDelta::Pixels { y, .. } => {
                            if y < 0.0 && self.scaling > Self::MIN_SCALING
                                || y > 0.0 && self.scaling < Self::MAX_SCALING
                            {
                                let old_scaling = self.scaling;

                                self.scaling = (self.scaling
                                    * (1.0 + y / 30.0))
                                    .max(Self::MIN_SCALING)
                                    .min(Self::MAX_SCALING);

                                if let Some(cursor_to_center) =
                                    cursor.position_from(bounds.center())
                                {
                                    let factor = self.scaling - old_scaling;

                                    self.translation = self.translation
                                        - Vector::new(
                                            cursor_to_center.x * factor
                                                / (old_scaling * old_scaling),
                                            cursor_to_center.y * factor
                                                / (old_scaling * old_scaling),
                                        );
                                }

                                self.cache.clear();
                            }

                            None
                        }
                    },
                    _ => None,
                },
            }
        }

        fn draw(&self, bounds: Rectangle, cursor: Cursor) -> Vec<Geometry> {
            let center = Vector::new(bounds.width / 2.0, bounds.height / 2.0);

            let life = self.cache.draw(bounds.size(), |frame| {
                let background = Path::rectangle(Point::ORIGIN, frame.size());
                frame.fill(&background, Color::from_rgb8(0x40, 0x44, 0x4B));

                frame.with_save(|frame| {
                    frame.translate(center);
                    frame.scale(self.scaling);
                    frame.translate(self.translation);
                    frame.scale(Cell::SIZE as f32);

                    let region = self.visible_region(frame.size());

                    for cell in self.life.within(region) {
                        frame.fill_rectangle(
                            Point::new(cell.j as f32, cell.i as f32),
                            Size::UNIT,
                            Color::WHITE,
                        );
                    }
                });
            });

            let hovered_cell = {
                let mut frame = Frame::new(bounds.size());

                frame.translate(center);
                frame.scale(self.scaling);
                frame.translate(self.translation);
                frame.scale(Cell::SIZE as f32);

                if let Some(cursor_position) = cursor.position_in(&bounds) {
                    let cell =
                        Cell::at(self.project(cursor_position, frame.size()));

                    frame.fill_rectangle(
                        Point::new(cell.j as f32, cell.i as f32),
                        Size::UNIT,
                        Color {
                            a: 0.5,
                            ..Color::BLACK
                        },
                    );
                }

                frame.into_geometry()
            };

            vec![life, hovered_cell]
        }

        fn mouse_interaction(
            &self,
            bounds: Rectangle,
            cursor: Cursor,
        ) -> mouse::Interaction {
            match self.interaction {
                Interaction::Drawing => mouse::Interaction::Crosshair,
                Interaction::Panning { .. } => mouse::Interaction::Grabbing,
                Interaction::None if cursor.is_over(&bounds) => {
                    mouse::Interaction::Crosshair
                }
                _ => mouse::Interaction::default(),
            }
        }
    }

    #[derive(Default)]
    pub struct Life {
        cells: FxHashSet<Cell>,
    }

    impl Life {
        fn tick(&mut self) {
            let mut adjacent_life = FxHashMap::default();

            for cell in &self.cells {
                let _ = adjacent_life.entry(*cell).or_insert(0);

                for neighbor in Cell::neighbors(*cell) {
                    let amount = adjacent_life.entry(neighbor).or_insert(0);

                    *amount += 1;
                }
            }

            for (cell, amount) in adjacent_life.iter() {
                match amount {
                    2 => {}
                    3 => {
                        let _ = self.cells.insert(*cell);
                    }
                    _ => {
                        let _ = self.cells.remove(cell);
                    }
                }
            }
        }

        fn contains(&self, cell: &Cell) -> bool {
            self.cells.contains(cell)
        }

        fn populate(&mut self, cell: Cell) {
            self.cells.insert(cell);
        }

        fn within(&self, region: Rectangle) -> impl Iterator<Item = &Cell> {
            let first_row = (region.y / Cell::SIZE as f32).floor() as isize;
            let first_column = (region.x / Cell::SIZE as f32).floor() as isize;

            let visible_rows =
                (region.height / Cell::SIZE as f32).ceil() as isize;
            let visible_columns =
                (region.width / Cell::SIZE as f32).ceil() as isize;

            let rows = first_row..=first_row + visible_rows;
            let columns = first_column..=first_column + visible_columns;

            self.cells.iter().filter(move |cell| {
                rows.contains(&cell.i) && columns.contains(&cell.j)
            })
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Cell {
        i: isize,
        j: isize,
    }

    impl Cell {
        const SIZE: usize = 20;

        fn at(position: Point) -> Cell {
            let i = (position.y / Cell::SIZE as f32).ceil() as isize;
            let j = (position.x / Cell::SIZE as f32).ceil() as isize;

            Cell {
                i: i.saturating_sub(1),
                j: j.saturating_sub(1),
            }
        }

        fn cluster(cell: Cell) -> impl Iterator<Item = Cell> {
            use itertools::Itertools;

            let rows = cell.i.saturating_sub(1)..=cell.i.saturating_add(1);
            let columns = cell.j.saturating_sub(1)..=cell.j.saturating_add(1);

            rows.cartesian_product(columns).map(|(i, j)| Cell { i, j })
        }

        fn neighbors(cell: Cell) -> impl Iterator<Item = Cell> {
            Cell::cluster(cell).filter(move |candidate| *candidate != cell)
        }
    }

    enum Interaction {
        None,
        Drawing,
        Panning { translation: Vector, start: Point },
    }
}
