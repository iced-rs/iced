//! Display a dropdown list of selectable values.
use crate::container;
use crate::core::alignment;
use crate::core::event::{self, Event};
use crate::core::keyboard;
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::text::{self, Paragraph as _, Text};
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
    text_size: Option<Pixels>,
    text_line_height: text::LineHeight,
    text_shaping: text::Shaping,
    font: Option<Renderer::Font>,
    handle: Handle<Renderer::Font>,
    style: <Renderer::Theme as StyleSheet>::Style,
}

impl<'a, T: 'a, Message, Renderer> PickList<'a, T, Message, Renderer>
where
    T: ToString + PartialEq,
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
            handle: Handle::default(),
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
        self.text_size = Some(size.into());
        self
    }

    /// Sets the text [`text::LineHeight`] of the [`PickList`].
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
    T: Clone + ToString + PartialEq + 'static,
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
        tree::Tag::of::<State<Renderer::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::<Renderer::Paragraph>::new())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Shrink,
        }
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout(
            tree.state.downcast_mut::<State<Renderer::Paragraph>>(),
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
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        update(
            event,
            layout,
            cursor,
            shell,
            self.on_selected.as_ref(),
            self.selected.as_ref(),
            &self.options,
            || tree.state.downcast_mut::<State<Renderer::Paragraph>>(),
        )
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse_interaction(layout, cursor)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let font = self.font.unwrap_or_else(|| renderer.default_font());
        draw(
            renderer,
            theme,
            layout,
            cursor,
            self.padding,
            self.text_size,
            self.text_line_height,
            self.text_shaping,
            font,
            self.placeholder.as_deref(),
            self.selected.as_ref(),
            &self.handle,
            &self.style,
            || tree.state.downcast_ref::<State<Renderer::Paragraph>>(),
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();

        overlay(
            layout,
            state,
            self.padding,
            self.text_size,
            self.text_shaping,
            self.font.unwrap_or_else(|| renderer.default_font()),
            &self.options,
            &self.on_selected,
            self.style.clone(),
        )
    }
}

impl<'a, T: 'a, Message, Renderer> From<PickList<'a, T, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    T: Clone + ToString + PartialEq + 'static,
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

/// The state of a [`PickList`].
#[derive(Debug)]
pub struct State<P: text::Paragraph> {
    menu: menu::State,
    keyboard_modifiers: keyboard::Modifiers,
    is_open: bool,
    hovered_option: Option<usize>,
    options: Vec<P>,
    placeholder: P,
}

impl<P: text::Paragraph> State<P> {
    /// Creates a new [`State`] for a [`PickList`].
    fn new() -> Self {
        Self {
            menu: menu::State::default(),
            keyboard_modifiers: keyboard::Modifiers::default(),
            is_open: bool::default(),
            hovered_option: Option::default(),
            options: Vec::new(),
            placeholder: P::default(),
        }
    }
}

impl<P: text::Paragraph> Default for State<P> {
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
        size: Option<Pixels>,
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
    pub size: Option<Pixels>,
    /// Line height of the content.
    pub line_height: text::LineHeight,
    /// The shaping strategy of the icon.
    pub shaping: text::Shaping,
}

/// Computes the layout of a [`PickList`].
pub fn layout<Renderer, T>(
    state: &mut State<Renderer::Paragraph>,
    renderer: &Renderer,
    limits: &layout::Limits,
    width: Length,
    padding: Padding,
    text_size: Option<Pixels>,
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

    let font = font.unwrap_or_else(|| renderer.default_font());
    let text_size = text_size.unwrap_or_else(|| renderer.default_size());

    state.options.resize_with(options.len(), Default::default);

    let option_text = Text {
        content: "",
        bounds: Size::new(
            f32::INFINITY,
            text_line_height.to_absolute(text_size).into(),
        ),
        size: text_size,
        line_height: text_line_height,
        font,
        horizontal_alignment: alignment::Horizontal::Left,
        vertical_alignment: alignment::Vertical::Center,
        shaping: text_shaping,
    };

    for (option, paragraph) in options.iter().zip(state.options.iter_mut()) {
        let label = option.to_string();

        paragraph.update(Text {
            content: &label,
            ..option_text
        });
    }

    if let Some(placeholder) = placeholder {
        state.placeholder.update(Text {
            content: placeholder,
            ..option_text
        });
    }

    let max_width = match width {
        Length::Shrink => {
            let labels_width =
                state.options.iter().fold(0.0, |width, paragraph| {
                    f32::max(width, paragraph.min_width())
                });

            labels_width.max(
                placeholder
                    .map(|_| state.placeholder.min_width())
                    .unwrap_or(0.0),
            )
        }
        _ => 0.0,
    };

    let size = {
        let intrinsic = Size::new(
            max_width + text_size.0 + padding.left,
            f32::from(text_line_height.to_absolute(text_size)),
        );

        limits
            .width(width)
            .shrink(padding)
            .resolve(width, Length::Shrink, intrinsic)
            .expand(padding)
    };

    layout::Node::new(size)
}

