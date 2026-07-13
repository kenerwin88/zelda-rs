use crate::game_output::{
    AudioEvent, AudioEventFrame, AudioEventKind, AudioMusicCommand, AudioNoteOrigin,
    AudioRouteState, AudioSfxBank, DspWriteEvent, EngineAudioCommandBatch, MusicControlState,
    VoiceParameterKind,
};
use crate::modern_music_catalog::{decode_note, packed_track, ModernMusicNote, PACKED_NOTE_BYTES};
use crate::modern_music_globals::events_at as music_global_events_at;
use crate::modern_sfx_catalog::{
    lookup_sfx_program_for_context, sfx_program_hash, ModernSfxProgram, ModernSfxRuntimeContext,
    ModernSfxWaveform,
};
use crate::modern_sfx_dsp_catalog::{exact_sfx_dsp_step, ExactSfxDspStep};
use crate::modern_sfx_pitch_catalog::pitch_events as exact_sfx_pitch_events;

const SFX_SLOTS: usize = 7;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct PendingSfxStep {
    step: crate::modern_sfx_catalog::ModernSfxStep,
    exact: Option<ExactSfxDspStep>,
    #[serde(default)]
    engine_dsp_envelope: Option<[u8; 3]>,
    delay_after_previous: u8,
    #[serde(default)]
    preserve_existing_volume: bool,
    #[serde(default)]
    refresh_repeat_on_keyon: bool,
    #[serde(default)]
    preserve_inactive_pitch_latch: bool,
    #[serde(default)]
    engine_keyoff_owned: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct PendingSfxPitchChange {
    samples_remaining: u32,
    pitch_word: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct LoopingSfxVoice {
    step: crate::modern_sfx_catalog::ModernSfxStep,
    exact: ExactSfxDspStep,
    overflows_remaining: u16,
    active: bool,
    active_overflows: u16,
    gap_overflows: u16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ModernAudioSequenceStats {
    pub music_commands: u32,
    pub sfx_commands: u32,
    pub note_events: u32,
    pub envelope_events: u32,
    pub ignored_commands: u32,
    pub known_sfx_commands: u32,
    pub unknown_sfx_commands: u32,
    pub fallback_sfx_commands: u32,
    #[serde(default)]
    pub exact_sfx_steps: u32,
    pub program_hash: u32,
    #[serde(default)]
    pub known_sfx_programs: [u16; SFX_SLOTS],
    #[serde(default)]
    pub unknown_sfx_programs: [u16; SFX_SLOTS],
    #[serde(default)]
    pub known_sfx_program_count: u8,
    #[serde(default)]
    pub unknown_sfx_program_count: u8,
    #[serde(default)]
    pub active_voice_mask: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ModernAudioSequencer {
    last_music_track: u8,
    #[serde(default)]
    music_frame_position: u16,
    last_sfx: [u8; SFX_SLOTS],
    active_voice_mask: u8,
    #[serde(default)]
    music_voice_mask: u8,
    #[serde(default)]
    music_keyoff_frames_remaining: [u16; 8],
    #[serde(default)]
    music_keyoff_sample_offset: [u16; 8],
    #[serde(default)]
    voice_frames_remaining: [u16; 8],
    #[serde(default)]
    pending_voice_steps: [Vec<PendingSfxStep>; 8],
    #[serde(default)]
    sfx_keyoff_samples_remaining: [u32; 8],
    #[serde(default)]
    pending_sfx_pitch_changes: [Vec<PendingSfxPitchChange>; 8],
    #[serde(default)]
    looping_sfx_voices: [Option<LoopingSfxVoice>; 8],
    #[serde(default)]
    previous_sfx_clock: Option<(u8, u8)>,
    #[serde(default)]
    semantic_sfx_pending_steps: [Vec<PendingSfxStep>; 8],
    #[serde(default)]
    semantic_sfx_repeat_steps: [Option<PendingSfxStep>; 8],
    #[serde(default)]
    semantic_pitch_latch_mask: u8,
    #[serde(default)]
    engine_automated_sfx_volume_mask: u8,
    #[serde(default)]
    semantic_bank1_command_voice: Option<u8>,
    #[serde(default)]
    semantic_bank2_command_voice: Option<u8>,
    #[serde(default)]
    engine_receipt_mode: bool,
    #[serde(default)]
    dsp_timer_cycles: u8,
    #[serde(default)]
    timer_initialized: bool,
    last_stats: ModernAudioSequenceStats,
}

impl Default for ModernAudioSequencer {
    fn default() -> Self {
        Self {
            last_music_track: 0,
            music_frame_position: 0,
            last_sfx: [0; SFX_SLOTS],
            active_voice_mask: 0,
            music_voice_mask: 0,
            music_keyoff_frames_remaining: [0; 8],
            music_keyoff_sample_offset: [0; 8],
            voice_frames_remaining: [0; 8],
            pending_voice_steps: std::array::from_fn(|_| Vec::new()),
            sfx_keyoff_samples_remaining: [0; 8],
            pending_sfx_pitch_changes: std::array::from_fn(|_| Vec::new()),
            looping_sfx_voices: [None; 8],
            previous_sfx_clock: None,
            semantic_sfx_pending_steps: std::array::from_fn(|_| Vec::new()),
            semantic_sfx_repeat_steps: [None; 8],
            semantic_pitch_latch_mask: 0,
            engine_automated_sfx_volume_mask: 0,
            semantic_bank1_command_voice: None,
            semantic_bank2_command_voice: None,
            engine_receipt_mode: false,
            dsp_timer_cycles: 0,
            timer_initialized: false,
            last_stats: ModernAudioSequenceStats::default(),
        }
    }
}

impl ModernAudioSequencer {
    /// Compatibility entry point for oracle traces and older callers that only
    /// expose APUI state. The playable runtime uses `sequence_engine_commands`.
    pub fn sequence_route(&mut self, route: AudioRouteState) -> AudioEventFrame {
        let commands = EngineAudioCommandBatch::from_legacy_ports(route.queue.input);
        self.sequence_commands_with_writes(route, commands, &[])
    }

    /// Expand gameplay-authored commands without decoding the APUI projection.
    pub fn sequence_engine_commands(
        &mut self,
        route: AudioRouteState,
        commands: EngineAudioCommandBatch,
    ) -> AudioEventFrame {
        self.sequence_commands_with_writes(route, commands, &[])
    }

    pub fn sequence_parity_writes(
        &mut self,
        route: AudioRouteState,
        writes: &[DspWriteEvent],
    ) -> AudioEventFrame {
        let commands = EngineAudioCommandBatch::from_legacy_ports(route.queue.input);
        self.sequence_commands_with_writes(route, commands, writes)
    }

    fn sequence_commands_with_writes(
        &mut self,
        route: AudioRouteState,
        commands: EngineAudioCommandBatch,
        writes: &[DspWriteEvent],
    ) -> AudioEventFrame {
        let ambient_command = commands
            .sfx(AudioSfxBank::Ambient)
            .map_or(0, |command| command.legacy_value());
        if ambient_command == 0x05 && self.last_sfx[0] != 0x05 {
            for voice in 0..8 {
                self.cancel_sfx_schedules(voice);
            }
        }
        self.semantic_bank1_command_voice = semantic_bank1_allocator_voice(route, commands);
        self.semantic_bank2_command_voice = semantic_bank2_allocator_voice(route, commands);
        if let Some(spc) = route.spc {
            self.engine_receipt_mode |= spc.sfx_kon_count != 0
                || spc.raw_kof_count != 0
                || spc.raw_pitch_count != 0
                || spc.raw_volume_count != 0
                || spc.raw_envelope_count != 0;
            self.semantic_pitch_latch_mask &= spc.port2_active;
        }
        self.initialize_timer(route);
        let mut frame = AudioEventFrame::from_route_and_dsp_writes(route, writes);
        frame.sequenced = true;
        let mut stats = ModernAudioSequenceStats::default();
        let mut processed_semantic_keyons = [0u8; 8];
        let mut semantic_pitch_latch_release = [None; 8];
        self.advance_music_keyoffs(&mut frame, &mut stats);
        self.advance_sfx_pitch_changes(&mut frame);
        self.emit_semantic_sfx_echo_changes(route.spc, &mut frame);
        self.emit_semantic_sfx_keyons(
            route.spc,
            &mut processed_semantic_keyons,
            &mut semantic_pitch_latch_release,
            &mut frame,
            &mut stats,
        );
        self.advance_voice_lifetimes(&mut frame, &mut stats);

        self.sequence_music(route.music, commands.music(), &mut frame, &mut stats);
        self.emit_ambient_music_reset(route, commands, &mut frame, &mut stats);
        self.sequence_sfx(
            route.music,
            commands,
            route.spc.map_or(0, |spc| spc.is_chan_on),
            route.spc.map(|spc| (spc.timer_cycles, spc.sfx_timer_accum)),
            &mut frame,
            &mut stats,
        );
        self.emit_semantic_sfx_keyons(
            route.spc,
            &mut processed_semantic_keyons,
            &mut semantic_pitch_latch_release,
            &mut frame,
            &mut stats,
        );
        self.emit_engine_music_keyons(route.spc, &mut frame, &mut stats);
        self.emit_semantic_sfx_pitch_changes(route.spc, &mut frame);
        self.emit_raw_pitch_changes(route.spc, semantic_pitch_latch_release, &mut frame);
        self.emit_raw_echo_enable_changes(route.spc, &mut frame);
        self.emit_raw_echo_volume_changes(route.spc, &mut frame);
        self.emit_raw_global_changes(route.spc, &mut frame);
        self.emit_semantic_sfx_volume_changes(route.spc, &mut frame);
        self.emit_raw_volume_changes(route.spc, &mut frame);
        self.emit_raw_envelope_changes(route.spc, &mut frame);
        self.emit_raw_music_keyoffs(route.spc, &mut frame, &mut stats);
        self.reconcile_semantic_sfx_keyoffs(route.spc, &mut frame);

        stats.active_voice_mask = self.active_voice_mask;
        self.last_stats = stats;
        self.advance_dsp_timer();
        self.previous_sfx_clock = route.spc.map(|spc| (spc.timer_cycles, spc.sfx_timer_accum));
        frame
    }

    pub fn last_stats(&self) -> ModernAudioSequenceStats {
        self.last_stats
    }

    fn initialize_timer(&mut self, route: AudioRouteState) {
        if self.timer_initialized {
            return;
        }
        self.dsp_timer_cycles = route.spc.map_or(0, |spc| {
            spc.timer_cycles.wrapping_sub((534 & 0x3f) as u8) & 0x3f
        });
        self.timer_initialized = true;
    }

    fn advance_dsp_timer(&mut self) {
        self.dsp_timer_cycles = self.dsp_timer_cycles.wrapping_add((534 & 0x3f) as u8) & 0x3f;
    }

    fn sfx_sample_offset(&self, exact: Option<ExactSfxDspStep>) -> i32 {
        let Some(exact) = exact else {
            return 0;
        };
        let first_boundary = self.dsp_timer_cycles.wrapping_neg() & 0x3f;
        i32::from(first_boundary) + i32::from(exact.scheduler_tick_index) * 64
    }

    fn sequence_music(
        &mut self,
        music: MusicControlState,
        command: AudioMusicCommand,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        let command_track = command.legacy_value();
        let track = first_nonzero([
            command_track,
            music.apui00,
            music.last_music_control,
            music.queued_music_control,
            music.music_control,
        ]);
        if track == self.last_music_track {
            self.advance_music_sequence(track, frame, stats);
            return;
        }

        if track == 0 || track == 0xf0 {
            if self.last_music_track != 0 {
                push_event(frame, AudioEventKind::StopMusic);
                self.release_music_voices(frame, stats);
                stats.music_commands += 1;
            }
            self.last_music_track = 0;
            self.music_frame_position = 0;
            return;
        }

        if track >= 0xf1 {
            stats.ignored_commands += 1;
            if self.last_music_track != 0 && self.last_music_track < 0xf0 {
                self.advance_music_sequence(self.last_music_track, frame, stats);
            }
            return;
        }

        push_event(frame, AudioEventKind::PlayMusic { track });
        push_event(
            frame,
            AudioEventKind::SetTempo {
                value: tempo_for_track(track),
            },
        );
        stats.music_commands += 1;
        self.last_music_track = track;
        if packed_track(track).is_some() {
            self.music_frame_position = 0;
            self.emit_music_globals_at_position(track, 0, frame);
            self.emit_music_notes_at_position(track, 0, frame, stats);
        } else {
            self.emit_music_note(
                frame,
                track,
                ModernMusicNote {
                    voice: 0,
                    pitch: pitch_for_code(track),
                    instrument: instrument_for_code(track),
                    volume: 88,
                    pan: 0,
                    start_frame: 0,
                    duration_frames: 0,
                    dsp_pitch: 0,
                    sample_offset: 0,
                    volume_left: 0,
                    volume_right: 0,
                    adsr1: 0,
                    adsr2: 0,
                    gain: 0,
                    echo_send: false,
                    keyoff_sample_offset: 0,
                },
                stats,
            );
        }
    }

    fn advance_music_sequence(
        &mut self,
        track: u8,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        self.music_frame_position = self.music_frame_position.saturating_add(1);
        self.emit_music_globals_at_position(track, self.music_frame_position, frame);
        self.emit_music_notes_at_position(track, self.music_frame_position, frame, stats);
    }

    fn emit_music_globals_at_position(
        &self,
        track: u8,
        music_frame_position: u16,
        frame: &mut AudioEventFrame,
    ) {
        if self.engine_receipt_mode {
            // Route-derived music register catalogs are a standalone fallback.
            // Once the engine is providing timed DSP receipts, replaying these
            // observations can mutate active envelopes on frames where the
            // engine made no register write at all.
            return;
        }
        for event in music_global_events_at(track, music_frame_position) {
            let kind = match event.register & 0x0f {
                0x00 => Some(VoiceParameterKind::VolumeLeft),
                0x01 => Some(VoiceParameterKind::VolumeRight),
                0x02 => Some(VoiceParameterKind::PitchLow),
                0x03 => Some(VoiceParameterKind::PitchHigh),
                0x04 => Some(VoiceParameterKind::Source),
                0x05 => Some(VoiceParameterKind::Adsr1),
                0x06 => Some(VoiceParameterKind::Adsr2),
                0x07 => Some(VoiceParameterKind::Gain),
                _ => None,
            };
            push_event_at(
                frame,
                i32::from(event.sample_offset),
                if let Some(parameter) = kind {
                    AudioEventKind::VoiceParameter {
                        voice: event.register >> 4,
                        parameter,
                        value: event.value,
                    }
                } else {
                    AudioEventKind::GlobalParameter {
                        register: event.register,
                        value: event.value,
                    }
                },
            );
        }
    }

    fn emit_music_notes_at_position(
        &mut self,
        track: u8,
        music_frame_position: u16,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        let Some(track_data) = packed_track(track) else {
            return;
        };
        for note in track_data
            .notes
            .chunks_exact(PACKED_NOTE_BYTES)
            .filter_map(decode_note)
            .filter(|note| note.start_frame + track_data.lead_in_frames == music_frame_position)
        {
            self.emit_music_note(frame, track, note, stats);
        }
    }

    fn emit_music_note(
        &mut self,
        frame: &mut AudioEventFrame,
        track: u8,
        note: ModernMusicNote,
        stats: &mut ModernAudioSequenceStats,
    ) {
        if self.engine_receipt_mode {
            self.mark_music_voice_active(note.voice);
            stats.note_events += 1;
            return;
        }
        push_event(
            frame,
            AudioEventKind::SetNoteOrigin {
                voice: note.voice,
                origin: AudioNoteOrigin::Music,
            },
        );
        push_event(
            frame,
            AudioEventKind::SetEnvelope {
                voice: note.voice,
                attack: 2,
                decay: 4 + (track & 3),
                sustain: 10,
                release: 4,
            },
        );
        if note.dsp_pitch != 0 {
            let sample_offset = i32::from(note.sample_offset);
            push_event_at(
                frame,
                sample_offset,
                AudioEventKind::SetPitchWord {
                    voice: note.voice,
                    pitch_word: note.dsp_pitch,
                },
            );
            push_event_at(
                frame,
                sample_offset,
                AudioEventKind::SetStereoVolume {
                    voice: note.voice,
                    left: note.volume_left,
                    right: note.volume_right,
                },
            );
            push_event_at(
                frame,
                sample_offset,
                AudioEventKind::SetDspEnvelope {
                    voice: note.voice,
                    adsr1: note.adsr1,
                    adsr2: note.adsr2,
                    gain: note.gain,
                },
            );
            push_event_at(
                frame,
                sample_offset,
                AudioEventKind::SetEchoSend {
                    voice: note.voice,
                    enabled: note.echo_send,
                },
            );
        }
        push_event_at(
            frame,
            i32::from(note.sample_offset),
            AudioEventKind::NoteOn {
                voice: note.voice,
                pitch: note.pitch,
                instrument: note.instrument,
                volume: note.volume,
            },
        );
        push_event(
            frame,
            AudioEventKind::SetPan {
                voice: note.voice,
                pan: note.pan,
            },
        );
        if note.duration_frames != 0 && note.dsp_pitch == 0 {
            push_event(
                frame,
                AudioEventKind::SetDuration {
                    voice: note.voice,
                    frames: note.duration_frames,
                },
            );
        }
        self.mark_music_voice_active(note.voice);
        if note.dsp_pitch != 0
            && note.duration_frames == 0
            && note.keyoff_sample_offset > note.sample_offset
        {
            push_event_at(
                frame,
                i32::from(note.keyoff_sample_offset),
                AudioEventKind::NoteOff { voice: note.voice },
            );
            self.mark_music_voice_inactive(note.voice);
            stats.note_events += 1;
        } else if note.dsp_pitch != 0 && note.duration_frames != 0 {
            let voice = usize::from(note.voice);
            self.music_keyoff_frames_remaining[voice] = note.duration_frames;
            self.music_keyoff_sample_offset[voice] = note.keyoff_sample_offset;
        }
        stats.note_events += 1;
        stats.envelope_events += 1;
    }

    fn sequence_sfx(
        &mut self,
        music: MusicControlState,
        commands: EngineAudioCommandBatch,
        sfx_voice_mask: u8,
        sfx_clock: Option<(u8, u8)>,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        let engine_sfx = AudioSfxBank::ALL.map(|bank| {
            commands
                .sfx(bank)
                .map_or(0, |command| command.legacy_value())
        });
        let candidates = [
            engine_sfx[0],
            engine_sfx[1],
            engine_sfx[2],
            music.sound_effect_ambient,
            music.sound_effect_1,
            music.sound_effect_2,
            // APUI00 is music port 0 and is consumed by sequence_music above.
            // Keep the seventh slot zeroed for checkpoint compatibility.
            0,
        ];

        for (slot, code) in candidates.into_iter().enumerate() {
            if code == self.last_sfx[slot] {
                continue;
            }
            let voice = (slot + 1).min(7) as u8;
            if code == 0 {
                // A command slot is not a DSP voice. Exact programs own their
                // actual voice lifetimes, and fallback notes carry bounded
                // durations. Mapping a cleared port back to `slot + 1` can key
                // off an unrelated music channel.
                self.last_sfx[slot] = 0;
                continue;
            }

            push_event(
                frame,
                AudioEventKind::PlaySfx {
                    bank: slot as u8,
                    id: code,
                },
            );

            if slot == 0 && code == 0x05 {
                for voice in 0..8 {
                    self.cancel_sfx_schedules(voice);
                }
            }
            if let Some(program) = lookup_sfx_program_for_context(
                slot as u8,
                code,
                ModernSfxRuntimeContext {
                    source_slot: slot as u8,
                    active_voice_mask: self.active_voice_mask & !sfx_voice_mask,
                },
            ) {
                self.expand_sfx_program(frame, program, sfx_clock, stats);
                stats.known_sfx_commands += 1;
                record_sfx_program(
                    &mut stats.known_sfx_programs,
                    &mut stats.known_sfx_program_count,
                    slot as u8,
                    code,
                );
                stats.program_hash =
                    fold_program_hash(stats.program_hash, sfx_program_hash(program));
            } else {
                self.expand_fallback_sfx(frame, voice, code, slot as u8, stats);
                stats.unknown_sfx_commands += 1;
                record_sfx_program(
                    &mut stats.unknown_sfx_programs,
                    &mut stats.unknown_sfx_program_count,
                    slot as u8,
                    code,
                );
                stats.fallback_sfx_commands += 1;
            }
            stats.sfx_commands += 1;
            self.last_sfx[slot] = code;
        }
    }

    fn emit_ambient_music_reset(
        &mut self,
        route: AudioRouteState,
        commands: EngineAudioCommandBatch,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        let ambient_command = commands
            .sfx(AudioSfxBank::Ambient)
            .map_or(0, |command| command.legacy_value());
        let should_reset = ambient_command == 0x03
            && route
                .spc
                .is_some_and(|spc| spc.spc_out[0] == 0 && spc.block_count == 0xff);
        if !should_reset {
            return;
        }
        let restore_offset = route.spc.map_or(0, |spc| {
            if spc.echo_enable_count != 0 {
                spc.echo_enable_offsets[0]
            } else {
                u16::from(spc.timer_cycles.wrapping_neg() & 0x3f)
            }
        });
        push_event_at(frame, 0, AudioEventKind::ResetEchoVolume { restore_offset });
        let active_voices = self.active_voice_mask | route.spc.map_or(0, |spc| spc.is_chan_on);
        for voice in 0..8 {
            if active_voices & (1 << voice) == 0 {
                continue;
            }
            push_event_at(frame, 0, AudioEventKind::NoteOff { voice: voice as u8 });
            self.active_voice_mask &= !(1 << voice);
            self.music_voice_mask &= !(1 << voice);
            self.music_keyoff_frames_remaining[voice] = 0;
            stats.note_events += 1;
        }
    }

    fn expand_sfx_program(
        &mut self,
        frame: &mut AudioEventFrame,
        program: &ModernSfxProgram,
        sfx_clock: Option<(u8, u8)>,
        stats: &mut ModernAudioSequenceStats,
    ) {
        if self.engine_receipt_mode {
            // Catalog voices are harvested routing observations.
            // Runtime setup/KON/KOF and parameter receipts contain the
            // authoritative allocated voice, envelope, pitch, volume, echo,
            // and timing, so these catalogs remain metadata-only at runtime.
            return;
        }
        let mut interrupted_voices = 0u8;
        let mut voice_timeline_end = [0u16; 8];
        for (step_index, catalog_step) in program.steps.iter().enumerate() {
            let mut step = *catalog_step;
            if program.bank == 1 && matches!(program.id, 0x1c | 0x5e) {
                step.voice = self.semantic_bank1_command_voice.unwrap_or(step.voice);
            }
            if step.voice >= 8 {
                continue;
            }
            let voice_bit = 1 << step.voice;
            let first_for_voice = interrupted_voices & voice_bit == 0;
            if first_for_voice {
                if !uses_semantic_sfx_keyons(program)
                    || (program.bank == 0
                        && program.id == 0x01
                        && program.variant_hash == 0x6f23aa01)
                {
                    self.interrupt_voice(step.voice);
                }
                interrupted_voices |= voice_bit;
            }
            if program.bank == 1 && program.id == 0x5e {
                self.engine_automated_sfx_volume_mask |= voice_bit;
            }
            if program.bank == 2 && program.id == 0x1f {
                self.engine_automated_sfx_volume_mask |= voice_bit;
            }
            if program.bank == 1 && program.id == 0x1e {
                self.engine_automated_sfx_volume_mask |= voice_bit;
            }
            if program.bank == 0 && program.id == 0x05 && program.variant_hash == 0x5c065005 {
                self.engine_automated_sfx_volume_mask |= voice_bit;
            }
            let voice = usize::from(step.voice);
            let exact_shape = if program.bank == 1 && matches!(program.id, 0x1c | 0x5e) {
                step
            } else {
                *catalog_step
            };
            let mut exact = exact_sfx_dsp_step(
                program.bank,
                program.id,
                program.variant_hash,
                step_index,
                exact_shape,
            );
            if let (
                Some(exact),
                Some((timer_cycles, sfx_timer_accum)),
                Some((overflow_index, keyoff_overflows)),
            ) = (
                exact.as_mut(),
                sfx_clock,
                exact_sfx_clock_timing(program, step_index),
            ) {
                let (frame_delta, scheduler_tick_index) =
                    sfx_clock_target(timer_cycles, sfx_timer_accum, overflow_index);
                exact.command_delay_frames = frame_delta;
                exact.scheduler_tick_index = scheduler_tick_index;
                let (key_frame, _, key_offset) =
                    sfx_clock_target_position(timer_cycles, sfx_timer_accum, overflow_index);
                let (keyoff_frame, _, keyoff_offset) = sfx_clock_target_position(
                    timer_cycles,
                    sfx_timer_accum,
                    overflow_index + keyoff_overflows,
                );
                exact.duration_samples = (u32::from(keyoff_frame - key_frame) * 534)
                    .saturating_add(u32::from(keyoff_offset))
                    .saturating_sub(u32::from(key_offset));
            }
            let start = exact.map_or(voice_timeline_end[voice], |exact| {
                u16::from(exact.command_delay_frames)
            });
            let delay_after_previous = start.saturating_sub(voice_timeline_end[voice]) as u8;
            voice_timeline_end[voice] = start.saturating_add(u16::from(step.duration_frames));
            let pending = PendingSfxStep {
                step,
                exact,
                engine_dsp_envelope: None,
                delay_after_previous,
                preserve_existing_volume: false,
                refresh_repeat_on_keyon: program.bank == 0
                    && program.id == 0x05
                    && program.variant_hash == 0x5c065005,
                preserve_inactive_pitch_latch: program.bank == 1 && program.id == 0x1d,
                engine_keyoff_owned: program.bank == 1
                    && program.id == 0x2b
                    && program.variant_hash == 0x4b866332,
            };
            if program.bank == 1
                && program.id == 0x1d
                && program.variant_hash == 0xf31c8f91
                && step_index == 0
            {
                self.semantic_sfx_repeat_steps[voice] = Some(pending);
            }
            if program.bank == 1
                && program.id == 0x36
                && program.variant_hash == 0x102e5506
                && step_index == 0
            {
                self.semantic_sfx_repeat_steps[voice] = Some(pending);
            }
            if program.bank == 0 && program.id == 0x01 && program.variant_hash == 0x6f23aa01 {
                self.semantic_sfx_repeat_steps[voice] = Some(pending);
            }
            if program.bank == 2 && program.id == 0x1b && program.variant_hash == 0xa44764fc {
                self.semantic_sfx_repeat_steps[voice] = Some(pending);
            }
            if program.bank == 0 && program.id == 0x05 && program.variant_hash == 0x5c065005 {
                self.semantic_sfx_repeat_steps[voice] = Some(pending);
            }
            if let (Some(exact), Some((initial_overflow, active_overflows, gap_overflows))) =
                (exact, looping_sfx_timing(program, step_index))
            {
                self.looping_sfx_voices[voice] = Some(LoopingSfxVoice {
                    step,
                    exact: ExactSfxDspStep {
                        duration_samples: 0,
                        ..exact
                    },
                    overflows_remaining: initial_overflow + active_overflows,
                    active: true,
                    active_overflows,
                    gap_overflows,
                });
            }
            if self.engine_receipt_mode && uses_semantic_sfx_keyons(program) {
                let allocator_voice = if program.bank == 2
                    && matches!(
                        program.id,
                        0x04 | 0x08
                            | 0x09
                            | 0x0b
                            | 0x0c
                            | 0x0d
                            | 0x0e
                            | 0x0f
                            | 0x11
                            | 0x14
                            | 0x15
                            | 0x16
                            | 0x17
                            | 0x1c
                            | 0x24
                            | 0x31
                            | 0x44
                            | 0x49
                            | 0x4b
                            | 0x57
                            | 0x5c
                            | 0x89
                            | 0x8b
                            | 0x97
                    ) {
                    self.semantic_bank2_command_voice
                } else if program.bank == 1
                    && (matches!(
                        program.id,
                        0x01 | 0x05
                            | 0x16
                            | 0x17
                            | 0x18
                            | 0x19
                            | 0x1d
                            | 0x20
                            | 0x22
                            | 0x26
                            | 0x29
                            | 0x2a
                            | 0x2d
                            | 0x3c
                            | 0x41
                            | 0x45
                            | 0x56
                            | 0x57
                            | 0x5f
                            | 0x6a
                            | 0x81
                            | 0x85
                            | 0x96
                            | 0x97
                            | 0x9e
                            | 0x9f
                    ) || (program.id == 0x2b && program.variant_hash == 0x4b866332))
                {
                    self.semantic_bank1_command_voice
                } else {
                    None
                };
                if let Some(allocator_voice) = allocator_voice {
                    let target_voice = usize::from(allocator_voice);
                    let mut pending = PendingSfxStep {
                        delay_after_previous: 0,
                        ..pending
                    };
                    pending.step.voice = target_voice as u8;
                    if pending.preserve_inactive_pitch_latch {
                        self.semantic_pitch_latch_mask |= 1 << target_voice;
                    }
                    if first_for_voice {
                        self.semantic_sfx_pending_steps[target_voice].clear();
                    }
                    self.semantic_sfx_pending_steps[target_voice].push(pending);
                } else {
                    if pending.preserve_inactive_pitch_latch {
                        self.semantic_pitch_latch_mask |= 1 << voice;
                    }
                    if first_for_voice {
                        self.semantic_sfx_pending_steps[voice].clear();
                    }
                    self.semantic_sfx_pending_steps[voice].push(PendingSfxStep {
                        delay_after_previous: 0,
                        ..pending
                    });
                }
                continue;
            }
            if first_for_voice && pending.delay_after_previous != 0 {
                self.voice_frames_remaining[voice] = u16::from(pending.delay_after_previous);
                let mut pending = pending;
                pending.delay_after_previous = 0;
                self.pending_voice_steps[voice].push(pending);
            } else if self.voice_frames_remaining[voice] == 0
                && self.pending_voice_steps[voice].is_empty()
            {
                self.emit_sfx_step(frame, pending, stats);
            } else {
                self.pending_voice_steps[voice].push(pending);
            }
        }
    }

    fn expand_fallback_sfx(
        &mut self,
        frame: &mut AudioEventFrame,
        voice: u8,
        code: u8,
        slot: u8,
        stats: &mut ModernAudioSequenceStats,
    ) {
        self.interrupt_voice(voice);
        push_event(
            frame,
            AudioEventKind::SetNoteOrigin {
                voice,
                origin: AudioNoteOrigin::Sfx,
            },
        );
        push_event(
            frame,
            AudioEventKind::SetEnvelope {
                voice,
                attack: 1,
                decay: 2 + (code & 7),
                sustain: 8,
                release: 2,
            },
        );
        push_event(
            frame,
            AudioEventKind::NoteOn {
                voice,
                pitch: pitch_for_code(code).saturating_add(slot),
                instrument: instrument_for_code(code),
                volume: 112,
            },
        );
        self.mark_voice_active(voice);
        push_event(frame, AudioEventKind::SetDuration { voice, frames: 6 });
        self.extend_voice_lifetime(voice, 6);
        stats.note_events += 1;
        stats.envelope_events += 1;
    }

    fn mark_voice_active(&mut self, voice: u8) {
        if voice < 8 {
            self.active_voice_mask |= 1 << voice;
        }
    }

    fn mark_music_voice_active(&mut self, voice: u8) {
        if voice < 8 {
            self.music_voice_mask |= 1 << voice;
            self.active_voice_mask |= 1 << voice;
        }
    }

    fn mark_voice_inactive(&mut self, voice: u8) {
        if voice < 8 {
            self.active_voice_mask &= !(1 << voice);
            self.voice_frames_remaining[usize::from(voice)] = 0;
            self.pending_voice_steps[usize::from(voice)].clear();
            self.sfx_keyoff_samples_remaining[usize::from(voice)] = 0;
            self.pending_sfx_pitch_changes[usize::from(voice)].clear();
        }
    }

    fn mark_music_voice_inactive(&mut self, voice: u8) {
        if voice < 8 {
            let voice = usize::from(voice);
            self.music_voice_mask &= !(1 << voice);
            self.music_keyoff_frames_remaining[voice] = 0;
            if self.voice_frames_remaining[voice] == 0
                && self.pending_voice_steps[voice].is_empty()
                && self.sfx_keyoff_samples_remaining[voice] == 0
            {
                self.active_voice_mask &= !(1 << voice);
            }
        }
    }

    fn release_music_voices(
        &mut self,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        let music_voices = self.music_voice_mask;
        for voice in 0..8 {
            if music_voices & (1 << voice) == 0 {
                continue;
            }
            push_event(frame, AudioEventKind::NoteOff { voice });
            self.mark_music_voice_inactive(voice);
            stats.note_events += 1;
        }
    }

    fn interrupt_voice(&mut self, voice: u8) {
        if voice < 8 {
            self.active_voice_mask &= !(1 << voice);
            self.voice_frames_remaining[usize::from(voice)] = 0;
            self.pending_voice_steps[usize::from(voice)].clear();
            self.sfx_keyoff_samples_remaining[usize::from(voice)] = 0;
            self.pending_sfx_pitch_changes[usize::from(voice)].clear();
            self.looping_sfx_voices[usize::from(voice)] = None;
            self.semantic_sfx_pending_steps[usize::from(voice)].clear();
            self.engine_automated_sfx_volume_mask &= !(1 << voice);
        }
    }

    fn cancel_sfx_schedules(&mut self, voice: u8) {
        if voice >= 8 {
            return;
        }
        let voice = usize::from(voice);
        self.voice_frames_remaining[voice] = 0;
        self.pending_voice_steps[voice].clear();
        self.sfx_keyoff_samples_remaining[voice] = 0;
        self.pending_sfx_pitch_changes[voice].clear();
        self.looping_sfx_voices[voice] = None;
        self.semantic_sfx_pending_steps[voice].clear();
        if voice < 6 {
            self.semantic_sfx_repeat_steps[voice] = None;
        }
        self.engine_automated_sfx_volume_mask &= !(1 << voice);
    }

    fn extend_voice_lifetime(&mut self, voice: u8, frames: u16) {
        if voice < 8 {
            let remaining = &mut self.voice_frames_remaining[usize::from(voice)];
            *remaining = remaining.saturating_add(frames);
        }
    }

    fn advance_voice_lifetimes(
        &mut self,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        for voice in 0..self.voice_frames_remaining.len() {
            let remaining = &mut self.voice_frames_remaining[voice];
            if *remaining == 0 {
                continue;
            }
            *remaining -= 1;
            if *remaining == 0 {
                if self.pending_voice_steps[voice].is_empty() {
                    self.active_voice_mask &= !(1 << voice);
                } else {
                    let delay = self.pending_voice_steps[voice][0].delay_after_previous;
                    if delay != 0 {
                        self.voice_frames_remaining[voice] = u16::from(delay);
                        self.pending_voice_steps[voice][0].delay_after_previous = 0;
                    } else {
                        let pending = self.pending_voice_steps[voice].remove(0);
                        self.emit_sfx_step(frame, pending, stats);
                    }
                }
            }
        }
    }

    fn advance_sfx_keyoffs(
        &mut self,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        for voice in 0..self.sfx_keyoff_samples_remaining.len() {
            let remaining = &mut self.sfx_keyoff_samples_remaining[voice];
            if *remaining == 0 {
                continue;
            }
            *remaining = remaining.saturating_sub(534);
            if *remaining <= 534 {
                push_event_at(
                    frame,
                    *remaining as i32,
                    AudioEventKind::NoteOff { voice: voice as u8 },
                );
                *remaining = 0;
                stats.note_events += 1;
            }
        }
    }

    fn advance_music_keyoffs(
        &mut self,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        for voice in 0..self.music_keyoff_frames_remaining.len() {
            let remaining = &mut self.music_keyoff_frames_remaining[voice];
            if *remaining == 0 {
                continue;
            }
            *remaining -= 1;
            if *remaining == 0 {
                push_event_at(
                    frame,
                    i32::from(self.music_keyoff_sample_offset[voice]),
                    AudioEventKind::NoteOff { voice: voice as u8 },
                );
                // Keep ownership latched for the track. A KOF and the next
                // music KON can share a DSP sample; clearing this mask after
                // sequencing would lose the following note's later KOF.
                stats.note_events += 1;
            }
        }
    }

    fn advance_sfx_pitch_changes(&mut self, frame: &mut AudioEventFrame) {
        for voice in 0..self.pending_sfx_pitch_changes.len() {
            let changes = &mut self.pending_sfx_pitch_changes[voice];
            let mut index = 0;
            while index < changes.len() {
                changes[index].samples_remaining =
                    changes[index].samples_remaining.saturating_sub(534);
                // Offset 534 is the first sample of the following native DSP
                // frame, not the final sample of the current one.
                if changes[index].samples_remaining < 534 {
                    let change = changes.remove(index);
                    push_event_at(
                        frame,
                        change.samples_remaining as i32,
                        AudioEventKind::SetPitchRegisterWord {
                            voice: voice as u8,
                            pitch_word: change.pitch_word,
                        },
                    );
                } else {
                    index += 1;
                }
            }
        }
    }

    fn advance_looping_sfx(
        &mut self,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        let Some((timer_cycles, mut sfx_timer_accum)) = self.previous_sfx_clock else {
            return;
        };
        let first_boundary = timer_cycles.wrapping_neg() & 0x3f;
        let mut sample_offset = u16::from(first_boundary);
        while sample_offset < 534 {
            let sum = u16::from(sfx_timer_accum) + 0x38;
            sfx_timer_accum = sum as u8;
            if sum >= 0x100 {
                let mut retriggers = Vec::new();
                for voice in 0..self.looping_sfx_voices.len() {
                    let Some(looping) = self.looping_sfx_voices[voice].as_mut() else {
                        continue;
                    };
                    looping.overflows_remaining = looping.overflows_remaining.saturating_sub(1);
                    if looping.overflows_remaining != 0 {
                        continue;
                    }
                    if looping.active {
                        push_event_at(
                            frame,
                            i32::from(sample_offset),
                            AudioEventKind::NoteOff { voice: voice as u8 },
                        );
                        looping.active = false;
                        looping.overflows_remaining = looping.gap_overflows;
                        stats.note_events += 1;
                    } else {
                        retriggers.push(PendingSfxStep {
                            step: looping.step,
                            exact: Some(looping.exact),
                            engine_dsp_envelope: None,
                            delay_after_previous: 0,
                            preserve_existing_volume: false,
                            refresh_repeat_on_keyon: false,
                            preserve_inactive_pitch_latch: false,
                            engine_keyoff_owned: false,
                        });
                        looping.active = true;
                        looping.overflows_remaining = looping.active_overflows;
                    }
                }
                for pending in retriggers {
                    self.emit_sfx_step_at(frame, pending, i32::from(sample_offset), stats);
                }
            }
            sample_offset += 64;
        }
    }

    fn emit_semantic_sfx_echo_changes(
        &mut self,
        spc: Option<crate::game_output::SpcSequencerState>,
        frame: &mut AudioEventFrame,
    ) {
        let Some(spc) = spc else {
            return;
        };
        for event_index in 0..usize::from(spc.sfx_echo_count.min(8)) {
            let mask = spc.sfx_echo_masks[event_index];
            let enabled = spc.sfx_echo_enabled[event_index];
            let sample_offset = i32::from(spc.sfx_echo_offsets[event_index]);
            for voice in 0..8 {
                if mask & (1 << voice) != 0 {
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::SetEchoSend {
                            voice: voice as u8,
                            enabled,
                        },
                    );
                }
            }
        }
    }

    fn emit_semantic_sfx_pitch_changes(
        &mut self,
        spc: Option<crate::game_output::SpcSequencerState>,
        frame: &mut AudioEventFrame,
    ) {
        let Some(spc) = spc else {
            return;
        };
        for event_index in 0..usize::from(spc.sfx_pitch_count.min(32)) {
            let mask = spc.sfx_pitch_masks[event_index];
            let pitch_word = spc.sfx_pitch_words[event_index];
            let sample_offset = i32::from(spc.sfx_pitch_offsets[event_index]);
            for voice in 0..8 {
                if mask & (1 << voice) != 0 {
                    frame.events.retain(|event| {
                        !(event.sample_offset >= sample_offset
                            && matches!(
                                event.kind,
                                AudioEventKind::SetPitchWord { voice: event_voice, .. }
                                    | AudioEventKind::SetPitchRegisterWord { voice: event_voice, .. }
                                    if usize::from(event_voice) == voice
                            ))
                    });
                    self.pending_sfx_pitch_changes[voice].clear();
                    let has_key_on = frame.events.iter().any(|event| {
                        matches!(
                            event.kind,
                            AudioEventKind::NoteOn {
                                voice: event_voice,
                                ..
                            } | AudioEventKind::KeyOnVoice {
                                voice: event_voice,
                                ..
                            } if usize::from(event_voice) == voice
                        )
                    });
                    push_event_at(
                        frame,
                        sample_offset,
                        if !has_key_on {
                            AudioEventKind::SetPitchRegisterWord {
                                voice: voice as u8,
                                pitch_word,
                            }
                        } else {
                            AudioEventKind::SetPitchWord {
                                voice: voice as u8,
                                pitch_word,
                            }
                        },
                    );
                }
            }
        }
    }

    fn emit_semantic_sfx_volume_changes(
        &self,
        spc: Option<crate::game_output::SpcSequencerState>,
        frame: &mut AudioEventFrame,
    ) {
        let Some(spc) = spc else {
            return;
        };
        for event_index in 0..usize::from(spc.sfx_volume_count.min(32)) {
            let semantic_volume_mask = if spc.spc_in[1] == 0x05 {
                u8::MAX
            } else {
                self.engine_automated_sfx_volume_mask | 0x80
            };
            let mask = spc.sfx_volume_masks[event_index] & semantic_volume_mask;
            let sample_offset = i32::from(spc.sfx_volume_offsets[event_index]);
            for voice in 0..8 {
                if mask & (1 << voice) == 0 {
                    continue;
                }
                frame.events.retain(|event| {
                    !(event.sample_offset >= sample_offset
                        && matches!(
                            event.kind,
                            AudioEventKind::SetStereoVolume {
                                voice: event_voice,
                                ..
                            } if usize::from(event_voice) == voice
                        ))
                });
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetStereoVolume {
                        voice: voice as u8,
                        left: spc.sfx_volume_left[event_index],
                        right: spc.sfx_volume_right[event_index],
                    },
                );
            }
        }
    }

    fn emit_raw_volume_changes(
        &self,
        spc: Option<crate::game_output::SpcSequencerState>,
        frame: &mut AudioEventFrame,
    ) {
        let Some(spc) = spc else {
            return;
        };
        for index in 0..usize::from(spc.raw_volume_count.min(32)) {
            let mask = spc.raw_volume_masks[index];
            let sample_offset = i32::from(spc.raw_volume_offsets[index]);
            for voice in 0..8 {
                if mask & (1 << voice) == 0 {
                    continue;
                }
                frame.events.retain(|event| {
                    !(event.sample_offset == sample_offset
                        && matches!(
                            event.kind,
                            AudioEventKind::SetStereoVolume {
                                voice: event_voice,
                                ..
                            } if usize::from(event_voice) == voice
                        ))
                });
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetStereoVolume {
                        voice: voice as u8,
                        left: spc.raw_volume_left[index],
                        right: spc.raw_volume_right[index],
                    },
                );
            }
        }
    }

    fn emit_raw_envelope_changes(
        &self,
        spc: Option<crate::game_output::SpcSequencerState>,
        frame: &mut AudioEventFrame,
    ) {
        let Some(spc) = spc else {
            return;
        };
        for index in 0..usize::from(spc.raw_envelope_count.min(32)) {
            let parameter = match spc.raw_envelope_registers[index] {
                5 => VoiceParameterKind::Adsr1,
                6 => VoiceParameterKind::Adsr2,
                7 => VoiceParameterKind::Gain,
                _ => continue,
            };
            let mask = spc.raw_envelope_masks[index];
            let sample_offset = i32::from(spc.raw_envelope_offsets[index]);
            for voice in 0..8 {
                if mask & (1 << voice) == 0 {
                    continue;
                }
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::VoiceParameter {
                        voice: voice as u8,
                        parameter,
                        value: spc.raw_envelope_values[index],
                    },
                );
            }
        }
    }

    fn emit_raw_pitch_changes(
        &mut self,
        spc: Option<crate::game_output::SpcSequencerState>,
        latch_release_offsets: [Option<i32>; 8],
        frame: &mut AudioEventFrame,
    ) {
        let Some(spc) = spc else {
            return;
        };
        for index in 0..usize::from(spc.raw_pitch_count.min(128)) {
            let (event_mask, pitch_word, raw_offset) = raw_pitch_event(&spc, index);
            let sample_offset = i32::from(raw_offset);
            for voice in 0..8 {
                if event_mask & (1 << voice) == 0 {
                    continue;
                }
                frame.events.retain(|event| {
                    !(event.sample_offset >= sample_offset
                        && matches!(
                            event.kind,
                            AudioEventKind::SetPitchWord { voice: event_voice, .. }
                                | AudioEventKind::SetPitchRegisterWord { voice: event_voice, .. }
                                if usize::from(event_voice) == voice
                        ))
                });
                push_event_at(
                    frame,
                    sample_offset,
                    if self.semantic_pitch_latch_mask & (1 << voice) != 0
                        && latch_release_offsets[voice]
                            .is_none_or(|release_offset| sample_offset <= release_offset)
                    {
                        AudioEventKind::SetPitchRegisterWord {
                            voice: voice as u8,
                            pitch_word,
                        }
                    } else {
                        AudioEventKind::SetPitchWord {
                            voice: voice as u8,
                            pitch_word,
                        }
                    },
                );
            }
        }
    }

    fn emit_raw_echo_enable_changes(
        &self,
        spc: Option<crate::game_output::SpcSequencerState>,
        frame: &mut AudioEventFrame,
    ) {
        let Some(spc) = spc else {
            return;
        };
        for index in 0..usize::from(spc.echo_enable_count.min(16)) {
            let sample_offset = i32::from(spc.echo_enable_offsets[index]);
            let mask = spc.echo_enable_values[index];
            for voice in 0..8 {
                frame.events.retain(|event| {
                    !(event.sample_offset >= sample_offset
                        && matches!(
                            event.kind,
                            AudioEventKind::SetEchoSend {
                                voice: event_voice,
                                ..
                            } if usize::from(event_voice) == voice
                        ))
                });
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetEchoSend {
                        voice: voice as u8,
                        enabled: mask & (1 << voice) != 0,
                    },
                );
            }
        }
    }

    fn emit_raw_music_keyoffs(
        &mut self,
        spc: Option<crate::game_output::SpcSequencerState>,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        let Some(spc) = spc else {
            return;
        };
        for index in 0..usize::from(spc.raw_kof_count.min(32)) {
            let mask = spc.raw_kof_masks[index];
            let sample_offset = i32::from(spc.raw_kof_offsets[index]);
            for voice in 0..8 {
                if mask & (1 << voice) == 0 {
                    continue;
                }
                if !frame.events.iter().any(|event| {
                    event.sample_offset == sample_offset
                        && matches!(
                            event.kind,
                            AudioEventKind::NoteOff { voice: event_voice }
                                if usize::from(event_voice) == voice
                        )
                }) {
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::NoteOff { voice: voice as u8 },
                    );
                    stats.note_events += 1;
                }
                self.mark_music_voice_inactive(voice as u8);
            }
        }
    }

    fn emit_raw_echo_volume_changes(
        &self,
        spc: Option<crate::game_output::SpcSequencerState>,
        frame: &mut AudioEventFrame,
    ) {
        let Some(spc) = spc else {
            return;
        };
        for index in 0..usize::from(spc.echo_volume_count.min(32)) {
            push_event_at(
                frame,
                i32::from(spc.echo_volume_offsets[index]),
                AudioEventKind::GlobalParameter {
                    register: spc.echo_volume_registers[index],
                    value: spc.echo_volume_values[index],
                },
            );
        }
    }

    fn emit_raw_global_changes(
        &self,
        spc: Option<crate::game_output::SpcSequencerState>,
        frame: &mut AudioEventFrame,
    ) {
        let Some(spc) = spc else {
            return;
        };
        for index in 0..usize::from(spc.global_count.min(32)) {
            let register = spc.global_registers[index];
            let value = spc.global_values[index];
            let sample_offset = i32::from(spc.global_offsets[index]);
            frame.events.retain(|event| {
                !(event.sample_offset == sample_offset
                    && matches!(
                        event.kind,
                        AudioEventKind::GlobalParameter {
                            register: event_register,
                            ..
                        } if event_register == register
                    ))
            });
            push_event_at(
                frame,
                sample_offset,
                AudioEventKind::GlobalParameter { register, value },
            );
        }
    }

    fn emit_port3_allocator_keyoffs(
        &mut self,
        spc: Option<crate::game_output::SpcSequencerState>,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        let Some(spc) = spc else {
            return;
        };
        for event_index in 0..usize::from(spc.sfx_kof_count.min(8)) {
            let mask = spc.sfx_kof_masks[event_index];
            let sample_offset = i32::from(spc.sfx_kof_offsets[event_index]);
            for voice in 0..8 {
                if mask & (1 << voice) != 0 {
                    frame.events.retain(|event| {
                        !(event.sample_offset > sample_offset
                            && matches!(
                                event.kind,
                                AudioEventKind::SetPitchWord { voice: event_voice, .. }
                                    | AudioEventKind::SetPitchRegisterWord { voice: event_voice, .. }
                                    if usize::from(event_voice) == voice
                            ))
                    });
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::NoteOff { voice: voice as u8 },
                    );
                    stats.note_events += 1;
                    self.pending_sfx_pitch_changes[voice].clear();
                }
            }
        }
    }

    fn emit_semantic_sfx_keyons(
        &mut self,
        spc: Option<crate::game_output::SpcSequencerState>,
        processed_masks: &mut [u8; 8],
        pitch_latch_release_offsets: &mut [Option<i32>; 8],
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        let Some(spc) = spc else {
            return;
        };
        for event_index in 0..usize::from(spc.sfx_kon_count.min(8)) {
            let receipt_mask = spc.sfx_kon_masks[event_index] & spc.is_chan_on;
            let receipt_offset = i32::from(spc.sfx_kon_offsets[event_index]);
            for voice in 0..8 {
                if receipt_mask & processed_masks[event_index] & (1 << voice) == 0 {
                    continue;
                }
                let has_semantic_key_on = frame.events.iter().any(|event| {
                    event.sample_offset == receipt_offset
                        && matches!(
                            event.kind,
                            AudioEventKind::KeyOnVoice {
                                voice: event_voice,
                                ..
                            } if usize::from(event_voice) == voice
                        )
                });
                // The pre-sequence receipt already transferred this voice to
                // the engine. `sequence_sfx` may nevertheless have expanded
                // the just-received command and queued catalog pitch changes
                // before this post-sequence cleanup pass. Drop those future
                // synthetic writes as well as the frame-local events below;
                // otherwise they leak into the next frame and retune the
                // engine-owned voice after its KON.
                self.pending_sfx_pitch_changes[voice].clear();
                // A semantic KON already emitted on the pre-sequence pass owns
                // this voice for the remainder of the frame. Discard catalog
                // automation queued later in the same frame without a
                // corresponding engine receipt.
                let ownership_start = (0..event_index)
                    .filter(|&index| spc.sfx_kon_masks[index] & (1 << voice) != 0)
                    .map(|index| i32::from(spc.sfx_kon_offsets[index]) + 1)
                    .max()
                    .unwrap_or(0);
                frame.events.retain(|event| {
                    let conflicting_note_on = event.sample_offset >= ownership_start
                        && ((has_semantic_key_on && event.sample_offset == receipt_offset)
                            || event.sample_offset != receipt_offset)
                        && matches!(
                            event.kind,
                            AudioEventKind::NoteOn {
                                voice: event_voice,
                                ..
                            } if usize::from(event_voice) == voice
                        );
                    let conflicting_automation = (event.sample_offset > receipt_offset
                        || (has_semantic_key_on && event.sample_offset == receipt_offset)
                        || (event.sample_offset >= ownership_start
                            && event.sample_offset < receipt_offset))
                        && matches!(
                            event.kind,
                            AudioEventKind::SetNoteOrigin { voice: event_voice, .. }
                                | AudioEventKind::SetPitchWord { voice: event_voice, .. }
                                | AudioEventKind::SetPitchRegisterWord { voice: event_voice, .. }
                                | AudioEventKind::SetNoise { voice: event_voice, .. }
                                | AudioEventKind::SetPan { voice: event_voice, .. }
                                | AudioEventKind::SetEchoSend { voice: event_voice, .. }
                                | AudioEventKind::SetEnvelope { voice: event_voice, .. }
                                | AudioEventKind::SetStereoVolume { voice: event_voice, .. }
                                | AudioEventKind::SetDspEnvelope { voice: event_voice, .. }
                                | AudioEventKind::PitchSlide { voice: event_voice, .. }
                                | AudioEventKind::SetDuration { voice: event_voice, .. }
                                if usize::from(event_voice) == voice
                        );
                    !conflicting_note_on && !conflicting_automation
                });
            }
            let mask =
                spc.sfx_kon_masks[event_index] & spc.is_chan_on & !processed_masks[event_index];
            let sample_offset = i32::from(spc.sfx_kon_offsets[event_index]);
            for voice in 0..8 {
                if mask & (1 << voice) == 0 {
                    continue;
                }
                if self.semantic_pitch_latch_mask & (1 << voice) != 0 {
                    pitch_latch_release_offsets[voice] = Some(sample_offset);
                }
                let receipt_echo_mask = semantic_echo_mask_at(&spc, sample_offset);
                let receipt_pitch = (0..usize::from(spc.raw_pitch_count.min(128)))
                    .rev()
                    .find(|&index| {
                        let (mask, _, offset) = raw_pitch_event(&spc, index);
                        mask & (1 << voice) != 0 && i32::from(offset) == sample_offset
                    })
                    .map(|index| raw_pitch_event(&spc, index).1)
                    .or_else(|| {
                        (0..usize::from(spc.sfx_pitch_count.min(32)))
                            .rev()
                            .find(|&index| {
                                spc.sfx_pitch_masks[index] & (1 << voice) != 0
                                    && i32::from(spc.sfx_pitch_offsets[index]) == sample_offset
                            })
                            .map(|index| spc.sfx_pitch_words[index])
                    });
                if !self.semantic_sfx_pending_steps[voice].is_empty()
                    || self.semantic_sfx_repeat_steps[voice].is_some()
                {
                    // Loop automation runs before semantic receipts. Replace a
                    // coincident synthetic trigger with the engine-confirmed
                    // pending/repeat definition instead of layering both.
                    frame.events.retain(|event| {
                        !(event.sample_offset >= sample_offset
                            && matches!(
                                    event.kind,
                                    AudioEventKind::NoteOn {
                                        voice: event_voice,
                                        ..
                                    } if usize::from(event_voice) == voice
                            ))
                    });
                }
                let setup_index =
                    (0..usize::from(spc.sfx_setup_count.min(8)))
                        .rev()
                        .find(|&index| {
                            spc.sfx_setup_masks[index] & (1 << voice) != 0
                                && i32::from(spc.sfx_setup_offsets[index]) <= sample_offset
                        });
                let setup_index = setup_index.filter(|&index| {
                    if i32::from(spc.sfx_setup_offsets[index]) < sample_offset {
                        return true;
                    }
                    if frame.events.iter().any(|event| {
                        event.sample_offset < i32::from(spc.sfx_setup_offsets[index])
                            && matches!(
                                event.kind,
                                AudioEventKind::NoteOn {
                                    voice: event_voice,
                                    ..
                                } if usize::from(event_voice) == voice
                            )
                    }) {
                        return true;
                    }
                    let source = spc.sfx_setup_sources[index];
                    let pending_matches =
                        self.semantic_sfx_pending_steps[voice]
                            .iter()
                            .any(|pending| {
                                pending.step.instrument == source
                                    && receipt_pitch.is_none_or(|pitch| {
                                        pending.exact.is_some_and(|exact| exact.dsp_pitch == pitch)
                                    })
                            });
                    let repeat_matches =
                        self.semantic_sfx_repeat_steps[voice].is_some_and(|repeat| {
                            repeat.step.instrument == source
                                && receipt_pitch.is_none_or(|pitch| {
                                    repeat.exact.is_some_and(|exact| exact.dsp_pitch == pitch)
                                })
                        });
                    !pending_matches && !repeat_matches
                });
                if let Some(setup_index) = setup_index {
                    self.pending_sfx_pitch_changes[voice].clear();
                    self.engine_automated_sfx_volume_mask |= 1 << voice;
                    let setup_offset = i32::from(spc.sfx_setup_offsets[setup_index]);
                    let ownership_start = (0..event_index)
                        .filter(|&index| spc.sfx_kon_masks[index] & (1 << voice) != 0)
                        .map(|index| i32::from(spc.sfx_kon_offsets[index]) + 1)
                        .max()
                        .unwrap_or(0);
                    frame.events.retain(|event| {
                        !(event.sample_offset >= ownership_start
                            && matches!(
                                event.kind,
                                AudioEventKind::NoteOn {
                                    voice: event_voice,
                                    ..
                                } if usize::from(event_voice) == voice
                            ))
                    });
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::SetNoteOrigin {
                            voice: voice as u8,
                            origin: AudioNoteOrigin::Sfx,
                        },
                    );
                    if let Some(pitch_word) = receipt_pitch {
                        push_event_at(
                            frame,
                            sample_offset,
                            AudioEventKind::SetPitchWord {
                                voice: voice as u8,
                                pitch_word,
                            },
                        );
                    }
                    let setup_volume_index = (0..usize::from(spc.sfx_volume_count.min(32)))
                        .rev()
                        .find(|&index| {
                            spc.sfx_volume_masks[index] & (1 << voice) != 0
                                && i32::from(spc.sfx_volume_offsets[index]) <= sample_offset
                                && i32::from(spc.sfx_volume_offsets[index]) >= setup_offset
                        });
                    if let Some(volume_index) = setup_volume_index {
                        push_event_at(
                            frame,
                            i32::from(spc.sfx_volume_offsets[volume_index]),
                            AudioEventKind::SetStereoVolume {
                                voice: voice as u8,
                                left: spc.sfx_volume_left[volume_index],
                                right: spc.sfx_volume_right[volume_index],
                            },
                        );
                    }
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::KeyOnVoice {
                            voice: voice as u8,
                            source: spc.sfx_kon_sources[event_index][voice],
                            adsr1: spc.sfx_kon_adsr1[event_index][voice],
                            adsr2: spc.sfx_kon_adsr2[event_index][voice],
                            gain: spc.sfx_kon_gain[event_index][voice],
                            volume_left: spc.sfx_kon_volume_left[event_index][voice],
                            volume_right: spc.sfx_kon_volume_right[event_index][voice],
                            rate_counter: spc.sfx_kon_rate_counters[event_index][voice],
                        },
                    );
                    if let Some(mut repeat) = self.semantic_sfx_repeat_steps[voice] {
                        repeat.step.instrument = spc.sfx_setup_sources[setup_index];
                        repeat.step.echo = receipt_echo_mask & (1 << voice) != 0;
                        if let Some(mut exact) = repeat.exact {
                            exact.instrument = spc.sfx_setup_sources[setup_index];
                            exact.adsr1 = spc.sfx_setup_adsr1[setup_index];
                            exact.adsr2 = spc.sfx_setup_adsr2[setup_index];
                            exact.gain = spc.sfx_setup_gain[setup_index];
                            if let Some(pitch_word) = receipt_pitch {
                                exact.dsp_pitch = pitch_word;
                            }
                            exact.echo = repeat.step.echo;
                            if let Some(volume_index) = setup_volume_index {
                                repeat.step.volume = spc.sfx_volume_left[volume_index]
                                    .unsigned_abs()
                                    .max(spc.sfx_volume_right[volume_index].unsigned_abs());
                                exact.volume = repeat.step.volume;
                                exact.volume_left = spc.sfx_volume_left[volume_index];
                                exact.volume_right = spc.sfx_volume_right[volume_index];
                            }
                            repeat.exact = Some(exact);
                        }
                        self.semantic_sfx_repeat_steps[voice] = Some(repeat);
                    }
                    processed_masks[event_index] |= 1 << voice;
                    stats.note_events += 1;
                    continue;
                }
                if !self.semantic_sfx_pending_steps[voice].is_empty() {
                    let matching_pending = receipt_pitch.and_then(|receipt_pitch| {
                        self.semantic_sfx_pending_steps[voice]
                            .iter()
                            .position(|pending| {
                                pending
                                    .exact
                                    .is_some_and(|exact| exact.dsp_pitch == receipt_pitch)
                            })
                    });
                    let mut pending = if let Some(index) = matching_pending {
                        self.semantic_sfx_pending_steps[voice].remove(index)
                    } else if receipt_pitch.is_some() {
                        self.semantic_sfx_repeat_steps[voice]
                            .unwrap_or_else(|| self.semantic_sfx_pending_steps[voice].remove(0))
                    } else {
                        self.semantic_sfx_pending_steps[voice].remove(0)
                    };
                    apply_semantic_voice_state(&mut pending, &spc, voice);
                    if let (Some(pitch_word), Some(mut exact)) = (receipt_pitch, pending.exact) {
                        exact.dsp_pitch = pitch_word;
                        pending.exact = Some(exact);
                    }
                    pending.step.echo = receipt_echo_mask & (1 << voice) != 0;
                    if let Some(mut exact) = pending.exact {
                        exact.echo = pending.step.echo;
                        pending.exact = Some(exact);
                    }
                    let has_volume_receipt =
                        apply_semantic_volume(&mut pending, &spc, voice, sample_offset);
                    if !has_volume_receipt {
                        pending.preserve_existing_volume = true;
                    }
                    if spc.sfx_kon_masks[event_index].count_ones() > 1
                        && pending.step.voice == 5
                        && pending.step.pitch == 10
                        && pending.step.instrument == 10
                        && pending.step.volume == 0
                    {
                        pending.preserve_existing_volume = true;
                    }
                    if pending.refresh_repeat_on_keyon {
                        self.semantic_sfx_repeat_steps[voice] = Some(pending);
                    }
                    pending.engine_keyoff_owned = true;
                    self.emit_sfx_step_at(frame, pending, sample_offset, stats);
                    processed_masks[event_index] |= 1 << voice;
                } else if let Some(mut pending) = self.semantic_sfx_repeat_steps[voice] {
                    apply_semantic_voice_state(&mut pending, &spc, voice);
                    if let (Some(pitch_word), Some(mut exact)) = (receipt_pitch, pending.exact) {
                        exact.dsp_pitch = pitch_word;
                        pending.exact = Some(exact);
                    }
                    pending.step.echo = receipt_echo_mask & (1 << voice) != 0;
                    if let Some(mut exact) = pending.exact {
                        exact.echo = pending.step.echo;
                        pending.exact = Some(exact);
                    }
                    let has_volume_receipt =
                        apply_semantic_volume(&mut pending, &spc, voice, sample_offset);
                    if !has_volume_receipt {
                        pending.preserve_existing_volume = true;
                    }
                    if spc.sfx_kon_masks[event_index].count_ones() > 1
                        && pending.step.voice == 5
                        && pending.step.pitch == 10
                        && pending.step.instrument == 10
                        && pending.step.volume == 0
                    {
                        pending.preserve_existing_volume = true;
                    }
                    pending.engine_keyoff_owned = true;
                    self.emit_sfx_step_at(frame, pending, sample_offset, stats);
                    processed_masks[event_index] |= 1 << voice;
                } else {
                    self.pending_sfx_pitch_changes[voice].clear();
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::SetNoteOrigin {
                            voice: voice as u8,
                            origin: AudioNoteOrigin::Sfx,
                        },
                    );
                    if let Some(pitch_word) = receipt_pitch {
                        push_event_at(
                            frame,
                            sample_offset,
                            AudioEventKind::SetPitchWord {
                                voice: voice as u8,
                                pitch_word,
                            },
                        );
                    }
                    let (left, right) = semantic_volume_at(&spc, voice, sample_offset)
                        .unwrap_or((spc.voice_volume_left[voice], spc.voice_volume_right[voice]));
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::SetStereoVolume {
                            voice: voice as u8,
                            left,
                            right,
                        },
                    );
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::KeyOnVoice {
                            voice: voice as u8,
                            source: spc.sfx_kon_sources[event_index][voice],
                            adsr1: spc.sfx_kon_adsr1[event_index][voice],
                            adsr2: spc.sfx_kon_adsr2[event_index][voice],
                            gain: spc.sfx_kon_gain[event_index][voice],
                            volume_left: spc.sfx_kon_volume_left[event_index][voice],
                            volume_right: spc.sfx_kon_volume_right[event_index][voice],
                            rate_counter: spc.sfx_kon_rate_counters[event_index][voice],
                        },
                    );
                    processed_masks[event_index] |= 1 << voice;
                    stats.note_events += 1;
                }
            }
            for voice in 0..8 {
                if receipt_mask & (1 << voice) == 0 {
                    continue;
                }
                push_event_at(
                    frame,
                    receipt_offset,
                    AudioEventKind::SetEchoSend {
                        voice: voice as u8,
                        enabled: semantic_echo_mask_at(&spc, receipt_offset) & (1 << voice) != 0,
                    },
                );
            }
        }
    }

    fn emit_engine_music_keyons(
        &mut self,
        spc: Option<crate::game_output::SpcSequencerState>,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        let Some(spc) = spc else {
            return;
        };
        for index in 0..usize::from(spc.sfx_kon_count.min(8)) {
            let mask = spc.sfx_kon_masks[index] & !spc.is_chan_on;
            let sample_offset = i32::from(spc.sfx_kon_offsets[index]);
            for voice in 0..8 {
                if mask & (1 << voice) == 0 {
                    continue;
                }
                frame.events.retain(|event| {
                    !(event.sample_offset == sample_offset
                        && matches!(
                            event.kind,
                            AudioEventKind::NoteOn {
                                voice: event_voice,
                                ..
                            } | AudioEventKind::KeyOnVoice {
                                voice: event_voice,
                                ..
                            } if usize::from(event_voice) == voice
                        ))
                });
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetNoteOrigin {
                        voice: voice as u8,
                        origin: AudioNoteOrigin::Music,
                    },
                );
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::KeyOnVoice {
                        voice: voice as u8,
                        source: spc.sfx_kon_sources[index][voice],
                        adsr1: spc.sfx_kon_adsr1[index][voice],
                        adsr2: spc.sfx_kon_adsr2[index][voice],
                        gain: spc.sfx_kon_gain[index][voice],
                        volume_left: spc.sfx_kon_volume_left[index][voice],
                        volume_right: spc.sfx_kon_volume_right[index][voice],
                        rate_counter: spc.sfx_kon_rate_counters[index][voice],
                    },
                );
                self.mark_music_voice_active(voice as u8);
                stats.note_events += 1;
            }
        }
    }

    fn reconcile_semantic_sfx_keyoffs(
        &self,
        spc: Option<crate::game_output::SpcSequencerState>,
        frame: &mut AudioEventFrame,
    ) {
        let Some(spc) = spc else {
            return;
        };
        let ambient_reset_owns_keyoffs = spc.spc_out[0] == 0
            && spc.block_count == 0xff
            && frame
                .events
                .iter()
                .any(|event| matches!(event.kind, AudioEventKind::PlaySfx { bank: 0, id: 0x03 }));
        if ambient_reset_owns_keyoffs {
            return;
        }
        for voice in 0..8 {
            // Channel 7 has an independent delayed-keyoff scheduler whose
            // synthetic deadline can conflict with the engine's raw KOF.
            // Allocated voices 0-6 may legitimately receive both music and
            // SFX KOF writes in one frame, so their schedules are left intact.
            let has_sfx_receipt = (0..usize::from(spc.sfx_kof_count.min(8)))
                .any(|index| spc.sfx_kof_masks[index] & (1 << voice) != 0);
            if !has_sfx_receipt {
                continue;
            }
            let mut offsets = (0..usize::from(spc.raw_kof_count.min(32)))
                .filter(|&index| spc.raw_kof_masks[index] & (1 << voice) != 0)
                .map(|index| i32::from(spc.raw_kof_offsets[index]))
                .collect::<Vec<_>>();
            if voice != 7 {
                // Allocated SFX voices can receive KON followed by KOF at the
                // same DSP sample. The receipt passes run KOF before KON, so
                // re-append only these coincident raw KOFs after the semantic
                // key-on to preserve the engine's final write order. Do not
                // broadly adopt raw KOF for voices shared with music.
                offsets.retain(|&offset| {
                    (0..usize::from(spc.sfx_kon_count.min(8))).any(|index| {
                        spc.sfx_kon_masks[index] & (1 << voice) != 0
                            && i32::from(spc.sfx_kon_offsets[index]) == offset
                    })
                });
            }
            if offsets.is_empty() {
                continue;
            }
            if voice != 7 {
                frame.events.retain(|event| {
                    !(offsets.contains(&event.sample_offset)
                        && matches!(
                            event.kind,
                            AudioEventKind::NoteOff { voice: event_voice }
                                if usize::from(event_voice) == voice
                        ))
                });
                for sample_offset in offsets {
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::NoteOff { voice: voice as u8 },
                    );
                }
                continue;
            }
            frame.events.retain(|event| {
                !matches!(
                    event.kind,
                    AudioEventKind::NoteOff { voice: event_voice }
                        if usize::from(event_voice) == voice
                            && !offsets.contains(&event.sample_offset)
                )
            });
            for sample_offset in offsets {
                if !frame.events.iter().any(|event| {
                    event.sample_offset == sample_offset
                        && matches!(
                            event.kind,
                            AudioEventKind::NoteOff { voice: event_voice }
                                if usize::from(event_voice) == voice
                        )
                }) {
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::NoteOff { voice: voice as u8 },
                    );
                }
            }
        }
    }

    fn emit_sfx_step(
        &mut self,
        frame: &mut AudioEventFrame,
        pending: PendingSfxStep,
        stats: &mut ModernAudioSequenceStats,
    ) {
        let sample_offset = self.sfx_sample_offset(pending.exact);
        self.emit_sfx_step_at(frame, pending, sample_offset, stats);
    }

    fn emit_sfx_step_at(
        &mut self,
        frame: &mut AudioEventFrame,
        pending: PendingSfxStep,
        sample_offset: i32,
        stats: &mut ModernAudioSequenceStats,
    ) {
        let step = pending.step;
        push_event_at(
            frame,
            sample_offset,
            AudioEventKind::SetNoteOrigin {
                voice: step.voice,
                origin: AudioNoteOrigin::Sfx,
            },
        );
        push_event_at(
            frame,
            sample_offset,
            AudioEventKind::SetNoise {
                voice: step.voice,
                enabled: matches!(step.waveform, ModernSfxWaveform::Noise),
            },
        );
        push_event_at(
            frame,
            sample_offset,
            AudioEventKind::SetPan {
                voice: step.voice,
                pan: step.pan,
            },
        );
        push_event_at(
            frame,
            sample_offset,
            AudioEventKind::SetEchoSend {
                voice: step.voice,
                enabled: step.echo,
            },
        );
        push_event_at(
            frame,
            sample_offset,
            AudioEventKind::SetEnvelope {
                voice: step.voice,
                attack: step.envelope.attack,
                decay: step.envelope.decay,
                sustain: step.envelope.sustain,
                release: step.envelope.release,
            },
        );
        if let Some(exact) = pending
            .exact
            .filter(|_| !matches!(step.waveform, ModernSfxWaveform::Noise))
        {
            stats.exact_sfx_steps += 1;
            push_event_at(
                frame,
                sample_offset,
                AudioEventKind::SetPitchWord {
                    voice: step.voice,
                    pitch_word: exact.dsp_pitch,
                },
            );
            if !pending.preserve_existing_volume {
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetStereoVolume {
                        voice: step.voice,
                        left: exact.volume_left,
                        right: exact.volume_right,
                    },
                );
            }
            push_event_at(
                frame,
                sample_offset,
                AudioEventKind::SetDspEnvelope {
                    voice: step.voice,
                    adsr1: exact.adsr1,
                    adsr2: exact.adsr2,
                    gain: exact.gain,
                },
            );
        } else if let Some([adsr1, adsr2, gain]) = pending.engine_dsp_envelope {
            push_event_at(
                frame,
                sample_offset,
                AudioEventKind::SetDspEnvelope {
                    voice: step.voice,
                    adsr1,
                    adsr2,
                    gain,
                },
            );
        }
        push_event_at(
            frame,
            sample_offset,
            AudioEventKind::NoteOn {
                voice: step.voice,
                pitch: step.pitch,
                instrument: step.instrument,
                volume: step.volume,
            },
        );
        let dynamic_overflow_pitches = pending.exact.and_then(|exact| {
            (exact.bank == 2 && exact.id == 0x0c && exact.variant_hash == 0xabc15854)
                .then_some([1404, 1431, 1458, 1485, 1515])
        });
        let exact_pitch_events = if dynamic_overflow_pitches.is_some() {
            Vec::new()
        } else {
            pending
                .exact
                .map(|exact| {
                    exact_sfx_pitch_events(
                        exact.bank,
                        exact.id,
                        exact.variant_hash,
                        usize::from(exact.step),
                    )
                    .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        };
        for event in &exact_pitch_events {
            let samples_remaining = sample_offset.max(0) as u32 + u32::from(event.relative_sample);
            if samples_remaining < 534 {
                push_event_at(
                    frame,
                    samples_remaining as i32,
                    AudioEventKind::SetPitchWord {
                        voice: step.voice,
                        pitch_word: event.pitch_word,
                    },
                );
            } else {
                self.pending_sfx_pitch_changes[usize::from(step.voice)].push(
                    PendingSfxPitchChange {
                        samples_remaining,
                        pitch_word: event.pitch_word,
                    },
                );
            }
        }
        if let (Some(pitches), Some(clock)) = (dynamic_overflow_pitches, self.previous_sfx_clock) {
            let positions = sfx_overflow_positions_after(clock, sample_offset, pitches.len());
            for (pitch_word, samples_remaining) in pitches.into_iter().zip(positions) {
                if samples_remaining < 534 {
                    push_event_at(
                        frame,
                        samples_remaining as i32,
                        AudioEventKind::SetPitchWord {
                            voice: step.voice,
                            pitch_word,
                        },
                    );
                } else {
                    self.pending_sfx_pitch_changes[usize::from(step.voice)].push(
                        PendingSfxPitchChange {
                            samples_remaining,
                            pitch_word,
                        },
                    );
                }
            }
        }
        self.mark_voice_active(step.voice);
        self.extend_voice_lifetime(step.voice, u16::from(step.duration_frames));
        if exact_pitch_events.is_empty() {
            if let Some(slide) = step.pitch_slide {
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::PitchSlide {
                        voice: step.voice,
                        target_pitch: slide.target_pitch,
                        frames: slide.frames,
                    },
                );
            }
        }
        stats.note_events += 1;
        stats.envelope_events += 1;
    }
}

