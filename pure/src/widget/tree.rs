use crate::widget::Element;

use std::any::{self, Any};

pub struct Tree {
    pub tag: any::TypeId,
    pub state: Box<dyn Any>,
    pub children: Vec<Tree>,
}

impl Tree {
    pub fn empty() -> Self {
        Self {
            tag: any::TypeId::of::<()>(),
            state: Box::new(()),
            children: Vec::new(),
        }
    }

    pub fn new<Message, Renderer>(
        element: &Element<'_, Message, Renderer>,
    ) -> Self {
        Self {
            tag: element.as_widget().tag(),
            state: element.as_widget().state(),
            children: element
                .as_widget()
                .children()
                .iter()
                .map(Self::new)
                .collect(),
        }
    }

    pub fn diff<Message, Renderer>(
        &mut self,
        new: &Element<'_, Message, Renderer>,
    ) {
        if self.tag == new.as_widget().tag() {
            let new_children = new.as_widget().children();

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
        } else {
            *self = Self::new(new);
        }
    }
}
