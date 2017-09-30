use vulkano::framebuffer::RenderPassDesc;
use vulkano::sync::GpuFuture;

use std::sync::Arc;
use std::iter;

pub mod shader;
pub mod render_pass;
mod primitives;
mod colors;

pub use self::primitives::primitive;
pub use self::colors::color;

lazy_static! {
    pub static ref GROUP_COUNTER: GroupCounter = GroupCounter::new();
}

pub struct GroupCounter {
    counter: ::std::sync::atomic::AtomicUsize,
}

impl GroupCounter {
    fn new() -> Self {
        GroupCounter { counter: ::std::sync::atomic::AtomicUsize::new(1) }
    }

    pub fn next(&self) -> u16 {
        self.counter.fetch_add(
            1,
            ::std::sync::atomic::Ordering::Relaxed,
        ) as u16
    }
}

#[derive(Debug, Clone)]
pub struct Vertex {
    position: [f32; 3],
}
impl_vertex!(Vertex, position);

#[derive(Debug, Clone)]
pub struct SecondVertex {
    position: [f32; 2],
}
impl_vertex!(SecondVertex, position);

#[derive(Clone)]
pub struct Data {
    pub device: Arc<::vulkano::device::Device>,
    pub queue: Arc<::vulkano::device::Queue>,
    pub swapchain: Arc<::vulkano::swapchain::Swapchain>,
    pub images: Vec<Arc<::vulkano::image::swapchain::SwapchainImage>>,
    pub depth_buffer_attachment: Arc<::vulkano::image::attachment::AttachmentImage>,
    pub tmp_image_attachment: Arc<::vulkano::image::attachment::AttachmentImage>,
    pub primitives_vertex_buffers: Vec<Arc<::vulkano::buffer::cpu_access::CpuAccessibleBuffer<[Vertex]>>>,

    pub fullscreen_vertex_buffer: Arc<::vulkano::buffer::cpu_access::CpuAccessibleBuffer<[SecondVertex]>>,
    pub render_pass: Arc<::vulkano::framebuffer::RenderPass<render_pass::CustomRenderPassDesc>>,
    pub second_render_pass: Arc<::vulkano::framebuffer::RenderPass<render_pass::SecondCustomRenderPassDesc>>,
    pub pipeline: Arc<::vulkano::pipeline::GraphicsPipeline<::vulkano::pipeline::vertex::SingleBufferDefinition<Vertex>, Box<::vulkano::descriptor::PipelineLayoutAbstract + Sync + Send>, ::Arc<::vulkano::framebuffer::RenderPass<render_pass::CustomRenderPassDesc>>>>,
    pub second_pipeline: Arc<::vulkano::pipeline::GraphicsPipeline<::vulkano::pipeline::vertex::SingleBufferDefinition<SecondVertex>, Box<::vulkano::descriptor::PipelineLayoutAbstract + Sync + Send>, ::Arc<::vulkano::framebuffer::RenderPass<render_pass::SecondCustomRenderPassDesc>>>>,
    pub second_pipeline_cursor: Arc<::vulkano::pipeline::GraphicsPipeline<::vulkano::pipeline::vertex::SingleBufferDefinition<SecondVertex>, Box<::vulkano::descriptor::PipelineLayoutAbstract + Sync + Send>, ::Arc<::vulkano::framebuffer::RenderPass<render_pass::SecondCustomRenderPassDesc>>>>,
    pub framebuffer: Arc<::vulkano::framebuffer::Framebuffer<Arc<::vulkano::framebuffer::RenderPass<render_pass::CustomRenderPassDesc>>, (((), Arc<::vulkano::image::AttachmentImage>), Arc<::vulkano::image::AttachmentImage>)>>,
    pub second_framebuffers: Vec<Arc<::vulkano::framebuffer::Framebuffer<Arc<::vulkano::framebuffer::RenderPass<render_pass::SecondCustomRenderPassDesc>>, ((), Arc<::vulkano::image::SwapchainImage>)>>>,
    pub width: u32,
    pub height: u32,
    pub view_uniform_buffer: ::vulkano::buffer::cpu_pool::CpuBufferPool<::graphics::shader::vs::ty::View>,
    pub tmp_image_set: Arc<::vulkano::descriptor::descriptor_set::PersistentDescriptorSet<Arc<::vulkano::pipeline::GraphicsPipeline<::vulkano::pipeline::vertex::SingleBufferDefinition<::graphics::SecondVertex>, Box<::vulkano::descriptor::PipelineLayoutAbstract + Sync + Send>, Arc<::vulkano::framebuffer::RenderPass<::graphics::render_pass::SecondCustomRenderPassDesc>>>>, (((), ::vulkano::descriptor::descriptor_set::PersistentDescriptorSetImg<Arc<::vulkano::image::AttachmentImage>>), ::vulkano::descriptor::descriptor_set::PersistentDescriptorSetSampler)>>,
    pub cursor_texture_set: Arc<::vulkano::descriptor::descriptor_set::PersistentDescriptorSet<Arc<::vulkano::pipeline::GraphicsPipeline<::vulkano::pipeline::vertex::SingleBufferDefinition<::graphics::SecondVertex>, Box<::vulkano::descriptor::PipelineLayoutAbstract + Sync + Send>, Arc<::vulkano::framebuffer::RenderPass<::graphics::render_pass::SecondCustomRenderPassDesc>>>>, (((), ::vulkano::descriptor::descriptor_set::PersistentDescriptorSetImg<Arc<::vulkano::image::ImmutableImage<::vulkano::format::R8G8B8A8Srgb>>>), ::vulkano::descriptor::descriptor_set::PersistentDescriptorSetSampler)>>,
}