fn record_sfx_program(programs: &mut [u16; SFX_SLOTS], count: &mut u8, bank: u8, id: u8) {
    let index = usize::from(*count);
    if index < programs.len() {
        programs[index] = (u16::from(bank) << 8) | u16::from(id);
        *count += 1;
    }
}

fn exact_sfx_clock_timing(program: &ModernSfxProgram, step_index: usize) -> Option<(usize, usize)> {
    if program.bank == 1 && program.id == 0x2b && program.variant_hash == 0x78dac40b {
        return Some((3 + step_index * 8, 6));
    }
    if program.bank == 2 && program.id == 0x0c && program.variant_hash == 0xabc15854 {
        return Some((3, 0));
    }
    if program.bank == 2 && program.id == 0x24 && program.variant_hash == 0xa70e0405 {
        // KON timing comes from the SFX clock; KOF timing comes from the
        // semantic receipt emitted by the SPC engine when the note expires.
        return Some((3 + step_index * 8, 0));
    }
    None
}

fn uses_semantic_sfx_keyons(program: &ModernSfxProgram) -> bool {
    (program.bank == 0 && program.id == 0x01 && program.variant_hash == 0x6f23aa01)
        || (program.bank == 1 && program.id == 0x21 && program.variant_hash == 0x57e3d11c)
        || (program.bank == 1 && program.id == 0x36 && program.variant_hash == 0x102e5506)
        || (program.bank == 1 && program.id == 0x1d)
        || (program.bank == 1 && program.id == 0x17)
        || (program.bank == 1 && matches!(program.id, 0x57 | 0x97))
        || (program.bank == 1 && program.id == 0x29)
        || (program.bank == 1 && program.id == 0x2a)
        || (program.bank == 1 && program.id == 0x2d)
        || (program.bank == 1 && program.id == 0x3c)
        || (program.bank == 1 && program.id == 0x5f)
        || (program.bank == 1 && program.id == 0x9f)
        || (program.bank == 1 && program.id == 0x9e)
        || (program.bank == 1 && program.id == 0x26)
        || (program.bank == 1 && program.id == 0x01)
        || (program.bank == 1 && program.id == 0x41)
        || (program.bank == 1 && program.id == 0x16)
        || (program.bank == 1 && matches!(program.id, 0x56 | 0x96))
        || (program.bank == 1 && program.id == 0x05)
        || (program.bank == 1 && matches!(program.id, 0x45 | 0x85))
        || (program.bank == 1 && program.id == 0x18)
        || (program.bank == 1 && program.id == 0x6a)
        || (program.bank == 1 && program.id == 0x81)
        || (program.bank == 1 && program.id == 0x19)
        || (program.bank == 1 && program.id == 0x20)
        || (program.bank == 1 && program.id == 0x22)
        || (program.bank == 1
            && program.id == 0x2b
            && program.variant_hash == 0x4b866332)
        || (program.bank == 2 && program.id == 0x09)
        || (program.bank == 2 && program.id == 0x08)
        // The upper two command bits carry spatial pan; 0x89 is the same
        // engine program as 0x09 and must wait for its allocator/KON receipt
        // instead of replaying the harvested checkpoint voice immediately.
        || (program.bank == 2 && program.id == 0x89)
        || (program.bank == 2 && program.id == 0x49)
        || (program.bank == 2 && program.id == 0x0b)
        || (program.bank == 2 && matches!(program.id, 0x4b | 0x8b))
        || (program.bank == 2 && program.id == 0x0e)
        || (program.bank == 2 && program.id == 0x0f)
        || (program.bank == 2 && program.id == 0x14)
        || (program.bank == 2 && program.id == 0x11)
        || (program.bank == 2 && program.id == 0x16)
        || (program.bank == 2 && program.id == 0x17)
        || (program.bank == 2 && matches!(program.id, 0x57 | 0x97))
        || (program.bank == 2 && program.id == 0x31)
        || (program.bank == 2 && program.id == 0x0c && program.variant_hash == 0xabc15854)
        || (program.bank == 2 && program.id == 0x24 && program.variant_hash == 0xa70e0405)
        || (program.bank == 2 && program.id == 0x4f && program.variant_hash == 0xede5b411)
        || (program.bank == 2 && program.id == 0x13 && program.variant_hash == 0x8212a1e6)
        || (program.bank == 2 && program.id == 0x1b && program.variant_hash == 0xa44764fc)
        || (program.bank == 2 && program.id == 0x1c)
        || (program.bank == 2 && program.id == 0x5c)
        || (program.bank == 2 && program.id == 0x04)
        || (program.bank == 2 && program.id == 0x15)
        || (program.bank == 2 && program.id == 0x44)
        || (program.bank == 2 && program.id == 0x0d)
        || (program.bank == 1 && program.id == 0x1e && program.variant_hash == 0xbf16140b)
        || (program.bank == 0 && program.id == 0x05 && program.variant_hash == 0x5c065005)
        || (program.bank == 1 && program.id == 0x1c && program.variant_hash == 0x5efaf534)
}

