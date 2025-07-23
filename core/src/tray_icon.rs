//! Tray icon

mod errors;
mod event;
mod settings;

use std::fmt::{Debug, Formatter};
use std::collections::HashMap;

pub use errors::Error;
pub use event::Event;
pub use settings::*;

/// Wrapper type for tray_icon
#[derive(Clone)]
pub struct TrayIcon {
    #[cfg(feature = "tray-icon")]
    icon: tray_icon::TrayIcon,
    /// Mapping of MenuItem id and tray_icon MenuId
    id_map: HashMap<String, String>,
}

impl TrayIcon {
    #[cfg(not(feature = "tray-icon"))]
    /// Create new TrayIcon from Settings
    pub fn new(_settings: Settings) -> Result<Self, Error> {
        Ok(Self {
            id_map: HashMap::new(),
        })
    }

    #[cfg(feature = "tray-icon")]
    /// Create new TrayIcon from Settings
    pub fn new(settings: Settings) -> Result<Self, Error> {
        let mut attrs = tray_icon::TrayIconAttributes::default();
        if let Some(title) = settings.title {
            attrs.title = Some(title.clone());
        }
        if let Some(icon) = settings.icon {
            let icon = icon.try_into()?;
            attrs.icon = Some(icon);
        }
        if let Some(tooltip) = settings.tooltip {
            attrs.tooltip = Some(tooltip.clone());
        }
        let id_map = if let Some(menu_items) = settings.menu_items {
            let mut id_map =
                HashMap::with_capacity(menu_items.len());
            let menu = tray_icon::menu::Menu::new();
            for menu_item in menu_items {
                Self::build_menu_item(&mut id_map, &menu, menu_item)?;
            }
            attrs.menu = Some(Box::new(menu));
            id_map
        } else {
            HashMap::new()
        };
        let icon = tray_icon::TrayIcon::new(attrs).map_err(Error::from)?;
        let this = Self {
            icon: icon,
            id_map: id_map,
        };

        Ok(this)
    }

