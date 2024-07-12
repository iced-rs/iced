//! Helper functions to create pure widgets.
use crate::button::{self, Button};
use crate::checkbox::{self, Checkbox};
use crate::combo_box::{self, ComboBox};
use crate::container::{self, Container};
use crate::core;
use crate::core::widget::operation;
use crate::core::{Element, Length, Pixels, Widget};
use crate::keyed;
use crate::overlay;
use crate::pick_list::{self, PickList};
use crate::progress_bar::{self, ProgressBar};
use crate::radio::{self, Radio};
use crate::rule::{self, Rule};
use crate::runtime::task::{self, Task};
use crate::runtime::Action;
use crate::scrollable::{self, Scrollable};
use crate::slider::{self, Slider};
use crate::text::{self, Text};
use crate::text_editor::{self, TextEditor};
use crate::text_input::{self, TextInput};
use crate::toggler::{self, Toggler};
use crate::tooltip::{self, Tooltip};
use crate::vertical_slider::{self, VerticalSlider};
use crate::{Column, MouseArea, Row, Space, Stack, Themer};

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

/// Creates a [`Stack`] with the given children.
///
/// [`Stack`]: crate::Stack
#[macro_export]
macro_rules! stack {
    () => (
        $crate::Stack::new()
    );
    ($($x:expr),+ $(,)?) => (
        $crate::Stack::with_children([$($crate::core::Element::from($x)),+])
    );
}

/// Creates a new [`Text`] widget with the provided content.
///
/// [`Text`]: core::widget::Text
///
/// This macro uses the same syntax as [`format!`], but creates a new [`Text`] widget instead.
///
/// See [the formatting documentation in `std::fmt`](std::fmt)
/// for details of the macro argument syntax.
///
/// # Examples
///
/// ```no_run
/// # mod iced {
/// #     pub struct Element<Message>(pub std::marker::PhantomData<Message>);
/// #     pub mod widget {
/// #         macro_rules! text {
/// #           ($($arg:tt)*) => {unimplemented!()}
/// #         }
/// #         pub(crate) use text;
/// #     }
/// # }
/// # struct Example;
/// # enum Message {}
/// use iced::Element;
/// use iced::widget::text;
///
/// impl Example {
///     fn view(&self) -> Element<Message> {
///         let simple = text!("Hello, world!");
///
///         let keyword = text!("Hello, {}", "world!");
///
///         let planet = "Earth";
///         let local_variable = text!("Hello, {planet}!");
///         // ...
///         # iced::Element(std::marker::PhantomData)
///     }
/// }
/// ```
#[macro_export]
macro_rules! text {
    ($($arg:tt)*) => {
        $crate::Text::new(format!($($arg)*))
    };
}

/// Creates a new [`Container`] with the provided content.
///
/// [`Container`]: crate::Container
pub fn container<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Theme: container::Catalog + 'a,
    Renderer: core::Renderer,
{
    Container::new(content)
}

