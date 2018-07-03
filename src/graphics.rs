use vulkano;
use winit;
use image;
use vulkano::sync::GpuFuture;
use vulkano_win::VkSurfaceBuild;
use vulkano_win;
use specs;
use specs::Join;
use std::collections::HashMap;
use std::sync::Arc;
use alga::general::SubsetOf;

#[derive(Debug, Clone)]
struct Vertex {
    position: [f32; 2],
}
impl_vertex!(Vertex, position);

pub struct Graphics {
    surface: Arc<vulkano::swapchain::Surface<winit::Window>>,
    device: Arc<vulkano::device::Device>,
    queue: Arc<vulkano::device::Queue>,
    recreate_swapchain: bool,
    previous_frame_end: Box<vulkano::sync::GpuFuture>,
    swapchain: Arc<vulkano::swapchain::Swapchain<winit::Window>>,
    framebuffers: Option<Vec<Arc<vulkano::framebuffer::FramebufferAbstract + Sync + Send>>>,
    renderpass: Arc<vulkano::framebuffer::RenderPassAbstract + Sync + Send>,
    images: Vec<Arc<vulkano::image::SwapchainImage<winit::Window>>>,
    pipeline: Arc<vulkano::pipeline::GraphicsPipelineAbstract + Sync + Send>,
    vertex_buffer: Arc<vulkano::buffer::CpuAccessibleBuffer<[Vertex]>>,
    transform_buffer_pool: vulkano::buffer::CpuBufferPool<vs::ty::Transform>,
    view_buffer_pool: vulkano::buffer::CpuBufferPool<vs::ty::View>,
    textures: HashMap<::Image, (u32, u32, Arc<vulkano::image::ImageViewAccess + Sync + Send>)>,
    sets_pool: vulkano::descriptor::descriptor_set::FixedSizeDescriptorSetsPool<Arc<vulkano::descriptor::pipeline_layout::PipelineLayoutAbstract + Sync + Send>>,
    dimensions: [u32; 2],
    sampler: Arc<vulkano::sampler::Sampler>,
}

fn mat4(mat3: [[f32; 3]; 3]) -> [[f32; 4]; 4] {
    [
        [mat3[0][0], mat3[0][1], 0.0, mat3[0][2]],
        [mat3[1][0], mat3[1][1], 0.0, mat3[1][2]],
        [       0.0,        0.0, 1.0,        0.0],
        [mat3[2][0], mat3[2][1], 0.0, mat3[2][2]],
    ]
}

