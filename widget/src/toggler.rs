//! Show toggle controls using togglers.
use std::borrow::Cow;

use crate::core::alignment;
use crate::core::event;
use crate::core::layout;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::text;
use crate::core::widget::Id;
use crate::core::widget::Tree;
use crate::core::{
    id, Alignment, Clipboard, Element, Event, Layout, Length, Pixels, Point,
    Rectangle, Shell, Widget,
};
use crate::{Row, Text};

pub use crate::style::toggler::{Appearance, StyleSheet};

/// A toggler widget.
///
/// # Example
///
/// ```no_run
/// # type Toggler<'a, Message> =
/// #     iced_widget::Toggler<'a, Message, iced_widget::renderer::Renderer<iced_widget::style::Theme>>;
/// #
/// pub enum Message {
///     TogglerToggled(bool),
/// }
///
/// let is_toggled = true;
///
/// Toggler::new(String::from("Toggle me!"), is_toggled, |b| Message::TogglerToggled(b));
/// ```
#[allow(missing_debug_implementations)]
pub struct Toggler<'a, Message, Renderer = crate::Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    id: Id,
    label_id: Option<Id>,
    #[cfg(feature = "a11y")]
    name: Option<Cow<'a, str>>,
    #[cfg(feature = "a11y")]
    description: Option<iced_accessibility::Description<'a>>,
    #[cfg(feature = "a11y")]
    labeled_by_widget: Option<Vec<iced_accessibility::accesskit::NodeId>>,
    is_toggled: bool,
    on_toggle: Box<dyn Fn(bool) -> Message + 'a>,
    label: Option<String>,
    width: Length,
    size: f32,
    text_size: Option<f32>,
    text_alignment: alignment::Horizontal,
    spacing: f32,
    font: Option<Renderer::Font>,
    style: <Renderer::Theme as StyleSheet>::Style,
}

