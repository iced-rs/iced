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
use crate::widget::container;
use crate::widget::scrollable;
use crate::widget::tree::{self, Tree};
use crate::{
    Clipboard, Element, Layout, Length, Padding, Point, Rectangle, Shell, Size,
    Widget,
};
use std::borrow::Cow;

pub use iced_style::pick_list::{Appearance, StyleSheet};

/// A widget for selecting a single value from a list of options.
#[allow(missing_debug_implementations)]
pub struct PickList<'a, T, Message, Renderer>
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
    text_size: Option<u16>,
    font: Renderer::Font,
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
    pub const DEFAULT_PADDING: Padding = Padding::new(5);

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
            text_size: None,
            padding: Self::DEFAULT_PADDING,
            font: Default::default(),
            style: Default::default(),
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
            &self.font,
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
        _tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        draw(
            renderer,
            theme,
            layout,
            cursor_position,
            self.padding,
            self.text_size,
            &self.font,
            self.placeholder.as_deref(),
            self.selected.as_ref(),
            &self.style,
        )
    }

    fn overlay<'b>(
        &'b self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        let state = tree.state.downcast_mut::<State<T>>();

        overlay(
            layout,
            state,
            self.padding,
            self.text_size,
            self.font.clone(),
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

/// Computes the layout of a [`PickList`].
pub fn layout<Renderer, T>(
    renderer: &Renderer,
    limits: &layout::Limits,
    width: Length,
    padding: Padding,
    text_size: Option<u16>,
    font: &Renderer::Font,
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
            let measure = |label: &str| -> u32 {
                let (width, _) = renderer.measure(
                    label,
                    text_size,
                    font.clone(),
                    Size::new(f32::INFINITY, f32::INFINITY),
                );

                width.round() as u32
            };

            let labels = options.iter().map(ToString::to_string);

            let labels_width =
                labels.map(|label| measure(&label)).max().unwrap_or(100);

            let placeholder_width = placeholder.map(measure).unwrap_or(100);

            labels_width.max(placeholder_width)
        }
        _ => 0,
    };

    let size = {
        let intrinsic = Size::new(
            max_width as f32 + f32::from(text_size) + f32::from(padding.left),
            f32::from(text_size),
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
    text_size: Option<u16>,
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
        .width(bounds.width.round() as u16)
        .padding(padding)
        .font(font)
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
pub fn draw<T, Renderer>(
    renderer: &mut Renderer,
    theme: &Renderer::Theme,
    layout: Layout<'_>,
    cursor_position: Point,
    padding: Padding,
    text_size: Option<u16>,
    font: &Renderer::Font,
    placeholder: Option<&str>,
    selected: Option<&T>,
    style: &<Renderer::Theme as StyleSheet>::Style,
) where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
    T: ToString,
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
            border_radius: style.border_radius,
        },
        style.background,
    );

    renderer.fill_text(Text {
        content: &Renderer::ARROW_DOWN_ICON.to_string(),
        font: Renderer::ICON_FONT,
        size: bounds.height * style.icon_size,
        bounds: Rectangle {
            x: bounds.x + bounds.width - f32::from(padding.horizontal()),
            y: bounds.center_y(),
            ..bounds
        },
        color: style.text_color,
        horizontal_alignment: alignment::Horizontal::Right,
        vertical_alignment: alignment::Vertical::Center,
    });

    let label = selected.map(ToString::to_string);

    if let Some(label) = label.as_deref().or(placeholder) {
        let text_size =
            f32::from(text_size.unwrap_or_else(|| renderer.default_size()));

        renderer.fill_text(Text {
            content: label,
            size: text_size,
            font: font.clone(),
            color: if is_selected {
                style.text_color
            } else {
                style.placeholder_color
            },
            bounds: Rectangle {
                x: bounds.x + f32::from(padding.left),
                y: bounds.center_y() - text_size / 2.0,
                width: bounds.width - f32::from(padding.horizontal()),
                height: text_size,
            },
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Top,
        });
    }
}
