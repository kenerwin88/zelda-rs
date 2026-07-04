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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GpuFrameWorkItem {
    MainSpritePriority(u32),
    MainBgLayer {
        layer_idx: usize,
        hi_priority: bool,
        layer_bit: u32,
        math_bit_pos: u32,
    },
    Mode7MainBg,
    ClearSubBackdrop,
    Mode7SubBg,
    SubBgLayer {
        layer_idx: usize,
        hi_priority: bool,
    },
    SubSpritePriority(u32),
    PostProcess,
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

        // Clear intermediate texture to CGRAM[0] (raw backdrop colour, pre-brightness).
        let backdrop = cgram_to_wgpu_color(frame.cgram.first().copied().unwrap_or(0));
        {
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

        let has_main_sprites = frame
            .scanlines
            .iter()
            .any(|scanline| scanline.screen_enabled_main & 0x10 != 0);
        let has_sub_sprites = frame.screen_enabled[1] & 0x10 != 0;
        let has_main_bg = frame
            .scanlines
            .iter()
            .any(|scanline| scanline.screen_enabled_main & 0x07 != 0);
        let has_sub_bg = frame.screen_enabled[1] & 0x07 != 0;
        if has_main_sprites || has_sub_sprites {
            self.sprites.prepare(
                queue,
                frame.vram,
                frame.oam,
                &frame.obj,
                frame.extra_left_right,
            );
        }

        if frame.mode == 7 {
            self.render_mode7_frame(
                encoder,
                queue,
                frame,
                output_view,
                has_main_sprites,
                has_sub_sprites,
            );
            return;
        }

        for work_item in
            mode1_work_items(has_main_bg, has_main_sprites, has_sub_bg, has_sub_sprites)
        {
            self.render_gpu_work_item(encoder, queue, frame, output_view, work_item);
        }
    }

    fn render_mode7_frame(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        frame: &GpuFrame<'_>,
        output_view: &wgpu::TextureView,
        has_main_sprites: bool,
        has_sub_sprites: bool,
    ) {
        let has_sub_mode7_bg = frame.screen_enabled[1] & 1 != 0;
        for work_item in mode7_work_items(has_main_sprites, has_sub_mode7_bg, has_sub_sprites) {
            self.render_gpu_work_item(encoder, queue, frame, output_view, work_item);
        }
    }

    fn render_gpu_work_item(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        frame: &GpuFrame<'_>,
        output_view: &wgpu::TextureView,
        work_item: GpuFrameWorkItem,
    ) {
        match work_item {
            GpuFrameWorkItem::MainSpritePriority(priority) => {
                self.render_main_sprites(encoder, queue, frame, priority);
            }
            GpuFrameWorkItem::MainBgLayer {
                layer_idx,
                hi_priority,
                layer_bit,
                math_bit_pos,
            } => {
                render_bg_pass(
                    &mut self.bg[layer_idx],
                    encoder,
                    queue,
                    frame,
                    layer_idx,
                    hi_priority,
                    &self.comp_view,
                    layer_bit,
                    math_bit_pos,
                );
            }
            GpuFrameWorkItem::Mode7MainBg => {
                self.mode7
                    .render(encoder, queue, frame, &self.comp_view, 0, 1);
            }
            GpuFrameWorkItem::ClearSubBackdrop => {
                self.clear_sub_backdrop(encoder);
            }
            GpuFrameWorkItem::Mode7SubBg => {
                self.mode7
                    .render(encoder, queue, frame, &self.sub_comp_view, 255, 0);
            }
            GpuFrameWorkItem::SubBgLayer {
                layer_idx,
                hi_priority,
            } => {
                render_sub_bg_pass(
                    &mut self.bg[layer_idx],
                    encoder,
                    queue,
                    frame,
                    layer_idx,
                    hi_priority,
                    &self.sub_comp_view,
                );
            }
            GpuFrameWorkItem::SubSpritePriority(priority) => {
                self.render_sub_sprites(encoder, queue, frame, priority);
            }
            GpuFrameWorkItem::PostProcess => {
                self.post_process.render(encoder, queue, frame, output_view);
            }
        }
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

    fn render_main_sprites(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        frame: &GpuFrame<'_>,
        priority: u32,
    ) {
        self.sprites.render(
            encoder,
            queue,
            &self.comp_view,
            4,
            priority,
            (frame.windowsel >> 16) & 0x0f,
            frame.screen_windowed[0] & 0x10 != 0,
            &frame.scanlines,
        );
    }

    fn render_sub_sprites(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        frame: &GpuFrame<'_>,
        priority: u32,
    ) {
        self.sprites.render(
            encoder,
            queue,
            &self.sub_comp_view,
            255,
            priority,
            (frame.windowsel >> 16) & 0x0f,
            frame.screen_windowed[1] & 0x10 != 0,
            &frame.scanlines,
        );
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

fn mode1_work_items(
    has_main_bg: bool,
    has_main_sprites: bool,
    has_sub_bg: bool,
    has_sub_sprites: bool,
) -> Vec<GpuFrameWorkItem> {
    let mut work_items = Vec::new();
    if has_main_sprites && !has_main_bg {
        work_items.extend((0..=3).map(GpuFrameWorkItem::MainSpritePriority));
    }

    if has_main_bg {
        // CPU Mode 1 z-order:
        //   BG3-lo, OBJ0, OBJ1, BG2-lo, BG1-lo, OBJ2,
        //   BG2-hi, BG1-hi, OBJ3, BG3-hi.
        work_items.push(main_bg_work_item(2, false, 2));
        if has_main_sprites {
            work_items.push(GpuFrameWorkItem::MainSpritePriority(0));
            work_items.push(GpuFrameWorkItem::MainSpritePriority(1));
        }
        work_items.push(main_bg_work_item(1, false, 1));
        work_items.push(main_bg_work_item(0, false, 0));
        if has_main_sprites {
            work_items.push(GpuFrameWorkItem::MainSpritePriority(2));
        }
        work_items.push(main_bg_work_item(1, true, 1));
        work_items.push(main_bg_work_item(0, true, 0));
        if has_main_sprites {
            work_items.push(GpuFrameWorkItem::MainSpritePriority(3));
        }
        work_items.push(main_bg_work_item(2, true, 2));
    }

    work_items.push(GpuFrameWorkItem::ClearSubBackdrop);
    work_items.push(GpuFrameWorkItem::SubBgLayer {
        layer_idx: 2,
        hi_priority: false,
    });
    if has_sub_sprites && !has_sub_bg {
        work_items.extend((0..=3).map(GpuFrameWorkItem::SubSpritePriority));
    }
    if has_sub_sprites && has_sub_bg {
        work_items.push(GpuFrameWorkItem::SubSpritePriority(0));
        work_items.push(GpuFrameWorkItem::SubSpritePriority(1));
    }
    work_items.push(GpuFrameWorkItem::SubBgLayer {
        layer_idx: 1,
        hi_priority: false,
    });
    work_items.push(GpuFrameWorkItem::SubBgLayer {
        layer_idx: 0,
        hi_priority: false,
    });
    if has_sub_sprites && has_sub_bg {
        work_items.push(GpuFrameWorkItem::SubSpritePriority(2));
    }
    work_items.push(GpuFrameWorkItem::SubBgLayer {
        layer_idx: 1,
        hi_priority: true,
    });
    work_items.push(GpuFrameWorkItem::SubBgLayer {
        layer_idx: 0,
        hi_priority: true,
    });
    if has_sub_sprites && has_sub_bg {
        work_items.push(GpuFrameWorkItem::SubSpritePriority(3));
    }
    work_items.push(GpuFrameWorkItem::SubBgLayer {
        layer_idx: 2,
        hi_priority: true,
    });

    work_items.push(GpuFrameWorkItem::PostProcess);
    work_items
}

fn main_bg_work_item(layer_idx: usize, hi_priority: bool, math_bit_pos: u32) -> GpuFrameWorkItem {
    GpuFrameWorkItem::MainBgLayer {
        layer_idx,
        hi_priority,
        layer_bit: 1u32 << layer_idx,
        math_bit_pos,
    }
}

fn mode7_work_items(
    has_main_sprites: bool,
    has_sub_mode7_bg: bool,
    has_sub_sprites: bool,
) -> Vec<GpuFrameWorkItem> {
    let mut work_items = Vec::new();
    if has_main_sprites {
        work_items.push(GpuFrameWorkItem::MainSpritePriority(0));
    }
    work_items.push(GpuFrameWorkItem::Mode7MainBg);
    if has_main_sprites {
        work_items.extend((1..=3).map(GpuFrameWorkItem::MainSpritePriority));
    }

    work_items.push(GpuFrameWorkItem::ClearSubBackdrop);
    if has_sub_mode7_bg {
        work_items.push(GpuFrameWorkItem::Mode7SubBg);
    }
    if has_sub_sprites {
        work_items.extend((0..=3).map(GpuFrameWorkItem::SubSpritePriority));
    }
    work_items.push(GpuFrameWorkItem::PostProcess);
    work_items
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
    layer_bit: u32,
    math_bit_pos: u32,
) {
    let is_2bpp = frame.mode == 1 && layer_idx == 2;
    let screen_idx = usize::from(math_bit_pos >= 255);
    let windowed = frame.screen_windowed[screen_idx] & (1u8 << layer_idx) != 0;
    let window_flags = (frame.windowsel >> (layer_idx * 4)) & 0x0f;
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
        window_flags,
        windowed,
        frame.mosaic_enabled & (1u8 << layer_idx) != 0,
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
) {
    let layer_bit = 1u8 << layer_idx;
    if frame.screen_enabled[1] & layer_bit == 0 {
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
        0,   // layer_bit=0: skip per-scanline TM check for sub-screen
        255, // math_bit_pos=255: output alpha=1.0 (real pixel marker)
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

    #[test]
    fn mode1_work_items_preserve_full_gpu_draw_order() {
        let work_items = mode1_work_items(true, true, true, true);

        assert_eq!(
            work_items,
            vec![
                main_bg_work_item(2, false, 2),
                GpuFrameWorkItem::MainSpritePriority(0),
                GpuFrameWorkItem::MainSpritePriority(1),
                main_bg_work_item(1, false, 1),
                main_bg_work_item(0, false, 0),
                GpuFrameWorkItem::MainSpritePriority(2),
                main_bg_work_item(1, true, 1),
                main_bg_work_item(0, true, 0),
                GpuFrameWorkItem::MainSpritePriority(3),
                main_bg_work_item(2, true, 2),
                GpuFrameWorkItem::ClearSubBackdrop,
                GpuFrameWorkItem::SubBgLayer {
                    layer_idx: 2,
                    hi_priority: false,
                },
                GpuFrameWorkItem::SubSpritePriority(0),
                GpuFrameWorkItem::SubSpritePriority(1),
                GpuFrameWorkItem::SubBgLayer {
                    layer_idx: 1,
                    hi_priority: false,
                },
                GpuFrameWorkItem::SubBgLayer {
                    layer_idx: 0,
                    hi_priority: false,
                },
                GpuFrameWorkItem::SubSpritePriority(2),
                GpuFrameWorkItem::SubBgLayer {
                    layer_idx: 1,
                    hi_priority: true,
                },
                GpuFrameWorkItem::SubBgLayer {
                    layer_idx: 0,
                    hi_priority: true,
                },
                GpuFrameWorkItem::SubSpritePriority(3),
                GpuFrameWorkItem::SubBgLayer {
                    layer_idx: 2,
                    hi_priority: true,
                },
                GpuFrameWorkItem::PostProcess,
            ]
        );
    }

    #[test]
    fn mode1_work_items_preserve_sprite_only_draw_order() {
        let work_items = mode1_work_items(false, true, false, true);

        assert_eq!(
            work_items,
            vec![
                GpuFrameWorkItem::MainSpritePriority(0),
                GpuFrameWorkItem::MainSpritePriority(1),
                GpuFrameWorkItem::MainSpritePriority(2),
                GpuFrameWorkItem::MainSpritePriority(3),
                GpuFrameWorkItem::ClearSubBackdrop,
                GpuFrameWorkItem::SubBgLayer {
                    layer_idx: 2,
                    hi_priority: false,
                },
                GpuFrameWorkItem::SubSpritePriority(0),
                GpuFrameWorkItem::SubSpritePriority(1),
                GpuFrameWorkItem::SubSpritePriority(2),
                GpuFrameWorkItem::SubSpritePriority(3),
                GpuFrameWorkItem::SubBgLayer {
                    layer_idx: 1,
                    hi_priority: false,
                },
                GpuFrameWorkItem::SubBgLayer {
                    layer_idx: 0,
                    hi_priority: false,
                },
                GpuFrameWorkItem::SubBgLayer {
                    layer_idx: 1,
                    hi_priority: true,
                },
                GpuFrameWorkItem::SubBgLayer {
                    layer_idx: 0,
                    hi_priority: true,
                },
                GpuFrameWorkItem::SubBgLayer {
                    layer_idx: 2,
                    hi_priority: true,
                },
                GpuFrameWorkItem::PostProcess,
            ]
        );
    }

    #[test]
    fn mode7_work_items_preserve_full_gpu_draw_order() {
        let work_items = mode7_work_items(true, true, true);

        assert_eq!(
            work_items,
            vec![
                GpuFrameWorkItem::MainSpritePriority(0),
                GpuFrameWorkItem::Mode7MainBg,
                GpuFrameWorkItem::MainSpritePriority(1),
                GpuFrameWorkItem::MainSpritePriority(2),
                GpuFrameWorkItem::MainSpritePriority(3),
                GpuFrameWorkItem::ClearSubBackdrop,
                GpuFrameWorkItem::Mode7SubBg,
                GpuFrameWorkItem::SubSpritePriority(0),
                GpuFrameWorkItem::SubSpritePriority(1),
                GpuFrameWorkItem::SubSpritePriority(2),
                GpuFrameWorkItem::SubSpritePriority(3),
                GpuFrameWorkItem::PostProcess,
            ]
        );
    }

    #[test]
    fn mode7_work_items_skip_disabled_surfaces_without_skipping_clears() {
        let work_items = mode7_work_items(false, false, false);

        assert_eq!(
            work_items,
            vec![
                GpuFrameWorkItem::Mode7MainBg,
                GpuFrameWorkItem::ClearSubBackdrop,
                GpuFrameWorkItem::PostProcess,
            ]
        );
    }
}
