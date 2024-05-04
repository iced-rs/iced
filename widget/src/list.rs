#![allow(missing_docs)]
use crate::core::event::{self, Event};
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget;
use crate::core::widget::tree::{self, Tree};
use crate::core::window;
use crate::core::{
    self, Clipboard, Element, Layout, Length, Point, Rectangle, Shell, Size,
    Vector, Widget,
};

use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::VecDeque;

#[allow(missing_debug_implementations)]
pub struct List<'a, T, Message, Theme, Renderer> {
    content: &'a Content<T>,
    view_item:
        Box<dyn Fn(usize, &'a T) -> Element<'a, Message, Theme, Renderer> + 'a>,
    visible_elements: Vec<Element<'a, Message, Theme, Renderer>>,
}

impl<'a, T, Message, Theme, Renderer> List<'a, T, Message, Theme, Renderer> {
    pub fn new(
        content: &'a Content<T>,
        view_item: impl Fn(usize, &'a T) -> Element<'a, Message, Theme, Renderer>
            + 'a,
    ) -> Self {
        Self {
            content,
            view_item: Box::new(view_item),
            visible_elements: Vec::new(),
        }
    }
}

struct State {
    last_limits: layout::Limits,
    visible_layouts: Vec<(usize, layout::Node, Tree)>,
    size: Size,
    offsets: Vec<f32>,
    task: Task,
}

enum Task {
    Idle,
    Computing {
        current: usize,
        offsets: Vec<f32>,
        size: Size,
    },
}

impl State {
    fn recompute(&mut self, size: usize) {
        let mut offsets = Vec::with_capacity(size + 1);
        offsets.push(0.0);

        self.task = Task::Computing {
            current: 0,
            offsets,
            size: Size::ZERO,
        };
        self.visible_layouts.clear();
    }
}

impl<'a, T, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for List<'a, T, Message, Theme, Renderer>
where
    Renderer: core::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            last_limits: layout::Limits::NONE,
            visible_layouts: Vec::new(),
            size: Size::ZERO,
            offsets: vec![0.0],
            task: Task::Idle,
        })
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    fn layout(
        &self,
        tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<State>();

        if &state.last_limits != limits {
            state.last_limits = *limits;
            state.recompute(self.content.len());
        }

        let size = limits.resolve(Length::Shrink, Length::Shrink, state.size);

        layout::Node::new(size)
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        let state = tree.state.downcast_mut::<State>();
        let offset = layout.position() - Point::ORIGIN;

        let status = self
            .visible_elements
            .iter_mut()
            .zip(&mut state.visible_layouts)
            .map(|(element, (_index, layout, tree))| {
                element.as_widget_mut().on_event(
                    tree,
                    event.clone(),
                    Layout::with_offset(offset, layout),
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                )
            })
            .fold(event::Status::Ignored, event::Status::merge);

        if let Event::Window(_, window::Event::RedrawRequested(_)) = event {
            // Process changes first
            {
                let mut changes = self.content.changes.borrow_mut();

                match state.task {
                    Task::Idle => {
                        while let Some(change) = changes.pop_front() {
                            match change {
                                Change::Updated(index) => {
                                    let mut new_element = (self.view_item)(
                                        index,
                                        &self.content.items[index],
                                    );

                                    let visible_index = state
                                        .visible_layouts
                                        .iter_mut()
                                        .position(|(i, _, _)| *i == index);

                                    // Update if visible
                                    let new_size = if let Some(visible_index) =
                                        visible_index
                                    {
                                        let (_, layout, tree) = &mut state
                                            .visible_layouts[visible_index];

                                        tree.diff(&new_element);

                                        let new_layout = new_element
                                            .as_widget_mut()
                                            .layout(
                                                tree,
                                                renderer,
                                                &state.last_limits,
                                            )
                                            .move_to((
                                                0.0,
                                                state.offsets[index],
                                            ));

                                        if let Some(element) = self
                                            .visible_elements
                                            .get_mut(visible_index)
                                        {
                                            *element = new_element;
                                        }

                                        *layout = new_layout;

                                        layout.size()
                                    } else {
                                        let mut tree = Tree::new(&new_element);

                                        let layout =
                                            new_element.as_widget_mut().layout(
                                                &mut tree,
                                                renderer,
                                                &state.last_limits,
                                            );

                                        layout.size()
                                    };

                                    let height_difference = new_size.height
                                        - (state.offsets[index + 1]
                                            - state.offsets[index]);

                                    for offset in
                                        &mut state.offsets[index + 1..]
                                    {
                                        *offset += height_difference;
                                    }

                                    if let Some(visible_index) = visible_index {
                                        for (i, layout, _) in &mut state
                                            .visible_layouts[visible_index..]
                                        {
                                            layout.move_to_mut((
                                                0.0,
                                                state.offsets[*i],
                                            ));
                                        }
                                    }

                                    state.size.height += height_difference;
                                }
                                Change::Removed(index) => {
                                    let visible_index = state
                                        .visible_layouts
                                        .iter_mut()
                                        .position(|(i, _, _)| *i == index);

                                    let height = state.offsets[index + 1]
                                        - state.offsets[index];

                                    let _ = state.offsets.remove(index);

                                    for offset in &mut state.offsets[index..] {
                                        *offset -= height;
                                    }

                                    state.size.height -= height;

                                    if let Some(visible_index) = visible_index {
                                        let _ = state
                                            .visible_layouts
                                            .remove(visible_index);

                                        for (i, layout, _) in &mut state
                                            .visible_layouts[visible_index..]
                                        {
                                            *i -= 1;

                                            layout.move_to_mut((
                                                0.0,
                                                state.offsets[*i],
                                            ));
                                        }

                                        if visible_index
                                            < self.visible_elements.len()
                                        {
                                            let _ = self
                                                .visible_elements
                                                .remove(visible_index);
                                        }
                                    }
                                }
                                Change::Pushed(index) => {
                                    let mut new_element = (self.view_item)(
                                        index,
                                        &self.content.items[index],
                                    );

                                    let mut tree = Tree::new(&new_element);

                                    let layout =
                                        new_element.as_widget_mut().layout(
                                            &mut tree,
                                            renderer,
                                            &state.last_limits,
                                        );

                                    let size = layout.size();

                                    state.offsets.push(
                                        state.offsets[index] + size.height,
                                    );

                                    state.size.width =
                                        state.size.width.max(size.width);
                                    state.size.height += size.height;
                                }
                            }

                            shell.invalidate_layout();
                        }
                    }
                    Task::Computing { .. } => {
                        if !changes.is_empty() {
                            // If changes happen during layout computation,
                            // we simply restart the computation
                            changes.clear();
                            state.recompute(self.content.len());
                        }
                    }
                }
            }

            // Recompute if new
            {
                let mut is_new = self.content.is_new.borrow_mut();

                if *is_new {
                    state.recompute(self.content.len());
                    *is_new = false;
                }
            }

            match &mut state.task {
                Task::Idle => {}
                Task::Computing {
                    current,
                    size,
                    offsets,
                } => {
                    const MAX_BATCH_SIZE: usize = 50;

                    let end =
                        (*current + MAX_BATCH_SIZE).min(self.content.len());

                    let batch = &self.content.items[*current..end];

                    let mut max_width = size.width;
                    let mut accumulated_height =
                        offsets.last().copied().unwrap_or(0.0);

                    for (i, item) in batch.iter().enumerate() {
                        let element = (self.view_item)(*current + i, item);
                        let mut tree = Tree::new(&element);

                        let layout = element
                            .as_widget()
                            .layout(&mut tree, renderer, &state.last_limits)
                            .move_to((0.0, accumulated_height));

                        let bounds = layout.bounds() + offset;

                        max_width = max_width.max(bounds.width);
                        accumulated_height += bounds.height;

                        offsets.push(accumulated_height);
                    }

                    *size = Size::new(max_width, accumulated_height);

                    if end < self.content.len() {
                        *current = end;
                    } else {
                        state.offsets = std::mem::take(offsets);
                        state.size = std::mem::take(size);
                        state.task = Task::Idle;
                    }

                    shell.invalidate_layout();
                    shell.request_redraw(window::RedrawRequest::NextFrame);
                }
            }

            let offsets = &state.offsets;

            let start = match offsets.binary_search_by(|height| {
                (height + offset.y)
                    .partial_cmp(&viewport.y)
                    .unwrap_or(Ordering::Equal)
            }) {
                Ok(i) => i,
                Err(i) => i.saturating_sub(1),
            }
            .min(self.content.len());

            let end = match offsets.binary_search_by(|height| {
                (height + offset.y)
                    .partial_cmp(&(viewport.y + viewport.height))
                    .unwrap_or(Ordering::Equal)
            }) {
                Ok(i) => i,
                Err(i) => i,
            }
            .min(self.content.len());

            if state.visible_layouts.len() != self.visible_elements.len() {
                self.visible_elements.clear();
            }

            // If view was recreated, we repopulate the visible elements
            // out of the internal visible layouts
            if self.visible_elements.is_empty() {
                self.visible_elements = state
                    .visible_layouts
                    .iter()
                    .map(|(i, _, _)| {
                        (self.view_item)(*i, &self.content.items[*i])
                    })
                    .collect();
            }

            dbg!(self.visible_elements.len());
            dbg!(state.visible_layouts.len());
            dbg!(start, end);

            // Clear no longer visible elements
            let top = state
                .visible_layouts
                .iter()
                .take_while(|(i, _, _)| *i < start)
                .count();

            let bottom = state
                .visible_layouts
                .iter()
                .rev()
                .take_while(|(i, _, _)| *i >= end)
                .count();

            let _ = self.visible_elements.splice(..top, []);
            let _ = state.visible_layouts.splice(..top, []);

            let _ = self
                .visible_elements
                .splice(self.visible_elements.len() - bottom.., []);
            let _ = state
                .visible_layouts
                .splice(state.visible_layouts.len() - bottom.., []);

            // Prepend new visible elements
            if let Some(first_visible) =
                state.visible_layouts.first().map(|(i, _, _)| *i)
            {
                dbg!(first_visible);

                if start < first_visible {
                    for (i, item) in self.content.items[start..first_visible]
                        .iter()
                        .enumerate()
                    {
                        let element = (self.view_item)(start + i, item);
                        let mut tree = Tree::new(&element);

                        let layout = element
                            .as_widget()
                            .layout(&mut tree, renderer, &state.last_limits)
                            .move_to((0.0, offsets[start + i]));

                        state
                            .visible_layouts
                            .insert(i, (start + i, layout, tree));
                        self.visible_elements.insert(i, element);
                    }
                }
            }

            // Append new visible elements
            let last_visible = state
                .visible_layouts
                .last()
                .map(|(i, _, _)| *i + 1)
                .unwrap_or(start);

            if last_visible < end {
                for (i, item) in
                    self.content.items[last_visible..end].iter().enumerate()
                {
                    let element = (self.view_item)(last_visible + i, item);
                    let mut tree = Tree::new(&element);

                    let layout = element
                        .as_widget()
                        .layout(&mut tree, renderer, &state.last_limits)
                        .move_to((0.0, offsets[last_visible + i]));

                    state.visible_layouts.push((
                        last_visible + i,
                        layout,
                        tree,
                    ));
                    self.visible_elements.push(element);
                }
            }
        }

        status
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
        let state = tree.state.downcast_ref::<State>();
        let offset = layout.position() - Point::ORIGIN;

        for (element, (_item, layout, tree)) in
            self.visible_elements.iter().zip(&state.visible_layouts)
        {
            element.as_widget().draw(
                tree,
                renderer,
                theme,
                style,
                Layout::with_offset(offset, layout),
                cursor,
                viewport,
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();
        let offset = layout.position() - Point::ORIGIN;

        self.visible_elements
            .iter()
            .zip(&state.visible_layouts)
            .map(|(element, (_item, layout, tree))| {
                element.as_widget().mouse_interaction(
                    tree,
                    Layout::with_offset(offset, layout),
                    cursor,
                    viewport,
                    renderer,
                )
            })
            .max()
            .unwrap_or_default()
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation<Message>,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let offset = layout.position() - Point::ORIGIN;

        for (element, (_item, layout, tree)) in
            self.visible_elements.iter().zip(&mut state.visible_layouts)
        {
            element.as_widget().operate(
                tree,
                Layout::with_offset(offset, layout),
                renderer,
                operation,
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let state = tree.state.downcast_mut::<State>();
        let offset = layout.position() - Point::ORIGIN;

        let children = self
            .visible_elements
            .iter_mut()
            .zip(&mut state.visible_layouts)
            .filter_map(|(child, (_item, layout, tree))| {
                child.as_widget_mut().overlay(
                    tree,
                    Layout::with_offset(offset, layout),
                    renderer,
                    translation,
                )
            })
            .collect::<Vec<_>>();

        (!children.is_empty())
            .then(|| overlay::Group::with_children(children).overlay())
    }
}

impl<'a, T, Message, Theme, Renderer>
    From<List<'a, T, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: core::Renderer + 'a,
{
    fn from(list: List<'a, T, Message, Theme, Renderer>) -> Self {
        Self::new(list)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Content<T> {
    items: Vec<T>,
    is_new: RefCell<bool>,
    changes: RefCell<VecDeque<Change>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Change {
    Updated(usize),
    Removed(usize),
    Pushed(usize),
}

impl<T> Content<T> {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            is_new: RefCell::new(true),
            changes: RefCell::new(VecDeque::new()),
        }
    }

    pub fn with_items(items: Vec<T>) -> Self {
        Self {
            items,
            is_new: RefCell::new(true),
            changes: RefCell::new(VecDeque::new()),
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.changes.borrow_mut().push_back(Change::Updated(index));
        self.items.get_mut(index)
    }

    pub fn push(&mut self, item: T) {
        self.changes
            .borrow_mut()
            .push_back(Change::Pushed(self.items.len()));

        self.items.push(item);
    }

    pub fn remove(&mut self, index: usize) -> T {
        let mut changes = self.changes.borrow_mut();

        // Update pending changes after removal
        changes.retain_mut(|change| match change {
            Change::Updated(i) | Change::Removed(i) | Change::Pushed(i) => {
                // Decrement index of later changes
                if *i > index {
                    *i -= 1;
                }

                // Remove any changes to the removed item
                *i != index
            }
        });

        changes.push_back(Change::Removed(index));

        self.items.remove(index)
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn into_vec(self) -> Vec<T> {
        self.items
    }
}

impl<T> Default for Content<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> FromIterator<T> for Content<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::with_items(iter.into_iter().collect())
    }
}
