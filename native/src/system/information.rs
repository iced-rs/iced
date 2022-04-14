/// Contains informations about the system (e.g. system name, processor, memory, graphics adapter).
#[derive(Clone, Debug)]
pub struct Information {
    /// Contains the system name.
    pub system_name: Option<String>,
    /// Contains the kernel version.
    pub system_kernel: Option<String>,
    /// Contains the systme version.
    pub system_version: Option<String>,
    /// Contains the processor brand.
    pub cpu_brand: String,
    /// Contains the number of physical cores on the processor.
    pub cpu_cores: Option<usize>,
    /// Contains the total RAM size in KB.
    pub memory_total: u64,
    /// Contains the system used RAM size in KB.
    pub memory_used: Option<u64>,
    /// Contains the graphics backend.
    pub graphics_backend: String,
    /// Contains the graphics adapter.
    pub graphics_adapter: String,
}
