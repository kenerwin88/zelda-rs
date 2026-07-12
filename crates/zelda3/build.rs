use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn write_if_changed(path: &Path, bytes: &[u8]) {
    if fs::read(path).ok().as_deref() != Some(bytes) {
        fs::write(path, bytes).unwrap();
    }
}

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let source = manifest_dir.join("../../assets/audio/modern_music.tsv");
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
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

    let sfx_source = manifest_dir.join("../../assets/audio/modern_sfx_dsp.tsv");
    println!("cargo:rerun-if-changed={}", sfx_source.display());
    let mut sfx_generated =
        String::from("pub const EXACT_SFX_DSP_STEPS: &[ExactSfxDspStep] = &[\n");
    for (line_index, raw) in fs::read_to_string(&sfx_source).unwrap().lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<_> = line.split('\t').collect();
        assert!(
            matches!(fields.len(), 20 | 21),
            "{}:{}: expected 20 or 21 fields",
            sfx_source.display(),
            line_index + 1
        );
        let bank: u8 = fields[0].parse().unwrap();
        let id = u8::from_str_radix(fields[1], 16).unwrap();
        let variant_hash = u32::from_str_radix(fields[2], 16).unwrap();
        let step: u8 = fields[3].parse().unwrap();
        let voice: u8 = fields[4].parse().unwrap();
        let pitch: u8 = fields[5].parse().unwrap();
        let instrument: u8 = fields[6].parse().unwrap();
        let volume: u8 = fields[7].parse().unwrap();
        let pan: i8 = fields[8].parse().unwrap();
        let duration_frames: u8 = fields[9].parse().unwrap();
        let echo: u8 = fields[10].parse().unwrap();
        let command_delay_frames: u8 = fields[11].parse().unwrap();
        let scheduler_tick_index: u8 = fields[12].parse().unwrap();
        let dsp_pitch: u16 = fields[13].parse().unwrap();
        let volume_left: i8 = fields[14].parse().unwrap();
        let volume_right: i8 = fields[15].parse().unwrap();
        let adsr1: u8 = fields[16].parse().unwrap();
        let adsr2: u8 = fields[17].parse().unwrap();
        let gain: u8 = fields[18].parse().unwrap();
        let sample_offset: u16 = fields[19].parse().unwrap();
        let duration_samples: u32 = fields.get(20).map_or(0, |value| value.parse().unwrap());
        sfx_generated.push_str(&format!(
            "    ExactSfxDspStep {{ bank: {bank}, id: 0x{id:02x}, variant_hash: 0x{variant_hash:08x}, step: {step}, voice: {voice}, pitch: {pitch}, instrument: {instrument}, volume: {volume}, pan: {pan}, duration_frames: {duration_frames}, echo: {echo} != 0, command_delay_frames: {command_delay_frames}, scheduler_tick_index: {scheduler_tick_index}, dsp_pitch: {dsp_pitch}, volume_left: {volume_left}, volume_right: {volume_right}, adsr1: {adsr1}, adsr2: {adsr2}, gain: {gain}, sample_offset: {sample_offset}, duration_samples: {duration_samples} }},\n"
        ));
    }
    sfx_generated.push_str("];\n");
    write_if_changed(
        &out_dir.join("modern_sfx_dsp_assets.rs"),
        sfx_generated.as_bytes(),
    );

    let pitch_source = manifest_dir.join("../../assets/audio/modern_sfx_pitch.tsv");
    println!("cargo:rerun-if-changed={}", pitch_source.display());
    let mut pitch_generated =
        String::from("pub const EXACT_SFX_PITCH_EVENTS: &[ExactSfxPitchEvent] = &[\n");
    for (line_index, raw) in fs::read_to_string(&pitch_source)
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
            6,
            "{}:{}: expected 6 fields",
            pitch_source.display(),
            line_index + 1
        );
        let bank: u8 = fields[0].parse().unwrap();
        let id = u8::from_str_radix(fields[1], 16).unwrap();
        let variant_hash = u32::from_str_radix(fields[2], 16).unwrap();
        let step: u8 = fields[3].parse().unwrap();
        let relative_sample: u16 = fields[4].parse().unwrap();
        let pitch_word: u16 = fields[5].parse().unwrap();
        pitch_generated.push_str(&format!(
            "    ExactSfxPitchEvent {{ bank: {bank}, id: 0x{id:02x}, variant_hash: 0x{variant_hash:08x}, step: {step}, relative_sample: {relative_sample}, pitch_word: {pitch_word} }},\n"
        ));
    }
    pitch_generated.push_str("];\n");
    write_if_changed(
        &out_dir.join("modern_sfx_pitch_assets.rs"),
        pitch_generated.as_bytes(),
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
