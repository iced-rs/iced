//! Create and run iced applications step by step.
//!
//! # Example
//! ```no_run
//! use iced::widget::{button, column, text, Column};
//! use iced::Theme;
//!
//! pub fn main() -> iced::Result {
//!     iced::application("A counter", update, view)
//!         .theme(|_| Theme::Dark)
//!         .centered()
//!         .run()
//! }
//!
//! #[derive(Debug, Clone)]
//! enum Message {
//!     Increment,
//! }
//!
//! fn update(value: &mut u64, message: Message) {
//!     match message {
//!         Message::Increment => *value += 1,
//!     }
//! }
//!
//! fn view(value: &u64) -> Column<Message> {
//!     column![
//!         text(value),
//!         button("+").on_press(Message::Increment),
//!     ]
//! }
//! ```
use crate::program::{self, Program};
use crate::window;
use crate::{Element, Font, Result, Settings, Size, Subscription, Task};

use std::borrow::Cow;

pub use crate::shell::program::{Appearance, DefaultStyle};

/// Creates an iced [`Application`] given its title, update, and view logic.
///
/// # Example
/// ```no_run
/// use iced::widget::{button, column, text, Column};
///
/// pub fn main() -> iced::Result {
///     iced::application("A counter", update, view).run()
/// }
///
/// #[derive(Debug, Clone)]
/// enum Message {
///     Increment,
/// }
///
/// fn update(value: &mut u64, message: Message) {
///     match message {
///         Message::Increment => *value += 1,
///     }
/// }
///
/// fn view(value: &u64) -> Column<Message> {
///     column![
///         text(value),
///         button("+").on_press(Message::Increment),
///     ]
/// }
/// ```
pub fn application<State, Message, Theme, Renderer>(
    title: impl Title<State>,
    update: impl Update<State, Message>,
    view: impl for<'a> self::View<'a, State, Message, Theme, Renderer>,
) -> Application<impl Program<State = State, Message = Message, Theme = Theme>>
where
    State: 'static,
    Message: Send + std::fmt::Debug + 'static,
    Theme: Default + DefaultStyle,
    Renderer: program::Renderer,
{
    use std::marker::PhantomData;

    struct Instance<State, Message, Theme, Renderer, Update, View> {
        update: Update,
        view: View,
        _state: PhantomData<State>,
        _message: PhantomData<Message>,
        _theme: PhantomData<Theme>,
        _renderer: PhantomData<Renderer>,
    }

    impl<State, Message, Theme, Renderer, Update, View> Program
        for Instance<State, Message, Theme, Renderer, Update, View>
    where
        Message: Send + std::fmt::Debug + 'static,
        Theme: Default + DefaultStyle,
        Renderer: program::Renderer,
        Update: self::Update<State, Message>,
        View: for<'a> self::View<'a, State, Message, Theme, Renderer>,
    {
        type State = State;
        type Message = Message;
        type Theme = Theme;
        type Renderer = Renderer;
        type Executor = iced_futures::backend::default::Executor;

        fn load(&self) -> Task<Self::Message> {
            Task::none()
        }

        fn update(
            &self,
            state: &mut Self::State,
            message: Self::Message,
        ) -> Task<Self::Message> {
            self.update.update(state, message).into()
        }

        fn view<'a>(
            &self,
            state: &'a Self::State,
            window: window::Id,
        ) -> Element<'a, Self::Message, Self::Theme, Self::Renderer> {
            self.view.view(state, Some(window)).into()
        }
    }

    Application {
        raw: Instance {
            update,
            view,
            _state: PhantomData,
            _message: PhantomData,
            _theme: PhantomData,
            _renderer: PhantomData,
        },
        settings: Settings::default(),
        windows: vec![window::Settings::default()],
    }
    .title(title)
}

/// The underlying definition and configuration of an iced application.
///
/// You can use this API to create and run iced applications
/// step by step—without coupling your logic to a trait
/// or a specific type.
///
/// You can create an [`Application`] with the [`application`] helper.
#[derive(Debug)]
pub struct Application<P: Program> {
    raw: P,
    settings: Settings,
    windows: Vec<window::Settings>,
}

