//! Create iced applications out of simple functions.
//!
//! You can use this API to create and run iced applications
//! step by step—without coupling your logic to a trait
//! or a specific type.
//!
//! This API is meant to be a more convenient—although less
//! powerful—alternative to the [`Sandbox`] and [`Application`] traits.
//!
//! [`Sandbox`]: crate::Sandbox
//!
//! # Example
//! ```no_run
//! use iced::widget::{button, column, text, Column};
//! use iced::Theme;
//!
//! pub fn main() -> iced::Result {
//!     iced::sandbox(update, view)
//!         .title("A counter")
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
use crate::application::{self, Application};
use crate::executor::{self, Executor};
use crate::window;
use crate::{Command, Element, Font, Result, Settings, Subscription};

use std::borrow::Cow;

/// Creates the most basic kind of [`Program`] from some update and view logic.
///
/// # Example
/// ```no_run
/// use iced::widget::{button, column, text, Column};
///
/// pub fn main() -> iced::Result {
///     iced::sandbox(update, view).title("A counter").run()
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
pub fn sandbox<State, Message>(
    update: impl Fn(&mut State, Message),
    view: impl for<'a> self::View<'a, State, Message>,
) -> Program<
    impl Definition<State = State, Message = Message, Theme = crate::Theme>,
>
where
    State: Default + 'static,
    Message: Send + std::fmt::Debug,
{
    use std::marker::PhantomData;

    struct Sandbox<State, Message, Update, View> {
        update: Update,
        view: View,
        _state: PhantomData<State>,
        _message: PhantomData<Message>,
    }

    impl<State, Message, Update, View> Definition
        for Sandbox<State, Message, Update, View>
    where
        State: Default + 'static,
        Message: Send + std::fmt::Debug,
        Update: Fn(&mut State, Message),
        View: for<'a> self::View<'a, State, Message>,
    {
        type State = State;
        type Message = Message;
        type Theme = crate::Theme;
        type Executor = iced_futures::backend::null::Executor;

        fn build(&self) -> (Self::State, Command<Self::Message>) {
            (State::default(), Command::none())
        }

        fn update(
            &self,
            state: &mut Self::State,
            message: Self::Message,
        ) -> Command<Self::Message> {
            (self.update)(state, message);

            Command::none()
        }

        fn view<'a>(
            &self,
            state: &'a Self::State,
        ) -> Element<'a, Self::Message, Self::Theme> {
            self.view.view(state).into()
        }
    }

    Program {
        raw: Sandbox {
            update,
            view,
            _state: PhantomData,
            _message: PhantomData,
        },
        settings: Settings::default(),
    }
}

/// Creates a [`Program`] that can leverage the [`Command`] API for
/// concurrent operations.
pub fn application<State, Message>(
    title: impl Title<State>,
    update: impl Fn(&mut State, Message) -> Command<Message>,
    view: impl for<'a> self::View<'a, State, Message>,
) -> Program<
    impl Definition<State = State, Message = Message, Theme = crate::Theme>,
>
where
    State: Default + 'static,
    Message: Send + std::fmt::Debug,
{
    use std::marker::PhantomData;

    struct Application<State, Message, Update, View> {
        update: Update,
        view: View,
        _state: PhantomData<State>,
        _message: PhantomData<Message>,
    }

    impl<State, Message, Update, View> Definition
        for Application<State, Message, Update, View>
    where
        State: Default,
        Message: Send + std::fmt::Debug,
        Update: Fn(&mut State, Message) -> Command<Message>,
        View: for<'a> self::View<'a, State, Message>,
    {
        type State = State;
        type Message = Message;
        type Theme = crate::Theme;
        type Executor = executor::Default;

        fn build(&self) -> (Self::State, Command<Self::Message>) {
            (Self::State::default(), Command::none())
        }

        fn update(
            &self,
            state: &mut Self::State,
            message: Self::Message,
        ) -> Command<Self::Message> {
            (self.update)(state, message)
        }

        fn view<'a>(
            &self,
            state: &'a Self::State,
        ) -> Element<'a, Self::Message, Self::Theme> {
            self.view.view(state).into()
        }
    }

    Program {
        raw: Application {
            update,
            view,
            _state: PhantomData,
            _message: PhantomData,
        },
        settings: Settings::default(),
    }
    .title(title)
}

