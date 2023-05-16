//! Build and show dropdown menus.
use crate::container::{self, Container};
use crate::core::alignment;
use crate::core::event::{self, Event};
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::text::{self, Text};
use crate::core::touch;
use crate::core::widget::Tree;
use crate::core::{
    Clipboard, Color, Length, Padding, Pixels, Point, Rectangle, Size, Vector,
};
use crate::core::{Element, Shell, Widget};
use crate::scrollable::{self, Scrollable};

pub use iced_style::menu::{Appearance, StyleSheet};

/// A list of selectable options.
#[allow(missing_debug_implementations)]
pub struct Menu<'a, T, Renderer = crate::Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    state: &'a mut State,
    options: &'a [T],
    hovered_option: &'a mut Option<usize>,
    last_selection: &'a mut Option<T>,
    width: f32,
    padding: Padding,
    text_size: Option<f32>,
    text_line_height: text::LineHeight,
    text_shaping: text::Shaping,
    font: Option<Renderer::Font>,
    style: <Renderer::Theme as StyleSheet>::Style,
}

impl<'a, T, Renderer> Menu<'a, T, Renderer>
where
    T: ToString + Clone,
    Renderer: text::Renderer + 'a,
    Renderer::Theme:
        StyleSheet + container::StyleSheet + scrollable::StyleSheet,
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
            width: 0.0,
            padding: Padding::ZERO,
            text_size: None,
            text_line_height: text::LineHeight::default(),
            text_shaping: text::Shaping::Basic,
            font: None,
            style: Default::default(),
        }
    }

    /// Sets the width of the [`Menu`].
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Sets the [`Padding`] of the [`Menu`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the text size of the [`Menu`].
    pub fn text_size(mut self, text_size: impl Into<Pixels>) -> Self {
        self.text_size = Some(text_size.into().0);
        self
    }

    /// Sets the text [`LineHeight`] of the [`Menu`].
    pub fn text_line_height(
        mut self,
        line_height: impl Into<text::LineHeight>,
    ) -> Self {
        self.text_line_height = line_height.into();
        self
    }

    /// Sets the [`text::Shaping`] strategy of the [`Menu`].
    pub fn text_shaping(mut self, shaping: text::Shaping) -> Self {
        self.text_shaping = shaping;
        self
    }

    /// Sets the font of the [`Menu`].
    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    /// Sets the style of the [`Menu`].
    pub fn style(
        mut self,
        style: impl Into<<Renderer::Theme as StyleSheet>::Style>,
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
#[derive(Debug)]
pub struct State {
    tree: Tree,
}

impl State {
    /// Creates a new [`State`] for a [`Menu`].
    pub fn new() -> Self {
        Self {
            tree: Tree::empty(),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

struct Overlay<'a, Message, Renderer>
where
    Renderer: crate::core::Renderer,
    Renderer::Theme: StyleSheet + container::StyleSheet,
{
    state: &'a mut Tree,
    container: Container<'a, Message, Renderer>,
    width: f32,
    target_height: f32,
    style: <Renderer::Theme as StyleSheet>::Style,
}

impl<'a, Message, Renderer> Overlay<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a,
    Renderer: text::Renderer,
    Renderer::Theme:
        StyleSheet + container::StyleSheet + scrollable::StyleSheet,
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
            text_line_height,
            text_shaping,
            style,
        } = menu;

        let mut container = Container::new(Scrollable::new(List {
            options,
            hovered_option,
            last_selection,
            font,
            text_size,
            text_line_height,
            text_shaping,
            padding,
            style: style.clone(),
        }));

        state.tree.diff(&mut container as &mut dyn Widget<_, _>);

        Self {
            state: &mut state.tree,
            container,
            width,
            target_height,
            style,
        }
    }
}

impl<'a, Message, Renderer> crate::core::Overlay<Message, Renderer>
    for Overlay<'a, Message, Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet + container::StyleSheet,
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
        .width(self.width);

        let mut node = self.container.layout(renderer, &limits);

        node.move_to(if space_below > space_above {
            position + Vector::new(0.0, self.target_height)
        } else {
            position - Vector::new(0.0, node.size().height)
        });

        node
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
            self.state,
            event,
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
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.container.mouse_interaction(
            self.state,
            layout,
            cursor_position,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
    ) {
        let appearance = theme.appearance(&self.style);
        let bounds = layout.bounds();

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border_color: appearance.border_color,
                border_width: appearance.border_width,
                border_radius: appearance.border_radius.into(),
            },
            appearance.background,
        );

        self.container.draw(
            self.state,
            renderer,
            theme,
            style,
            layout,
            cursor_position,
            &bounds,
        );
    }
}

