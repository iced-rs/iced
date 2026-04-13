//! Mark a widget for automatic focus.
//!
//! Wrapping a focusable widget with [`auto_focus`] tells the framework to
//! automatically focus this widget when it first appears in the tree.
//!
//! Unlike a transparent wrapper, `AutoFocus` maintains its own internal
//! state to track whether it has already triggered focus.  On first mount
//! (when [`state()`] creates fresh state) the widget requests auto-focus
//! during the next [`update()`] call.  It never fires again for the same
//! tree node — **no focus stealing**.
//!
//! For multi-page apps where the same widget structure appears at the same
//! tree position across pages, supply a [`key`](AutoFocus::key) to let the
//! widget detect the page change:
//!
//! ```no_run
//! use iced::widget::{auto_focus, text_input};
//! use iced::Element;
//!
//! fn view<'a>(page: u32, value: &'a str) -> Element<'a, ()> {
//!     auto_focus(text_input("Name", value)).key(page).into()
//! }
//! ```
//!
//! When the key changes, the auto-focus flag resets and the widget focuses
//! again on the next event — exactly like Flutter's `autofocus: true` which
//! re-triggers when the widget is re-mounted.
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget::Operation;
use crate::core::widget::tree::{self, Tree};
use crate::core::{Element, Event, Length, Rectangle, Shell, Size, Vector, Widget};

use std::hash::{Hash, Hasher};

/// Internal state stored in the widget tree.
#[derive(Debug, Clone, Default)]
struct AutoFocusState {
    /// `true` after this widget has successfully requested auto-focus.
    /// Prevents re-triggering on subsequent events (no focus stealing).
    did_auto_focus: bool,
    /// Hash of the key supplied via [`AutoFocus::key`].
    /// When the key changes across `diff()`, `did_auto_focus` is reset.
    key_hash: Option<u64>,
}

/// A wrapper that marks its content for automatic focus on first mount.
///
/// Supply an optional [`key`](Self::key) so the widget can detect page
/// transitions that reuse the same tree position and re-focus accordingly.
#[allow(missing_debug_implementations)]
pub struct AutoFocus<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    key_hash: Option<u64>,
}

impl<'a, Message, Theme, Renderer> AutoFocus<'a, Message, Theme, Renderer> {
    /// Creates a new [`AutoFocus`] wrapper around the given content.
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            content: content.into(),
            key_hash: None,
        }
    }

    /// Sets a key that identifies this auto-focus site.
    ///
    /// When the key changes (e.g. a different page is shown at the same tree
    /// position), auto-focus re-triggers automatically — no explicit
    /// [`focus_auto`] Task needed.
    ///
    /// Any [`Hash`]-able value works: an enum variant, a string, an integer.
    ///
    /// [`focus_auto`]: crate::widget::operation::focus_auto
    #[must_use]
    pub fn key(mut self, key: impl Hash) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        self.key_hash = Some(hasher.finish());
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for AutoFocus<'_, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    // -- Own state (non-transparent) ----------------------------------------

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<AutoFocusState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(AutoFocusState {
            did_auto_focus: false,
            key_hash: self.key_hash,
        })
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(self.content.as_widget())]
    }

    fn diff(&self, tree: &mut Tree) {
        // Detect key change → reset auto-focus so it fires again.
        let state = tree.state.downcast_mut::<AutoFocusState>();
        if self.key_hash != state.key_hash {
            log::debug!(
                "[AutoFocus] key changed ({:?} → {:?}) — resetting",
                state.key_hash,
                self.key_hash,
            );
            state.did_auto_focus = false;
            state.key_hash = self.key_hash;
        }

        // Detect child widget type change → also reset.
        if tree.children.len() == 1 {
            let child_tag = self.content.as_widget().tag();
            if tree.children[0].tag != child_tag {
                log::debug!("[AutoFocus] child widget type changed — resetting");
                state.did_auto_focus = false;
            }
        }

        // Reconcile child tree.
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
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

    // -- Custom behaviour ---------------------------------------------------

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        // Mark the next focusable descendant as the auto-focus target.
        operation.auto_focusable(None, layout.bounds());

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
        let state = tree.state.downcast_mut::<AutoFocusState>();
        if !state.did_auto_focus {
            log::debug!("[AutoFocus] first update after mount/key-change — requesting auto-focus");
            state.did_auto_focus = true;
            shell.request_auto_focus();
        }

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

    // -- Delegation to child ------------------------------------------------

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

impl<'a, Message, Theme, Renderer> From<AutoFocus<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(auto_focus: AutoFocus<'a, Message, Theme, Renderer>) -> Self {
        Self::new(auto_focus)
    }
}

/// Creates an [`AutoFocus`] wrapper that marks its content for automatic
/// focus on first mount.
///
/// For multi-page apps, chain [`.key(page_id)`](AutoFocus::key) so the
/// widget re-focuses when the page changes:
///
/// ```no_run
/// use iced::widget::{auto_focus, text_input};
/// use iced::Element;
///
/// fn view<'a>(value: &'a str) -> Element<'a, ()> {
///     auto_focus(text_input("Name", value)).key("profile").into()
/// }
/// ```
pub fn auto_focus<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> AutoFocus<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    AutoFocus::new(content)
}
