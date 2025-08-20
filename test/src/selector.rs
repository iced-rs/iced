//! Select widgets of a user interface.
use crate::core::text;
use crate::core::widget;
use crate::core::{Rectangle, Vector};

/// A selector describes a strategy to find a certain widget in a user interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selector {
    /// Find the widget with the given [`widget::Id`].
    Id(widget::Id),
    /// Find the widget containing the given [`text::Fragment`].
    Text(text::Fragment<'static>),
}

impl Selector {
    pub fn operation<'a>(&self) -> impl widget::Operation<Vec<Target>> + 'a {
        match self {
            Selector::Id(id) => {
                struct FindById {
                    id: widget::Id,
                    target: Option<Target>,
                }

                impl widget::Operation<Vec<Target>> for FindById {
                    fn container(
                        &mut self,
                        id: Option<&widget::Id>,
                        bounds: Rectangle,
                        operate_on_children: &mut dyn FnMut(
                            &mut dyn widget::Operation<Vec<Target>>,
                        ),
                    ) {
                        if self.target.is_some() {
                            return;
                        }

                        if Some(&self.id) == id {
                            self.target = Some(Target { bounds });
                            return;
                        }

                        operate_on_children(self);
                    }

                    fn scrollable(
                        &mut self,
                        id: Option<&widget::Id>,
                        bounds: Rectangle,
                        _content_bounds: Rectangle,
                        _translation: Vector,
                        _state: &mut dyn widget::operation::Scrollable,
                    ) {
                        if self.target.is_some() {
                            return;
                        }

                        if Some(&self.id) == id {
                            self.target = Some(Target { bounds });
                        }
                    }

                    fn text_input(
                        &mut self,
                        id: Option<&widget::Id>,
                        bounds: Rectangle,
                        _state: &mut dyn widget::operation::TextInput,
                    ) {
                        if self.target.is_some() {
                            return;
                        }

                        if Some(&self.id) == id {
                            self.target = Some(Target { bounds });
                        }
                    }

                    fn text(
                        &mut self,
                        id: Option<&widget::Id>,
                        bounds: Rectangle,
                        _text: &str,
                    ) {
                        if self.target.is_some() {
                            return;
                        }

                        if Some(&self.id) == id {
                            self.target = Some(Target { bounds });
                        }
                    }

                    fn custom(
                        &mut self,
                        id: Option<&widget::Id>,
                        bounds: Rectangle,
                        _state: &mut dyn std::any::Any,
                    ) {
                        if self.target.is_some() {
                            return;
                        }

                        if Some(&self.id) == id {
                            self.target = Some(Target { bounds });
                        }
                    }

                    fn finish(
                        &self,
                    ) -> widget::operation::Outcome<Vec<Target>>
                    {
                        if let Some(target) = self.target {
                            widget::operation::Outcome::Some(vec![target])
                        } else {
                            widget::operation::Outcome::None
                        }
                    }
                }

                Box::new(FindById {
                    id: id.clone(),
                    target: None,
                }) as Box<dyn widget::Operation<_>>
            }
            Selector::Text(text) => {
                struct FindByText {
                    text: text::Fragment<'static>,
                    target: Vec<Target>,
                }

                impl widget::Operation<Vec<Target>> for FindByText {
                    fn container(
                        &mut self,
                        _id: Option<&widget::Id>,
                        _bounds: Rectangle,
                        operate_on_children: &mut dyn FnMut(
                            &mut dyn widget::Operation<Vec<Target>>,
                        ),
                    ) {
                        operate_on_children(self);
                    }

                    fn text_input(
                        &mut self,
                        _id: Option<&widget::Id>,
                        bounds: Rectangle,
                        state: &mut dyn widget::operation::TextInput,
                    ) {
                        if self.text == state.text() {
                            self.target.push(Target { bounds });
                        }
                    }

                    fn text(
                        &mut self,
                        _id: Option<&widget::Id>,
                        bounds: Rectangle,
                        text: &str,
                    ) {
                        if self.text == text {
                            self.target.push(Target { bounds });
                        }
                    }

                    fn finish(
                        &self,
                    ) -> widget::operation::Outcome<Vec<Target>>
                    {
                        widget::operation::Outcome::Some(self.target.clone())
                    }
                }

                Box::new(FindByText {
                    text: text.clone(),
                    target: Vec::new(),
                })
            }
        }
    }
}

impl From<widget::Id> for Selector {
    fn from(id: widget::Id) -> Self {
        Self::Id(id)
    }
}

impl From<&'static str> for Selector {
    fn from(text: &'static str) -> Self {
        Self::Text(text.into())
    }
}

/// Creates a [`Selector`] that finds the widget with the given [`widget::Id`].
pub fn id(id: impl Into<widget::Id>) -> Selector {
    Selector::Id(id.into())
}

/// Creates a [`Selector`] that finds the widget containing the given text fragment.
pub fn text(fragment: impl text::IntoFragment<'static>) -> Selector {
    Selector::Text(fragment.into_fragment())
}

/// A specific area, normally containing a widget.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Target {
    /// The bounds of the area.
    pub bounds: Rectangle,
}