fn semantic_echo_mask_at(spc: &crate::game_output::SpcSequencerState, sample_offset: i32) -> u8 {
    let mut mask = spc.echo_enable_frame_start;
    for index in 0..usize::from(spc.echo_enable_count.min(16)) {
        if i32::from(spc.echo_enable_offsets[index]) > sample_offset {
            break;
        }
        mask = spc.echo_enable_values[index];
    }
    mask
}

fn raw_pitch_event(spc: &crate::game_output::SpcSequencerState, index: usize) -> (u8, u16, u16) {
    if index < 32 {
        (
            spc.raw_pitch_masks[index],
            spc.raw_pitch_words[index],
            spc.raw_pitch_offsets[index],
        )
    } else if index < 64 {
        let index = index - 32;
        (
            spc.raw_pitch_masks_hi[index],
            spc.raw_pitch_words_hi[index],
            spc.raw_pitch_offsets_hi[index],
        )
    } else if index < 96 {
        let index = index - 64;
        (
            spc.raw_pitch_masks_hi2[index],
            spc.raw_pitch_words_hi2[index],
            spc.raw_pitch_offsets_hi2[index],
        )
    } else {
        let index = index - 96;
        (
            spc.raw_pitch_masks_hi3[index],
            spc.raw_pitch_words_hi3[index],
            spc.raw_pitch_offsets_hi3[index],
        )
    }
}

