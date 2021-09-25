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
    /// the function is called from [`Element`] and [`Application::update`]
    pub fn enter(&mut self, id: &str) -> Entry<'_> {
        Entry::new(self)
            .enter(id)
    }
}

/// Access to the state at a lower level
/// This `struct` is constructed from the [`enter`] method on [`StateStorage`].
#[derive(Debug)]
pub struct Entry <'a> {
    storage: &'a mut StateStorage,
    is_enter: bool,
}

impl <'a> Entry<'a> {
    fn new(storage: &'a mut StateStorage) -> Self {
        Entry {
            storage, 
            is_enter: false,
        }
    }
    fn enter(mut self, id: &str) -> Self {
        self.storage.path.push(id);
        self.is_enter = true;
        self
    }
}

impl <'a> Drop for Entry <'a> {
    fn drop(&mut self) {
        if self.is_enter {
            let _ = self.path.pop();
        }
    }
}

impl <'a> std::ops::Deref for Entry<'a> {
    type Target = StateStorage;
    fn deref(&self) -> &StateStorage {
        &self.storage
    }
}

impl <'a> std::ops::DerefMut for Entry<'a> {
    fn deref_mut(&mut self) -> &mut StateStorage {
        &mut self.storage
    }
}
