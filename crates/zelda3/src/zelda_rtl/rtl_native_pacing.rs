//! Lagless cost accounting. Runtime decisions use native work and hardware
//! field/interrupt ownership only; source lag counts remain census data.
use super::*;
#[path = "rtl_native_pacing_bus.rs"]
mod bus;

#[derive(Clone, Debug, Default)]
pub(crate) struct NativePacing {
    pending: u32,
    pub(super) song_upload: bool,
    deferred_song: Option<(u8, Vec<u8>)>,
    upload_prefix_holds: u32,
    upload_return_latch: Option<bool>,
    masked_callback: bool,
    accounting: bool,
    entry_handler: u64,
    entry_worker: u64,
    entry_lag_handler: u64,
    worker_segments: Vec<(u64, u64, bool)>,
    pub(super) worker_after_irq: bool,
    irq_initialized: bool,
    irq_enabled: bool,
    irq_line: u8,
    handler_updates: u64,
    last_lag_handler: u64,
    closing_lag_handler: Option<u64>,
    closing_handler: Option<u64>,
    closing_poly_upload: bool,
    record_closing_handler: bool,
    wait_wall: [u64; 3],
    predictions: [u32; 3],
    early_sound: Option<[u8; 2]>,
    late_sound: Option<[u8; 2]>,
    finished_main: u64,
    finished_lag_worker: u64,
    main_dma: u64,
    bus_program: Option<bus::Program>,
    bus_epoch: u64,
    bus_admitted: bool,
    bus_decline: u16,
    finished_hdma: u64,
    raster_barriers: Vec<(u64, u16, u64)>,
    finished_raster_wait: u64,

}

impl ZeldaState {
    pub(super) fn native_begin_main_cost(&mut self) {
        self.native_pacing.accounting = self.lag_free;
        if self.lag_free && !self.native_pacing.irq_initialized {
            // A foreign seed is post-NMI. $1F0A holds the stack that NMI
            // suspended: a worker stack means NMI returned to main.
            let stack = self.game_state.display.nmi_thread_stack_pointer;
            self.native_pacing.worker_after_irq = self.game_state.display.nmi_thread_active
                && stack & 0xff00 == 0x1f00;
            // A stale bootstrap stack survives song uploads; it does not
            // prove $4200 still enables IRQ. Only an active post-NMI thread
            // or the ordinary handler's $0128 programming establishes that.
            self.native_pacing.irq_enabled = self.game_state.display.nmi_thread_active
                || self.game_state.display.has_irq_control_flag();
            self.native_pacing.irq_line = if self.game_state.display.nmi_thread_active {
                self.game_state.display.vertical_irq_trigger
            } else { 128 };
            self.native_pacing.irq_initialized = true;
        }
        // Presentation holds follow the native closing NMI, although their
        // source interruptions precede it. They must not replace the handler
        // through which the next main iteration actually enters.
        self.native_pacing.entry_handler = self.native_previous_closing_handler();
        self.native_pacing.entry_lag_handler = self.native_pacing.closing_lag_handler
            .unwrap_or(self.native_pacing.last_lag_handler);
        if self.native_pacing.closing_handler.is_some() {
            self.lag_free_nmi_uploaded_poly = self.native_pacing.closing_poly_upload;
        }
        // $00:8144..8149 selected the preceding NMI's thread before this
        // main call. A later $012A activation/clear cannot change that slice;
        // intervening lag NMIs still inspect the live enable byte.
        self.native_pacing.entry_worker = if self.native_pacing.worker_after_irq { 0 } else { self.native_pacing_worker_share(
            self.native_pacing.entry_handler.min(1364 * 262 - 1),
        ) };
        self.native_pacing.wait_wall = [0; 3];
        self.native_pacing.raster_barriers.clear();
        self.native_pacing.main_dma = 0;
        self.native_pacing.worker_segments.clear();
        self.native_pacing.predictions = [0; 3];
        if self.lag_free { self.native_begin_bus_lease(); }
        self.native_pacing.early_sound = None;
        self.native_pacing.late_sound = None;
    }

