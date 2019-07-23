mod renderer;
mod tour;
mod widget;

use renderer::Renderer;
use tour::Tour;
use widget::Column;

use ggez;
use ggez::event;
use ggez::graphics;
use ggez::input::mouse;

pub fn main() -> ggez::GameResult {
    let cb = ggez::ContextBuilder::new("iced", "ggez").window_mode(
        ggez::conf::WindowMode {
            width: 1280.0,
            height: 1024.0,
            ..ggez::conf::WindowMode::default()
        },
    );
    let (ctx, event_loop) = &mut cb.build()?;
    let state = &mut Game::new(ctx)?;
    event::run(ctx, event_loop, state)
}

struct Game {
    spritesheet: graphics::Image,

    runtime: iced::Runtime,
    tour: Tour,
}

impl Game {
    fn new(context: &mut ggez::Context) -> ggez::GameResult<Game> {
        graphics::set_default_filter(context, graphics::FilterMode::Nearest);

        Ok(Game {
            spritesheet: graphics::Image::new(context, "/ui.png").unwrap(),

            runtime: iced::Runtime::new(),
            tour: Tour::new(),
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
        button: mouse::MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.runtime.on_event(iced::Event::Mouse(
            iced::input::mouse::Event::Input {
                state: iced::input::ButtonState::Pressed,
                button: iced::input::mouse::Button::Left, // TODO: Map `button`
            },
        ));
    }

    fn mouse_button_up_event(
        &mut self,
        _context: &mut ggez::Context,
        button: mouse::MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.runtime.on_event(iced::Event::Mouse(
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
        self.runtime.on_event(iced::Event::Mouse(
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
        graphics::clear(context, [0.3, 0.3, 0.6, 1.0].into());

        self.tour.draw(context).expect("Draw tour");

        let screen = graphics::screen_coordinates(context);

        let (messages, cursor) = {
            let layout = self.tour.layout();

            let content = Column::new()
                .width(screen.w as u32)
                .height(screen.h as u32)
                .align_items(iced::Align::Center)
                .justify_content(iced::Justify::Center)
                .push(layout);

            let renderer =
                &mut Renderer::new(context, self.spritesheet.clone());

            let mut ui = self.runtime.compute(content.into(), renderer);

            let messages = ui.update();
            let cursor = ui.draw(renderer);

            renderer.flush();

            (messages, cursor)
        };

        for message in messages {
            self.tour.react(message);
        }

        mouse::set_cursor_type(context, into_cursor_type(cursor));

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