    #[cfg(feature = "tray-icon")]
    fn build_menu_item(
        id_map: &mut HashMap<String, String>,
        menu: &impl tray_icon::menu::ContextMenu,
        menu_item: MenuItem,
    ) -> Result<(), Error> {
        let menu_id = menu_item.id();
        let add_to_menu =
            |item: &dyn tray_icon::menu::IsMenuItem,
             id_map: &mut HashMap<String, String>|
             -> Result<(), Error> {
                if let Some(menu) = menu.as_menu() {
                    let _ = id_map.insert(item.id().0.clone(), menu_id);
                    menu.append(item).map_err(Error::from)
                } else if let Some(submenu) = menu.as_submenu() {
                    let _ = id_map.insert(item.id().0.clone(), menu_id);
                    submenu.append(item).map_err(Error::from)
                } else {
                    Err(Error::MenuError(
                        tray_icon::menu::Error::NotAChildOfThisMenu,
                    ))
                }
            };
        match menu_item {
            MenuItem::Submenu {
                text, menu_items, ..
            } => {
                let submenu = tray_icon::menu::Submenu::new(text, true);
                for sub_menu_item in menu_items {
                    Self::build_menu_item(id_map, &submenu, sub_menu_item)?;
                }
                add_to_menu(&submenu, id_map)
            }
            MenuItem::Predefined {
                predefined_type,
                alternate_text,
            } => {
                let p = match predefined_type {
                    PredefinedMenuItem::Separator => {
                        tray_icon::menu::PredefinedMenuItem::separator()
                    }
                    PredefinedMenuItem::Copy => {
                        tray_icon::menu::PredefinedMenuItem::copy(
                            alternate_text.as_deref(),
                        )
                    }
                    PredefinedMenuItem::Cut => {
                        tray_icon::menu::PredefinedMenuItem::cut(
                            alternate_text.as_deref(),
                        )
                    }
                    PredefinedMenuItem::Paste => {
                        tray_icon::menu::PredefinedMenuItem::paste(
                            alternate_text.as_deref(),
                        )
                    }
                    PredefinedMenuItem::SelectAll => {
                        tray_icon::menu::PredefinedMenuItem::select_all(
                            alternate_text.as_deref(),
                        )
                    }
                    PredefinedMenuItem::Undo => {
                        tray_icon::menu::PredefinedMenuItem::undo(
                            alternate_text.as_deref(),
                        )
                    }
                    PredefinedMenuItem::Redo => {
                        tray_icon::menu::PredefinedMenuItem::redo(
                            alternate_text.as_deref(),
                        )
                    }
                    PredefinedMenuItem::Minimize => {
                        tray_icon::menu::PredefinedMenuItem::minimize(
                            alternate_text.as_deref(),
                        )
                    }
                    PredefinedMenuItem::Maximize => {
                        tray_icon::menu::PredefinedMenuItem::maximize(
                            alternate_text.as_deref(),
                        )
                    }
                    PredefinedMenuItem::Fullscreen => {
                        tray_icon::menu::PredefinedMenuItem::fullscreen(
                            alternate_text.as_deref(),
                        )
                    }
                    PredefinedMenuItem::Hide => {
                        tray_icon::menu::PredefinedMenuItem::hide(
                            alternate_text.as_deref(),
                        )
                    }
                    PredefinedMenuItem::HideOthers => {
                        tray_icon::menu::PredefinedMenuItem::hide_others(
                            alternate_text.as_deref(),
                        )
                    }
                    PredefinedMenuItem::ShowAll => {
                        tray_icon::menu::PredefinedMenuItem::show_all(
                            alternate_text.as_deref(),
                        )
                    }
                    PredefinedMenuItem::CloseWindow => {
                        tray_icon::menu::PredefinedMenuItem::close_window(
                            alternate_text.as_deref(),
                        )
                    }
                    PredefinedMenuItem::Quit => {
                        tray_icon::menu::PredefinedMenuItem::quit(
                            alternate_text.as_deref(),
                        )
                    }
                    PredefinedMenuItem::About(about_metadata) => {
                        let a: Option<tray_icon::menu::AboutMetadata> =
                            if let Some(a) = about_metadata {
                                let about = a.try_into()?;
                                Some(about)
                            } else {
                                None
                            };
                        tray_icon::menu::PredefinedMenuItem::about(
                            alternate_text.as_deref(),
                            a,
                        )
                    }
                    PredefinedMenuItem::Services => {
                        tray_icon::menu::PredefinedMenuItem::services(
                            alternate_text.as_deref(),
                        )
                    }
                    PredefinedMenuItem::BringAllToFront => {
                        tray_icon::menu::PredefinedMenuItem::bring_all_to_front(
                            alternate_text.as_deref(),
                        )
                    }
                };
                add_to_menu(&p, id_map)
            }
            MenuItem::Text {
                text,
                enabled,
                accelerator,
                ..
            } => {
                let a: Option<tray_icon::menu::accelerator::Accelerator> =
                    if let Some(a) = accelerator {
                        let a = a.try_into()?;
                        Some(a)
                    } else {
                        None
                    };
                let t = tray_icon::menu::MenuItem::new(text, enabled, a);
                add_to_menu(&t, id_map)
            }
            MenuItem::Check {
                text,
                enabled,
                checked,
                accelerator,
                ..
            } => {
                let a: Option<tray_icon::menu::accelerator::Accelerator> =
                    if let Some(a) = accelerator {
                        let a = a.try_into()?;
                        Some(a)
                    } else {
                        None
                    };
                let c = tray_icon::menu::CheckMenuItem::new(
                    text, enabled, checked, a,
                );
                add_to_menu(&c, id_map)
            }
            MenuItem::Icon {
                text,
                enabled,
                icon,
                accelerator,
                ..
            } => {
                let i = icon.try_into()?;
                let a: Option<tray_icon::menu::accelerator::Accelerator> =
                    if let Some(a) = accelerator {
                        let a = a.try_into()?;
                        Some(a)
                    } else {
                        None
                    };
                let c = tray_icon::menu::IconMenuItem::new(
                    text,
                    enabled,
                    Some(i),
                    a,
                );
                add_to_menu(&c, id_map)
            }
        }
    }

    /// Fetch MenuItem Id for tray_icon MenuId
    pub fn id_map(&self) -> HashMap<String, String> {
        self.id_map.clone()
    }
}


impl Debug for TrayIcon {
    #[cfg(feature = "tray-icon")]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrayIcon")
            .field("icon", &self.icon.id())
            .field("id_map", &self.id_map)
            .finish()
    }

    #[cfg(not(feature = "tray-icon"))]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrayIcon")
            .field("id_map", &self.id_map)
            .finish()
    }
}
