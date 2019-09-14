mod renderer;
mod tour;
mod widget;

use renderer::Renderer;
use tour::Tour;
use widget::Column;

use ggez;
use ggez::event;
use ggez::filesystem;
use ggez::graphics;
use ggez::input::mouse;

extern crate env_logger;

pub fn main() -> ggez::GameResult {
    let (context, event_loop) = {
        &mut ggez::ContextBuilder::new("iced", "ggez")
            .window_mode(ggez::conf::WindowMode {
                width: 1280.0,
                height: 1024.0,
                resizable: true,
                ..ggez::conf::WindowMode::default()
            })
            .build()?
    };

    filesystem::mount(
        context,
        std::path::Path::new(&format!(
            "{}/examples/resources",
            env!("CARGO_MANIFEST_DIR")
        )),
        true,
    );

    env_logger::init();
    let state = &mut Game::new(context)?;

    event::run(context, event_loop, state)
}

struct Game {
    spritesheet: graphics::Image,
    font: graphics::Font,
    tour: Tour,

    events: Vec<iced::Event>,
    cache: Option<iced::Cache>,
}

impl Game {
    fn new(context: &mut ggez::Context) -> ggez::GameResult<Game> {
        graphics::set_default_filter(context, graphics::FilterMode::Nearest);

        Ok(Game {
            spritesheet: graphics::Image::new(context, "/ui.png").unwrap(),
            font: graphics::Font::new(context, "/Roboto-Regular.ttf").unwrap(),
            tour: Tour::new(context),

            events: Vec::new(),
            cache: Some(iced::Cache::default()),
        })
    }
}

impl event::EventHandler for Game {
    fn update(&mut self, _ctx: &mut ggez::Context) -> ggez::GameResult {
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _context: &mut ggez::Context,
        _button: mouse::MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.events.push(iced::Event::Mouse(
            iced::input::mouse::Event::Input {
                state: iced::input::ButtonState::Pressed,
                button: iced::input::mouse::Button::Left, // TODO: Map `button`
            },
        ));
    }

    fn mouse_button_up_event(
        &mut self,
        _context: &mut ggez::Context,
        _button: mouse::MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.events.push(iced::Event::Mouse(
            iced::input::mouse::Event::Input {
                state: iced::input::ButtonState::Released,
                button: iced::input::mouse::Button::Left, // TODO: Map `button`
            },
        ));
    }

    fn mouse_motion_event(
        &mut self,
        _context: &mut ggez::Context,
        x: f32,
        y: f32,
        _dx: f32,
        _dy: f32,
    ) {
        self.events.push(iced::Event::Mouse(
            iced::input::mouse::Event::CursorMoved { x, y },
        ));
    }

    fn resize_event(
        &mut self,
        context: &mut ggez::Context,
        width: f32,
        height: f32,
    ) {
        graphics::set_screen_coordinates(
            context,
            graphics::Rect {
                x: 0.0,
                y: 0.0,
                w: width,
                h: height,
            },
        )
        .expect("Set screen coordinates");
    }

    fn draw(&mut self, context: &mut ggez::Context) -> ggez::GameResult {
        graphics::clear(context, graphics::WHITE);

        let screen = graphics::screen_coordinates(context);

        let (messages, cursor) = {
            let view = self.tour.view();

            let content = Column::new()
                .width(screen.w as u16)
                .height(screen.h as u16)
                .padding(20)
                .align_items(iced::Align::Center)
                .justify_content(iced::Justify::Center)
                .push(view);

            let renderer = &mut Renderer::new(
                context,
                self.spritesheet.clone(),
                self.font,
            );

            let mut ui = iced::UserInterface::build(
                content,
                self.cache.take().unwrap(),
                renderer,
            );

            let messages = ui.update(self.events.drain(..));
            let cursor = ui.draw(renderer);

            self.cache = Some(ui.into_cache());

            renderer.flush();

            (messages, cursor)
        };

        for message in messages {
            self.tour.update(message);
        }

        let cursor_type = into_cursor_type(cursor);

        if mouse::cursor_type(context) != cursor_type {
            mouse::set_cursor_type(context, cursor_type);
        }

        graphics::present(context)?;
        Ok(())
    }
}

fn into_cursor_type(cursor: iced::MouseCursor) -> mouse::MouseCursor {
    match cursor {
        iced::MouseCursor::OutOfBounds => mouse::MouseCursor::Default,
        iced::MouseCursor::Idle => mouse::MouseCursor::Default,
        iced::MouseCursor::Pointer => mouse::MouseCursor::Hand,
        iced::MouseCursor::Working => mouse::MouseCursor::Progress,
        iced::MouseCursor::Grab => mouse::MouseCursor::Grab,
        iced::MouseCursor::Grabbing => mouse::MouseCursor::Grabbing,
    }
}