impl<P: Program> Application<P> {
    /// Runs the [`Application`].
    ///
    /// The state of the [`Application`] must implement [`Default`].
    /// If your state does not implement [`Default`], use [`run_with`]
    /// instead.
    ///
    /// [`run_with`]: Self::run_with
    pub fn run(self) -> Result
    where
        Self: 'static,
        P::State: Default,
    {
        self.run_with(P::State::default)
    }

    /// Runs the [`Application`] with a closure that creates the initial state.
    pub fn run_with<I>(self, initialize: I) -> Result
    where
        Self: 'static,
        I: Fn() -> P::State + Clone + 'static,
    {
        self.raw
            .run_with(self.settings, self.windows, initialize)
    }

    /// Sets the [`Settings`] that will be used to run the [`Application`].
    pub fn settings(self, settings: Settings) -> Self {
        Self { settings, ..self }
    }

    /// Sets the [`Settings::antialiasing`] of the [`Application`].
    pub fn antialiasing(self, antialiasing: bool) -> Self {
        Self {
            settings: Settings {
                antialiasing,
                ..self.settings
            },
            ..self
        }
    }

    /// Sets the default [`Font`] of the [`Application`].
    pub fn default_font(self, default_font: Font) -> Self {
        Self {
            settings: Settings {
                default_font,
                ..self.settings
            },
            ..self
        }
    }

    /// Adds a font to the list of fonts that will be loaded at the start of the [`Application`].
    pub fn font(mut self, font: impl Into<Cow<'static, [u8]>>) -> Self {
        self.settings.fonts.push(font.into());
        self
    }

    /// Adds a [`window::Settings`] to the [`Application`].
    pub fn without_windows(mut self) -> Self {
        self.windows.clear();
        self
    }

    /// Adds a [`window::Settings`] to the [`Application`].
    pub fn add_window(mut self, window: window::Settings) -> Self {
        self.windows.push(window);
        self
    }

    /// Sets the [`window::Settings::position`] to [`window::Position::Centered`] in the [`Application`].
    pub fn centered(mut self) -> Self {
        for window in &mut self.windows {
            window.position = window::Position::Centered;
        }
        self
    }

    /// Sets the [`window::Settings::exit_on_close_request`] of the [`Application`].
    pub fn exit_on_close_request(mut self, exit_on_close_request: bool) -> Self {
        for window in &mut self.windows {
            window.exit_on_close_request = exit_on_close_request;
        }
        self
    }

    /// Sets the [`window::Settings::size`] of the [`Application`].
    pub fn window_size(mut self, size: impl Into<Size> + Clone) -> Self {
        for window in &mut self.windows {
            window.size = size.clone().into();
        }
        self
    }

    /// Sets the [`window::Settings::transparent`] of the [`Application`].
    pub fn transparent(mut self, transparent: bool) -> Self {
        for window in &mut self.windows {
            window.transparent = transparent;
        }
        self
    }

    /// Sets the [`window::Settings::resizable`] of the [`Application`].
    pub fn resizable(mut self, resizable: bool) -> Self {
        for window in &mut self.windows {
            window.resizable = resizable;
        }
        self
    }

    /// Sets the [`window::Settings::decorations`] of the [`Application`].
    pub fn decorations(mut self, decorations: bool) -> Self {
        for window in &mut self.windows {
            window.decorations = decorations;
        }
        self
    }

    /// Sets the [`window::Settings::position`] of the [`Application`].
    pub fn position(mut self, position: window::Position) -> Self {
        for window in &mut self.windows {
            window.position = position;
        }
        self
    }

    /// Sets the [`window::Settings::level`] of the [`Application`].
    pub fn level(mut self, level: window::Level) -> Self {
        for window in &mut self.windows {
            window.level = level;
        }
        self
    }

    /// Sets the [`Title`] of the [`Application`].
    pub(crate) fn title(
        self,
        title: impl Title<P::State>,
    ) -> Application<
        impl Program<State = P::State, Message = P::Message, Theme = P::Theme>,
    > {
        Application {
            raw: program::with_title(self.raw, move |state, _window| {
                title.title(state)
            }),
            settings: self.settings,
            windows: self.windows,
        }
    }

