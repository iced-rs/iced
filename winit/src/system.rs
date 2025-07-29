//! Access the native system.
use crate::graphics::compositor;
use crate::runtime::system::{Action, Information};
use crate::runtime::{self, Task};
use std::sync::OnceLock;

/// Query for available system information.
pub fn fetch_information() -> Task<Information> {
    runtime::task::oneshot(|channel| {
        runtime::Action::System(Action::QueryInformation(channel))
    })
}

static STATIC_SYSTEM_INFO: OnceLock<StaticSystemInfo> = OnceLock::new();

struct StaticSystemInfo {
    system_name: Option<String>,
    system_kernel: Option<String>,
    system_version: Option<String>,
    system_short_version: Option<String>,
    cpu_brand: String,
    cpu_cores: Option<usize>,
}

impl StaticSystemInfo {
    fn new() -> Self {
        use sysinfo::{CpuRefreshKind, RefreshKind, System};

        let system = System::new_with_specifics(
            RefreshKind::nothing().with_cpu(CpuRefreshKind::nothing()),
        );

        let cpu_brand = system
            .cpus()
            .first()
            .map(sysinfo::Cpu::brand)
            .filter(|brand| !brand.is_empty())
            .map(String::from)
            .unwrap_or_else(|| "Unknown".to_string());

        Self {
            system_name: System::name(),
            system_kernel: System::kernel_version(),
            system_version: System::long_os_version(),
            system_short_version: System::os_version(),
            cpu_brand,
            cpu_cores: System::physical_core_count(),
        }
    }
}

pub(crate) fn information(
    graphics_info: compositor::Information,
) -> Information {
    use sysinfo::{
        MemoryRefreshKind, Process, ProcessRefreshKind, ProcessesToUpdate,
        RefreshKind, System,
    };

    let mut system = System::new_with_specifics(
        RefreshKind::nothing()
            .with_memory(MemoryRefreshKind::nothing().with_ram()),
    );

    let memory_used = sysinfo::get_current_pid()
        .and_then(|pid| {
            let _ = system.refresh_processes_specifics(
                ProcessesToUpdate::Some(&[pid]),
                true,
                ProcessRefreshKind::nothing().with_memory(),
            );
            system.process(pid).ok_or("Process not found")
        })
        .map(Process::memory)
        .ok();

    let static_info = STATIC_SYSTEM_INFO.get_or_init(StaticSystemInfo::new);

    Information {
        system_name: static_info.system_name.clone(),
        system_kernel: static_info.system_kernel.clone(),
        system_version: static_info.system_version.clone(),
        system_short_version: static_info.system_short_version.clone(),
        cpu_brand: static_info.cpu_brand.clone(),
        cpu_cores: static_info.cpu_cores,
        memory_total: system.total_memory(),
        memory_used,
        graphics_adapter: graphics_info.adapter,
        graphics_backend: graphics_info.backend,
    }
}
