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
use crate::bg_layer::BgLayerRenderer;
use crate::gpu_frame::GpuFrame;
use crate::gpu_frame_render_plan::GpuFrameRenderPlanContext;
use crate::gpu_frame_work_command::{
    GpuFrameMainWorkCommand, GpuFrameRenderPlan, GpuFrameSpritePass, GpuFrameSubWorkCommand,
    GpuFrameWindowSelector, GpuFrameWorkCommand,
};
use crate::mode7_renderer::Mode7Renderer;
use crate::post_process::PostProcessRenderer;
use crate::sprite_renderer::SpriteRenderer;
use crate::tile_atlas::{CgramPalette, RgbaTileOverrideData, RgbaTileOverrideTextures, TileAtlas};

pub struct GpuFrameRenderer {
    tile_atlas: TileAtlas,
    cgram_palette: CgramPalette,
    _rgba_tile_overrides: RgbaTileOverrideTextures,
    /// bg[0]=BG1, bg[1]=BG2, bg[2]=BG3; indices match `GpuFrame.bg[]`.
    bg: [BgLayerRenderer; 3],
    mode7: Mode7Renderer,
    sprites: SpriteRenderer,
    post_process: PostProcessRenderer,
    /// Intermediate texture: main-screen BG+sprite composite before color math.
    #[allow(dead_code)]
    comp_tex: wgpu::Texture,
    comp_view: wgpu::TextureView,
    /// Intermediate texture: sub-screen composite (second operand for color math).
    #[allow(dead_code)]
    sub_comp_tex: wgpu::Texture,
    sub_comp_view: wgpu::TextureView,
}

const COMP_WIDTH: u32 = 256;
const COMP_HEIGHT: u32 = 224;

