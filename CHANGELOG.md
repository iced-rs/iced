# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Reactive rendering. [#2662](https://github.com/iced-rs/iced/pull/2662)
- Time travel debugging. [#2910](https://github.com/iced-rs/iced/pull/2910)
- `Animation` API for application code. [#2757](https://github.com/iced-rs/iced/pull/2757)
- Headless mode testing. [#2698](https://github.com/iced-rs/iced/pull/2698)
- First-class end-to-end testing. [#3059](https://github.com/iced-rs/iced/pull/3059)
- Input method support. [#2777](https://github.com/iced-rs/iced/pull/2777)
- Hot reloading. [#3000](https://github.com/iced-rs/iced/pull/3000)
- Concurrent image decoding and uploading (and more cool stuff). [#3092](https://github.com/iced-rs/iced/pull/3092)
- `comet` debugger and `devtools` foundations. [#2879](https://github.com/iced-rs/iced/pull/2879)
- Presentation metrics for `comet`. [#2881](https://github.com/iced-rs/iced/pull/2881)
- Custom performance metrics for `comet`. [#2891](https://github.com/iced-rs/iced/pull/2891)
- Headless mode for `iced_wgpu` and concurrency foundations. [#2857](https://github.com/iced-rs/iced/pull/2857)
- Smart scrollbars. [#2922](https://github.com/iced-rs/iced/pull/2922)
- System theme reactions. [#3051](https://github.com/iced-rs/iced/pull/3051)
- `table` widget. [#3018](https://github.com/iced-rs/iced/pull/3018)
- `grid` widget. [#2885](https://github.com/iced-rs/iced/pull/2885)
- `sensor` widget. [#2751](https://github.com/iced-rs/iced/pull/2751)
- `float` widget and other cool stuff. [#2916](https://github.com/iced-rs/iced/pull/2916)
- `pin` widget. [#2673](https://github.com/iced-rs/iced/pull/2673)
- `wrap` method for `column` widget. [#2884](https://github.com/iced-rs/iced/pull/2884)
- `auto_scroll` support for `scrollable` widget. [#2973](https://github.com/iced-rs/iced/pull/2973)
- `delay` support for `tooltip` widget. [#2960](https://github.com/iced-rs/iced/pull/2960)
- `Auto` strategy to `text::Shaping`. [#3048](https://github.com/iced-rs/iced/pull/3048)
- Incremental `markdown` parsing. [#2776](https://github.com/iced-rs/iced/pull/2776)
- Customizable markdown rendering and image support. [#2786](https://github.com/iced-rs/iced/pull/2786)
- Quote support for `markdown` widget. [#3005](https://github.com/iced-rs/iced/pull/3005)
- Tasklist support for `markdown` widget. [#3022](https://github.com/iced-rs/iced/pull/3022)
- Basic layer merging for `graphics::layer::Stack`. [#3033](https://github.com/iced-rs/iced/pull/3033)
- Primitive culling in `column` and `row` widgets. [#2611](https://github.com/iced-rs/iced/pull/2611)
- Lazy `Compositor` initialization in `winit` shell. [#2722](https://github.com/iced-rs/iced/pull/2722)
- Support for `Justified` text alignment. [#2836](https://github.com/iced-rs/iced/pull/2836)
- `crisp` feature for default quad snapping. [#2969](https://github.com/iced-rs/iced/pull/2969)
- Support for double click event to `mouse_area`. [#2602](https://github.com/iced-rs/iced/pull/2602)
- `Default` implementation for `iced_wgpu::geometry::Cache`. [#2619](https://github.com/iced-rs/iced/pull/2619)
- `physical_key` field to `KeyReleased` event. [#2608](https://github.com/iced-rs/iced/pull/2608)
- `total_size` method for `qr_code` widget. [#2606](https://github.com/iced-rs/iced/pull/2606)
- `PartialEq` implementations for widget styles. [#2637](https://github.com/iced-rs/iced/pull/2637)
- `Send` marker to `iced_wgpu::Renderer` by using `Arc` in caches. [#2692](https://github.com/iced-rs/iced/pull/2692)
- Disabled `Status` for `scrollbar` widget. [#2585](https://github.com/iced-rs/iced/pull/2585)
- `warning` color to `theme::Palette`. [#2607](https://github.com/iced-rs/iced/pull/2607)
- `maximized` and `fullscreen` fields to `window::Settings`. [#2627](https://github.com/iced-rs/iced/pull/2627)
- `window` tasks for controlling sizes and resize increments. [#2633](https://github.com/iced-rs/iced/pull/2633)
- `window` task for drag resizing. [#2642](https://github.com/iced-rs/iced/pull/2642)
- Helper functions for alignment to `widget` module. [#2746](https://github.com/iced-rs/iced/pull/2746)
- `time::repeat` subscription. [#2747](https://github.com/iced-rs/iced/pull/2747)
- Vertical support for `progress_bar`. [#2748](https://github.com/iced-rs/iced/pull/2748)
- `scale` support for `image` widget. [#2755](https://github.com/iced-rs/iced/pull/2755)
- `LineEnding` support for `text_editor`. [#2759](https://github.com/iced-rs/iced/pull/2759)
- `Mul<Transformation>` implementation for `mouse::Cursor` and `mouse::Click`. [#2758](https://github.com/iced-rs/iced/pull/2758)
- `animation` module support for Wasm target. [#2764](https://github.com/iced-rs/iced/pull/2764)
- Flake for a dev shell in `DEPENDENCIES`. [#2769](https://github.com/iced-rs/iced/pull/2769)
- `unfocus` widget operation. [#2804](https://github.com/iced-rs/iced/pull/2804)
- `sipper` support and some QoL. [#2805](https://github.com/iced-rs/iced/pull/2805)
- Variable text size for preedit IME window. [#2790](https://github.com/iced-rs/iced/pull/2790)
- `is_focused` widget operation. [#2812](https://github.com/iced-rs/iced/pull/2812)
- Notification of `window` pre-presentation to windowing system. [#2849](https://github.com/iced-rs/iced/pull/2849)
- Customizable vertical `spacing` for wrapped rows. [#2852](https://github.com/iced-rs/iced/pull/2852)
- Indent and unindent actions for `text_editor`. [#2901](https://github.com/iced-rs/iced/pull/2901)
- Floating Images. [#2903](https://github.com/iced-rs/iced/pull/2903)
- `min_size` method to `PaneGrid`. [#2911](https://github.com/iced-rs/iced/pull/2911)
- Generic key for `sensor` widget. [#2944](https://github.com/iced-rs/iced/pull/2944)
- `Debug` implementation for `Task`. [#2955](https://github.com/iced-rs/iced/pull/2955)
- `draw_with_bounds` method to `canvas::Cache`. [#3035](https://github.com/iced-rs/iced/pull/3035)
- Synchronous `Task` Execution and `RedrawRequested` Consistency. [#3084](https://github.com/iced-rs/iced/pull/3084)
- `id` method to `text_editor`. [#2653](https://github.com/iced-rs/iced/pull/2653)
- `horizontal` and `vertical` methods to `Padding`. [#2655](https://github.com/iced-rs/iced/pull/2655)
- `is_focused` selector and `find` / `find_all` operations. [#2664](https://github.com/iced-rs/iced/pull/2664)
- `push` and `into_options` methods to `combo_box::State`. [#2684](https://github.com/iced-rs/iced/pull/2684)
- `Hidden` variant to `mouse::Interaction`. [#2685](https://github.com/iced-rs/iced/pull/2685)
- `menu_height` method to `pick_list` and `combo_box` widgets. [#2699](https://github.com/iced-rs/iced/pull/2699)
- `text_color` to `toggler::Style`. [#2707](https://github.com/iced-rs/iced/pull/2707)
- `text_shaping` method to `combo_box` widget. [#2714](https://github.com/iced-rs/iced/pull/2714)
- `transparent` field for `window::Settings`. [#2728](https://github.com/iced-rs/iced/pull/2728)
- `closeable` and `minimizable` fields to `window::Settings`. [#2735](https://github.com/iced-rs/iced/pull/2735)
- `window::monitor_size` task. [#2754](https://github.com/iced-rs/iced/pull/2754)
- Division operation for `Size` and `Vector`. [#2767](https://github.com/iced-rs/iced/pull/2767)
- `hidden` method to `scrollable` widget. [#2775](https://github.com/iced-rs/iced/pull/2775)
- Support for macOS-specific key shortcuts with `Control` modifier. [#2801](https://github.com/iced-rs/iced/pull/2801)
- Additional variants to `mouse::Interaction`. [#2815](https://github.com/iced-rs/iced/pull/2815)
- `vsync` field to `window::Settings`. [#2837](https://github.com/iced-rs/iced/pull/2837)
- `wgpu-bare` feature flag to disable default `wgpu` features. [#2828](https://github.com/iced-rs/iced/pull/2828)
- `ratio` method for `Size`. [#2861](https://github.com/iced-rs/iced/pull/2861)
- Support for `⌘ + Backspace` and `⌘ + Delete` macOS shortcuts. [#2862](https://github.com/iced-rs/iced/pull/2862)
- Expandable selection-by-word after double click in text editors. [#2865](https://github.com/iced-rs/iced/pull/2865)
- `x11` and `wayland` feature flags. [#2869](https://github.com/iced-rs/iced/pull/2869)
- `label` method for `checkbox` widget. [#2873](https://github.com/iced-rs/iced/pull/2873)
- `shader::Pipeline` trait for easier `wgpu` resource management. [#2876](https://github.com/iced-rs/iced/pull/2876)
- `select_range` widget operation. [#2890](https://github.com/iced-rs/iced/pull/2890)
- `grid!` macro helper. [#2904](https://github.com/iced-rs/iced/pull/2904)
- `warning` style for `container` widget. [#2912](https://github.com/iced-rs/iced/pull/2912)
- Current toggle state to `toggler::Status::Disabled`. [#2908](https://github.com/iced-rs/iced/pull/2908)
- Cursor size awareness for input methods. [#2918](https://github.com/iced-rs/iced/pull/2918)
- `allow_automatic_tabbing` task to `runtime::window`. [#2933](https://github.com/iced-rs/iced/pull/2933)
- `FromStr` and `Display` implementations for `Color`. [#2937](https://github.com/iced-rs/iced/pull/2937)
- `text::Renderer` trait in `iced_graphics` with `fill_raw` method. [#2958](https://github.com/iced-rs/iced/pull/2958)
- `font_maybe` helper for `text` widget. [#2988](https://github.com/iced-rs/iced/pull/2988)
- `filter_map` method to `Subscription`. [#2981](https://github.com/iced-rs/iced/pull/2981)
- `repeat` field to `keyboard::Event::KeyPressed`. [#2991](https://github.com/iced-rs/iced/pull/2991)
- Additional settings to control the fonts used for `markdown` rendering. [#2999](https://github.com/iced-rs/iced/pull/2999)
- `Rescaled` variant to `window::Event`. [#3001](https://github.com/iced-rs/iced/pull/3001)
- Environment variable to define `beacon` server listen address. [#3003](https://github.com/iced-rs/iced/pull/3003)
- `push_under` method to `stack` widget. [#3010](https://github.com/iced-rs/iced/pull/3010)
- `NONE` constant to `keyboard::Modifiers`. [#3037](https://github.com/iced-rs/iced/pull/3037)
- `shadow` field to `overlay::menu::Style`. [#3049](https://github.com/iced-rs/iced/pull/3049)
- `draw_mesh_cache` method in `mesh::Renderer` trait. [#3070](https://github.com/iced-rs/iced/pull/3070)
- Efficient `is_empty` method for `text_editor::Content`. [#3117](https://github.com/iced-rs/iced/pull/3117)
- `*Assign` implementations for `Point` and `Vector`. [#3131](https://github.com/iced-rs/iced/pull/3131)
- Support `Background` instead of `Color` styling for `scrollable`. [#3127](https://github.com/iced-rs/iced/pull/3127)
- `CornerPreference` window setting for Windows. [#3128](https://github.com/iced-rs/iced/pull/3128)
- `move_to` method for `Editor` API. [#3125](https://github.com/iced-rs/iced/pull/3125)
- `Background` and `padding_ratio` support for `toggler` styling. [#3129](https://github.com/iced-rs/iced/pull/3129)
- More syntaxes for `iced_highlighter`. [#2822](https://github.com/iced-rs/iced/pull/2822)

### Changed
- Replace `Rc` with `Arc` for `markdown` caching. [#2599](https://github.com/iced-rs/iced/pull/2599)
- Improved `button::Catalog` and `Style` documentation. [#2590](https://github.com/iced-rs/iced/pull/2590)
- Improved `clock` example to display ticks and numbers. [#2644](https://github.com/iced-rs/iced/pull/2644)
- Derived `PartialEq` and `Eq` for `mouse::click::Kind`. [#2741](https://github.com/iced-rs/iced/pull/2741)
- Marked `Color::from_rgb8` and `Color::from_rgba8` as const. [#2749](https://github.com/iced-rs/iced/pull/2749)
- Replaced unmaintained `directories-next` crate with `directories`. [#2761](https://github.com/iced-rs/iced/pull/2761)
- Changed `Widget::update` to take `Event` by reference. [#2781](https://github.com/iced-rs/iced/pull/2781)
- Improved `gallery` example with blurhash previews. [#2796](https://github.com/iced-rs/iced/pull/2796)
- Replaced `wasm-timer` with `wasmtimer`. [#2780](https://github.com/iced-rs/iced/pull/2780)
- Tweaked `Palette` Generation. [#2811](https://github.com/iced-rs/iced/pull/2811)
- Relaxed `Task::perform` bound from `Fn` to `FnOnce`. [#2827](https://github.com/iced-rs/iced/pull/2827)
- Improved `quad` shader to use a single SDF in `iced_wgpu`. [#2967](https://github.com/iced-rs/iced/pull/2967)
- Leveraged `Limits::min` directly in `scrollable::layout`. [#3004](https://github.com/iced-rs/iced/pull/3004)
- Overhauled `theme::Palette` generation by leveraging `Oklch`. [#3028](https://github.com/iced-rs/iced/pull/3028)
- Mutable `Widget` Methods. [#3038](https://github.com/iced-rs/iced/pull/3038)
- Prioritized `Shrink` over `Fill` in `layout` logic. [#3045](https://github.com/iced-rs/iced/pull/3045)
- Replaced `format!` with `concat!` for string literals. [#2695](https://github.com/iced-rs/iced/pull/2695)
- Replaced `window::run_with_handle` with a more powerful `window::run`. [#2718](https://github.com/iced-rs/iced/pull/2718)
- Made color helpers in `palette` module public. [#2771](https://github.com/iced-rs/iced/pull/2771)
- Changed default `PowerPreference` to `HighPerformance` in `iced_wgpu`. [#2813](https://github.com/iced-rs/iced/pull/2813)
- Made `button::DEFAULT_PADDING` public. [#2858](https://github.com/iced-rs/iced/pull/2858)
- Replaced `Url` parsing in `markdown` widget with `String` URIs. [#2992](https://github.com/iced-rs/iced/pull/2992)
- Improved alignment docs of `container`. [#2871](https://github.com/iced-rs/iced/pull/2871)
- Made `input_method` module public. [#2897](https://github.com/iced-rs/iced/pull/2897)
- `iced` logo to built-in icons font. [#2902](https://github.com/iced-rs/iced/pull/2902)
- Made `Layout::children` return an `ExactSizeIterator`. [#2915](https://github.com/iced-rs/iced/pull/2915)
- Enabled `fancy-regex` instead of `onig` for `syntect`. [#2932](https://github.com/iced-rs/iced/pull/2932)
- Added `warning` status to `toast` example. [#2936](https://github.com/iced-rs/iced/pull/2936)
- Improved `scroll_to` and `snap_to` to allow operating on a single axis. [#2994](https://github.com/iced-rs/iced/pull/2994)
- Disabled `png-format` feature from `iced_tiny_skia`. [#3043](https://github.com/iced-rs/iced/pull/3043)
- Unified `keyboard` subscriptions into a single `listen` subscription. [#3135](https://github.com/iced-rs/iced/pull/3135)
- Updated to Rust 2024. [#2809](https://github.com/iced-rs/iced/pull/2809)
- Updated `wgpu` to `22.0`. [#2510](https://github.com/iced-rs/iced/pull/2510)
- Updated `wgpu` to `23.0`. [#2663](https://github.com/iced-rs/iced/pull/2663)
- Updated `wgpu` to `24.0`. [#2832](https://github.com/iced-rs/iced/pull/2832)
- Updated `wgpu` to `26.0`. [#3019](https://github.com/iced-rs/iced/pull/3019)
- Updated `wgpu` to `27.0`. [#3097](https://github.com/iced-rs/iced/pull/3097)
- Updated `image` to `0.25`. [#2716](https://github.com/iced-rs/iced/pull/2716)
- Updated `cosmic-text` to `0.13`. [#2834](https://github.com/iced-rs/iced/pull/2834)
- Updated `cosmic-text` to `0.14`. [#2880](https://github.com/iced-rs/iced/pull/2880)
- Updated `cosmic-text` to `0.15`. [#3098](https://github.com/iced-rs/iced/pull/3098)
- Updated `resvg` to `0.45`. [#2846](https://github.com/iced-rs/iced/pull/2846)
- Updated `wasmtimer` to `0.4.2`. [#3012](https://github.com/iced-rs/iced/pull/3012)
- Updated `dark-light` to `2.0`. [#2724](https://github.com/iced-rs/iced/pull/2724)
- Updated `openssl` to `0.10.70`. [#2783](https://github.com/iced-rs/iced/pull/2783)
- Updated our `winit` fork with `0.30.8` fixes. [#2737](https://github.com/iced-rs/iced/pull/2737)

### Fixed
- Slow `wgpu` documentation. [#2593](https://github.com/iced-rs/iced/pull/2593)
- Documentation for `open_events`. [#2594](https://github.com/iced-rs/iced/pull/2594)
- Layout for wrapped `row` with `spacing`. [#2596](https://github.com/iced-rs/iced/pull/2596)
- Flex layout of `Fill` elements in a `Shrink` cross axis. [#2598](https://github.com/iced-rs/iced/pull/2598)
- Incorrect triangle mesh counting in `wgpu`. [#2601](https://github.com/iced-rs/iced/pull/2601)
- Dropped images and meshes when pasting `Frame`. [#2605](https://github.com/iced-rs/iced/pull/2605)
- `loading_spinners` example skipping part of the animation cycle. [#2617](https://github.com/iced-rs/iced/pull/2617)
- Window `File*` events not marked as unsupported for Wayland. [#2615](https://github.com/iced-rs/iced/pull/2615)
- Coupling of `markdown::view` iterator lifetime with resulting `Element`. [#2623](https://github.com/iced-rs/iced/pull/2623)
- Delete key not working in `text_editor` widget. [#2632](https://github.com/iced-rs/iced/pull/2632)
- Consecutive clicks triggering independently of distance. [#2639](https://github.com/iced-rs/iced/pull/2639)
- `pane_grid` losing continuity when adding or removing panes. [#2628](https://github.com/iced-rs/iced/pull/2628)
- Synthetic keyboard events not being discarded. [#2649](https://github.com/iced-rs/iced/pull/2649)
- `sort_by` without total ordering in `tiny-skia` damage tracking. [#2651](https://github.com/iced-rs/iced/pull/2651)
- Outdated docs of `Scrollable::with_direction` and `direction`. [#2668](https://github.com/iced-rs/iced/pull/2668)
- `button` calling its `on_press` handler unnecessarily. [#2683](https://github.com/iced-rs/iced/pull/2683)
- `system_information` example getting stuck at boot. [#2681](https://github.com/iced-rs/iced/pull/2681)
- `tooltip` widget not redrawing when hovered. [#2675](https://github.com/iced-rs/iced/pull/2675)
- `pane_grid::DragEvent::Canceled` not emitted within deadband. [#2691](https://github.com/iced-rs/iced/pull/2691)
- Inconsistent positions in window-related operations. [#2688](https://github.com/iced-rs/iced/pull/2688)
- `text::Wrapping` not being applied to `Paragraph`. [#2723](https://github.com/iced-rs/iced/pull/2723)
- Broken nested `markdown` lists without empty line. [#2641](https://github.com/iced-rs/iced/pull/2641)
- Unnecessary cast in `the_matrix` example. [#2731](https://github.com/iced-rs/iced/pull/2731)
- Incorrect layer counting in `iced_wgpu`. [#2701](https://github.com/iced-rs/iced/pull/2701)
- `Image` not respecting `viewport` bounds. [#2752](https://github.com/iced-rs/iced/pull/2752)
- Attempting to draw empty meshes in `iced_wgpu`. [#2782](https://github.com/iced-rs/iced/pull/2782)
- Input placeholder text not clearing when IME is activated. [#2785](https://github.com/iced-rs/iced/pull/2785)
- Missing redraw request in `image::Viewer`. [#2795](https://github.com/iced-rs/iced/pull/2795)
- Wrong position of preedit text on scrolled content. [#2798](https://github.com/iced-rs/iced/pull/2798)
- Wrong initial candidate position for IME. [#2793](https://github.com/iced-rs/iced/pull/2793)
- Text spans in IME preedit not being properly cached. [#2806](https://github.com/iced-rs/iced/pull/2806)
- `cpu_brand` in `system_information` always being empty. [#2797](https://github.com/iced-rs/iced/pull/2797)
- Horizontal text alignment being ignored on multi-line text. [#2835](https://github.com/iced-rs/iced/pull/2835)
- Missing redraw request in `mouse_area` when hovered. [#2845](https://github.com/iced-rs/iced/pull/2845)
- `futures-executor` being pulled even when it's not the default executor. [#2841](https://github.com/iced-rs/iced/pull/2841)
- WebGPU failing to boot in Chromium. [#2686](https://github.com/iced-rs/iced/pull/2686)
- Crash when using WebGL due to wrong binding alignment. [#2883](https://github.com/iced-rs/iced/pull/2883)
- Wrong calculation of rows in `grid` widget when evenly distributed. [#2896](https://github.com/iced-rs/iced/pull/2896)
- Panic in `combo_box` due to cleared children during `diff`. [#2905](https://github.com/iced-rs/iced/pull/2905)
- OpenGL backend in `wgpu` interpreting atlas texture as cube map instead of texture array. [#2919](https://github.com/iced-rs/iced/pull/2919)
- `quad` shader blending without pre-multiplication. [#2925](https://github.com/iced-rs/iced/pull/2925)
- Inconsistent primitive pixel snapping in `iced_wgpu`. [#2962](https://github.com/iced-rs/iced/pull/2962)
- Inconsistent `Rectangle::is_within` implementation. [#2966](https://github.com/iced-rs/iced/pull/2966)
- Text damage calculation in `iced_tiny_skia`. [#2964](https://github.com/iced-rs/iced/pull/2964)
- Leftover `title` mention in documentation. [#2972](https://github.com/iced-rs/iced/pull/2972)
- Text bounds cutoff in `iced_wgpu`. [#2975](https://github.com/iced-rs/iced/pull/2975)
- Rectangle vertices not being snapped to the pixel grid independently. [#2768](https://github.com/iced-rs/iced/pull/2768)
- Lints for Rust 1.89. [#3030](https://github.com/iced-rs/iced/pull/3030)
- `debug` builds on macOS Tahoe. [#3056](https://github.com/iced-rs/iced/pull/3056)
- Typo in documentation comment for `filter_map`. [#3052](https://github.com/iced-rs/iced/pull/3052)
- `container::Style` not respecting `crisp` feature. [#3112](https://github.com/iced-rs/iced/pull/3112)
- Incorrect padding in `text_editor`. [#3115](https://github.com/iced-rs/iced/pull/3115)
- Outdated documentation of `Widget::mouse_interaction`. [#2696](https://github.com/iced-rs/iced/pull/2696)
- Incorrect render pass viewport in `custom_shader` example. [#2738](https://github.com/iced-rs/iced/pull/2738)
- Capturing `ButtonReleased` event inside `image::Viewer`. [#2744](https://github.com/iced-rs/iced/pull/2744)
- Incomplete docs for `on_link_click` in `rich_text`. [#2803](https://github.com/iced-rs/iced/pull/2803)
- Stale syntax highlighting on `text_editor` after theme changes. [#2818](https://github.com/iced-rs/iced/pull/2818)
- Wrong background color for `window::Preedit` on translucent themes. [#2819](https://github.com/iced-rs/iced/pull/2819)
- Panic on Chromium-like browsers when canvas initial size is `(0, 0)`. [#2829](https://github.com/iced-rs/iced/pull/2829)
- Outdated dev shell templates. [#2840](https://github.com/iced-rs/iced/pull/2840)
- Missing `derive` feature for `serde` dependency. [#2854](https://github.com/iced-rs/iced/pull/2854)
- `bezier_tool` listed as an example in the `Widget` trait docs. [#2867](https://github.com/iced-rs/iced/pull/2867)
- Incomplete doc comment of `Length::is_fill`. [#2892](https://github.com/iced-rs/iced/pull/2892)
- `scrollable` touch scrolling when out of bounds. [#2906](https://github.com/iced-rs/iced/pull/2906)
- `Element::explain` being hidden by multi-layer widgets. [#2913](https://github.com/iced-rs/iced/pull/2913)
- Missing `Shell::request_redraw` on `component`. [#2930](https://github.com/iced-rs/iced/pull/2930)
- Text clipping in `iced_tiny_skia`. [#2929](https://github.com/iced-rs/iced/pull/2929)
- Inconsistent naming of `tree` parameter in `Widget` trait. [#2950](https://github.com/iced-rs/iced/pull/2950)
- `text_editor` syntax highlighting not updating on paste. [#2947](https://github.com/iced-rs/iced/pull/2947)
- `svg` scaling in `iced_tiny_skia`. [#2954](https://github.com/iced-rs/iced/pull/2954)
- Stroke bounds calculation and clip transformations in `iced_tiny_skia`. [#2882](https://github.com/iced-rs/iced/pull/2882)
- Artifacts when drawing small arcs in `canvas` widget. [#2959](https://github.com/iced-rs/iced/pull/2959)
- Path not being closed in `Path::circle`. [#2979](https://github.com/iced-rs/iced/pull/2979)
- Incorrect transformation of cached primitives in `iced_tiny_skia`. [#2977](https://github.com/iced-rs/iced/pull/2977)
- Panic when drawing empty image in `iced_tiny_skia`. [#2986](https://github.com/iced-rs/iced/pull/2986)
- Incorrect mapping of navigation keys on higher keyboard layers. [#3007](https://github.com/iced-rs/iced/pull/3007)
- `Status` of `svg` widget not being updated on cursor movement. [#3009](https://github.com/iced-rs/iced/pull/3009)
- `hover` widget ignoring events in certain conditions. [#3015](https://github.com/iced-rs/iced/pull/3015)
- OpenGL backend in `iced_wgpu` choosing wrong texture format in `wgpu::image::atlas`. [#3016](https://github.com/iced-rs/iced/pull/3016)
- Missing redraw request in `geometry` example. [#3020](https://github.com/iced-rs/iced/pull/3020)
- Buffer presentation logic in `iced_tiny_skia`. [#3032](https://github.com/iced-rs/iced/pull/3032)
- `combo_box` text not getting cleared on selection. [#3063](https://github.com/iced-rs/iced/pull/3063)
- `wgpu` surface not being reconfigured on `SurfaceError::Lost` or `Outdated`. [#3067](https://github.com/iced-rs/iced/pull/3067)
- Incorrect cursor for `slider` widget on Windows . [#3068](https://github.com/iced-rs/iced/pull/3068)
- `Paragraph::hit_span` returning false positives at end of content. [#3072](https://github.com/iced-rs/iced/pull/3072)
- Incorrect `Limits::loose` documentation. [#3116](https://github.com/iced-rs/iced/pull/3116)
- Missing semicolon triggering a `clippy` lint. [#3118](https://github.com/iced-rs/iced/pull/3118)
- `iced_tiny_skia` using a `Window` instead of a `Display` handle for `softbuffer::Context` creation. [#3090](https://github.com/iced-rs/iced/pull/3090)
- Missing `fn operate` in `tooltip` widget. [#3132](https://github.com/iced-rs/iced/pull/3132)
- Panic when rendering problematic `svg`. [#3123](https://github.com/iced-rs/iced/pull/3123)
- Hotkey combinations not working on non-latin keyboard layouts. [#3134](https://github.com/iced-rs/iced/pull/3134)
- `keyboard::listen` reporting captured key events. [#3136](https://github.com/iced-rs/iced/pull/3136)

### Removed
- `is_over` method in `Overlay` trait. [#2921](https://github.com/iced-rs/iced/pull/2921)
- Short-hand notation support for `color!` macro. [#2592](https://github.com/iced-rs/iced/pull/2592)
- `surface` argument of `Compositor::screenshot`. [#2672](https://github.com/iced-rs/iced/pull/2672)
- `once_cell` dependency. [#2626](https://github.com/iced-rs/iced/pull/2626)
- `winapi` dependency. [#2760](https://github.com/iced-rs/iced/pull/2760)
- `palette` dependency. [#2839](https://github.com/iced-rs/iced/pull/2839)

Many thanks to...
- @edwloef
- @rhysd
- @DKolter
- @pml68
- @andymandias
- @dtzxporter
- @tarkah
- @tvolk131
- @alex-ds13
- @B0ney
- @bbb651
- @JL710
- @kenz-gelsoft
- @mfreeborn
- @mtkennerly
- @watsaig
- @13r0ck
- @airstrike
- @bungoboingo
- @EmmanuelDodoo
- @karolisr
- @Redhawk18
- @Remmirad
- @semiversus
- @Ultrasquid9
- @xosxos
- @Zarthus
- @7h0ma5
- @7sDream
- @Adam-Ladd
- @AMS21
- @Atreyagaurav
- @AustinEvansWX
- @Azorlogh
- @berserkware
- @biglizards
- @boondocklabs
- @bradysimon
- @camspiers
- @chrismanning
- @codewing
- @csmoe
- @davehorner
- @DavidAguilo
- @dcz-self
- @dejang
- @dependabot[bot]
- @EleDiaz
- @ellieplayswow
- @Exidex
- @Fili-pk
- @flakes
- @Gobbel2000
- @GyulyVGC
- @hammerlink
- @hydra
- @ibaryshnikov
- @ids1024
- @iMohmmedSA
- @Integral-Tech
- @inthehack
- @jakobhellermann
- @janTatesa
- @jbirnick
- @jcdickinson
- @Jinderamarak
- @jsatka
- @kbjr
- @kgday
- @kiedtl
- @Konsl
- @Koranir
- @kosayoda
- @Krahos
- @l-const
- @l4l
- @laycookie
- @leo030303
- @Leonie-Theobald
- @libkurisu
- @lmaxyz
- @mariinkys
- @max-privatevoid
- @MichelleGranat
- @misaka10987
- @mytdragon
- @njust
- @nrjais
- @nz366
- @OpenSauce
- @Ottatop
- @rhogenson
- @rizzen-yazston
- @rotmh
- @Rudxain
- @ryco117
- @Seppel3210
- @sgued
- @sopvop
- @T-256
- @tafia
- @thorn132
- @tigerros
- @tsuza
- @vincenthz
- @will-lynas

## [0.13.1] - 2024-09-19
### Added
- Some `From` trait implementations for `text_input::Id`. [#2582](https://github.com/iced-rs/iced/pull/2582)
- Custom `Executor` support for `Application` and `Daemon`. [#2580](https://github.com/iced-rs/iced/pull/2580)
- `rust-version` metadata to `Cargo.toml`. [#2579](https://github.com/iced-rs/iced/pull/2579)
- Widget examples to API reference. [#2587](https://github.com/iced-rs/iced/pull/2587)

### Fixed
- Inverted scrolling direction with trackpad in `scrollable`. [#2583](https://github.com/iced-rs/iced/pull/2583)
- `scrollable` transactions when `on_scroll` is not set. [#2584](https://github.com/iced-rs/iced/pull/2584)
- Incorrect text color styling in `text_editor` widget. [#2586](https://github.com/iced-rs/iced/pull/2586)

Many thanks to...
- @dcampbell24
- @lufte
- @mtkennerly

## [0.13.0] - 2024-09-18
### Added
- Introductory chapters to the [official guide book](https://book.iced.rs/).
- [Pocket guide](https://docs.rs/iced/0.13.0/iced/#the-pocket-guide) in API reference.
- `Program` API. [#2331](https://github.com/iced-rs/iced/pull/2331)
- `Task` API. [#2463](https://github.com/iced-rs/iced/pull/2463)
- `Daemon` API and Shell Runtime Unification. [#2469](https://github.com/iced-rs/iced/pull/2469)
- `rich_text` and `markdown` widgets. [#2508](https://github.com/iced-rs/iced/pull/2508)
- `stack` widget. [#2405](https://github.com/iced-rs/iced/pull/2405)
- `hover` widget. [#2408](https://github.com/iced-rs/iced/pull/2408)
- `row::Wrapping` widget. [#2539](https://github.com/iced-rs/iced/pull/2539)
- `text` macro helper. [#2338](https://github.com/iced-rs/iced/pull/2338)
- `text::Wrapping` support. [#2279](https://github.com/iced-rs/iced/pull/2279)
- Functional widget styling. [#2312](https://github.com/iced-rs/iced/pull/2312)
- Closure-based widget styling. [#2326](https://github.com/iced-rs/iced/pull/2326)
- Class-based Theming. [#2350](https://github.com/iced-rs/iced/pull/2350)
- Type-Driven Renderer Fallback. [#2351](https://github.com/iced-rs/iced/pull/2351)
- Background styling to `rich_text` widget. [#2516](https://github.com/iced-rs/iced/pull/2516)
- Underline support for `rich_text`. [#2526](https://github.com/iced-rs/iced/pull/2526)
- Strikethrough support for `rich_text`. [#2528](https://github.com/iced-rs/iced/pull/2528)
- Abortable `Task`. [#2496](https://github.com/iced-rs/iced/pull/2496)
- `abort_on_drop` to `task::Handle`. [#2503](https://github.com/iced-rs/iced/pull/2503)
- `Ferra` theme. [#2329](https://github.com/iced-rs/iced/pull/2329)
- `auto-detect-theme` feature. [#2343](https://github.com/iced-rs/iced/pull/2343)
- Custom key binding support for `text_editor`. [#2522](https://github.com/iced-rs/iced/pull/2522)
- `align_x` for `text_input` widget. [#2535](https://github.com/iced-rs/iced/pull/2535)
- `center` widget helper. [#2423](https://github.com/iced-rs/iced/pull/2423)
- Rotation support for `image` and `svg` widgets. [#2334](https://github.com/iced-rs/iced/pull/2334)
- Dynamic `opacity` support for `image` and `svg`. [#2424](https://github.com/iced-rs/iced/pull/2424)
- Scroll transactions for `scrollable` widget. [#2401](https://github.com/iced-rs/iced/pull/2401)
- `physical_key` and `modified_key` to `keyboard::Event`. [#2576](https://github.com/iced-rs/iced/pull/2576)
- `fetch_position` command in `window` module. [#2280](https://github.com/iced-rs/iced/pull/2280)
- `filter_method` property for `image::Viewer` widget. [#2324](https://github.com/iced-rs/iced/pull/2324)
- Support for pre-multiplied alpha `wgpu` composite mode. [#2341](https://github.com/iced-rs/iced/pull/2341)
- `text_size` and `line_height` properties for `text_editor` widget. [#2358](https://github.com/iced-rs/iced/pull/2358)
- `is_focused` method for `text_editor::State`. [#2386](https://github.com/iced-rs/iced/pull/2386)
- `canvas::Cache` Grouping. [#2415](https://github.com/iced-rs/iced/pull/2415)
- `ICED_PRESENT_MODE` env var to pick a `wgpu::PresentMode`. [#2428](https://github.com/iced-rs/iced/pull/2428)
- `SpecificWith` variant to `window::Position`. [#2435](https://github.com/iced-rs/iced/pull/2435)
- `scale_factor` field to `window::Screenshot`. [#2449](https://github.com/iced-rs/iced/pull/2449)
- Styling support for `overlay::Menu` of `pick_list` widget. [#2457](https://github.com/iced-rs/iced/pull/2457)
- `window::Id` in `Event` subscriptions. [#2456](https://github.com/iced-rs/iced/pull/2456)
- `FromIterator` implementation for `row` and `column`. [#2460](https://github.com/iced-rs/iced/pull/2460)
- `content_fit` for `image::viewer` widget. [#2330](https://github.com/iced-rs/iced/pull/2330)
- `Display` implementation for `Radians`. [#2446](https://github.com/iced-rs/iced/pull/2446)
- Helper methods for `window::Settings` in `Application`. [#2470](https://github.com/iced-rs/iced/pull/2470)
- `Copy` implementation for `canvas::Fill` and `canvas::Stroke`. [#2475](https://github.com/iced-rs/iced/pull/2475)
- Clarification of `Border` alignment for `Quad`. [#2485](https://github.com/iced-rs/iced/pull/2485)
- "Select All" functionality on `Ctrl+A` to `text_editor`. [#2321](https://github.com/iced-rs/iced/pull/2321)
- `stream::try_channel` helper. [#2497](https://github.com/iced-rs/iced/pull/2497)
- `iced` widget helper to display the iced logo :comet:. [#2498](https://github.com/iced-rs/iced/pull/2498)
- `align_x` and `align_y` helpers to `scrollable`. [#2499](https://github.com/iced-rs/iced/pull/2499)
- Built-in text styles for each `Palette` color. [#2500](https://github.com/iced-rs/iced/pull/2500)
- Embedded `Scrollbar` support for `scrollable`. [#2269](https://github.com/iced-rs/iced/pull/2269)
- `on_press_with` method for `button`. [#2502](https://github.com/iced-rs/iced/pull/2502)
- `resize_events` subscription to `window` module. [#2505](https://github.com/iced-rs/iced/pull/2505)
- `Link` support to `rich_text` widget. [#2512](https://github.com/iced-rs/iced/pull/2512)
- `image` and `svg` support for `canvas` widget. [#2537](https://github.com/iced-rs/iced/pull/2537)
- `Compact` variant for `pane_grid::Controls`. [#2555](https://github.com/iced-rs/iced/pull/2555)
- `image-without-codecs` feature flag. [#2244](https://github.com/iced-rs/iced/pull/2244)
- `container::background` styling helper. [#2261](https://github.com/iced-rs/iced/pull/2261)
- `undecorated_shadow` window setting for Windows. [#2285](https://github.com/iced-rs/iced/pull/2285)
- Tasks for setting mouse passthrough. [#2284](https://github.com/iced-rs/iced/pull/2284)
- `*_maybe` helpers for `text_input` widget. [#2390](https://github.com/iced-rs/iced/pull/2390)
- Wasm support for `download_progress` example. [#2419](https://github.com/iced-rs/iced/pull/2419)
- `scrollable::scroll_by` widget operation. [#2436](https://github.com/iced-rs/iced/pull/2436)
- Enhancements to `slider` widget styling. [#2444](https://github.com/iced-rs/iced/pull/2444)
- `on_scroll` handler to `mouse_area` widget. [#2450](https://github.com/iced-rs/iced/pull/2450)
- `stroke_rectangle` method to `canvas::Frame`. [#2473](https://github.com/iced-rs/iced/pull/2473)
- `override_redirect` setting for X11 windows. [#2476](https://github.com/iced-rs/iced/pull/2476)
- Disabled state support for `toggler` widget. [#2478](https://github.com/iced-rs/iced/pull/2478)
- `Color::parse` helper for parsing color strings. [#2486](https://github.com/iced-rs/iced/pull/2486)
- `rounded_rectangle` method to `canvas::Path`. [#2491](https://github.com/iced-rs/iced/pull/2491)
- `width` method to `text_editor` widget. [#2513](https://github.com/iced-rs/iced/pull/2513)
- `on_open` handler to `combo_box` widget. [#2534](https://github.com/iced-rs/iced/pull/2534)
- Additional `mouse::Interaction` cursors. [#2551](https://github.com/iced-rs/iced/pull/2551)
- Scroll wheel handling in `slider` widget. [#2565](https://github.com/iced-rs/iced/pull/2565)

### Changed
- Use a `StagingBelt` in `iced_wgpu` for regular buffer uploads. [#2357](https://github.com/iced-rs/iced/pull/2357)
- Use generic `Content` in `Text` to avoid reallocation in `fill_text`. [#2360](https://github.com/iced-rs/iced/pull/2360)
- Use `Iterator::size_hint` to initialize `Column` and `Row` capacity. [#2362](https://github.com/iced-rs/iced/pull/2362)
- Specialize `widget::text` helper. [#2363](https://github.com/iced-rs/iced/pull/2363)
- Use built-in `[lints]` table in `Cargo.toml`. [#2377](https://github.com/iced-rs/iced/pull/2377)
- Target `#iced` container by default on Wasm. [#2342](https://github.com/iced-rs/iced/pull/2342)
- Improved architecture for `iced_wgpu` and `iced_tiny_skia`. [#2382](https://github.com/iced-rs/iced/pull/2382)
- Make image `Cache` eviction strategy less aggressive in `iced_wgpu`. [#2403](https://github.com/iced-rs/iced/pull/2403)
- Retain caches in `iced_wgpu` as long as `Rc` values are alive. [#2409](https://github.com/iced-rs/iced/pull/2409)
- Use `bytes` crate for `image` widget. [#2356](https://github.com/iced-rs/iced/pull/2356)
- Update `winit` to `0.30`. [#2427](https://github.com/iced-rs/iced/pull/2427)
- Reuse `glyphon::Pipeline` state in `iced_wgpu`. [#2430](https://github.com/iced-rs/iced/pull/2430)
- Ask for explicit `Length` in `center_*` methods. [#2441](https://github.com/iced-rs/iced/pull/2441)
- Hide internal `Task` constructors. [#2492](https://github.com/iced-rs/iced/pull/2492)
- Hide `Subscription` internals. [#2493](https://github.com/iced-rs/iced/pull/2493)
- Improved `view` ergonomics. [#2504](https://github.com/iced-rs/iced/pull/2504)
- Update `cosmic-text` and `resvg`. [#2416](https://github.com/iced-rs/iced/pull/2416)
- Snap `Quad` lines to the pixel grid in `iced_wgpu`. [#2531](https://github.com/iced-rs/iced/pull/2531)
- Update `web-sys` to `0.3.69`. [#2507](https://github.com/iced-rs/iced/pull/2507)
- Allow disabled `TextInput` to still be interacted with. [#2262](https://github.com/iced-rs/iced/pull/2262)
- Enable horizontal scrolling without shift modifier for `srollable` widget. [#2392](https://github.com/iced-rs/iced/pull/2392)
- Add `mouse::Button` to `mouse::Click`. [#2414](https://github.com/iced-rs/iced/pull/2414)
- Notify `scrollable::Viewport` changes. [#2438](https://github.com/iced-rs/iced/pull/2438)
- Improved documentation of `Component` state management. [#2556](https://github.com/iced-rs/iced/pull/2556)

### Fixed
- Fix `block_on` in `iced_wgpu` hanging Wasm builds. [#2313](https://github.com/iced-rs/iced/pull/2313)
- Private `PaneGrid` style fields. [#2316](https://github.com/iced-rs/iced/pull/2316)
- Some documentation typos. [#2317](https://github.com/iced-rs/iced/pull/2317)
- Blurry input caret with non-integral scaling. [#2320](https://github.com/iced-rs/iced/pull/2320)
- Scrollbar stuck in a `scrollable` under some circumstances. [#2322](https://github.com/iced-rs/iced/pull/2322)
- Broken `wgpu` examples link in issue template. [#2327](https://github.com/iced-rs/iced/pull/2327)
- Empty `wgpu` draw calls in `image` pipeline. [#2344](https://github.com/iced-rs/iced/pull/2344)
- Layout invalidation for `Responsive` widget. [#2345](https://github.com/iced-rs/iced/pull/2345)
- Incorrect shadows on quads with rounded corners. [#2354](https://github.com/iced-rs/iced/pull/2354)
- Empty menu overlay on `combo_box`. [#2364](https://github.com/iced-rs/iced/pull/2364)
- Copy / cut vulnerability in a secure `TextInput`. [#2366](https://github.com/iced-rs/iced/pull/2366)
- Inadequate readability / contrast for built-in themes. [#2376](https://github.com/iced-rs/iced/pull/2376)
- Fix `pkg-config` typo in `DEPENDENCIES.md`. [#2379](https://github.com/iced-rs/iced/pull/2379)
- Unbounded memory consumption by `iced_winit::Proxy`. [#2389](https://github.com/iced-rs/iced/pull/2389)
- Typo in `icon::Error` message. [#2393](https://github.com/iced-rs/iced/pull/2393)
- Nested scrollables capturing all scroll events. [#2397](https://github.com/iced-rs/iced/pull/2397)
- Content capturing scrollbar events in a `scrollable`. [#2406](https://github.com/iced-rs/iced/pull/2406)
- Out of bounds caret and overflow when scrolling in `text_editor`. [#2407](https://github.com/iced-rs/iced/pull/2407)
- Missing `derive(Default)` in overview code snippet. [#2412](https://github.com/iced-rs/iced/pull/2412)
- `image::Viewer` triggering grab from outside the widget. [#2420](https://github.com/iced-rs/iced/pull/2420)
- Different windows fighting over shared `image::Cache`. [#2425](https://github.com/iced-rs/iced/pull/2425)
- Images not aligned to the (logical) pixel grid in `iced_wgpu`. [#2440](https://github.com/iced-rs/iced/pull/2440)
- Incorrect local time in `clock` example under Unix systems. [#2421](https://github.com/iced-rs/iced/pull/2421)
- `⌘ + ←` and `⌘ + →` behavior for `text_input` on macOS. [#2315](https://github.com/iced-rs/iced/pull/2315)
- Wayland packages in `DEPENDENCIES.md`. [#2465](https://github.com/iced-rs/iced/pull/2465)
- Typo in documentation. [#2487](https://github.com/iced-rs/iced/pull/2487)
- Extraneous comment in `scrollable` module. [#2488](https://github.com/iced-rs/iced/pull/2488)
- Top layer in `hover` widget hiding when focused. [#2544](https://github.com/iced-rs/iced/pull/2544)
- Out of bounds text in `text_editor` widget. [#2536](https://github.com/iced-rs/iced/pull/2536)
- Segfault on Wayland when closing the app. [#2547](https://github.com/iced-rs/iced/pull/2547)
- `lazy` feature flag sometimes not present in documentation. [#2289](https://github.com/iced-rs/iced/pull/2289)
- Border of `progress_bar` widget being rendered below the active bar. [#2443](https://github.com/iced-rs/iced/pull/2443)
- `radii` typo in `iced_wgpu` shader. [#2484](https://github.com/iced-rs/iced/pull/2484)
- Incorrect priority of `Binding::Delete` in `text_editor`. [#2514](https://github.com/iced-rs/iced/pull/2514)
- Division by zero in `multitouch` example. [#2517](https://github.com/iced-rs/iced/pull/2517)
- Invisible text in `svg` widget. [#2560](https://github.com/iced-rs/iced/pull/2560)
- `wasm32` deployments not displaying anything. [#2574](https://github.com/iced-rs/iced/pull/2574)
- Unnecessary COM initialization on Windows. [#2578](https://github.com/iced-rs/iced/pull/2578)

### Removed
- Unnecessary struct from `download_progress` example. [#2380](https://github.com/iced-rs/iced/pull/2380)
- Out of date comment from `custom_widget` example. [#2549](https://github.com/iced-rs/iced/pull/2549)
- `Clone` bound for `graphics::Cache::clear`. [#2575](https://github.com/iced-rs/iced/pull/2575)

Many thanks to...
- @Aaron-McGuire
- @airstrike
- @alex-ds13
- @alliby
- @Andrew-Schwartz
- @ayeniswe
- @B0ney
- @Bajix
- @blazra
- @Brady-Simon
- @breynard0
- @bungoboingo
- @casperstorm
- @Davidster
- @derezzedex
- @DKolter
- @dtoniolo
- @dtzxporter
- @fenhl
- @Gigas002
- @gintsgints
- @henrispriet
- @IsaacMarovitz
- @ivanceras
- @Jinderamarak
- @JL710
- @jquesada2016
- @JustSoup312
- @kiedtl
- @kmoon2437
- @Koranir
- @lufte
- @LuisLiraC
- @m4rch3n1ng
- @meithecatte
- @mtkennerly
- @myuujiku
- @n1ght-hunter
- @nrjais
- @PgBiel
- @PolyMeilex
- @rustrover
- @ryankopf
- @saihaze
- @shartrec
- @skygrango
- @SolidStateDj
- @sundaram123krishnan
- @tarkah
- @vladh
- @WailAbou
- @wiiznokes
- @woelfman
- @Zaubentrucker

## [0.12.1] - 2024-02-22
### Added
- `extend` and `from_vec` methods for `Column` and `Row`. [#2264](https://github.com/iced-rs/iced/pull/2264)
- `PartialOrd`, `Ord`, and `Hash` implementations for `keyboard::Modifiers`. [#2270](https://github.com/iced-rs/iced/pull/2270)
- `clipboard` module in `advanced` module. [#2272](https://github.com/iced-rs/iced/pull/2272)
- Default `disabled` style for `checkbox` and `hovered` style for `Svg`. [#2273](https://github.com/iced-rs/iced/pull/2273)
- `From<u16>` and `From<i32>` implementations for `border::Radius`. [#2274](https://github.com/iced-rs/iced/pull/2274)
- `size_hint` method for `Component` trait. [#2275](https://github.com/iced-rs/iced/pull/2275)

### Fixed
- Black images when using OpenGL backend in `iced_wgpu`. [#2259](https://github.com/iced-rs/iced/pull/2259)
- Documentation for `horizontal_space` and `vertical_space` helpers. [#2265](https://github.com/iced-rs/iced/pull/2265)
- WebAssembly platform. [#2271](https://github.com/iced-rs/iced/pull/2271)
- Decouple `Key` from `keyboard::Modifiers` and apply them to `text` in `KeyboardInput`. [#2238](https://github.com/iced-rs/iced/pull/2238)
- Text insertion not being prioritized in `TextInput` and `TextEditor`. [#2278](https://github.com/iced-rs/iced/pull/2278)
- `iced_tiny_skia` clipping line strokes. [#2282](https://github.com/iced-rs/iced/pull/2282)

Many thanks to...

- @PolyMeilex
- @rizzen-yazston
- @wash2

## [0.12.0] - 2024-02-15
### Added
- Multi-window support. [#1964](https://github.com/iced-rs/iced/pull/1964)
- `TextEditor` widget (or multi-line text input). [#2123](https://github.com/iced-rs/iced/pull/2123)
- `Shader` widget. [#2085](https://github.com/iced-rs/iced/pull/2085)
- Shadows. [#1882](https://github.com/iced-rs/iced/pull/1882)
- Vectorial text for `Canvas`. [#2204](https://github.com/iced-rs/iced/pull/2204)
- Layout consistency. [#2192](https://github.com/iced-rs/iced/pull/2192)
- Explicit text caching. [#2058](https://github.com/iced-rs/iced/pull/2058)
- Gradients in Oklab color space. [#2055](https://github.com/iced-rs/iced/pull/2055)
- `Themer` widget. [#2209](https://github.com/iced-rs/iced/pull/2209)
- `Transform` primitive. [#2120](https://github.com/iced-rs/iced/pull/2120)
- Cut functionality for `TextEditor`. [#2215](https://github.com/iced-rs/iced/pull/2215)
- Primary clipboard support. [#2240](https://github.com/iced-rs/iced/pull/2240)
- Disabled state for `Checkbox`. [#2109](https://github.com/iced-rs/iced/pull/2109)
- `skip_taskbar` window setting for Windows. [#2211](https://github.com/iced-rs/iced/pull/2211)
- `fetch_maximized` and `fetch_minimized` commands in `window`. [#2189](https://github.com/iced-rs/iced/pull/2189)
- `run_with_handle` command in `window`. [#2200](https://github.com/iced-rs/iced/pull/2200)
- `show_system_menu` command in `window`. [#2243](https://github.com/iced-rs/iced/pull/2243)
- `text_shaping` method for `Tooltip`. [#2172](https://github.com/iced-rs/iced/pull/2172)
- `interaction` method for `MouseArea`. [#2207](https://github.com/iced-rs/iced/pull/2207)
- `hovered` styling for `Svg` widget. [#2163](https://github.com/iced-rs/iced/pull/2163)
- `height` method for `TextEditor`. [#2221](https://github.com/iced-rs/iced/pull/2221)
- Customizable style for `TextEditor`. [#2159](https://github.com/iced-rs/iced/pull/2159)
- Customizable style for `QRCode`. [#2229](https://github.com/iced-rs/iced/pull/2229)
- Border width styling for `Toggler`. [#2219](https://github.com/iced-rs/iced/pull/2219)
- `RawText` variant for `Primitive` in `iced_graphics`. [#2158](https://github.com/iced-rs/iced/pull/2158)
- `Stream` support for `Command`. [#2150](https://github.com/iced-rs/iced/pull/2150)
- Access to bounds/content bounds from a `Scrollable` viewport. [#2072](https://github.com/iced-rs/iced/pull/2072)
- `Frame::scale_nonuniform` method. [#2070](https://github.com/iced-rs/iced/pull/2070)
- `theme::Custom::with_fn` to generate completely custom themes. [#2067](https://github.com/iced-rs/iced/pull/2067)
- `style` attribute for `Font`. [#2041](https://github.com/iced-rs/iced/pull/2041)
- Texture filtering options for `Image`. [#1894](https://github.com/iced-rs/iced/pull/1894)
- `default` and `shift_step` methods for `slider` widgets. [#2100](https://github.com/iced-rs/iced/pull/2100)
- `Custom` variant to `command::Action`. [#2146](https://github.com/iced-rs/iced/pull/2146)
- Mouse movement events for `MouseArea`. [#2147](https://github.com/iced-rs/iced/pull/2147)
- Dracula, Nord, Solarized, and Gruvbox variants for `Theme`. [#2170](https://github.com/iced-rs/iced/pull/2170)
- Catppuccin, Tokyo Night, Kanagawa, Moonfly, Nightfly and Oxocarbon variants for `Theme`. [#2233](https://github.com/iced-rs/iced/pull/2233)
- `From<T> where T: Into<PathBuf>` for `svg::Handle`. [#2235](https://github.com/iced-rs/iced/pull/2235)
- `on_open` and `on_close` handlers for `PickList`. [#2174](https://github.com/iced-rs/iced/pull/2174)
- Support for generic `Element` in `Tooltip`. [#2228](https://github.com/iced-rs/iced/pull/2228)
- Container and `gap` styling for `Scrollable`. [#2239](https://github.com/iced-rs/iced/pull/2239)
- Use `Borrow` for both `options` and `selected` in PickList. [#2251](https://github.com/iced-rs/iced/pull/2251)
- `clip` property for `Container`, `Column`, `Row`, and `Button`. #[2252](https://github.com/iced-rs/iced/pull/2252)

### Changed
- Enable WebGPU backend in `wgpu` by default instead of WebGL. [#2068](https://github.com/iced-rs/iced/pull/2068)
- Update `glyphon` to `0.4`. [#2203](https://github.com/iced-rs/iced/pull/2203)
- Require `Send` on stored pipelines. [#2197](https://github.com/iced-rs/iced/pull/2197)
- Update `wgpu` to `0.19`, `glyphon` to `0.5`, `softbuffer` to `0.4`, `window-clipboard` to `0.4`, and `raw-window-handle` to `0.6`. [#2191](https://github.com/iced-rs/iced/pull/2191)
- Update `winit` to `0.29`. [#2169](https://github.com/iced-rs/iced/pull/2169)
- Provide actual bounds to `Shader` primitives. [#2149](https://github.com/iced-rs/iced/pull/2149)
- Deny warnings in `test` workflow. [#2135](https://github.com/iced-rs/iced/pull/2135)
- Update `wgpu` to `0.18` and `cosmic-text` to `0.10`. [#2122](https://github.com/iced-rs/iced/pull/2122)
- Compute vertex positions in the shader. [#2099](https://github.com/iced-rs/iced/pull/2099)
- Migrate twox-hash -> xxhash-rust and switch to Xxh3 for better performance. [#2080](https://github.com/iced-rs/iced/pull/2080)
- Add `keyboard` subscriptions and rename `subscription::events` to `event::listen`. [#2073](https://github.com/iced-rs/iced/pull/2073)
- Use workspace dependencies and package inheritance. [#2069](https://github.com/iced-rs/iced/pull/2069)
- Update `wgpu` to `0.17`. [#2065](https://github.com/iced-rs/iced/pull/2065)
- Support automatic style type casting for `Button`. [#2046](https://github.com/iced-rs/iced/pull/2046)
- Make `with_clip` and `with_save` in `Frame` able to return the data of the provided closure. [#1994](https://github.com/iced-rs/iced/pull/1994)
- Use `Radians` for angle fields in `Arc` and `arc::Elliptical`. [#2027](https://github.com/iced-rs/iced/pull/2027)
- Assert dimensions of quads are normal in `iced_tiny_skia`. [#2082](https://github.com/iced-rs/iced/pull/2082)
- Remove `position` from `overlay::Element`. [#2226](https://github.com/iced-rs/iced/pull/2226)
- Add a capacity limit to the `GlyphCache` in `iced_tiny_skia`. [#2210](https://github.com/iced-rs/iced/pull/2210)
- Use pointer equality to speed up `PartialEq` implementation of `image::Bytes`. [#2220](https://github.com/iced-rs/iced/pull/2220)
- Update `bitflags`, `glam`, `kurbo`, `ouroboros`, `qrcode`, and `sysinfo` dependencies. [#2227](https://github.com/iced-rs/iced/pull/2227)
- Improve some widget ergonomics. [#2253](https://github.com/iced-rs/iced/pull/2253)

### Fixed
- Clipping of `TextInput` selection. [#2199](https://github.com/iced-rs/iced/pull/2199)
- `Paragraph::grapheme_position` when ligatures are present. [#2196](https://github.com/iced-rs/iced/pull/2196)
- Docs to include missing feature tags. [#2184](https://github.com/iced-rs/iced/pull/2184)
- `PaneGrid` click interaction on the top edge. [#2168](https://github.com/iced-rs/iced/pull/2168)
- `iced_wgpu` not rendering text in SVGs. [#2161](https://github.com/iced-rs/iced/pull/2161)
- Text clipping. [#2154](https://github.com/iced-rs/iced/pull/2154)
- Text transparency in `iced_tiny_skia`. [#2250](https://github.com/iced-rs/iced/pull/2250)
- Layout invalidation when `Tooltip` changes `overlay`. [#2143](https://github.com/iced-rs/iced/pull/2143)
- `Overlay` composition. [#2142](https://github.com/iced-rs/iced/pull/2142)
- Incorrect GIF for the `progress_bar` example. [#2141](https://github.com/iced-rs/iced/pull/2141)
- Standalone compilation of `iced_renderer` crate. [#2134](https://github.com/iced-rs/iced/pull/2134)
- Maximize window button enabled when `Settings::resizable` is `false`. [#2124](https://github.com/iced-rs/iced/pull/2124)
- Width of horizontal scrollbar in `Scrollable`. [#2084](https://github.com/iced-rs/iced/pull/2084)
- `ComboBox` widget panic on wasm. [#2078](https://github.com/iced-rs/iced/pull/2078)
- Majority of unresolved documentation links. [#2077](https://github.com/iced-rs/iced/pull/2077)
- Web examples not running. [#2076](https://github.com/iced-rs/iced/pull/2076)
- GIFs and video examples broken. [#2074](https://github.com/iced-rs/iced/pull/2074)
- `@interpolate(flat)` not used as attribute. [#2071](https://github.com/iced-rs/iced/pull/2071)
- `Checkbox` and `Toggler` hidden behind scrollbar in `styling` example. [#2062](https://github.com/iced-rs/iced/pull/2062)
- Absolute `LineHeight` sometimes being `0`. [#2059](https://github.com/iced-rs/iced/pull/2059)
- Paste while holding ALT. [#2006](https://github.com/iced-rs/iced/pull/2006)
- `Command<T>::perform` to return a `Command<T>`. [#2000](https://github.com/iced-rs/iced/pull/2000)
- `convert_text` not called on `Svg` trees. [#1908](https://github.com/iced-rs/iced/pull/1908)
- Unused `backend.rs` file in renderer crate. [#2182](https://github.com/iced-rs/iced/pull/2182)
- Some `clippy::pedantic` lints. [#2096](https://github.com/iced-rs/iced/pull/2096)
- Some minor clippy fixes. [#2092](https://github.com/iced-rs/iced/pull/2092)
- Clippy docs keyword quoting. [#2091](https://github.com/iced-rs/iced/pull/2091)
- Clippy map transformations. [#2090](https://github.com/iced-rs/iced/pull/2090)
- Inline format args for ease of reading. [#2089](https://github.com/iced-rs/iced/pull/2089)
- Stuck scrolling in `Scrollable` with touch events. [#1940](https://github.com/iced-rs/iced/pull/1940)
- Incorrect unit in `system::Information`. [#2223](https://github.com/iced-rs/iced/pull/2223)
- `size_hint` not being called from `element::Map`. [#2224](https://github.com/iced-rs/iced/pull/2224)
- `size_hint` not being called from `element::Explain`. [#2225](https://github.com/iced-rs/iced/pull/2225)
- Slow touch scrolling for `TextEditor` widget. [#2140](https://github.com/iced-rs/iced/pull/2140)
- `Subscription::map` using unreliable function pointer hash to identify mappers. [#2237](https://github.com/iced-rs/iced/pull/2237)
- Missing feature flag docs for `time::every`. [#2188](https://github.com/iced-rs/iced/pull/2188)
- Event loop not being resumed on Windows while resizing. [#2214](https://github.com/iced-rs/iced/pull/2214)
- Alpha mode misconfiguration in `iced_wgpu`. [#2231](https://github.com/iced-rs/iced/pull/2231)
- Outdated documentation leading to a dead link. [#2232](https://github.com/iced-rs/iced/pull/2232)


Many thanks to...

- @akshayr-mecha
- @alec-deason
- @arslee07
- @AustinMReppert
- @avsaase
- @blazra
- @brianch
- @bungoboingo
- @Calastrophe
- @casperstorm
- @cfrenette
- @clarkmoody
- @Davidster
- @Decodetalkers
- @derezzedex
- @DoomDuck
- @dtzxporter
- @Dworv
- @fogarecious
- @GyulyVGC
- @hicaru
- @ids1024
- @Imberflur
- @jhannyj
- @jhff
- @jim-ec
- @joshuamegnauth54
- @jpttrssn
- @julianbraha
- @Koranir
- @lufte
- @matze
- @MichalLebeda
- @MoSal
- @MrAntix
- @nicksenger
- @Nisatru
- @nyurik
- @Remmirad
- @ripytide
- @snaggen
- @Tahinli
- @tarkah
- @tzemanovic
- @varbhat
- @VAWVAW
- @william-shere
- @wyatt-herkamp

## [0.10.0] - 2023-07-28
### Added
- Text shaping, font fallback, and `iced_wgpu` overhaul. [#1697](https://github.com/iced-rs/iced/pull/1697)
- Software renderer, runtime renderer fallback, and core consolidation. [#1748](https://github.com/iced-rs/iced/pull/1748)
- Incremental rendering for `iced_tiny_skia`. [#1811](https://github.com/iced-rs/iced/pull/1811)
- Configurable `LineHeight` support for text widgets. [#1828](https://github.com/iced-rs/iced/pull/1828)
- `text::Shaping` strategy selection. [#1822](https://github.com/iced-rs/iced/pull/1822)
- Subpixel glyph positioning and layout linearity. [#1921](https://github.com/iced-rs/iced/pull/1921)
- Background gradients. [#1846](https://github.com/iced-rs/iced/pull/1846)
- Offscreen rendering and screenshots. [#1845](https://github.com/iced-rs/iced/pull/1845)
- Nested overlays. [#1719](https://github.com/iced-rs/iced/pull/1719)
- Cursor availability. [#1904](https://github.com/iced-rs/iced/pull/1904)
- Backend-specific primitives. [#1932](https://github.com/iced-rs/iced/pull/1932)
- `ComboBox` widget. [#1954](https://github.com/iced-rs/iced/pull/1954)
- `web-colors` feature flag to enable "sRGB linear" blending. [#1888](https://github.com/iced-rs/iced/pull/1888)
- `PaneGrid` logic to split panes by drag & drop. [#1856](https://github.com/iced-rs/iced/pull/1856)
- `PaneGrid` logic to drag & drop panes to the edges. [#1865](https://github.com/iced-rs/iced/pull/1865)
- Type-safe `Scrollable` direction. [#1878](https://github.com/iced-rs/iced/pull/1878)
- `Scrollable` alignment. [#1912](https://github.com/iced-rs/iced/pull/1912)
- Helpers to change viewport alignment of a `Scrollable`. [#1953](https://github.com/iced-rs/iced/pull/1953)
- `scroll_to` widget operation. [#1796](https://github.com/iced-rs/iced/pull/1796)
- `scroll_to` helper. [#1804](https://github.com/iced-rs/iced/pull/1804)
- `visible_bounds` widget operation for `Container`. [#1971](https://github.com/iced-rs/iced/pull/1971)
- Command to fetch window size. [#1927](https://github.com/iced-rs/iced/pull/1927)
- Conversion support from `Fn` trait to custom theme. [#1861](https://github.com/iced-rs/iced/pull/1861)
- Configurable border radii on relevant widgets. [#1869](https://github.com/iced-rs/iced/pull/1869)
- `border_radius` styling to `Slider` rail. [#1892](https://github.com/iced-rs/iced/pull/1892)
- `application_id` in `PlatformSpecific` settings for Linux. [#1963](https://github.com/iced-rs/iced/pull/1963)
- Aliased entries in `text::Cache`. [#1934](https://github.com/iced-rs/iced/pull/1934)
- Text cache modes. [#1938](https://github.com/iced-rs/iced/pull/1938)
- `operate` method for `program::State`. [#1913](https://github.com/iced-rs/iced/pull/1913)
- `Viewport` argument to `Widget::on_event`. [#1956](https://github.com/iced-rs/iced/pull/1956)
- Nix instructions to `DEPENDENCIES.md`. [#1859](https://github.com/iced-rs/iced/pull/1859)
- Loading spinners example. [#1902](https://github.com/iced-rs/iced/pull/1902)
- Workflow that verifies `CHANGELOG` is always up-to-date. [#1970](https://github.com/iced-rs/iced/pull/1970)
- Outdated mentions of `iced_native` in `README`. [#1979](https://github.com/iced-rs/iced/pull/1979)

### Changed
- Updated `wgpu` to `0.16`. [#1807](https://github.com/iced-rs/iced/pull/1807)
- Updated `glam` to `0.24`. [#1840](https://github.com/iced-rs/iced/pull/1840)
- Updated `winit` to `0.28`. [#1738](https://github.com/iced-rs/iced/pull/1738)
- Updated `palette` to `0.7`. [#1875](https://github.com/iced-rs/iced/pull/1875)
- Updated `ouroboros` to `0.17`. [#1925](https://github.com/iced-rs/iced/pull/1925)
- Updated `resvg` to `0.35` and `tiny-skia` to `0.10`. [#1907](https://github.com/iced-rs/iced/pull/1907)
- Changed `mouse::Button::Other` to take `u16` instead of `u8`. [#1797](https://github.com/iced-rs/iced/pull/1797)
- Changed `subscription::channel` to take a `FnOnce` non-`Sync` closure. [#1917](https://github.com/iced-rs/iced/pull/1917)
- Removed `Copy` requirement for text `StyleSheet::Style`. [#1814](https://github.com/iced-rs/iced/pull/1814)
- Removed `min_width` of 1 from scrollbar & scroller for `Scrollable`. [#1844](https://github.com/iced-rs/iced/pull/1844)
- Used `Widget::overlay` for `Tooltip`. [#1692](https://github.com/iced-rs/iced/pull/1692)

### Fixed
- `Responsive` layout not invalidated when shell layout is invalidated. [#1799](https://github.com/iced-rs/iced/pull/1799)
- `Responsive` layout not invalidated when size changes without a `view` call. [#1890](https://github.com/iced-rs/iced/pull/1890)
- Broken link in `ROADMAP.md`. [#1815](https://github.com/iced-rs/iced/pull/1815)
- `bounds` of selected option background in `Menu`. [#1831](https://github.com/iced-rs/iced/pull/1831)
- Border radius logic in `iced_tiny_skia`. [#1842](https://github.com/iced-rs/iced/pull/1842)
- `Svg` filtered color not premultiplied. [#1841](https://github.com/iced-rs/iced/pull/1841)
- Race condition when growing an `image::Atlas`. [#1847](https://github.com/iced-rs/iced/pull/1847)
- Clearing damaged surface with background color in `iced_tiny_skia`. [#1854](https://github.com/iced-rs/iced/pull/1854)
- Private gradient pack logic for `iced_graphics::Gradient`. [#1871](https://github.com/iced-rs/iced/pull/1871)
- Unordered quads of different background types. [#1873](https://github.com/iced-rs/iced/pull/1873)
- Panic in `glyphon` when glyphs are missing. [#1883](https://github.com/iced-rs/iced/pull/1883)
- Empty scissor rectangle in `iced_wgpu::triangle` pipeline. [#1893](https://github.com/iced-rs/iced/pull/1893)
- `Scrollable` scrolling when mouse not over it. [#1910](https://github.com/iced-rs/iced/pull/1910)
- `translation` in `layout` of `Nested` overlay. [#1924](https://github.com/iced-rs/iced/pull/1924)
- Build when using vendored dependencies. [#1928](https://github.com/iced-rs/iced/pull/1928)
- Minor grammar mistake. [#1931](https://github.com/iced-rs/iced/pull/1931)
- Quad rendering including border only inside of the bounds. [#1843](https://github.com/iced-rs/iced/pull/1843)
- Redraw requests not being forwarded for `Component` overlays. [#1949](https://github.com/iced-rs/iced/pull/1949)
- Blinking input cursor when window loses focus. [#1955](https://github.com/iced-rs/iced/pull/1955)
- `BorderRadius` not exposed in root crate. [#1972](https://github.com/iced-rs/iced/pull/1972)
- Outdated `ROADMAP`. [#1958](https://github.com/iced-rs/iced/pull/1958)

### Patched
- Keybinds to cycle `ComboBox` options. [#1991](https://github.com/iced-rs/iced/pull/1991)
- `Tooltip` overlay position inside `Scrollable`. [#1978](https://github.com/iced-rs/iced/pull/1978)
- `iced_wgpu` freezing on empty layers. [#1996](https://github.com/iced-rs/iced/pull/1996)
- `image::Viewer` reacting to any scroll event. [#1998](https://github.com/iced-rs/iced/pull/1998)
- `TextInput` pasting text when `Alt` key is pressed. [#2006](https://github.com/iced-rs/iced/pull/2006)
- Broken link to old `iced_native` crate in `README`. [#2024](https://github.com/iced-rs/iced/pull/2024)
- `Rectangle::contains` being non-exclusive. [#2017](https://github.com/iced-rs/iced/pull/2017)
- Documentation for `Arc` and `arc::Elliptical`. [#2008](https://github.com/iced-rs/iced/pull/2008)

Many thanks to...

- @a1phyr
- @alec-deason
- @AustinMReppert
- @bbb651
- @bungoboingo
- @casperstorm
- @clarkmoody
- @Davidster
- @Drakulix
- @genusistimelord
- @GyulyVGC
- @ids1024
- @jhff
- @JonathanLindsey
- @kr105
- @marienz
- @malramsay64
- @nicksenger
- @nicoburns
- @NyxAlexandra
- @Redhawk18
- @RGBCube
- @rs017991
- @tarkah
- @thunderstorm010
- @ua-kxie
- @wash2
- @wiiznokes

## [0.9.0] - 2023-04-13
### Added
- `MouseArea` widget. [#1594](https://github.com/iced-rs/iced/pull/1594)
- `channel` helper in `subscription`. [#1786](https://github.com/iced-rs/iced/pull/1786)
- Configurable `width` for `Scrollable`. [#1749](https://github.com/iced-rs/iced/pull/1749)
- Support for disabled `TextInput`. [#1744](https://github.com/iced-rs/iced/pull/1744)
- Platform-specific window settings. [#1730](https://github.com/iced-rs/iced/pull/1730)
- Left and right colors for sliders. [#1643](https://github.com/iced-rs/iced/pull/1643)
- Icon for `TextInput`. [#1702](https://github.com/iced-rs/iced/pull/1702)
- Mouse over scrollbar flag for `scrollable::StyleSheet`. [#1669](https://github.com/iced-rs/iced/pull/1669)
- Better example for `Radio`. [#1762](https://github.com/iced-rs/iced/pull/1762)

### Changed
- `wgpu` has been updated to `0.15` in `iced_wgpu`. [#1789](https://github.com/iced-rs/iced/pull/1789)
- `resvg` has been updated to `0.29` in `iced_graphics`. [#1733](https://github.com/iced-rs/iced/pull/1733)
- `subscription::run` now takes a function pointer. [#1723](https://github.com/iced-rs/iced/pull/1723)

### Fixed
- Redundant `on_scroll` messages for `Scrollable`. [#1788](https://github.com/iced-rs/iced/pull/1788)
- Outdated items in `ROADMAP.md` [#1782](https://github.com/iced-rs/iced/pull/1782)
- Colons in shader labels causing compilation issues in `iced_wgpu`. [#1779](https://github.com/iced-rs/iced/pull/1779)
- Re-expose winit features for window servers in Linux. [#1777](https://github.com/iced-rs/iced/pull/1777)
- Replacement of application node in Wasm. [#1765](https://github.com/iced-rs/iced/pull/1765)
- `clippy` lints for Rust 1.68. [#1755](https://github.com/iced-rs/iced/pull/1755)
- Unnecessary `Component` rebuilds. [#1754](https://github.com/iced-rs/iced/pull/1754)
- Incorrect package name in checkbox example docs. [#1750](https://github.com/iced-rs/iced/pull/1750)
- Fullscreen only working on primary monitor. [#1742](https://github.com/iced-rs/iced/pull/1742)
- `Padding::fit` on irregular values for an axis. [#1734](https://github.com/iced-rs/iced/pull/1734)
- `Debug` implementation of `Font` displaying its bytes. [#1731](https://github.com/iced-rs/iced/pull/1731)
- Sliders bleeding over their rail. [#1721](https://github.com/iced-rs/iced/pull/1721)

### Removed
- `Fill` variant for `Alignment`. [#1735](https://github.com/iced-rs/iced/pull/1735)

Many thanks to...

- @ahoneybun
- @bq-wrongway
- @bungoboingo
- @casperstorm
- @Davidster
- @ElhamAryanpur
- @FinnPerry
- @GyulyVGC
- @JungleTryne
- @lupd
- @mmstick
- @nicksenger
- @Night-Hunter-NF
- @tarkah
- @traxys
- @Xaeroxe

## [0.8.0] - 2023-02-18
### Added
- Generic pixel units. [#1711](https://github.com/iced-rs/iced/pull/1711)
- `custom` method to `widget::Operation` trait. [#1649](https://github.com/iced-rs/iced/pull/1649)
- `Group` overlay. [#1655](https://github.com/iced-rs/iced/pull/1655)
- Standalone `draw` helper for `image`. [#1682](https://github.com/iced-rs/iced/pull/1682)
- Dynamic `pick_list::Handle`. [#1675](https://github.com/iced-rs/iced/pull/1675)
- `Id` support for `Container`. [#1695](https://github.com/iced-rs/iced/pull/1695)
- Custom `Checkbox` icon support. [#1707](https://github.com/iced-rs/iced/pull/1707)
- `window` action to change always on top setting. [#1587](https://github.com/iced-rs/iced/pull/1587)
- `window` action to fetch its unique identifier. [#1589](https://github.com/iced-rs/iced/pull/1589)

### Changed
- Annotated `Command` and `Subscription` with `#[must_use]`. [#1676](https://github.com/iced-rs/iced/pull/1676)
- Replaced `Fn` with `FnOnce` in `canvas::Cache::draw`. [#1694](https://github.com/iced-rs/iced/pull/1694)
- Used `[default]` on enum in `game_of_life` example. [#1660](https://github.com/iced-rs/iced/pull/1660)
- Made `QRCode` hide when data is empty in `qr_code` example. [#1665](https://github.com/iced-rs/iced/pull/1665)
- Replaced `Cow` with `Bytes` in `image` to accept any kind of data that implements `AsRef<[u8]>`. [#1551](https://github.com/iced-rs/iced/pull/1551)

### Fixed
- Blank window on application startup. [#1698](https://github.com/iced-rs/iced/pull/1698)
- Off-by-one pixel error on `pick_list` width. [#1679](https://github.com/iced-rs/iced/pull/1679)
- Missing `text_input` implementation in `operation::Map`. [#1678](https://github.com/iced-rs/iced/pull/1678)
- Widget-driven animations for `Component`. [#1685](https://github.com/iced-rs/iced/pull/1685)
- Layout translation in `overlay::Group`. [#1686](https://github.com/iced-rs/iced/pull/1686)
- Missing `is_over` implementation for overlays of `iced_lazy` widgets. [#1699](https://github.com/iced-rs/iced/pull/1699)
- Panic when overlay event processing removes overlay. [#1700](https://github.com/iced-rs/iced/pull/1700)
- Panic when using operations with components in certain cases. [#1701](https://github.com/iced-rs/iced/pull/1701)
- `TextInput` width when using padding. [#1706](https://github.com/iced-rs/iced/pull/1706)
- `iced_glow` crash on some hardware. [#1703](https://github.com/iced-rs/iced/pull/1703)
- Height of `overlay::Menu`. [#1714](https://github.com/iced-rs/iced/pull/1714)
- Size of images in `README`. [#1659](https://github.com/iced-rs/iced/pull/1659)
- New `clippy` lints. [#1681](https://github.com/iced-rs/iced/pull/1681)

Many thanks to...

- @13r0ck
- @bungoboingo
- @casperstorm
- @frey
- @greatest-ape
- @ids1024
- @Jedsek
- @nicksenger
- @Night-Hunter-NF
- @sdroege
- @Sn-Kinos
- @sushigiri
- @tarkah

## [0.7.0] - 2023-01-14
### Added
- Widget-driven animations. [#1647](https://github.com/iced-rs/iced/pull/1647)
- Multidirectional scrolling support for `Scrollable`. [#1550](https://github.com/iced-rs/iced/pull/1550)
- `VerticalSlider` widget. [#1596](https://github.com/iced-rs/iced/pull/1596)
- `Shift+Click` text selection support in `TextInput`. [#1622](https://github.com/iced-rs/iced/pull/1622)
- Profiling support with the `chrome-trace` feature. [#1565](https://github.com/iced-rs/iced/pull/1565)
- Customization of the handle of a `PickList`. [#1562](https://github.com/iced-rs/iced/pull/1562)
- `window` action to request user attention. [#1584](https://github.com/iced-rs/iced/pull/1584)
- `window` action to gain focus. [#1585](https://github.com/iced-rs/iced/pull/1585)
- `window` action to toggle decorations. [#1588](https://github.com/iced-rs/iced/pull/1588)
- `Copy` implementation for `gradient::Location`. [#1636](https://github.com/iced-rs/iced/pull/1636)

### Changed
- Replaced `Application::should_exit` with a `window::close` action. [#1606](https://github.com/iced-rs/iced/pull/1606)
- Made `focusable::Count` fields public. [#1635](https://github.com/iced-rs/iced/pull/1635)
- Added `Dependency` argument to the closure of `Lazy`. [#1646](https://github.com/iced-rs/iced/pull/1646)
- Switched arguments order of `Toggler::new` for consistency. [#1616](https://github.com/iced-rs/iced/pull/1616)
- Switched arguments order of `Checkbox::new` for consistency. [#1633](https://github.com/iced-rs/iced/pull/1633)

### Fixed
- Compilation error in `iced_glow` when the `image` feature is enabled but `svg` isn't. [#1593](https://github.com/iced-rs/iced/pull/1593)
- Widget operations for `Responsive` widget. [#1615](https://github.com/iced-rs/iced/pull/1615)
- Overlay placement for `Responsive`. [#1638](https://github.com/iced-rs/iced/pull/1638)
- `overlay` implementation for `Lazy`. [#1644](https://github.com/iced-rs/iced/pull/1644)
- Minor typo in documentation. [#1624](https://github.com/iced-rs/iced/pull/1624)
- Links in documentation. [#1634](https://github.com/iced-rs/iced/pull/1634)
- Missing comment in documentation. [#1648](https://github.com/iced-rs/iced/pull/1648)

Many thanks to...

- @13r0ck
- @Araxeus
- @ben-wallis
- @bungoboingo
- @casperstorm
- @nicksenger
- @Night-Hunter-NF
- @rpitasky
- @rs017991
- @tarkah
- @wiktor-k

## [0.6.0] - 2022-12-07
### Added
- Support for non-uniform border radius for `Primitive::Quad`. [#1506](https://github.com/iced-rs/iced/pull/1506)
- Operation to query the current focused widget. [#1526](https://github.com/iced-rs/iced/pull/1526)
- Additional operations for `TextInput`. [#1529](https://github.com/iced-rs/iced/pull/1529)
- Styling support for `Svg`. [#1578](https://github.com/iced-rs/iced/pull/1578)

### Changed
- Triangle geometry using a solid color is now drawn in a single draw call. [#1538](https://github.com/iced-rs/iced/pull/1538)

### Fixed
- Gradients for WebAssembly target. [#1524](https://github.com/iced-rs/iced/pull/1524)
- `Overlay` layout cache not being invalidated. [#1528](https://github.com/iced-rs/iced/pull/1528)
- Operations not working for `PaneGrid`. [#1533](https://github.com/iced-rs/iced/pull/1533)
- Mapped `widget::Operation` always returning `Outcome::None`. [#1536](https://github.com/iced-rs/iced/pull/1536)
- Padding of `TextInput` with `Length::Units` width. [#1539](https://github.com/iced-rs/iced/pull/1539)
- Clipping of `Image` and `Svg` widgets in `iced_glow`. [#1557](https://github.com/iced-rs/iced/pull/1557)
- Invalid links in documentation. [#1560](https://github.com/iced-rs/iced/pull/1560)
- `Custom` style of `PickList` widget. [#1570](https://github.com/iced-rs/iced/pull/1570)
- Scroller in `Scrollable` always being drawn. [#1574](https://github.com/iced-rs/iced/pull/1574)

Many thanks to...

- @bungoboingo
- @l1Dan
- @mmstick
- @mtkennerly
- @PolyMeilex
- @rksm
- @rs017991
- @tarkah
- @wash2

## [0.5.0] - 2022-11-10
### Added
- __[Stabilization of stateless widgets][stateless]__ (#1393)  
  The old widget API has been completely replaced by stateless widgets (introduced in #1284). Alongside the new API, there are a bunch of new helper functions and macros for easily describing view logic (like `row!` and `column!`).

- __[First-class theming][theming]__ (#1362)  
  A complete overhaul of our styling primitives, introducing a `Theme` as a first-class concept of the library.

- __[Widget operations][operations]__ (#1399)  
  An abstraction that can be used to traverse (and operate on) the widget tree of an application in order to query or update some widget state.

- __[`Lazy` widget][lazy]__ (#1400)  
  A widget that can call some view logic lazily only when some data has changed. Thanks to @nicksenger!

- __[Linear gradient support for `Canvas`][gradient]__ (#1448)  
  The `Canvas` widget can draw linear gradients now. Thanks to @bungoboingo!

- __[Touch support for `Canvas`][touch]__ (#1305)  
  The `Canvas` widget now supports touch events. Thanks to @artursapek!

- __[`Image` and `Svg` support for `iced_glow`][image]__ (#1485)  
  Our OpenGL renderer now is capable of rendering both the `Image` and `Svg` widgets. Thanks to @ids1024!

[stateless]: https://github.com/iced-rs/iced/pull/1393
[theming]: https://github.com/iced-rs/iced/pull/1362
[operations]: https://github.com/iced-rs/iced/pull/1399
[lazy]: https://github.com/iced-rs/iced/pull/1400
[gradient]: https://github.com/iced-rs/iced/pull/1448
[touch]: https://github.com/iced-rs/iced/pull/1305
[image]: https://github.com/iced-rs/iced/pull/1485

## [0.4.2] - 2022-05-03
### Fixed
- `Padding` type not exposed in `iced`.

## [0.4.1] - 2022-05-02
### Fixed
- Version number in `README`.

## [0.4.0] - 2022-05-02
### Added
- __[Stateless widgets][stateless]__ (#1284)  
  A brand new widget API that removes the need to keep track of internal widget state. No more `button::State` in your application!

- __[`Component` trait][component]__ (#1131)  
  A new trait to implement custom widgets with internal mutable state while using composition and [The Elm Architecture].

- __[`Responsive` widget][responsive]__ (#1193)  
  A widget that is aware of its dimensions and can be used to easily build responsive user interfaces.

- __[Experimental WebGL support][webgl]__ (#1096)  
  Applications can now be rendered into an HTML `canvas` when targeting Wasm by leveraging the WebGL support in [`wgpu`]. Thanks to @pacmancoder and @kaimast!

- __[Support for Raspberry Pis and older devices][raspberry]__ (#1160)  
  The compatibility of our OpenGL renderer has been improved and should run on any hardware that supports OpenGL 3.0+ or OpenGL ES 2.0+. Additionally, we started maintaining [Docker images for `aarch64` and `armv7`](https://github.com/orgs/iced-rs/packages) to easily cross-compile `iced` applications and target Raspberry Pis. Thanks to @derezzedex!

- __[Simpler `Renderer` APIs][renderer_apis]__ (#1110)  
  The surface of the `Renderer` APIs of the library has been considerably reduced. Instead of a `Renderer` trait per widget, now there are only 3 traits that are reused by all the widgets.

[webgl]: https://github.com/iced-rs/iced/pull/1096
[renderer_apis]: https://github.com/iced-rs/iced/pull/1110
[component]: https://github.com/iced-rs/iced/pull/1131
[raspberry]: https://github.com/iced-rs/iced/pull/1160
[responsive]: https://github.com/iced-rs/iced/pull/1193
[stateless]: https://github.com/iced-rs/iced/pull/1284
[The Elm Architecture]: https://guide.elm-lang.org/architecture/
[`wgpu`]: https://github.com/gfx-rs/wgpu


## [0.3.0] - 2021-03-31
### Added
- Touch support. [#57] [#650] (thanks to @simlay and @discordance!)
- Clipboard write access for
  - `TextInput` widget. [#770]
  - `Application::update`. [#773]
- `image::Viewer` widget. It allows panning and scaling of an image. [#319] (thanks to @tarkah!)
- `Tooltip` widget. It annotates content with some text on mouse hover. [#465] (thanks to @yusdacra!)
- Support for the [`smol`] async runtime. [#699] (thanks to @JayceFayne!)
- Support for graceful exiting when using the `Application` trait. [#804]
- Image format features in [`iced_wgpu`] to reduce code bloat. [#392] (thanks to @unrelentingtech!)
- `Focused` and `Unfocused` variant to `window::Event`. [#701] (thanks to @cossonleo!)
- `WGPU_BACKEND` environment variable to configure the internal graphics backend of `iced_wgpu`. [#789] (thanks to @Cupnfish!)

### Changed
- The `TitleBar` of a `PaneGrid` now supports generic elements. [#657] (thanks to @clarkmoody!)
- The `Error` type now implements `Send` and `Sync`. [#719] (thanks to @taiki-e!)
- The `Style` types in `iced_style` now implement `Clone` and `Copy`. [#720] (thanks to @taiki-e!)
- The following dependencies have been updated:
  - [`font-kit`] → `0.10` [#669]
  - [`glutin`] → `0.26` [#658]
  - [`resvg`] → `0.12` [#669]
  - [`tokio`] → `1.0` [#672] (thanks to @yusdacra!)
  - [`winit`] → `0.24` [#658]
  - [`wgpu`] → `0.7` [#725] (thanks to @PolyMeilex)
- The following examples were improved:
  - `download_progress` now showcases multiple file downloads at once. [#283] (thanks to @Folyd!)
  - `solar_system` uses the new `rand` API. [#760] (thanks to @TriedAngle!)

### Fixed
- Button events not being propagated to contents. [#668]
- Incorrect overlay implementation for the `Button` widget. [#764]
- `Viewport::physical_width` returning the wrong value. [#700]
- Outdated documentation for the `Sandbox` trait. [#710]

[#57]: https://github.com/iced-rs/iced/pull/57
[#283]: https://github.com/iced-rs/iced/pull/283
[#319]: https://github.com/iced-rs/iced/pull/319
[#392]: https://github.com/iced-rs/iced/pull/392
[#465]: https://github.com/iced-rs/iced/pull/465
[#650]: https://github.com/iced-rs/iced/pull/650
[#657]: https://github.com/iced-rs/iced/pull/657
[#658]: https://github.com/iced-rs/iced/pull/658
[#668]: https://github.com/iced-rs/iced/pull/668
[#669]: https://github.com/iced-rs/iced/pull/669
[#672]: https://github.com/iced-rs/iced/pull/672
[#699]: https://github.com/iced-rs/iced/pull/699
[#700]: https://github.com/iced-rs/iced/pull/700
[#701]: https://github.com/iced-rs/iced/pull/701
[#710]: https://github.com/iced-rs/iced/pull/710
[#719]: https://github.com/iced-rs/iced/pull/719
[#720]: https://github.com/iced-rs/iced/pull/720
[#725]: https://github.com/iced-rs/iced/pull/725
[#760]: https://github.com/iced-rs/iced/pull/760
[#764]: https://github.com/iced-rs/iced/pull/764
[#770]: https://github.com/iced-rs/iced/pull/770
[#773]: https://github.com/iced-rs/iced/pull/773
[#789]: https://github.com/iced-rs/iced/pull/789
[#804]: https://github.com/iced-rs/iced/pull/804
[`smol`]: https://github.com/smol-rs/smol
[`winit`]: https://github.com/rust-windowing/winit
[`glutin`]: https://github.com/rust-windowing/glutin
[`font-kit`]: https://github.com/servo/font-kit

## [0.2.0] - 2020-11-26
### Added
- __[`Canvas` interactivity][canvas]__ (#325)  
  A trait-based approach to react to mouse and keyboard interactions in [the `Canvas` widget][#193].

- __[`iced_graphics` subcrate][opengl]__ (#354)  
  A backend-agnostic graphics subcrate that can be leveraged to build new renderers.

- __[OpenGL renderer][opengl]__ (#354)  
  An OpenGL renderer powered by [`iced_graphics`], [`glow`], and [`glutin`]. It is an alternative to the default [`wgpu`] renderer.

- __[Overlay support][pick_list]__ (#444)  
  Basic support for superpositioning interactive widgets on top of other widgets.

- __[Faster event loop][view]__ (#597)  
  The event loop now takes advantage of the data dependencies in [The Elm Architecture] and leverages the borrow checker to keep the widget tree alive between iterations, avoiding unnecessary rebuilds.

- __[Event capturing][event]__ (#614)  
  The runtime now can tell whether a widget has handled an event or not, easing [integration with existing applications].

- __[`PickList` widget][pick_list]__ (#444)  
  A drop-down selector widget built on top of the new overlay support.

- __[`QRCode` widget][qr_code]__ (#622)  
  A widget that displays a QR code, powered by [the `qrcode` crate].

[canvas]: https://github.com/iced-rs/iced/pull/325
[opengl]: https://github.com/iced-rs/iced/pull/354
[`iced_graphics`]: https://github.com/iced-rs/iced/pull/354
[pane_grid]: https://github.com/iced-rs/iced/pull/397
[pick_list]: https://github.com/iced-rs/iced/pull/444
[error]: https://github.com/iced-rs/iced/pull/514
[view]: https://github.com/iced-rs/iced/pull/597
[event]: https://github.com/iced-rs/iced/pull/614
[color]: https://github.com/iced-rs/iced/pull/200
[qr_code]: https://github.com/iced-rs/iced/pull/622
[#193]: https://github.com/iced-rs/iced/pull/193
[`glutin`]: https://github.com/rust-windowing/glutin
[`wgpu`]: https://github.com/gfx-rs/wgpu
[`glow`]: https://github.com/grovesNL/glow
[the `qrcode` crate]: https://docs.rs/qrcode/0.12.0/qrcode/
[integration with existing applications]: https://github.com/iced-rs/iced/pull/183
[The Elm Architecture]: https://guide.elm-lang.org/architecture/


## [0.1.1] - 2020-04-15
### Added
- `Settings::with_flags` to easily initialize some default settings with flags. [#266]
- `Default` implementation for `canvas::layer::Cache`. [#267]
- `Ctrl + Del` support for `TextInput`. [#268]
- Helper methods in `canvas::Path` to easily draw lines, rectangles, and circles. [#293]
- `From<Color>` implementation for `canvas::Fill`. [#293]
- `From<String>` implementation for `canvas::Text`. [#293]
- `From<&str>` implementation for `canvas::Text`. [#293]

### Changed
- `new` method of `Radio` and `Checkbox` now take a generic `Into<String>` for the label. [#260]
- `Frame::fill` now takes a generic `Into<canvas::Fill>`. [#293]
- `Frame::stroke` now takes a generic `Into<canvas::Stroke>`. [#293]
- `Frame::fill_text` now takes a generic `Into<canvas::Text>`. [#293]

### Fixed
- Feature flags not being referenced in documentation. [#259]
- Crash in some graphics drivers when displaying an empty `Canvas`. [#278]
- Text measuring when spaces where present at the beginning of a `TextInput` value. [#279]
- `TextInput` producing a `Clip` primitive when unnecessary. [#279]
- Alignment of `Text` primitive in `iced_wgpu`. [#281]
- `CursorEntered` and `CursorLeft` not being generated. [#289]

### Removed
- Unnecessary `'static` lifetimes in `Renderer` bounds. [#290]

[#259]: https://github.com/iced-rs/iced/pull/259
[#260]: https://github.com/iced-rs/iced/pull/260
[#266]: https://github.com/iced-rs/iced/pull/266
[#267]: https://github.com/iced-rs/iced/pull/267
[#268]: https://github.com/iced-rs/iced/pull/268
[#278]: https://github.com/iced-rs/iced/pull/278
[#279]: https://github.com/iced-rs/iced/pull/279
[#281]: https://github.com/iced-rs/iced/pull/281
[#289]: https://github.com/iced-rs/iced/pull/289
[#290]: https://github.com/iced-rs/iced/pull/290
[#293]: https://github.com/iced-rs/iced/pull/293


## [0.1.0] - 2020-04-02
### Added
- __[Event subscriptions]__ (#122)  
  A declarative way to listen to external events asynchronously by leveraging [streams].

- __[Custom styling]__ (#146)  
  A simple, trait-based approach for customizing the appearance of different widgets.

- __[`Canvas` widget]__ (#193)  
  A widget for drawing 2D graphics with an interface inspired by the [Web Canvas API] and powered by [`lyon`].

- __[`PaneGrid` widget]__ (#224)  
  A widget that dynamically organizes layout by splitting panes that can be resized and drag and dropped.

- __[`Svg` widget]__ (#111)  
  A widget that renders vector graphics on top of [`resvg`] and [`raqote`]. Thanks to @Maldela!

- __[`ProgressBar` widget]__ (#141)  
  A widget to notify progress of asynchronous tasks to your users. Thanks to @Songtronix!

- __[Configurable futures executor]__ (#164)  
  Support for plugging [`tokio`], [`async-std`], [`wasm-bindgen-futures`], or your own custom futures executor to an application.

- __[Compatibility with existing `wgpu` projects]__ (#183)  
  A bunch of improvements to the flexibility of [`iced_wgpu`] to allow integration in existing codebases.

- __[Text selection for `TextInput`]__ (#202)  
  Thanks to @FabianLars and @Finnerale!

- __[Texture atlas for `iced_wgpu`]__ (#154)  
  An atlas on top of [`guillotiere`] for batching draw calls. Thanks to @Maldela!

[Event subscriptions]: https://github.com/iced-rs/iced/pull/122
[Custom styling]: https://github.com/iced-rs/iced/pull/146
[`Canvas` widget]: https://github.com/iced-rs/iced/pull/193
[`PaneGrid` widget]: https://github.com/iced-rs/iced/pull/224
[`Svg` widget]: https://github.com/iced-rs/iced/pull/111
[`ProgressBar` widget]: https://github.com/iced-rs/iced/pull/141
[Configurable futures executor]: https://github.com/iced-rs/iced/pull/164
[Compatibility with existing `wgpu` projects]: https://github.com/iced-rs/iced/pull/183
[Clipboard access]: https://github.com/iced-rs/iced/pull/132
[Texture atlas for `iced_wgpu`]: https://github.com/iced-rs/iced/pull/154
[Text selection for `TextInput`]: https://github.com/iced-rs/iced/pull/202
[`lyon`]: https://github.com/nical/lyon
[`guillotiere`]: https://github.com/nical/guillotiere
[Web Canvas API]: https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API
[streams]: https://docs.rs/futures/0.3.4/futures/stream/index.html
[`tokio`]: https://github.com/tokio-rs/tokio
[`async-std`]: https://github.com/async-rs/async-std
[`wasm-bindgen-futures`]: https://github.com/rustwasm/wasm-bindgen/tree/master/crates/futures
[`resvg`]: https://github.com/RazrFalcon/resvg
[`raqote`]: https://github.com/jrmuizel/raqote
[`iced_wgpu`]: wgpu/


## [0.1.0-beta] - 2019-11-25
### Changed
- The old `iced` becomes `iced_native`. The current `iced` crate turns into a batteries-included, cross-platform GUI library.


## [0.1.0-alpha] - 2019-09-05
### Added
- First release! :tada:

[Unreleased]: https://github.com/iced-rs/iced/compare/0.13.1...HEAD
[0.13.1]: https://github.com/iced-rs/iced/compare/0.13.0...0.13.1
[0.13.0]: https://github.com/iced-rs/iced/compare/0.12.1...0.13.0
[0.12.1]: https://github.com/iced-rs/iced/compare/0.12.0...0.12.1
[0.12.0]: https://github.com/iced-rs/iced/compare/0.10.0...0.12.0
[0.10.0]: https://github.com/iced-rs/iced/compare/0.9.0...0.10.0
[0.9.0]: https://github.com/iced-rs/iced/compare/0.8.0...0.9.0
[0.8.0]: https://github.com/iced-rs/iced/compare/0.7.0...0.8.0
[0.7.0]: https://github.com/iced-rs/iced/compare/0.6.0...0.7.0
[0.6.0]: https://github.com/iced-rs/iced/compare/0.5.0...0.6.0
[0.5.0]: https://github.com/iced-rs/iced/compare/0.4.2...0.5.0
[0.4.2]: https://github.com/iced-rs/iced/compare/0.4.1...0.4.2
[0.4.1]: https://github.com/iced-rs/iced/compare/0.4.0...0.4.1
[0.4.0]: https://github.com/iced-rs/iced/compare/0.3.0...0.4.0
[0.3.0]: https://github.com/iced-rs/iced/compare/0.2.0...0.3.0
[0.2.0]: https://github.com/iced-rs/iced/compare/0.1.1...0.2.0
[0.1.1]: https://github.com/iced-rs/iced/compare/0.1.0...0.1.1
[0.1.0]: https://github.com/iced-rs/iced/compare/0.1.0-beta...0.1.0
[0.1.0-beta]: https://github.com/iced-rs/iced/compare/0.1.0-alpha...0.1.0-beta
[0.1.0-alpha]: https://github.com/iced-rs/iced/releases/tag/0.1.0-alpha
