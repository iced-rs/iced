//! UI scaling is important, so read the docs for this module if you don't want to be confused.
//!
//! ## Why should I care about UI scaling?
//!
//! Modern computer screens don't have a consistent relationship between resolution and size.
//! 1920x1080 is a common resolution for both desktop and mobile screens, despite mobile screens
//! normally being less than a quarter the size of their desktop counterparts. What's more, neither
//! desktop nor mobile screens are consistent resolutions within their own size classes - common
//! mobile screens range from below 720p to above 1440p, and desktop screens range from 720p to 5K
//! and beyond.
//!
//! Given that, it's a mistake to assume that 2D content will only be displayed on screens with
//! a consistent pixel density. If you were to render a 96-pixel-square image on a 1080p screen,
//! then render the same image on a similarly-sized 4K screen, the 4K rendition would only take up
//! about a quarter of the physical space as it did on the 1080p screen. That issue is especially
//! problematic with text rendering, where quarter-sized text becomes a significant legibility
//! problem.
//!
//! Failure to account for the scale factor can create a significantly degraded user experience.
//! Most notably, it can make users feel like they have bad eyesight, which will potentially cause
//! them to think about growing elderly, resulting in them having an existential crisis. Once users
//! enter that state, they will no longer be focused on your application.
//!
//! ## How should I handle it?
//!
//! The solution to this problem is to account for the device's *scale factor*. The scale factor is
//! the factor UI elements should be scaled by to be consistent with the rest of the user's system -
//! for example, a button that's normally 50 pixels across would be 100 pixels across on a device
//! with a scale factor of `2.0`, or 75 pixels across with a scale factor of `1.5`.
//!
//! Many UI systems, such as CSS, expose DPI-dependent units like [points] or [picas]. That's
//! usually a mistake, since there's no consistent mapping between the scale factor and the screen's
//! actual DPI. Unless you're printing to a physical medium, you should work in scaled pixels rather
//! than any DPI-dependent units.
//!
//! ### Position and Size types
//!
//! Winit's [`PhysicalPosition`] / [`PhysicalSize`] types correspond with the actual pixels on the
//! device, and the [`LogicalPosition`] / [`LogicalSize`] types correspond to the physical pixels
//! divided by the scale factor.
//! All of Winit's functions return physical types, but can take either logical or physical
//! coordinates as input, allowing you to use the most convenient coordinate system for your
//! particular application.
//!
//! Winit's position and size types types are generic over their exact pixel type, `P`, to allow the
//! API to have integer precision where appropriate (e.g. most window manipulation functions) and
//! floating precision when necessary (e.g. logical sizes for fractional scale factors and touch
//! input). If `P` is a floating-point type, please do not cast the values with `as {int}`. Doing so
//! will truncate the fractional part of the float, rather than properly round to the nearest
//! integer. Use the provided `cast` function or [`From`]/[`Into`] conversions, which handle the
//! rounding properly. Note that precision loss will still occur when rounding from a float to an
//! int, although rounding lessens the problem.
//!
//! ### Events
//!
//! Winit will dispatch a [`ScaleFactorChanged`] event whenever a window's scale factor has changed.
//! This can happen if the user drags their window from a standard-resolution monitor to a high-DPI
//! monitor, or if the user changes their DPI settings. This gives you a chance to rescale your
//! application's UI elements and adjust how the platform changes the window's size to reflect the new
//! scale factor. If a window hasn't received a [`ScaleFactorChanged`] event,  then its scale factor
//! can be found by calling [`window.scale_factor()`].
//!
//! ## How is the scale factor calculated?
//!
//! Scale factor is calculated differently on different platforms:
//!
//! - **Windows:** On Windows 8 and 10, per-monitor scaling is readily configured by users from the
//!   display settings. While users are free to select any option they want, they're only given a
//!   selection of "nice" scale factors, i.e. 1.0, 1.25, 1.5... on Windows 7, the scale factor is
//!   global and changing it requires logging out. See [this article][windows_1] for technical
//!   details.
//! - **macOS:** Recent versions of macOS allow the user to change the scaling factor for certain
//!   displays. When this is available, the user may pick a per-monitor scaling factor from a set
//!   of pre-defined settings. All "retina displays" have a scaling factor above 1.0 by default but
//!   the specific value varies across devices.
//! - **X11:** Many man-hours have been spent trying to figure out how to handle DPI in X11. Winit
//!   currently uses a three-pronged approach:
//!   + Use the value in the `WINIT_X11_SCALE_FACTOR` environment variable, if present.
//!   + If not present, use the value set in `Xft.dpi` in Xresources.
//!   + Otherwise, calculate the scale factor based on the millimeter monitor dimensions provided by XRandR.
//!
//!   If `WINIT_X11_SCALE_FACTOR` is set to `randr`, it'll ignore the `Xft.dpi` field and use the
//!   XRandR scaling method. Generally speaking, you should try to configure the standard system
//!   variables to do what you want before resorting to `WINIT_X11_SCALE_FACTOR`.
//! - **Wayland:** On Wayland, scale factors are set per-screen by the server, and are always
//!   integers (most often 1 or 2).
//! - **iOS:** Scale factors are set by Apple to the value that best suits the device, and range
//!   from `1.0` to `3.0`. See [this article][apple_1] and [this article][apple_2] for more
//!   information.
//! - **Android:** Scale factors are set by the manufacturer to the value that best suits the
//!   device, and range from `1.0` to `4.0`. See [this article][android_1] for more information.
//! - **Web:** The scale factor is the ratio between CSS pixels and the physical device pixels.
//!   In other words, it is the value of [`window.devicePixelRatio`][web_1]. It is affected by
//!   both the screen scaling and the browser zoom level and can go below `1.0`.
//!
//!
//! [points]: https://en.wikipedia.org/wiki/Point_(typography)
//! [picas]: https://en.wikipedia.org/wiki/Pica_(typography)
//! [`ScaleFactorChanged`]: crate::event::WindowEvent::ScaleFactorChanged
//! [`window.scale_factor()`]: crate::window::Window::scale_factor
//! [windows_1]: https://docs.microsoft.com/en-us/windows/win32/hidpi/high-dpi-desktop-application-development-on-windows
//! [apple_1]: https://developer.apple.com/library/archive/documentation/DeviceInformation/Reference/iOSDeviceCompatibility/Displays/Displays.html
//! [apple_2]: https://developer.apple.com/design/human-interface-guidelines/macos/icons-and-images/image-size-and-resolution/
//! [android_1]: https://developer.android.com/training/multiscreen/screendensities
//! [web_1]: https://developer.mozilla.org/en-US/docs/Web/API/Window/devicePixelRatio

