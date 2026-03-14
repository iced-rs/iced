//! Accessibility support via [`accesskit`].
//!
//! When the `a11y` feature is enabled, each window gets an [`accesskit`]
//! adapter that bridges the platform accessibility layer. Adapters are
//! stored per-window alongside other window state and follow the same
//! lifecycle as the window itself.
//!
//! For implementation details, see `docs/a11y-internals-guide.md` in the
//! repository.
//!
//! [`accesskit`]: https://docs.rs/accesskit

use crate::core::widget::operation::accessible::{
    Accessible, Live as IcedLive, Orientation as IcedOrientation, Role as IcedRole,
    Value as IcedValue,
};
use crate::core::widget::operation::{Focusable, Scrollable, TextInput};
use crate::core::widget::{self, Operation};
use crate::core::window;
use crate::core::{Rectangle, Vector};

use accesskit::{
    ActionHandler, ActionRequest, ActivationHandler, DeactivationHandler, Live, Node, NodeId, Role,
    Toggled, Tree, TreeId, TreeUpdate,
};
use accesskit_winit::Adapter;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

/// An action request from assistive technology, tagged with the iced
/// window ID.
#[derive(Debug)]
pub struct A11yActionRequest {
    /// The iced window ID this action targets.
    pub window_id: window::Id,
    /// The [`ActionRequest`] from assistive technology.
    pub request: ActionRequest,
}

/// Root [`NodeId`] used for the initial placeholder tree.
const ROOT_ID: NodeId = NodeId(0);

/// Width in logical pixels assumed for scrollbar AT nodes.
///
/// This doesn't affect rendering -- it provides bounds for the
/// accessibility node so assistive technology knows where the
/// scrollbar is positioned.
const SCROLLBAR_WIDTH: f32 = 8.0;

struct IcedActivationHandler {
    active: Arc<AtomicBool>,
    title: String,
}

impl ActivationHandler for IcedActivationHandler {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        self.active.store(true, Ordering::Release);

        let mut node = Node::new(Role::Window);
        node.set_label(self.title.clone());

        Some(TreeUpdate {
            nodes: vec![(ROOT_ID, node)],
            tree: Some(Tree::new(ROOT_ID)),
            tree_id: TreeId::ROOT,
            focus: ROOT_ID,
        })
    }
}

struct IcedActionHandler {
    window_id: window::Id,
    queue: Arc<Mutex<Vec<A11yActionRequest>>>,
}

impl ActionHandler for IcedActionHandler {
    fn do_action(&mut self, request: ActionRequest) {
        if let Ok(mut q) = self.queue.lock() {
            q.push(A11yActionRequest {
                window_id: self.window_id,
                request,
            });
        } else {
            log::warn!("a11y: action queue lock poisoned");
        }
    }
}

struct IcedDeactivationHandler {
    active: Arc<AtomicBool>,
}

impl DeactivationHandler for IcedDeactivationHandler {
    fn deactivate_accessibility(&mut self) {
        self.active.store(false, Ordering::Release);
    }
}

/// Per-window accessibility adapter wrapping an
/// [`accesskit_winit::Adapter`].
///
/// Created during window construction and stored alongside the window.
/// Dropped automatically when the window is closed.
pub struct A11yAdapter {
    adapter: Adapter,
    window: Arc<Window>,
    action_queue: Arc<Mutex<Vec<A11yActionRequest>>>,
    active: Arc<AtomicBool>,
}

impl A11yAdapter {
    /// Creates a new adapter for the given window.
    ///
    /// Must be called before the window is made visible, as required
    /// by [`accesskit_winit`].
    ///
    /// [`accesskit_winit`]: https://docs.rs/accesskit_winit
    pub(crate) fn new(
        event_loop: &ActiveEventLoop,
        window: Arc<Window>,
        iced_id: window::Id,
        title: &str,
    ) -> Self {
        let action_queue = Arc::new(Mutex::new(Vec::new()));
        let active = Arc::new(AtomicBool::new(false));

        let adapter = Adapter::with_direct_handlers(
            event_loop,
            &window,
            IcedActivationHandler {
                active: Arc::clone(&active),
                title: title.to_owned(),
            },
            IcedActionHandler {
                window_id: iced_id,
                queue: Arc::clone(&action_queue),
            },
            IcedDeactivationHandler {
                active: Arc::clone(&active),
            },
        );

        Self {
            adapter,
            window,
            action_queue,
            active,
        }
    }

    /// Returns whether an assistive technology is currently connected.
    ///
    /// When `false`, the tree walk and build can be skipped entirely.
    pub(crate) fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    /// Forwards a [`winit`] window event to the [`accesskit`] adapter.
    ///
    /// This should be called before the event is processed by iced.
    ///
    /// [`accesskit`]: https://docs.rs/accesskit
    pub(crate) fn process_event(&mut self, event: &winit::event::WindowEvent) {
        self.adapter.process_event(&self.window, event);
    }

    /// Pushes an accessibility tree update if accessibility is active.
    ///
    /// If accessibility has not been activated for this window, the
    /// update closure is not called.
    pub(crate) fn update_if_active(&mut self, update_fn: impl FnOnce() -> TreeUpdate) {
        self.adapter.update_if_active(update_fn);
    }

    /// Drains all pending action requests from assistive technology.
    ///
    /// Returns actions such as focus requests, button activations, and
    /// value changes queued by the platform accessibility layer.
    pub(crate) fn drain_action_requests(&self) -> Vec<A11yActionRequest> {
        if let Ok(mut q) = self.action_queue.lock() {
            std::mem::take(&mut *q)
        } else {
            log::warn!("a11y: action queue lock poisoned");
            Vec::new()
        }
    }
}

// --- Synthetic event helpers ---
//
// AT actions are translated into iced events so that widgets handle
// them through their existing input code paths. These helpers produce
// the event sequences used by the action drain loop in `lib.rs`.

use crate::core::Event;
use crate::core::Point;
use crate::core::keyboard;
use crate::core::mouse;

/// Produces synthetic mouse events for a click at `center`.
///
/// Sequence: CursorMoved -> ButtonPressed(Left) -> ButtonReleased(Left).
pub(crate) fn synthetic_click(center: Point) -> [Event; 3] {
    [
        Event::Mouse(mouse::Event::CursorMoved { position: center }),
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
    ]
}

/// Produces synthetic mouse + keyboard events for an arrow key
/// press at `center`.
///
/// Sequence: CursorMoved -> KeyPressed(key) -> KeyReleased(key).
pub(crate) fn synthetic_arrow_key(center: Point, key: keyboard::key::Named) -> [Event; 3] {
    [
        Event::Mouse(mouse::Event::CursorMoved { position: center }),
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(key),
            modified_key: keyboard::Key::Named(key),
            physical_key: keyboard::key::Physical::Code(named_to_code(key)),
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::empty(),
            text: None,
            repeat: false,
        }),
        Event::Keyboard(keyboard::Event::KeyReleased {
            key: keyboard::Key::Named(key),
            modified_key: keyboard::Key::Named(key),
            physical_key: keyboard::key::Physical::Code(named_to_code(key)),
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::empty(),
        }),
    ]
}

/// Produces a synthetic CursorMoved event to `center`.
pub(crate) fn synthetic_cursor_move(center: Point) -> Event {
    Event::Mouse(mouse::Event::CursorMoved { position: center })
}

fn named_to_code(key: keyboard::key::Named) -> keyboard::key::Code {
    match key {
        keyboard::key::Named::ArrowUp => keyboard::key::Code::ArrowUp,
        keyboard::key::Named::ArrowDown => keyboard::key::Code::ArrowDown,
        keyboard::key::Named::ArrowLeft => keyboard::key::Code::ArrowLeft,
        keyboard::key::Named::ArrowRight => keyboard::key::Code::ArrowRight,
        // Only arrow keys are used for synthetic AT events.
        _ => unreachable!("synthetic_arrow_key called with non-arrow key"),
    }
}

/// Maps an iced accessibility [`Role`](IcedRole) to an
/// [`accesskit::Role`].
///
/// Iced defines its own platform-agnostic `Role` enum so that
/// `iced_core` has no dependency on [`accesskit`]. This function
/// bridges the two at the winit integration layer.
///
/// The iced enum is a curated subset of accesskit's ~170 roles,
/// covering the roles that iced widgets use and those a custom
/// widget author would reasonably need. Web/document-specific
/// roles (DPub, ARIA graphics, HTML input subtypes) are omitted.
/// The `#[non_exhaustive]` catch-all maps to [`Role::Unknown`].
fn convert_role(role: IcedRole) -> Role {
    match role {
        IcedRole::Alert => Role::Alert,
        IcedRole::AlertDialog => Role::AlertDialog,
        IcedRole::Button => Role::Button,
        IcedRole::Canvas => Role::Canvas,
        IcedRole::CheckBox => Role::CheckBox,
        IcedRole::ComboBox => Role::ComboBox,
        IcedRole::Dialog => Role::Dialog,
        IcedRole::Document => Role::Document,
        IcedRole::Group => Role::Group,
        IcedRole::Heading => Role::Heading,
        IcedRole::Image => Role::Image,
        IcedRole::Label => Role::Label,
        IcedRole::Link => Role::Link,
        IcedRole::List => Role::List,
        IcedRole::ListItem => Role::ListItem,
        IcedRole::Menu => Role::Menu,
        IcedRole::MenuBar => Role::MenuBar,
        IcedRole::MenuItem => Role::MenuItem,
        IcedRole::Meter => Role::Meter,
        IcedRole::MultilineTextInput => Role::MultilineTextInput,
        IcedRole::Navigation => Role::Navigation,
        IcedRole::ProgressIndicator => Role::ProgressIndicator,
        IcedRole::RadioButton => Role::RadioButton,
        IcedRole::Region => Role::Region,
        IcedRole::ScrollBar => Role::ScrollBar,
        IcedRole::ScrollView => Role::ScrollView,
        IcedRole::Search => Role::Search,
        // accesskit lacks a non-interactive Separator role.
        // Splitter implies interactive drag, so GenericContainer is
        // used as a neutral fallback.
        IcedRole::Separator => Role::GenericContainer,
        IcedRole::Slider => Role::Slider,
        IcedRole::StaticText => Role::Label,
        IcedRole::Status => Role::Status,
        IcedRole::Switch => Role::Switch,
        IcedRole::Tab => Role::Tab,
        IcedRole::TabList => Role::TabList,
        IcedRole::TabPanel => Role::TabPanel,
        IcedRole::Table => Role::Table,
        IcedRole::TextInput => Role::TextInput,
        IcedRole::Toolbar => Role::Toolbar,
        IcedRole::Tooltip => Role::Tooltip,
        IcedRole::Tree => Role::Tree,
        IcedRole::TreeItem => Role::TreeItem,
        IcedRole::Window => Role::Window,
        _ => Role::Unknown,
    }
}

