//! Use storage state of widgets

use std::collections::HashMap;
use std::path::PathBuf;
use std::any::Any;

type StateBox = Box<dyn Any>;

/// Structure for storing widget states
#[derive(Debug, Default)]
pub struct StateStorage {
    states: HashMap<PathBuf, StateBox>,
    path: PathBuf,
}

impl StateStorage {
    /// Insert a state from widget. 
    /// Used for Widhet::into_states function
    #[track_caller]
    pub fn insert(&mut self, id: &str, state: StateBox) {
//         println!("insert: {}", id);
        if let Some(_s) = self.states.insert(self.path.join(id), state) {
            panic!("The state by ({}) already exists", id);
        }
    }
    
    /// Take the state by id
    pub fn take(&mut self, id: &str) -> Option<StateBox> {
        self.states.remove(&self.path.join(id))
    }
    
    /// Take the state by id and convert type
    pub fn take_state<S: Any>(&mut self, id: &str) -> Option<Box<S>> {
        self.states.remove(&self.path.join(id))
            .and_then(|s| s.downcast().ok())
    }
    
    /// Get the state ref by id
    pub fn get_ref<S: Any>(&self, id: &str) -> Option<&S> {
        self.states.get(&self.path.join(id))
            .and_then(|s| s.downcast_ref())
//             .as_ref()
    }
    
    /// Get the state mutable ref by id
    pub fn get_mut<S: Any>(&mut self, id: &str) -> Option<&mut S> {
        self.states.get_mut(&self.path.join(id))
            .and_then(|s| s.downcast_mut())
//             .as_mut()
    }
    
    /// Get the state mutable ref by id, or insert the default state
    pub fn get_mut_or_default<S: Any>(&mut self, id: &str) -> Option<&mut S> 
        where S: Default
    {
        self.states.entry(self.path.join(id))
            .or_insert(Box::new(S::default()))
            .downcast_mut()
    }
    
    /// Get the state mutable ref by id, or insert the new state
    pub fn get_mut_or_insert<S: Any>(&mut self, id: &str, s: S) -> Option<&mut S> 
    {
        self.states.entry(self.path.join(id))
            .or_insert(Box::new(s))
            .downcast_mut()
    }
    
    /// Entering a state group
    pub fn enter(&mut self, id: &str) {
        self.path.push(id);
    }
    /// Exit from the group
    pub fn exit(&mut self) {
        let _ = self.path.pop();
    }
}
