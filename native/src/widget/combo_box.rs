use crate::{
    layout, mouse,
    overlay::menu::{self, Menu},
    scrollable, text, Clipboard, Element, Event, Hasher, Layout, Length,
    Overlay, Point, Rectangle, Size, Widget,
};
use std::borrow::Cow;

pub struct ComboBox<'a, T, Message, Renderer: self::Renderer>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    internal: Internal<'a, T, Message>,
    options: Cow<'a, [T]>,
    selected: Option<T>,
    width: Length,
    padding: u16,
    text_size: Option<u16>,
    style: <Renderer as self::Renderer>::Style,
}

#[derive(Default)]
pub struct State {
    menu: menu::State,
}

pub struct Internal<'a, T, Message> {
    menu: &'a mut menu::State,
    on_selected: Box<dyn Fn(T) -> Message>,
}

impl<'a, T: 'a, Message, Renderer: self::Renderer>
    ComboBox<'a, T, Message, Renderer>
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
            internal: Internal {
                menu: &mut state.menu,
                on_selected: Box::new(on_selected),
            },
            options: options.into(),
            selected,
            width: Length::Shrink,
            text_size: None,
            padding: Renderer::DEFAULT_PADDING,
            style: <Renderer as self::Renderer>::Style::default(),
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

    /// Sets the style of the [`ComboBox`].
    ///
    /// [`ComboBox`]: struct.ComboBox.html
    pub fn style(
        mut self,
        style: impl Into<<Renderer as self::Renderer>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }
}

impl<'a, T: 'a, Message, Renderer> Widget<'a, Message, Renderer>
    for ComboBox<'a, T, Message, Renderer>
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
                max_width as f32
                    + f32::from(text_size)
                    + f32::from(self.padding),
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
                if layout.bounds().contains(cursor_position) {
                    let selected = self.selected.as_ref();

                    self.internal.menu.open(
                        self.options
                            .iter()
                            .position(|option| Some(option) == selected),
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
            self.selected.as_ref().map(ToString::to_string),
            self.text_size.unwrap_or(renderer.default_size()),
            self.padding,
            &self.style,
        )
    }

    fn overlay(
        &mut self,
        layout: Layout<'_>,
    ) -> Option<Overlay<'_, Message, Renderer>> {
        if self.internal.menu.is_open() {
            let bounds = layout.bounds();

            Some(Overlay::new(
                layout.position(),
                Box::new(Menu::new(
                    self.internal.menu,
                    self.options.clone(),
                    &self.internal.on_selected,
                    bounds.width.round() as u16,
                    bounds.height,
                    self.text_size,
                    self.padding,
                    Renderer::menu_style(&self.style),
                )),
            ))
        } else {
            None
        }
    }
}

pub trait Renderer: text::Renderer + menu::Renderer {
    type Style: Default;

    const DEFAULT_PADDING: u16;

    fn menu_style(
        style: &<Self as Renderer>::Style,
    ) -> <Self as menu::Renderer>::Style;

    fn draw(
        &mut self,
        bounds: Rectangle,
        cursor_position: Point,
        selected: Option<String>,
        text_size: u16,
        padding: u16,
        style: &<Self as Renderer>::Style,
    ) -> Self::Output;
}

impl<'a, T: 'a, Message, Renderer> Into<Element<'a, Message, Renderer>>
    for ComboBox<'a, T, Message, Renderer>
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
