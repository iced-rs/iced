#[cfg(not(target_arch = "wasm32"))]
mod platform {
    /// An extension trait that enforces `Send` only on native platforms.
    ///
    /// Useful to write cross-platform async code!
    pub trait MaybeSend: Send {}

    impl<T> MaybeSend for T where T: Send {}
}

#[cfg(target_arch = "wasm32")]
mod platform {
    /// An extension trait that enforces `Send` only on native platforms.
    ///
    /// Useful to write cross-platform async code!
    pub trait MaybeSend {}

    impl<T> MaybeSend for T {}
}

pub use platform::MaybeSend;
