use crate::theme::{self, Theme};
use crate::{Application, Command, Element, Error, Settings, Subscription};

/// A sandboxed [`Application`].
///
/// If you are a just getting started with the library, this trait offers a
/// simpler interface than [`Application`].
///
/// Unlike an [`Application`], a [`Sandbox`] cannot run any asynchronous
/// actions or be initialized with some external flags. However, both traits
/// are very similar and upgrading from a [`Sandbox`] is very straightforward.
///
/// Therefore, it is recommended to always start by implementing this trait and
/// upgrade only once necessary.
///
/// # Examples
/// [The repository has a bunch of examples] that use the [`Sandbox`] trait:
///
/// - [`bezier_tool`], a Paint-like tool for drawing Bézier curves using the
/// [`Canvas widget`].
/// - [`counter`], the classic counter example explained in [the overview].
/// - [`custom_widget`], a demonstration of how to build a custom widget that
/// draws a circle.
/// - [`geometry`], a custom widget showcasing how to draw geometry with the
/// `Mesh2D` primitive in [`iced_wgpu`].
/// - [`pane_grid`], a grid of panes that can be split, resized, and
/// reorganized.
/// - [`progress_bar`], a simple progress bar that can be filled by using a
/// slider.
/// - [`styling`], an example showcasing custom styling with a light and dark
/// theme.
/// - [`svg`], an application that renders the [Ghostscript Tiger] by leveraging
/// the [`Svg` widget].
/// - [`tour`], a simple UI tour that can run both on native platforms and the
/// web!
///
/// [The repository has a bunch of examples]: https://github.com/iced-rs/iced/tree/0.10/examples
/// [`bezier_tool`]: https://github.com/iced-rs/iced/tree/0.10/examples/bezier_tool
/// [`counter`]: https://github.com/iced-rs/iced/tree/0.10/examples/counter
/// [`custom_widget`]: https://github.com/iced-rs/iced/tree/0.10/examples/custom_widget
/// [`geometry`]: https://github.com/iced-rs/iced/tree/0.10/examples/geometry
/// [`pane_grid`]: https://github.com/iced-rs/iced/tree/0.10/examples/pane_grid
/// [`progress_bar`]: https://github.com/iced-rs/iced/tree/0.10/examples/progress_bar
/// [`styling`]: https://github.com/iced-rs/iced/tree/0.10/examples/styling
/// [`svg`]: https://github.com/iced-rs/iced/tree/0.10/examples/svg
/// [`tour`]: https://github.com/iced-rs/iced/tree/0.10/examples/tour
/// [`Canvas widget`]: crate::widget::Canvas
/// [the overview]: index.html#overview
/// [`iced_wgpu`]: https://github.com/iced-rs/iced/tree/0.10/wgpu
/// [`Svg` widget]: crate::widget::Svg
/// [Ghostscript Tiger]: https://commons.wikimedia.org/wiki/File:Ghostscript_Tiger.svg
///
/// ## A simple "Hello, world!"
///
/// If you just want to get started, here is a simple [`Sandbox`] that
/// says "Hello, world!":
///
/// ```no_run
/// use iced::{Element, Sandbox, Settings};
///
/// pub fn main() -> iced::Result {
///     Hello::run(Settings::default())
/// }
///
/// struct Hello;
///
/// impl Sandbox for Hello {
///     type Message = ();
///
///     fn new() -> Hello {
///         Hello
///     }
///
///     fn title(&self) -> String {
///         String::from("A cool application")
///     }
///
///     fn update(&mut self, _message: Self::Message) {
///         // This application has no interactions
///     }
///
///     fn view(&self) -> Element<Self::Message> {
///         "Hello, world!".into()
///     }
/// }
/// ```
pub trait Sandbox {
    /// The type of __messages__ your [`Sandbox`] will produce.
    type Message: std::fmt::Debug + Send;

    /// Initializes the [`Sandbox`].
    ///
    /// Here is where you should return the initial state of your app.
    fn new() -> Self;

    /// Returns the current title of the [`Sandbox`].
    ///
    /// This title can be dynamic! The runtime will automatically update the
    /// title of your application when necessary.
    fn title(&self) -> String;

    /// Handles a __message__ and updates the state of the [`Sandbox`].
    ///
    /// This is where you define your __update logic__. All the __messages__,
    /// produced by user interactions, will be handled by this method.
    fn update(&mut self, message: Self::Message);

    /// Returns the widgets to display in the [`Sandbox`].
    ///
    /// These widgets can produce __messages__ based on user interaction.
    fn view(&self) -> Element<'_, Self::Message>;

    /// Returns the current [`Theme`] of the [`Sandbox`].
    ///
    /// If you want to use your own custom theme type, you will have to use an
    /// [`Application`].
    ///
    /// By default, it returns [`Theme::default`].
    fn theme(&self) -> Theme {
        Theme::default()
    }

    /// Returns the current style variant of [`theme::Application`].
    ///
    /// By default, it returns [`theme::Application::default`].
    fn style(&self) -> theme::Application {
        theme::Application::default()
    }

    /// Returns the scale factor of the [`Sandbox`].
    ///
    /// It can be used to dynamically control the size of the UI at runtime
    /// (i.e. zooming).
    ///
    /// For instance, a scale factor of `2.0` will make widgets twice as big,
    /// while a scale factor of `0.5` will shrink them to half their size.
    ///
    /// By default, it returns `1.0`.
    fn scale_factor(&self) -> f64 {
        1.0
    }

    /// Runs the [`Sandbox`].
    ///
    /// On native platforms, this method will take control of the current thread
    /// and __will NOT return__.
    ///
    /// It should probably be that last thing you call in your `main` function.
    fn run(settings: Settings<()>) -> Result<(), Error>
    where
        Self: 'static + Sized,
    {
        <Self as Application>::run(settings)
    }
}

impl<T> Application for T
where
    T: Sandbox,
{
    type Executor = iced_futures::backend::null::Executor;
    type Flags = ();
    type Message = T::Message;
    type Theme = Theme;

    fn new(_flags: ()) -> (Self, Command<T::Message>) {
        (T::new(), Command::none())
    }

    fn title(&self) -> String {
        T::title(self)
    }

    fn update(&mut self, message: T::Message) -> Command<T::Message> {
        T::update(self, message);

        Command::none()
    }

    fn view(&self) -> Element<'_, T::Message> {
        T::view(self)
    }

    fn theme(&self) -> Self::Theme {
        T::theme(self)
    }

    fn style(&self) -> theme::Application {
        T::style(self)
    }

    fn subscription(&self) -> Subscription<T::Message> {
        Subscription::none()
    }

    fn scale_factor(&self) -> f64 {
        T::scale_factor(self)
    }
}
