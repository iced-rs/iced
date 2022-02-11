use crate::widget::Element;

use std::any::Any;
use std::marker::PhantomData;

pub struct Tree<Message, Renderer> {
    pub state: Box<dyn Any>,
    pub children: Vec<Tree<Message, Renderer>>,
    types_: PhantomData<(Message, Renderer)>,
}

impl<Message, Renderer> Tree<Message, Renderer> {
    pub fn new(element: &Element<Message, Renderer>) -> Self {
        Self {
            state: element.as_widget().state(),
            children: element
                .as_widget()
                .children()
                .iter()
                .map(Self::new)
                .collect(),
            types_: PhantomData,
        }
    }

    pub fn diff(
        &mut self,
        current: &Element<Message, Renderer>,
        new: &Element<Message, Renderer>,
    ) {
        if current.as_widget().tag() == new.as_widget().tag() {
            let current_children = current.as_widget().children();
            let new_children = new.as_widget().children();

            if current_children.len() > new_children.len() {
                self.children.truncate(new_children.len());
            }

            for (child_state, (current, new)) in self
                .children
                .iter_mut()
                .zip(current_children.iter().zip(new_children.iter()))
            {
                child_state.diff(current, new);
            }

            if current_children.len() < new_children.len() {
                self.children.extend(
                    new_children[current_children.len()..]
                        .iter()
                        .map(Self::new),
                );
            }
        } else {
            *self = Self::new(new);
        }
    }
}