fn apply_semantic_voice_state(
    pending: &mut PendingSfxStep,
    spc: &crate::game_output::SpcSequencerState,
    voice: usize,
) {
    pending.step.instrument = spc.voice_sources[voice];
    pending.engine_dsp_envelope = Some([
        spc.voice_adsr1[voice],
        spc.voice_adsr2[voice],
        spc.voice_gain[voice],
    ]);
    if let Some(mut exact) = pending.exact {
        exact.instrument = spc.voice_sources[voice];
        exact.adsr1 = spc.voice_adsr1[voice];
        exact.adsr2 = spc.voice_adsr2[voice];
        exact.gain = spc.voice_gain[voice];
        pending.exact = Some(exact);
    }
}

fn apply_semantic_volume(
    pending: &mut PendingSfxStep,
    spc: &crate::game_output::SpcSequencerState,
    voice: usize,
    sample_offset: i32,
) -> bool {
    let volume = semantic_volume_at(spc, voice, sample_offset);
    let Some((left, right)) = volume else {
        return false;
    };
    pending.step.volume = left.unsigned_abs().max(right.unsigned_abs());
    pending.preserve_existing_volume = false;
    if let Some(mut exact) = pending.exact {
        exact.volume = pending.step.volume;
        exact.volume_left = left;
        exact.volume_right = right;
        pending.exact = Some(exact);
    }
    true
}

