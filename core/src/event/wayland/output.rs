use sctk::output::OutputInfo;

/// output events
#[derive(Debug, Clone)]
pub enum OutputEvent {
    /// created output
    Created(Option<OutputInfo>),
    /// removed output
    Removed,
    /// Output Info
    InfoUpdate(OutputInfo),
}

impl Eq for OutputEvent {}

impl PartialEq for OutputEvent {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Created(l0), Self::Created(r0)) => {
                if let Some((l0, r0)) = l0.as_ref().zip(r0.as_ref()) {
                    l0.id == r0.id && l0.make == r0.make && l0.model == r0.model
                } else {
                    l0.is_none() && r0.is_none()
                }
            }
            (Self::InfoUpdate(l0), Self::InfoUpdate(r0)) => {
                l0.id == r0.id && l0.make == r0.make && l0.model == r0.model
            }
            _ => {
                core::mem::discriminant(self) == core::mem::discriminant(other)
            }
        }
    }
}
