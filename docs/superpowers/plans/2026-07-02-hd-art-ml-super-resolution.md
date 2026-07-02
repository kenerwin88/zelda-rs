# HD Art via ML Super-Resolution Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Author real HD art by super-resolving captured frames and slicing it into the existing per-source-key override cells, so genuine HD is visible in-game (Kakariko, 4×) with zero change to the render/parity path.

**Architecture:** Four offline stages producing gitignored assets under `hd_art/`: (1) a Rust `--dump-hd-capture` subcommand emits native composited frames + per-frame placement maps + the reference palette; (2) a Python torch/MPS Real-ESRGAN script upscales the frames; (3) a Rust `--slice-hd-cells` subcommand crops each source key's region from the SR'd frames and (4) writes the manifest. The renderer already loads the manifest (Phase 1/2); no render code changes.

**Tech Stack:** Rust (`crates/renderer`, `zelda3-bin`), serde/serde_json (already deps), the `png` crate (already used), Python 3 + torch 2.12 (MPS) + PIL/numpy for SR.

## Global Constraints

- **Parity is sacred:** no renderer/game code changes. The override system is already gated by `ZELDA3_MODERN_HD_OVERRIDES` (unset → byte-identical). New Rust code is authoring-only subcommands; do not touch `modern_software.rs`/`modern_extract.rs`/`lib.rs` render logic.
- **One canonical palette per manifest:** SR cells are colorized with the capture CGRAM, and the manifest's `reference_palette` IS that CGRAM. Capture all frames of one run under one palette/scene (first cut: Kakariko).
- **Cells are 8×8** native; HD cell dims are multiples of 8 (`8*scale`).
- **Source key format:** hex string `0x{:016x}`, matching the atlas dump and `ModernHdOverrides` manifest (`key` field). `NO_SOURCE_KEY == 0` cells are never emitted.
- **Manifest schema (do not change):** `{ "reference_palette": "<png>", "overrides": [ { "key": "0x..", "rgba": "<png>" } ] }`, paths relative to the manifest.
- **All `hd_art/` outputs + model weights are gitignored.** Commit code, tests, the SR script, gitignore, and the README — never the generated assets or weights.
- Commit with targeted `git add` of only the files a task changed; use `--no-verify` (the heavy parity pre-commit hook races the user's concurrent commits). Never `git add -A`/`.`, never `git checkout` a file, never stage unrelated user WIP (`crates/zelda3/*`, other `zelda3-bin/*`).

---

### Task 1: Placement-map core (`build_hd_placement_map`)

**Files:**
- Create: `crates/renderer/src/hd_authoring.rs`
- Modify: `crates/renderer/src/lib.rs` (add `pub mod hd_authoring;`)
- Test: in `hd_authoring.rs` `mod tests`

**Interfaces:**
- Consumes: `ModernFrame` (`modern_frame.rs`), `ModernIndexTile` (`modern_index_atlas.rs`, fields `id, indices:[u8;64], source_key:u64, hflip, vflip`), `NO_SOURCE_KEY` (`modern_hd_overrides.rs`).
- Produces: `pub struct HdPlacement { key: String, x: i16, y: i16, w: u16, h: u16 }` (derives `Serialize, Deserialize`) and `pub fn build_hd_placement_map(frame: &ModernFrame, bg_cells: &[ModernIndexTile], sprite_cells: &[ModernIndexTile]) -> Vec<HdPlacement>`.

- [ ] **Step 1: Add the module declaration.** In `crates/renderer/src/lib.rs`, add `pub mod hd_authoring;` next to the other `pub mod modern_*;` lines.

- [ ] **Step 2: Write the failing test** in `crates/renderer/src/hd_authoring.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_frame::ModernFrame;
    use crate::modern_index_atlas::ModernIndexTile;
    use crate::modern_hd_overrides::NO_SOURCE_KEY;

    fn cell(id: u32, source_key: u64) -> ModernIndexTile {
        ModernIndexTile { id, indices: [0u8; 64], source_key, hflip: false, vflip: false }
    }

    #[test]
    fn placement_map_records_keyed_bg_and_sprite_positions_and_skips_unkeyed() {
        use crate::modern_frame::{ModernIndexTileInstance, ModernIndexSpriteInstance};
        let mut frame = ModernFrame::empty();
        // BG tile: keyed cell 0 at (16,24); unkeyed cell 1 at (0,0) -> skipped.
        frame.bg_layers[0].index_tiles.push(ModernIndexTileInstance {
            cell_id: 0, screen_x: 16, screen_y: 24, palette: 0, hflip: false, vflip: false, priority: false,
        });
        frame.bg_layers[0].index_tiles.push(ModernIndexTileInstance {
            cell_id: 1, screen_x: 0, screen_y: 0, palette: 0, hflip: false, vflip: false, priority: false,
        });
        let bg_cells = vec![cell(0, 0xABCD), cell(1, NO_SOURCE_KEY)];
        // Sprite: keyed cell 0 at (32,40).
        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 0, screen_x: 32, screen_y: 40, palette: 0, priority: 0, hflip: false, vflip: false, row_mask: 0xff,
        });
        let sprite_cells = vec![cell(0, 0x1234)];

        let map = build_hd_placement_map(&frame, &bg_cells, &sprite_cells);
        assert_eq!(map, vec![
            HdPlacement { key: "0x000000000000abcd".into(), x: 16, y: 24, w: 8, h: 8 },
            HdPlacement { key: "0x0000000000001234".into(), x: 32, y: 40, w: 8, h: 8 },
        ]);
    }
}
```

- [ ] **Step 3: Run it, expect FAIL** (types/fn undefined):
`cargo test --profile parity -p renderer placement_map_records 2>&1 | tail -5` → FAIL.

- [ ] **Step 4: Implement** at the top of `crates/renderer/src/hd_authoring.rs`:

```rust
//! Offline HD-art authoring helpers (not on the render/parity path). Builds the
//! per-frame placement map (source key -> screen rect) used to slice HD cells out
//! of super-resolved frames.
use serde::{Deserialize, Serialize};

use crate::modern_frame::ModernFrame;
use crate::modern_hd_overrides::NO_SOURCE_KEY;
use crate::modern_index_atlas::ModernIndexTile;

/// One drawn 8×8 cell occurrence: its source key and native-pixel screen rect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HdPlacement {
    /// Source key as hex string `0x{:016x}` (matches the atlas dump + manifest).
    pub key: String,
    pub x: i16,
    pub y: i16,
    pub w: u16,
    pub h: u16,
}

/// Enumerate every drawn tile/sprite instance that has a real source key, with its
/// native screen position. Cells are 8×8. `NO_SOURCE_KEY` cells are skipped.
pub fn build_hd_placement_map(
    frame: &ModernFrame,
    bg_cells: &[ModernIndexTile],
    sprite_cells: &[ModernIndexTile],
) -> Vec<HdPlacement> {
    let mut out = Vec::new();
    for layer in &frame.bg_layers {
        for inst in &layer.index_tiles {
            if let Some(c) = bg_cells.get(inst.cell_id as usize) {
                if c.source_key != NO_SOURCE_KEY {
                    out.push(HdPlacement {
                        key: format!("0x{:016x}", c.source_key),
                        x: inst.screen_x, y: inst.screen_y, w: 8, h: 8,
                    });
                }
            }
        }
    }
    for inst in &frame.index_sprites {
        if let Some(c) = sprite_cells.get(inst.cell_id as usize) {
            if c.source_key != NO_SOURCE_KEY {
                out.push(HdPlacement {
                    key: format!("0x{:016x}", c.source_key),
                    x: inst.screen_x, y: inst.screen_y, w: 8, h: 8,
                });
            }
        }
    }
    out
}
```

- [ ] **Step 5: Run it, expect PASS.** `cargo test --profile parity -p renderer placement_map_records 2>&1 | tail -5` → PASS.

- [ ] **Step 6: Commit.**
```bash
git add crates/renderer/src/hd_authoring.rs crates/renderer/src/lib.rs
git commit --no-verify -m "feat(renderer): HD-authoring placement map (source key -> screen rect)"
```

---

### Task 2: Slice core (`slice_hd_cell`)

**Files:**
- Modify: `crates/renderer/src/hd_authoring.rs`
- Test: same file `mod tests`

**Interfaces:**
- Produces: `pub fn slice_hd_cell(sr: &[u8], sr_w: u32, sr_h: u32, x: i16, y: i16, w: u16, h: u16, scale: u32) -> Option<Vec<u8>>` — returns the `(w*scale)·(h*scale)·4` RGBA crop at native `(x,y)` upscaled by `scale`, or `None` if the cell is not fully on-screen (negative or overhanging placements are skipped).

- [ ] **Step 1: Write the failing test** (add to `mod tests`):

```rust
    #[test]
    fn slice_extracts_scaled_region_and_skips_offscreen() {
        // 4x2 native SR frame at scale 2 -> sr is 8x4 RGBA. Fill each pixel R=x, G=y.
        let (sr_w, sr_h, scale) = (8u32, 4u32, 2u32);
        let mut sr = vec![0u8; (sr_w * sr_h * 4) as usize];
        for py in 0..sr_h { for px in 0..sr_w {
            let i = ((py * sr_w + px) * 4) as usize;
            sr[i] = px as u8; sr[i + 1] = py as u8; sr[i + 3] = 0xff;
        }}
        // Native cell 1x1 footprint at native (1,0), scale 2 -> crop sr region x=2..4, y=0..2.
        let got = slice_hd_cell(&sr, sr_w, sr_h, 1, 0, 1, 1, scale).expect("on-screen");
        assert_eq!(got.len(), (2 * 2 * 4) as usize);
        assert_eq!(&got[0..4], &[2, 0, 0, 0xff]);   // sr (2,0)
        assert_eq!(&got[4..8], &[3, 0, 0, 0xff]);   // sr (3,0)
        assert_eq!(&got[8..12], &[2, 1, 0, 0xff]);  // sr (2,1)
        // Negative and overhanging placements skip.
        assert!(slice_hd_cell(&sr, sr_w, sr_h, -1, 0, 1, 1, scale).is_none());
        assert!(slice_hd_cell(&sr, sr_w, sr_h, 4, 0, 1, 1, scale).is_none()); // x*scale=8 >= sr_w
    }
```

- [ ] **Step 2: Run it, expect FAIL.** `cargo test --profile parity -p renderer slice_extracts 2>&1 | tail -5` → FAIL.

- [ ] **Step 3: Implement** (append to `hd_authoring.rs`, above `mod tests`):

```rust
/// Crop one cell's HD pixels from a super-resolved frame. `sr` is row-major RGBA8
/// of `sr_w × sr_h`. The cell footprint is `w×h` native px at native `(x,y)`,
/// upscaled by `scale` (so the crop is `(w*scale)×(h*scale)`). Returns `None` if
/// the upscaled crop is not fully inside the frame (partial/negative → skip).
pub fn slice_hd_cell(
    sr: &[u8], sr_w: u32, sr_h: u32,
    x: i16, y: i16, w: u16, h: u16, scale: u32,
) -> Option<Vec<u8>> {
    if x < 0 || y < 0 {
        return None;
    }
    let ow = w as u32 * scale;
    let oh = h as u32 * scale;
    let ox = x as u32 * scale;
    let oy = y as u32 * scale;
    if ox + ow > sr_w || oy + oh > sr_h {
        return None;
    }
    let row_bytes = (ow * 4) as usize;
    let mut out = vec![0u8; (ow * oh * 4) as usize];
    for row in 0..oh {
        let src = (((oy + row) * sr_w + ox) * 4) as usize;
        let dst = (row * ow * 4) as usize;
        out[dst..dst + row_bytes].copy_from_slice(&sr[src..src + row_bytes]);
    }
    Some(out)
}
```

- [ ] **Step 4: Run it, expect PASS.** `cargo test --profile parity -p renderer slice_extracts 2>&1 | tail -5` → PASS.

- [ ] **Step 5: Commit.**
```bash
git add crates/renderer/src/hd_authoring.rs
git commit --no-verify -m "feat(renderer): slice_hd_cell crops per-key HD region from an SR frame"
```

---

### Task 3: `--dump-hd-capture` subcommand

**Files:**
- Modify: `zelda3-bin/src/main.rs` (arg dispatch near line 238 + a new `run_dump_hd_capture` fn)

**Interfaces:**
- Consumes: `renderer::hd_authoring::build_hd_placement_map`, `renderer::modern_extract::{extract_modern_frame_from_sources, extract_modern_sprites_from_sources}`, `renderer::modern_software::render_modern_frame_full_scaled`, `renderer::modern_hd_overrides::HdOverrideCtx`, `game.vram_chr_source()`, the source atlas loader `renderer::modern_source_atlas::load_modern_source_atlas`, and the existing reference-palette dump logic in `run_dump_reference_palette`.
- Produces (on disk, into `hd_art/capture/`, created if absent): for each captured frame `n`: `frame_<n>.png` (native 256×224 RGBA), `frame_<n>.map.json` (`Vec<HdPlacement>` via `serde_json`); plus one `reference_palette.png` (256×1 RGBA) from the first captured frame.

**Reuse the replay/setup harness from `run_dump_assets_by_source`** (ROM at `<CARGO_MANIFEST_DIR>/../saves/zelda3.sfc`, replay `<...>/../saves/zelda3-combined-route.sav`, the timing-hack envs, and the frame-stepping loop). This task adds the per-target-frame capture; do not reimplement the replay boilerplate — mirror that function's setup, then insert the capture block below when `completed` equals a requested target frame.

- [ ] **Step 1: Add the dispatch line** after the `--dump-assets-by-source` block (~line 240):
```rust
    if args.get(1).map(String::as_str) == Some("--dump-hd-capture") {
        run_dump_hd_capture(&args[2..]);
        return;
    }
```

- [ ] **Step 2: Write `run_dump_hd_capture`.** Parse args as a list of target frame numbers (`args.iter().filter_map(|s| s.parse::<u32>().ok()).collect::<Vec<_>>()`; if empty, print usage `usage: zelda3 --dump-hd-capture <frame> [frame...]` and `process::exit(2)`). Load the source atlas once via `load_modern_source_atlas(Path::new("."))` (exit with its error if it fails — capture needs it). Create `hd_art/capture/` with `std::fs::create_dir_all`. Set up the replay exactly as `run_dump_assets_by_source` does, stepping frames; when `completed` matches a requested target, run the capture block:

**Building the `GpuFrame`:** `run_dump_assets_by_source` walks VRAM directly; you instead need the same `GpuFrame` the live present path builds, so the sources extractor and colors match the renderer. Construct it exactly as `GpuPlayRenderer::present_frame` does (main.rs ~1441): `game.cgram_after_first_hdma_line()`, `game.ppu_scanline_windows()`, `game.ppu.clone()`, then `gpu_frame_from_ppu`. So the capture task = `run_dump_assets_by_source`'s **replay setup + frame-step loop** + this present-path **GpuFrame construction** at each target frame:

```rust
// --- capture block (runs when `completed` == a requested target frame) ---
use renderer::hd_authoring::build_hd_placement_map;
let hdma_cgram = game.cgram_after_first_hdma_line();
let scanlines_raw = game.ppu_scanline_windows();
let ppu = game.ppu.clone();
let gpu_frame = gpu_frame_from_ppu(&ppu, &hdma_cgram, scanlines_from_raw(&scanlines_raw));
if gpu_frame.mode == 7 {
    eprintln!("frame {completed}: Mode 7 not supported by the sources path; skipping");
} else {
    let src_slice: Vec<(u8, u16, u16)> = game
        .vram_chr_source().as_slice().iter()
        .map(|s| (s.kind, s.pack, s.tile_off)).collect();
    let (mut modern, bg_cells) =
        renderer::modern_extract::extract_modern_frame_from_sources(&gpu_frame, &src_slice[..], &atlas);
    let (sprite_cells, sprites) =
        renderer::modern_extract::extract_modern_sprites_from_sources(&gpu_frame, &src_slice[..], &atlas);
    modern.index_sprites = sprites;

    // Native RGBA (scale 1, overrides disabled) — the colorized frame SR ingests.
    let ctx = renderer::modern_hd_overrides::HdOverrideCtx::disabled();
    let rgba = renderer::modern_software::render_modern_frame_full_scaled(
        &modern, &bg_cells, &sprite_cells, &ctx, 1);
    // Reuse the existing indexed/edge PNG writer helpers: write a plain RGBA8 PNG.
    write_rgba_png(&format!("hd_art/capture/frame_{completed}.png"), &rgba, 256, 224);

    // Placement map.
    let map = build_hd_placement_map(&modern, &bg_cells, &sprite_cells);
    std::fs::write(
        format!("hd_art/capture/frame_{completed}.map.json"),
        serde_json::to_vec_pretty(&map).expect("serialize placement map"),
    ).expect("write placement map");

    // Reference palette from the FIRST captured frame's CGRAM (256x1 RGBA).
    if first_capture {
        write_reference_palette_png("hd_art/capture/reference_palette.png", &modern.cgram_rgba);
        first_capture = false;
    }
    eprintln!("captured frame {completed}: {} placements", map.len());
}
```

Provide the two small PNG helpers if they don't already exist in `main.rs`: `write_rgba_png(path, rgba, w, h)` (a straight `png::Encoder` RGBA8 write) and `write_reference_palette_png(path, cgram_rgba: &[[u8;4];256])` (256×1 RGBA8 from the CGRAM — mirror `run_dump_reference_palette`'s encoding). If `run_dump_reference_palette` already has a reusable writer, call it instead of duplicating.

