pub(crate) fn required_limits(adapter_limits: wgpu::Limits) -> wgpu::Limits {
    let mut limits = adapter_limits;

    limits.max_bind_groups = limits.max_bind_groups.min(2);
    limits.max_non_sampler_bindings = limits.max_non_sampler_bindings.min(2048);

    limits
}
