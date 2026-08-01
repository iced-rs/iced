//! Combo boxes display a dropdown list of searchable and selectable options.
//!
//! # Example
//! ```no_run
//! # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
//! # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
//! #
//! use iced::widget::combo_box;
//!
//! struct State {
//!    fruits: combo_box::State<Fruit>,
//!    favorite: Option<Fruit>,
//! }
//!
//! #[derive(Debug, Clone)]
//! enum Fruit {
//!     Apple,
//!     Orange,
//!     Strawberry,
//!     Tomato,
//! }
//!
//! #[derive(Debug, Clone)]
//! enum Message {
//!     FruitSelected(Fruit),
//! }
//!
//! fn view(state: &State) -> Element<'_, Message> {
//!     combo_box(
//!         &state.fruits,
//!         "Select your favorite fruit...",
//!         state.favorite.as_ref(),
//!         Message::FruitSelected
//!     )
//!     .into()
//! }
//!
//! fn update(state: &mut State, message: Message) {
//!     match message {
//!         Message::FruitSelected(fruit) => {
//!             state.favorite = Some(fruit);
//!         }
//!     }
//! }
//!
//! impl std::fmt::Display for Fruit {
//!     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//!         f.write_str(match self {
//!             Self::Apple => "Apple",
//!             Self::Orange => "Orange",
//!             Self::Strawberry => "Strawberry",
//!             Self::Tomato => "Tomato",
//!         })
//!     }
//! }
//! ```
use crate::core::keyboard;
use crate::core::keyboard::key;
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::text;
use crate::core::text::editor;
use crate::core::text::input;
use crate::core::widget::operation::Focusable as _;
use crate::core::widget::{self, Widget};
use crate::core::window;
use crate::core::{Element, Event, Length, Padding, Pixels, Rectangle, Shell, Size, Theme, Vector};
use crate::overlay::menu;
use crate::text::LineHeight;
use crate::text_input;

use std::fmt::Display;
use std::sync::atomic::{self, AtomicU64};

/// A widget for searching and selecting a single value from a list of options.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// #
/// use iced::widget::combo_box;
///
/// struct State {
///    fruits: combo_box::State<Fruit>,
///    favorite: Option<Fruit>,
/// }
///
/// #[derive(Debug, Clone)]
/// enum Fruit {
///     Apple,
///     Orange,
///     Strawberry,
///     Tomato,
/// }
///
/// #[derive(Debug, Clone)]
/// enum Message {
///     FruitSelected(Fruit),
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     combo_box(
///         &state.fruits,
///         "Select your favorite fruit...",
///         state.favorite.as_ref(),
///         Message::FruitSelected
///     )
///     .into()
/// }
///
/// fn update(state: &mut State, message: Message) {
///     match message {
///         Message::FruitSelected(fruit) => {
///             state.favorite = Some(fruit);
///         }
///     }
/// }
///
/// impl std::fmt::Display for Fruit {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         f.write_str(match self {
///             Self::Apple => "Apple",
///             Self::Orange => "Orange",
///             Self::Strawberry => "Strawberry",
///             Self::Tomato => "Tomato",
///         })
///     }
/// }
/// ```
pub struct ComboBox<'a, T, Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    state: &'a State<T>,
    id: Option<widget::Id>,
    placeholder: text::Fragment<'a>,
    selection: String,
    width: Length,
    line_height: LineHeight,
    font: Option<Renderer::Font>,
    on_selected: Box<dyn Fn(T) -> Message + 'a>,
    on_option_hovered: Option<Box<dyn Fn(T) -> Message + 'a>>,
    on_open: Option<Message>,
    on_close: Option<Message>,
    on_input: Option<Box<dyn Fn(String) -> Message + 'a>>,
    padding: Padding,
    size: Option<Pixels>,
    shaping: text::Shaping,
    ellipsis: text::Ellipsis,
    input_class: <Theme as text_input::Catalog>::Class<'a>,
    menu_class: <Theme as menu::Catalog>::Class<'a>,
    menu_height: Length,
    last_status: Option<text_input::Status>,
}

