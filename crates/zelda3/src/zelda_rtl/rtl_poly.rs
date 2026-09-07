//! `ZeldaState` methods split out of `zelda_rtl.rs` by topic (poly).
//! Mechanical move: bodies are unchanged; private methods became
//! `pub(super)` so the parent module and its siblings can call them.

use super::*;

impl ZeldaState {
    pub fn intro_poly_upload_delay(&self) -> u8 {
        self.intro_poly_upload_delay
    }

    pub fn debug_nmi_poly_upload_deferred(&self) -> u8 {
        self.nmi_poly_upload_deferred
    }

    pub fn debug_nmi_poly_upload_started(&self) -> bool {
        self.nmi_poly_upload_started
    }

    pub fn debug_snes9x_poly_scheduler_counter(&self) -> u8 {
        self.snes9x_poly_scheduler_counter
    }

    pub fn debug_last_poly_work(&self) -> PolyWorkMetrics {
        self.last_poly_work
    }

    pub(crate) fn request_polyhedral_nmi_update(&mut self) {
        self.display_core_mut().request_polyhedral_nmi_update();
    }

    pub(crate) fn clear_pending_polyhedral_update(&mut self) {
        self.display_core_mut().clear_pending_polyhedral_update();
    }

    pub(crate) fn polyhedral_tile_buffer(&self) -> &[u8] {
        self.game_state.display.polyhedral_tile_buffer(&self.ram)
    }

    pub(super) fn selected_intro_poly_display_buffer(&self) -> Vec<u16> {
        if env::var_os("ZELDA3_INTRO_POLY_PRESENT_OBJ_LATCH").is_some() {
            if let Some(latched_vram) = self.ppu.obj_vram_latch.as_deref() {
                if latched_vram.len() >= 0x5c00 {
                    return latched_vram[0x5800..0x5c00].to_vec();
                }
            }
        }
        let diagnostic_lag = env::var("ZELDA3_INTRO_POLY_PRESENT_HISTORY_LAG")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(0);
        if diagnostic_lag != 0 {
            if let Some((_, vram, _)) = self
                .intro_poly_vram_history
                .len()
                .checked_sub(1 + diagnostic_lag)
                .and_then(|index| self.intro_poly_vram_history.get(index))
            {
                return vram.clone();
            }
        }
        self.ppu.vram[0x5800..0x5c00].to_vec()
    }

    /// The crystal maiden's IRQ poly thread: active from her Sprite_AB
    /// activation through her dialogue (Module0E keeps calling Sprite_Main,
    /// which re-arms the thread) until the cutscene deactivates it.
    pub(super) fn dungeon_poly_thread_is_active(&self) -> bool {
        self.game_state.display.nmi_thread_active
            && matches!(self.game_state.frame.main_module, 0x07 | 0x0e)
            && !self.triforce_room_poly_thread_is_active()
    }

    /// `Module19_TriforceRoom`'s V-IRQ thread (and the same thread while
    /// its Triforce message runs under module $0E with $19 saved as the
    /// return module).
    pub(crate) fn triforce_room_poly_thread_is_active(&self) -> bool {
        self.game_state.display.nmi_thread_active
            && (matches!(self.game_state.frame.main_module, 0x19 | 0x1a)
                || (self.game_state.frame.main_module == 0x0e
                    && self.game_state.frame.saved_module_for_menu == 0x19))
    }

    /// Whether a preemptive (V-IRQ switched) polyhedral thread is live under
    /// original timing: the dungeon crystal maiden or the Triforce room.
    pub(super) fn preemptive_poly_thread_is_active(&self) -> bool {
        self.rom_startup_timing
            && (self.dungeon_poly_thread_is_active() || self.triforce_room_poly_thread_is_active())
    }

