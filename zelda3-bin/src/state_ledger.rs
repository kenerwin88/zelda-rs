//! Streaming, GPU-independent Rust regression ledgers.
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use zelda3::ZeldaState;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Record {
    #[serde(deserialize_with = "record_schema")]
    schema: u32,
    frame: u32,
    input: u16,
    lag_free: bool,
    state: String,
    display: String,
    sram: String,
    audio: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    rgb: Option<String>,
    main: u8,
    sub: u8,
    room: u16,
}

fn record_schema<'de, D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<u32, D::Error> {
    let schema = u32::deserialize(deserializer)?;
    if schema != 3 {
        return Err(serde::de::Error::custom(
            "incompatible state ledger: schema 3 / zelda3-state-v2 required; re-record v1 baselines",
        ));
    }
    Ok(schema)
}

impl Record {
    pub(crate) fn capture(
        frame: u32,
        input: u16,
        game: &ZeldaState,
        audio: &[i16],
        display: u64,
    ) -> Self {
        // Hash the sample wire representation, never native-endian memory.
        let audio = audio.iter().flat_map(|sample| sample.to_le_bytes());
        let image = game.wram_image();
        Self {
            schema: 3,
            frame,
            input,
            lag_free: game.lag_free(),
            state: format!("{:016x}", game.canonical_state_hash()),
            display: format!("{display:016x}"),
            sram: format!("{:016x}", super::fnv1a64(&game.sram)),
            audio: format!("{:016x}", hash_bytes(audio)),
            rgb: None,
            main: image[0x10],
            sub: image[0x11],
            room: u16::from_le_bytes([image[0xa0], image[0xa1]]),
        }
    }

    fn differences(&self, other: &Self) -> Vec<&'static str> {
        let mut fields = Vec::new();
        macro_rules! check { ($($field:ident),*) => { $(
            if self.$field != other.$field { fields.push(stringify!($field)); }
        )* }; }
        check!(schema, frame, input, lag_free, state, display, sram, audio, rgb, main, sub, room);
        fields
    }
}

fn hash_bytes(bytes: impl IntoIterator<Item = u8>) -> u64 {
    bytes.into_iter().fold(0xcbf29ce484222325, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
    })
}

// Hash the already-published frame, not a second mutable display snapshot or
// renderer scratch buffers. Explicit bincode fields give fixed LE wire bytes.
fn display_hash(
    ppu: &snes::ppu::PpuState,
    cgram: &[u16],
    scanlines: &[renderer::gpu_frame::RawScanlineRegs],
) -> u64 {
    struct Digest(u64);
    impl Write for Digest {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            for &byte in bytes {
                self.0 = (self.0 ^ u64::from(byte)).wrapping_mul(0x100000001b3);
            }
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut digest = Digest(hash_bytes(*b"zelda3-display-v1"));
    macro_rules! fields { ($($value:expr),* $(,)?) => { $(
        bincode::serialize_into(&mut digest, &$value).expect("display hash serialization");
    )* }; }
    fields!(
        ppu.vram,
        ppu.cgram,
        cgram,
        ppu.oam,
        ppu.bg_vram_latch,
        ppu.obj_vram_latch,
        ppu.obj_previous_frame_vram,
        ppu.screen_enabled,
        ppu.screen_windowed,
        ppu.mosaic_enabled,
        ppu.mosaic_size,
        ppu.obj_tile_adr1,
        ppu.obj_tile_adr2,
        ppu.obj_size,
        ppu.window1_left,
        ppu.window1_right,
        ppu.window2_left,
        ppu.window2_right,
        ppu.windowsel,
        ppu.clip_mode,
        ppu.prevent_math_mode,
        ppu.add_subscreen,
        ppu.subtract_color,
        ppu.half_color,
        ppu.math_enabled,
        ppu.fixed_color_r,
        ppu.fixed_color_g,
        ppu.fixed_color_b,
        ppu.forced_blank,
        ppu.forced_blank_scanlines,
        ppu.forced_blank_from_scanline,
        ppu.retain_active_display_history,
        ppu.brightness,
        ppu.scanout_brightness_override,
        ppu.scanout_top_crop,
        ppu.mode,
        ppu.bg_layer,
        ppu.m7_matrix,
        ppu.m7_large_field,
        ppu.m7_char_fill,
        ppu.m7_x_flip,
        ppu.m7_y_flip,
        ppu.m7_ext_bg_always_zero,
        ppu.m7_start_x,
        ppu.m7_start_y,
        scanlines,
    );
    digest.0
}

fn reader(path: &str) -> Result<Box<dyn BufRead>> {
    let file = File::open(path)?;
    Ok(if path.ends_with(".zst") {
        Box::new(BufReader::new(zstd::stream::read::Decoder::new(file)?))
    } else {
        Box::new(BufReader::new(file))
    })
}

enum Output {
    Plain(BufWriter<File>),
    Zstd(zstd::stream::write::Encoder<'static, BufWriter<File>>),
}
impl Write for Output {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Plain(w) => w.write(bytes),
            Self::Zstd(w) => w.write(bytes),
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Plain(w) => w.flush(),
            Self::Zstd(w) => w.flush(),
        }
    }
}
impl Output {
    fn finish(self) -> std::io::Result<()> {
        match self {
            Self::Plain(mut w) => w.flush(),
            Self::Zstd(w) => w.finish()?.flush(),
        }
    }
}

