//! Display a dropdown list of selectable values.
use crate::container;
use crate::core::alignment;
use crate::core::event::{self, Event};
use crate::core::keyboard;
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::text::{self, Text};
use crate::core::touch;
use crate::core::widget::tree::{self, Tree};
use crate::core::{
    Clipboard, Element, Layout, Length, Padding, Pixels, Point, Rectangle,
    Shell, Size, Widget,
};
use crate::overlay::menu::{self, Menu};
use crate::scrollable;

use std::borrow::Cow;

pub use crate::style::pick_list::{Appearance, StyleSheet};

/// A widget for selecting a single value from a list of options.
#[allow(missing_debug_implementations)]
pub struct PickList<'a, T, Message, Renderer = crate::Renderer>
where
    [T]: ToOwned<Owned = Vec<T>>,
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    on_selected: Box<dyn Fn(T) -> Message + 'a>,
    options: Cow<'a, [T]>,
    placeholder: Option<String>,
    selected: Option<T>,
    width: Length,
    padding: Padding,
    text_size: Option<f32>,
    text_line_height: text::LineHeight,
    text_shaping: text::Shaping,
    font: Option<Renderer::Font>,
    handle: Handle<Renderer::Font>,
    style: <Renderer::Theme as StyleSheet>::Style,
}

impl<'a, T: 'a, Message, Renderer> PickList<'a, T, Message, Renderer>
where
    T: ToString + Eq,
    [T]: ToOwned<Owned = Vec<T>>,
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet
        + scrollable::StyleSheet
        + menu::StyleSheet
        + container::StyleSheet,
    <Renderer::Theme as menu::StyleSheet>::Style:
        From<<Renderer::Theme as StyleSheet>::Style>,
{
    /// The default padding of a [`PickList`].
    pub const DEFAULT_PADDING: Padding = Padding::new(5.0);

    /// Creates a new [`PickList`] with the given list of options, the current
    /// selected value, and the message to produce when an option is selected.
    pub fn new(
        options: impl Into<Cow<'a, [T]>>,
        selected: Option<T>,
        on_selected: impl Fn(T) -> Message + 'a,
    ) -> Self {
        Self {
            on_selected: Box::new(on_selected),
            options: options.into(),
            placeholder: None,
            selected,
            width: Length::Shrink,
            padding: Self::DEFAULT_PADDING,
            text_size: None,
            text_line_height: text::LineHeight::default(),
            text_shaping: text::Shaping::Basic,
            font: None,
            handle: Default::default(),
            style: Default::default(),
        }
    }

    /// Sets the placeholder of the [`PickList`].
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Sets the width of the [`PickList`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the [`Padding`] of the [`PickList`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the text size of the [`PickList`].
    pub fn text_size(mut self, size: impl Into<Pixels>) -> Self {
        self.text_size = Some(size.into().0);
        self
    }

    /// Sets the text [`LineHeight`] of the [`PickList`].
    pub fn text_line_height(
        mut self,
        line_height: impl Into<text::LineHeight>,
    ) -> Self {
        self.text_line_height = line_height.into();
        self
    }

    /// Sets the [`text::Shaping`] strategy of the [`PickList`].
    pub fn text_shaping(mut self, shaping: text::Shaping) -> Self {
        self.text_shaping = shaping;
        self
    }

    /// Sets the font of the [`PickList`].
    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    /// Sets the [`Handle`] of the [`PickList`].
    pub fn handle(mut self, handle: Handle<Renderer::Font>) -> Self {
        self.handle = handle;
        self
    }

    /// Sets the style of the [`PickList`].
    pub fn style(
        mut self,
        style: impl Into<<Renderer::Theme as StyleSheet>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }
}

impl<'a, T: 'a, Message, Renderer> Widget<Message, Renderer>
    for PickList<'a, T, Message, Renderer>
where
    T: Clone + ToString + Eq + 'static,
    [T]: ToOwned<Owned = Vec<T>>,
    Message: 'a,
    Renderer: text::Renderer + 'a,
    Renderer::Theme: StyleSheet
        + scrollable::StyleSheet
        + menu::StyleSheet
        + container::StyleSheet,
    <Renderer::Theme as menu::StyleSheet>::Style:
        From<<Renderer::Theme as StyleSheet>::Style>,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<T>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::<T>::new())
    }

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
        layout(
            renderer,
            limits,
            self.width,
            self.padding,
            self.text_size,
            self.text_line_height,
            self.text_shaping,
            self.font,
            self.placeholder.as_deref(),
            &self.options,
        )
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        update(
            event,
            layout,
            cursor_position,
            shell,
            self.on_selected.as_ref(),
            self.selected.as_ref(),
            &self.options,
            || tree.state.downcast_mut::<State<T>>(),
        )
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse_interaction(layout, cursor_position)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        let font = self.font.unwrap_or_else(|| renderer.default_font());
        draw(
            renderer,
            theme,
            layout,
            cursor_position,
            self.padding,
            self.text_size,
            self.text_line_height,
            self.text_shaping,
            font,
            self.placeholder.as_deref(),
            self.selected.as_ref(),
            &self.handle,
            &self.style,
            || tree.state.downcast_ref::<State<T>>(),
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        let state = tree.state.downcast_mut::<State<T>>();

        overlay(
            layout,
            state,
            self.padding,
            self.text_size,
            self.text_shaping,
            self.font.unwrap_or_else(|| renderer.default_font()),
            &self.options,
            self.style.clone(),
        )
    }
}

