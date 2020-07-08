use std::hash::Hash;
use crate::{Element, Widget, Length, Align, Hasher, layout, Point};
pub use iced_style::text_input::{Style, StyleSheet};

#[allow(missing_debug_implementations)]
pub struct Container<'a, Message> {
    padding: u16,
    width: Length,
    height: Length,
    max_width: u32,
    max_height: u32,
    horizontal_alignment: Align,
    vertical_alignment: Align,
    style_sheet: Box<dyn StyleSheet>,
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
            style_sheet: Default::default(),
            content: content.into(),
        }
    }
}
impl<'a, Message> Widget<Message> for Container<'a, Message>
where
    Message: 'static,
{

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.padding.hash(state);
        self.width.hash(state);
        self.height.hash(state);
        self.max_width.hash(state);
        self.max_height.hash(state);

        self.content.hash_layout(state);
    }

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        limits: &layout::Limits,
    ) -> layout::Node {
        let padding = f32::from(self.padding);

        let limits = limits
            .loose()
            .max_width(self.max_width)
            .max_height(self.max_height)
            .width(self.width)
            .height(self.height)
            .pad(padding);

        let mut content = self.content.layout(&limits.loose());
        let size = limits.resolve(content.size());

        content.move_to(Point::new(padding, padding));
        content.align(self.horizontal_alignment, self.vertical_alignment, size);

        layout::Node::with_children(size.pad(padding), vec![content])
    }
}
impl<'a, Message> From<Container<'a, Message>> for Element<'a, Message>
where
    Message: 'static,
{
    fn from(container: Container<'a, Message>) -> Element<'a, Message> {
        Element::new(container)
    }
}