/// A fully functioning and configured iced application.
///
/// It can be [`run`]!
///
/// Create one with either the [`sandbox`] or [`application`] helpers.
///
/// [`run`]: Program::run
/// [`application`]: self::application()
#[derive(Debug)]
pub struct Program<P: Definition> {
    raw: P,
    settings: Settings,
}

impl<P: Definition> Program<P> {
    /// Runs the [`Program`].
    pub fn run(self) -> Result
    where
        Self: 'static,
    {
        struct Instance<P: Definition> {
            program: P,
            state: P::State,
        }

        impl<P: Definition> Application for Instance<P> {
            type Message = P::Message;
            type Theme = P::Theme;
            type Flags = P;
            type Executor = P::Executor;

            fn new(program: Self::Flags) -> (Self, Command<Self::Message>) {
                let (state, command) = P::build(&program);

                (Self { program, state }, command)
            }

            fn title(&self) -> String {
                self.program.title(&self.state)
            }

            fn update(
                &mut self,
                message: Self::Message,
            ) -> Command<Self::Message> {
                self.program.update(&mut self.state, message)
            }

            fn view(
                &self,
            ) -> crate::Element<'_, Self::Message, Self::Theme, crate::Renderer>
            {
                self.program.view(&self.state)
            }

            fn theme(&self) -> Self::Theme {
                self.program.theme(&self.state)
            }

            fn subscription(&self) -> Subscription<Self::Message> {
                self.program.subscription(&self.state)
            }
        }

        let Self { raw, settings } = self;

        Instance::run(Settings {
            flags: raw,
            id: settings.id,
            window: settings.window,
            fonts: settings.fonts,
            default_font: settings.default_font,
            default_text_size: settings.default_text_size,
            antialiasing: settings.antialiasing,
        })
    }

    /// Sets the [`Settings`] that will be used to run the [`Program`].
    pub fn settings(self, settings: Settings) -> Self {
        Self { settings, ..self }
    }

    /// Toggles the [`Settings::antialiasing`] to `true` for the [`Program`].
    pub fn antialiased(self) -> Self {
        Self {
            settings: Settings {
                antialiasing: true,
                ..self.settings
            },
            ..self
        }
    }

    /// Sets the default [`Font`] of the [`Program`].
    pub fn default_font(self, default_font: Font) -> Self {
        Self {
            settings: Settings {
                default_font,
                ..self.settings
            },
            ..self
        }
    }

    /// Sets the fonts that will be loaded at the start of the [`Program`].
    pub fn fonts(
        self,
        fonts: impl IntoIterator<Item = Cow<'static, [u8]>>,
    ) -> Self {
        Self {
            settings: Settings {
                fonts: fonts.into_iter().collect(),
                ..self.settings
            },
            ..self
        }
    }

    /// Sets the [`window::Settings::position`] to [`window::Position::Centered`] in the [`Program`].
    pub fn centered(self) -> Self {
        Self {
            settings: Settings {
                window: window::Settings {
                    position: window::Position::Centered,
                    ..self.settings.window
                },
                ..self.settings
            },
            ..self
        }
    }

    /// Sets the [`window::Settings::exit_on_close_request`] to `false` in the [`Program`].
    pub fn ignore_close_request(self) -> Self {
        Self {
            settings: Settings {
                window: window::Settings {
                    exit_on_close_request: false,
                    ..self.settings.window
                },
                ..self.settings
            },
            ..self
        }
    }

    /// Sets the [`Title`] of the [`Program`].
    pub fn title(
        self,
        title: impl Title<P::State>,
    ) -> Program<
        impl Definition<State = P::State, Message = P::Message, Theme = P::Theme>,
    > {
        Program {
            raw: with_title(self.raw, title),
            settings: self.settings,
        }
    }

    /// Runs the [`Command`] produced by the closure at startup.
    pub fn load(
        self,
        f: impl Fn() -> Command<P::Message>,
    ) -> Program<
        impl Definition<State = P::State, Message = P::Message, Theme = P::Theme>,
    > {
        Program {
            raw: with_load(self.raw, f),
            settings: self.settings,
        }
    }

