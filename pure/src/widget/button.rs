use crate::widget::{Element, Tree, Widget};

use iced_native::event::{self, Event};
use iced_native::layout;
use iced_native::mouse;
use iced_native::renderer;
use iced_native::widget::button;
use iced_native::{
    Clipboard, Hasher, Layout, Length, Padding, Point, Rectangle, Shell,
};
use iced_style::button::StyleSheet;

use std::any::Any;

pub use button::State;

pub struct Button<'a, Message, Renderer> {
    content: Element<'a, Message, Renderer>,
    on_press: Option<Message>,
    style_sheet: Box<dyn StyleSheet + 'a>,
    width: Length,
    height: Length,
    padding: Padding,
}

impl<'a, Message, Renderer> Button<'a, Message, Renderer> {
    pub fn new(content: impl Into<Element<'a, Message, Renderer>>) -> Self {
        Button {
            content: content.into(),
            on_press: None,
            style_sheet: Default::default(),
            width: Length::Shrink,
            height: Length::Shrink,
            padding: Padding::new(5),
        }
    }

    pub fn on_press(mut self, on_press: Message) -> Self {
        self.on_press = Some(on_press);
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Button<'a, Message, Renderer>
where
    Message: 'static + Clone,
    Renderer: 'static + iced_native::Renderer,
{
    fn tag(&self) -> std::any::TypeId {
        std::any::TypeId::of::<State>()
    }

    fn state(&self) -> Box<dyn Any> {
        Box::new(State::new())
    }

    fn children(&self) -> &[Element<Message, Renderer>] {
        std::slice::from_ref(&self.content)
    }

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash;

        self.tag().hash(state);
        self.width.hash(state);
        self.height.hash(state);
        self.padding.hash(state);
        self.content.as_widget().hash_layout(state);
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        button::layout(
            renderer,
            limits,
            self.width,
            self.height,
            self.padding,
            |renderer, limits| {
                self.content.as_widget().layout(renderer, &limits)
            },
        )
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        if let event::Status::Captured = self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event.clone(),
            layout.children().next().unwrap(),
            cursor_position,
            renderer,
            clipboard,
            shell,
        ) {
            return event::Status::Captured;
        }

        button::update(
            event,
            layout,
            cursor_position,
            shell,
            &self.on_press,
            || tree.state_mut::<State>(),
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let content_layout = layout.children().next().unwrap();

        let styling = button::draw(
            renderer,
            bounds,
            cursor_position,
            self.on_press.is_some(),
            self.style_sheet.as_ref(),
            || tree.state::<State>(),
        );

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            &renderer::Style {
                text_color: styling.text_color,
            },
            content_layout,
            cursor_position,
            &bounds,
        );
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        button::mouse_interaction(
            layout,
            cursor_position,
            self.on_press.is_some(),
        )
    }
}

impl<'a, Message, Renderer> Into<Element<'a, Message, Renderer>>
    for Button<'a, Message, Renderer>
where
    Message: Clone + 'static,
    Renderer: iced_native::Renderer + 'static,
{
    fn into(self) -> Element<'a, Message, Renderer> {
        Element::new(self)
    }
}
