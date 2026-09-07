//! Split out of `snes9x_compare.rs` by topic (video_compare). Mechanical move:
//! bodies are unchanged; private items became `pub(crate)` and `super::`
//! paths became `crate::` (the parent's super is the crate root).

use super::*;

pub(crate) const fn should_render_video_frame(
    frame: u32,
    compare_from_frame: u32,
    video_requested: bool,
) -> bool {
    video_requested && frame.saturating_add(VIDEO_WARMUP_PRIMING_FRAMES) >= compare_from_frame
}

pub(crate) fn compare_snes9x_video_frame(
    rust_frame: &[u8],
    rust_width: u32,
    rust_height: u32,
    snes9x: &LibretroFrame,
) -> Option<String> {
    compare_libretro_video_frame(rust_frame, rust_width, rust_height, snes9x, 0, 0)
}

pub(crate) fn rgb_within_tolerance(mine: [u8; 4], theirs: [u8; 4], tolerance: u8) -> bool {
    mine[..3]
        .iter()
        .zip(theirs[..3].iter())
        .all(|(&mine, &theirs)| mine.abs_diff(theirs) <= tolerance)
}

pub(crate) fn align_snes9x_video_capture(
    snes9x: &mut LibretroCore,
    mut capture: LibretroFrame,
    rust_frame: &[u8],
    width: u32,
    height: u32,
    input: u16,
    max_extra_frames: u32,
    color_tolerance: u8,
    max_mismatched_pixels: usize,
) -> (LibretroFrame, u32, bool) {
    if compare_libretro_video_frame(
        rust_frame,
        width,
        height,
        &capture,
        color_tolerance,
        max_mismatched_pixels,
    )
    .is_none()
    {
        return (capture, 0, true);
    }
    for extra in 1..=max_extra_frames {
        capture = snes9x.run_frame_with_input(input);
        if compare_libretro_video_frame(
            rust_frame,
            width,
            height,
            &capture,
            color_tolerance,
            max_mismatched_pixels,
        )
        .is_none()
        {
            return (capture, extra, true);
        }
    }
    println!("auto-align video found no RGB match within {max_extra_frames} extra snes9x frame(s)");
    (capture, max_extra_frames, false)
}

pub(crate) fn rgba_pixel_at(frame: &[u8], offset: usize) -> Option<[u8; 4]> {
    let bytes = frame.get(offset..offset + 4)?;
    Some([bytes[0], bytes[1], bytes[2], bytes[3]])
}

pub(crate) fn snes9x_pixel_stride(pixel_format: u32) -> Option<usize> {
    match pixel_format {
        0 | 2 => Some(2),
        1 => Some(4),
        _ => None,
    }
}

pub(crate) fn snes9x_rgba_pixel_at(frame: &LibretroFrame, offset: usize) -> Option<[u8; 4]> {
    match frame.pixel_format {
        0 => {
            let lo = *frame.video.get(offset)? as u16;
            let hi = *frame.video.get(offset + 1)? as u16;
            let raw = lo | (hi << 8);
            Some([
                expand_5_to_8((raw >> 10) & 0x1f),
                expand_5_to_8((raw >> 5) & 0x1f),
                expand_5_to_8(raw & 0x1f),
                0xff,
            ])
        }
        1 => {
            let bytes = frame.video.get(offset..offset + 4)?;
            Some([bytes[2], bytes[1], bytes[0], 0xff])
        }
        2 => {
            let lo = *frame.video.get(offset)? as u16;
            let hi = *frame.video.get(offset + 1)? as u16;
            let raw = lo | (hi << 8);
            Some([
                expand_5_to_8((raw >> 11) & 0x1f),
                // Snes9x expands the SNES five-bit green channel into RGB565.
                // Collapse the duplicated low bit before comparing with the
                // modern renderer's RGB555-equivalent output.
                expand_5_to_8(((raw >> 5) & 0x3f) >> 1),
                expand_5_to_8(raw & 0x1f),
                0xff,
            ])
        }
        _ => None,
    }
}

pub(crate) fn expand_5_to_8(value: u16) -> u8 {
    ((value << 3) | (value >> 2)) as u8
}

pub(crate) fn render_full_apu_audio(
    apu: &mut snes::apu::ApuState,
    audio: &mut [i16],
    samples: usize,
    channels: usize,
) {
    let mut guard = 0usize;
    while apu.dsp.sample_offset < 534 && guard < 32_000 {
        apu.cycle();
        guard += 1;
    }
    if apu.dsp.sample_offset < 534 {
        audio.fill(0);
        return;
    }
    apu.dsp.get_samples(audio, samples, channels);
}

pub(crate) fn render_full_apu_audio_exact(
    apu: &mut snes::apu::ApuState,
    audio: &mut [i16],
    samples: usize,
    channels: usize,
) -> Result<(), String> {
    let mut guard = 0usize;
    let guard_limit = samples.saturating_mul(32).saturating_add(64);
    while (apu.dsp.sample_offset as usize) < samples && guard < guard_limit {
        apu.cycle();
        guard += 1;
    }
    if (apu.dsp.sample_offset as usize) < samples {
        return Err(format!(
            "APU produced only {} of {samples} requested exact samples after {guard} clocks",
            apu.dsp.sample_offset
        ));
    }
    apu.dsp.drain_samples_exact(audio, samples, channels)
}