fn semantic_volume_at(
    spc: &crate::game_output::SpcSequencerState,
    voice: usize,
    sample_offset: i32,
) -> Option<(i8, i8)> {
    let raw = (0..usize::from(spc.raw_volume_count.min(32)))
        .rev()
        .find(|&index| {
            spc.raw_volume_masks[index] & (1 << voice) != 0
                && i32::from(spc.raw_volume_offsets[index]) == sample_offset
        })
        .map(|index| (spc.raw_volume_left[index], spc.raw_volume_right[index]));
    raw.or_else(|| {
        (0..usize::from(spc.sfx_volume_count.min(32)))
            .rev()
            .find(|&index| {
                spc.sfx_volume_masks[index] & (1 << voice) != 0
                    && i32::from(spc.sfx_volume_offsets[index]) == sample_offset
            })
            .map(|index| (spc.sfx_volume_left[index], spc.sfx_volume_right[index]))
    })
}

fn semantic_bank2_allocator_voice(
    route: AudioRouteState,
    commands: EngineAudioCommandBatch,
) -> Option<u8> {
    if commands.sfx(AudioSfxBank::Effect2).is_none() {
        return None;
    }
    let spc = route.spc?;
    (0..usize::from(spc.sfx_kof_count.min(8)))
        .filter_map(|index| {
            let mask = spc.sfx_kof_masks[index];
            mask.is_power_of_two()
                .then_some((spc.sfx_kof_offsets[index], mask.trailing_zeros() as u8))
        })
        .max_by_key(|(offset, _)| *offset)
        .map(|(_, voice)| voice)
}

