use std::env;

use zelda3::{
    game_output::{
        checksum_dsp_write_values, checksum_dsp_writes, checksum_samples, AudioSampleStats,
        AudioTraceFrameSummary, DspWriteEvent,
    },
    modern_audio::ModernAudioEngine,
    modern_audio_sequence::ModernAudioSequencer,
    ZeldaState,
};

use crate::{TRACE_MAIN_MODULE_INDEX, TRACE_SUBMODULE_INDEX, TRACE_SUBSUBMODULE_INDEX};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct AudioFrameStats {
    pub(crate) samples_per_channel: usize,
    pub(crate) peak: i16,
    pub(crate) first_nonzero: Option<usize>,
    pub(crate) mean_abs: u32,
}

impl AudioFrameStats {
    pub(crate) fn from_interleaved_stereo(samples: &[i16]) -> Self {
        let stats = AudioSampleStats::from_interleaved(samples, 2);
        Self {
            samples_per_channel: stats.samples_per_channel,
            peak: stats.peak,
            first_nonzero: stats.first_nonzero,
            mean_abs: stats.mean_abs,
        }
    }
}

pub(crate) fn first_peak_frame(stats: &[AudioFrameStats], threshold: i16) -> Option<usize> {
    stats.iter().position(|stats| stats.peak >= threshold)
}

pub(crate) fn max_peak_frame(stats: &[AudioFrameStats]) -> Option<(usize, i16)> {
    stats
        .iter()
        .enumerate()
        .max_by_key(|(_, stats)| stats.peak)
        .map(|(i, stats)| (i, stats.peak))
}

pub(crate) fn print_audio_window(
    label: &str,
    stats: &[AudioFrameStats],
    debug: &[String],
    center: Option<usize>,
) {
    let Some(center) = center else {
        println!("{label}: no non-silent frames captured");
        return;
    };
    let start = center.saturating_sub(4);
    let end = (center + 12).min(stats.len().saturating_sub(1));
    println!("{label} window frames {start}..={end}:");
    for i in start..=end {
        let stats = stats[i];
        if let Some(debug) = debug.get(i) {
            println!(
                "  {i:>5}: peak={:>5} mean_abs={:>4} first={:?} samples={} {debug}",
                stats.peak, stats.mean_abs, stats.first_nonzero, stats.samples_per_channel,
            );
        } else {
            println!(
                "  {i:>5}: peak={:>5} mean_abs={:>4} first={:?} samples={}",
                stats.peak, stats.mean_abs, stats.first_nonzero, stats.samples_per_channel,
            );
        }
    }
}

pub(crate) fn replay_checksum_samples(samples: &[i16]) -> u32 {
    checksum_samples(samples)
}

pub(crate) fn should_write_fingerprint(fingerprint_frame: Option<u32>, frame: u32) -> bool {
    fingerprint_frame.is_none_or(|target| frame == target)
}

