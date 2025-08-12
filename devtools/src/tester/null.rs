use crate::Program;
use crate::core::window;
use crate::core::{Element, Theme};
use crate::futures::Subscription;
use crate::runtime::Task;
use crate::widget::horizontal_space;

use std::marker::PhantomData;

pub struct Tester<P: Program> {
    _type: PhantomData<P::Message>,
}

#[derive(Debug, Clone)]
pub enum Message {}

#[allow(missing_debug_implementations)]
pub struct Tick<P: Program> {
    _type: PhantomData<P::Message>,
}

impl<P: Program> Tester<P> {
    pub fn new(_program: &P) -> Self {
        Self { _type: PhantomData }
    }

    pub fn is_idle(&self) -> bool {
        true
    }

    pub fn is_busy(&self) -> bool {
        false
    }

    pub fn update(&mut self, _program: &P, _message: Message) -> Task<Tick<P>> {
        Task::none()
    }

    pub fn tick(&mut self, _program: &P, _tick: Tick<P>) -> Task<Tick<P>> {
        Task::none()
    }

    pub fn subscription(&self, _program: &P) -> Subscription<Tick<P>> {
        Subscription::none()
    }

    pub fn view<'a, T: 'static>(
        &'a self,
        _program: &P,
        _current: impl FnOnce() -> Element<'a, T, Theme, P::Renderer>,
        _emulate: impl Fn(Tick<P>) -> T + 'a,
    ) -> Element<'a, T, Theme, P::Renderer> {
        horizontal_space().into()
    }

    pub fn controls(&self) -> Element<'_, Message, Theme, P::Renderer> {
        horizontal_space().into()
    }
}