    /// Called at every NMI entry: measure this NMI handler's duration to its
    /// thread swap on the RAM the handler will see, so the next host's thread
    /// slot starts at the right raster position.
    pub(super) fn stash_preemptive_poly_thread_nmi_swap(&mut self) {
        if !self.preemptive_poly_thread_is_active() {
            self.poly_next_host_swap_master = None;
            return;
        }
        // The shadow's slot pairing consumes this stash one host later, so
        // the swap-duration bookkeeping keeps attributing the upload to the
        // NMI of the completing host (the calibration of the dialogue-phase
        // cadence, hosts 414033-416096, rests on that pairing); only the
        // uploaded VRAM content is deferred to the next host's NMI. A deferred
        // upload executing at that next NMI therefore does not count again.
        let upload = self.game_state.display.has_pending_polyhedral_update()
            && self
                .poly_completed_upload
                .as_ref().is_none_or(|(host, _)| *host >= self.frame_ctr_dbg);
        // The NMI this trailing handler models is normally accepted at the
        // end of the current host's run (the last `NmiAccepted` of its
        // receipts). When the run boundary falls just before the acceptance,
        // the receipt moves to the head of the next host's vector and the
        // slot consumer reads it there.
        let wire_latch = self
            .original_timing_semantic_receipts
            .as_ref()
            .and_then(|receipts| {
                receipts
                    .semantic()
                    .iter()
                    .rev()
                    .find_map(|receipt| match receipt {
                        OriginalTimingSemanticReceipt::NmiAccepted(gate) => {
                            Some(matches!(gate, NmiUpdateGate::LatchHeld))
                        }
                        _ => None,
                    })
            });
        let latch = wire_latch;
        self.poly_next_host_nmi_state = Some((latch, upload));
        self.poly_next_host_swap_master = None;
        if crate::debug_env::var_os("ZELDA3_DEBUG_POLY").is_some() {
            eprintln!(
                "[POLY-STASH] host={} latch={:?} upload={}",
                self.frame_ctr_dbg, latch, upload,
            );
        }
    }

    /// Start the shadow render of the frame the ROM thread begins this host.
    /// Must run before the native rasterizer mutates the poly runtime state.
    pub(super) fn begin_poly_shadow_frame(&mut self) {
        const THREAD_WORK_RANGE: std::ops::Range<usize> = 0x1f10..0x2000;
        const THREAD_BITMAP_RANGE: std::ops::Range<usize> = 0xe800..0xf000;
        let timing_dma = self.dma_with_native_hdma_enable();
        let mut ram_image = self.ram.to_vec();
        if let Some(saved) = self.triforce_poly_shadow_ram.as_ref() {
            ram_image[THREAD_WORK_RANGE].copy_from_slice(&saved[THREAD_WORK_RANGE]);
            ram_image[THREAD_BITMAP_RANGE].copy_from_slice(&saved[THREAD_BITMAP_RANGE]);
        }
        self.poly_shadow_run = RomCpuTimingRun::new(
            &self.rom,
            &ram_image,
            &self.sram,
            &self.ppu,
            &timing_dma,
            self.zelda_audio_apu_output_ports(),
            TRIFORCE_ROOM_POLY_RENDER_CPU_CHECKPOINT,
        )
        .ok();
        self.poly_shadow_master = 0;
        self.poly_shadow_hosts = 0;
    }

