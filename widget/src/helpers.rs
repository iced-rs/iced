//! Helper functions to create pure widgets.
use crate::button::{self, Button};
use crate::checkbox::{self, Checkbox};
use crate::combo_box::{self, ComboBox};
use crate::container::{self, Container};
use crate::core;
use crate::core::widget::operation;
use crate::core::{Element, Length, Pixels};
use crate::keyed;
use crate::overlay;
use crate::pick_list::{self, PickList};
use crate::progress_bar::{self, ProgressBar};
use crate::radio::{self, Radio};
use crate::rule::{self, Rule};
use crate::runtime::Command;
use crate::scrollable::{self, Scrollable};
use crate::slider::{self, Slider};
use crate::text::{self, Text};
use crate::text_editor::{self, TextEditor};
use crate::text_input::{self, TextInput};
use crate::toggler::{self, Toggler};
use crate::tooltip::{self, Tooltip};
use crate::{Column, MouseArea, Row, Space, VerticalSlider};

use std::borrow::Cow;
use std::ops::RangeInclusive;

/// Creates a [`Column`] with the given children.
///
/// [`Column`]: crate::Column
#[macro_export]
macro_rules! column {
    () => (
        $crate::Column::new()
    );
    ($($x:expr),+ $(,)?) => (
        $crate::Column::with_children([$($crate::core::Element::from($x)),+])
    );
}

/// Creates a [`Row`] with the given children.
///
/// [`Row`]: crate::Row
#[macro_export]
macro_rules! row {
    () => (
        $crate::Row::new()
    );
    ($($x:expr),+ $(,)?) => (
        $crate::Row::with_children([$($crate::core::Element::from($x)),+])
    );
}

/// Creates a new [`Container`] with the provided content.
///
/// [`Container`]: crate::Container
pub fn container<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
) -> Container<'a, Message, Renderer>
where
    Renderer: core::Renderer,
    Renderer::Theme: container::StyleSheet,
{
    Container::new(content)
}

/// Creates a new [`Column`] with the given children.
pub fn column<'a, Message, Renderer>(
    children: impl IntoIterator<Item = Element<'a, Message, Renderer>>,
) -> Column<'a, Message, Renderer>
where
    Renderer: core::Renderer,
{
    Column::with_children(children)
}

/// Creates a new [`keyed::Column`] with the given children.
pub fn keyed_column<'a, Key, Message, Renderer>(
    children: impl IntoIterator<Item = (Key, Element<'a, Message, Renderer>)>,
) -> keyed::Column<'a, Key, Message, Renderer>
where
    Key: Copy + PartialEq,
    Renderer: core::Renderer,
{
    keyed::Column::with_children(children)
}

/// Creates a new [`Row`] with the given children.
///
/// [`Row`]: crate::Row
pub fn row<'a, Message, Renderer>(
    children: impl IntoIterator<Item = Element<'a, Message, Renderer>>,
) -> Row<'a, Message, Renderer>
where
    Renderer: core::Renderer,
{
    Row::with_children(children)
}

/// Creates a new [`Scrollable`] with the provided content.
///
/// [`Scrollable`]: crate::Scrollable
pub fn scrollable<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
) -> Scrollable<'a, Message, Renderer>
where
    Renderer: core::Renderer,
    Renderer::Theme: scrollable::StyleSheet,
{
    Scrollable::new(content)
}

/// Creates a new [`Button`] with the provided content.
///
/// [`Button`]: crate::Button
pub fn button<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
) -> Button<'a, Message, Renderer>
where
    Renderer: core::Renderer,
    Renderer::Theme: button::StyleSheet,
    <Renderer::Theme as button::StyleSheet>::Style: Default,
{
    Button::new(content)
}

/// Creates a new [`Tooltip`] with the provided content, tooltip text, and [`tooltip::Position`].
///
/// [`Tooltip`]: crate::Tooltip
/// [`tooltip::Position`]: crate::tooltip::Position
pub fn tooltip<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
    tooltip: impl ToString,
    position: tooltip::Position,
) -> crate::Tooltip<'a, Message, Renderer>
where
    Renderer: core::text::Renderer,
    Renderer::Theme: container::StyleSheet + text::StyleSheet,
{
    Tooltip::new(content, tooltip.to_string(), position)
}

