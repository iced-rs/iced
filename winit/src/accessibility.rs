//! Accessibility adapter handlers for accesskit_winit integration.

use crate::Proxy;
use crate::runtime::Action;
use crate::runtime::accessibility;

use accesskit::{
    ActionHandler, ActionRequest, ActivationHandler, DeactivationHandler,
    TreeUpdate,
};

use std::sync::{Arc, Mutex};

/// Shared state for accessibility handlers.
///
/// This allows handlers (which can be called from any thread) to communicate
/// back to the main event loop via the iced Proxy.
pub struct HandlerState<Message: 'static> {
    proxy: Proxy<Message>,
    /// The current tree update, set by the main thread after UI rebuilds.
    /// Handlers can read this to provide the initial tree.
    current_tree: Arc<Mutex<Option<TreeUpdate>>>,
}

impl<Message> HandlerState<Message> {
    pub fn new(proxy: Proxy<Message>) -> Self {
        Self {
            proxy,
            current_tree: Arc::new(Mutex::new(None)),
        }
    }

    /// Updates the current accessibility tree.
    /// Called by the main thread after UI updates.
    pub fn update_tree(&self, tree: TreeUpdate) {
        if let Ok(mut current) = self.current_tree.lock() {
            *current = Some(tree);
        }
    }

    /// Gets a clone of the tree state for sharing with handlers.
    pub fn tree_state(&self) -> Arc<Mutex<Option<TreeUpdate>>> {
        Arc::clone(&self.current_tree)
    }
}

/// Handler for accessibility activation events.
pub struct IcedActivationHandler {
    tree_state: Arc<Mutex<Option<TreeUpdate>>>,
}

impl IcedActivationHandler {
    pub fn new(tree_state: Arc<Mutex<Option<TreeUpdate>>>) -> Self {
        Self { tree_state }
    }
}

impl ActivationHandler for IcedActivationHandler {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        // Return the current tree if available
        if let Ok(tree) = self.tree_state.lock() {
            tree.clone()
        } else {
            None
        }
    }
}

/// Handler for accessibility action requests.
pub struct IcedActionHandler<Message: 'static> {
    proxy: Proxy<Message>,
}

impl<Message> IcedActionHandler<Message> {
    pub fn new(proxy: Proxy<Message>) -> Self {
        Self { proxy }
    }
}

impl<Message: 'static> ActionHandler for IcedActionHandler<Message> {
    fn do_action(&mut self, request: ActionRequest) {
        // Send the action request to the main event loop
        self.proxy.send_action(Action::Accessibility(
            accessibility::Action::ActionRequested(request),
        ));
    }
}

/// Handler for accessibility deactivation events.
pub struct IcedDeactivationHandler<Message: 'static> {
    proxy: Proxy<Message>,
}

impl<Message> IcedDeactivationHandler<Message> {
    pub fn new(proxy: Proxy<Message>) -> Self {
        Self { proxy }
    }
}

impl<Message: 'static> DeactivationHandler
    for IcedDeactivationHandler<Message>
{
    fn deactivate_accessibility(&mut self) {
        // Notify the main event loop that accessibility was deactivated
        self.proxy.send_action(Action::Accessibility(
            accessibility::Action::Deactivated,
        ));
    }
}
