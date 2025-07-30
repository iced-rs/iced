//! Tray icon events

#[cfg(feature = "tray-icon")]
use tray_icon::{TrayIconEvent, menu::MenuEvent};

use crate::{Point, Rectangle, mouse::Button};

/// A tray icon interaction
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// Tray icon click started
    MouseButtonPressed {
        /// Id of the tray icon which triggered this event.
        id: String,
        /// Position of the mouse cursor
        position: Point,
        /// Bounding rectangle of the tray icon
        rect: Rectangle,
        /// Mouse button which triggered this event [Left, Middle, Right]
        button: Button,
    },
    /// Tray icon click ended
    MouseButtonReleased {
        /// Id of the tray icon which triggered this event.
        id: String,
        /// Position of the mouse cursor
        position: Point,
        /// Bounding rectangle of the tray icon
        rect: Rectangle,
        /// Mouse button which triggered this event [Left, Middle, Right]
        button: Button,
    },
    /// Tray icon double clicked
    DoubleClicked {
        /// Id of the tray icon which triggered this event.
        id: String,
        /// Position of the mouse cursor
        position: Point,
        /// Bounding rectangle of the tray icon
        rect: Rectangle,
        /// Mouse button which triggered this event [Left, Middle, Right]
        button: Button,
    },
    /// Mouse entered tray icon
    MouseEntered {
        /// Id of the tray icon which triggered this event.
        id: String,
        /// Position of the mouse cursor
        position: Point,
        /// Bounding rectangle of the tray icon
        rect: Rectangle,
    },
    /// Mouse moved over tray icon
    MouseMoved {
        /// Id of the tray icon which triggered this event.
        id: String,
        /// Position of the mouse cursor
        position: Point,
        /// Bounding rectangle of the tray icon
        rect: Rectangle,
    },
    /// Mouse exited tray icon
    MouseExited {
        /// Id of the tray icon which triggered this event.
        id: String,
        /// Position of the mouse cursor
        position: Point,
        /// Bounding rectangle of the tray icon
        rect: Rectangle,
    },
    /// Tray icon menu item clicked
    MenuItemClicked {
        /// Id of the tray icon which triggered this event.
        id: String,
    },
}

#[cfg(feature = "tray-icon")]
impl From<MenuEvent> for Event {
    fn from(value: MenuEvent) -> Self {
        Self::MenuItemClicked { id: value.id.0 }
    }
}

#[cfg(feature = "tray-icon")]
impl From<TrayIconEvent> for Event {
    fn from(value: TrayIconEvent) -> Self {
        match value {
            TrayIconEvent::Click {
                id,
                position,
                rect,
                button,
                button_state,
            } => match button_state {
                tray_icon::MouseButtonState::Up => Self::MouseButtonPressed {
                    id: id.0,
                    position: Point {
                        x: position.x as f32,
                        y: position.y as f32,
                    },
                    rect: Rectangle {
                        x: rect.position.x as f32,
                        y: rect.position.y as f32,
                        width: rect.size.width as f32,
                        height: rect.size.height as f32,
                    },
                    button: match button {
                        tray_icon::MouseButton::Left => Button::Left,
                        tray_icon::MouseButton::Middle => Button::Middle,
                        tray_icon::MouseButton::Right => Button::Right,
                    },
                },
                tray_icon::MouseButtonState::Down => {
                    Self::MouseButtonReleased {
                        id: id.0,
                        position: Point {
                            x: position.x as f32,
                            y: position.y as f32,
                        },
                        rect: Rectangle {
                            x: rect.position.x as f32,
                            y: rect.position.y as f32,
                            width: rect.size.width as f32,
                            height: rect.size.height as f32,
                        },
                        button: match button {
                            tray_icon::MouseButton::Left => Button::Left,
                            tray_icon::MouseButton::Middle => Button::Middle,
                            tray_icon::MouseButton::Right => Button::Right,
                        },
                    }
                }
            },
            TrayIconEvent::DoubleClick {
                id,
                position,
                rect,
                button,
            } => Self::DoubleClicked {
                id: id.0,
                position: Point {
                    x: position.x as f32,
                    y: position.y as f32,
                },
                rect: Rectangle {
                    x: rect.position.x as f32,
                    y: rect.position.y as f32,
                    width: rect.size.width as f32,
                    height: rect.size.height as f32,
                },
                button: match button {
                    tray_icon::MouseButton::Left => Button::Left,
                    tray_icon::MouseButton::Middle => Button::Middle,
                    tray_icon::MouseButton::Right => Button::Right,
                },
            },
            TrayIconEvent::Enter { id, position, rect } => Self::MouseEntered {
                id: id.0,
                position: Point {
                    x: position.x as f32,
                    y: position.y as f32,
                },
                rect: Rectangle {
                    x: rect.position.x as f32,
                    y: rect.position.y as f32,
                    width: rect.size.width as f32,
                    height: rect.size.height as f32,
                },
            },
            TrayIconEvent::Move { id, position, rect } => Self::MouseMoved {
                id: id.0,
                position: Point {
                    x: position.x as f32,
                    y: position.y as f32,
                },
                rect: Rectangle {
                    x: rect.position.x as f32,
                    y: rect.position.y as f32,
                    width: rect.size.width as f32,
                    height: rect.size.height as f32,
                },
            },
            TrayIconEvent::Leave { id, position, rect } => Self::MouseExited {
                id: id.0,
                position: Point {
                    x: position.x as f32,
                    y: position.y as f32,
                },
                rect: Rectangle {
                    x: rect.position.x as f32,
                    y: rect.position.y as f32,
                    width: rect.size.width as f32,
                    height: rect.size.height as f32,
                },
            },
            _ => todo!("Unknown TrayIconEvent"),
        }
    }
}
