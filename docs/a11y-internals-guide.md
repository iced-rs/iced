# Accessibility Internals Guide

How the accessibility infrastructure works, how to extend it, and how to
verify it. For widget-level usage (choosing roles, setting labels), see
[a11y-widget-guide.md](a11y-widget-guide.md).

## Background

Accessibility in iced means exposing the widget tree to **assistive
technology** (AT) -- software that helps people with disabilities use
computers. The most common AT is a **screen reader** (NVDA on Windows,
VoiceOver on macOS, Orca on Linux), which reads the UI aloud.

Screen readers don't look at pixels. They read an **accessibility tree**
-- a parallel data structure that describes every widget's role ("button"),
name ("Submit"), and state ("disabled"). The tree is maintained by the
application and consumed by the platform's accessibility API:

| Platform | API | Screen readers |
|----------|-----|---------------|
| Linux | AT-SPI2 (D-Bus protocol) | Orca |
| Windows | UI Automation | NVDA, JAWS, Narrator |
| macOS | NSAccessibility | VoiceOver |

**AccessKit** is the Rust library that bridges our tree to all three
platforms. It takes a `TreeUpdate` (a list of nodes with roles,
properties, and parent-child relationships) and translates it into
platform-specific API calls.

## Architecture

Accessibility spans three crates:

**`iced_core`** (`core/src/widget/operation/accessible.rs`)

Platform-agnostic types that widgets interact with: `Role`, `Accessible`,
`Value`, `Live`, `Orientation`. No dependency on accesskit. This is the
public API surface -- changes here affect all downstream crates.

The `Operation` trait has an `accessible()` method that widgets call to
expose their metadata. This method is part of the same visitor pattern
used by `focusable()`, `text_input()`, and `scrollable()`.

**`iced_winit`** (`winit/src/a11y.rs`)

The platform bridge. Contains:

- `A11yAdapter` -- per-window wrapper around `accesskit_winit::Adapter`.
  Created before the window is visible (accesskit requirement). Manages
  the connection to the platform AT layer.

- `TreeBuilder` -- an `impl Operation` that walks the widget tree and
  produces an `accesskit::TreeUpdate`. This is where iced's
  platform-agnostic types are converted to accesskit types.

- `convert_role()` -- maps `iced_core::Role` variants to
  `accesskit::Role` variants.

- Synthetic event helpers -- translates AT actions (click, focus,
  increment) into iced mouse/keyboard events.

This is the only crate that depends on accesskit.

**`iced_widget`** (`widget/src/*.rs`)

Each widget calls `operation.accessible()` in its `operate()` method.
Widgets never import accesskit directly -- they only see the
platform-agnostic types from `iced_core`.

## How the tree gets built

When the tree is dirty and AT is connected:

1. The event loop creates a `TreeBuilder` (an `impl Operation`).
2. `ui.operate(&renderer, &mut builder)` walks the widget tree.
3. Each widget's `operate()` calls methods on the builder:
   - `accessible()` -- creates a node with a role and properties
   - `container()` -- creates a structural grouping node
   - `traverse()` -- descends into children
   - `text()` -- creates a Label node for text content
   - `focusable()` -- records focus state on the current node
   - `scrollable()` -- records scroll position and creates ScrollBar nodes
4. `builder.build()` resolves cross-node references (`labelled_by`,
   `described_by`), assigns children to parents, and produces the final
   `TreeUpdate`.
5. The adapter pushes the update to the platform.

When no AT is connected, `adapter.is_active()` returns false and none
of this runs. The cost is one atomic load per window per frame.

## Node IDs

AccessKit nodes need stable IDs across tree rebuilds. If IDs change every
frame, the screen reader thinks every element was destroyed and recreated.

The tree builder uses hash-based IDs:
- Widgets with a `widget::Id` get a NodeId by hashing the widget ID
  (stable across rebuilds).