pub(crate) fn route_inputs(path: &Path, frames: u32) -> Result<Vec<u16>> {
    #[derive(Deserialize)]
    struct Input {
        frame: usize,
        input: String,
    }
    let mut inputs = Vec::new();
    for line in BufReader::new(File::open(path)?).lines() {
        let row: Input = serde_json::from_str(&line?)?;
        if row.frame != inputs.len() {
            return Err(format!("noncontiguous route input at frame {}", inputs.len()).into());
        }
        inputs.push(u16::from_str_radix(row.input.trim_start_matches("0x"), 16)?);
    }
    if inputs.len() < frames as usize {
        return Err(format!("route has {} inputs, requested {frames}", inputs.len()).into());
    }
    Ok(inputs)
}

// This CPU diagnostic renderer is optional: full per-frame pixel rendering
// costs much more than hashing state. It never constructs GPU resources.
struct SoftwareRgb {
    sources: renderer::modern_source_atlas::ModernSourceAtlas,
    variants: renderer::modern_variant_atlas::ModernVariantAtlas,
}
impl SoftwareRgb {
    fn load() -> Result<Self> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        Ok(Self {
            sources: renderer::modern_source_atlas::load_modern_source_atlas(root)?,
            variants: renderer::modern_variant_atlas::load_modern_canonical_art_atlas(root)?,
        })
    }
    fn hash(&self, capture: &crate::gpu_capture::LiveGpuFrameCapture) -> String {
        let gpu_frame = capture.gpu_frame(); // A borrowed data descriptor, not a GPU device.
        let table = renderer::source_table_from_entries(capture.source_entries());
        let assets = renderer::modern_extract::extract_asset_resolved_modern_frame_from_sources(
            &gpu_frame,
            &table,
            &self.sources,
        );
        let scene =
            renderer::ModernAssetFrameScene::from_player_indoors_flag(capture.player_indoors());
        let (rgba, _) = renderer::modern_software::render_modern_frame_software_variant_atlas(
            &assets.frame,
            &assets.bg_cells,
            &assets.sprite_cells,
            &self.variants,
            scene.bg_palette_name(),
            scene.sprite_palette_name(),
        );
        format!(
            "{:016x}",
            hash_bytes(rgba.chunks_exact(4).flat_map(|p| p[..3].iter().copied()))
        )
    }
}

pub(crate) struct Ledger {
    pub(crate) interval: u32,
    rgb: Option<SoftwareRgb>,
    output: Option<Output>,
    baseline: Option<Box<dyn BufRead>>,
}