struct List<'a, T, Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    options: &'a [T],
    hovered_option: &'a mut Option<usize>,
    last_selection: &'a mut Option<T>,
    padding: Padding,
    text_size: Option<f32>,
    text_line_height: text::LineHeight,
    text_shaping: text::Shaping,
    font: Option<Renderer::Font>,
    style: <Renderer::Theme as StyleSheet>::Style,
}

impl<'a, T, Message, Renderer> Widget<Message, Renderer>
    for List<'a, T, Renderer>
where
    T: Clone + ToString,
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
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
        let text_size =
            self.text_size.unwrap_or_else(|| renderer.default_size());

        let text_line_height =
            self.text_line_height.to_absolute(Pixels(text_size));

        let size = {
            let intrinsic = Size::new(
                0.0,
                (f32::from(text_line_height) + self.padding.vertical())
                    * self.options.len() as f32,
            );

            limits.resolve(intrinsic)
        };

        layout::Node::new(size)
    }

    fn on_event(
        &mut self,
        _state: &mut Tree,
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
                    let text_size = self
                        .text_size
                        .unwrap_or_else(|| renderer.default_size());

                    let option_height = f32::from(
                        self.text_line_height.to_absolute(Pixels(text_size)),
                    ) + self.padding.vertical();

                    *self.hovered_option = Some(
                        ((cursor_position.y - bounds.y) / option_height)
                            as usize,
                    );
                }
            }
            Event::Touch(touch::Event::FingerPressed { .. }) => {
                let bounds = layout.bounds();

                if bounds.contains(cursor_position) {
                    let text_size = self
                        .text_size
                        .unwrap_or_else(|| renderer.default_size());

                    let option_height = f32::from(
                        self.text_line_height.to_absolute(Pixels(text_size)),
                    ) + self.padding.vertical();

                    *self.hovered_option = Some(
                        ((cursor_position.y - bounds.y) / option_height)
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
        _state: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
        _renderer: &Renderer,
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
        _state: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor_position: Point,
        viewport: &Rectangle,
    ) {
        let appearance = theme.appearance(&self.style);
        let bounds = layout.bounds();

        let text_size =
            self.text_size.unwrap_or_else(|| renderer.default_size());
        let option_height =
            f32::from(self.text_line_height.to_absolute(Pixels(text_size)))
                + self.padding.vertical();

        let offset = viewport.y - bounds.y;
        let start = (offset / option_height) as usize;
        let end = ((offset + viewport.height) / option_height).ceil() as usize;

        let visible_options = &self.options[start..end.min(self.options.len())];

        for (i, option) in visible_options.iter().enumerate() {
            let i = start + i;
            let is_selected = *self.hovered_option == Some(i);

            let bounds = Rectangle {
                x: bounds.x,
                y: bounds.y + (option_height * i as f32),
                width: bounds.width,
                height: option_height,
            };

            if is_selected {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x + appearance.border_width,
                            width: bounds.width - appearance.border_width * 2.0,
                            ..bounds
                        },
                        border_color: Color::TRANSPARENT,
                        border_width: 0.0,
                        border_radius: appearance.border_radius.into(),
                    },
                    appearance.selected_background,
                );
            }

            renderer.fill_text(Text {
                content: &option.to_string(),
                bounds: Rectangle {
                    x: bounds.x + self.padding.left,
                    y: bounds.center_y(),
                    width: f32::INFINITY,
                    ..bounds
                },
                size: text_size,
                line_height: self.text_line_height,
                font: self.font.unwrap_or_else(|| renderer.default_font()),
                color: if is_selected {
                    appearance.selected_text_color
                } else {
                    appearance.text_color
                },
                horizontal_alignment: alignment::Horizontal::Left,
                vertical_alignment: alignment::Vertical::Center,
                shaping: self.text_shaping,
            });
        }
    }
}

impl<'a, T, Message, Renderer> From<List<'a, T, Renderer>>
    for Element<'a, Message, Renderer>
where
    T: ToString + Clone,
    Message: 'a,
    Renderer: 'a + text::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn from(list: List<'a, T, Renderer>) -> Self {
        Element::new(list)
    }
}