pub trait Pixel: Copy + Into<f64> {
    fn from_f64(f: f64) -> Self;
    fn cast<P: Pixel>(self) -> P {
        P::from_f64(self.into())
    }
}

impl Pixel for u8 {
    fn from_f64(f: f64) -> Self {
        f.round() as u8
    }
}
impl Pixel for u16 {
    fn from_f64(f: f64) -> Self {
        f.round() as u16
    }
}
impl Pixel for u32 {
    fn from_f64(f: f64) -> Self {
        f.round() as u32
    }
}
impl Pixel for i8 {
    fn from_f64(f: f64) -> Self {
        f.round() as i8
    }
}
impl Pixel for i16 {
    fn from_f64(f: f64) -> Self {
        f.round() as i16
    }
}
impl Pixel for i32 {
    fn from_f64(f: f64) -> Self {
        f.round() as i32
    }
}
impl Pixel for f32 {
    fn from_f64(f: f64) -> Self {
        f as f32
    }
}
impl Pixel for f64 {
    fn from_f64(f: f64) -> Self {
        f
    }
}

/// Checks that the scale factor is a normal positive `f64`.
///
/// All functions that take a scale factor assert that this will return `true`. If you're sourcing scale factors from
/// anywhere other than winit, it's recommended to validate them using this function before passing them to winit;
/// otherwise, you risk panics.
#[inline]
pub fn validate_scale_factor(scale_factor: f64) -> bool {
    scale_factor.is_sign_positive() && scale_factor.is_normal()
}

/// A position represented in logical pixels.
///
/// The position is stored as floats, so please be careful. Casting floats to integers truncates the
/// fractional part, which can cause noticable issues. To help with that, an `Into<(i32, i32)>`
/// implementation is provided which does the rounding for you.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LogicalPosition<P> {
    pub x: P,
    pub y: P,
}

impl<P> LogicalPosition<P> {
    #[inline]
    pub const fn new(x: P, y: P) -> Self {
        LogicalPosition { x, y }
    }
}

