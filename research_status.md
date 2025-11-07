# AccessKit Integration Research Status

**Last Updated**: Nov 6, 2025 (Stable NodeIDs Implemented)
**Current Branch**: `accesskit-integration` (commit: latest)  
**Status**: Stable NodeID system implemented, accessibility tree working with VoiceOver-ready

This document tracks what research can be done immediately versus what needs clarification or user input.

---

## 🎉 Implementation Progress

### ✅ Completed Nov 6, 2025: Stable NodeID System

**Critical Achievement**: Implemented automatic stable NodeID generation with **ZERO developer burden**.

**Implementation** (runtime/src/accessibility.rs):
- **Path-based ID generation**: Each widget gets stable ID from tree position
  - Example paths: `"window/button[0]"`, `"window/button[1]"`, `"window/label[0]"`
  - Hash path → stable u64 → AccessKit NodeId
- **TreeBuilder enhancements**:
  - `path_stack: Vec<String>` - tracks position in widget tree
  - `type_counters: HashMap<String, usize>` - counts widgets by type per level
  - `generate_stable_id(widget_type)` - creates deterministic NodeId from path
- **All Operation methods updated**: container, focusable, text, text_input, scrollable, accessibility
- **Role-to-type mapping**: Maps AccessKit Role to widget type string for path generation

**Verification**:
```
First build:  button[0] → NodeId 7447623757530889483
Second build: button[0] → NodeId 7447623757530889483  ✅ STABLE!
```

**Key Features**:
- ✅ **Automatic** - No developer code changes needed
- ✅ **Stable** - Same widget position = same NodeId across frames
- ✅ **Path-based** - Derived from widget tree structure
- ✅ **Zero breaking changes** - Fully backward compatible
- ✅ **Better than egui** - No manual `.id_salt()` needed for static layouts

**Compared to Browser Approach**:
- Browsers: DOM elements have persistent object identity
- egui (immediate-mode): Requires manual `.id_salt()` for dynamic content
- iced (retained-mode): Widget tree persists → natural stability from tree structure

**Status**: Counter example running with stable IDs, ready for VoiceOver testing!

---

### ✅ Completed Nov 2, 2025: Initial Infrastructure
1. **Dependencies Added** (Cargo.toml):
   - accesskit = "0.21.1"
   - accesskit_winit = "0.29.2"  
   - [patch.crates-io] to redirect winit dependency to iced's fork ✅ Works!

2. **Core Types Created** (core/src/accessibility/):
   - `AccessibilityNode` wrapper with builder pattern
   - Public fields (no getters needed - no invariants)
   - Re-exports of AccessKit types (Role, Action, NodeId)

3. **Widget Trait Extended** (core/src/widget.rs:574):
   - Added `accessibility()` method with default `None` implementation
   - Non-breaking change - all existing widgets compile without changes
   - Comprehensive documentation added

4. **First Widget Implementation** (widget/src/button.rs:468):
   - Button returns `Some(AccessibilityNode)` with:
     - Role::Button
     - Enabled state based on on_press handler
     - Focusable = true
     - Label extraction (TODO: currently defaults to "Button")

5. **Verification**:
   - ✅ `cargo check` passes in 7.13s
   - ✅ Counter example builds and runs
   - ✅ No breaking changes to existing code
   - ✅ All changes committed to `accesskit-integration` branch

6. **Tree Collection via Operation Pattern** (runtime/src/accessibility.rs):
   - ✅ `TreeBuilder` struct implementing `Operation` trait
   - ✅ Collects container, focusable, text_input, text, scrollable widgets
   - ✅ Generates stable NodeIds (path-based hashing) ⬅️ Updated Nov 6!
   - ✅ Returns complete AccessKit TreeUpdate with bounds mapping

7. **UserInterface Integration** (runtime/src/user_interface.rs:579):
   - ✅ Added `UserInterface::accessibility()` method
   - ✅ Calls TreeBuilder operation and returns TreeUpdate
   - ✅ Ready to be invoked from event loop

