//! Build and show dropdown menus.
use crate::{
    container, layout, mouse, overlay, scrollable, text, Clipboard, Container,
    Element, Event, Hasher, Layout, Length, Point, Rectangle, Scrollable, Size,
    Vector, Widget,
};

/// A list of selectable options.
#[allow(missing_debug_implementations)]
pub struct Menu<'a, T, Message, Renderer: self::Renderer> {
    state: &'a mut State,
    options: &'a [T],
    on_selected: &'a dyn Fn(T) -> Message,
    width: u16,
    padding: u16,
    text_size: Option<u16>,
    font: Renderer::Font,
    style: <Renderer as self::Renderer>::Style,
}

impl<'a, T, Message, Renderer> Menu<'a, T, Message, Renderer>
where
    T: ToString + Clone,
    Message: 'a,
    Renderer: self::Renderer + 'a,
{
    /// Creates a new [`Menu`] with the given [`State`], a list of options, and
    /// the message to produced when an option is selected.
    ///
    /// [`Menu`]: struct.Menu.html
    /// [`State`]: struct.State.html
    pub fn new(
        state: &'a mut State,
        options: &'a [T],
        on_selected: &'a dyn Fn(T) -> Message,
    ) -> Self {
        Menu {
            state,
            options,
            on_selected,
            width: 0,
            padding: 0,
            text_size: None,
            font: Default::default(),
            style: Default::default(),
        }
    }

    /// Sets the width of the [`Menu`].
    ///
    /// [`Menu`]: struct.Menu.html
    pub fn width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    /// Sets the padding of the [`Menu`].
    ///
    /// [`Menu`]: struct.Menu.html
    pub fn padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    /// Sets the text size of the [`Menu`].
    ///
    /// [`Menu`]: struct.Menu.html
    pub fn text_size(mut self, text_size: u16) -> Self {
        self.text_size = Some(text_size);
        self
    }

    /// Sets the font of the [`Menu`].
    ///
    /// [`Menu`]: struct.Menu.html
    pub fn font(mut self, font: Renderer::Font) -> Self {
        self.font = font;
        self
    }

    /// Sets the style of the [`Menu`].
    ///
    /// [`Menu`]: struct.Menu.html
    pub fn style(
        mut self,
        style: impl Into<<Renderer as self::Renderer>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }

    /// Turns the [`Menu`] into an overlay [`Element`] at the given target
    /// position.
    ///
    /// The `target_height` will be used to display the menu either on top
    /// of the target or under it, depending on the screen position and the
    /// dimensions of the [`Menu`].
    ///
    /// [`Menu`]: struct.Menu.html
    pub fn overlay(
        self,
        position: Point,
        target_height: f32,
    ) -> overlay::Element<'a, Message, Renderer> {
        overlay::Element::new(
            position,
            Box::new(Overlay::new(self, target_height)),
        )
    }
}

/// The local state of a [`Menu`].
///
/// [`Menu`]: struct.Menu.html
#[derive(Debug, Clone, Default)]
pub struct State {
    scrollable: scrollable::State,
    hovered_option: Option<usize>,
    is_open: bool,
}

impl State {
    /// Creates a new [`State`] for a [`Menu`].
    ///
    /// [`State`]: struct.State.html
    /// [`Menu`]: struct.Menu.html
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns whether the [`Menu`] is currently open or not.
    ///
    /// [`Menu`]: struct.Menu.html
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Opens the [`Menu`] with the given option hovered by default.
    ///
    /// [`Menu`]: struct.Menu.html
    pub fn open(&mut self, hovered_option: Option<usize>) {
        self.is_open = true;
        self.hovered_option = hovered_option;
    }
}

struct Overlay<'a, Message, Renderer: self::Renderer> {
    container: Container<'a, Message, Renderer>,
    is_open: &'a mut bool,
    width: u16,
    target_height: f32,
    style: <Renderer as self::Renderer>::Style,
}