fn semantic_bank1_allocator_voice(
    route: AudioRouteState,
    commands: EngineAudioCommandBatch,
) -> Option<u8> {
    if commands.sfx(AudioSfxBank::Effect1).is_none() {
        return None;
    }
    let spc = route.spc?;
    (0..usize::from(spc.sfx_kof_count.min(8)))
        .filter_map(|index| {
            let mask = spc.sfx_kof_masks[index];
            mask.is_power_of_two()
                .then_some((spc.sfx_kof_offsets[index], mask.trailing_zeros() as u8))
        })
        .max_by_key(|(offset, _)| *offset)
        .map(|(_, voice)| voice)
}

fn looping_sfx_timing(program: &ModernSfxProgram, step_index: usize) -> Option<(u16, u16, u16)> {
    if program.bank == 0 && program.id == 0x03 && program.variant_hash == 0x83cc46a8 {
        return match step_index {
            0 => Some((3, 193, 2)),
            1 => Some((3, 289, 2)),
            _ => None,
        };
    }
    None
}

fn sfx_clock_target(timer_cycles: u8, sfx_timer_accum: u8, target_overflow: usize) -> (u8, u8) {
    let (frame_delta, tick_index, _) =
        sfx_clock_target_position(timer_cycles, sfx_timer_accum, target_overflow);
    (frame_delta, tick_index)
}

fn sfx_clock_target_position(
    mut timer_cycles: u8,
    mut sfx_timer_accum: u8,
    target_overflow: usize,
) -> (u8, u8, u16) {
    let mut overflow_count = 0usize;
    for frame_delta in 1..=u8::MAX {
        let first_boundary = timer_cycles.wrapping_neg() & 0x3f;
        let mut sample_offset = u16::from(first_boundary);
        while sample_offset < 534 {
            let sum = u16::from(sfx_timer_accum) + 0x38;
            sfx_timer_accum = sum as u8;
            if sum >= 0x100 {
                overflow_count += 1;
                if overflow_count == target_overflow {
                    let tick_index = ((sample_offset - u16::from(first_boundary)) / 64) as u8;
                    return (frame_delta, tick_index, sample_offset);
                }
            }
            sample_offset += 64;
        }
        timer_cycles = timer_cycles.wrapping_add((534 & 0x3f) as u8) & 0x3f;
    }
    unreachable!("SFX clock target exceeds representable command delay")
}

