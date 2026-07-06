use crate::gpu_capture::{capture_gpu_frame_from_game, LiveGpuFrameCapture};
use snes::ppu::PpuRenderFlags;
use zelda3::ZeldaState;

pub(crate) struct GpuReadbackRenderer {
    offscreen: renderer::OffscreenRenderer,
}

pub(crate) struct GpuRgbaReadbackFrame {
    rgba: Vec<u8>,
}

pub(crate) struct OptionalGpuReadbackRenderer {
    renderer: Option<GpuReadbackRenderer>,
}

pub(crate) struct ReplayRenderHashCapture {
    capture: LiveGpuFrameCapture,
}

pub(crate) struct ReplayRenderHashGpuReadback {
    frame: GpuRgbaReadbackFrame,
}

impl GpuReadbackRenderer {
    pub(crate) fn new(width: u32, height: u32) -> Self {
        Self {
            offscreen: pollster::block_on(renderer::OffscreenRenderer::new(width, height)),
        }
    }

    fn render_gpu_capture_rgba(&mut self, capture: &LiveGpuFrameCapture) -> GpuRgbaReadbackFrame {
        GpuRgbaReadbackFrame {
            rgba: self.offscreen.render_gpu_frame(&capture.gpu_frame()),
        }
    }

    fn render_bgra_frame_to_rgba(&mut self, frame: &[u8]) -> GpuRgbaReadbackFrame {
        self.offscreen.upload_bgra_frame(frame);
        GpuRgbaReadbackFrame {
            rgba: self.offscreen.render_to_rgba(),
        }
    }

    pub(crate) fn render_live_gpu_capture_rgba(
        &mut self,
        capture: &LiveGpuFrameCapture,
    ) -> GpuRgbaReadbackFrame {
        self.render_gpu_capture_rgba(capture)
    }

    pub(crate) fn render_cpu_bgra_frame_rgba(&mut self, frame: &[u8]) -> GpuRgbaReadbackFrame {
        self.render_bgra_frame_to_rgba(frame)
    }
}

impl OptionalGpuReadbackRenderer {
    pub(crate) fn new(required: bool, width: u32, height: u32) -> Self {
        Self {
            renderer: required.then(|| GpuReadbackRenderer::new(width, height)),
        }
    }

    pub(crate) fn required(&mut self) -> &mut GpuReadbackRenderer {
        self.renderer
            .as_mut()
            .expect("GPU readback renderer allocated")
    }

    pub(crate) fn render_live_gpu_capture_rgba(
        &mut self,
        capture: &LiveGpuFrameCapture,
    ) -> GpuRgbaReadbackFrame {
        self.required().render_live_gpu_capture_rgba(capture)
    }

    pub(crate) fn render_cpu_bgra_frame_rgba(&mut self, frame: &[u8]) -> GpuRgbaReadbackFrame {
        self.required().render_cpu_bgra_frame_rgba(frame)
    }

    pub(crate) fn capture_replay_render_hash_frame(
        &self,
        game: &mut ZeldaState,
    ) -> ReplayRenderHashCapture {
        ReplayRenderHashCapture {
            capture: capture_gpu_frame_from_game(game),
        }
    }

    pub(crate) fn render_replay_hash_cpu_frame_rgba(
        &mut self,
        game: &mut ZeldaState,
        frame: &mut [u8],
    ) -> GpuRgbaReadbackFrame {
        let width = 256usize;
        crate::classic_frame_renderer::render_play_frame_bgra(
            game,
            frame,
            width * 4,
            PpuRenderFlags::empty(),
        );
        self.render_cpu_bgra_frame_rgba(frame)
    }
}

impl ReplayRenderHashCapture {
    pub(crate) fn render_gpu_rgba(
        &self,
        readback: &mut OptionalGpuReadbackRenderer,
    ) -> ReplayRenderHashGpuReadback {
        ReplayRenderHashGpuReadback {
            frame: readback.render_live_gpu_capture_rgba(&self.capture),
        }
    }

    fn cgram_color(&self, index: usize) -> u16 {
        self.capture.cgram().get(index).copied().unwrap_or(0)
    }

    pub(crate) fn cgram_color_hex(&self, index: usize) -> String {
        format!("{:#06x}", self.cgram_color(index))
    }

    pub(crate) fn debug_frame_800_scanline_screen_enabled_main_line(&self) -> String {
        let values = self.capture.raw_scanlines()[60..70]
            .iter()
            .map(|e| e.4)
            .collect::<Vec<_>>();
        format!("[gpu-dbg] f800 scanlines[60..70] screen_enabled_main: {values:?}")
    }

    pub(crate) fn debug_cgram_render_diff_lines(
        &self,
        frame: u32,
        post_cgram: &[u16],
    ) -> Vec<String> {
        let diffs = self
            .capture
            .cgram()
            .iter()
            .enumerate()
            .zip(post_cgram.iter())
            .filter(|((_, &h), &p)| h != p)
            .map(|((i, &h), &p)| (i, h, p))
            .collect::<Vec<_>>();
        let mut lines = Vec::with_capacity(1 + diffs.len().min(20));
        lines.push(format!(
            "[gpu-dbg] frame={frame} CGRAM changes during render: {} entries differ",
            diffs.len()
        ));
        lines.extend(diffs.iter().take(20).map(|(i, before, after)| {
            format!("[gpu-dbg]   cgram[{i}]: {before:#06x} -> {after:#06x}")
        }));
        lines
    }

