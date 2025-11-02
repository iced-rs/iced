# Adding Accessibility Methods to Widget Trait

## Analysis: What if accessibility was part of the Widget trait?

This document explores what it would look like to add accessibility methods directly to the `Widget` trait instead of using a separate `Accessible` trait.

---

## Current Widget Trait Structure

The `Widget` trait currently has **13 methods**:

```rust
pub trait Widget<Message, Theme, Renderer>
where
    Renderer: crate::Renderer,
{
    // Required methods (2)
    fn size(&self) -> Size<Length>;
    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node;
    fn draw(&self, tree: &Tree, renderer: &mut Renderer, theme: &Theme, style: &renderer::Style, 
            layout: Layout<'_>, cursor: mouse::Cursor, viewport: &Rectangle);
    
    // Optional methods with defaults (10)
    fn size_hint(&self) -> Size<Length> { self.size() }
    fn tag(&self) -> tree::Tag { tree::Tag::stateless() }
    fn state(&self) -> tree::State { tree::State::None }
    fn children(&self) -> Vec<Tree> { Vec::new() }
    fn diff(&self, tree: &mut Tree) { tree.children.clear(); }
    fn operate(&mut self, ...) { }
    fn update(&mut self, ...) { }
    fn mouse_interaction(&self, ...) -> mouse::Interaction { mouse::Interaction::None }
    fn overlay<'a>(&'a mut self, ...) -> Option<overlay::Element<'a, ...>> { None }
}
```

**Key Pattern**: Most methods have default no-op implementations, allowing widgets to only implement what they need.

---

## Option A: Minimal Accessibility Methods (RECOMMENDED)

### Add to Widget Trait:

```rust
pub trait Widget<Message, Theme, Renderer>
where
    Renderer: crate::Renderer,
{
    // ... existing methods ...
    
    /// Builds accessibility information for this widget.
    ///
    /// By default, returns a generic node with `Role::Unknown`.
    /// Widgets should override this to provide meaningful accessibility.
    fn accessibility(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
    ) -> AccessibilityNode {
        AccessibilityNode::new(layout.bounds())
    }
}
```

### Simple Accessibility Node Type:

```rust
// In iced_core/src/accessibility/mod.rs
pub struct AccessibilityNode {
    bounds: Rectangle,
    role: Option<Role>,
    label: Option<String>,
    value: Option<String>,
    enabled: bool,
    focusable: bool,
    children: Vec<AccessibilityNode>,
}

impl AccessibilityNode {
    pub fn new(bounds: Rectangle) -> Self {
        Self {
            bounds,
            role: None,
            label: None,
            value: None,
            enabled: true,
            focusable: false,
            children: Vec::new(),
        }
    }
    
    pub fn role(mut self, role: Role) -> Self {
        self.role = Some(role);
        self
    }
    
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
    
    // Builder pattern for easy construction
}
```

### Example Implementation (Button):

```rust
impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Button<'_, Message, Theme, Renderer>
where
    Renderer: crate::core::Renderer,
    Theme: Catalog,
{
    // ... existing implementations ...
    
    fn accessibility(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
    ) -> AccessibilityNode {
        AccessibilityNode::new(layout.bounds())
            .role(Role::Button)
            .label(self.extract_text_label())  // Helper method
            .enabled(self.on_press.is_some())
            .focusable(true)
    }
}
```

### ✅ Advantages:
1. **Single source of truth** - All widget info in one trait
2. **Discoverable** - Widget authors see accessibility when implementing Widget
3. **Consistent pattern** - Follows existing Widget trait style (layout, draw, etc.)
4. **Access to internal state** - Widget can use `self` to build accurate info
5. **No separate trait** - Simpler conceptual model
6. **Builder pattern** - Easy to construct nodes with fluent API

### ❌ Disadvantages:
1. **Adds to Widget trait** - Makes trait slightly larger (but still small)
2. **Called every frame** - Like draw(), needs to be efficient
3. **Not optional** - Every widget has the method (but default is safe)

---

## Option B: Multiple Accessibility Methods

### More granular approach:

