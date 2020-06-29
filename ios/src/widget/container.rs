use crate::{Element, Widget, Length, Align};

#[allow(missing_debug_implementations)]
pub struct Container<'a, Message> {
    padding: u16,
    width: Length,
    height: Length,
    max_width: u32,
    max_height: u32,
    horizontal_alignment: Align,
    vertical_alignment: Align,
    //style_sheet: Box<dyn StyleSheet>,
    content: Element<'a, Message>,
}

impl<'a, Message> Container<'a, Message> {
    /// Creates an empty [`Container`].
    ///
    /// [`Container`]: struct.Container.html
    pub fn new<T>(content: T) -> Self
    where
        T: Into<Element<'a, Message>>,
    {
        use std::u32;

        Container {
            padding: 0,
            width: Length::Shrink,
            height: Length::Shrink,
            max_width: u32::MAX,
            max_height: u32::MAX,
            horizontal_alignment: Align::Start,
            vertical_alignment: Align::Start,
            //style_sheet: Default::default(),
            content: content.into(),
        }
    }
}
/*
impl<'a, Message> Widget<Message> for Container<'a, Message>
where
    Message: 'static,
{
}
impl<'a, Message> From<Container<'a, Message>> for Element<'a, Message>
where
    Message: 'static,
{
    fn from(container: Container<'a, Message>) -> Element<'a, Message> {
        Element::new(container)
    }
}
*/
