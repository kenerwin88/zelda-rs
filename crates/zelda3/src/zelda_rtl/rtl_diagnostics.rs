//! `ZeldaState` methods split out of `zelda_rtl.rs` by topic (diagnostics).
//! Mechanical move: bodies are unchanged; private methods became
//! `pub(super)` so the parent module and its siblings can call them.

use super::*;

impl ZeldaState {
    /// Dump the composed display, after DMA/scanout receipts, independently
    /// of live CPU RAM. Engine host N corresponds to comparison frame N-1.
    pub(super) fn debug_dump_presented_state(&self, plan: &DisplayPublicationPlan) {
        if !nmi::debug_frame_selection_env_matches(
            "ZELDA3_DEBUG_PRESENTED_FRAMES",
            self.frame_ctr_dbg,
        ) {
            return;
        }
        let root = std::path::PathBuf::from(
            crate::debug_env::var("ZELDA3_DEBUG_PRESENTED_DIR")
                .expect("selected presented frames require ZELDA3_DEBUG_PRESENTED_DIR"),
        );
        std::fs::create_dir_all(&root).expect("create presented-state directory");
        let write_words = |name: &str, words: &[u16]| {
            let bytes: Vec<u8> = words.iter().flat_map(|word| word.to_le_bytes()).collect();
            std::fs::write(root.join(format!("{}-{name}.bin", self.frame_ctr_dbg)), bytes)
                .expect("write presented-state words");
        };
        write_words("vram", &self.ppu.vram);
        write_words("obj-vram", self.ppu.obj_vram_latch.as_deref().unwrap_or(&self.ppu.vram));
        write_words("cgram", &self.ppu.cgram);
        write_words("oam", &self.ppu.oam);
        std::fs::write(root.join(format!("{}-ram.bin", self.frame_ctr_dbg)), &self.ram)
            .expect("write presented-state RAM");
        let scroll: Vec<_> = self.ppu.bg_layer.iter().map(|bg| (bg.h_scroll, bg.v_scroll)).collect();
        let registers = format!(
            "host={} brightness={} blank={}/{}/{:?} crop={} scroll={scroll:?} bg_override={:?}\nplan={plan:?}\n",
            self.frame_ctr_dbg, self.ppu.brightness, self.ppu.forced_blank,
            self.ppu.forced_blank_scanlines, self.ppu.forced_blank_from_scanline,
            self.ppu.scanout_top_crop, self.active_presented_bg_scroll,
        );
        std::fs::write(root.join(format!("{}-registers.txt", self.frame_ctr_dbg)), registers)
            .expect("write presented-state registers");
    }

    /// Native↔RAM coherence guard. With `ZELDA3_ASSERT_NATIVE_COHERENT` set, report (or
    /// `=panic` to abort on) any native sub-state that has drifted out of sync with RAM
    /// at this labeled step — the signature of a stale-native-field or RAM-written-
    /// without-native-sync bug. Optionally scope to one frame with
    /// `ZELDA3_ASSERT_COHERENT_FRAME=<n>` to keep the (heavy) check cheap.
    /// True at the labeled checkpoint of the target frame, whether the run started from
    /// frame 0 (`frame_ctr_dbg`) or resumed from a `--load-state` checkpoint
    /// (`replay_frame_counter`, which IS restored while `frame_ctr_dbg` counts from load).
    /// Use for every trace-frame gate so the diagnostics work with checkpoint resume.
    pub(super) fn trace_frame_matches(&self, target: u32) -> bool {
        self.frame_ctr_dbg == target || self.state_recorder.replay_frame_counter == target
    }

