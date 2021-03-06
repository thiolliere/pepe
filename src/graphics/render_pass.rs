use vulkano::framebuffer::{LayoutAttachmentDescription, LayoutPassDependencyDescription,
                           LayoutPassDescription, LoadOp, RenderPassDesc,
                           RenderPassDescClearValues, StoreOp};
use vulkano::image::ImageLayout;
use vulkano::format::{ClearValue, Format};
use vulkano::sync::{AccessFlagBits, PipelineStages};
pub struct CustomRenderPassDesc;

unsafe impl RenderPassDesc for CustomRenderPassDesc {
    #[inline]
    fn num_attachments(&self) -> usize {
        4
    }

    #[inline]
    fn attachment_desc(&self, id: usize) -> Option<LayoutAttachmentDescription> {
        match id {
            // Colors
            0 => Some(LayoutAttachmentDescription {
                format: Format::R8G8B8A8Uint,
                samples: 1,
                load: LoadOp::Clear,
                store: StoreOp::Store,
                stencil_load: LoadOp::Clear,
                stencil_store: StoreOp::Store,
                initial_layout: ImageLayout::Undefined,
                final_layout: ImageLayout::ColorAttachmentOptimal,
            }),
            // Erasers
            1 => Some(LayoutAttachmentDescription {
                format: Format::R8Uint,
                samples: 1,
                load: LoadOp::Clear,
                store: StoreOp::Store,
                stencil_load: LoadOp::Clear,
                stencil_store: StoreOp::Store,
                initial_layout: ImageLayout::Undefined,
                final_layout: ImageLayout::ColorAttachmentOptimal,
            }),
            // Depth buffer
            2 => Some(LayoutAttachmentDescription {
                format: Format::D16Unorm,
                samples: 1,
                load: LoadOp::Clear,
                store: StoreOp::DontCare,
                stencil_load: LoadOp::Clear,
                stencil_store: StoreOp::DontCare,
                initial_layout: ImageLayout::Undefined,
                final_layout: ImageLayout::DepthStencilAttachmentOptimal,
            }),
            // HUD depth buffer
            3 => Some(LayoutAttachmentDescription {
                format: Format::D16Unorm,
                samples: 1,
                load: LoadOp::Clear,
                store: StoreOp::DontCare,
                stencil_load: LoadOp::Clear,
                stencil_store: StoreOp::DontCare,
                initial_layout: ImageLayout::Undefined,
                final_layout: ImageLayout::DepthStencilAttachmentOptimal,
            }),
            _ => None,
        }
    }

    #[inline]
    fn num_subpasses(&self) -> usize {
        3
    }

    #[inline]
    fn subpass_desc(&self, id: usize) -> Option<LayoutPassDescription> {
        match id {
            // draw
            0 => Some(LayoutPassDescription {
                color_attachments: vec![(0, ImageLayout::ColorAttachmentOptimal)],
                depth_stencil: Some((2, ImageLayout::DepthStencilAttachmentOptimal)),
                input_attachments: vec![],
                resolve_attachments: vec![],
                preserve_attachments: vec![1, 3],
            }),
            // erase
            1 => Some(LayoutPassDescription {
                color_attachments: vec![(1, ImageLayout::ColorAttachmentOptimal)],
                depth_stencil: Some((2, ImageLayout::DepthStencilAttachmentOptimal)),
                input_attachments: vec![],
                resolve_attachments: vec![],
                preserve_attachments: vec![0, 3],
            }),
            // draw HUD
            2 => Some(LayoutPassDescription {
                color_attachments: vec![(0, ImageLayout::ColorAttachmentOptimal)],
                depth_stencil: Some((3, ImageLayout::DepthStencilAttachmentOptimal)),
                input_attachments: vec![],
                resolve_attachments: vec![],
                preserve_attachments: vec![1, 2],
            }),
            _ => None,
        }
    }

    #[inline]
    fn num_dependencies(&self) -> usize {
        2
    }

