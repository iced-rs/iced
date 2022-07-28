pub trait Focusable {
    fn is_focused(&self) -> bool;
    fn focus(&mut self);
    fn unfocus(&mut self);
}