    pub(super) fn replay_assert_native_coherent(&self, label: &str) {
        let Ok(mode) = crate::debug_env::var("ZELDA3_ASSERT_NATIVE_COHERENT") else {
            return;
        };
        if let Some(frame) = Self::parse_trace_env_u32("ZELDA3_ASSERT_COHERENT_FRAME") {
            if !self.trace_frame_matches(frame) {
                return;
            }
        }
        let mut bad = self.game_state.report_incoherent_with_ram(&self.ram);
        // Some states legitimately diverge mid-frame (gated/mode-reuse projections, the
        // cached-sprite shadow). Pass a comma-separated allow-list in
        // ZELDA3_ASSERT_COHERENT_IGNORE to suppress that baseline so `=panic` aborts only
        // on a genuinely-unexpected drift.
        if let Ok(ignore) = crate::debug_env::var("ZELDA3_ASSERT_COHERENT_IGNORE") {
            let ignore: Vec<&str> = ignore.split(',').map(|s| s.trim()).collect();
            bad.retain(|name| !ignore.contains(name));
        }
        if bad.is_empty() {
            return;
        }
        let f = &self.game_state.frame;
        let msg = format!(
            "native-incoherent frame={} m=0x{:02x} sm=0x{:02x} ssm=0x{:02x} after '{label}': {:?}",
            self.frame_ctr_dbg, f.main_module, f.submodule, f.subsubmodule, bad
        );
        if mode == "panic" {
            panic!("{msg}");
        } else {
            eprintln!("{msg}");
        }
    }

    /// `$7F5800` ancilla-scratch clobber detector. The eight effect states that model that
    /// C-aliased window all bulk-project every frame, so last-writer-wins; this reports the
    /// frames where two of them actually disagree about a byte (i.e. a real clobber rather
    /// than a latent overlap). See GameState::report_scratch_conflicts.
    pub(super) fn replay_assert_scratch_conflicts(&self, label: &str) {
        let Ok(mode) = crate::debug_env::var("ZELDA3_ASSERT_SCRATCH_CONFLICTS") else {
            return;
        };
        let conflicts = self.game_state.report_scratch_conflicts(&self.ram);
        if conflicts.is_empty() {
            return;
        }
        let f = &self.game_state.frame;
        for (addr, winner, wv, loser, lv) in &conflicts {
            let msg = format!(
                "scratch-conflict frame={} m=0x{:02x} sm=0x{:02x} ssm=0x{:02x} after '{label}': \
                 0x{addr:05x} {winner}=0x{wv:02x} clobbers {loser}=0x{lv:02x}",
                self.frame_ctr_dbg, f.main_module, f.submodule, f.subsubmodule
            );
            if mode == "panic" {
                panic!("{msg}");
            } else {
                eprintln!("{msg}");
            }
        }
    }

    pub(super) fn replay_trace_ram_watch(&self, label: &str) {
        self.replay_assert_native_coherent(label);
        self.replay_assert_scratch_conflicts(label);
        let Some(target) = Self::parse_trace_env_u32("ZELDA3_REPLAY_RAM_WATCH_FRAME") else {
            return;
        };
        if !self.trace_frame_matches(target) {
            return;
        }
        let watched_addr = Self::parse_trace_env_u32("ZELDA3_REPLAY_RAM_WATCH_ADDR")
            .and_then(|addr| self.ram.get(addr as usize).map(|value| (addr, *value)));
        let frame = &self.game_state.frame;
        eprintln!(
            "ram-watch frame={} {label} fc=0x{:02x} main={} sub={} subsub={} watch={} d340={:02x} d341={:02x} d342={:02x} d343={:02x} d344={:02x} d345={:02x} d346={:02x} d347={:02x} deep=0x{:04x} normal=0x{:04x} inwater=0x{:02x} link=0x{:04x}/0x{:04x} state=0x{:02x}",
            self.frame_ctr_dbg,
            frame.frame_counter,
            frame.main_module,
            frame.submodule,
            frame.subsubmodule,
            watched_addr
                .map(|(addr, value)| format!("0x{addr:05x}=0x{value:02x}"))
                .unwrap_or_else(|| "none".to_string()),
            self.game_state.player.follower_link.swim_direction_flags(),
            self.game_state.player.tile_detection.deepwater() as u8,
            self.game_state.player.tile_detection.deepwater_high(),
            self.game_state.player.tile_detection.normal_tiles() as u8,
            self.game_state.player.tile_detection.normal_tiles_high(),
            self.game_state.player.follower_link.deep_water_state(),
            self.game_state.player.follower_link.palette_bits_of_oam(),
            self.game_state
                .player
                .follower_link
                .palette_bits_of_oam_word()
                >> 8,
            self.game_state.player.tile_detection.deepwater(),
            self.game_state.player.tile_detection.normal_tiles(),
            self.game_state.player.follower_link.deep_water_state(),
            self.game_state.player.follower_link.x(),
            self.game_state.player.follower_link.y(),
            self.game_state.player.follower_link.handler_state(),
        );
    }

