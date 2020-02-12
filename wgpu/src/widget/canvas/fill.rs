use iced_native::Color;

#[derive(Debug, Clone, Copy)]
pub enum Fill {
    Color(Color),
}

impl Default for Fill {
    fn default() -> Fill {
        Fill::Color(Color::BLACK)
    }
}