8. **iced_winit Integration Started** (winit/src/window.rs:174):
   - ✅ Added `accessibility: Option<accesskit_winit::Adapter>` field to Window
   - ⚠️ Initialized as None (needs proper adapter setup)

### 🚧 What's Still Missing to Make It Functional

**Nov 6 Update**: Many items completed! Remaining work:

- ~~**Stable NodeIDs**~~ ✅ DONE - Path-based hashing implemented
- ~~**Adapter initialization**~~ ✅ DONE - Created ActivationHandler and ActionHandler
- ~~**Event loop integration**~~ ✅ DONE - Calls `ui.accessibility()` after UI rebuilds
- ~~**Tree updates**~~ ✅ DONE - Sends TreeUpdate to adapter via `update_if_active()`
- ~~**Action handling**~~ ✅ DONE - Processes accessibility events, synthesizes mouse clicks
- ~~**Click actions on buttons**~~ ✅ DONE - Buttons have Click + Focus actions
- **Testing with screen reader**: Need real-world VoiceOver/Narrator/Orca validation
- **More widgets**: Text, TextInput, Checkbox, Radio, Slider, etc.
- **Overlay support**: Tooltips, modals, dropdowns
- **Optional widget IDs**: Allow developers to provide explicit IDs for extra stability

### 🎯 Major Architectural Decision Made (Nov 2, 2025)

**Tree Traversal Strategy: Use iced's Operation Pattern** ✅

After investigating tree traversal approaches, we've decided to use iced's existing `Operation` trait pattern:

**Why Operation Pattern:**
- ✅ **Non-breaking**: Uses existing infrastructure, widgets already implement `operate()`
- ✅ **Proper traversal**: Widgets handle Element + Tree + Layout zipping correctly
- ✅ **Future-proof**: New widget types automatically supported
- ✅ **Gets bounds for free**: Operation methods receive `bounds: Rectangle`
- ✅ **Consistent with iced**: Same pattern as focus, scrollable, text_input operations

**Incremental Update Strategy:**
- **Phase 1 (MVP)**: Full tree rebuild on every update
  - Simple, correct, works immediately
  - AccessKit adapters handle frequent rebuilds efficiently
  - Good enough for most UIs
- **Phase 2 (Optimization)**: Diff-based incremental updates
  - Store previous TreeUpdate, compare with new
  - Only send changed nodes to AccessKit
  - AccessKit docs: "should only include nodes that are new or changed"

**Next Critical Steps for Next Session**: 
1. ✅ ~~Implement accessibility Operation in `iced_runtime`~~ DONE
2. ✅ ~~Integrate Operation call into UserInterface lifecycle~~ DONE
3. ✅ ~~Complete accesskit_winit adapter initialization~~ DONE (Nov 6):
   - ✅ Implemented ActivationHandler trait (returns initial tree)
   - ✅ Implemented ActionHandler trait (handles screen reader actions)
   - ✅ Adapter initialized with event_loop_proxy during window creation
4. ✅ ~~Wire up tree updates in event loop~~ DONE (Nov 6):
   - ✅ Calls `ui.accessibility()` after UI rebuilds
   - ✅ Sends TreeUpdate to `adapter.update_if_active()`
   - ✅ Stores NodeId → bounds mapping for action routing
5. ✅ ~~Implement stable NodeID generation~~ DONE (Nov 6):
   - ✅ Path-based hashing (e.g., "window/button[0]" → NodeId)
   - ✅ Automatic, zero developer burden
   - ✅ Stable across frame updates
6. 🧪 Test with counter example and screen reader - READY FOR TESTING

---

## ✅ Resolved Design Decisions (from User)

### **1. Feature Flag Strategy** ✅ RESOLVED
- **Decision**: Accessibility will be **always on** initially (no feature flag)
- **Rationale**: Simplifies initial implementation, can add feature flag later after maturity
- **Impact**: No conditional compilation needed in Phase 1-7
- **Future**: Can gate behind feature after accessibility is stable and proven

