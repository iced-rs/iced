//! Display a dropdown list of searchable and selectable options.
use crate::core::event::{self, Event};
use crate::core::keyboard;
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::text;
use crate::core::widget::{self, Widget};
use crate::core::{Clipboard, Element, Length, Padding, Rectangle, Shell};
use crate::overlay::menu;
use crate::text::LineHeight;
use crate::{container, scrollable, text_input, TextInput};

use std::cell::RefCell;
use std::fmt::Display;
use std::time::Instant;

/// A widget for searching and selecting a single value from a list of options.
///
/// This widget is composed by a [`TextInput`] that can be filled with the text
/// to search for corresponding values from the list of options that are displayed
/// as a [`Menu`].
#[allow(missing_debug_implementations)]
pub struct ComboBox<'a, T, Message, Renderer = crate::Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: text_input::StyleSheet + menu::StyleSheet,
{
    state: &'a State<T>,
    text_input: TextInput<'a, TextInputEvent, Renderer>,
    font: Option<Renderer::Font>,
    selection: text_input::Value,
    on_selected: Box<dyn Fn(T) -> Message>,
    on_selection: Option<Box<dyn Fn(T) -> Message>>,
    on_blur: Option<Message>,
    on_input: Option<Box<dyn Fn(String) -> Message>>,
    menu_style: <Renderer::Theme as menu::StyleSheet>::Style,
    padding: Padding,
    size: Option<f32>,
}

impl<'a, T, Message, Renderer> ComboBox<'a, T, Message, Renderer>
where
    T: std::fmt::Display + Clone,
    Renderer: text::Renderer,
    Renderer::Theme: text_input::StyleSheet + menu::StyleSheet,
{
    /// Creates a new [`ComboBox`] with the given list of options, a placeholder,
    /// the current selected value, and the message to produce when an option is
    /// selected.
    pub fn new(
        state: &'a State<T>,
        placeholder: &str,
        selection: Option<&T>,
        on_selected: impl Fn(T) -> Message + 'static,
    ) -> Self {
        let text_input = TextInput::new(placeholder, &state.value())
            .on_input(TextInputEvent::TextChanged);

        let selection = selection.map(T::to_string).unwrap_or_default();

        Self {
            state,
            text_input,
            font: None,
            selection: text_input::Value::new(&selection),
            on_selected: Box::new(on_selected),
            on_selection: None,
            on_input: None,
            on_blur: None,
            menu_style: Default::default(),
            padding: Padding::new(0.0),
            size: None,
        }
    }

    /// Sets the message that should be produced when some text is typed into
    /// the [`TextInput`] of the [`ComboBox`].
    pub fn on_input(
        mut self,
        on_input: impl Fn(String) -> Message + 'static,
    ) -> Self {
        self.on_input = Some(Box::new(on_input));
        self
    }

    /// Sets the message that will be produced when an option of the
    /// [`ComboBox`] is hovered using the arrow keys.
    pub fn on_selection(
        mut self,
        on_selection: impl Fn(T) -> Message + 'static,
    ) -> Self {
        self.on_selection = Some(Box::new(on_selection));
        self
    }

    /// Sets the message that will be produced when the outside area
    /// of the [`ComboBox`] is pressed.
    pub fn on_blur(mut self, message: Message) -> Self {
        self.on_blur = Some(message);
        self
    }

    /// Sets the [`Padding`] of the [`ComboBox`].
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self.text_input = self.text_input.padding(self.padding);
        self
    }

    /// Sets the style of the [`ComboBox`].
    // TODO: Define its own `StyleSheet` trait
    pub fn style<S>(mut self, style: S) -> Self
    where
        S: Into<<Renderer::Theme as text_input::StyleSheet>::Style>
            + Into<<Renderer::Theme as menu::StyleSheet>::Style>
            + Clone,
    {
        self.menu_style = style.clone().into();
        self.text_input = self.text_input.style(style);
        self
    }

    /// Sets the style of the [`TextInput`] of the [`ComboBox`].
    pub fn text_input_style<S>(mut self, style: S) -> Self
    where
        S: Into<<Renderer::Theme as text_input::StyleSheet>::Style> + Clone,
    {
        self.text_input = self.text_input.style(style);
        self
    }

    /// Sets the [`Font`] of the [`ComboBox`].
    pub fn font(mut self, font: Renderer::Font) -> Self {
        self.text_input = self.text_input.font(font);
        self.font = Some(font);
        self
    }

    /// Sets the [`Icon`] of the [`ComboBox`].
    pub fn icon(mut self, icon: text_input::Icon<Renderer::Font>) -> Self {
        self.text_input = self.text_input.icon(icon);
        self
    }

    /// Returns whether the [`ComboBox`] is currently focused or not.
    pub fn is_focused(&self) -> bool {
        self.state.is_focused()
    }

    /// Sets the text sixe of the [`ComboBox`].
    pub fn size(mut self, size: f32) -> Self {
        self.text_input = self.text_input.size(size);
        self.size = Some(size);
        self
    }

    /// Sets the [`LineHeight`] of the [`ComboBox`].
    pub fn line_height(self, line_height: impl Into<LineHeight>) -> Self {
        Self {
            text_input: self.text_input.line_height(line_height),
            ..self
        }
    }

    /// Sets the width of the [`ComboBox`].
    pub fn width(self, width: impl Into<Length>) -> Self {
        Self {
            text_input: self.text_input.width(width),
            ..self
        }
    }
}

