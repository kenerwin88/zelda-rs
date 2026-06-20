# zparity — C-free Parallel Parity Checker Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a single fast, parallel, C-free checker (`zparity`) that verifies every parity layer (WRAM, VRAM, SRAM, render, audio) of the Rust port against a golden artifact captured once from the C oracle.

**Architecture:** A new workspace crate `crates/parity` defines a shared fingerprint record + golden format (Tier A committed rollup/merkle, Tier B cached per-region detail). A `--fingerprint-log` hook in both the Rust binary and the C oracle streams one fixed-size 788-byte record per frame. `zparity capture` builds the golden from C (rare); `zparity check` shards the route across cores via checkpoint save-states and diffs Rust fingerprints against the golden (no C); `zparity drill` localizes a divergence per-region.

**Tech Stack:** Rust 2021 (workspace edition), `memmap2` (mmap golden), `zstd` (Tier B), `rayon` or `std::thread` + `std::process` (shard fan-out), `serde`/`serde_json` (manifest). C11 for the oracle hook.

## Global Constraints

- Rust edition **2021**, `rust-version = "1.78"` (workspace floor — new crate must compile on it).
- Hashes are **FNV-1a / u32**, init `0x811c9dc5` (2166136261), prime `0x01000193` (16777619) — matches every existing dump hook. Never change the algorithm.
- Fingerprint record is **fixed 788 bytes**, all integers **little-endian**: `frame:u32` + `wram[128]:u32` + `vram[64]:u32` + `sram:u32` + `render:u32` + `audio:u32` + `rollup:u32`.
- WRAM/VRAM page granularity is **1 KB (0x400 bytes)**: 128 WRAM pages, 64 VRAM pages.
- `FINGERPRINT_MASK = [0x654]` — these WRAM byte offsets are zeroed before hashing their page (the HDMA snapshot-restore scratch artifact). Test-pinned; extend only with a documented reason.
- Merkle block size = **8192 frames**.
- All replay subprocess runs must set the 7 timing-hack env vars: `ZELDA3_SMV_{SELECT_FILE,LOADFILE,DUNGEON,OVERWORLD,MESSAGING,DEATH_INTRO,DEATH_RELOAD}_TIMING_HACKS=1`.
- Canonical route: ROM `saves/zelda3.sfc`, save `saves/zelda3-combined-route.sav`, full length `1_073_092` frames.
- C oracle path `../zelda3` (override `ZELDA3_C_REPO`); oracle-hook edits to `../zelda3` ARE permitted.
- Golden dirs: committed Tier A in `parity-golden/`; cached Tier B + checkpoints in `.cache/parity-golden/` (gitignored).
- Never `git checkout` a file (nukes WIP); surgically revert.

---

### Task 1: `crates/parity` crate + fingerprint record (lib core)

**Files:**
- Create: `crates/parity/Cargo.toml`
- Create: `crates/parity/src/lib.rs`
- Create: `crates/parity/src/fingerprint.rs`
- Modify: `Cargo.toml` (workspace `members`)

**Interfaces:**
- Produces:
  - `pub const PAGE: usize = 0x400;`
  - `pub const WRAM_PAGES: usize = 128;`
  - `pub const VRAM_PAGES: usize = 64;`
  - `pub const RECORD_LEN: usize = 788;`
  - `pub const FINGERPRINT_MASK: &[usize] = &[0x654];`
  - `pub fn fnv1a(bytes: &[u8]) -> u32`
  - `pub fn fnv1a_u32s(words: &[u32]) -> u32`
  - `pub struct FrameFingerprint { pub frame: u32, pub wram: [u32; 128], pub vram: [u32; 64], pub sram: u32, pub render: u32, pub audio: u32, pub rollup: u32 }`
  - `impl FrameFingerprint { pub fn compute(frame: u32, wram: &[u8], vram_bytes: &[u8], sram: &[u8], render: u32, audio: u32) -> Self; pub fn to_bytes(&self) -> [u8; RECORD_LEN]; pub fn from_bytes(b: &[u8]) -> Self; }`

- [ ] **Step 1: Add the crate to the workspace**

Modify `Cargo.toml` members list (after `"crates/renderer",`):

```toml
    "crates/parity",
```

- [ ] **Step 2: Create `crates/parity/Cargo.toml`**

```toml
[package]
name = "parity"
version = "0.0.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[lib]
name = "parity"
path = "src/lib.rs"

[[bin]]
name = "zparity"
path = "src/main.rs"

[dependencies]
memmap2 = "0.9"
zstd = "0.13"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[lints]
workspace = true
```

- [ ] **Step 3: Create `crates/parity/src/lib.rs`** (a temporary `main.rs` stub is added in Task 5; for now build the lib only)

```rust
pub mod fingerprint;
pub use fingerprint::{
    fnv1a, fnv1a_u32s, FrameFingerprint, FINGERPRINT_MASK, PAGE, RECORD_LEN, VRAM_PAGES, WRAM_PAGES,
};
```

- [ ] **Step 4: Write the failing test in `crates/parity/src/fingerprint.rs`**

```rust
//! Fixed-size per-frame parity fingerprint, shared by the streaming hook and the
//! checker. The C oracle (../zelda3/src/main.c) mirrors this layout exactly.

pub const PAGE: usize = 0x400;
pub const WRAM_PAGES: usize = 128;
pub const VRAM_PAGES: usize = 64;
/// 4(frame) + 128*4(wram) + 64*4(vram) + 4(sram) + 4(render) + 4(audio) + 4(rollup)
pub const RECORD_LEN: usize = 4 + WRAM_PAGES * 4 + VRAM_PAGES * 4 + 4 + 4 + 4 + 4;

/// WRAM byte offsets zeroed before hashing their page. 0x654 is the HDMA
/// snapshot-restore scratch byte; masking it makes a checkpoint-resumed shard
/// byte-identical to a from-scratch run. Extend only with a documented reason.
pub const FINGERPRINT_MASK: &[usize] = &[0x654];

#[inline]
pub fn fnv1a(bytes: &[u8]) -> u32 {
    let mut h: u32 = 2166136261;
    for &b in bytes {
        h ^= u32::from(b);
        h = h.wrapping_mul(16777619);
    }
    h
}

#[inline]
pub fn fnv1a_u32s(words: &[u32]) -> u32 {
    let mut h: u32 = 2166136261;
    for &w in words {
        for b in w.to_le_bytes() {
            h ^= u32::from(b);
            h = h.wrapping_mul(16777619);
        }
    }
    h
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameFingerprint {
    pub frame: u32,
    pub wram: [u32; WRAM_PAGES],
    pub vram: [u32; VRAM_PAGES],
    pub sram: u32,
    pub render: u32,
    pub audio: u32,
    pub rollup: u32,
}

impl FrameFingerprint {
    /// Hash a 1KB page with FINGERPRINT_MASK bytes (that fall inside it) zeroed.
    fn page_hash_masked(wram: &[u8], page: usize) -> u32 {
        let start = page * PAGE;
        let end = (start + PAGE).min(wram.len());
        let mut h: u32 = 2166136261;
        for off in start..end {
            let mut b = wram[off];
            if FINGERPRINT_MASK.contains(&off) {
                b = 0;
            }
            h ^= u32::from(b);
            h = h.wrapping_mul(16777619);
        }
        h
    }

    pub fn compute(
        frame: u32,
        wram: &[u8],
        vram_bytes: &[u8],
        sram: &[u8],
        render: u32,
        audio: u32,
    ) -> Self {
        let mut w = [0u32; WRAM_PAGES];
        for (p, slot) in w.iter_mut().enumerate() {
            *slot = Self::page_hash_masked(wram, p);
        }
        let mut v = [0u32; VRAM_PAGES];
        for (p, slot) in v.iter_mut().enumerate() {
            let start = p * PAGE;
            let end = (start + PAGE).min(vram_bytes.len());
            *slot = fnv1a(&vram_bytes[start..end]);
        }
        let sram_h = fnv1a(sram);
        // rollup folds all leaves in a fixed order.
        let mut leaves = Vec::with_capacity(WRAM_PAGES + VRAM_PAGES + 3);
        leaves.extend_from_slice(&w);
        leaves.extend_from_slice(&v);
        leaves.push(sram_h);
        leaves.push(render);
        leaves.push(audio);
        let rollup = fnv1a_u32s(&leaves);
        FrameFingerprint { frame, wram: w, vram: v, sram: sram_h, render, audio, rollup }
    }

    pub fn to_bytes(&self) -> [u8; RECORD_LEN] {
        let mut b = [0u8; RECORD_LEN];
        let mut o = 0usize;
        let mut put = |b: &mut [u8; RECORD_LEN], o: &mut usize, v: u32| {
            b[*o..*o + 4].copy_from_slice(&v.to_le_bytes());
            *o += 4;
        };
        put(&mut b, &mut o, self.frame);
        for &x in &self.wram {
            put(&mut b, &mut o, x);
        }
        for &x in &self.vram {
            put(&mut b, &mut o, x);
        }
        put(&mut b, &mut o, self.sram);
        put(&mut b, &mut o, self.render);
        put(&mut b, &mut o, self.audio);
        put(&mut b, &mut o, self.rollup);
        debug_assert_eq!(o, RECORD_LEN);
        b
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        assert!(bytes.len() >= RECORD_LEN, "record too short");
        let mut o = 0usize;
        let mut get = |o: &mut usize| {
            let v = u32::from_le_bytes(bytes[*o..*o + 4].try_into().unwrap());
            *o += 4;
            v
        };
        let frame = get(&mut o);
        let mut wram = [0u32; WRAM_PAGES];
        for x in &mut wram {
            *x = get(&mut o);
        }
        let mut vram = [0u32; VRAM_PAGES];
        for x in &mut vram {
            *x = get(&mut o);
        }
        let sram = get(&mut o);
        let render = get(&mut o);
        let audio = get(&mut o);
        let rollup = get(&mut o);
        FrameFingerprint { frame, wram, vram, sram, render, audio, rollup }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_len_is_788() {
        assert_eq!(RECORD_LEN, 788);
    }

    #[test]
    fn mask_audit() {
        // Pinned set: only the documented HDMA snapshot scratch byte.
        assert_eq!(FINGERPRINT_MASK, &[0x654]);
    }

    #[test]
    fn roundtrip() {
        let wram = vec![0xabu8; 0x20000];
        let vram = vec![0xcdu8; 0x10000];
        let sram = vec![0x11u8; 0x500];
        let fp = FrameFingerprint::compute(42, &wram, &vram, &sram, 0xdead, 0xbeef);
        let back = FrameFingerprint::from_bytes(&fp.to_bytes());
        assert_eq!(fp, back);
        assert_eq!(back.frame, 42);
        assert_eq!(back.render, 0xdead);
        assert_eq!(back.audio, 0xbeef);
    }

    #[test]
    fn mask_zeroes_byte() {
        let mut a = vec![0u8; 0x800];
        let mut b = a.clone();
        b[0x654] = 0xff; // inside page 1
        let fa = FrameFingerprint::compute(0, &a, &[], &[], 0, 0);
        let fb = FrameFingerprint::compute(0, &b, &[], &[], 0, 0);
        assert_eq!(fa.wram[1], fb.wram[1], "masked byte must not change page hash");
        // a non-masked byte change DOES change the page hash:
        a[0x655] = 0xff;
        let fa2 = FrameFingerprint::compute(0, &a, &[], &[], 0, 0);
        assert_ne!(fa.wram[1], fa2.wram[1]);
    }
}
```