impl<'a, Message, Renderer: self::Renderer> Overlay<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a,
{
    pub fn new<T>(
        menu: Menu<'a, T, Message, Renderer>,
        target_height: f32,
    ) -> Self
    where
        T: Clone + ToString,
    {
        let Menu {
            state,
            options,
            on_selected,
            width,
            padding,
            font,
            text_size,
            style,
        } = menu;

        let container =
            Container::new(Scrollable::new(&mut state.scrollable).push(List {
                options,
                hovered_option: &mut state.hovered_option,
                on_selected,
                font,
                text_size,
                padding,
                style: style.clone(),
            }))
            .padding(1);

        Self {
            container,
            is_open: &mut state.is_open,
            width: width,
            target_height,
            style: style,
        }
    }
}

impl<'a, Message, Renderer> crate::Overlay<Message, Renderer>
    for Overlay<'a, Message, Renderer>
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

        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        (position.x as u32).hash(state);
        (position.y as u32).hash(state);
        self.container.hash_layout(state);
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

        let option_was_selected = current_messages < messages.len();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if !bounds.contains(cursor_position) || option_was_selected =>
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

        renderer.decorate(
            layout.bounds(),
            cursor_position,
            &self.style,
            primitives,
        )
    }
}

struct List<'a, T, Message, Renderer: self::Renderer> {
    options: &'a [T],
    hovered_option: &'a mut Option<usize>,
    on_selected: &'a dyn Fn(T) -> Message,
    padding: u16,
    text_size: Option<u16>,
    font: Renderer::Font,
    style: <Renderer as self::Renderer>::Style,
}

impl<'a, T, Message, Renderer: self::Renderer> Widget<Message, Renderer>
    for List<'a, T, Message, Renderer>
where
    T: Clone + ToString,
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
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        use std::f32;

        let limits = limits.width(Length::Fill).height(Length::Shrink);
        let text_size = self.text_size.unwrap_or(renderer.default_size());

        let size = {
            let intrinsic = Size::new(
                0.0,
                f32::from(text_size + self.padding * 2)
                    * self.options.len() as f32,
            );

            limits.resolve(intrinsic)
        };

        layout::Node::new(size)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash as _;

        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.options.len().hash(state);
        self.text_size.hash(state);
        self.padding.hash(state);
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
        renderer: &Renderer,
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
                let text_size =
                    self.text_size.unwrap_or(renderer.default_size());

                if bounds.contains(cursor_position) {
                    *self.hovered_option = Some(
                        ((cursor_position.y - bounds.y)
                            / f32::from(text_size + self.padding * 2))
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
            self.options,
            *self.hovered_option,
            self.padding,
            self.text_size.unwrap_or(renderer.default_size()),
            self.font,
            &self.style,
        )
    }
}

/// The renderer of a [`Menu`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`Menu`] in your user interface.
///
/// [`Menu`]: struct.Menu.html
/// [renderer]: ../../renderer/index.html
pub trait Renderer:
    scrollable::Renderer + container::Renderer + text::Renderer
{
    /// The [`Menu`] style supported by this renderer.
    ///
    /// [`Menu`]: struct.Menu.html
    type Style: Default + Clone;

    /// Decorates a the list of options of a [`Menu`].
    ///
    /// This method can be used to draw a background for the [`Menu`].
    ///
    /// [`Menu`]: struct.Menu.html
    fn decorate(
        &mut self,
        bounds: Rectangle,
        cursor_position: Point,
        style: &<Self as Renderer>::Style,
        primitive: Self::Output,
    ) -> Self::Output;

    /// Draws the list of options of a [`Menu`].
    ///
    /// [`Menu`]: struct.Menu.html
    fn draw<T: ToString>(
        &mut self,
        bounds: Rectangle,
        cursor_position: Point,
        options: &[T],
        hovered_option: Option<usize>,
        padding: u16,
        text_size: u16,
        font: Self::Font,
        style: &<Self as Renderer>::Style,
    ) -> Self::Output;
}

impl<'a, T, Message, Renderer> Into<Element<'a, Message, Renderer>>
    for List<'a, T, Message, Renderer>
where
    T: ToString + Clone,
    Message: 'a,
    Renderer: 'a + self::Renderer,
{
    fn into(self) -> Element<'a, Message, Renderer> {
        Element::new(self)
    }
}