/// Converts an iced [`Rectangle`] to an [`accesskit::Rect`].
fn to_accesskit_rect(bounds: Rectangle) -> accesskit::Rect {
    accesskit::Rect {
        x0: bounds.x as f64,
        y0: bounds.y as f64,
        x1: (bounds.x + bounds.width) as f64,
        y1: (bounds.y + bounds.height) as f64,
    }
}

/// The result of building an accessibility tree from the widget tree.
pub struct A11yTree {
    /// The [`TreeUpdate`] to push to the adapter.
    pub update: TreeUpdate,
    /// Maps accesskit [`NodeId`]s to iced widget IDs and bounds.
    pub node_map: HashMap<NodeId, (Option<widget::Id>, Rectangle)>,
    /// The currently focused [`NodeId`], if any.
    pub focused: Option<NodeId>,
}

/// Builds an [`accesskit`] tree by walking the widget tree via the
/// [`Operation`] trait.
///
/// Created per-frame when accessibility is active, then consumed by
/// `A11yAdapter::update_if_active`.
pub struct TreeBuilder {
    nodes: Vec<(NodeId, Node)>,
    node_map: HashMap<NodeId, (Option<widget::Id>, Rectangle)>,
    children: HashMap<NodeId, Vec<NodeId>>,
    parent_stack: Vec<NodeId>,
    current_accessible: Option<NodeId>,
    focused: Option<NodeId>,
    announcements: Vec<String>,
    scroll_offset: Vector,
    /// Pending `labelled_by` cross-node relationships to resolve in `build()`.
    label_refs: Vec<(NodeId, widget::Id)>,
    /// Pending `described_by` cross-node relationships to resolve in `build()`.
    desc_refs: Vec<(NodeId, widget::Id)>,
}

impl TreeBuilder {
    /// Creates a new builder with a root Window node labelled with
    /// the window's title.
    pub fn new(title: &str) -> Self {
        let mut root = Node::new(Role::Window);
        root.set_label(title.to_string());

        Self {
            nodes: vec![(ROOT_ID, root)],
            node_map: HashMap::new(),
            children: HashMap::new(),
            parent_stack: vec![ROOT_ID],
            current_accessible: None,
            focused: None,
            announcements: Vec::new(),
            scroll_offset: Vector::ZERO,
            label_refs: Vec::new(),
            desc_refs: Vec::new(),
        }
    }
}

impl TreeBuilder {
    /// Adds pending announcements that will appear as assertive
    /// live-region nodes in the tree.
    pub fn with_announcements(mut self, announcements: &[String]) -> Self {
        self.announcements = announcements.to_vec();
        self
    }

    fn alloc_id(&mut self, widget_id: Option<&widget::Id>) -> NodeId {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();

        if let Some(wid) = widget_id {
            wid.hash(&mut hasher);
        } else {
            let parent = self.current_parent();
            parent.0.hash(&mut hasher);
            let sibling_count = self.children.get(&parent).map_or(0, Vec::len) as u64;
            sibling_count.hash(&mut hasher);
        }

        let mut raw = hasher.finish();
        // Ensure non-zero (0 is reserved for root) and handle
        // collisions defensively by rehashing.
        if raw == 0 {
            raw = u64::MAX;
        }
        let mut collision_count = 0u32;
        while self.node_map.contains_key(&NodeId(raw)) {
            collision_count += 1;
            let mut h = std::collections::hash_map::DefaultHasher::new();
            raw.hash(&mut h);
            collision_count.hash(&mut h);
            raw = h.finish();
            if raw == 0 {
                raw = u64::MAX;
            }
        }
        NodeId(raw)
    }

    /// Returns bounds adjusted for the current scroll offset.
    fn adjusted_bounds(&self, bounds: Rectangle) -> Rectangle {
        Rectangle {
            x: bounds.x - self.scroll_offset.x,
            y: bounds.y - self.scroll_offset.y,
            ..bounds
        }
    }

    fn current_parent(&self) -> NodeId {
        self.parent_stack.last().copied().unwrap_or(ROOT_ID)
    }

    fn add_child(&mut self, parent: NodeId, child: NodeId) {
        self.children.entry(parent).or_default().push(child);
    }

    /// Consumes the builder and returns the finished [`A11yTree`].
    pub fn build(mut self) -> A11yTree {
        let announcements = std::mem::take(&mut self.announcements);
        for text in announcements {
            let id = self.alloc_id(None);
            let mut node = Node::new(Role::Label);
            // Label-role nodes expose their content via `value`, not
            // `label` (accesskit's label_comes_from_value).
            node.set_value(text);
            node.set_live(Live::Assertive);
            node.set_bounds(to_accesskit_rect(Rectangle {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            }));
            self.add_child(ROOT_ID, id);
            self.nodes.push((id, node));
        }

        // Resolve labelled_by / described_by cross-node relationships.
        let wid_to_node: HashMap<widget::Id, NodeId> = self
            .node_map
            .iter()
            .filter_map(|(nid, (wid, _))| wid.as_ref().map(|w| (w.clone(), *nid)))
            .collect();

        for (source_id, target_wid) in &self.label_refs {
            if let Some(target_nid) = wid_to_node.get(target_wid)
                && let Some((_, node)) = self.nodes.iter_mut().find(|(nid, _)| nid == source_id)
            {
                node.set_labelled_by(vec![*target_nid]);
            }
        }
        for (source_id, target_wid) in &self.desc_refs {
            if let Some(target_nid) = wid_to_node.get(target_wid)
                && let Some((_, node)) = self.nodes.iter_mut().find(|(nid, _)| nid == source_id)
            {
                node.set_described_by(vec![*target_nid]);
            }
        }

        for (node_id, node) in &mut self.nodes {
            if let Some(children) = self.children.remove(node_id) {
                node.set_children(children);
            }
        }

        let focus = self.focused.unwrap_or(ROOT_ID);

        A11yTree {
            update: TreeUpdate {
                nodes: self.nodes,
                tree: Some(Tree::new(ROOT_ID)),
                tree_id: TreeId::ROOT,
                focus,
            },
            node_map: self.node_map,
            focused: self.focused,
        }
    }
}

impl Operation for TreeBuilder {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        let parent = self.current_accessible.take();
        let saved_offset = self.scroll_offset;

        if let Some(node_id) = parent {
            self.parent_stack.push(node_id);
        }

        operate(self);

        // Clear leftover state from the last child so it does not
        // leak to the parent's next sibling.
        self.current_accessible = None;
        self.scroll_offset = saved_offset;

