//! Generate messages when content pops in and out of view.
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::time::{Duration, Instant};
use crate::core::widget;
use crate::core::widget::tree::{self, Tree};
use crate::core::window;
use crate::core::{
    self, Clipboard, Element, Event, Layout, Length, Pixels, Rectangle, Shell,
    Size, Vector, Widget,
};

/// A widget that can generate messages when its content pops in and out of view.
///
/// It can even notify you with anticipation at a given distance!
#[allow(missing_debug_implementations)]
pub struct Pop<
    'a,
    Key,
    Message,
    Theme = crate::Theme,
    Renderer = crate::Renderer,
> {
    content: Element<'a, Message, Theme, Renderer>,
    key: Key,
    on_show: Option<Box<dyn Fn(Size) -> Message + 'a>>,
    on_resize: Option<Box<dyn Fn(Size) -> Message + 'a>>,
    on_hide: Option<Message>,
    anticipate: Pixels,
    delay: Duration,
}

impl<'a, Message, Theme, Renderer> Pop<'a, (), Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: core::Renderer,
{
    /// Creates a new [`Pop`] widget with the given content.
    pub fn new(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        Self {
            content: content.into(),
            key: (),
            on_show: None,
            on_resize: None,
            on_hide: None,
            anticipate: Pixels::ZERO,
            delay: Duration::ZERO,
        }
    }
}

impl<'a, Key, Message, Theme, Renderer> Pop<'a, Key, Message, Theme, Renderer>
where
    Message: Clone,
    Key: self::Key,
    Renderer: core::Renderer,
{
    /// Sets the message to be produced when the content pops into view.
    ///
    /// The closure will receive the [`Size`] of the content in that moment.
    pub fn on_show(mut self, on_show: impl Fn(Size) -> Message + 'a) -> Self {
        self.on_show = Some(Box::new(on_show));
        self
    }

    /// Sets the message to be produced when the content changes [`Size`] once its in view.
    ///
    /// The closure will receive the new [`Size`] of the content.
    pub fn on_resize(
        mut self,
        on_resize: impl Fn(Size) -> Message + 'a,
    ) -> Self {
        self.on_resize = Some(Box::new(on_resize));
        self
    }

    /// Sets the message to be produced when the content pops out of view.
    pub fn on_hide(mut self, on_hide: Message) -> Self {
        self.on_hide = Some(on_hide);
        self
    }

    /// Sets the key of the [`Pop`] widget, for continuity.
    ///
    /// If the key changes, the [`Pop`] widget will trigger again.
    pub fn key<K>(
        self,
        key: K,
    ) -> Pop<'a, impl self::Key, Message, Theme, Renderer>
    where
        K: Clone + PartialEq + 'static,
    {
        Pop {
            content: self.content,
            key: OwnedKey(key),
            on_show: self.on_show,
            on_resize: self.on_resize,
            on_hide: self.on_hide,
            anticipate: self.anticipate,
            delay: self.delay,
        }
    }

    /// Sets the key of the [`Pop`] widget, for continuity; using a reference.
    ///
    /// If the key changes, the [`Pop`] widget will trigger again.
    pub fn key_ref<K>(
        self,
        key: &'a K,
    ) -> Pop<'a, &'a K, Message, Theme, Renderer>
    where
        K: ToOwned + PartialEq<K::Owned> + ?Sized,
        K::Owned: 'static,
    {
        Pop {
            content: self.content,
            key,
            on_show: self.on_show,
            on_resize: self.on_resize,
            on_hide: self.on_hide,
            anticipate: self.anticipate,
            delay: self.delay,
        }
    }

    /// Sets the distance in [`Pixels`] to use in anticipation of the
    /// content popping into view.
    ///
    /// This can be quite useful to lazily load items in a long scrollable
    /// behind the scenes before the user can notice it!
    pub fn anticipate(mut self, distance: impl Into<Pixels>) -> Self {
        self.anticipate = distance.into();
        self
    }

    /// Sets the amount of time to wait before firing an [`on_show`] or
    /// [`on_hide`] event; after the content is shown or hidden.
    ///
    /// When combined with [`key`], this can be useful to debounce key changes.
    ///
    /// [`on_show`]: Self::on_show
    /// [`on_hide`]: Self::on_hide
    /// [`key`]: Self::key
    pub fn delay(mut self, delay: impl Into<Duration>) -> Self {
        self.delay = delay.into();
        self
    }
}