impl Ledger {
    pub(crate) fn open(args: &[String]) -> Result<Option<Self>> {
        let mut out = None;
        let mut baseline = None;
        let mut interval = 1;
        let mut rgb = false;
        let mut i = 0;
        while i < args.len() {
            if args[i] == "--ledger-rgb" && !rgb {
                rgb = true;
                i += 1;
                continue;
            }
            let value = args.get(i + 1).ok_or("missing ledger option value")?;
            match args[i].as_str() {
                "--state-ledger" if out.is_none() => out = Some(value.as_str()),
                "--compare-state-ledger" if baseline.is_none() => baseline = Some(value.as_str()),
                "--ledger-interval" => interval = value.parse::<u32>()?,
                flag => return Err(format!("unknown/duplicate smoke option {flag}").into()),
            }
            i += 2;
        }
        if interval == 0 {
            return Err("ledger interval must be positive".into());
        }
        if out.is_none() && baseline.is_none() {
            if !args.is_empty() {
                return Err("interval requires a ledger action".into());
            }
            return Ok(None);
        }
        // Open the baseline first and reject aliases before creating any output.
        let baseline_reader = baseline.map(reader).transpose()?;
        if let (Some(out), Some(base)) = (out, baseline) {
            if Path::new(out).exists()
                && std::fs::canonicalize(out)? == std::fs::canonicalize(base)?
            {
                return Err("output must differ from baseline".into());
            }
        }
        let output = out
            .map(|path| -> Result<Output> {
                // Never overwrite an existing baseline (including hard-link aliases).
                let file = File::options().write(true).create_new(true).open(path)?;
                Ok(if path.ends_with(".zst") {
                    Output::Zstd(zstd::stream::write::Encoder::new(BufWriter::new(file), 3)?)
                } else {
                    Output::Plain(BufWriter::new(file))
                })
            })
            .transpose()?;
        Ok(Some(Self {
            interval,
            rgb: if rgb {
                Some(SoftwareRgb::load()?)
            } else {
                None
            },
            output,
            baseline: baseline_reader,
        }))
    }

    pub(crate) fn capture(
        &self,
        frame: u32,
        input: u16,
        game: &ZeldaState,
        audio: &[i16],
        display: &crate::gpu_capture::LiveGpuFrameCapture,
    ) -> Record {
        let hash = display_hash(
            display.presented_ppu(),
            display.cgram(),
            display.presented_scanlines(),
        );
        let mut record = Record::capture(frame, input, game, audio, hash);
        record.rgb = self.rgb.as_ref().map(|rgb| rgb.hash(display));
        record
    }

    pub(crate) fn observe(&mut self, actual: &Record) -> Result<()> {
        if let Some(baseline) = &mut self.baseline {
            let mut line = String::new();
            if baseline.read_line(&mut line)? == 0 {
                return Err(format!("state ledger divergence frame={} fields=missing_record main={:#04x} sub={:#04x} room={:#06x}", actual.frame, actual.main, actual.sub, actual.room).into());
            }
            let expected: Record = serde_json::from_str(&line)?;
            let fields = actual.differences(&expected);
            if !fields.is_empty() {
                return Err(format!("state ledger divergence frame={} fields={} main={:#04x} sub={:#04x} room={:#06x}\nexpected={expected:?}\nactual={actual:?}", actual.frame, fields.join(","), actual.main, actual.sub, actual.room).into());
            }
        }
        if let Some(output) = &mut self.output {
            serde_json::to_writer(&mut *output, actual)?;
            output.write_all(b"\n")?;
        }
        Ok(())
    }