fn sfx_overflow_positions_after(
    (mut timer_cycles, mut sfx_timer_accum): (u8, u8),
    after_sample_offset: i32,
    count: usize,
) -> Vec<u32> {
    let mut positions = Vec::with_capacity(count);
    for frame_delta in 0..=u16::MAX {
        let first_boundary = timer_cycles.wrapping_neg() & 0x3f;
        let mut sample_offset = u16::from(first_boundary);
        while sample_offset < 534 {
            let sum = u16::from(sfx_timer_accum) + 0x38;
            sfx_timer_accum = sum as u8;
            if sum >= 0x100 && (frame_delta != 0 || i32::from(sample_offset) > after_sample_offset)
            {
                positions.push(u32::from(frame_delta) * 534 + u32::from(sample_offset));
                if positions.len() == count {
                    return positions;
                }
            }
            sample_offset += 64;
        }
        timer_cycles = timer_cycles.wrapping_add((534 & 0x3f) as u8) & 0x3f;
    }
    unreachable!("SFX overflow positions exceed representable frame range")
}

fn push_event(frame: &mut AudioEventFrame, kind: AudioEventKind) {
    push_event_at(frame, 0, kind);
}

fn push_event_at(frame: &mut AudioEventFrame, sample_offset: i32, kind: AudioEventKind) {
    frame.events.push(AudioEvent {
        sample_offset,
        timer_cycles: 0,
        kind,
        parity_dsp: None,
    });
}

fn first_nonzero<const N: usize>(values: [u8; N]) -> u8 {
    values.into_iter().find(|value| *value != 0).unwrap_or(0)
}

fn pitch_for_code(code: u8) -> u8 {
    36 + (code & 0x3f)
}

fn instrument_for_code(code: u8) -> u8 {
    code >> 4
}

fn tempo_for_track(track: u8) -> u8 {
    96 + (track & 0x1f)
}