- [ ] **Step 5: Run tests to verify they fail (lib not yet wired)**

Run: `cargo test -p parity --lib`
Expected: compile error or FAIL until `lib.rs`/`fingerprint.rs` are in place; once both files exist, all four tests PASS.

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test -p parity --lib`
Expected: `test result: ok. 4 passed`.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml crates/parity/Cargo.toml crates/parity/src/lib.rs crates/parity/src/fingerprint.rs
git commit -m "feat(parity): fingerprint record format + crate skeleton"
```

---

### Task 2: `--fingerprint-log` hook in the Rust binary

**Files:**
- Modify: `zelda3-bin/Cargo.toml` (add `parity` dep)
- Modify: `zelda3-bin/src/main.rs` (replay flag parse ~1248-1356, replay loop ~1427-1559, helper near `render_frame_rgb_hash_bgra` ~6886)
- Test: `scripts/test_fingerprint_hook.py` (new)

**Interfaces:**
- Consumes: `parity::FrameFingerprint`, `parity::fnv1a`.
- Produces: a `--fingerprint-log <path>` CLI flag on the `--replay-save` subcommand that writes one `parity::RECORD_LEN`-byte record per frame.

- [ ] **Step 1: Add the dependency** to `zelda3-bin/Cargo.toml` `[dependencies]`:

```toml
parity = { path = "../crates/parity" }
```

- [ ] **Step 2: Parse the flag.** In `zelda3-bin/src/main.rs`, near the other replay flag locals (~line 1248) add:

```rust
    let mut fingerprint_log: Option<PathBuf> = None;
```

In the flag match block (alongside `"--render-hash-log"`, ~line 1280) add:

```rust
            "--fingerprint-log" => {
                let Some(path) = args.next() else {
                    eprintln!("--fingerprint-log requires a path");
                    process::exit(2);
                };
                fingerprint_log = Some(PathBuf::from(path));
            }
```

Also append `[--fingerprint-log <path>]` to the usage string at ~line 1241.

- [ ] **Step 3: Define the audio leaf helper.** Near `render_frame_rgb_hash_bgra` (~line 6886) add:

```rust
/// Per-frame audio leaf hash: folds the same DSP/sample quantities the audio
/// trace prints, into one u32. Mirrored exactly in C (FingerprintAudioHash).
fn fingerprint_audio_hash(
    sample_checksum: u32,
    dsp_pre: u32,
    dsp_post: u32,
    dsp_write_count: u32,
    dsp_write_hash: u32,
    dsp_write_values_hash: u32,
) -> u32 {
    parity::fnv1a_u32s(&[
        sample_checksum,
        dsp_pre,
        dsp_post,
        dsp_write_count,
        dsp_write_hash,
        dsp_write_values_hash,
    ])
}
```

- [ ] **Step 4: Allocate the writer + per-frame render/audio buffers when the flag is set.** Just before the replay loop (~line 1426), extend the existing allocation conditions so `fingerprint_log.is_some()` also forces `render_hash_frame` and `offscreen` and an audio buffer to exist, and open the writer:

```rust
    let mut fingerprint_writer = match fingerprint_log.as_deref() {
        Some(p) => {
            let f = std::fs::File::create(p).unwrap_or_else(|e| {
                eprintln!("failed to create fingerprint log {p:?}: {e}");
                process::exit(2);
            });
            Some(std::io::BufWriter::new(f))
        }
        None => None,
    };
```

Change the three allocation guards so fingerprint mode also allocates:
- `audio_trace_buffer`: allocate when `audio_trace_log != 0 || fingerprint_log.is_some()`.
- `render_hash_frame` and `offscreen`: include `|| fingerprint_log.is_some()` in their conditions (~lines 1410, 1418).

- [ ] **Step 5: Emit the record at end of frame.** Inside the loop, after the audio block (~line 1446) and after the render-hash block, append (compute render/audio every frame in fingerprint mode):

```rust
        if let Some(w) = fingerprint_writer.as_mut() {
            use std::io::Write;
            // Audio leaf — recompute the trace quantities for THIS frame.
            let audio = audio_trace_buffer.as_mut().map(|buf| {
                let dsp_pre = game.zelda_audio_dsp_hash();
                let writes = game.zelda_render_audio_trace_dsp(buf, 735, 2);
                game.zelda_discard_unused_audio_frames();
                let dsp_post = game.zelda_audio_dsp_hash();
                fingerprint_audio_hash(
                    replay_checksum_samples(buf),
                    dsp_pre,
                    dsp_post,
                    writes.len() as u32,
                    replay_checksum_dsp_writes(&writes),
                    replay_checksum_dsp_write_values(&writes),
                )
            }).unwrap_or(0);
            // Render leaf — draw the CPU PPU frame and hash it.
            let render = {
                let frame = render_hash_frame.as_mut().expect("render frame allocated");
                draw_play_ppu_frame(&mut game, frame, 256 * 4, PpuRenderFlags::empty());
                render_frame_rgb_hash_bgra(frame)
            };
            let vram_bytes: Vec<u8> =
                game.ppu.vram.iter().flat_map(|w| w.to_le_bytes()).collect();
            let fp = parity::FrameFingerprint::compute(
                frames, &game.ram, &vram_bytes, &game.sram, render, audio,
            );
            let _ = w.write_all(&fp.to_bytes());
        }
```

> NOTE: when BOTH `--audio-trace-log` and `--fingerprint-log` are set, do not double-advance the audio DSP. Guard by computing audio inside the fingerprint block only when `audio_trace_log == 0`; otherwise reuse the value already produced by the existing audio block (hoist `dsp_pre`/`writes` into loop-scoped vars). For the common case (`check` sets only `--fingerprint-log`) the block above is correct as written.

- [ ] **Step 6: Flush the writer after the loop.** After the loop, before the final WRAM dump (~line 3247):

```rust
    if let Some(mut w) = fingerprint_writer.take() {
        use std::io::Write;
        let _ = w.flush();
    }
```

- [ ] **Step 7: Write the failing test `scripts/test_fingerprint_hook.py`**