/// The local state of a [`ComboBox`].
#[derive(Debug, Clone)]
pub struct State<T>(RefCell<Inner<T>>);

#[derive(Debug, Clone)]
struct Inner<T> {
    text_input: text_input::State,
    value: String,
    options: Vec<T>,
    option_matchers: Vec<String>,
    filtered_options: Filtered<T>,
}

#[derive(Debug, Clone)]
struct Filtered<T> {
    options: Vec<T>,
    updated: Instant,
}

impl<T> State<T>
where
    T: Display + Clone,
{
    /// Creates a new [`State`] for a [`ComboBox`] with the given list of options.
    pub fn new(options: Vec<T>) -> Self {
        Self::with_selection(options, None)
    }

    /// Creates a new [`State`] for a [`ComboBox`] with the given list of options
    /// and selected value.
    pub fn with_selection(options: Vec<T>, selection: Option<&T>) -> Self {
        let value = selection.map(T::to_string).unwrap_or_default();

        // Pre-build "matcher" strings ahead of time so that search is fast
        let option_matchers = build_matchers(&options);

        let filtered_options = Filtered::new(
            search(&options, &option_matchers, &value)
                .cloned()
                .collect(),
        );

        Self(RefCell::new(Inner {
            text_input: text_input::State::new(),
            value,
            options,
            option_matchers,
            filtered_options,
        }))
    }

    /// Focuses the [`ComboBox`].
    pub fn focused(self) -> Self {
        self.focus();
        self
    }

    /// Focuses the [`ComboBox`].
    pub fn focus(&self) {
        let mut inner = self.0.borrow_mut();

        inner.text_input.focus();
    }

    /// Unfocuses the [`ComboBox`].
    pub fn unfocus(&self) {
        let mut inner = self.0.borrow_mut();

        inner.text_input.unfocus();
    }

    /// Returns whether the [`ComboBox`] is currently focused or not.
    pub fn is_focused(&self) -> bool {
        let inner = self.0.borrow();

        inner.text_input.is_focused()
    }

    fn value(&self) -> String {
        let inner = self.0.borrow();

        inner.value.clone()
    }

    fn text_input_tree(&self) -> widget::Tree {
        let inner = self.0.borrow();

        inner.text_input_tree()
    }

    fn update_text_input(&self, tree: widget::Tree) {
        let mut inner = self.0.borrow_mut();

        inner.update_text_input(tree)
    }

    fn with_inner<O>(&self, f: impl FnOnce(&Inner<T>) -> O) -> O {
        let inner = self.0.borrow();

        f(&inner)
    }

    fn with_inner_mut(&self, f: impl FnOnce(&mut Inner<T>)) {
        let mut inner = self.0.borrow_mut();

        f(&mut inner);
    }

    fn sync_filtered_options(&self, options: &mut Filtered<T>) {
        let inner = self.0.borrow();

        inner.filtered_options.sync(options);
    }
}

impl<T> Inner<T> {
    fn text_input_tree(&self) -> widget::Tree {
        widget::Tree {
            tag: widget::tree::Tag::of::<text_input::State>(),
            state: widget::tree::State::new(self.text_input.clone()),
            children: vec![],
        }
    }

    fn update_text_input(&mut self, tree: widget::Tree) {
        self.text_input =
            tree.state.downcast_ref::<text_input::State>().clone();
    }
}