impl GpuFrameRenderer {
    /// Build all GPU resources for an `Rgba8Unorm` output target.
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        rgba_tile_overrides: Option<RgbaTileOverrideData<'_>>,
    ) -> Self {
        let tile_atlas = TileAtlas::new(device);
        let cgram_palette = CgramPalette::new(device);
        let rgba_tile_overrides = RgbaTileOverrideTextures::new(device, queue, rgba_tile_overrides);
        let bg = [
            BgLayerRenderer::new(
                device,
                &tile_atlas,
                &cgram_palette,
                &rgba_tile_overrides,
                wgpu::TextureFormat::Rgba8Unorm,
            ),
            BgLayerRenderer::new(
                device,
                &tile_atlas,
                &cgram_palette,
                &rgba_tile_overrides,
                wgpu::TextureFormat::Rgba8Unorm,
            ),
            BgLayerRenderer::new(
                device,
                &tile_atlas,
                &cgram_palette,
                &rgba_tile_overrides,
                wgpu::TextureFormat::Rgba8Unorm,
            ),
        ];
        let sprites = SpriteRenderer::new(
            device,
            &tile_atlas,
            &cgram_palette,
            wgpu::TextureFormat::Rgba8Unorm,
        );
        let mode7 = Mode7Renderer::new(device, &cgram_palette, wgpu::TextureFormat::Rgba8Unorm);
        let comp_tex_desc = wgpu::TextureDescriptor {
            label: Some("comp_intermediate"),
            size: wgpu::Extent3d {
                width: COMP_WIDTH,
                height: COMP_HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let comp_tex = device.create_texture(&comp_tex_desc);
        let comp_view = comp_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let sub_comp_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("sub_comp_intermediate"),
            ..comp_tex_desc
        });
        let sub_comp_view = sub_comp_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let post_process = PostProcessRenderer::new(
            device,
            wgpu::TextureFormat::Rgba8Unorm,
            &comp_view,
            &sub_comp_view,
        );
        Self {
            tile_atlas,
            cgram_palette,
            _rgba_tile_overrides: rgba_tile_overrides,
            bg,
            mode7,
            sprites,
            post_process,
            comp_tex,
            comp_view,
            sub_comp_tex,
            sub_comp_view,
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
        self.tile_atlas.update(queue, frame.vram);
        self.cgram_palette.update(queue, frame.cgram);

        let render_plan_context = GpuFrameRenderPlanContext::from_frame(frame);
        if render_plan_context.uses_sprites() {
            self.sprites.prepare(
                queue,
                frame.vram,
                frame.oam,
                &frame.obj,
                frame.extra_left_right,
            );
        }

        let render_plan = render_plan_context.render_plan();
        self.execute_render_plan(encoder, queue, frame, output_view, render_plan);
    }

    fn execute_render_plan(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        frame: &GpuFrame<'_>,
        output_view: &wgpu::TextureView,
        render_plan: GpuFrameRenderPlan,
    ) {
        render_plan.execute_with(|command| {
            self.render_gpu_work_command(encoder, queue, frame, output_view, command);
        });
    }

    fn render_gpu_work_command(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        frame: &GpuFrame<'_>,
        output_view: &wgpu::TextureView,
        command: GpuFrameWorkCommand,
    ) {
        match command {
            GpuFrameWorkCommand::Main(command) => {
                self.render_main_gpu_work_item(encoder, queue, frame, command);
            }
            GpuFrameWorkCommand::Sub(command) => {
                self.render_sub_gpu_work_item(encoder, queue, frame, command);
            }
            GpuFrameWorkCommand::PostProcess => {
                self.render_post_process_gpu_work_item(encoder, queue, frame, output_view);
            }
        }
    }

    fn render_main_gpu_work_item(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        frame: &GpuFrame<'_>,
        command: GpuFrameMainWorkCommand,
    ) {
        match command {
            GpuFrameMainWorkCommand::ClearBackdrop => {
                self.clear_main_backdrop(encoder, frame);
            }
            GpuFrameMainWorkCommand::SpritePriority(pass) => {
                render_sprite_pass(
                    &mut self.sprites,
                    encoder,
                    queue,
                    frame,
                    &self.comp_view,
                    pass,
                );
            }
            GpuFrameMainWorkCommand::BgLayer {
                layer_idx,
                hi_priority,
                is_2bpp,
                layer_bit,
                math_bit_pos,
                mosaic_layer_bit,
                window,
            } => {
                render_bg_pass(
                    &mut self.bg[layer_idx],
                    encoder,
                    queue,
                    frame,
                    layer_idx,
                    hi_priority,
                    &self.comp_view,
                    is_2bpp,
                    layer_bit,
                    math_bit_pos,
                    mosaic_layer_bit,
                    window,
                );
            }
            GpuFrameMainWorkCommand::Mode7Bg {
                math_bit_pos,
                layer_bit,
                window,
            } => {
                self.mode7.render(
                    encoder,
                    queue,
                    frame,
                    &self.comp_view,
                    math_bit_pos,
                    layer_bit,
                    window.screen_idx,
                    window.layer_bit,
                    window.flags_shift,
                );
            }
        }
    }

    fn render_sub_gpu_work_item(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        frame: &GpuFrame<'_>,
        command: GpuFrameSubWorkCommand,
    ) {
        match command {
            GpuFrameSubWorkCommand::ClearBackdrop => {
                self.clear_sub_backdrop(encoder);
            }
            GpuFrameSubWorkCommand::Mode7Bg {
                math_bit_pos,
                layer_bit,
                window,
            } => {
                self.mode7.render(
                    encoder,
                    queue,
                    frame,
                    &self.sub_comp_view,
                    math_bit_pos,
                    layer_bit,
                    window.screen_idx,
                    window.layer_bit,
                    window.flags_shift,
                );
            }
            GpuFrameSubWorkCommand::BgLayer {
                layer_idx,
                hi_priority,
                is_2bpp,
                screen_layer_bit,
                render_layer_bit,
                math_bit_pos,
                mosaic_layer_bit,
                window,
            } => {
                render_sub_bg_pass(
                    &mut self.bg[layer_idx],
                    encoder,
                    queue,
                    frame,
                    layer_idx,
                    hi_priority,
                    &self.sub_comp_view,
                    is_2bpp,
                    screen_layer_bit,
                    render_layer_bit,
                    math_bit_pos,
                    mosaic_layer_bit,
                    window,
                );
            }
            GpuFrameSubWorkCommand::SpritePriority(pass) => {
                render_sprite_pass(
                    &mut self.sprites,
                    encoder,
                    queue,
                    frame,
                    &self.sub_comp_view,
                    pass,
                );
            }
        }
    }

    fn render_post_process_gpu_work_item(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        frame: &GpuFrame<'_>,
        output_view: &wgpu::TextureView,
    ) {
        self.post_process.render(encoder, queue, frame, output_view);
    }

    fn clear_main_backdrop(&self, encoder: &mut wgpu::CommandEncoder, frame: &GpuFrame<'_>) {
        // Clear intermediate texture to CGRAM[0] (raw backdrop colour, pre-brightness).
        let backdrop = cgram_to_wgpu_color(frame.cgram.first().copied().unwrap_or(0));
        let _clear = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("backdrop_clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.comp_view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(backdrop),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }

    fn clear_sub_backdrop(&self, encoder: &mut wgpu::CommandEncoder) {
        let _clear = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("sub_backdrop_clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.sub_comp_view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
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

fn render_sprite_pass(
    sprites: &mut SpriteRenderer,
    encoder: &mut wgpu::CommandEncoder,
    queue: &wgpu::Queue,
    frame: &GpuFrame<'_>,
    output_view: &wgpu::TextureView,
    pass: GpuFrameSpritePass,
) {
    let window = pass.window;
    sprites.render(
        encoder,
        queue,
        output_view,
        pass.math_bit_pos,
        window.screen_idx,
        pass.priority,
        window.flags(frame.windowsel),
        window.is_windowed(frame.screen_windowed),
        &frame.scanlines,
    );
}

#[allow(clippy::too_many_arguments)]
fn render_bg_pass(
    bg: &mut BgLayerRenderer,
    encoder: &mut wgpu::CommandEncoder,
    queue: &wgpu::Queue,
    frame: &GpuFrame<'_>,
    layer_idx: usize,
    hi_priority: bool,
    output_view: &wgpu::TextureView,
    is_2bpp: bool,
    layer_bit: u32,
    math_bit_pos: u32,
    mosaic_layer_bit: u8,
    window: GpuFrameWindowSelector,
) {
    bg.render(
        encoder,
        queue,
        frame.vram,
        layer_idx,
        &frame.bg[layer_idx],
        output_view,
        wgpu::LoadOp::Load,
        is_2bpp,
        hi_priority,
        layer_bit,
        math_bit_pos,
        window.screen_idx,
        window.flags(frame.windowsel),
        window.is_windowed(frame.screen_windowed),
        frame.mosaic_enabled & mosaic_layer_bit != 0,
        frame.mosaic_size,
        &frame.scanlines,
    );
}

fn render_sub_bg_pass(
    bg: &mut BgLayerRenderer,
    encoder: &mut wgpu::CommandEncoder,
    queue: &wgpu::Queue,
    frame: &GpuFrame<'_>,
    layer_idx: usize,
    hi_priority: bool,
    output_view: &wgpu::TextureView,
    is_2bpp: bool,
    screen_layer_bit: u8,
    render_layer_bit: u32,
    math_bit_pos: u32,
    mosaic_layer_bit: u8,
    window: GpuFrameWindowSelector,
) {
    if frame.screen_enabled[1] & screen_layer_bit == 0 {
        return;
    }
    render_bg_pass(
        bg,
        encoder,
        queue,
        frame,
        layer_idx,
        hi_priority,
        output_view,
        is_2bpp,
        render_layer_bit,
        math_bit_pos,
        mosaic_layer_bit,
        window,
    );
}

/// Decode a 15-bit SNES BGR CGRAM entry to a `wgpu::Color` for the backdrop clear.
///
/// Alpha = 5/255 encodes math_enabled bit position 5 (backdrop) so the
/// post-process shader applies color math to backdrop pixels only when bit 5
/// of math_enabled is set, matching CPU PPU behavior.
fn cgram_to_wgpu_color(entry: u16) -> wgpu::Color {
    let r = f64::from(((entry & 0x1F) as u8) << 3) / 255.0;
    let g = f64::from((((entry >> 5) & 0x1F) as u8) << 3) / 255.0;
    let b = f64::from((((entry >> 10) & 0x1F) as u8) << 3) / 255.0;
    wgpu::Color {
        r,
        g,
        b,
        a: 5.0 / 255.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backdrop_black() {
        let c = cgram_to_wgpu_color(0x0000);
        assert_eq!(c.r, 0.0);
        assert_eq!(c.g, 0.0);
        assert_eq!(c.b, 0.0);
        assert_eq!(c.a, 5.0 / 255.0);
    }

    #[test]
    fn backdrop_white() {
        let c = cgram_to_wgpu_color(0x7FFF); // R=31, G=31, B=31
        let expected = f64::from(248u8) / 255.0; // 31 << 3 = 248
        assert!((c.r - expected).abs() < 1e-10);
        assert!((c.g - expected).abs() < 1e-10);
        assert!((c.b - expected).abs() < 1e-10);
    }

    #[test]
    fn backdrop_red_only() {
        let c = cgram_to_wgpu_color(0x001F); // BGR: R=31, G=0, B=0
        let expected = f64::from(248u8) / 255.0;
        assert!((c.r - expected).abs() < 1e-10);
        assert!(c.g.abs() < 1e-10);
        assert!(c.b.abs() < 1e-10);
    }

    #[test]
    fn backdrop_blue_only() {
        let c = cgram_to_wgpu_color(0x7C00); // BGR: R=0, G=0, B=31
        let expected = f64::from(248u8) / 255.0;
        assert!(c.r.abs() < 1e-10);
        assert!(c.g.abs() < 1e-10);
        assert!((c.b - expected).abs() < 1e-10);
    }
}
