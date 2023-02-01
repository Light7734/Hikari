use crate::vec3::Vec3;
use bytemuck::{Pod, Zeroable};
use display_json::DebugAsJsonPretty;
use pbr::ProgressBar;
use serde::Serialize;
use std::io::Stdout;
use std::sync::Arc;

use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, Queue,
        QueueCreateInfo,
    },
    instance::{Instance, InstanceCreateInfo},
    memory::allocator::StandardMemoryAllocator,
    pipeline::{ComputePipeline, Pipeline, PipelineBindPoint},
    sync::{self, GpuFuture},
    VulkanLibrary,
};

#[derive(Copy, Clone, Zeroable, Pod, Default, Serialize, DebugAsJsonPretty)]
#[repr(C)]
pub struct Camera {
    pub origin: Vec3,
    pub _0: f32,
    pub lower_left_corner: Vec3,
    pub _1: f32,
    pub horizontal: Vec3,
    pub _2: f32,
    pub vertical: Vec3,
    pub _3: f32,
    pub up: Vec3,
    pub _4: f32,
    pub u: Vec3,
    pub _5: f32,
    pub v: Vec3,
    pub _6: f32,
    pub w: Vec3,
    pub _7: f32,
    pub lens_radius: f32,
}

#[derive(Pod, Zeroable, Copy, Clone, Default, DebugAsJsonPretty, Serialize)]
#[repr(C)]
pub struct Config {
    pub num_spheres: u32,
    pub sample_count: u32,
    pub max_bounces: u32,
    pub width: u32,
    pub height: u32,

    pub _0: f32,
    pub _1: f32,
    pub _2: f32,
    pub camera: Camera,
}

#[derive(Pod, Zeroable, Copy, Clone, Default)]
#[repr(C)]
pub struct Sphere {
    pub radius: f32,
    pub mat_type: u32,
    pub fuzz_or_ir: f32,
    pub _0: f32,

    pub albedo: Vec3,
    pub _1: f32,

    pub center: Vec3,
    pub _2: f32,
}

pub struct Raytracer {
    device: Arc<Device>,
    queue: Arc<Queue>,
    command_buffer_allocator: StandardCommandBufferAllocator,
    pipeline: Arc<ComputePipeline>,
    descriptor_set: Arc<PersistentDescriptorSet>,

    data_buffer: Arc<CpuAccessibleBuffer<[u32]>>,
    config_buffer: Arc<CpuAccessibleBuffer<Config>>,
    scene_buffer: Arc<CpuAccessibleBuffer<[Sphere]>>,

    total_pixels: u32,
    samples_per_dispatch: u32,
    pixels_per_dispatch: u32,

    progress_bar: pbr::ProgressBar<Stdout>,
}

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "./src/compute.glsl",
        types_meta: {
            use bytemuck::{Pod,Zeroable};
            #[derive(Clone, Copy, Zeroable, Pod)]
        },
    }
}

