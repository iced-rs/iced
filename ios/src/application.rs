use crate::{Runtime, Command, Element, Executor, Proxy, Subscription};
use winit::{
    event::{self, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    platform::ios::{EventLoopExtIOS, WindowBuilderExtIOS, WindowExtIOS},
    window::WindowBuilder,
};
use uikit_sys::{
    self,
    //CGRect,
    //CGPoint,
    //CGSize,
    id,
    IUIColor,
    //IUISwitch,
    //IUIView,
    UIColor,
    //UIView_UIViewGeometry,
    //UISwitch,
    UIView,
    //UIView_UIViewHierarchy,
    //UIView,
    //UIViewController,
    UIView_UIViewRendering,
};

pub trait Application: Sized {
    type Executor: Executor;

    /// The type of __messages__ your [`Application`] will produce.
    ///
    /// [`Application`]: trait.Application.html
    type Message: std::fmt::Debug + Send;

    /// The data needed to initialize your [`Application`].
    ///
    /// [`Application`]: trait.Application.html
    type Flags;

    /// Initializes the [`Application`].
    ///
    /// Here is where you should return the initial state of your app.
    ///
    /// Additionally, you can return a [`Command`](struct.Command.html) if you
    /// need to perform some async action in the background on startup. This is
    /// useful if you want to load state from a file, perform an initial HTTP
    /// request, etc.
    ///
    /// [`Application`]: trait.Application.html
    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>)
    where
        Self: Sized;

    /// Returns the current title of the [`Application`].
    ///
    /// This title can be dynamic! The runtime will automatically update the
    /// title of your application when necessary.
    ///
    /// [`Application`]: trait.Application.html
    fn title(&self) -> String;

    /// Handles a __message__ and updates the state of the [`Application`].
    ///
    /// This is where you define your __update logic__. All the __messages__,
    /// produced by either user interactions or commands, will be handled by
    /// this method.
    ///
    /// Any [`Command`] returned will be executed immediately in the background.
    ///
    /// [`Application`]: trait.Application.html
    /// [`Command`]: struct.Command.html
    fn update(&mut self, message: Self::Message) -> Command<Self::Message>;

    /// Returns the widgets to display in the [`Application`].
    ///
    /// These widgets can produce __messages__ based on user interaction.
    ///
    /// [`Application`]: trait.Application.html
    fn view(&mut self) -> Element<'_, Self::Message>;

    /// Returns the event [`Subscription`] for the current state of the
    /// application.
    ///
    /// A [`Subscription`] will be kept alive as long as you keep returning it,
    /// and the __messages__ produced will be handled by
    /// [`update`](#tymethod.update).
    ///
    /// By default, this method returns an empty [`Subscription`].
    ///
    /// [`Subscription`]: struct.Subscription.html
    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }

    /// Runs the [`Application`].
    ///
    /// [`Application`]: trait.Application.html
    fn run(flags: Self::Flags)
    where
        Self: 'static + Sized,
    {

        let event_loop = EventLoop::with_user_event();
        let mut runtime = {
            let executor = Self::Executor::new().expect("Create executor");

            Runtime::new(executor, Proxy::new(event_loop.create_proxy()))
        };

        let (app, command) = runtime.enter(|| Self::new(flags));

        let title = app.title();

        let window = {
            let mut window_builder = WindowBuilder::new();

            //let (width, height) = settings.window.size;

            window_builder = window_builder
                .with_title(title)
                .with_maximized(true)
                .with_fullscreen(None)
                //.with_inner_size(winit::dpi::LogicalSize { width: 100, height: 100})
            ;
            /*
            .with_resizable(settings.window.resizable)
            .with_decorations(settings.window.decorations);
            */
            window_builder.build(&event_loop).expect("Open window")
        };

        window.request_redraw();

        let root_view: UIView = UIView(window.ui_view() as id);
        unsafe {
            let color = UIColor::alloc();
            let background = UIColor(
                color.initWithRed_green_blue_alpha_(0.1, 1.0, 2.0, 2.0),
            );
            root_view.setBackgroundColor_(background.0);
        }
        //let root_view: UIView = UIView(window.ui_view() as id);
        //add_views(root_view);

        event_loop.run(move |event: winit::event::Event<()>, _, control_flow| match event {
            event::Event::MainEventsCleared => {
                window.request_redraw();
            }
            event::Event::UserEvent(_message) => {
                //external_messages.push(message);
            }
            event::Event::RedrawRequested(_) => {}
            event::Event::WindowEvent {
                event: _window_event,
                ..
            } => {
            }
            _ => {
                *control_flow = ControlFlow::Wait;
            }
        })
    }
}