    /// Parse a frame/number debug switch (`N` or `0xN`) through `debug_env`.
    pub(super) fn parse_trace_env_u32(name: &str) -> Option<u32> {
        let value = crate::debug_env::var(name).ok()?;
        if let Some(hex) = value
            .strip_prefix("0x")
            .or_else(|| value.strip_prefix("0X"))
        {
            u32::from_str_radix(hex, 16).ok()
        } else {
            value.parse::<u32>().ok()
        }
    }

    pub fn debug_snes9x_hold_intro_step_this_frame(&self) -> bool {
        self.snes9x_hold_intro_step_this_frame
    }

    pub fn debug_snes9x_intro_step_carry_phase_active(&self) -> bool {
        self.snes9x_intro_step_carry_phase_active
    }

    pub fn debug_snes9x_intro_step_hold_alternate(&self) -> bool {
        self.snes9x_intro_step_hold_alternate
    }

    #[track_caller]
    pub(crate) fn assert_native_frame_state_matches_ram(&self) {
        debug_assert_eq!(
            self.game_state.frame,
            crate::game_state::FrameState::load_from_ram(&self.ram),
            "native frame state diverged from compatibility RAM",
        );
    }

    #[track_caller]
    pub(crate) fn assert_native_world_location_state_matches_ram(&self) {
        debug_assert_eq!(
            self.game_state.world.location,
            crate::game_state::WorldLocationState::load_from_ram(&self.ram),
            "native world location state diverged from compatibility RAM",
        );
    }

    pub fn state_recorder_read_next_replay_state(&mut self, sr: &mut StateRecorder) -> u16 {
        assert!(sr.replay_mode);
        while sr.frames_since_last >= sr.replay_next_cmd_at {
            let mut replay_pos = sr.replay_pos as usize;
            if replay_pos != sr.replay_pos_last_complete as usize {
                sr.frames_since_last = 0;
                if sr.replay_cmd < 0xc0 {
                    sr.last_inputs ^= 1 << (sr.replay_cmd >> 4);
                } else if sr.replay_cmd < 0xd0 {
                    let mut nb = 1usize + ((sr.replay_cmd >> 2) & 3) as usize;
                    if nb == 4 {
                        loop {
                            let t = sr.log.data[replay_pos];
                            replay_pos += 1;
                            nb += t as usize;
                            if t != 255 {
                                break;
                            }
                        }
                    }
                    let mut addr = (((sr.replay_cmd >> 1) & 1) as u32) << 16;
                    addr |= (sr.log.data[replay_pos] as u32) << 8;
                    replay_pos += 1;
                    addr |= sr.log.data[replay_pos] as u32;
                    replay_pos += 1;
                    while nb != 0 {
                        let offset = (addr & 0x1ffff) as usize;
                        self.set_compatibility_ram_byte(offset, sr.log.data[replay_pos]);
                        replay_pos += 1;
                        self.emu_sync_memory_region(offset, 1);
                        addr = addr.wrapping_add(1);
                        nb -= 1;
                    }
                } else if sr.replay_cmd < 0xe0 {
                    let snapshot_size =
                        Self::state_recorder_read_vl(&sr.log.data, &mut replay_pos) as usize;
                    assert!(snapshot_size <= sr.log.size().saturating_sub(replay_pos));
                    let snapshot_end = replay_pos + snapshot_size;
                    let mut state = LoadFuncState::new(&sr.log.data[replay_pos..snapshot_end]);
                    let mut load = SaveLoadFunc::Load(&mut state);
                    self.load_snes_state(&mut load);
                    assert_eq!(state.remaining(), 0);
                    replay_pos = snapshot_end;
                    sr.last_inputs = 0;
                } else {
                    panic!("unknown replay command {:02x}", sr.replay_cmd);
                }
            }
            sr.replay_pos_last_complete = replay_pos as u32;
            if replay_pos >= sr.log.size() {
                sr.replay_pos = replay_pos as u32;
                sr.replay_next_cmd_at = u32::MAX;
                break;
            }

            let cmd = sr.log.data[replay_pos];
            replay_pos += 1;
            let mask = if cmd < 0xc0 { 0xf } else { 0x1 };
            let mut frames = (cmd & mask) as u32;
            if frames == mask as u32 {
                loop {
                    let t = sr.log.data[replay_pos];
                    replay_pos += 1;
                    frames += t as u32;
                    if t != 255 {
                        break;
                    }
                }
            }
            sr.replay_next_cmd_at = frames;
            sr.replay_cmd = cmd;
            sr.replay_pos = replay_pos as u32;
        }
        sr.frames_since_last = sr.frames_since_last.wrapping_add(1);
        sr.replay_frame_counter = sr.replay_frame_counter.wrapping_add(1);
        if sr.replay_frame_counter >= sr.total_frames {
            sr.replay_mode = false;
        }
        sr.last_inputs
    }

