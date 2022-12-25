use crate::vec3::Vec3;
use std::sync::Arc;

use std::time;

use display_json::DebugAsJsonPretty;
use serde::Serialize;

use vulkano::{
    buffer::cpu_access::WriteLock,
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo,
    },
    instance::{Instance, InstanceCreateInfo},
    memory::allocator::StandardMemoryAllocator,
    pipeline::{ComputePipeline, Pipeline, PipelineBindPoint},
    sync::{self, GpuFuture},
    VulkanLibrary,
};

use bytemuck::{Pod, Zeroable};

#[derive(Copy, Clone, Zeroable, Pod, Default, Serialize, DebugAsJsonPretty)]
#[repr(C)]
pub struct Camera {
    pub origin: Vec3,
    pub _pad0: f32,
    pub lower_left_corner: Vec3,
    pub _pad1: f32,
    pub horizontal: Vec3,
    pub _pad2: f32,
    pub vertical: Vec3,
    pub _pad3: f32,
    pub up: Vec3,
    pub _pad4: f32,
    pub u: Vec3,
    pub _pad5: f32,
    pub v: Vec3,
    pub _pad6: f32,
    pub w: Vec3,
    pub _pad7: f32,
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

    pub _pad1: f32,
    pub _pad2: f32,
    pub _pad3: f32,
    pub camera: Camera,
}

#[derive(Pod, Zeroable, Copy, Clone, Default)]
#[repr(C)]
pub struct Sphere {
    pub radius: f32,
    pub mat_type: u32,
    pub fuzz_or_ir: f32,
    pub _pad0: f32,

    pub albedo: Vec3,
    pub _pad1: f32,

    pub center: Vec3,
    pub _pad2: f32,
}

pub struct Vulkan {}

impl Vulkan {
    pub fn raytrace(config: Config, spheres: Vec<Sphere>) -> Vec<u32> {
        // As with other examples, the first step is to create an instance.
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

        // Choose which physical device to use.
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

        // Now initializing the device.
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

        // Since we can request multiple queues, the `queues` variable is in fact an iterator. In this
        // example we use only one queue, so we just retrieve the first and only element of the
        // iterator and throw it away.
        let queue = queues.next().unwrap();

        // Now let's get to the actual example.
        //
        // What we are going to do is very basic: we are going to fill a buffer with 64k integers
        // and ask the GPU to multiply each of them by 12.
        //
        // GPUs are very good at parallel computations (SIMD-like operations), and thus will do this
        // much more quickly than a CPU would do. While a CPU would typically multiply them one by one
        // or four by four, a GPU will do it by groups of 32 or 64.
        //
        // Note however that in a real-life situation for such a simple operation the cost of
        // accessing memory usually outweighs the benefits of a faster calculation. Since both the CPU
        // and the GPU will need to access data, there is no other choice but to transfer the data
        // through the slow PCI express bus.

        // We need to create the compute pipeline that describes our operation.
        //
        // If you are familiar with graphics pipeline, the principle is the same except that compute
        // pipelines are much simpler to create.
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

        let memory_allocator = StandardMemoryAllocator::new_default(device.clone());
        let descriptor_set_allocator = StandardDescriptorSetAllocator::new(device.clone());
        let command_buffer_allocator =
            StandardCommandBufferAllocator::new(device.clone(), Default::default());

        // We start by creating the buffer that will store the data.
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

        // In order to let the shader access the buffer, we need to build a *descriptor set* that
        // contains the buffer.
        //
        // The resources that we bind to the descriptor set must match the resources expected by the
        // pipeline which we pass as the first parameter.
        //
        // If you want to run the pipeline on multiple different buffers, you need to create multiple
        // descriptor sets that each contain the buffer you want to run the shader on.
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

        WriteDescriptorSet::buffer(0, data_buffer.clone());

        let rows_per_dispatch = 1;
        for i in 0..(config.height / rows_per_dispatch + 1) {
            let push_constants = cs::ty::PushConstantData {
                y_offset: i * rows_per_dispatch,
            };

            // In order to execute our operation, we have to build a command buffer.
            let mut builder = match AutoCommandBufferBuilder::primary(
                &command_buffer_allocator,
                queue.clone().queue_family_index(),
                CommandBufferUsage::OneTimeSubmit,
            ) {
                Err(why) => panic!("Failed to one time submit: {}", why),
                Ok(val) => val,
            };
            builder
                // The command buffer only does one thing: execute the compute pipeline.
                // This is called a *dispatch* operation.
                //
                // Note that we clone the pipeline and the set. Since they are both wrapped around an
                // `Arc`, this only clones the `Arc` and not the whole pipeline or set (which aren't
                // cloneable anyway). In this example we would avoid cloning them since this is the last
                // time we use them, but in a real code you would probably need to clone them.
                .bind_pipeline_compute(pipeline.clone())
                .bind_descriptor_sets(
                    PipelineBindPoint::Compute,
                    pipeline.layout().clone(),
                    0,
                    set.clone(),
                )
                .push_constants(pipeline.layout().clone(), 0, push_constants)
                .dispatch([config.width, rows_per_dispatch, 1])
                .unwrap();
            // Finish building the command buffer by calling `build`.
            let command_buffer = builder.build().unwrap();

            // Let's execute this command buffer now.
            // To do so, we TODO: this is a bit clumsy, probably needs a shortcut

            let timer = time::Instant::now();
            let future = match sync::now(device.clone())
                .then_execute(queue.clone(), command_buffer)
                .unwrap()
                // This line instructs the GPU to signal a *fence* once the command buffer has finished
                // execution. A fence is a Vulkan object that allows the CPU to know when the GPU has
                // reached a certain point.
                // We need to signal a fence here because below we want to block the CPU until the GPU has
                // reached that point in the execution.
                .then_signal_fence_and_flush()
            {
                Err(why) => panic!("Failed to future: {}", why),
                Ok(future) => future,
            };

            // Blocks execution until the GPU has finished the operation. This method only exists on the
            // future that corresponds to a signalled fence. In other words, this method wouldn't be
            // available if we didn't call `.then_signal_fence_and_flush()` earlier.
            // The `None` parameter is an optional timeout.
            //
            // Note however that dropping the `future` variable (with `drop(future)` for example) would
            // block execution as well, and this would be the case even if we didn't call
            // `.then_signal_fence_and_flush()`.
            // Therefore the actual point of calling `.then_signal_fence_and_flush()` and `.wait()` is to
            // make things more explicit. In the future, if the Rust language gets linear types vulkano may
            // get modified so that only fence-signalled futures can get destroyed like this.
            match future.wait(None) {
                Err(why) => panic!("Flush error: {}", why),
                Ok(_) => {
                    println!(
                        "Row {}/{} took {}ms",
                        i,
                        config.height,
                        timer.elapsed().as_millis()
                    )
                }
            };
        }

        // Now that the GPU is done, the content of the buffer should have been modified. Let's
        // check it out.
        // The call to `read()` would return an error if the buffer was still in use by the GPU.

        return data_buffer.read().unwrap().to_vec();
    }
}
