use super::*;

impl ZeldaState {
    /// Clone Modern state and align its echo history to the legacy DSP oracle
    /// for side-by-side trace diagnostics.
    pub fn zelda_oracle_aligned_modern_audio_trace_state(
        &self,
    ) -> (ModernAudioSequencer, ModernAudioEngine) {
        let mut engine = self.audio.modern_audio.clone();
        self.zelda_sync_modern_audio_trace_engine(&mut engine, 0);
        (self.audio.modern_sequence.clone(), engine)
    }

    pub fn zelda_sync_modern_audio_trace_engine(
        &self,
        engine: &mut ModernAudioEngine,
        rewind_samples: u16,
    ) {
        let dsp = crate::spc_player::spc_player_save_dsp_c_saveload(self.audio.spc_player);
        let ram = crate::spc_player::spc_player_save_ram(self.audio.spc_player);
        let word = |offset| i16::from_le_bytes([dsp[offset], dsp[offset + 1]]);
        engine.seed_echo_checkpoint_state(
            &ram,
            dsp[0x6d],
            dsp[0x7d],
            u16::from_le_bytes([dsp[840], dsp[841]]),
            u16::from_le_bytes([dsp[838], dsp[839]]),
            dsp[842],
            std::array::from_fn(|index| word(852 + index * 2)),
            std::array::from_fn(|index| word(868 + index * 2)),
            rewind_samples,
        );
    }

    pub fn zelda_audio_dsp_hash(&self) -> u32 {
        let mut hash = 2166136261u32;
        for byte in crate::spc_player::spc_player_save_dsp_c_saveload(self.audio.spc_player) {
            hash = (hash ^ u32::from(byte)).wrapping_mul(16777619);
        }
        hash
    }

    pub fn zelda_audio_dsp_snapshot(&self) -> Vec<u8> {
        crate::spc_player::spc_player_save_dsp_c_saveload(self.audio.spc_player)
    }

    pub fn zelda_audio_dsp_global_state(&self) -> crate::game_output::ClassicDspGlobalState {
        let bytes = crate::spc_player::spc_player_save_dsp_c_saveload(self.audio.spc_player);
        crate::game_output::ClassicDspGlobalState {
            master_volume_left: bytes[821] as i8,
            master_volume_right: bytes[822] as i8,
            echo_volume_left: bytes[831] as i8,
            echo_volume_right: bytes[832] as i8,
            echo_feedback: bytes[833] as i8,
            flags: bytes[0x6c],
            echo_enable_mask: bytes[0x4d],
            pitch_modulation_mask: bytes[0x2d],
            noise_enable_mask: bytes[0x3d],
            echo_start_page: bytes[0x6d],
            echo_delay: bytes[0x7d],
            fir: std::array::from_fn(|index| bytes[843 + index] as i8),
            echo_buffer_index: u16::from_le_bytes([bytes[840], bytes[841]]),
            echo_remaining: u16::from_le_bytes([bytes[838], bytes[839]]),
            fir_history_index: bytes[842],
            fir_history_left: std::array::from_fn(|index| {
                let offset = 852 + index * 2;
                i16::from_le_bytes([bytes[offset], bytes[offset + 1]])
            }),
            fir_history_right: std::array::from_fn(|index| {
                let offset = 868 + index * 2;
                i16::from_le_bytes([bytes[offset], bytes[offset + 1]])
            }),
        }
    }

    pub fn zelda_audio_dsp_voice_states(&self) -> [crate::game_output::ClassicDspVoiceState; 8] {
        let bytes = crate::spc_player::spc_player_save_dsp_c_saveload(self.audio.spc_player);
        std::array::from_fn(|voice| {
            let base = 0x80 + voice * 86;
            let word =
                |offset| u16::from_le_bytes([bytes[base + offset], bytes[base + offset + 1]]);
            crate::game_output::ClassicDspVoiceState {
                pitch: word(0),
                pitch_counter: word(2),
                source: bytes[base + 44],
                envelope_state: bytes[base + 66],
                envelope_rate_counter: word(64),
                gain: word(76),
                sample_out: word(80) as i16,
                volume_left: bytes[base + 82] as i8,
                volume_right: bytes[base + 83] as i8,
            }
        })
    }

    pub fn zelda_audio_dsp_debug_voice_samples(&self) -> [Vec<i16>; 8] {
        crate::spc_player::spc_player_dsp_debug_voice_samples(self.audio.spc_player)
    }

    pub fn zelda_render_audio_trace_dsp(
        &mut self,
        audio_buffer: &mut [i16],
        samples: i32,
        channels: i32,
    ) -> Vec<(u8, u8, i32, u8)> {
        self.zelda_pop_apu_state();
        let count = (samples.max(0) as usize).saturating_mul(channels.max(0) as usize);
        let mut writes = Vec::new();
        if samples > 0 && channels > 0 {
            if let Some(player) = unsafe { self.audio.spc_player.as_mut() } {
                player.input_ports = self.audio.modern_apu.input_ports;
                let mut hist = crate::spc_player::DspRegWriteHistory::default();
                player.reg_write_history = &mut hist;
                crate::spc_player::spc_player_generate_samples(player);
                player.reg_write_history = std::ptr::null_mut();
                writes.reserve(hist.count);
                for i in 0..hist.count {
                    writes.push((
                        hist.addr[i],
                        hist.val[i],
                        hist.sample_offset[i],
                        hist.timer_cycles[i],
                    ));
                }
                crate::spc_player::dsp_get_samples(
                    player.dsp,
                    audio_buffer,
                    samples as usize,
                    channels as usize,
                );
            } else {
                for value in audio_buffer.iter_mut().take(count) {
                    *value = 0;
                }
            }
        }
        if self.audio.msu_player.has_file && channels == 2 {
            self.msu_player_mix(audio_buffer, samples);
        }
        writes
    }

    pub fn zelda_render_audio_trace_dsp_events(
        &mut self,
        audio_buffer: &mut [i16],
        samples: i32,
        channels: i32,
    ) -> Vec<DspWriteEvent> {
        self.zelda_render_audio_trace_dsp(audio_buffer, samples, channels)
            .into_iter()
            .map(|(addr, value, sample_offset, timer_cycles)| {
                DspWriteEvent::new(addr, value, sample_offset, timer_cycles)
            })
            .collect()
    }
}