- Anonymous widgets get a NodeId by hashing `(parent_id, sibling_index)`
  (stable as long as sibling order doesn't change).
- A rehash loop handles hash collisions.
- NodeId(0) is reserved for the root Window node.

## Role conversion

`convert_role()` maps iced's platform-agnostic `Role` enum to accesskit's
`Role` enum. Most variants map 1:1. Exceptions are documented with
comments in the match arms (e.g., `Separator` maps to `GenericContainer`
because accesskit has no non-interactive separator role).

The `Role` enum is `#[non_exhaustive]`. New variants are handled by the
`_ => Role::Unknown` catch-all -- the widget appears in the tree but the
screen reader has no semantic information about it.

To add a new role:
1. Add the variant to `Role` in `core/.../accessible.rs` (alphabetically)
2. Add the mapping in `convert_role()` in `winit/src/a11y.rs`
3. Add a test verifying the mapping

## Property mapping

`TreeBuilder::accessible()` converts each field of the `Accessible`
struct to the corresponding accesskit `Node` setter:

| Accessible field | AccessKit setter | Notes |
|-----------------|-----------------|-------|
| `label` | `set_label()` | Also sets `set_placeholder()` for text input roles |
| `description` | `set_description()` | |
| `disabled` | `set_disabled()` | |
| `toggled` | `set_toggled()` | `Toggled::True` or `Toggled::False` |
| `selected` | `set_selected()` | |
| `expanded` | `set_expanded()` | Also adds Expand/Collapse actions |
| `live` | `set_live()` | `Live::Polite` or `Live::Assertive` |
| `required` | `set_required()` | |
| `level` | `set_level()` | |
| `orientation` | `set_orientation()` | `Horizontal` or `Vertical` |
| `value` (Text) | `set_value()` | |
| `value` (Numeric) | `set_numeric_value()` + min/max/step | Also adds Increment/Decrement actions if step is set |
| `labelled_by` | resolved in `build()` | Maps widget::Id to NodeId |
| `described_by` | resolved in `build()` | Maps widget::Id to NodeId |

The tree builder also infers actions from the role:
- Button, CheckBox, RadioButton, Switch, Link, MenuItem, Tab get
  `Action::Click`
- ComboBox gets `HasPopup::Listbox`
- `focusable()` adds `Action::Focus`

To add a new property:
1. Add the field to `Accessible` in `core/.../accessible.rs`
2. Map it in `TreeBuilder::accessible()` in `winit/src/a11y.rs`
3. Add a test verifying the property reaches the accesskit node

## Text content and Label nodes

The `text()` method on the tree builder creates `Role::Label` nodes with
`set_value()` (not `set_label()` -- accesskit reads Label content from
the `value` property).

For roles that support it (Button, CheckBox, RadioButton, Link, MenuItem),
accesskit automatically derives the accessible name from descendant Label
nodes. GenericContainers between the parent and the Label are
transparently skipped.

When a widget sets `Accessible.label` AND calls `text()` with the same
string, the tree builder detects the match and skips creating the Label
node to avoid duplicate announcements.

## GenericContainer and tree filtering

Layout containers (Row, Column, Container) create nodes with
`Role::GenericContainer`. AccessKit filters these out of the platform
tree (`ExcludeNode`), promoting their children to the nearest visible
ancestor. This keeps the screen reader's view clean -- users navigate
semantic elements, not layout wrappers.

`Role::Group` is the visible alternative. Use it for semantically
meaningful groupings (radio button groups, form sections).

## AT action routing

When a screen reader user activates a widget (e.g., presses Enter on a
focused button), accesskit sends an `ActionRequest`. The event loop
translates these into synthetic iced events:

| AT action | Synthetic iced event |
|-----------|---------------------|
| Focus | `operation::focusable::focus(widget_id)` or synthetic CursorMoved |
| Click | CursorMoved + ButtonPressed + ButtonReleased |
| Increment | CursorMoved + ArrowUp KeyPressed + KeyReleased |
| Decrement | CursorMoved + ArrowDown KeyPressed + KeyReleased |

This means widgets handle AT actions through their existing mouse/keyboard
event handlers -- no a11y-specific event handling needed.

## Announcements

`runtime::announce(text)` creates an `Action::Announce` that queues text
for the next tree rebuild. The tree builder creates an assertive
live-region Label node from it. The text is cleared after at least one
active adapter has consumed it.

## Testing

### Automated tests

**Unit tests** (in `winit/src/a11y.rs`): Construct a `TreeBuilder`,
call `accessible()`, `text()`, `container()`, `traverse()` directly,
then inspect the `A11yTree`. Tests the tree building logic without a
real window or AT connection.

**Selector tests** (`iced_test` / `iced_selector`): Use `by_role()` and
`by_label()` to query the widget tree as an AT would. Tests that widgets
call `operation.accessible()` with the right metadata. Work without the
`a11y` feature.

### Manual testing with screen readers

Automated tests verify tree structure. Manual testing verifies the
end-to-end experience -- what the user actually hears.

**Linux:**
- **Orca** (`orca`) -- primary screen reader for GNOME/Linux. Requires
  AT-SPI2 and the `ScreenReaderEnabled` D-Bus property to be true.
- **Python AT-SPI bindings** (`python-gobject` with
  `gi.repository.Atspi`) -- scripted tree inspection without a GUI.
- **Accerciser** (`accerciser`) -- GUI tree inspector. May hang on
  some Wayland compositors.

**Windows:**
- **NVDA** (free, nvaccess.org) -- has browse mode (virtual buffer) and
  focus mode. Test both.
- **Inspect.exe** (Windows SDK) -- shows the UI Automation tree.

**macOS:**
- **VoiceOver** (built-in, Cmd+F5) -- uses the rotor for categorical
  navigation.
- **Accessibility Inspector** (Xcode) -- shows the NSAccessibility tree.

## Further reading

**Standards and specifications:**
- [WCAG 2.1](https://www.w3.org/TR/WCAG21/) -- Web Content Accessibility
  Guidelines. The principles (Perceivable, Operable, Understandable,
  Robust) apply to all UIs, not just web. Success criteria 2.1.1
  (Keyboard), 4.1.2 (Name, Role, Value), and 4.1.3 (Status Messages) are
  directly relevant.
- [WAI-ARIA 1.2](https://www.w3.org/TR/wai-aria-1.2/) -- Roles, states,
  and properties. The role definitions map closely to accesskit's Role
  enum and iced's Accessible struct fields.
- [ARIA Authoring Practices Guide](https://www.w3.org/WAI/ARIA/apg/) --
  Concrete keyboard interaction patterns for every common widget type
  (button, checkbox, slider, combobox, tabs, dialog, etc.). The
  definitive reference for how keyboard navigation should work.

**AccessKit:**
- [accesskit crate docs](https://docs.rs/accesskit/latest/accesskit/) --
  The Node API, Role enum, and action system.
- [accesskit_winit crate docs](https://docs.rs/accesskit_winit/latest/accesskit_winit/) --
  The Adapter and platform integration.
- [AccessKit repository](https://github.com/AccessKit/accesskit) -- source
  code, examples (`platforms/winit/examples/simple.rs` is particularly
  useful), and the consumer crate's test suite for tree structure patterns.

**Screen readers:**
- [NVDA User Guide](https://www.nvaccess.org/files/nvda/documentation/userGuide.html) --
  browse mode, focus mode, and keyboard commands.
- [VoiceOver Getting Started](https://support.apple.com/guide/voiceover/welcome/mac) --
  rotor navigation, VO keys.
- [Orca documentation](https://help.gnome.org/users/orca/stable/) --
  AT-SPI navigation, flat review.

**Platform accessibility APIs:**
- [AT-SPI2 specification](https://www.freedesktop.org/wiki/Accessibility/AT-SPI2/) --
  the Linux accessibility protocol used by accesskit_unix.
- [UI Automation overview](https://learn.microsoft.com/en-us/windows/win32/winauto/entry-uiauto-win32) --
  the Windows accessibility API used by accesskit_windows.
- [NSAccessibility](https://developer.apple.com/documentation/appkit/nsaccessibility) --
  the macOS accessibility API used by accesskit_macos.