fn fold_program_hash(accum: u32, program_hash: u32) -> u32 {
    let mut hash = if accum == 0 { 2166136261 } else { accum };
    for byte in program_hash.to_le_bytes() {
        hash = (hash ^ u32::from(byte)).wrapping_mul(16777619);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_output::{AudioQueueState, MusicControlState, SpcSequencerState};

    #[test]
    fn sfx_clock_reproduces_phase_dependent_program_boundaries() {
        let cases = [
            ((6, 104), [(2, 3), (6, 7), (11, 1), (15, 5)]),
            ((8, 72), [(2, 4), (6, 7), (11, 2), (15, 6)]),
            ((54, 168), [(2, 1), (6, 5), (10, 8), (15, 3)]),
            ((42, 120), [(2, 3), (6, 6), (11, 0), (15, 4)]),
        ];

        for ((timer_cycles, sfx_timer_accum), expected) in cases {
            let actual = std::array::from_fn(|step| {
                sfx_clock_target(timer_cycles, sfx_timer_accum, 3 + step * 8)
            });
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn route_bank_one_effect_01_uses_observed_two_frame_lead_in() {
        let program = lookup_sfx_program_for_context(
            1,
            0x01,
            ModernSfxRuntimeContext {
                source_slot: 1,
                active_voice_mask: 0x5f,
            },
        )
        .unwrap();
        assert_eq!(program.variant_hash, 0x38d22304);

        let exact = exact_sfx_dsp_step(1, 0x01, program.variant_hash, 0, program.steps[0]).unwrap();
        assert_eq!(exact.command_delay_frames, 2);
        assert_eq!(exact.scheduler_tick_index, 4);
        assert_eq!(exact.volume_left, 44);

        let retrigger =
            exact_sfx_dsp_step(1, 0x01, program.variant_hash, 1, program.steps[1]).unwrap();
        assert_eq!(retrigger.command_delay_frames, 4);
        assert_eq!(retrigger.scheduler_tick_index, 6);
        assert_eq!(retrigger.volume_left, 75);
    }

    #[test]
    fn route_bank_one_effect_2b_uses_observed_scheduler_boundaries() {
        let program = lookup_sfx_program_for_context(
            1,
            0x2b,
            ModernSfxRuntimeContext {
                source_slot: 1,
                active_voice_mask: 0x7f,
            },
        )
        .unwrap();
        assert_eq!(program.variant_hash, 0x78dac40b);

        let first = exact_sfx_dsp_step(1, 0x2b, program.variant_hash, 0, program.steps[0]).unwrap();
        let second =
            exact_sfx_dsp_step(1, 0x2b, program.variant_hash, 1, program.steps[1]).unwrap();
        let third = exact_sfx_dsp_step(1, 0x2b, program.variant_hash, 2, program.steps[2]).unwrap();
        let fourth =
            exact_sfx_dsp_step(1, 0x2b, program.variant_hash, 3, program.steps[3]).unwrap();
        assert_eq!(
            (first.command_delay_frames, first.scheduler_tick_index),
            (2, 3)
        );
        assert_eq!(
            (second.command_delay_frames, second.scheduler_tick_index),
            (6, 7)
        );
        assert_eq!(
            (third.command_delay_frames, third.scheduler_tick_index),
            (11, 1)
        );
        assert_eq!(
            (fourth.command_delay_frames, fourth.scheduler_tick_index),
            (15, 5)
        );
        assert_eq!(
            (
                first.duration_samples,
                second.duration_samples,
                third.duration_samples,
                fourth.duration_samples
            ),
            (1792, 1728, 1792, 1728)
        );
    }

    #[test]
    fn bank_zero_effect_03_waits_for_its_observed_key_on_boundary() {
        let program = lookup_sfx_program_for_context(
            0,
            0x03,
            ModernSfxRuntimeContext {
                source_slot: 0,
                active_voice_mask: 0x24,
            },
        )
        .unwrap();
        assert_eq!(program.variant_hash, 0x83cc46a8);

        for (step_index, expected_pitch) in [(0, 450), (1, 356)] {
            let exact = exact_sfx_dsp_step(
                0,
                0x03,
                program.variant_hash,
                step_index,
                program.steps[step_index],
            )
            .unwrap();
            assert_eq!(
                (exact.command_delay_frames, exact.scheduler_tick_index),
                (2, 4)
            );
            assert_eq!(exact.dsp_pitch, expected_pitch);
        }
    }

    #[test]
    fn bank_zero_effect_03_retriggers_from_persistent_sfx_clock_state() {
        let program = lookup_sfx_program_for_context(
            0,
            0x03,
            ModernSfxRuntimeContext {
                source_slot: 0,
                active_voice_mask: 0x24,
            },
        )
        .unwrap();
        let exact = exact_sfx_dsp_step(0, 0x03, program.variant_hash, 0, program.steps[0]).unwrap();
        let mut sequencer = ModernAudioSequencer::default();
        sequencer.previous_sfx_clock = Some((42, 72));
        sequencer.looping_sfx_voices[7] = Some(LoopingSfxVoice {
            step: program.steps[0],
            exact,
            overflows_remaining: 1,
            active: true,
            active_overflows: 193,
            gap_overflows: 2,
        });

        let mut frame = AudioEventFrame::from_route_and_dsp_writes(AudioRouteState::default(), &[]);
        let mut stats = ModernAudioSequenceStats::default();
        sequencer.advance_looping_sfx(&mut frame, &mut stats);
        assert!(frame.events.iter().any(|event| {
            event.sample_offset == 214 && matches!(event.kind, AudioEventKind::NoteOff { voice: 7 })
        }));

        sequencer.previous_sfx_clock = Some((0, 8));
        let mut frame = AudioEventFrame::from_route_and_dsp_writes(AudioRouteState::default(), &[]);
        sequencer.advance_looping_sfx(&mut frame, &mut stats);
        assert!(frame.events.iter().any(|event| {
            event.sample_offset == 256
                && matches!(event.kind, AudioEventKind::NoteOn { voice: 7, .. })
        }));
        let looping = sequencer.looping_sfx_voices[7].unwrap();
        assert!(looping.active);
        assert_eq!(looping.overflows_remaining, 192);
    }

    #[test]
    fn duration_bounded_sfx_voice_expires_from_variant_context() {
        let mut sequencer = ModernAudioSequencer::default();
        let route = AudioRouteState {
            queue: AudioQueueState {
                input: [0, 0xfe, 0, 0],
                ..AudioQueueState::default()
            },
            ..AudioRouteState::default()
        };

        sequencer.sequence_route(route);
        assert_eq!(sequencer.active_voice_mask & (1 << 1), 1 << 1);

        for _ in 0..6 {
            sequencer.sequence_route(route);
        }

        assert_eq!(sequencer.active_voice_mask & (1 << 1), 0);
    }

    #[test]
    fn same_voice_catalog_steps_advance_across_declared_frame_durations() {
        let mut sequencer = ModernAudioSequencer::default();
        let route = AudioRouteState {
            queue: AudioQueueState {
                input: [0, 0, 0, 0x0a],
                ..AudioQueueState::default()
            },
            ..AudioRouteState::default()
        };

        let program = lookup_sfx_program_for_context(
            2,
            0x0a,
            ModernSfxRuntimeContext {
                source_slot: 2,
                active_voice_mask: 0,
            },
        )
        .unwrap();
        assert_eq!(program.variant_hash, 0xdea3d882);
        assert_eq!(
            exact_sfx_dsp_step(2, 0x0a, program.variant_hash, 0, program.steps[0])
                .unwrap()
                .command_delay_frames,
            2
        );

        let command = sequencer.sequence_route(route);
        let lead_in = sequencer.sequence_route(route);
        let first = sequencer.sequence_route(route);
        let inter_step_gap = sequencer.sequence_route(route);
        let inter_step_gap_2 = sequencer.sequence_route(route);
        let second = sequencer.sequence_route(route);

        assert!(!command
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::NoteOn { voice: 7, .. })));
        assert!(!lead_in
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::NoteOn { voice: 7, .. })));

        assert_eq!(
            first
                .events
                .iter()
                .filter(|event| matches!(event.kind, AudioEventKind::NoteOn { voice: 7, .. }))
                .count(),
            1
        );
        assert!([inter_step_gap, inter_step_gap_2].iter().all(|frame| !frame
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::NoteOn { voice: 7, .. }))));
        assert_eq!(
            second
                .events
                .iter()
                .filter(|event| matches!(event.kind, AudioEventKind::NoteOn { voice: 7, .. }))
                .count(),
            1
        );
    }

    #[test]
    fn catalog_steps_emit_route_derived_stereo_pan() {
        let mut sequencer = ModernAudioSequencer::default();
        let route = AudioRouteState {
            queue: AudioQueueState {
                input: [0, 0x03, 0, 0],
                ..AudioQueueState::default()
            },
            ..AudioRouteState::default()
        };
        sequencer.sequence_route(route);
        sequencer.sequence_route(route);
        let frame = sequencer.sequence_route(route);

        assert!(frame.events.iter().any(|event| matches!(
            event.kind,
            AudioEventKind::SetPan {
                voice: 7,
                pan: -127
            }
        )));
        assert!(frame
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::SetPan { voice: 6, pan: 127 })));
    }

    #[test]
    fn sequences_music_track_into_play_and_note_intents() {
        let mut sequencer = ModernAudioSequencer::default();
        let route = AudioRouteState {
            music: MusicControlState {
                music_control: 0x12,
                ..MusicControlState::default()
            },
            ..AudioRouteState::default()
        };
        let frame = sequencer.sequence_route(route);

        assert!(frame
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::PlayMusic { track: 0x12 })));
        assert!(!frame
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::NoteOn { .. })));
        assert_eq!(sequencer.last_stats().music_commands, 1);
        assert_eq!(sequencer.last_stats().note_events, 0);

        for _ in 0..3 {
            let lead_in = sequencer.sequence_route(route);
            assert!(!lead_in
                .events
                .iter()
                .any(|event| matches!(event.kind, AudioEventKind::NoteOn { .. })));
        }

        let first_notes = sequencer.sequence_route(route);
        assert_eq!(
            first_notes
                .events
                .iter()
                .filter(|event| matches!(event.kind, AudioEventKind::NoteOn { instrument: 10, .. }))
                .count(),
            2
        );
        assert_eq!(sequencer.last_stats().note_events, 2);
    }

    #[test]
    fn queued_track_outlives_stale_music_control_after_live_port_clears() {
        let mut sequencer = ModernAudioSequencer::default();
        sequencer.sequence_route(AudioRouteState {
            queue: AudioQueueState {
                input: [0x14, 0, 0, 0],
                ..AudioQueueState::default()
            },
            music: MusicControlState {
                music_control: 0x10,
                queued_music_control: 0x10,
                last_music_control: 0x14,
                ..MusicControlState::default()
            },
            ..AudioRouteState::default()
        });

        let continued = sequencer.sequence_route(AudioRouteState {
            music: MusicControlState {
                music_control: 0x10,
                queued_music_control: 0x10,
                last_music_control: 0x14,
                ..MusicControlState::default()
            },
            ..AudioRouteState::default()
        });

        assert_eq!(sequencer.last_music_track, 0x14);
        assert!(!continued
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::PlayMusic { track: 0x10 })));
    }

    #[test]
    fn transition_command_keeps_advancing_the_active_track() {
        let mut sequencer = ModernAudioSequencer::default();
        sequencer.sequence_route(AudioRouteState {
            queue: AudioQueueState {
                input: [0x01, 0, 0, 0],
                ..AudioQueueState::default()
            },
            ..AudioRouteState::default()
        });
        let before = sequencer.music_frame_position;

        sequencer.sequence_route(AudioRouteState {
            queue: AudioQueueState {
                input: [0xf1, 0, 0, 0],
                ..AudioQueueState::default()
            },
            ..AudioRouteState::default()
        });

        assert_eq!(sequencer.last_music_track, 0x01);
        assert_eq!(sequencer.music_frame_position, before + 1);
    }

    #[test]
    fn zero_lead_in_track_emits_its_first_notes_on_the_command_frame() {
        let mut sequencer = ModernAudioSequencer::default();
        let frame = sequencer.sequence_route(AudioRouteState {
            queue: AudioQueueState {
                input: [0x07, 0, 0, 0],
                ..AudioQueueState::default()
            },
            music: MusicControlState {
                last_music_control: 0x07,
                ..MusicControlState::default()
            },
            ..AudioRouteState::default()
        });

        assert!(frame.events.iter().any(|event| matches!(
            event.kind,
            AudioEventKind::NoteOn { instrument: 0, .. }
                | AudioEventKind::NoteOn { instrument: 9, .. }
                | AudioEventKind::NoteOn { instrument: 10, .. }
                | AudioEventKind::NoteOn { instrument: 22, .. }
        )));
        assert_eq!(sequencer.music_frame_position, 0);
    }

    #[test]
    fn active_track_one_uses_trace_backed_delayed_multi_note_sequence() {
        let mut sequencer = ModernAudioSequencer::default();
        let route = AudioRouteState {
            music: MusicControlState {
                last_music_control: 0x01,
                ..MusicControlState::default()
            },
            ..AudioRouteState::default()
        };

        let start = sequencer.sequence_route(route);
        assert!(start
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::PlayMusic { track: 0x01 })));
        assert!(!start
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::NoteOn { voice: 0, .. })));

        for _ in 0..5 {
            let frame = sequencer.sequence_route(route);
            assert!(!frame
                .events
                .iter()
                .any(|event| matches!(event.kind, AudioEventKind::NoteOn { voice: 0, .. })));
        }
        let first_note = sequencer.sequence_route(route);
        assert!(first_note.events.iter().any(|event| matches!(
            event.kind,
            AudioEventKind::NoteOn {
                voice: 1,
                pitch: 60,
                ..
            }
        )));

        for _ in 0..5 {
            sequencer.sequence_route(route);
        }
        let second_note = sequencer.sequence_route(route);
        assert!(second_note.events.iter().any(|event| matches!(
            event.kind,
            AudioEventKind::NoteOn {
                voice: 0,
                pitch: 53,
                ..
            }
        )));
    }

    #[test]
    fn active_track_0b_starts_trace_backed_stereo_chord_after_lead_in() {
        let mut sequencer = ModernAudioSequencer::default();
        let route = AudioRouteState {
            music: MusicControlState {
                last_music_control: 0x0b,
                ..MusicControlState::default()
            },
            ..AudioRouteState::default()
        };

        let start = sequencer.sequence_route(route);
        assert!(start
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::PlayMusic { track: 0x0b })));
        for _ in 0..24 {
            let frame = sequencer.sequence_route(route);
            assert!(!frame
                .events
                .iter()
                .any(|event| matches!(event.kind, AudioEventKind::NoteOn { .. })));
        }

        let chord = sequencer.sequence_route(route);
        assert_eq!(
            chord
                .events
                .iter()
                .filter(|event| matches!(event.kind, AudioEventKind::NoteOn { .. }))
                .count(),
            2
        );
        assert!(chord.events.iter().any(|event| matches!(
            event.kind,
            AudioEventKind::NoteOn {
                voice: 0,
                pitch: 90,
                ..
            }
        )));
        assert!(chord
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::SetPan { voice: 0, pan: 127 })));
    }

    #[test]
    fn sequences_sfx_ports_once_until_command_changes() {
        let mut sequencer = ModernAudioSequencer::default();
        let route = AudioRouteState {
            queue: AudioQueueState {
                input: [0, 0x34, 0, 0],
                ..AudioQueueState::default()
            },
            ..AudioRouteState::default()
        };

        let first = sequencer.sequence_route(route);
        let first_stats = sequencer.last_stats();
        let second = sequencer.sequence_route(route);

        assert!(first
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::PlaySfx { bank: 0, id: 0x34 })));
        assert!(first
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::NoteOn { voice: 1, .. })));
        assert!(first.events.iter().any(|event| matches!(
            event.kind,
            AudioEventKind::PitchSlide {
                voice: 1,
                target_pitch: 38,
                frames: 8
            }
        )));
        assert_eq!(first_stats.known_sfx_commands, 1);
        assert_eq!(first_stats.unknown_sfx_commands, 0);
        assert_eq!(first_stats.known_sfx_program_count, 1);
        assert_eq!(first_stats.known_sfx_programs[0], 0x0034);
        assert_ne!(first_stats.program_hash, 0);
        assert!(!second.events.iter().any(|event| matches!(
            event.kind,
            AudioEventKind::PlaySfx { .. } | AudioEventKind::NoteOn { .. }
        )));
    }

    #[test]
    fn clearing_sfx_port_does_not_key_off_slot_numbered_music_voice() {
        let mut sequencer = ModernAudioSequencer::default();
        sequencer.sequence_route(AudioRouteState {
            queue: AudioQueueState {
                input: [0, 0, 0x01, 0],
                ..AudioQueueState::default()
            },
            ..AudioRouteState::default()
        });

        let cleared = sequencer.sequence_route(AudioRouteState::default());

        assert!(!cleared
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::NoteOff { voice: 2 })));
    }

    #[test]
    fn apui00_music_command_is_not_duplicated_as_bank_six_sfx() {
        let mut sequencer = ModernAudioSequencer::default();
        let frame = sequencer.sequence_route(AudioRouteState {
            music: MusicControlState {
                apui00: 1,
                ..MusicControlState::default()
            },
            ..AudioRouteState::default()
        });

        assert!(frame
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::PlayMusic { track: 1 })));
        assert!(!frame
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::PlaySfx { bank: 6, .. })));
        assert_eq!(sequencer.last_stats().sfx_commands, 0);
    }

    #[test]
    fn semantic_sfx_waits_for_kon_and_emits_route_derived_echo_send() {
        let mut sequencer = ModernAudioSequencer::default();
        let command_route = AudioRouteState {
            queue: AudioQueueState {
                input: [0, 0, 0x01, 0],
                ..AudioQueueState::default()
            },
            spc: Some(SpcSequencerState {
                raw_volume_masks: [0x80; 32],
                raw_volume_count: 1,
                ..SpcSequencerState::default()
            }),
            ..AudioRouteState::default()
        };
        let command = sequencer.sequence_route(command_route);
        assert!(!command.events.iter().any(|event| matches!(
            event.kind,
            AudioEventKind::NoteOn { voice: 7, .. } | AudioEventKind::SetEchoSend { voice: 7, .. }
        )));

        let mut sfx_kon_sources = [[0; 8]; 8];
        sfx_kon_sources[0][7] = 3;
        let receipt = sequencer.sequence_route(AudioRouteState {
            spc: Some(SpcSequencerState {
                is_chan_on: 0x80,
                sfx_kon_masks: [0x80, 0, 0, 0, 0, 0, 0, 0],
                sfx_kon_offsets: [123, 0, 0, 0, 0, 0, 0, 0],
                sfx_kon_count: 1,
                sfx_kon_sources,
                sfx_setup_masks: [0x80, 0, 0, 0, 0, 0, 0, 0],
                sfx_setup_sources: [3, 0, 0, 0, 0, 0, 0, 0],
                sfx_setup_offsets: [123, 0, 0, 0, 0, 0, 0, 0],
                sfx_setup_count: 1,
                echo_enable_frame_start: 0,
                ..SpcSequencerState::default()
            }),
            ..AudioRouteState::default()
        });

        assert!(receipt.events.iter().any(|event| matches!(
            event.kind,
            AudioEventKind::SetEchoSend {
                voice: 7,
                enabled: false
            } if event.sample_offset == 123
        )));
        assert!(receipt.events.iter().any(|event| matches!(
            event.kind,
            AudioEventKind::KeyOnVoice { voice: 7, source: 3, .. }
                if event.sample_offset == 123
        )));
    }

    #[test]
    fn raw_keyoff_receipts_are_not_truncated_after_eight_writes() {
        let mut raw_kof_masks = [0; 32];
        let mut raw_kof_offsets = [0; 32];
        raw_kof_masks[8] = 0x02;
        raw_kof_offsets[8] = 298;

        let frame = ModernAudioSequencer::default().sequence_route(AudioRouteState {
            spc: Some(SpcSequencerState {
                raw_kof_masks,
                raw_kof_offsets,
                raw_kof_count: 9,
                ..SpcSequencerState::default()
            }),
            ..AudioRouteState::default()
        });

        assert!(frame.events.iter().any(|event| {
            event.sample_offset == 298 && matches!(event.kind, AudioEventKind::NoteOff { voice: 1 })
        }));
    }

    #[test]
    fn raw_envelope_register_receipts_preserve_active_voice_updates() {
        let mut raw_envelope_masks = [0; 32];
        let mut raw_envelope_registers = [0; 32];
        let mut raw_envelope_values = [0; 32];
        let mut raw_envelope_offsets = [0; 32];
        raw_envelope_masks[0] = 0x09;
        raw_envelope_registers[0] = 6;
        raw_envelope_values[0] = 0xe0;
        raw_envelope_offsets[0] = 91;

        let frame = ModernAudioSequencer::default().sequence_route(AudioRouteState {
            spc: Some(SpcSequencerState {
                raw_envelope_masks,
                raw_envelope_registers,
                raw_envelope_values,
                raw_envelope_offsets,
                raw_envelope_count: 1,
                ..SpcSequencerState::default()
            }),
            ..AudioRouteState::default()
        });

        for voice in [0, 3] {
            assert!(frame.events.iter().any(|event| {
                event.sample_offset == 91
                    && matches!(
                        event.kind,
                        AudioEventKind::VoiceParameter {
                            voice: event_voice,
                            parameter: VoiceParameterKind::Adsr2,
                            value: 0xe0,
                        } if event_voice == voice
                    )
            }));
        }
    }

    #[test]
    fn full_raw_volume_receipt_buffer_is_still_authoritative() {
        let mut raw_volume_masks = [0; 32];
        let mut raw_volume_left = [0; 32];
        let mut raw_volume_right = [0; 32];
        let mut raw_volume_offsets = [0; 32];
        raw_volume_masks[31] = 0x01;
        raw_volume_left[31] = 15;
        raw_volume_right[31] = -2;
        raw_volume_offsets[31] = 524;

        let frame = ModernAudioSequencer::default().sequence_route(AudioRouteState {
            spc: Some(SpcSequencerState {
                raw_volume_masks,
                raw_volume_left,
                raw_volume_right,
                raw_volume_offsets,
                raw_volume_count: 32,
                ..SpcSequencerState::default()
            }),
            ..AudioRouteState::default()
        });

        assert!(frame.events.iter().any(|event| {
            event.sample_offset == 524
                && matches!(
                    event.kind,
                    AudioEventKind::SetStereoVolume {
                        voice: 0,
                        left: 15,
                        right: -2,
                    }
                )
        }));
    }

    #[test]
    fn music_keyon_uses_volume_at_kon_not_end_of_frame_volume() {
        let mut sfx_kon_rate_counters = [[0; 8]; 8];
        sfx_kon_rate_counters[0][7] = 1;
        let mut sfx_kon_volume_left = [[0; 8]; 8];
        let mut sfx_kon_volume_right = [[0; 8]; 8];
        sfx_kon_volume_left[0][7] = 23;
        sfx_kon_volume_right[0][7] = 0;
        let mut raw_volume_masks = [0; 32];
        let mut raw_volume_left = [0; 32];
        let mut raw_volume_right = [0; 32];
        let mut raw_volume_offsets = [0; 32];
        raw_volume_masks[..2].copy_from_slice(&[0x80, 0x80]);
        raw_volume_left[..2].copy_from_slice(&[23, 23]);
        raw_volume_right[..2].copy_from_slice(&[0, 1]);
        raw_volume_offsets[..2].copy_from_slice(&[50, 498]);

        let frame = ModernAudioSequencer::default().sequence_route(AudioRouteState {
            spc: Some(SpcSequencerState {
                sfx_kon_masks: [0x80, 0, 0, 0, 0, 0, 0, 0],
                sfx_kon_offsets: [370, 0, 0, 0, 0, 0, 0, 0],
                sfx_kon_count: 1,
                sfx_kon_rate_counters,
                sfx_kon_volume_left,
                sfx_kon_volume_right,
                raw_volume_masks,
                raw_volume_left,
                raw_volume_right,
                raw_volume_offsets,
                raw_volume_count: 2,
                voice_volume_left: [23; 8],
                voice_volume_right: [1; 8],
                ..SpcSequencerState::default()
            }),
            ..AudioRouteState::default()
        });

        assert!(frame.events.iter().any(|event| {
            event.sample_offset == 370
                && matches!(
                    event.kind,
                    AudioEventKind::KeyOnVoice {
                        voice: 7,
                        volume_left: 23,
                        volume_right: 0,
                        rate_counter: 1,
                        ..
                    }
                )
        }));
    }

    #[test]
    fn engine_receipt_mode_suppresses_route_derived_music_registers() {
        let mut sequencer = ModernAudioSequencer::default();
        let mut fallback =
            AudioEventFrame::from_route_and_dsp_writes(AudioRouteState::default(), &[]);
        sequencer.emit_music_globals_at_position(1, 6, &mut fallback);
        assert!(fallback
            .events
            .iter()
            .any(|event| { matches!(event.kind, AudioEventKind::VoiceParameter { .. }) }));

        sequencer.engine_receipt_mode = true;
        let mut live = AudioEventFrame::from_route_and_dsp_writes(AudioRouteState::default(), &[]);
        sequencer.emit_music_globals_at_position(1, 6, &mut live);
        assert!(!live.events.iter().any(|event| {
            matches!(
                event.kind,
                AudioEventKind::VoiceParameter { .. } | AudioEventKind::GlobalParameter { .. }
            )
        }));
    }

    #[test]
    fn unknown_sfx_uses_fallback_and_reports_coverage_gap() {
        let mut sequencer = ModernAudioSequencer::default();
        let frame = sequencer.sequence_route(AudioRouteState {
            queue: AudioQueueState {
                input: [0, 0xfe, 0, 0],
                ..AudioQueueState::default()
            },
            ..AudioRouteState::default()
        });

        assert!(frame
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::NoteOn { voice: 1, .. })));
        assert_eq!(sequencer.last_stats().known_sfx_commands, 0);
        assert_eq!(sequencer.last_stats().unknown_sfx_commands, 1);
        assert_eq!(sequencer.last_stats().unknown_sfx_program_count, 1);
        assert_eq!(sequencer.last_stats().unknown_sfx_programs[0], 0x00fe);
        assert_eq!(sequencer.last_stats().fallback_sfx_commands, 1);
        assert_eq!(sequencer.last_stats().program_hash, 0);
    }

    #[test]
    fn context_only_sfx_is_known_without_inventing_a_fallback_note() {
        let mut sequencer = ModernAudioSequencer::default();
        let frame = sequencer.sequence_route(AudioRouteState {
            queue: AudioQueueState {
                input: [0, 0, 0, 0x83],
                ..AudioQueueState::default()
            },
            ..AudioRouteState::default()
        });

        assert!(frame
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::PlaySfx { bank: 2, id: 0x83 })));
        assert!(!frame
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::NoteOn { .. })));
        assert_eq!(sequencer.last_stats().known_sfx_commands, 1);
        assert_eq!(sequencer.last_stats().fallback_sfx_commands, 0);
    }
}