impl<T> Filtered<T>
where
    T: Clone,
{
    fn new(options: Vec<T>) -> Self {
        Self {
            options,
            updated: Instant::now(),
        }
    }

    fn empty() -> Self {
        Self {
            options: vec![],
            updated: Instant::now(),
        }
    }

    fn update(&mut self, options: Vec<T>) {
        self.options = options;
        self.updated = Instant::now();
    }

    fn sync(&self, other: &mut Filtered<T>) {
        if other.updated != self.updated {
            *other = self.clone();
        }
    }
}

struct Menu<T> {
    menu: menu::State,
    hovered_option: Option<usize>,
    new_selection: Option<T>,
    filtered_options: Filtered<T>,
}

#[derive(Debug, Clone)]
enum TextInputEvent {
    TextChanged(String),
}

impl<'a, T, Message, Renderer> Widget<Message, Renderer>
    for ComboBox<'a, T, Message, Renderer>
where
    T: Display + Clone + 'static,
    Message: Clone,
    Renderer: text::Renderer,
    Renderer::Theme: container::StyleSheet
        + text_input::StyleSheet
        + scrollable::StyleSheet
        + menu::StyleSheet,
{
    fn width(&self) -> Length {
        Widget::<TextInputEvent, Renderer>::width(&self.text_input)
    }

    fn height(&self) -> Length {
        Widget::<TextInputEvent, Renderer>::height(&self.text_input)
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.text_input.layout(renderer, limits)
    }

    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<Menu<T>>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(Menu::<T> {
            menu: menu::State::new(),
            filtered_options: Filtered::empty(),
            hovered_option: Some(0),
            new_selection: None,
        })
    }

    fn on_event(
        &mut self,
        tree: &mut widget::Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        let menu = tree.state.downcast_mut::<Menu<T>>();

        let started_focused = self.state.is_focused();
        // This is intended to check whether or not the message buffer was empty,
        // since `Shell` does not expose such functionality.
        let mut published_message_to_shell = false;

        // Create a new list of local messages
        let mut local_messages = Vec::new();
        let mut local_shell = Shell::new(&mut local_messages);

        // Provide it to the widget
        let mut tree = self.state.text_input_tree();
        let mut event_status = self.text_input.on_event(
            &mut tree,
            event.clone(),
            layout,
            cursor,
            renderer,
            clipboard,
            &mut local_shell,
            viewport,
        );
        self.state.update_text_input(tree);

        // Then finally react to them here
        for message in local_messages {
            let TextInputEvent::TextChanged(new_value) = message;
            if let Some(on_input) = &self.on_input {
                shell.publish((on_input)(new_value.clone()));
                published_message_to_shell = true;
            }

            // Couple the filtered options with the `ComboBox`
            // value and only recompute them when the value changes,
            // instead of doing it in every `view` call
            self.state.with_inner_mut(|state| {
                menu.hovered_option = Some(0);
                state.value = new_value;

                state.filtered_options.update(
                    search(
                        &state.options,
                        &state.option_matchers,
                        &state.value,
                    )
                    .cloned()
                    .collect(),
                );
            });
            shell.invalidate_layout();
        }

        if self.state.is_focused() {
            self.state.with_inner(|state| {
                if let Event::Keyboard(keyboard::Event::KeyPressed {
                    key_code,
                    ..
                }) = event
                {
                    match key_code {
                        keyboard::KeyCode::Enter => {
                            if let Some(index) = &menu.hovered_option {
                                if let Some(option) =
                                    state.filtered_options.options.get(*index)
                                {
                                    menu.new_selection = Some(option.clone());
                                }
                            }

                            event_status = event::Status::Captured;
                        }
                        keyboard::KeyCode::Up => {
                            if let Some(index) = &mut menu.hovered_option {
                                *index = index.saturating_sub(1);
                            } else {
                                menu.hovered_option = Some(0);
                            }

                            if let Some(on_selection) = &mut self.on_selection {
                                if let Some(option) =
                                    menu.hovered_option.and_then(|index| {
                                        state
                                            .filtered_options
                                            .options
                                            .get(index)
                                    })
                                {
                                    // Notify the selection
                                    shell.publish((on_selection)(
                                        option.clone(),
                                    ));
                                    published_message_to_shell = true;
                                }
                            }

                            event_status = event::Status::Captured;
                        }
                        keyboard::KeyCode::Down => {
                            if let Some(index) = &mut menu.hovered_option {
                                *index = index.saturating_add(1).min(
                                    state
                                        .filtered_options
                                        .options
                                        .len()
                                        .saturating_sub(1),
                                );
                            } else {
                                menu.hovered_option = Some(0);
                            }

                            if let Some(on_selection) = &mut self.on_selection {
                                if let Some(option) =
                                    menu.hovered_option.and_then(|index| {
                                        state
                                            .filtered_options
                                            .options
                                            .get(index)
                                    })
                                {
                                    // Notify the selection
                                    shell.publish((on_selection)(
                                        option.clone(),
                                    ));
                                    published_message_to_shell = true;
                                }
                            }

                            event_status = event::Status::Captured;
                        }
                        _ => {}
                    }
                }
            });
        }

        // If the overlay menu has selected something
        self.state.with_inner_mut(|state| {
            if let Some(selection) = menu.new_selection.take() {
                // Clear the value and reset the options and menu
                state.value = String::new();
                state.filtered_options.update(state.options.clone());
                menu.menu = menu::State::default();

                // Notify the selection
                shell.publish((self.on_selected)(selection));
                published_message_to_shell = true;

                // Unfocus the input
                let mut tree = state.text_input_tree();
                let _ = self.text_input.on_event(
                    &mut tree,
                    Event::Mouse(mouse::Event::ButtonPressed(
                        mouse::Button::Left,
                    )),
                    layout,
                    mouse::Cursor::Unavailable,
                    renderer,
                    clipboard,
                    &mut Shell::new(&mut vec![]),
                    viewport,
                );
                state.update_text_input(tree);
            }
        });

        if started_focused
            && !self.state.is_focused()
            && !published_message_to_shell
        {
            if let Some(message) = self.on_blur.take() {
                shell.publish(message);
            }
        }

        // Focus changed, invalidate widget tree to force a fresh `view`
        if started_focused != self.state.is_focused() {
            shell.invalidate_widgets();
        }

        event_status
    }

    fn mouse_interaction(
        &self,
        _tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let tree = self.state.text_input_tree();
        self.text_input
            .mouse_interaction(&tree, layout, cursor, viewport, renderer)
    }

    fn draw(
        &self,
        _tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let selection = if self.state.is_focused() || self.selection.is_empty()
        {
            None
        } else {
            Some(&self.selection)
        };

        let tree = self.state.text_input_tree();
        self.text_input
            .draw(&tree, renderer, theme, layout, cursor, selection);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        let Menu {
            menu,
            filtered_options,
            hovered_option,
            ..
        } = tree.state.downcast_mut::<Menu<T>>();

        if self.state.is_focused() {
            let bounds = layout.bounds();

            self.state.sync_filtered_options(filtered_options);

            let mut menu = menu::Menu::new(
                menu,
                &filtered_options.options,
                hovered_option,
                |x| (self.on_selected)(x),
            )
            .width(bounds.width)
            .padding(self.padding)
            .style(self.menu_style.clone());

            if let Some(font) = self.font {
                menu = menu.font(font);
            }

            if let Some(size) = self.size {
                menu = menu.text_size(size);
            }

            Some(menu.overlay(layout.position(), bounds.height))
        } else {
            None
        }
    }
}

