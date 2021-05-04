//! Forces dynamic linking of Iced.
//!
//! Dynamically linking Iced makes the "link" step much faster. This can be achieved by adding
//! `iced_dynamic` as dependency and `#[allow(unused_imports)] use iced_dynamic` to `main.rs`. It is
//! recommended to disable the `iced_dynamic` dependency in release mode by adding
//! `#[cfg(debug_assertions)]` to the `use` statement. Otherwise you will have to ship `libstd.so`
//! and `libiced_dynamic.so` with your game.

// Force linking of the main iced crate
#[allow(unused_imports)]
#[allow(clippy::single_component_path_imports)]
use iced_internal;