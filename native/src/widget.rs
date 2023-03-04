//! Use the built-in widgets or create your own.
//!
//! # Built-in widgets
//! Every built-in drawable widget has its own module with a `Renderer` trait
//! that must be implemented by a [renderer] before being able to use it as
//! a [`Widget`].
//!
//! # Custom widgets
//! If you want to implement a custom widget, you simply need to implement the
//! [`Widget`] trait. You can use the API of the built-in widgets as a guide or
//! source of inspiration.
//!
//! [renderer]: crate::renderer
mod action;

pub use action::Action;