### **2. Breaking Changes Philosophy** ✅ RESOLVED
- **Decision**: **Widget authors** bear the burden, **NOT app developers**
- **Principle**: Simple UIs should get accessibility support automatically without code changes
- **Constraint**: Widget trait extensions must have sensible defaults
- **Impact**: Favors Option 1 (separate trait with blanket impl) or Option 2 (Widget trait with default impl)
- **Goal**: `cargo build` on existing apps should "just work" with accessibility

### **3. Crate Organization** ✅ RESOLVED & IMPLEMENTED
- **Decision**: Split approach - types in **iced_core**, collection in **iced_runtime**
- **Rationale**: Core types platform-independent, tree building needs UserInterface access
- **Implementation**: 
  - ✅ `iced_core::accessibility` module with AccessibilityNode and Widget trait method
  - 🚧 Tree collection logic will go in `iced_runtime` (not yet implemented)
  - 🚧 Platform adapter integration in `iced_winit` (not yet implemented)
- **Impact**: Clean separation of concerns, non-breaking changes

### **4. Custom Widget Error Handling** ✅ RESOLVED
- **Decision**: Missing accessibility **must NOT cause runtime errors**
- **Behavior**: Graceful degradation - no panic, no crash
- **Default**: Likely Role::Unknown or invisible node in tree
- **Impact**: Need robust error handling and default implementations

---

## ❓ Remaining Open Questions

### **High Priority (Need answers before Week 2)**
1. **iced_core vs iced_runtime placement**: Where should accessibility module live?
   - iced_core: Platform-independent, but no access to UserInterface
   - iced_runtime: Has UserInterface, but pulls in more dependencies
   
2. **Platform priority**: Windows first, or all platforms (Windows/Mac/Linux) simultaneously?
   - All platforms: More complex, but ensures cross-platform design
   - Windows first: Faster iteration, but may need refactoring

3. **WASM support**: Required, nice-to-have, or out-of-scope?
   - AccessKit has limited WASM support
   - May affect adapter architecture

4. **Per-window vs unified tree**: Should each window have its own accessibility tree?
   - Research needed, but user preference matters

### **Medium Priority (Can be decided during Week 2)**
5. **Focus system**: Enhance existing or just expose what's there?
6. **Timeline expectations**: Is 8-9 weeks realistic or flexible?
7. **Maintainer coordination**: Are you working with iced maintainers?

---

## ✅ Research Available Now (Immediately Actionable)

### **Phase 0.1: Deep Dive into Iced Architecture**

#### 1. widget::Id System (Ready)
- ✅ Already read `iced_core/src/widget/id.rs`
- ✅ Can trace how `Id::unique()` and `Id::new()` are used in existing widgets
- ✅ Can search codebase for widgets that assign IDs
- **Action**: Grep for `widget::Id` usage patterns across all widgets

#### 2. Multi-Window Support (Ready with caveat)
- ⚠️ **Issue**: Plan mentions `iced_runtime/src/multi_window.rs` but this file may not exist
- ✅ Can study `iced_winit/src/window.rs` and `WindowManager` (already found)
- ✅ Can trace window lifecycle through `iced_winit/src/lib.rs`
- ✅ Can analyze `window::Id` usage
- **Action**: Verify actual file structure, use WindowManager in winit instead

#### 3. Overlay System (Ready)
- ✅ Already started reading `iced_core/src/overlay.rs`
- ✅ Can examine all overlay implementations in `widget/src/`
- ✅ Can trace overlay lifecycle in `UserInterface`
- **Action**: Map all overlay types (tooltips, modals, dropdowns, combo_box, pick_list)

#### 4. UserInterface Lifecycle (Ready)
- ✅ Already read `user_interface.rs`
- ✅ Can map exact hook points (build, update, draw, operate)
- ✅ Can understand `Tree::diff()` mechanism
- **Action**: Document complete lifecycle with potential accessibility insertion points

### **Phase 0.2: Focus System Investigation**

