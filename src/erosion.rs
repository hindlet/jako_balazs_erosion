use std::sync::Arc;

use vulkano::{command_buffer::allocator::StandardCommandBufferAllocator, descriptor_set::allocator::StandardDescriptorSetAllocator};
use vulkano_util::{context::VulkanoContext, window::VulkanoWindows};
use winit::{application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop, window::WindowId};




mod erosion_shader {
    vulkano_shaders::shader!{
        ty: "compute",
        path: "src/erosion.glsl",
    }
}


pub struct ErosionApp {
    pub context: VulkanoContext,
    pub windows: VulkanoWindows,
    pub command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    pub descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
}

impl ErosionApp {
    pub fn new(

    ) -> Self {

        let context = VulkanoContext::default();
        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            context.device().clone(),
            Default::default()
        ));
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            context.device().clone(),
            Default::default()
        ));


        Self {
            context,
            windows: VulkanoWindows::default(),
            command_buffer_allocator,
            descriptor_set_allocator
        }
    }
}



impl ApplicationHandler for ErosionApp {
    fn resumed(
        &mut self,
        event_loop: &ActiveEventLoop
    ) {
        if let Some(primary_window_id) = self.windows.primary_window_id() {
            self.windows.remove_renderer(primary_window_id);
        }

        self.windows
            .create_window(event_loop, &self.context, &Default::default(), |_| {});
        let window_renderer = self.windows.get_primary_renderer_mut().unwrap();
        let window_size = window_renderer.window().inner_size();





    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        


        match event {

            WindowEvent::CloseRequested => {
                event_loop.exit();
            },

            _ => {}
        }
    }


}