/// Creates a new [`Text`] widget with the provided content.
///
/// [`Text`]: core::widget::Text
pub fn text<'a, Renderer>(text: impl ToString) -> Text<'a, Renderer>
where
    Renderer: core::text::Renderer,
    Renderer::Theme: text::StyleSheet,
{
    Text::new(text.to_string())
}

/// Creates a new [`Checkbox`].
///
/// [`Checkbox`]: crate::Checkbox
pub fn checkbox<'a, Message, Renderer>(
    label: impl Into<String>,
    is_checked: bool,
    f: impl Fn(bool) -> Message + 'a,
) -> Checkbox<'a, Message, Renderer>
where
    Renderer: core::text::Renderer,
    Renderer::Theme: checkbox::StyleSheet + text::StyleSheet,
{
    Checkbox::new(label, is_checked, f)
}

/// Creates a new [`Radio`].
///
/// [`Radio`]: crate::Radio
pub fn radio<Message, Renderer, V>(
    label: impl Into<String>,
    value: V,
    selected: Option<V>,
    on_click: impl FnOnce(V) -> Message,
) -> Radio<Message, Renderer>
where
    Message: Clone,
    Renderer: core::text::Renderer,
    Renderer::Theme: radio::StyleSheet,
    V: Copy + Eq,
{
    Radio::new(label, value, selected, on_click)
}

/// Creates a new [`Toggler`].
///
/// [`Toggler`]: crate::Toggler
pub fn toggler<'a, Message, Renderer>(
    label: impl Into<Option<String>>,
    is_checked: bool,
    f: impl Fn(bool) -> Message + 'a,
) -> Toggler<'a, Message, Renderer>
where
    Renderer: core::text::Renderer,
    Renderer::Theme: toggler::StyleSheet,
{
    Toggler::new(label, is_checked, f)
}

/// Creates a new [`TextInput`].
///
/// [`TextInput`]: crate::TextInput
pub fn text_input<'a, Message, Renderer>(
    placeholder: &str,
    value: &str,
) -> TextInput<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: core::text::Renderer,
    Renderer::Theme: text_input::StyleSheet,
{
    TextInput::new(placeholder, value)
}

/// Creates a new [`TextEditor`].
///
/// [`TextEditor`]: crate::TextEditor
pub fn text_editor<Message, Renderer>(
    content: &text_editor::Content<Renderer>,
) -> TextEditor<'_, core::text::highlighter::PlainText, Message, Renderer>
where
    Message: Clone,
    Renderer: core::text::Renderer,
    Renderer::Theme: text_editor::StyleSheet,
{
    TextEditor::new(content)
}

/// Creates a new [`Slider`].
///
/// [`Slider`]: crate::Slider
pub fn slider<'a, T, Message, Renderer>(
    range: std::ops::RangeInclusive<T>,
    value: T,
    on_change: impl Fn(T) -> Message + 'a,
) -> Slider<'a, T, Message, Renderer>
where
    T: Copy + From<u8> + std::cmp::PartialOrd,
    Message: Clone,
    Renderer: core::Renderer,
    Renderer::Theme: slider::StyleSheet,
{
    Slider::new(range, value, on_change)
}

/// Creates a new [`VerticalSlider`].
///
/// [`VerticalSlider`]: crate::VerticalSlider
pub fn vertical_slider<'a, T, Message, Renderer>(
    range: std::ops::RangeInclusive<T>,
    value: T,
    on_change: impl Fn(T) -> Message + 'a,
) -> VerticalSlider<'a, T, Message, Renderer>
where
    T: Copy + From<u8> + std::cmp::PartialOrd,
    Message: Clone,
    Renderer: core::Renderer,
    Renderer::Theme: slider::StyleSheet,
{
    VerticalSlider::new(range, value, on_change)
}

/// Creates a new [`PickList`].
///
/// [`PickList`]: crate::PickList
pub fn pick_list<'a, Message, Renderer, T>(
    options: impl Into<Cow<'a, [T]>>,
    selected: Option<T>,
    on_selected: impl Fn(T) -> Message + 'a,
) -> PickList<'a, T, Message, Renderer>
where
    T: ToString + PartialEq + 'static,
    [T]: ToOwned<Owned = Vec<T>>,
    Renderer: core::text::Renderer,
    Renderer::Theme: pick_list::StyleSheet
        + scrollable::StyleSheet
        + overlay::menu::StyleSheet
        + container::StyleSheet,
    <Renderer::Theme as overlay::menu::StyleSheet>::Style:
        From<<Renderer::Theme as pick_list::StyleSheet>::Style>,
{
    PickList::new(options, selected, on_selected)
}