/// Processes an [`Event`] and updates the [`State`] of a [`PickList`]
/// accordingly.
pub fn update<'a, T, P, Message>(
    event: Event,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    shell: &mut Shell<'_, Message>,
    on_selected: &dyn Fn(T) -> Message,
    selected: Option<&T>,
    options: &[T],
    state: impl FnOnce() -> &'a mut State<P>,
) -> event::Status
where
    T: PartialEq + Clone + 'a,
    P: text::Paragraph + 'a,
{
    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerPressed { .. }) => {
            let state = state();

            if state.is_open {
                // Event wasn't processed by overlay, so cursor was clicked either outside it's
                // bounds or on the drop-down, either way we close the overlay.
                state.is_open = false;

                event::Status::Captured
            } else if cursor.is_over(layout.bounds()) {
                state.is_open = true;
                state.hovered_option =
                    options.iter().position(|option| Some(option) == selected);

                event::Status::Captured
            } else {
                event::Status::Ignored
            }
        }
        Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Lines { y, .. },
        }) => {
            let state = state();

            if state.keyboard_modifiers.command()
                && cursor.is_over(layout.bounds())
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
    cursor: mouse::Cursor,
) -> mouse::Interaction {
    let bounds = layout.bounds();
    let is_mouse_over = cursor.is_over(bounds);

    if is_mouse_over {
        mouse::Interaction::Pointer
    } else {
        mouse::Interaction::default()
    }
}

/// Returns the current overlay of a [`PickList`].
pub fn overlay<'a, T, Message, Renderer>(
    layout: Layout<'_>,
    state: &'a mut State<Renderer::Paragraph>,
    padding: Padding,
    text_size: Option<Pixels>,
    text_shaping: text::Shaping,
    font: Renderer::Font,
    options: &'a [T],
    on_selected: &'a dyn Fn(T) -> Message,
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
            |option| {
                state.is_open = false;

                (on_selected)(option)
            },
            None,
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
    cursor: mouse::Cursor,
    padding: Padding,
    text_size: Option<Pixels>,
    text_line_height: text::LineHeight,
    text_shaping: text::Shaping,
    font: Renderer::Font,
    placeholder: Option<&str>,
    selected: Option<&T>,
    handle: &Handle<Renderer::Font>,
    style: &<Renderer::Theme as StyleSheet>::Style,
    state: impl FnOnce() -> &'a State<Renderer::Paragraph>,
    viewport: &Rectangle,
) where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
    T: ToString + 'a,
{
    let bounds = layout.bounds();
    let is_mouse_over = cursor.is_over(bounds);
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

    let handle = match handle {
        Handle::Arrow { size } => Some((
            Renderer::ICON_FONT,
            Renderer::ARROW_DOWN_ICON,
            *size,
            text::LineHeight::default(),
            text::Shaping::Basic,
        )),
        Handle::Static(Icon {
            font,
            code_point,
            size,
            line_height,
            shaping,
        }) => Some((*font, *code_point, *size, *line_height, *shaping)),
        Handle::Dynamic { open, closed } => {
            if state().is_open {
                Some((
                    open.font,
                    open.code_point,
                    open.size,
                    open.line_height,
                    open.shaping,
                ))
            } else {
                Some((
                    closed.font,
                    closed.code_point,
                    closed.size,
                    closed.line_height,
                    closed.shaping,
                ))
            }
        }
        Handle::None => None,
    };

    if let Some((font, code_point, size, line_height, shaping)) = handle {
        let size = size.unwrap_or_else(|| renderer.default_size());

        renderer.fill_text(
            Text {
                content: &code_point.to_string(),
                size,
                line_height,
                font,
                bounds: Size::new(
                    bounds.width,
                    f32::from(line_height.to_absolute(size)),
                ),
                horizontal_alignment: alignment::Horizontal::Right,
                vertical_alignment: alignment::Vertical::Center,
                shaping,
            },
            Point::new(
                bounds.x + bounds.width - padding.horizontal(),
                bounds.center_y(),
            ),
            style.handle_color,
            *viewport,
        );
    }

    let label = selected.map(ToString::to_string);

    if let Some(label) = label.as_deref().or(placeholder) {
        let text_size = text_size.unwrap_or_else(|| renderer.default_size());

        renderer.fill_text(
            Text {
                content: label,
                size: text_size,
                line_height: text_line_height,
                font,
                bounds: Size::new(
                    bounds.width - padding.horizontal(),
                    f32::from(text_line_height.to_absolute(text_size)),
                ),
                horizontal_alignment: alignment::Horizontal::Left,
                vertical_alignment: alignment::Vertical::Center,
                shaping: text_shaping,
            },
            Point::new(bounds.x + padding.left, bounds.center_y()),
            if is_selected {
                style.text_color
            } else {
                style.placeholder_color
            },
            *viewport,
        );
    }
}