    pub(super) fn native_record_closing_handler(&mut self) {
        if self.native_pacing.song_upload { return; }
        if self.native_pacing.record_closing_handler {
            self.native_pacing.closing_handler = self.last_nmi_handler_master_cycles().map(u64::from);
            self.native_pacing.closing_lag_handler = Some(self.native_pacing.last_lag_handler);
            self.native_pacing.closing_poly_upload = self.lag_free_nmi_uploaded_poly;
            // $00:8230..823A programs $FF/A1 on the thread path. The
            // ordinary $00:8200..8218 path only reprograms with $0128 set;
            // zero leaves an earlier V-IRQ enabled. A negative $0128 makes
            // its IRQ clear the enable at $00:830F..8314.
            if self.game_state.display.nmi_thread_active {
                self.native_pacing.irq_enabled = true;
                self.native_pacing.irq_line = self.game_state.display.vertical_irq_trigger;
            } else if self.game_state.display.has_irq_control_flag() {
                self.native_pacing.irq_enabled = self.game_state.display.irq_control_flag & 0x80 == 0;
                self.native_pacing.irq_line = 128;
            }
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

    pub fn native_cost_lag_handler(&self) -> u64 {
        self.native_pacing.entry_lag_handler
    }

    pub(crate) fn native_note_handler_updates(&mut self, master: u64) {
        if self.lag_free { self.native_pacing.handler_updates = master; }
    }

    pub(crate) fn native_note_handler_cost(&mut self, master: u64) {
        if self.lag_free {
            self.native_pacing.last_lag_handler = master.saturating_sub(self.native_pacing.handler_updates);
        }
    }

    pub(crate) fn native_checkpoint_worker_ownership(&mut self) {
        if !self.native_pacing.accounting { return; }
        // $00:8144 samples $012A at each NMI, not at the end of main.
        // Retain the work completed under the old enable/IRQ-line state
        // before a caller changes it. Already elapsed NMIs keep their owner.
        let main = self.native_main_work();
        let worker = self.native_pacing_worker_share(self.native_pacing.entry_lag_handler);
        self.native_pacing.worker_segments.push((main, worker, self.native_pacing.worker_after_irq));
    }

    pub(crate) fn native_activate_worker_ownership(&mut self) {
        if !self.native_pacing.accounting { return; }
        let main = self.native_main_work();
        let elapsed = self.native_cost_elapsed(main)[2];
        // Both $00:82C9..82CF and $00:8324..832A swap stacks. If an
        // already-enabled V-IRQ arrives first, it switches main TO the
        // worker; NMI subsequently switches back. GO does not select this.
        let irq = pacing_irq_offset(self.native_pacing.irq_line);
        self.native_pacing.worker_after_irq = self.native_pacing.irq_enabled
            && elapsed % (1364 * 262) < irq;
    }

    pub(crate) fn native_song_upload_restores_nmi_only(&mut self) {
        if !self.lag_free { return; }
        // The callers of the song-bank upload restore $4200=$81, not A1:
        // $02:8564..8566, $02:9C06..9C08, $02:A046..A048,
        // $09:F66F..F671 (and the reset caller $00:802F..8031).
        // A later $012A activation must wait for NMI to program V-IRQ again.
        self.native_pacing.irq_enabled = false;
    }

    fn native_main_work(&self) -> u64 {
        crate::cycle_ledger::master().saturating_sub(self.lag_free_frame_master_start)
            + self.lag_free_vwf_master + self.native_pacing.main_dma
    }

    pub(super) fn native_main_cost_active(&self) -> bool {
        self.native_pacing.accounting && !crate::cycle_ledger::is_muted()
    }

    pub(super) fn native_note_main_dma(&mut self, master: u64) {
        if self.native_main_cost_active() { self.native_pacing.main_dma += master; }
    }

    /// Bus stalls are separate from the instruction-only cycle ledger.
    pub fn native_cost_main_dma(&self) -> u64 {
        self.native_pacing.main_dma
    }

    /// Completed P2 refresh and raster-wait components, for CPU diagnostics.
    pub fn native_cost_bus_waits(&self) -> (u64, u64) {
        ((self.native_pacing.finished_main + self.native_pacing.finished_hdma).div_ceil(1324) * 40,
            self.native_pacing.finished_raster_wait)
    }

    pub fn native_cost_main_hdma(&self) -> (u64, bool) {
        (self.native_pacing.finished_hdma, self.native_pacing.bus_admitted)
    }

    /// Completed-call admission reasons, frozen before the closing NMI can
    /// reprogram HDMA or replace another timing owner.
    pub fn native_cost_main_hdma_decline(&self) -> u16 {
        self.native_pacing.bus_decline
    }

    fn native_cost_elapsed(&self, main: u64) -> [u64; 3] {
        self.native_cost_elapsed_with_worker(main,
            self.native_pacing_worker_share(self.native_pacing.entry_lag_handler))
    }

    fn native_cost_elapsed_with_worker(&self, main: u64, lag_worker: u64) -> [u64; 3] {
        self.native_cost_components(main, lag_worker).0
    }

    fn native_cost_components(&self, main: u64, lag_worker: u64) -> ([u64; 3], u64, u64) {
        const FIELD: u64 = 1364 * 262;
        let handler = self.native_pacing.entry_handler.min(FIELD - 1);
        let worker = self.native_pacing.entry_worker.min(FIELD - handler - 1);
        let lag_handler = self.native_pacing.entry_lag_handler.min(FIELD - 1);
        let lag_worker = lag_worker.min(FIELD - lag_handler - 1);
        let refreshed = main + main.div_ceil(1324) * 40;
        // Main enters after one ordinary handler. Subsequent fields execute
        // the source's latched NMI path, excluding NMI_DoUpdates and joypads.
        let mut elapsed = [handler + worker; 2];
        let mut previous = [0; 2];
        for (end, share, after_irq) in self.native_pacing.worker_segments.iter().copied()
            .chain(std::iter::once((main, lag_worker, self.native_pacing.worker_after_irq))) {
            if end > main { break; }
            let work = [end, end + end.div_ceil(1324) * 40];
            for i in 0..2 {
                elapsed[i] = advance_main_cost(work[i] - previous[i], elapsed[i], lag_handler, share, after_irq);
            }
            previous = work;
        }
        debug_assert_eq!(previous, [main, refreshed]);
        let mut result = [handler + main * FIELD / (FIELD - worker), elapsed[0], elapsed[1]];
        for (wall, wait) in result.iter_mut().zip(self.native_pacing.wait_wall) { *wall += wait; }
        let mut hdma = 0;
        let mut raster_wait = self.native_pacing.wait_wall[2];
        // Complete-call HDMA ownership is admitted only after the program
        // lease has been checked. Mid-call timed producers retain their
        // existing scheduling. Raster barriers own wall time, including
        // the bus events that occur while polling $00:F3A3..F3B2.
        if !self.native_pacing.accounting && self.native_pacing.bus_admitted {
            if let Some((wall, bus, wait)) = self.native_pacing.bus_program.as_ref().and_then(|program|
                program.compose(main, handler + worker, lag_handler, lag_worker,
                    self.native_pacing.worker_after_irq, &self.native_pacing.raster_barriers)) {
                result[2] = wall;
                hdma = bus;
                raster_wait = wait;
            }
        }
        (result, hdma, raster_wait)
    }

    /// Generic raster barrier: a V-counter loop exits at or after line L.
    /// Charge elapsed waiting separately from instruction work, since CPU
    /// work excludes NMI/worker ownership and refresh while wall time does not.
    pub(super) fn native_wait_until_line(&mut self, line: u16, loop_master: u64) {
        if !self.native_pacing.accounting { return; }
        const FIELD: u64 = 1364 * 262;
        const NMI: u64 = 225 * 1364;
        let main = self.native_main_work();
        let elapsed = self.native_cost_elapsed(main);
        self.native_pacing.raster_barriers.push((main, line, loop_master));
        for (i, work) in elapsed.into_iter().enumerate() {
            let raster = (NMI + work) % FIELD;
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
        let main = self.native_main_work();
        if self.native_cost_elapsed(main)[2] < 1364 * 262 {
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

    /// Diagnostic inverse of the completed P2 model. The replay CLI supplies
    /// a source count only after main has returned; this never schedules work.
    /// Extra work is appended after the existing ownership checkpoints, with
    /// the completed call's worker share and raster wait held fixed.
    pub fn native_cost_shortfall(&self, holds: u32) -> (u64, u64) {
        const FIELD: u64 = 1364 * 262;
        let elapsed = |extra| self.native_cost_elapsed_with_worker(
            self.native_pacing.finished_main + extra,
            self.native_pacing.finished_lag_worker)[2];
        let base = elapsed(0);
        if holds <= self.native_pacing.predictions[2] { return (base, 0); }
        let target = u64::from(holds) * FIELD;
        let mut lo = 0;
        let mut hi = target.saturating_sub(base);
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            if elapsed(mid) >= target { hi = mid; } else { lo = mid + 1; }
        }
        (base, lo)
    }

    pub(super) fn native_finish_main_cost(&mut self) {
        if !self.native_pacing.accounting { return; }
        self.native_pacing.bus_admitted = self.native_bus_lease_valid();
        self.native_pacing.bus_decline = self.native_bus_lease_decline();
        self.native_pacing.accounting = false;
        self.native_pacing.record_closing_handler = true;
        let main = self.native_main_work();
        let (elapsed, hdma, raster_wait) = self.native_cost_components(main,
            self.native_pacing_worker_share(self.native_pacing.entry_lag_handler));
        self.native_pacing.finished_hdma = hdma;
        self.native_pacing.finished_raster_wait = raster_wait;
        self.native_pacing.finished_main = main;
        self.native_pacing.finished_lag_worker =
            self.native_pacing_worker_share(self.native_pacing.entry_lag_handler);
        self.native_pacing.predictions = std::array::from_fn(|i|
            (elapsed[i] / (1364 * 262)) as u32);
        // P2 includes the source refresh stalls and the generic raster barrier.
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
            if self.native_upload_prefix_pending() {
                // $02:855D is reached after the caller's ordinary work and
                // its accepted lag NMIs, before $4200-masked upload time.
                if self.native_pacing.upload_return_latch.is_none() {
                    self.native_pacing.upload_return_latch =
                        Some(self.game_state.display.nmi_update_is_latched());
                    // Main has not reached $00:805A's STZ $12 yet. Its
                    // prefix interruptions must take $00:813A's latched
                    // branch, not publish the completed atomic main early.
                    self.latch_nmi_update();
                }
                self.native_pacing.pending -= 1;
                self.native_pacing.upload_prefix_holds -= 1;
                if self.native_pacing.upload_prefix_holds == 0 { self.native_start_deferred_song(); }
                return false;
            }
            self.native_start_deferred_song();
            self.native_pacing.masked_callback = true;
            true
        } else { false }
    }

    pub(super) fn native_begin_callback(&mut self) {
        self.native_pacing.masked_callback = false;
    }

    pub(crate) fn native_defer_song_upload(&mut self, bank: u8, stream: &[u8]) -> bool {
        if !self.native_pacing.accounting || !self.native_pacing.song_upload { return false; }
        let main = self.native_main_work();
        // Capture at the command, not at main's return: any later ordinary
        // instruction work belongs after the masked transfer.
        self.native_pacing.upload_prefix_holds =
            (self.native_cost_elapsed(main)[2] / (1364 * 262)) as u32;
        self.native_pacing.deferred_song = Some((bank, stream.to_vec()));
        true
    }

    fn native_start_deferred_song(&mut self) {
        if let Some((bank, stream)) = self.native_pacing.deferred_song.take() {
            self.start_deferred_song_bank_transfer(bank, &stream);
        }
    }

    pub(super) fn native_upload_prefix_pending(&self) -> bool {
        self.native_pacing.deferred_song.is_some() && self.native_pacing.upload_prefix_holds != 0
    }

    pub(super) fn run_native_upload_prefix_frame(&mut self, input: u16) {
        self.native_pacing.pending -= 1;
        self.native_pacing.upload_prefix_holds -= 1;
        self.frame_ctr_dbg = self.frame_ctr_dbg.wrapping_add(1);
        self.capture_display_snapshot();
        // $00:80C9 accepts the latched NMI while the main caller is still
        // before the upload. It is not the main's ordinary closing handler.
        self.interrupt_nmi_with_animated_bg_operands(input, None, false, None);
        if self.native_pacing.upload_prefix_holds == 0 { self.native_start_deferred_song(); }
        self.zelda_push_apu_state();
        self.poison_owned_wram();
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
            if let Some(latched) = self.native_pacing.upload_return_latch.take() {
                if latched { self.latch_nmi_update(); }
                else { self.clear_nmi_update_latch(); }
            }
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

/// Wall time through a first field with one ordinary entry handler and any
/// further fields with lag handlers. Equality accepts the NMI. No update DMA
/// is repeated just because main takes more than one field to return.
fn elapsed_with_lag_handlers(main: u64, entry: u64, repeated: u64) -> u64 {
    advance_main_cost(main, entry, repeated, 0, false)
}

pub(super) fn pacing_irq_offset(line: u8) -> u64 {
    // Same NMI H=86 / V-only IRQ H=10 as poly_thread_phase_window.
    ((u64::from(line) + 262 - 225) * 1364 + 10 + 1364 * 262 - 86) % (1364 * 262)
}

fn advance_main_cost(main: u64, elapsed: u64, handler: u64, worker: u64, after_irq: bool) -> u64 {
    const FIELD: u64 = 1364 * 262;
    let repeated = (handler + worker).min(FIELD - 1);
    // In the reversed orientation, the IRQ starts the worker interval and
    // its terminating NMI supplies the handler. Work before IRQ remains
    // available to main; do not move this interval to the field's entry.
    let offset = if after_irq && worker != 0 { FIELD - worker } else { 0 };
    let mut next = (elapsed / FIELD) * FIELD + offset;
    if next <= elapsed { next += FIELD; }
    let end = elapsed + main;
    let nmis = if end < next { 0 } else { 1 + (end - next) / (FIELD - repeated) };
    end + nmis * repeated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_shortfall_is_the_first_clock_reaching_the_requested_hold() {
        const FIELD: u64 = 1364 * 262;
        for worker in [false, true] {
            for main in [0, 1324, 200_000, 900_000] {
                let mut game = ZeldaState::default();
                game.set_lag_free(true);
                game.game_state.display.nmi_thread_active = worker;
                game.game_state.display.vertical_irq_trigger = 144;
                game.lag_free_frame_master_start = crate::cycle_ledger::master();
                game.native_begin_main_cost();
                crate::cycle_ledger::charge(main / 2);
                game.native_checkpoint_worker_ownership();
                crate::cycle_ledger::charge(main - main / 2);
                game.native_wait_until_line(192, 178);
                game.native_finish_main_cost();
                let prediction = game.native_cost_predictions()[2];
                assert_eq!(game.native_cost_shortfall(prediction).1, 0);
                for target in prediction + 1..=prediction + 3 {
                    let (elapsed, extra) = game.native_cost_shortfall(target);
                    assert_eq!(elapsed / FIELD, u64::from(prediction));
                    let evaluate = |delta| game.native_cost_elapsed_with_worker(
                        main + delta, game.native_pacing.finished_lag_worker)[2];
                    assert!(extra > 0);
                    assert!(evaluate(extra) >= u64::from(target) * FIELD);
                    assert!(evaluate(extra - 1) < u64::from(target) * FIELD);
                }
            }
        }
    }

    #[test]
    fn upload_command_waits_for_the_preceding_instruction_holds() {
        let Ok(assets) = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../zelda3_assets.dat")) else { return; };
        let mut game = ZeldaState::new();
        game.set_assets(&assets).unwrap();
        game.set_lag_free(true);
        game.lag_free_intro_schedule = None;
        game.lag_free_lag_phases_run = true;
        game.lag_free_frame_master_start = crate::cycle_ledger::master();
        game.native_begin_main_cost();
        crate::cycle_ledger::charge(800_000);
        game.native_begin_song_upload();
        assert!(game.native_defer_song_upload(0, &[0, 0]));
        game.set_pending_nmi_subroutine(7);
        game.clear_nmi_update_latch();
        let recorder = game.state_recorder.total_frames;
        let frame = game.game_state.frame;
        game.native_finish_main_cost();
        let holds = game.native_cost_predictions()[2];
        assert!(holds >= 2);
        assert!(!game.native_song_bank_transfer_pending());
        assert!(!game.native_mask_song_upload_nmi());
        assert!(game.game_state.display.nmi_update_is_latched());
        assert_eq!(game.native_held_frames_pending(), holds - 1);
        assert!(!game.native_last_callback_masked_nmi());
        game.native_record_closing_handler();
        assert!(game.native_pacing.closing_handler.is_none());
        for remaining in (0..holds - 1).rev() {
            game.run_native_upload_prefix_frame(0);
            assert_eq!(game.native_held_frames_pending(), remaining);
            assert_eq!(game.native_song_bank_transfer_pending(), remaining == 0);
            assert!(!game.native_last_callback_masked_nmi());
            assert!(game.native_pacing.closing_handler.is_none());
            assert_eq!(game.game_state.display.pending_nmi_subroutine, 7,
                "prefix NMI must not consume main's pending graphics updates");
        }
        assert_eq!(game.native_cost_predictions()[2], holds);
        assert_eq!(game.state_recorder.total_frames, recorder);
        assert_eq!(game.game_state.frame, frame);
        assert!(game.native_mask_song_upload_nmi());
        assert!(game.native_last_callback_masked_nmi());
        // Retire the fixture's receiver separately from the ownership test.
        // Its closing NMI must recover the saved main-return update latch.
        game.clear_spc_driver_clock();
        game.run_native_song_upload_frame(0);
        assert_eq!(game.game_state.display.pending_nmi_subroutine, 0);
        assert!(game.native_pacing.closing_handler.is_some());
        assert!(game.native_pacing.upload_return_latch.is_none());
    }

    #[test]
    fn song_restore_and_inactive_seed_do_not_inherit_a_stale_irq_enable() {
        let mut game = ZeldaState::default();
        game.set_lag_free(true);
        game.game_state.display.nmi_thread_stack_pointer = 0x1f31;
        game.game_state.display.vertical_irq_trigger = 48;
        game.lag_free_frame_master_start = crate::cycle_ledger::master();
        game.native_begin_main_cost();
        assert!(!game.native_pacing.irq_enabled);
        game.native_pacing.irq_enabled = true;
        game.native_song_upload_restores_nmi_only();
        assert!(!game.native_pacing.irq_enabled);
        game.activate_nmi_thread();
        assert!(!game.native_pacing.worker_after_irq);
        game.native_pacing.record_closing_handler = true;
        game.native_record_closing_handler();
        assert!(game.native_pacing.irq_enabled);
        assert_eq!(game.native_pacing.irq_line, 48);
        game.native_pacing = Default::default();
        game.game_state.display.nmi_thread_active = false;
        game.game_state.display.irq_control_flag = 1;
        game.native_begin_main_cost();
        // $00:8208 programs line 128 on the ordinary path, independently
        // of the stale polygon stack and $FF mirror in this foreign seed.
        assert!(game.native_pacing.irq_enabled);
        assert_eq!(game.native_pacing.irq_line, 128);
    }

    #[test]
    fn already_programmed_irq_can_enter_worker_before_first_threaded_nmi() {
        let mut game = ZeldaState::default();
        game.set_lag_free(true);
        game.game_state.display.vertical_irq_trigger = 144;
        game.lag_free_frame_master_start = crate::cycle_ledger::master();
        game.native_begin_main_cost();
        game.native_pacing.irq_enabled = true;
        game.native_pacing.irq_line = 144;
        crate::cycle_ledger::charge(88_000);
        game.activate_nmi_thread();
        assert!(game.native_pacing.worker_after_irq);
        // Main still owns the early part of this field, up to V-IRQ.
        assert_eq!(game.native_cost_elapsed(206_000)[2], 206_000 + 206_000u64.div_ceil(1324) * 40);
        assert!(game.native_cost_elapsed(300_000)[2] >= 1364 * 262);
        let irq = pacing_irq_offset(144);
        // The IRQ handler runs within IRQ-to-NMI wall time. One clock of
        // main work past IRQ resumes one clock after the terminating NMI's
        // handler, without charging the IRQ handler a second time.
        assert_eq!(advance_main_cost(irq + 1, 0, 3284, 1364 * 262 - irq, true),
            1364 * 262 + 3284 + 1);
        // A foreign post-NMI worker stack records the same orientation.
        game.native_pacing = Default::default();
        game.game_state.display.nmi_thread_stack_pointer = 0x1f27;
        game.native_begin_main_cost();
        assert!(game.native_pacing.worker_after_irq);
        assert_eq!(game.native_pacing.entry_worker, 0);
    }

    #[test]
    fn worker_activation_does_not_own_nmis_before_its_store() {
        const FIELD: u64 = 1364 * 262;
        let mut game = ZeldaState::default();
        game.set_lag_free(true);
        game.game_state.display.vertical_irq_trigger = 144;
        game.lag_free_frame_master_start = crate::cycle_ledger::master();
        game.native_begin_main_cost();
        crate::cycle_ledger::charge(2 * FIELD + 1000);
        let before = game.native_cost_elapsed(2 * FIELD + 1000);
        game.activate_nmi_thread();
        assert_eq!(game.native_cost_elapsed(2 * FIELD + 1000), before);
        crate::cycle_ledger::charge(FIELD);
        assert!(game.native_cost_elapsed(3 * FIELD + 1000)[2] > before[2] + FIELD);
        let before_clear = game.native_cost_elapsed(3 * FIELD + 1000);
        game.deactivate_nmi_thread();
        assert_eq!(game.native_cost_elapsed(3 * FIELD + 1000), before_clear);
    }

    #[test]
    fn entry_worker_slice_survives_enable_and_disable_inside_main() {
        let mut game = ZeldaState::default();
        game.set_lag_free(true);
        game.game_state.display.vertical_irq_trigger = 96;
        game.game_state.display.nmi_thread_active = false;
        game.native_begin_main_cost();
        assert_eq!(game.native_pacing.entry_worker, 0);
        game.game_state.display.nmi_thread_active = true;
        // No worker ran before this call. A call shorter than one field
        // cannot retroactively give its preceding NMI a worker phase.
        assert_eq!(game.native_cost_elapsed(200_000)[1], 200_000);
        // Once main crosses NMI, the newly enabled worker does own a slice.
        assert!(game.native_cost_elapsed(400_000)[1] > 400_000);
        game.native_begin_main_cost();
        let entered = game.native_pacing.entry_worker;
        assert!(entered > 0);
        game.game_state.display.nmi_thread_active = false;
        // The preceding slice has already happened; clearing the byte only
        // removes worker ownership from subsequent lag NMIs.
        assert_eq!(game.native_cost_elapsed(200_000)[1], entered + 200_000);
    }

    #[test]
    fn worker_cpu_ownership_follows_the_enable_byte_across_module_routing() {
        let mut game = ZeldaState::default();
        game.set_lag_free(true);
        game.game_state.display.vertical_irq_trigger = 96;
        game.game_state.display.nmi_thread_active = true;
        game.game_state.frame.main_module = 7;
        let enabled = game.native_pacing_worker_share(3000);
        assert!(enabled > 0);
        // No module duration or category participates in the hardware rule.
        for module in 0..=0x1b {
            game.game_state.frame.main_module = module;
            assert_eq!(game.native_pacing_worker_share(3000), enabled);
        }
        game.game_state.display.nmi_thread_active = false;
        assert_eq!(game.native_pacing_worker_share(3000), 0);
        game.game_state.display.nmi_thread_active = true;
        game.set_lag_free(false);
        assert_eq!(game.native_pacing_worker_share(3000), 0);
    }

    #[test]
    fn intervening_nmis_use_lag_cost_and_accept_equality() {
        const FIELD: u64 = 1364 * 262;
        let entry = 80_000;
        let lag = 3_000;
        assert_eq!(elapsed_with_lag_handlers(FIELD - entry - 1, entry, lag), FIELD - 1);
        assert_eq!(elapsed_with_lag_handlers(FIELD - entry, entry, lag), FIELD + lag);
        assert_eq!(elapsed_with_lag_handlers(2 * FIELD - entry - lag - 1, entry, lag), 2 * FIELD - 1);
        assert_eq!(elapsed_with_lag_handlers(2 * FIELD - entry - lag, entry, lag), 2 * FIELD + lag);
        let long = elapsed_with_lag_handlers(20 * FIELD - entry - 19 * lag, entry, lag);
        assert_eq!(long, 20 * FIELD + lag);
    }

    #[test]
    fn ordinary_update_dma_is_removed_from_the_retained_lag_handler() {
        let mut game = ZeldaState::default();
        game.set_lag_free(true);
        game.lag_free_intro_schedule = None;
        game.lag_free_lag_phases_run = true;
        game.clear_nmi_update_latch();
        game.interrupt_nmi(0, None, false);
        let ordinary = u64::from(game.last_nmi_handler_master_cycles().unwrap());
        let stripped = game.native_pacing.last_lag_handler;
        assert!(stripped < ordinary);
        game.native_pacing.record_closing_handler = true;
        game.native_record_closing_handler();
        game.interrupt_nmi(0, None, false);
        assert_eq!(stripped, u64::from(game.last_nmi_handler_master_cycles().unwrap()));
        game.native_begin_main_cost();
        assert_eq!(game.native_cost_lag_handler(), stripped);
        assert_eq!(game.native_cost_entry_handler(), ordinary);
    }

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
