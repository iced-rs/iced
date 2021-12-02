//! Display a dropdown list of selectable values.
use crate::alignment;
use crate::event::{self, Event};
use crate::keyboard;
use crate::layout;
use crate::mouse;
use crate::overlay;
use crate::overlay::menu::{self, Menu};
use crate::renderer;
use crate::text::{self, Text};
use crate::touch;
use crate::{
    Clipboard, Element, Hasher, Layout, Length, Padding, Point, Rectangle,
    Shell, Size, Widget,
};
use std::borrow::Cow;

pub use iced_style::pick_list::{Style, StyleSheet};

/// A widget for selecting a single value from a list of options.
#[allow(missing_debug_implementations)]
pub struct PickList<'a, T, Message, Renderer: text::Renderer>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    menu: &'a mut menu::State,
    keyboard_modifiers: &'a mut keyboard::Modifiers,
    is_open: &'a mut bool,
    hovered_option: &'a mut Option<usize>,
    last_selection: &'a mut Option<T>,
    on_selected: Box<dyn Fn(T) -> Message>,
    options: Cow<'a, [T]>,
    placeholder: Option<String>,
    selected: Option<T>,
    width: Length,
    padding: Padding,
    text_size: Option<u16>,
    font: Renderer::Font,
    style_sheet: Box<dyn StyleSheet + 'a>,
}

/// The local state of a [`PickList`].
#[derive(Debug, Clone)]
pub struct State<T> {
    menu: menu::State,
    keyboard_modifiers: keyboard::Modifiers,
    is_open: bool,
    hovered_option: Option<usize>,
    last_selection: Option<T>,
}

impl<T> Default for State<T> {
    fn default() -> Self {
        Self {
            menu: menu::State::default(),
            keyboard_modifiers: keyboard::Modifiers::default(),
            is_open: bool::default(),
            hovered_option: Option::default(),
            last_selection: Option::default(),
        }
    }
}

impl<'a, T: 'a, Message, Renderer: text::Renderer>
    PickList<'a, T, Message, Renderer>
