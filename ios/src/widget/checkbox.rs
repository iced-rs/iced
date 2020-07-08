use crate::{Element, Widget, Length, Align, Hasher, layout, widget::{Text, Row}};
use std::hash::Hash;
use std::rc::Rc;
use iced_style::checkbox::StyleSheet;

#[allow(missing_debug_implementations)]
pub struct Checkbox<Message> {
    is_checked: bool,
    on_toggle: Rc<dyn Fn(bool) -> Message>,
    label: String,
    width: Length,
    style: Box<dyn StyleSheet>,
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
            style: Default::default(),
        }
    }
}
impl<Message> Widget<Message> for Checkbox<Message>
where
    Message: 'static,
{
    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.label.hash(state);
    }
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(
        &self,
        limits: &layout::Limits,
    ) -> layout::Node {
        /*
        Row::<()>::new()
            .width(self.width)
            .spacing(self.spacing)
            .align_items(Align::Center)
            .push(
                Row::new()
                    .width(Length::Units(self.size))
                    .height(Length::Units(self.size)),
            )
             * TODO: Add the text
            .push(
                Text::new(&self.label)
                    .width(self.width)
                    .size(self.text_size.unwrap_or(renderer.default_size())),
            )
            .layout(limits)
        */
        todo!();
    }

}
impl<'a, Message> From<Checkbox<Message>> for Element<'a, Message>
where
    Message: 'static,
{
    fn from(checkbox: Checkbox<Message>) -> Element<'a, Message> {
        Element::new(checkbox)
    }
}
