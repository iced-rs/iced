use iced_native::event;
use iced_native::layout::{self, Layout};
use iced_native::mouse;
use iced_native::overlay;
use iced_native::renderer;
use iced_native::widget::tree::{self, Tree};
use iced_native::widget::{self, Widget};
use iced_native::Element;
use iced_native::{Clipboard, Hasher, Length, Point, Rectangle, Shell, Size};

use ouroboros::self_referencing;
use std::cell::{Ref, RefCell, RefMut};
use std::hash::{Hash, Hasher as H};
use std::marker::PhantomData;
use std::rc::Rc;

#[allow(missing_debug_implementations)]
pub struct Lazy<'a, Message, Renderer, Dependency, View> {
    dependency: Dependency,
    view: Box<dyn Fn() -> View + 'a>,
    element: RefCell<Option<Rc<RefCell<Element<'static, Message, Renderer>>>>>,
}

impl<'a, Message, Renderer, Dependency, View>
    Lazy<'a, Message, Renderer, Dependency, View>
where
    Dependency: Hash + 'a,
    View: Into<Element<'static, Message, Renderer>>,
{
    pub fn new(dependency: Dependency, view: impl Fn() -> View + 'a) -> Self {
        Self {
            dependency,
            view: Box::new(view),
            element: RefCell::new(None),
        }
    }

    fn with_element<T>(
        &self,
        f: impl FnOnce(Ref<Element<Message, Renderer>>) -> T,
    ) -> T {
        f(self.element.borrow().as_ref().unwrap().borrow())
    }

    fn with_element_mut<T>(
        &self,
        f: impl FnOnce(RefMut<Element<Message, Renderer>>) -> T,
    ) -> T {
        f(self.element.borrow().as_ref().unwrap().borrow_mut())
    }
}

struct Internal<Message, Renderer> {
    element: Rc<RefCell<Element<'static, Message, Renderer>>>,
    hash: u64,
}

impl<'a, Message, Renderer, Dependency, View> Widget<Message, Renderer>
    for Lazy<'a, Message, Renderer, Dependency, View>
where
    View: Into<Element<'static, Message, Renderer>> + 'static,
    Dependency: Hash + 'a,
    Message: 'static,
    Renderer: iced_native::Renderer + 'static,
{
    fn tag(&self) -> tree::Tag {
        struct Tag<T>(T);
        tree::Tag::of::<Tag<View>>()
    }

    fn state(&self) -> tree::State {
        let mut hasher = Hasher::default();
        self.dependency.hash(&mut hasher);
        let hash = hasher.finish();

        let element = Rc::new(RefCell::new((self.view)().into()));

        (*self.element.borrow_mut()) = Some(element.clone());

        tree::State::new(Internal { element, hash })
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(
            self.element.borrow().as_ref().unwrap().borrow().as_widget(),
        )]
    }

    fn diff(&self, tree: &mut Tree) {
        let current = tree.state.downcast_mut::<Internal<Message, Renderer>>();

        let mut hasher = Hasher::default();
        self.dependency.hash(&mut hasher);
        let new_hash = hasher.finish();

        if current.hash != new_hash {
            current.hash = new_hash;

            let element = (self.view)().into();
            current.element = Rc::new(RefCell::new(element));

            (*self.element.borrow_mut()) = Some(current.element.clone());
            tree.diff_children(std::slice::from_ref(
                &self.element.borrow().as_ref().unwrap().borrow().as_widget(),
            ));
        } else {
            (*self.element.borrow_mut()) = Some(current.element.clone());
        }
    }

    fn width(&self) -> Length {
        self.with_element(|element| element.as_widget().width())
    }

    fn height(&self) -> Length {
        self.with_element(|element| element.as_widget().height())
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.with_element(|element| {
            element.as_widget().layout(renderer, limits)
        })
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation<Message>,
    ) {
        self.with_element(|element| {
            element.as_widget().operate(
                &mut tree.children[0],
                layout,
                renderer,
                operation,
            );
        });
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: iced_native::Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        self.with_element_mut(|mut element| {
            element.as_widget_mut().on_event(
                &mut tree.children[0],
                event,
                layout,
                cursor_position,
                renderer,
                clipboard,
                shell,
            )
        })
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.with_element(|element| {
            element.as_widget().mouse_interaction(
                &tree.children[0],
                layout,
                cursor_position,
                viewport,
                renderer,
            )
        })
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        self.with_element(|element| {
            element.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                layout,
                cursor_position,
                viewport,
            )
        })
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'_, Message, Renderer>> {
        let overlay = OverlayBuilder {
            cached: self,
            tree: &mut tree.children[0],
            types: PhantomData,
            overlay_builder: |cached, tree| {
                Rc::get_mut(cached.element.get_mut().as_mut().unwrap())
                    .unwrap()
                    .get_mut()
                    .as_widget_mut()
                    .overlay(tree, layout, renderer)
            },
        }
        .build();

        let has_overlay = overlay.with_overlay(|overlay| {
            overlay.as_ref().map(overlay::Element::position)
        });

        has_overlay
            .map(|position| overlay::Element::new(position, Box::new(overlay)))
    }
}

