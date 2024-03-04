use crate::application::Application;
use std::sync::Arc;
use std::time::Instant;
use rand::Rng;
use vulkano::buffer::{BufferContents};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer,
};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::device::Queue;
use vulkano::format::Format;
use vulkano::image::{ImageAccess, ImageUsage, StorageImage};
use vulkano::memory::allocator::{ MemoryAllocator};
use vulkano::pipeline::{ComputePipeline, Pipeline, PipelineBindPoint};

use vulkano::sync::GpuFuture;
use vulkano_util::renderer::DeviceImageView;
use crate::render_pass::RenderPassPlaceOverFrame;

pub struct SimpleVulkanRendererComputePipeline {
    compute_queue: Arc<Queue>,
    initialize_compute_pipeline: Arc<ComputePipeline>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    time: Instant,
    image: DeviceImageView
}

impl SimpleVulkanRendererComputePipeline {
    pub fn new(
        app: &Application,
        compute_queue: Arc<Queue>,
        size: [u32; 2]
    ) -> SimpleVulkanRendererComputePipeline {

        let memory_allocator = app.context.memory_allocator();
        let initialize_compute_pipeline: Arc<ComputePipeline> = {
            let shader = triangle_sdf_compute::load(compute_queue.device().clone()).unwrap();
            ComputePipeline::new(
                compute_queue.device().clone(),
                shader.entry_point("main").unwrap(),
                &(),
                None,
                |_| {},
            ).unwrap()
        };

        let image = StorageImage::general_purpose_image_view(
            memory_allocator,
            compute_queue.clone(),
            size,
            Format::R8G8B8A8_UNORM,
            ImageUsage::SAMPLED | ImageUsage::STORAGE | ImageUsage::TRANSFER_DST,
        ).unwrap();

        SimpleVulkanRendererComputePipeline {
            compute_queue,
            initialize_compute_pipeline,
            time: Instant::now(),
            command_buffer_allocator: app.command_buffer_allocator.clone(),
            descriptor_set_allocator: app.descriptor_set_allocator.clone(),
            image,
        }
    }

    pub fn color_image(&self) -> DeviceImageView {
        self.image.clone()
    }

    pub fn compute(
        &mut self,
        before_future: Box<dyn GpuFuture>,
    ) -> Box<dyn GpuFuture> {

        let mut builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.compute_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit
        ).unwrap();

        self.dispatch(&mut builder);

        let command_buffer = builder.build().unwrap();
        let finished = before_future.then_execute(self.compute_queue.clone(), command_buffer).unwrap();
        let after_pipeline = finished.then_signal_fence_and_flush().unwrap().boxed();
        after_pipeline
    }

    fn dispatch(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer, Arc<StandardCommandBufferAllocator>>,
    ) {
        // Resize image if needed.
        let img_dims = self.image.image().dimensions().width_height();
        let pipeline_layout = self.initialize_compute_pipeline.layout();
        let desc_layout = pipeline_layout.set_layouts().get(0).unwrap();
        let persistent_descriptor_set = match PersistentDescriptorSet::new(
            &self.descriptor_set_allocator,
            desc_layout.clone(),
            [
                WriteDescriptorSet::image_view(0, self.image.clone())
            ],
        ) {
            Ok(x) => x,
            Err(e) => panic!("Failed to bind descriptor sets: {}", e),
        };

        let current_time = self.time.elapsed().as_secs_f32();

        let push_constants = triangle_sdf_compute::PushConstants {
            time: current_time,
        };

        let dispatch_count_x = img_dims[0] / 8;
        let dispatch_count_y = img_dims[1] / 8;

        builder.bind_pipeline_compute(self.initialize_compute_pipeline.clone())
            .bind_descriptor_sets(PipelineBindPoint::Compute, pipeline_layout.clone(), 0, persistent_descriptor_set)
            .push_constants(pipeline_layout.clone(), 0, push_constants)
            .dispatch([dispatch_count_x, dispatch_count_y, 1])
            .unwrap();
    }
}

pub struct SimpleVulkanRendererRenderPipeline {
    pub compute: SimpleVulkanRendererComputePipeline,
    pub place_over_frame: RenderPassPlaceOverFrame,
}

impl SimpleVulkanRendererRenderPipeline {
    pub fn new(
        app: &Application,
        compute_queue: Arc<Queue>,
        graphics_queue: Arc<Queue>,
        size: [u32; 2],
        swap_chain_format: Format
    ) -> SimpleVulkanRendererRenderPipeline {
        SimpleVulkanRendererRenderPipeline {
            compute: SimpleVulkanRendererComputePipeline::new(app, compute_queue, size),
            place_over_frame: RenderPassPlaceOverFrame::new(app, graphics_queue, swap_chain_format),
        }
    }
}

mod triangle_sdf_compute {
    vulkano_shaders::shader! {
        ty: "compute",
        include: ["src/shaders"],
        path: "src/shaders/shapes_cs.glsl",
    }
}