pub(crate) fn print_replay_audio_trace(
    frame: u32,
    game: &ZeldaState,
    audio: &[i16],
    samples: usize,
    channels: usize,
    dsp_pre_hash: u32,
    dsp_writes: &[DspWriteEvent],
    spc_ram_pre: &[u8],
    modern_sequence: &mut ModernAudioSequencer,
    modern_engine: &mut ModernAudioEngine,
) {
    let dsp_globals = game.zelda_audio_dsp_global_state();
    let dsp_voices = game.zelda_audio_dsp_voice_states();
    let dsp_globals_json = format!(
        "{{\"master\":[{},{}],\"echo_volume\":[{},{}],\"echo_feedback\":{},\"flags\":{},\"echo_enable\":{},\"pitch_modulation\":{},\"noise_enable\":{},\"echo_start_page\":{},\"echo_delay\":{},\"fir\":{:?},\"echo_index\":{},\"echo_remaining\":{},\"fir_index\":{}}}",
        dsp_globals.master_volume_left,
        dsp_globals.master_volume_right,
        dsp_globals.echo_volume_left,
        dsp_globals.echo_volume_right,
        dsp_globals.echo_feedback,
        dsp_globals.flags,
        dsp_globals.echo_enable_mask,
        dsp_globals.pitch_modulation_mask,
        dsp_globals.noise_enable_mask,
        dsp_globals.echo_start_page,
        dsp_globals.echo_delay,
        dsp_globals.fir,
        dsp_globals.echo_buffer_index,
        dsp_globals.echo_remaining,
        dsp_globals.fir_history_index,
    );
    let dsp_voices_json = format!(
        "[{}]",
        dsp_voices
            .iter()
            .enumerate()
            .map(|(index, voice)| format!(
                "{{\"voice\":{index},\"pitch\":{},\"counter\":{},\"source\":{},\"state\":{},\"rate_counter\":{},\"gain\":{},\"sample\":{},\"volume\":[{},{}]}}",
                voice.pitch,
                voice.pitch_counter,
                voice.source,
                voice.envelope_state,
                voice.envelope_rate_counter,
                voice.gain,
                voice.sample_out,
                voice.volume_left,
                voice.volume_right,
            ))
            .collect::<Vec<_>>()
            .join(",")
    );
    let event_frame = game.zelda_audio_event_frame_from_dsp_writes(&dsp_writes);
    let mut modern_route = game.zelda_audio_route_state();
    // C SPC receipts are reference observations. Feeding them into the
    // candidate sequencer would replace modern scheduling decisions with
    // oracle KON/KOF timing and make lifecycle regressions invisible.
    modern_route.spc = None;
    let modern_event_frame =
        modern_sequence.sequence_engine_commands(modern_route, game.zelda_engine_audio_commands());
    let modern_sequence_stats = modern_sequence.last_stats();
    let classic_spc = game.zelda_audio_route_state().spc;
    let (classic_sfx_events_json, classic_sfx_voice_mask) = classic_sfx_events_json(classic_spc);
    let modern_sfx_events_json = modern_sfx_events_json(&modern_event_frame, modern_sequence_stats);
    let sfx_lockstep_json = format!(
        ",\"classic_sfx_events\":{classic_sfx_events_json},\"modern_sfx_events\":{modern_sfx_events_json},\"classic_sfx_voice_mask\":{classic_sfx_voice_mask},\"modern_sfx_voice_mask\":{}",
        modern_sequence_stats.sfx_voice_mask,
    );
    let mut modern_audio = vec![0i16; samples.saturating_mul(channels)];
    let static_compat_ram = std::env::var_os("ZELDA3_MODERN_AUDIO_STATIC_SAMPLE_RAM")
        .map(|_| game.zelda_modern_audio_compat_ram());
    let modern_sample_ram = static_compat_ram.as_deref().unwrap_or(spc_ram_pre);
    if std::env::var("ZELDA3_AUDIO_GLOBAL_DUMP_FRAME")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        == Some(frame)
    {
        eprintln!(
            "modern audio global pre frame {frame}: {:?}",
            modern_engine.global_debug_state()
        );
    }
    let modern_audio_stats = modern_engine.render_frame_with_sample_ram(
        &modern_event_frame,
        &mut modern_audio,
        samples as i32,
        channels as i32,
        Some(modern_sample_ram),
    );
    let modern_pitch_events_json = format!(
        "[{}]",
        modern_event_frame
            .events
            .iter()
            .filter_map(|event| match event.kind {
                zelda3::game_output::AudioEventKind::SetPitchWord { voice, pitch_word } => {
                    Some(format!(
                        "[{},{},{}]",
                        event.sample_offset, voice, pitch_word
                    ))
                }
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(",")
    );
    let modern_voices = modern_engine.voice_debug_states();
    let modern_echo_state = modern_engine.echo_debug_state();
    let modern_echo_value = modern_engine.echo_debug_value();
    let live_ram = game.zelda_audio_live_spc_ram();
    if std::env::var("ZELDA3_AUDIO_ECHO_RING_DUMP_FRAME")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        == Some(frame)
    {
        let (modern_left, modern_right) = modern_engine.echo_debug_ring();
        let (fir_left, fir_right) = modern_engine.echo_debug_fir_history();
        let start = usize::from(dsp_globals.echo_start_page) << 8;
        let differences = (0..modern_left.len().min(modern_right.len()))
            .filter_map(|index| {
                let address = start + index * 4;
                let bytes = live_ram.get(address..address + 4)?;
                let classic_left = i16::from_le_bytes([bytes[0], bytes[1]]);
                let classic_right = i16::from_le_bytes([bytes[2], bytes[3]]);
                (classic_left != modern_left[index] || classic_right != modern_right[index])
                    .then_some((
                        index,
                        classic_left,
                        modern_left[index],
                        classic_right,
                        modern_right[index],
                    ))
            })
            .collect::<Vec<_>>();
        eprintln!(
            "audio echo ring differences={} first={:?} last={:?}",
            differences.len(),
            differences.first(),
            differences.last()
        );
        eprintln!("audio echo fir left={fir_left:?} right={fir_right:?}");
        eprintln!(
            "audio classic fir left={:?} right={:?}",
            dsp_globals.fir_history_left, dsp_globals.fir_history_right
        );
        let ring_len = modern_left.len();
        let preceding = (1..=8)
            .rev()
            .map(|distance| {
                let index = (modern_echo_state.0 + ring_len - distance) % ring_len;
                (index, modern_left[index] >> 1, modern_right[index] >> 1)
            })
            .collect::<Vec<_>>();
        eprintln!("audio echo preceding={preceding:?}");
    }
    let echo_address = (usize::from(dsp_globals.echo_start_page) << 8)
        + usize::from(dsp_globals.echo_buffer_index) * 4;
    let classic_echo_value = live_ram
        .get(echo_address..echo_address + 4)
        .map(|bytes| {
            (
                i16::from_le_bytes([bytes[0], bytes[1]]),
                i16::from_le_bytes([bytes[2], bytes[3]]),
            )
        })
        .unwrap_or((0, 0));
    let modern_voices_json = format!(
        "[{}]",
        modern_voices
            .iter()
            .enumerate()
            .map(|(index, voice)| format!(
                "{{\"voice\":{index},\"active\":{},\"volume\":[{},{}],\"echo_send\":{},\"pitch\":{},\"counter\":{},\"state\":{},\"rate_counter\":{},\"gain\":{},\"adsr\":[{},{},{}],\"sample\":{},\"sample_backed\":{},\"sample_length\":{},\"sample_loops\":{},\"block_start\":{}}}",
                voice.active,
                voice.volume_left,
                voice.volume_right,
                voice.echo_send,
                voice.pitch,
                voice.pitch_counter,
                voice.envelope_state,
                voice.envelope_rate_counter,
                voice.gain,
                voice.adsr1,
                voice.adsr2,
                voice.gain_config,
                voice.sample_out,
                voice.sample_backed,
                voice.sample_length,
                voice.sample_loops,
                voice.brr_block_start,
            ))
            .collect::<Vec<_>>()
            .join(",")
    );
    maybe_dump_audio_samples(frame, audio, &modern_audio);
    let (modern_left_abs, modern_right_abs) = stereo_channel_abs(&modern_audio, channels);
    let modern_oracle_diff = sample_diff(audio, &modern_audio);
    if std::env::var("ZELDA3_AUDIO_VOICE_DUMP_FRAME")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        == Some(frame)
    {
        let classic_voices = game.zelda_audio_dsp_debug_voice_samples();
        eprintln!(
            "audio voice 2 prefix classic={:?} modern={:?}",
            &classic_voices[2][..classic_voices[2].len().min(16)],
            &modern_engine.debug_voice_samples()[2]
                [..modern_engine.debug_voice_samples()[2].len().min(16)]
        );
        eprintln!(
            "audio voice edge prefix v6={:?} v7={:?}",
            &classic_voices[6][..classic_voices[6].len().min(4)],
            &classic_voices[7][..classic_voices[7].len().min(4)]
        );
        eprintln!(
            "audio voice 4 prefix={:?}",
            &classic_voices[4][..classic_voices[4].len().min(80)]
        );
        for voice in 0..8 {
            let modern_voice = &modern_engine.debug_voice_samples()[voice];
            let differences = classic_voices[voice]
                .iter()
                .zip(modern_voice)
                .enumerate()
                .filter_map(|(sample, (&classic, &modern))| {
                    (classic != modern).then_some((sample, classic, modern))
                })
                .collect::<Vec<_>>();
            if !differences.is_empty() {
                eprintln!(
                    "audio voice {voice} differences={} first={:?} last={:?}",
                    differences.len(),
                    differences.first(),
                    differences.last()
                );
                let encode = |samples: &[i16]| {
                    samples
                        .iter()
                        .flat_map(|sample| sample.to_le_bytes())
                        .collect::<Vec<_>>()
                };
                let _ = std::fs::write(
                    format!("target/audio-voice-{frame}-{voice}-classic.pcm"),
                    encode(&classic_voices[voice]),
                );
                let _ = std::fs::write(
                    format!("target/audio-voice-{frame}-{voice}-modern.pcm"),
                    encode(modern_voice),
                );
            }
        }
    }
    if std::env::var("ZELDA3_AUDIO_EVENT_DUMP_FRAME")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        == Some(frame)
    {
        eprintln!(
            "modern audio events frame {frame}: {:#?}",
            modern_event_frame.events
        );
        eprintln!(
            "semantic SPC receipts frame {frame}: {:#?}",
            game.zelda_audio_route_state().spc
        );
        eprintln!("classic DSP writes frame {frame}: {dsp_writes:#?}");
    }
    let summary = AudioTraceFrameSummary::from_parts(
        audio,
        channels,
        dsp_pre_hash,
        game.zelda_audio_dsp_hash(),
        dsp_writes,
        &event_frame,
    );
    let stats = AudioFrameStats::from_interleaved_stereo(audio);
    let mean_abs = if audio.is_empty() {
        0.0
    } else {
        audio
            .iter()
            .map(|sample| i64::from(sample.saturating_abs()))
            .sum::<i64>() as f64
            / audio.len() as f64
    };
    print!(
        "{{\"frame\":{frame},\"samples\":{samples},\"channels\":{channels},\"peak\":{},\"first_nonzero\":",
        stats.peak
    );
    if let Some(first_nonzero) = stats.first_nonzero {
        print!("{first_nonzero}");
    } else {
        print!("null");
    }
    println!(
        ",\"mean_abs\":{mean_abs:.6},\"hash\":\"0x{:08x}\",\"apui\":[{},{},{},{}],\"music\":[{},{},{}],\"main\":{},\"sub\":{},\"subsub\":{},\"inidisp\":{},\"dsp_pre\":\"0x{:08x}\",\"dsp_post\":\"0x{:08x}\",\"dsp_globals\":{},\"dsp_voices\":{},\"dsp_writes\":{},\"dsp_write_hash\":\"0x{:08x}\",\"dsp_write_values_hash\":\"0x{:08x}\",\"command_events\":{},\"command_hash\":\"0x{:08x}\",\"unresolved_dsp_writes\":{},\"modern_sfx_known\":{},\"modern_sfx_unknown\":{},\"modern_sfx_exact_steps\":{},\"modern_sfx_known_programs\":{},\"modern_sfx_unknown_programs\":{},\"modern_voice_mask\":{},\"modern_program_hash\":\"0x{:08x}\",\"modern_command_events\":{},\"modern_command_hash\":\"0x{:08x}\",\"modern_note_events\":{},\"modern_pitch_events\":{},\"modern_voices\":{},\"modern_echo_index\":{},\"modern_fir_index\":{},\"modern_echo_remaining\":{},\"classic_echo_value\":[{},{}],\"modern_echo_value\":[{},{}],\"modern_audio\":{{\"peak\":{},\"hash\":\"0x{:08x}\",\"active_voices\":{},\"understood_events\":{},\"ignored_events\":{},\"left_abs\":{},\"right_abs\":{},\"oracle_mean_abs_diff\":{:.6},\"oracle_max_abs_diff\":{},\"oracle_exact_samples\":{}}}{} {},{}{}",
        summary.sample_stats.checksum,
        event_frame.music.apui00,
        event_frame.music.music_control,
        event_frame.music.sound_effect_ambient,
        event_frame.music.sound_effect_1,
        event_frame.music.sound_effect_2,
        event_frame.music.queued_music_control,
        event_frame.music.last_music_control,
        game.ram[TRACE_MAIN_MODULE_INDEX],
        game.ram[TRACE_SUBMODULE_INDEX],
        game.ram[TRACE_SUBSUBMODULE_INDEX],
        game.ram[0x13],
        summary.dsp_pre_hash,
        summary.dsp_post_hash,
        dsp_globals_json,
        dsp_voices_json,
        summary.dsp_write_count,
        summary.dsp_write_hash,
        summary.dsp_write_values_hash,
        summary.command_event_count,
        summary.command_event_hash,
        summary.unresolved_dsp_writes,
        modern_sequence_stats.known_sfx_commands,
        modern_sequence_stats.unknown_sfx_commands,
        modern_sequence_stats.exact_sfx_steps,
        sfx_programs_json(
            &modern_sequence_stats.known_sfx_programs,
            modern_sequence_stats.known_sfx_program_count,
        ),
        sfx_programs_json(
            &modern_sequence_stats.unknown_sfx_programs,
            modern_sequence_stats.unknown_sfx_program_count,
        ),
        modern_sequence_stats.active_voice_mask,
        modern_sequence_stats.program_hash,
        modern_event_frame.events.len(),
        modern_event_frame.command_hash(),
        modern_note_events_json(&modern_event_frame),
        modern_pitch_events_json,
        modern_voices_json,
        modern_echo_state.0,
        modern_echo_state.1,
        modern_echo_state.2,
        classic_echo_value.0,
        classic_echo_value.1,
        modern_echo_value.0,
        modern_echo_value.1,
        modern_audio_stats.peak,
        modern_audio_stats.checksum,
        modern_audio_stats.active_voices,
        modern_audio_stats.understood_events,
        modern_audio_stats.ignored_events,
        modern_left_abs,
        modern_right_abs,
        modern_oracle_diff.mean_abs,
        modern_oracle_diff.max_abs,
        modern_oracle_diff.exact_samples,
        sfx_lockstep_json,
        replay_dsp_write_events_json(frame, dsp_writes),
        game.zelda_audio_route_debug_json(),
        "}",
    );
}

#[derive(Debug)]
struct NormalizedSfxEvent {
    offset: i32,
    order: u8,
    voice: u8,
    json: String,
}

fn normalized_sfx_events_json(mut events: Vec<NormalizedSfxEvent>) -> String {
    events.sort_by_key(|event| (event.offset, event.order, event.voice));
    format!(
        "[{}]",
        events
            .into_iter()
            .map(|event| event.json)
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn classic_sfx_events_json(spc: Option<zelda3::game_output::SpcSequencerState>) -> (String, u8) {
    let Some(spc) = spc else {
        return ("[]".to_owned(), 0);
    };
    let mut events = Vec::new();
    for index in 0..usize::from(spc.sfx_kon_count.min(8)) {
        let mask = spc.sfx_kon_owned_masks[index];
        for voice in 0..8usize {
            if mask & (1 << voice) == 0 {
                continue;
            }
            let offset = i32::from(spc.sfx_kon_offsets[index]);
            events.push(NormalizedSfxEvent {
                offset,
                order: 0,
                voice: voice as u8,
                json: format!(
                    "[\"on\",{offset},{voice},{},{},{},{},{},{}]",
                    spc.sfx_kon_sources[index][voice],
                    spc.sfx_kon_adsr1[index][voice],
                    spc.sfx_kon_adsr2[index][voice],
                    spc.sfx_kon_gain[index][voice],
                    spc.sfx_kon_volume_left[index][voice],
                    spc.sfx_kon_volume_right[index][voice],
                ),
            });
        }
    }
    for index in 0..usize::from(spc.sfx_pitch_count.min(32)) {
        for voice in 0..8usize {
            if spc.sfx_pitch_masks[index] & (1 << voice) == 0 {
                continue;
            }
            let offset = i32::from(spc.sfx_pitch_offsets[index]);
            let pitch = spc.sfx_pitch_words[index];
            events.push(NormalizedSfxEvent {
                offset,
                order: 1,
                voice: voice as u8,
                json: format!("[\"pitch\",{offset},{voice},{pitch}]"),
            });
        }
    }
    for index in 0..usize::from(spc.sfx_volume_count.min(32)) {
        for voice in 0..8usize {
            if spc.sfx_volume_masks[index] & (1 << voice) == 0 {
                continue;
            }
            let offset = i32::from(spc.sfx_volume_offsets[index]);
            let left = spc.sfx_volume_left[index];
            let right = spc.sfx_volume_right[index];
            events.push(NormalizedSfxEvent {
                offset,
                order: 2,
                voice: voice as u8,
                json: format!("[\"volume\",{offset},{voice},{left},{right}]"),
            });
        }
    }
    for index in 0..usize::from(spc.sfx_kof_count.min(8)) {
        for voice in 0..8usize {
            if spc.sfx_kof_masks[index] & (1 << voice) == 0 {
                continue;
            }
            let offset = i32::from(spc.sfx_kof_offsets[index]);
            events.push(NormalizedSfxEvent {
                offset,
                order: 3,
                voice: voice as u8,
                json: format!("[\"off\",{offset},{voice}]"),
            });
        }
    }
    (normalized_sfx_events_json(events), spc.is_chan_on)
}

fn modern_sfx_events_json(
    frame: &zelda3::game_output::AudioEventFrame,
    stats: zelda3::modern_audio_sequence::ModernAudioSequenceStats,
) -> String {
    use zelda3::game_output::{AudioEventKind, AudioNoteOrigin};

    let mut relevant_mask = stats.sfx_voice_mask_start | stats.sfx_voice_mask;
    for event in &frame.events {
        if let AudioEventKind::SetNoteOrigin {
            voice,
            origin: AudioNoteOrigin::Sfx,
        } = event.kind
        {
            relevant_mask |= 1 << voice;
        }
    }
    let mut envelope = [[0u8; 3]; 8];
    let mut volume = [[0i8; 2]; 8];
    let mut events = Vec::new();
    for event in &frame.events {
        match event.kind {
            AudioEventKind::SetDspEnvelope {
                voice,
                adsr1,
                adsr2,
                gain,
            } if relevant_mask & (1 << voice) != 0 => {
                envelope[usize::from(voice)] = [adsr1, adsr2, gain];
            }
            AudioEventKind::SetStereoVolume { voice, left, right }
                if relevant_mask & (1 << voice) != 0 =>
            {
                volume[usize::from(voice)] = [left, right];
                let offset = event.sample_offset;
                events.push(NormalizedSfxEvent {
                    offset,
                    order: 2,
                    voice,
                    json: format!("[\"volume\",{offset},{voice},{left},{right}]"),
                });
            }
            AudioEventKind::SetPitchWord { voice, pitch_word }
            | AudioEventKind::SetPitchRegisterWord { voice, pitch_word }
                if relevant_mask & (1 << voice) != 0 =>
            {
                let offset = event.sample_offset;
                events.push(NormalizedSfxEvent {
                    offset,
                    order: 1,
                    voice,
                    json: format!("[\"pitch\",{offset},{voice},{pitch_word}]"),
                });
            }
            AudioEventKind::NoteOn {
                voice, instrument, ..
            } if relevant_mask & (1 << voice) != 0 => {
                let offset = event.sample_offset;
                let [adsr1, adsr2, gain] = envelope[usize::from(voice)];
                let [left, right] = volume[usize::from(voice)];
                events.push(NormalizedSfxEvent {
                    offset,
                    order: 0,
                    voice,
                    json: format!(
                        "[\"on\",{offset},{voice},{instrument},{adsr1},{adsr2},{gain},{left},{right}]"
                    ),
                });
            }
            AudioEventKind::KeyOnVoice {
                voice,
                source,
                adsr1,
                adsr2,
                gain,
                volume_left,
                volume_right,
                ..
            } if relevant_mask & (1 << voice) != 0 => {
                let offset = event.sample_offset;
                events.push(NormalizedSfxEvent {
                    offset,
                    order: 0,
                    voice,
                    json: format!(
                        "[\"on\",{offset},{voice},{source},{adsr1},{adsr2},{gain},{volume_left},{volume_right}]"
                    ),
                });
            }
            AudioEventKind::NoteOff { voice } if relevant_mask & (1 << voice) != 0 => {
                let offset = event.sample_offset;
                events.push(NormalizedSfxEvent {
                    offset,
                    order: 3,
                    voice,
                    json: format!("[\"off\",{offset},{voice}]"),
                });
            }
            _ => {}
        }
    }
    normalized_sfx_events_json(events)
}

fn maybe_dump_audio_samples(frame: u32, classic: &[i16], modern: &[i16]) {
    let Some(target) = std::env::var("ZELDA3_AUDIO_SAMPLE_DUMP_FRAME")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
    else {
        return;
    };
    if frame != target {
        return;
    }
    let prefix = std::env::var("ZELDA3_AUDIO_SAMPLE_DUMP_PREFIX")
        .unwrap_or_else(|_| format!("target/audio-samples-{frame}"));
    let encode = |samples: &[i16]| {
        samples
            .iter()
            .flat_map(|sample| sample.to_le_bytes())
            .collect::<Vec<_>>()
    };
    if let Err(error) = std::fs::write(format!("{prefix}-classic.pcm"), encode(classic)) {
        eprintln!("failed to dump classic audio samples: {error}");
    }
    if let Err(error) = std::fs::write(format!("{prefix}-modern.pcm"), encode(modern)) {
        eprintln!("failed to dump modern audio samples: {error}");
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct SampleDiff {
    mean_abs: f64,
    max_abs: u16,
    exact_samples: usize,
}

fn sample_diff(reference: &[i16], candidate: &[i16]) -> SampleDiff {
    let count = reference.len().min(candidate.len());
    if count == 0 {
        return SampleDiff::default();
    }
    let mut total = 0u64;
    let mut max_abs = 0u16;
    let mut exact_samples = 0usize;
    for (&reference, &candidate) in reference.iter().zip(candidate).take(count) {
        let difference = (i32::from(reference) - i32::from(candidate)).unsigned_abs();
        total += u64::from(difference);
        max_abs = max_abs.max(difference.min(u32::from(u16::MAX)) as u16);
        exact_samples += usize::from(difference == 0);
    }
    SampleDiff {
        mean_abs: total as f64 / count as f64,
        max_abs,
        exact_samples,
    }
}

fn sfx_programs_json(programs: &[u16], count: u8) -> String {
    let values = programs
        .iter()
        .take(usize::from(count))
        .map(|program| format!("[{},{}]", program >> 8, program & 0xff))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{values}]")
}

fn stereo_channel_abs(audio: &[i16], channels: usize) -> (u64, u64) {
    if channels == 0 {
        return (0, 0);
    }
    let mut left = 0u64;
    let mut right = 0u64;
    for frame in audio.chunks(channels) {
        left += u64::from(frame[0].unsigned_abs());
        right += u64::from(frame.get(1).copied().unwrap_or(frame[0]).unsigned_abs());
    }
    (left, right)
}

fn modern_note_events_json(frame: &zelda3::game_output::AudioEventFrame) -> String {
    let mut pans = [0i8; 8];
    let mut origins = ["unknown"; 8];
    let mut pending_note = [None; 8];
    let mut notes: Vec<(u8, u8, u8, u8, i8, &str)> = Vec::new();
    for event in &frame.events {
        match &event.kind {
            zelda3::game_output::AudioEventKind::SetNoteOrigin { voice, origin } => {
                pending_note[usize::from(*voice)] = None;
                if let Some(slot) = origins.get_mut(usize::from(*voice)) {
                    *slot = match origin {
                        zelda3::game_output::AudioNoteOrigin::Music => "music",
                        zelda3::game_output::AudioNoteOrigin::Sfx => "sfx",
                    };
                }
            }
            zelda3::game_output::AudioEventKind::NoteOn {
                voice,
                pitch,
                instrument,
                volume,
            } => {
                let index = notes.len();
                notes.push((
                    *voice,
                    *pitch,
                    *instrument,
                    *volume,
                    pans.get(usize::from(*voice)).copied().unwrap_or_default(),
                    origins
                        .get(usize::from(*voice))
                        .copied()
                        .unwrap_or("unknown"),
                ));
                pending_note[usize::from(*voice)] = Some(index);
            }
            zelda3::game_output::AudioEventKind::SetPan { voice, pan } => {
                let voice_index = usize::from(*voice);
                pans[voice_index] = *pan;
                if let Some(index) = pending_note[voice_index] {
                    notes[index].4 = *pan;
                }
            }
            zelda3::game_output::AudioEventKind::SetDuration { voice, .. } => {
                pending_note[usize::from(*voice)] = None;
            }
            _ => {}
        }
    }
    let notes = notes
        .into_iter()
        .map(|(voice, pitch, instrument, volume, pan, origin)| {
            format!(
                "{{\"voice\":{voice},\"pitch\":{pitch},\"instrument\":{instrument},\"volume\":{volume},\"pan\":{pan},\"origin\":\"{origin}\"}}"
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("[{notes}]")
}

fn replay_dsp_write_events_json(frame: u32, writes: &[DspWriteEvent]) -> String {
    let target = env::var("ZELDA3_AUDIO_TRACE_DSP_WRITES_FRAME")
        .ok()
        .and_then(|value| value.parse::<u32>().ok());
    let range = env::var("ZELDA3_AUDIO_TRACE_DSP_WRITES_FRAME_RANGE").ok();
    if !should_write_dsp_write_events(target, range.as_deref(), frame) {
        return String::new();
    }
    let events = writes
        .iter()
        .map(|write| {
            format!(
                "[{},{},{},{}]",
                write.addr, write.value, write.sample_offset, write.timer_cycles
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(",\"dsp_write_events\":[{events}]")
}

fn should_write_dsp_write_events(target: Option<u32>, range: Option<&str>, frame: u32) -> bool {
    if target == Some(frame) {
        return true;
    }
    let Some(range) = range else {
        return false;
    };
    let Some((start, end)) = parse_dsp_write_frame_range(range) else {
        return false;
    };
    start <= frame && frame <= end
}

fn parse_dsp_write_frame_range(value: &str) -> Option<(u32, u32)> {
    let (start, end) = value
        .split_once(':')
        .or_else(|| value.split_once("..="))
        .or_else(|| value.split_once(".."))?;
    let start = start.parse::<u32>().ok()?;
    let end = end.parse::<u32>().ok()?;
    (start <= end).then_some((start, end))
}

pub(crate) fn replay_checksum_dsp_writes(writes: &[DspWriteEvent]) -> u32 {
    checksum_dsp_writes(writes)
}

pub(crate) fn replay_checksum_dsp_write_values(writes: &[DspWriteEvent]) -> u32 {
    checksum_dsp_write_values(writes)
}

/// Per-frame audio leaf hash: folds the same DSP/sample quantities the audio
/// trace prints, into one u32. Mirrored exactly in C (FingerprintAudioHash).
pub(crate) fn fingerprint_audio_hash(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_frame_filter_preserves_default_all_frames_behavior() {
        assert!(should_write_fingerprint(None, 41));
        assert!(should_write_fingerprint(Some(42), 42));
        assert!(!should_write_fingerprint(Some(42), 41));
    }

    #[test]
    fn audio_stats_reports_peak_and_first_nonzero_sample() {
        let stats = AudioFrameStats::from_interleaved_stereo(&[0, -3, 7, 1]);

        assert_eq!(stats.samples_per_channel, 2);
        assert_eq!(stats.peak, 7);
        assert_eq!(stats.first_nonzero, Some(1));
        assert_eq!(stats.mean_abs, 2);
    }

    #[test]
    fn dsp_write_event_filter_accepts_exact_frame_or_range() {
        assert!(should_write_dsp_write_events(Some(41), None, 41));
        assert!(!should_write_dsp_write_events(Some(41), None, 42));
        assert!(should_write_dsp_write_events(None, Some("10:12"), 11));
        assert!(should_write_dsp_write_events(None, Some("10..=12"), 12));
        assert!(!should_write_dsp_write_events(None, Some("10..12"), 13));
        assert!(!should_write_dsp_write_events(None, Some("12:10"), 11));
        assert!(!should_write_dsp_write_events(None, Some("bad"), 11));
    }

    #[test]
    fn sample_diff_reports_exact_and_opposite_extremes_without_overflow() {
        let diff = sample_diff(&[0, i16::MIN, 7], &[0, i16::MAX, -3]);

        assert_eq!(diff.max_abs, u16::MAX);
        assert_eq!(diff.exact_samples, 1);
        assert!((diff.mean_abs - 21848.333333).abs() < 0.000001);
    }
}
