//! Coordinate text selection across sibling text widgets.
//!
//! [`selectable_group`] wraps any [`Element`] and lets the user
//! drag-select continuously across the [`text`] / [`rich_text`]
//! children inside it that opted in via `.selectable(true)`. `Ctrl+C`
//! copies the concatenated selection in tree order, joined by
//! newlines. Wrapping is opt-in — non-grouped selectable widgets keep
//! working per-widget exactly as before.
//!
//! [`text`]: crate::text
//! [`rich_text`]: crate::rich_text
//! [`selectable_group`]: fn@selectable_group
use crate::core::clipboard;
use crate::core::keyboard;
use crate::core::layout;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::text;
use crate::core::widget::operation::Selectable;
use crate::core::widget::tree::{self, Tree};
use crate::core::{self, Element, Event, Layout, Length, Rectangle, Shell, Size, Widget};

use std::marker::PhantomData;

/// A widget that coordinates drag-selection across the selectable
/// `text` and `rich_text` widgets it contains.
pub struct SelectableGroup<'a, Link, Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    Link: Clone + 'static,
    Renderer: text::Renderer,
{
    content: Element<'a, Message, Theme, Renderer>,
    _link: PhantomData<Link>,
}

/// Wraps `content` so its selectable text widgets share a single
/// drag-selection. Cross-widget selection only works when this
/// wrapper is present; without it, each widget selects on its own.
pub fn selectable_group<'a, Link, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> SelectableGroup<'a, Link, Message, Theme, Renderer>
where
    Link: Clone + 'static,
    Renderer: text::Renderer,
{
    SelectableGroup {
        content: content.into(),
        _link: PhantomData,
    }
}

#[derive(Default)]
struct GroupState {
    /// Index into the flattened list of selectables where the drag
    /// started, plus the byte offset within that selectable.
    anchor: Option<(usize, usize)>,
    /// The current focus end of the selection — moves on drag and
    /// `Shift+Arrow`. Stored alongside `anchor` so keyboard navigation
    /// has a starting point even after the drag is over.
    focus: Option<(usize, usize)>,
    /// Screen X column the user is "tracking" for vertical
    /// navigation (`Shift+Up`/`Down`). Updated on click, drag, and
    /// horizontal keyboard moves; preserved across vertical moves so
    /// repeated `Shift+Down` lands at the same column even when
    /// intermediate lines are shorter than the original.
    preferred_x: Option<f32>,
    /// Most recent keyboard modifier state, mirrored from
    /// `ModifiersChanged` events. Mouse events don't carry modifier
    /// info, so press handlers consult this to detect `Shift+Click`.
    modifiers: keyboard::Modifiers,
    /// Whether the user is currently extending a selection by drag.
    selecting: bool,
    /// Most recent left-click; chained into `mouse::Click::new` so
    /// repeated presses within iced's threshold escalate Single →
    /// Double → Triple.
    last_click: Option<mouse::Click>,
}

