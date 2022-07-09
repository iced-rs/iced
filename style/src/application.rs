use iced_core::Color;

pub trait StyleSheet {
    type Style: Default + Copy;

    fn appearance(&self, style: Self::Style) -> Appearance;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Appearance {
    pub background_color: Color,
    pub text_color: Color,
}