    pub(super) fn zelda_run_poly_loop(&mut self) {
        /// Crystal scanlines the maiden's IRQ poly thread rasterizes per host.
        const DUNGEON_POLY_THREAD_SCANLINES_PER_HOST: u32 = 32;
        /// Per-frame setup of the crystal thread, in scanline equivalents,
        /// while Module07 runs the cutscene.
        const DUNGEON_POLY_THREAD_SETUP_SCANLINES: u32 = 67;
        /// Through the maiden's Module0E dialogue the thread's per-host share
        /// in estimated 65816 cycles and the affine per-frame offset (fit on
        /// the oracle's step cadence at route hosts 414033-414115).
        const DUNGEON_POLY_THREAD_DIALOGUE_BUDGET_CYCLES: i64 = 7_572;
        const DUNGEON_POLY_THREAD_DIALOGUE_SETUP_CYCLES: i64 = -20_641;
        // The C port interleaves one poly step per main-loop step
        // (`intro_did_run_step`). The ROM's V-IRQ threads (dungeon crystal
        // maiden, Triforce room) start their next frame at the next host's
        // swap whether or not the main iteration is held by the wire (oracle
        // 415132: the thread restarts at V=257 while the Module0E iteration
        // is still suspended); they wait only for the NMI upload of the
        // finished frame.
        // `intro_did_run_step` IS the ROM thread's go byte $1F00: the thread
        // loop ($09:F81D) spins until the main thread sets it (the crystal
        // maiden's `Sprite_AB_CrystalMaiden` at V≈65 of its main slot, once
        // per completed frame) and the NMI has cleared the upload byte $1F0C.
        // A bitmap completed in an earlier host's slot is uploaded by THIS
        // host's NMI, which precedes this host's slot chronologically: the
        // thread may start its next frame in this slot (oracle run 413573:
        // upload at its NMI, `$09:F825` restart at V=261 of the same run).
        let pending_upload_precedes_this_slot = self
            .poly_completed_upload
            .as_ref()
            .is_some_and(|(host, _)| *host < self.frame_ctr_dbg);
        let can_run_poly = self.game_state.ending.attract_scene.intro_did_run_step() != 0
            && (!self.game_state.display.has_pending_polyhedral_update()
                || pending_upload_precedes_this_slot);
        let live_timing = matches!(self.original_timing_owner, OriginalTimingOwnerState::Live);
        let source_started_preemptive_render =
            self.take_original_timing_preemptive_polyhedral_render_started();
        if source_started_preemptive_render {
            assert!(
                self.preemptive_poly_thread_is_active(),
                "source started a preemptive poly render outside the native preemptive-thread phase",
            );
            assert!(
                can_run_poly,
                "source started a preemptive poly render while the native go/upload gates rejected it",
            );
        }
        if self.preemptive_poly_thread_is_active() {
            // A run's leading acceptance (before any handler completion)
            // belongs to the handler that begins this run's slot; a trailing
            // acceptance (no completion after it) belongs to the next run's.
            let gates = self
                .original_timing_semantic_receipts
                .as_ref()
                .map(|receipts| {
                    let held = |receipt: &OriginalTimingSemanticReceipt| match receipt {
                        OriginalTimingSemanticReceipt::NmiAccepted(gate) => {
                            Some(matches!(gate, NmiUpdateGate::LatchHeld))
                        }
                        _ => None,
                    };
                    let completed = |receipt: &OriginalTimingSemanticReceipt| {
                        matches!(receipt, OriginalTimingSemanticReceipt::NmiHandlerCompleted)
                    };
                    let semantic = receipts.semantic();
                    let leading = semantic
                        .iter()
                        .take_while(|receipt| !completed(receipt))
                        .find_map(held);
                    let trailing = semantic
                        .iter()
                        .rev()
                        .take_while(|receipt| !completed(receipt))
                        .find_map(held);
                    (leading, trailing)
                });
            self.poly_receipt_gates_prev = self.poly_receipt_gates_cur.take();
            self.poly_receipt_gates_cur = gates;
        } else {
            self.poly_receipt_gates_prev = None;
            self.poly_receipt_gates_cur = None;
        }
        if self.dungeon_poly_thread_is_active()
            && crate::debug_env::var_os("ZELDA3_DEBUG_POLY").is_some()
        {
            eprintln!(
                "[POLY] host={} entry: did_run_step={} pending_update={} in_flight={} hold_frames={} startup_hold={:?} maiden_ai={}",
                self.frame_ctr_dbg,
                self.game_state.ending.attract_scene.intro_did_run_step(),
                self.game_state.display.has_pending_polyhedral_update(),
                self.poly_job_in_flight,
                self.poly_job_hold_frames,
                self.poly_dungeon_thread_startup_hold,
                self.sprite_slot_view(15).ai_state(),
            );
        }
        if can_run_poly {
            let frame = self.game_state.frame;
            // The crystal maiden's cutscene (Sprite_AB, dungeon module) runs
            // the same IRQ-driven poly worker as the title triforce; its
            // frame cost gates the cutscene sprite the same way (route host
            // 413549: the ROM's first poly frame held the maiden four hosts).
            let dungeon_poly_thread = self.dungeon_poly_thread_is_active();
            // The Triforce room's V-IRQ thread renders the zooming Triforce
            // while Module19 runs the text, sprites, and LinkOam each frame:
            // the oracle's zoom steps land every host whose render costs
            // less than one host's thread share and every second host
            // otherwise (route hosts 1557810-1558053: single-host steps at
            // config1 $cf/$cd/$cb, $c1/$bf/$bd, $a1/$9f/$9d = the frames
            // under ~25.2k estimated cycles; 44.9k-cycle frames still take
            // two hosts).
            let triforce_room_poly_thread = self.triforce_room_poly_thread_is_active();
            const TRIFORCE_ROOM_POLY_THREAD_HOST_CYCLES: u32 = 25_200;
            if !self.game_state.display.nmi_thread_active {
                self.poly_dungeon_thread_startup_hold = None;
                self.poly_dungeon_frames_rendered = 0;
                self.triforce_poly_shadow_ram = None;
                self.poly_shadow_run = None;
                self.poly_shadow_hosts = 0;
                self.poly_shadow_master = 0;
                self.poly_completed_upload = None;
            }
            let use_timed_worker = self.rom_startup_timing
                && (rom_intro_poly_thread_is_active(frame.main_module, frame.submodule)
                    || dungeon_poly_thread
                    || triforce_room_poly_thread);
            let incremental_shadow = (dungeon_poly_thread || triforce_room_poly_thread)
                && std::env::var("ZELDA3_POLY_SHADOW_INCREMENTAL").map_or(true, |v| v != "0");
            if incremental_shadow {
                if live_timing && self.poly_job_in_flight {
                    assert!(
                        !source_started_preemptive_render,
                        "source started a preemptive poly render while the native ROM-CPU shadow was still in flight",
                    );
                }
                if live_timing && !self.poly_job_in_flight && !source_started_preemptive_render {
                    return;
                }
                if live_timing && !self.poly_job_in_flight {
                    self.poly_dungeon_thread_startup_hold = None;
                } else if dungeon_poly_thread && !self.poly_job_in_flight {
                    // The ROM's `CrystalCutscene_InitializePolyhedral` sets the go byte
                    // four source hosts after `Polyhedral_InitializeThread`
                    // (oracle 500942 → go-byte write late in 500946). The
                    // render begins only when the next NMI swaps to the thread
                    // (source 500947 `$09:F825`), which is debug host +6.
                    let hold = self.poly_dungeon_thread_startup_hold.get_or_insert(6);
                    let elapsed = self
                        .frame_ctr_dbg
                        .saturating_sub(self.poly_dungeon_activation_host);
                    if u32::from(*hold) > elapsed {
                        return;
                    }
                    *hold = 0;
                }
                if !self.poly_job_in_flight {
                    // The ROM thread reads the frame's config at its start;
                    // capture that state before the native rasterizer advances it.
                    self.begin_poly_shadow_frame();
                    self.poly_run_frame();
                    if dungeon_poly_thread {
                        self.poly_dungeon_frames_rendered =
                            self.poly_dungeon_frames_rendered.saturating_add(1);
                    }
                    self.poly_job_in_flight = true;
                    self.poly_job_hold_frames = 0;
                }
                if !self.advance_poly_shadow_host() {
                    return;
                }
                if crate::debug_env::var_os("ZELDA3_DEBUG_POLY").is_some() {
                    eprintln!(
                        "[POLY-FRAME] host={} module={:02x} config1={} scanlines={} hosts={} master={}",
                        self.frame_ctr_dbg,
                        frame.main_module,
                        self.game_state.poly.runtime.config1(),
                        self.last_poly_work.scanlines,
                        self.poly_shadow_hosts,
                        self.poly_shadow_master,
                    );
                }
                self.poly_job_in_flight = false;
            } else if use_timed_worker {
                if dungeon_poly_thread && crate::debug_env::var_os("ZELDA3_DEBUG_POLY").is_some() {
                    eprintln!(
                        "[POLY] host={} dungeon thread: startup_hold={:?} in_flight={} hold_frames={} rendered={} pending_update={}",
                        self.frame_ctr_dbg,
                        self.poly_dungeon_thread_startup_hold,
                        self.poly_job_in_flight,
                        self.poly_job_hold_frames,
                        self.poly_dungeon_frames_rendered,
                        self.game_state.display.has_pending_polyhedral_update(),
                    );
                }
                if dungeon_poly_thread && !self.poly_job_in_flight {
                    // The maiden activates the thread inside Sprite_Main (route
                    // host 413550, counted by `activate_nmi_thread`); the IRQ
                    // thread's first slice, which writes the crystal config,
                    // runs four hosts later (413554). The activation host's
                    // own lane may bypass this loop, so the hold counts hosts
                    // from the activation itself.
                    let hold = self.poly_dungeon_thread_startup_hold.get_or_insert(4);
                    let elapsed = self
                        .frame_ctr_dbg
                        .saturating_sub(self.poly_dungeon_activation_host);
                    if u32::from(*hold) > elapsed {
                        return;
                    }
                    *hold = 0;
                }
                if !self.poly_job_in_flight {
                    // Measure the ROM thread's exact render cost before the
                    // native rasterizer mutates the frame state.
                    let dungeon_shadow_render = dungeon_poly_thread
                        && std::env::var("ZELDA3_POLY_DUNGEON_SHADOW").map_or(true, |v| v != "0");
                    let triforce_render_hosts =
                        if triforce_room_poly_thread || dungeon_shadow_render {
                            self.rom_triforce_poly_render_hosts()
                        } else {
                            None
                        };
                    self.poly_run_frame();
                    self.poly_job_hold_frames = if dungeon_poly_thread {
                        // The thread rasterizes 32 crystal scanlines per host
                        // after a fixed per-frame setup measured in scanline
                        // equivalents: 67 while Module07 runs the cutscene
                        // (every one of the 86 oracle frames at route hosts
                        // 413554-414023: 4 hosts at 38-60 scanlines, 5 at
                        // 64-92, 6 at 94-106, 7 at 130-138) and 56 through
                        // the maiden's Module0E dialogue, whose lighter main
                        // iteration leaves the thread about a third of a host
                        // more per frame (414033-416096: 5 hosts at ~100
                        // scanlines, 6 at ~134, 7 at 138).
                        self.poly_dungeon_frames_rendered =
                            self.poly_dungeon_frames_rendered.saturating_add(1);
                        let scanlines = self.last_poly_work.scanlines;
                        let slices = if let Some((hosts, master)) = triforce_render_hosts {
                            if crate::debug_env::var_os("ZELDA3_DEBUG_POLY").is_some() {
                                eprintln!(
                                    "[POLY-DUNG] host={} module={:02x} config1={} shadow_master={} scanlines={} hosts={}",
                                    self.frame_ctr_dbg,
                                    frame.main_module,
                                    self.game_state.poly.runtime.config1(),
                                    master,
                                    scanlines,
                                    hosts,
                                );
                            }
                            hosts.clamp(1, 16)
                        } else if frame.main_module == 0x0e {
                            // Through the dialogue the oracle's frame lengths
                            // separate by the estimated cycle cost rather than
                            // by scanlines (137-scanline frames complete in 6
                            // hosts at 63.3k cycles, 138-scanline ones in 7 at
                            // 66.7k). `ZELDA3_POLY_DIALOGUE_BUDGET=B,S`
                            // overrides the calibration.
                            static DIALOGUE_BUDGET: std::sync::OnceLock<(i64, i64)> =
                                std::sync::OnceLock::new();
                            let (budget, setup) = *DIALOGUE_BUDGET.get_or_init(|| {
                                std::env::var("ZELDA3_POLY_DIALOGUE_BUDGET")
                                    .ok()
                                    .and_then(|value| {
                                        let (b, s) = value.split_once(',')?;
                                        Some((b.trim().parse().ok()?, s.trim().parse().ok()?))
                                    })
                                    .unwrap_or((
                                        DUNGEON_POLY_THREAD_DIALOGUE_BUDGET_CYCLES,
                                        DUNGEON_POLY_THREAD_DIALOGUE_SETUP_CYCLES,
                                    ))
                            });
                            let cost = (i64::from(self.last_poly_work.estimated_65816_cycles())
                                + setup)
                                .max(1);
                            ((cost + budget - 1) / budget).clamp(1, 16) as u8
                        } else {
                            (scanlines + DUNGEON_POLY_THREAD_SETUP_SCANLINES)
                                .div_ceil(DUNGEON_POLY_THREAD_SCANLINES_PER_HOST)
                                .clamp(1, 16) as u8
                        };
                        self.poly_dungeon_current_frame_slices = slices;
                        slices - 1
                    } else if triforce_room_poly_thread {
                        // The thread owns the scanlines from the NMI's return
                        // to the V-IRQ line each host; the exact render cost
                        // decides how many of those slots the frame needs.
                        // Calibrated on route hosts 1557810-1558053: the
                        // oracle's single-host renders cost at most 188,016
                        // master cycles in the shadow and its two-host ones
                        // at least 190,392 (about 139 scanlines of thread time
                        // between the NMI's return and the V-IRQ at line 144).
                        let hosts = match triforce_render_hosts {
                            Some((hosts, _)) => hosts.clamp(1, 16),
                            None => self
                                .last_poly_work
                                .estimated_65816_cycles()
                                .div_ceil(TRIFORCE_ROOM_POLY_THREAD_HOST_CYCLES)
                                .clamp(1, 16) as u8,
                        };
                        if crate::debug_env::var_os("ZELDA3_DEBUG_POLY").is_some() {
                            eprintln!(
                                "[POLY-TRI] host={} config1={} shadow_master={:?} estimate={} hosts={}",
                                self.frame_ctr_dbg,
                                self.game_state.poly.runtime.config1(),
                                triforce_render_hosts.map(|(_, master)| master),
                                self.last_poly_work.estimated_65816_cycles(),
                                hosts,
                            );
                        }
                        hosts - 1
                    } else {
                        self.last_poly_work.worker_frames() - 1
                    };
                    self.poly_job_in_flight = true;
                    if crate::debug_env::var_os("ZELDA3_DEBUG_POLY").is_some() {
                        eprintln!(
                            "[POLY] host={} module={:02x}/{:02x} cycles={} worker_frames={} config1={:#x} metrics={:?}",
                            self.frame_ctr_dbg,
                            frame.main_module,
                            frame.submodule,
                            self.last_poly_work.estimated_65816_cycles(),
                            self.last_poly_work.worker_frames(),
                            self.game_state.poly.runtime.config1(),
                            self.last_poly_work,
                        );
                    }
                }
                if self.poly_job_hold_frames != 0 {
                    self.poly_job_hold_frames -= 1;
                    return;
                }
                self.poly_job_in_flight = false;
            } else {
                self.poly_run_frame();
                if crate::debug_env::var_os("ZELDA3_DEBUG_POLY").is_some() {
                    eprintln!(
                        "[POLY] host={} untimed frame: module={:02x}/{:02x} config1={} scanlines={} cycles={} worker_frames={} work={:?}",
                        self.frame_ctr_dbg,
                        frame.main_module,
                        frame.submodule,
                        self.game_state.poly.runtime.config1(),
                        self.last_poly_work.scanlines,
                        self.last_poly_work.estimated_65816_cycles(),
                        self.last_poly_work.worker_frames(),
                        self.last_poly_work,
                    );
                }
            }
            self.attract_scene_mut().clear_intro_did_run_step();
            self.request_polyhedral_nmi_update();
            if incremental_shadow {
                self.poly_completed_upload = Some((
                    self.frame_ctr_dbg,
                    self.ram[POLYHEDRAL_BUFFER..POLYHEDRAL_BUFFER + 0x800].to_vec(),
                ));
            }
        }
    }

    pub(super) fn clear_poly_thread_work_area(&mut self) {
        SystemWorkArea::clear_poly_thread_work_area(&mut self.ram);
        // The clear zeros the poly thread work area (0x1f00-0x1fff) directly in RAM, but the
        // PolyState native model (num_vertices at 0x1f3f, projected vertices, face coords, raster
        // edges) still holds the previous polyhedron's values. Without resyncing, a stale poly
        // field re-stamps RAM (e.g. num_vertices reverts the just-cleared 0x1f3f), leaving the
        // poly scratch a frame out of phase with the old clone (f465536).
        self.game_state.poly = crate::game_state::PolyState::load_from_ram(&self.ram);
    }

    pub(super) fn write_poly_thread_init_bytes(&mut self) {
        SystemWorkArea::write_poly_thread_bootstrap_bytes(&mut self.ram);
    }
}
