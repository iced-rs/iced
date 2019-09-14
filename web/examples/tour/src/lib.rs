use futures::{future, Future};
use iced_web::UserInterface;
use wasm_bindgen::prelude::*;

mod tour;

use tour::Tour;

#[wasm_bindgen(start)]
pub fn run() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Trace)
        .expect("Initialize logging");

    let tour = Tour::new();

    tour.run();
}

impl iced_web::UserInterface for Tour {
    type Message = tour::Message;

    fn update(
        &mut self,
        message: tour::Message,
    ) -> Box<dyn Future<Item = tour::Message, Error = ()>> {
        self.update(message);

        Box::new(future::err(()))
    }

    fn view(&mut self) -> iced_web::Element<tour::Message> {
        self.view()
    }
}
