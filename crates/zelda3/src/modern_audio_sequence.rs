use crate::game_output::{
    AudioEvent, AudioEventFrame, AudioEventKind, AudioRouteState, DspWriteEvent, MusicControlState,
};
use crate::modern_sfx_catalog::{
    lookup_sfx_program, sfx_program_hash, ModernSfxProgram, ModernSfxWaveform,
};

const SFX_SLOTS: usize = 7;

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
    pub program_hash: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ModernAudioSequencer {
    last_music_track: u8,
    last_sfx: [u8; SFX_SLOTS],
    last_stats: ModernAudioSequenceStats,
}

impl Default for ModernAudioSequencer {
    fn default() -> Self {
        Self {
            last_music_track: 0,
            last_sfx: [0; SFX_SLOTS],
            last_stats: ModernAudioSequenceStats::default(),
        }
    }
}

impl ModernAudioSequencer {
    pub fn sequence_route(&mut self, route: AudioRouteState) -> AudioEventFrame {
        let mut frame = AudioEventFrame::from_route_and_dsp_writes(route, &[]);
        let mut stats = ModernAudioSequenceStats::default();

        self.sequence_music(route.music, route.queue.input[0], &mut frame, &mut stats);
        self.sequence_sfx(route.music, route.queue.input, &mut frame, &mut stats);

        self.last_stats = stats;
        frame
    }

    pub fn sequence_parity_writes(
        &mut self,
        route: AudioRouteState,
        writes: &[DspWriteEvent],
    ) -> AudioEventFrame {
        let mut frame = AudioEventFrame::from_route_and_dsp_writes(route, writes);
        let mut stats = ModernAudioSequenceStats::default();
        self.sequence_music(route.music, route.queue.input[0], &mut frame, &mut stats);
        self.sequence_sfx(route.music, route.queue.input, &mut frame, &mut stats);
        self.last_stats = stats;
        frame
    }

    pub fn last_stats(&self) -> ModernAudioSequenceStats {
        self.last_stats
    }

    fn sequence_music(
        &mut self,
        music: MusicControlState,
        port0: u8,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        let track = first_nonzero([port0, music.music_control, music.queued_music_control]);
        if track == self.last_music_track {
            return;
        }

        if track == 0 || track == 0xf0 {
            if self.last_music_track != 0 {
                push_event(frame, AudioEventKind::StopMusic);
                push_event(frame, AudioEventKind::NoteOff { voice: 0 });
                stats.music_commands += 1;
                stats.note_events += 1;
            }
            self.last_music_track = 0;
            return;
        }

        if track >= 0xf1 {
            stats.ignored_commands += 1;
            self.last_music_track = track;
            return;
        }

        push_event(frame, AudioEventKind::PlayMusic { track });
        push_event(
            frame,
            AudioEventKind::SetTempo {
                value: tempo_for_track(track),
            },
        );
        push_event(
            frame,
            AudioEventKind::SetEnvelope {
                voice: 0,
                attack: 2,
                decay: 4 + (track & 3),
                sustain: 10,
                release: 4,
            },
        );
        push_event(
            frame,
            AudioEventKind::NoteOn {
                voice: 0,
                pitch: pitch_for_code(track),
                instrument: instrument_for_code(track),
                volume: 88,
            },
        );
        stats.music_commands += 1;
        stats.note_events += 1;
        stats.envelope_events += 1;
        self.last_music_track = track;
    }

