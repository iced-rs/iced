use crate::Element;

pub struct TitleBar<'a, Message, Renderer> {
    title: String,
    buttons: Option<Element<'a, Message, Renderer>>,
}