impl Graphics {
    pub fn new(events_loop: &winit::EventsLoop) -> Self {
        let extensions = vulkano_win::required_extensions();
        let instance = vulkano::instance::Instance::new(None, &extensions, &[])
            .expect("failed to create instance");

        let surface = winit::WindowBuilder::new()
            .with_title("Airjump Multi")
            .with_fullscreen(Some(events_loop.get_primary_monitor()))
            .build_vk_surface(&events_loop, instance.clone())
            .unwrap();

        surface.window().set_cursor(winit::MouseCursor::NoneCursor);

        let physical = vulkano::instance::PhysicalDevice::enumerate(&instance)
            .next()
            .expect("no device available");

        let dimensions;

        let queue = physical
            .queue_families()
            .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
            .expect("couldn't find a graphical queue family");

        let device_ext = vulkano::device::DeviceExtensions {
            khr_swapchain: true,
            ..vulkano::device::DeviceExtensions::none()
        };
        let (device, mut queues) = vulkano::device::Device::new(
            physical,
            physical.supported_features(),
            &device_ext,
            [(queue, 0.5)].iter().cloned(),
        ).expect("failed to create device");
        let queue = queues.next().unwrap();

        let (swapchain, images) = {
            let caps = surface
                .capabilities(physical)
                .expect("failed to get surface capabilities");

            dimensions = caps.current_extent.unwrap_or([1024, 768]);
            let usage = caps.supported_usage_flags;
            let alpha = caps.supported_composite_alpha.iter().next().unwrap();
            let format = caps.supported_formats.iter()
                .max_by_key(|format| {
                    match format {
                        (vulkano::format::Format::B8G8R8A8Unorm, vulkano::swapchain::ColorSpace::SrgbNonLinear) => 6,
                        (vulkano::format::Format::B8G8R8A8Srgb, vulkano::swapchain::ColorSpace::SrgbNonLinear) => 5,
                        (_, vulkano::swapchain::ColorSpace::SrgbNonLinear) => 4,
                        (_, vulkano::swapchain::ColorSpace::ExtendedSrgbLinear) => 3,
                        (_, vulkano::swapchain::ColorSpace::AdobeRgbNonLinear) => 2,
                        (_, vulkano::swapchain::ColorSpace::AdobeRgbLinear) => 1,
                        _ => 0,
                    }
                }).unwrap().0;

            vulkano::swapchain::Swapchain::new(
                device.clone(),
                surface.clone(),
                caps.min_image_count,
                format,
                dimensions,
                1,
                usage,
                &queue,
                vulkano::swapchain::SurfaceTransform::Identity,
                alpha,
                vulkano::swapchain::PresentMode::Fifo,
                true,
                None,
            ).expect("failed to create swapchain")
        };

        let vertex_buffer = vulkano::buffer::cpu_access::CpuAccessibleBuffer::<[Vertex]>::from_iter(
            device.clone(),
            vulkano::buffer::BufferUsage::all(),
            [
                Vertex {
                    position: [-0.5, -0.5],
                },
                Vertex {
                    position: [-0.5, 0.5],
                },
                Vertex {
                    position: [0.5, -0.5],
                },
                Vertex {
                    position: [0.5, 0.5],
                },
            ].iter()
                .cloned(),
        ).expect("failed to create buffer");

        let vs = vs::Shader::load(device.clone()).expect("failed to create shader module");
        let fs = fs::Shader::load(device.clone()).expect("failed to create shader module");

        let renderpass = Arc::new(
            single_pass_renderpass!(device.clone(),
                attachments: {
                    color: {
                        load: Clear,
                        store: Store,
                        format: swapchain.format(),
                        samples: 1,
                    }
                },
                pass: {
                    color: [color],
                    depth_stencil: {}
                }
            ).unwrap(),
        );

        let mut previous_frame_end = Box::new(vulkano::sync::now(device.clone())) as Box<vulkano::sync::GpuFuture>;

        let mut textures = HashMap::new();
        for image in ::Image::iter_variants() {
            let image_load = image::load_from_memory_with_format(
                image.data(),
                image::ImageFormat::PNG,
            ).unwrap()
                .to_rgba();

            let w = image_load.width();
            let h = image_load.height();
            let image_data = image_load.into_raw().clone();

            let (texture, future) = vulkano::image::immutable::ImmutableImage::from_iter(
                image_data.iter().cloned(),
                vulkano::image::Dimensions::Dim2d {
                    width: w,
                    height: h,
                },
                vulkano::format::R8G8B8A8Srgb,
                queue.clone(),
            ).unwrap();
            textures.insert(image, (w, h, texture as Arc<_>));
            previous_frame_end = Box::new(previous_frame_end.join(future)) as Box<_>;
        }

        let pipeline = Arc::new(
            vulkano::pipeline::GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_strip()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                .blend_alpha_blending()
                .render_pass(vulkano::framebuffer::Subpass::from(renderpass.clone(), 0).unwrap())
                .build(device.clone())
                .unwrap(),
        );

        let sets_pool = vulkano::descriptor::descriptor_set::FixedSizeDescriptorSetsPool::new(pipeline.clone() as Arc<_>, 0);
        let transform_buffer_pool = ::vulkano::buffer::CpuBufferPool::uniform_buffer(device.clone());
        let view_buffer_pool = ::vulkano::buffer::CpuBufferPool::uniform_buffer(device.clone());

        let framebuffers = None;
        let recreate_swapchain = false;

        let sampler = vulkano::sampler::Sampler::simple_repeat_linear(device.clone());

        Graphics {
            sampler,
            transform_buffer_pool,
            view_buffer_pool,
            sets_pool,
            device,
            dimensions,
            previous_frame_end,
            framebuffers,
            recreate_swapchain,
            pipeline,
            vertex_buffer,
            queue,
            renderpass,
            images,
            textures,
            swapchain,
            surface,
        }
    }

