//! Use the built-in widgets or create your own.
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/iced-rs/iced/9ab6923e943f784985e9ef9ca28b10278297225d/docs/logo.svg"
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
pub use iced_renderer as renderer;
pub use iced_renderer::core;
pub use iced_renderer::graphics;

pub use core::widget::{Id, Void};

mod action;
mod column;
mod mouse_area;
mod pin;
mod responsive;
mod stack;
mod themer;

pub mod button;
pub mod checkbox;
pub mod combo_box;
pub mod component;
pub mod container;
pub mod float;
pub mod grid;
pub mod keyed;
pub mod lazy;
pub mod overlay;
pub mod pane_grid;
pub mod pick_list;
pub mod progress_bar;
pub mod radio;
pub mod row;
pub mod rule;
pub mod scrollable;
pub mod sensor;
pub mod slider;
pub mod space;
pub mod sticky;
pub mod table;
pub mod text;
pub mod text_editor;
pub mod text_input;
pub mod toggler;
pub mod tooltip;
pub mod transition;
pub mod vertical_slider;

mod helpers;

pub use helpers::*;

#[doc(no_inline)]
pub use button::Button;
#[doc(no_inline)]
pub use checkbox::Checkbox;
#[doc(no_inline)]
pub use column::Column;
#[doc(no_inline)]
pub use combo_box::ComboBox;
#[doc(no_inline)]
pub use component::Component;
#[doc(no_inline)]
pub use container::Container;
#[doc(no_inline)]
pub use float::Float;
#[doc(no_inline)]
pub use grid::Grid;
#[doc(no_inline)]
pub use lazy::Lazy;
#[doc(no_inline)]
pub use mouse_area::MouseArea;
#[doc(no_inline)]
pub use pane_grid::PaneGrid;
#[doc(no_inline)]
pub use pick_list::PickList;
#[doc(no_inline)]
pub use pin::Pin;
#[doc(no_inline)]
pub use progress_bar::ProgressBar;
#[doc(no_inline)]
pub use radio::Radio;
#[doc(no_inline)]
pub use responsive::Responsive;
#[doc(no_inline)]
pub use row::Row;
#[doc(no_inline)]
pub use rule::Rule;
#[doc(no_inline)]
pub use scrollable::Scrollable;
#[doc(no_inline)]
pub use sensor::Sensor;
#[doc(no_inline)]
pub use slider::Slider;
#[doc(no_inline)]
pub use space::Space;
#[doc(no_inline)]
pub use stack::Stack;
#[doc(no_inline)]
pub use sticky::Sticky;
#[doc(no_inline)]
pub use text::Text;
#[doc(no_inline)]
pub use text_editor::TextEditor;
#[doc(no_inline)]
pub use text_input::TextInput;
#[doc(no_inline)]
pub use themer::Themer;
#[doc(no_inline)]
pub use toggler::Toggler;
#[doc(no_inline)]
pub use tooltip::Tooltip;
#[doc(no_inline)]
pub use vertical_slider::VerticalSlider;

#[cfg(feature = "wgpu")]
pub mod shader;

#[cfg(feature = "wgpu")]
#[doc(no_inline)]
pub use shader::Shader;

#[cfg(feature = "svg")]
pub mod svg;

#[cfg(feature = "svg")]
#[doc(no_inline)]
pub use svg::Svg;

#[cfg(feature = "image")]
pub mod image;

#[cfg(feature = "image")]
#[doc(no_inline)]
pub use image::Image;

#[cfg(feature = "canvas")]
pub mod canvas;

#[cfg(feature = "canvas")]
#[doc(no_inline)]
pub use canvas::Canvas;

#[cfg(feature = "qr_code")]
pub mod qr_code;

#[cfg(feature = "qr_code")]
#[doc(no_inline)]
pub use qr_code::QRCode;

#[cfg(feature = "markdown")]
pub mod markdown;

pub use crate::core::theme::{self, Theme};
pub use action::Action;
pub use renderer::Renderer;

use crate::core::Element;
use crate::core::widget;

/// TODO
pub trait Widget<Message, Theme = crate::Theme, Renderer = crate::Renderer>:
    core::Widget<Message, Theme, Renderer>
