use wgpu::{Backends, Instance, InstanceFlags};

#[cfg(not(target_arch = "wasm32"))]
use dawn_rs::{InstanceDescriptor, InstanceFeatureName};

#[cfg(not(target_arch = "wasm32"))]
use dawn_wgpu::to_wgpu_instance;

pub async fn create_instance(backends: Backends, flags: InstanceFlags) -> Instance {
    #[cfg(target_arch = "wasm32")]
    {
        wgpu::util::new_instance_with_webgpu_detection(&wgpu::InstanceDescriptor {
            backends,
            flags,
            ..Default::default()
        })
        .await
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (backends, flags);

        let mut descriptor = InstanceDescriptor::new();
        descriptor.required_features = Some(vec![InstanceFeatureName::TimedWaitAny]);

        let instance = dawn_rs::Instance::new(Some(&descriptor));
        to_wgpu_instance(instance)
    }
}