    #[inline]
    fn dependency_desc(&self, id: usize) -> Option<LayoutPassDependencyDescription> {
        match id {
            0 => Some(LayoutPassDependencyDescription {
                source_subpass: 0,
                destination_subpass: 1,
                source_stages: PipelineStages {
                    late_fragment_tests: true,
                    ..PipelineStages::none()
                },
                destination_stages: PipelineStages {
                    early_fragment_tests: true,
                    ..PipelineStages::none()
                },
                source_access: AccessFlagBits {
                    // TODO: color attachment ?
                    depth_stencil_attachment_write: true,
                    depth_stencil_attachment_read: true,
                    ..AccessFlagBits::none()
                },
                destination_access: AccessFlagBits {
                    // TODO: color attachment ?
                    depth_stencil_attachment_write: true,
                    depth_stencil_attachment_read: true,
                    ..AccessFlagBits::none()
                },
                by_region: true,
            }),
            1 => Some(LayoutPassDependencyDescription {
                source_subpass: 0,
                destination_subpass: 2,
                source_stages: PipelineStages {
                    late_fragment_tests: true,
                    ..PipelineStages::none()
                },
                destination_stages: PipelineStages {
                    early_fragment_tests: true,
                    ..PipelineStages::none()
                },
                source_access: AccessFlagBits {
                    color_attachment_write: true,
                    ..AccessFlagBits::none()
                },
                destination_access: AccessFlagBits {
                    color_attachment_write: true,
                    ..AccessFlagBits::none()
                },
                by_region: true,
            }),
            _ => None,
        }
    }
}

unsafe impl RenderPassDescClearValues<Vec<ClearValue>> for CustomRenderPassDesc {
    fn convert_clear_values(&self, values: Vec<ClearValue>) -> Box<Iterator<Item = ClearValue>> {
        // FIXME: safety checks
        Box::new(values.into_iter())
    }
}

pub struct SecondCustomRenderPassDesc {
    swapchain_format: Format
}

impl SecondCustomRenderPassDesc {
    pub fn new(swapchain_format: Format) -> Self {
        SecondCustomRenderPassDesc {
            swapchain_format,
        }
    }
}

unsafe impl RenderPassDesc for SecondCustomRenderPassDesc {
    #[inline]
    fn num_attachments(&self) -> usize {
        1
    }

    #[inline]
    fn attachment_desc(&self, id: usize) -> Option<LayoutAttachmentDescription> {
        match id {
            0 => Some(LayoutAttachmentDescription {
                format: self.swapchain_format,
                samples: 1,
                load: LoadOp::DontCare,
                store: StoreOp::Store,
                stencil_load: LoadOp::DontCare,
                stencil_store: StoreOp::Store,
                initial_layout: ImageLayout::Undefined,
                final_layout: ImageLayout::ColorAttachmentOptimal,
            }),
            _ => None,
        }
    }

    #[inline]
    fn num_subpasses(&self) -> usize {
        1
    }

    #[inline]
    fn subpass_desc(&self, id: usize) -> Option<LayoutPassDescription> {
        match id {
            0 => Some(LayoutPassDescription {
                color_attachments: vec![(0, ImageLayout::ColorAttachmentOptimal)],
                depth_stencil: None,
                input_attachments: vec![],
                resolve_attachments: vec![],
                preserve_attachments: vec![],
            }),
            _ => None,
        }
    }

    #[inline]
    fn num_dependencies(&self) -> usize {
        0
    }

    #[inline]
    fn dependency_desc(&self, _id: usize) -> Option<LayoutPassDependencyDescription> {
        None
    }
}

unsafe impl RenderPassDescClearValues<Vec<ClearValue>> for SecondCustomRenderPassDesc {
    fn convert_clear_values(&self, values: Vec<ClearValue>) -> Box<Iterator<Item = ClearValue>> {
        // FIXME: safety checks
        Box::new(values.into_iter())
    }
}
