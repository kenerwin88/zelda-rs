//! `ZeldaState` methods split out of `zelda_rtl.rs` by topic (audio).
//! Mechanical move: bodies are unchanged; private methods became
//! `pub(super)` so the parent module and its siblings can call them.

use super::*;

impl ZeldaState {
    pub(crate) fn set_music_control(&mut self, value: u8) {
        self.system_signals_mut().set_music_control(value);
    }

    pub(crate) fn set_current_music_control(&mut self, value: u8) {
        self.system_signals_mut().set_current_music_control(value);
    }

    pub(crate) fn set_last_music_control(&mut self, value: u8) {
        self.system_signals_mut().set_last_music_control(value);
    }

    pub(crate) fn set_queued_music_control(&mut self, value: u8) {
        self.system_signals_mut().set_queued_music_control(value);
    }

    pub(crate) fn set_apui00(&mut self, value: u8) {
        self.system_signals_mut().set_apui00(value);
    }

    pub(crate) fn save_current_music_as_last(&mut self) {
        self.system_signals_mut().save_current_music_as_last();
    }

    pub(crate) fn set_raw_sfx_pan_value(&mut self, value: u8) {
        self.system_signals_mut().set_raw_sfx_pan_value(value);
    }

    pub(crate) fn set_rupee_sfx_sound_delay(&mut self, value: u8) {
        self.hud_mut().set_rupee_sfx_sound_delay(value);
    }
}