    pub(crate) fn debug_cgram_value_lines(
        &self,
        frame: u32,
        label: &str,
        value: u16,
    ) -> Vec<String> {
        self.capture
            .cgram()
            .iter()
            .enumerate()
            .filter_map(|(index, &cgram_value)| {
                (cgram_value == value)
                    .then(|| format!("[gpu-dbg] frame={frame} {label}[{index}]={value:#06x}"))
            })
            .collect()
    }

    pub(crate) fn debug_math_state_line(&self) -> String {
        let gpu_frame = self.capture.gpu_frame();
        format!(
            "[gpu-dbg] math_enabled={:#04x} subtract={} half={} fixed_rgb=({},{},{}) add_sub={} clip_mode={} prevent_math={} windowsel_cm={:#04x} brightness={}",
            gpu_frame.math_enabled,
            gpu_frame.subtract_color,
            gpu_frame.half_color,
            gpu_frame.fixed_color_r,
            gpu_frame.fixed_color_g,
            gpu_frame.fixed_color_b,
            gpu_frame.add_subscreen,
            gpu_frame.clip_mode,
            gpu_frame.prevent_math_mode,
            gpu_frame.windowsel_cm,
            gpu_frame.brightness
        )
    }

    pub(crate) fn debug_frame_332_math_line(&self) -> String {
        let gpu_frame = self.capture.gpu_frame();
        format!(
            "[gpu-dbg] frame=332 math_enabled={:#04x} subtract={} half={} fixed=({},{},{}) clip_mode={} prevent_math={} windowsel_cm={:#04x} add_sub={}",
            gpu_frame.math_enabled,
            gpu_frame.subtract_color,
            gpu_frame.half_color,
            gpu_frame.fixed_color_r,
            gpu_frame.fixed_color_g,
            gpu_frame.fixed_color_b,
            gpu_frame.clip_mode,
            gpu_frame.prevent_math_mode,
            gpu_frame.windowsel_cm,
            gpu_frame.add_subscreen
        )
    }

    pub(crate) fn debug_frame_332_scanline_window_line(&self) -> String {
        let gpu_frame = self.capture.gpu_frame();
        format!(
            "[gpu-dbg] frame=332 scanline[0]: w1l={} w1r={}",
            gpu_frame.scanlines[0].window1_left, gpu_frame.scanlines[0].window1_right
        )
    }

    pub(crate) fn debug_effect_math_line(
        &self,
        frame: u32,
        bg1_hscroll: u16,
        irq_flag: u8,
    ) -> String {
        let gpu_frame = self.capture.gpu_frame();
        format!(
            "[gpu-dbg] f{frame} math={:#04x} add_sub={} subtract={} half={} fixed_r={} fixed_g={} fixed_b={} bg1_hscroll={} irq_flag={}",
            gpu_frame.math_enabled,
            gpu_frame.add_subscreen,
            gpu_frame.subtract_color,
            gpu_frame.half_color,
            gpu_frame.fixed_color_r,
            gpu_frame.fixed_color_g,
            gpu_frame.fixed_color_b,
            bg1_hscroll,
            irq_flag
        )
    }

    pub(crate) fn debug_scanline_tm_probe_line(&self, frame: u32, cy: i32) -> String {
        let gpu_frame = self.capture.gpu_frame();
        format!(
            "[gpu-dbg] f{frame} scanline_tm row{}={:#04x} row{}={:#04x}",
            cy,
            gpu_frame.scanlines[cy as usize].screen_enabled_main,
            cy + 1,
            gpu_frame.scanlines[(cy + 1) as usize].screen_enabled_main
        )
    }
}

impl GpuRgbaReadbackFrame {
    pub(crate) fn from_rgba(rgba: Vec<u8>) -> Self {
        Self { rgba }
    }

    pub(crate) fn as_slice(&self) -> &[u8] {
        &self.rgba
    }

    fn render_hash_log_line(&self, frame: u32) -> String {
        renderer::render_hash_frame_rgba(frame, &self.rgba).line
    }

    fn gpu_render_hash_line(&self, frame: u32) -> String {
        renderer::gpu_render_hash_frame_rgba(frame, &self.rgba).line
    }

    fn hash_pair_with_cpu_bgra(&self, cpu_bgra: &[u8]) -> renderer::RenderHashPair {
        renderer::render_hash_pair_bgra_rgba(cpu_bgra, &self.rgba)
    }
}

impl ReplayRenderHashGpuReadback {
    pub(crate) fn render_hash_log_line(&self, frame: u32) -> String {
        self.frame.render_hash_log_line(frame)
    }

    pub(crate) fn gpu_render_hash_log_line(&self, frame: u32) -> String {
        self.frame.gpu_render_hash_line(frame)
    }

    pub(crate) fn debug_hash_line_with_cpu_bgra(&self, frame: u32, cpu_bgra: &[u8]) -> String {
        let hashes = self.frame.hash_pair_with_cpu_bgra(cpu_bgra);
        format!(
            "[gpu-dbg] frame={frame} cpu_hash={:#010x} gpu_hash={:#010x}",
            hashes.cpu_hash, hashes.gpu_hash
        )
    }
}

impl std::ops::Deref for GpuRgbaReadbackFrame {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.rgba
    }
}

impl std::ops::Deref for ReplayRenderHashGpuReadback {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.frame.as_slice()
    }
}
