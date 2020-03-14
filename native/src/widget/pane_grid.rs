mod direction;
mod node;
mod pane;
mod split;
mod state;

pub use direction::Direction;
pub use pane::Pane;
pub use split::Split;
pub use state::{Focus, State};

use crate::{
    input::{keyboard, mouse, ButtonState},
    layout, Clipboard, Element, Event, Hasher, Layout, Length, Point, Size,
    Widget,
};

#[allow(missing_debug_implementations)]
pub struct PaneGrid<'a, Message, Renderer> {
    state: &'a mut state::Internal,
    modifiers: &'a mut keyboard::ModifiersState,
    elements: Vec<(Pane, Element<'a, Message, Renderer>)>,
    width: Length,
    height: Length,
    spacing: u16,
    on_drag: Option<Box<dyn Fn(DragEvent) -> Message>>,
}

impl<'a, Message, Renderer> PaneGrid<'a, Message, Renderer> {
    pub fn new<T>(
        state: &'a mut State<T>,
        view: impl Fn(
            Pane,
            &'a mut T,
            Option<Focus>,
        ) -> Element<'a, Message, Renderer>,
    ) -> Self {
        let elements = {
            let focused_pane = state.internal.focused();

            state
                .panes
                .iter_mut()
                .map(move |(pane, pane_state)| {
                    let focus = match focused_pane {
                        state::FocusedPane::Some {
                            pane: focused_pane,
                            focus,
                        } if *pane == focused_pane => Some(focus),
                        _ => None,
                    };

                    (*pane, view(*pane, pane_state, focus))
                })
                .collect()
        };

        Self {
            state: &mut state.internal,
            modifiers: &mut state.modifiers,
            elements,
            width: Length::Fill,
            height: Length::Fill,
            spacing: 0,
            on_drag: None,
        }
    }

    /// Sets the width of the [`PaneGrid`].
    ///
    /// [`PaneGrid`]: struct.Column.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`PaneGrid`].
    ///
    /// [`PaneGrid`]: struct.Column.html
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the spacing _between_ the panes of the [`PaneGrid`].
    ///
    /// [`PaneGrid`]: struct.Column.html
    pub fn spacing(mut self, units: u16) -> Self {
        self.spacing = units;
        self
    }

    pub fn on_drag(
        mut self,
        f: impl Fn(DragEvent) -> Message + 'static,
    ) -> Self {
        self.on_drag = Some(Box::new(f));
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DragEvent {
    Picked { pane: Pane },
    Dropped { pane: Pane, target: Pane },
    Canceled { pane: Pane },
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for PaneGrid<'a, Message, Renderer>
where
    Renderer: self::Renderer + 'static,
    Message: 'static,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);
        let size = limits.resolve(Size::ZERO);

        let regions = self.state.regions(f32::from(self.spacing), size);

        let children = self
            .elements
            .iter()
            .filter_map(|(pane, element)| {
                let region = regions.get(pane)?;
                let size = Size::new(region.width, region.height);

                let mut node =
                    element.layout(renderer, &layout::Limits::new(size, size));

                node.move_to(Point::new(region.x, region.y));

                Some(node)
            })
            .collect();

        layout::Node::with_children(size, children)
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
        renderer: &Renderer,
        clipboard: Option<&dyn Clipboard>,
    ) {
        match event {
            Event::Mouse(mouse::Event::Input {
                button: mouse::Button::Left,
                state,
            }) => match state {
                ButtonState::Pressed => {
                    let mut clicked_region =
                        self.elements.iter().zip(layout.children()).filter(
                            |(_, layout)| {
                                layout.bounds().contains(cursor_position)
                            },
                        );

                    if let Some(((pane, _), _)) = clicked_region.next() {
                        match &self.on_drag {
                            Some(on_drag) if self.modifiers.alt => {
                                self.state.drag(pane);

                                messages.push(on_drag(DragEvent::Picked {
                                    pane: *pane,
                                }));
                            }
                            _ => {
                                self.state.focus(pane);
                            }
                        }
                    } else {
                        self.state.unfocus();
                    }
                }
                ButtonState::Released => {
                    if let Some(pane) = self.state.dragged() {
                        self.state.focus(&pane);

                        if let Some(on_drag) = &self.on_drag {
                            let mut dropped_region = self
                                .elements
                                .iter()
                                .zip(layout.children())
                                .filter(|(_, layout)| {
                                    layout.bounds().contains(cursor_position)
                                });

                            let event = match dropped_region.next() {
                                Some(((target, _), _)) if pane != *target => {
                                    DragEvent::Dropped {
                                        pane,
                                        target: *target,
                                    }
                                }
                                _ => DragEvent::Canceled { pane },
                            };

                            messages.push(on_drag(event));
                        }
                    }
                }
            },
            Event::Keyboard(keyboard::Event::Input { modifiers, .. }) => {
                *self.modifiers = modifiers;
            }
            _ => {}
        }

        if self.state.dragged().is_none() {
            {
                self.elements.iter_mut().zip(layout.children()).for_each(
                    |((_, pane), layout)| {
                        pane.widget.on_event(
                            event.clone(),
                            layout,
                            cursor_position,
                            messages,
                            renderer,
                            clipboard,
                        )
                    },
                );
            }
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Renderer::Output {
        renderer.draw(
            defaults,
            &self.elements,
            self.state.dragged(),
            layout,
            cursor_position,
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash;

        std::any::TypeId::of::<PaneGrid<'_, Message, Renderer>>().hash(state);
        self.width.hash(state);
        self.height.hash(state);
        self.state.hash_layout(state);

        for (_, element) in &self.elements {
            element.hash_layout(state);
        }
    }
}

/// The renderer of a [`PaneGrid`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`PaneGrid`] in your user interface.
///
/// [`PaneGrid`]: struct.PaneGrid.html
/// [renderer]: ../../renderer/index.html
pub trait Renderer: crate::Renderer + Sized {
    /// Draws a [`PaneGrid`].
    ///
    /// It receives:
    /// - the elements of the [`PaneGrid`]
    /// - the [`Pane`] that is currently being dragged
    /// - the [`Layout`] of the [`PaneGrid`] and its elements
    /// - the cursor position
    ///
    /// [`Column`]: struct.Row.html
    /// [`Layout`]: ../layout/struct.Layout.html
    fn draw<Message>(
        &mut self,
        defaults: &Self::Defaults,
        content: &[(Pane, Element<'_, Message, Self>)],
        dragging: Option<Pane>,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<PaneGrid<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: self::Renderer + 'static,
    Message: 'static,
{
    fn from(
        pane_grid: PaneGrid<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(pane_grid)
    }
}