impl<P: Pixel> LogicalPosition<P> {
    #[inline]
    pub fn from_physical<T: Into<PhysicalPosition<X>>, X: Pixel>(
        physical: T,
        scale_factor: f64,
    ) -> Self {
        physical.into().to_logical(scale_factor)
    }

    #[inline]
    pub fn to_physical<X: Pixel>(
        &self,
        scale_factor: f64,
    ) -> PhysicalPosition<X> {
        assert!(validate_scale_factor(scale_factor));
        let x = self.x.into() * scale_factor;
        let y = self.y.into() * scale_factor;
        PhysicalPosition::new(x, y).cast()
    }

    #[inline]
    pub fn cast<X: Pixel>(&self) -> LogicalPosition<X> {
        LogicalPosition {
            x: self.x.cast(),
            y: self.y.cast(),
        }
    }
}

impl<P: Pixel, X: Pixel> From<(X, X)> for LogicalPosition<P> {
    fn from((x, y): (X, X)) -> LogicalPosition<P> {
        LogicalPosition::new(x.cast(), y.cast())
    }
}

impl<P: Pixel, X: Pixel> From<LogicalPosition<P>> for (X, X) {
    fn from(p: LogicalPosition<P>) -> (X, X) {
        (p.x.cast(), p.y.cast())
    }
}

impl<P: Pixel, X: Pixel> From<[X; 2]> for LogicalPosition<P> {
    fn from([x, y]: [X; 2]) -> LogicalPosition<P> {
        LogicalPosition::new(x.cast(), y.cast())
    }
}

impl<P: Pixel, X: Pixel> From<LogicalPosition<P>> for [X; 2] {
    fn from(p: LogicalPosition<P>) -> [X; 2] {
        [p.x.cast(), p.y.cast()]
    }
}

#[cfg(feature = "mint")]
impl<P: Pixel> From<mint::Point2<P>> for LogicalPosition<P> {
    fn from(p: mint::Point2<P>) -> Self {
        Self::new(p.x, p.y)
    }
}

#[cfg(feature = "mint")]
impl<P: Pixel> From<LogicalPosition<P>> for mint::Point2<P> {
    fn from(p: LogicalPosition<P>) -> Self {
        mint::Point2 { x: p.x, y: p.y }
    }
}

/// A position represented in physical pixels.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PhysicalPosition<P> {
    pub x: P,
    pub y: P,
}

impl<P> PhysicalPosition<P> {
    #[inline]
    pub const fn new(x: P, y: P) -> Self {
        PhysicalPosition { x, y }
    }
}

impl<P: Pixel> PhysicalPosition<P> {
    #[inline]
    pub fn from_logical<T: Into<LogicalPosition<X>>, X: Pixel>(
        logical: T,
        scale_factor: f64,
    ) -> Self {
        logical.into().to_physical(scale_factor)
    }

    #[inline]
    pub fn to_logical<X: Pixel>(
        &self,
        scale_factor: f64,
    ) -> LogicalPosition<X> {
        assert!(validate_scale_factor(scale_factor));
        let x = self.x.into() / scale_factor;
        let y = self.y.into() / scale_factor;
        LogicalPosition::new(x, y).cast()
    }

    #[inline]
    pub fn cast<X: Pixel>(&self) -> PhysicalPosition<X> {
        PhysicalPosition {
            x: self.x.cast(),
            y: self.y.cast(),
        }
    }
}

impl<P: Pixel, X: Pixel> From<(X, X)> for PhysicalPosition<P> {
    fn from((x, y): (X, X)) -> PhysicalPosition<P> {
        PhysicalPosition::new(x.cast(), y.cast())
    }
}

impl<P: Pixel, X: Pixel> From<PhysicalPosition<P>> for (X, X) {
    fn from(p: PhysicalPosition<P>) -> (X, X) {
        (p.x.cast(), p.y.cast())
    }
}

impl<P: Pixel, X: Pixel> From<[X; 2]> for PhysicalPosition<P> {
    fn from([x, y]: [X; 2]) -> PhysicalPosition<P> {
        PhysicalPosition::new(x.cast(), y.cast())
    }
}

