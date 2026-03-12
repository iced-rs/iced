//! Popup surface management for Wayland xdg_popup.
//!
//! This module provides infrastructure for rendering content to popup surfaces
//! that can extend outside their parent window bounds.

use crate::core::Size;
use crate::core::theme;
use crate::core::window;
use crate::graphics::{Compositor, Viewport};
use crate::program::Program;

use std::collections::BTreeMap;
use std::ptr::NonNull;

use winit::raw_window_handle;

/// Unique ID for a popup surface within iced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PopupId(pub u64);

impl PopupId {
    /// Generate a new unique popup ID.
    #[allow(dead_code)]
    pub fn unique() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// Raw window handles for a Wayland popup surface.
/// Implements HasWindowHandle and HasDisplayHandle for wgpu surface creation.
#[derive(Clone)]
pub struct PopupSurface {
    surface_ptr: NonNull<std::ffi::c_void>,
    display_ptr: NonNull<std::ffi::c_void>,
}

impl PopupSurface {
    /// Create a new popup surface wrapper from raw pointers.
    ///
    /// # Safety
    /// The pointers must be valid wl_surface and wl_display pointers.
    pub fn new(
        surface_ptr: NonNull<std::ffi::c_void>,
        display_ptr: NonNull<std::ffi::c_void>,
    ) -> Self {
        Self {
            surface_ptr,
            display_ptr,
        }
    }
}

// Safety: The pointers are from the Wayland event loop which is Send
#[allow(unsafe_code)]
unsafe impl Send for PopupSurface {}
// Safety: The pointers are from the Wayland event loop which is Sync
#[allow(unsafe_code)]
unsafe impl Sync for PopupSurface {}

#[allow(unsafe_code)]
impl raw_window_handle::HasWindowHandle for PopupSurface {
    fn window_handle(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        let handle = raw_window_handle::WaylandWindowHandle::new(self.surface_ptr);
        // Safety: The surface pointer is valid for the lifetime of self
        Ok(unsafe { raw_window_handle::WindowHandle::borrow_raw(handle.into()) })
    }
}

#[allow(unsafe_code)]
impl raw_window_handle::HasDisplayHandle for PopupSurface {
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        let handle = raw_window_handle::WaylandDisplayHandle::new(self.display_ptr);
        // Safety: The display pointer is valid for the lifetime of self
        Ok(unsafe { raw_window_handle::DisplayHandle::borrow_raw(handle.into()) })
    }
}

/// State for a popup being managed by iced.
pub struct Popup<C>
where
    C: Compositor,
{
    /// The popup's iced ID.
    #[allow(dead_code)]
    pub id: PopupId,
    /// The popup's iced window ID (used for view lookup).
    pub iced_id: window::Id,
    /// Parent window ID.
    pub parent_id: window::Id,
    /// The winit popup ID.
    pub winit_popup_id: Option<u64>,
    /// Size of the popup.
    pub size: Size<u32>,
    /// Scale factor (inherited from parent).
    pub scale_factor: f32,
    /// Viewport for rendering.
    pub viewport: Option<Viewport>,
    /// Compositor surface for rendering.
    pub surface: Option<C::Surface>,
    /// Renderer for this popup.
    pub renderer: Option<C::Renderer>,
    /// Whether the popup has been configured by the compositor.
    pub configured: bool,
}

/// Manages popup surfaces and their rendering state.
pub struct PopupManager<P, C>
where
    P: Program,
    C: Compositor<Renderer = P::Renderer>,
    P::Theme: theme::Base,
{
    entries: BTreeMap<PopupId, Popup<C>>,
    _marker: std::marker::PhantomData<P>,
}

