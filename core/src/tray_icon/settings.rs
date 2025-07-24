//! Tray icon settings

use std::fmt::{self, Debug, Display};

#[cfg(feature = "tray-icon")]
use crate::tray_icon::Error;

use crate::{
    Size,
    keyboard::{Modifiers, key::Code},
};

/// Tray icon settings
#[derive(Debug)]
pub struct Settings {
    /// Title of the icon
    pub title: Option<String>,
    /// Icon to show (not available on web)
    pub icon: Option<Icon>,
    /// Tooltip to show on hover
    pub tooltip: Option<String>,
    /// Menu items
    pub menu_items: Option<Vec<MenuItem>>,
}

/// Displays a key shortcut next to menu item
/// Only triggers when menu is open
#[derive(Debug)]
pub struct Accelerator(pub Code, pub Modifiers);

#[cfg(feature = "tray-icon")]
impl TryFrom<Accelerator> for tray_icon::menu::accelerator::Accelerator {
    type Error = Error;

    fn try_from(value: Accelerator) -> Result<Self, Self::Error> {
        let code = value.0.into();
        let modifiers = if value.1.is_empty() {
            None
        } else {
            let mut m = tray_icon::menu::accelerator::Modifiers::empty();
            if value.1.alt() {
                m |= tray_icon::menu::accelerator::Modifiers::ALT;
            }
            if value.1.shift() {
                m |= tray_icon::menu::accelerator::Modifiers::SHIFT;
            }
            if value.1.control() {
                m |= tray_icon::menu::accelerator::Modifiers::CONTROL;
            }
            if value.1.logo() {
                m |= tray_icon::menu::accelerator::Modifiers::SUPER;
            }
            Some(m)
        };
        Ok(Self::new(modifiers, code))
    }
}

/// About description of the application
#[derive(Debug)]
pub struct AboutMetadata {
    /// Name of the application
    pub name: Option<String>,
    /// Version identifier
    pub version: Option<String>,
    /// Short version identifier
    pub short_version: Option<String>,
    /// Authors list
    pub authors: Option<Vec<String>>,
    /// Comments
    pub comments: Option<String>,
    /// Copyright details
    pub copyright: Option<String>,
    /// License name
    pub license: Option<String>,
    /// Website address
    pub website: Option<String>,
    /// Label fdor website
    pub website_label: Option<String>,
    /// Credits
    pub credits: Option<String>,
    /// Icon for about dialog
    pub icon: Option<Icon>,
}

#[cfg(feature = "tray-icon")]
impl TryFrom<AboutMetadata> for tray_icon::menu::AboutMetadata {
    type Error = Error;

    fn try_from(value: AboutMetadata) -> Result<Self, Self::Error> {
        let icon: Option<tray_icon::menu::Icon> = if let Some(icon) = value.icon
        {
            let icon = icon.try_into()?;
            Some(icon)
        } else {
            None
        };
        Ok(tray_icon::menu::AboutMetadata {
            name: value.name,
            version: value.version,
            short_version: value.short_version,
            authors: value.authors,
            comments: value.comments,
            copyright: value.copyright,
            license: value.license,
            website: value.website,
            website_label: value.website_label,
            credits: value.credits,
            icon: icon,
        })
    }
}

/// Predefined Menu Item configs
#[derive(Debug)]
pub enum PredefinedMenuItem {
    /// Separator between menu items
    Separator,
    /// Copy
    Copy,
    /// Cut
    Cut,
    /// Paste
    Paste,
    /// Select all
    SelectAll,
    /// Undo
    Undo,
    /// Redo
    Redo,
    /// Minimize
    Minimize,
    /// Maximize
    Maximize,
    /// Fullscreen
    Fullscreen,
    /// Hide
    Hide,
    /// Hide others
    HideOthers,
    /// ShowAll
    ShowAll,
    /// CloseWindow
    CloseWindow,
    /// Quit
    Quit,
    /// About
    About(Option<AboutMetadata>),
    /// Services
    Services,
    /// Bring all to front
    BringAllToFront,
}

impl Display for PredefinedMenuItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

/// Tray Icon Menu types
#[derive(Debug)]
pub enum MenuItem {
    /// Define a new submenu
    Submenu {
        /// Id of the Menu Item
        id: String,
        /// Text to show for menu item
        text: String,
        /// Menu items in submenu
        menu_items: Vec<MenuItem>,
    },
    /// Define a predefined menu item
    Predefined {
        /// The menu item type
        predefined_type: PredefinedMenuItem,
        /// Override the text of the menu item
        alternate_text: Option<String>,
    },
    /// Define a menu item with text
    Text {
        /// Id of the Menu Item
        id: String,
        /// Text to show for menu item
        text: String,
        /// Is menu item enabled
        enabled: bool,
        /// Key shortcut
        accelerator: Option<Accelerator>,
    },
    /// Define a menu item with a checkbox
    Check {
        /// Id of the Menu Item
        id: String,
        /// Text to show for menu item
        text: String,
        /// Is menu item enabled
        enabled: bool,
        /// Is the checkbox checked
        checked: bool,
        /// Key shortcut
        accelerator: Option<Accelerator>,
    },
    /// Define a menu item with an icon
    Icon {
        /// Id of the Menu Item
        id: String,
        /// Text to show for menu item
        text: String,
        /// Is menu item enabled
        enabled: bool,
        /// Icon for the menu item
        icon: Icon,
        /// Key shortcut
        accelerator: Option<Accelerator>,
    },
}