    /// Sets the subscription logic of the [`Program`].
    pub fn subscription(
        self,
        f: impl Fn(&P::State) -> Subscription<P::Message>,
    ) -> Program<
        impl Definition<State = P::State, Message = P::Message, Theme = P::Theme>,
    > {
        Program {
            raw: with_subscription(self.raw, f),
            settings: self.settings,
        }
    }

    /// Sets the theme logic of the [`Program`].
    pub fn theme(
        self,
        f: impl Fn(&P::State) -> P::Theme,
    ) -> Program<
        impl Definition<State = P::State, Message = P::Message, Theme = P::Theme>,
    > {
        Program {
            raw: with_theme(self.raw, f),
            settings: self.settings,
        }
    }
}

/// The internal definition of a [`Program`].
///
/// You should not need to implement this trait directly. Instead, use the
/// helper functions available in the [`program`] module and the [`Program`] struct.
///
/// [`program`]: crate::program
#[allow(missing_docs)]
pub trait Definition: Sized {
    /// The state of the program.
    type State;

    /// The message of the program.
    type Message: Send + std::fmt::Debug;

    /// The theme of the program.
    type Theme: Default + application::DefaultStyle;

    /// The executor of the program.
    type Executor: Executor;

    fn build(&self) -> (Self::State, Command<Self::Message>);

    fn update(
        &self,
        state: &mut Self::State,
        message: Self::Message,
    ) -> Command<Self::Message>;

    fn view<'a>(
        &self,
        state: &'a Self::State,
    ) -> Element<'a, Self::Message, Self::Theme>;

    fn title(&self, _state: &Self::State) -> String {
        String::from("A cool iced application!")
    }

    fn subscription(
        &self,
        _state: &Self::State,
    ) -> Subscription<Self::Message> {
        Subscription::none()
    }

    fn theme(&self, _state: &Self::State) -> Self::Theme {
        Self::Theme::default()
    }
}

fn with_title<P: Definition>(
    program: P,
    title: impl Title<P::State>,
) -> impl Definition<State = P::State, Message = P::Message, Theme = P::Theme> {
    struct WithTitle<P, Title> {
        program: P,
        title: Title,
    }

    impl<P, Title> Definition for WithTitle<P, Title>
    where
        P: Definition,
        Title: self::Title<P::State>,
    {
        type State = P::State;
        type Message = P::Message;
        type Theme = P::Theme;
        type Executor = P::Executor;

        fn build(&self) -> (Self::State, Command<Self::Message>) {
            self.program.build()
        }

        fn title(&self, state: &Self::State) -> String {
            self.title.title(state)
        }

        fn update(
            &self,
            state: &mut Self::State,
            message: Self::Message,
        ) -> Command<Self::Message> {
            self.program.update(state, message)
        }

        fn view<'a>(
            &self,
            state: &'a Self::State,
        ) -> Element<'a, Self::Message, Self::Theme> {
            self.program.view(state)
        }

        fn theme(&self, state: &Self::State) -> Self::Theme {
            self.program.theme(state)
        }

        fn subscription(
            &self,
            state: &Self::State,
        ) -> Subscription<Self::Message> {
            self.program.subscription(state)
        }
    }

    WithTitle { program, title }
}

fn with_load<P: Definition>(
    program: P,
    f: impl Fn() -> Command<P::Message>,
) -> impl Definition<State = P::State, Message = P::Message, Theme = P::Theme> {
    struct WithLoad<P, F> {
        program: P,
        load: F,
    }

    impl<P: Definition, F> Definition for WithLoad<P, F>
    where
        F: Fn() -> Command<P::Message>,
    {
        type State = P::State;
        type Message = P::Message;
        type Theme = P::Theme;
        type Executor = executor::Default;

        fn build(&self) -> (Self::State, Command<Self::Message>) {
            let (state, command) = self.program.build();

            (state, Command::batch([command, (self.load)()]))
        }

        fn update(
            &self,
            state: &mut Self::State,
            message: Self::Message,
        ) -> Command<Self::Message> {
            self.program.update(state, message)
        }

        fn view<'a>(
            &self,
            state: &'a Self::State,
        ) -> Element<'a, Self::Message, Self::Theme> {
            self.program.view(state)
        }

        fn title(&self, state: &Self::State) -> String {
            self.program.title(state)
        }

        fn theme(&self, state: &Self::State) -> Self::Theme {
            self.program.theme(state)
        }

        fn subscription(
            &self,
            state: &Self::State,
        ) -> Subscription<Self::Message> {
            self.program.subscription(state)
        }
    }

    WithLoad { program, load: f }
}

