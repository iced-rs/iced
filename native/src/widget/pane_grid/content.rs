use crate::pane_grid::Axis;

#[derive(Debug, Clone)]
pub enum Content<T> {
    Split {
        axis: Axis,
        ratio: f32,
        a: Box<Content<T>>,
        b: Box<Content<T>>,
    },
    Pane(T),
}
