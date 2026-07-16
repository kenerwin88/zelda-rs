use super::*;

fn oracle_unavailable() -> ! {
    panic!("DSP audio diagnostics require the audio-oracle feature")
}

impl ZeldaState {
    pub fn zelda_oracle_aligned_modern_audio_trace_state(
        &self,
    ) -> (ModernAudioSequencer, ModernAudioEngine) {
        oracle_unavailable()
    }

    pub fn zelda_sync_modern_audio_trace_engine(
        &self,
        _engine: &mut ModernAudioEngine,
        _rewind_samples: u16,
    ) {
        oracle_unavailable()
    }

    pub fn zelda_audio_dsp_hash(&self) -> u32 {
        oracle_unavailable()
    }

    pub fn zelda_audio_dsp_snapshot(&self) -> Vec<u8> {
        oracle_unavailable()
    }

    pub fn zelda_audio_dsp_global_state(&self) -> crate::game_output::ClassicDspGlobalState {
        oracle_unavailable()
    }

    pub fn zelda_audio_dsp_voice_states(&self) -> [crate::game_output::ClassicDspVoiceState; 8] {
        oracle_unavailable()
    }

    pub fn zelda_audio_dsp_debug_voice_samples(&self) -> [Vec<i16>; 8] {
        oracle_unavailable()
    }

    pub fn zelda_render_audio_trace_dsp_events(
        &mut self,
        _audio_buffer: &mut [i16],
        _samples: i32,
        _channels: i32,
    ) -> Vec<DspWriteEvent> {
        oracle_unavailable()
    }

    pub fn zelda_prepare_audio_trace_dsp(&mut self) {
        oracle_unavailable()
    }

    pub fn zelda_render_prepared_audio_trace_dsp_events(
        &mut self,
        _audio_buffer: &mut [i16],
        _dsp_only_audio_buffer: &mut [i16],
        _samples: i32,
        _channels: i32,
    ) -> Vec<DspWriteEvent> {
        oracle_unavailable()
    }
}