impl<P: Pixel, X: Pixel> From<PhysicalPosition<P>> for [X; 2] {
    fn from(p: PhysicalPosition<P>) -> [X; 2] {
        [p.x.cast(), p.y.cast()]
    }
}

#[cfg(feature = "mint")]
impl<P: Pixel> From<mint::Point2<P>> for PhysicalPosition<P> {
    fn from(p: mint::Point2<P>) -> Self {
        Self::new(p.x, p.y)
    }
}

#[cfg(feature = "mint")]
impl<P: Pixel> From<PhysicalPosition<P>> for mint::Point2<P> {
    fn from(p: PhysicalPosition<P>) -> Self {
        mint::Point2 { x: p.x, y: p.y }
    }
}

/// A size represented in logical pixels.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LogicalSize<P> {
    pub width: P,
    pub height: P,
}

impl<P> LogicalSize<P> {
    #[inline]
    pub const fn new(width: P, height: P) -> Self {
        LogicalSize { width, height }
    }
}

impl<P: Pixel> LogicalSize<P> {
    #[inline]
    pub fn from_physical<T: Into<PhysicalSize<X>>, X: Pixel>(
        physical: T,
        scale_factor: f64,
    ) -> Self {
        physical.into().to_logical(scale_factor)
    }

    #[inline]
    pub fn to_physical<X: Pixel>(&self, scale_factor: f64) -> PhysicalSize<X> {
        assert!(validate_scale_factor(scale_factor));
        let width = self.width.into() * scale_factor;
        let height = self.height.into() * scale_factor;
        PhysicalSize::new(width, height).cast()
    }

    #[inline]
    pub fn cast<X: Pixel>(&self) -> LogicalSize<X> {
        LogicalSize {
            width: self.width.cast(),
            height: self.height.cast(),
        }
    }
}

impl<P: Pixel, X: Pixel> From<(X, X)> for LogicalSize<P> {
    fn from((x, y): (X, X)) -> LogicalSize<P> {
        LogicalSize::new(x.cast(), y.cast())
    }
}

impl<P: Pixel, X: Pixel> From<LogicalSize<P>> for (X, X) {
    fn from(s: LogicalSize<P>) -> (X, X) {
        (s.width.cast(), s.height.cast())
    }
}

impl<P: Pixel, X: Pixel> From<[X; 2]> for LogicalSize<P> {
    fn from([x, y]: [X; 2]) -> LogicalSize<P> {
        LogicalSize::new(x.cast(), y.cast())
    }
}

impl<P: Pixel, X: Pixel> From<LogicalSize<P>> for [X; 2] {
    fn from(s: LogicalSize<P>) -> [X; 2] {
        [s.width.cast(), s.height.cast()]
    }
}

#[cfg(feature = "mint")]
impl<P: Pixel> From<mint::Vector2<P>> for LogicalSize<P> {
    fn from(v: mint::Vector2<P>) -> Self {
        Self::new(v.x, v.y)
    }
}

#[cfg(feature = "mint")]
impl<P: Pixel> From<LogicalSize<P>> for mint::Vector2<P> {
    fn from(s: LogicalSize<P>) -> Self {
        mint::Vector2 {
            x: s.width,
            y: s.height,
        }
    }
}

/// A size represented in physical pixels.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PhysicalSize<P> {
    pub width: P,
    pub height: P,
}

impl<P> PhysicalSize<P> {
    #[inline]
    pub const fn new(width: P, height: P) -> Self {
        PhysicalSize { width, height }
    }
}

impl<P: Pixel> PhysicalSize<P> {
    #[inline]
    pub fn from_logical<T: Into<LogicalSize<X>>, X: Pixel>(
        logical: T,
        scale_factor: f64,
    ) -> Self {
        logical.into().to_physical(scale_factor)
    }

    #[inline]
    pub fn to_logical<X: Pixel>(&self, scale_factor: f64) -> LogicalSize<X> {
        assert!(validate_scale_factor(scale_factor));
        let width = self.width.into() / scale_factor;
        let height = self.height.into() / scale_factor;
        LogicalSize::new(width, height).cast()
    }

    #[inline]
    pub fn cast<X: Pixel>(&self) -> PhysicalSize<X> {
        PhysicalSize {
            width: self.width.cast(),
            height: self.height.cast(),
        }
    }
}

impl<P: Pixel, X: Pixel> From<(X, X)> for PhysicalSize<P> {
    fn from((x, y): (X, X)) -> PhysicalSize<P> {
        PhysicalSize::new(x.cast(), y.cast())
    }
}