fn with_subscription<P: Definition>(
    program: P,
    f: impl Fn(&P::State) -> Subscription<P::Message>,
) -> impl Definition<State = P::State, Message = P::Message, Theme = P::Theme> {
    struct WithSubscription<P, F> {
        program: P,
        subscription: F,
    }

    impl<P: Definition, F> Definition for WithSubscription<P, F>
    where
        F: Fn(&P::State) -> Subscription<P::Message>,
    {
        type State = P::State;
        type Message = P::Message;
        type Theme = P::Theme;
        type Executor = executor::Default;

        fn subscription(
            &self,
            state: &Self::State,
        ) -> Subscription<Self::Message> {
            (self.subscription)(state)
        }

        fn build(&self) -> (Self::State, Command<Self::Message>) {
            self.program.build()
        }

        fn update(
            &self,
            state: &mut Self::State,
            message: Self::Message,
        ) -> Command<Self::Message> {
            self.program.update(state, message)
        }

        fn view<'a>(
            &self,
            state: &'a Self::State,
        ) -> Element<'a, Self::Message, Self::Theme> {
            self.program.view(state)
        }

        fn title(&self, state: &Self::State) -> String {
            self.program.title(state)
        }

        fn theme(&self, state: &Self::State) -> Self::Theme {
            self.program.theme(state)
        }
    }

    WithSubscription {
        program,
        subscription: f,
    }
}

fn with_theme<P: Definition>(
    program: P,
    f: impl Fn(&P::State) -> P::Theme,
) -> impl Definition<State = P::State, Message = P::Message, Theme = P::Theme> {
    struct WithTheme<P, F> {
        program: P,
        theme: F,
    }

    impl<P: Definition, F> Definition for WithTheme<P, F>
    where
        F: Fn(&P::State) -> P::Theme,
    {
        type State = P::State;
        type Message = P::Message;
        type Theme = P::Theme;
        type Executor = P::Executor;

        fn theme(&self, state: &Self::State) -> Self::Theme {
            (self.theme)(state)
        }

        fn build(&self) -> (Self::State, Command<Self::Message>) {
            self.program.build()
        }

        fn title(&self, state: &Self::State) -> String {
            self.program.title(state)
        }

        fn update(
            &self,
            state: &mut Self::State,
            message: Self::Message,
        ) -> Command<Self::Message> {
            self.program.update(state, message)
        }

        fn view<'a>(
            &self,
            state: &'a Self::State,
        ) -> Element<'a, Self::Message, Self::Theme> {
            self.program.view(state)
        }

        fn subscription(
            &self,
            state: &Self::State,
        ) -> Subscription<Self::Message> {
            self.program.subscription(state)
        }
    }

    WithTheme { program, theme: f }
}

/// The title logic of some [`Program`].
///
/// This trait is implemented both for `&static str` and
/// any closure `Fn(&State) -> String`.
///
/// You can use any of these in [`Program::title`].
pub trait Title<State> {
    /// Produces the title of the [`Program`].
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

/// The view logic of some [`Program`].
///
/// This trait allows [`sandbox`] and [`application`] to
/// take any closure that returns any `Into<Element<'_, Message>>`.
///
/// [`application`]: self::application()
pub trait View<'a, State, Message> {
    /// The widget returned by the view logic.
    type Widget: Into<Element<'a, Message>>;

    /// Produces the widget of the [`Program`].
    fn view(&self, state: &'a State) -> Self::Widget;
}

impl<'a, T, State, Message, Widget> View<'a, State, Message> for T
where
    T: Fn(&'a State) -> Widget,
    State: 'static,
    Widget: Into<Element<'a, Message>>,
{
    type Widget = Widget;

    fn view(&self, state: &'a State) -> Self::Widget {
        self(state)
    }
}
