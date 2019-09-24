use futures::Future;
use iced_web::UserInterface;
use wasm_bindgen::prelude::*;

use crate::tour::{self, Tour};

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
    ) -> Option<Box<dyn Future<Output = tour::Message>>> {
        self.update(message);

        None
    }

    fn view(&mut self) -> iced_web::Element<tour::Message> {
        self.view()
    }
}
