mod renderer;
mod widget;

use renderer::Renderer;
use widget::Text;

use ggez;
use ggez::event;
use ggez::graphics;

use iced::Interface;

pub fn main() -> ggez::GameResult {
    let cb = ggez::ContextBuilder::new("iced", "ggez");
    let (ctx, event_loop) = &mut cb.build()?;
    let state = &mut Game::new()?;
    event::run(ctx, event_loop, state)
}

struct Game {}

impl Game {
    fn new() -> ggez::GameResult<Game> {
        Ok(Game {})
    }
}

impl event::EventHandler for Game {
    fn update(&mut self, _ctx: &mut ggez::Context) -> ggez::GameResult {
        Ok(())
    }

    fn draw(&mut self, context: &mut ggez::Context) -> ggez::GameResult {
        graphics::clear(context, [0.1, 0.2, 0.3, 1.0].into());

        {
            let renderer = &mut Renderer { context };
            let ui: Interface<(), Renderer> =
                Interface::compute(Text::new("Hello, iced!").into(), renderer);

            let mouse_cursor = ui.draw(renderer, iced::Point::new(0.0, 0.0));

            renderer.flush();
        }

        graphics::present(context)?;
        Ok(())
    }
}