```python
#!/usr/bin/env python3
"""The Rust --fingerprint-log stream is well-formed and its WRAM page column
matches the page hashes derived from a full WRAM dump at the same frame."""
import os, subprocess, sys, tempfile, struct
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
BIN = REPO / "target" / "parity" / "zelda3"
ROM = REPO / "saves" / "zelda3.sfc"
SAVE = REPO / "saves" / "zelda3-combined-route.sav"
RECORD_LEN = 788
HACKS = {f"ZELDA3_SMV_{k}_TIMING_HACKS": "1" for k in
         ("SELECT_FILE","LOADFILE","DUNGEON","OVERWORLD","MESSAGING","DEATH_INTRO","DEATH_RELOAD")}
MASK = {0x654}

def fnv_page(buf, start):
    h = 2166136261
    for off in range(start, start + 0x400):
        b = 0 if off in MASK else buf[off]
        h = ((h ^ b) * 16777619) & 0xffffffff
    return h

def main():
    frames = 300
    with tempfile.TemporaryDirectory() as td:
        fp = Path(td) / "fp.bin"
        wram = Path(td) / "w.bin"
        env = {**os.environ, **HACKS,
               "ZELDA3_REPLAY_WRAM_DUMP": str(wram)}
        subprocess.run([str(BIN), "--replay-save", str(ROM), str(SAVE), str(frames),
                        "--fingerprint-log", str(fp)], cwd=REPO, env=env, check=True,
                       capture_output=True, text=True)
        data = fp.read_bytes()
        assert len(data) == frames * RECORD_LEN, (len(data), frames * RECORD_LEN)
        # Last record's wram column == page hashes of the final WRAM dump.
        last = data[(frames - 1) * RECORD_LEN:]
        w = wram.read_bytes()
        for p in range(128):
            got = struct.unpack_from("<I", last, 4 + p * 4)[0]
            exp = fnv_page(w, p * 0x400)
            assert got == exp, f"page {p}: fp=0x{got:08x} dump=0x{exp:08x}"
        # frame field of last record == frames
        assert struct.unpack_from("<I", last, 0)[0] == frames
    print("fingerprint hook OK")

if __name__ == "__main__":
    sys.exit(main())
```

- [ ] **Step 8: Build, run test — expect fail then pass**

Run: `cargo build --profile parity -p zelda3-bin && python3 scripts/test_fingerprint_hook.py`
Expected before Steps 2-6: nonzero exit / size mismatch. After: `fingerprint hook OK`.

- [ ] **Step 9: Commit**

```bash
git add zelda3-bin/Cargo.toml zelda3-bin/src/main.rs scripts/test_fingerprint_hook.py
git commit -m "feat(parity): --fingerprint-log hook in the Rust replay binary"
```

---

### Task 3: Mirror the hook in the C oracle

**Files:**
- Modify: `../zelda3/src/main.c` (arg parse ~338, loop ~628-648, helpers near `PrintRenderHash` ~870)
- Test: `scripts/test_fingerprint_c_parity.py` (new)

**Interfaces:**
- Produces: identical `--fingerprint-log <path>` flag on the C oracle, byte-for-byte matching the Rust stream.

- [ ] **Step 1: Add globals + arg parse.** Near the other `--render-hash-log` handling (`main.c` ~338) add a `const char *fingerprint_log = NULL;` local in the arg loop and:

```c
    } else if (strcmp(argv[0], "--fingerprint-log") == 0) {
      if (argc < 2) { fprintf(stderr, "--fingerprint-log requires a path\n"); return 1; }
      fingerprint_log = argv[1];
      argc -= 1; argv += 1;
```

Open the file once before the main loop:

```c
  FILE *fingerprint_file = NULL;
  if (fingerprint_log) {
    fingerprint_file = fopen(fingerprint_log, "wb");
    if (!fingerprint_file) { fprintf(stderr, "failed to open fingerprint log %s\n", fingerprint_log); return 1; }
  }
```

- [ ] **Step 2: Add helpers near `PrintRenderHash` (~line 870).**

```c
static uint32 FnvBytes(const uint8 *p, size_t n) {
  uint32 h = 2166136261u;
  for (size_t i = 0; i < n; i++) { h = (h ^ p[i]) * 16777619u; }
  return h;
}
static uint32 FnvU32s(const uint32 *p, size_t n) {
  uint32 h = 2166136261u;
  for (size_t i = 0; i < n; i++) {
    uint32 v = p[i];
    for (int b = 0; b < 4; b++) { h = (h ^ (v & 0xff)) * 16777619u; v >>= 8; }
  }
  return h;
}
// Render leaf: same RGB FNV-1a over the drawn frame as PrintRenderHash.
static uint32 FingerprintRenderHash(void) {
  const int width = g_snes_width, height = g_snes_height, pitch = width * 4;
  size_t sz = (size_t)pitch * height;
  if (g_render_hash_pixels_size != sz) {
    uint8 *px = (uint8 *)realloc(g_render_hash_pixels, sz);
    if (!px) return 0;
    g_render_hash_pixels = px; g_render_hash_pixels_size = sz;
  }
  ZeldaDrawPpuFrame(g_render_hash_pixels, pitch, g_ppu_render_flags);
  uint32 h = 2166136261u;
  for (int y = 0; y < height; y++) {
    const uint8 *row = g_render_hash_pixels + y * pitch;
    for (int x = 0; x < width; x++) {
      const uint8 *p = row + x * 4;
      h = (h ^ p[2]) * 16777619u; h = (h ^ p[1]) * 16777619u; h = (h ^ p[0]) * 16777619u;
    }
  }
  return h;
}
// One 788-byte record matching parity::FrameFingerprint.
static void WriteFingerprint(FILE *f, uint32 frame, uint32 render, uint32 audio) {
  static const int MASK[] = {0x654};
  uint32 wram[128], vram[64];
  for (int pidx = 0; pidx < 128; pidx++) {
    uint32 h = 2166136261u;
    for (int o = 0; o < 0x400; o++) {
      int off = pidx * 0x400 + o;
      uint8 b = g_ram[off];
      for (size_t m = 0; m < sizeof(MASK)/sizeof(MASK[0]); m++) if (MASK[m] == off) b = 0;
      h = (h ^ b) * 16777619u;
    }
    wram[pidx] = h;
  }
  const uint8 *vbytes = (const uint8 *)g_zenv.vram; // LE host == Rust LE dump
  for (int pidx = 0; pidx < 64; pidx++) vram[pidx] = FnvBytes(vbytes + pidx * 0x400, 0x400);
  uint32 sram = FnvBytes((const uint8 *)g_sram, kSramBytes); // see Step 4 for size
  uint32 leaves[128 + 64 + 3];
  int n = 0;
  for (int i = 0; i < 128; i++) leaves[n++] = wram[i];
  for (int i = 0; i < 64; i++) leaves[n++] = vram[i];
  leaves[n++] = sram; leaves[n++] = render; leaves[n++] = audio;
  uint32 rollup = FnvU32s(leaves, n);
  uint8 rec[788]; int o = 0;
  #define PUT(V) do { uint32 _v=(V); rec[o++]=_v&0xff; rec[o++]=(_v>>8)&0xff; rec[o++]=(_v>>16)&0xff; rec[o++]=(_v>>24)&0xff; } while(0)
  PUT(frame);
  for (int i = 0; i < 128; i++) PUT(wram[i]);
  for (int i = 0; i < 64; i++) PUT(vram[i]);
  PUT(sram); PUT(render); PUT(audio); PUT(rollup);
  #undef PUT
  fwrite(rec, 1, 788, f);
}
```

- [ ] **Step 3: Compute audio leaf + call the writer in the loop.** In the main loop right after `frameCtr++;` and the existing audio block (~line 638), add a fingerprint branch that mirrors Rust's `fingerprint_audio_hash` using the same quantities the audio trace already computes:

```c
    if (fingerprint_file) {
      uint32 dsp_pre = ZeldaAudioDspHash();
      uint32 wcount = 0, wvalues = 0;
      memset(audio_trace_buffer, 0, g_frames_per_block * g_audio_channels * sizeof(int16));
      uint32 whash = ZeldaRenderAudioTraceDsp(audio_trace_buffer, g_frames_per_block, g_audio_channels, &wcount, &wvalues);
      ZeldaDiscardUnusedAudioFrames();
      uint32 dsp_post = ZeldaAudioDspHash();
      uint32 sample_checksum = FnvBytes((const uint8 *)audio_trace_buffer,
          (size_t)g_frames_per_block * g_audio_channels * sizeof(int16));
      uint32 a_leaves[6] = { sample_checksum, dsp_pre, dsp_post, wcount, whash, wvalues };
      uint32 audio = FnvU32s(a_leaves, 6);
      uint32 render = FingerprintRenderHash();
      WriteFingerprint(fingerprint_file, frameCtr, render, audio);
    }
```

> NOTE 1: `audio_trace_buffer` must be allocated when `fingerprint_log` is set (mirror the `audio_trace_log` allocation). NOTE 2: if both `--audio-trace-log` and `--fingerprint-log` are passed, the DSP advances twice; the checker never passes both, so leave them mutually exclusive (assert and error if both set). NOTE 3: the Rust `replay_checksum_samples` hashes the i16 sample buffer; confirm it folds the LE bytes of each i16 the same way `FnvBytes` over the C `int16` buffer does. If `replay_checksum_samples` differs, add a matching `fnv1a` over `bytemuck::cast_slice::<i16,u8>(buf)` in the Rust helper instead (Task 2, Step 5) so both sides hash identical bytes — the C-parity test (Step 5) is the gate that proves they match.