impl<'a, Message, Renderer> Toggler<'a, Message, Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// The default size of a [`Toggler`].
    pub const DEFAULT_SIZE: f32 = 20.0;

    /// Creates a new [`Toggler`].
    ///
    /// It expects:
    ///   * a boolean describing whether the [`Toggler`] is checked or not
    ///   * An optional label for the [`Toggler`]
    ///   * a function that will be called when the [`Toggler`] is toggled. It
    ///     will receive the new state of the [`Toggler`] and must produce a
    ///     `Message`.
    pub fn new<F>(
        label: impl Into<Option<String>>,
        is_toggled: bool,
        f: F,
    ) -> Self
    where
        F: 'a + Fn(bool) -> Message,
    {
        let label = label.into();

        Toggler {
            id: Id::unique(),
            label_id: label.as_ref().map(|_| Id::unique()),
            #[cfg(feature = "a11y")]
            name: None,
            #[cfg(feature = "a11y")]
            description: None,
            #[cfg(feature = "a11y")]
            labeled_by_widget: None,
            is_toggled,
            on_toggle: Box::new(f),
            label: label,
            width: Length::Fill,
            size: Self::DEFAULT_SIZE,
            text_size: None,
            text_alignment: alignment::Horizontal::Left,
            spacing: 0.0,
            font: None,
            style: Default::default(),
        }
    }

    /// Sets the size of the [`Toggler`].
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = size.into().0;
        self
    }

    /// Sets the width of the [`Toggler`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the text size o the [`Toggler`].
    pub fn text_size(mut self, text_size: impl Into<Pixels>) -> Self {
        self.text_size = Some(text_size.into().0);
        self
    }

    /// Sets the horizontal alignment of the text of the [`Toggler`]
    pub fn text_alignment(mut self, alignment: alignment::Horizontal) -> Self {
        self.text_alignment = alignment;
        self
    }

    /// Sets the spacing between the [`Toggler`] and the text.
    pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.spacing = spacing.into().0;
        self
    }

    /// Sets the [`Font`] of the text of the [`Toggler`]
    ///
    /// [`Font`]: crate::text::Renderer::Font
    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    /// Sets the style of the [`Toggler`].
    pub fn style(
        mut self,
        style: impl Into<<Renderer::Theme as StyleSheet>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }

    #[cfg(feature = "a11y")]
    /// Sets the name of the [`Button`].
    pub fn name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = Some(name.into());
        self
    }

    #[cfg(feature = "a11y")]
    /// Sets the description of the [`Button`].
    pub fn description_widget<T: iced_accessibility::Describes>(
        mut self,
        description: &T,
    ) -> Self {
        self.description = Some(iced_accessibility::Description::Id(
            description.description(),
        ));
        self
    }

    #[cfg(feature = "a11y")]
    /// Sets the description of the [`Button`].
    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description =
            Some(iced_accessibility::Description::Text(description.into()));
        self
    }

    #[cfg(feature = "a11y")]
    /// Sets the label of the [`Button`] using another widget.
    pub fn label(mut self, label: &dyn iced_accessibility::Labels) -> Self {
        self.labeled_by_widget =
            Some(label.label().into_iter().map(|l| l.into()).collect());
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Toggler<'a, Message, Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet + crate::text::StyleSheet,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let mut row = Row::<(), Renderer>::new()
            .width(self.width)
            .spacing(self.spacing)
            .align_items(Alignment::Center);

        if let Some(label) = &self.label {
            row = row.push(
                Text::new(label)
                    .horizontal_alignment(self.text_alignment)
                    .font(self.font.unwrap_or_else(|| renderer.default_font()))
                    .width(self.width)
                    .size(
                        self.text_size
                            .unwrap_or_else(|| renderer.default_size()),
                    ),
            );
        }

        row = row.push(Row::new().width(2.0 * self.size).height(self.size));

        row.layout(renderer, limits)
    }

    fn on_event(
        &mut self,
        _state: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let mouse_over = layout.bounds().contains(cursor_position);

                if mouse_over {
                    shell.publish((self.on_toggle)(!self.is_toggled));

                    event::Status::Captured
                } else {
                    event::Status::Ignored
                }
            }
            _ => event::Status::Ignored,
        }
    }

    fn mouse_interaction(
        &self,
        _state: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if layout.bounds().contains(cursor_position) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        _state: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        /// Makes sure that the border radius of the toggler looks good at every size.
        const BORDER_RADIUS_RATIO: f32 = 32.0 / 13.0;

        /// The space ratio between the background Quad and the Toggler bounds, and
        /// between the background Quad and foreground Quad.
        const SPACE_RATIO: f32 = 0.05;

        let mut children = layout.children();

        if let Some(label) = &self.label {
            let label_layout = children.next().unwrap();

            crate::text::draw(
                renderer,
                style,
                label_layout,
                label,
                self.text_size,
                self.font,
                Default::default(),
                self.text_alignment,
                alignment::Vertical::Center,
            );
        }

        let toggler_layout = children.next().unwrap();
        let bounds = toggler_layout.bounds();

        let is_mouse_over = bounds.contains(cursor_position);

        let style = if is_mouse_over {
            theme.hovered(&self.style, self.is_toggled)
        } else {
            theme.active(&self.style, self.is_toggled)
        };

        let border_radius = bounds.height / BORDER_RADIUS_RATIO;
        let space = SPACE_RATIO * bounds.height;

        let toggler_background_bounds = Rectangle {
            x: bounds.x + space,
            y: bounds.y + space,
            width: bounds.width - (2.0 * space),
            height: bounds.height - (2.0 * space),
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: toggler_background_bounds,
                border_radius: border_radius.into(),
                border_width: 1.0,
                border_color: style
                    .background_border
                    .unwrap_or(style.background),
            },
            style.background,
        );

        let toggler_foreground_bounds = Rectangle {
            x: bounds.x
                + if self.is_toggled {
                    bounds.width - 2.0 * space - (bounds.height - (4.0 * space))
                } else {
                    2.0 * space
                },
            y: bounds.y + (2.0 * space),
            width: bounds.height - (4.0 * space),
            height: bounds.height - (4.0 * space),
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: toggler_foreground_bounds,
                border_radius: border_radius.into(),
                border_width: 1.0,
                border_color: style
                    .foreground_border
                    .unwrap_or(style.foreground),
            },
            style.foreground,
        );
    }

    #[cfg(feature = "a11y")]
    /// get the a11y nodes for the widget
    fn a11y_nodes(
        &self,
        layout: Layout<'_>,
        _state: &Tree,
        cursor_position: Point,
    ) -> iced_accessibility::A11yTree {
        use iced_accessibility::{
            accesskit::{
                Action, CheckedState, NodeBuilder, NodeId, Rect, Role,
            },
            A11yNode, A11yTree,
        };

        let bounds = layout.bounds();
        let is_hovered = bounds.contains(cursor_position);
        let Rectangle {
            x,
            y,
            width,
            height,
        } = bounds;

        let bounds = Rect::new(
            x as f64,
            y as f64,
            (x + width) as f64,
            (y + height) as f64,
        );

        let mut node = NodeBuilder::new(Role::Switch);
        node.add_action(Action::Focus);
        node.add_action(Action::Default);
        node.set_bounds(bounds);
        if let Some(name) = self.name.as_ref() {
            node.set_name(name.clone());
        }
        match self.description.as_ref() {
            Some(iced_accessibility::Description::Id(id)) => {
                node.set_described_by(
                    id.iter()
                        .cloned()
                        .map(|id| NodeId::from(id))
                        .collect::<Vec<_>>(),
                );
            }
            Some(iced_accessibility::Description::Text(text)) => {
                node.set_description(text.clone());
            }
            None => {}
        }
        node.set_checked_state(if self.is_toggled {
            CheckedState::True
        } else {
            CheckedState::False
        });
        if is_hovered {
            node.set_hovered();
        }
        node.add_action(Action::Default);
        if let Some(label) = self.label.as_ref() {
            let mut label_node = NodeBuilder::new(Role::StaticText);

            label_node.set_name(label.clone());
            // TODO proper label bounds for the label
            label_node.set_bounds(bounds);

            A11yTree::node_with_child_tree(
                A11yNode::new(node, self.id.clone()),
                A11yTree::leaf(label_node, self.label_id.clone().unwrap()),
            )
        } else {
            if let Some(labeled_by_widget) = self.labeled_by_widget.as_ref() {
                node.set_labelled_by(labeled_by_widget.clone());
            }
            A11yTree::leaf(node, self.id.clone())
        }
    }

    fn id(&self) -> Option<Id> {
        if self.label.is_some() {
            Some(Id(id::Internal::Set(vec![
                self.id.0.clone(),
                self.label_id.clone().unwrap().0,
            ])))
        } else {
            Some(self.id.clone())
        }
    }

    fn set_id(&mut self, id: Id) {
        if let Id(id::Internal::Set(list)) = id {
            if list.len() == 2 && self.label.is_some() {
                self.id.0 = list[0].clone();
                self.label_id = Some(Id(list[1].clone()));
            }
        } else if self.label.is_none() {
            self.id = id;
        }
    }
}

impl<'a, Message, Renderer> From<Toggler<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + text::Renderer,
    Renderer::Theme: StyleSheet + crate::text::StyleSheet,
{
    fn from(
        toggler: Toggler<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(toggler)
    }
}