impl<'a, T, Message, Theme, Renderer> ComboBox<'a, T, Message, Theme, Renderer>
where
    T: std::fmt::Display + Clone,
    Theme: Catalog,
    Renderer: text::Renderer,
{
    /// Creates a new [`ComboBox`] with the given list of options, a placeholder,
    /// the current selected value, and the message to produce when an option is
    /// selected.
    pub fn new(
        state: &'a State<T>,
        placeholder: impl text::IntoFragment<'a>,
        selection: Option<&T>,
        on_selected: impl Fn(T) -> Message + 'a,
    ) -> Self {
        Self {
            state,
            id: None,
            placeholder: placeholder.into_fragment(),
            selection: selection.map(T::to_string).unwrap_or_default(),
            width: Length::Fill,
            line_height: LineHeight::default(),
            font: None,
            on_selected: Box::new(on_selected),
            on_option_hovered: None,
            on_input: None,
            on_open: None,
            on_close: None,
            padding: text_input::DEFAULT_PADDING,
            size: None,
            shaping: text::Shaping::default(),
            ellipsis: text::Ellipsis::End,
            input_class: <Theme as Catalog>::default_input(),
            menu_class: <Theme as Catalog>::default_menu(),
            menu_height: Length::Shrink,
            last_status: None,
        }
    }

    /// Sets the [`widget::Id`] of the [`ComboBox`].
    pub fn id(mut self, id: impl Into<widget::Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the message that should be produced when some text is typed into
    /// the [`ComboBox`].
    pub fn on_input(mut self, on_input: impl Fn(String) -> Message + 'a) -> Self {
        self.on_input = Some(Box::new(on_input));
        self
    }

    /// Sets the message that will be produced when an option of the
    /// [`ComboBox`] is hovered using the arrow keys.
    pub fn on_option_hovered(mut self, on_option_hovered: impl Fn(T) -> Message + 'a) -> Self {
        self.on_option_hovered = Some(Box::new(on_option_hovered));
        self
    }

    /// Sets the message that will be produced when the  [`ComboBox`] is
    /// opened.
    pub fn on_open(mut self, message: Message) -> Self {
        self.on_open = Some(message);
        self
    }

    /// Sets the message that will be produced when the outside area
    /// of the [`ComboBox`] is pressed.
    pub fn on_close(mut self, message: Message) -> Self {
        self.on_close = Some(message);
        self
    }

    /// Sets the [`Padding`] of the [`ComboBox`].
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the [`Renderer::Font`] of the [`ComboBox`].
    ///
    /// [`Renderer::Font`]: text::Renderer
    pub fn font(mut self, font: Renderer::Font) -> Self {
        self.font = Some(font);
        self
    }

    /// Sets the text sixe of the [`ComboBox`].
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = Some(size.into());
        self
    }

    /// Sets the width of the [`ComboBox`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the [`LineHeight`] of the [`ComboBox`].
    pub fn line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
        self.line_height = line_height.into();
        self
    }

    /// Sets the height of the menu of the [`ComboBox`].
    pub fn menu_height(mut self, menu_height: impl Into<Length>) -> Self {
        self.menu_height = menu_height.into();
        self
    }

    /// Sets the [`text::Shaping`] strategy of the [`ComboBox`].
    pub fn shaping(mut self, shaping: text::Shaping) -> Self {
        self.shaping = shaping;
        self
    }

    /// Sets the [`text::Ellipsis`] strategy of the [`ComboBox`].
    pub fn ellipsis(mut self, ellipsis: text::Ellipsis) -> Self {
        self.ellipsis = ellipsis;
        self
    }

    /// Sets the style of the input of the [`ComboBox`].
    #[must_use]
    pub fn input_style(
        mut self,
        style: impl Fn(&Theme, text_input::Status) -> text_input::Style + 'a,
    ) -> Self
    where
        <Theme as text_input::Catalog>::Class<'a>: From<text_input::StyleFn<'a, Theme>>,
    {
        self.input_class = (Box::new(style) as text_input::StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style of the menu of the [`ComboBox`].
    #[must_use]
    pub fn menu_style(mut self, style: impl Fn(&Theme) -> menu::Style + 'a) -> Self
    where
        <Theme as menu::Catalog>::Class<'a>: From<menu::StyleFn<'a, Theme>>,
    {
        self.menu_class = (Box::new(style) as menu::StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the input of the [`ComboBox`].
    #[cfg(feature = "advanced")]
    #[must_use]
    pub fn input_class(
        mut self,
        class: impl Into<<Theme as text_input::Catalog>::Class<'a>>,
    ) -> Self {
        self.input_class = class.into();
        self
    }

    /// Sets the style class of the menu of the [`ComboBox`].
    #[cfg(feature = "advanced")]
    #[must_use]
    pub fn menu_class(mut self, class: impl Into<<Theme as menu::Catalog>::Class<'a>>) -> Self {
        self.menu_class = class.into();
        self
    }
}

/// The local state of a [`ComboBox`].
#[derive(Debug, Clone)]
pub struct State<T> {
    options: Vec<T>,
    version: u64,
}

static VERSION: AtomicU64 = AtomicU64::new(0);

impl<T> State<T>
where
    T: Display + Clone,
{
    /// Creates a new [`State`] for a [`ComboBox`] with the given list of options.
    pub fn new(options: Vec<T>) -> Self {
        Self {
            options,
            version: VERSION.fetch_add(1, atomic::Ordering::Relaxed),
        }
    }

    /// Returns the options of the [`State`].
    ///
    /// These are the options provided when the [`State`]
    /// was constructed with [`State::new`].
    pub fn options(&self) -> &[T] {
        &self.options
    }

    /// Pushes a new option to the [`State`].
    pub fn push(&mut self, new_option: T) {
        self.options.push(new_option);
        self.version = VERSION.fetch_add(1, atomic::Ordering::Relaxed);
    }

    /// Returns ownership of the options of the [`State`].
    pub fn into_options(self) -> Vec<T> {
        self.options
    }
}

impl<T> Default for State<T>
where
    T: Display + Clone,
{
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

struct Internal<T, R: text::Renderer> {
    editor: Editor<R>,
    menu: menu::State,
    hovered_option: Option<usize>,
    new_selection: Option<T>,
    option_matchers: Vec<String>,
    filtered_options: Vec<T>,
    version: u64,
}

impl<T: Display + Clone, R: text::Renderer> Internal<T, R> {
    fn filter(&mut self, options: &[T]) {
        self.option_matchers = build_matchers(options);
        self.filtered_options = search(options, &self.option_matchers, &self.editor.value)
            .cloned()
            .collect();
    }
}

struct Editor<R: text::Renderer> {
    input: text::Input<R>,
    value: String,
    selection: Option<String>,
}

impl<T, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for ComboBox<'_, T, Message, Theme, Renderer>
where
    T: Display + Clone + 'static,
    Message: Clone,
    Theme: Catalog,
    Renderer: text::Renderer + 'static,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Fit,
        }
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<Internal<T, Renderer>>();

        state.editor.input.layout(
            renderer,
            limits,
            input::Layout {
                width: self.width,
                height: Length::Fit,
                padding: self.padding,
                placeholder: &self.placeholder,
                font: self.font,
                size: self.size,
                line_height: self.line_height,
                alignment: text::Alignment::Default,
                multiline: None,
            },
        )
    }

    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<Internal<T, Renderer>>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(Internal::<T, Renderer> {
            editor: Editor {
                input: text::Input::new(),
                value: String::new(),
                selection: None,
            },
            menu: menu::State::new(),
            filtered_options: Vec::new(),
            option_matchers: Vec::new(),
            hovered_option: Some(0),
            new_selection: None,
            version: 0,
        })
    }

    fn diff(&mut self, tree: &mut widget::Tree) {
        let state = tree.state.downcast_mut::<Internal<T, Renderer>>();

        if state.version != self.state.version
            || state.editor.selection.as_deref() != Some(&self.selection)
        {
            state.editor.input.overwrite(&self.selection);
            state.editor.selection = Some(self.selection.clone());
            state.editor.value = self.selection.clone();
            state.filter(&self.state.options);

            state.version = self.state.version;
        }
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let internal = tree.state.downcast_mut::<Internal<T, Renderer>>();

        let was_focused = internal.editor.input.is_focused();

        let edit = internal.editor.input.update::<Message>(
            event,
            layout.bounds(),
            cursor,
            shell,
            editor::Binding::from_key_press,
        );

        if edit.is_some() {
            let value = internal.editor.input.value();

            if let Some(on_input) = &self.on_input {
                shell.publish(on_input(value.clone()));
            }

            internal.editor.value = value;
            internal.filter(&self.state.options);
        }

        let is_focused = internal.editor.input.is_focused();

        if is_focused {
            if !was_focused {
                internal.editor.input.overwrite("");
                internal.editor.value.clear();
                internal.filtered_options = self.state.options.clone();

                if let Some(on_option_hovered) = &mut self.on_option_hovered {
                    let hovered_option = internal.hovered_option.unwrap_or(0);

                    if let Some(option) = internal.filtered_options.get(hovered_option) {
                        shell.publish(on_option_hovered(option.clone()));
                    }
                }
            }

            if let Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(named_key),
                modifiers,
                ..
            }) = event
            {
                match (named_key, modifiers.shift()) {
                    (key::Named::Enter, _) => {
                        if let Some(index) = &internal.hovered_option
                            && let Some(option) = internal.filtered_options.get(*index)
                        {
                            internal.new_selection = Some(option.clone());
                        }

                        shell.capture_event();
                        shell.request_redraw();
                    }
                    (key::Named::ArrowUp, _) | (key::Named::Tab, true) => {
                        if let Some(index) = &mut internal.hovered_option {
                            if *index == 0 {
                                *index = internal.filtered_options.len().saturating_sub(1);
                            } else {
                                *index = index.saturating_sub(1);
                            }
                        } else {
                            internal.hovered_option = Some(0);
                        }

                        if let Some(on_option_hovered) = &mut self.on_option_hovered
                            && let Some(option) = internal
                                .hovered_option
                                .and_then(|index| internal.filtered_options.get(index))
                        {
                            // Notify the selection
                            shell.publish((on_option_hovered)(option.clone()));
                        }

                        shell.capture_event();
                        shell.request_redraw();
                    }
                    (key::Named::ArrowDown, _) | (key::Named::Tab, false) => {
                        if let Some(index) = &mut internal.hovered_option {
                            if *index >= internal.filtered_options.len().saturating_sub(1) {
                                *index = 0;
                            } else {
                                *index = index
                                    .saturating_add(1)
                                    .min(internal.filtered_options.len().saturating_sub(1));
                            }
                        } else {
                            internal.hovered_option = Some(0);
                        }

                        if let Some(on_option_hovered) = &mut self.on_option_hovered
                            && let Some(option) = internal
                                .hovered_option
                                .and_then(|index| internal.filtered_options.get(index))
                        {
                            // Notify the selection
                            shell.publish((on_option_hovered)(option.clone()));
                        }

                        shell.capture_event();
                        shell.request_redraw();
                    }
                    _ => {}
                }
            }
        }

        // If the overlay menu has selected something
        if let Some(selection) = internal.new_selection.take() {
            // Clear the value and reset the options and menu
            internal.menu = menu::State::default();

            internal.editor.input.overwrite(&selection.to_string());
            internal.editor.input.unfocus();
            internal.editor.value = String::new();

            internal.filter(&self.state.options);

            // Notify the selection
            shell.publish((self.on_selected)(selection));
        }

        if was_focused != is_focused {
            if is_focused {
                if let Some(on_open) = self.on_open.take() {
                    shell.publish(on_open);
                }
            } else if let Some(on_close) = self.on_close.take() {
                internal.editor.input.overwrite(&self.selection);
                shell.publish(on_close);
            }
        }

        let status = if internal.editor.input.is_focused() {
            text_input::Status::Focused {
                is_hovered: cursor.is_over(layout.bounds()),
            }
        } else if cursor.is_over(layout.bounds()) {
            text_input::Status::Hovered
        } else {
            text_input::Status::Active
        };

        if let Event::Window(window::Event::RedrawRequested(_now)) = event {
            self.last_status = Some(status);

            shell.request_input_method(
                &internal
                    .editor
                    .input
                    .input_method(layout.bounds().shrink(self.padding).position()),
            );
        } else if self
            .last_status
            .is_some_and(|last_status| status != last_status)
        {
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Text
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let internal = tree.state.downcast_ref::<Internal<T, Renderer>>();

        let bounds = layout.bounds();
        let style = text_input::Catalog::style(
            theme,
            &self.input_class,
            self.last_status.unwrap_or(text_input::Status::Disabled),
        );

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: style.border,
                ..renderer::Quad::default()
            },
            style.background,
        );

        internal.editor.input.draw(
            renderer,
            bounds,
            *viewport,
            input::Style {
                value: style.value,
                selection: style.selection,
                placeholder: style.placeholder,
            },
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let internal = tree.state.downcast_mut::<Internal<T, Renderer>>();
        let is_focused = internal.editor.input.is_focused();

        if is_focused {
            let Internal {
                menu,
                filtered_options,
                hovered_option,
                new_selection,
                ..
            } = tree.state.downcast_mut::<Internal<T, Renderer>>();

            if filtered_options.is_empty() {
                None
            } else {
                let bounds = layout.bounds();

                let mut menu = menu::Menu::new(
                    menu,
                    filtered_options,
                    hovered_option,
                    &T::to_string,
                    |selection| {
                        *new_selection = Some(selection.clone());

                        (self.on_selected)(selection)
                    },
                    self.on_option_hovered.as_deref(),
                    &self.menu_class,
                )
                .width(bounds.width)
                .padding(self.padding)
                .shaping(self.shaping)
                .ellipsis(self.ellipsis);

                if let Some(font) = self.font {
                    menu = menu.font(font);
                }

                if let Some(size) = self.size {
                    menu = menu.text_size(size);
                }

                Some(menu.overlay(
                    layout.position() + translation,
                    *viewport,
                    bounds.height,
                    self.menu_height,
                ))
            }
        } else {
            None
        }
    }

    fn operate(
        &mut self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        let state = tree.state.downcast_mut::<Internal<T, Renderer>>();
        let bounds = layout.bounds();

        operation.focusable(self.id.as_ref(), bounds, &mut state.editor.input);
        operation.text_input(self.id.as_ref(), bounds, &mut state.editor.input);
    }
}

