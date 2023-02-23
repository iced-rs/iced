/// The platform specific window settings of an application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlatformSpecific;

impl From<PlatformSpecific> for iced_winit::settings::PlatformSpecific {
    fn from(_: PlatformSpecific) -> Self {
        Default::default()
    }
}