    pub(crate) fn finish(mut self) -> Result<()> {
        if let Some(baseline) = &mut self.baseline {
            let mut line = String::new();
            if baseline.read_line(&mut line)? != 0 {
                let extra: Record = serde_json::from_str(&line)?;
                return Err(format!("state ledger divergence frame={} fields=extra_record main={:#04x} sub={:#04x} room={:#06x} (baseline has extra records after replay end)", extra.frame, extra.main, extra.sub, extra.room).into());
            }
        }
        if let Some(output) = self.output.take() {
            output.finish()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fnv_wire_vectors() {
        assert_eq!(hash_bytes([]), 0xcbf29ce484222325);
        assert_eq!(hash_bytes(*b"hello"), 0xa430d84680aabd0b);
        assert_eq!(
            hash_bytes([-2i16, 1].iter().flat_map(|s| s.to_le_bytes())),
            hash_bytes([254, 255, 1, 0])
        );
    }
    #[test]
    fn display_hash_covers_memories_registers_and_scanlines() {
        let game = ZeldaState::new();
        let ppu = &game.ppu;
        let initial = display_hash(ppu, &ppu.cgram, &[]);
        macro_rules! changed { ($($change:expr),* $(,)?) => { $(
            let mut altered = ppu.clone();
            ($change)(&mut altered);
            assert_ne!(initial, display_hash(&altered, &ppu.cgram, &[]), stringify!($change));
            // Display coverage must not change the canonical state definition.
        )* }; }
        changed!(
            |p: &mut snes::ppu::PpuState| p.vram[17] ^= 1,
            |p: &mut snes::ppu::PpuState| p.cgram[17] ^= 1,
            |p: &mut snes::ppu::PpuState| p.oam[17] ^= 1,
            |p: &mut snes::ppu::PpuState| p.oam[0x100] ^= 1,
            |p: &mut snes::ppu::PpuState| p.obj_vram_latch = Some(vec![0; 0x8000]),
            |p: &mut snes::ppu::PpuState| p.bg_vram_latch = Some(vec![0; 0x8000]),
            |p: &mut snes::ppu::PpuState| p.obj_previous_frame_vram = Some(vec![0; 0x8000]),
            |p: &mut snes::ppu::PpuState| p.mode ^= 1,
            |p: &mut snes::ppu::PpuState| p.bg_layer[0].h_scroll ^= 1,
            |p: &mut snes::ppu::PpuState| p.bg_layer[3].v_scroll ^= 1,
            |p: &mut snes::ppu::PpuState| p.bg_layer[1].tile_adr ^= 1,
            |p: &mut snes::ppu::PpuState| p.screen_enabled[0] ^= 1,
            |p: &mut snes::ppu::PpuState| p.screen_windowed[1] ^= 1,
            |p: &mut snes::ppu::PpuState| p.window1_left ^= 1,
            |p: &mut snes::ppu::PpuState| p.windowsel ^= 1,
            |p: &mut snes::ppu::PpuState| p.math_enabled ^= 1,
            |p: &mut snes::ppu::PpuState| p.fixed_color_r ^= 1,
            |p: &mut snes::ppu::PpuState| p.brightness ^= 1,
            |p: &mut snes::ppu::PpuState| p.forced_blank = !p.forced_blank,
            |p: &mut snes::ppu::PpuState| p.forced_blank_from_scanline = Some(32),
            |p: &mut snes::ppu::PpuState| p.mosaic_size ^= 1,
            |p: &mut snes::ppu::PpuState| p.m7_matrix[0] ^= 1,
        );
        let mut latched = ppu.clone();
        latched.obj_vram_latch = Some(vec![0; 0x8000]);
        let before = display_hash(&latched, &ppu.cgram, &[]);
        latched.obj_vram_latch.as_mut().unwrap()[17] = 1;
        assert_ne!(before, display_hash(&latched, &ppu.cgram, &[]));
        let mut palette = ppu.cgram.clone();
        palette[17] ^= 1;
        assert_ne!(initial, display_hash(ppu, &palette, &[]));
        let mut lines = vec![(0, 0, 0, 0, 0, [0; 4], [0; 4], [0; 8], false); 224];
        let before = display_hash(ppu, &ppu.cgram, &lines);
        lines[17].5[0] = 1;
        assert_ne!(before, display_hash(ppu, &ppu.cgram, &lines));
        let mut scratch = ppu.clone();
        scratch.render_buffer = Some(vec![255; 128]);
        scratch.bg_buffers[0].data[0] ^= 1;
        assert_eq!(initial, display_hash(&scratch, &ppu.cgram, &[]));
    }

    #[test]
    fn comparison_identifies_display_and_mode_and_rejects_old_records() {
        let mut game = ZeldaState::new();
        let canonical = game.canonical_state_hash();
        let a = Record::capture(0, 0, &game, &[], 1);
        game.ppu.vram[17] ^= 1;
        assert_eq!(canonical, game.canonical_state_hash());
        let b = Record::capture(0, 0, &game, &[], 2);
        assert_eq!(a.differences(&b), vec!["display"]);
        game.set_lag_free(true);
        assert!(game.lag_free());
        let c = Record::capture(0, 0, &game, &[], 2);
        assert_eq!(b.differences(&c), vec!["lag_free", "state"]);
        let mut old = serde_json::to_value(&a).unwrap();
        old.as_object_mut().unwrap().remove("display");
        assert!(serde_json::from_value::<Record>(old).is_err());
    }
    #[test]
    fn comparison_rejects_missing_and_trailing_records() {
        let game = ZeldaState::new();
        let record = Record::capture(12, 0, &game, &[0], 0);
        let line = serde_json::to_string(&record).unwrap() + "\n";
        let make = |text: String| Ledger {
            interval: 1,
            rgb: None,
            output: None,
            baseline: Some(Box::new(std::io::Cursor::new(text.into_bytes()))),
        };
        let mut matching = make(line.clone());
        matching.observe(&record).unwrap();
        matching.finish().unwrap();
        let mut legacy = serde_json::to_value(&record).unwrap();
        legacy["schema"] = 2.into();
        let error = make(legacy.to_string() + "\n").observe(&record).unwrap_err().to_string();
        assert!(error.contains("zelda3-state-v2 required; re-record v1 baselines"));
        let mut short = make(String::new());
        let error = short.observe(&record).unwrap_err().to_string();
        assert!(error.contains("frame=12 fields=missing_record"));
        assert!(make(line)
            .finish()
            .unwrap_err()
            .to_string()
            .contains("extra records"));
    }

    #[test]
    fn compressed_output_is_finalized_and_roundtrips() {
        let path =
            std::env::temp_dir().join(format!("z3-ledger-test-{}.jsonl.zst", std::process::id()));
        let args = vec!["--state-ledger".into(), path.to_str().unwrap().into()];
        let mut ledger = Ledger::open(&args).unwrap().unwrap();
        let game = ZeldaState::new();
        let record = Record::capture(0, 0, &game, &[0], 0);
        ledger.observe(&record).unwrap();
        ledger.finish().unwrap();
        let args = vec![
            "--compare-state-ledger".into(),
            path.to_str().unwrap().into(),
        ];
        let mut ledger = Ledger::open(&args).unwrap().unwrap();
        ledger.observe(&record).unwrap();
        ledger.finish().unwrap();
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn state_owners_and_comparison() {
        let mut game = ZeldaState::new();
        let initial = game.canonical_state_hash();
        game.ram[5] ^= 1;
        assert_ne!(initial, game.canonical_state_hash());
        game.ram[5] ^= 1;
        game.sram[5] ^= 1;
        assert_ne!(initial, game.canonical_state_hash());
        game.sram[5] ^= 1;
        game.wanted_zelda_features ^= 1;
        assert_ne!(initial, game.canonical_state_hash());
        let a = Record::capture(7, 0, &game, &[0], 0);
        let b = Record::capture(7, 1, &game, &[1], 0);
        assert_eq!(a.differences(&b), vec!["input", "audio"]);
        let roundtrip: Record = serde_json::from_str(&serde_json::to_string(&a).unwrap()).unwrap();
        assert!(a.differences(&roundtrip).is_empty());
    }
}
