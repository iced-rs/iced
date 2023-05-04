//! Store internal widget state in a state tree to ensure continuity.
use crate::id::{Id, Internal};
use crate::Widget;
use std::any::{self, Any};
use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

/// A persistent state widget tree.
///
/// A [`Tree`] is normally associated with a specific widget in the widget tree.
#[derive(Debug)]
pub struct Tree {
    /// The tag of the [`Tree`].
    pub tag: Tag,

    /// the Id of the [`Tree`]
    pub id: Option<Id>,

    /// The [`State`] of the [`Tree`].
    pub state: State,

    /// The children of the root widget of the [`Tree`].
    pub children: Vec<Tree>,
}

impl Tree {
    /// Creates an empty, stateless [`Tree`] with no children.
    pub fn empty() -> Self {
        Self {
            id: None,
            tag: Tag::stateless(),
            state: State::None,
            children: Vec::new(),
        }
    }

    /// Creates a new [`Tree`] for the provided [`Widget`].
    pub fn new<'a, Message, Renderer>(
        widget: impl Borrow<dyn Widget<Message, Renderer> + 'a>,
    ) -> Self
    where
        Renderer: crate::Renderer,
    {
        let widget = widget.borrow();

        Self {
            id: widget.id(),
            tag: widget.tag(),
            state: widget.state(),
            children: widget.children(),
        }
    }

    /// Reconciliates the current tree with the provided [`Widget`].
    ///
    /// If the tag of the [`Widget`] matches the tag of the [`Tree`], then the
    /// [`Widget`] proceeds with the reconciliation (i.e. [`Widget::diff`] is called).
    ///
    /// Otherwise, the whole [`Tree`] is recreated.
    ///
    /// [`Widget::diff`]: crate::Widget::diff
    pub fn diff<'a, Message, Renderer>(
        &mut self,
        mut new: impl BorrowMut<dyn Widget<Message, Renderer> + 'a>,
    ) where
        Renderer: crate::Renderer,
    {
        let borrowed: &mut dyn Widget<Message, Renderer> = new.borrow_mut();
        if self.tag == borrowed.tag() {
            // TODO can we take here?
            if let Some(id) = self.id.clone() {
                if matches!(id, Id(Internal::Custom(_, _))) {
                    borrowed.set_id(id);
                } else if borrowed.id() == Some(id.clone()) {
                    for (old_c, new_c) in
                        self.children.iter_mut().zip(borrowed.children())
                    {
                        old_c.id = new_c.id;
                    }
                } else {
                    borrowed.set_id(id);
                }
            }
            borrowed.diff(self)
        } else {
            *self = Self::new(new);
        }
    }

    /// Reconciliates the children of the tree with the provided list of widgets.
    pub fn diff_children<'a, Message, Renderer>(
        &mut self,
        new_children: &mut [impl BorrowMut<dyn Widget<Message, Renderer> + 'a>],
    ) where
        Renderer: crate::Renderer,
    {
        self.diff_children_custom(
            new_children,
            new_children.iter().map(|c| c.borrow().id()).collect(),
            |tree, widget| {
                let borrowed: &mut dyn Widget<_, _> = widget.borrow_mut();
                tree.diff(borrowed)
            },
            |widget| {
                let borrowed: &dyn Widget<_, _> = widget.borrow();
                Self::new(borrowed)
            },
        )
    }

    /// Reconciliates the children of the tree with the provided list of widgets using custom
    /// logic both for diffing and creating new widget state.
    pub fn diff_children_custom<T>(
        &mut self,
        new_children: &mut [T],
        new_ids: Vec<Option<Id>>,
        diff: impl Fn(&mut Tree, &mut T),
        new_state: impl Fn(&T) -> Self,
    ) {
        if self.children.len() > new_children.len() {
            self.children.truncate(new_children.len());
        }

        let len_changed = self.children.len() != new_children.len();

        let children_len = self.children.len();
        let (mut id_map, mut id_list): (
            HashMap<Id, &mut Tree>,
            Vec<&mut Tree>,
        ) = self.children.iter_mut().fold(
            (HashMap::new(), Vec::with_capacity(children_len)),
            |(mut id_map, mut id_list), c| {
                if let Some(id) = c.id.as_ref() {
                    if matches!(id.0, Internal::Custom(_, _)) {
                        let _ = id_map.insert(id.clone(), c);
                    } else {
                        id_list.push(c);
                    }
                } else {
                    id_list.push(c);
                }
                (id_map, id_list)
            },
        );

        let mut child_state_i = 0;
        for (new, new_id) in new_children.iter_mut().zip(new_ids.iter()) {
            let child_state = if let Some(c) =
                new_id.as_ref().and_then(|id| id_map.remove(id))
            {
                c
            } else if child_state_i < id_list.len() {
                let c = &mut id_list[child_state_i];
                if len_changed {
                    c.id = new_id.clone();
                }
                child_state_i += 1;
                c
            } else {
                continue;
            };

            diff(child_state, new);
        }

        if self.children.len() < new_children.len() {
            self.children.extend(
                new_children[self.children.len()..].iter().map(new_state),
            );
        }
    }
}

/// The identifier of some widget state.
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Tag(any::TypeId);

impl Tag {
    /// Creates a [`Tag`] for a state of type `T`.
    pub fn of<T>() -> Self
    where
        T: 'static,
    {
        Self(any::TypeId::of::<T>())
    }

    /// Creates a [`Tag`] for a stateless widget.
    pub fn stateless() -> Self {
        Self::of::<()>()
    }
}

/// The internal [`State`] of a widget.
pub enum State {
    /// No meaningful internal state.
    None,

    /// Some meaningful internal state.
    Some(Box<dyn Any>),
}

impl State {
    /// Creates a new [`State`].
    pub fn new<T>(state: T) -> Self
    where
        T: 'static,
    {
        State::Some(Box::new(state))
    }

    /// Downcasts the [`State`] to `T` and returns a reference to it.
    ///
    /// # Panics
    /// This method will panic if the downcast fails or the [`State`] is [`State::None`].
    pub fn downcast_ref<T>(&self) -> &T
    where
        T: 'static,
    {
        match self {
            State::None => panic!("Downcast on stateless state"),
            State::Some(state) => {
                state.downcast_ref().expect("Downcast widget state")
            }
        }
    }

    /// Downcasts the [`State`] to `T` and returns a mutable reference to it.
    ///
    /// # Panics
    /// This method will panic if the downcast fails or the [`State`] is [`State::None`].
    pub fn downcast_mut<T>(&mut self) -> &mut T
    where
        T: 'static,
    {
        match self {
            State::None => panic!("Downcast on stateless state"),
            State::Some(state) => {
                state.downcast_mut().expect("Downcast widget state")
            }
        }
    }
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "State::None"),
            Self::Some(_) => write!(f, "State::Some"),
        }
    }
}