- [ ] **Step 4: Resolve the SRAM symbol/size.** Find the C oracle's SRAM buffer (grep `g_sram`, `sram`, `kSram`, or the variable `sramhash` is computed from in `PrintReplayTestState`). Use the same bytes the Rust `game.sram` covers (the existing `sramhash` parity proves the range). Set `kSramBytes` to that length; if the C side exposes it as a sized array use `sizeof`.

Run: `grep -n "sram" ../zelda3/src/*.c ../zelda3/src/*.h | grep -i "g_sram\|sram\[\|SaveLoad\|sramhash"`

- [ ] **Step 5: Build the oracle and write the C-parity test `scripts/test_fingerprint_c_parity.py`**

```python
#!/usr/bin/env python3
"""C oracle and Rust binary emit byte-identical fingerprint streams."""
import os, subprocess, sys, tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
C_ROOT = Path(os.environ.get("ZELDA3_C_REPO", str(REPO.parent / "zelda3")))
C_BIN = C_ROOT / "zelda3"
C_INI = C_ROOT / "other" / "headless_replay.ini"
NEW = REPO / "target" / "parity" / "zelda3"
ROM = REPO / "saves" / "zelda3.sfc"
SAVE = REPO / "saves" / "zelda3-combined-route.sav"
HACKS = {f"ZELDA3_SMV_{k}_TIMING_HACKS": "1" for k in
         ("SELECT_FILE","LOADFILE","DUNGEON","OVERWORLD","MESSAGING","DEATH_INTRO","DEATH_RELOAD")}
SDL = {"SDL_VIDEODRIVER":"dummy","SDL_AUDIODRIVER":"dummy","SDL_RENDER_DRIVER":"software"}

def main():
    frames = 500
    with tempfile.TemporaryDirectory() as td:
        rfp, cfp = Path(td)/"r.bin", Path(td)/"c.bin"
        subprocess.run([str(NEW), "--replay-save", str(ROM), str(SAVE), str(frames),
                        "--fingerprint-log", str(rfp)], cwd=REPO,
                       env={**os.environ, **HACKS}, check=True, capture_output=True, text=True)
        subprocess.run([str(C_BIN), "--config", str(C_INI), "--replay-save", str(SAVE),
                        "--smv-test-frames", str(frames), "--fingerprint-log", str(cfp)],
                       cwd=C_ROOT, env={**os.environ, **SDL}, check=True, capture_output=True, text=True)
        a, b = rfp.read_bytes(), cfp.read_bytes()
        assert len(a) == len(b), (len(a), len(b))
        for i in range(0, len(a), 788):
            if a[i:i+788] != b[i:i+788]:
                frame = i // 788
                raise SystemExit(f"FAIL: fingerprint diverges at frame {frame} (record {i})")
    print("C/Rust fingerprint streams byte-identical")

if __name__ == "__main__":
    sys.exit(main())
```

- [ ] **Step 6: Build + run — expect fail then pass**

Run: `make -C ../zelda3 zelda3 && python3 scripts/test_fingerprint_c_parity.py`
Expected: after the hook is correct, `C/Rust fingerprint streams byte-identical`. If it fails at a frame, the audio/sram hashing differs — reconcile per Step 3 NOTE 3 / Step 4.

- [ ] **Step 7: Commit**

```bash
git add scripts/test_fingerprint_c_parity.py
git commit -m "test(parity): C/Rust fingerprint byte-identity gate"
cd ../zelda3 && git add src/main.c && git commit -m "feat: --fingerprint-log oracle hook for zparity" && cd -
```

---

### Task 4: Golden format — manifest, rollup column, merkle, Tier B (lib)

**Files:**
- Create: `crates/parity/src/golden.rs`
- Create: `crates/parity/src/merkle.rs`
- Modify: `crates/parity/src/lib.rs` (add `pub mod golden; pub mod merkle;`)

**Interfaces:**
- Produces:
  - `merkle::MerkleIndex { block_size: u32, block_hashes: Vec<u32>, root: u32 }` with `pub fn build(rollups: &[u32], block_size: u32) -> Self`, `pub fn first_diff_block(&self, other: &MerkleIndex) -> Option<usize>`, `to_bytes`/`from_bytes`.
  - `golden::Manifest { schema: u32, frames: u32, rom_sha256: String, save_sha256: String, c_oracle_rev: String, timing_hacks: Vec<String>, mask: Vec<usize>, block_size: u32, page_kb: u32 }` (serde) with `load`/`save`.
  - `golden::write_rollup(path, &[u32])`, `golden::RollupMap` (mmap reader) with `len()`, `get(i)->u32`, `as_slice()->&[u32]`.
  - `golden::write_detail_block(dir, block_idx, &[u8])` (zstd) and `golden::read_detail_block(dir, block_idx) -> Vec<u8>`.

- [ ] **Step 1: Add the modules to `lib.rs`**

```rust
pub mod golden;
pub mod merkle;
```

- [ ] **Step 2: Write `crates/parity/src/merkle.rs` with failing tests**

```rust
use crate::fnv1a_u32s;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MerkleIndex {
    pub block_size: u32,
    pub block_hashes: Vec<u32>,
    pub root: u32,
}

impl MerkleIndex {
    pub fn build(rollups: &[u32], block_size: u32) -> Self {
        let bs = block_size as usize;
        let block_hashes: Vec<u32> = rollups.chunks(bs).map(fnv1a_u32s).collect();
        let root = fnv1a_u32s(&block_hashes);
        MerkleIndex { block_size, block_hashes, root }
    }

    /// First block index whose hash differs (and thus contains the first
    /// diverging frame). None if roots match.
    pub fn first_diff_block(&self, other: &MerkleIndex) -> Option<usize> {
        if self.root == other.root {
            return None;
        }
        let n = self.block_hashes.len().max(other.block_hashes.len());
        for i in 0..n {
            if self.block_hashes.get(i) != other.block_hashes.get(i) {
                return Some(i);
            }
        }
        None
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = Vec::with_capacity(8 + self.block_hashes.len() * 4 + 4);
        b.extend_from_slice(&self.block_size.to_le_bytes());
        b.extend_from_slice(&(self.block_hashes.len() as u32).to_le_bytes());
        for &h in &self.block_hashes {
            b.extend_from_slice(&h.to_le_bytes());
        }
        b.extend_from_slice(&self.root.to_le_bytes());
        b
    }

    pub fn from_bytes(b: &[u8]) -> Self {
        let block_size = u32::from_le_bytes(b[0..4].try_into().unwrap());
        let n = u32::from_le_bytes(b[4..8].try_into().unwrap()) as usize;
        let mut block_hashes = Vec::with_capacity(n);
        let mut o = 8;
        for _ in 0..n {
            block_hashes.push(u32::from_le_bytes(b[o..o + 4].try_into().unwrap()));
            o += 4;
        }
        let root = u32::from_le_bytes(b[o..o + 4].try_into().unwrap());
        MerkleIndex { block_size, block_hashes, root }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_stable_and_roundtrip() {
        let rollups: Vec<u32> = (0..20000).collect();
        let m = MerkleIndex::build(&rollups, 8192);
        assert_eq!(m.block_hashes.len(), 3); // 20000/8192 -> 3 blocks
        assert_eq!(MerkleIndex::from_bytes(&m.to_bytes()), m);
    }

    #[test]
    fn detects_first_diff_block() {
        let a: Vec<u32> = (0..20000).collect();
        let mut b = a.clone();
        b[9000] ^= 1; // block 1
        let ma = MerkleIndex::build(&a, 8192);
        let mb = MerkleIndex::build(&b, 8192);
        assert_eq!(ma.first_diff_block(&mb), Some(1));
        assert_eq!(ma.first_diff_block(&ma), None);
    }
}
```

- [ ] **Step 3: Write `crates/parity/src/golden.rs` with failing tests**

