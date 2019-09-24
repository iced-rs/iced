# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- `iced_core`, `iced_native`, and `iced_web` subcrates. [#17]
- The `Length` type. It unifies methods like `width`, `fill_width`, etc. [#17]

### Changed
- `iced` becomes `iced_native`. The current `iced` crate will be empty until we have a first prototype of a low-level renderer. [#17]
- Widgets expose their fields publicly, simplifying the `Renderer` traits in `iced_native`. [#17]
- Widgets no longer hold a default `Style`. Related to [#6] and [#12]. [#17]
- All `Renderer` traits now have total control to produce a `Node`. Related to [#6] and [#12]. [#17]
- The tour example lives in its own crate now and it can be run both on native and web. [#17]

### Removed
- `Color` generic parameter in widgets. The new `Color` type is used now. This keeps widgets as reusable as possible. It may complicate implementing some runtimes, but a color mapper should be easily implementable in situations where colors are limited. [#17]

[#6]: https://github.com/hecrj/iced/issues/6
[#12]: https://github.com/hecrj/iced/issues/12
[#17]: https://github.com/hecrj/iced/pull/17


## [0.1.0-alpha] - 2019-09-05
### Added
- First release! :tada:

[Unreleased]: https://github.com/hecrj/iced/compare/0.1.0-alpha...HEAD
[0.1.0-alpha]: https://github.com/hecrj/iced/releases/tag/0.1.0-alpha
