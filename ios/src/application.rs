use crate::{
    event::{EventHandler, WidgetEvent},
    widget::{RenderAction, Widget, WidgetNode},
    Command, Element, Executor, Runtime, Subscription,
};
use winit::{
    event,
    event_loop::{ControlFlow, EventLoop},
    platform::ios::WindowExtIOS,
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
    //UISwitch,
    UIView,
    //UIView_UIViewGeometry,
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
        let event_loop: EventLoop<WidgetEvent> = EventLoop::with_user_event();
        let proxy = event_loop.create_proxy();
        EventHandler::init(proxy.clone());
        let (sender, _receiver) =
            iced_futures::futures::channel::mpsc::unbounded();

        let mut runtime = {
            let executor = Self::Executor::new().expect("Create executor");

            Runtime::new(executor, sender)
        };

        let (mut app, command) = runtime.enter(|| Self::new(flags));
        runtime.spawn(command);

        let title = app.title();

        let window = {
            let mut window_builder = WindowBuilder::new();

            //let (width, height) = settings.window.size;

            window_builder = window_builder
                .with_title(title)
                .with_maximized(true)
                //.with_fullscreen(None)
                //.with_inner_size(winit::dpi::LogicalSize { width: 1000, height: 1000})
            ;
            /*
            .with_resizable(settings.window.resizable)
            .with_decorations(settings.window.decorations);
            */
            window_builder.build(&event_loop).expect("Open window")
        };

        let root_view: UIView = UIView(window.ui_view() as id);
        unsafe {
            let background = UIColor(UIColor::greenColor());
            //let background = UIColor(UIColor::whiteColor());
            root_view.setBackgroundColor_(background.0);
            /*
            let rect = CGRect {
                origin: CGPoint { x: 0.0, y: 0.0 },
                size: CGSize {
                    height: 400.0,
                    width: 300.0,
                },
            };
            root_view.setFrame_(rect);
            */
        }
        let mut widget_tree: WidgetNode = app.view().get_widget_node();

        event_loop.run(
            move |event: winit::event::Event<WidgetEvent>, _, control_flow| {
                //let new_title = application.borrow().title();
                //debug!("NEW EVENT: {:?}", event);
                let mut messages: Vec<Self::Message> = Vec::new();
                match event {
                    event::Event::MainEventsCleared => {}
                    event::Event::UserEvent(widget_event) => {
                        {
                            let mut element = app.view();
                            element.widget.on_widget_event(
                                widget_event,
                                &mut messages,
                                &widget_tree,
                            );
                            debug!("Root widget before: {:?}", widget_tree);
                        }
                        for message in messages {
                            let (command, subscription) = runtime.enter(|| {
                                let command = app.update(message);
                                let subscription = app.subscription();

                                (command, subscription)
                            });

                            runtime.spawn(command);
                            runtime.track(subscription);
                        }
                        let mut element = app.view();
                        let new_tree = element.build_uiview();
                        if new_tree != widget_tree {
                            new_tree.draw(root_view);
                            widget_tree.drop_from_ui();
                            widget_tree = new_tree;
                        }
                        /*

                        match element
                            .get_render_action(widget_tree)
                        {
                            RenderAction::Add | RenderAction::Update => {
                                debug!("Adding or updating root widget {:?} with {:?}", widget_tree.as_ref(), element.get_widget_type());
                                widget_tree = element.update_or_add(
                                    Some(root_view),
                                    widget_tree,
                                );
                            }
                            RenderAction::Remove => {
                                debug!("Removing root widget {:?} with {:?}", node, element.get_widget_type());
                                node.drop_from_ui();
                                widget_tree = element.update_or_add(
                                    Some(root_view),
                                    widget_tree,
                                );
                            },
                        }
                        */
                        debug!("Root widget after: {:?}", widget_tree);
                    }
                    event::Event::RedrawRequested(_) => {}
                    event::Event::WindowEvent {
                        event: _window_event,
                        ..
                    } => {}
                    event::Event::NewEvents(event::StartCause::Init) => {
                        let mut element = app.view();
                        widget_tree = element.build_uiview();
                        let root_view: UIView = UIView(window.ui_view() as id);
                        widget_tree.draw(root_view);
                    }
                    _ => {
                        *control_flow = ControlFlow::Wait;
                    }
                }
            },
        );
    }
}
