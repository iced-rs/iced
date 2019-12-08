# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- `image::Handle` type with `from_path` and `from_memory` methods. [#90]
- `image::Data` enum representing different kinds of image data. [#90]
- `text_input::Renderer::measure_value` required method to measure the width of a `TextInput` value. [#108]
- Click-based cursor positioning for `TextInput`. [#108]
- `Home` and `End` keys support for `TextInput`. [#108]
- `Ctrl+Left` and `Ctrl+Right` cursor word jump for `TextInput`. [#108]
- `keyboard::ModifiersState` struct which contains the state of the keyboard modifiers. [#108]
- `TextInput::password` method to enable secure password input mode. [#113]

### Changed
- `Image::new` takes an `Into<image::Handle>` now instead of an `Into<String>`. [#90]
- `Button::background` takes an `Into<Background>` now instead of a `Background`.
- `keyboard::Event::Input` now contains key modifiers state. [#108]

### Fixed
- `Image` widget not keeping aspect ratio consistently. [#90]
- `TextInput` not taking grapheme clusters into account. [#108]

[#90]: https://github.com/hecrj/iced/pull/90
[#108]: https://github.com/hecrj/iced/pull/108
[#113]: https://github.com/hecrj/iced/pull/113


## [0.1.0] - 2019-11-25
### Added
- First release! :tada:

[Unreleased]: https://github.com/hecrj/iced/compare/native-0.1.0...HEAD
[0.1.0]: https://github.com/hecrj/iced/releases/tag/native-0.1.0
