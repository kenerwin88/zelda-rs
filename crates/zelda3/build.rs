use sha2::{Digest, Sha256};
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
    #[serde(default)]
    interrupt_voice: bool,
    #[serde(default)]
    interrupt_delay_frames: u8,
    #[serde(default)]
    interrupt_scheduler_tick_index: u8,
    #[serde(default)]
    ownership_duration_samples: u32,
    #[serde(default)]
    ownership_release_overflows: u8,
    #[serde(default)]
    volume_via_parameters: bool,
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

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SampleBankManifest {
    format: String,
    sample_rate: u32,
    samples: Vec<SampleManifest>,
    banks: Vec<BankManifest>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SampleManifest {
    id: String,
    file: String,
    sha256: String,
    blocks: usize,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct BankManifest {
    id: u8,
    name: String,
    instruments: Vec<InstrumentManifest>,
    echo_seed: EchoSeedManifest,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct InstrumentManifest {
    source: u8,
    sample: String,
    loop_offset: usize,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct EchoSeedManifest {
    start_address: usize,
    file: String,
    sha256: String,
}

#[derive(serde::Serialize)]
struct PackedSampleBank {
    sample_rate: u32,
    samples: Vec<PackedSample>,
    banks: Vec<PackedBank>,
}
#[derive(serde::Serialize)]
struct PackedSample {
    brr: Vec<u8>,
}
#[derive(serde::Serialize)]
struct PackedBank {
    id: u8,
    name: String,
    instruments: Vec<PackedInstrument>,
    echo_start: usize,
    echo_seed: Vec<u8>,
}
#[derive(serde::Serialize)]
struct PackedInstrument {
    source: u8,
    sample_index: usize,
    loop_offset: usize,
}

fn compile_sample_bank(manifest_dir: &Path, out_dir: &Path) {
    let root = manifest_dir.join("../../assets/audio/modern_samples");
    let source = root.join("manifest.json");
    println!("cargo:rerun-if-changed={}", source.display());
    let manifest: SampleBankManifest = serde_json::from_slice(&fs::read(&source).unwrap())
        .unwrap_or_else(|error| panic!("{}: {error}", source.display()));
    assert_eq!(manifest.format, "zelda3_modern_sample_bank_v1");
    assert_eq!(manifest.sample_rate, 32_000);

    let mut sample_ids = BTreeMap::new();
    let mut samples = Vec::new();
    for sample in manifest.samples {
        assert!(
            sample_ids
                .insert(sample.id.clone(), samples.len())
                .is_none(),
            "duplicate sample {}",
            sample.id
        );
        let path = root.join(&sample.file);
        println!("cargo:rerun-if-changed={}", path.display());
        let brr = fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        assert_eq!(
            brr.len(),
            sample.blocks * 9,
            "{}: BRR block count",
            path.display()
        );
        assert!(
            !brr.is_empty() && brr.len() % 9 == 0,
            "{}: invalid BRR stream",
            path.display()
        );
        assert!(
            brr.chunks_exact(9).last().unwrap()[0] & 1 != 0,
            "{}: missing BRR end flag",
            path.display()
        );
        assert_eq!(
            format!("{:x}", Sha256::digest(&brr)),
            sample.sha256,
            "{}: sha256",
            path.display()
        );
        samples.push(PackedSample { brr });
    }

    let mut bank_ids = BTreeSet::new();
    let mut banks = Vec::new();
    for bank in manifest.banks {
        assert!(bank_ids.insert(bank.id), "duplicate bank {}", bank.id);
        assert!(!bank.name.is_empty(), "bank {} has no name", bank.id);
        let mut sources = BTreeSet::new();
        let mut instruments = Vec::new();
        for instrument in bank.instruments {
            assert!(
                sources.insert(instrument.source),
                "duplicate source {} in bank {}",
                instrument.source,
                bank.id
            );
            let sample_index = *sample_ids
                .get(&instrument.sample)
                .unwrap_or_else(|| panic!("unknown sample {}", instrument.sample));
            let sample_len = samples[sample_index].brr.len();
            assert!(
                instrument.loop_offset % 9 == 0 && instrument.loop_offset < sample_len,
                "invalid loop offset for source {} in bank {}",
                instrument.source,
                bank.id
            );
            instruments.push(PackedInstrument {
                source: instrument.source,
                sample_index,
                loop_offset: instrument.loop_offset,
            });
        }
        assert_eq!(
            sources.into_iter().collect::<Vec<_>>(),
            (0..25).collect::<Vec<_>>(),
            "bank {} must map sources 0..24",
            bank.id
        );
        instruments.sort_by_key(|instrument| instrument.source);
        let echo_path = root.join(&bank.echo_seed.file);
        println!("cargo:rerun-if-changed={}", echo_path.display());
        let echo_seed =
            fs::read(&echo_path).unwrap_or_else(|error| panic!("{}: {error}", echo_path.display()));
        assert_eq!(
            bank.echo_seed.start_address + echo_seed.len(),
            0x10000,
            "{}: echo seed range",
            echo_path.display()
        );
        assert_eq!(
            format!("{:x}", Sha256::digest(&echo_seed)),
            bank.echo_seed.sha256,
            "{}: sha256",
            echo_path.display()
        );
        banks.push(PackedBank {
            id: bank.id,
            name: bank.name,
            instruments,
            echo_start: bank.echo_seed.start_address,
            echo_seed,
        });
    }
    banks.sort_by_key(|bank| bank.id);
    assert_eq!(
        banks.iter().map(|bank| bank.id).collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    let packed = bincode::serialize(&PackedSampleBank {
        sample_rate: manifest.sample_rate,
        samples,
        banks,
    })
    .expect("pack modern sample bank");
    write_if_changed(&out_dir.join("modern_sample_bank.bin"), &packed);
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

const MUSIC_FRAME_SAMPLES: u32 = 534;

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let source = manifest_dir.join("../../assets/audio/modern_music.tsv");
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    compile_sfx_assets(&manifest_dir, &out_dir);
    compile_sample_bank(&manifest_dir, &out_dir);
    println!("cargo:rerun-if-changed={}", source.display());
    let music_loops_source = manifest_dir.join("../../assets/audio/modern_music_loops.tsv");
    println!("cargo:rerun-if-changed={}", music_loops_source.display());
    let mut music_loops = BTreeMap::<u8, (u32, u32)>::new();
    for (line_index, raw) in fs::read_to_string(&music_loops_source)
        .unwrap()
        .lines()
        .enumerate()
    {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        assert_eq!(
            fields.len(),
            5,
            "{}:{}: expected 5 fields",
            music_loops_source.display(),
            line_index + 1
        );
        let track = u8::from_str_radix(fields[0], 16).unwrap();
        let start_frame: u32 = fields[1].parse().unwrap();
        let start_sample: u32 = fields[2].parse().unwrap();
        let end_frame: u32 = fields[3].parse().unwrap();
        let end_sample: u32 = fields[4].parse().unwrap();
        assert!(start_sample < MUSIC_FRAME_SAMPLES);
        assert!(end_sample < MUSIC_FRAME_SAMPLES);
        let start = start_frame * MUSIC_FRAME_SAMPLES + start_sample;
        let end = end_frame * MUSIC_FRAME_SAMPLES + end_sample;
        assert!(end > start, "music loop must have positive duration");
        assert!(music_loops.insert(track, (start, end)).is_none());
    }

    let text = fs::read_to_string(&source).unwrap();
    let mut tracks: BTreeMap<u8, (u16, Vec<[u8; 23]>)> = BTreeMap::new();
    for (line_index, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(
            fields.len(),
            20,
            "{}:{}: expected 20 fields",
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
        let kon_phase: u8 = fields[18].parse().unwrap();
        let keyoff_phase: u8 = fields[19].parse().unwrap();
        assert!(kon_phase < 32);
        assert!(keyoff_phase < 32);
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
            kon_phase,
            keyoff_phase,
        ]);
    }

    let mut generated = String::from("pub const PACKED_TRACKS: &[PackedModernMusicTrack] = &[\n");
    for (track, (lead, notes)) in tracks {
        let filename = format!("modern_music_{track:02x}.bin");
        let packed: Vec<u8> = notes.into_iter().flatten().collect();
        write_if_changed(&out_dir.join(&filename), &packed);
        let (loop_start_sample, loop_end_sample) =
            music_loops.get(&track).copied().unwrap_or_default();
        generated.push_str(&format!(
            "    PackedModernMusicTrack {{ track: 0x{track:02x}, lead_in_frames: {lead}, loop_start_sample: {loop_start_sample}, loop_end_sample: {loop_end_sample}, notes: include_bytes!(concat!(env!(\"OUT_DIR\"), \"/{filename}\")) }},\n"
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
            4,
            "{}:{}: expected 4 fields",
            globals_source.display(),
            line_index + 1
        );
        let track = u8::from_str_radix(fields[0], 16).unwrap();
        let dsp_cycle: u32 = fields[1].parse().unwrap();
        let register = u8::from_str_radix(fields[2], 16).unwrap();
        let value: u8 = fields[3].parse().unwrap();
        globals_generated.push_str(&format!(
            "    ModernMusicGlobalEvent {{ track: 0x{track:02x}, dsp_cycle: {dsp_cycle}, register: 0x{register:02x}, value: {value} }},\n"
        ));
    }
    globals_generated.push_str("];\n");
    write_if_changed(
        &out_dir.join("modern_music_global_assets.rs"),
        globals_generated.as_bytes(),
    );
}