impl<'a, T: 'a, Message, Renderer> From<PickList<'a, T, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    T: Clone + ToString + Eq + 'static,
    [T]: ToOwned<Owned = Vec<T>>,
    Message: 'a,
    Renderer: text::Renderer + 'a,
    Renderer::Theme: StyleSheet
        + scrollable::StyleSheet
        + menu::StyleSheet
        + container::StyleSheet,
    <Renderer::Theme as menu::StyleSheet>::Style:
        From<<Renderer::Theme as StyleSheet>::Style>,
{
    fn from(pick_list: PickList<'a, T, Message, Renderer>) -> Self {
        Self::new(pick_list)
    }
}

/// The local state of a [`PickList`].
#[derive(Debug)]
pub struct State<T> {
    menu: menu::State,
    keyboard_modifiers: keyboard::Modifiers,
    is_open: bool,
    hovered_option: Option<usize>,
    last_selection: Option<T>,
}

impl<T> State<T> {
    /// Creates a new [`State`] for a [`PickList`].
    pub fn new() -> Self {
        Self {
            menu: menu::State::default(),
            keyboard_modifiers: keyboard::Modifiers::default(),
            is_open: bool::default(),
            hovered_option: Option::default(),
            last_selection: Option::default(),
        }
    }
}

impl<T> Default for State<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// The handle to the right side of the [`PickList`].
#[derive(Debug, Clone, PartialEq)]
pub enum Handle<Font> {
    /// Displays an arrow icon (▼).
    ///
    /// This is the default.
    Arrow {
        /// Font size of the content.
        size: Option<f32>,
    },
    /// A custom static handle.
    Static(Icon<Font>),
    /// A custom dynamic handle.
    Dynamic {
        /// The [`Icon`] used when [`PickList`] is closed.
        closed: Icon<Font>,
        /// The [`Icon`] used when [`PickList`] is open.
        open: Icon<Font>,
    },
    /// No handle will be shown.
    None,
}

impl<Font> Default for Handle<Font> {
    fn default() -> Self {
        Self::Arrow { size: None }
    }
}

/// The icon of a [`Handle`].
#[derive(Debug, Clone, PartialEq)]
pub struct Icon<Font> {
    /// Font that will be used to display the `code_point`,
    pub font: Font,
    /// The unicode code point that will be used as the icon.
    pub code_point: char,
    /// Font size of the content.
    pub size: Option<f32>,
    /// The shaping strategy of the icon.
    pub shaping: text::Shaping,
}

