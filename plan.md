Current Implementation Status (as of Nov 2, 2025 - End of Session)

✅ Completed in This Session

Branch: accesskit-integration (latest: 38e9f05dd)
Commits this session:
- 9cccd4688: Initial foundation (AccessibilityNode, Widget trait, Button)
- 0e5ef0162: Tree collection infrastructure
- 32eb227d1: Documentation updates (Operation pattern decision)
- 9b1beb2a5: Rewrite using Operation pattern
- 38e9f05dd: UserInterface integration complete

Implemented:
1. AccessibilityNode wrapper type (iced_core)
2. Widget::accessibility() method with default None (non-breaking)
3. Button widget implementation
4. TreeBuilder implementing Operation trait (iced_runtime)
5. UserInterface::accessibility() method
6. Window struct field for adapter (iced_winit, needs initialization)

Status: Compiles successfully, all infrastructure in place, ready for adapter wiring

🔍 Architecture Decisions Made

No feature flag initially - accessibility always on
Non-breaking - existing apps/widgets work without changes
Split placement: types in iced_core, collection in iced_runtime, integration in iced_winit
Option<AccessibilityNode> - None = transparent to tree (layout-only widgets)
Widget author responsibility - widgets opt-in to accessibility, apps get it free
Operation pattern for tree traversal - leverages existing iced infrastructure

🚧 What's Next (For Next Session)
Infrastructure is complete but not yet wired up. Remaining work:

1. Initialize accesskit_winit Adapter:
   - Implement ActivationHandler (returns initial TreeUpdate)
   - Implement ActionHandler (processes screen reader actions)
   - Call Adapter::with_event_loop_proxy during window creation
   
2. Wire up tree updates in event loop:
   - Call ui.accessibility() after UI rebuilds
   - Send TreeUpdate to adapter.update_if_active()
   
3. Test with counter example + VoiceOver/Narrator/Orca

4. Implement more widgets (Text, TextInput, Checkbox, etc.)

🎯 Tree Traversal Strategy Decision (Nov 2, 2025)

DECISION: Use iced's Operation pattern for tree traversal

Why this approach:
- widget::Operation trait already exists for tree traversal (see focus, scrollable, text_input operations)
- Widgets implement operate() which handles Element + Tree + Layout zipping
- Non-breaking - uses existing infrastructure
- Future-proof - new widget types automatically supported
- Gets bounds/layout info for free

Implementation approach:
1. Create accessibility::Operation in iced_runtime
2. Operation collects accessibility info during traversal
3. Builds AccessKit TreeUpdate in finish() method
4. Call via UserInterface::operate() in update cycle

Incremental updates:
- Phase 1 (MVP): Full rebuild every frame (simple, correct)
- Phase 2 (Later): Diff previous tree, send only changes to AccessKit

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

Next Steps

Start with Phase 0 research - Cannot proceed without understanding existing architecture
Create proof-of-concept for ID stability by end of Week 1
Get maintainer input on architectural decisions before Phase 2
Iterate based on discoveries during research phase

This revised plan addresses the critical gaps identified, particularly around understanding iced's actual architecture before making design decisions. The research phase is now essential to avoid building on incorrect assumptions.