impl<'a, T, Message, Renderer> From<ComboBox<'a, T, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    T: Display + Clone + 'static,
    Message: 'a + Clone,
    Renderer: text::Renderer + 'a,
    Renderer::Theme: container::StyleSheet
        + text_input::StyleSheet
        + scrollable::StyleSheet
        + menu::StyleSheet,
{
    fn from(combo_box: ComboBox<'a, T, Message, Renderer>) -> Self {
        Self::new(combo_box)
    }
}

/// Search list of options for a given query.
pub fn search<'a, T, A>(
    options: impl IntoIterator<Item = T> + 'a,
    option_matchers: impl IntoIterator<Item = &'a A> + 'a,
    query: &'a str,
) -> impl Iterator<Item = T> + 'a
where
    A: AsRef<str> + 'a,
{
    let query: Vec<String> = query
        .to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .map(String::from)
        .collect();

    options
        .into_iter()
        .zip(option_matchers.into_iter())
        // Make sure each part of the query is found in the option
        .filter_map(move |(option, matcher)| {
            if query.iter().all(|part| matcher.as_ref().contains(part)) {
                Some(option)
            } else {
                None
            }
        })
}

/// Build matchers from given list of options.
pub fn build_matchers<'a, T>(
    options: impl IntoIterator<Item = T> + 'a,
) -> Vec<String>
where
    T: Display + 'a,
{
    options
        .into_iter()
        .map(|opt| {
            let mut matcher = opt.to_string();
            matcher.retain(|c| c.is_ascii_alphanumeric());
            matcher.to_lowercase()
        })
        .collect()
}
