//! Lagless cost accounting. Runtime decisions use native work and hardware
//! field/interrupt ownership only; source lag counts remain census data.
use super::*;

#[derive(Clone, Debug, Default)]
pub(crate) struct NativePacing {
    pending: u32,
    pub(super) song_upload: bool,
    masked_callback: bool,
    accounting: bool,
    entry_handler: u64,
    closing_handler: Option<u64>,
    closing_poly_upload: bool,
    record_closing_handler: bool,
    wait_wall: [u64; 3],
    predictions: [u32; 3],
    early_sound: Option<[u8; 2]>,
    late_sound: Option<[u8; 2]>,

}

impl ZeldaState {
    pub(super) fn native_begin_main_cost(&mut self) {
        self.native_pacing.accounting = self.lag_free;
        // Presentation holds follow the native closing NMI, although their
        // source interruptions precede it. They must not replace the handler
        // through which the next main iteration actually enters.
        self.native_pacing.entry_handler = self.native_previous_closing_handler();
        if self.native_pacing.closing_handler.is_some() {
            self.lag_free_nmi_uploaded_poly = self.native_pacing.closing_poly_upload;
        }
        self.native_pacing.wait_wall = [0; 3];
        self.native_pacing.predictions = [0; 3];
        self.native_pacing.early_sound = None;
        self.native_pacing.late_sound = None;
    }

    pub(super) fn native_record_closing_handler(&mut self) {
        if self.native_pacing.record_closing_handler {
            self.native_pacing.closing_handler = self.last_nmi_handler_master_cycles().map(u64::from);
            self.native_pacing.closing_poly_upload = self.lag_free_nmi_uploaded_poly;
            self.native_pacing.record_closing_handler = false;
        }
    }

    pub(super) fn native_previous_closing_handler(&self) -> u64 {
        self.native_pacing.closing_handler
            .unwrap_or_else(|| u64::from(self.last_nmi_handler_master_cycles().unwrap_or(0)))
    }

    pub fn native_cost_entry_handler(&self) -> u64 {
        self.native_pacing.entry_handler
    }

    fn native_cost_elapsed(&self, main: u64) -> [u64; 3] {
        const FIELD: u64 = 1364 * 262;
        let handler = self.native_pacing.entry_handler.min(FIELD - 1);
        let worker = self.native_pacing_worker_share(handler).min(FIELD - handler - 1);
        let budget = FIELD - handler - worker;
        let refreshed = main + main.div_ceil(1324) * 40;
        [handler + main * FIELD / (FIELD - worker),
            main * FIELD / budget, refreshed * FIELD / budget]
    }

    /// Generic raster barrier: a V-counter loop exits at or after line L.
    /// Charge elapsed waiting separately from instruction work, since CPU
    /// work excludes NMI/worker ownership and refresh while wall time does not.
    pub(super) fn native_wait_until_line(&mut self, line: u16, loop_master: u64) {
        if !self.native_pacing.accounting { return; }
        const FIELD: u64 = 1364 * 262;
        const NMI: u64 = 225 * 1364;
        let main = crate::cycle_ledger::master().saturating_sub(self.lag_free_frame_master_start)
            + self.lag_free_vwf_master;
        let elapsed = self.native_cost_elapsed(main);
        for (i, work) in elapsed.into_iter().enumerate() {
            let raster = (NMI + work + self.native_pacing.wait_wall[i]) % FIELD;
            let target = u64::from(line) * 1364;
            if raster < target {
                self.native_pacing.wait_wall[i] += (target - raster).div_ceil(loop_master) * loop_master;
            }
        }
    }

    /// Retain the queue consumed by an elapsed NMI before a later native
    /// consumer inspects it. Delivery is published on the first callback;
    /// commands queued by the remaining main suffix reach the final one.
    pub(super) fn native_checkpoint_sound_queue(&mut self) {
        if !self.native_pacing.accounting || self.native_pacing.song_upload
            || self.native_pacing.early_sound.is_some() { return; }
        let main = crate::cycle_ledger::master().saturating_sub(self.lag_free_frame_master_start)
            + self.lag_free_vwf_master;
        if self.native_cost_elapsed(main)[2] + self.native_pacing.wait_wall[2] < 1364 * 262 {
            return;
        }
        let sounds = [self.game_state.system_signals.sound_effect_1(),
            self.game_state.system_signals.sound_effect_2()];
        self.native_pacing.early_sound = Some(sounds);
        // $00:811E..812D transfers and clears these queues at every NMI.
        self.set_sound_effect_1(0);
        self.set_sound_effect_2(0);
    }

    pub fn native_cost_predictions(&self) -> [u32; 3] {
        self.native_pacing.predictions
    }

