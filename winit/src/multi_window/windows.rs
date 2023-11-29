use crate::core::{window, Size};
use crate::multi_window::{Application, State};
use iced_graphics::Compositor;
use iced_style::application::StyleSheet;
use std::fmt::{Debug, Formatter};
use winit::monitor::MonitorHandle;

pub struct Windows<A: Application, C: Compositor>
where
    <A::Renderer as crate::core::Renderer>::Theme: StyleSheet,
    C: Compositor<Renderer = A::Renderer>,
{
    pub ids: Vec<window::Id>,
    pub raw: Vec<winit::window::Window>,
    pub states: Vec<State<A>>,
    pub viewport_versions: Vec<usize>,
    pub exit_on_close_requested: Vec<bool>,
    pub surfaces: Vec<C::Surface>,
    pub renderers: Vec<A::Renderer>,
    pub pending_destroy: Vec<(window::Id, winit::window::WindowId)>,
}

impl<A: Application, C: Compositor> Debug for Windows<A, C>
where
    <A::Renderer as crate::core::Renderer>::Theme: StyleSheet,
    C: Compositor<Renderer = A::Renderer>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Windows")
            .field("ids", &self.ids)
            .field(
                "raw",
                &self
                    .raw
                    .iter()
                    .map(|raw| raw.id())
                    .collect::<Vec<winit::window::WindowId>>(),
            )
            .field("states", &self.states)
            .field("viewport_versions", &self.viewport_versions)
            .finish()
    }
}

impl<A: Application, C: Compositor> Windows<A, C>
where
    <A::Renderer as crate::core::Renderer>::Theme: StyleSheet,
    C: Compositor<Renderer = A::Renderer>,
{
    /// Creates a new [`Windows`] with a single `window::Id::MAIN` window.
    pub fn new(
        application: &A,
        compositor: &mut C,
        renderer: A::Renderer,
        main: winit::window::Window,
        exit_on_close_requested: bool,
    ) -> Self {
        let state = State::new(application, window::Id::MAIN, &main);
        let viewport_version = state.viewport_version();
        let physical_size = state.physical_size();
        let surface = compositor.create_surface(
            &main,
            physical_size.width,
            physical_size.height,
        );

        Self {
            ids: vec![window::Id::MAIN],
            raw: vec![main],
            states: vec![state],
            viewport_versions: vec![viewport_version],
            exit_on_close_requested: vec![exit_on_close_requested],
            surfaces: vec![surface],
            renderers: vec![renderer],
            pending_destroy: vec![],
        }
    }

    /// Adds a new window to [`Windows`]. Returns the size of the newly created window in logical
    /// pixels & the index of the window within [`Windows`].
    pub fn add(
        &mut self,
        application: &A,
        compositor: &mut C,
        id: window::Id,
        window: winit::window::Window,
        exit_on_close_requested: bool,
    ) -> (Size, usize) {
        let state = State::new(application, id, &window);
        let window_size = state.logical_size();
        let viewport_version = state.viewport_version();
        let physical_size = state.physical_size();
        let surface = compositor.create_surface(
            &window,
            physical_size.width,
            physical_size.height,
        );
        let renderer = compositor.renderer();

        self.ids.push(id);
        self.raw.push(window);
        self.states.push(state);
        self.exit_on_close_requested.push(exit_on_close_requested);
        self.viewport_versions.push(viewport_version);
        self.surfaces.push(surface);
        self.renderers.push(renderer);

        (window_size, self.ids.len() - 1)
    }

    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    pub fn main(&self) -> &winit::window::Window {
        &self.raw[0]
    }

    pub fn index_from_raw(&self, id: winit::window::WindowId) -> usize {
        self.raw
            .iter()
            .position(|window| window.id() == id)
            .expect("No raw window in multi_window::Windows")
    }

    pub fn index_from_id(&self, id: window::Id) -> usize {
        self.ids
            .iter()
            .position(|window_id| *window_id == id)
            .expect("No window in multi_window::Windows")
    }

    pub fn last_monitor(&self) -> Option<MonitorHandle> {
        self.raw.last().and_then(|w| w.current_monitor())
    }

    pub fn last(&self) -> usize {
        self.ids.len() - 1
    }

    pub fn with_raw(&self, id: window::Id) -> &winit::window::Window {
        let i = self.index_from_id(id);
        &self.raw[i]
    }

    /// Deletes the window with `id` from [`Windows`]. Returns the index that the window had.
    pub fn delete(&mut self, id: window::Id) -> usize {
        let i = self.index_from_id(id);

        let id = self.ids.remove(i);
        let window = self.raw.remove(i);
        let _ = self.states.remove(i);
        let _ = self.exit_on_close_requested.remove(i);
        let _ = self.viewport_versions.remove(i);
        let _ = self.surfaces.remove(i);

        self.pending_destroy.push((id, window.id()));

        i
    }

    /// Gets the winit `window` that is pending to be destroyed if it exists.
    pub fn get_pending_destroy(
        &mut self,
        window: winit::window::WindowId,
    ) -> window::Id {
        let i = self
            .pending_destroy
            .iter()
            .position(|(_, window_id)| window == *window_id)
            .unwrap();

        let (id, _) = self.pending_destroy.remove(i);
        id
    }

    /// Returns the windows that need to be requested to closed, and also the windows that can be
    /// closed immediately.
    pub fn partition_close_requests(
        &self,
    ) -> (Vec<window::Id>, Vec<window::Id>) {
        self.exit_on_close_requested.iter().enumerate().fold(
            (vec![], vec![]),
            |(mut close_immediately, mut needs_request_closed), (i, close)| {
                let id = self.ids[i];

                if *close {
                    close_immediately.push(id);
                } else {
                    needs_request_closed.push(id);
                }

                (close_immediately, needs_request_closed)
            },
        )
    }
}
