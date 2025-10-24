use crate::core::{self, Size};
use crate::graphics::Shell;
use crate::image::atlas::{self, Atlas};

use std::collections::BTreeSet;
use std::sync::mpsc;
use std::thread;

#[derive(Debug)]
pub struct Cache {
    atlas: Atlas,
    #[cfg(feature = "image")]
    raster: Raster,
    #[cfg(feature = "svg")]
    vector: crate::image::vector::Cache,
    #[cfg(feature = "image")]
    jobs: mpsc::SyncSender<Job>,
    #[cfg(feature = "image")]
    work: mpsc::Receiver<Work>,
    #[cfg(feature = "image")]
    worker_: Option<thread::JoinHandle<()>>,
}

impl Cache {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        backend: wgpu::Backend,
        layout: wgpu::BindGroupLayout,
        shell: &Shell,
    ) -> Self {
        #[cfg(feature = "image")]
        let (worker, jobs, work) =
            Worker::new(device, queue, backend, layout.clone(), shell);

        #[cfg(feature = "image")]
        let handle = thread::spawn(move || worker.run());

        Self {
            atlas: Atlas::new(device, backend, layout),
            #[cfg(feature = "image")]
            raster: Raster {
                cache: crate::image::raster::Cache::default(),
                pending: BTreeSet::new(),
                jobs: jobs.clone(),
            },
            #[cfg(feature = "svg")]
            vector: crate::image::vector::Cache::default(),
            #[cfg(feature = "image")]
            jobs,
            #[cfg(feature = "image")]
            work,
            #[cfg(feature = "image")]
            worker_: Some(handle),
        }
    }

    #[cfg(feature = "image")]
    pub fn measure_image(&mut self, handle: &core::image::Handle) -> Size<u32> {
        self.receive();

        if let Some(memory) = load_image(
            &mut self.raster.cache,
            &mut self.raster.pending,
            &mut self.raster.jobs,
            handle,
        ) {
            return memory.dimensions();
        }

        Size::new(0, 0)
    }

    #[cfg(feature = "svg")]
    pub fn measure_svg(&mut self, handle: &core::svg::Handle) -> Size<u32> {
        // TODO: Concurrency
        self.vector.load(handle).viewport_dimensions()
    }

    #[cfg(feature = "image")]
    pub fn upload_raster(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        belt: &mut wgpu::util::StagingBelt,
        handle: &core::image::Handle,
    ) -> Option<(&atlas::Entry, &wgpu::BindGroup)> {
        use crate::image::raster::Memory;

        self.receive();

        let memory = load_image(
            &mut self.raster.cache,
            &mut self.raster.pending,
            &mut self.raster.jobs,
            handle,
        )?;

        if let Memory::Device { entry, bind_group } = memory {
            return Some((
                entry,
                bind_group.as_ref().unwrap_or(self.atlas.bind_group()),
            ));
        }

        let image = memory.host()?;

        const MAX_SYNC_SIZE: usize = 2 * 1024 * 1024;

        if image.len() < MAX_SYNC_SIZE {
            let entry = self.atlas.upload(
                device,
                encoder,
                belt,
                image.width(),
                image.height(),
                &image,
            )?;

            *memory = Memory::Device {
                entry,
                bind_group: None,
            };

            if let Memory::Device { entry, .. } = memory {
                return Some((entry, self.atlas.bind_group()));
            }
        }

        if !self.raster.pending.contains(&handle.id()) {
            let _ = self.jobs.send(Job::Upload {
                handle: handle.clone(),
                rgba: image.clone().into_raw(),
                width: image.width(),
                height: image.height(),
            });

            let _ = self.raster.pending.insert(handle.id());
        }

        None
    }

    #[cfg(feature = "svg")]
    pub fn upload_vector(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        belt: &mut wgpu::util::StagingBelt,
        handle: &core::svg::Handle,
        color: Option<core::Color>,
        size: [f32; 2],
        scale: f32,
    ) -> Option<(&atlas::Entry, &wgpu::BindGroup)> {
        // TODO: Concurrency
        self.vector
            .upload(
                device,
                encoder,
                belt,
                handle,
                color,
                size,
                scale,
                &mut self.atlas,
            )
            .map(|entry| (entry, self.atlas.bind_group()))
    }

    pub fn trim(&mut self) {
        #[cfg(feature = "image")]
        self.raster.cache.trim(&mut self.atlas, |bind_group| {
            let _ = self.jobs.send(Job::Drop(bind_group));
        });

        #[cfg(feature = "svg")]
        self.vector.trim(&mut self.atlas); // TODO: Concurrency
    }

    fn receive(&mut self) {
        use crate::image::raster::Memory;

        while let Ok(work) = self.work.try_recv() {
            match work {
                Work::Upload {
                    handle,
                    entry,
                    bind_group,
                } => {
                    self.raster.cache.insert(
                        &handle,
                        Memory::Device {
                            entry,
                            bind_group: Some(bind_group),
                        },
                    );

                    let _ = self.raster.pending.remove(&handle.id());
                }
                Work::Error { handle, error } => {
                    self.raster.cache.insert(&handle, Memory::error(error));
                }
            }
        }
    }
}