/// Computes the layout of a [`PickList`].
pub fn layout<Renderer, T>(
    renderer: &Renderer,
    limits: &layout::Limits,
    width: Length,
    padding: Padding,
    text_size: Option<f32>,
    text_line_height: text::LineHeight,
    text_shaping: text::Shaping,
    font: Option<Renderer::Font>,
    placeholder: Option<&str>,
    options: &[T],
) -> layout::Node
where
    Renderer: text::Renderer,
    T: ToString,
{
    use std::f32;

    let limits = limits.width(width).height(Length::Shrink).pad(padding);
    let text_size = text_size.unwrap_or_else(|| renderer.default_size());

    let max_width = match width {
        Length::Shrink => {
            let measure = |label: &str| -> f32 {
                let width = renderer.measure_width(
                    label,
                    text_size,
                    font.unwrap_or_else(|| renderer.default_font()),
                    text_shaping,
                );

                width.round()
            };

            let labels = options.iter().map(ToString::to_string);

            let labels_width = labels
                .map(|label| measure(&label))
                .fold(100.0, |candidate, current| current.max(candidate));

            let placeholder_width = placeholder.map(measure).unwrap_or(100.0);

            labels_width.max(placeholder_width)
        }
        _ => 0.0,
    };

    let size = {
        let intrinsic = Size::new(
            max_width + text_size + padding.left,
            f32::from(text_line_height.to_absolute(Pixels(text_size))),
        );

        limits.resolve(intrinsic).pad(padding)
    };

    layout::Node::new(size)
}

/// Processes an [`Event`] and updates the [`State`] of a [`PickList`]
/// accordingly.
pub fn update<'a, T, Message>(
    event: Event,
    layout: Layout<'_>,
    cursor_position: Point,
    shell: &mut Shell<'_, Message>,
    on_selected: &dyn Fn(T) -> Message,
    selected: Option<&T>,
    options: &[T],
    state: impl FnOnce() -> &'a mut State<T>,
) -> event::Status
where
    T: PartialEq + Clone + 'a,
{
    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerPressed { .. }) => {
            let state = state();

            let event_status = if state.is_open {
                // Event wasn't processed by overlay, so cursor was clicked either outside it's
                // bounds or on the drop-down, either way we close the overlay.
                state.is_open = false;

                event::Status::Captured
            } else if layout.bounds().contains(cursor_position) {
                state.is_open = true;
                state.hovered_option =
                    options.iter().position(|option| Some(option) == selected);

                event::Status::Captured
            } else {
                event::Status::Ignored
            };

            if let Some(last_selection) = state.last_selection.take() {
                shell.publish((on_selected)(last_selection));

                state.is_open = false;

                event::Status::Captured
            } else {
                event_status
            }
        }
        Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Lines { y, .. },
        }) => {
            let state = state();

            if state.keyboard_modifiers.command()
                && layout.bounds().contains(cursor_position)
                && !state.is_open
            {
                fn find_next<'a, T: PartialEq>(
                    selected: &'a T,
                    mut options: impl Iterator<Item = &'a T>,
                ) -> Option<&'a T> {
                    let _ = options.find(|&option| option == selected);

                    options.next()
                }

                let next_option = if y < 0.0 {
                    if let Some(selected) = selected {
                        find_next(selected, options.iter())
                    } else {
                        options.first()
                    }
                } else if y > 0.0 {
                    if let Some(selected) = selected {
                        find_next(selected, options.iter().rev())
                    } else {
                        options.last()
                    }
                } else {
                    None
                };

                if let Some(next_option) = next_option {
                    shell.publish((on_selected)(next_option.clone()));
                }

                event::Status::Captured
            } else {
                event::Status::Ignored
            }
        }
        Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
            let state = state();

            state.keyboard_modifiers = modifiers;

            event::Status::Ignored
        }
        _ => event::Status::Ignored,
    }
}

/// Returns the current [`mouse::Interaction`] of a [`PickList`].
pub fn mouse_interaction(
    layout: Layout<'_>,
    cursor_position: Point,
) -> mouse::Interaction {
    let bounds = layout.bounds();
    let is_mouse_over = bounds.contains(cursor_position);

    if is_mouse_over {
        mouse::Interaction::Pointer
    } else {
        mouse::Interaction::default()
    }
}

