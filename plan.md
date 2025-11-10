# Current Implementation Status (as of Nov 9, 2025)

## ✅ Major Milestone: Stable Node IDs Verified!

**Branch**: `accesskit-integration` (latest: TBD - needs commit)
**Status**: Hybrid ID system working, tested and verified stable across frames

### 🎉 Completed Nov 9, 2025: Node ID Stability Verification

**Critical Achievement**: Verified that the hybrid ID system provides stable NodeIDs automatically

Completed:
1. ✅ **Researched existing widget::Id system**
   - Widget IDs are optional (Button, TextInput, Container, Scrollable support them)
   - `Id::new("name")` creates stable IDs, `Id::unique()` does not
   - All Operation methods receive `id: Option<&Id>` parameter

2. ✅ **Verified hybrid ID approach is already working**
   - Priority 1: Use explicit widget::Id when provided (maximum stability)
   - Priority 2: Fall back to flat "window/TYPE[INDEX]" hashing
   - Counter example shows identical NodeIDs across multiple frames

3. ✅ **Testing confirmed stability**
   - Added debug output to track NodeID generation
   - Ran counter example multiple times
   - All NodeIDs identical between frames for static content
   - Example: `window/container[0]` → NodeId(185100343528096102) (stable!)

4. ✅ **Updated documentation**
   - Documented flat type+index approach in plan.md
   - Added verification results with concrete examples
   - Clarified when to use explicit widget IDs vs automatic fallback

**Key Insight**: The system already works perfectly for static layouts with ZERO developer burden.
For dynamic lists, developers should use explicit widget IDs (e.g., `.id(Id::new(format!("item-{}", key)))`).

---

## ✅ Previous Milestone: Leaf Node Semantics & Button Accessibility!

**Status**: Button leaf node implementation complete, screen reader tested

### 🎉 Completed Nov 9, 2025: Leaf Node Architecture & Button Widget

**Critical Achievement**: Proper accessibility tree semantics for composite widgets

Implemented:
1. ✅ **Leaf node concept** (core/src/accessibility/node.rs)
   - Added `is_leaf_node: bool` field to AccessibilityNode
   - Leaf nodes present as single accessible elements without traversable children
   - Semantic API: `.is_leaf_node(true)` clearly indicates terminal elements

2. ✅ **TreeBuilder leaf node handling** (runtime/src/accessibility.rs)
   - Added `inside_leaf_node: bool` state tracking
   - When inside a leaf node, child accessibility nodes are not added to tree
   - Proper reset after traversal ensures siblings are not affected

3. ✅ **Button widget as leaf node** (widget/src/button.rs)
   - Buttons set `.is_leaf_node(true)` - text children don't create separate nodes
   - Added `accessibility_label` field for explicit labeling
   - Added `.accessibility_label()` builder method
   - Falls back to "Button" if no label set

4. ✅ **Counter example updated** (examples/counter/src/main.rs)
   - Buttons labeled with `.accessibility_label("Increment")` and `.accessibility_label("Decrement")`
   - Screen reader testing confirmed: 3 focus stops (2 buttons + 1 text) instead of 5
   - Button children no longer appear as separate accessibility nodes

5. ✅ **API naming improvements**
   - Renamed `traverse_children` → `is_leaf_node` (more semantic)
   - Renamed `skip_children` → `inside_leaf_node` (clearer state tracking)
   - Better conveys the tree relationship: property declaration vs internal state

Status: **Buttons work correctly with screen readers! Text embeds in button label.**

### 🎉 Completed Nov 6, 2025: Stable NodeID System

**Critical Achievement**: Automatic stable NodeID generation with ZERO developer burden

Implemented:
1. ✅ **Path-based ID generation** (runtime/src/accessibility.rs)
   - TreeBuilder tracks widget position via `path_stack` and `type_counters`
   - Generates paths like `"window/button[0]"`, `"window/button[1]"`
   - Hashes path → stable u64 → AccessKit NodeId
   - Example: button[0] → NodeId 7447623757530889483 (stable across frames!)