#[self_referencing]
struct Overlay<'a, 'b, Message, Renderer, Dependency, View> {
    cached: &'a mut Lazy<'b, Message, Renderer, Dependency, View>,
    tree: &'a mut Tree,
    types: PhantomData<(Message, Dependency, View)>,

    #[borrows(mut cached, mut tree)]
    #[covariant]
    overlay: Option<overlay::Element<'this, Message, Renderer>>,
}

impl<'a, 'b, Message, Renderer, Dependency, View>
    Overlay<'a, 'b, Message, Renderer, Dependency, View>
{
    fn with_overlay_maybe<T>(
        &self,
        f: impl FnOnce(&overlay::Element<'_, Message, Renderer>) -> T,
    ) -> Option<T> {
        self.borrow_overlay().as_ref().map(f)
    }

    fn with_overlay_mut_maybe<T>(
        &mut self,
        f: impl FnOnce(&mut overlay::Element<'_, Message, Renderer>) -> T,
    ) -> Option<T> {
        self.with_overlay_mut(|overlay| overlay.as_mut().map(f))
    }
}

impl<'a, 'b, Message, Renderer, Dependency, View>
    overlay::Overlay<Message, Renderer>
    for Overlay<'a, 'b, Message, Renderer, Dependency, View>
where
    Renderer: iced_native::Renderer,
{
    fn layout(
        &self,
        renderer: &Renderer,
        bounds: Size,
        position: Point,
    ) -> layout::Node {
        self.with_overlay_maybe(|overlay| {
            let vector = position - overlay.position();

            overlay.layout(renderer, bounds).translate(vector)
        })
        .unwrap_or_default()
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
    ) {
        let _ = self.with_overlay_maybe(|overlay| {
            overlay.draw(renderer, theme, style, layout, cursor_position);
        });
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.with_overlay_maybe(|overlay| {
            overlay.mouse_interaction(
                layout,
                cursor_position,
                viewport,
                renderer,
            )
        })
        .unwrap_or_default()
    }

    fn on_event(
        &mut self,
        event: iced_native::Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        self.with_overlay_mut_maybe(|overlay| {
            overlay.on_event(
                event,
                layout,
                cursor_position,
                renderer,
                clipboard,
                shell,
            )
        })
        .unwrap_or(iced_native::event::Status::Ignored)
    }
}

impl<'a, Message, Renderer, Dependency, View>
    From<Lazy<'a, Message, Renderer, Dependency, View>>
    for Element<'a, Message, Renderer>
where
    View: Into<Element<'static, Message, Renderer>> + 'static,
    Renderer: iced_native::Renderer + 'static,
    Message: 'static,
    Dependency: Hash + 'a,
{
    fn from(lazy: Lazy<'a, Message, Renderer, Dependency, View>) -> Self {
        Self::new(lazy)
    }
}
