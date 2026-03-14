//! Operate on widgets that expose accessibility metadata.
//!
//! Unlike the sibling [`focusable`], [`scrollable`], and [`text_input`]
//! modules, this module does not define a trait for widgets to implement.
//! Instead, widgets construct an [`Accessible`] value and pass it to
//! [`Operation::accessible`] during their [`operate`] method.
//!
//! [`focusable`]: super::focusable
//! [`scrollable`]: super::scrollable
//! [`text_input`]: super::text_input
//! [`Operation::accessible`]: super::Operation::accessible
//! [`operate`]: crate::widget::Widget::operate

use crate::widget;

/// The role a widget plays in the accessibility tree.
///
/// Used by assistive technology to convey the purpose and interaction
/// model of a widget to the user. Defaults to [`Role::Group`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Role {
    /// An alert message.
    Alert,
    /// A dialog conveying an alert.
    AlertDialog,
    /// A push button.
    Button,
    /// A canvas element for custom drawing.
    Canvas,
    /// A check box.
    CheckBox,
    /// A combo box / drop-down list.
    ComboBox,
    /// A dialog or modal window.
    Dialog,
    /// A document-like content area.
    Document,
    /// A generic container grouping related widgets.
    #[default]
    Group,
    /// A heading element (used with levels 1--6).
    Heading,
    /// A raster or vector image.
    Image,
    /// A label for another widget.
    Label,
    /// A hyperlink.
    Link,
    /// A list container.
    List,
    /// An item within a list.
    ListItem,
    /// A menu container.
    Menu,
    /// A menu bar container.
    MenuBar,
    /// An item within a menu.
    MenuItem,
    /// A meter or gauge.
    Meter,
    /// A multiline text input field.
    MultilineTextInput,
    /// A navigation landmark.
    Navigation,
    /// A progress indicator.
    ProgressIndicator,
    /// A radio button.
    RadioButton,
    /// A generic landmark region.
    Region,
    /// A scrollbar control.
    ScrollBar,
    /// A scrollable area.
    ScrollView,
    /// A search landmark.
    Search,
    /// A visual separator between sections.
    Separator,
    /// A slider control.
    Slider,
    /// Non-interactive text content.
    StaticText,
    /// A status message area.
    Status,
    /// A toggle switch.
    Switch,
    /// A single tab within a tab list.
    Tab,
    /// A container of tabs.
    TabList,
    /// A panel associated with a tab.
    TabPanel,
    /// A data table.
    Table,
    /// A text input field.
    TextInput,
    /// A toolbar container.
    Toolbar,
    /// A tooltip popup.
    Tooltip,
    /// A tree view.
    Tree,
    /// An item within a tree view.
    TreeItem,
    /// A window or pane.
    Window,
}

/// The current value of a widget, exposed to assistive technology.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value<'a> {
    /// A textual value (e.g. text input content).
    Text(&'a str),
    /// A numeric value with its valid range.
    Numeric {
        /// The current value.
        current: f64,
        /// The minimum value.
        min: f64,
        /// The maximum value.
        max: f64,
        /// The step increment, if any.
        step: Option<f64>,
    },
}

/// How urgently assistive technology should announce changes to a
/// widget's content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Live {
    /// Changes are announced at the next graceful opportunity.
    Polite,
    /// Changes are announced immediately, interrupting the current
    /// speech.
    Assertive,
}

/// The orientation of a widget (e.g. a slider or toolbar).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    /// Horizontal layout (default for most widgets).
    Horizontal,
    /// Vertical layout.
    Vertical,
}

/// Accessibility metadata for a single widget.
///
/// Passed to [`Operation::accessible`] by widgets that wish to
/// participate in the accessibility tree. All fields beyond [`role`]
/// are optional; widgets should only populate the fields that apply.
///
/// [`role`]: Accessible::role
/// [`Operation::accessible`]: super::Operation::accessible
#[derive(Debug, Clone, Default)]
pub struct Accessible<'a> {
    /// The semantic role of the widget.
    pub role: Role,
    /// A human-readable name for the widget (e.g. button label).
    pub label: Option<&'a str>,
    /// A longer human-readable description (e.g. tooltip text).
    pub description: Option<&'a str>,
    /// The current value, if the widget carries one.
    pub value: Option<Value<'a>>,
    /// Whether the widget is disabled.
    pub disabled: bool,
    /// The toggle state, for widgets like check boxes and switches.
    pub toggled: Option<bool>,
    /// The selection state, for widgets like radio buttons and list
    /// items.
    pub selected: Option<bool>,
    /// Whether a collapsible section is expanded.
    pub expanded: Option<bool>,
    /// The live-region setting, for widgets whose content changes
    /// should be announced by assistive technology.
    pub live: Option<Live>,
    /// The heading level (1--6), for widgets with [`Role::Heading`].
    pub level: Option<usize>,
    /// Whether the widget is required (e.g. a required form field).
    pub required: bool,
    /// The widget's orientation, for sliders and toolbars.
    pub orientation: Option<Orientation>,
    /// Another widget that provides this widget's label.
    ///
    /// Use this instead of [`label`](Self::label) when the label
    /// comes from a separate widget in the tree.
    pub labelled_by: Option<&'a widget::Id>,
    /// Another widget that provides this widget's description.
    ///
    /// Use this instead of [`description`](Self::description) when
    /// the description comes from a separate widget in the tree.
    pub described_by: Option<&'a widget::Id>,
}
