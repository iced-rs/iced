# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[#57]: https://github.com/hecrj/iced/pull/57
[#283]: https://github.com/hecrj/iced/pull/283
[#319]: https://github.com/hecrj/iced/pull/319
[#392]: https://github.com/hecrj/iced/pull/392
[#465]: https://github.com/hecrj/iced/pull/465
[#650]: https://github.com/hecrj/iced/pull/650
[#657]: https://github.com/hecrj/iced/pull/657
[#658]: https://github.com/hecrj/iced/pull/658
[#668]: https://github.com/hecrj/iced/pull/668
[#669]: https://github.com/hecrj/iced/pull/669
[#672]: https://github.com/hecrj/iced/pull/672
[#699]: https://github.com/hecrj/iced/pull/699
[#700]: https://github.com/hecrj/iced/pull/700
[#701]: https://github.com/hecrj/iced/pull/701
[#710]: https://github.com/hecrj/iced/pull/710
[#719]: https://github.com/hecrj/iced/pull/719
[#720]: https://github.com/hecrj/iced/pull/720
[#725]: https://github.com/hecrj/iced/pull/725
[#760]: https://github.com/hecrj/iced/pull/760
[#764]: https://github.com/hecrj/iced/pull/764
[#770]: https://github.com/hecrj/iced/pull/770
[#773]: https://github.com/hecrj/iced/pull/773
[#789]: https://github.com/hecrj/iced/pull/789
[#804]: https://github.com/hecrj/iced/pull/804
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

[canvas]: https://github.com/hecrj/iced/pull/325
[opengl]: https://github.com/hecrj/iced/pull/354
[`iced_graphics`]: https://github.com/hecrj/iced/pull/354
[pane_grid]: https://github.com/hecrj/iced/pull/397
[pick_list]: https://github.com/hecrj/iced/pull/444
[error]: https://github.com/hecrj/iced/pull/514
[view]: https://github.com/hecrj/iced/pull/597
[event]: https://github.com/hecrj/iced/pull/614
[color]: https://github.com/hecrj/iced/pull/200
[qr_code]: https://github.com/hecrj/iced/pull/622
[#193]: https://github.com/hecrj/iced/pull/193
[`glutin`]: https://github.com/rust-windowing/glutin
[`wgpu`]: https://github.com/gfx-rs/wgpu-rs
[`glow`]: https://github.com/grovesNL/glow
[the `qrcode` crate]: https://docs.rs/qrcode/0.12.0/qrcode/
[integration with existing applications]: https://github.com/hecrj/iced/pull/183
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

[#259]: https://github.com/hecrj/iced/pull/259
[#260]: https://github.com/hecrj/iced/pull/260
[#266]: https://github.com/hecrj/iced/pull/266
[#267]: https://github.com/hecrj/iced/pull/267
[#268]: https://github.com/hecrj/iced/pull/268
[#278]: https://github.com/hecrj/iced/pull/278
[#279]: https://github.com/hecrj/iced/pull/279
[#281]: https://github.com/hecrj/iced/pull/281
[#289]: https://github.com/hecrj/iced/pull/289
[#290]: https://github.com/hecrj/iced/pull/290
[#293]: https://github.com/hecrj/iced/pull/293


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

[Event subscriptions]: https://github.com/hecrj/iced/pull/122
[Custom styling]: https://github.com/hecrj/iced/pull/146
[`Canvas` widget]: https://github.com/hecrj/iced/pull/193
[`PaneGrid` widget]: https://github.com/hecrj/iced/pull/224
[`Svg` widget]: https://github.com/hecrj/iced/pull/111
[`ProgressBar` widget]: https://github.com/hecrj/iced/pull/141
[Configurable futures executor]: https://github.com/hecrj/iced/pull/164
[Compatibility with existing `wgpu` projects]: https://github.com/hecrj/iced/pull/183
[Clipboard access]: https://github.com/hecrj/iced/pull/132
[Texture atlas for `iced_wgpu`]: https://github.com/hecrj/iced/pull/154
[Text selection for `TextInput`]: https://github.com/hecrj/iced/pull/202
[`lyon`]: https://github.com/nical/lyon
[`guillotiere`]: https://github.com/nical/guillotiere
[Web Canvas API]: https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API
[streams]: https://docs.rs/futures/0.3.4/futures/stream/index.html
[`tokio`]: https://github.com/tokio-rs/tokio
[`async-std`]: https://github.com/async-rs/async-std
[`wasm-bindgen-futures`]: https://github.com/rustwasm/wasm-bindgen/tree/master/crates/futures
[`resvg`]: https://github.com/RazrFalcon/resvg
[`raqote`]: https://github.com/jrmuizel/raqote
[`iced_wgpu`]: https://github.com/hecrj/iced/tree/master/wgpu


## [0.1.0-beta] - 2019-11-25
### Changed
- The old `iced` becomes `iced_native`. The current `iced` crate turns into a batteries-included, cross-platform GUI library.


## [0.1.0-alpha] - 2019-09-05
### Added
- First release! :tada:

[Unreleased]: https://github.com/hecrj/iced/compare/0.3.0...HEAD
[0.3.0]: https://github.com/hecrj/iced/compare/0.2.0...0.3.0
[0.2.0]: https://github.com/hecrj/iced/compare/0.1.1...0.2.0
[0.1.1]: https://github.com/hecrj/iced/compare/0.1.0...0.1.1
[0.1.0]: https://github.com/hecrj/iced/compare/0.1.0-beta...0.1.0
[0.1.0-beta]: https://github.com/hecrj/iced/compare/0.1.0-alpha...0.1.0-beta
[0.1.0-alpha]: https://github.com/hecrj/iced/releases/tag/0.1.0-alpha
