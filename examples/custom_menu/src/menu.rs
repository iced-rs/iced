use muda::{
    AboutMetadata, CheckMenuItem, IconMenuItem, Menu, MenuItem,
    PredefinedMenuItem, Submenu,
    accelerator::{Accelerator, Code, Modifiers},
};

pub struct AppMenu {
    pub menu_bar: Menu,
    _file_menu: Submenu,
    _edit_menu: Submenu,
    pub window_menu: Submenu,
    pub custom_item: MenuItem,
}

impl AppMenu {
    pub fn new(menu_bar: Menu) -> Self {
        #[cfg(target_os = "macos")]
        {
            let app_menu = Submenu::new("App", true);
            app_menu
                .append_items(&[
                    &PredefinedMenuItem::about(None, None),
                    &PredefinedMenuItem::separator(),
                    &PredefinedMenuItem::services(None),
                    &PredefinedMenuItem::separator(),
                    &PredefinedMenuItem::hide(None),
                    &PredefinedMenuItem::hide_others(None),
                    &PredefinedMenuItem::show_all(None),
                    &PredefinedMenuItem::separator(),
                    &PredefinedMenuItem::quit(None),
                ])
                .unwrap();
            menu_bar.append(&app_menu).unwrap();
        }

        let file_menu = Submenu::new("&File", true);
        let edit_menu = Submenu::new("&Edit", true);
        let window_menu = Submenu::new("&Window", true);

        menu_bar
            .append_items(&[&file_menu, &edit_menu, &window_menu])
            .unwrap();

        let custom_i_1 = MenuItem::new(
            "C&ustom 1",
            true,
            Some(Accelerator::new(Some(Modifiers::ALT), Code::KeyC)),
        );

        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/icon.png");
        let icon = load_icon(std::path::Path::new(path));
        let image_item =
            IconMenuItem::new("Image Custom 1", true, Some(icon), None);

        let check_custom_i_1 =
            CheckMenuItem::new("Check Custom 1", true, true, None);
        let check_custom_i_2 =
            CheckMenuItem::new("Check Custom 2", false, true, None);
        let check_custom_i_3 = CheckMenuItem::new(
            "Check Custom 3",
            true,
            true,
            Some(Accelerator::new(Some(Modifiers::SHIFT), Code::KeyD)),
        );

        let copy_i = PredefinedMenuItem::copy(None);
        let cut_i = PredefinedMenuItem::cut(None);
        let paste_i = PredefinedMenuItem::paste(None);

        file_menu
            .append_items(&[
                &custom_i_1,
                &image_item,
                &window_menu,
                &PredefinedMenuItem::separator(),
                &check_custom_i_1,
                &check_custom_i_2,
            ])
            .unwrap();

        window_menu
            .append_items(&[
                &PredefinedMenuItem::minimize(None),
                &PredefinedMenuItem::maximize(None),
                &PredefinedMenuItem::close_window(Some("Close")),
                &PredefinedMenuItem::fullscreen(None),
                &PredefinedMenuItem::bring_all_to_front(None),
                &PredefinedMenuItem::about(
                    None,
                    Some(AboutMetadata {
                        name: Some("winit".to_string()),
                        version: Some("1.2.3".to_string()),
                        copyright: Some("Copyright winit".to_string()),
                        ..Default::default()
                    }),
                ),
                &check_custom_i_3,
                &image_item,
                &custom_i_1,
            ])
            .unwrap();

        edit_menu
            .append_items(&[
                &copy_i,
                &cut_i,
                &PredefinedMenuItem::separator(),
                &paste_i,
            ])
            .unwrap();

        Self {
            menu_bar,
            _file_menu: file_menu,
            _edit_menu: edit_menu,
            window_menu,
            custom_item: custom_i_1,
        }
    }

    pub fn init(&self) {
        #[cfg(target_os = "windows")]
        {
            use winit::raw_window_handle::*;
            if let RawWindowHandle::Win32(handle) =
                window.window_handle().unwrap().as_raw()
            {
                unsafe { menu.init_for_hwnd(handle.hwnd.get()) };
            }
            if let RawWindowHandle::Win32(handle) =
                window2.window_handle().unwrap().as_raw()
            {
                unsafe {
                    self.app_menu.menu_bar.init_for_hwnd(handle.hwnd.get())
                };
            }
        }
        #[cfg(target_os = "macos")]
        {
            self.menu_bar.init_for_nsapp();
            self.window_menu.set_as_windows_menu_for_nsapp();
        }
    }
}

fn load_icon(path: &std::path::Path) -> muda::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    muda::Icon::from_rgba(icon_rgba, icon_width, icon_height)
        .expect("Failed to open icon")
}
