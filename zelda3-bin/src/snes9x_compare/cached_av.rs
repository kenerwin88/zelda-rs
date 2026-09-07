//! Split out of `snes9x_compare.rs` by topic (cached_av). Mechanical move:
//! bodies are unchanged; private items became `pub(crate)` and `super::`
//! paths became `crate::` (the parent's super is the crate root).

use super::*;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CachedOracleAvRecord {
    pub(crate) schema: u32,
    pub(crate) frame: u32,
    pub(crate) input: String,
    pub(crate) oracle_audio_sample_frames: Option<usize>,
    pub(crate) video: Option<parity::av::VideoDigest>,
    pub(crate) audio: Option<parity::av::AudioDigest>,
}

pub(crate) struct PendingCachedAvFrame {
    pub(crate) record: CachedOracleAvRecord,
    pub(crate) replay_input: u16,
    pub(crate) sample_frames: usize,
    pub(crate) rust_audio: Option<serde_json::Value>,
    pub(crate) rust_video: Option<crate::gpu_capture::QueuedGpuVideoDigest>,
    pub(crate) paired_boundary: Option<(u32, Box<ZeldaState>)>,
    pub(crate) compare: bool,
}

pub(crate) fn cached_ledger_input(value: &str) -> Result<u16, String> {
    let digits = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
        .ok_or_else(|| format!("cached input is not hexadecimal: {value}"))?;
    u16::from_str_radix(digits, 16)
        .map_err(|error| format!("invalid cached input {value}: {error}"))
}

pub(crate) fn canonical_audio_digest(samples: &[i16]) -> serde_json::Value {
    let mut digest = parity::evidence::Sha256Digest::new();
    digest.update_i16_le(samples);
    serde_json::json!({
        "sample_frames": samples.len() / 2,
        "channels": 2,
        "sha256": digest.finish(),
    })
}

pub(crate) fn canonical_rust_video_digest(
    rgba: &[u8],
    width: u32,
    height: u32,
) -> Result<serde_json::Value, String> {
    let pixels = width as usize * height as usize;
    let expected = pixels
        .checked_mul(4)
        .ok_or_else(|| format!("Rust video geometry overflows: {width}x{height}"))?;
    if rgba.len() != expected {
        return Err(format!(
            "Rust RGBA byte count {} does not match {width}x{height} ({expected})",
            rgba.len()
        ));
    }
    let mut digest = parity::evidence::Sha256Digest::new();
    digest.update_rgb_from_rgba(std::iter::once(rgba));
    Ok(serde_json::json!({
        "width": width,
        "height": height,
        "sha256": digest.finish(),
    }))
}

pub(crate) fn canonical_oracle_video_digest(
    frame: &LibretroFrame,
) -> Result<serde_json::Value, String> {
    if frame.video.is_empty() {
        return Err("oracle provided no video frame".to_string());
    }
    let stride = snes9x_pixel_stride(frame.pixel_format)
        .ok_or_else(|| format!("unsupported oracle pixel format {}", frame.pixel_format))?;
    let visible_row_bytes = frame.video_width as usize * stride;
    if frame.video_pitch < visible_row_bytes {
        return Err(format!(
            "oracle pitch {} is smaller than visible row {}",
            frame.video_pitch, visible_row_bytes
        ));
    }
    let required =
        frame.video_height.saturating_sub(1) as usize * frame.video_pitch + visible_row_bytes;
    if frame.video.len() < required {
        return Err(format!(
            "oracle video byte count {} is smaller than required {required}",
            frame.video.len()
        ));
    }
    let mut digest = parity::evidence::Sha256Digest::new();
    for y in 0..frame.video_height as usize {
        for x in 0..frame.video_width as usize {
            let offset = y * frame.video_pitch + x * stride;
            let [r, g, b, _] = snes9x_rgba_pixel_at(frame, offset)
                .ok_or_else(|| format!("cannot decode oracle pixel ({x}, {y})"))?;
            digest.update(&[r, g, b]);
        }
    }
    Ok(serde_json::json!({
        "width": frame.video_width,
        "height": frame.video_height,
        "sha256": digest.finish(),
    }))
}

pub(crate) fn canonical_video_digest_pair(
    rust_rgba: &[u8],
    rust_width: u32,
    rust_height: u32,
    oracle: &LibretroFrame,
) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "rust": canonical_rust_video_digest(rust_rgba, rust_width, rust_height)?,
        "oracle": canonical_oracle_video_digest(oracle)?,
    }))
}

pub(crate) fn write_av_hash_record(
    writer: Option<&mut BufWriter<fs::File>>,
    frame: u32,
    input: u16,
    oracle_audio_sample_frames: usize,
    video: Option<serde_json::Value>,
    audio: Option<serde_json::Value>,
) {
    let Some(writer) = writer else {
        return;
    };
    serde_json::to_writer(
        &mut *writer,
        &serde_json::json!({
            "schema": 1,
            "frame": frame,
            "input": format!("0x{input:04x}"),
            "oracle_audio_sample_frames": oracle_audio_sample_frames,
            "video": video,
            "audio": audio,
        }),
    )
    .unwrap_or_else(|error| {
        eprintln!("failed to write canonical A/V hash record: {error}");
        process::exit(1);
    });
    writer.write_all(b"\n").unwrap_or_else(|error| {
        eprintln!("failed to terminate canonical A/V hash record: {error}");
        process::exit(1);
    });
}