/// Creates a new [`ComboBox`].
///
/// [`ComboBox`]: crate::ComboBox
pub fn combo_box<'a, T, Message, Renderer>(
    state: &'a combo_box::State<T>,
    placeholder: &str,
    selection: Option<&T>,
    on_selected: impl Fn(T) -> Message + 'static,
) -> ComboBox<'a, T, Message, Renderer>
where
    T: std::fmt::Display + Clone,
    Renderer: core::text::Renderer,
    Renderer::Theme: text_input::StyleSheet + overlay::menu::StyleSheet,
{
    ComboBox::new(state, placeholder, selection, on_selected)
}

/// Creates a new horizontal [`Space`] with the given [`Length`].
///
/// [`Space`]: crate::Space
pub fn horizontal_space(width: impl Into<Length>) -> Space {
    Space::with_width(width)
}

/// Creates a new vertical [`Space`] with the given [`Length`].
///
/// [`Space`]: crate::Space
pub fn vertical_space(height: impl Into<Length>) -> Space {
    Space::with_height(height)
}

/// Creates a horizontal [`Rule`] with the given height.
///
/// [`Rule`]: crate::Rule
pub fn horizontal_rule<Renderer>(height: impl Into<Pixels>) -> Rule<Renderer>
where
    Renderer: core::Renderer,
    Renderer::Theme: rule::StyleSheet,
{
    Rule::horizontal(height)
}

/// Creates a vertical [`Rule`] with the given width.
///
/// [`Rule`]: crate::Rule
pub fn vertical_rule<Renderer>(width: impl Into<Pixels>) -> Rule<Renderer>
where
    Renderer: core::Renderer,
    Renderer::Theme: rule::StyleSheet,
{
    Rule::vertical(width)
}

/// Creates a new [`ProgressBar`].
///
/// It expects:
///   * an inclusive range of possible values, and
///   * the current value of the [`ProgressBar`].
///
/// [`ProgressBar`]: crate::ProgressBar
pub fn progress_bar<Renderer>(
    range: RangeInclusive<f32>,
    value: f32,
) -> ProgressBar<Renderer>
where
    Renderer: core::Renderer,
    Renderer::Theme: progress_bar::StyleSheet,
{
    ProgressBar::new(range, value)
}

/// Creates a new [`Image`].
///
/// [`Image`]: crate::Image
#[cfg(feature = "image")]
pub fn image<Handle>(handle: impl Into<Handle>) -> crate::Image<Handle> {
    crate::Image::new(handle.into())
}

/// Creates a new [`Svg`] widget from the given [`Handle`].
///
/// [`Svg`]: crate::Svg
/// [`Handle`]: crate::svg::Handle
#[cfg(feature = "svg")]
pub fn svg<Renderer>(
    handle: impl Into<core::svg::Handle>,
) -> crate::Svg<Renderer>
where
    Renderer: core::svg::Renderer,
    Renderer::Theme: crate::svg::StyleSheet,
{
    crate::Svg::new(handle)
}

/// Creates a new [`Canvas`].
///
/// [`Canvas`]: crate::Canvas
#[cfg(feature = "canvas")]
pub fn canvas<P, Message, Renderer>(
    program: P,
) -> crate::Canvas<P, Message, Renderer>
where
    Renderer: crate::graphics::geometry::Renderer,
    P: crate::canvas::Program<Message, Renderer>,
{
    crate::Canvas::new(program)
}

/// Creates a new [`Shader`].
///
/// [`Shader`]: crate::Shader
#[cfg(feature = "wgpu")]
pub fn shader<Message, P>(program: P) -> crate::Shader<Message, P>
where
    P: crate::shader::Program<Message>,
{
    crate::Shader::new(program)
}

/// Focuses the previous focusable widget.
pub fn focus_previous<Message>() -> Command<Message>
where
    Message: 'static,
{
    Command::widget(operation::focusable::focus_previous())
}

/// Focuses the next focusable widget.
pub fn focus_next<Message>() -> Command<Message>
where
    Message: 'static,
{
    Command::widget(operation::focusable::focus_next())
}

/// A container intercepting mouse events.
pub fn mouse_area<'a, Message, Renderer>(
    widget: impl Into<Element<'a, Message, Renderer>>,
) -> MouseArea<'a, Message, Renderer>
where
    Renderer: core::Renderer,
{
    MouseArea::new(widget)
}