/// Creates a new [`Container`] that fills all the available space
/// and centers its contents inside.
///
/// This is equivalent to:
/// ```rust,no_run
/// # use iced_widget::core::Length;
/// # use iced_widget::Container;
/// # fn container<A>(x: A) -> Container<'static, ()> { unreachable!() }
/// let centered = container("Centered!").center(Length::Fill);
/// ```
///
/// [`Container`]: crate::Container
pub fn center<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Theme: container::Catalog + 'a,
    Renderer: core::Renderer,
{
    container(content).center(Length::Fill)
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

/// Creates a new [`Stack`] with the given children.
///
/// [`Stack`]: crate::Stack
pub fn stack<'a, Message, Theme, Renderer>(
    children: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Stack<'a, Message, Theme, Renderer>
where
    Renderer: core::Renderer,
{
    Stack::with_children(children)
}

/// Wraps the given widget and captures any mouse button presses inside the bounds of
/// the widget—effectively making it _opaque_.
///
/// This helper is meant to be used to mark elements in a [`Stack`] to avoid mouse
/// events from passing through layers.
///
/// [`Stack`]: crate::Stack
pub fn opaque<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: core::Renderer + 'a,
{
    use crate::core::event::{self, Event};
    use crate::core::layout::{self, Layout};
    use crate::core::mouse;
    use crate::core::renderer;
    use crate::core::widget::tree::{self, Tree};
    use crate::core::{Rectangle, Shell, Size};

    struct Opaque<'a, Message, Theme, Renderer> {
        content: Element<'a, Message, Theme, Renderer>,
    }

    impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
        for Opaque<'a, Message, Theme, Renderer>
    where
        Renderer: core::Renderer,
    {
        fn tag(&self) -> tree::Tag {
            self.content.as_widget().tag()
        }

        fn state(&self) -> tree::State {
            self.content.as_widget().state()
        }

        fn children(&self) -> Vec<Tree> {
            self.content.as_widget().children()
        }

        fn diff(&self, tree: &mut Tree) {
            self.content.as_widget().diff(tree);
        }

        fn size(&self) -> Size<Length> {
            self.content.as_widget().size()
        }

        fn size_hint(&self) -> Size<Length> {
            self.content.as_widget().size_hint()
        }

        fn layout(
            &self,
            tree: &mut Tree,
            renderer: &Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            self.content.as_widget().layout(tree, renderer, limits)
        }

        fn draw(
            &self,
            tree: &Tree,
            renderer: &mut Renderer,
            theme: &Theme,
            style: &renderer::Style,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
        ) {
            self.content
                .as_widget()
                .draw(tree, renderer, theme, style, layout, cursor, viewport);
        }

        fn operate(
            &self,
            state: &mut Tree,
            layout: Layout<'_>,
            renderer: &Renderer,
            operation: &mut dyn operation::Operation<()>,
        ) {
            self.content
                .as_widget()
                .operate(state, layout, renderer, operation);
        }

        fn on_event(
            &mut self,
            state: &mut Tree,
            event: Event,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            renderer: &Renderer,
            clipboard: &mut dyn core::Clipboard,
            shell: &mut Shell<'_, Message>,
            viewport: &Rectangle,
        ) -> event::Status {
            let is_mouse_press = matches!(
                event,
                core::Event::Mouse(mouse::Event::ButtonPressed(_))
            );

            if let core::event::Status::Captured =
                self.content.as_widget_mut().on_event(
                    state, event, layout, cursor, renderer, clipboard, shell,
                    viewport,
                )
            {
                return event::Status::Captured;
            }

            if is_mouse_press && cursor.is_over(layout.bounds()) {
                event::Status::Captured
            } else {
                event::Status::Ignored
            }
        }

        fn mouse_interaction(
            &self,
            state: &core::widget::Tree,
            layout: core::Layout<'_>,
            cursor: core::mouse::Cursor,
            viewport: &core::Rectangle,
            renderer: &Renderer,
        ) -> core::mouse::Interaction {
            let interaction = self
                .content
                .as_widget()
                .mouse_interaction(state, layout, cursor, viewport, renderer);

            if interaction == mouse::Interaction::None
                && cursor.is_over(layout.bounds())
            {
                mouse::Interaction::Idle
            } else {
                interaction
            }
        }

        fn overlay<'b>(
            &'b mut self,
            state: &'b mut core::widget::Tree,
            layout: core::Layout<'_>,
            renderer: &Renderer,
            translation: core::Vector,
        ) -> Option<core::overlay::Element<'b, Message, Theme, Renderer>>
        {
            self.content.as_widget_mut().overlay(
                state,
                layout,
                renderer,
                translation,
            )
        }
    }

    Element::new(Opaque {
        content: content.into(),
    })
}

