# Accessibility Module Placement Analysis

## Decision: iced_core vs iced_runtime

This document analyzes where to place the AccessKit accessibility integration in the iced codebase.

---

## Executive Summary

**RECOMMENDATION: Place accessibility in `iced_runtime`**

**Rationale**: 
- AccessKit requires access to `UserInterface` and the widget tree traversal system
- Needs window handles (available through runtime layer)
- Requires integration with event loop and action system
- Platform adapters need runtime context

**Split Approach**: 
- Core types and traits in `iced_core` (minimal, platform-agnostic)
- Implementation and adapter integration in `iced_runtime`

---

## Architecture Analysis

### Current Crate Structure

```
iced_core (82 files)
├── Foundation types (Widget, Element, Layout, Event, etc.)
├── Platform-agnostic primitives
├── widget::operation trait system
└── No dependencies on runtime or platform

iced_runtime (12 files)
├── UserInterface (builds, updates, draws widget trees)
├── Action system (Widget, Clipboard, Window actions)
├── Task system
├── window module
└── Depends on: iced_core, iced_futures, raw-window-handle

iced_program
├── Depends on: iced_runtime, iced_graphics
└── Higher-level program abstractions

iced_winit
├── Depends on: iced_program (which includes runtime)
├── Event loop integration
├── Window management (WindowManager)
└── Platform-specific window handles
```

### Dependency Flow
```
iced_core (foundation)
    ↓
iced_runtime (widget tree lifecycle)
    ↓
iced_program (program abstractions)
    ↓
iced_winit (platform integration)
```

---

## Option 1: Place in `iced_core`

### ✅ Advantages
1. **Platform-agnostic** - Core types available everywhere
2. **Minimal dependencies** - Doesn't pull in runtime overhead
3. **Widget trait proximity** - Close to Widget and Operation traits
4. **Conceptual fit** - Accessibility is "foundational" to widgets
5. **Easier testing** - Can test without runtime infrastructure

### ❌ Disadvantages
1. **No UserInterface access** - Can't integrate with widget tree lifecycle
2. **No window context** - AccessKit adapters need window handles
3. **No Action system** - Can't emit accessibility actions naturally
4. **Circular dependency risk** - If runtime needs accessibility, creates issues
5. **No event loop** - Can't handle AccessKit events (ActionRequested, etc.)
6. **Limited tree traversal** - widget::operation exists but no tree building

### 🚫 Critical Blockers
- **AccessKit adapters require window handles** (`raw_window_handle::HasRawWindowHandle`)
- **Tree building needs UserInterface::build() integration point**
- **Action routing needs runtime::Action system**
- **Event handling needs event loop access**

---

## Option 2: Place in `iced_runtime`

### ✅ Advantages
1. **UserInterface integration** - Direct access to build/update/draw/operate lifecycle
2. **Action system** - Natural place for `Action::Accessibility` variant
3. **Window context** - Already has `raw-window-handle` dependency
4. **Event loop proximity** - Can integrate with iced_winit event loop
5. **Tree traversal** - Can walk widget tree during UserInterface operations
6. **Task integration** - Can emit accessibility updates as Tasks
7. **Established pattern** - Already handles Widget, Clipboard, Window, System actions

### ❌ Disadvantages
1. **Runtime dependency** - All apps get accessibility even if minimal
2. **Not in "foundation"** - Conceptually less "core" than widget traits
3. **Less visible** - Widget authors work more with core than runtime
4. **Harder to test** - Requires more infrastructure

### ✅ Mitigations
1. **Always-on decision** - Already decided accessibility is always enabled
2. **Minimal overhead** - Lazy initialization, only build trees when needed
3. **Documentation** - Guide widget authors to runtime docs for accessibility
4. **Test harness** - Build testing utilities in runtime for widget authors

---

## Option 3: Split Approach (RECOMMENDED)

### Strategy
Place **traits and core types** in `iced_core`, **implementation** in `iced_runtime`

### In `iced_core`:
```rust
// iced_core/src/accessibility/mod.rs
pub mod accessibility {
    pub use accesskit::{NodeId, Role, Action};
    
    /// Trait for widgets to provide accessibility information
    pub trait Accessible {
        fn accessibility_role(&self) -> Role {
            Role::Unknown  // Safe default
        }
        
        fn accessibility_label(&self) -> Option<&str> {
            None  // Safe default
        }
        
        // etc - minimal, platform-agnostic interface
    }
}

// Add to widget::Operation trait:
impl Operation {
    fn accessible(
        &mut self,
        _id: Option<&Id>,
        _bounds: Rectangle,
        _accessible: &dyn Accessible,
    ) {
        // Default no-op
    }
}
```

### In `iced_runtime`:
```rust
// iced_runtime/src/accessibility/mod.rs
pub mod accessibility {
    use iced_core::accessibility::Accessible;
    
    pub struct AccessibilityTree { /* ... */ }
    pub struct TreeBuilder { /* ... */ }
    
    // Integration with UserInterface
    impl UserInterface {
        pub fn accessibility_tree(&self) -> AccessibilityTree {
            // Build tree from widget tree
        }
    }
}

// Add Action variant:
pub enum Action<T> {
    // ... existing variants ...
    Accessibility(accessibility::Action),
}
```

### In `iced_winit`:
```rust
// iced_winit/src/accessibility.rs
use accesskit_winit::Adapter;

pub struct AccessibilityAdapter {
    adapter: Option<Adapter>,
    // Platform-specific integration
}

// Integrate into WindowManager and event loop
```

