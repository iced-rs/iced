use crate::time::Instant;

/// A request to redraw a window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RedrawRequest {
    /// Redraw the next frame.
    NextFrame,

    /// Redraw at the given time.
    At(Instant),

    /// No redraw is needed.
    Wait,
}

impl From<Instant> for RedrawRequest {
    fn from(time: Instant) -> Self {
        Self::At(time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::Duration;

    #[test]
    fn ordering() {
        let now = Instant::now();
        let later = now + Duration::from_millis(10);

        assert_eq!(RedrawRequest::NextFrame, RedrawRequest::NextFrame);
        assert_eq!(RedrawRequest::At(now), RedrawRequest::At(now));

        assert!(RedrawRequest::NextFrame < RedrawRequest::At(now));
        assert!(RedrawRequest::At(now) > RedrawRequest::NextFrame);
        assert!(RedrawRequest::At(now) < RedrawRequest::At(later));
        assert!(RedrawRequest::At(later) > RedrawRequest::At(now));

        assert!(RedrawRequest::NextFrame <= RedrawRequest::NextFrame);
        assert!(RedrawRequest::NextFrame <= RedrawRequest::At(now));
        assert!(RedrawRequest::At(now) >= RedrawRequest::NextFrame);
        assert!(RedrawRequest::At(now) <= RedrawRequest::At(now));
        assert!(RedrawRequest::At(now) <= RedrawRequest::At(later));
        assert!(RedrawRequest::At(later) >= RedrawRequest::At(now));

        assert!(RedrawRequest::Wait > RedrawRequest::NextFrame);
        assert!(RedrawRequest::Wait > RedrawRequest::At(later));
    }
}