    pub(super) fn native_finish_main_cost(&mut self) {
        if !self.native_pacing.accounting { return; }
        self.native_pacing.accounting = false;
        self.native_pacing.record_closing_handler = true;
        let main = crate::cycle_ledger::master().saturating_sub(self.lag_free_frame_master_start)
            + self.lag_free_vwf_master;
        let elapsed = self.native_cost_elapsed(main);
        self.native_pacing.predictions = std::array::from_fn(|i|
            ((elapsed[i] + self.native_pacing.wait_wall[i]) / (1364 * 262)) as u32);
        // P2 is selected by the complete route census. Keep the existing
        // coarse iris envelope until the raster-barrier comparison passes.
        self.native_pacing.pending = self.native_pacing.pending.max(self.native_pacing.predictions[2]);
        if let Some(sounds) = self.native_pacing.early_sound.take() {
            self.native_pacing.late_sound = Some([
                self.game_state.system_signals.sound_effect_1(),
                self.game_state.system_signals.sound_effect_2()]);
            self.set_sound_effect_1(sounds[0]);
            self.set_sound_effect_2(sounds[1]);
        }
    }

    pub fn native_presentation_pending(&self) -> bool {
        self.native_held_frames_pending() != 0 || self.lag_free && self.native_pacing.song_upload
    }

    /// Elapsed audio while $4200 masks NMI is not an accepted lag NMI.
    pub fn native_last_callback_masked_nmi(&self) -> bool {
        self.lag_free && self.native_pacing.masked_callback
    }

    pub(crate) fn native_begin_song_upload(&mut self) {
        if self.lag_free && self.lag_free_intro_schedule.is_none() {
            self.native_pacing.song_upload = true;
        }
    }

    pub(crate) fn native_mask_song_upload_nmi(&mut self) -> bool {
        if self.lag_free && self.native_pacing.song_upload {
            self.native_pacing.masked_callback = true;
            true
        } else { false }
    }

    pub(super) fn native_begin_callback(&mut self) {
        self.native_pacing.masked_callback = false;
    }

    pub(super) fn run_native_song_upload_frame(&mut self, input: u16) {
        self.frame_ctr_dbg = self.frame_ctr_dbg.wrapping_add(1);
        self.capture_display_snapshot();
        let transferring = self.native_song_bank_transfer_pending();
        if transferring {
            self.native_pacing.masked_callback = true;
        } else {
            // The retained receiver has acknowledged the final block and
            // cleared the ports. Restore the closing NMI, exactly once.
            self.native_pacing.song_upload = false;
            self.interrupt_nmi(input, None, false);
        }
        self.zelda_push_apu_state();
        self.poison_owned_wram();
    }

    pub fn native_held_frames_pending(&self) -> u32 {
        if self.lag_free { self.native_pacing.pending } else { 0 }
    }