#### 5. Current Focus Management (Ready)
- ✅ Can search for `widget::operation::focusable` 
- ✅ Can grep for focus-related code in widgets
- ✅ Can trace keyboard navigation in event handlers
- ✅ Can examine TextInput and other focusable widgets
- **Action**: Create comprehensive focus system map

### **Phase 0.3: ID Stability Strategy Research** ✅ COMPLETED (Nov 6)

#### 6. ID Generation Approaches ✅ RESOLVED
- ✅ Researched browser approach (DOM element lifetime provides stability)
- ✅ Researched egui approach (hash-based with manual `.id_salt()` for dynamic content)
- ✅ Researched pop-os/iced accessibility branch (auto-generated IDs in constructors - doesn't work with immediate-mode)
- ✅ **Decision**: Path-based hashing leveraging iced's retained-mode architecture
- ✅ **Implementation**: `generate_stable_id(widget_type)` hashes tree path
- ✅ **Result**: Automatic stability without developer burden

#### 7. Study Other Implementations ✅ COMPLETED
- ✅ Deep dive into egui's AccessKit integration
- ✅ Analyzed pop-os/iced `iced-accessibility` branch
- ✅ Studied browser accessibility architecture (Chromium/Firefox)
- ✅ Key insight: iced's retained-mode gives natural stability vs egui's immediate-mode
- ✅ Verified: path-based IDs stable across frame updates in counter example

### **Phase 1.1: Non-Breaking Widget Extension**

#### 8. Widget Extension Strategy ✅ IMPLEMENTED
- ✅ **Decision Made**: Option 2 - Add method to Widget trait with default impl
- ✅ **Implementation**: `Widget::accessibility()` returns `Option<AccessibilityNode>` 
- ✅ **Default**: Returns `None` (transparent to accessibility tree)
- ✅ **Proof-of-concept**: Button widget implementation complete
- **Location**: core/src/widget.rs:574, widget/src/button.rs:468

### **Additional Research Available Now**

#### 9. Existing Widget Implementations (Ready)
- ✅ Can read Button, Text, TextInput, Checkbox implementations
- ✅ Can understand their state management in `widget::Tree`
- ✅ Can see how they use `operate()` method
- **Action**: Document widget patterns for accessibility mapping

#### 10. AccessKit Integration Patterns (Ready)
- ✅ Can read more AccessKit documentation
- ✅ Can fetch more examples from AccessKit repository
- ✅ Can understand platform-specific adapters
- **Action**: Study accesskit_windows, accesskit_macos, accesskit_unix APIs

---

## ⚠️ Research Blocked or Needs Clarification

### **Critical Path Items**

#### 1. Multi-Window File Location
- ❌ `iced_runtime/src/multi_window.rs` may not exist
- ✅ **Action**: Verify actual file structure, use WindowManager in winit instead
- **Status**: Can be resolved immediately with file search

#### 2. ID Stability Testing
- ❌ Plan says "Prototype ID stability solutions" but we're in plan mode
- ⚠️ **Needs clarification**: Can we create temporary test files in plan mode?
- 🤔 **Alternative**: Design mentally, document approach, implement later
- **Blocker**: Need permission to create test files

#### 3. Performance Baselines
- ❌ Can't measure "tree construction < 1ms" without running benchmarks
- ⚠️ **Needs clarification**: Should we identify what to benchmark?
- **Blocker**: Need to implement before measuring

### **Decision Points Requiring User Input**

#### 4. Architecture Decisions (PARTIALLY RESOLVED)
- ❓ **Per-window vs unified accessibility tree** (affects fundamental design) - STILL OPEN
- ✅ **Adapter placement**: ~~separate crate~~ → **iced_core or iced_runtime** (which one still open)
- ⚠️ **Widget extension approach** (affects all subsequent work) - Constrained by "no app-level breaking changes"
- ❓ **ID strategy choice** (affects tree stability) - STILL OPEN
- **Impact**: Blocks Phase 1 and beyond

#### 5. Scope Clarifications (PARTIALLY RESOLVED)
- ❓ Is WASM support required? (affects adapter choice) - STILL OPEN
- ❓ Are all platforms (Windows/Mac/Linux) equally important? - STILL OPEN
- ❓ Should focus system be enhanced or just exposed? - STILL OPEN
- **Impact**: Affects implementation complexity and timeline

---

## 🔍 Things Still Overlooked or Unclear

### **Architectural Gaps**

#### 1. Renderer Abstraction
- ⚠️ Iced supports multiple renderers (wgpu, tiny_skia)
- ❓ Does accessibility need renderer-specific code?
- 🔍 **Overlooked**: How layout coordinates map to screen coordinates per renderer
- **Impact**: May need renderer-specific accessibility code

#### 2. Shell Usage for Accessibility
- ⚠️ Plan shows `Shell<'_, Message>` but accessibility may need different message type
- 🔍 **Unclear**: Should AccessKit actions become user Messages or separate events?
- ❓ Type signature compatibility: `Shell<'_, AccessibilityMessage>` vs `Shell<'_, UserMessage>`
- **Impact**: Affects event handling architecture

#### 3. Widget State vs Accessibility State
- ⚠️ `widget::Tree` stores internal state
- 🔍 **Unclear**: Should accessibility state live in widget::Tree or separately?
- ❓ How to sync widget state changes with accessibility tree updates?
- **Impact**: Memory overhead and synchronization complexity

#### 4. Event Ordering and Precedence
- ⚠️ AccessKit events need to coexist with user events
- 🔍 **Overlooked**: Priority when both screen reader and user click button
- 🔍 **Overlooked**: Event loop integration in `iced_winit::run()`
- **Impact**: May cause race conditions or conflicts

### **Implementation Details Still Vague**

#### 5. Overlay Tree Merging
- ⚠️ Plan mentions it but doesn't explain algorithm
- 🔍 **Need to design**: How overlay nodes insert into base tree
- ❓ Do overlays have separate root or parent to base widget?
- **Impact**: Complex tree structure management

#### 6. ID Cache Invalidation
- ⚠️ Plan has `IdCache` but no invalidation strategy
- 🔍 **Overlooked**: When to clear cache (window close, widget rebuild)?
- 🔍 **Overlooked**: Memory bounds on cache growth
- **Impact**: Memory leaks possible

#### 7. Lazy/Component Widget Internals
- ⚠️ Component is deprecated but may still be in use
- 🔍 **Need research**: How does widget::Lazy actually work?
- ❓ Can lazy widgets provide accessibility info before full render?
- **Impact**: May have incomplete accessibility tree

#### 8. Platform-Specific Quirks
- ⚠️ Plan mentions platform testing but not platform-specific code
- 🔍 **Overlooked**: Windows, Mac, Linux may need different node properties
- 🔍 **Example**: Windows needs HWND, Mac needs NSView - how to abstract?
- **Impact**: Platform-specific code paths needed

### **Testing Gaps**

#### 9. CI/CD Accessibility Testing
- ⚠️ Plan asks "How to test without screen readers in CI?" but doesn't answer
- 🔍 **Need solution**: Mock screen reader or tree validator?
- ❓ Can AccessKit provide testing utilities?
- **Impact**: Difficult to prevent regressions

#### 10. Regression Testing
- 🔍 **Overlooked**: How to ensure accessibility doesn't break with iced updates?
- ❓ Should accessibility be in iced repo or separate (affects CI integration)?
- **Impact**: Maintenance burden

### **Feature Flag Design** ✅ RESOLVED (Initially)

#### 11. Feature Flag Granularity ✅ RESOLVED
- ✅ **Decision**: NO feature flag initially - always enabled
- 🔍 **Future consideration**: May add feature flag after stability proven
- ✅ **No-op cost when disabled**: N/A - always enabled
- **Impact**: Simpler initial implementation, no conditional compilation

#### 12. API Surface When Disabled ✅ RESOLVED
- ✅ **Decision**: N/A - accessibility always present
- ✅ **Public API**: All accessibility APIs always available
- **Impact**: No API design constraints from feature flags

### **Migration and Compatibility** ✅ RESOLVED

#### 13. Existing Applications ✅ RESOLVED
- ✅ **Decision**: Apps need **ZERO code changes** - just recompile
- ✅ **Migration path**: Automatic accessibility for existing apps
- ✅ **Opt-in vs opt-out**: Automatic opt-in, works out of the box
- **Impact**: Maximum adoption, minimal friction

#### 14. Custom Widget Authors ✅ RESOLVED
- ✅ **Decision**: Accessibility is **optional** - missing implementation is **safe**
- ✅ **Error behavior**: **NO runtime errors** - graceful degradation
- ✅ **Default behavior**: Generic/unknown node in accessibility tree
- **Impact**: Custom widgets work without accessibility, but should add it

---

## 📋 Recommended Research Order

### **Week 1, Days 1-2 (Can Start Immediately)**
1. ✅ Verify multi-window file structure (`find . -name "*window*.rs"`)
2. ✅ Study widget::Id usage across codebase
3. ✅ Map UserInterface lifecycle completely
4. ✅ Research focus system (grep for "focus", "focusable")
5. ✅ Read all overlay implementations

**Deliverable**: Architecture map of iced's current state

### **Week 1, Days 3-4**
6. ✅ Study egui's AccessKit integration in detail
7. ✅ Analyze more AccessKit examples
8. ✅ Document widget state management patterns
9. ✅ Prototype ID generation strategies (design, not implement)
10. ✅ Map widget trait extension options

**Deliverable**: Comparison of ID strategies and widget extension approaches

### **Week 1, Day 5**
11. 📝 Document findings from Days 1-4
12. ❓ Prepare decision points for user/maintainer discussion
13. 📊 Create comparison matrix for architecture options

**Deliverable**: Decision document with recommendations

### **Week 2 (After Initial Research)**
14. 🤝 Get user input on critical decisions
15. 🔨 Build minimal proof-of-concept (after plan approval)
16. ✅ Finalize architecture based on POC results

**Deliverable**: Validated architecture ready for implementation

---

## 🎯 Critical Questions for User/Maintainer

### **✅ ANSWERED** (Updated based on user input)
3. ✅ **Breaking changes acceptable?** → NO for apps, YES for widget authors
4. ✅ **Separate crate vs in-tree?** → In-tree (iced_core or iced_runtime)
10. ✅ **Opt-in vs opt-out accessibility?** → Automatic (opt-in by default)
11. ✅ **Custom widget requirements** → Optional, graceful degradation if missing

### **❓ STILL NEED ANSWERS**

#### **Process Questions**
1. **Is there an iced maintainer you're coordinating with?** (affects architecture decisions)
2. **Timeline flexibility?** (8-9 weeks is aggressive for one person)

#### **Technical Questions**
5. **Platform priority**: Windows first? All platforms simultaneously?
   - Recommendation: Start Windows, ensure cross-platform design
6. **WASM support required?** (AccessKit has limited WASM support)
   - AccessKit WASM support is experimental/limited
7. **Per-window vs unified tree preference?** (or should we research and recommend?)
   - Needs research, but affects fundamental architecture
8. **Adapter placement preference**: iced_core or iced_runtime?
   - iced_core: Platform-agnostic, but no UserInterface access
   - iced_runtime: Has UserInterface, but more dependencies
   - Recommendation: Research and propose based on findings

#### **Design Philosophy Questions**
9. **Should focus system be enhanced or just expose what's there?** (scope creep vs completeness)
   - Affects whether we build new focus tracking or use existing

---

## 📊 Research Progress Tracking

### **Immediate Research Tasks (Week 1, Days 1-2)**
- [ ] Verify multi-window file structure
- [ ] Complete widget::Id usage analysis
- [ ] Map UserInterface lifecycle with hook points
- [ ] Document current focus system
- [ ] Catalog all overlay types and behaviors

### **Secondary Research Tasks (Week 1, Days 3-4)**
- [ ] Deep dive into egui's AccessKit integration
- [ ] Study AccessKit examples (beyond simple.rs)
- [ ] Analyze widget state management patterns
- [ ] Design ID generation strategy options
- [ ] Design widget extension approach options

### **Synthesis Tasks (Week 1, Day 5)**
- [ ] Create architecture findings document
- [ ] Build ID strategy comparison matrix
- [ ] Build widget extension comparison matrix
- [ ] Prepare decision points document
- [ ] List open questions for maintainers

### **Decision Points (Week 2)**
- [ ] Get user input on critical questions
- [ ] Select ID strategy
- [ ] Select widget extension approach
- [ ] Select adapter placement strategy
- [ ] Select per-window vs unified tree approach

### **Proof of Concept (Week 2)**
- [ ] Implement minimal ID stability test
- [ ] Test widget extension approach
- [ ] Validate UserInterface integration point
- [ ] Test overlay tree merging concept
- [ ] Measure baseline performance

---

## 📈 Research Readiness Summary

**Immediately Available**: ~80% of Phase 0 research can be done with readonly tools ⬆️ (up from 70%)
- ✅ Iced architecture analysis
- ✅ Existing code pattern study
- ✅ External integration examples (egui, AccessKit)
- ✅ Design and comparison work
- ✅ Widget extension strategy (constrained by "no app breaking changes")

**Blocked on Decisions**: ~10% requires user/maintainer input ⬇️ (down from 20%)
- ⚠️ iced_core vs iced_runtime placement (can research and recommend)
- ⚠️ Per-window vs unified tree (can research and recommend)
- ❓ Platform priority and WASM support
- ❓ Focus system enhancement scope

**Blocked on Implementation**: ~10% requires building prototypes
- ❌ Performance measurements
- ❌ ID stability testing
- ❌ Integration validation

**Key Constraints from User Decisions**:
1. ✅ No app-level breaking changes → Default implementations required
2. ✅ Always-on accessibility → No conditional compilation needed
3. ✅ Graceful degradation → Robust error handling essential
4. ✅ In-tree placement → Integrate with existing crate structure

---

## 🚦 Recommendation

**Start immediately with Phase 0.1-0.3 readonly research**. With 4 major decisions resolved, we can now focus on the remaining technical questions that research can answer.

### **This Week's Focus**
1. Complete iced architecture deep dive (Days 1-2)
   - Determine best crate for accessibility (iced_core vs iced_runtime)
   - Map multi-window architecture and recommend tree strategy
2. Study external examples and patterns (Days 3-4)
   - Deep dive into egui's AccessKit integration
   - Understand ID stability patterns
3. Synthesize findings and prepare recommendations (Day 5)
   - Propose crate placement with rationale
   - Propose per-window vs unified tree with rationale
   - Propose ID strategy with trade-offs

### **Next Week's Focus**
1. Discuss findings and recommendations with user
2. Get answers to remaining questions (platform priority, WASM, focus scope)
3. Make final architectural decisions
4. Build proof-of-concept to validate decisions

### **Key Success Metric**
By end of Week 1, have enough information to make informed architectural recommendations with clear trade-offs documented.

### **Key Advantages from Resolved Decisions**
- ✅ No feature flag complexity → simpler implementation
- ✅ Always-on → can assume accessibility exists in all code paths
- ✅ No app breaking changes → guides widget trait design
- ✅ In-tree placement → can access iced internals directly

### **Updated Research Priority**
With user decisions, these become most critical to research:
1. **iced_core vs iced_runtime** - Which crate gives best architecture?
2. **Widget trait extension** - How to add defaults without breaking apps?
3. **ID stability** - How to maintain stable IDs in immediate-mode UI?
4. **Multi-window** - Per-window trees or unified?
5. **Overlay integration** - How to merge overlay accessibility into main tree?
