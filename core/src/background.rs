use crate::Color;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Background {
    Color(Color),
    // TODO: Add gradient and image variants
}
