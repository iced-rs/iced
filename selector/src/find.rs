use crate::Selector;
use crate::core::widget::operation::{
    Focusable, Outcome, Scrollable, TextInput,
};
use crate::core::widget::{Id, Operation};
use crate::core::{Rectangle, Vector};
use crate::target::Candidate;

use std::any::Any;

/// An [`Operation`] that runs the [`Selector`] and stops after
/// the first [`Output`](Selector::Output) is produced.
pub type Find<S> = Finder<One<S>>;

/// An [`Operation`] that runs the [`Selector`] for the entire
/// widget tree and aggregates all of its [`Output`](Selector::Output).
pub type FindAll<S> = Finder<All<S>>;

#[derive(Debug)]
pub struct One<S>
where
    S: Selector,
{
    selector: S,
    output: Option<S::Output>,
}

impl<S> One<S>
where
    S: Selector,
{
    pub fn new(selector: S) -> Self {
        Self {
            selector,
            output: None,
        }
    }
}

impl<S> Strategy for One<S>
where
    S: Selector,
    S::Output: Clone,
{
    type Output = Option<S::Output>;

    fn feed(&mut self, target: Candidate<'_>) {
        if let Some(output) = self.selector.select(target) {
            self.output = Some(output);
        }
    }

    fn is_done(&self) -> bool {
        self.output.is_some()
    }

    fn finish(&self) -> Self::Output {
        self.output.clone()
    }
}

#[derive(Debug)]
pub struct All<S>
where
    S: Selector,
{
    selector: S,
    outputs: Vec<S::Output>,
}

impl<S> All<S>
where
    S: Selector,
{
    pub fn new(selector: S) -> Self {
        Self {
            selector,
            outputs: Vec::new(),
        }
    }
}

impl<S> Strategy for All<S>
where
    S: Selector,
    S::Output: Clone,
{
    type Output = Vec<S::Output>;

    fn feed(&mut self, target: Candidate<'_>) {
        if let Some(output) = self.selector.select(target) {
            self.outputs.push(output);
        }
    }

    fn is_done(&self) -> bool {
        false
    }

    fn finish(&self) -> Self::Output {
        self.outputs.clone()
    }
}

pub trait Strategy {
    type Output;

    fn feed(&mut self, target: Candidate<'_>);

    fn is_done(&self) -> bool;

    fn finish(&self) -> Self::Output;
}

#[derive(Debug)]
pub struct Finder<S> {
    strategy: S,
    stack: Vec<(Rectangle, Vector)>,
    viewport: Rectangle,
    translation: Vector,
}

impl<S> Finder<S> {
    pub fn new(strategy: S) -> Self {
        Self {
            strategy,
            stack: vec![(Rectangle::INFINITE, Vector::ZERO)],
            viewport: Rectangle::INFINITE,
            translation: Vector::ZERO,
        }
    }
}

impl<S> Operation<S::Output> for Finder<S>
where
    S: Strategy + Send,
    S::Output: Send,
{
    fn traverse(
        &mut self,
        operate: &mut dyn FnMut(&mut dyn Operation<S::Output>),
    ) {
        if self.strategy.is_done() {
            return;
        }

        self.stack.push((self.viewport, self.translation));
        operate(self);
        let _ = self.stack.pop();

        let (viewport, translation) = self.stack.last().unwrap();
        self.viewport = *viewport;
        self.translation = *translation;
    }

    fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
        if self.strategy.is_done() {
            return;
        }

        self.strategy.feed(Candidate::Container {
            id,
            bounds,
            visible_bounds: self
                .viewport
                .intersection(&(bounds + self.translation)),
        });
    }

    fn focusable(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        state: &mut dyn Focusable,
    ) {
        if self.strategy.is_done() {
            return;
        }

        self.strategy.feed(Candidate::Focusable {
            id,
            bounds,
            visible_bounds: self
                .viewport
                .intersection(&(bounds + self.translation)),
            state,
        });
    }

    fn scrollable(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        content_bounds: Rectangle,
        translation: Vector,
        state: &mut dyn Scrollable,
    ) {
        if self.strategy.is_done() {
            return;
        }

        let visible_bounds =
            self.viewport.intersection(&(bounds + self.translation));

        self.strategy.feed(Candidate::Scrollable {
            id,
            bounds,
            visible_bounds,
            content_bounds,
            translation,
            state,
        });

        self.translation = self.translation - translation;
        self.viewport = visible_bounds.unwrap_or_default();
    }

    fn text_input(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        state: &mut dyn TextInput,
    ) {
        if self.strategy.is_done() {
            return;
        }

        self.strategy.feed(Candidate::TextInput {
            id,
            bounds,
            visible_bounds: self
                .viewport
                .intersection(&(bounds + self.translation)),
            state,
        });
    }

    fn text(&mut self, id: Option<&Id>, bounds: Rectangle, text: &str) {
        if self.strategy.is_done() {
            return;
        }

        self.strategy.feed(Candidate::Text {
            id,
            bounds,
            visible_bounds: self
                .viewport
                .intersection(&(bounds + self.translation)),
            content: text,
        });
    }

    fn custom(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        state: &mut dyn Any,
    ) {
        if self.strategy.is_done() {
            return;
        }

        self.strategy.feed(Candidate::Custom {
            id,
            bounds,
            visible_bounds: self
                .viewport
                .intersection(&(bounds + self.translation)),
            state,
        });
    }

    fn finish(&self) -> Outcome<S::Output> {
        Outcome::Some(self.strategy.finish())
    }
}