    pub fn render(&mut self, world: &mut specs::World) {
        self.previous_frame_end.cleanup_finished();
        if self.recreate_swapchain {
            self.dimensions = self.surface
                .capabilities(self.device.physical_device())
                .expect("failed to get surface capabilities")
                .current_extent
                .unwrap_or([1024, 768]);

            let (new_swapchain, new_images) = match self.swapchain.recreate_with_dimension(self.dimensions) {
                Ok(r) => r,
                Err(vulkano::swapchain::SwapchainCreationError::UnsupportedDimensions) => {
                    self.render(world);
                    return
                }
                Err(err) => panic!("{:?}", err),
            };

            self.swapchain = new_swapchain;
            self.images = new_images;

            self.framebuffers = None;

            self.recreate_swapchain = false;
        }

        if self.framebuffers.is_none() {
            self.framebuffers = Some(
                self.images
                    .iter()
                    .map(|image| {
                        Arc::new(
                            vulkano::framebuffer::Framebuffer::start(self.renderpass.clone())
                                .add(image.clone())
                                .unwrap()
                                .build()
                                .unwrap(),
                        ) as Arc<_>
                    })
                    .collect::<Vec<_>>(),
            );
        }

        let (image_num, future) =
            match vulkano::swapchain::acquire_next_image(self.swapchain.clone(), None) {
                Ok(r) => r,
                Err(vulkano::swapchain::AcquireError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    self.render(world);
                    return
                }
                Err(err) => panic!("{:?}", err),
            };

        let cb = self.build_command_buffer(image_num, world);

        let mut previous_frame_end = Box::new(vulkano::sync::now(self.device.clone())) as Box<_>;
        ::std::mem::swap(&mut previous_frame_end, &mut self.previous_frame_end);
        let future = previous_frame_end
            .join(future)
            .then_execute(self.queue.clone(), cb)
            .unwrap()
            .then_swapchain_present(self.queue.clone(), self.swapchain.clone(), image_num)
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                self.previous_frame_end = Box::new(future) as Box<_>;
            }
            Err(vulkano::sync::FlushError::OutOfDate) => {
                self.recreate_swapchain = true;
            }
            Err(e) => {
                println!("{:?}", e);
            }
        }
    }

    fn build_command_buffer(&mut self, image_num: usize, world: &mut specs::World) -> vulkano::command_buffer::AutoCommandBuffer {
        let mut view: ::na::Transform2<f32> = ::na::one();
        view[(0, 0)] = self.dimensions[1] as f32/self.dimensions[0] as f32;

        let view = self.view_buffer_pool.next(vs::ty::View {
            view: mat4(view.unwrap().into()),
        }).unwrap();

        let state = vulkano::command_buffer::DynamicState {
            line_width: None,
            viewports: Some(vec![vulkano::pipeline::viewport::Viewport {
                origin: [0.0, 0.0],
                dimensions: [self.dimensions[0] as f32, self.dimensions[1] as f32],
                depth_range: 0.0..1.0,
            }]),
            scissors: None,
        };

        let mut cb = vulkano::command_buffer::AutoCommandBufferBuilder::primary_one_time_submit(
            self.device.clone(),
            self.queue.family(),
        ).unwrap()
            .begin_render_pass(
                self.framebuffers.as_ref().unwrap()[image_num].clone(),
                false,
                vec![[0.0, 0.0, 1.0, 1.0].into()],
            )
            .unwrap();

        let mut draw_image = |image, mut trans: ::na::Transform2<f32>, z: f32, cb: vulkano::command_buffer::AutoCommandBufferBuilder<vulkano::command_buffer::pool::standard::StandardCommandPoolBuilder>| {
            let ref texture = self.textures[&image];
            trans[(0, 0)] *= texture.0 as f32/ texture.1 as f32;
            let trans = self.transform_buffer_pool.next(vs::ty::Transform {
                trans: mat4(trans.unwrap().into()),
                z,
            }).unwrap();

            let set = self.sets_pool.next()
                .add_buffer(trans)
                .unwrap()
                .add_sampled_image(texture.2.clone(), self.sampler.clone())
                .unwrap()
                .add_buffer(view.clone())
                .unwrap()
                .build()
                .unwrap();

            cb.draw(
                    self.pipeline.clone(),
                    state.clone(),
                    vec![self.vertex_buffer.clone()],
                    set,
                    (),
                )
                .unwrap()
        };

        let mut trans: ::na::Transform2<f32> = ::na::one();
        trans[(0, 0)] *= 2.0;
        trans[(1, 1)] *= 2.0;
        cb = draw_image(::Image::Wallpaper, trans, 0.0, cb);

        let bodies = world.read_storage::<::component::RigidBody>();
        let images = world.read_storage::<::component::Image>();
        let physic_world = world.read_resource::<::resource::PhysicWorld>();
        for (image, body) in (&images, &bodies).join() {
            let mut trans: ::na::Transform2<f32> = ::na::Similarity2::from_isometry(
                body.get(&physic_world).position(),
                image.0*2.0,
            ).to_superset();
            cb = draw_image(image.1, trans, 1.0, cb);
        }

        if let Some(image) = world.write_resource::<::resource::DrawImage>().take() {
            let mut trans: ::na::Transform2<f32> = ::na::one();
            cb = draw_image(image, trans, 1.0, cb);
        }

        cb.end_render_pass()
            .unwrap()
            .build()
            .unwrap()
    }
}

mod vs {
    #[derive(VulkanoShader)]
    #[ty = "vertex"]
    #[src = "
#version 450

layout(location = 0) in vec2 position;
layout(location = 0) out vec2 tex_coords;

layout(set = 0, binding = 0) uniform Transform {
    mat4 trans;
    float z;
} transform;

layout(set = 0, binding = 2) uniform View {
    mat4 view;
} view;

void main() {
    gl_Position = mat4(view.view) * mat4(transform.trans) * vec4(position, transform.z, 1.0);
    // // https://matthewwellings.com/blog/the-new-vulkan-coordinate-system/
    // gl_Position.y = -gl_Position.y;
    tex_coords = position + vec2(0.5);
}
"]
    struct _Dummy;
}

mod fs {
    #[derive(VulkanoShader)]
    #[ty = "fragment"]
    #[src = "
#version 450

layout(location = 0) in vec2 tex_coords;
layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 1) uniform sampler2D tex;

void main() {
    f_color = texture(tex, tex_coords);
}
"]
    struct _Dummy;
}
