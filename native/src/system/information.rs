/// Contains informations about the system (e.g. system name, processor, memory, graphics adapter).
#[derive(Debug)]
pub struct Information {
    system_name: String,
    system_kernel: String,
    system_version: String,
    cpu_brand: String,
    cpu_vendor: String,
    cpu_name: String,
    cpu_cores: String,
    memory_total: String,
}
