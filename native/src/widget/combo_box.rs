use crate::{
    layer::{self, menu},
    layout, mouse, scrollable, text, Clipboard, Element, Event, Hasher, Layout,
    Length, Overlay, Point, Rectangle, Size, Vector, Widget,
};
use std::borrow::Cow;

pub struct ComboBox<'a, T, Message>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    internal: Option<Internal<'a, T, Message>>,
    options: Cow<'a, [T]>,
    selected: Option<T>,
    width: Length,
    padding: u16,
    text_size: Option<u16>,
}

#[derive(Default)]
pub struct State {
    menu: menu::State,
}

pub struct Internal<'a, T, Message> {
    menu: &'a mut menu::State,
    on_selected: Box<dyn Fn(T) -> Message>,
}

impl<'a, T: 'a, Message> ComboBox<'a, T, Message>
where
    T: ToString,
    [T]: ToOwned<Owned = Vec<T>>,
{
    pub fn new(
        state: &'a mut State,
        options: impl Into<Cow<'a, [T]>>,
        selected: Option<T>,
        on_selected: impl Fn(T) -> Message + 'static,
    ) -> Self {
        Self {
            internal: Some(Internal {
                menu: &mut state.menu,
                on_selected: Box::new(on_selected),
            }),
            options: options.into(),
            selected,
            width: Length::Shrink,
            text_size: None,
            padding: 5,
        }
    }

    /// Sets the width of the [`ComboBox`].
    ///
    /// [`ComboBox`]: struct.Button.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the padding of the [`ComboBox`].
    ///
    /// [`ComboBox`]: struct.Button.html
    pub fn padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    pub fn text_size(mut self, size: u16) -> Self {
        self.text_size = Some(size);
        self
    }
}

impl<'a, T: 'a, Message, Renderer> Widget<'a, Message, Renderer>
    for ComboBox<'a, T, Message>
where
    T: Clone + ToString + Eq,
    [T]: ToOwned<Owned = Vec<T>>,
    Message: 'static,
    Renderer: self::Renderer + scrollable::Renderer + 'a,
{
    fn width(&self) -> Length {
        Length::Shrink
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        use std::f32;

        let limits = limits
            .width(self.width)
            .height(Length::Shrink)
            .pad(f32::from(self.padding));

        let text_size = self.text_size.unwrap_or(renderer.default_size());

        let max_width = match self.width {
            Length::Shrink => {
                let labels = self.options.iter().map(ToString::to_string);

                labels
                    .map(|label| {
                        let (width, _) = renderer.measure(
                            &label,
                            text_size,
                            Renderer::Font::default(),
                            Size::new(f32::INFINITY, f32::INFINITY),
                        );

                        width.round() as u32
                    })
                    .max()
                    .unwrap_or(100)
            }
            _ => 0,
        };

        let size = {
            let intrinsic = Size::new(
                max_width as f32 + f32::from(text_size),
                f32::from(text_size),
            );

            limits.resolve(intrinsic).pad(f32::from(self.padding))
        };

        layout::Node::new(size)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash as _;

        match self.width {
            Length::Shrink => {
                self.options
                    .iter()
                    .map(ToString::to_string)
                    .for_each(|label| label.hash(state));
            }
            _ => {
                self.width.hash(state);
            }
        }
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        _messages: &mut Vec<Message>,
        _renderer: &Renderer,
        _clipboard: Option<&dyn Clipboard>,
    ) {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(internal) = &mut self.internal {
                    if layout.bounds().contains(cursor_position) {
                        let selected = self.selected.as_ref();

                        internal.menu.open(
                            self.options
                                .iter()
                                .position(|option| Some(option) == selected),
                        );
                    }
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
            self.selected.as_ref().map(ToString::to_string),
            self.text_size.unwrap_or(renderer.default_size()),
            self.padding,
        )
    }

    fn overlay(
        &mut self,
        layout: Layout<'_>,
    ) -> Option<Overlay<'a, Message, Renderer>> {
        let is_open = self
            .internal
            .as_ref()
            .map(|internal| internal.menu.is_open())
            .unwrap_or(false);

        if is_open {
            if let Some(Internal { menu, on_selected }) = self.internal.take() {
                Some(Overlay::new(
                    layout.position()
                        + Vector::new(0.0, layout.bounds().height),
                    Box::new(layer::Menu::new(
                        menu,
                        self.options.clone(),
                        on_selected,
                        layout.bounds().width.round() as u16,
                        self.text_size.unwrap_or(20),
                        self.padding,
                    )),
                ))
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub trait Renderer: text::Renderer + menu::Renderer {
    const DEFAULT_PADDING: u16;

    fn draw(
        &mut self,
        bounds: Rectangle,
        cursor_position: Point,
        selected: Option<String>,
        text_size: u16,
        padding: u16,
    ) -> Self::Output;
}

impl<'a, T: 'a, Message, Renderer> Into<Element<'a, Message, Renderer>>
    for ComboBox<'a, T, Message>
where
    T: Clone + ToString + Eq,
    [T]: ToOwned<Owned = Vec<T>>,
    Renderer: self::Renderer + 'a,
    Message: 'static,
{
    fn into(self) -> Element<'a, Message, Renderer> {
        Element::new(self)
    }
}