```rust
pub trait Widget<Message, Theme, Renderer>
where
    Renderer: crate::Renderer,
{
    // ... existing methods ...
    
    /// Returns the accessibility role of this widget.
    fn accessibility_role(&self) -> Option<Role> {
        None
    }
    
    /// Returns the accessibility label for this widget.
    fn accessibility_label(&self, tree: &Tree, layout: Layout<'_>) -> Option<String> {
        None
    }
    
    /// Returns the accessibility value for this widget.
    fn accessibility_value(&self, tree: &Tree) -> Option<String> {
        None
    }
    
    /// Returns whether this widget is focusable.
    fn accessibility_focusable(&self) -> bool {
        false
    }
    
    /// Returns child accessibility nodes.
    fn accessibility_children(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
    ) -> Vec<AccessibilityNode> {
        Vec::new()
    }
}
```

### ✅ Advantages:
1. **Granular control** - Override only what you need
2. **Clear semantics** - Each method has single purpose
3. **Simple return types** - Just Option<Role>, Option<String>, etc.

### ❌ Disadvantages:
1. **More methods** - Widget trait gets bigger (5 new methods)
2. **Multiple calls** - Runtime has to call 5 methods per widget
3. **Less flexible** - Hard to add new properties later
4. **Boilerplate** - Widget authors override multiple methods
5. **Coordination** - Multiple methods need to stay in sync

---

## Option C: Single Method + Builder (BEST BALANCE)

### Hybrid approach:

```rust
pub trait Widget<Message, Theme, Renderer>
where
    Renderer: crate::Renderer,
{
    // ... existing methods ...
    
    /// Builds accessibility information for this widget.
    ///
    /// The default implementation returns `None`, making the widget invisible
    /// to accessibility tools. Widgets should override this to provide
    /// meaningful accessibility.
    ///
    /// # Example
    /// ```
    /// fn accessibility(&self, tree: &Tree, layout: Layout<'_>) -> Option<AccessibilityNode> {
    ///     Some(
    ///         AccessibilityNode::new(layout.bounds())
    ///             .role(Role::Button)
    ///             .label("Click me")
    ///             .enabled(self.on_press.is_some())
    ///     )
    /// }
    /// ```
    fn accessibility(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
    ) -> Option<AccessibilityNode> {
        None  // Default: invisible to accessibility
    }
}
```

### Why `Option<AccessibilityNode>` instead of just `AccessibilityNode`?

1. **Explicit opt-in** - Returning `None` means "I haven't implemented this yet"
2. **Container widgets** - Can return `None` and rely on children
3. **Decorative widgets** - Some widgets are truly not accessible (pure visuals)
4. **Graceful degradation** - Missing impl = invisible, not broken

### Example Implementations:

#### Button (Interactive):
```rust
fn accessibility(&self, tree: &Tree, layout: Layout<'_>) -> Option<AccessibilityNode> {
    Some(
        AccessibilityNode::new(layout.bounds())
            .role(Role::Button)
            .label(self.extract_label())
            .enabled(self.on_press.is_some())
            .focusable(true)
    )
}
```

#### Text (Static):
```rust
fn accessibility(&self, _tree: &Tree, layout: Layout<'_>) -> Option<AccessibilityNode> {
    Some(
        AccessibilityNode::new(layout.bounds())
            .role(Role::StaticText)
            .label(&self.content)
    )
}
```

#### Container (Transparent):
```rust
fn accessibility(&self, _tree: &Tree, _layout: Layout<'_>) -> Option<AccessibilityNode> {
    None  // Children will be accessible directly
}
```

#### TextInput (Complex):
```rust
fn accessibility(&self, tree: &Tree, layout: Layout<'_>) -> Option<AccessibilityNode> {
    let state = tree.state.downcast_ref::<State>();
    
    Some(
        AccessibilityNode::new(layout.bounds())
            .role(Role::TextInput)
            .label(self.placeholder.as_ref())
            .value(&state.value)
            .enabled(true)
            .focusable(true)
            .text_selection(state.cursor.selection())
    )
}
```

---

## Integration with Runtime

### In UserInterface (iced_runtime):

```rust
impl UserInterface {
    pub fn accessibility_tree(&self) -> AccessibilityTree {
        let mut builder = AccessibilityTreeBuilder::new();
        
        // Traverse the widget tree
        self.build_accessibility_node(
            self.root.as_widget(),
            &self.state,
            Layout::new(&self.base),
            &mut builder,
        );
        
        builder.build()
    }
    
