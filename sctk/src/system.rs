//! Access the native system.
use crate::runtime::command::{self, Command};
use crate::runtime::system::{Action, Information};
use iced_graphics::compositor;

/// Query for available system information.
pub fn fetch_information<Message>(
    f: impl Fn(Information) -> Message + Send + 'static,
) -> Command<Message> {
    Command::single(command::Action::System(Action::QueryInformation(
        Box::new(f),
    )))
}

pub(crate) fn information(
    graphics_info: compositor::Information,
) -> Information {
    use sysinfo::{CpuExt, ProcessExt, System, SystemExt};
    let mut system = System::new_all();
    system.refresh_all();

    let cpu = system.global_cpu_info();

    let memory_used = sysinfo::get_current_pid()
        .and_then(|pid| system.process(pid).ok_or("Process not found"))
        .map(|process| process.memory())
        .ok();

    Information {
        system_name: system.name(),
        system_kernel: system.kernel_version(),
        system_version: system.long_os_version(),
        system_short_version: system.os_version(),
        cpu_brand: cpu.brand().into(),
        cpu_cores: system.physical_core_count(),
        memory_total: system.total_memory(),
        memory_used,
        graphics_adapter: graphics_info.adapter,
        graphics_backend: graphics_info.backend,
    }
}
