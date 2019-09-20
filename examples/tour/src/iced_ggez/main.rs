use iced_tour::{iced_ggez, Tour};

use ggez;
use ggez::event;
use ggez::filesystem;
use ggez::graphics;
use ggez::input::mouse;

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
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")),
        true,
    );

    let state = &mut Game::new(context)?;

    event::run(context, event_loop, state)
}

struct Game {
    spritesheet: graphics::Image,
    font: graphics::Font,
    images: iced_ggez::ImageCache,
    tour: Tour,

    events: Vec<iced_native::Event>,
    cache: Option<iced_native::Cache>,
}

impl Game {
    fn new(context: &mut ggez::Context) -> ggez::GameResult<Game> {
        graphics::set_default_filter(context, graphics::FilterMode::Nearest);

        Ok(Game {
            spritesheet: graphics::Image::new(context, "/resources/ui.png")
                .unwrap(),
            font: graphics::Font::new(context, "/resources/Roboto-Regular.ttf")
                .unwrap(),
            images: iced_ggez::ImageCache::new(),
            tour: Tour::new(),

            events: Vec::new(),
            cache: Some(iced_native::Cache::default()),
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
        self.events.push(iced_native::Event::Mouse(
            iced_native::input::mouse::Event::Input {
                state: iced_native::input::ButtonState::Pressed,
                button: iced_native::input::mouse::Button::Left, // TODO: Map `button`
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
        self.events.push(iced_native::Event::Mouse(
            iced_native::input::mouse::Event::Input {
                state: iced_native::input::ButtonState::Released,
                button: iced_native::input::mouse::Button::Left, // TODO: Map `button`
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
        self.events.push(iced_native::Event::Mouse(
            iced_native::input::mouse::Event::CursorMoved { x, y },
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

            let content = iced_ggez::Column::new()
                .width(iced_native::Length::Units(screen.w as u16))
                .height(iced_native::Length::Units(screen.h as u16))
                .padding(20)
                .align_items(iced_native::Align::Center)
                .justify_content(iced_native::Justify::Center)
                .push(view);

            let renderer = &mut iced_ggez::Renderer::new(
                context,
                &mut self.images,
                self.spritesheet.clone(),
                self.font,
            );

            let mut ui = iced_native::UserInterface::build(
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

fn into_cursor_type(cursor: iced_native::MouseCursor) -> mouse::MouseCursor {
    match cursor {
        iced_native::MouseCursor::OutOfBounds => mouse::MouseCursor::Default,
        iced_native::MouseCursor::Idle => mouse::MouseCursor::Default,
        iced_native::MouseCursor::Pointer => mouse::MouseCursor::Hand,
        iced_native::MouseCursor::Working => mouse::MouseCursor::Progress,
        iced_native::MouseCursor::Grab => mouse::MouseCursor::Grab,
        iced_native::MouseCursor::Grabbing => mouse::MouseCursor::Grabbing,
    }
}
