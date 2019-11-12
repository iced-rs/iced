/// The strategy used to fill space in a specific dimension.
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub enum Length {
    Fill,
    Shrink,
    Units(u16),
}

impl Length {
    pub fn fill_factor(&self) -> u16 {
        match self {
            Length::Fill => 1,
            Length::Shrink => 0,
            Length::Units(_) => 0,
        }
    }
}
