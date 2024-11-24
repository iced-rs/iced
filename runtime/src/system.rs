//! Access the native system.
use crate::futures::futures::channel::oneshot;

/// An operation to be performed on the system.
#[derive(Debug)]
pub enum Action {
    /// Query system information and produce `T` with the result.
    QueryInformation(oneshot::Sender<Information>),
}

/// Contains information about the system (e.g. system name, processor, memory, graphics adapter).
#[derive(Clone, Debug)]
pub struct Information {
    /// The operating system name
    pub system_name: Option<String>,
    /// Operating system kernel version
    pub system_kernel: Option<String>,
    /// Long operating system version
    ///
    /// Examples:
    /// - MacOS 10.15 Catalina
    /// - Windows 10 Pro
    /// - Ubuntu 20.04 LTS (Focal Fossa)
    pub system_version: Option<String>,
    /// Short operating system version number
    pub system_short_version: Option<String>,
    /// Detailed processor model information
    pub cpu_brand: String,
    /// The number of physical cores on the processor
    pub cpu_cores: Option<usize>,
    /// Total RAM size, in bytes
    pub memory_total: u64,
    /// Memory used by this process, in bytes
    pub memory_used: Option<u64>,
    /// Underlying graphics backend for rendering
    pub graphics_backend: String,
    /// Model information for the active graphics adapter
    pub graphics_adapter: String,
}