    pub fn state_recorder_stop_replay(sr: &mut StateRecorder) {
        if !sr.replay_mode {
            return;
        }
        sr.replay_mode = false;
        sr.total_frames = sr.replay_frame_counter;
        sr.log.data.truncate(sr.replay_pos_last_complete as usize);
    }

    pub fn state_recorder_read_next_replay_state_with_input_override(
        &mut self,
        sr: &mut StateRecorder,
        input_override: Option<u16>,
    ) -> u16 {
        let replay_input = self.state_recorder_read_next_replay_state(sr);
        input_override.unwrap_or(replay_input)
    }

    pub fn zelda_run_frame_with_replay_input_override(
        &mut self,
        inputs: i32,
        replay_input_override: Option<u16>,
    ) -> bool {
        self.rom_random_replay.begin_frame();
        let raw_inputs = inputs as u16;
        let raw_replay_input_override = replay_input_override;
        let inputs = Self::sanitize_frame_inputs(inputs);
        let replay_input_override =
            replay_input_override.map(|input| Self::sanitize_frame_inputs(input as i32));
        self.frame_ctr_dbg = self.frame_ctr_dbg.wrapping_add(1);
        self.replay_trace_ram_watch("frame-entry");
        let mut state_recorder = std::mem::take(&mut self.state_recorder);
        let is_replay = state_recorder.replay_mode;
        let input_state = if is_replay {
            let input_state = self.state_recorder_read_next_replay_state_with_input_override(
                &mut state_recorder,
                replay_input_override,
            );
            self.replay_trace_ram_watch("after-replay-command");
            input_state
        } else {
            Self::state_recorder_record(&mut state_recorder, inputs);
            let apui00 = self.zelda_is_music_playing() as u8;
            if apui00 != self.game_state.system_signals.apui00() {
                self.set_apui00(apui00);
                let apui00_offset = SystemSignalsState::apui00_offset();
                self.emu_sync_memory_region(apui00_offset, 1);
                Self::state_recorder_record_patch_byte(
                    &mut state_recorder,
                    apui00_offset as u32,
                    &[apui00],
                    1,
                );
            }
            if self.game_state.display.has_animated_tile_data_source() {
                if self.game_state.system_signals.bugs_fixed() < BUGFIX_LATEST
                    && !self.rom_startup_timing
                {
                    self.set_bugs_fixed(BUGFIX_LATEST);
                    self.emu_sync_memory_region(RAM_BUGS_FIXED, 1);
                    Self::state_recorder_record_patch_byte(
                        &mut state_recorder,
                        RAM_BUGS_FIXED as u32,
                        &[BUGFIX_LATEST],
                        1,
                    );
                }
                let enhanced_features0 = self.game_state.enhanced_features.bits();
                let wanted_zelda_features = self.wanted_zelda_features;
                if enhanced_features0 != wanted_zelda_features {
                    self.enhanced_features_mut().set_bits(wanted_zelda_features);
                    self.emu_sync_memory_region(ENHANCED_FEATURES0, 4);
                    Self::state_recorder_record_patch_byte(
                        &mut state_recorder,
                        ENHANCED_FEATURES0 as u32,
                        &wanted_zelda_features.to_le_bytes(),
                        4,
                    );
                }
            }
            inputs
        };
        self.previous_host_controller_input = if is_replay {
            raw_replay_input_override.unwrap_or(input_state)
        } else {
            raw_inputs
        };
        self.state_recorder = state_recorder;

        self.sync_native_game_state_from_ram();
        self.assert_native_frame_state_matches_ram();
        self.assert_native_world_location_state_matches_ram();
        self.assert_native_display_state_matches_ram();
        let frame = &self.game_state.frame;
        let use_timed_poly_worker = self.rom_startup_timing
            && rom_intro_poly_thread_is_active(frame.main_module, frame.submodule);
        let title_fade_poly_thread =
            self.rom_startup_timing && frame.main_module == 0 && frame.submodule == 5;
        let bg_fade_poly_thread = self.rom_startup_timing
            && frame.main_module == 0
            && frame.submodule == 7
            && self.game_state.intro_sword.anim_step_raw() == 2;
        let wait_player_poly_teardown = self.rom_startup_timing
            && rom_intro_wait_player_tears_down_poly_thread(
                self.game_state.frame.main_module,
                self.game_state.frame.submodule,
                self.game_state.display.nmi_thread_active,
            );
        self.snes9x_hold_intro_step_this_frame = false;
        let poly_thread_teardown_frame = self.intro_poly_thread_teardown_pending;
        self.intro_poly_thread_teardown_pending = false;
        let run_what = if poly_thread_teardown_frame {
            0
        } else if wait_player_poly_teardown {
            3
        } else if title_fade_poly_thread {
            let poly_phase = self.intro_title_fade_poly_phase;
            let run_main = rom_intro_title_fade_runs_main(poly_phase);
            self.intro_title_fade_defer_suffix_this_frame =
                rom_intro_title_fade_should_yield_suffix(poly_phase);
            self.intro_title_fade_poly_phase = (self.intro_title_fade_poly_phase + 1) % 3;
            if run_main {
                3
            } else {
                2
            }
        } else if bg_fade_poly_thread {
            let (run_main, yield_before_suffix, carry_frames, poly_phase) =
                rom_intro_bg_fade_main_decision(
                    self.intro_bg_fade_carry_frames,
                    self.intro_bg_fade_poly_phase,
                );
            self.intro_bg_fade_defer_suffix_this_frame = yield_before_suffix;
            self.intro_bg_fade_carry_frames = carry_frames;
            self.intro_bg_fade_poly_phase = poly_phase;
            if run_main {
                3
            } else {
                2
            }
        } else if rom_file_select_teardown_runs_with_outgoing_poly_worker(
            frame.main_module,
            frame.submodule,
            self.game_state.display.nmi_thread_active,
            self.game_state.display.nmi_thread_uses_poly_stack(),
        ) {
            3
        } else if legacy_poly_scheduler_is_active(
            self.game_state.system_signals.bugs_fixed(),
            use_timed_poly_worker,
            self.game_state.display.nmi_thread_active,
        ) {
            if self.game_state.display.nmi_thread_uses_poly_stack() {
                2
            } else {
                1
            }
        } else {
            let virq = self.game_state.display.vertical_irq_trigger;
            let carry = if self.game_state.display.nmi_thread_active {
                if use_timed_poly_worker {
                    self.game_state.ending.attract_scene.intro_did_run_step() != 0
                } else {
                    let carry = self.advance_crystal_rotation_counter(virq);
                    self.emu_sync_memory_region(CRYSTAL_ROTATION_COUNTER, 1);
                    carry
                }
            } else {
                false
            };
            if carry {
                3
            } else {
                1
            }
        };
        if !title_fade_poly_thread {
            self.intro_title_fade_poly_phase = 0;
            self.intro_title_fade_defer_suffix_this_frame = false;
        }
        if !bg_fade_poly_thread {
            self.intro_bg_fade_carry_frames = 0;
            self.intro_bg_fade_poly_phase = 0;
            self.intro_bg_fade_defer_suffix_this_frame = false;
        }
        // The final replay-sanitized input and run decision are now fixed.
        // Own exactly one source S9xMainLoop interval across either dispatch
        // branch; a callback which calls the public internal entry sees this
        // active guard and cannot advance it again.
        let owns_original_timing_dispatch = self.begin_original_timing_host_dispatch(input_state);
        if self.emu_runframe.is_none()
            || self.game_state.enhanced_features.bits() != 0
            || self.dialogue_flags != 0
        {
            crate::types::ww_set_cur_frame(self.frame_ctr_dbg);
            self.replay_trace_ram_watch("before-run-frame-internal");
            self.zelda_run_frame_internal(input_state, run_what as u8);
            self.replay_trace_ram_watch("after-run-frame-internal");
        } else if let Some(func) = self.emu_runframe {
            func(self, input_state, run_what);
        }
        self.zelda_push_apu_state();
        if self
            .game_execution_scheduler
            .take_audio_nmi_after_host_publication()
        {
            // The resumed C caller reached the following NMI after this host's
            // audio batch was published. Sample its commands now so the next
            // batch exposes them at the same APU clock boundary as the ROM.
            let ambient_nmi = (
                self.game_state.system_signals.ambient_sound_effect(),
                self.game_state.system_signals.last_ambient_sound_effect(),
            );
            self.interrupt_nmi_audio_parts();
            self.audio_after_publication_ambient_nmi = Some(ambient_nmi);
            self.audio_nmi_processed_before_main = true;
        }
        self.replay_trace_ram_watch("after-apu");
        self.finish_original_timing_host_dispatch(owns_original_timing_dispatch);
        is_replay
    }

