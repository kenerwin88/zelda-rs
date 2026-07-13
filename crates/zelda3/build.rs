use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SfxAssets {
    format: String,
    programs: Vec<SfxProgram>,
    exact_dsp_steps: Vec<ExactDspStep>,
    pitch_events: Vec<PitchEvent>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SfxProgram {
    bank: u8,
    id: u8,
    variant: u8,
    variant_hash: u32,
    name: String,
    promotion_status: String,
    context: SfxContext,
    steps: Vec<SfxStep>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SfxContext {
    source_slot: u8,
    voice_mask: u8,
    context_voice_mask: u8,
    step_count: u8,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SfxEnvelope {
    attack: u8,
    decay: u8,
    sustain: u8,
    release: u8,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SfxSlide {
    target_pitch: u8,
    frames: u8,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SfxStep {
    voice: u8,
    pitch: u8,
    instrument: u8,
    waveform: String,
    volume: u8,
    pan: i8,
    echo: bool,
    envelope: SfxEnvelope,
    duration_frames: u8,
    pitch_slide: Option<SfxSlide>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ExactDspStep {
    bank: u8,
    id: u8,
    variant_hash: u32,
    step: u8,
    voice: u8,
    pitch: u8,
    instrument: u8,
    volume: u8,
    pan: i8,
    duration_frames: u8,
    echo: bool,
    command_delay_frames: u8,
    scheduler_tick_index: u8,
    dsp_pitch: u16,
    volume_left: i8,
    volume_right: i8,
    adsr1: u8,
    adsr2: u8,
    gain: u8,
    sample_offset: u16,
    #[serde(default)]
    duration_samples: u32,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PitchEvent {
    bank: u8,
    id: u8,
    variant_hash: u32,
    step: u8,
    relative_sample: u16,
    pitch_word: u16,
}

fn compile_sfx_assets(manifest_dir: &Path, out_dir: &Path) {
    let source = manifest_dir.join("../../assets/audio/modern_sfx.json");
    println!("cargo:rerun-if-changed={}", source.display());
    let assets: SfxAssets = serde_json::from_slice(&fs::read(&source).unwrap())
        .unwrap_or_else(|error| panic!("{}: {error}", source.display()));
    assert_eq!(
        assets.format, "zelda3_modern_sfx_assets_v1",
        "unsupported SFX asset format"
    );
    assert!(!assets.programs.is_empty(), "SFX asset catalog is empty");
    let mut identities = BTreeSet::new();
    for program in &assets.programs {
        assert_eq!(
            program.promotion_status, "review_ready",
            "unreviewed SFX program {}",
            program.name
        );
        assert!(!program.name.is_empty(), "SFX program name is empty");
        assert!(
            identities.insert((program.bank, program.id, program.variant_hash)),
            "duplicate SFX program identity for {}",
            program.name
        );
        assert!(
            program.steps.iter().all(|step| step.voice < 8),
            "invalid voice in {}",
            program.name
        );
        assert!(
            program.steps.iter().all(|step| matches!(
                step.waveform.as_str(),
                "Pulse" | "Saw" | "Triangle" | "Noise"
            )),
            "invalid waveform in {}",
            program.name
        );
    }
    assert!(
        assets.exact_dsp_steps.iter().all(|step| step.voice < 8),
        "invalid voice in exact SFX DSP records"
    );
    let mut exact_identities = BTreeSet::new();
    assert!(
        assets
            .exact_dsp_steps
            .iter()
            .all(|step| exact_identities.insert((
                step.bank,
                step.id,
                step.variant_hash,
                step.step
            ))),
        "duplicate exact SFX DSP record"
    );
    let mut pitch_identities = BTreeSet::new();
    assert!(
        assets
            .pitch_events
            .iter()
            .all(|event| pitch_identities.insert((
                event.bank,
                event.id,
                event.variant_hash,
                event.step,
                event.relative_sample
            ))),
        "duplicate exact SFX pitch event"
    );
    let packed = bincode::serialize(&assets).expect("pack modern SFX assets");
    write_if_changed(&out_dir.join("modern_sfx_assets.bin"), &packed);
}

fn write_if_changed(path: &Path, bytes: &[u8]) {
    if fs::read(path).ok().as_deref() != Some(bytes) {
        fs::write(path, bytes).unwrap();
    }
}

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let source = manifest_dir.join("../../assets/audio/modern_music.tsv");
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    compile_sfx_assets(&manifest_dir, &out_dir);
    println!("cargo:rerun-if-changed={}", source.display());

    let text = fs::read_to_string(&source).unwrap();
    let mut tracks: BTreeMap<u8, (u16, Vec<[u8; 21]>)> = BTreeMap::new();
    for (line_index, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(
            fields.len(),
            18,
            "{}:{}: expected 18 fields",
            source.display(),
            line_index + 1
        );
        let track = u8::from_str_radix(fields[0], 16).unwrap();
        let lead: u16 = fields[1].parse().unwrap();
        let voice: u8 = fields[2].parse().unwrap();
        let pitch: u8 = fields[3].parse().unwrap();
        let instrument: u8 = fields[4].parse().unwrap();
        let volume: u8 = fields[5].parse().unwrap();
        let pan: i8 = fields[6].parse().unwrap();
        let start: u16 = fields[7].parse().unwrap();
        let duration: u16 = fields[8].parse().unwrap();
        let dsp_pitch: u16 = fields[9].parse().unwrap();
        let sample_offset: u16 = fields[10].parse().unwrap();
        let volume_left: i8 = fields[11].parse().unwrap();
        let volume_right: i8 = fields[12].parse().unwrap();
        let adsr1: u8 = fields[13].parse().unwrap();
        let adsr2: u8 = fields[14].parse().unwrap();
        let gain: u8 = fields[15].parse().unwrap();
        let echo_send: u8 = fields[16].parse().unwrap();
        let keyoff_sample_offset: u16 = fields[17].parse().unwrap();
        assert!(
            voice < 8,
            "{}:{}: invalid voice",
            source.display(),
            line_index + 1
        );

        let entry = tracks.entry(track).or_insert_with(|| (lead, Vec::new()));
        assert_eq!(
            entry.0,
            lead,
            "{}:{}: inconsistent lead-in",
            source.display(),
            line_index + 1
        );
        if let Some(previous) = entry.1.last() {
            let previous_start = u16::from_le_bytes([previous[5], previous[6]]);
            assert!(
                start >= previous_start,
                "{}:{}: notes are not time-sorted",
                source.display(),
                line_index + 1
            );
        }
        let [start_lo, start_hi] = start.to_le_bytes();
        let [duration_lo, duration_hi] = duration.to_le_bytes();
        let [dsp_pitch_lo, dsp_pitch_hi] = dsp_pitch.to_le_bytes();
        let [sample_offset_lo, sample_offset_hi] = sample_offset.to_le_bytes();
        let [keyoff_offset_lo, keyoff_offset_hi] = keyoff_sample_offset.to_le_bytes();
        entry.1.push([
            voice,
            pitch,
            instrument,
            volume,
            pan as u8,
            start_lo,
            start_hi,
            duration_lo,
            duration_hi,
            dsp_pitch_lo,
            dsp_pitch_hi,
            sample_offset_lo,
            sample_offset_hi,
            volume_left as u8,
            volume_right as u8,
            adsr1,
            adsr2,
            gain,
            echo_send,
            keyoff_offset_lo,
            keyoff_offset_hi,
        ]);
    }

    let mut generated = String::from("pub const PACKED_TRACKS: &[PackedModernMusicTrack] = &[\n");
    for (track, (lead, notes)) in tracks {
        let filename = format!("modern_music_{track:02x}.bin");
        let packed: Vec<u8> = notes.into_iter().flatten().collect();
        write_if_changed(&out_dir.join(&filename), &packed);
        generated.push_str(&format!(
            "    PackedModernMusicTrack {{ track: 0x{track:02x}, lead_in_frames: {lead}, notes: include_bytes!(concat!(env!(\"OUT_DIR\"), \"/{filename}\")) }},\n"
        ));
    }
    generated.push_str("];\n");
    write_if_changed(
        &out_dir.join("modern_music_assets.rs"),
        generated.as_bytes(),
    );

    let globals_source = manifest_dir.join("../../assets/audio/modern_music_globals.tsv");
    println!("cargo:rerun-if-changed={}", globals_source.display());
    let mut globals_generated =
        String::from("pub const MUSIC_GLOBAL_EVENTS: &[ModernMusicGlobalEvent] = &[\n");
    for (line_index, raw) in fs::read_to_string(&globals_source)
        .unwrap()
        .lines()
        .enumerate()
    {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(
            fields.len(),
            5,
            "{}:{}: expected 5 fields",
            globals_source.display(),
            line_index + 1
        );
        let track = u8::from_str_radix(fields[0], 16).unwrap();
        let start_frame: u16 = fields[1].parse().unwrap();
        let sample_offset: u16 = fields[2].parse().unwrap();
        let register = u8::from_str_radix(fields[3], 16).unwrap();
        let value: u8 = fields[4].parse().unwrap();
        globals_generated.push_str(&format!(
            "    ModernMusicGlobalEvent {{ track: 0x{track:02x}, start_frame: {start_frame}, sample_offset: {sample_offset}, register: 0x{register:02x}, value: {value} }},\n"
        ));
    }
    globals_generated.push_str("];\n");
    write_if_changed(
        &out_dir.join("modern_music_global_assets.rs"),
        globals_generated.as_bytes(),
    );
}