2. ✅ **All Operation methods updated**
   - `accessibility()` - maps Role to widget type for path generation
   - `container()`, `focusable()`, `text()`, `text_input()`, `scrollable()` - all use stable IDs
   - Removed old counter-based `next_id()` system

3. ✅ **Verification completed**
   - Debug output confirmed ID stability across multiple frame updates
   - Counter example running with stable IDs
   - No breaking changes to existing code

### ✅ Completed Nov 2, 2025: Initial Infrastructure

Commits:
- 9cccd4688: Initial foundation (AccessibilityNode, Widget trait, Button)
- 0e5ef0162: Tree collection infrastructure  
- 32eb227d1: Documentation updates (Operation pattern decision)
- 9b1beb2a5: Rewrite using Operation pattern
- 38e9f05dd: UserInterface integration complete

Implemented:
1. AccessibilityNode wrapper type (iced_core)
2. Widget::accessibility() method with default None (non-breaking)
3. Button widget implementation with Click + Focus actions
4. Text widget implementation (Label role)
5. TreeBuilder implementing Operation trait (iced_runtime)
6. UserInterface::accessibility() method returning TreeUpdate + bounds mapping
7. Window struct with accessibility adapter (iced_winit)
8. ActivationHandler and ActionHandler implementations
9. Event loop integration - tree updates after UI rebuilds
10. Action routing - synthesizes mouse clicks from accessibility actions

Status: **Fully functional end-to-end accessibility pipeline!**

🔍 Architecture Decisions Made

No feature flag initially - accessibility always on
Non-breaking - existing apps/widgets work without changes
Split placement: types in iced_core, collection in iced_runtime, integration in iced_winit
Option<AccessibilityNode> - None = transparent to tree (layout-only widgets)
Widget author responsibility - widgets opt-in to accessibility, apps get it free
Operation pattern for tree traversal - leverages existing iced infrastructure

## 🎯 What's Next

### Immediate Priorities (This Week)

1. **Hybrid ID System - Widget ID Support** ✅ COMPLETE (Nov 9, 2025)
   - ✅ Widget ID field already added to AccessibilityNode
   - ✅ TreeBuilder checks widget_id before falling back to type+index ID
   - ✅ **VERIFIED WORKING** - tested with counter example, IDs stable across frames
   - ✅ Flat type+index approach provides zero-burden stability for static layouts
   - 📝 Recommendation: Dynamic lists should use explicit widget IDs for maximum stability

2. **Text Widget Accessibility** 🎯 HIGH PRIORITY
   - Current: Text creates Role::Label nodes
   - ⚠️ Issue: Count value (text widget) should be readable but not focusable
   - Need to verify text widgets are working correctly with screen readers
   - May need `.focusable(false)` for static text
   - Test reading order in complex layouts

3. **More Screen Reader Testing** 🚧 IN PROGRESS
   - ✅ VoiceOver (macOS) - counter example tested
   - ⏳ Narrator (Windows)
   - ⏳ Orca (Linux)
   - Validate button press actions work end-to-end
   - Test navigation patterns and announcement text

### Short Term (Next 1-2 Weeks)

4. **Core Interactive Widgets** 🎯 PRIORITY
   - **TextInput** (Role::TextInput with value editing)
     - Similar to Button - is a leaf node
     - Needs value property and text editing actions
     - Focus management critical here
   - **Checkbox** (Role::CheckBox with checked state)
     - Leaf node with checked/unchecked state
     - Toggle action support
   - **Slider** (Role::Slider with value range)
     - Leaf node with value, min, max
     - Increment/Decrement actions

5. **Container Widgets** 🎯 MEDIUM PRIORITY
   - **Column/Row** - Should these create accessibility nodes?
     - Likely transparent (no accessibility node)
     - Just pass through to children
   - **Scrollable** - Role::ScrollView
     - Needs scroll position and range
     - May be a leaf node containing virtual children
   - **Container** - Likely transparent

6. **Overlay & Complex Widgets** 📅 LATER
   - Tooltips (Role::Tooltip)
   - Modals (Role::Dialog)
   - Dropdowns (Role::Menu)
   - PickList, ComboBox
   - Image (Role::Image with alt text)

