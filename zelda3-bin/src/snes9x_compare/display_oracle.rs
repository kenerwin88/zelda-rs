//! Split out of `snes9x_compare.rs` by topic (display_oracle). Mechanical move:
//! bodies are unchanged; private items became `pub(crate)` and `super::`
//! paths became `crate::` (the parent's super is the crate root).

use super::*;

/// Display-domain receipt captured from the Snes9x PPU and the immutable Rust
/// scanout snapshot. This is deliberately upstream of RGBA comparison: it
/// tells us whether a failure is a ROM/state-publication issue or a renderer
/// issue before any pixel-level investigation begins.
#[derive(Debug, Serialize)]
pub(crate) struct DisplayOracleReceipt {
    pub(crate) frame: u32,
    pub(crate) stage: &'static str,
    pub(crate) oracle: DisplayPpuProbe,
    pub(crate) rust: DisplayPpuProbe,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) rust_candidates: Vec<DisplayPublicationCandidateProbe>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) rust_context: Option<DisplayPublicationContextProbe>,
}

#[derive(Debug, Serialize)]
pub(crate) struct DisplayPublicationCandidateProbe {
    pub(crate) name: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) cgram: Option<Vec<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) cgram_difference: Option<ValueDomainDiff>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) presented_oam: Option<Vec<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) presented_oam_difference: Option<ValueDomainDiff>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) presented_obj_tile_cache: Option<Vec<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) presented_obj_tile_cache_valid_difference: Option<ValueDomainDiff>,
}