impl<'a, T, Message, Theme, Renderer> From<ComboBox<'a, T, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    T: Display + Clone + 'static,
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: text::Renderer + 'static,
{
    fn from(combo_box: ComboBox<'a, T, Message, Theme, Renderer>) -> Self {
        Self::new(combo_box)
    }
}

/// The theme catalog of a [`ComboBox`].
pub trait Catalog: text_input::Catalog + menu::Catalog {
    /// The default class for the text input of the [`ComboBox`].
    fn default_input<'a>() -> <Self as text_input::Catalog>::Class<'a> {
        <Self as text_input::Catalog>::default()
    }

    /// The default class for the menu of the [`ComboBox`].
    fn default_menu<'a>() -> <Self as menu::Catalog>::Class<'a> {
        <Self as menu::Catalog>::default()
    }
}

impl Catalog for Theme {}

fn search<'a, T, A>(
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
        .zip(option_matchers)
        // Make sure each part of the query is found in the option
        .filter_map(move |(option, matcher)| {
            if query.iter().all(|part| matcher.as_ref().contains(part)) {
                Some(option)
            } else {
                None
            }
        })
}

fn build_matchers<'a, T>(options: impl IntoIterator<Item = T> + 'a) -> Vec<String>
where
    T: Display + 'a,
{
    options.into_iter().map(build_matcher).collect()
}

fn build_matcher<T>(option: T) -> String
where
    T: Display,
{
    let mut matcher = option.to_string();
    matcher.retain(|c| c.is_ascii_alphanumeric());
    matcher.to_lowercase()
}