- [ ] **Step 3: Build clean.** `cargo build --profile parity -p zelda3-bin 2>&1 | tail -3` → no errors.

- [ ] **Step 4: Smoke test (documented, needs the ROM).** Run:
```bash
HACKS=(ZELDA3_SMV_SELECT_FILE_TIMING_HACKS=1 ZELDA3_SMV_LOADFILE_TIMING_HACKS=1 ZELDA3_SMV_DUNGEON_TIMING_HACKS=1 ZELDA3_SMV_OVERWORLD_TIMING_HACKS=1 ZELDA3_SMV_MESSAGING_TIMING_HACKS=1 ZELDA3_SMV_DEATH_INTRO_TIMING_HACKS=1 ZELDA3_SMV_DEATH_RELOAD_TIMING_HACKS=1)
env "${HACKS[@]}" target/parity/zelda3 --dump-hd-capture 20000
ls -la hd_art/capture/
```
Expect `frame_20000.png` (256×224), `frame_20000.map.json` (non-empty array), `reference_palette.png` (256×1). Note the outputs in the task report; do not commit them.

- [ ] **Step 5: Commit** (code only — `hd_art/` is gitignored by Task 6; if that task hasn't run yet, add only the source file):
```bash
git add zelda3-bin/src/main.rs
git commit --no-verify -m "feat(zelda3-bin): --dump-hd-capture (frame RGBA + placement map + reference palette)"
```

---

### Task 4: Super-resolution script (`scripts/hd_super_resolve.py`)

**Files:**
- Create: `scripts/hd_super_resolve.py`

**Interfaces:**
- CLI: `python3 scripts/hd_super_resolve.py --in hd_art/capture --out hd_art/sr --scale 4 [--model anime|photo]`. Reads every `frame_*.png` in `--in` (skips `reference_palette.png`), writes `frame_<n>.x<scale>.png` to `--out`. Uses torch MPS if available. Model weights cached under `hd_art/models/`.

- [ ] **Step 1: Write a geometry self-test first.** Create `scripts/hd_super_resolve.py` with an argparse CLI and a `--self-test` flag that, without downloading weights, runs the *plumbing* on a synthetic image using a trivial nearest-upscale fallback and asserts output dims == input × scale. This keeps the script testable without the model:

```python
#!/usr/bin/env python3
"""Super-resolve captured frames with Real-ESRGAN (torch/MPS) for HD-art authoring.

Offline authoring only; not on the game path. Output PNGs are gitignored.
"""
import argparse, os, sys, glob
from PIL import Image
import numpy as np

def _device():
    import torch
    if torch.backends.mps.is_available():
        return "mps"
    return "cuda" if torch.cuda.is_available() else "cpu"

def _load_model(scale, model, cache_dir):
    """Return a callable img(np.uint8 HWC RGB) -> np.uint8 HWC RGB upscaled by `scale`.
    Uses Real-ESRGAN RRDBNet weights (anime by default). Downloads once to cache_dir."""
    import torch
    from basicsr.archs.rrdbnet_arch import RRDBNet
    from realesrgan import RealESRGANer
    os.makedirs(cache_dir, exist_ok=True)
    if model == "anime":
        net = RRDBNet(num_in_ch=3, num_out_ch=3, num_feat=64, num_block=6, num_grow_ch=32, scale=4)
        url = "https://github.com/xinntao/Real-ESRGAN/releases/download/v0.2.2.4/RealESRGAN_x4plus_anime_6B.pth"
        name = "RealESRGAN_x4plus_anime_6B.pth"
    else:
        net = RRDBNet(num_in_ch=3, num_out_ch=3, num_feat=64, num_block=23, num_grow_ch=32, scale=4)
        url = "https://github.com/xinntao/Real-ESRGAN/releases/download/v0.1.0/RealESRGAN_x4plus.pth"
        name = "RealESRGAN_x4plus.pth"
    path = os.path.join(cache_dir, name)
    if not os.path.exists(path):
        torch.hub.download_url_to_file(url, path)
    up = RealESRGANer(scale=4, model_path=path, model=net, half=False, device=_device())
    def run(arr):
        out, _ = up.enhance(arr, outscale=scale)
        return out
    return run

def _nearest(arr, scale):
    return np.array(Image.fromarray(arr).resize(
        (arr.shape[1] * scale, arr.shape[0] * scale), Image.NEAREST))

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--in", dest="indir", default="hd_art/capture")
    ap.add_argument("--out", dest="outdir", default="hd_art/sr")
    ap.add_argument("--scale", type=int, default=4)
    ap.add_argument("--model", choices=["anime", "photo"], default="anime")
    ap.add_argument("--cache", default="hd_art/models")
    ap.add_argument("--self-test", action="store_true")
    args = ap.parse_args()

    if args.self_test:
        img = (np.random.default_rng(0).integers(0, 256, (8, 8, 3))).astype(np.uint8)
        out = _nearest(img, args.scale)
        assert out.shape == (8 * args.scale, 8 * args.scale, 3), out.shape
        print("self-test OK:", out.shape)
        return

    os.makedirs(args.outdir, exist_ok=True)
    run = _load_model(args.scale, args.model, args.cache)
    frames = [f for f in sorted(glob.glob(os.path.join(args.indir, "frame_*.png")))
              if "reference_palette" not in os.path.basename(f)]
    if not frames:
        print(f"no frame_*.png in {args.indir}", file=sys.stderr); sys.exit(1)
    for f in frames:
        base = os.path.splitext(os.path.basename(f))[0]
        img = np.array(Image.open(f).convert("RGB"))
        out = run(img)
        dst = os.path.join(args.outdir, f"{base}.x{args.scale}.png")
        Image.fromarray(out).save(dst)
        print("wrote", dst, out.shape)

if __name__ == "__main__":
    main()
```

- [ ] **Step 2: Run the self-test, expect PASS.** `python3 scripts/hd_super_resolve.py --self-test --scale 4` → prints `self-test OK: (32, 32, 3)`.

- [ ] **Step 3: Document the real dependency.** In the task report, note that the full run needs `pip install realesrgan basicsr` (which pull the model arch); the weights download once to `hd_art/models/`. Do not add these to any Rust manifest.

- [ ] **Step 4: Commit** (script only):
```bash
git add scripts/hd_super_resolve.py
git commit --no-verify -m "feat(scripts): Real-ESRGAN (torch/MPS) SR for HD-art authoring"
```

---

### Task 5: `--slice-hd-cells` subcommand + manifest assembly

**Files:**
- Modify: `zelda3-bin/src/main.rs` (dispatch near line 240 + a new `run_slice_hd_cells` fn)

**Interfaces:**
- Consumes: `renderer::hd_authoring::{HdPlacement, slice_hd_cell}`, the SR frames in `hd_art/sr/frame_<n>.x<scale>.png`, the maps in `hd_art/capture/frame_<n>.map.json`, and `hd_art/capture/reference_palette.png`.
- Produces: `hd_art/cells/<key>.png` per unique key (keep-first), and `hd_art/manifest.json` referencing `capture/reference_palette.png` + `cells/<key>.png`.

- [ ] **Step 1: Add the dispatch line** after Task 3's block:
```rust
    if args.get(1).map(String::as_str) == Some("--slice-hd-cells") {
        run_slice_hd_cells(&args[2..]);
        return;
    }
```

- [ ] **Step 2: Write `run_slice_hd_cells`.** Args: `<scale>` (default 4). For each `frame_<n>.map.json` in `hd_art/capture/` (sorted ascending by `n` for deterministic keep-first): load the map (`serde_json::from_slice::<Vec<HdPlacement>>`), open the matching `hd_art/sr/frame_<n>.x<scale>.png` (decode to RGBA8 `sr`, `sr_w`, `sr_h`; skip the frame with a warning if the SR file is missing). For each placement not already written: parse `key` (`u64::from_str_radix(key.trim_start_matches("0x"), 16)`), call `slice_hd_cell(&sr, sr_w, sr_h, p.x, p.y, p.w, p.h, scale)`; on `Some(cell)`, `write_rgba_png("hd_art/cells/<key>.png", &cell, p.w as u32*scale, p.h as u32*scale)` and record the key in a `HashSet` + an ordered `Vec<(String,String)>` of `(key, "cells/<key>.png")`. Then write the manifest:

```rust
#[derive(serde::Serialize)]
struct OutManifest { reference_palette: String, overrides: Vec<OutOverride> }
#[derive(serde::Serialize)]
struct OutOverride { key: String, rgba: String }

let overrides: Vec<OutOverride> = written.iter()
    .map(|(k, path)| OutOverride { key: k.clone(), rgba: path.clone() }).collect();
let manifest = OutManifest { reference_palette: "capture/reference_palette.png".into(), overrides };
std::fs::write("hd_art/manifest.json", serde_json::to_vec_pretty(&manifest).unwrap())
    .expect("write manifest");
println!("wrote {} cells + hd_art/manifest.json", written.len());
```

Reuse the `write_rgba_png` helper from Task 3. Create `hd_art/cells/` with `create_dir_all` first.

- [ ] **Step 3: Build clean.** `cargo build --profile parity -p zelda3-bin 2>&1 | tail -3` → no errors.

- [ ] **Step 4: Smoke test with synthetic SR input (no ROM/model needed).** After a Task-3 capture exists, fake an SR frame by nearest-upscaling the captured frame and slice:
```bash
python3 scripts/hd_super_resolve.py --in hd_art/capture --out hd_art/sr --scale 4 --self-test  # sanity
# Produce SR frames via nearest as a stand-in for the smoke (real run uses the model):
python3 - <<'PY'
import glob, os, numpy as np
from PIL import Image
os.makedirs("hd_art/sr", exist_ok=True)
for f in glob.glob("hd_art/capture/frame_*.png"):
    if "reference_palette" in f: continue
    im = np.array(Image.open(f).convert("RGB"))
    up = Image.fromarray(im).resize((im.shape[1]*4, im.shape[0]*4), Image.NEAREST)
    b = os.path.splitext(os.path.basename(f))[0]
    up.save(f"hd_art/sr/{b}.x4.png")
PY
target/parity/zelda3 --slice-hd-cells 4
ls hd_art/cells | head; cat hd_art/manifest.json | head
```
Expect `hd_art/cells/0x*.png` files (each 32×32) and a `manifest.json` with matching `overrides`. Verify the manifest loads:
```bash
ZELDA3_MODERN_HD_OVERRIDES=hd_art/manifest.json target/parity/zelda3 --dump-frame /tmp/x.png 20000 2>&1 | tail -3
```
(no "parse"/"read" manifest errors). Note results in the report; do not commit `hd_art/`.

- [ ] **Step 5: Commit** (code only):
```bash
git add zelda3-bin/src/main.rs
git commit --no-verify -m "feat(zelda3-bin): --slice-hd-cells + HD override manifest assembly"
```

---

### Task 6: Gitignore, README, and the Kakariko 4× proof

**Files:**
- Modify: `.gitignore`
- Create: `hd_art/README.md`

**Interfaces:** Consumes all prior tasks + `scripts/hd_super_resolve.py`.

- [ ] **Step 1: Gitignore the authoring outputs.** Add to `.gitignore`:
```
# HD-art authoring outputs + model weights (regenerable; never commit)
/hd_art/
```
Verify: `git status --porcelain hd_art/ | head` prints nothing (ignored).

- [ ] **Step 2: Write `hd_art/README.md`** documenting the 4-stage pipeline verbatim as commands (capture → `hd_super_resolve.py` → `--slice-hd-cells` → run), the one-palette-per-manifest constraint, and the proof command. Include the `pip install realesrgan basicsr` prerequisite and the timing-hack env block. (This file lives under the gitignored dir but is force-added below so the docs are tracked.)

- [ ] **Step 3: Find representative Kakariko frames.** Kakariko is an overworld scene. Locate frames where it is on-screen in the combined replay by dumping a coarse sweep and eyeballing:
```bash
for n in 15000 20000 25000 30000 35000 40000; do
  env "${HACKS[@]}" target/parity/zelda3 --dump-frame /tmp/f_$n.png $n; done
# open /tmp/f_*.png, pick the frame numbers showing Kakariko village (houses, well, paths)
```
Record the chosen Kakariko frame numbers in the report. (If none of the sweep shows Kakariko, widen the sweep; the combined route does pass through it.)

- [ ] **Step 4: Run the real pipeline for the chosen frames.**
```bash
pip install realesrgan basicsr    # once
env "${HACKS[@]}" target/parity/zelda3 --dump-hd-capture <k1> <k2> <k3>
python3 scripts/hd_super_resolve.py --in hd_art/capture --out hd_art/sr --scale 4 --model anime
target/parity/zelda3 --slice-hd-cells 4
```

- [ ] **Step 5: Prove HD in-game (manual visual check).**
```bash
env "${HACKS[@]}" ZELDA3_RENDERER=assets-anim ZELDA3_HD_SCALE=4 \
  ZELDA3_MODERN_HD_OVERRIDES=hd_art/manifest.json \
  target/parity/zelda3 --replay-save saves/zelda3.sfc saves/zelda3-combined-route.sav <k1>
```
Confirm: Kakariko cells with overrides show HD detail (crisper than the blocky nearest baseline); cells without overrides stay blocky; and WITHOUT the three env vars the display is unchanged. Capture a before/after screenshot pair for the report.

- [ ] **Step 6: Commit** (docs + gitignore only; the `hd_art/*` assets stay ignored, README force-added):
```bash
git add .gitignore
git add -f hd_art/README.md
git commit --no-verify -m "docs: HD-art authoring pipeline + gitignore generated assets"
```

---

## Notes for the executor

- Tasks 1, 2, 4 are self-contained and fully testable without the ROM or the model. Tasks 3, 5, 6 are integration/authoring glue exercised with the ROM; their automated coverage is the Task 1/2 cores plus the documented smoke runs.
- If `slice_hd_cell` returns `None` for most placements, the SR frame size doesn't match `256*scale × 224*scale` — check the SR `--scale` matches the slice `<scale>` and `ZELDA3_HD_SCALE`.
- Keep the render/parity path untouched; if any task tempts you to edit `modern_software.rs` render logic, stop — the override system is already complete.