    fn build_accessibility_node(
        &self,
        widget: &dyn Widget<Message, Theme, Renderer>,
        tree: &Tree,
        layout: Layout<'_>,
        builder: &mut AccessibilityTreeBuilder,
    ) {
        if let Some(node) = widget.accessibility(tree, layout) {
            builder.add_node(node);
        }
        
        // Recurse into children
        for (child_widget, child_tree, child_layout) in ... {
            self.build_accessibility_node(child_widget, child_tree, child_layout, builder);
        }
    }
}
```

### Tree Building:
1. Runtime calls `widget.accessibility(tree, layout)` during tree traversal
2. Widget returns `Some(node)` or `None`
3. Runtime builds AccessKit tree from collected nodes
4. Overlay widgets also contribute nodes

---

## Comparison: Separate Trait vs Widget Method

### Separate `Accessible` Trait:
```rust
pub trait Accessible {
    fn accessibility(&self, tree: &Tree, layout: Layout<'_>) -> Option<AccessibilityNode> {
        None
    }
}

// All widgets need:
impl Accessible for Button { ... }
impl<M, T, R> Widget<M, T, R> for Button { ... }  // Existing
```

**Issues:**
- Two trait impls per widget
- Less discoverable (widget authors might miss it)
- Runtime needs to downcast to Accessible

### In Widget Trait:
```rust
pub trait Widget<M, T, R> {
    // ... existing methods ...
    fn accessibility(&self, tree: &Tree, layout: Layout<'_>) -> Option<AccessibilityNode> {
        None
    }
}

// All widgets need:
impl<M, T, R> Widget<M, T, R> for Button {
    // ... existing methods ...
    fn accessibility(&self, tree: &Tree, layout: Layout<'_>) -> Option<AccessibilityNode> {
        Some(...)
    }
}
```

**Benefits:**
- Single trait impl
- More discoverable
- No downcasting needed
- Natural part of widget lifecycle

---

## Impact Analysis

### On Existing Widgets (in iced_widget):
```rust
// Before (no accessibility):
impl Widget for Button {
    fn layout(...) { ... }
    fn draw(...) { ... }
    // ~10 methods
}

// After (with default):
impl Widget for Button {
    fn layout(...) { ... }
    fn draw(...) { ... }
    fn accessibility(...) { None }  // Default, or override
    // ~11 methods
}
```

**Change needed**: ✅ ZERO - default implementation covers it

**To add accessibility**: Override one method

### On Custom Widgets (user code):
```rust
// User's existing custom widget
struct MyWidget;

impl Widget<(), Theme, Renderer> for MyWidget {
    fn size(&self) -> Size<Length> { ... }
    fn layout(...) -> layout::Node { ... }
    fn draw(...) { ... }
}

// COMPILES WITHOUT CHANGES - default accessibility() is used
```

**Impact**: ✅ ZERO breaking changes for users

### Adding Accessibility to Widget:
```rust
impl Widget<(), Theme, Renderer> for MyWidget {
    // ... existing methods ...
    
    fn accessibility(&self, tree: &Tree, layout: Layout<'_>) -> Option<AccessibilityNode> {
        Some(
            AccessibilityNode::new(layout.bounds())
                .role(Role::Generic)
                .label("My custom widget")
        )
    }
}
```

**Effort**: One method override

---

## Performance Considerations

### Call Frequency:
- **layout()**: Called on resize, rebuild
- **draw()**: Called every frame
- **accessibility()**: Called when accessibility tree needs update

### Optimization Strategy:
```rust
impl UserInterface {
    // Cache the accessibility tree
    accessibility_cache: Option<(u64, AccessibilityTree)>,
    
