use crate::system;

use iced_futures::MaybeSend;
use std::fmt;

/// An operation to be performed on the system.
pub enum Action<T> {
    /// Query system information and produce `T` with the result.
    QueryInformation(Box<dyn Closure<T>>),
}

pub trait Closure<T>: Fn(system::Information) -> T + MaybeSend {}

impl<T, O> Closure<O> for T where T: Fn(system::Information) -> O + MaybeSend {}

impl<T> Action<T> {
    /// Maps the output of a system [`Action`] using the provided closure.
    pub fn map<A>(
        self,
        f: impl Fn(T) -> A + 'static + MaybeSend + Sync,
    ) -> Action<A>
    where
        T: 'static,
    {
        match self {
            Self::QueryInformation(o) => {
                Action::QueryInformation(Box::new(move |s| f(o(s))))
            }
        }
    }
}

impl<T> fmt::Debug for Action<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::QueryInformation(_) => write!(f, "Action::QueryInformation"),
        }
    }
}
