//! Distribute content horizontally.
//!
//! A [`row`] may have some local [`state`] if an animation is applied.
use crate::animation;
use crate::event::{self, Event};
use crate::layout::{self, Layout};
use crate::mouse;
use crate::overlay;
use crate::renderer;
use crate::widget::tree::{self, Tree};
use crate::widget::Operation;
use crate::{
    Alignment, Animation, Clipboard, Element, Length, Padding, Point,
    Rectangle, Shell, Widget,
};
use iced_core::time::Duration;
use iced_core::time::Instant;
use std::borrow::Borrow;
use std::rc::Rc;

const DEFAULT_WIDTH: Length = Length::Shrink;
const DEFAULT_HEIGHT: Length = Length::Shrink;
const DEFAULT_PADDING: Padding = Padding::ZERO;
const DEFAULT_SPACING: u16 = 0;

/// A container that distributes its contents horizontally.
#[allow(missing_debug_implementations)]
pub struct Row<'a, Message, Renderer> {
    spacing: u16,
    padding: Padding,
    width: Length,
    height: Length,
    align_items: Alignment,
    animation: Option<Rc<Animation>>,
    playhead: Option<animation::Handle>,
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
            spacing: DEFAULT_SPACING,
            padding: DEFAULT_PADDING,
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            align_items: Alignment::Start,
            animation: None,
            playhead: None,
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
        // TODO this is duplicated code. Need a better solution
        // for detecting if value are not default.
        let mut k = Keyframe::default();

        if self.width != DEFAULT_WIDTH {
            k = k.width(self.width)
        }
        if self.height != DEFAULT_HEIGHT {
            k = k.height(self.height)
        }
        if self.padding != DEFAULT_PADDING {
            k = k.padding(self.padding)
        }
        if self.spacing != DEFAULT_SPACING {
            k = k.spacing(self.spacing)
        }
        self.animation = Some(Rc::new(animation.insert(k)));
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
        match self.animation.borrow() {
            Some(_) => tree::Tag::of::<State>(),
            None => tree::Tag::stateless(),
        }
    }

    fn state(&self) -> tree::State {
        match &self.animation {
            Some(animation) => tree::State::new(State::new(animation.clone())),
            None => tree::State::None,
        }
    }
    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.children)
    }

    fn diff_mut(
        &mut self,
        acc: animation::Request,
        tree: &mut Tree,
        app_start: &Instant,
    ) -> animation::Request {
        tree.diff_children_mut(acc, &mut self.children, app_start)
    }

    fn interp(
        &mut self,
        state: &mut tree::State,
        app_start: &Instant,
    ) -> animation::Request {
        let (playhead, request) = state.downcast_mut::<State>().interp(
            app_start,
            self.width,
            self.height,
            self.padding,
            self.spacing,
        );
        self.playhead = Some(playhead);
        request
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
        let (limits, padding, spacing) = match &self.playhead {
            // TODO fix for generic API
            Some(playhead) => {
                let playhead = &playhead.keyframe.modifiers();

                let width = Length::Units(
                    playhead[0].unwrap_or((animation::Ease::Linear, 0)).1
                        as u16,
                );
                let height = Length::Units(
                    playhead[1].unwrap_or((animation::Ease::Linear, 0)).1
                        as u16,
                );
                let spacing = playhead[2]
                    .unwrap_or((animation::Ease::Linear, 0))
                    .1 as u16;
                let padding = Padding::from([
                    playhead[3].unwrap_or((animation::Ease::Linear, 0)).1
                        as u16,
                    playhead[4].unwrap_or((animation::Ease::Linear, 0)).1
                        as u16,
                    playhead[5].unwrap_or((animation::Ease::Linear, 0)).1
                        as u16,
                    playhead[6].unwrap_or((animation::Ease::Linear, 0)).1
                        as u16,
                ]);

                (limits.width(width).height(height), padding, spacing)
            }
            None => (
                limits.width(self.width).height(self.height),
                self.padding,
                self.spacing,
            ),
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
    animation: Rc<Animation>,
}

impl State {
    /// Creates a new [`State`].
    pub fn new(animation: Rc<Animation>) -> State {
        State { animation }
    }

    /// Applies animation to a [`row`] called from [`row::interp`]
    /// See `interp` in the widget trait for more information.
    pub fn interp(
        &mut self,
        app_start: &Instant,
        width: Length,
        height: Length,
        padding: Padding,
        spacing: u16,
    ) -> (animation::Handle, animation::Request) {
        let mut playhead: animation::Handle = {
            // TODO this is duplicated code. Needs a better solution
            let mut k = Keyframe::default();

            if width != DEFAULT_WIDTH {
                k = k.width(width)
            }
            if height != DEFAULT_HEIGHT {
                k = k.height(height)
            }
            if padding != DEFAULT_PADDING {
                k = k.padding(padding)
            }
            if spacing != DEFAULT_SPACING {
                k = k.spacing(spacing)
            }
            k.into()
        };

        let animation: &animation::Animation = self.animation.borrow();
        let request = animation.interp(
            app_start,
            // TODO: This currently assumes that if a value is the default, then there is no animation requested.
            // This doesn't currently cause a problem as the default values arn't animatable, just Length::Units,
            // and Length::FillPortion. Though it would be nice in the future to be able to animate from "non-percise"
            // sizes to percise ones, such as Length::Fill to Length::Units(100)
            &mut playhead,
        );
        (playhead, animation::Request::AnimationFrame)
    }
}

/// An animatable keyframe for a Row.
///
/// For iced internal devs:
/// modifiers:
/// [0] = width,
/// [1] = height,
/// [2] = spacing,
/// [3] = padding - top,
/// [4] = padding - right,
/// [5] = padding - bottom,
/// [6] = padding - left,
#[derive(Debug, Clone)]
pub struct Keyframe {
    after: Duration,
    modifiers: Vec<Option<(animation::Ease, isize)>>,
}

impl std::default::Default for Keyframe {
    fn default() -> Self {
        Keyframe {
            after: Duration::ZERO,
            modifiers: [None; 7].into(),
        }
    }
}

impl animation::Keyframe for Keyframe {
    fn after(&self) -> Duration {
        self.after
    }

    fn set_after(&mut self, after: Duration) {
        self.after = after;
    }

    fn modifiers(&self) -> &Vec<Option<(animation::Ease, isize)>> {
        &self.modifiers
    }

    fn modifiers_mut(&mut self) -> &mut Vec<Option<(animation::Ease, isize)>> {
        &mut self.modifiers
    }
}

// TODO remove this. This is a temp value
const EASE: animation::Ease = animation::Ease::Linear;

impl Keyframe {
    /// Create a new Row Keyframe
    /// Requires the time that the animation state should be equal to keyframe values.
    /// The time is passed as a `Duration` since the start of the animation.
    pub fn new(after: Duration) -> Self {
        Keyframe {
            after,
            ..Keyframe::default()
        }
    }

    /// Set the Row;s width at the Keyframe's time.
    pub fn width(mut self, width: Length) -> Self {
        self.modifiers[0] = Some((EASE, width.as_u16().unwrap() as isize));
        self
    }

    /// Set the Row;s height at the Keyframe's time.
    pub fn height(mut self, height: Length) -> Self {
        self.modifiers[1] = Some((EASE, height.as_u16().unwrap() as isize));
        self
    }

    /// Set the Row;s spacing at the Keyframe's time.
    pub fn spacing(mut self, units: u16) -> Self {
        self.modifiers[2] = Some((EASE, units as isize));
        self
    }

    /// Set the Row;s padding at the Keyframe's time.
    pub fn padding(mut self, padding: Padding) -> Self {
        self.modifiers[3] = Some((EASE, padding.top as isize));
        self.modifiers[4] = Some((EASE, padding.right as isize));
        self.modifiers[5] = Some((EASE, padding.bottom as isize));
        self.modifiers[6] = Some((EASE, padding.left as isize));
        self
    }
}

impl std::convert::From<Keyframe> for animation::Handle {
    fn from(keyframe: Keyframe) -> animation::Handle {
        animation::Handle {
            keyframe: Box::new(keyframe),
        }
    }
}