#[derive(Debug, Serialize)]
pub(crate) struct DisplayPublicationContextProbe {
    pub(crate) render_host_frame: u32,
    pub(crate) publication_host_frame: u32,
    pub(crate) entry_frame: [u8; 4],
    pub(crate) following_frame: [u8; 4],
    pub(crate) dungeon_room: u8,
    pub(crate) staircase_index: u8,
    pub(crate) palette_filter_countdown: u16,
    pub(crate) nmi_update_latch: u8,
    pub(crate) oam_scanout_source: String,
    pub(crate) retain_captured_oam: bool,
    pub(crate) link_obj_scanout_generation: String,
    pub(crate) link_obj_source_generation: String,
    pub(crate) captured_to_host_oam_mismatches: usize,
    pub(crate) oam_dma_completed_after_active_scanout: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct DisplayPpuProbe {
    pub(crate) capture_source: &'static str,
    pub(crate) comparison_frame: Option<u32>,
    pub(crate) host_frame: Option<u32>,
    pub(crate) mode: i32,
    pub(crate) brightness: i32,
    pub(crate) scanout_brightness_override: Option<u8>,
    pub(crate) forced_blank: bool,
    pub(crate) brightness_white: i32,
    pub(crate) cgram: Vec<i32>,
    pub(crate) fixed_color: [i32; 3],
    pub(crate) display_control: [i32; 6],
    pub(crate) bg_scroll: [i32; 8],
    /// OAM visible after the most recent DMA and the generation actually used
    /// for the completed scanout. Snes9x can advance the former before the
    /// host observes the latter, so keep both domains explicit.
    pub(crate) oam: Vec<i32>,
    pub(crate) presented_oam: Vec<i32>,
    pub(crate) mode7: [i32; 8],
    pub(crate) mode7_scanlines: Vec<[i32; 8]>,
    /// Window 1/2 left/right positions actually consumed for each completed
    /// scanline, after HDMA writes.
    pub(crate) window_scanlines: Vec<[i32; 4]>,
    /// Snes9x's renderer-resolved main/sub clip spans for BG1..backdrop.
    ///
    /// Each of the twelve clips contributes Count followed by six
    /// (DrawMode, Left, Right) spans. Rust does not cache this derived form,
    /// so its receipt leaves the field absent.
    pub(crate) presented_clip: Option<Vec<i32>>,
    /// Final Snes9x draw operands for `ZELDA3_SNES9X_TRACE_PIXEL=x,y`.
    pub(crate) presented_pixel: Option<Vec<i32>>,
    /// Snes9x's completed per-scanline OBJ evaluation. Raw OAM can be exact
    /// while the evaluated sprite rows still belong to an earlier PPU
    /// boundary, so preserve the derived list separately.
    pub(crate) presented_obj: Option<PresentedObjEvaluation>,
    /// Decoded 4bpp OBJ tile cache retained from the completed scanout. The
    /// raw VRAM image can advance before libretro returns, while this cache
    /// still contains the pixels Snes9x actually drew.
    pub(crate) presented_obj_tile_cache: Option<Vec<i32>>,
    pub(crate) presented_obj_tile_cache_valid: Option<Vec<i32>>,
}

#[derive(Debug, Serialize)]
pub(crate) struct PresentedObjEvaluation {
    pub(crate) lines: Vec<PresentedObjLine>,
    pub(crate) visible_tiles: Vec<i32>,
    pub(crate) widths: Vec<i32>,
}

#[derive(Debug, Serialize)]
pub(crate) struct PresentedObjLine {
    pub(crate) sprites: Vec<i32>,
    pub(crate) rows: Vec<i32>,
    pub(crate) tiles_remaining: i32,
    pub(crate) range_time_over: i32,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub(crate) struct ValueDomainDiff {
    pub(crate) rust_values: usize,
    pub(crate) oracle_values: usize,
    pub(crate) mismatched_values: usize,
    pub(crate) first_mismatch: Option<usize>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub(crate) struct VramDomainReceipt {
    pub(crate) rust_words: usize,
    pub(crate) oracle_words: usize,
    pub(crate) rust_sha256: String,
    pub(crate) oracle_sha256: String,
    pub(crate) mismatched_words: usize,
    pub(crate) first_mismatch_word: Option<usize>,
    pub(crate) first_rust_word: Option<u16>,
    pub(crate) first_oracle_word: Option<u16>,
    pub(crate) mismatch_ranges: Vec<[usize; 2]>,
    pub(crate) mismatch_ranges_truncated: bool,
    /// `[block_start_word, mismatched_words]` for every non-exact 0x100-word
    /// block. Unlike the capped exact ranges, this always covers all VRAM.
    pub(crate) mismatch_blocks: Vec<[usize; 2]>,
}

pub(crate) fn vram_domain_receipt(
    rust_words: &[u16],
    oracle_bytes: Option<&[u8]>,
) -> Option<VramDomainReceipt> {
    const MAX_MISMATCH_RANGES: usize = 16;
    let oracle_bytes = oracle_bytes?;
    let oracle_words = oracle_bytes
        .chunks_exact(2)
        .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
        .collect::<Vec<_>>();
    let rust_bytes = rust_words
        .iter()
        .flat_map(|word| word.to_le_bytes())
        .collect::<Vec<_>>();
    let first_mismatch_word = rust_words
        .iter()
        .zip(&oracle_words)
        .position(|(rust, oracle)| rust != oracle)
        .or_else(|| {
            (rust_words.len() != oracle_words.len())
                .then(|| rust_words.len().min(oracle_words.len()))
        });
    let mut mismatched_words = rust_words.len().abs_diff(oracle_words.len());
    let mut mismatch_ranges = Vec::new();
    let mut mismatch_blocks = Vec::new();
    let mut open_range = None;
    let mut mismatch_ranges_truncated = false;
    for (index, (rust, oracle)) in rust_words.iter().zip(&oracle_words).enumerate() {
        if rust != oracle {
            mismatched_words += 1;
            open_range.get_or_insert(index);
        } else if let Some(start) = open_range.take() {
            if mismatch_ranges.len() < MAX_MISMATCH_RANGES {
                mismatch_ranges.push([start, index]);
            } else {
                mismatch_ranges_truncated = true;
            }
        }
    }
    if let Some(start) = open_range {
        if mismatch_ranges.len() < MAX_MISMATCH_RANGES {
            mismatch_ranges.push([start, rust_words.len().min(oracle_words.len())]);
        } else {
            mismatch_ranges_truncated = true;
        }
    }
    if rust_words.len() != oracle_words.len() {
        let start = rust_words.len().min(oracle_words.len());
        let end = rust_words.len().max(oracle_words.len());
        if mismatch_ranges.len() < MAX_MISMATCH_RANGES {
            mismatch_ranges.push([start, end]);
        } else {
            mismatch_ranges_truncated = true;
        }
    }
    for block_start in (0..rust_words.len().max(oracle_words.len())).step_by(0x100) {
        let rust = rust_words.get(block_start..).unwrap_or_default();
        let oracle = oracle_words.get(block_start..).unwrap_or_default();
        let rust = &rust[..rust.len().min(0x100)];
        let oracle = &oracle[..oracle.len().min(0x100)];
        let mismatches = rust.iter().zip(oracle).filter(|(a, b)| a != b).count()
            + rust.len().abs_diff(oracle.len());
        if mismatches != 0 {
            mismatch_blocks.push([block_start, mismatches]);
        }
    }
    Some(VramDomainReceipt {
        rust_words: rust_words.len(),
        oracle_words: oracle_words.len(),
        rust_sha256: parity::evidence::sha256_bytes(&rust_bytes),
        oracle_sha256: parity::evidence::sha256_bytes(oracle_bytes),
        mismatched_words,
        first_mismatch_word,
        first_rust_word: first_mismatch_word.and_then(|index| rust_words.get(index).copied()),
        first_oracle_word: first_mismatch_word.and_then(|index| oracle_words.get(index).copied()),
        mismatch_ranges,
        mismatch_ranges_truncated,
        mismatch_blocks,
    })
}

impl ValueDomainDiff {
    pub(crate) fn is_exact(&self) -> bool {
        self.mismatched_values == 0
    }
}

pub(crate) fn summarize_value_domain<T: PartialEq>(rust: &[T], oracle: &[T]) -> ValueDomainDiff {
    let mismatched_shared = rust
        .iter()
        .zip(oracle)
        .filter(|(rust, oracle)| rust != oracle)
        .count();
    let first_mismatch = rust
        .iter()
        .zip(oracle)
        .position(|(rust, oracle)| rust != oracle)
        .or_else(|| (rust.len() != oracle.len()).then(|| rust.len().min(oracle.len())));
    ValueDomainDiff {
        rust_values: rust.len(),
        oracle_values: oracle.len(),
        mismatched_values: mismatched_shared + rust.len().abs_diff(oracle.len()),
        first_mismatch,
    }
}

pub(crate) fn summarize_presented_obj_cache(
    rust: Option<&[i32]>,
    oracle: Option<&[i32]>,
    oracle_valid: Option<&[i32]>,
) -> Option<ValueDomainDiff> {
    let (rust, oracle, oracle_valid) = (rust?, oracle?, oracle_valid?);
    let comparable_pixels = oracle_valid
        .iter()
        .take(64)
        .filter(|&&valid| valid != 0)
        .count()
        * 64;
    let mut mismatched_values = 0;
    let mut first_mismatch = None;
    for (tile, &valid) in oracle_valid.iter().take(64).enumerate() {
        if valid == 0 {
            continue;
        }
        for pixel in 0..64 {
            let index = tile * 64 + pixel;
            if rust.get(index) != oracle.get(index) {
                mismatched_values += 1;
                first_mismatch.get_or_insert(index);
            }
        }
    }
    Some(ValueDomainDiff {
        rust_values: comparable_pixels,
        oracle_values: comparable_pixels,
        mismatched_values,
        first_mismatch,
    })
}

#[derive(Debug, Serialize)]
pub(crate) struct DisplayOracleDifferences {
    pub(crate) divergent_domains: Vec<&'static str>,
    pub(crate) registers: ValueDomainDiff,
    pub(crate) cgram: ValueDomainDiff,
    pub(crate) live_oam: ValueDomainDiff,
    pub(crate) presented_oam: ValueDomainDiff,
    pub(crate) presented_obj_tile_cache: Option<ValueDomainDiff>,
    pub(crate) window1_scanlines: ValueDomainDiff,
    pub(crate) window2_scanlines: ValueDomainDiff,
    pub(crate) mode7: Option<ValueDomainDiff>,
    pub(crate) mode7_scanlines: Option<ValueDomainDiff>,
}

pub(crate) fn display_register_values(probe: &DisplayPpuProbe) -> Vec<i32> {
    let mut values = vec![probe.mode, probe.brightness, i32::from(probe.forced_blank)];
    values.extend(probe.fixed_color);
    values.extend(probe.display_control);
    values.extend(probe.bg_scroll);
    values
}

pub(crate) fn display_oracle_differences(
    receipt: &DisplayOracleReceipt,
) -> DisplayOracleDifferences {
    let rust_registers = display_register_values(&receipt.rust);
    let oracle_registers = display_register_values(&receipt.oracle);
    let rust_scanlines = receipt
        .rust
        .mode7_scanlines
        .iter()
        .flatten()
        .copied()
        .collect::<Vec<_>>();
    let oracle_scanlines = receipt
        .oracle
        .mode7_scanlines
        .iter()
        .flatten()
        .copied()
        .collect::<Vec<_>>();
    let window_scanlines = |probe: &DisplayPpuProbe, window: usize| {
        probe
            .window_scanlines
            .iter()
            .flat_map(|scanline| scanline[window * 2..window * 2 + 2].iter().copied())
            .collect::<Vec<_>>()
    };
    let rust_window1_scanlines = window_scanlines(&receipt.rust, 0);
    let oracle_window1_scanlines = window_scanlines(&receipt.oracle, 0);
    let rust_window2_scanlines = window_scanlines(&receipt.rust, 1);
    let oracle_window2_scanlines = window_scanlines(&receipt.oracle, 1);
    let registers = summarize_value_domain(&rust_registers, &oracle_registers);
    let cgram = summarize_value_domain(&receipt.rust.cgram, &receipt.oracle.cgram);
    let live_oam = summarize_value_domain(&receipt.rust.oam, &receipt.oracle.oam);
    let presented_oam =
        summarize_value_domain(&receipt.rust.presented_oam, &receipt.oracle.presented_oam);
    let presented_obj_tile_cache = summarize_presented_obj_cache(
        receipt.rust.presented_obj_tile_cache.as_deref(),
        receipt.oracle.presented_obj_tile_cache.as_deref(),
        receipt.oracle.presented_obj_tile_cache_valid.as_deref(),
    );
    let window1_scanlines =
        summarize_value_domain(&rust_window1_scanlines, &oracle_window1_scanlines);
    let window2_scanlines =
        summarize_value_domain(&rust_window2_scanlines, &oracle_window2_scanlines);
    let mode7_active = receipt.rust.mode == 7 || receipt.oracle.mode == 7;
    let mode7 =
        mode7_active.then(|| summarize_value_domain(&receipt.rust.mode7, &receipt.oracle.mode7));
    let mode7_scanlines =
        mode7_active.then(|| summarize_value_domain(&rust_scanlines, &oracle_scanlines));
    let mut divergent_domains = [
        ("registers", &registers),
        ("cgram", &cgram),
        ("live_oam", &live_oam),
        ("presented_oam", &presented_oam),
        ("window1_scanlines", &window1_scanlines),
        ("window2_scanlines", &window2_scanlines),
    ]
    .into_iter()
    .filter_map(|(name, difference)| (!difference.is_exact()).then_some(name))
    .collect::<Vec<_>>();
    if mode7
        .as_ref()
        .is_some_and(|difference| !difference.is_exact())
    {
        divergent_domains.push("mode7");
    }
    if mode7_scanlines
        .as_ref()
        .is_some_and(|difference| !difference.is_exact())
    {
        divergent_domains.push("mode7_scanlines");
    }
    if presented_obj_tile_cache
        .as_ref()
        .is_some_and(|difference| !difference.is_exact())
    {
        divergent_domains.push("presented_obj_tile_cache");
    }
    DisplayOracleDifferences {
        divergent_domains,
        registers,
        cgram,
        live_oam,
        presented_oam,
        presented_obj_tile_cache,
        window1_scanlines,
        window2_scanlines,
        mode7,
        mode7_scanlines,
    }
}

pub(crate) fn capture_oracle_ppu_probe(oracle: &LibretroCore) -> Option<DisplayPpuProbe> {
    oracle.debug_ppu_value(0, 0)?;
    let presented_obj_tile_cache_supported = oracle
        .debug_ppu_value(29, 0)
        .is_some_and(|value| value >= 0);
    let presented_obj = PresentedObjEvaluation {
        lines: (0..224)
            .map(|line| {
                let sprites = (0..128)
                    .map(|slot| oracle.debug_ppu_value(21, line * 128 + slot).unwrap_or(-1))
                    .take_while(|sprite| *sprite >= 0)
                    .collect::<Vec<_>>();
                let rows = (0..sprites.len())
                    .map(|slot| {
                        oracle
                            .debug_ppu_value(26, line * 128 + slot as i32)
                            .unwrap_or(-1)
                    })
                    .collect();
                PresentedObjLine {
                    sprites,
                    rows,
                    tiles_remaining: oracle.debug_ppu_value(22, line).unwrap_or(-1),
                    range_time_over: oracle.debug_ppu_value(23, line).unwrap_or(-1),
                }
            })
            .collect(),
        visible_tiles: (0..128)
            .map(|sprite| oracle.debug_ppu_value(24, sprite).unwrap_or(-1))
            .collect(),
        widths: (0..128)
            .map(|sprite| oracle.debug_ppu_value(25, sprite).unwrap_or(-1))
            .collect(),
    };
    Some(DisplayPpuProbe {
        capture_source: "snes9x_last_completed_scanout",
        comparison_frame: None,
        host_frame: None,
        mode: oracle.debug_ppu_value(0, 0)?,
        brightness: oracle.debug_ppu_value(1, 0)?,
        scanout_brightness_override: None,
        forced_blank: oracle.debug_ppu_value(7, 0)? != 0,
        brightness_white: oracle.debug_ppu_value(8, 0)?,
        cgram: (0..256)
            .map(|i| oracle.debug_ppu_value(2, i).unwrap_or(-1))
            .collect(),
        fixed_color: std::array::from_fn(|i| oracle.debug_ppu_value(4, i as i32).unwrap_or(-1)),
        display_control: std::array::from_fn(|i| {
            oracle.debug_ppu_value(16, i as i32).unwrap_or(-1)
        }),
        bg_scroll: std::array::from_fn(|i| oracle.debug_ppu_value(14, i as i32).unwrap_or(-1)),
        oam: (0..544)
            .map(|i| oracle.debug_ppu_value(15, i).unwrap_or(-1))
            .collect(),
        presented_oam: (0..544)
            .map(|i| oracle.debug_ppu_value(20, i).unwrap_or(-1))
            .collect(),
        mode7: std::array::from_fn(|i| oracle.debug_ppu_value(5, i as i32).unwrap_or(-1)),
        mode7_scanlines: (0..224)
            .map(|line| {
                std::array::from_fn(|field| {
                    oracle
                        .debug_scanline_mode7_value(line, field as i32)
                        .unwrap_or(-1)
                })
            })
            .collect(),
        window_scanlines: (0..224)
            .map(|line| {
                std::array::from_fn(|field| {
                    oracle
                        .debug_scanline_mode7_value(line, field as i32 + 8)
                        .unwrap_or(-1)
                })
            })
            .collect(),
        presented_clip: Some(
            (0..228)
                .map(|i| oracle.debug_ppu_value(27, i).unwrap_or(-1))
                .collect(),
        ),
        presented_pixel: Some(
            (0..10)
                .map(|i| oracle.debug_ppu_value(28, i).unwrap_or(-1))
                .collect(),
        ),
        presented_obj: Some(presented_obj),
        // Older trace cores return -1 for unknown probe fields. Treat that as
        // an unavailable capability, not 4096 bytes of cache divergence.
        presented_obj_tile_cache: presented_obj_tile_cache_supported.then(|| {
            (0..64 * 64)
                .map(|index| oracle.debug_ppu_value(29, index).unwrap_or(-1))
                .collect()
        }),
        presented_obj_tile_cache_valid: presented_obj_tile_cache_supported.then(|| {
            (0..64)
                .map(|index| oracle.debug_ppu_value(30, index).unwrap_or(-1))
                .collect()
        }),
    })
}

pub(crate) fn capture_rust_ppu_probe(
    game: &mut ZeldaState,
) -> (
    DisplayPpuProbe,
    Vec<DisplayPublicationCandidateProbe>,
    Option<DisplayPublicationContextProbe>,
) {
    let live_oam = game
        .ppu
        .oam
        .iter()
        .flat_map(|word| word.to_le_bytes().map(i32::from))
        .collect();
    game.with_display_snapshot(move |snapshot| {
        let scanlines = snapshot.ppu_scanline_windows();
        capture_rust_ppu_probe_from_presented(
            snapshot,
            &snapshot.ppu,
            &scanlines,
            live_oam,
            "rust_recomposed_display_snapshot",
            None,
            snapshot.frame_ctr_dbg,
        )
    })
}

pub(crate) fn capture_rendered_rust_ppu_probe(
    game: &ZeldaState,
    rendered: &crate::gpu_capture::LiveGpuFrameCapture,
) -> (
    DisplayPpuProbe,
    Vec<DisplayPublicationCandidateProbe>,
    Option<DisplayPublicationContextProbe>,
) {
    let live_oam = game
        .ppu
        .oam
        .iter()
        .flat_map(|word| word.to_le_bytes().map(i32::from))
        .collect();
    capture_rust_ppu_probe_from_presented(
        game,
        rendered.presented_ppu(),
        rendered.presented_scanlines(),
        live_oam,
        "native_window_render_capture",
        rendered.comparison_frame(),
        rendered.host_frame(),
    )
}

pub(crate) fn capture_rust_ppu_probe_from_presented(
    game: &ZeldaState,
    ppu: &snes::ppu::PpuState,
    scanlines: &renderer::gpu_frame::RawScanlineFrame,
    live_oam: Vec<i32>,
    capture_source: &'static str,
    comparison_frame: Option<u32>,
    host_frame: u32,
) -> (
    DisplayPpuProbe,
    Vec<DisplayPublicationCandidateProbe>,
    Option<DisplayPublicationContextProbe>,
) {
    let probe = DisplayPpuProbe {
        capture_source,
        comparison_frame,
        host_frame: Some(host_frame),
        mode: i32::from(ppu.bg_mode()),
        brightness: i32::from(ppu.brightness),
        scanout_brightness_override: ppu.scanout_brightness_override,
        forced_blank: ppu.forced_blank,
        brightness_white: i32::from(ppu.brightness_mult.get(31).copied().unwrap_or(0) >> 3),
        cgram: ppu.cgram.iter().map(|&value| i32::from(value)).collect(),
        fixed_color: [
            i32::from(ppu.fixed_color_r),
            i32::from(ppu.fixed_color_g),
            i32::from(ppu.fixed_color_b),
        ],
        display_control: [
            i32::from(ppu.screen_enabled[0]),
            i32::from(ppu.screen_enabled[1]),
            i32::from(ppu.screen_windowed[0]),
            i32::from(ppu.screen_windowed[1]),
            i32::from(
                u8::from(ppu.add_subscreen) << 1 | ppu.prevent_math_mode << 4 | ppu.clip_mode << 6,
            ),
            i32::from(
                ppu.math_enabled
                    | u8::from(ppu.half_color) << 6
                    | u8::from(ppu.subtract_color) << 7,
            ),
        ],
        bg_scroll: std::array::from_fn(|i| {
            let layer = &ppu.bg_layer[i / 2];
            i32::from(if i % 2 == 0 {
                layer.h_scroll
            } else {
                layer.v_scroll
            })
        }),
        oam: live_oam,
        presented_oam: ppu
            .oam
            .iter()
            .flat_map(|word| word.to_le_bytes().map(i32::from))
            .collect(),
        mode7: ppu.m7_matrix.map(i32::from),
        mode7_scanlines: scanlines.iter().map(|line| line.7.map(i32::from)).collect(),
        window_scanlines: scanlines
            .iter()
            .map(|line| [line.0, line.1, line.2, line.3].map(i32::from))
            .collect(),
        presented_clip: None,
        presented_pixel: None,
        presented_obj: None,
        presented_obj_tile_cache: Some(
            (0..64u16)
                .flat_map(|tile| {
                    let presented_obj_vram = ppu.obj_vram_latch.as_deref().unwrap_or(&ppu.vram);
                    renderer::modern_extract::decode_snes_4bpp_tile_indices(
                        presented_obj_vram,
                        0x4000,
                        tile,
                    )
                    .map(i32::from)
                })
                .collect(),
        ),
        presented_obj_tile_cache_valid: None,
    };
    let candidates = game
        .zelda_debug_display_publication_candidates()
        .iter()
        .map(|candidate| DisplayPublicationCandidateProbe {
            name: candidate.name,
            cgram: None,
            cgram_difference: None,
            presented_oam: candidate.oam.as_ref().map(|oam| {
                oam.iter()
                    .flat_map(|word| word.to_le_bytes().map(i32::from))
                    .collect()
            }),
            presented_oam_difference: None,
            presented_obj_tile_cache: candidate.obj_vram.as_ref().map(|obj_vram| {
                (0..64u16)
                    .flat_map(|tile| {
                        renderer::modern_extract::decode_snes_4bpp_tile_indices(obj_vram, 0, tile)
                            .map(i32::from)
                    })
                    .collect()
            }),
            presented_obj_tile_cache_valid_difference: None,
        })
        .collect::<Vec<_>>();
    let mut candidates = candidates;
    for cgram_candidate in game.zelda_debug_display_cgram_candidates() {
        let cgram = Some(
            cgram_candidate
                .cgram
                .iter()
                .copied()
                .map(i32::from)
                .collect(),
        );
        if let Some(candidate) = candidates
            .iter_mut()
            .find(|candidate| candidate.name == cgram_candidate.name)
        {
            candidate.cgram = cgram;
        } else {
            candidates.push(DisplayPublicationCandidateProbe {
                name: cgram_candidate.name,
                cgram,
                cgram_difference: None,
                presented_oam: None,
                presented_oam_difference: None,
                presented_obj_tile_cache: None,
                presented_obj_tile_cache_valid_difference: None,
            });
        }
    }
    let context = game
        .zelda_debug_display_publication_context()
        .map(|context| DisplayPublicationContextProbe {
            render_host_frame: context.render_host_frame,
            publication_host_frame: context.publication_host_frame,
            entry_frame: context.entry_frame,
            following_frame: context.following_frame,
            dungeon_room: context.dungeon_room,
            staircase_index: context.staircase_index,
            palette_filter_countdown: context.palette_filter_countdown,
            nmi_update_latch: context.nmi_update_latch,
            oam_scanout_source: context.oam_scanout_source.clone(),
            retain_captured_oam: context.retain_captured_oam,
            link_obj_scanout_generation: context.link_obj_scanout_generation.clone(),
            link_obj_source_generation: context.link_obj_source_generation.clone(),
            captured_to_host_oam_mismatches: context.captured_to_host_oam_mismatches,
            oam_dma_completed_after_active_scanout: context.oam_dma_completed_after_active_scanout,
        });
    (probe, candidates, context)
}

pub(crate) fn write_display_oracle_receipt(
    writer: &mut BufWriter<fs::File>,
    frame: u32,
    stage: &'static str,
    oracle: &LibretroCore,
    game: &mut ZeldaState,
) {
    let Some(oracle_ppu) = capture_oracle_ppu_probe(oracle) else {
        eprintln!("display-oracle capture requires an instrumented Snes9x core");
        process::exit(2);
    };
    let (rust, mut rust_candidates, rust_context) = capture_rust_ppu_probe(game);
    annotate_display_candidate_differences(&oracle_ppu, &mut rust_candidates);
    let receipt = DisplayOracleReceipt {
        frame,
        stage,
        oracle: oracle_ppu,
        rust,
        rust_candidates,
        rust_context,
    };
    serde_json::to_writer(&mut *writer, &receipt).unwrap_or_else(|error| {
        eprintln!("failed to write display-oracle receipt: {error}");
        process::exit(1);
    });
    writeln!(writer).unwrap_or_else(|error| {
        eprintln!("failed to terminate display-oracle receipt: {error}");
        process::exit(1);
    });
    writer.flush().unwrap_or_else(|error| {
        eprintln!("failed to flush display-oracle receipt: {error}");
        process::exit(1);
    });
}

pub(crate) fn annotate_display_candidate_differences(
    oracle: &DisplayPpuProbe,
    candidates: &mut [DisplayPublicationCandidateProbe],
) {
    for candidate in candidates {
        candidate.cgram_difference = candidate
            .cgram
            .as_deref()
            .map(|candidate| summarize_value_domain(candidate, &oracle.cgram));
        candidate.presented_oam_difference = candidate
            .presented_oam
            .as_deref()
            .map(|candidate| summarize_value_domain(candidate, &oracle.presented_oam));
        candidate.presented_obj_tile_cache_valid_difference = summarize_presented_obj_cache(
            candidate.presented_obj_tile_cache.as_deref(),
            oracle.presented_obj_tile_cache.as_deref(),
            oracle.presented_obj_tile_cache_valid.as_deref(),
        );
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct HashedDomainDiff {
    pub(crate) rust_fnv1a32: u32,
    pub(crate) oracle_fnv1a32: u32,
    pub(crate) difference: ValueDomainDiff,
}

#[derive(Debug, Serialize)]
pub(crate) struct ObjStateLedgerReceipt {
    pub(crate) frame: u32,
    /// Semantic WRAM fields modeled by the native engine. Full WRAM remains
    /// below as an observational hash because unmodeled/scratch bytes are not
    /// a valid parity gate.
    pub(crate) modeled_wram_mismatches: Vec<String>,
    pub(crate) wram: HashedDomainDiff,
    pub(crate) raw_obj_vram: HashedDomainDiff,
    pub(crate) live_oam: HashedDomainDiff,
    pub(crate) presented_oam: HashedDomainDiff,
    pub(crate) presented_obj_tile_cache: HashedDomainDiff,
    pub(crate) oracle_valid_obj_tiles: usize,
}

impl ObjStateLedgerReceipt {
    pub(crate) fn presented_cache_is_exact(&self) -> bool {
        self.presented_obj_tile_cache.difference.is_exact()
    }
}

pub(crate) fn fnv1a32(values: impl IntoIterator<Item = u8>) -> u32 {
    values.into_iter().fold(2_166_136_261u32, |hash, value| {
        (hash ^ u32::from(value)).wrapping_mul(16_777_619)
    })
}

pub(crate) fn hashed_byte_domain(rust: &[u8], oracle: &[u8]) -> HashedDomainDiff {
    HashedDomainDiff {
        rust_fnv1a32: fnv1a32(rust.iter().copied()),
        oracle_fnv1a32: fnv1a32(oracle.iter().copied()),
        difference: summarize_value_domain(rust, oracle),
    }
}

pub(crate) fn hashed_i32_domain(rust: &[i32], oracle: &[i32]) -> HashedDomainDiff {
    HashedDomainDiff {
        rust_fnv1a32: fnv1a32(rust.iter().map(|&value| value as u8)),
        oracle_fnv1a32: fnv1a32(oracle.iter().map(|&value| value as u8)),
        difference: summarize_value_domain(rust, oracle),
    }
}

pub(crate) fn capture_obj_state_ledger_receipt(
    frame: u32,
    oracle: &LibretroCore,
    game: &mut ZeldaState,
) -> Option<ObjStateLedgerReceipt> {
    oracle.debug_ppu_value(29, 0)?;
    let oracle_wram = oracle.memory_bytes(RETRO_MEMORY_SYSTEM_RAM)?;
    let oracle_vram = oracle.memory_bytes(RETRO_MEMORY_VIDEO_RAM)?;
    let oracle_obj_vram = oracle_vram.get(0x8000..0x8800)?;
    let rust_obj_vram = game.ppu.vram[0x4000..0x4400]
        .iter()
        .flat_map(|word| word.to_le_bytes())
        .collect::<Vec<_>>();
    let rust_live_oam = game
        .ppu
        .oam
        .iter()
        .flat_map(|word| word.to_le_bytes().map(i32::from))
        .collect::<Vec<_>>();
    let oracle_live_oam = (0..544)
        .map(|index| oracle.debug_ppu_value(15, index).unwrap_or(-1))
        .collect::<Vec<_>>();
    let (rust_presented_oam, rust_presented_cache) = game.with_display_snapshot(|snapshot| {
        let presented_obj_vram = snapshot
            .ppu
            .obj_vram_latch
            .as_deref()
            .unwrap_or(&snapshot.ppu.vram);
        let cache = (0..64u16)
            .flat_map(|tile| {
                renderer::modern_extract::decode_snes_4bpp_tile_indices(
                    presented_obj_vram,
                    0x4000,
                    tile,
                )
                .map(i32::from)
            })
            .collect::<Vec<_>>();
        let oam = snapshot
            .ppu
            .oam
            .iter()
            .flat_map(|word| word.to_le_bytes().map(i32::from))
            .collect::<Vec<_>>();
        (oam, cache)
    });
    let oracle_presented_oam = (0..544)
        .map(|index| oracle.debug_ppu_value(20, index).unwrap_or(-1))
        .collect::<Vec<_>>();
    let oracle_valid = (0..64)
        .map(|index| oracle.debug_ppu_value(30, index).unwrap_or(-1))
        .collect::<Vec<_>>();
    let oracle_presented_cache = (0..64 * 64)
        .map(|index| oracle.debug_ppu_value(29, index).unwrap_or(-1))
        .collect::<Vec<_>>();
    let mut rust_valid_cache = Vec::new();
    let mut oracle_valid_cache = Vec::new();
    for (tile, &valid) in oracle_valid.iter().enumerate() {
        if valid == 0 {
            continue;
        }
        let range = tile * 64..tile * 64 + 64;
        rust_valid_cache.extend_from_slice(&rust_presented_cache[range.clone()]);
        oracle_valid_cache.extend_from_slice(&oracle_presented_cache[range]);
    }
    let presented_cache_difference = summarize_presented_obj_cache(
        Some(&rust_presented_cache),
        Some(&oracle_presented_cache),
        Some(&oracle_valid),
    )?;
    Some(ObjStateLedgerReceipt {
        frame,
        modeled_wram_mismatches: compact_engine_state_mismatches(&game.ram, oracle_wram),
        wram: hashed_byte_domain(&game.ram, oracle_wram),
        raw_obj_vram: hashed_byte_domain(&rust_obj_vram, oracle_obj_vram),
        live_oam: hashed_i32_domain(&rust_live_oam, &oracle_live_oam),
        presented_oam: hashed_i32_domain(&rust_presented_oam, &oracle_presented_oam),
        presented_obj_tile_cache: HashedDomainDiff {
            rust_fnv1a32: fnv1a32(rust_valid_cache.iter().map(|&value| value as u8)),
            oracle_fnv1a32: fnv1a32(oracle_valid_cache.iter().map(|&value| value as u8)),
            difference: presented_cache_difference,
        },
        oracle_valid_obj_tiles: oracle_valid.iter().filter(|&&valid| valid != 0).count(),
    })
}

pub(crate) fn snes9x_presented_scanline_for_video_y(video_height: usize, video_y: usize) -> usize {
    // The trace core exposes Snes9x's uncropped scanline caches, while the libretro video
    // callback can crop the top overscan rows. Keep the translation next to the consumer so
    // pixel-owner diagnostics cannot silently inspect a different scanline than the RGBA pixel.
    let top_crop = match video_height {
        224 => 7,
        448 => 14,
        _ => 0,
    };
    video_y + top_crop
}

pub(crate) fn snes9x_presented_animated_bg_tiles(
    oracle: &LibretroCore,
) -> Result<Option<PresentedAnimatedBgTiles>, String> {
    // Zelda's leading NMI publishes this complete 0x400-byte domain before the
    // active field, and Snes9x returns at the following VBlank before another
    // NMI can replace it. Decode the exact post-host VRAM generation instead
    // of consulting TileCached: a tile consumed only through H-flip has no
    // entry in the unflipped cache even though it was visibly presented.
    let Some(destination) =
        decode_presented_animated_bg_destination(oracle.debug_ppu_value(40, 0))?
    else {
        return Ok(None);
    };
    let vram = oracle
        .memory_bytes(RETRO_MEMORY_VIDEO_RAM)
        .ok_or_else(|| "Snes9x video RAM is unavailable".to_string())?;
    decode_presented_animated_bg_tiles(destination, vram).map(Some)
}

pub(crate) fn decode_presented_animated_bg_destination(
    value: Option<i32>,
) -> Result<Option<zelda3::PresentedAnimatedBgDestination>, String> {
    match value {
        None | Some(-1) => return Ok(None),
        Some(0x3b00) => Ok(Some(zelda3::PresentedAnimatedBgDestination::Dungeon)),
        Some(0x3c00) => Ok(Some(zelda3::PresentedAnimatedBgDestination::Overworld)),
        // Zelda leaves this operand at Snes9x's 0x55 reset fill until graphics
        // setup. Cold initialization then clears WRAM to zero before either
        // DecompressAnimated* routine publishes the first real destination.
        // Both values therefore mean that no animated-BG generation exists.
        Some(0 | 0x5555) => Ok(None),
        Some(value) => Err(format!(
            "presented animated-BG destination is invalid: {value}"
        )),
    }
}

pub(crate) fn decode_presented_animated_bg_tiles(
    destination: zelda3::PresentedAnimatedBgDestination,
    vram: &[u8],
) -> Result<PresentedAnimatedBgTiles, String> {
    let word_address = match destination {
        zelda3::PresentedAnimatedBgDestination::Dungeon => 0x3b00,
        zelda3::PresentedAnimatedBgDestination::Overworld => 0x3c00,
    };
    let byte_address = word_address * 2;
    let byte_count = PresentedAnimatedBgTiles::TILE_COUNT * 32;
    let bytes = vram
        .get(byte_address..byte_address + byte_count)
        .ok_or_else(|| "Snes9x video RAM omits the animated-BG publication".to_string())?;
    let mut pixels = vec![0; PresentedAnimatedBgTiles::TILE_COUNT * 64];
    for tile in 0..PresentedAnimatedBgTiles::TILE_COUNT {
        let packed = &bytes[tile * 32..tile * 32 + 32];
        for y in 0..8 {
            let planes = [
                packed[y * 2],
                packed[y * 2 + 1],
                packed[16 + y * 2],
                packed[16 + y * 2 + 1],
            ];
            for x in 0..8 {
                let mask = 1 << (7 - x);
                pixels[tile * 64 + y * 8 + x] =
                    planes.iter().enumerate().fold(0, |pixel, (plane, value)| {
                        pixel | u8::from(value & mask != 0) << plane
                    });
            }
        }
    }
    PresentedAnimatedBgTiles::new(destination, pixels)
        .ok_or_else(|| "presented animated-BG receipt has an invalid shape".to_string())
}

pub(crate) fn snes9x_presented_hud_tilemap(
    oracle: &LibretroCore,
) -> Result<Option<PresentedHudTilemap>, String> {
    let first = match oracle.debug_ppu_value(37, 0) {
        None | Some(-1) => return Ok(None),
        Some(value) => value,
    };
    let words = (0..PresentedHudTilemap::WORD_COUNT)
        .map(|index| {
            let value = if index == 0 {
                first
            } else {
                oracle
                    .debug_ppu_value(37, index as i32)
                    .ok_or_else(|| format!("presented HUD tilemap word {index} is unavailable"))?
            };
            u16::try_from(value)
                .map_err(|_| format!("presented HUD tilemap word {index} is invalid: {value}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    PresentedHudTilemap::new(words)
        .map(Some)
        .ok_or_else(|| "presented HUD tilemap receipt has an invalid shape".to_string())
}

pub(crate) fn snes9x_presented_scanout_geometry(
    oracle: &LibretroCore,
) -> Result<Option<zelda3::PresentedScanoutGeometry>, String> {
    let top_crop = match oracle.debug_ppu_value(39, 0) {
        None | Some(-1) => return Ok(None),
        Some(value) => u8::try_from(value)
            .ok()
            .filter(|&rows| rows <= zelda3::PresentedScanoutGeometry::MAX_TOP_CROP)
            .ok_or_else(|| format!("presented scanout top crop is invalid: {value}"))?,
    };
    zelda3::PresentedScanoutGeometry::new(top_crop)
        .map(Some)
        .ok_or_else(|| format!("presented scanout top crop is invalid: {top_crop}"))
}

#[derive(Clone, Debug)]
pub(crate) struct PresentedOracleVideo {
    pub(crate) bytes: Vec<u8>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) pitch: usize,
    pub(crate) pixel_format: u32,
}

impl From<&LibretroFrame> for PresentedOracleVideo {
    fn from(frame: &LibretroFrame) -> Self {
        Self {
            bytes: frame.video.clone(),
            width: frame.video_width,
            height: frame.video_height,
            pitch: frame.video_pitch,
            pixel_format: frame.pixel_format,
        }
    }
}

pub(crate) fn presented_video_rows_match_prior_surface(
    current: &LibretroFrame,
    previous: Option<&PresentedOracleVideo>,
    start_row: usize,
    end_row: usize,
) -> Result<bool, String> {
    if start_row >= end_row {
        return Ok(false);
    }
    let Some(previous) = previous else {
        return Ok(false);
    };
    if current.video_width != previous.width
        || current.video_height != previous.height
        || current.video_pitch != previous.pitch
        || current.pixel_format != previous.pixel_format
    {
        return Err("consecutive Snes9x host surfaces changed layout".to_string());
    }
    let stride = snes9x_pixel_stride(current.pixel_format)
        .ok_or_else(|| format!("unsupported Snes9x pixel format {}", current.pixel_format))?;
    let row_bytes = usize::try_from(current.video_width)
        .ok()
        .and_then(|width| width.checked_mul(stride))
        .ok_or_else(|| "Snes9x host row byte count overflowed".to_string())?;
    if row_bytes > current.video_pitch || end_row > current.video_height as usize {
        return Err("Snes9x host surface cannot contain the presented row interval".to_string());
    }
    for row in start_row..end_row {
        let start = row
            .checked_mul(current.video_pitch)
            .ok_or_else(|| "Snes9x host row offset overflowed".to_string())?;
        let end = start
            .checked_add(row_bytes)
            .ok_or_else(|| "Snes9x host row end overflowed".to_string())?;
        let current_row = current
            .video
            .get(start..end)
            .ok_or_else(|| format!("current Snes9x host row {row} is truncated"))?;
        let previous_row = previous
            .bytes
            .get(start..end)
            .ok_or_else(|| format!("previous Snes9x host row {row} is truncated"))?;
        if current_row != previous_row {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(crate) fn snes9x_presented_inidisp(
    oracle: &LibretroCore,
    geometry: Option<zelda3::PresentedScanoutGeometry>,
    current_video: &LibretroFrame,
    previous_video: Option<&PresentedOracleVideo>,
) -> Result<Option<zelda3::PresentedInidisp>, String> {
    let top_crop = usize::from(geometry.map_or(0, |geometry| geometry.top_crop()));
    let first = match oracle.debug_ppu_value(38, top_crop as i32) {
        None | Some(-1) => return Ok(None),
        Some(value) => value,
    };
    if !(0..=0xff).contains(&first) {
        return Err(format!("presented INIDISP line 0 is invalid: {first}"));
    }
    let mut lines = Vec::with_capacity(zelda3::PresentedInidisp::VISIBLE_LINES);
    lines.push(((first & 0x0f) as u8, first & 0x80 != 0));
    for line in 1..zelda3::PresentedInidisp::VISIBLE_LINES {
        let source_line = line + top_crop;
        let value = oracle
            .debug_ppu_value(38, source_line as i32)
            .ok_or_else(|| format!("presented INIDISP source line {source_line} is unavailable"))?;
        if !(0..=0xff).contains(&value) {
            return Err(format!("presented INIDISP line {line} is invalid: {value}"));
        }
        lines.push(((value & 0x0f) as u8, value & 0x80 != 0));
    }

    let prefix = lines.iter().take_while(|line| line.1).count();
    let suffix = lines
        .iter()
        .enumerate()
        .skip(prefix)
        .find_map(|(line, value)| value.1.then_some(line));
    let visible_end = suffix.unwrap_or(lines.len());
    if lines[prefix..visible_end].iter().any(|line| line.1)
        || lines[visible_end..].iter().any(|line| !line.1)
    {
        return Err("presented INIDISP has a non-contiguous forced-blank raster".to_string());
    }
    let brightness = lines
        .get(prefix)
        .filter(|_| prefix < visible_end)
        .map(|line| line.0)
        .unwrap_or(lines[0].0);
    if lines[prefix..visible_end]
        .iter()
        .any(|line| line.0 != brightness)
    {
        return Err("presented INIDISP has per-line brightness changes".to_string());
    }
    // Pinned Snes9x's `S9xUpdateScreen` skips drawing entirely while
    // `PPU.ForcedBlanking` is set. A fully blank completed scanout therefore
    // returns the preceding libretro surface rather than a newly cleared
    // black surface. Partial suffix blanking retains only the already-scanned
    // visible interval for the same source-level reason.
    let retain_prior_surface = if prefix == zelda3::PresentedInidisp::VISIBLE_LINES {
        presented_video_rows_match_prior_surface(
            current_video,
            previous_video,
            0,
            zelda3::PresentedInidisp::VISIBLE_LINES,
        )?
    } else if suffix.is_some() {
        presented_video_rows_match_prior_surface(
            current_video,
            previous_video,
            prefix,
            visible_end,
        )?
    } else {
        false
    };
    let prefix = u8::try_from(prefix)
        .map_err(|_| "presented INIDISP blank prefix is invalid".to_string())?;
    let suffix = suffix
        .map(u8::try_from)
        .transpose()
        .map_err(|_| "presented INIDISP blank suffix is invalid".to_string())?;
    let receipt = zelda3::PresentedInidisp::new(brightness, prefix, suffix)
        .ok_or_else(|| "presented INIDISP receipt has an invalid shape".to_string())?;
    let receipt = receipt.with_retained_prior_surface(retain_prior_surface);
    Ok(Some(receipt))
}

/// Decode the address-bearing OBJ cache exposed by the pinned oracle.
///
/// Slots 0..256 name the first OBSEL page and slots 256..512 name the second.
/// The address probe remains authoritative rather than being reconstructed by
/// this adapter, while the page metadata proves that every slot belongs to the
/// expected hardware page. `TileCached` uses 0 for invalid, 1 for decoded, and
/// `BLANK_TILE` (2) for a decoded all-zero tile.
pub(crate) fn decode_snes9x_presented_obj_tiles(
    mut read: impl FnMut(i32, i32) -> Option<i32>,
) -> Result<Option<PresentedObjTiles>, String> {
    let Some(abi) = read(PRESENTED_OBJ_CACHE_META_FIELD, 0) else {
        return Ok(None);
    };
    if abi != PRESENTED_OBJ_CACHE_ABI {
        return Err(format!(
            "unsupported presented OBJ cache ABI {abi}; expected {PRESENTED_OBJ_CACHE_ABI}"
        ));
    }

    let metadata = |read: &mut dyn FnMut(i32, i32) -> Option<i32>, index, name| {
        read(PRESENTED_OBJ_CACHE_META_FIELD, index)
            .ok_or_else(|| format!("presented OBJ cache metadata {name} is unavailable"))
    };
    let slot_count = metadata(&mut read, 1, "slot count")?;
    if slot_count != PRESENTED_OBJ_CACHE_SLOT_COUNT as i32 {
        return Err(format!(
            "presented OBJ cache slot count is invalid: {slot_count}"
        ));
    }
    let pixels_per_tile = metadata(&mut read, 2, "pixels per tile")?;
    if pixels_per_tile != PresentedObjTiles::PIXELS_PER_TILE as i32 {
        return Err(format!(
            "presented OBJ cache pixels per tile is invalid: {pixels_per_tile}"
        ));
    }
    let page_bases = [
        metadata(&mut read, 3, "page 0 word base")?,
        metadata(&mut read, 4, "page 1 word base")?,
    ]
    .map(|value| {
        u16::try_from(value)
            .ok()
            .filter(|&address| {
                usize::from(address) < 0x8000
                    && usize::from(address) % PresentedObjTiles::WORDS_PER_TILE == 0
            })
            .ok_or_else(|| format!("presented OBJ cache page base is invalid: {value}"))
    });
    let [page_0_base, page_1_base] = [page_bases[0].clone()?, page_bases[1].clone()?];

    let mut tile_word_addresses = Vec::new();
    let mut tile_pixels = Vec::new();
    let mut seen_addresses = [false; PresentedObjTiles::MAX_TILE_COUNT];
    for slot in 0..PRESENTED_OBJ_CACHE_SLOT_COUNT {
        let slot_index = i32::try_from(slot).expect("OBJ cache slot count fits i32");
        let validity = read(PRESENTED_OBJ_CACHE_VALID_FIELD, slot_index)
            .ok_or_else(|| format!("presented OBJ cache validity {slot} is unavailable"))?;
        if !(0..=2).contains(&validity) {
            return Err(format!(
                "presented OBJ cache validity {slot} is invalid: {validity}"
            ));
        }

        let value = read(PRESENTED_OBJ_CACHE_WORD_ADDRESS_FIELD, slot_index)
            .ok_or_else(|| format!("presented OBJ cache word address {slot} is unavailable"))?;
        if validity == 0 {
            if value != -1 {
                return Err(format!(
                    "invalid presented OBJ cache slot {slot} has word address {value}"
                ));
            }
            continue;
        }
        let address = u16::try_from(value)
            .ok()
            .filter(|&address| {
                usize::from(address) < 0x8000
                    && usize::from(address) % PresentedObjTiles::WORDS_PER_TILE == 0
            })
            .ok_or_else(|| {
                format!("presented OBJ cache word address {slot} is invalid: {value}")
            })?;
        let (page_base, page_slot) = if slot < PRESENTED_OBJ_CACHE_PAGE_TILE_COUNT {
            (page_0_base, slot)
        } else {
            (page_1_base, slot - PRESENTED_OBJ_CACHE_PAGE_TILE_COUNT)
        };
        let expected_address =
            (usize::from(page_base) + page_slot * PresentedObjTiles::WORDS_PER_TILE) & 0x7fff;
        if usize::from(address) != expected_address {
            return Err(format!(
                "presented OBJ cache word address {slot} is {address:#06x}, expected {expected_address:#06x}"
            ));
        }
        let tile_index = usize::from(address) / PresentedObjTiles::WORDS_PER_TILE;
        if std::mem::replace(&mut seen_addresses[tile_index], true) {
            return Err(format!(
                "presented OBJ cache repeats physical word address {address:#06x}"
            ));
        }
        tile_word_addresses.push(address);
        let pixel_base = slot
            .checked_mul(PresentedObjTiles::PIXELS_PER_TILE)
            .expect("fixed OBJ cache dimensions cannot overflow");
        for pixel in 0..PresentedObjTiles::PIXELS_PER_TILE {
            let index = pixel_base + pixel;
            let value = read(
                PRESENTED_OBJ_CACHE_PIXELS_FIELD,
                i32::try_from(index).expect("OBJ cache pixel index fits i32"),
            )
            .ok_or_else(|| format!("presented OBJ cache pixel {index} is unavailable"))?;
            let pixel = u8::try_from(value)
                .ok()
                .filter(|&pixel| pixel < 16)
                .ok_or_else(|| format!("presented OBJ cache pixel {index} is invalid: {value}"))?;
            tile_pixels.push(pixel);
        }
    }

    PresentedObjTiles::new(tile_word_addresses, tile_pixels)
        .map(Some)
        .ok_or_else(|| "presented OBJ tile receipt has an invalid shape".to_string())
}

pub(crate) fn snes9x_presented_obj_tiles(
    oracle: &LibretroCore,
) -> Result<Option<PresentedObjTiles>, String> {
    decode_snes9x_presented_obj_tiles(|field, index| oracle.debug_ppu_value(field, index))
}

pub(crate) fn snes9x_presented_oam_bytes(oracle: &LibretroCore) -> Result<Option<Vec<u8>>, String> {
    if oracle.debug_ppu_value(20, 0).is_none() {
        return Ok(None);
    }
    let bytes = (0..PresentedOam::BYTE_COUNT)
        .map(|index| {
            let value = oracle
                .debug_ppu_value(20, index as i32)
                .ok_or_else(|| format!("presented OAM byte {index} is unavailable"))?;
            u8::try_from(value)
                .map_err(|_| format!("presented OAM byte {index} is invalid: {value}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some(bytes))
}

pub(crate) fn snes9x_presented_oam(oracle: &LibretroCore) -> Result<Option<PresentedOam>, String> {
    let Some(bytes) = snes9x_presented_oam_bytes(oracle)? else {
        return Ok(None);
    };
    PresentedOam::new(bytes)
        .map(Some)
        .ok_or_else(|| "presented OAM receipt has an invalid shape".to_string())
}

pub(crate) fn snes9x_presented_cgram(
    oracle: &LibretroCore,
) -> Result<Option<PresentedCgram>, String> {
    if oracle.debug_ppu_value(36, 0).is_none() {
        return Ok(None);
    }
    let colors = (0..PresentedCgram::COLOR_COUNT)
        .map(|index| {
            let value = oracle
                .debug_ppu_value(36, index as i32)
                .ok_or_else(|| format!("presented CGRAM color {index} is unavailable"))?;
            u16::try_from(value)
                .ok()
                .filter(|&color| color <= 0x7fff)
                .ok_or_else(|| format!("presented CGRAM color {index} is invalid: {value}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    PresentedCgram::new(colors)
        .map(Some)
        .ok_or_else(|| "presented CGRAM receipt has an invalid shape".to_string())
}
