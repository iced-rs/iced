# Accessibility for Widget Developers

How to make iced widgets accessible to screen readers and other assistive
technology.

## The basics

Every iced widget can participate in the **accessibility tree** -- a
parallel representation of the UI that assistive technology (AT) reads
instead of looking at pixels. Screen readers like NVDA, VoiceOver, and
Orca traverse this tree and speak what they find: "Submit, button" or
"Email address, text entry, blank."

A widget joins the tree by calling `operation.accessible()` in its
`operate()` method. The call provides a role (what kind of widget this
is), a label (what it's called), and state (disabled, checked, expanded,
etc.). The framework handles everything else -- building the tree, pushing
it to the platform, and routing AT actions back to the widget.

## Making a widget accessible

A typical `operate()` implementation:

```rust
fn operate(
    &mut self,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
    operation: &mut dyn Operation,
) {
    let state = tree.state.downcast_mut::<State>();

    // 1. Expose accessible metadata first
    operation.accessible(
        self.id.as_ref(),
        layout.bounds(),
        &Accessible {
            role: Role::Button,
            label: Some("Submit"),
            disabled: self.on_press.is_none(),
            ..Accessible::default()
        },
    );

    // 2. Then focus/text state (associated with the node above)
    operation.focusable(self.id.as_ref(), layout.bounds(), state);

    // 3. Then children
    operation.container(self.id.as_ref(), layout.bounds());
    operation.traverse(&mut |operation| {
        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            operation,
        );
    });
}
```

Not every widget needs every step:

- A **leaf widget** (slider, progress bar) only needs the `accessible()`
  call.
- A **container widget** (column, row) only needs `container()` +
  `traverse()` to forward to its children.
- A **semantic widget with children** (button, scrollable) needs
  `accessible()` first, then `container()` + `traverse()`.

**Call ordering:** `accessible()` must come before `focusable()`,
`text_input()`, and `text()`. The tree builder associates these calls
with the most recent `accessible()` node -- if `focusable()` comes first,
the focus state gets associated with the wrong node.

## Choosing a role

The role tells the screen reader what kind of widget it's looking at.
Pick the role that matches the widget's *behavior*, not its appearance:

| Widget behavior | Role | Notes |
|----------------|------|-------|
| Clickable action | `Button` | |
| On/off toggle | `Switch` or `CheckBox` | Switch for mode toggles, CheckBox for form fields |
| One-of-many selection | `RadioButton` | |
| Text entry (single line) | `TextInput` | |
| Text entry (multi-line) | `MultilineTextInput` | |
| Numeric range | `Slider` | Set `Value::Numeric` with min/max/step; set `orientation` for vertical |
| Progress indication | `ProgressIndicator` | Set `Value::Numeric` with min/max |
| Dropdown / select | `ComboBox` | Set `expanded` |
| Static graphic | `Image` | Canvas and SVG use this too |
| Expandable section | `Group` | Set `expanded: Some(bool)` |
| Hyperlink | `Link` | |
| Menu item | `MenuItem` | Use `MenuItemCheckBox` or `MenuItemRadio` for stateful items |
| Tab | `Tab` | Inside a `TabList`; set `selected` |
| Modal window | `Dialog` | |
| Scrollable region | `ScrollView` | |
| Visual divider | `Separator` | |
| Structural grouping | `Group` | Default role; use when nothing else fits |

If nothing fits, use `Group`. It tells the screen reader "this is a
container with children" without making claims about interactivity.