/// Displays a widget on top of another one, only when the base widget is hovered.
///
/// This works analogously to a [`stack`], but it will only display the layer on top
/// when the cursor is over the base. It can be useful for removing visual clutter.
///
/// [`stack`]: stack()
pub fn hover<'a, Message, Theme, Renderer>(
    base: impl Into<Element<'a, Message, Theme, Renderer>>,
    top: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: core::Renderer + 'a,
{
    use crate::core::event::{self, Event};
    use crate::core::layout::{self, Layout};
    use crate::core::mouse;
    use crate::core::renderer;
    use crate::core::widget::tree::{self, Tree};
    use crate::core::{Rectangle, Shell, Size};

    struct Hover<'a, Message, Theme, Renderer> {
        base: Element<'a, Message, Theme, Renderer>,
        top: Element<'a, Message, Theme, Renderer>,
        is_top_overlay_active: bool,
    }

    impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
        for Hover<'a, Message, Theme, Renderer>
    where
        Renderer: core::Renderer,
    {
        fn tag(&self) -> tree::Tag {
            struct Tag;
            tree::Tag::of::<Tag>()
        }

        fn children(&self) -> Vec<Tree> {
            vec![Tree::new(&self.base), Tree::new(&self.top)]
        }

        fn diff(&self, tree: &mut Tree) {
            tree.diff_children(&[&self.base, &self.top]);
        }

        fn size(&self) -> Size<Length> {
            self.base.as_widget().size()
        }

        fn size_hint(&self) -> Size<Length> {
            self.base.as_widget().size_hint()
        }

        fn layout(
            &self,
            tree: &mut Tree,
            renderer: &Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            let base = self.base.as_widget().layout(
                &mut tree.children[0],
                renderer,
                limits,
            );

            let top = self.top.as_widget().layout(
                &mut tree.children[1],
                renderer,
                &layout::Limits::new(Size::ZERO, base.size()),
            );

            layout::Node::with_children(base.size(), vec![base, top])
        }

        fn draw(
            &self,
            tree: &Tree,
            renderer: &mut Renderer,
            theme: &Theme,
            style: &renderer::Style,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
        ) {
            if let Some(bounds) = layout.bounds().intersection(viewport) {
                let mut children = layout.children().zip(&tree.children);

                let (base_layout, base_tree) = children.next().unwrap();

                self.base.as_widget().draw(
                    base_tree,
                    renderer,
                    theme,
                    style,
                    base_layout,
                    cursor,
                    viewport,
                );

                if cursor.is_over(layout.bounds()) || self.is_top_overlay_active
                {
                    let (top_layout, top_tree) = children.next().unwrap();

                    renderer.with_layer(bounds, |renderer| {
                        self.top.as_widget().draw(
                            top_tree, renderer, theme, style, top_layout,
                            cursor, viewport,
                        );
                    });
                }
            }
        }

        fn operate(
            &self,
            tree: &mut Tree,
            layout: Layout<'_>,
            renderer: &Renderer,
            operation: &mut dyn operation::Operation<()>,
        ) {
            let children = [&self.base, &self.top]
                .into_iter()
                .zip(layout.children().zip(&mut tree.children));

            for (child, (layout, tree)) in children {
                child.as_widget().operate(tree, layout, renderer, operation);
            }
        }

        fn on_event(
            &mut self,
            tree: &mut Tree,
            event: Event,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            renderer: &Renderer,
            clipboard: &mut dyn core::Clipboard,
            shell: &mut Shell<'_, Message>,
            viewport: &Rectangle,
        ) -> event::Status {
            let mut children = layout.children().zip(&mut tree.children);
            let (base_layout, base_tree) = children.next().unwrap();

            let top_status = if matches!(
                event,
                Event::Mouse(
                    mouse::Event::CursorMoved { .. }
                        | mouse::Event::ButtonReleased(_)
                )
            ) || cursor.is_over(layout.bounds())
            {
                let (top_layout, top_tree) = children.next().unwrap();

                self.top.as_widget_mut().on_event(
                    top_tree,
                    event.clone(),
                    top_layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                )
            } else {
                event::Status::Ignored
            };

            if top_status == event::Status::Captured {
                return top_status;
            }

            self.base.as_widget_mut().on_event(
                base_tree,
                event.clone(),
                base_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            )
        }

        fn mouse_interaction(
            &self,
            tree: &Tree,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
            renderer: &Renderer,
        ) -> mouse::Interaction {
            [&self.base, &self.top]
                .into_iter()
                .rev()
                .zip(layout.children().rev().zip(tree.children.iter().rev()))
                .map(|(child, (layout, tree))| {
                    child.as_widget().mouse_interaction(
                        tree, layout, cursor, viewport, renderer,
                    )
                })
                .find(|&interaction| interaction != mouse::Interaction::None)
                .unwrap_or_default()
        }

        fn overlay<'b>(
            &'b mut self,
            tree: &'b mut core::widget::Tree,
            layout: core::Layout<'_>,
            renderer: &Renderer,
            translation: core::Vector,
        ) -> Option<core::overlay::Element<'b, Message, Theme, Renderer>>
        {
            let mut overlays = [&mut self.base, &mut self.top]
                .into_iter()
                .zip(layout.children().zip(tree.children.iter_mut()))
                .map(|(child, (layout, tree))| {
                    child.as_widget_mut().overlay(
                        tree,
                        layout,
                        renderer,
                        translation,
                    )
                });

            if let Some(base_overlay) = overlays.next()? {
                return Some(base_overlay);
            }

            let top_overlay = overlays.next()?;
            self.is_top_overlay_active = top_overlay.is_some();

            top_overlay
        }
    }

    Element::new(Hover {
        base: base.into(),
        top: top.into(),
        is_top_overlay_active: false,
    })
}

/// Creates a new [`Scrollable`] with the provided content.
///
/// [`Scrollable`]: crate::Scrollable
pub fn scrollable<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Scrollable<'a, Message, Theme, Renderer>
where
    Theme: scrollable::Catalog + 'a,
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
    Theme: button::Catalog + 'a,
    Renderer: core::Renderer,
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
    Theme: container::Catalog + 'a,
    Renderer: core::text::Renderer,
{
    Tooltip::new(content, tooltip, position)
}

/// Creates a new [`Text`] widget with the provided content.
///
/// [`Text`]: core::widget::Text
pub fn text<'a, Theme, Renderer>(
    text: impl text::IntoFragment<'a>,
) -> Text<'a, Theme, Renderer>
where
    Theme: text::Catalog + 'a,
    Renderer: core::text::Renderer,
{
    Text::new(text)
}