where
    T: ToString + Eq,
    [T]: ToOwned<Owned = Vec<T>>,
{
    /// The default padding of a [`PickList`].
    pub const DEFAULT_PADDING: Padding = Padding::new(5);

    /// Creates a new [`PickList`] with the given [`State`], a list of options,
    /// the current selected value, and the message to produce when an option is
    /// selected.
    pub fn new(
        state: &'a mut State<T>,
        options: impl Into<Cow<'a, [T]>>,
        selected: Option<T>,
        on_selected: impl Fn(T) -> Message + 'static,
    ) -> Self {
        let State {
            menu,
            keyboard_modifiers,
            is_open,
            hovered_option,
            last_selection,
        } = state;

        Self {
            menu,
            keyboard_modifiers,
            is_open,
            hovered_option,
            last_selection,
            on_selected: Box::new(on_selected),
            options: options.into(),
            placeholder: None,
            selected,
            width: Length::Shrink,
            text_size: None,
            padding: Self::DEFAULT_PADDING,
            font: Default::default(),
            style_sheet: Default::default(),
        }
    }

    /// Sets the placeholder of the [`PickList`].
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Sets the width of the [`PickList`].
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the [`Padding`] of the [`PickList`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the text size of the [`PickList`].
    pub fn text_size(mut self, size: u16) -> Self {
        self.text_size = Some(size);
        self
    }

    /// Sets the font of the [`PickList`].
    pub fn font(mut self, font: Renderer::Font) -> Self {
        self.font = font;
        self
    }

    /// Sets the style of the [`PickList`].
    pub fn style(
        mut self,
        style_sheet: impl Into<Box<dyn StyleSheet + 'a>>,
    ) -> Self {
        self.style_sheet = style_sheet.into();
        self
    }
}

impl<'a, T: 'a, Message, Renderer> Widget<Message, Renderer>
    for PickList<'a, T, Message, Renderer>
where
    T: Clone + ToString + Eq,
    [T]: ToOwned<Owned = Vec<T>>,
    Message: 'static,
    Renderer: text::Renderer + 'a,
{
    fn width(&self) -> Length {
        self.width
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
            .pad(self.padding);

        let text_size = self.text_size.unwrap_or(renderer.default_size());
        let font = self.font;

        let max_width = match self.width {
            Length::Shrink => {
                let measure = |label: &str| -> u32 {
                    let (width, _) = renderer.measure(
                        label,
                        text_size,
                        font,
                        Size::new(f32::INFINITY, f32::INFINITY),
                    );

                    width.round() as u32
                };

                let labels = self.options.iter().map(ToString::to_string);

                let labels_width =
                    labels.map(|label| measure(&label)).max().unwrap_or(100);

                let placeholder_width = self
                    .placeholder
                    .as_ref()
                    .map(String::as_str)
                    .map(measure)
                    .unwrap_or(100);

                labels_width.max(placeholder_width)
            }
            _ => 0,
        };

        let size = {
            let intrinsic = Size::new(
                max_width as f32
                    + f32::from(text_size)
                    + f32::from(self.padding.left),
                f32::from(text_size),
            );

            limits.resolve(intrinsic).pad(self.padding)
        };

        layout::Node::new(size)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash as _;

        match self.width {
            Length::Shrink => {
                self.placeholder.hash(state);

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
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                let event_status = if *self.is_open {
                    // TODO: Encode cursor availability in the type system
                    *self.is_open =
                        cursor_position.x < 0.0 || cursor_position.y < 0.0;

                    event::Status::Captured
                } else if layout.bounds().contains(cursor_position) {
                    let selected = self.selected.as_ref();

                    *self.is_open = true;
                    *self.hovered_option = self
                        .options
                        .iter()
                        .position(|option| Some(option) == selected);

                    event::Status::Captured
                } else {
                    event::Status::Ignored
                };

                if let Some(last_selection) = self.last_selection.take() {
                    shell.publish((self.on_selected)(last_selection));

                    *self.is_open = false;

                    event::Status::Captured
                } else {
                    event_status
                }
            }
            Event::Mouse(mouse::Event::WheelScrolled {
                delta: mouse::ScrollDelta::Lines { y, .. },
            }) if self.keyboard_modifiers.command()
                && layout.bounds().contains(cursor_position)
                && !*self.is_open =>
            {
                fn find_next<'a, T: PartialEq>(
                    selected: &'a T,
                    mut options: impl Iterator<Item = &'a T>,
                ) -> Option<&'a T> {
                    let _ = options.find(|&option| option == selected);

                    options.next()
                }

                let next_option = if y < 0.0 {
                    if let Some(selected) = self.selected.as_ref() {
                        find_next(selected, self.options.iter())
                    } else {
                        self.options.first()
                    }
                } else if y > 0.0 {
                    if let Some(selected) = self.selected.as_ref() {
                        find_next(selected, self.options.iter().rev())
                    } else {
                        self.options.last()
                    }
                } else {
                    None
                };

                if let Some(next_option) = next_option {
                    shell.publish((self.on_selected)(next_option.clone()));
                }

                event::Status::Captured
            }
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                *self.keyboard_modifiers = modifiers;

                event::Status::Ignored
            }
            _ => event::Status::Ignored,
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) -> mouse::Interaction {
        let bounds = layout.bounds();
        let is_mouse_over = bounds.contains(cursor_position);

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
        cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let is_mouse_over = bounds.contains(cursor_position);
        let is_selected = self.selected.is_some();

        let style = if is_mouse_over {
            self.style_sheet.hovered()
        } else {
            self.style_sheet.active()
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border_color: style.border_color,
                border_width: style.border_width,
                border_radius: style.border_radius,
            },
            style.background,
        );

        renderer.fill_text(Text {
            content: &Renderer::ARROW_DOWN_ICON.to_string(),
            font: Renderer::ICON_FONT,
            size: bounds.height * style.icon_size,
            bounds: Rectangle {
                x: bounds.x + bounds.width
                    - f32::from(self.padding.horizontal()),
                y: bounds.center_y(),
                ..bounds
            },
            color: style.text_color,
            horizontal_alignment: alignment::Horizontal::Right,
            vertical_alignment: alignment::Vertical::Center,
        });

        if let Some(label) = self
            .selected
            .as_ref()
            .map(ToString::to_string)
            .as_ref()
            .or_else(|| self.placeholder.as_ref())
        {
            renderer.fill_text(Text {
                content: label,
                size: f32::from(
                    self.text_size.unwrap_or(renderer.default_size()),
                ),
                font: self.font,
                color: is_selected
                    .then(|| style.text_color)
                    .unwrap_or(style.placeholder_color),
                bounds: Rectangle {
                    x: bounds.x + f32::from(self.padding.left),
                    y: bounds.center_y(),
                    ..bounds
                },
                horizontal_alignment: alignment::Horizontal::Left,
                vertical_alignment: alignment::Vertical::Center,
            })
        }
    }

    fn overlay(
        &mut self,
        layout: Layout<'_>,
    ) -> Option<overlay::Element<'_, Message, Renderer>> {
        if *self.is_open {
            let bounds = layout.bounds();

            let mut menu = Menu::new(
                &mut self.menu,
                &self.options,
                &mut self.hovered_option,
                &mut self.last_selection,
            )
            .width(bounds.width.round() as u16)
            .padding(self.padding)
            .font(self.font)
            .style(self.style_sheet.menu());

            if let Some(text_size) = self.text_size {
                menu = menu.text_size(text_size);
            }

            Some(menu.overlay(layout.position(), bounds.height))
        } else {
            None
        }
    }
}

impl<'a, T: 'a, Message, Renderer> Into<Element<'a, Message, Renderer>>
    for PickList<'a, T, Message, Renderer>
where
    T: Clone + ToString + Eq,
    [T]: ToOwned<Owned = Vec<T>>,
    Renderer: text::Renderer + 'a,
    Message: 'static,
{
    fn into(self) -> Element<'a, Message, Renderer> {
        Element::new(self)
    }
}