impl<'a, Link, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for SelectableGroup<'a, Link, Message, Theme, Renderer>
where
    Link: Clone + 'static,
    Renderer: text::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<GroupState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(GroupState::default())
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
        operation: &mut dyn core::widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        defaults: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            defaults,
            layout,
            cursor,
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
        // Mark every selectable in the subtree as externally managed
        // so its own `update` skips drag-select and Ctrl+C; we own
        // those here.
        visit_selectables(
            &mut self.content,
            &mut tree.children[0],
            layout,
            renderer,
            |_, _, state| state.set_externally_managed(true),
        );

        let cursor_position = cursor.position();

        match event {
            Event::Keyboard(keyboard::Event::ModifiersChanged(m)) => {
                tree.state.downcast_mut::<GroupState>().modifiers = *m;
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let (extend, prior_anchor, last_click) = {
                    let group = tree.state.downcast_ref::<GroupState>();
                    (group.modifiers.shift(), group.anchor, group.last_click)
                };

                let mut hit_index = None;
                let mut hit_byte = 0usize;

                if let Some(point) = cursor_position {
                    visit_selectables(
                        &mut self.content,
                        &mut tree.children[0],
                        layout,
                        renderer,
                        |index, bounds, state| {
                            if hit_index.is_some() {
                                return;
                            }
                            if !bounds.contains(point) {
                                return;
                            }
                            let local = point - bounds.position();
                            let byte = state.hit_test(core::Point::ORIGIN + local).unwrap_or(0);
                            hit_index = Some(index);
                            hit_byte = byte;
                        },
                    );
                }

                if let Some(focus_idx) = hit_index {
                    let click_point = cursor_position.unwrap_or(core::Point::ORIGIN);
                    let click = mouse::Click::new(click_point, mouse::Button::Left, last_click);
                    // Extend (Shift+click) takes priority over count
                    // escalation — Single starts/extends a drag,
                    // Double selects word, Triple selects line.
                    let kind = if extend {
                        mouse::click::Kind::Single
                    } else {
                        click.kind()
                    };

                    let mut word_or_line: Option<(usize, usize)> = None;
                    if matches!(
                        kind,
                        mouse::click::Kind::Double | mouse::click::Kind::Triple
                    ) {
                        visit_selectables(
                            &mut self.content,
                            &mut tree.children[0],
                            layout,
                            renderer,
                            |index, _, state| {
                                if index != focus_idx {
                                    return;
                                }
                                let len = state.text_len();
                                word_or_line = Some(match kind {
                                    mouse::click::Kind::Double => (
                                        state.step_byte_word(hit_byte, -1),
                                        state.step_byte_word(hit_byte, 1),
                                    ),
                                    mouse::click::Kind::Triple => (
                                        state.line_edge_byte(hit_byte, -1).unwrap_or(0),
                                        state.line_edge_byte(hit_byte, 1).unwrap_or(len),
                                    ),
                                    mouse::click::Kind::Single => unreachable!(),
                                });
                            },
                        );
                    }

                    let (anchor, focus, selecting) =
                        if let Some((start, end)) = word_or_line {
                            ((focus_idx, start), (focus_idx, end), false)
                        } else if extend {
                            (
                                prior_anchor.unwrap_or((focus_idx, hit_byte)),
                                (focus_idx, hit_byte),
                                true,
                            )
                        } else {
                            (
                                (focus_idx, hit_byte),
                                (focus_idx, hit_byte),
                                true,
                            )
                        };
                    let (a_idx, a_byte) = anchor;
                    let (f_idx, f_byte) = focus;

                    visit_selectables(
                        &mut self.content,
                        &mut tree.children[0],
                        layout,
                        renderer,
                        |index, _, state| {
                            let len = state.text_len();
                            let range =
                                selection_range_for(index, a_idx, a_byte, f_idx, f_byte, len);
                            state.set_selection(range);
                        },
                    );

                    let group = tree.state.downcast_mut::<GroupState>();
                    group.anchor = Some(anchor);
                    group.focus = Some(focus);
                    group.preferred_x = cursor_position.map(|p| p.x);
                    group.selecting = selecting;
                    group.last_click = Some(click);
                    shell.capture_event();
                    shell.request_redraw();
                } else {
                    visit_selectables(
                        &mut self.content,
                        &mut tree.children[0],
                        layout,
                        renderer,
                        |_, _, state| state.set_selection(None),
                    );
                    let group = tree.state.downcast_mut::<GroupState>();
                    group.anchor = None;
                    group.focus = None;
                    group.preferred_x = None;
                    group.selecting = false;
                    group.last_click = None;
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let group = tree.state.downcast_ref::<GroupState>();

                if let (true, Some((anchor_idx, anchor_byte)), Some(point)) =
                    (group.selecting, group.anchor, cursor_position)
                {
                    let mut focus_index = None;
                    let mut focus_byte = 0usize;
                    let mut totals: Vec<(Rectangle, usize)> = Vec::new();

                    visit_selectables(
                        &mut self.content,
                        &mut tree.children[0],
                        layout,
                        renderer,
                        |index, bounds, state| {
                            let len = state.text_len();
                            totals.push((bounds, len));

                            if focus_index.is_some() {
                                return;
                            }
                            if bounds.y > point.y || bounds.contains(point) {
                                let local = point - bounds.position();
                                let byte = state.hit_test(core::Point::ORIGIN + local).unwrap_or(0);
                                focus_index = Some(index);
                                focus_byte = byte;
                            }
                        },
                    );

                    // Cursor past the last selectable — clamp to its end.
                    let focus = focus_index
                        .map(|i| (i, focus_byte))
                        .or_else(|| totals.last().map(|(_, len)| (totals.len() - 1, *len)));

                    if let Some((focus_idx, focus_byte)) = focus {
                        visit_selectables(
                            &mut self.content,
                            &mut tree.children[0],
                            layout,
                            renderer,
                            |index, _, state| {
                                let len = state.text_len();
                                let range = selection_range_for(
                                    index,
                                    anchor_idx,
                                    anchor_byte,
                                    focus_idx,
                                    focus_byte,
                                    len,
                                );
                                state.set_selection(range);
                            },
                        );

                        let group = tree.state.downcast_mut::<GroupState>();
                        group.focus = Some((focus_idx, focus_byte));
                        group.preferred_x = Some(point.x);
                        shell.request_redraw();
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let group = tree.state.downcast_mut::<GroupState>();
                group.selecting = false;

                // Collapse zero-width "click only" selections so a
                // stray single click doesn't leave a stale 0..0 range.
                visit_selectables(
                    &mut self.content,
                    &mut tree.children[0],
                    layout,
                    renderer,
                    |_, _, state| {
                        if let Some((a, b)) = state.selection()
                            && a == b
                        {
                            state.set_selection(None);
                        }
                    },
                );
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Character(c),
                modifiers,
                ..
            }) if modifiers.command() && matches!(c.as_str(), "c" | "C") => {
                let mut chunks: Vec<String> = Vec::new();

                visit_selectables(
                    &mut self.content,
                    &mut tree.children[0],
                    layout,
                    renderer,
                    |_, _, state| {
                        if let Some((a, b)) = state.selection() {
                            let (start, end) = if a <= b { (a, b) } else { (b, a) };
                            if start < end {
                                chunks.push(state.selection_text(start, end));
                            }
                        }
                    },
                );

                if !chunks.is_empty() {
                    let extracted = chunks.join("\n");
                    if !extracted.is_empty() {
                        shell.write_clipboard(clipboard::Content::Text(extracted));
                        shell.capture_event();
                    }
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Character(c),
                modifiers,
                ..
            }) if modifiers.command()
                && matches!(c.as_str(), "a" | "A")
                && {
                    let group = tree.state.downcast_ref::<GroupState>();
                    group.anchor.is_some() || group.focus.is_some()
                } =>
            {
                // Select all selectables in tree order. The anchor
                // becomes the start of the first one, focus the end
                // of the last. Only fires when this group has an
                // existing selection / caret — without that gate every
                // sibling group would steal `Ctrl+A` from each other
                // and from focused `text_editor` widgets.
                let mut total_count = 0usize;
                let mut last_len = 0usize;

                visit_selectables(
                    &mut self.content,
                    &mut tree.children[0],
                    layout,
                    renderer,
                    |_, _, state| {
                        let len = state.text_len();
                        state.set_selection(if len > 0 { Some((0, len)) } else { None });
                        total_count += 1;
                        last_len = len;
                    },
                );

                if total_count > 0 {
                    let group = tree.state.downcast_mut::<GroupState>();
                    group.anchor = Some((0, 0));
                    group.focus = Some((total_count - 1, last_len));
                    shell.capture_event();
                    shell.request_redraw();
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(named),
                modifiers,
                ..
            }) => match named {
                keyboard::key::Named::Escape => {
                    let group = tree.state.downcast_mut::<GroupState>();
                    let had_anything = group.anchor.is_some() || group.focus.is_some();
                    group.anchor = None;
                    group.focus = None;
                    group.selecting = false;

                    visit_selectables(
                        &mut self.content,
                        &mut tree.children[0],
                        layout,
                        renderer,
                        |_, _, state| state.set_selection(None),
                    );

                    if had_anything {
                        shell.capture_event();
                        shell.request_redraw();
                    }
                }
                _ => {
                    let action = match named {
                        keyboard::key::Named::ArrowLeft if modifiers.command() => {
                            Some(KeyAction::Word(-1))
                        }
                        keyboard::key::Named::ArrowRight if modifiers.command() => {
                            Some(KeyAction::Word(1))
                        }
                        keyboard::key::Named::ArrowLeft => Some(KeyAction::Char(-1)),
                        keyboard::key::Named::ArrowRight => Some(KeyAction::Char(1)),
                        keyboard::key::Named::ArrowUp => Some(KeyAction::Line(-1)),
                        keyboard::key::Named::ArrowDown => Some(KeyAction::Line(1)),
                        keyboard::key::Named::Home if modifiers.command() => {
                            Some(KeyAction::DocEdge(-1))
                        }
                        keyboard::key::Named::End if modifiers.command() => {
                            Some(KeyAction::DocEdge(1))
                        }
                        keyboard::key::Named::Home => Some(KeyAction::LineEdge(-1)),
                        keyboard::key::Named::End => Some(KeyAction::LineEdge(1)),
                        _ => None,
                    };

                    if let Some(action) = action {
                        apply_keyboard_action(
                            &mut self.content,
                            tree,
                            layout,
                            renderer,
                            action,
                            modifiers.shift(),
                            shell,
                        );
                    }
                }
            },
            _ => {}
        }

        // Forward the event to the wrapped content so individual
        // widgets (links, etc.) still see it.
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
}

/// Computes the per-selectable selection range during a multi-widget
/// drag. `index` is the selectable being queried; `anchor_*` and
/// `focus_*` define the drag endpoints; `len` is the selectable's
/// total text length.
enum KeyAction {
    Char(i32),
    Word(i32),
    Line(i32),
    LineEdge(i32),
    DocEdge(i32),
}

/// Applies a keyboard navigation action to the group, computing the
/// new focus and writing the resulting per-child selection ranges.
fn apply_keyboard_action<Message, Theme, Renderer>(
    content: &mut Element<'_, Message, Theme, Renderer>,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
    action: KeyAction,
    extend: bool,
    shell: &mut Shell<'_, Message>,
) where
    Renderer: text::Renderer,
{
    let (prior_anchor, prior_focus, preferred_x) = {
        let group = tree.state.downcast_ref::<GroupState>();
        (group.anchor, group.focus, group.preferred_x)
    };

    let Some((focus_idx, focus_byte)) = prior_focus else {
        return;
    };
    let anchor = prior_anchor.unwrap_or((focus_idx, focus_byte));
    let is_vertical = matches!(action, KeyAction::Line(_));

    // Single walk: gather text lengths, the focused widget's screen
    // position, and try to step the focus inside its own widget.
    let mut lens: Vec<usize> = Vec::new();
    let mut focus_x: Option<f32> = None;
    let mut in_widget_focus: Option<(usize, usize)> = None;

    visit_selectables(
        content,
        &mut tree.children[0],
        layout,
        renderer,
        |index, bounds, state| {
            lens.push(state.text_len());

            if index != focus_idx {
                return;
            }

            if let Some(p) = state.byte_position(focus_byte) {
                focus_x = Some(bounds.x + p.x);
            }

            in_widget_focus = match action {
                KeyAction::Char(dir) => {
                    let stepped = state.step_byte(focus_byte, dir);
                    (stepped != focus_byte).then_some((index, stepped))
                }
                KeyAction::Word(dir) => {
                    let stepped = state.step_byte_word(focus_byte, dir);
                    (stepped != focus_byte).then_some((index, stepped))
                }
                KeyAction::Line(dir) => state
                    .step_byte_line(focus_byte, dir)
                    .filter(|&b| b != focus_byte)
                    .map(|b| (index, b)),
                KeyAction::LineEdge(dir) => state
                    .line_edge_byte(focus_byte, dir)
                    .filter(|&b| b != focus_byte)
                    .map(|b| (index, b)),
                KeyAction::DocEdge(_) => None,
            };
        },
    );

    let target_x = preferred_x.or(focus_x).unwrap_or(0.0);

    // If the in-widget step worked, use it. Otherwise fall through to
    // the action's cross-sibling rule.
    let new_focus = in_widget_focus.or_else(|| match action {
        KeyAction::Char(dir) | KeyAction::Word(dir) => {
            if dir > 0 && focus_idx + 1 < lens.len() {
                Some((focus_idx + 1, 0))
            } else if dir < 0 && focus_idx > 0 {
                Some((focus_idx - 1, lens[focus_idx - 1]))
            } else {
                None
            }
        }
        KeyAction::Line(dir) => hit_test_sibling(
            content, tree, layout, renderer, focus_idx, dir, target_x, &lens,
        ),
        KeyAction::LineEdge(_) => None,
        KeyAction::DocEdge(dir) => {
            if dir < 0 {
                Some((0, 0))
            } else if !lens.is_empty() {
                Some((lens.len() - 1, *lens.last().unwrap()))
            } else {
                None
            }
        }
    });

    let Some((new_idx, new_byte)) = new_focus else {
        return;
    };

    // With Shift: keep the existing anchor and extend.
    // Without Shift: collapse — anchor follows focus to the new
    // position (no visible selection, but the caret moves).
    let (a_idx, a_byte) = if extend { anchor } else { (new_idx, new_byte) };

    // Apply per-child selection ranges and snapshot the new focus's
    // screen X (for non-vertical actions, so chained `Shift+Up`/
    // `Down` keep tracking the original column).
    let mut new_focus_x: Option<f32> = None;
    visit_selectables(
        content,
        &mut tree.children[0],
        layout,
        renderer,
        |index, bounds, state| {
            let len = state.text_len();
            let range = selection_range_for(index, a_idx, a_byte, new_idx, new_byte, len);
            state.set_selection(range);

            if !is_vertical
                && index == new_idx
                && let Some(p) = state.byte_position(new_byte)
            {
                new_focus_x = Some(bounds.x + p.x);
            }
        },
    );

    let group = tree.state.downcast_mut::<GroupState>();
    group.anchor = Some((a_idx, a_byte));
    group.focus = Some((new_idx, new_byte));
    group.preferred_x = if is_vertical {
        preferred_x
    } else {
        new_focus_x.or(preferred_x)
    };
    shell.capture_event();
    shell.request_redraw();
}

fn hit_test_sibling<Message, Theme, Renderer>(
    content: &mut Element<'_, Message, Theme, Renderer>,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
    focus_idx: usize,
    dir: i32,
    target_x: f32,
    lens: &[usize],
) -> Option<(usize, usize)>
where
    Renderer: text::Renderer,
{
    let target_idx = if dir > 0 {
        focus_idx + 1
    } else if focus_idx > 0 {
        focus_idx - 1
    } else {
        return None;
    };
    if target_idx >= lens.len() {
        return None;
    }

    let mut new_focus: Option<(usize, usize)> = None;
    visit_selectables(
        content,
        &mut tree.children[0],
        layout,
        renderer,
        |index, bounds, state| {
            if index != target_idx {
                return;
            }
            let lh = state.visual_line_height().unwrap_or(0.0);
            let local_y = if dir > 0 {
                0.0
            } else {
                (bounds.height - lh).max(0.0)
            };
            let local_x = target_x - bounds.x;
            if let Some(byte) = state.hit_test(core::Point::new(local_x, local_y)) {
                new_focus = Some((index, byte));
            }
        },
    );

    new_focus.or_else(|| {
        if dir > 0 {
            Some((target_idx, 0))
        } else {
            Some((target_idx, lens[target_idx]))
        }
    })
}

fn selection_range_for(
    index: usize,
    anchor_idx: usize,
    anchor_byte: usize,
    focus_idx: usize,
    focus_byte: usize,
    len: usize,
) -> Option<(usize, usize)> {
    if anchor_idx == focus_idx {
        if index == anchor_idx {
            Some((anchor_byte, focus_byte))
        } else {
            None
        }
    } else if anchor_idx < focus_idx {
        if index < anchor_idx || index > focus_idx {
            None
        } else if index == anchor_idx {
            Some((anchor_byte, len))
        } else if index == focus_idx {
            Some((0, focus_byte))
        } else {
            Some((0, len))
        }
    } else {
        // anchor_idx > focus_idx (dragging upward)
        if index < focus_idx || index > anchor_idx {
            None
        } else if index == anchor_idx {
            Some((0, anchor_byte))
        } else if index == focus_idx {
            Some((focus_byte, len))
        } else {
            Some((0, len))
        }
    }
}

/// Walks the wrapped content via the [`Operation`] system and calls
/// `callback` on each [`Selectable`] in tree order.
fn visit_selectables<Message, Theme, Renderer, F>(
    content: &mut Element<'_, Message, Theme, Renderer>,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
    callback: F,
) where
    Renderer: text::Renderer,
    F: FnMut(usize, Rectangle, &mut dyn Selectable) + Send,
{
    struct Visitor<F> {
        counter: usize,
        callback: F,
    }

    impl<F> core::widget::Operation for Visitor<F>
    where
        F: FnMut(usize, Rectangle, &mut dyn Selectable) + Send,
    {
        fn selectable(
            &mut self,
            _id: Option<&core::widget::Id>,
            bounds: Rectangle,
            state: &mut dyn Selectable,
        ) {
            (self.callback)(self.counter, bounds, state);
            self.counter += 1;
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn core::widget::Operation)) {
            operate(self);
        }
    }

    let mut visitor = Visitor {
        counter: 0,
        callback,
    };
    content
        .as_widget_mut()
        .operate(tree, layout, renderer, &mut visitor);
}

impl<'a, Link, Message, Theme, Renderer> From<SelectableGroup<'a, Link, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Link: Clone + 'static,
    Theme: 'a,
    Renderer: text::Renderer + 'a,
{
    fn from(
        group: SelectableGroup<'a, Link, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(group)
    }
}
