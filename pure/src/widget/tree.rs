use crate::Element;

use std::any::{self, Any};

pub struct Tree {
    pub tag: Tag,
    pub state: State,
    pub children: Vec<Tree>,
}

impl Tree {
    pub fn empty() -> Self {
        Self {
            tag: Tag::stateless(),
            state: State::None,
            children: Vec::new(),
        }
    }

    pub fn new<Message, Renderer>(
        element: &Element<'_, Message, Renderer>,
    ) -> Self {
        Self {
            tag: element.as_widget().tag(),
            state: element.as_widget().state(),
            children: element.as_widget().children(),
        }
    }

    pub fn diff<Message, Renderer>(
        &mut self,
        new: &Element<'_, Message, Renderer>,
    ) {
        if self.tag == new.as_widget().tag() {
            new.as_widget().diff(self)
        } else {
            *self = Self::new(new);
        }
    }

    pub fn diff_children<Message, Renderer>(
        &mut self,
        new_children: &[Element<'_, Message, Renderer>],
    ) {
        self.diff_children_custom(
            new_children,
            |new, child_state| child_state.diff(new),
            Self::new,
        )
    }

    pub fn diff_children_custom<T>(
        &mut self,
        new_children: &[T],
        diff: impl Fn(&T, &mut Tree),
        new_state: impl Fn(&T) -> Self,
    ) {
        if self.children.len() > new_children.len() {
            self.children.truncate(new_children.len());
        }

        for (child_state, new) in
            self.children.iter_mut().zip(new_children.iter())
        {
            diff(new, child_state);
        }

        if self.children.len() < new_children.len() {
            self.children.extend(
                new_children[self.children.len()..].iter().map(new_state),
            );
        }
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Tag(any::TypeId);

impl Tag {
    pub fn of<T>() -> Self
    where
        T: 'static,
    {
        Self(any::TypeId::of::<T>())
    }

    pub fn stateless() -> Self {
        Self::of::<()>()
    }
}

pub enum State {
    None,
    Some(Box<dyn Any>),
}

impl State {
    pub fn new<T>(state: T) -> Self
    where
        T: 'static,
    {
        State::Some(Box::new(state))
    }

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
