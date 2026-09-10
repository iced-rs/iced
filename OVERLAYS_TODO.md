# Overlays — Reintroduction TODO

Overlays were removed entirely so the design can start from scratch. This file lists
the functionality the new design must provide.

## Core capabilities

- [ ] Display interactive content on top of the rest of the user interface
- [ ] Layering: multiple overlays stack, with higher layers on top
- [ ] Nesting: an overlay can itself contain overlays
- [ ] Anchoring: overlays are positioned relative to their source widget and the
      viewport
- [ ] Correct positioning and input for content that is transformed (scaled or
      translated) or nested
- [ ] Input on overlays: hover, press, release, focus, and keyboard events
- [ ] The cursor shape reflects the interactive element under the pointer inside
      overlays (e.g. a text cursor over a text input, a pointer over a menu option)
- [ ] Operable/inspectable content: overlay content can be targeted by operations
      (focus, element and text selection, e.g. from end-to-end tests)
- [ ] Overlays producing a different message type than the application can be
      composed with it
- [ ] The overlay API is exposed through the public `iced` API

## Runtime behavior

- [ ] Overlays receive input before the widgets below them; input captured by an
      overlay does not reach the widgets below
- [ ] While the pointer is over an interactive overlay, the widgets below it do not
      change hover or focus state
- [ ] Layout changes while an overlay is open (window resize, content changes) keep
      the overlay correct, including it disappearing mid-interaction
- [ ] Overlays are rendered on top of the base user interface

## Widget functionality

### `tooltip`

- [ ] Show a hint over the wrapped content while it is hovered
- [ ] Placement: above, below, left, right, or following the cursor
- [ ] Configurable delay before the tooltip appears
- [ ] Configurable gap between the content and the tooltip, and tooltip padding
- [ ] The tooltip stays within the viewport
- [ ] Styling of the tooltip appearance
- (Currently: nothing is shown when hovering)

### `float`

- [ ] Display content scaled or translated out of its normal position, on top of the
      other content
- [ ] Floating content stays interactive: it receives input and reflects hover state
- [ ] The float's shadow styling applies while floating
- (Currently: floating content is neither shown nor interactive)

### `combo_box`

- [ ] A dropdown menu listing the filtered options, positioned above or below the
      input depending on the available space
- [ ] Options highlight on hover and can be selected with the mouse
- [ ] Menu height and menu appearance settings

### `pick_list`

- [ ] A dropdown menu of options when the list is opened
- [ ] Options can be selected with the mouse
- [ ] Menu height and menu appearance settings

### Dropdown menu

- [ ] A menu of selectable options displayed on top of the user interface (used by
      the combo box and the pick list)
- [ ] The menu flips between above and below its anchor based on the available
      viewport space
- [ ] Styling: background, border, text, hovered/selected option, shadow

### `scrollable`

- [ ] A visual indicator of the auto-scroll direction while auto-scrolling
- [ ] Appearance settings for the indicator
- (The auto-scroll behavior itself still works)

### `pane_grid`

- [ ] A pane being dragged follows the cursor while reordering panes
- (Currently: drag tracking and the drop-region highlight still work; the dragged
  pane itself is not drawn)

### `component`

- [ ] A component's view can display overlays
- [ ] The user interface re-lays-out when a component's overlay opens or closes

### `themer`

- [ ] Overlays respect the theme applied by a surrounding themer

### `hover()` helper

- [ ] The top layer stays visible while its own overlay is open
- (Currently: it is only shown while hovered or focused)

### `opaque()` helper

- [ ] Opaque content does not block overlays

### Tester

- [ ] Interaction recording and hover highlighting inside overlays
- [ ] End-to-end selectors can target overlay content (e.g. an open menu option)

## Example

- [ ] Toast notifications: auto-expiring messages with a configurable timeout,
      status-based styling, a close button, stacked over the user interface content
