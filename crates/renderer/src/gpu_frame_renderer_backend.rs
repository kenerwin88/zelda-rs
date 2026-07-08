use crate::bg_layer::BgLayerRenderer;
use crate::gpu_frame::GpuFrame;
use crate::gpu_frame_work_command::{
    GpuFrameBackdropClearPass, GpuFrameBgPass, GpuFrameMode7Pass, GpuFramePlanBackend,
    GpuFramePostProcessPass, GpuFrameRenderScreen, GpuFrameSpritePass,
};
use crate::mode7_renderer::Mode7Renderer;
use crate::post_process::PostProcessRenderer;
use crate::sprite_renderer::SpriteRenderer;
use crate::tile_atlas::{CgramPalette, TileAtlas};

pub(crate) struct GpuFrameRendererBackend<'a, 'frame> {
    pub(crate) tile_atlas: &'a mut TileAtlas,
    pub(crate) cgram_palette: &'a mut CgramPalette,
    pub(crate) bg: &'a mut [BgLayerRenderer; 3],
    pub(crate) mode7: &'a mut Mode7Renderer,
    pub(crate) sprites: &'a mut SpriteRenderer,
    pub(crate) post_process: &'a mut PostProcessRenderer,
    pub(crate) comp_view: &'a wgpu::TextureView,
    pub(crate) sub_comp_view: &'a wgpu::TextureView,
    pub(crate) encoder: &'a mut wgpu::CommandEncoder,
    pub(crate) queue: &'a wgpu::Queue,
    pub(crate) frame: &'a GpuFrame<'frame>,
    pub(crate) output_view: &'a wgpu::TextureView,
    pub(crate) mode7_source_chars: Option<&'a [u8]>,
}

impl GpuFramePlanBackend for GpuFrameRendererBackend<'_, '_> {
    fn prepare_cgram_palette(&mut self) {
        // The modern mode7-source route (identified by source chars) resolves
        // colors from the provenance mirror when it is complete — the classic
        // route (no source chars) always keeps live CGRAM.
        if self.mode7_source_chars.is_some() {
            if let Some(words) = self.frame.complete_provenance_words() {
                self.cgram_palette.update(self.queue, words);
                return;
            }
            crate::modern_extract::note_live_cgram_fallback(
                "mode7-source palette upload",
                self.frame.cgram_provenance,
            );
        }
        self.cgram_palette.update(self.queue, self.frame.cgram);
    }

    fn prepare_tile_atlas(&mut self) {
        self.tile_atlas.update(self.queue, self.frame.vram);
    }

    fn prepare_mode7_vram(&mut self) {
        self.mode7.prepare_vram(self.queue, self.frame.vram);
        match self.mode7_source_chars {
            Some(chars) => self.mode7.prepare_source_chars(self.queue, chars),
            None => self
                .mode7
                .prepare_source_chars_from_vram(self.queue, self.frame.vram),
        }
    }

    fn prepare_sprites(&mut self) {
        self.sprites.prepare(
            self.queue,
            self.frame.vram,
            self.frame.oam,
            &self.frame.obj,
            self.frame.extra_left_right,
        );
    }

    fn render_backdrop_clear(
        &mut self,
        screen: GpuFrameRenderScreen,
        pass: GpuFrameBackdropClearPass,
    ) {
        let output_view = match screen {
            GpuFrameRenderScreen::Main => self.comp_view,
            GpuFrameRenderScreen::Sub => self.sub_comp_view,
        };
        render_backdrop_clear_pass(self.encoder, self.frame, output_view, pass);
    }

    fn render_sprite_priority(&mut self, screen: GpuFrameRenderScreen, pass: GpuFrameSpritePass) {
        let output_view = match screen {
            GpuFrameRenderScreen::Main => self.comp_view,
            GpuFrameRenderScreen::Sub => self.sub_comp_view,
        };
        render_sprite_pass(
            self.sprites,
            self.encoder,
            self.queue,
            self.frame,
            output_view,
            pass,
        );
    }

    fn render_bg_layer(&mut self, screen: GpuFrameRenderScreen, pass: GpuFrameBgPass) {
        let output_view = match screen {
            GpuFrameRenderScreen::Main => self.comp_view,
            GpuFrameRenderScreen::Sub => self.sub_comp_view,
        };
        render_bg_pass(
            &mut self.bg[pass.layer_idx],
            self.encoder,
            self.queue,
            self.frame,
            output_view,
            pass,
        );
    }

    fn render_mode7_bg(&mut self, screen: GpuFrameRenderScreen, pass: GpuFrameMode7Pass) {
        let output_view = match screen {
            GpuFrameRenderScreen::Main => self.comp_view,
            GpuFrameRenderScreen::Sub => self.sub_comp_view,
        };
        render_mode7_pass(
            self.mode7,
            self.encoder,
            self.queue,
            self.frame,
            output_view,
            pass,
        );
    }

    fn render_post_process(&mut self, pass: GpuFramePostProcessPass) {
        let _ = pass;
        self.post_process
            .render(self.encoder, self.queue, self.frame, self.output_view);
    }
}

fn render_backdrop_clear_pass(
    encoder: &mut wgpu::CommandEncoder,
    frame: &GpuFrame<'_>,
    output_view: &wgpu::TextureView,
    pass: GpuFrameBackdropClearPass,
) {
    let clear_color = match pass {
        GpuFrameBackdropClearPass::MainCgram => {
            cgram_to_wgpu_color(frame.cgram.first().copied().unwrap_or(0))
        }
        GpuFrameBackdropClearPass::SubTransparent => wgpu::Color::TRANSPARENT,
    };
    let label = match pass {
        GpuFrameBackdropClearPass::MainCgram => "backdrop_clear",
        GpuFrameBackdropClearPass::SubTransparent => "sub_backdrop_clear",
    };

    let _clear = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(label),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: output_view,
            resolve_target: None,
            depth_slice: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(clear_color),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
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

fn render_mode7_pass(
    mode7: &mut Mode7Renderer,
    encoder: &mut wgpu::CommandEncoder,
    queue: &wgpu::Queue,
    frame: &GpuFrame<'_>,
    output_view: &wgpu::TextureView,
    pass: GpuFrameMode7Pass,
) {
    let window = pass.window;
    mode7.render_prepared(
        encoder,
        queue,
        frame,
        output_view,
        pass.math_bit_pos,
        pass.layer_bit,
        window.screen_idx,
        window.layer_bit,
        window.flags_shift,
    );
}

fn render_bg_pass(
    bg: &mut BgLayerRenderer,
    encoder: &mut wgpu::CommandEncoder,
    queue: &wgpu::Queue,
    frame: &GpuFrame<'_>,
    output_view: &wgpu::TextureView,
    pass: GpuFrameBgPass,
) {
    if !pass.is_screen_enabled(frame.screen_enabled) {
        return;
    }

    let window = pass.window;
    bg.render(
        encoder,
        queue,
        frame.vram,
        pass.layer_idx,
        &frame.bg[pass.layer_idx],
        output_view,
        wgpu::LoadOp::Load,
        pass.is_2bpp,
        pass.hi_priority,
        pass.render_layer_bit,
        pass.math_bit_pos,
        window.screen_idx,
        window.flags(frame.windowsel),
        window.is_windowed(frame.screen_windowed),
        frame.mosaic_enabled & pass.mosaic_layer_bit != 0,
        frame.mosaic_size,
        &frame.scanlines,
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
