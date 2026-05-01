//! Intercept keyboard events before they reach child widgets.
//!
//! This widget allows consuming specific key combinations so that
//! child widgets never see them — similar to `event.stopPropagation()`
//! or `event.preventDefault()` in web frameworks.
use crate::core::keyboard;
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget::Operation;
use crate::core::widget::tree::{self, Tree};
use crate::core::{Element, Event, Layout, Length, Rectangle, Shell, Size, Vector, Widget};

/// A widget that intercepts keyboard events before they reach its child.
///
/// Use this to "consume" specific key combinations at a parent level,
/// preventing child widgets (like buttons) from processing them.
///
/// # Example
/// ```no_run
/// use iced::widget::shortcut;
///
/// let content = shortcut(my_button)
///     .on_key(
///         |key, modifiers| {
///             matches!(
///                 key,
///                 iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter)
///             ) && modifiers.control()
///         },
///         MyMessage::CtrlEnterPressed,
///     );
/// ```
pub struct Shortcut<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    bindings: Vec<Binding<'a, Message>>,
}

struct Binding<'a, Message> {
    matcher: Box<dyn Fn(&keyboard::Key, keyboard::Modifiers) -> bool + 'a>,
    message: Message,
}

impl<'a, Message, Theme, Renderer> Shortcut<'a, Message, Theme, Renderer> {
    /// Creates a new [`Shortcut`] widget wrapping the given content.
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            content: content.into(),
            bindings: Vec::new(),
        }
    }

    /// Registers a key binding that will be intercepted before reaching children.
    ///
    /// The `matcher` function receives the key and modifiers and returns `true`
    /// if the event should be consumed. When consumed, the `message` is published
    /// and child widgets never see the event.
    #[must_use]
    pub fn on_key(
        mut self,
        matcher: impl Fn(&keyboard::Key, keyboard::Modifiers) -> bool + 'a,
        message: Message,
    ) -> Self {
        self.bindings.push(Binding {
            matcher: Box::new(matcher),
            message,
        });
        self
    }
}

/// Creates a [`Shortcut`] widget that intercepts keyboard events before its child.
pub fn shortcut<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Shortcut<'a, Message, Theme, Renderer> {
    Shortcut::new(content)
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Shortcut<'_, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
    Message: Clone,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::stateless()
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        // Check if this is a key press that matches any registered binding.
        // If so, consume the event BEFORE passing to children.
        if let Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event {
            for binding in &self.bindings {
                if (binding.matcher)(key, *modifiers) {
                    shell.publish(binding.message.clone());
                    shell.capture_event();
                    return;
                }
            }
        }

        // No binding matched — pass the event through to the child.
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<Shortcut<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(shortcut: Shortcut<'a, Message, Theme, Renderer>) -> Self {
        Element::new(shortcut)
    }
}