## 🎯 Key Architecture Decisions

### Leaf Node Architecture (Nov 9, 2025)

**DECISION**: Use `is_leaf_node` flag to control child traversal

Why this approach:
- **Composite widgets** (Button, TextInput) need to present as single accessible units
- **Prevents duplicate focus stops** - button text doesn't get its own node
- **Clean semantics** - `is_leaf_node(true)` clearly indicates terminal element
- **Flexible** - widgets can choose to be containers or leaves

Implementation:
```rust
// Widget declares itself as leaf
AccessibilityNode::new(bounds)
    .role(Role::Button)
    .label("Click me")
    .is_leaf_node(true)  // Children won't be added to tree

// TreeBuilder tracks state
inside_leaf_node: bool  // Set when processing leaf node's children
```

Key insights:
- **Leaf node ≠ no children** - widget can have visual children but be accessibility leaf
- **Proper reset critical** - `inside_leaf_node` must reset after traverse() or siblings affected
- **Button use case** - button contains text visually, but text embedded in button's label for a11y

### Tree Traversal Strategy (Nov 2, 2025)

**DECISION**: Use iced's Operation pattern for tree traversal

Why this approach:
- widget::Operation trait already exists for tree traversal (see focus, scrollable, text_input operations)
- Widgets implement operate() which handles Element + Tree + Layout zipping
- Non-breaking - uses existing infrastructure
- Future-proof - new widget types automatically supported
- Gets bounds/layout info for free

Implementation approach:
1. ✅ Create accessibility::Operation in iced_runtime
2. ✅ Operation collects accessibility info during traversal
3. ✅ Builds AccessKit TreeUpdate in build() method
4. ✅ Call via UserInterface::operate() in update cycle

Incremental updates:
- ✅ Phase 1 (MVP): Full rebuild every frame (simple, correct) - IMPLEMENTED
- Phase 2 (Later): Diff previous tree, send only changes to AccessKit

### Stable NodeID Strategy (Nov 9, 2025 - VERIFIED WORKING)

**DECISION**: Hybrid ID system - widget IDs preferred, flat type+index fallback

✅ **TESTED AND VERIFIED** - Counter example shows stable IDs across frames

Why this approach:
- **Leverages iced's widget::Id system** - widgets can opt-in to explicit stable IDs
- **Automatic fallback** - type+index hashing for widgets without explicit IDs
- **Zero developer burden for static layouts** - same widget type at same position = same NodeId
- **Better than egui** - no manual `.id_salt()` needed for static layouts
- **Zero breaking changes** - works automatically for all existing apps

Implementation:
```rust
// TreeBuilder tracks widget type counts globally (flat structure)
type_counters: HashMap<String, usize>  // {"button": 2, "label": 1, ...}

// Hybrid ID generation
fn generate_stable_id(widget_type: &str, widget_id: Option<&Id>) -> NodeId {
    match widget_id {
        // Priority 1: Use explicit widget ID (maximum stability)
        Some(id) => NodeId::from(id),  // Deterministic hash of widget::Id

        // Priority 2: Flat type+index fallback
        None => {
            let index = type_counters[widget_type]++;
            let path = format!("window/{}[{}]", widget_type, index);
            NodeId(hash(path))  // e.g., "window/button[0]" → stable across frames
        }
    }
}
```

**Verification results (Nov 9, 2025)**:
```
Frame 1:
  window/container[0] → NodeId(185100343528096102)
  window/text[0]      → NodeId(17103381964764871104)
  window/label[0]     → NodeId(2477138832485704243)

Frame 2:
  window/container[0] → NodeId(185100343528096102)  ✅ SAME
  window/text[0]      → NodeId(17103381964764871104) ✅ SAME
  window/label[0]     → NodeId(2477138832485704243)  ✅ SAME
```

