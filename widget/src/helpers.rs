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
use crate::style::application;
use crate::text::{self, Text};
use crate::text_editor::{self, TextEditor};
use crate::text_input::{self, TextInput};
use crate::toggler::{self, Toggler};
use crate::tooltip::{self, Tooltip};
use crate::{Column, MouseArea, Row, Space, Themer, VerticalSlider};

use std::borrow::Borrow;
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
pub fn container<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Theme: container::StyleSheet,
    Renderer: core::Renderer,
{
    Container::new(content)
}

/// Creates a new [`Column`] with the given children.
pub fn column<'a, Message, Theme, Renderer>(
    children: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Column<'a, Message, Theme, Renderer>
where
    Renderer: core::Renderer,
{
    Column::with_children(children)
}

/// Creates a new [`keyed::Column`] with the given children.
pub fn keyed_column<'a, Key, Message, Theme, Renderer>(
    children: impl IntoIterator<Item = (Key, Element<'a, Message, Theme, Renderer>)>,
) -> keyed::Column<'a, Key, Message, Theme, Renderer>
where
    Key: Copy + PartialEq,
    Renderer: core::Renderer,
{
    keyed::Column::with_children(children)
}

/// Creates a new [`Row`] with the given children.
///
/// [`Row`]: crate::Row
pub fn row<'a, Message, Theme, Renderer>(
    children: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Row<'a, Message, Theme, Renderer>
where
    Renderer: core::Renderer,
{
    Row::with_children(children)
}

/// Creates a new [`Scrollable`] with the provided content.
///
/// [`Scrollable`]: crate::Scrollable
pub fn scrollable<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Scrollable<'a, Message, Theme, Renderer>
where
    Theme: scrollable::StyleSheet,
    Renderer: core::Renderer,
{
    Scrollable::new(content)
}

/// Creates a new [`Button`] with the provided content.
///
/// [`Button`]: crate::Button
pub fn button<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Button<'a, Message, Theme, Renderer>
where
    Renderer: core::Renderer,
    Theme: button::StyleSheet,
{
    Button::new(content)
}

/// Creates a new [`Tooltip`] for the provided content with the given
/// [`Element`] and [`tooltip::Position`].
///
/// [`Tooltip`]: crate::Tooltip
/// [`tooltip::Position`]: crate::tooltip::Position
pub fn tooltip<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    tooltip: impl Into<Element<'a, Message, Theme, Renderer>>,
    position: tooltip::Position,
) -> crate::Tooltip<'a, Message, Theme, Renderer>
where
    Theme: container::StyleSheet + text::StyleSheet,
    Renderer: core::text::Renderer,
{
    Tooltip::new(content, tooltip, position)
}

/// Creates a new [`Text`] widget with the provided content.
///
/// [`Text`]: core::widget::Text
pub fn text<'a, Theme, Renderer>(
    text: impl ToString,
) -> Text<'a, Theme, Renderer>
where
    Theme: text::StyleSheet,
    Renderer: core::text::Renderer,
{
    Text::new(text.to_string())
}

/// Creates a new [`Checkbox`].
///
/// [`Checkbox`]: crate::Checkbox
pub fn checkbox<'a, Message, Theme, Renderer>(
    label: impl Into<String>,
    is_checked: bool,
) -> Checkbox<'a, Message, Theme, Renderer>
where
    Theme: checkbox::StyleSheet + text::StyleSheet,
    Renderer: core::text::Renderer,
{
    Checkbox::new(label, is_checked)
}

/// Creates a new [`Radio`].
///
/// [`Radio`]: crate::Radio
pub fn radio<Message, Theme, Renderer, V>(
    label: impl Into<String>,
    value: V,
    selected: Option<V>,
    on_click: impl FnOnce(V) -> Message,
) -> Radio<Message, Theme, Renderer>
where
    Message: Clone,
    Theme: radio::StyleSheet,
    Renderer: core::text::Renderer,
    V: Copy + Eq,
{
    Radio::new(label, value, selected, on_click)
}

/// Creates a new [`Toggler`].
///
/// [`Toggler`]: crate::Toggler
pub fn toggler<'a, Message, Theme, Renderer>(
    label: impl Into<Option<String>>,
    is_checked: bool,
    f: impl Fn(bool) -> Message + 'a,
) -> Toggler<'a, Message, Theme, Renderer>
where
    Renderer: core::text::Renderer,
    Theme: toggler::StyleSheet,
{
    Toggler::new(label, is_checked, f)
}

/// Creates a new [`TextInput`].
///
/// [`TextInput`]: crate::TextInput
pub fn text_input<'a, Message, Theme, Renderer>(
    placeholder: &str,
    value: &str,
) -> TextInput<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: text_input::StyleSheet,
    Renderer: core::text::Renderer,
{
    TextInput::new(placeholder, value)
}

/// Creates a new [`TextEditor`].
///
/// [`TextEditor`]: crate::TextEditor
pub fn text_editor<Message, Theme, Renderer>(
    content: &text_editor::Content<Renderer>,
) -> TextEditor<'_, core::text::highlighter::PlainText, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: text_editor::StyleSheet,
    Renderer: core::text::Renderer,
{
    TextEditor::new(content)
}