    fn sequence_sfx(
        &mut self,
        music: MusicControlState,
        ports: [u8; 4],
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        let candidates = [
            ports[1],
            ports[2],
            ports[3],
            music.sound_effect_ambient,
            music.sound_effect_1,
            music.sound_effect_2,
            music.apui00,
        ];

        for (slot, code) in candidates.into_iter().enumerate() {
            if code == self.last_sfx[slot] {
                continue;
            }
            let voice = (slot + 1).min(7) as u8;
            if code == 0 {
                if self.last_sfx[slot] != 0 {
                    push_event(frame, AudioEventKind::NoteOff { voice });
                    stats.note_events += 1;
                }
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

            if let Some(program) = lookup_sfx_program(slot as u8, code) {
                self.expand_sfx_program(frame, program, stats);
                stats.known_sfx_commands += 1;
                stats.program_hash =
                    fold_program_hash(stats.program_hash, sfx_program_hash(program));
            } else {
                self.expand_fallback_sfx(frame, voice, code, slot as u8, stats);
                stats.unknown_sfx_commands += 1;
                stats.fallback_sfx_commands += 1;
            }
            stats.sfx_commands += 1;
            self.last_sfx[slot] = code;
        }
    }

    fn expand_sfx_program(
        &self,
        frame: &mut AudioEventFrame,
        program: &ModernSfxProgram,
        stats: &mut ModernAudioSequenceStats,
    ) {
        for step in program.steps {
            push_event(
                frame,
                AudioEventKind::SetNoise {
                    voice: step.voice,
                    enabled: matches!(step.waveform, ModernSfxWaveform::Noise),
                },
            );
            push_event(
                frame,
                AudioEventKind::SetEnvelope {
                    voice: step.voice,
                    attack: step.envelope.attack,
                    decay: step.envelope.decay,
                    sustain: step.envelope.sustain,
                    release: step.envelope.release,
                },
            );
            push_event(
                frame,
                AudioEventKind::NoteOn {
                    voice: step.voice,
                    pitch: step.pitch,
                    instrument: step.instrument,
                    volume: step.volume,
                },
            );
            push_event(
                frame,
                AudioEventKind::SetDuration {
                    voice: step.voice,
                    frames: step.duration_frames,
                },
            );
            if let Some(slide) = step.pitch_slide {
                push_event(
                    frame,
                    AudioEventKind::PitchSlide {
                        voice: step.voice,
                        target_pitch: slide.target_pitch,
                        frames: slide.frames,
                    },
                );
            }
            stats.note_events += 1;
            stats.envelope_events += 1;
        }
    }

    fn expand_fallback_sfx(
        &self,
        frame: &mut AudioEventFrame,
        voice: u8,
        code: u8,
        slot: u8,
        stats: &mut ModernAudioSequenceStats,
    ) {
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
        push_event(frame, AudioEventKind::SetDuration { voice, frames: 6 });
        stats.note_events += 1;
        stats.envelope_events += 1;
    }
}

fn push_event(frame: &mut AudioEventFrame, kind: AudioEventKind) {
    frame.events.push(AudioEvent {
        sample_offset: 0,
        timer_cycles: 0,
        kind,
        parity_dsp: None,
    });
}

fn first_nonzero(values: [u8; 3]) -> u8 {
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
    use crate::game_output::{AudioQueueState, MusicControlState};

    #[test]
    fn sequences_music_track_into_play_and_note_intents() {
        let mut sequencer = ModernAudioSequencer::default();
        let frame = sequencer.sequence_route(AudioRouteState {
            music: MusicControlState {
                music_control: 0x12,
                ..MusicControlState::default()
            },
            ..AudioRouteState::default()
        });

        assert!(frame
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::PlayMusic { track: 0x12 })));
        assert!(frame.events.iter().any(|event| matches!(
            event.kind,
            AudioEventKind::NoteOn {
                voice: 0,
                instrument: 1,
                ..
            }
        )));
        assert_eq!(sequencer.last_stats().music_commands, 1);
        assert_eq!(sequencer.last_stats().note_events, 1);
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
        assert_ne!(first_stats.program_hash, 0);
        assert!(!second.events.iter().any(|event| matches!(
            event.kind,
            AudioEventKind::PlaySfx { .. } | AudioEventKind::NoteOn { .. }
        )));
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
        assert_eq!(sequencer.last_stats().fallback_sfx_commands, 1);
        assert_eq!(sequencer.last_stats().program_hash, 0);
    }
}
