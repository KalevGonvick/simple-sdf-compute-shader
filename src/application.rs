use std::collections::HashMap;
use std::sync::Arc;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::descriptor_set::allocator::{StandardDescriptorSetAllocator};
use vulkano_util::context::{VulkanoConfig, VulkanoContext};
use vulkano_util::window::{VulkanoWindows, WindowDescriptor};
use winit::event_loop::EventLoop;
use winit::window::WindowId;
use crate::signed_distance_function_renderer::{SimpleVulkanRendererRenderPipeline};


pub struct Application {
    pub context: VulkanoContext,
    pub windows: VulkanoWindows,
    pub command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    pub descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    pub pipelines: HashMap<WindowId, SimpleVulkanRendererRenderPipeline>,
}

impl Application {

    pub fn open_new_window(
        &mut self,
        event_loop: &EventLoop<()>,
        window_descriptor: WindowDescriptor,
    ) {
        let new_window = self.windows.create_window(
            event_loop,
            &self.context,
            &window_descriptor,
            |_| {},
        );


        self.pipelines.insert(
            new_window,
            SimpleVulkanRendererRenderPipeline::new(
                self,
                self.context.graphics_queue().clone(),
                self.context.graphics_queue().clone(),
                [window_descriptor.width as u32, window_descriptor.height as u32],
                self.windows.get_primary_renderer().unwrap().swapchain_format()
            ));
    }
}

impl Default for Application {
    fn default() -> Self {
        let context = VulkanoContext::new(VulkanoConfig::default());
        let standard_command_buffer_allocator = StandardCommandBufferAllocator::new(context.device().clone(), Default::default());
        let command_buffer_allocator = Arc::new(standard_command_buffer_allocator);

        let standard_descriptor_set_allocator = StandardDescriptorSetAllocator::new(context.device().clone());
        let descriptor_set_allocator = Arc::new(standard_descriptor_set_allocator);

        Application {
            context,
            windows: VulkanoWindows::default(),
            command_buffer_allocator,
            descriptor_set_allocator,
            pipelines: HashMap::new(),
        }
    }
}