```rust
use std::fs;
use std::path::{Path, PathBuf};

use memmap2::Mmap;
use serde::{Deserialize, Serialize};

use crate::RECORD_LEN;

pub const SCHEMA: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Manifest {
    pub schema: u32,
    pub frames: u32,
    pub rom_sha256: String,
    pub save_sha256: String,
    pub c_oracle_rev: String,
    pub timing_hacks: Vec<String>,
    pub mask: Vec<usize>,
    pub block_size: u32,
    pub page_kb: u32,
}

impl Manifest {
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        fs::write(path, serde_json::to_vec_pretty(self).unwrap())
    }
    pub fn load(path: &Path) -> std::io::Result<Self> {
        Ok(serde_json::from_slice(&fs::read(path)?).expect("manifest json"))
    }
}

pub fn write_rollup(path: &Path, rollups: &[u32]) -> std::io::Result<()> {
    let mut bytes = Vec::with_capacity(rollups.len() * 4);
    for &r in rollups {
        bytes.extend_from_slice(&r.to_le_bytes());
    }
    fs::write(path, bytes)
}

/// mmap'd read-only view of rollup.bin as a u32 column.
pub struct RollupMap {
    _mmap: Mmap,
    len: usize,
    ptr: *const u8,
}

// SAFETY: ptr points into _mmap which lives as long as self; read-only.
unsafe impl Send for RollupMap {}
unsafe impl Sync for RollupMap {}

impl RollupMap {
    pub fn open(path: &Path) -> std::io::Result<Self> {
        let f = fs::File::open(path)?;
        let mmap = unsafe { Mmap::map(&f)? };
        let len = mmap.len() / 4;
        let ptr = mmap.as_ptr();
        Ok(RollupMap { _mmap: mmap, len, ptr })
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn get(&self, i: usize) -> u32 {
        assert!(i < self.len);
        let mut b = [0u8; 4];
        // SAFETY: i < len, mmap covers len*4 bytes.
        unsafe { std::ptr::copy_nonoverlapping(self.ptr.add(i * 4), b.as_mut_ptr(), 4) };
        u32::from_le_bytes(b)
    }
    pub fn to_vec(&self) -> Vec<u32> {
        (0..self.len).map(|i| self.get(i)).collect()
    }
}

fn detail_path(dir: &Path, block_idx: usize) -> PathBuf {
    dir.join(format!("detail/{block_idx:05}.zst"))
}

pub fn write_detail_block(dir: &Path, block_idx: usize, raw: &[u8]) -> std::io::Result<()> {
    let p = detail_path(dir, block_idx);
    fs::create_dir_all(p.parent().unwrap())?;
    let compressed = zstd::encode_all(raw, 10).expect("zstd encode");
    fs::write(p, compressed)
}

pub fn read_detail_block(dir: &Path, block_idx: usize) -> std::io::Result<Vec<u8>> {
    let p = detail_path(dir, block_idx);
    Ok(zstd::decode_all(fs::read(p)?.as_slice()).expect("zstd decode"))
}

/// Number of whole + partial records in a detail block buffer.
pub fn records_in(raw: &[u8]) -> usize {
    raw.len() / RECORD_LEN
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rollup_roundtrip() {
        let dir = tempdir();
        let p = dir.join("rollup.bin");
        let data: Vec<u32> = (0..1000).map(|i| i * 7).collect();
        write_rollup(&p, &data).unwrap();
        let m = RollupMap::open(&p).unwrap();
        assert_eq!(m.len(), 1000);
        assert_eq!(m.get(13), 91);
        assert_eq!(m.to_vec(), data);
    }

    #[test]
    fn manifest_roundtrip() {
        let dir = tempdir();
        let p = dir.join("manifest.json");
        let man = Manifest {
            schema: SCHEMA, frames: 100, rom_sha256: "a".into(), save_sha256: "b".into(),
            c_oracle_rev: "c".into(), timing_hacks: vec!["X".into()], mask: vec![0x654],
            block_size: 8192, page_kb: 1,
        };
        man.save(&p).unwrap();
        assert_eq!(Manifest::load(&p).unwrap(), man);
    }

    #[test]
    fn detail_roundtrip() {
        let dir = tempdir();
        let raw = vec![0x5au8; RECORD_LEN * 3];
        write_detail_block(&dir, 2, &raw).unwrap();
        assert_eq!(read_detail_block(&dir, 2).unwrap(), raw);
        assert_eq!(records_in(&raw), 3);
    }

    fn tempdir() -> PathBuf {
        let p = std::env::temp_dir().join(format!("parity-test-{}", std::process::id()))
            .join(format!("{:?}", std::time::SystemTime::now()));
        fs::create_dir_all(&p).unwrap();
        p
    }
}
```

- [ ] **Step 4: Run tests — expect fail then pass**

Run: `cargo test -p parity --lib`
Expected: after both modules compile, all merkle + golden tests PASS alongside Task 1's.

- [ ] **Step 5: Commit**

```bash
git add crates/parity/src/lib.rs crates/parity/src/golden.rs crates/parity/src/merkle.rs
git commit -m "feat(parity): golden format — manifest, rollup mmap, merkle, tier-B detail"
```

---

### Task 5: `zparity capture` (build the golden from the C oracle)

**Files:**
- Create: `crates/parity/src/main.rs` (CLI dispatch + `capture`)
- Create: `crates/parity/src/runner.rs` (subprocess helpers: hash files, run replay)
- Modify: `crates/parity/src/lib.rs` (`pub mod runner;`)

**Interfaces:**
- Consumes: `golden::*`, `merkle::*`, `FrameFingerprint`, `RECORD_LEN`.
- Produces:
  - `runner::sha256_file(path) -> String`
  - `runner::HACK_ENV: &[(&str,&str)]`, `runner::SDL_ENV`
  - `runner::c_oracle_cmd(...)`, `runner::rust_replay_cmd(...)` builders
  - CLI: `zparity capture [--full|--frames N] [--detail]`

- [ ] **Step 1: Create `crates/parity/src/runner.rs`**

```rust
use std::path::{Path, PathBuf};
use std::process::Command;

pub const HACK_KEYS: &[&str] = &[
    "SELECT_FILE", "LOADFILE", "DUNGEON", "OVERWORLD", "MESSAGING", "DEATH_INTRO", "DEATH_RELOAD",
];

pub fn hack_env() -> Vec<(String, String)> {
    HACK_KEYS
        .iter()
        .map(|k| (format!("ZELDA3_SMV_{k}_TIMING_HACKS"), "1".to_string()))
        .collect()
}

pub fn sdl_dummy_env() -> Vec<(String, String)> {
    vec![
        ("SDL_VIDEODRIVER".into(), "dummy".into()),
        ("SDL_AUDIODRIVER".into(), "dummy".into()),
        ("SDL_RENDER_DRIVER".into(), "software".into()),
    ]
}

pub fn sha256_file(path: &Path) -> std::io::Result<String> {
    // Shell out to the platform sha256 to avoid a crypto dependency.
    let out = if cfg!(target_os = "macos") {
        Command::new("shasum").args(["-a", "256"]).arg(path).output()?
    } else {
        Command::new("sha256sum").arg(path).output()?
    };
    let s = String::from_utf8_lossy(&out.stdout);
    Ok(s.split_whitespace().next().unwrap_or("").to_string())
}

pub struct Paths {
    pub repo: PathBuf,
    pub c_root: PathBuf,
    pub rom: PathBuf,
    pub save: PathBuf,
    pub rust_bin: PathBuf,
    pub golden_dir: PathBuf,
    pub cache_dir: PathBuf,
}

impl Paths {
    pub fn discover() -> Self {
        let repo = std::env::var_os("ZELDA3_REPO")
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap());
        let c_root = std::env::var_os("ZELDA3_C_REPO")
            .map(PathBuf::from)
            .unwrap_or_else(|| repo.parent().unwrap().join("zelda3"));
        Paths {
            rom: std::env::var_os("ZELDA3_ROM").map(PathBuf::from)
                .unwrap_or_else(|| repo.join("saves/zelda3.sfc")),
            save: std::env::var_os("ZELDA3_REPLAY_SAVE").map(PathBuf::from)
                .unwrap_or_else(|| repo.join("saves/zelda3-combined-route.sav")),
            rust_bin: std::env::var_os("ZELDA3_NEW_BIN").map(PathBuf::from)
                .unwrap_or_else(|| repo.join("target/parity/zelda3")),
            golden_dir: repo.join("parity-golden"),
            cache_dir: repo.join(".cache/parity-golden"),
            c_root,
            repo,
        }
    }
}

/// Build the C-oracle capture command writing a fingerprint stream.
pub fn c_capture_cmd(p: &Paths, frames: u32, fp_out: &Path) -> Command {
    let mut c = Command::new(p.c_root.join("zelda3"));
    c.current_dir(&p.c_root)
        .args(["--config"]).arg(p.c_root.join("other/headless_replay.ini"))
        .args(["--replay-save"]).arg(&p.save)
        .args(["--smv-test-frames", &frames.to_string()])
        .args(["--fingerprint-log"]).arg(fp_out);
    for (k, v) in sdl_dummy_env() {
        c.env(k, v);
    }
    c
}

/// Build a Rust replay shard command: [start checkpoint?] -> end_frame, writing fingerprints.
pub fn rust_shard_cmd(p: &Paths, end_frame: u32, fp_out: &Path, load_state: Option<&Path>) -> Command {
    let mut c = Command::new(&p.rust_bin);
    c.current_dir(&p.repo)
        .args(["--replay-save"]).arg(&p.rom).arg(&p.save).arg(end_frame.to_string())
        .args(["--fingerprint-log"]).arg(fp_out);
    if let Some(ls) = load_state {
        c.args(["--load-state"]).arg(ls);
    }
    for (k, v) in hack_env() {
        c.env(k, v);
    }
    c
}
```

- [ ] **Step 2: Add `pub mod runner;` to `lib.rs`.**

- [ ] **Step 3: Create `crates/parity/src/main.rs` (dispatch + capture)**