The full list of available roles is in the `Role` enum in
`core/src/widget/operation/accessible.rs`. For guidance on which role
matches your widget's interaction model, see the
[ARIA Authoring Practices Guide](https://www.w3.org/WAI/ARIA/apg/patterns/)
-- it organizes by widget behavior rather than role name. If the role you
need isn't available in iced's `Role` enum, open an issue to request it
be added.

## How widgets get their names

The **accessible name** is what the screen reader says first: "Submit" in
"Submit, button." There are three ways a widget gets its name, checked in
this order:

### 1. Direct label (most common)

Set `label` in the `Accessible` struct. This is the simplest approach
for widgets that know their own text:

```rust
Accessible {
    role: Role::CheckBox,
    label: Some("Accept terms"),
    ..Accessible::default()
}
```

### 2. Name from contents (automatic for some roles)

For Button, CheckBox, RadioButton, Link, and MenuItem, accesskit
automatically derives the name from descendant text content. A button
containing a Text widget gets its name from the text:

```rust
// In the view:
button(text("Save"))

// Screen reader announces: "Save, button"
```

This also works through layout containers. A button containing a row
with multiple texts:

```rust
button(row![text("Save"), text("(Ctrl+S)")])
// Screen reader announces: "Save (Ctrl+S), button"
```

### 3. Cross-widget label

When a separate widget provides the label, use `labelled_by`:

```rust
// A text input labelled by a nearby Text widget
let label_id = widget::Id::unique();

column![
    text("Email address").id(label_id.clone()),
    text_input("", &self.email),
]

// In the text input's operate():
Accessible {
    role: Role::TextInput,
    labelled_by: Some(&label_id),
    ..Accessible::default()
}
```

### Icon fonts and name-from-contents

A button like `button(row![icon('\u{F1F8}'), "Delete"])` produces two
text children. Accesskit concatenates them, producing something like
"? Delete" where ? is the non-displayable icon glyph. To avoid this, set
an explicit label -- it takes precedence over descendant-derived names:

```rust
Accessible {
    role: Role::Button,
    label: Some("Delete"),
    ..Accessible::default()
}
```

### Avoiding duplicate announcements

If your widget sets `Accessible.label` and also has a Text child with the
same string, the tree builder detects the duplication and avoids creating
a redundant text node. This is handled automatically -- you don't need to
do anything special.

## What the framework handles for you

The tree builder automatically infers several properties from the role
and fields you set. You don't need to set these:

| What you set | What gets added | Effect |
|-------------|----------------|--------|
| Role is Button, CheckBox, RadioButton, Switch, Link, MenuItem, or Tab | `Action::Click` | Screen reader offers "press" or "activate" |
| `value` is `Numeric` with a `step` | `Action::Increment` + `Action::Decrement` | Screen reader offers value adjustment |
| `expanded` is set | `Action::Expand` + `Action::Collapse` | Screen reader offers open/close |
| Role is `ComboBox` | `HasPopup::Listbox` | Screen reader knows a popup will open |
| Role is `TextInput` or `MultilineTextInput` with a `label` | Placeholder property | Screen reader reads it as hint text |
| `orientation` is set | Orientation property | Screen reader knows horizontal vs vertical |
| `focusable()` is called | `Action::Focus` | Screen reader can move focus to the widget |

Scrollable containers also get scroll position properties, content
clipping for off-screen items, and a ScrollBar child node.

## Widget creation paths

How accessibility works depends on how you build your widget:

**Composing existing widgets** (wrapping Button, TextInput, etc.):
Accessibility is automatic. Container widgets forward the accessibility
visitor to their children. The inner widgets already call `accessible()`.

**Direct Widget implementation** (custom layout and drawing):
Add `operation.accessible()` to your `operate()` method with the
appropriate role and metadata.

**Canvas** (custom 2D geometry via `canvas::Program`):
Canvas calls `accessible()` with `Role::Image` by default. This is
correct for static graphics. For interactive canvas content (a color
picker, a clickable chart), wrap the Canvas in a custom widget whose
`operate()` provides a more descriptive role and label, or restructure
to use semantic widgets for the interactive parts.

**Shader** (wgpu custom rendering via `shader::Program`):
Shader does not call `accessible()`. Wrap it in a custom widget that
provides an `operate()` method with appropriate metadata.

**Responsive / Lazy wrappers:**
Transparent to accessibility. They forward `operate()` to their inner
widget.

## Live regions

Live regions cause the screen reader to announce content changes. Most
widgets don't need them -- they're for content that updates while the
user is looking at (or listening to) something else.

- **`Live::Polite`** -- Queued. The screen reader finishes its current
  speech before announcing the change. Use for ambient status updates
  like progress bars or connection status.

- **`Live::Assertive`** -- Interrupting. The screen reader speaks
  immediately. Reserved for the `announce()` API and urgent alerts.

Setting `Live::Polite` on static text (text that doesn't change) causes
the screen reader to re-announce it on every tree rebuild.

## Setting state

The `Accessible` struct has fields for common widget states. All fields
default to `None`, `false`, or `0` -- set only what applies.

**`disabled: bool`** -- Set to `true` when the widget exists but can't be
interacted with (e.g., a submit button before a form is filled in). The
screen reader announces "dimmed" or "unavailable." Typically driven by
whether the widget's callback is `None`: `disabled: self.on_press.is_none()`.

**`toggled: Option<bool>`** -- For widgets with an on/off state. Set to
`Some(true)` when on/checked, `Some(false)` when off/unchecked. Leave as
`None` for widgets that aren't toggleable. The screen reader announces
"checked" or "not checked." Used by CheckBox and Switch.

**`selected: Option<bool>`** -- For widgets that can be chosen from a set.
Set to `Some(true)` when this option is the active one, `Some(false)`
otherwise. The screen reader announces "selected" or "not selected."
Used by RadioButton and Tab.

**`expanded: Option<bool>`** -- For widgets that show/hide content. Set to
`Some(true)` when the content is visible, `Some(false)` when collapsed.
The screen reader announces "expanded" or "collapsed." Used by ComboBox
and pick lists. The tree builder automatically adds Expand/Collapse
actions when this is set.

**`required: bool`** -- Set to `true` for form fields that must be filled
in. The screen reader announces "required."

**`level: Option<usize>`** -- For heading widgets. Set to the heading
level (1 through 6). Screen readers use this for heading navigation
(e.g., NVDA's H key jumps between headings).

**`orientation: Option<Orientation>`** -- For widgets where the axis
matters. Set to `Some(Orientation::Vertical)` for vertical sliders or
vertical tab lists. The default for most widgets is horizontal, so only
set this when the widget is vertical.

**`live: Option<Live>`** -- For content that updates while visible. Set
to `Some(Live::Polite)` for ambient updates (progress bars, status text)
or `Some(Live::Assertive)` for urgent alerts. See the
[Live regions](#live-regions) section.

**`value: Option<Value>`** -- The widget's current content. Two variants:
- `Value::Text("current text")` -- for text inputs and editors.
- `Value::Numeric { current, min, max, step }` -- for sliders and
  progress bars. Include `step: Some(n)` when the value is adjustable
  (sliders); omit step for read-only indicators (progress bars).

**`description: Option<&str>`** -- A longer explanation beyond the label.
The screen reader speaks this after the label and role. Use for help text
or context that isn't the widget's name (e.g., "Must be at least 8
characters" on a password field).

Use struct update syntax to set only what applies:

```rust
Accessible {
    role: Role::Slider,
    value: Some(Value::Numeric {
        current: 50.0,
        min: 0.0,
        max: 100.0,
        step: Some(1.0),
    }),
    orientation: Some(Orientation::Vertical),
    ..Accessible::default()
}
```

## Testing your widget

### Automated

The selector API lets you verify that your widget appears in the
accessibility tree with the right role and label. This works in any
project that depends on `iced_test`:

```rust
use iced_test::simulator;
use iced::widget::selector::{by_role, by_label};
use iced::core::widget::operation::accessible::Role;

let mut ui = simulator(my_view());

let button = ui.find(by_role(Role::Button));
assert!(button.is_ok(), "button should appear in the tree");

let submit = ui.find(by_label("Submit"));
assert!(submit.is_ok(), "button should have the label 'Submit'");
```

No screen reader or `a11y` feature needed -- the selectors query the
widget tree directly.

### Manual

Build and run any example with the `a11y` feature:

```sh
cargo run -p todos --features iced/a11y
```

Then verify what the screen reader sees:

**Linux:**
- **Orca** (`orca`) -- screen reader for GNOME/Linux. Start it before
  the app. Tab through your widget and listen for the role, label, and
  state announcements.
- **Python AT-SPI bindings** (`python-gobject` with
  `gi.repository.Atspi`) -- dump the accessibility tree to see your
  widget's node, role, name, and children without needing audio.

**Windows:**
- **NVDA** (free, nvaccess.org) -- Tab to your widget and listen. Press
  Insert+F7 to open the element list and find your widget by role.
- **Inspect.exe** (Windows SDK) -- browse the UI Automation tree
  visually.

**macOS:**
- **VoiceOver** (built-in, Cmd+F5) -- navigate to your widget with
  VO+Arrow keys.
- **Accessibility Inspector** (Xcode) -- inspect your widget's
  properties in the NSAccessibility tree.