/// Creates a new [`Slider`].
///
/// [`Slider`]: crate::Slider
pub fn slider<'a, T, Message, Theme>(
    range: std::ops::RangeInclusive<T>,
    value: T,
    on_change: impl Fn(T) -> Message + 'a,
) -> Slider<'a, T, Message, Theme>
where
    T: Copy + From<u8> + std::cmp::PartialOrd,
    Message: Clone,
    Theme: slider::StyleSheet,
{
    Slider::new(range, value, on_change)
}

/// Creates a new [`VerticalSlider`].
///
/// [`VerticalSlider`]: crate::VerticalSlider
pub fn vertical_slider<'a, T, Message, Theme>(
    range: std::ops::RangeInclusive<T>,
    value: T,
    on_change: impl Fn(T) -> Message + 'a,
) -> VerticalSlider<'a, T, Message, Theme>
where
    T: Copy + From<u8> + std::cmp::PartialOrd,
    Message: Clone,
    Theme: slider::StyleSheet,
{
    VerticalSlider::new(range, value, on_change)
}

/// Creates a new [`PickList`].
///
/// [`PickList`]: crate::PickList
pub fn pick_list<'a, T, L, V, Message, Theme, Renderer>(
    options: L,
    selected: Option<V>,
    on_selected: impl Fn(T) -> Message + 'a,
) -> PickList<'a, T, L, V, Message, Theme, Renderer>
where
    T: ToString + PartialEq + Clone + 'a,
    L: Borrow<[T]> + 'a,
    V: Borrow<T> + 'a,
    Message: Clone,
    Renderer: core::text::Renderer,
    Theme: pick_list::StyleSheet
        + scrollable::StyleSheet
        + overlay::menu::StyleSheet
        + container::StyleSheet,
    <Theme as overlay::menu::StyleSheet>::Style:
        From<<Theme as pick_list::StyleSheet>::Style>,
{
    PickList::new(options, selected, on_selected)
}

/// Creates a new [`ComboBox`].
///
/// [`ComboBox`]: crate::ComboBox
pub fn combo_box<'a, T, Message, Theme, Renderer>(
    state: &'a combo_box::State<T>,
    placeholder: &str,
    selection: Option<&T>,
    on_selected: impl Fn(T) -> Message + 'static,
) -> ComboBox<'a, T, Message, Theme, Renderer>
where
    T: std::fmt::Display + Clone,
    Theme: text_input::StyleSheet + overlay::menu::StyleSheet,
    Renderer: core::text::Renderer,
{
    ComboBox::new(state, placeholder, selection, on_selected)
}

/// Creates a new [`Space`] widget that fills the available
/// horizontal space.
///
/// This can be useful to separate widgets in a [`Row`].
pub fn horizontal_space() -> Space {
    Space::with_width(Length::Fill)
}

/// Creates a new [`Space`] widget that fills the available
/// vertical space.
///
/// This can be useful to separate widgets in a [`Column`].
pub fn vertical_space() -> Space {
    Space::with_height(Length::Fill)
}

/// Creates a horizontal [`Rule`] with the given height.
///
/// [`Rule`]: crate::Rule
pub fn horizontal_rule<Theme>(height: impl Into<Pixels>) -> Rule<Theme>
where
    Theme: rule::StyleSheet,
{
    Rule::horizontal(height)
}

/// Creates a vertical [`Rule`] with the given width.
///
/// [`Rule`]: crate::Rule
pub fn vertical_rule<Theme>(width: impl Into<Pixels>) -> Rule<Theme>
where
    Theme: rule::StyleSheet,
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
pub fn progress_bar<Theme>(
    range: RangeInclusive<f32>,
    value: f32,
) -> ProgressBar<Theme>
where
    Theme: progress_bar::StyleSheet,
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
pub fn svg<Theme>(handle: impl Into<core::svg::Handle>) -> crate::Svg<Theme>
where
    Theme: crate::svg::StyleSheet,
{
    crate::Svg::new(handle)
}

/// Creates a new [`Canvas`].
///
/// [`Canvas`]: crate::Canvas
#[cfg(feature = "canvas")]
pub fn canvas<P, Message, Theme, Renderer>(
    program: P,
) -> crate::Canvas<P, Message, Theme, Renderer>
where
    Renderer: crate::graphics::geometry::Renderer,
    P: crate::canvas::Program<Message, Theme, Renderer>,
{
    crate::Canvas::new(program)
}

/// Creates a new [`QRCode`] widget from the given [`Data`].
///
/// [`QRCode`]: crate::QRCode
/// [`Data`]: crate::qr_code::Data
#[cfg(feature = "qr_code")]
pub fn qr_code<Theme>(data: &crate::qr_code::Data) -> crate::QRCode<'_, Theme>
where
    Theme: crate::qr_code::StyleSheet,
{
    crate::QRCode::new(data)
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
pub fn mouse_area<'a, Message, Theme, Renderer>(
    widget: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> MouseArea<'a, Message, Theme, Renderer>
where
    Renderer: core::Renderer,
{
    MouseArea::new(widget)
}

/// A widget that applies any `Theme` to its contents.
pub fn themer<'a, Message, Theme, Renderer>(
    theme: Theme,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Themer<'a, Message, Theme, Renderer>
where
    Renderer: core::Renderer,
    Theme: application::StyleSheet,
{
    Themer::new(theme, content)
}