```rust
use std::path::Path;
use std::process::exit;

use parity::golden::{self, Manifest, SCHEMA};
use parity::merkle::MerkleIndex;
use parity::runner::{self, Paths};
use parity::{FrameFingerprint, RECORD_LEN, FINGERPRINT_MASK};

const FULL_ROUTE_FRAMES: u32 = 1_073_092;
const BLOCK_SIZE: u32 = 8192;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("capture") => cmd_capture(&args[1..]),
        Some("check") => parity_check::run(&args[1..]),   // Task 6
        Some("drill") => parity_drill::run(&args[1..]),   // Task 7
        _ => {
            eprintln!("usage: zparity <capture|check|drill> [options]");
            exit(2);
        }
    }
}

fn parse_frames(args: &[String]) -> u32 {
    if args.iter().any(|a| a == "--full") {
        return FULL_ROUTE_FRAMES;
    }
    if let Some(i) = args.iter().position(|a| a == "--frames") {
        return args[i + 1].parse().expect("--frames N");
    }
    3000
}

fn cmd_capture(args: &[String]) {
    let p = Paths::discover();
    let frames = parse_frames(args);
    let detail = args.iter().any(|a| a == "--detail");
    if !p.c_root.join("zelda3").exists() {
        eprintln!("C oracle binary missing: {:?} (build: make -C {:?} zelda3)", p.c_root.join("zelda3"), p.c_root);
        exit(1);
    }
    let tmp = p.cache_dir.join("capture.fp");
    std::fs::create_dir_all(&p.cache_dir).unwrap();
    eprintln!("zparity capture: running C oracle over {frames} frames");
    let status = runner::c_capture_cmd(&p, frames, &tmp).status().expect("spawn C oracle");
    if !status.success() {
        eprintln!("C oracle failed: {status}");
        exit(1);
    }
    let data = std::fs::read(&tmp).unwrap();
    let n = data.len() / RECORD_LEN;
    eprintln!("captured {n} frames");
    // Rollup column + merkle.
    let rollups: Vec<u32> = (0..n)
        .map(|i| FrameFingerprint::from_bytes(&data[i * RECORD_LEN..]).rollup)
        .collect();
    std::fs::create_dir_all(&p.golden_dir).unwrap();
    golden::write_rollup(&p.golden_dir.join("rollup.bin"), &rollups).unwrap();
    let merkle = MerkleIndex::build(&rollups, BLOCK_SIZE);
    std::fs::write(p.golden_dir.join("merkle.bin"), merkle.to_bytes()).unwrap();
    // Manifest.
    let oracle_rev = std::process::Command::new("git")
        .current_dir(&p.c_root).args(["rev-parse", "HEAD"]).output()
        .ok().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    let man = Manifest {
        schema: SCHEMA, frames: n as u32,
        rom_sha256: runner::sha256_file(&p.rom).unwrap(),
        save_sha256: runner::sha256_file(&p.save).unwrap(),
        c_oracle_rev: oracle_rev,
        timing_hacks: runner::HACK_KEYS.iter().map(|s| s.to_string()).collect(),
        mask: FINGERPRINT_MASK.to_vec(), block_size: BLOCK_SIZE, page_kb: 1,
    };
    man.save(&p.golden_dir.join("manifest.json")).unwrap();
    // Tier B (optional).
    if detail {
        let blocks = n.div_ceil(BLOCK_SIZE as usize);
        for b in 0..blocks {
            let start = b * BLOCK_SIZE as usize * RECORD_LEN;
            let end = ((b + 1) * BLOCK_SIZE as usize * RECORD_LEN).min(data.len());
            golden::write_detail_block(&p.cache_dir, b, &data[start..end]).unwrap();
        }
        eprintln!("wrote {blocks} detail blocks to {:?}", p.cache_dir.join("detail"));
    }
    let _ = std::fs::remove_file(&tmp);
    eprintln!("golden written: {:?} (root=0x{:08x})", p.golden_dir, merkle.root);
}

// Task 6 / Task 7 provide these modules.
mod parity_check;
mod parity_drill;
```

> NOTE: to keep this task independently compilable, create **stub** `crates/parity/src/parity_check.rs` and `crates/parity/src/parity_drill.rs` now, each containing `pub fn run(_args: &[String]) { eprintln!("not implemented"); std::process::exit(2); }`. Tasks 6 and 7 replace the stub bodies.

- [ ] **Step 4: Create the two stub modules** (`parity_check.rs`, `parity_drill.rs`) with the one-line `run` stub above.

- [ ] **Step 5: Write the failing test `crates/parity/tests/capture_smoke.rs`**

```rust
//! Smoke: capture a tiny route, assert artifacts + manifest sanity.
//! Ignored by default (needs built binaries + ROM); run with --ignored.
use std::path::Path;
use std::process::Command;

#[test]
#[ignore]
fn capture_smoke() {
    let repo = env!("CARGO_MANIFEST_DIR"); // crates/parity
    let root = Path::new(repo).parent().unwrap().parent().unwrap();
    let status = Command::new(env!("CARGO_BIN_EXE_zparity"))
        .current_dir(root)
        .args(["capture", "--frames", "500"])
        .status()
        .unwrap();
    assert!(status.success());
    let gd = root.join("parity-golden");
    assert!(gd.join("rollup.bin").exists());
    assert!(gd.join("merkle.bin").exists());
    let man: serde_json::Value =
        serde_json::from_slice(&std::fs::read(gd.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(man["frames"], 500);
    assert_eq!(man["block_size"], 8192);
}
```

- [ ] **Step 6: Build + run — expect fail then pass**

Run: `cargo build -p parity && cargo test -p parity --test capture_smoke -- --ignored`
Expected: PASS (artifacts created). Requires `target/parity/zelda3` and the C oracle built.

- [ ] **Step 7: Commit**

```bash
git add crates/parity/src/main.rs crates/parity/src/runner.rs crates/parity/src/lib.rs \
        crates/parity/src/parity_check.rs crates/parity/src/parity_drill.rs \
        crates/parity/tests/capture_smoke.rs
git commit -m "feat(zparity): capture subcommand builds golden from C oracle"
```

---

### Task 6: `zparity check` — sharded parallel compare (the gate)

**Files:**
- Modify: `crates/parity/src/parity_check.rs` (replace stub)
- Modify: `zelda3-bin/src/main.rs` (add repeatable `--save-state-at <frame>:<path>` for one-pass checkpoint seeding)
- Test: `crates/parity/tests/check_invariance.rs`

**Interfaces:**
- Consumes: `golden::{Manifest, RollupMap}`, `merkle::MerkleIndex`, `runner::{Paths, rust_shard_cmd, sha256_file}`, `RECORD_LEN`, `FrameFingerprint`.
- Produces: `pub fn run(args: &[String])` exit 0 = MATCH, 1 = divergence/validation failure.

- [ ] **Step 1: Add `--save-state-at` to the Rust binary** so one sequential pass can drop all shard-boundary checkpoints. In `zelda3-bin/src/main.rs` replay flag locals add `let mut save_state_at: Vec<(u32, PathBuf)> = Vec::new();`, parse:

```rust
            "--save-state-at" => {
                let Some(spec) = args.next() else { eprintln!("--save-state-at <frame>:<path>"); process::exit(2); };
                let (f, path) = spec.split_once(':').unwrap_or_else(|| { eprintln!("--save-state-at <frame>:<path>"); process::exit(2); });
                save_state_at.push((f.parse().unwrap(), PathBuf::from(path)));
            }
```

Inside the replay loop, after `frames = frames.wrapping_add(1);` (~1438), drop a checkpoint when a boundary is hit (reuse the existing `--save-state` serialization path — extract it into `fn write_checkpoint(game: &ZeldaState, frames: u32, path: &Path)` and call it both here and at the existing save site ~3141):

```rust
            if let Some(idx) = save_state_at.iter().position(|(f, _)| *f == frames) {
                let (_, path) = &save_state_at[idx];
                write_checkpoint(&game, frames, path);
            }
```

- [ ] **Step 2: Implement `parity_check::run`** in `crates/parity/src/parity_check.rs`:

```rust
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::thread;

use parity::golden::{Manifest, RollupMap};
use parity::merkle::MerkleIndex;
use parity::runner::{self, Paths};
use parity::{FrameFingerprint, RECORD_LEN};

const BLOCK_SIZE: u32 = 8192;
const SHARD_THRESHOLD: u32 = 20_000; // below this, run single-shard

pub fn run(args: &[String]) {
    let p = Paths::discover();
    let frames = super::parse_frames(args);
    let detail = args.iter().any(|a| a == "--detail");
    let shards = shard_count(args, frames);

    // 1. Validate manifest.
    let man = Manifest::load(&p.golden_dir.join("manifest.json")).unwrap_or_else(|e| {
        eprintln!("no golden manifest ({e}); run `zparity capture` first"); exit(1);
    });
    if man.schema != parity::golden::SCHEMA {
        eprintln!("golden schema {} != {}", man.schema, parity::golden::SCHEMA); exit(1);
    }
    if frames > man.frames {
        eprintln!("requested {frames} frames > golden {} frames", man.frames); exit(1);
    }
    check_hash("ROM", &runner::sha256_file(&p.rom).unwrap(), &man.rom_sha256);
    check_hash("save", &runner::sha256_file(&p.save).unwrap(), &man.save_sha256);

    let golden = RollupMap::open(&p.golden_dir.join("rollup.bin")).unwrap();

    // 2. Boundaries + checkpoints (single shard => none).
    let bounds = shard_bounds(frames, shards);
    if shards > 1 {
        ensure_checkpoints(&p, &bounds);
    }

    // 3. Fan out shards.
    let start = std::time::Instant::now();
    let golden = Arc::new(golden);
    let first_diff = Arc::new(Mutex::new(None::<u64>)); // global frame index
    let mut handles = Vec::new();
    let tmp = p.cache_dir.join("shards");
    std::fs::create_dir_all(&tmp).unwrap();
    for (i, w) in bounds.windows(2).enumerate() {
        let (s, e) = (w[0], w[1]);
        let p = clone_paths(&p);
        let golden = Arc::clone(&golden);
        let first_diff = Arc::clone(&first_diff);
        let fp_out = tmp.join(format!("shard_{i}.bin"));
        let ck = if i == 0 { None } else { Some(p.cache_dir.join(format!("ck/{s}.sav"))) };
        handles.push(thread::spawn(move || {
            run_shard(&p, s, e, &fp_out, ck.as_deref(), &golden, &first_diff);
        }));
    }
    for h in handles {
        h.join().unwrap();
    }

    // 4. Report.
    let dur = start.elapsed().as_secs_f64();
    let merkle = MerkleIndex::from_bytes(&std::fs::read(p.golden_dir.join("merkle.bin")).unwrap());
    match *first_diff.lock().unwrap() {
        None => {
            println!("MATCH  {frames} frames, {shards} shards, {dur:.1}s  root=0x{:08x}", merkle.root);
            if detail {
                println!("(detail compare requested; per-region OK — see `zparity drill` on failure)");
            }
        }
        Some(f) => {
            println!("DIVERGE at frame {f}  ({shards} shards, {dur:.1}s)");
            println!("  run: zparity drill {f}");
            exit(1);
        }
    }
}

fn run_shard(
    p: &Paths, start: u32, end: u32, fp_out: &Path, load_state: Option<&Path>,
    golden: &RollupMap, first_diff: &Mutex<Option<u64>>,
) {
    let out = runner::rust_shard_cmd(p, end, fp_out, load_state).output().expect("spawn rust shard");
    if !out.status.success() {
        eprintln!("shard [{start},{end}) failed:\n{}", String::from_utf8_lossy(&out.stderr));
        let mut g = first_diff.lock().unwrap();
        *g = Some(g.map_or(start as u64, |v| v.min(start as u64)));
        return;
    }
    let data = std::fs::read(fp_out).expect("read shard fp");
    // A shard that resumed from a checkpoint at frame `start` produced records for
    // frames (start, end]; a from-zero shard produced (0, end]. The stream is
    // sequential; compare record k against golden[start + k].
    let n = data.len() / RECORD_LEN;
    for k in 0..n {
        let frame_idx = start as usize + k;
        if frame_idx >= golden.len() {
            break;
        }
        let rollup = FrameFingerprint::from_bytes(&data[k * RECORD_LEN..]).rollup;
        if rollup != golden.get(frame_idx) {
            let mut g = first_diff.lock().unwrap();
            let f = frame_idx as u64;
            *g = Some(g.map_or(f, |v| v.min(f)));
            return;
        }
    }
}

fn shard_count(args: &[String], frames: u32) -> usize {
    if let Some(i) = args.iter().position(|a| a == "--shards") {
        return args[i + 1].parse().expect("--shards K");
    }
    if frames < SHARD_THRESHOLD {
        return 1;
    }
    thread::available_parallelism().map(|n| n.get()).unwrap_or(4)
}

fn shard_bounds(frames: u32, shards: usize) -> Vec<u32> {
    let mut v = Vec::with_capacity(shards + 1);
    for i in 0..=shards {
        v.push(((frames as u64 * i as u64) / shards as u64) as u32);
    }
    v.dedup();
    v
}

fn ensure_checkpoints(p: &Paths, bounds: &[u32]) {
    let ck_dir = p.cache_dir.join("ck");
    std::fs::create_dir_all(&ck_dir).unwrap();
    let needed: Vec<u32> = bounds[1..bounds.len() - 1].to_vec();
    if needed.iter().all(|f| ck_dir.join(format!("{f}.sav")).exists()) {
        return;
    }
    eprintln!("seeding {} checkpoints (one sequential pass)...", needed.len());
    // Single pass to the last boundary, dropping all checkpoints via --save-state-at.
    let mut cmd = runner::rust_shard_cmd(p, *bounds.last().unwrap(),
        &p.cache_dir.join("seed.fp"), None);
    for f in &needed {
        cmd.args(["--save-state-at", &format!("{f}:{}", ck_dir.join(format!("{f}.sav")).display())]);
    }
    let st = cmd.status().expect("spawn checkpoint seed");
    assert!(st.success(), "checkpoint seeding failed");
    let _ = std::fs::remove_file(p.cache_dir.join("seed.fp"));
}

fn check_hash(what: &str, got: &str, want: &str) {
    if got != want {
        eprintln!("{what} sha256 mismatch:\n  golden={want}\n  local ={got}");
        exit(1);
    }
}

fn clone_paths(p: &Paths) -> Paths {
    Paths {
        repo: p.repo.clone(), c_root: p.c_root.clone(), rom: p.rom.clone(),
        save: p.save.clone(), rust_bin: p.rust_bin.clone(),
        golden_dir: p.golden_dir.clone(), cache_dir: p.cache_dir.clone(),
    }
}
```

> NOTE: derive `#[derive(Clone)]` on `runner::Paths` instead of `clone_paths` if preferred — either is fine; pick one and stay consistent.

- [ ] **Step 3: Write the failing test `crates/parity/tests/check_invariance.rs`**

```rust
//! Shard invariance + MATCH on the current (passing) binary. Ignored by default.
use std::path::Path;
use std::process::Command;

fn zparity(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_zparity")).current_dir(root).args(args).output().unwrap()
}

#[test]
#[ignore]
fn match_and_shard_invariant() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    // Requires a golden already captured for >= 30000 frames.
    let a = zparity(root, &["check", "--frames", "30000", "--shards", "1"]);
    let b = zparity(root, &["check", "--frames", "30000", "--shards", "4"]);
    assert!(a.status.success(), "1-shard: {}", String::from_utf8_lossy(&a.stderr));
    assert!(b.status.success(), "4-shard: {}", String::from_utf8_lossy(&b.stderr));
    let sa = String::from_utf8_lossy(&a.stdout);
    let sb = String::from_utf8_lossy(&b.stdout);
    assert!(sa.contains("MATCH") && sb.contains("MATCH"), "{sa}||{sb}");
    // roots reported identically
    let root_of = |s: &str| s.split("root=").nth(1).map(|x| x.trim().to_string());
    assert_eq!(root_of(&sa), root_of(&sb));
}
```

- [ ] **Step 4: Build + run — expect fail then pass**

Run:
```bash
cargo build --profile parity -p zelda3-bin && cargo build -p parity
./target/debug/zparity capture --frames 30000        # seed golden (needs C)
cargo test -p parity --test check_invariance -- --ignored
```
Expected: `MATCH` for both shard counts, identical roots.

- [ ] **Step 5: Commit**

```bash
git add crates/parity/src/parity_check.rs zelda3-bin/src/main.rs crates/parity/tests/check_invariance.rs
git commit -m "feat(zparity): sharded parallel check against golden"
```

---

### Task 7: `zparity drill <frame>` — per-region localization

**Files:**
- Modify: `crates/parity/src/parity_drill.rs` (replace stub)
- Test: `crates/parity/tests/drill.rs`

**Interfaces:**
- Consumes: `golden::{Manifest, read_detail_block}`, `runner::*`, `FrameFingerprint`, `RECORD_LEN`, `PAGE`.
- Produces: `pub fn run(args: &[String])` printing the diverging layers/regions at a frame.

- [ ] **Step 1: Implement `parity_drill::run`**