    /// Runs the [`Task`] produced by the closure at startup.
    pub fn load(
        self,
        f: impl Fn() -> Task<P::Message>,
    ) -> Application<
        impl Program<State = P::State, Message = P::Message, Theme = P::Theme>,
    > {
        Application {
            raw: program::with_load(self.raw, f),
            settings: self.settings,
            windows: self.windows,
        }
    }

    /// Sets the subscription logic of the [`Application`].
    pub fn subscription(
        self,
        f: impl Fn(&P::State) -> Subscription<P::Message>,
    ) -> Application<
        impl Program<State = P::State, Message = P::Message, Theme = P::Theme>,
    > {
        Application {
            raw: program::with_subscription(self.raw, f),
            settings: self.settings,
            windows: self.windows,
        }
    }

    /// Sets the theme logic of the [`Application`].
    pub fn theme(
        self,
        f: impl Fn(&P::State) -> P::Theme,
    ) -> Application<
        impl Program<State = P::State, Message = P::Message, Theme = P::Theme>,
    > {
        Application {
            raw: program::with_theme(self.raw, move |state, _window| f(state)),
            settings: self.settings,
            windows: self.windows,
        }
    }

    /// Sets the style logic of the [`Application`].
    pub fn style(
        self,
        f: impl Fn(&P::State, &P::Theme) -> Appearance,
    ) -> Application<
        impl Program<State = P::State, Message = P::Message, Theme = P::Theme>,
    > {
        Application {
            raw: program::with_style(self.raw, f),
            settings: self.settings,
            windows: self.windows,
        }
    }

    /// Sets the scale factor of the [`Application`].
    pub fn scale_factor(
        self,
        f: impl Fn(&P::State) -> f64,
    ) -> Application<
        impl Program<State = P::State, Message = P::Message, Theme = P::Theme>,
    > {
        Application {
            raw: program::with_scale_factor(self.raw, move |state, _window| {
                f(state)
            }),
            settings: self.settings,
            windows: self.windows,
        }
    }
}

/// The title logic of some [`Application`].
///
/// This trait is implemented both for `&static str` and
/// any closure `Fn(&State) -> String`.
///
/// This trait allows the [`application`] builder to take any of them.
pub trait Title<State> {
    /// Produces the title of the [`Application`].
    fn title(&self, state: &State) -> String;
}

impl<State> Title<State> for &'static str {
    fn title(&self, _state: &State) -> String {
        self.to_string()
    }
}

impl<T, State> Title<State> for T
where
    T: Fn(&State) -> String,
{
    fn title(&self, state: &State) -> String {
        self(state)
    }
}

/// The update logic of some [`Application`].
///
/// This trait allows the [`application`] builder to take any closure that
/// returns any `Into<Task<Message>>`.
pub trait Update<State, Message> {
    /// Processes the message and updates the state of the [`Application`].
    fn update(
        &self,
        state: &mut State,
        message: Message,
    ) -> impl Into<Task<Message>>;
}

impl<T, State, Message, C> Update<State, Message> for T
where
    T: Fn(&mut State, Message) -> C,
    C: Into<Task<Message>>,
{
    fn update(
        &self,
        state: &mut State,
        message: Message,
    ) -> impl Into<Task<Message>> {
        self(state, message)
    }
}

/// The view logic of some [`Application`].
///
/// This trait allows the [`application`] builder to take any closure that
/// returns any `Into<Element<'_, Message>>`.
pub trait View<'a, State, Message, Theme, Renderer> {
    /// Produces the widget of the [`Application`].
    fn view(
        &self,
        state: &'a State,
        window: Option<window::Id>,
    ) -> impl Into<Element<'a, Message, Theme, Renderer>>;
}

impl<'a, T, State, Message, Theme, Renderer, Widget>
    View<'a, State, Message, Theme, Renderer> for T
where
    T: Fn(&'a State, Option<window::Id>) -> Widget,
    State: 'static,
    Widget: Into<Element<'a, Message, Theme, Renderer>>,
{
    fn view(
        &self,
        state: &'a State,
        window: Option<window::Id>,
    ) -> impl Into<Element<'a, Message, Theme, Renderer>> {
        self(state, window)
    }
}
