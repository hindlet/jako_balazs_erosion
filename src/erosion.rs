use std::{sync::Arc, time::Duration};

use maths::{Camera, Matrix4};
use vulkano::{buffer::{allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo}, Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer}, command_buffer::{allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents}, descriptor_set::{allocator::StandardDescriptorSetAllocator, DescriptorSet, WriteDescriptorSet}, image::view::ImageView, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator}, pipeline::{graphics::{color_blend::{ColorBlendAttachmentState, ColorBlendState}, input_assembly::InputAssemblyState, multisample::MultisampleState, rasterization::RasterizationState, vertex_input::{Vertex, VertexDefinition}, viewport::{Viewport, ViewportState}, GraphicsPipelineCreateInfo}, layout::PipelineDescriptorSetLayoutCreateInfo, DynamicState, GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout, PipelineShaderStageCreateInfo}, render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass}, sync::GpuFuture};
use vulkano_util::{context::VulkanoContext, window::VulkanoWindows};
use winit::{application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop, window::WindowId};




mod erosion_shader {
    vulkano_shaders::shader!{
        ty: "compute",
        path: "src/erosion.glsl",
    }
}

mod vs {
    vulkano_shaders::shader!{
        ty: "vertex",
        path: "src/vert.glsl"
    }
}

mod fs {
    vulkano_shaders::shader!{
        ty: "fragment",
        path: "src/frag.glsl"
    }
}




pub struct ErosionApp {
    context: VulkanoContext,
    windows: VulkanoWindows,

    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    uniform_buffer_allocator: SubbufferAllocator,

    camera: Camera<winit::keyboard::KeyCode>,
    render_context: Option<RenderContext>,

    width: usize,
    height: usize,
    mesh_vertices: Subbuffer<[MeshVertex]>,
    mesh_indices: Subbuffer<[u32]>
}

struct RenderContext {
    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,
    pipeline: Arc<GraphicsPipeline>,
    viewport: Viewport,
}

impl ErosionApp {
    pub fn new(
        map_width: usize,
        map_height: usize,
        camera: Camera<winit::keyboard::KeyCode>
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

        let memory_allocator =Arc::new(StandardMemoryAllocator::new_default(context.device().clone()));

        let uniform_buffer_allocator = SubbufferAllocator::new(
            memory_allocator.clone(),
            SubbufferAllocatorCreateInfo {
                buffer_usage: BufferUsage::UNIFORM_BUFFER,
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            }
        );

        let (mesh_vertices, mesh_indices) = gen_mesh_vertices(map_width, map_height, 1.0, memory_allocator.clone());


        Self {
            context,
            windows: VulkanoWindows::default(),
            command_buffer_allocator,
            descriptor_set_allocator,
            render_context: None,
            camera,
            memory_allocator,
            uniform_buffer_allocator,
            mesh_vertices,
            mesh_indices,
            width: map_width,
            height: map_height
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

        self.windows.create_window(event_loop, &self.context, &Default::default(), |_| {});
        let window_renderer = self.windows.get_primary_renderer_mut().unwrap();
        let window_size = window_renderer.window().inner_size();


        let render_pass = vulkano::single_pass_renderpass!(
            self.context.device().clone(),
            attachments: {
                color: {
                    format: window_renderer.swapchain_format(),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store
                },
            },
            pass: {
                color: [color],
                depth_stencil: {},
            },
        ).unwrap();

        let framebuffers = window_size_dependent_setup(window_renderer.swapchain_image_views(), &render_pass);

        let pipeline = {
            let vs = vs::load(self.context.device().clone())
                .unwrap()
                .entry_point("main")
                .unwrap();
            let fs = fs::load(self.context.device().clone())
                .unwrap()
                .entry_point("main")
                .unwrap();


            let vertex_input_state = [MeshVertex::per_vertex(), VertexHeight::per_vertex()].definition(&vs).unwrap();


            let stages = [
                PipelineShaderStageCreateInfo::new(vs),
                PipelineShaderStageCreateInfo::new(fs)
            ];

            let layout = PipelineLayout::new(
                self.context.device().clone(),
                PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                    .into_pipeline_layout_create_info(self.context.device().clone())
                    .unwrap()
            ).unwrap();

            let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

            GraphicsPipeline::new(
                self.context.device().clone(),
                None,
                GraphicsPipelineCreateInfo {
                    stages: stages.into_iter().collect(),
                    // How vertex data is read from the vertex buffers into the vertex shader.
                    vertex_input_state: Some(vertex_input_state),
                    // How vertices are arranged into primitive shapes. The default primitive shape
                    // is a triangle.
                    input_assembly_state: Some(InputAssemblyState::default()),
                    // How primitives are transformed and clipped to fit the framebuffer. We use a
                    // resizable viewport, set to draw over the entire window.
                    viewport_state: Some(ViewportState::default()),
                    // How polygons are culled and converted into a raster of pixels. The default
                    // value does not perform any culling.
                    rasterization_state: Some(RasterizationState::default()),
                    // How multiple fragment shader samples are converted to a single pixel value.
                    // The default value does not perform any multisampling.
                    multisample_state: Some(MultisampleState::default()),
                    // How pixel values are combined with the values already present in the
                    // framebuffer. The default value overwrites the old value with the new one,
                    // without any blending.
                    color_blend_state: Some(ColorBlendState::with_attachment_states(
                        subpass.num_color_attachments(),
                        ColorBlendAttachmentState::default(),
                    )),
                    // Dynamic states allows us to specify parts of the pipeline settings when
                    // recording the command buffer, before we perform drawing. Here, we specify
                    // that the viewport should be dynamic.
                    dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                    subpass: Some(subpass.into()),
                    ..GraphicsPipelineCreateInfo::layout(layout)
                },
            ).unwrap()
        };

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: window_size.into(),
            depth_range: 0.0..=1.0
        };

        self.render_context = Some(RenderContext {
            render_pass,
            framebuffers,
            pipeline,
            viewport
        });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let window_renderer = self.windows.get_primary_renderer_mut().unwrap();
        let rcx = self.render_context.as_mut().unwrap();

        match event {

            WindowEvent::CloseRequested => {
                event_loop.exit();
            },

            WindowEvent::Resized(_) => {
                window_renderer.resize();
            },


            WindowEvent::RedrawRequested => {
                let window_size = window_renderer.window().inner_size();


                // skip if size 0
                if window_size.width == 0 || window_size.height == 0 {
                    return;
                }

                // generate uniform buffer
                let uniform_buffer = {
                    let view = self.camera.get_view_matrix();
                    let proj = Matrix4::persective_matrix(std::f32::consts::FRAC_PI_2, window_size.width as f32 / window_size.height as f32, 0.01, 100.0);

                    let uniform_data = vs::Data {
                        view: view.into(),
                        proj: proj.into()
                    };

                    let buffer = self.uniform_buffer_allocator.allocate_sized().unwrap();
                    *buffer.write().unwrap() = uniform_data;

                    buffer
                };




                // get previous frame end from window render
                let previous_frame_end = window_renderer
                    .acquire(Some(Duration::from_millis(1000)), |swapchain_images| {

                        rcx.framebuffers = window_size_dependent_setup(swapchain_images, &rcx.render_pass);
                        rcx.viewport.extent = window_size.into();
                    })
                    .unwrap();



                let mut builder = AutoCommandBufferBuilder::primary(
                    self.command_buffer_allocator.clone(),
                    self.context.graphics_queue().queue_family_index(),
                    CommandBufferUsage::OneTimeSubmit
                ).unwrap();


                // build descriptor set
                let layout = &rcx.pipeline.layout().set_layouts()[0];
                let descriptor_set = DescriptorSet::new(
                    self.descriptor_set_allocator.clone(),
                    layout.clone(),
                    [WriteDescriptorSet::buffer(0, uniform_buffer)],
                    []
                ).unwrap();


                builder
                    .begin_render_pass(
                        RenderPassBeginInfo {
                            clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into())],
                            
                            ..RenderPassBeginInfo::framebuffer(
                                rcx.framebuffers[window_renderer.image_index() as usize].clone(),
                            )
                        },
                        SubpassBeginInfo {
                            contents: SubpassContents::Inline,
                            ..Default::default()
                        }
                    )
                    .unwrap()
                    .bind_pipeline_graphics(rcx.pipeline.clone())
                    .unwrap()
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        rcx.pipeline.layout().clone(),
                        0,
                        descriptor_set
                    )
                    .unwrap();
                