/// Returns the current overlay of a [`PickList`].
pub fn overlay<'a, T, Message, Renderer>(
    layout: Layout<'_>,
    state: &'a mut State<T>,
    padding: Padding,
    text_size: Option<f32>,
    text_shaping: text::Shaping,
    font: Renderer::Font,
    options: &'a [T],
    style: <Renderer::Theme as StyleSheet>::Style,
) -> Option<overlay::Element<'a, Message, Renderer>>
where
    T: Clone + ToString,
    Message: 'a,
    Renderer: text::Renderer + 'a,
    Renderer::Theme: StyleSheet
        + scrollable::StyleSheet
        + menu::StyleSheet
        + container::StyleSheet,
    <Renderer::Theme as menu::StyleSheet>::Style:
        From<<Renderer::Theme as StyleSheet>::Style>,
{
    if state.is_open {
        let bounds = layout.bounds();

        let mut menu = Menu::new(
            &mut state.menu,
            options,
            &mut state.hovered_option,
            &mut state.last_selection,
        )
        .width(bounds.width)
        .padding(padding)
        .font(font)
        .text_shaping(text_shaping)
        .style(style);

        if let Some(text_size) = text_size {
            menu = menu.text_size(text_size);
        }

        Some(menu.overlay(layout.position(), bounds.height))
    } else {
        None
    }
}

/// Draws a [`PickList`].
pub fn draw<'a, T, Renderer>(
    renderer: &mut Renderer,
    theme: &Renderer::Theme,
    layout: Layout<'_>,
    cursor_position: Point,
    padding: Padding,
    text_size: Option<f32>,
    text_line_height: text::LineHeight,
    text_shaping: text::Shaping,
    font: Renderer::Font,
    placeholder: Option<&str>,
    selected: Option<&T>,
    handle: &Handle<Renderer::Font>,
    style: &<Renderer::Theme as StyleSheet>::Style,
    state: impl FnOnce() -> &'a State<T>,
) where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
    T: ToString + 'a,
{
    let bounds = layout.bounds();
    let is_mouse_over = bounds.contains(cursor_position);
    let is_selected = selected.is_some();

    let style = if is_mouse_over {
        theme.hovered(style)
    } else {
        theme.active(style)
    };

    renderer.fill_quad(
        renderer::Quad {
            bounds,
            border_color: style.border_color,
            border_width: style.border_width,
            border_radius: style.border_radius.into(),
        },
        style.background,
    );

    let handle = match handle {
        Handle::Arrow { size } => Some((
            Renderer::ICON_FONT,
            Renderer::ARROW_DOWN_ICON,
            *size,
            text::Shaping::Basic,
        )),
        Handle::Static(Icon {
            font,
            code_point,
            size,
            shaping,
        }) => Some((*font, *code_point, *size, *shaping)),
        Handle::Dynamic { open, closed } => {
            if state().is_open {
                Some((open.font, open.code_point, open.size, open.shaping))
            } else {
                Some((
                    closed.font,
                    closed.code_point,
                    closed.size,
                    closed.shaping,
                ))
            }
        }
        Handle::None => None,
    };

    if let Some((font, code_point, size, shaping)) = handle {
        let size = size.unwrap_or_else(|| renderer.default_size());

        renderer.fill_text(Text {
            content: &code_point.to_string(),
            size,
            line_height: text::LineHeight::default(),
            font,
            color: style.handle_color,
            bounds: Rectangle {
                x: bounds.x + bounds.width - padding.horizontal(),
                y: bounds.center_y(),
                height: size * 1.2,
                ..bounds
            },
            horizontal_alignment: alignment::Horizontal::Right,
            vertical_alignment: alignment::Vertical::Center,
            shaping,
        });
    }

    let label = selected.map(ToString::to_string);

    if let Some(label) = label.as_deref().or(placeholder) {
        let text_size = text_size.unwrap_or_else(|| renderer.default_size());

        renderer.fill_text(Text {
            content: label,
            size: text_size,
            line_height: text_line_height,
            font,
            color: if is_selected {
                style.text_color
            } else {
                style.placeholder_color
            },
            bounds: Rectangle {
                x: bounds.x + padding.left,
                y: bounds.center_y(),
                width: bounds.width - padding.horizontal(),
                height: text_size * 1.2,
            },
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Center,
            shaping: text_shaping,
        });
    }
}