impl Raytracer {
    pub fn new(config: Config, spheres: Vec<Sphere>) -> Raytracer {
        // Create instance
        let library = VulkanLibrary::new().unwrap();
        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                // Enable enumerating devices that use non-conformant vulkan implementations. (ex. MoltenVK)
                enumerate_portability: true,
                ..Default::default()
            },
        )
        .unwrap();

        // Choose which physical device to use
        let device_extensions = DeviceExtensions {
            khr_storage_buffer_storage_class: true,
            ..DeviceExtensions::empty()
        };
        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()
            .unwrap()
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter_map(|p| {
                // The Vulkan specs guarantee that a compliant implementation must provide at least one queue
                // that supports compute operations.
                p.queue_family_properties()
                    .iter()
                    .position(|q| q.queue_flags.compute)
                    .map(|i| (p, i as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            })
            .unwrap();

        println!(
            "Using device: {} (type: {:?})",
            physical_device.properties().device_name,
            physical_device.properties().device_type
        );

        // Initialize device
        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                enabled_extensions: device_extensions,
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
        .unwrap();

        // Pick the first queue
        let queue = queues.next().unwrap();
        //
        // Create Allocators
        let memory_allocator = StandardMemoryAllocator::new_default(device.clone());
        let descriptor_set_allocator = StandardDescriptorSetAllocator::new(device.clone());
        let command_buffer_allocator =
            StandardCommandBufferAllocator::new(device.clone(), Default::default());

        // Create bufferrs
        let data_buffer: Arc<CpuAccessibleBuffer<[u32]>> = {
            unsafe {
                CpuAccessibleBuffer::uninitialized_array(
                    &memory_allocator,
                    (config.width * config.height * 3) as u64,
                    BufferUsage {
                        storage_buffer: true,
                        ..BufferUsage::empty()
                    },
                    false,
                )
                .unwrap()
            }
        };

        let config_buffer = {
            CpuAccessibleBuffer::from_data(
                &memory_allocator,
                BufferUsage {
                    storage_buffer: true,
                    ..BufferUsage::empty()
                },
                false,
                config,
            )
            .unwrap()
        };

        let scene_buffer = {
            CpuAccessibleBuffer::from_iter(
                &memory_allocator,
                BufferUsage {
                    storage_buffer: true,
                    ..BufferUsage::empty()
                },
                false,
                spheres,
            )
            .unwrap()
        };

        // Create shader & pipeline
        let pipeline = {
            let shader = cs::load(device.clone()).unwrap();
            ComputePipeline::new(
                device.clone(),
                shader.entry_point("main").unwrap(),
                &(),
                None,
                |_| {},
            )
            .unwrap()
        };

        // Bind buffers
        let layout = pipeline.layout().set_layouts().get(0).unwrap();
        let set = PersistentDescriptorSet::new(
            &descriptor_set_allocator,
            layout.clone(),
            [
                WriteDescriptorSet::buffer(0, data_buffer.clone()),
                WriteDescriptorSet::buffer(1, config_buffer.clone()),
                WriteDescriptorSet::buffer(2, scene_buffer.clone()),
            ],
        )
        .unwrap();

        let total_pixels = config.width * config.height;
        let samples_per_dispatch = (1024 * 10000) / config.max_bounces;
        let pixels_per_dispatch = samples_per_dispatch / config.sample_count;

        println!(
            "Dimensions: [{} x {}] -> {}",
            config.width, config.height, total_pixels
        );
        println!("Sample count: {}", config.sample_count);
        println!("Max bounces: {}", config.max_bounces);
        println!("Samples per dispatch: {}", samples_per_dispatch);
        println!("Pixels per dispatch: {}", pixels_per_dispatch);

        let mut progress_bar = ProgressBar::new((total_pixels / pixels_per_dispatch) as u64);
        progress_bar.format("╢▌▌░╟");
        return Raytracer {
            device: device.clone(),
            queue: queue.clone(),
            command_buffer_allocator: command_buffer_allocator,
            pipeline: pipeline.clone(),
            descriptor_set: set.clone(),

            data_buffer: data_buffer.clone(),
            config_buffer: config_buffer.clone(),
            scene_buffer: scene_buffer.clone(),

            total_pixels,
            samples_per_dispatch,
            pixels_per_dispatch,

            progress_bar,
        };
    }
    pub fn raytrace(&mut self) -> Vec<u32> {
        let mut i = 0;

        while i + self.pixels_per_dispatch < self.total_pixels {
            self.dispatch(i, self.pixels_per_dispatch);
            i += self.pixels_per_dispatch;
        }

        if i < self.total_pixels {
            self.dispatch(i, self.total_pixels - i);
        }

        self.progress_bar.finish_print("Finished");
        return self.data_buffer.read().unwrap().to_vec();
    }

    fn dispatch(&mut self, index: u32, num_pixels: u32) {
        let mut builder = match AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.queue.clone().queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        ) {
            Err(why) => panic!("Failed to one time submit: {}", why),
            Ok(val) => val,
        };

        builder
            .bind_pipeline_compute(self.pipeline.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                self.pipeline.layout().clone(),
                0,
                self.descriptor_set.clone(),
            )
            .push_constants(
                self.pipeline.layout().clone(),
                0,
                cs::ty::PushConstantData { index },
            )
            .dispatch([num_pixels, 1, 1])
            .unwrap();

        let command_buffer = builder.build().unwrap();

        let future = match sync::now(self.device.clone())
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
        {
            Err(why) => panic!("Failed to future: {}", why),
            Ok(future) => future,
        };
        match future.wait(None) {
            Err(why) => panic!("Flush error: {}", why),
            Ok(_) => {
                self.progress_bar.inc();
            }
        };
    }
}