#[derive(Debug, Clone)]
struct State<Key> {
    has_popped_in: bool,
    should_notify_at: Option<(bool, Instant)>,
    last_size: Option<Size>,
    last_key: Key,
}

impl<Key, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Pop<'_, Key, Message, Theme, Renderer>
where
    Key: self::Key,
    Message: Clone,
    Renderer: core::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Key::Owned>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            has_popped_in: false,
            should_notify_at: None,
            last_size: None,
            last_key: self.key.to_owned(),
        })
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.content]);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if let Event::Window(window::Event::RedrawRequested(now)) = &event {
            let state = tree.state.downcast_mut::<State<Key::Owned>>();

            if state.has_popped_in && !self.key.eq(&state.last_key) {
                state.has_popped_in = false;
                state.should_notify_at = None;
                state.last_key = self.key.to_owned();
            }

            let bounds = layout.bounds();
            let top_left_distance = viewport.distance(bounds.position());

            let bottom_right_distance = viewport
                .distance(bounds.position() + Vector::from(bounds.size()));

            let distance = top_left_distance.min(bottom_right_distance);

            if state.has_popped_in {
                if distance <= self.anticipate.0 {
                    if let Some(on_resize) = &self.on_resize {
                        let size = bounds.size();

                        if Some(size) != state.last_size {
                            state.last_size = Some(size);
                            shell.publish(on_resize(size));
                        }
                    }
                } else if self.on_hide.is_some() {
                    state.has_popped_in = false;
                    state.should_notify_at = Some((false, *now + self.delay));
                }
            } else if self.on_show.is_some() && distance <= self.anticipate.0 {
                let size = bounds.size();

                state.has_popped_in = true;
                state.should_notify_at = Some((true, *now + self.delay));
                state.last_size = Some(size);
            }

            match &state.should_notify_at {
                Some((has_popped_in, at)) if at <= now => {
                    if *has_popped_in {
                        if let Some(on_show) = &self.on_show {
                            shell.publish(on_show(layout.bounds().size()));
                        }
                    } else if let Some(on_hide) = &self.on_hide {
                        shell.publish(on_hide.clone());
                    }

                    state.should_notify_at = None;
                }
                Some((_, at)) => {
                    shell.request_redraw_at(*at);
                }
                None => {}
            }
        }

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: layout::Layout<'_>,
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

    fn operate(
        &self,
        tree: &mut Tree,
        layout: core::Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.content.as_widget().operate(
            &mut tree.children[0],
            layout,
            renderer,
            operation,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: core::Layout<'_>,
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

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: core::Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: core::Vector,
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

impl<'a, Key, Message, Theme, Renderer>
    From<Pop<'a, Key, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Key: self::Key + 'a,
    Renderer: core::Renderer + 'a,
    Theme: 'a,
{
    fn from(pop: Pop<'a, Key, Message, Theme, Renderer>) -> Self {
        Element::new(pop)
    }
}

/// The key of a widget.
///
/// You should generally not need to care about this trait.
pub trait Key {
    /// The owned version of the key.
    type Owned: 'static;

    /// Returns the owned version of the key.
    fn to_owned(&self) -> Self::Owned;

    /// Compares the key with the given owned version.
    fn eq(&self, other: &Self::Owned) -> bool;
}

impl<T> Key for &T
where
    T: ToOwned + PartialEq<T::Owned> + ?Sized,
    T::Owned: 'static,
{
    type Owned = T::Owned;

    fn to_owned(&self) -> <Self as Key>::Owned {
        ToOwned::to_owned(*self)
    }

    fn eq(&self, other: &Self::Owned) -> bool {
        *self == other
    }
}

struct OwnedKey<T>(T);

impl<T> Key for OwnedKey<T>
where
    T: PartialEq + Clone + 'static,
{
    type Owned = T;

    fn to_owned(&self) -> Self::Owned {
        self.0.clone()
    }

    fn eq(&self, other: &Self::Owned) -> bool {
        &self.0 == other
    }
}

impl<T> PartialEq<T> for OwnedKey<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &T) -> bool {
        &self.0 == other
    }
}

impl Key for () {
    type Owned = ();

    fn to_owned(&self) -> Self::Owned {}

    fn eq(&self, _other: &Self::Owned) -> bool {
        true
    }
}
