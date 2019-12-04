# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- `image::Handle` type with `from_path` and `from_memory` methods. [#90]
- `image::Data` enum representing different kinds of image data. [#90]

### Changed
- `Image::new` takes an `Into<image::Handle>` now instead of an `Into<String>`. [#90]

### Fixed
- `Image` widget not keeping aspect ratio consistently. [#90]

[#90]: https://github.com/hecrj/iced/pull/90

## [0.1.0] - 2019-11-25
### Added
- First release! :tada:

[Unreleased]: https://github.com/hecrj/iced/compare/native-0.1.0...HEAD
[0.1.0]: https://github.com/hecrj/iced/releases/tag/native-0.1.0