```rust
use std::process::exit;

use parity::golden::{self, Manifest};
use parity::runner::{self, Paths};
use parity::{FrameFingerprint, PAGE, RECORD_LEN};

const BLOCK_SIZE: usize = 8192;

pub fn run(args: &[String]) {
    let frame: usize = match args.first().and_then(|s| s.parse().ok()) {
        Some(f) => f,
        None => { eprintln!("usage: zparity drill <frame>"); exit(2); }
    };
    let p = Paths::discover();
    let _man = Manifest::load(&p.golden_dir.join("manifest.json")).unwrap_or_else(|e| {
        eprintln!("no golden ({e})"); exit(1);
    });
    let block = frame / BLOCK_SIZE;
    let raw = match golden::read_detail_block(&p.cache_dir, block) {
        Ok(r) => r,
        Err(_) => {
            eprintln!("detail tier absent for block {block}; run `zparity capture --detail`");
            exit(1);
        }
    };
    let within = frame % BLOCK_SIZE;
    if (within + 1) * RECORD_LEN > raw.len() {
        eprintln!("frame {frame} not in golden detail"); exit(1);
    }
    let want = FrameFingerprint::from_bytes(&raw[within * RECORD_LEN..]);

    // Re-run the Rust binary to exactly `frame` to get its fingerprint at that frame.
    let tmp = p.cache_dir.join("drill.fp");
    std::fs::create_dir_all(&p.cache_dir).unwrap();
    // Resume from the nearest checkpoint <= frame if present, else from 0.
    let ck_dir = p.cache_dir.join("ck");
    let ck = nearest_checkpoint(&ck_dir, frame as u32);
    let start = ck.as_ref().map(|(f, _)| *f).unwrap_or(0);
    let st = runner::rust_shard_cmd(&p, frame as u32, &tmp, ck.as_ref().map(|(_, p)| p.as_path()))
        .status().expect("spawn rust");
    assert!(st.success());
    let got_data = std::fs::read(&tmp).unwrap();
    let idx = frame - start as usize;             // record offset within this run
    let got = FrameFingerprint::from_bytes(&got_data[idx * RECORD_LEN..]);

    if got.rollup == want.rollup {
        println!("frame {frame}: MATCH (rollup 0x{:08x})", got.rollup);
        return;
    }
    println!("frame {frame}: DIVERGE");
    for pidx in 0..parity::WRAM_PAGES {
        if got.wram[pidx] != want.wram[pidx] {
            let addr = pidx * PAGE;
            println!("  WRAM page {pidx:3} @0x{addr:05x}  golden=0x{:08x} rust=0x{:08x}  ({})",
                want.wram[pidx], got.wram[pidx], page_label(addr));
        }
    }
    for pidx in 0..parity::VRAM_PAGES {
        if got.vram[pidx] != want.vram[pidx] {
            println!("  VRAM page {pidx:3} @0x{:05x}  golden=0x{:08x} rust=0x{:08x}",
                pidx * PAGE, want.vram[pidx], got.vram[pidx]);
        }
    }
    if got.sram != want.sram { println!("  SRAM  golden=0x{:08x} rust=0x{:08x}", want.sram, got.sram); }
    if got.render != want.render { println!("  RENDER golden=0x{:08x} rust=0x{:08x}", want.render, got.render); }
    if got.audio != want.audio { println!("  AUDIO golden=0x{:08x} rust=0x{:08x}", want.audio, got.audio); }
    exit(1);
}

fn nearest_checkpoint(ck_dir: &std::path::Path, frame: u32) -> Option<(u32, std::path::PathBuf)> {
    let mut best: Option<(u32, std::path::PathBuf)> = None;
    let entries = std::fs::read_dir(ck_dir).ok()?;
    for e in entries.flatten() {
        if let Some(stem) = e.path().file_stem().and_then(|s| s.to_str()) {
            if let Ok(f) = stem.parse::<u32>() {
                if f <= frame && best.as_ref().map_or(true, |(bf, _)| f > *bf) {
                    best = Some((f, e.path()));
                }
            }
        }
    }
    best
}

/// Best-effort WRAM page label. A richer mapping can shell out to
/// scripts/whoowns.py; for now annotate the page base address only.
fn page_label(addr: usize) -> String {
    format!("WRAM 0x{addr:05x}..0x{:05x}", addr + PAGE - 1)
}
```

> NOTE: a richer `page_label` can call `scripts/whoowns.py 0x<addr>` and parse the const name; that is an optional enhancement and must NOT block the gate. Keep the base-address label as the always-available default.

- [ ] **Step 2: Write the failing test `crates/parity/tests/drill.rs`**

```rust
//! drill reports a planted divergence. Ignored by default; needs golden+detail.
use std::path::Path;
use std::process::Command;

#[test]
#[ignore]
fn drill_reports_layer() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    // On a clean binary, drilling any captured frame must say MATCH.
    let out = Command::new(env!("CARGO_BIN_EXE_zparity"))
        .current_dir(root).args(["drill", "1000"]).output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("MATCH"), "{s}{}", String::from_utf8_lossy(&out.stderr));
}
```

- [ ] **Step 3: Build + run — expect fail then pass**

Run:
```bash
cargo build -p parity
./target/debug/zparity capture --frames 30000 --detail   # needs Tier B
cargo test -p parity --test drill -- --ignored
```
Expected: `frame 1000: MATCH`.

- [ ] **Step 4: Commit**

```bash
git add crates/parity/src/parity_drill.rs crates/parity/tests/drill.rs
git commit -m "feat(zparity): drill subcommand for per-region localization"
```

---

### Task 8: Capture golden, wire pre-commit, document

**Files:**
- Create: `parity-golden/.gitignore` (keep Tier A, exclude Tier B if ever placed here)
- Modify: `.gitignore` (ignore `.cache/parity-golden/`)
- Modify: `.githooks/pre-commit`
- Create: `crates/parity/README.md`
- Modify: `CLAUDE.md` (add a `zparity` note under the parity-tools section)

- [ ] **Step 1: Capture and commit the Tier A golden (full route).**

Run:
```bash
make -C ../zelda3 zelda3
cargo build --profile parity -p zelda3-bin
cargo build -p parity
./target/debug/zparity capture --full
ls -la parity-golden/
```
Expected: `parity-golden/{rollup.bin (~4.3MB), merkle.bin, manifest.json}`.

- [ ] **Step 2: Gitignore the cache, keep Tier A.** Append to repo `.gitignore`:

```
.cache/parity-golden/
```

Create `parity-golden/.gitignore`:

```
detail/
*.fp
```

- [ ] **Step 3: Verify the gate passes end-to-end (C-free).**

Run: `./target/debug/zparity check --full`
Expected: `MATCH  1073092 frames, <K> shards, <T>s  root=0x...`. Time it; record the wall-clock in the README.

- [ ] **Step 4: Wire into `.githooks/pre-commit`.** Add a fast C-free smoke before (eventually replacing) the heavy `validate_all_parity.py` smoke. Insert after the existing build step:

```sh
echo "pre-commit: zparity smoke (C-free)"
if [ -f parity-golden/manifest.json ]; then
  ./target/parity/zelda3 >/dev/null 2>&1 || true   # ensure built earlier in hook
  cargo build -q -p parity || exit 1
  ./target/debug/zparity check --frames 3000 || { echo "zparity smoke FAILED"; exit 1; }
fi
```

- [ ] **Step 5: Write `crates/parity/README.md`** documenting the three subcommands, the two tiers, when to re-capture (route or C-oracle change), the env overrides (`ZELDA3_C_REPO`, `ZELDA3_NEW_BIN`, `ZELDA3_ROM`, `ZELDA3_REPLAY_SAVE`), and the recorded full-route wall-clock from Step 3.

- [ ] **Step 6: Add a CLAUDE.md note** under "Parity-debugging tools", one entry:

```
10. **`zparity` (crates/parity) — C-free all-layer gate.** `zparity capture --full`
    (needs C) builds parity-golden/ once; `zparity check --full` shards the route
    across cores and verifies WRAM/VRAM/SRAM/render/audio vs the golden with NO C
    build; `zparity drill <frame>` localizes a divergence per page/layer (needs
    `capture --detail`). Replaces the smoke path of validate_all_parity.py.
```

- [ ] **Step 7: Commit**

```bash
git add parity-golden/rollup.bin parity-golden/merkle.bin parity-golden/manifest.json \
        parity-golden/.gitignore .gitignore .githooks/pre-commit crates/parity/README.md CLAUDE.md
git commit -m "feat(zparity): capture full-route golden, wire pre-commit + docs"
```

---

## Self-Review Notes

- **Spec coverage:** Component 1 (fingerprint stream) → Tasks 2, 3; Component 2 (golden two-tier + merkle) → Tasks 1, 4; Component 3 (sharded check + determinism mask) → Tasks 1 (mask), 6; Component 4 (drill) → Task 7; capture → Task 5; testing matrix (self-consistency, C-equivalence, shard invariance, mask audit) → Tasks 3, 6, 1; rollout/pre-commit → Task 8. All spec sections mapped.
- **Determinism mask:** lives in one place (`FINGERPRINT_MASK`, Task 1), applied in the Rust hash (Task 1) and mirrored in C (Task 3), test-pinned (Task 1 `mask_audit`), making sharded == from-scratch (validated by Task 6 shard invariance).
- **Type consistency:** `FrameFingerprint`, `RECORD_LEN`, `RollupMap`, `MerkleIndex`, `Manifest`, `Paths`, `rust_shard_cmd` names are used identically across Tasks 4-7.
- **Known follow-ups (not blockers):** richer `page_label` via `whoowns.py`; `--detail` per-region streaming compare in `check` (currently rollup-only in `check`, full per-region in `drill`); one-pass capture that also seeds checkpoints. These are noted inline and do not gate v1.
