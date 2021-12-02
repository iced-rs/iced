//! Build and show dropdown menus.
use crate::alignment;
use crate::event::{self, Event};
use crate::layout;
use crate::mouse;
use crate::overlay;
use crate::renderer;
use crate::text::{self, Text};
use crate::touch;
use crate::widget::scrollable::{self, Scrollable};
use crate::widget::Container;
use crate::{
    Clipboard, Color, Element, Hasher, Layout, Length, Padding, Point,
    Rectangle, Shell, Size, Vector, Widget,
};

pub use iced_style::menu::Style;

/// A list of selectable options.
#[allow(missing_debug_implementations)]
pub struct Menu<'a, T, Renderer: text::Renderer> {
    state: &'a mut State,
    options: &'a [T],
    hovered_option: &'a mut Option<usize>,
    last_selection: &'a mut Option<T>,
    width: u16,
    padding: Padding,
    text_size: Option<u16>,
    font: Renderer::Font,
    style: Style,
}

impl<'a, T, Renderer> Menu<'a, T, Renderer>
where
    T: ToString + Clone,
    Renderer: text::Renderer + 'a,
{
    /// Creates a new [`Menu`] with the given [`State`], a list of options, and
    /// the message to produced when an option is selected.
    pub fn new(
        state: &'a mut State,
        options: &'a [T],
        hovered_option: &'a mut Option<usize>,
        last_selection: &'a mut Option<T>,
    ) -> Self {
        Menu {
            state,
            options,
            hovered_option,
            last_selection,
            width: 0,
            padding: Padding::ZERO,
            text_size: None,
            font: Default::default(),
            style: Default::default(),
        }
    }

    /// Sets the width of the [`Menu`].
    pub fn width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    /// Sets the [`Padding`] of the [`Menu`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the text size of the [`Menu`].
    pub fn text_size(mut self, text_size: u16) -> Self {
        self.text_size = Some(text_size);
        self
    }

    /// Sets the font of the [`Menu`].
    pub fn font(mut self, font: Renderer::Font) -> Self {
        self.font = font;
        self
    }

    /// Sets the style of the [`Menu`].
    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.style = style.into();
        self
    }

    /// Turns the [`Menu`] into an overlay [`Element`] at the given target
    /// position.
    ///
    /// The `target_height` will be used to display the menu either on top
    /// of the target or under it, depending on the screen position and the
    /// dimensions of the [`Menu`].
    pub fn overlay<Message: 'a>(
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
#[derive(Debug, Clone, Default)]
pub struct State {
    scrollable: scrollable::State,
}

impl State {
    /// Creates a new [`State`] for a [`Menu`].
    pub fn new() -> Self {
        Self::default()
    }
}

struct Overlay<'a, Message, Renderer: text::Renderer> {
    container: Container<'a, Message, Renderer>,
    width: u16,
    target_height: f32,
    style: Style,
}

impl<'a, Message, Renderer: text::Renderer> Overlay<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a,
{
    pub fn new<T>(menu: Menu<'a, T, Renderer>, target_height: f32) -> Self
    where
        T: Clone + ToString,
    {
        let Menu {
            state,
            options,
            hovered_option,
            last_selection,
            width,
            padding,
            font,
            text_size,
            style,
        } = menu;

        let container =
            Container::new(Scrollable::new(&mut state.scrollable).push(List {
                options,
                hovered_option,
                last_selection,
                font,
                text_size,
                padding,
                style: style.clone(),
            }))
            .padding(1);

        Self {
            container,
            width: width,
            target_height,
            style: style,
        }
    }
}

impl<'a, Message, Renderer> crate::Overlay<Message, Renderer>
    for Overlay<'a, Message, Renderer>
where
    Renderer: text::Renderer,
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
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        self.container.on_event(
            event.clone(),
            layout,
            cursor_position,
            renderer,
            clipboard,
            shell,
        )
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) -> mouse::Interaction {
        self.container
            .mouse_interaction(layout, cursor_position, viewport)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
    ) {
        let bounds = layout.bounds();

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border_color: self.style.border_color,
                border_width: self.style.border_width,
                border_radius: 0.0,
            },
            self.style.background,
        );

        self.container
            .draw(renderer, style, layout, cursor_position, &bounds);
    }
}