where
    Renderer: core::Renderer,
{
    /// Applies a transformation to the produced message of the [`Element`].
    ///
    /// This method is useful when you want to decouple different parts of your
    /// UI and make them __composable__.
    ///
    /// # Example
    /// Imagine we want to use [our counter](index.html#usage). But instead of
    /// showing a single counter, we want to display many of them. We can reuse
    /// the `Counter` type as it is!
    ///
    /// We use composition to model the __state__ of our new application:
    ///
    /// ```
    /// # mod counter {
    /// #     pub struct Counter;
    /// # }
    /// use counter::Counter;
    ///
    /// struct ManyCounters {
    ///     counters: Vec<Counter>,
    /// }
    /// ```
    ///
    /// We can store the state of multiple counters now. However, the
    /// __messages__ we implemented before describe the user interactions
    /// of a __single__ counter. Right now, we need to also identify which
    /// counter is receiving user interactions. Can we use composition again?
    /// Yes.
    ///
    /// ```
    /// # mod counter {
    /// #     #[derive(Debug, Clone, Copy)]
    /// #     pub enum Message {}
    /// # }
    /// #[derive(Debug, Clone, Copy)]
    /// pub enum Message {
    ///     Counter(usize, counter::Message)
    /// }
    /// ```
    ///
    /// We compose the previous __messages__ with the index of the counter
    /// producing them. Let's implement our __view logic__ now:
    ///
    /// ```no_run
    /// # mod iced {
    /// #     pub use iced_widget::core::Function;
    /// #     pub use iced_widget::Widget;
    /// #     pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::core::Theme, ()>;
    /// #
    /// #     pub mod widget {
    /// #         pub fn row<'a, Message>(iter: impl IntoIterator<Item = super::Element<'a, Message>>) -> super::Element<'a, Message> {
    /// #             unimplemented!()
    /// #         }
    /// #     }
    /// # }
    /// #
    /// # mod counter {
    /// #     #[derive(Debug, Clone, Copy)]
    /// #     pub enum Message {}
    /// #     pub struct Counter;
    /// #
    /// #     pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::core::Theme, ()>;
    /// #
    /// #     impl Counter {
    /// #         pub fn view(&self) -> Element<Message> {
    /// #             unimplemented!()
    /// #         }
    /// #     }
    /// # }
    /// #
    /// use counter::Counter;
    ///
    /// use iced::widget::row;
    /// use iced::{Element, Function, Widget};
    ///
    /// struct ManyCounters {
    ///     counters: Vec<Counter>,
    /// }
    ///
    /// #[derive(Debug, Clone, Copy)]
    /// pub enum Message {
    ///     Counter(usize, counter::Message),
    /// }
    ///
    /// impl ManyCounters {
    ///     pub fn view(&self) -> Element<Message> {
    ///         // We can quickly populate a `row` by mapping our counters
    ///         row(
    ///             self.counters
    ///                 .iter()
    ///                 .map(Counter::view)
    ///                 .enumerate()
    ///                 .map(|(index, counter)| {
    ///                     // Here we turn our `Element<counter::Message>` into
    ///                     // an `Element<Message>` by combining the `index` and the
    ///                     // message of the `element`.
    ///                     counter.map(Message::Counter.with(index)).boxed()
    ///                 }),
    ///         )
    ///         .into()
    ///     }
    /// }
    /// ```
    ///
    /// Finally, our __update logic__ is pretty straightforward: simple
    /// delegation.
    ///
    /// ```
    /// # mod counter {
    /// #     #[derive(Debug, Clone, Copy)]
    /// #     pub enum Message {}
    /// #     pub struct Counter;
    /// #
    /// #     impl Counter {
    /// #         pub fn update(&mut self, _message: Message) {}
    /// #     }
    /// # }
    /// #
    /// # use counter::Counter;
    /// #
    /// # struct ManyCounters {
    /// #     counters: Vec<Counter>,
    /// # }
    /// #
    /// # #[derive(Debug, Clone, Copy)]
    /// # pub enum Message {
    /// #    Counter(usize, counter::Message)
    /// # }
    /// impl ManyCounters {
    ///     pub fn update(&mut self, message: Message) {
    ///         match message {
    ///             Message::Counter(index, counter_msg) => {
    ///                 if let Some(counter) = self.counters.get_mut(index) {
    ///                     counter.update(counter_msg);
    ///                 }
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    fn map<F, B>(self, f: F) -> widget::Map<Self, F, Message>
    where
        Self: Sized,
        F: Fn(Message) -> B,
    {
        core::Widget::map(self, f)
    }

    /// TODO
    fn boxed<'a>(self) -> Element<'a, Message, Theme, Renderer>
    where
        Self: Sized + 'a,
    {
        core::Widget::boxed(self)
    }
}

impl<T, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for T
where
    T: core::Widget<Message, Theme, Renderer>,
    Renderer: core::Renderer,
{
}