    pub fn install_rom_random_replay(
        &mut self,
        samples: Vec<crate::RomRandomSample>,
        start_execution_frame: u32,
    ) {
        self.rom_random_replay
            .install(samples, start_execution_frame);
    }

    pub fn finish_rom_random_replay(&self) -> Result<(), String> {
        self.rom_random_replay.finish()
    }

    pub fn finish_rom_random_replay_through(&self, end_execution_frame: u32) -> Result<(), String> {
        let result = self.rom_random_replay.finish_through(end_execution_frame);
        if result.is_err()
            && self
                .game_execution_scheduler
                .work_suspends_translated_call_stack()
        {
            // The ROM's host boundary landed inside a sprite slot after that
            // slot had already drawn its random numbers, while the native
            // loop parks before the slot (route hosts 1241074, 1414148). The
            // parked slot resumes before the next iteration increments the
            // frame counter, so the samples it consumes there are the same
            // ones the ROM drew: carry them into the next host.
            return Ok(());
        }
        result
    }

    pub fn replay_asset_memblk_bytes(&self, asset: usize, index: usize) -> Option<Vec<u8>> {
        self.asset_memblk(asset, index).map(|blk| blk.ptr.to_vec())
    }

    pub fn replay_asset_word(&self, asset: usize, word_index: usize) -> Option<u16> {
        let bytes = self.asset_raw(asset)?;
        let offset = word_index.checked_mul(2)?;
        (offset + 1 < bytes.len()).then(|| read_word_from_slice(bytes, offset))
    }

    pub fn replay_gloves_color(&self, index: usize) -> u16 {
        self.gloves_color[index & 1]
    }
}