Research findings:
- **Browsers**: DOM element lifetime provides natural stability
- **egui**: Hash-based IDs with auto-increment counter (requires manual `.id_salt()` for dynamic lists)
- **pop-os/iced**: `Id::unique()` in constructors (doesn't work - widgets recreated every frame)
- **Our solution**: Hybrid approach - explicit IDs when available, flat type+index fallback

Stability characteristics:
- ✅ Same widget type at same traversal order = same ID (perfect for static layouts)
- ✅ Explicit widget IDs provide maximum stability (recommended for dynamic content)
- ⚠️ Inserting widget in middle shifts subsequent type indices (use widget::Id for dynamic lists)
- ✅ Adding widget at end doesn't change existing IDs
- ✅ **VERIFIED STABLE** across multiple frame updates

Phase 0: Research and Architecture Design (Week 1-2)
0.1 Deep Dive into Iced Architecture

 Study widget::Id system:

Analyze iced_core/src/widget/id.rs
Understand Id::unique() vs Id::new() behavior
Document how IDs are currently used (if at all) by widgets
Research if widgets can have stable IDs assigned


 Analyze multi-window support:

Study iced_runtime/src/multi_window.rs and WindowManager
Understand window lifecycle and widget tree per window
Document how window::Id relates to widget trees
Decision point: Per-window or unified accessibility tree?


 Understand overlay system:

Study iced_core/src/overlay.rs and Overlay trait
Map overlay types (tooltips, modals, dropdowns)
Understand overlay positioning and lifecycle
Document how overlays relate to parent widgets


 Research UserInterface lifecycle:

Study iced_runtime/src/user_interface.rs
Map exact points where tree is built, updated, drawn
Identify where accessibility tree construction should hook in
Understand State::diff() and tree reconciliation



0.2 Focus System Investigation

 Current focus management:

Search for focus handling in event processing
Check if widget::Tree tracks focus state
Analyze keyboard navigation implementation
Document gaps in current focus system
Research widget::operation::focusable


 Focus requirements for accessibility:

Map AccessKit focus requirements
Design focus tracking strategy
Plan keyboard navigation enhancement



0.3 ID Stability Strategy Research

 Investigate ID generation approaches:

rust  // Option 1: Use widget::Id directly
  // Option 2: Hash-based IDs from widget position + type
  // Option 3: Manual ID assignment via builder pattern
  // Option 4: Hybrid approach

 Study how other immediate-mode GUIs solve this:

Deep dive into egui's Id generation
Research Dear ImGui accessibility forks
Look at Flutter's semantic tree approach


 Prototype ID stability solutions:

Create minimal test case with dynamic content
Test ID stability across frame updates
Benchmark memory overhead of ID caching



Phase 1: Architectural Design (Week 2)
1.1 Non-Breaking Widget Extension Strategy ✅ COMPLETE

 ✅ Design accessibility trait: DECISION MADE

  // ✅ Option 2 CHOSEN: Add to Widget trait with default impl
  // Located in: core/src/widget.rs
  fn accessibility(
      &self,
      _state: &Tree,
      _layout: Layout<'_>,
  ) -> Option<crate::accessibility::AccessibilityNode> {
      None  // Default: transparent to accessibility tree
  }

 ✅ Decision: Option 2 chosen - direct Widget trait method with default None
 ✅ Proof-of-concept complete: Button widget implemented (widget/src/button.rs:468)

1.2 Tree Synchronization Architecture

 Design tree mapping strategy:

rust  pub struct AccessibilityTreeManager {
      widget_tree_version: u64,
      node_cache: HashMap<StableId, CachedNode>,
      window_trees: HashMap<window::Id, AccessibilityTree>,
  }
```
- [ ] **Handle dynamic content**:
  - Scrollable areas with virtual content
  - List builders and lazy loading
  - Conditional rendering

### 1.3 Adapter Placement Decision
- [ ] **Evaluate options**:
  - Option A: In `iced_winit::Application`
  - Option B: In `iced_runtime::UserInterface`
  - Option C: Separate `iced_accessibility` crate
- [ ] **Consider**:
  - Platform independence
  - State management
  - Event routing
- [ ] **Decision document** with rationale

## Phase 2: Foundation Implementation (Week 3-4)

### 2.1 Core Infrastructure
- [ ] **Create accessibility module structure**:
```
  iced_accessibility/
  ├── src/
  │   ├── lib.rs
  │   ├── tree.rs         # Tree management
  │   ├── id.rs           # ID generation/caching
  │   ├── adapter.rs      # Platform adapter wrapper
  │   ├── node_builder.rs # Node construction
  │   └── widget_ext.rs   # Widget extensions
2.2 ID Management System

 Implement chosen ID strategy:

rust  pub struct IdManager {
      // Stable ID generation based on research findings
      generation_strategy: IdStrategy,
      cache: IdCache,
  }

 Handle edge cases:

Widgets without IDs
Dynamically created widgets
Widgets that change type at same position



2.3 Tree Construction Pipeline

 Hook into UserInterface:

rust  impl UserInterface {
      pub fn build_accessibility_tree(&self) -> Option<TreeUpdate> {
          // Implementation based on research
      }
  }

 Implement tree visitor:

Walk widget tree
Walk overlay tree
Maintain parent-child relationships



Phase 3: Widget Support (Week 4-5)
3.1 Widget Accessibility Implementation

 Create widget mapper:

rust  pub struct WidgetAccessibilityMapper {
      handlers: HashMap<TypeId, Box<dyn WidgetHandler>>,
  }

 Implement for core widgets (with specific iced types):

widget::Button → Role::Button
widget::Text → Role::StaticText
widget::TextInput → Role::TextInput
widget::Checkbox → Role::CheckBox
widget::Radio → Role::RadioButton
widget::Scrollable → Role::ScrollView



3.2 Overlay Accessibility

 Map overlay types:

widget::tooltip::Tooltip → Role::Tooltip
Modal overlays → Role::Dialog
Dropdown overlays → Role::Menu


 Handle overlay focus trapping
 Implement overlay tree merging

3.3 Component and Lazy Widgets

 Research widget::Component trait
 Handle widget::Lazy
 Ensure stable IDs for dynamic content

Phase 4: Event and Action Handling (Week 5-6)
4.1 Action Router Design

 Map AccessKit actions to iced events:

rust  pub fn route_action(
      action: ActionRequest,
      window_id: window::Id,
      widget_tree: &widget::Tree,
  ) -> Vec<Event> {
      // Convert AccessKit action to iced Event
  }

 Handle platform-specific actions
 Implement action queueing if needed

4.2 Focus Management Implementation

 Extend or create focus system:

rust  pub struct FocusManager {
      focused_widget: Option<widget::Id>,
      focus_chain: Vec<widget::Id>,
      focus_scopes: HashMap<widget::Id, FocusScope>,
  }

 Implement tab navigation
 Handle focus scopes and trapping

4.3 Event Integration

 Modify event processing pipeline
 Add accessibility event types
 Ensure event ordering is correct

Phase 5: Multi-Window Support (Week 6)
5.1 Window Management

 Implement per-window trees:

rust  pub struct MultiWindowAccessibility {
      window_adapters: HashMap<window::Id, Adapter>,
      window_trees: HashMap<window::Id, TreeState>,
  }

 Handle window focus changes
 Manage window lifecycle events

5.2 Cross-Window Relationships

 Handle modal windows
 Parent-child window relationships
 Focus return after window close

Phase 6: Testing Framework (Week 7)
6.1 Testing Infrastructure

 Create accessibility test utilities:

rust  pub mod testing {
      pub fn assert_tree_structure(tree: &Tree, expected: &str);
      pub fn assert_node_properties(node: &Node, role: Role);
      pub fn simulate_screen_reader_navigation(tree: &Tree);
  }
6.2 Widget Test Coverage

 Create test for each widget type
 Test dynamic content scenarios
 Test focus navigation paths
 Test overlay interactions

6.3 Integration Tests

 Multi-window scenarios
 Complex widget hierarchies
 Performance benchmarks:

Measure tree construction time
Measure update generation time
Memory usage tracking



Phase 7: Platform Testing (Week 8)
7.1 Screen Reader Validation

 Create test applications:

Simple widget showcase
Complex form application
Multi-window application
Dynamic content application


 Platform-specific testing:

Windows: NVDA, JAWS, Narrator
macOS: VoiceOver
Linux: Orca (if available)



7.2 Accessibility Tool Validation

 Use platform accessibility inspectors
 Document any platform-specific issues
 Create workarounds if needed

Phase 8: Documentation and Migration (Week 9)
8.1 Documentation

 Architecture documentation:

ID stability solution
Tree construction pipeline
Event handling flow
Multi-window support


 Widget developer guide:

How to make custom widgets accessible
Best practices
Common patterns


 Migration guide:

For existing custom widgets
Performance considerations
Troubleshooting guide



8.2 Example Applications

 Update existing examples
 Create accessibility-specific examples
 Create debugging/inspection tools

Critical Research Questions to Answer First

Can we assign stable widget::Id to all widgets without breaking changes?
How does iced's immediate-mode architecture affect incremental updates?
Where in UserInterface lifecycle should we construct accessibility trees?
How to handle Overlay accessibility without duplicating logic?
Should we have one tree per window or a unified tree?
How to extend Widget trait without breaking existing implementations?
Can we track focus without modifying core event loop?
How to handle Component and Lazy widgets' internal state?
Performance impact of maintaining accessibility state?
How to test accessibility without screen readers in CI?

Risk Mitigation Strategies
Technical Risks

ID Stability: Build comprehensive caching system with fallbacks
Performance: Use lazy initialization, only build trees when needed
Breaking Changes: Use feature flags and separate traits
Multi-window Complexity: Start with single-window, expand gradually

Unknown Unknowns

Time box research phase: Maximum 2 weeks
Build minimal prototype early: Week 2
Get maintainer feedback early: Before Phase 3
Have fallback strategies for each architectural decision

Success Metrics

 All core widgets pass accessibility audit
 ID stability maintained across 1000+ frame updates
 Tree construction < 1ms for typical application
 Zero breaking changes to existing public API
 Screen readers can navigate all content
 Focus navigation works with keyboard only
 Multi-window applications fully supported
 90%+ test coverage for accessibility code

## 📋 Recommended Next Steps (Priority Order)

### 1. Text Widget Review & Testing 🎯 **START HERE**
**Why**: Text is fundamental and needs verification
- Check if static text is focusable (should not be)
- Verify screen reader reads text in correct order
- Test with counter example - does "0" announce correctly?
- Estimated time: 30 minutes

### 2. TextInput Widget Implementation 🎯 **HIGH VALUE**
**Why**: First interactive input widget, critical for forms
- Similar pattern to Button (is a leaf node)
- Add `value` property to AccessibilityNode
- Implement text editing actions (SetValue)
- Focus management is critical here
- Estimated time: 2-3 hours

### 3. Checkbox Widget Implementation 🎯 **HIGH VALUE**
**Why**: Second most common interactive widget
- Leaf node with checked state
- Add checked/indeterminate states to AccessibilityNode
- Implement Toggle action
- Simpler than TextInput, good learning example
- Estimated time: 1-2 hours

### 4. Dynamic Content Testing 📊 **IMPORTANT**
**Why**: Validates hybrid ID system works
- Create test with list that adds/removes items
- Verify widget IDs maintain stability
- Test screen reader doesn't lose context
- Estimated time: 1-2 hours

### 5. Container Widget Audit 🔍 **CLEANUP**
**Why**: Many widgets might not need accessibility nodes
- Review Column, Row, Container - likely transparent
- Document which widgets should return None
- Add comments explaining why
- Estimated time: 1 hour

## ⚠️ Known Issues to Address

1. **Text focusability** - Static text might be focusable when it shouldn't be
2. **Missing widgets** - Most iced widgets don't have accessibility yet
3. **Dynamic lists** - Need real-world testing with widget IDs
4. **Focus management** - No keyboard navigation between widgets yet
5. **Action routing** - Button clicks work, but need to verify other actions

## 🎓 Lessons Learned

1. **Naming matters** - `is_leaf_node` is much clearer than `traverse_children`
2. **State tracking is tricky** - `inside_leaf_node` reset bug affected all siblings
3. **Screen reader testing essential** - Discovered button text issue immediately
4. **Explicit labels needed** - Can't auto-extract text from Elements easily
5. **Start simple** - Button was perfect first widget (single action, clear semantics)