impl MenuItem {
    /// Get the Id of the MenuItem
    pub fn id(&self) -> String {
        match self {
            Self::Submenu { id, .. } => id.clone(),
            Self::Text { id, .. } => id.clone(),
            Self::Check { id, .. } => id.clone(),
            Self::Icon { id, .. } => id.clone(),
            Self::Predefined { predefined_type, .. } => predefined_type.to_string(),
        }
    }
}

/// Icon data
#[derive(Debug)]
#[allow(dead_code)]
pub struct Icon {
    /// RGBA byte data of icon image
    pub rgba: Vec<u8>,
    /// Size of icon image
    pub size: Size<u32>,
}

#[cfg(feature = "tray-icon")]
impl TryFrom<Icon> for tray_icon::Icon {
    type Error = Error;
    fn try_from(value: Icon) -> Result<Self, Self::Error> {
        Self::from_rgba(value.rgba, value.size.width, value.size.height)
            .map_err(Self::Error::from)
    }
}

#[cfg(feature = "tray-icon")]
impl TryFrom<Icon> for tray_icon::menu::Icon {
    type Error = Error;
    fn try_from(value: Icon) -> Result<Self, Self::Error> {
        Self::from_rgba(value.rgba, value.size.width, value.size.height)
            .map_err(Self::Error::from)
    }
}

#[allow(unused_macros)]
/// Macro rule for easing enum conversion boilerplate even just a little
macro_rules! convert_enum {
    ($src: ident, $dst: path, [$($src_variant: ident, $dst_variant: ident,)*], $($variant: ident,)*) => {
        impl From<$src> for $dst {
            fn from(src: $src) -> Self {
                match src {
                    $($src::$src_variant => Self::$dst_variant,)*
                    $($src::$variant => Self::$variant,)*
                }
            }
        }
    };
}

#[cfg(feature = "tray-icon")]
convert_enum!(
    Code,
    tray_icon::menu::accelerator::Code,
    [Meta, Super, SuperLeft, MetaLeft, SuperRight, MetaRight,],
    Backquote,
    Backslash,
    BracketLeft,
    BracketRight,
    Comma,
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    Equal,
    IntlBackslash,
    IntlRo,
    IntlYen,
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyG,
    KeyH,
    KeyI,
    KeyJ,
    KeyK,
    KeyL,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyV,
    KeyW,
    KeyX,
    KeyY,
    KeyZ,
    Minus,
    Period,
    Quote,
    Semicolon,
    Slash,
    AltLeft,
    AltRight,
    Backspace,
    CapsLock,
    ContextMenu,
    ControlLeft,
    ControlRight,
    Enter,
    ShiftLeft,
    ShiftRight,
    Space,
    Tab,
    Convert,
    KanaMode,
    Lang1,
    Lang2,
    Lang3,
    Lang4,
    Lang5,
    NonConvert,
    Delete,
    End,
    Help,
    Home,
    Insert,
    PageDown,
    PageUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    NumLock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadBackspace,
    NumpadClear,
    NumpadClearEntry,
    NumpadComma,
    NumpadDecimal,
    NumpadDivide,
    NumpadEnter,
    NumpadEqual,
    NumpadHash,
    NumpadMemoryAdd,
    NumpadMemoryClear,
    NumpadMemoryRecall,
    NumpadMemoryStore,
    NumpadMemorySubtract,
    NumpadMultiply,
    NumpadParenLeft,
    NumpadParenRight,
    NumpadStar,
    NumpadSubtract,
    Escape,
    Fn,
    FnLock,
    PrintScreen,
    ScrollLock,
    Pause,
    BrowserBack,
    BrowserFavorites,
    BrowserForward,
    BrowserHome,
    BrowserRefresh,
    BrowserSearch,
    BrowserStop,
    Eject,
    LaunchApp1,
    LaunchApp2,
    LaunchMail,
    MediaPlayPause,
    MediaSelect,
    MediaStop,
    MediaTrackNext,
    MediaTrackPrevious,
    Power,
    Sleep,
    AudioVolumeDown,
    AudioVolumeMute,
    AudioVolumeUp,
    WakeUp,
    Hyper,
    Turbo,
    Abort,
    Resume,
    Suspend,
    Again,
    Copy,
    Cut,
    Find,
    Open,
    Paste,
    Props,
    Select,
    Undo,
    Hiragana,
    Katakana,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    F26,
    F27,
    F28,
    F29,
    F30,
    F31,
    F32,
    F33,
    F34,
    F35,
);