struct List<'a, T, Renderer: text::Renderer> {
    options: &'a [T],
    hovered_option: &'a mut Option<usize>,
    last_selection: &'a mut Option<T>,
    padding: Padding,
    text_size: Option<u16>,
    font: Renderer::Font,
    style: Style,
}

impl<'a, T, Message, Renderer> Widget<Message, Renderer>
    for List<'a, T, Renderer>
where
    T: Clone + ToString,
    Renderer: text::Renderer,
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
                f32::from(text_size + self.padding.vertical())
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
        renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        _shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let bounds = layout.bounds();

                if bounds.contains(cursor_position) {
                    if let Some(index) = *self.hovered_option {
                        if let Some(option) = self.options.get(index) {
                            *self.last_selection = Some(option.clone());
                        }
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let bounds = layout.bounds();

                if bounds.contains(cursor_position) {
                    let text_size =
                        self.text_size.unwrap_or(renderer.default_size());

                    *self.hovered_option = Some(
                        ((cursor_position.y - bounds.y)
                            / f32::from(text_size + self.padding.vertical()))
                            as usize,
                    );
                }
            }
            Event::Touch(touch::Event::FingerPressed { .. }) => {
                let bounds = layout.bounds();

                if bounds.contains(cursor_position) {
                    let text_size =
                        self.text_size.unwrap_or(renderer.default_size());

                    *self.hovered_option = Some(
                        ((cursor_position.y - bounds.y)
                            / f32::from(text_size + self.padding.vertical()))
                            as usize,
                    );

                    if let Some(index) = *self.hovered_option {
                        if let Some(option) = self.options.get(index) {
                            *self.last_selection = Some(option.clone());
                        }
                    }
                }
            }
            _ => {}
        }

        event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) -> mouse::Interaction {
        let is_mouse_over = layout.bounds().contains(cursor_position);

        if is_mouse_over {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor_position: Point,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        let text_size = self.text_size.unwrap_or(renderer.default_size());
        let option_height = (text_size + self.padding.vertical()) as usize;

        let offset = viewport.y - bounds.y;
        let start = (offset / option_height as f32) as usize;
        let end =
            ((offset + viewport.height) / option_height as f32).ceil() as usize;

        let visible_options = &self.options[start..end.min(self.options.len())];

        for (i, option) in visible_options.iter().enumerate() {
            let i = start + i;
            let is_selected = *self.hovered_option == Some(i);

            let bounds = Rectangle {
                x: bounds.x,
                y: bounds.y + (option_height * i) as f32,
                width: bounds.width,
                height: f32::from(text_size + self.padding.vertical()),
            };

            if is_selected {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds,
                        border_color: Color::TRANSPARENT,
                        border_width: 0.0,
                        border_radius: 0.0,
                    },
                    self.style.selected_background,
                );
            }

            renderer.fill_text(Text {
                content: &option.to_string(),
                bounds: Rectangle {
                    x: bounds.x + self.padding.left as f32,
                    y: bounds.center_y(),
                    width: f32::INFINITY,
                    ..bounds
                },
                size: f32::from(text_size),
                font: self.font,
                color: if is_selected {
                    self.style.selected_text_color
                } else {
                    self.style.text_color
                },
                horizontal_alignment: alignment::Horizontal::Left,
                vertical_alignment: alignment::Vertical::Center,
            });
        }
    }
}

impl<'a, T, Message, Renderer> Into<Element<'a, Message, Renderer>>
    for List<'a, T, Renderer>
where
    T: ToString + Clone,
    Message: 'a,
    Renderer: 'a + text::Renderer,
{
    fn into(self) -> Element<'a, Message, Renderer> {
        Element::new(self)
    }
}