                builder.end_render_pass(Default::default()).unwrap();

                let command_buffer = builder.build().unwrap();

                let future = previous_frame_end
                    .then_execute(self.context.graphics_queue().clone(), command_buffer)
                    .unwrap()
                    .boxed();

                window_renderer.present(future, false);


            },

            _ => {}
        }
    }


}



#[derive(BufferContents, Vertex)]
#[repr(C)]
struct MeshVertex {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2]
}

#[derive(BufferContents, Vertex)]
#[repr(C)]
struct VertexHeight {
    #[format(R32_SFLOAT)]
    height: f32
}



fn gen_mesh_vertices(
    width: usize,
    height: usize,
    gap: f32,

    allocator: Arc<StandardMemoryAllocator>
) -> (Subbuffer<[MeshVertex]>, Subbuffer<[u32]>){
    let mut mesh_vertices = Vec::with_capacity(width * height);
    let mut mesh_indices = Vec::with_capacity((width - 1) * (height - 1) * 6);
    let u32_height = height as u32;

    for x in 0..width {
        for y in 0..height {
            mesh_vertices.push(MeshVertex{position: [x as f32 * gap, y as f32 * gap]});

            if x != (width - 1) && y != (height - 1) {
                let curr_index = (mesh_vertices.len() - 1) as u32;
                mesh_indices.push(curr_index);
                mesh_indices.push(curr_index + 1);
                mesh_indices.push(curr_index + u32_height + 1);
                mesh_indices.push(curr_index);
                mesh_indices.push(curr_index + u32_height + 1);
                mesh_indices.push(curr_index + u32_height);
            } 

        }
    }

    let vertex_buffer = Buffer::from_iter(
        allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::VERTEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        mesh_vertices
    ).unwrap();

    let index_buffer = Buffer::from_iter(
        allocator,
        BufferCreateInfo {
            usage: BufferUsage::INDEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        mesh_indices
    ).unwrap();

    (vertex_buffer, index_buffer)

}   



fn window_size_dependent_setup(
    swapchain_images: &[Arc<ImageView>],
    render_pass: &Arc<RenderPass>,
) -> Vec<Arc<Framebuffer>> {
    swapchain_images
        .iter()
        .map(|swapchain_image| {
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![swapchain_image.clone()],
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>()
}
