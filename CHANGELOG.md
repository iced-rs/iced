# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