    pub fn accessibility_tree(&mut self) -> &AccessibilityTree {
        let current_version = self.state_version();
        
        if let Some((version, tree)) = &self.accessibility_cache {
            if *version == current_version {
                return tree;  // Return cached
            }
        }
        
        // Rebuild only when widget tree changed
        let tree = self.build_accessibility_tree();
        self.accessibility_cache = Some((current_version, tree));
        tree
    }
}
```

**Performance**: Only rebuild when widget tree changes, not every frame

---

## Documentation Strategy

### Widget Trait Documentation:
```rust
/// Builds accessibility information for this widget.
///
/// This method is called by the accessibility system to build a tree
/// that assistive technologies (like screen readers) can use to understand
/// and interact with your widget.
///
/// # Default Behavior
/// The default implementation returns `None`, making the widget invisible
/// to accessibility tools. This is appropriate for purely decorative widgets.
///
/// # When to Override
/// Override this method if your widget:
/// - Can be interacted with (buttons, inputs, sliders)
/// - Displays meaningful content (text, images with alt text)
/// - Has state that should be announced (checkboxes, toggles)
///
/// # Examples
/// ```
/// fn accessibility(&self, tree: &Tree, layout: Layout<'_>) -> Option<AccessibilityNode> {
///     Some(
///         AccessibilityNode::new(layout.bounds())
///             .role(Role::Button)
///             .label("Click me")
///             .enabled(self.on_press.is_some())
///             .focusable(true)
///     )
/// }
/// ```
///
/// # Container Widgets
/// Container widgets can either return `None` (to be transparent) or return
/// a node with children. Children are automatically traversed.
fn accessibility(&self, tree: &Tree, layout: Layout<'_>) -> Option<AccessibilityNode> {
    None
}
```

### Migration Guide:
```markdown
# Adding Accessibility to Custom Widgets

If you have custom widgets, they will continue to work without changes.
To make them accessible:

1. Override the `accessibility()` method
2. Return `Some(AccessibilityNode::new(...))` with your widget's info
3. Test with a screen reader

See the examples/ directory for complete examples.
```

---

## Recommendation: Add to Widget Trait

### Final Design:

```rust
// iced_core/src/widget.rs
pub trait Widget<Message, Theme, Renderer>
where
    Renderer: crate::Renderer,
{
    // ... existing 10 methods ...
    
    /// Builds accessibility information for this widget.
    fn accessibility(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
    ) -> Option<AccessibilityNode> {
        None
    }
}
```

```rust
// iced_core/src/accessibility.rs
pub struct AccessibilityNode {
    bounds: Rectangle,
    role: Option<Role>,
    label: Option<String>,
    value: Option<String>,
    enabled: bool,
    focusable: bool,
    // ... other properties
}

impl AccessibilityNode {
    pub fn new(bounds: Rectangle) -> Self { ... }
    pub fn role(mut self, role: Role) -> Self { ... }
    pub fn label(mut self, label: impl Into<String>) -> Self { ... }
    // Builder methods...
}
```

### Why This Works:

1. ✅ **No breaking changes** - Default returns `None`
2. ✅ **Discoverable** - Widget authors see it in the trait
3. ✅ **Simple API** - One method, builder pattern for node
4. ✅ **Flexible** - Can return `None` for transparent widgets
5. ✅ **Access to state** - Widget has access to `self` and `tree`
6. ✅ **Efficient** - Called only when tree needs rebuild
7. ✅ **Testable** - Easy to test node construction
8. ✅ **Familiar pattern** - Like `layout()` and `draw()`

### Matches User Requirements:

1. ✅ **Widget authors bear burden** - They override the method
2. ✅ **App developers get it free** - Built-in widgets have implementations
3. ✅ **No runtime errors** - Missing impl = `None` return
4. ✅ **Always on** - Part of Widget trait, no feature flags
5. ✅ **Graceful degradation** - `None` means invisible, not broken

---

## Next Steps

1. **Prototype** `AccessibilityNode` type in iced_core
2. **Add method** to Widget trait with default `None`
3. **Implement** for Button, Text, TextInput
4. **Test** that existing code compiles without changes
5. **Build** tree collection in iced_runtime
6. **Integrate** with AccessKit in iced_winit

This approach provides the cleanest integration with iced's existing architecture while meeting all the stated requirements.
