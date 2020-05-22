use crate::{
    container, layout, mouse, overlay, scrollable, Clipboard, Container,
    Element, Event, Hasher, Layout, Length, Point, Rectangle, Scrollable, Size,
    Vector, Widget,
};
use std::borrow::Cow;

pub struct Menu<'a, Message, Renderer: self::Renderer> {
    container: Container<'a, Message, Renderer>,
    is_open: &'a mut bool,
    width: u16,
    target_height: f32,
}

#[derive(Default)]
pub struct State {
    scrollable: scrollable::State,
    hovered_option: Option<usize>,
    is_open: bool,
}

impl State {
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn open(&mut self, hovered_option: Option<usize>) {
        self.is_open = true;
        self.hovered_option = hovered_option;
    }
}

impl<'a, Message, Renderer: self::Renderer> Menu<'a, Message, Renderer>
where
    Message: 'static,
    Renderer: 'a,
{
    pub fn new<T: 'a>(
        state: &'a mut State,
        options: impl Into<Cow<'a, [T]>>,
        on_selected: Box<dyn Fn(T) -> Message>,
        width: u16,
        target_height: f32,
        text_size: u16,
        padding: u16,
    ) -> Self
    where
        T: Clone + ToString,
        [T]: ToOwned<Owned = Vec<T>>,
    {
        let container = Container::new(
            Scrollable::new(&mut state.scrollable).push(List::new(
                &mut state.hovered_option,
                options,
                on_selected,
                text_size,
                padding,
            )),
        )
        .padding(1);

        Self {
            container,
            is_open: &mut state.is_open,
            width,
            target_height,
        }
    }
}

impl<'a, Message, Renderer> overlay::Content<Message, Renderer>
    for Menu<'a, Message, Renderer>
where
    Renderer: self::Renderer,
{
    fn layout(
        &self,
        renderer: &Renderer,
        bounds: Size,
        position: Point,
    ) -> layout::Node {
        let space_below = bounds.height - (position.y + self.target_height);
        let space_above = position.y;

        let limits = layout::Limits::new(
            Size::ZERO,
            Size::new(
                bounds.width - position.x,
                if space_below > space_above {
                    space_below
                } else {
                    space_above
                },
            ),
        )
        .width(Length::Units(self.width));

        let mut node = self.container.layout(renderer, &limits);

        node.move_to(if space_below > space_above {
            position + Vector::new(0.0, self.target_height)
        } else {
            position - Vector::new(0.0, node.size().height)
        });

        node
    }

    fn hash_layout(&self, state: &mut Hasher, position: Point) {
        use std::hash::Hash;

        (position.x as u32).hash(state);
        (position.y as u32).hash(state);
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
        renderer: &Renderer,
        clipboard: Option<&dyn Clipboard>,
    ) {
        let bounds = layout.bounds();
        let current_messages = messages.len();

        self.container.on_event(
            event.clone(),
            layout,
            cursor_position,
            messages,
            renderer,
            clipboard,
        );

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if !bounds.contains(cursor_position)
                    || current_messages < messages.len() =>
            {
                *self.is_open = false;
            }
            _ => {}
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Renderer::Output {
        let primitives =
            self.container
                .draw(renderer, defaults, layout, cursor_position);

        renderer.decorate(layout.bounds(), cursor_position, primitives)
    }
}

struct List<'a, T, Message>
where
    [T]: ToOwned,
{
    hovered_option: &'a mut Option<usize>,
    options: Cow<'a, [T]>,
    on_selected: Box<dyn Fn(T) -> Message>,
    text_size: u16,
    padding: u16,
}

impl<'a, T, Message> List<'a, T, Message>
where
    [T]: ToOwned,
{
    pub fn new(
        hovered_option: &'a mut Option<usize>,
        options: impl Into<Cow<'a, [T]>>,
        on_selected: Box<dyn Fn(T) -> Message>,
        text_size: u16,
        padding: u16,
    ) -> Self {
        List {
            hovered_option,
            options: options.into(),
            on_selected,
            text_size,
            padding,
        }
    }
}

impl<'a, T, Message, Renderer> Widget<'a, Message, Renderer>
    for List<'a, T, Message>
where
    T: ToString + Clone,
    [T]: ToOwned,
    Renderer: self::Renderer,
{
    fn width(&self) -> Length {
        Length::Fill
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(
        &self,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        use std::f32;

        let limits = limits.width(Length::Fill).height(Length::Shrink);

        let size = {
            let intrinsic = Size::new(
                0.0,
                f32::from(self.text_size + self.padding * 2)
                    * self.options.len() as f32,
            );

            limits.resolve(intrinsic)
        };

        layout::Node::new(size)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash as _;

        0.hash(state);
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
        _renderer: &Renderer,
        _clipboard: Option<&dyn Clipboard>,
    ) {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let bounds = layout.bounds();

                if bounds.contains(cursor_position) {
                    if let Some(index) = *self.hovered_option {
                        if let Some(option) = self.options.get(index) {
                            messages.push((self.on_selected)(option.clone()));
                        }
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let bounds = layout.bounds();

                if bounds.contains(cursor_position) {
                    *self.hovered_option = Some(
                        ((cursor_position.y - bounds.y)
                            / f32::from(self.text_size + self.padding * 2))
                            as usize,
                    );
                }
            }
            _ => {}
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Renderer::Output {
        self::Renderer::draw(
            renderer,
            layout.bounds(),
            cursor_position,
            &self.options,
            *self.hovered_option,
            self.text_size,
            self.padding,
        )
    }
}

pub trait Renderer: scrollable::Renderer + container::Renderer {
    fn decorate(
        &mut self,
        bounds: Rectangle,
        cursor_position: Point,
        primitive: Self::Output,
    ) -> Self::Output;

    fn draw<T: ToString>(
        &mut self,
        bounds: Rectangle,
        cursor_position: Point,
        options: &[T],
        hovered_option: Option<usize>,
        text_size: u16,
        padding: u16,
    ) -> Self::Output;
}

impl<'a, T, Message, Renderer> Into<Element<'a, Message, Renderer>>
    for List<'a, T, Message>
where
    T: ToString + Clone,
    [T]: ToOwned,
    Message: 'static,
    Renderer: self::Renderer,
{
    fn into(self) -> Element<'a, Message, Renderer> {
        Element::new(self)
    }
}