impl Drop for Cache {
    fn drop(&mut self) {
        // Stop worker gracefully
        let (sender, _) = mpsc::sync_channel(1);
        self.jobs = sender.clone();
        self.raster.jobs = sender;

        let _ = self.worker_.take().unwrap().join();
    }
}

#[cfg(feature = "image")]
#[derive(Debug)]
struct Raster {
    cache: crate::image::raster::Cache,
    pending: BTreeSet<core::image::Id>,
    jobs: mpsc::SyncSender<Job>,
}

#[cfg(feature = "image")]
fn load_image<'a>(
    cache: &'a mut crate::image::raster::Cache,
    pending: &mut BTreeSet<core::image::Id>,
    jobs: &mut mpsc::SyncSender<Job>,
    handle: &core::image::Handle,
) -> Option<&'a mut crate::image::raster::Memory> {
    use crate::image::raster::Memory;

    if !cache.contains(handle) {
        // Load RGBA handles synchronously, since it's very cheap
        if let core::image::Handle::Rgba { .. } = handle {
            cache.insert(handle, Memory::load(handle));
        } else {
            let _ = jobs.send(Job::Load(handle.clone()));
            let _ = pending.insert(handle.id());
        }
    }

    cache.get_mut(handle)
}

#[cfg(feature = "image")]
enum Job {
    Load(core::image::Handle),
    Upload {
        handle: core::image::Handle,
        rgba: core::image::Bytes,
        width: u32,
        height: u32,
    },
    Drop(wgpu::BindGroup),
}

#[cfg(feature = "image")]
enum Work {
    Upload {
        handle: core::image::Handle,
        entry: atlas::Entry,
        bind_group: wgpu::BindGroup,
    },
    Error {
        handle: core::image::Handle,
        error: crate::graphics::image::image_rs::error::ImageError,
    },
}

#[cfg(feature = "image")]
struct Worker {
    device: wgpu::Device,
    queue: wgpu::Queue,
    backend: wgpu::Backend,
    texture_layout: wgpu::BindGroupLayout,
    shell: Shell,
    belt: wgpu::util::StagingBelt,
    jobs: mpsc::Receiver<Job>,
    output: mpsc::SyncSender<Work>,
}

#[cfg(feature = "image")]
impl Worker {
    fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        backend: wgpu::Backend,
        texture_layout: wgpu::BindGroupLayout,
        shell: &Shell,
    ) -> (Self, mpsc::SyncSender<Job>, mpsc::Receiver<Work>) {
        let (jobs_sender, jobs_receiver) = mpsc::sync_channel(1_000);
        let (work_sender, work_receiver) = mpsc::sync_channel(1_000);

        (
            Self {
                device: device.clone(),
                queue: queue.clone(),
                backend,
                texture_layout,
                shell: shell.clone(),
                belt: wgpu::util::StagingBelt::new(4 * 1024 * 1024),
                jobs: jobs_receiver,
                output: work_sender,
            },
            jobs_sender,
            work_receiver,
        )
    }

    fn run(mut self) {
        while let Ok(job) = self.jobs.recv() {
            match job {
                Job::Load(handle) => {
                    match crate::graphics::image::load(&handle) {
                        Ok(image) => self.upload(
                            handle,
                            image.width(),
                            image.height(),
                            image.into_raw(),
                            Shell::invalidate_layout,
                        ),
                        Err(error) => {
                            let _ =
                                self.output.send(Work::Error { handle, error });
                        }
                    }
                }
                Job::Upload {
                    handle,
                    rgba,
                    width,
                    height,
                } => {
                    self.upload(
                        handle,
                        width,
                        height,
                        rgba,
                        Shell::request_redraw,
                    );
                }
                Job::Drop(bind_group) => {
                    drop(bind_group);
                }
            }
        }
    }

    fn upload(
        &mut self,
        handle: core::image::Handle,
        width: u32,
        height: u32,
        rgba: core::image::Bytes,
        callback: fn(&Shell),
    ) {
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("raster image upload"),
            },
        );

        let mut atlas = Atlas::with_size(
            &self.device,
            self.backend,
            self.texture_layout.clone(),
            width.max(height),
        );

        let Some(entry) = atlas.upload(
            &self.device,
            &mut encoder,
            &mut self.belt,
            width,
            height,
            &rgba,
        ) else {
            return;
        };

        let output = self.output.clone();
        let shell = self.shell.clone();

        self.belt.finish();
        let submission = self.queue.submit([encoder.finish()]);
        self.belt.recall();

        self.queue.on_submitted_work_done(move || {
            let _ = output.send(Work::Upload {
                handle,
                entry,
                bind_group: atlas.bind_group().clone(),
            });

            callback(&shell);
        });

        let _ = self
            .device
            .poll(wgpu::PollType::WaitForSubmissionIndex(submission));
    }
}