/// Creates a new [`Text`] widget that displays the provided value.
///
/// [`Text`]: core::widget::Text
pub fn value<'a, Theme, Renderer>(
    value: impl ToString,
) -> Text<'a, Theme, Renderer>
where
    Theme: text::Catalog + 'a,
    Renderer: core::text::Renderer,
{
    Text::new(value.to_string())
}

/// Creates a new [`Checkbox`].
///
/// [`Checkbox`]: crate::Checkbox
pub fn checkbox<'a, Message, Theme, Renderer>(
    label: impl Into<String>,
    is_checked: bool,
) -> Checkbox<'a, Message, Theme, Renderer>
where
    Theme: checkbox::Catalog + 'a,
    Renderer: core::text::Renderer,
{
    Checkbox::new(label, is_checked)
}

/// Creates a new [`Radio`].
///
/// [`Radio`]: crate::Radio
pub fn radio<'a, Message, Theme, Renderer, V>(
    label: impl Into<String>,
    value: V,
    selected: Option<V>,
    on_click: impl FnOnce(V) -> Message,
) -> Radio<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: radio::Catalog + 'a,
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
    Theme: toggler::Catalog + 'a,
    Renderer: core::text::Renderer,
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
    Theme: text_input::Catalog + 'a,
    Renderer: core::text::Renderer,
{
    TextInput::new(placeholder, value)
}

/// Creates a new [`TextEditor`].
///
/// [`TextEditor`]: crate::TextEditor
pub fn text_editor<'a, Message, Theme, Renderer>(
    content: &'a text_editor::Content<Renderer>,
) -> TextEditor<'a, core::text::highlighter::PlainText, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: text_editor::Catalog + 'a,
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
    Theme: slider::Catalog + 'a,
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
    Theme: vertical_slider::Catalog + 'a,
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
    Theme: pick_list::Catalog + overlay::menu::Catalog,
    Renderer: core::text::Renderer,
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
    Theme: combo_box::Catalog + 'a,
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
pub fn horizontal_rule<'a, Theme>(height: impl Into<Pixels>) -> Rule<'a, Theme>
where
    Theme: rule::Catalog + 'a,
{
    Rule::horizontal(height)
}

/// Creates a vertical [`Rule`] with the given width.
///
/// [`Rule`]: crate::Rule
pub fn vertical_rule<'a, Theme>(width: impl Into<Pixels>) -> Rule<'a, Theme>
where
    Theme: rule::Catalog + 'a,
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
pub fn progress_bar<'a, Theme>(
    range: RangeInclusive<f32>,
    value: f32,
) -> ProgressBar<'a, Theme>
where
    Theme: progress_bar::Catalog + 'a,
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
pub fn svg<'a, Theme>(
    handle: impl Into<core::svg::Handle>,
) -> crate::Svg<'a, Theme>
where
    Theme: crate::svg::Catalog,
{
    crate::Svg::new(handle)
}

/// Creates an [`Element`] that displays the iced logo with the given `text_size`.
///
/// Useful for showing some love to your favorite GUI library in your "About" screen,
/// for instance.
#[cfg(feature = "svg")]
pub fn iced<'a, Message, Theme, Renderer>(
    text_size: impl Into<Pixels>,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: core::Renderer
        + core::text::Renderer<Font = core::Font>
        + core::svg::Renderer
        + 'a,
    Theme: text::Catalog + crate::svg::Catalog + 'a,
{
    use crate::core::{Alignment, Font};
    use crate::svg;
    use once_cell::sync::Lazy;

    static LOGO: Lazy<svg::Handle> = Lazy::new(|| {
        svg::Handle::from_memory(include_bytes!("../assets/iced-logo.svg"))
    });

    let text_size = text_size.into();

    row![
        svg(LOGO.clone()).width(text_size * 1.3),
        text("iced").size(text_size).font(Font::MONOSPACE)
    ]
    .spacing(text_size.0 / 3.0)
    .align_y(Alignment::Center)
    .into()
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
pub fn qr_code<'a, Theme>(
    data: &'a crate::qr_code::Data,
) -> crate::QRCode<'a, Theme>
where
    Theme: crate::qr_code::Catalog + 'a,
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
pub fn focus_previous<T>() -> Task<T> {
    task::effect(Action::widget(operation::focusable::focus_previous()))
}

/// Focuses the next focusable widget.
pub fn focus_next<T>() -> Task<T> {
    task::effect(Action::widget(operation::focusable::focus_next()))
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
pub fn themer<'a, Message, OldTheme, NewTheme, Renderer>(
    new_theme: NewTheme,
    content: impl Into<Element<'a, Message, NewTheme, Renderer>>,
) -> Themer<
    'a,
    Message,
    OldTheme,
    NewTheme,
    impl Fn(&OldTheme) -> NewTheme,
    Renderer,
>
where
    Renderer: core::Renderer,
    NewTheme: Clone,
{
    Themer::new(move |_| new_theme.clone(), content)
}
