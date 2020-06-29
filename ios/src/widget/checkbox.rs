use crate::{Element, Widget, Length, Align};
use std::rc::Rc;

#[allow(missing_debug_implementations)]
pub struct Checkbox<Message> {
    is_checked: bool,
    on_toggle: Rc<dyn Fn(bool) -> Message>,
    label: String,
    width: Length,
    //style: Box<dyn StyleSheet>,
}

impl<Message> Checkbox<Message> {
    pub fn new<F>(is_checked: bool, label: impl Into<String>, f: F) -> Self
    where
        F: 'static + Fn(bool) -> Message,
    {
        Checkbox {
            is_checked,
            on_toggle: Rc::new(f),
            label: label.into(),
            width: Length::Shrink,
            //style: Default::default(),
        }
    }
}
/*
impl<Message> Widget<Message> for Checkbox<Message>
where
    Message: 'static,
{
}
impl<'a, Message> From<Checkbox<Message>> for Element<'a, Message>
where
    Message: 'static,
{
    fn from(checkbox: Checkbox<Message>) -> Element<'a, Message> {
        Element::new(checkbox)
    }
}
*/
