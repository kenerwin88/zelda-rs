//! Coarse work budgets for lagless presentation. No receipt or cycle ledger
//! participates in these decisions. A unit is half a raster line of work.
use super::*;

#[derive(Clone, Debug, Default)]
pub(crate) struct NativePacing {
    iris_work: u32,
    pending: u32,
}

impl ZeldaState {
    pub fn native_held_frames_pending(&self) -> u32 {
        if self.lag_free { self.native_pacing.pending } else { 0 }
    }

    /// One real presentation frame with no main-loop, recorder, RNG or input
    /// advancement. The software update latch retains the ordinary NMI gate:
    /// sound delivery and ready polygon uploads still run, core updates do not
    /// repeat. Existing polygon lag phases already ran at the closing NMI.
    pub(super) fn run_native_held_frame(&mut self, input: u16) {
        self.native_pacing.pending -= 1;
        self.frame_ctr_dbg = self.frame_ctr_dbg.wrapping_add(1);
        self.capture_display_snapshot();
        self.interrupt_nmi(input, None, false);
        self.zelda_push_apu_state();
        self.poison_owned_wram();
    }

    pub(super) fn native_iris_row_work(&mut self, circle: bool) {
        if self.lag_free && self.lag_free_intro_schedule.is_none() {
            // $00:F361..F3A0 tests and stores a pair of rows. The divide,
            // lookup, multiply and clamps in $F4CC..F53D add two work units.
            self.native_pacing.iris_work += 1 + 2 * u32::from(circle);
        }
    }

    pub(super) fn native_iris_projection_work(&mut self) {
        let work = std::mem::take(&mut self.native_pacing.iris_work);
        if !self.lag_free || work == 0 { return; }
        // Main enters after the preceding NMI and its caller prefix. Use a
        // coarse entry at line 250, not an instruction/raster CPU model.
        // $F3A3..F3B2 waits for V >= 192 before $F3B7..F3C3 copies 224
        // words. Reserve 28 lines for that programmed copy and eight for
        // the radius/control/caller suffix. These are throughput estimates,
        // independent of scene, room, crystal, iteration and ledger counts.
        const FIELD: u32 = 262 * 2;
        let mut end = 250 * 2 + work;
        let line = end % FIELD;
        if line < 192 * 2 { end += 192 * 2 - line; }
        end += 28 * 2 + 8 * 2;
        let next_nmi = FIELD + 225 * 2;
        if end >= next_nmi {
            self.native_pacing.pending += 1 + (end - next_nmi) / FIELD;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn work_not_scene_selects_holds_and_default_never_holds() {
        let mut game = ZeldaState::default();
        game.native_iris_row_work(true);
        game.native_iris_projection_work();
        assert_eq!(game.native_held_frames_pending(), 0);
        game.set_lag_free(true);
        game.lag_free_intro_schedule = None;
        for _ in 0..189 { game.native_iris_row_work(false); }
        game.native_iris_projection_work();
        assert_eq!(game.native_held_frames_pending(), 1);
        game.native_pacing.pending = 0;
        for _ in 0..1000 { game.native_iris_row_work(true); }
        game.native_iris_projection_work();
        assert!(game.native_held_frames_pending() > 1);
    }
}
