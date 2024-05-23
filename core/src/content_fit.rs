//! Control the fit of some content (like an image) within a space.
use crate::Size;

use std::fmt;

/// The strategy used to fit the contents of a widget to its bounding box.
///
/// Each variant of this enum is a strategy that can be applied for resolving
/// differences in aspect ratio and size between the image being displayed and
/// the space its being displayed in.
///
/// For an interactive demonstration of these properties as they are implemented
/// in CSS, see [Mozilla's docs][1], or run the `tour` example
///
/// [1]: https://developer.mozilla.org/en-US/docs/Web/CSS/object-fit
#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, Default)]
pub enum ContentFit {
    /// Scale as big as it can be without needing to crop or hide parts.
    ///
    /// The image will be scaled (preserving aspect ratio) so that it just fits
    /// within the window.  This won't distort the image or crop/hide any edges,
    /// but if the image doesn't fit perfectly, there may be whitespace on the
    /// top/bottom or left/right.
    ///
    /// This is a great fit for when you need to display an image without losing
    /// any part of it, particularly when the image itself is the focus of the
    /// screen.
    #[default]
    Contain,

    /// Scale the image to cover all of the bounding box, cropping if needed.
    ///
    /// This doesn't distort the image, and it ensures that the widget's area is
    /// completely covered, but it might crop off a bit of the edges of the
    /// widget, particularly when there is a big difference between the aspect
    /// ratio of the widget and the aspect ratio of the image.
    ///
    /// This is best for when you're using an image as a background, or to fill
    /// space, and any details of the image around the edge aren't too
    /// important.
    Cover,

    /// Distort the image so the widget is 100% covered without cropping.
    ///
    /// This stretches the image to fit the widget, without any whitespace or
    /// cropping. However, because of the stretch, the image may look distorted
    /// or elongated, particularly when there's a mismatch of aspect ratios.
    Fill,

    /// Don't resize or scale the image at all.
    ///
    /// This will not apply any transformations to the provided image, but also
    /// means that unless you do the math yourself, the widget's area will not
    /// be completely covered, or the image might be cropped.
    ///
    /// This is best for when you've sized the image yourself.
    None,

    /// Scale the image down if it's too big for the space, but never scale it
    /// up.
    ///
    /// This works much like [`Contain`](Self::Contain), except that if the
    /// image would have been scaled up, it keeps its original resolution to
    /// avoid the bluring that accompanies upscaling images.
    ScaleDown,
}

impl ContentFit {
    /// Attempt to apply the given fit for a content size within some bounds.
    ///
    /// The returned value is the recommended scaled size of the content.
    pub fn fit(&self, content: Size, bounds: Size) -> Size {
        let content_ar = content.width / content.height;
        let bounds_ar = bounds.width / bounds.height;

        match self {
            Self::Contain => {
                if bounds_ar > content_ar {
                    Size {
                        width: content.width * bounds.height / content.height,
                        ..bounds
                    }
                } else {
                    Size {
                        height: content.height * bounds.width / content.width,
                        ..bounds
                    }
                }
            }
            Self::Cover => {
                if bounds_ar < content_ar {
                    Size {
                        width: content.width * bounds.height / content.height,
                        ..bounds
                    }
                } else {
                    Size {
                        height: content.height * bounds.width / content.width,
                        ..bounds
                    }
                }
            }
            Self::Fill => bounds,
            Self::None => content,
            Self::ScaleDown => {
                if bounds_ar > content_ar && bounds.height < content.height {
                    Size {
                        width: content.width * bounds.height / content.height,
                        ..bounds
                    }
                } else if bounds.width < content.width {
                    Size {
                        height: content.height * bounds.width / content.width,
                        ..bounds
                    }
                } else {
                    content
                }
            }
        }
    }
}

impl fmt::Display for ContentFit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ContentFit::Contain => "Contain",
            ContentFit::Cover => "Cover",
            ContentFit::Fill => "Fill",
            ContentFit::None => "None",
            ContentFit::ScaleDown => "Scale Down",
        })
    }
}