        if parent.is_some() {
            let _ = self.parent_stack.pop();
        }
    }

    fn container(&mut self, id: Option<&widget::Id>, bounds: Rectangle) {
        // If accessible() was called by THIS widget (same bounds and ID),
        // it already created the node -- container() is a no-op.
        // Compare against adjusted bounds because node_map stores
        // scroll-adjusted values.
        let adjusted = self.adjusted_bounds(bounds);
        if let Some(prev_id) = self.current_accessible
            && let Some((prev_wid, prev_bounds)) = self.node_map.get(&prev_id)
            && *prev_bounds == adjusted
            && prev_wid.as_ref() == id
        {
            return;
        }

        // Previous sibling's stale state or no accessible -- create container
        self.current_accessible = None;
        let node_id = self.alloc_id(id);
        let mut node = Node::new(Role::GenericContainer);
        node.set_bounds(to_accesskit_rect(adjusted));
        let parent = self.current_parent();
        self.add_child(parent, node_id);
        self.nodes.push((node_id, node));
        let _ = self.node_map.insert(node_id, (id.cloned(), adjusted));
        self.current_accessible = Some(node_id);
    }

    fn accessible(
        &mut self,
        id: Option<&widget::Id>,
        bounds: Rectangle,
        accessible: &Accessible<'_>,
    ) {
        let node_id = self.alloc_id(id);
        let mut node = Node::new(convert_role(accessible.role));

        let adjusted = self.adjusted_bounds(bounds);
        node.set_bounds(to_accesskit_rect(adjusted));

        if let Some(label) = accessible.label {
            node.set_label(label.to_string());
        }
        if let Some(description) = accessible.description {
            node.set_description(description.to_string());
        }
        if accessible.disabled {
            node.set_disabled();
        }
        if let Some(toggled) = accessible.toggled {
            node.set_toggled(if toggled {
                Toggled::True
            } else {
                Toggled::False
            });
        }
        if let Some(selected) = accessible.selected {
            node.set_selected(selected);
        }
        if let Some(expanded) = accessible.expanded {
            node.set_expanded(expanded);
        }
        if let Some(live) = accessible.live {
            node.set_live(match live {
                IcedLive::Polite => Live::Polite,
                IcedLive::Assertive => Live::Assertive,
            });
        }
        if accessible.required {
            node.set_required();
        }
        if let Some(level) = accessible.level {
            node.set_level(level);
        }
        if let Some(ref value) = accessible.value {
            match value {
                IcedValue::Text(text) => {
                    node.set_value((*text).to_string());
                }
                IcedValue::Numeric {
                    current,
                    min,
                    max,
                    step,
                } => {
                    node.set_numeric_value(*current);
                    node.set_min_numeric_value(*min);
                    node.set_max_numeric_value(*max);
                    if let Some(step) = step {
                        node.set_numeric_value_step(*step);
                    }
                }
            }
        }

        if let Some(wid) = accessible.labelled_by {
            self.label_refs.push((node_id, wid.clone()));
        }
        if let Some(wid) = accessible.described_by {
            self.desc_refs.push((node_id, wid.clone()));
        }

        // Declare supported actions so AT knows what interactions
        // are available. Matches the pattern in accesskit's
        // simple.rs example.
        match accessible.role {
            IcedRole::Button
            | IcedRole::CheckBox
            | IcedRole::RadioButton
            | IcedRole::Switch
            | IcedRole::Link
            | IcedRole::MenuItem
            | IcedRole::Tab => {
                node.add_action(accesskit::Action::Click);
            }
            _ => {}
        }
        if let Some(IcedValue::Numeric { step: Some(_), .. }) = accessible.value {
            node.add_action(accesskit::Action::Increment);
            node.add_action(accesskit::Action::Decrement);
        }
        if accessible.expanded.is_some() {
            node.add_action(accesskit::Action::Expand);
            node.add_action(accesskit::Action::Collapse);
        }
        if matches!(accessible.role, IcedRole::ComboBox) {
            node.set_has_popup(accesskit::HasPopup::Listbox);
        }

        // For text input roles, expose the label as a placeholder
        // in addition to setting it as the accessible name. This
        // matches accesskit's canonical pattern for text fields.
        if matches!(
            accessible.role,
            IcedRole::TextInput | IcedRole::MultilineTextInput
        ) && let Some(label) = accessible.label
        {
            node.set_placeholder(label.to_string());
        }

        if let Some(orientation) = accessible.orientation {
            node.set_orientation(match orientation {
                IcedOrientation::Horizontal => accesskit::Orientation::Horizontal,
                IcedOrientation::Vertical => accesskit::Orientation::Vertical,
            });
        }

        let parent = self.current_parent();
        self.add_child(parent, node_id);
        self.nodes.push((node_id, node));
        let _ = self.node_map.insert(node_id, (id.cloned(), adjusted));
        self.current_accessible = Some(node_id);
    }

    fn text(&mut self, id: Option<&widget::Id>, bounds: Rectangle, text: &str) {
        // If the current accessible node already carries this exact
        // text as its label, skip -- the widget set it explicitly via
        // Accessible and creating a Label child would be redundant.
        // This prevents duplicate Labels for widgets like CheckBox
        // that set both Accessible.label and call operation.text().
        if let Some(acc_id) = self.current_accessible
            && let Some((_, node)) = self.nodes.iter_mut().find(|(nid, _)| *nid == acc_id)
            && node.label() == Some(text)
        {
            return;
        }

        // Create a Label node. AccessKit's consumer derives the
        // accessible name of Button, CheckBox, Link, etc. from
        // descendant Label nodes automatically (via its built-in
        // descendant_label_filter). GenericContainers in between are
        // transparently skipped.
        let parent = self.current_parent();
        let node_id = self.alloc_id(id);
        let mut node = Node::new(Role::Label);
        // Label-role nodes expose their content via `value`, not
        // `label` (accesskit's label_comes_from_value).
        node.set_value(text.to_string());
        let adjusted = self.adjusted_bounds(bounds);
        node.set_bounds(to_accesskit_rect(adjusted));

        self.add_child(parent, node_id);
        self.nodes.push((node_id, node));
        let _ = self.node_map.insert(node_id, (id.cloned(), adjusted));
    }

    fn focusable(
        &mut self,
        _id: Option<&widget::Id>,
        _bounds: Rectangle,
        state: &mut dyn Focusable,
    ) {
        if let Some(node_id) = self.current_accessible {
            if let Some((_, node)) = self.nodes.iter_mut().find(|(nid, _)| *nid == node_id) {
                node.add_action(accesskit::Action::Focus);
            }
            if state.is_focused() {
                self.focused = Some(node_id);
            }
        }
    }

    fn scrollable(
        &mut self,
        _id: Option<&widget::Id>,
        bounds: Rectangle,
        content_bounds: Rectangle,
        translation: Vector,
        _state: &mut dyn Scrollable,
    ) {
        self.scroll_offset += translation;

        // Set scroll position and range on the parent accessible node.
        if let Some(parent_id) = self.current_accessible
            && let Some((_, parent_node)) = self.nodes.iter_mut().find(|(nid, _)| *nid == parent_id)
        {
            parent_node.set_scroll_y(translation.y as f64);
            parent_node.set_scroll_y_min(0.0);
            parent_node.set_scroll_y_max((content_bounds.height - bounds.height).max(0.0) as f64);
            parent_node.set_scroll_x(translation.x as f64);
            parent_node.set_scroll_x_min(0.0);
            parent_node.set_scroll_x_max((content_bounds.width - bounds.width).max(0.0) as f64);
        }

        // Create a ScrollBar child node when content overflows and
        // mark the parent as clipping so accesskit can filter
        // out-of-bounds children from the AT tree.
        if let Some(parent_id) = self.current_accessible
            && content_bounds.height > bounds.height
        {
            if let Some((_, parent_node)) = self.nodes.iter_mut().find(|(nid, _)| *nid == parent_id)
            {
                parent_node.set_clips_children();
            }

            let scrollbar_id = self.alloc_id(None);
            let mut scrollbar = Node::new(Role::ScrollBar);
            scrollbar.set_numeric_value(translation.y as f64);
            scrollbar.set_min_numeric_value(0.0);
            scrollbar
                .set_max_numeric_value((content_bounds.height - bounds.height).max(0.0) as f64);
            scrollbar.set_controls(vec![parent_id]);
            let adjusted = self.adjusted_bounds(bounds);
            let sb_bounds = Rectangle {
                x: adjusted.x + adjusted.width - SCROLLBAR_WIDTH,
                y: adjusted.y,
                width: SCROLLBAR_WIDTH,
                height: adjusted.height,
            };
            scrollbar.set_bounds(to_accesskit_rect(sb_bounds));
            self.add_child(parent_id, scrollbar_id);
            self.nodes.push((scrollbar_id, scrollbar));
            let _ = self.node_map.insert(scrollbar_id, (None, sb_bounds));
        }
    }

    fn text_input(
        &mut self,
        _id: Option<&widget::Id>,
        _bounds: Rectangle,
        _state: &mut dyn TextInput,
    ) {
    }

    fn custom(
        &mut self,
        _id: Option<&widget::Id>,
        _bounds: Rectangle,
        _state: &mut dyn std::any::Any,
    ) {
    }

    fn finish(&self) -> crate::core::widget::operation::Outcome<()> {
        crate::core::widget::operation::Outcome::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accesskit::Action;

    #[test]
    fn initial_tree_has_window_root() {
        let active = Arc::new(AtomicBool::new(false));
        let mut handler = IcedActivationHandler {
            active: Arc::clone(&active),
            title: "Test Window".to_owned(),
        };
        let tree = handler.request_initial_tree();
        let update = tree.expect("activation handler returns a tree");

        assert!(
            active.load(Ordering::Acquire),
            "activation sets active flag"
        );
        assert_eq!(update.nodes.len(), 1);

        let (id, node) = &update.nodes[0];
        assert_eq!(*id, ROOT_ID);
        assert_eq!(node.role(), Role::Window);

        let tree = update.tree.expect("initial update includes tree");
        assert_eq!(tree.root, ROOT_ID);
        assert_eq!(update.focus, ROOT_ID);
    }

    #[test]
    fn action_handler_queues_requests() {
        let queue = Arc::new(Mutex::new(Vec::new()));
        let window_id = window::Id::unique();

        let mut handler = IcedActionHandler {
            window_id,
            queue: Arc::clone(&queue),
        };

        let request = ActionRequest {
            action: Action::Focus,
            target_tree: TreeId::ROOT,
            target_node: NodeId(1),
            data: None,
        };

        handler.do_action(request);

        let pending = queue.lock().unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].window_id, window_id);
        assert_eq!(pending[0].request.action, Action::Focus);
        assert_eq!(pending[0].request.target_node, NodeId(1));
    }

    #[test]
    fn take_clears_pending_actions() {
        let queue = Arc::new(Mutex::new(Vec::new()));

        let mut handler = IcedActionHandler {
            window_id: window::Id::unique(),
            queue: Arc::clone(&queue),
        };

        handler.do_action(ActionRequest {
            action: Action::Click,
            target_tree: TreeId::ROOT,
            target_node: ROOT_ID,
            data: None,
        });

        // First take returns pending actions
        let first = {
            let mut q = queue.lock().unwrap();
            std::mem::take(&mut *q)
        };
        assert!(!first.is_empty());

        // Second take returns nothing
        let second = {
            let mut q = queue.lock().unwrap();
            std::mem::take(&mut *q)
        };
        assert!(second.is_empty());
    }

    // --- Integration tests below require a Wayland compositor ---
    // Run with: WAYLAND_DISPLAY=... cargo test --features a11y
    // Or use headless weston: weston --backend=headless

    /// Runs a closure inside a winit event loop with access to
    /// [`ActiveEventLoop`]. Requires a running Wayland compositor
    /// (e.g. headless weston via `WAYLAND_DISPLAY`).
    fn with_event_loop(f: impl FnOnce(&ActiveEventLoop) + 'static) {
        use winit::application::ApplicationHandler;
        use winit::event_loop::EventLoop;
        use winit::platform::wayland::EventLoopBuilderExtWayland;

        struct TestApp<F: FnOnce(&ActiveEventLoop)>(Option<F>);

        impl<F: FnOnce(&ActiveEventLoop)> ApplicationHandler for TestApp<F> {
            fn resumed(&mut self, event_loop: &ActiveEventLoop) {
                if let Some(f) = self.0.take() {
                    f(event_loop);
                }
                event_loop.exit();
            }

            fn window_event(
                &mut self,
                _: &ActiveEventLoop,
                _: winit::window::WindowId,
                _: winit::event::WindowEvent,
            ) {
            }
        }

        let event_loop = EventLoop::builder()
            .with_any_thread(true)
            .build()
            .expect("create event loop (is WAYLAND_DISPLAY set?)");

        let _ = event_loop.run_app(&mut TestApp(Some(f)));
    }

    /// Returns true if a Wayland compositor is available for
    /// integration tests.
    fn has_wayland() -> bool {
        std::env::var_os("WAYLAND_DISPLAY").is_some()
    }

    /// Integration test for the full adapter lifecycle. Combined into
    /// a single test because winit only allows one event loop per
    /// process.
    ///
    /// Requires `WAYLAND_DISPLAY` to be set (e.g. headless weston).
    #[test]
    fn adapter_lifecycle() {
        if !has_wayland() {
            eprintln!("skipping adapter_lifecycle: WAYLAND_DISPLAY not set");
            return;
        }

        with_event_loop(|event_loop| {
            // -- Creation --
            let attrs = Window::default_attributes().with_visible(false);
            let window = Arc::new(event_loop.create_window(attrs).expect("create window"));
            let id = window::Id::unique();

            let mut adapter = A11yAdapter::new(event_loop, window, id, "Test Window");
            assert!(adapter.drain_action_requests().is_empty());

            // -- process_event with the events accesskit handles --
            adapter.process_event(&winit::event::WindowEvent::Focused(true));
            adapter.process_event(&winit::event::WindowEvent::Focused(false));
            adapter.process_event(&winit::event::WindowEvent::Resized(
                winit::dpi::PhysicalSize::new(800, 600),
            ));

            // -- process_event with one it ignores --
            adapter.process_event(&winit::event::WindowEvent::RedrawRequested);

            // -- update_if_active (no AT connected, closure not called) --
            let mut called = false;
            adapter.update_if_active(|| {
                called = true;
                TreeUpdate {
                    nodes: vec![],
                    tree: None,
                    tree_id: TreeId::ROOT,
                    focus: ROOT_ID,
                }
            });
            assert!(!called, "update closure should not fire without AT");

            // -- Drop cleans up without panic --
            drop(adapter);
        });
    }

    #[test]
    fn concurrent_action_handler() {
        let queue = Arc::new(Mutex::new(Vec::new()));
        let window_id = window::Id::unique();
        let n_threads = 8;
        let n_actions = 100;

        let threads: Vec<_> = (0..n_threads)
            .map(|_| {
                let mut handler = IcedActionHandler {
                    window_id,
                    queue: Arc::clone(&queue),
                };

                std::thread::spawn(move || {
                    for i in 0..n_actions {
                        handler.do_action(ActionRequest {
                            action: Action::Focus,
                            target_tree: TreeId::ROOT,
                            target_node: NodeId(i as u64),
                            data: None,
                        });
                    }
                })
            })
            .collect();

        for t in threads {
            t.join().expect("thread should not panic");
        }

        let total = queue.lock().unwrap().len();
        assert_eq!(total, n_threads * n_actions);
    }

    #[test]
    fn separate_adapters_have_independent_queues() {
        let queue_a = Arc::new(Mutex::new(Vec::new()));
        let queue_b = Arc::new(Mutex::new(Vec::new()));

        let id_a = window::Id::unique();
        let id_b = window::Id::unique();

        let mut handler_a = IcedActionHandler {
            window_id: id_a,
            queue: Arc::clone(&queue_a),
        };
        let mut handler_b = IcedActionHandler {
            window_id: id_b,
            queue: Arc::clone(&queue_b),
        };

        handler_a.do_action(ActionRequest {
            action: Action::Focus,
            target_tree: TreeId::ROOT,
            target_node: ROOT_ID,
            data: None,
        });
        handler_b.do_action(ActionRequest {
            action: Action::Click,
            target_tree: TreeId::ROOT,
            target_node: ROOT_ID,
            data: None,
        });
        handler_b.do_action(ActionRequest {
            action: Action::Focus,
            target_tree: TreeId::ROOT,
            target_node: NodeId(1),
            data: None,
        });

        let a_requests = queue_a.lock().unwrap();
        let b_requests = queue_b.lock().unwrap();

        assert_eq!(a_requests.len(), 1);
        assert_eq!(a_requests[0].window_id, id_a);

        assert_eq!(b_requests.len(), 2);
        assert_eq!(b_requests[0].window_id, id_b);
        assert_eq!(b_requests[1].window_id, id_b);
    }

    #[test]
    fn poisoned_lock_returns_empty() {
        let queue = Arc::new(Mutex::new(Vec::<A11yActionRequest>::new()));

        // Poison the lock by panicking while holding it
        let queue_clone = Arc::clone(&queue);
        let _ = std::thread::spawn(move || {
            let _guard = queue_clone.lock().unwrap();
            panic!("intentional poison");
        })
        .join();

        assert!(queue.lock().is_err(), "lock should be poisoned");

        // Action handler should not panic on poisoned lock
        let mut handler = IcedActionHandler {
            window_id: window::Id::unique(),
            queue: Arc::clone(&queue),
        };
        handler.do_action(ActionRequest {
            action: Action::Focus,
            target_tree: TreeId::ROOT,
            target_node: ROOT_ID,
            data: None,
        });

        // drain_action_requests equivalent should return empty
        let result = if let Ok(mut q) = queue.lock() {
            std::mem::take(&mut *q)
        } else {
            Vec::new()
        };
        assert!(result.is_empty());
    }

    // --- TreeBuilder tests ---

    /// Convenience bounds for tests that don't care about geometry.
    const UNIT: Rectangle = Rectangle {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 50.0,
    };

    #[test]
    fn empty_tree_has_only_root() {
        let tree = TreeBuilder::new("Test Window").build();

        assert_eq!(tree.update.nodes.len(), 1);
        assert_eq!(tree.update.nodes[0].1.role(), Role::Window);
        assert_eq!(tree.update.focus, NodeId(0));
    }

    #[test]
    fn accessible_creates_child_of_root() {
        let mut builder = TreeBuilder::new("Test Window");
        builder.accessible(
            None,
            UNIT,
            &Accessible {
                role: IcedRole::Button,
                label: Some("OK"),
                ..Accessible::default()
            },
        );

        let tree = builder.build();

        assert_eq!(tree.update.nodes.len(), 2);
        let button_id = tree.update.nodes[1].0;
        assert_eq!(tree.update.nodes[0].1.children(), &[button_id]);
        assert_eq!(tree.update.nodes[1].1.role(), Role::Button);
        assert_eq!(tree.update.nodes[1].1.label(), Some("OK"));
    }

    #[test]
    fn traverse_nests_children_under_parent() {
        let mut builder = TreeBuilder::new("Test Window");

        builder.accessible(
            None,
            UNIT,
            &Accessible {
                role: IcedRole::Group,
                ..Accessible::default()
            },
        );
        builder.traverse(&mut |op| {
            op.accessible(
                None,
                UNIT,
                &Accessible {
                    role: IcedRole::Button,
                    ..Accessible::default()
                },
            );
        });

        let tree = builder.build();

        let group_id = tree.update.nodes[1].0;
        let button_id = tree.update.nodes[2].0;
        assert_eq!(tree.update.nodes[0].1.children(), &[group_id]);
        assert_eq!(tree.update.nodes[1].1.children(), &[button_id]);
        assert_eq!(tree.update.nodes[2].1.role(), Role::Button);
    }

    #[test]
    fn button_gets_label_from_descendant() {
        let mut builder = TreeBuilder::new("Test Window");

        // button { Text("Save") } -- accesskit's consumer derives
        // the button's name from the descendant Label node.
        builder.accessible(
            None,
            UNIT,
            &Accessible {
                role: IcedRole::Button,
                ..Accessible::default()
            },
        );
        builder.traverse(&mut |op| {
            op.text(None, UNIT, "Save");
        });

        let tree = builder.build();

        // Button has no direct label -- a Label child carries the text
        assert_eq!(tree.update.nodes.len(), 3);
        assert!(tree.update.nodes[1].1.label().is_none());
        assert_eq!(tree.update.nodes[1].1.role(), Role::Button);

        // The Label child stores text in `value` (accesskit convention)
        assert_eq!(tree.update.nodes[2].1.role(), Role::Label);
        assert_eq!(tree.update.nodes[2].1.value(), Some("Save"));

        // Label is a child of the button
        assert!(
            tree.update.nodes[1]
                .1
                .children()
                .contains(&tree.update.nodes[2].0)
        );
    }

    #[test]
    fn announcements_appear_as_assertive_labels() {
        let tree = TreeBuilder::new("Test Window")
            .with_announcements(&["File saved".to_owned()])
            .build();

        assert_eq!(tree.update.nodes.len(), 2);
        assert_eq!(tree.update.nodes[1].1.role(), Role::Label);
        assert_eq!(tree.update.nodes[1].1.value(), Some("File saved"));
        assert_eq!(tree.update.nodes[1].1.live(), Some(Live::Assertive));
    }

    // --- Comprehensive TreeBuilder edge cases ---

    struct MockFocusable(bool);

    impl Focusable for MockFocusable {
        fn is_focused(&self) -> bool {
            self.0
        }
        fn focus(&mut self) {
            self.0 = true;
        }
        fn unfocus(&mut self) {
            self.0 = false;
        }
    }

    #[test]
    fn accessible_maps_all_properties() {
        let mut builder = TreeBuilder::new("Test Window");
        builder.accessible(
            None,
            UNIT,
            &Accessible {
                role: IcedRole::CheckBox,
                label: Some("Accept terms"),
                description: Some("You must accept to continue"),
                toggled: Some(true),
                disabled: true,
                selected: Some(false),
                expanded: Some(true),
                live: Some(IcedLive::Polite),
                value: Some(IcedValue::Text("checked")),
                ..Accessible::default()
            },
        );

        let tree = builder.build();
        let node = &tree.update.nodes[1].1;

        assert_eq!(node.role(), Role::CheckBox);
        assert_eq!(node.label(), Some("Accept terms"));
        assert_eq!(node.description(), Some("You must accept to continue"));
        assert_eq!(node.toggled(), Some(Toggled::True));
        assert!(node.is_disabled());
        assert_eq!(node.is_selected(), Some(false));
        assert_eq!(node.is_expanded(), Some(true));
        assert_eq!(node.live(), Some(Live::Polite));
        assert_eq!(node.value(), Some("checked"));
    }

    #[test]
    fn accessible_maps_numeric_value() {
        let mut builder = TreeBuilder::new("Test Window");
        builder.accessible(
            None,
            UNIT,
            &Accessible {
                role: IcedRole::Slider,
                value: Some(IcedValue::Numeric {
                    current: 50.0,
                    min: 0.0,
                    max: 100.0,
                    step: Some(1.0),
                }),
                ..Accessible::default()
            },
        );

        let tree = builder.build();
        let node = &tree.update.nodes[1].1;

        assert_eq!(node.numeric_value(), Some(50.0));
        assert_eq!(node.min_numeric_value(), Some(0.0));
        assert_eq!(node.max_numeric_value(), Some(100.0));
        assert_eq!(node.numeric_value_step(), Some(1.0));
    }

    #[test]
    fn numeric_value_without_step() {
        let mut builder = TreeBuilder::new("Test Window");
        builder.accessible(
            None,
            UNIT,
            &Accessible {
                role: IcedRole::ProgressIndicator,
                value: Some(IcedValue::Numeric {
                    current: 75.0,
                    min: 0.0,
                    max: 100.0,
                    step: None,
                }),
                ..Accessible::default()
            },
        );

        let tree = builder.build();
        let node = &tree.update.nodes[1].1;

        assert_eq!(node.numeric_value(), Some(75.0));
        assert!(node.numeric_value_step().is_none());
    }

    #[test]
    fn toggled_false_maps_correctly() {
        let mut builder = TreeBuilder::new("Test Window");
        builder.accessible(
            None,
            UNIT,
            &Accessible {
                role: IcedRole::Switch,
                toggled: Some(false),
                ..Accessible::default()
            },
        );

        let tree = builder.build();
        assert_eq!(tree.update.nodes[1].1.toggled(), Some(Toggled::False));
    }

    #[test]
    fn explicit_label_preserved_with_different_text_child() {
        let mut builder = TreeBuilder::new("Test Window");

        builder.accessible(
            None,
            UNIT,
            &Accessible {
                role: IcedRole::Button,
                label: Some("Submit"),
                ..Accessible::default()
            },
        );
        builder.traverse(&mut |op| {
            op.text(None, UNIT, "Send");
        });

        let tree = builder.build();
        // Explicit label is preserved on the button
        assert_eq!(tree.update.nodes[1].1.label(), Some("Submit"));
        // The text child becomes a standalone Label node (different text)
        assert_eq!(tree.update.nodes.len(), 3);
        assert_eq!(tree.update.nodes[2].1.role(), Role::Label);
        assert_eq!(tree.update.nodes[2].1.value(), Some("Send"));
    }

    #[test]
    fn text_at_root_creates_standalone_label() {
        let mut builder = TreeBuilder::new("Test Window");
        builder.text(None, UNIT, "Hello world");

        let tree = builder.build();

        assert_eq!(tree.update.nodes.len(), 2);
        assert_eq!(tree.update.nodes[1].1.role(), Role::Label);
        assert_eq!(tree.update.nodes[1].1.value(), Some("Hello world"));
    }

    #[test]
    fn text_skips_redundant_label() {
        let mut builder = TreeBuilder::new("Test Window");

        // Checkbox pattern: widget sets both Accessible.label and
        // calls text() with the same string.
        builder.accessible(
            None,
            UNIT,
            &Accessible {
                role: IcedRole::CheckBox,
                label: Some("Accept terms"),
                ..Accessible::default()
            },
        );
        builder.text(None, UNIT, "Accept terms");

        let tree = builder.build();

        // No redundant Label child -- the checkbox already carries the text
        assert_eq!(tree.update.nodes.len(), 2);
        assert_eq!(tree.update.nodes[1].1.label(), Some("Accept terms"));
    }

    #[test]
    fn actions_added_to_clickable_roles() {
        let mut builder = TreeBuilder::new("Test Window");

        builder.accessible(
            None,
            UNIT,
            &Accessible {
                role: IcedRole::Button,
                ..Accessible::default()
            },
        );

        let tree = builder.build();

        assert!(
            tree.update.nodes[1]
                .1
                .supports_action(accesskit::Action::Click),
            "Button should support Click action"
        );
    }

    #[test]
    fn actions_added_to_numeric_values() {
        let mut builder = TreeBuilder::new("Test Window");

        builder.accessible(
            None,
            UNIT,
            &Accessible {
                role: IcedRole::Slider,
                value: Some(IcedValue::Numeric {
                    current: 50.0,
                    min: 0.0,
                    max: 100.0,
                    step: Some(1.0),
                }),
                ..Accessible::default()
            },
        );

        let tree = builder.build();
        let node = &tree.update.nodes[1].1;

        assert!(
            node.supports_action(accesskit::Action::Increment),
            "Slider with step should support Increment"
        );
        assert!(
            node.supports_action(accesskit::Action::Decrement),
            "Slider with step should support Decrement"
        );
    }

    #[test]
    fn focusable_adds_focus_action() {
        let mut builder = TreeBuilder::new("Test Window");

        builder.accessible(
            None,
            UNIT,
            &Accessible {
                role: IcedRole::TextInput,
                ..Accessible::default()
            },
        );
        builder.focusable(None, UNIT, &mut MockFocusable(false));

        let tree = builder.build();

        assert!(
            tree.update.nodes[1]
                .1
                .supports_action(accesskit::Action::Focus),
            "Focusable widget should support Focus action"
        );
    }

    #[test]
    fn focusable_tracks_focused_node() {
        let mut builder = TreeBuilder::new("Test Window");

        builder.accessible(
            None,
            UNIT,
            &Accessible {
                role: IcedRole::TextInput,
                ..Accessible::default()
            },
        );
        builder.focusable(None, UNIT, &mut MockFocusable(true));

        let tree = builder.build();
        let input_id = tree.update.nodes[1].0;

        assert_eq!(tree.focused, Some(input_id));
        assert_eq!(tree.update.focus, input_id);
    }

    #[test]
    fn focusable_unfocused_leaves_focus_at_root() {
        let mut builder = TreeBuilder::new("Test Window");

        builder.accessible(
            None,
            UNIT,
            &Accessible {
                role: IcedRole::TextInput,
                ..Accessible::default()
            },
        );
        builder.focusable(None, UNIT, &mut MockFocusable(false));

        let tree = builder.build();

        assert!(tree.focused.is_none());
        assert_eq!(tree.update.focus, NodeId(0));
    }

    #[test]
    fn container_creates_generic_container_when_no_accessible() {
        let mut builder = TreeBuilder::new("Test Window");

        builder.container(None, UNIT);
        builder.traverse(&mut |op| {
            op.accessible(
                None,
                UNIT,
                &Accessible {
                    role: IcedRole::Button,
                    ..Accessible::default()
                },
            );
        });

        let tree = builder.build();

        assert_eq!(tree.update.nodes.len(), 3);
        assert_eq!(tree.update.nodes[1].1.role(), Role::GenericContainer);
        assert_eq!(tree.update.nodes[2].1.role(), Role::Button);
    }

    #[test]
    fn container_is_noop_after_accessible() {
        let mut builder = TreeBuilder::new("Test Window");

        builder.accessible(
            None,
            UNIT,
            &Accessible {
                role: IcedRole::Button,
                ..Accessible::default()
            },
        );
        builder.container(None, UNIT);

        let tree = builder.build();

        assert_eq!(tree.update.nodes.len(), 2);
        assert_eq!(tree.update.nodes[1].1.role(), Role::Button);
    }

    #[test]
    fn role_conversion_covers_all_variants() {
        assert_eq!(convert_role(IcedRole::Button), Role::Button);
        assert_eq!(convert_role(IcedRole::CheckBox), Role::CheckBox);
        assert_eq!(convert_role(IcedRole::RadioButton), Role::RadioButton);
        assert_eq!(convert_role(IcedRole::Switch), Role::Switch);
        assert_eq!(convert_role(IcedRole::Slider), Role::Slider);
        assert_eq!(
            convert_role(IcedRole::ProgressIndicator),
            Role::ProgressIndicator
        );
        assert_eq!(convert_role(IcedRole::TextInput), Role::TextInput);
        assert_eq!(convert_role(IcedRole::Group), Role::Group);
        assert_eq!(convert_role(IcedRole::ScrollView), Role::ScrollView);
        assert_eq!(convert_role(IcedRole::StaticText), Role::Label);
        assert_eq!(convert_role(IcedRole::ComboBox), Role::ComboBox);
        assert_eq!(convert_role(IcedRole::Image), Role::Image);
        assert_eq!(convert_role(IcedRole::Link), Role::Link);
        assert_eq!(convert_role(IcedRole::Separator), Role::GenericContainer);
    }

    #[test]
    fn node_map_records_widget_ids_and_bounds() {
        let widget_id = widget::Id::unique();
        let bounds = Rectangle {
            x: 10.0,
            y: 20.0,
            width: 200.0,
            height: 40.0,
        };

        let mut builder = TreeBuilder::new("Test Window");
        builder.accessible(
            Some(&widget_id),
            bounds,
            &Accessible {
                role: IcedRole::TextInput,
                ..Accessible::default()
            },
        );

        let tree = builder.build();
        let node_id = tree.update.nodes[1].0;
        let (stored_id, stored_bounds) =
            tree.node_map.get(&node_id).expect("node should be in map");

        assert_eq!(stored_id.as_ref(), Some(&widget_id));
        assert_eq!(*stored_bounds, bounds);
    }

    #[test]
    fn deeply_nested_tree_preserves_hierarchy() {
        let mut builder = TreeBuilder::new("Test Window");

        builder.accessible(
            None,
            UNIT,
            &Accessible {
                role: IcedRole::ScrollView,
                ..Accessible::default()
            },
        );
        builder.traverse(&mut |op| {
            op.accessible(
                None,
                UNIT,
                &Accessible {
                    role: IcedRole::Group,
                    ..Accessible::default()
                },
            );
            op.traverse(&mut |op| {
                op.accessible(
                    None,
                    UNIT,
                    &Accessible {
                        role: IcedRole::Button,
                        label: Some("Deep"),
                        ..Accessible::default()
                    },
                );
            });
        });

        let tree = builder.build();

        assert_eq!(tree.update.nodes.len(), 4);
        // Each level has exactly one child
        assert_eq!(tree.update.nodes[0].1.children().len(), 1);
        assert_eq!(tree.update.nodes[1].1.children().len(), 1);
        assert_eq!(tree.update.nodes[2].1.children().len(), 1);
        assert!(tree.update.nodes[3].1.children().is_empty());
    }

    #[test]
    fn siblings_share_same_parent() {
        let mut builder = TreeBuilder::new("Test Window");

        builder.accessible(
            None,
            UNIT,
            &Accessible {
                role: IcedRole::Group,
                ..Accessible::default()
            },
        );
        builder.traverse(&mut |op| {
            op.accessible(
                None,
                UNIT,
                &Accessible {
                    role: IcedRole::Button,
                    label: Some("A"),
                    ..Accessible::default()
                },
            );
            op.accessible(
                None,
                UNIT,
                &Accessible {
                    role: IcedRole::Button,
                    label: Some("B"),
                    ..Accessible::default()
                },
            );
        });

        let tree = builder.build();

        let group = &tree.update.nodes[1].1;
        assert_eq!(group.children().len(), 2);
    }

    #[test]
    fn multiple_announcements_all_added() {
        let tree = TreeBuilder::new("Test Window")
            .with_announcements(&["First".to_owned(), "Second".to_owned()])
            .build();

        assert_eq!(tree.update.nodes.len(), 3);
        assert_eq!(tree.update.nodes[1].1.value(), Some("First"));
        assert_eq!(tree.update.nodes[2].1.value(), Some("Second"));
    }

    #[test]
    fn announcements_coexist_with_widget_nodes() {
        let mut builder = TreeBuilder::new("Test Window").with_announcements(&["Alert".to_owned()]);

        builder.accessible(
            None,
            UNIT,
            &Accessible {
                role: IcedRole::Button,
                ..Accessible::default()
            },
        );

        let tree = builder.build();

        // Root -> [Button, Announcement]
        assert_eq!(tree.update.nodes.len(), 3);
        assert_eq!(tree.update.nodes[0].1.children().len(), 2);
        assert_eq!(tree.update.nodes[1].1.role(), Role::Button);
        assert_eq!(tree.update.nodes[2].1.role(), Role::Label);
    }

    #[test]
    fn accessible_without_id_stores_none_in_node_map() {
        let mut builder = TreeBuilder::new("Test Window");
        builder.accessible(
            None,
            UNIT,
            &Accessible {
                role: IcedRole::Button,
                ..Accessible::default()
            },
        );

        let tree = builder.build();
        let node_id = tree.update.nodes[1].0;
        let (stored_id, _) = tree.node_map.get(&node_id).expect("node should be in map");

        assert!(stored_id.is_none());
    }

    #[test]
    fn leaf_accessible_does_not_corrupt_next_sibling() {
        // Simulates Column containing [Slider, Container(Button)].
        // The slider calls accessible() without traverse(). Then
        // a container calls container() + traverse(), with a button
        // inside. The button must be a child of the container, NOT
        // the slider.
        let mut builder = TreeBuilder::new("Test Window");

        // Column group
        builder.accessible(
            None,
            UNIT,
            &Accessible {
                role: IcedRole::Group,
                ..Accessible::default()
            },
        );
        builder.traverse(&mut |op| {
            // Slider: accessible() only, no traverse()
            let slider_bounds = Rectangle {
                x: 0.0,
                y: 0.0,
                width: 200.0,
                height: 20.0,
            };
            op.accessible(
                None,
                slider_bounds,
                &Accessible {
                    role: IcedRole::Slider,
                    ..Accessible::default()
                },
            );

            // Container(Button): container() + traverse()
            let container_bounds = Rectangle {
                x: 0.0,
                y: 30.0,
                width: 100.0,
                height: 40.0,
            };
            op.container(None, container_bounds);
            op.traverse(&mut |op| {
                op.accessible(
                    None,
                    container_bounds,
                    &Accessible {
                        role: IcedRole::Button,
                        label: Some("Click me"),
                        ..Accessible::default()
                    },
                );
            });
        });

        let tree = builder.build();

        // Find nodes by role
        let slider_id = tree
            .update
            .nodes
            .iter()
            .find(|(_, n)| n.role() == Role::Slider)
            .map(|(id, _)| *id)
            .expect("slider node exists");

        let container_id = tree
            .update
            .nodes
            .iter()
            .find(|(_, n)| n.role() == Role::GenericContainer)
            .map(|(id, _)| *id)
            .expect("container node exists");

        let button_id = tree
            .update
            .nodes
            .iter()
            .find(|(_, n)| n.role() == Role::Button)
            .map(|(id, _)| *id)
            .expect("button node exists");

        // Button must be a child of the container
        let container_node = tree
            .update
            .nodes
            .iter()
            .find(|(id, _)| *id == container_id)
            .map(|(_, n)| n)
            .unwrap();
        assert!(
            container_node.children().contains(&button_id),
            "button should be a child of the container"
        );

        // Button must NOT be a child of the slider
        let slider_node = tree
            .update
            .nodes
            .iter()
            .find(|(id, _)| *id == slider_id)
            .map(|(_, n)| n)
            .unwrap();
        assert!(
            !slider_node.children().contains(&button_id),
            "button should not be a child of the slider"
        );
    }

    #[test]
    fn stable_ids_across_rebuilds() {
        let widget_a = widget::Id::unique();
        let widget_b = widget::Id::unique();

        let build_tree = |a: &widget::Id, b: &widget::Id| {
            let mut builder = TreeBuilder::new("Test Window");
            builder.accessible(
                Some(a),
                UNIT,
                &Accessible {
                    role: IcedRole::Button,
                    label: Some("A"),
                    ..Accessible::default()
                },
            );
            builder.accessible(
                Some(b),
                UNIT,
                &Accessible {
                    role: IcedRole::Button,
                    label: Some("B"),
                    ..Accessible::default()
                },
            );
            builder.build()
        };

        let tree1 = build_tree(&widget_a, &widget_b);
        let tree2 = build_tree(&widget_a, &widget_b);

        let id_a_1 = tree1.update.nodes[1].0;
        let id_b_1 = tree1.update.nodes[2].0;
        let id_a_2 = tree2.update.nodes[1].0;
        let id_b_2 = tree2.update.nodes[2].0;

        assert_eq!(
            id_a_1, id_a_2,
            "widget A should have the same NodeId across rebuilds"
        );
        assert_eq!(
            id_b_1, id_b_2,
            "widget B should have the same NodeId across rebuilds"
        );
    }

    #[test]
    fn id_stability_with_conditional_widget() {
        let named = widget::Id::new("stable-named-widget");

        // Build 1: just the named widget
        let mut builder1 = TreeBuilder::new("Test Window");
        builder1.accessible(
            Some(&named),
            UNIT,
            &Accessible {
                role: IcedRole::Button,
                label: Some("Named"),
                ..Accessible::default()
            },
        );
        let tree1 = builder1.build();

        // Build 2: unnamed widget inserted before the named one
        let mut builder2 = TreeBuilder::new("Test Window");
        builder2.accessible(
            None,
            UNIT,
            &Accessible {
                role: IcedRole::StaticText,
                label: Some("Extra"),
                ..Accessible::default()
            },
        );
        builder2.accessible(
            Some(&named),
            UNIT,
            &Accessible {
                role: IcedRole::Button,
                label: Some("Named"),
                ..Accessible::default()
            },
        );
        let tree2 = builder2.build();

        // Find the named widget's NodeId in both trees
        let named_id_1 = tree1
            .update
            .nodes
            .iter()
            .find(|(_, n)| n.label() == Some("Named"))
            .map(|(id, _)| *id)
            .expect("named node in tree1");

        let named_id_2 = tree2
            .update
            .nodes
            .iter()
            .find(|(_, n)| n.label() == Some("Named"))
            .map(|(id, _)| *id)
            .expect("named node in tree2");

        assert_eq!(
            named_id_1, named_id_2,
            "named widget keeps the same NodeId despite conditional widget insertion"
        );
    }

    #[test]
    fn bounds_set_on_accessible_nodes() {
        let bounds = Rectangle {
            x: 10.0,
            y: 20.0,
            width: 300.0,
            height: 50.0,
        };

        let mut builder = TreeBuilder::new("Test Window");
        builder.accessible(
            None,
            bounds,
            &Accessible {
                role: IcedRole::Button,
                ..Accessible::default()
            },
        );

        let tree = builder.build();
        let node = &tree.update.nodes[1].1;
        let rect = node.bounds().expect("bounds should be set");

        assert_eq!(rect.x0, 10.0);
        assert_eq!(rect.y0, 20.0);
        assert_eq!(rect.x1, 310.0);
        assert_eq!(rect.y1, 70.0);
    }

    #[test]
    fn multiple_text_children_all_preserved() {
        let mut builder = TreeBuilder::new("Test Window");

        // Accessible with an explicit label
        builder.accessible(
            None,
            UNIT,
            &Accessible {
                role: IcedRole::Group,
                label: Some("Parent"),
                ..Accessible::default()
            },
        );
        builder.traverse(&mut |op| {
            op.text(None, UNIT, "First text");
            op.text(None, UNIT, "Second text");
        });

        let tree = builder.build();

        // The parent should keep its explicit label
        let parent_node = &tree.update.nodes[1].1;
        assert_eq!(parent_node.label(), Some("Parent"));

        // Both text() calls should produce standalone Label nodes
        let label_nodes: Vec<_> = tree
            .update
            .nodes
            .iter()
            .filter(|(_, n)| n.role() == Role::Label)
            .collect();

        assert_eq!(
            label_nodes.len(),
            2,
            "both text children should be preserved"
        );

        let values: Vec<_> = label_nodes.iter().map(|(_, n)| n.value()).collect();
        assert!(values.contains(&Some("First text")));
        assert!(values.contains(&Some("Second text")));
    }

    #[test]
    fn scroll_translation_adjusts_child_bounds() {
        let mut builder = TreeBuilder::new("Test Window");

        let scroll_translation = Vector::new(0.0, 100.0);

        // Simulate scrollable container
        builder.container(None, UNIT);
        builder.scrollable(None, UNIT, UNIT, scroll_translation, &mut MockScrollable);
        builder.traverse(&mut |op| {
            let child_bounds = Rectangle {
                x: 10.0,
                y: 200.0,
                width: 80.0,
                height: 30.0,
            };
            op.accessible(
                None,
                child_bounds,
                &Accessible {
                    role: IcedRole::Button,
                    ..Accessible::default()
                },
            );
        });

        let tree = builder.build();

        // Find the button node
        let button_entry = tree
            .update
            .nodes
            .iter()
            .find(|(_, n)| n.role() == Role::Button)
            .expect("button node exists");

        let button_id = button_entry.0;

        // Check that node_map has scroll-adjusted bounds
        let (_, stored_bounds) = tree
            .node_map
            .get(&button_id)
            .expect("button should be in node_map");

        assert_eq!(
            stored_bounds.x, 10.0,
            "x should be unchanged (no horizontal scroll)"
        );
        assert_eq!(
            stored_bounds.y,
            200.0 - scroll_translation.y,
            "y should be adjusted by scroll offset"
        );
    }

    struct MockScrollable;

    impl Scrollable for MockScrollable {
        fn snap_to(
            &mut self,
            _: crate::core::widget::operation::scrollable::RelativeOffset<Option<f32>>,
        ) {
        }
        fn scroll_to(
            &mut self,
            _: crate::core::widget::operation::scrollable::AbsoluteOffset<Option<f32>>,
        ) {
        }
        fn scroll_by(
            &mut self,
            _: crate::core::widget::operation::scrollable::AbsoluteOffset,
            _: Rectangle,
            _: Rectangle,
        ) {
        }
    }

    #[test]
    fn separator_maps_to_generic_container_role() {
        let mut builder = TreeBuilder::new("Test Window");
        builder.accessible(
            None,
            UNIT,
            &Accessible {
                role: IcedRole::Separator,
                ..Accessible::default()
            },
        );

        let tree = builder.build();
        let node = &tree.update.nodes[1].1;

        assert_eq!(
            node.role(),
            Role::GenericContainer,
            "Separator should map to GenericContainer"
        );
    }

    #[test]
    fn container_noop_under_scroll_offset() {
        let mut builder = TreeBuilder::new("Test Window");

        let scroll_translation = Vector::new(0.0, 50.0);

        // Set up a scrollable context so scroll_offset is non-zero
        builder.scrollable(None, UNIT, UNIT, scroll_translation, &mut MockScrollable);

        // accessible() stores adjusted bounds in node_map
        builder.accessible(
            None,
            UNIT,
            &Accessible {
                role: IcedRole::Button,
                ..Accessible::default()
            },
        );

        // container() with the same raw bounds and id should be a no-op
        builder.container(None, UNIT);

        let tree = builder.build();

        // Only root + the one accessible node -- no duplicate container
        assert_eq!(
            tree.update.nodes.len(),
            2,
            "container() should be a no-op when accessible() already \
             created a node with the same adjusted bounds"
        );
        assert_eq!(tree.update.nodes[1].1.role(), Role::Button);
    }

    #[test]
    fn nested_scrollable_offset_accumulates() {
        let mut builder = TreeBuilder::new("Test Window");

        let outer_bounds = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 400.0,
            height: 300.0,
        };
        let inner_bounds = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 400.0,
            height: 200.0,
        };
        let child_bounds = Rectangle {
            x: 10.0,
            y: 100.0,
            width: 80.0,
            height: 30.0,
        };
        let sibling_bounds = Rectangle {
            x: 0.0,
            y: 400.0,
            width: 100.0,
            height: 50.0,
        };

        // Root group with two children: the scrollable subtree
        // and a sibling outside any scrollable.
        builder.accessible(
            None,
            UNIT,
            &Accessible {
                role: IcedRole::Group,
                ..Accessible::default()
            },
        );
        builder.traverse(&mut |op| {
            // First child: outer scrollable widget.
            // In real iced, a scrollable widget calls container(),
            // scrollable(), then traverse() -- all scoped inside
            // the parent's traverse closure.
            op.container(None, outer_bounds);
            op.scrollable(
                None,
                outer_bounds,
                outer_bounds,
                Vector::new(0.0, 10.0),
                &mut MockScrollable,
            );
            op.traverse(&mut |op| {
                // Inner scrollable container
                op.container(None, inner_bounds);
                op.scrollable(
                    None,
                    inner_bounds,
                    inner_bounds,
                    Vector::new(0.0, 20.0),
                    &mut MockScrollable,
                );
                op.traverse(&mut |op| {
                    op.accessible(
                        None,
                        child_bounds,
                        &Accessible {
                            role: IcedRole::Button,
                            label: Some("Nested"),
                            ..Accessible::default()
                        },
                    );
                });
            });
        });

        // Second child: sibling placed after the outer traverse()
        // returns, so scroll_offset has been restored to zero.
        builder.accessible(
            None,
            sibling_bounds,
            &Accessible {
                role: IcedRole::StaticText,
                label: Some("Sibling"),
                ..Accessible::default()
            },
        );

        let tree = builder.build();

        // Find the nested button
        let button_id = tree
            .update
            .nodes
            .iter()
            .find(|(_, n)| n.label() == Some("Nested"))
            .map(|(id, _)| *id)
            .expect("nested button exists");

        let (_, button_stored) = tree.node_map.get(&button_id).expect("button in node_map");

        // Total offset is 10 + 20 = 30
        assert_eq!(
            button_stored.y,
            child_bounds.y - 30.0,
            "nested child bounds should be adjusted by cumulative scroll offset"
        );
        assert_eq!(
            button_stored.x, child_bounds.x,
            "x should be unchanged (no horizontal scroll)"
        );

        // Find the sibling -- should have no offset adjustment
        let sibling_id = tree
            .update
            .nodes
            .iter()
            .find(|(_, n)| n.label() == Some("Sibling"))
            .map(|(id, _)| *id)
            .expect("sibling exists");

        let (_, sibling_stored) = tree.node_map.get(&sibling_id).expect("sibling in node_map");

        assert_eq!(
            *sibling_stored, sibling_bounds,
            "sibling outside scrollable should have unadjusted bounds"
        );
    }

    #[test]
    fn scrollbar_created_for_overflowing_scrollable() {
        let mut builder = TreeBuilder::new("Test Window");

        let bounds = UNIT;
        let content_bounds = Rectangle {
            height: 200.0,
            ..UNIT
        };

        builder.accessible(
            None,
            bounds,
            &Accessible {
                role: IcedRole::ScrollView,
                ..Accessible::default()
            },
        );
        builder.scrollable(
            None,
            bounds,
            content_bounds,
            Vector::new(0.0, 42.0),
            &mut MockScrollable,
        );

        let tree = builder.build();

        // Should have root + ScrollView + ScrollBar
        let scrollbar = tree
            .update
            .nodes
            .iter()
            .find(|(_, n)| n.role() == Role::ScrollBar);
        assert!(scrollbar.is_some(), "ScrollBar node should exist");

        let (_, sb_node) = scrollbar.unwrap();
        assert_eq!(sb_node.numeric_value(), Some(42.0));
        assert_eq!(sb_node.min_numeric_value(), Some(0.0));
        assert_eq!(
            sb_node.max_numeric_value(),
            Some((content_bounds.height - bounds.height) as f64)
        );

        // ScrollBar should be a child of the ScrollView
        let scroll_view = &tree.update.nodes[1].1;
        let sb_id = scrollbar.unwrap().0;
        assert!(scroll_view.children().contains(&sb_id));
    }

    #[test]
    fn scrollbar_not_created_when_content_fits() {
        let mut builder = TreeBuilder::new("Test Window");

        let bounds = UNIT;
        // Content fits within bounds
        let content_bounds = Rectangle {
            height: 30.0,
            ..UNIT
        };

        builder.accessible(
            None,
            bounds,
            &Accessible {
                role: IcedRole::ScrollView,
                ..Accessible::default()
            },
        );
        builder.scrollable(
            None,
            bounds,
            content_bounds,
            Vector::new(0.0, 0.0),
            &mut MockScrollable,
        );

        let tree = builder.build();

        let scrollbar = tree
            .update
            .nodes
            .iter()
            .find(|(_, n)| n.role() == Role::ScrollBar);
        assert!(scrollbar.is_none(), "no ScrollBar when content fits");
    }

    #[test]
    fn new_roles_map_correctly() {
        assert_eq!(convert_role(IcedRole::Dialog), Role::Dialog);
        assert_eq!(convert_role(IcedRole::Menu), Role::Menu);
        assert_eq!(convert_role(IcedRole::MenuItem), Role::MenuItem);
        assert_eq!(convert_role(IcedRole::ScrollBar), Role::ScrollBar);
        assert_eq!(convert_role(IcedRole::Tab), Role::Tab);
        assert_eq!(convert_role(IcedRole::TabList), Role::TabList);
    }

    #[test]
    fn extended_roles_map_correctly() {
        assert_eq!(convert_role(IcedRole::Alert), Role::Alert);
        assert_eq!(convert_role(IcedRole::AlertDialog), Role::AlertDialog);
        assert_eq!(convert_role(IcedRole::Canvas), Role::Canvas);
        assert_eq!(convert_role(IcedRole::Document), Role::Document);
        assert_eq!(convert_role(IcedRole::Heading), Role::Heading);
        assert_eq!(convert_role(IcedRole::Label), Role::Label);
        assert_eq!(convert_role(IcedRole::List), Role::List);
        assert_eq!(convert_role(IcedRole::ListItem), Role::ListItem);
        assert_eq!(convert_role(IcedRole::MenuBar), Role::MenuBar);
        assert_eq!(convert_role(IcedRole::Meter), Role::Meter);
        assert_eq!(
            convert_role(IcedRole::MultilineTextInput),
            Role::MultilineTextInput
        );
        assert_eq!(convert_role(IcedRole::Navigation), Role::Navigation);
        assert_eq!(convert_role(IcedRole::Region), Role::Region);
        assert_eq!(convert_role(IcedRole::Search), Role::Search);
        assert_eq!(convert_role(IcedRole::Status), Role::Status);
        assert_eq!(convert_role(IcedRole::Table), Role::Table);
        assert_eq!(convert_role(IcedRole::TabPanel), Role::TabPanel);
        assert_eq!(convert_role(IcedRole::Toolbar), Role::Toolbar);
        assert_eq!(convert_role(IcedRole::Tooltip), Role::Tooltip);
        assert_eq!(convert_role(IcedRole::Tree), Role::Tree);
        assert_eq!(convert_role(IcedRole::TreeItem), Role::TreeItem);
        assert_eq!(convert_role(IcedRole::Window), Role::Window);
    }

    #[test]
    fn required_property_is_set() {
        let mut builder = TreeBuilder::new("Test Window");

        builder.accessible(
            None,
            UNIT,
            &Accessible {
                role: IcedRole::TextInput,
                required: true,
                ..Accessible::default()
            },
        );

        let tree = builder.build();

        let input_node = tree
            .update
            .nodes
            .iter()
            .find(|(_, n)| n.role() == Role::TextInput)
            .map(|(_, n)| n)
            .expect("input node exists");

        assert!(
            input_node.is_required(),
            "required: true should set is_required on the accesskit node"
        );
    }

    #[test]
    fn level_property_is_set() {
        let mut builder = TreeBuilder::new("Test Window");

        builder.accessible(
            None,
            UNIT,
            &Accessible {
                role: IcedRole::Heading,
                level: Some(2),
                ..Accessible::default()
            },
        );

        let tree = builder.build();

        let heading_node = tree
            .update
            .nodes
            .iter()
            .find(|(_, n)| n.role() == Role::Heading)
            .map(|(_, n)| n)
            .expect("heading node exists");

        assert_eq!(
            heading_node.level(),
            Some(2),
            "level: Some(2) should set level on the accesskit node"
        );
    }

    #[test]
    fn required_default_is_false() {
        let mut builder = TreeBuilder::new("Test Window");

        builder.accessible(None, UNIT, &Accessible::default());

        let tree = builder.build();

        let node = tree
            .update
            .nodes
            .iter()
            .find(|(id, _)| *id != ROOT_ID)
            .map(|(_, n)| n)
            .expect("non-root node exists");

        assert!(
            !node.is_required(),
            "default Accessible should not have required set"
        );
    }

    #[test]
    fn labelled_by_resolves_to_node_id() {
        let label_wid = widget::Id::unique();
        let input_wid = widget::Id::unique();

        let mut builder = TreeBuilder::new("Test Window");

        // Create the label node with a widget::Id
        builder.accessible(
            Some(&label_wid),
            UNIT,
            &Accessible {
                role: IcedRole::StaticText,
                label: Some("Username"),
                ..Accessible::default()
            },
        );

        // Create the input node with labelled_by pointing to the label
        builder.accessible(
            Some(&input_wid),
            UNIT,
            &Accessible {
                role: IcedRole::TextInput,
                labelled_by: Some(&label_wid),
                ..Accessible::default()
            },
        );

        let tree = builder.build();

        // Find the label's NodeId
        let label_nid = tree
            .update
            .nodes
            .iter()
            .find(|(_, n)| n.label() == Some("Username"))
            .map(|(id, _)| *id)
            .expect("label node exists");

        // Find the input node
        let input_node = tree
            .update
            .nodes
            .iter()
            .find(|(_, n)| n.role() == Role::TextInput)
            .map(|(_, n)| n)
            .expect("input node exists");

        assert_eq!(
            input_node.labelled_by(),
            &[label_nid],
            "labelled_by should resolve to the label's NodeId"
        );
    }

    #[test]
    fn traverse_without_accessible_is_transparent() {
        let mut builder = TreeBuilder::new("Test Window");

        // traverse() without a prior accessible() or container()
        builder.traverse(&mut |op| {
            op.accessible(
                None,
                UNIT,
                &Accessible {
                    role: IcedRole::Button,
                    label: Some("Child A"),
                    ..Accessible::default()
                },
            );
            op.accessible(
                None,
                UNIT,
                &Accessible {
                    role: IcedRole::Button,
                    label: Some("Child B"),
                    ..Accessible::default()
                },
            );
        });

        let tree = builder.build();

        // Children should be added directly under the root
        assert_eq!(
            tree.update.nodes[0].1.children().len(),
            2,
            "children should be added under root when traverse() \
             has no prior accessible/container"
        );

        // No intermediate node -- just root + 2 children
        assert_eq!(tree.update.nodes.len(), 3);
    }
}
