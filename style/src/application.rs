use iced_core::Color;

pub trait StyleSheet {
    fn background_color(&self) -> Color;

    fn text_color(&self) -> Color;
}
