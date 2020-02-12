use crate::canvas::Frame;

pub trait Layer: std::fmt::Debug {}

use std::marker::PhantomData;
use std::sync::{Arc, Weak};

#[derive(Debug)]
pub struct Cached<T: Drawable> {
    input: PhantomData<T>,
}

impl<T> Cached<T>
where
    T: Drawable + std::fmt::Debug,
{
    pub fn new() -> Self {
        Cached { input: PhantomData }
    }

    pub fn clear(&mut self) {}

    pub fn with<'a>(&'a self, input: &'a T) -> impl Layer + 'a {
        Bind {
            cache: self,
            input: input,
        }
    }
}

#[derive(Debug)]
struct Bind<'a, T: Drawable> {
    cache: &'a Cached<T>,
    input: &'a T,
}

impl<'a, T> Layer for Bind<'a, T> where T: Drawable + std::fmt::Debug {}

pub trait Drawable {
    fn draw(&self, frame: &mut Frame);
}