### ✅ Advantages of Split
1. **Core traits in foundation** - Widget authors see accessibility in core
2. **Implementation where needed** - Runtime has UserInterface integration
3. **Clear separation** - Platform-agnostic vs platform-specific
4. **No circular dependencies** - Core → Runtime → Winit flow preserved
5. **Minimal core footprint** - Just traits and types, no AccessKit dependency in core
6. **Widget authors** - Implement `Accessible` trait from core
7. **App developers** - Get automatic accessibility from runtime

---

## Comparison with egui

### egui's Approach
- **egui crate**: Has `accesskit` as **optional** dependency
- **egui-winit crate**: Has `accesskit_winit` as **optional** dependency
- **Pattern**: Core logic in main crate, platform adapter in winit crate

### Key Difference for iced
- **egui** is a single immediate-mode rendering crate
- **iced** has separated concerns: core → runtime → winit
- **iced's advantage**: Can place implementation in runtime layer

---

## AccessKit Integration Requirements

### What AccessKit Needs:
1. ✅ **Tree structure** - Available in UserInterface (runtime)
2. ✅ **Window handles** - Available via raw-window-handle (runtime has it)
3. ✅ **Event handling** - Need event loop integration (winit layer)
4. ✅ **Action routing** - Need to route ActionRequests to widgets (runtime)
5. ✅ **Node IDs** - Need stable IDs across frames (runtime tree management)
6. ✅ **Screen coordinates** - Need layout bounds (available in UserInterface)

### Where These Exist:
- **UserInterface**: iced_runtime ✅
- **Window handles**: iced_runtime (has raw-window-handle) ✅
- **Event loop**: iced_winit ✅
- **Widget tree**: iced_runtime (UserInterface) ✅

---

## Final Recommendation

### Primary Placement: **iced_runtime**

**Module Structure:**
```
iced_core/src/
├── accessibility/        (NEW - minimal traits/types)
│   ├── mod.rs           (re-export accesskit types)
│   └── accessible.rs    (Accessible trait with defaults)
└── widget/
    └── operation.rs     (add accessible() method)

iced_runtime/src/
├── accessibility/        (NEW - implementation)
│   ├── mod.rs
│   ├── tree.rs          (AccessibilityTree builder)
│   ├── id.rs            (NodeId allocation/caching)
│   └── action.rs        (Action routing)
├── user_interface.rs     (MODIFY - add accessibility_tree())
└── lib.rs               (MODIFY - add Action::Accessibility)

iced_winit/src/
├── accessibility.rs      (NEW - Adapter integration)
├── window.rs            (MODIFY - add adapter to Window)
└── lib.rs               (MODIFY - handle accessibility events)
```

### Dependencies to Add:
```toml
# iced_core/Cargo.toml
[dependencies]
# Don't add accesskit - keep core lightweight

# iced_runtime/Cargo.toml
[dependencies]
accesskit = "0.16"  # Core AccessKit types

# iced_winit/Cargo.toml
[dependencies]
accesskit_winit = "0.22"  # Platform adapters
```

### Rationale:
1. **Core stays minimal** - Just trait definitions, no heavy dependencies
2. **Runtime has infrastructure** - UserInterface, Action system, window context
3. **Winit has platform access** - Event loop, window handles, adapters
4. **Natural layering** - Core (what) → Runtime (how) → Winit (platform)
5. **Widget authors** - See accessibility in core where they work
6. **App developers** - Get it automatically from runtime
7. **No breaking changes** - Default implementations mean existing code works

---

## Implementation Strategy

### Phase 1: iced_core (Minimal)
1. Add `accessibility` module with re-exported AccessKit types
2. Define `Accessible` trait with default implementations
3. Add `accessible()` to widget::Operation
4. Update existing widgets with default impl (no-op initially)

### Phase 2: iced_runtime (Core Logic)
1. Add AccessKit dependency
2. Implement tree building from UserInterface
3. Add ID caching and stability logic
4. Add Action::Accessibility variant
5. Integrate with UserInterface lifecycle

### Phase 3: iced_winit (Platform Integration)
1. Add accesskit_winit dependency
2. Implement Adapter per window
3. Hook into event loop
4. Route accessibility events to runtime

### Phase 4: Widget Implementations
1. Implement Accessible for Button, Text, TextInput, etc.
2. Add rich accessibility metadata
3. Test with screen readers

---

## Risk Mitigation

### Risk: Runtime becomes too heavy
- **Mitigation**: Lazy initialization, only build trees when accessibility active
- **Measurement**: Profile overhead, ensure < 1ms when no screen reader

### Risk: Widget authors don't find accessibility APIs
- **Mitigation**: Clear documentation, examples, migration guide
- **Measurement**: Survey widget author experience

### Risk: Circular dependencies
- **Mitigation**: Keep core → runtime → winit unidirectional
- **Verification**: `cargo tree` checks in CI

---

## Conclusion

**Place accessibility implementation in `iced_runtime` with minimal traits in `iced_core`.**

This provides:
- ✅ Access to UserInterface and widget tree
- ✅ Integration with Action and Task systems
- ✅ Window context for platform adapters
- ✅ Clear separation of concerns
- ✅ No breaking changes for existing code
- ✅ Natural fit with iced's architecture

The split approach gives us the best of both worlds: visibility in core, implementation where infrastructure exists.
