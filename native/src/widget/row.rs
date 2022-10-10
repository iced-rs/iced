//! Distribute content horizontally.
//!
//! A [`row`] may have some local [`state`] if an animation is applied.
use crate::event::{self, Event};
use crate::layout::{self, Layout};
use crate::mouse;
use crate::overlay;
use crate::renderer;
use crate::animation;
use crate::widget::Operation;
use crate::widget::tree::{self, Tree};
use crate::{
    Alignment, Clipboard, Element, Length, Padding, Point, Rectangle, Shell,
    Widget, Animation,
};
use iced_core::time::Instant;

/// A container that distributes its contents horizontally.
#[allow(missing_debug_implementations)]
pub struct Row<'a, Message, Renderer> {
    spacing: u16,
    padding: Padding,
    width: Length,
    height: Length,
    align_items: Alignment,
    animation: Option<Animation>,
    children: Vec<Element<'a, Message, Renderer>>,
}

impl<'a, Message, Renderer> Row<'a, Message, Renderer> {
    /// Creates an empty [`Row`].
    pub fn new() -> Self {
        Self::with_children(Vec::new())
    }

    /// Creates a [`Row`] with the given elements.
    pub fn with_children(
        children: Vec<Element<'a, Message, Renderer>>,
    ) -> Self {
        Row {
            spacing: 0,
            padding: Padding::ZERO,
            width: Length::Shrink,
            height: Length::Shrink,
            align_items: Alignment::Start,
            animation: None,
            children,
        }
    }

    /// Sets the horizontal spacing _between_ elements.
    ///
    /// Custom margins per element do not exist in iced. You should use this
    /// method instead! While less flexible, it helps you keep spacing between
    /// elements consistent.
    pub fn spacing(mut self, units: u16) -> Self {
        self.spacing = units;
        self
    }

    /// Sets the [`Padding`] of the [`Row`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the width of the [`Row`].
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Row`].
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Set the animation of the [`Row`]
    pub fn animation(mut self, animation: Animation) -> Self {
        self.animation = Some(animation);
        self
    }

    /// Sets the vertical alignment of the contents of the [`Row`] .
    pub fn align_items(mut self, align: Alignment) -> Self {
        self.align_items = align;
        self
    }

    /// Adds an [`Element`] to the [`Row`].
    pub fn push(
        mut self,
        child: impl Into<Element<'a, Message, Renderer>>,
    ) -> Self {
        self.children.push(child.into());
        self
    }
}

impl<'a, Message, Renderer> Default for Row<'a, Message, Renderer> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Row<'a, Message, Renderer>
where
    Renderer: crate::Renderer,
{
    fn tag(&self) -> tree::Tag {
        match self.animation {
            Some(_) => tree::Tag::of::<State>(),
            None => tree::Tag::stateless()
        }
    }

    fn state(&self) -> tree::State {
        match &self.animation {
            Some(animation) => tree::State::new(State::new((*animation).clone())),
            None => tree::State::None,
        }
    }
    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.children)
    }

    fn diff_mut(&mut self, acc: animation::Request, tree: &mut Tree, app_start: &Instant) -> animation::Request {
        tree.diff_children_mut(acc, &mut self.children, app_start)
    }

    fn interp(&mut self, state: &mut tree::State, app_start: &Instant) -> animation::Request {
        println!("interp called");
        state.downcast_mut::<State>().interp(app_start)
    }

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
        tree: &Tree,
    ) -> layout::Node {
        let (limits, padding, spacing) = match &self.animation {
            Some(animation) => {
                (limits.width(animation.width()).height(self.height()),
                 animation.padding(),
                 animation.spacing(),
                )
            }
            None => {
                (limits.width(self.width).height(self.height),
                 self.padding,
                 self.spacing,
                )
            }
        };

        layout::flex::resolve(
            layout::flex::Axis::Horizontal,
            renderer,
            &limits,
            padding,
            spacing as f32,
            self.align_items,
            &self.children,
            &tree.children,
        )
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        operation: &mut dyn Operation<Message>,
    ) {
        operation.container(None, &mut |operation| {
            self.children
                .iter()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child.as_widget().operate(state, layout, operation);
                })
        });
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        self.children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child.as_widget_mut().on_event(
                    state,
                    event.clone(),
                    layout,
                    cursor_position,
                    renderer,
                    clipboard,
                    shell,
                )
            })
            .fold(event::Status::Ignored, event::Status::merge)
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child.as_widget().mouse_interaction(
                    state,
                    layout,
                    cursor_position,
                    viewport,
                    renderer,
                )
            })
            .max()
            .unwrap_or_default()
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
        for ((child, state), layout) in self
            .children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
        {
            child.as_widget().draw(
                state,
                renderer,
                theme,
                style,
                layout,
                cursor_position,
                viewport,
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        overlay::from_children(&mut self.children, tree, layout, renderer)
    }
}

impl<'a, Message, Renderer> From<Row<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: crate::Renderer + 'a,
{
    fn from(row: Row<'a, Message, Renderer>) -> Self {
        Self::new(row)
    }
}

/// The local state of a [`Row`].
#[derive(Debug)]
pub struct State {
    animation: Animation,
}

impl State {
    /// Creates a new [`State`].
    pub fn new(animation: Animation) -> State {
        State {
            animation,
        }
    }

    /// Applies animation to a [`row`] called from [`row::interp`]
    /// See `interp` in the widget trait for more information.
    pub fn interp(&mut self, app_start: &Instant) -> animation::Request {
        self.animation.interp(app_start, animation::Keyframe::new().width(Length::Shrink).height(Length::Shrink))
    }
}
