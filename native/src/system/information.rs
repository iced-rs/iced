/// Contains informations about the system (e.g. system name, processor, memory, graphics adapter).
#[derive(Debug)]
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
}
