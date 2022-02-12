use crate::widget::Element;

use std::any::{self, Any};

pub struct Tree {
    pub tag: any::TypeId,
    pub state: State,
    pub children: Vec<Tree>,
}

impl Tree {
    pub fn empty() -> Self {
        Self {
            tag: any::TypeId::of::<()>(),
            state: State(Box::new(())),
            children: Vec::new(),
        }
    }

    pub fn new<Message, Renderer>(
        element: &Element<'_, Message, Renderer>,
    ) -> Self {
        Self {
            tag: element.as_widget().tag(),
            state: State(element.as_widget().state()),
            children: element.as_widget().children_state(),
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
        if self.children.len() > new_children.len() {
            self.children.truncate(new_children.len());
        }

        for (child_state, new) in
            self.children.iter_mut().zip(new_children.iter())
        {
            child_state.diff(new);
        }

        if self.children.len() < new_children.len() {
            self.children.extend(
                new_children[self.children.len()..].iter().map(Self::new),
            );
        }
    }
}

pub struct State(Box<dyn Any>);

impl State {
    pub fn downcast_ref<T>(&self) -> &T
    where
        T: 'static,
    {
        self.0.downcast_ref().expect("Downcast widget state")
    }

    pub fn downcast_mut<T>(&mut self) -> &mut T
    where
        T: 'static,
    {
        self.0.downcast_mut().expect("Downcast widget state")
    }
}
