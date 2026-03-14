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