pub struct Graphics<'a> {
    pub physical: ::vulkano::instance::PhysicalDevice<'a>,
    pub data: Data,
}

impl<'a> Graphics<'a> {
    pub fn new(window: &'a ::vulkano_win::Window) -> Graphics<'a> {
        //TODO: read config and save device
        let physical = ::vulkano::instance::PhysicalDevice::enumerate(&window.surface().instance())
            .next()
            .expect("no device available");

        let queue_family = physical
            .queue_families()
            .find(|&q| {
                q.supports_graphics() && window.surface().is_supported(q).unwrap_or(false)
            })
            .expect("couldn't find a graphical queue family");

        let (device, mut queues) = {
            let device_ext = ::vulkano::device::DeviceExtensions {
                khr_swapchain: true,
                ..::vulkano::device::DeviceExtensions::none()
            };

            ::vulkano::device::Device::new(
                physical,
                physical.supported_features(),
                &device_ext,
                [(queue_family, 0.5)].iter().cloned(),
            ).expect("failed to create device")
        };

        let queue = queues.next().unwrap();

        let (swapchain, images) = {
            let caps = window.surface().capabilities(physical).expect(
                "failed to get surface capabilities",
            );

            let dimensions = caps.current_extent.unwrap_or([1280, 1024]);
            let image_usage = ::vulkano::image::ImageUsage {
                sampled: true,
                color_attachment: true,
                ..::vulkano::image::ImageUsage::none()
            };

            ::vulkano::swapchain::Swapchain::new(
                device.clone(),
                window.surface().clone(),
                caps.min_image_count,
                ::vulkano::format::Format::B8G8R8A8Srgb,
                dimensions,
                1,
                image_usage,
                &queue,
                ::vulkano::swapchain::SurfaceTransform::Identity,
                ::vulkano::swapchain::CompositeAlpha::Opaque,
                ::vulkano::swapchain::PresentMode::Fifo,
                true,
                None,
            ).expect("failed to create swapchain")
        };

        let width = images[0].dimensions()[0];
        let height = images[0].dimensions()[1];

        let depth_buffer_attachment = ::vulkano::image::attachment::AttachmentImage::transient(
            device.clone(),
            images[0].dimensions(),
            ::vulkano::format::Format::D16Unorm,
        ).unwrap();

        let tmp_image_attachment = {
            let usage = ::vulkano::image::ImageUsage {
                color_attachment: true,
                sampled: true,
                ..::vulkano::image::ImageUsage::none()
            };
            ::vulkano::image::attachment::AttachmentImage::with_usage(
                device.clone(),
                images[0].dimensions(),
                ::vulkano::format::Format::R16G16Uint,
                usage,
            ).unwrap()
        };

        let primitives_vertex_buffers = primitives::instance_primitives(device.clone());

        let fullscreen_vertex_buffer =
            ::vulkano::buffer::cpu_access::CpuAccessibleBuffer::from_iter(
                device.clone(),
                ::vulkano::buffer::BufferUsage::vertex_buffer(),
                [
                    SecondVertex { position: [-1.0f32, -1.0] },
                    SecondVertex { position: [1.0, -1.0] },
                    SecondVertex { position: [-1.0, 1.0] },
                    SecondVertex { position: [1.0, 1.0] },
                    SecondVertex { position: [-1.0, 1.0] },
                    SecondVertex { position: [1.0, -1.0] },
                ].iter()
                    .cloned(),
        ).expect("failed to create buffer");

        let vs = shader::vs::Shader::load(device.clone()).expect("failed to create shader module");
        let fs = shader::fs::Shader::load(device.clone()).expect("failed to create shader module");

        let second_vs = shader::second_vs::Shader::load(device.clone()).expect(
            "failed to create shader module",
        );
        let second_fs = shader::second_fs::Shader::load(device.clone()).expect(
            "failed to create shader module",
        );

        let second_vs_cursor = shader::second_vs_cursor::Shader::load(device.clone()).expect(
            "failed to create shader module",
        );
        let second_fs_cursor = shader::second_fs_cursor::Shader::load(device.clone()).expect(
            "failed to create shader module",
        );

        let render_pass = Arc::new(
            render_pass::CustomRenderPassDesc
                .build_render_pass(device.clone())
                .unwrap(),
        );
        let second_render_pass = Arc::new(
            render_pass::SecondCustomRenderPassDesc
                .build_render_pass(device.clone())
                .unwrap(),
        );

        let pipeline = Arc::new(
            ::vulkano::pipeline::GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(vs.main_entry_point(), ())
                .viewports(iter::once(::vulkano::pipeline::viewport::Viewport {
                    origin: [0.0, 0.0],
                    depth_range: 0.0..1.0,
                    dimensions: [width as f32, height as f32],
                }))
                .fragment_shader(fs.main_entry_point(), ())
                .depth_stencil_simple_depth()
                .render_pass(
                    ::vulkano::framebuffer::Subpass::from(render_pass.clone(), 0).unwrap(),
                )
                .build(device.clone())
                .unwrap(),
        );

        let second_pipeline = Arc::new(
            ::vulkano::pipeline::GraphicsPipeline::start()
                .vertex_input_single_buffer::<SecondVertex>()
                .vertex_shader(second_vs.main_entry_point(), ())
                .triangle_list()
                .viewports(iter::once(::vulkano::pipeline::viewport::Viewport {
                    origin: [0.0, 0.0],
                    depth_range: 0.0..1.0,
                    dimensions: [width as f32, height as f32],
                }))
                .fragment_shader(second_fs.main_entry_point(), ())
                .render_pass(
                    ::vulkano::framebuffer::Subpass::from(second_render_pass.clone(), 0).unwrap(),
                )
                .build(device.clone())
                .unwrap(),
        );

        let second_pipeline_cursor = Arc::new(
            ::vulkano::pipeline::GraphicsPipeline::start()
                .vertex_input_single_buffer::<SecondVertex>()
                .vertex_shader(second_vs_cursor.main_entry_point(), ())
                .triangle_list()
                .viewports(iter::once(::vulkano::pipeline::viewport::Viewport {
                    origin: [0.0, 0.0],
                    depth_range: 0.0..1.0,
                    dimensions: [width as f32, height as f32],
                }))
                .fragment_shader(second_fs_cursor.main_entry_point(), ())
                .blend_alpha_blending()
                .render_pass(
                    ::vulkano::framebuffer::Subpass::from(second_render_pass.clone(), 0).unwrap(),
                )
                .build(device.clone())
                .unwrap(),
        );

        let framebuffer = Arc::new(
            ::vulkano::framebuffer::Framebuffer::start(render_pass.clone())
                .add(tmp_image_attachment.clone())
                .unwrap()
                .add(depth_buffer_attachment.clone())
                .unwrap()
                .build()
                .unwrap(),
        );

        let second_framebuffers = images
            .iter()
            .map(|image| {
                Arc::new(
                    ::vulkano::framebuffer::Framebuffer::start(second_render_pass.clone())
                        .add(image.clone())
                        .unwrap()
                        .build()
                        .unwrap(),
                )
            })
            .collect::<Vec<_>>();

        let view_uniform_buffer =
            ::vulkano::buffer::cpu_pool::CpuBufferPool::<::graphics::shader::vs::ty::View>::new(
                device.clone(),
                ::vulkano::buffer::BufferUsage::uniform_buffer(),
            );

        //TODO: maybe use simple instead of persistent
        let tmp_image_set = Arc::new(
            ::vulkano::descriptor::descriptor_set::PersistentDescriptorSet::start(
                second_pipeline.clone(),
                0,
            ).add_sampled_image(
                tmp_image_attachment.clone(),
                // Sampler::simple_repeat_linear_no_mipmap(graphics.device.clone()),
                ::vulkano::sampler::Sampler::unnormalized(
                    device.clone(),
                    ::vulkano::sampler::Filter::Nearest,
                    ::vulkano::sampler::UnnormalizedSamplerAddressMode::ClampToEdge,
                    ::vulkano::sampler::UnnormalizedSamplerAddressMode::ClampToEdge,
                ).unwrap(),
            )
                .unwrap()
                .build()
                .unwrap(),
        );

        let (cursor_texture, mut cursor_tex_future) = {
            // TODO: The cursor must be configurable
            // TODO: use PNG instead of image
            let file = ::std::io::Cursor::new(include_bytes!("default_cursor.png").as_ref());
            let (info, mut reader) = ::png::Decoder::new(file).read_info().unwrap();
            // TODO: is format OK ?
            assert_eq!(info.color_type, ::png::ColorType::RGBA);
            let mut buf = vec![0; info.buffer_size()];
            reader.next_frame(&mut buf).unwrap();
            println!("{:#?}", buf);

            ::vulkano::image::immutable::ImmutableImage::from_iter(
                buf.iter().cloned(),
                // TODO: take width and height from png image
                ::vulkano::image::Dimensions::Dim2d { width: info.width, height: info.height },
                ::vulkano::format::R8G8B8A8Srgb,
                queue.clone()).unwrap()
        };

        let cursor_sampler = ::vulkano::sampler::Sampler::new(device.clone(), ::vulkano::sampler::Filter::Linear,
                                                     ::vulkano::sampler::Filter::Linear, ::vulkano::sampler::MipmapMode::Nearest,
                                                     ::vulkano::sampler::SamplerAddressMode::ClampToEdge,
                                                     ::vulkano::sampler::SamplerAddressMode::ClampToEdge,
                                                     ::vulkano::sampler::SamplerAddressMode::ClampToEdge,
                                                     // TODO: What values here
                                                     0.0, 1.0, 0.0, 0.0).unwrap();

        let cursor_texture_set = Arc::new(::vulkano::descriptor::descriptor_set::PersistentDescriptorSet::start(second_pipeline.clone(), 0)
            .add_sampled_image(cursor_texture.clone(), cursor_sampler.clone()).unwrap()
            .build().unwrap()
        );

        // TODO: return this future to enforce it later ?
        cursor_tex_future.cleanup_finished();

        Graphics {
            physical,
            data: Data {
                fullscreen_vertex_buffer,
                depth_buffer_attachment,
                tmp_image_attachment,
                swapchain,
                images,
                device,
                queue,
                render_pass,
                second_render_pass,
                pipeline,
                second_pipeline,
                framebuffer,
                second_framebuffers,
                width,
                height,
                tmp_image_set,
                view_uniform_buffer,
                primitives_vertex_buffers,
                cursor_texture_set,
                second_pipeline_cursor,
            },
        }
    }
}