impl<P, C> PopupManager<P, C>
where
    P: Program,
    C: Compositor<Renderer = P::Renderer>,
    P::Theme: theme::Base,
{
    /// Create a new popup manager.
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            _marker: std::marker::PhantomData,
        }
    }

    /// Insert a new popup (before it's configured).
    pub fn insert(
        &mut self,
        id: PopupId,
        iced_id: window::Id,
        parent_id: window::Id,
        size: Size<u32>,
        scale_factor: f32,
    ) {
        let _ = self.entries.insert(
            id,
            Popup {
                id,
                iced_id,
                parent_id,
                winit_popup_id: None,
                size,
                scale_factor,
                viewport: None,
                surface: None,
                renderer: None,
                configured: false,
            },
        );
    }

    /// Mark a popup as configured and set up rendering surfaces.
    pub fn configure(
        &mut self,
        id: PopupId,
        winit_popup_id: u64,
        width: u32,
        height: u32,
        popup_surface: PopupSurface,
        compositor: &mut C,
    ) -> bool {
        if let Some(popup) = self.entries.get_mut(&id) {
            popup.winit_popup_id = Some(winit_popup_id);
            popup.configured = true;

            // width/height from Wayland are in logical coordinates
            // Convert to physical size for rendering
            let physical_width = (width as f32 * popup.scale_factor).ceil() as u32;
            let physical_height = (height as f32 * popup.scale_factor).ceil() as u32;

            popup.size = Size::new(physical_width, physical_height);

            // Create viewport for rendering using physical size
            popup.viewport = Some(Viewport::with_physical_size(
                Size::new(physical_width, physical_height),
                popup.scale_factor,
            ));

            // Create compositor surface for rendering at physical size
            let surface = compositor.create_surface(popup_surface, physical_width, physical_height);
            let renderer = compositor.create_renderer();

            popup.surface = Some(surface);
            popup.renderer = Some(renderer);

            true
        } else {
            false
        }
    }

    /// Get a popup by ID.
    #[allow(dead_code)]
    pub fn get(&self, id: PopupId) -> Option<&Popup<C>> {
        self.entries.get(&id)
    }

    /// Get a mutable reference to a popup.
    #[allow(dead_code)]
    pub fn get_mut(&mut self, id: PopupId) -> Option<&mut Popup<C>> {
        self.entries.get_mut(&id)
    }

    /// Remove a popup.
    pub fn remove(&mut self, id: PopupId) -> Option<Popup<C>> {
        self.entries.remove(&id)
    }

    /// Remove a popup by its iced window ID.
    pub fn remove_by_iced_id(&mut self, iced_id: window::Id) -> Option<Popup<C>> {
        let popup_id = self
            .entries
            .iter()
            .find(|(_, p)| p.iced_id == iced_id)
            .map(|(id, _)| *id)?;
        self.entries.remove(&popup_id)
    }

    /// Find a popup by its winit popup ID.
    pub fn find_by_winit_id(&self, winit_popup_id: u64) -> Option<&Popup<C>> {
        self.entries
            .values()
            .find(|p| p.winit_popup_id == Some(winit_popup_id))
    }

    /// Resize a popup by its iced window ID.
    ///
    /// Updates the popup's size, viewport, and reconfigures the compositor
    /// surface. Returns the winit popup ID and parent ID if successful.
    pub fn resize(
        &mut self,
        iced_id: window::Id,
        width: u32,
        height: u32,
        compositor: &mut C,
    ) -> Option<(u64, window::Id)> {
        let popup = self.entries.values_mut().find(|p| p.iced_id == iced_id)?;

        let scale = popup.scale_factor;
        let physical_w = (width as f32 * scale).ceil() as u32;
        let physical_h = (height as f32 * scale).ceil() as u32;

        popup.size = Size::new(physical_w, physical_h);
        popup.viewport = Some(Viewport::with_physical_size(
            Size::new(physical_w, physical_h),
            scale,
        ));

        if let Some(ref mut surface) = popup.surface {
            compositor.configure_surface(surface, physical_w, physical_h);
        }

        let winit_id = popup.winit_popup_id?;
        Some((winit_id, popup.parent_id))
    }

    /// Check if manager is empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterate over all popups.
    pub fn iter(&self) -> impl Iterator<Item = (&PopupId, &Popup<C>)> {
        self.entries.iter()
    }

    /// Iterate mutably over all popups.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&PopupId, &mut Popup<C>)> {
        self.entries.iter_mut()
    }

    /// Get all configured popups that are ready for rendering.
    #[allow(dead_code)]
    pub fn configured_popups(&mut self) -> impl Iterator<Item = &mut Popup<C>> {
        self.entries.values_mut().filter(|p| p.configured)
    }
}

impl<P, C> Default for PopupManager<P, C>
where
    P: Program,
    C: Compositor<Renderer = P::Renderer>,
    P::Theme: theme::Base,
{
    fn default() -> Self {
        Self::new()
    }
}