    /// One real presentation frame with no main-loop, recorder, RNG or input
    /// advancement. The software update latch retains the ordinary NMI gate:
    /// sound delivery and ready polygon uploads still run, core updates do not
    /// repeat. Existing polygon lag phases already ran at the closing NMI.
    pub(super) fn run_native_held_frame(&mut self, input: u16) {
        self.native_pacing.pending -= 1;
        if self.native_pacing.pending == 0 {
            if let Some(sounds) = self.native_pacing.late_sound.take() {
                self.set_sound_effect_1(sounds[0]);
                self.set_sound_effect_2(sounds[1]);
            }
        }
        self.frame_ctr_dbg = self.frame_ctr_dbg.wrapping_add(1);
        self.capture_display_snapshot();
        self.interrupt_nmi(input, None, false);
        self.zelda_push_apu_state();
        self.poison_owned_wram();
    }


}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raster_barrier_counts_fields_once_across_held_callbacks() {
        let mut game = ZeldaState::default();
        game.set_lag_free(true);
        game.lag_free_intro_schedule = None;
        game.lag_free_frame_master_start = crate::cycle_ledger::master();
        game.lag_free_vwf_master = 0;
        game.native_begin_main_cost();
        game.native_pacing.entry_handler = 0;
        // Cross V0, wait to V192, then cross the next NMI at V225.
        crate::cycle_ledger::charge(60_000);
        game.native_wait_until_line(192, 178);
        crate::cycle_ledger::charge(50_000);
        game.native_finish_main_cost();
        assert_eq!(game.native_cost_predictions(), [1, 1, 1]);
        assert_eq!(game.native_held_frames_pending(), 1);
        game.native_finish_main_cost();
        assert_eq!(game.native_held_frames_pending(), 1);
        game.set_lag_free(false);
        assert_eq!(game.native_held_frames_pending(), 0);
    }

    #[test]
    fn elapsed_sound_queue_is_delivered_before_the_hud_suffix_queue() {
        let mut game = ZeldaState::default();
        game.set_lag_free(true);
        game.lag_free_intro_schedule = None;
        game.lag_free_lag_phases_run = true;
        game.latch_nmi_update();
        game.lag_free_frame_master_start = crate::cycle_ledger::master();
        game.lag_free_vwf_master = 0;
        game.native_begin_main_cost();
        game.native_pacing.entry_handler = 0;
        game.set_sound_effect_1(0x60);
        crate::cycle_ledger::charge(2 * 1364 * 262);
        game.native_checkpoint_sound_queue();
        assert_eq!(game.game_state.system_signals.sound_effect_1(), 0);
        game.set_sound_effect_1(0x29);
        game.native_finish_main_cost();
        game.interrupt_nmi(0, None, false);
        game.zelda_push_apu_state();
        assert_eq!(game.zelda_last_pushed_apu_ports().1[2], 0x60);
        game.run_native_held_frame(0);
        game.run_native_held_frame(0);
        assert_eq!(game.zelda_last_pushed_apu_ports().1[2], 0x29);
        assert_eq!(game.native_held_frames_pending(), 0);
        assert_eq!(game.game_state.system_signals.sound_effect_1(), 0);
    }

    #[test]
    fn presentation_holds_preserve_the_next_mains_closing_handler_cost() {
        let mut game = ZeldaState::default();
        game.set_lag_free(true);
        game.lag_free_intro_schedule = None;
        game.lag_free_lag_phases_run = true;
        game.lag_free_frame_master_start = crate::cycle_ledger::master();
        game.lag_free_vwf_master = 0;
        game.native_begin_main_cost();
        game.native_pacing.entry_handler = 0;
        crate::cycle_ledger::charge(1364 * 262);
        game.native_finish_main_cost();
        game.interrupt_nmi(0, None, false);
        let closing = game.last_nmi_handler_master_cycles().unwrap();
        game.lag_free_nmi_uploaded_poly = true;
        game.native_pacing.record_closing_handler = true;
        game.native_record_closing_handler();
        game.run_native_held_frame(0);
        game.native_begin_main_cost();
        assert_eq!(game.native_cost_entry_handler(), u64::from(closing));
        assert!(game.lag_free_nmi_uploaded_poly);
        game.set_lag_free(false);
        assert!(game.native_pacing.closing_handler.is_none());
    }

    #[test]
    fn masked_upload_preserves_nmi_commands_and_resumes_closing_nmi_once() {
        let mut game = ZeldaState::default();
        game.set_lag_free(true);
        game.lag_free_intro_schedule = None;
        game.lag_free_lag_phases_run = true;
        game.latch_nmi_update();
        game.set_sound_effect_1(0x29);
        let frame = game.game_state.frame;
        let recorder = game.state_recorder.total_frames;
        game.native_begin_song_upload();
        game.interrupt_nmi(0, None, false);
        assert!(game.native_last_callback_masked_nmi());
        assert_eq!(game.game_state.system_signals.sound_effect_1(), 0x29);
        assert_eq!(game.native_held_frames_pending(), 0);
        assert!(game.native_presentation_pending());
        // No receiver is installed in this fixture: the upload has returned.
        game.zelda_run_frame(0);
        assert!(!game.native_last_callback_masked_nmi());
        assert!(!game.native_presentation_pending());
        assert_eq!(game.game_state.frame, frame);
        assert_eq!(game.state_recorder.total_frames, recorder);
        assert_eq!(game.game_state.system_signals.sound_effect_1(), 0);
        game.native_begin_song_upload();
        game.set_lag_free(false);
        assert!(!game.native_presentation_pending());
        assert!(!game.native_pacing.song_upload);
    }

    #[test]
    fn held_frame_preserves_main_and_recorder_but_delivers_sound() {
        let mut game = ZeldaState::default();
        game.set_lag_free(true);
        game.lag_free_intro_schedule = None;
        game.lag_free_lag_phases_run = true;
        game.latch_nmi_update();
        game.set_sound_effect_1(0x29);
        game.native_pacing.pending = 2;
        let frame = game.game_state.frame;
        let recorder = game.state_recorder.total_frames;
        game.run_native_held_frame(0);
        assert_eq!(game.game_state.frame, frame);
        assert_eq!(game.state_recorder.total_frames, recorder);
        assert_eq!(game.native_held_frames_pending(), 1);
        assert_eq!(game.game_state.system_signals.sound_effect_1(), 0);
    }

}