impl<P: Pixel, X: Pixel> From<PhysicalSize<P>> for (X, X) {
    fn from(s: PhysicalSize<P>) -> (X, X) {
        (s.width.cast(), s.height.cast())
    }
}

impl<P: Pixel, X: Pixel> From<[X; 2]> for PhysicalSize<P> {
    fn from([x, y]: [X; 2]) -> PhysicalSize<P> {
        PhysicalSize::new(x.cast(), y.cast())
    }
}

impl<P: Pixel, X: Pixel> From<PhysicalSize<P>> for [X; 2] {
    fn from(s: PhysicalSize<P>) -> [X; 2] {
        [s.width.cast(), s.height.cast()]
    }
}

#[cfg(feature = "mint")]
impl<P: Pixel> From<mint::Vector2<P>> for PhysicalSize<P> {
    fn from(v: mint::Vector2<P>) -> Self {
        Self::new(v.x, v.y)
    }
}

#[cfg(feature = "mint")]
impl<P: Pixel> From<PhysicalSize<P>> for mint::Vector2<P> {
    fn from(s: PhysicalSize<P>) -> Self {
        mint::Vector2 {
            x: s.width,
            y: s.height,
        }
    }
}

/// A size that's either physical or logical.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Size {
    Physical(PhysicalSize<u32>),
    Logical(LogicalSize<f64>),
}

impl Size {
    pub fn new<S: Into<Size>>(size: S) -> Size {
        size.into()
    }

    pub fn to_logical<P: Pixel>(&self, scale_factor: f64) -> LogicalSize<P> {
        match *self {
            Size::Physical(size) => size.to_logical(scale_factor),
            Size::Logical(size) => size.cast(),
        }
    }

    pub fn to_physical<P: Pixel>(&self, scale_factor: f64) -> PhysicalSize<P> {
        match *self {
            Size::Physical(size) => size.cast(),
            Size::Logical(size) => size.to_physical(scale_factor),
        }
    }

    pub fn clamp<S: Into<Size>>(
        input: S,
        min: S,
        max: S,
        scale_factor: f64,
    ) -> Size {
        let (input, min, max) = (
            input.into().to_physical::<f64>(scale_factor),
            min.into().to_physical::<f64>(scale_factor),
            max.into().to_physical::<f64>(scale_factor),
        );

        let clamp = |input: f64, min: f64, max: f64| {
            if input < min {
                min
            } else if input > max {
                max
            } else {
                input
            }
        };

        let width = clamp(input.width, min.width, max.width);
        let height = clamp(input.height, min.height, max.height);

        PhysicalSize::new(width, height).into()
    }
}

impl<P: Pixel> From<PhysicalSize<P>> for Size {
    #[inline]
    fn from(size: PhysicalSize<P>) -> Size {
        Size::Physical(size.cast())
    }
}

impl<P: Pixel> From<LogicalSize<P>> for Size {
    #[inline]
    fn from(size: LogicalSize<P>) -> Size {
        Size::Logical(size.cast())
    }
}

/// A position that's either physical or logical.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Position {
    Physical(PhysicalPosition<i32>),
    Logical(LogicalPosition<f64>),
}

impl Position {
    pub fn new<S: Into<Position>>(position: S) -> Position {
        position.into()
    }

    pub fn to_logical<P: Pixel>(
        &self,
        scale_factor: f64,
    ) -> LogicalPosition<P> {
        match *self {
            Position::Physical(position) => position.to_logical(scale_factor),
            Position::Logical(position) => position.cast(),
        }
    }

    pub fn to_physical<P: Pixel>(
        &self,
        scale_factor: f64,
    ) -> PhysicalPosition<P> {
        match *self {
            Position::Physical(position) => position.cast(),
            Position::Logical(position) => position.to_physical(scale_factor),
        }
    }
}

impl<P: Pixel> From<PhysicalPosition<P>> for Position {
    #[inline]
    fn from(position: PhysicalPosition<P>) -> Position {
        Position::Physical(position.cast())
    }
}

impl<P: Pixel> From<LogicalPosition<P>> for Position {
    #[inline]
    fn from(position: LogicalPosition<P>) -> Position {
        Position::Logical(position.cast())
    }
}
