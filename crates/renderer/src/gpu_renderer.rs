/// GPU tile frame compositor.
///
/// Renders a complete [`GpuFrame`] using the SNES Mode 1 BG + sprite priority order:
///
/// ```text
/// backdrop → BG3-lo → OBJ0 → OBJ1 → BG2-lo → BG1-lo
///          → OBJ2 → BG2-hi → BG1-hi → OBJ3 → BG3-hi
/// ```
///
/// Mode 1 BG3 is rendered as 2bpp (4-color sub-palettes, tile slots at N/2).
///
/// After compositing, a post-process pass applies SNES color math and brightness.
use crate::gpu_frame::GpuFrame;
use crate::gpu_frame_renderer_resources::GpuFrameRendererResources;
use crate::gpu_frame_work_command::{GpuFramePlan, GpuFramePlanExecutor};
use crate::tile_atlas::RgbaTileOverrideData;

pub struct GpuFrameRenderer {
    resources: GpuFrameRendererResources,
}

impl GpuFrameRenderer {
    /// Build all GPU resources for an `Rgba8Unorm` output target.
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        rgba_tile_overrides: Option<RgbaTileOverrideData<'_>>,
    ) -> Self {
        Self {
            resources: GpuFrameRendererResources::new(device, queue, rgba_tile_overrides),
        }
    }

    /// Render one frame to `output_view`.
    ///
    /// Phase 1: BG+sprite composite → intermediate `comp_tex` (raw CGRAM colours).
    /// Phase 2: post-process pass applies color math + brightness → `output_view`.
    pub fn render_frame(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        frame: &GpuFrame<'_>,
        output_view: &wgpu::TextureView,
    ) {
        self.execute_frame_plan(
            encoder,
            queue,
            frame,
            output_view,
            GpuFramePlan::from_frame(frame),
        );
    }

    fn execute_frame_plan(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        frame: &GpuFrame<'_>,
        output_view: &wgpu::TextureView,
        frame_plan: GpuFramePlan,
    ) {
        let mut backend = self.resources.backend(encoder, queue, frame, output_view);
        GpuFramePlanExecutor::execute(frame_plan, &mut backend);
    }

    /// Placeholder for the modern GPU BG renderer comparison pass (Task 9 implements this).
    pub fn render_modern_frame_for_compare(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        frame: &GpuFrame<'_>,
        output_view: &wgpu::TextureView,
    ) {
        let _ = (encoder, queue, frame, output_view);
    }
}
