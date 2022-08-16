use iced_core::Color;

pub trait StyleSheet {
    type Style: Default + Copy;

    fn appearance(&self, style: Self::Style) -> Appearance;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Appearance {
    pub color: Option<Color>,
}
