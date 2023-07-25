use std::fmt;

use iced_futures::MaybeSend;


/// Get proxy
pub enum Action<T> {
    /// Query proxy
    QueryProxy(Box<dyn Closure<T>>)
}

///
pub trait Closure<T>: Fn(Box<dyn Proxy<T>>) -> T + MaybeSend {}


impl<T, O> Closure<O> for T
where T: Fn(Box<dyn Proxy<O>>) -> O + MaybeSend {}

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
            Self::QueryProxy(o) => {
                Action::QueryProxy(Box::new(move |s| f(o(s))))
            }
        }
    }
}


///
pub trait Proxy<T> {
    
}

impl<T> fmt::Debug for Action<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::QueryProxy(_) => write!(f, "Action::QueryProxy"),
        }
    }
}
