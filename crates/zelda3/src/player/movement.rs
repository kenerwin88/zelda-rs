//! Swimming, movement animation, velocity integration, slopes, and direction limits.

use super::*;

#[cfg(test)]
mod marker30_tests {
    use super::*;

    #[test]
    fn marker30_direction_limits_whole_body_matches_rom() {
        let path=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc");
        let Ok(rom)=std::fs::read(path)else{return};
        let machine = |native:&ZeldaState,entry:u16| {
            let mut s=snes::Snes::new();s.cart.load(snes::CartType::LoRom,&rom,0x2000);
            s.ram.copy_from_slice(&native.wram_image());s.ram[0x1fd..0x200].copy_from_slice(&[0xff,0x7f,0]);
            s.cpu.e=false;s.cpu.unpack_flags(0x30);s.cpu.k=7;s.cpu.db=7;s.cpu.pc=entry;s.cpu.sp=0x1fc;s
        };
        for direction in 0..16 {
            let mut native=ZeldaState::new();let mut image=native.wram_image().into_owned();
            image[0x10]=7;image[0x1b]=1;image[0x67]=direction;image[0x20]=0x40;image[0x22]=0x40;
            native.import_wram_image(&image);native.sync_native_game_state_from_ram();native.set_lag_free(true);
            crate::cycle_ledger::assert_rom_marker_work(0x07b660,machine(&native,0xb660),
                |s|s.cpu.k==7 && (0xb660..0xb7c3).contains(&s.cpu.pc),|| {
                    let _scope=crate::cycle_ledger::routine(0x07b660);native.player_limit_directions_inner();
                });
        }
        // Enter after each real probe with independently varied result bits.
        // These unmodified-ROM suffix fixtures cover the response branches
        // that an all-clear tile map cannot reach in the whole-call cases.
        for vertical in [false,true] {
            for bits in [0,1,2,3,4,8,16,32,48,51] {
                for door in [0,2] {
                    for direction in [1,2,4,8,5,10] {
                        for collision in [0,2] {
                            for (auxiliary,moving,spike) in [(0,0,0),(1,0,0),(0,1,0),(0,1,3)] {
                                let mut native=ZeldaState::new();let mut image=native.wram_image().into_owned();
                                for(a,v)in[(0x10,7),(0x1b,1),(0x67,direction),(0x0e,bits),(0x62,door),
                                    (0x46c,collision),(0x4d,auxiliary),(0x34a,moving),(0x2e8,spike),
                                    (0x0c,3),(0x20,0x40),(0x22,0x40)] {image[a]=v;}
                                native.import_wram_image(&image);native.sync_native_game_state_from_ram();native.set_lag_free(true);
                                let (start,end)=if vertical{(0xb682,0xb701)}else{(0xb71b,0xb798)};
                                crate::cycle_ledger::assert_rom_marker_work(0x07b660,machine(&native,start),
                                    |s|s.cpu.k==7 && (start..end).contains(&s.cpu.pc),|| {
                                        let _scope=crate::cycle_ledger::routine(0x07b660);
                                        native.limit_direction_response_markers(vertical);
                                    });
                            }
                        }
                    }
                }
            }
        }
    }
}
use crate::game_state::{CollisionAxis, CollisionDirection, CollisionOrder, MovementProbeKind, PlayerAxis};
use crate::tile_definition::NativeTile;

#[cfg(test)]
mod velocity_price_tests {
    use super::*;
    use snes::{CartType, CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline,
        CpuRasterPosition, RomCpuTimingProbe, Snes, SourcePpuReadState};

    #[test]
    fn movement_tail_prices_match_executed_rom_paths() {
        let rom = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc")).unwrap();
        let mut cases = Vec::new();
        for airborne in [0u8, 1] {
            for velocity in [0u8, 1, 0x7f, 0x80, 0xff] {
                for handler in [0u8, 10] {
                    for platform in [0u8, 2] {
                        cases.push((0xe373u16, vec![(0x4d, airborne), (0x27, velocity),
                            (0x28, velocity), (0x29, velocity), (0x5d, handler), (0x2f5, platform)]));
                    }
                }
            }
        }
        cases.push((0xe3e0, vec![(0xb7c, 0xff), (0xb7e, 1)]));
        for timer in [0u8, 1, 2, 0x80] {
            for mode in [0u8, 1, 2, 3] {
                for flag in [0u8, 1] {
                    for acceleration in [0u16, 1, 0x80, 0x200, 0x8000] {
                        cases.push((0xe42a, vec![(0x326, timer), (0x328, timer),
                            (0x32b, mode), (0x32d, mode), (0x34a, flag), (0x6a, flag),
                            (0x33c, acceleration as u8), (0x33d, (acceleration >> 8) as u8),
                            (0x33e, acceleration as u8), (0x33f, (acceleration >> 8) as u8),
                            (0x334, 0x80), (0x336, 0x80), (0x338, flag), (0x33a, flag)]));
                    }
                }
            }
        }
        for velocity in [0u16, 0x7f, 0x800, 0x7fff, 0x8000, 0xffff] {
            cases.push((0xe4d3, vec![(0xa, velocity as u8), (0xb, (velocity >> 8) as u8),
                (8, velocity as u8), (9, (velocity >> 8) as u8)]));
        }
        for direction in 0u8..16 {
            for input in [0u8, 1] {
                for water in [0u8, 1] {
                    cases.push((0xe545, vec![(0x67, direction), (0xf0, input), (0x351, water)]));
                }
            }
        }
        for collision in [0u8, 1, 4] {
            for z in [0u8, 1, 0xff] {
                for belt in 0u8..=4 {
                    for floor in [0u8, 1, 0xff] {
                        cases.push((0xe373, vec![(0x46c, collision), (0x24, z),
                            (0x3f3, belt), (0x310, floor), (0x312, floor), (0x322, 3)]));
                    }
                }
            }
        }
        for (entry, changes) in cases {
            let mut native = ZeldaState::new();
            native.set_lag_free(true);
            let mut image = native.wram_image().into_owned();
            for &(address, value) in &changes { image[address] = value; }
            native.import_wram_image(&image);
            native.sync_native_game_state_from_ram();
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&native.wram_image());
            machine.cpu.e = false;
            machine.cpu.unpack_flags(if entry == 0xe4d3 { 0x10 } else { 0x30 });
            machine.cpu.k = 7;
            machine.cpu.db = 7;
            machine.cpu.pc = entry;
            machine.cpu.sp = 0x1fc;
            let clock = CpuMasterTimeline::at_raster(0, CpuRasterPosition::new(0, 0),
                CpuBusWorkload::default(), CpuFieldTiming::non_interlace(false));
            let mut source = RomCpuTimingProbe::new(machine, clock,
                SourcePpuReadState::snes9x_reset()).unwrap();
            let mut measured = 0;
            for _ in 0..3000 {
                let pc = source.snes().cpu.pc;
                let step = source.step().unwrap();
                if (0xe373..=0xe670).contains(&pc) {
                    measured += step.transactions.iter()
                        .map(|t| u64::from(t.duration_master_cycles)).sum::<u64>();
                }
                if pc == 0xe405 || pc == 0xe575 { break; }
            }
            assert_eq!(source.snes().cpu.k, 0, "return entry={entry:x}");
            let before = crate::cycle_ledger::master();
            match entry {
                0xe373 => native.link_move_position(),
                0xe545 => native.link_dash_in_place(),
                0xe3e0 => native.link_handle_velocity_and_sand_drag(0, 0),
                0xe42a => native.handle_swim_stroke_and_subpixels(),
                0xe4d3 => native.player_something_with_velocity_tired_or_swim(
                    u16::from_le_bytes([image[0xa], image[0xb]]),
                    u16::from_le_bytes([image[8], image[9]])),
                _ => unreachable!(),
            }
            let total = crate::cycle_ledger::master() - before;
            assert_eq!(total, measured, "entry={entry:x} operands={changes:?}");
        }
    }

    #[test]
    fn velocity_selection_prices_match_executed_rom_paths() {
        let rom = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc")).unwrap();
        // Each tuple changes live branch operands, never measured lag.
        let mut cases = Vec::new();
        for direction in 0u8..16 {
            for speed in [0u8, 12, 16, 24] {
                for water in [0u8, 1] {
                    cases.push(vec![(0x67, direction), (0x5e, speed), (0x351, water)]);
                }
            }
        }
        for near in [0u8, 1, 3] {
            for modifier in [0u8, 1, 2, 15, 16, 47, 48, 255] {
                for sub in [0u8, 2, 8, 16] {
                    cases.push(vec![(0x5b, near), (0x57, modifier), (0x11, sub), (0x67, 9)]);
                }
            }
        }
        for changes in [vec![(0x11, 2), (0x10, 14)], vec![(0xb7b, 1)],
            vec![(0x5d, 4)], vec![(0x34a, 1)], vec![(0x34a, 1), (0x372, 1), (0x2f1, 32)],
            vec![(0x372, 1), (0x2f1, 32)], vec![(0x316, 15)], vec![(0x317, 15)]] {
            cases.push(changes);
        }
        for changes in cases {
            let mut native = ZeldaState::new();
            native.set_lag_free(true);
            let mut image = native.wram_image().into_owned();
            for &(address, value) in &changes { image[address] = value; }
            native.import_wram_image(&image);
            native.sync_native_game_state_from_ram();
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&native.wram_image());
            machine.cpu.e = false;
            machine.cpu.unpack_flags(0x30);
            machine.cpu.k = 7;
            machine.cpu.pc = 0xe245;
            machine.cpu.sp = 0x1fc;
            let clock = CpuMasterTimeline::at_raster(0, CpuRasterPosition::new(0, 0),
                CpuBusWorkload::default(), CpuFieldTiming::non_interlace(false));
            let mut source = RomCpuTimingProbe::new(machine, clock,
                SourcePpuReadState::snes9x_reset()).unwrap();
            let mut measured = 0;
            for _ in 0..1000 {
                let step = source.step().unwrap();
                measured += step.transactions.iter()
                    .map(|t| u64::from(t.duration_master_cycles)).sum::<u64>();
                if matches!(source.snes().cpu.pc, 0xe373 | 0xe3e0 | 0xe42a | 0xe545 | 0xe402) { break; }
            }
            assert!(matches!(source.snes().cpu.pc, 0xe373 | 0xe3e0 | 0xe42a | 0xe545 | 0xe402));
            let own = crate::cycle_ledger::measure_routine_self_work(0x07e245, || {
                native.link_handle_velocity_before_move_position();
            });
            assert_eq!(own, measured, "operands={changes:?}");
        }
    }
}

impl ZeldaState {
    pub(in crate::zelda_rtl) fn check_if_room_needs_double_layer_check_marked(&mut self) -> bool {
        // $07:B97C..B9B2; even BNE +0 at B98B consumes a taken-branch cycle.
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x07b97c));
        let b = crate::cycle_ledger::blocks(self.lag_free);
        let collision = self.game_state.dungeon.room_load.header_collision();
        b.block(0x07b97c);
        if collision == 0 { b.taken(0x07b97f); b.block(0x07b9b1); }
        else {
            b.block(0x07b981);
            if collision == 4 { b.taken(0x07b983); b.block(0x07b9b1); }
            else {
                b.block(0x07b985);
                if collision < 2 { b.taken(0x07b987); }
                else {
                    b.block(0x07b989);
                    if collision != 3 { b.taken(0x07b98b); }
                    b.block(0x07b98d);
                }
                b.block(0x07b9ab);
            }
        }
        self.check_if_room_needs_double_layer_check()
    }


    pub(in crate::zelda_rtl) fn link_handle_swim_accels(&mut self) {
        let mut mask = 0x0c;
        for offset in [0, 2] {
            if self.game_state.player.follower_link.joypad1h_last() & mask != 0 {
                let acceleration = self
                    .game_state
                    .player
                    .swim_acceleration
                    .acceleration(offset);
                let max_speed = self.game_state.player.swim_acceleration.max_speed(offset);
                if acceleration != 0 && max_speed >= 384 {
                    let target = LINK_HANDLE_SWIM_ACCELS_SWIM_ACCELERATION_TARGETS
                        .iter()
                        .copied()
                        .find(|value| *value >= acceleration)
                        .unwrap_or(384);
                    self.swim_acceleration_mut().set_max_speed(offset, target);
                } else if max_speed != 0 {
                    let target = max_speed.wrapping_add(160).min(384);
                    self.swim_acceleration_mut().set_max_speed(offset, target);
                } else {
                    self.swim_acceleration_mut().set_acceleration(offset, 1);
                    self.swim_acceleration_mut().set_max_speed(offset, 240);
                }
            }
            mask >>= 2;
        }
    }

    pub(super) fn link_flag_max_accels(&mut self) {
        if self.game_state.player.follower_link.flag_moving() == 0 {
            return;
        }

        for offset in [2, 0] {
            let acceleration = self
                .game_state
                .player
                .swim_acceleration
                .acceleration(offset);
            if acceleration != 0 {
                self.swim_acceleration_mut()
                    .set_max_speed(offset, acceleration);
                self.swim_acceleration_mut().set_mode(offset, 1);
            }
        }
    }

    pub(super) fn link_set_ice_max_accel(&mut self) {
        if self.game_state.player.follower_link.flag_moving() == 0 {
            return;
        }
        self.swim_acceleration_mut().set_max_speed(0, 0x0180);
        self.swim_acceleration_mut().set_max_speed(2, 0x0180);
    }

    pub(in crate::zelda_rtl) fn link_set_momentum(&mut self) {
        let joy = self.game_state.player.follower_link.joypad1h_last() & 0x0f;
        let mut mask = 0x0c;
        let mut bit = 0x08;
        for offset in [0, 2] {
            if joy & mask != 0 {
                let flag_moving = self.game_state.player.follower_link.flag_moving();
                let stroke_timer = if flag_moving != 0 {
                    LINK_SET_MOMENTUM_SWIM_STROKE_TIMERS_BY_MOVING_FLAG[(flag_moving - 1) as usize]
                } else {
                    32
                };
                self.follower_link_state_mut()
                    .set_swim_stroke_frame_counter(offset, stroke_timer as u16);

                if (self.game_state.player.follower_link.swim_direction_flags()
                    | self.game_state.player.follower_link.direction())
                    & mask
                    == mask
                {
                    self.swim_acceleration_mut().set_mode(offset, 2);
                } else {
                    let direction = if joy & bit != 0 { 0 } else { 1 };
                    self.swim_acceleration_mut()
                        .set_acceleration_direction(offset, direction);
                    self.swim_acceleration_mut().set_mode(offset, 0);
                }

                if self.game_state.player.swim_acceleration.max_speed(offset) == 0 {
                    self.swim_acceleration_mut().set_max_speed(offset, 240);
                }
            }
            mask >>= 2;
            bit >>= 2;
        }
    }

    pub(in crate::zelda_rtl) fn player_handler_04_swimming(&mut self) {
        if self.game_state.player.follower_link.auxiliary_state() != 0 {
            self.follower_link_state_mut()
                .interrupt_swimming_for_auxiliary_state();
            self.reset_all_acceleration();
            self.link_state_recoil();
            return;
        }

        self.follower_link_state_mut().clear_swimming_action_state();
        if !self.game_state.inventory.items.has_flippers() {
            return;
        }

        let has_swim_velocity = self.game_state.player.swim_acceleration.acceleration(0)
            | self.game_state.player.swim_acceleration.acceleration(2)
            != 0;
        if !has_swim_velocity {
            let swim = &self.game_state.player.swim_acceleration;
            if swim.mode_low(0) != 2 && swim.mode_low(1) != 2 {
                self.reset_all_acceleration();
            }
            self.follower_link_state_mut().advance_idle_swim_animation();
        } else {
            self.follower_link_state_mut()
                .advance_active_swim_animation(
                    &PLAYER_HANDLER_04_SWIMMING_ACTIVE_SWIM_ANIMATION_DELAYS,
                );
        }

        if self.game_state.player.follower_link.hard_swim_stroke() == 0 {
            let hard_stroke = ((self.game_state.player.follower_link.filtered_joypad_l() & 0x80)
                | self.game_state.player.follower_link.filtered_joypad_h())
                & 0xc0;
            if !has_swim_velocity || hard_stroke == 0 {
                self.link_handle_swim_movements();
                return;
            }
            self.follower_link_state_mut()
                .start_hard_swim_stroke(hard_stroke);
            self.ancilla_sfx2_near(37);
            self.link_handle_swim_accels();
        }

        self.follower_link_state_mut().tick_hard_swim_stroke();
        self.link_handle_swim_movements();
    }

    fn advance_link_animation_step(&mut self, delay: u8, wrap_at: u8, wrap_to: u8) {
        if self
            .follower_link_state_mut()
            .advance_frame_change_counter(delay)
        {
            self.follower_link_state_mut()
                .advance_animation_step(wrap_at, wrap_to);
        }
    }

    fn advance_link_animation_step_at_least(&mut self, delay: u8, wrap_at: u8, wrap_to: u8) {
        if self
            .follower_link_state_mut()
            .advance_frame_change_counter(delay)
        {
            self.follower_link_state_mut()
                .advance_animation_step_at_least(wrap_at, wrap_to);
        }
    }

    pub(in crate::zelda_rtl) fn link_handle_swim_movements(&mut self) {
        let mut direction = (self
            .game_state
            .player
            .follower_link
            .force_move_any_direction() as u8)
            & 0x0f;
        if direction == 0 {
            direction = self.game_state.player.follower_link.joypad1h_last() & 0x0f;
        }

        if direction == 0 {
            self.follower_link_state_mut()
                .clear_swim_movement_velocity();
            self.link_flag_max_accels();
            if self.game_state.player.follower_link.flag_moving() != 0 {
                if self.game_state.player.follower_link.is_running() {
                    direction = self.game_state.player.follower_link.swim_direction_flags();
                } else {
                    if self.game_state.player.swim_acceleration.acceleration(0)
                        | self.game_state.player.swim_acceleration.acceleration(2)
                        == 0
                    {
                        self.follower_link_state_mut().clear_defense_flags();
                        self.link_reset_swimming_state_marked();
                    }
                    self.finish_swim_movement_tail();
                    return;
                }
            } else {
                self.follower_link_state_mut()
                    .reset_idle_swim_animation_if_out_of_water();
                self.finish_swim_movement_tail();
                return;
            }
        }

        if direction != self.game_state.player.follower_link.swim_direction_flags() {
            self.follower_link_state_mut()
                .set_swim_direction_flags(direction);
            self.follower_link_state_mut()
                .reset_swim_subpixel_and_defense_state();
        }
        self.link_set_ice_max_accel();
        self.link_set_momentum();
        self.link_set_the_max_accel();
        self.finish_swim_movement_tail();
    }

    fn finish_swim_movement_tail(&mut self) {
        self.link_handle_diagonal_collision();
        self.link_handle_velocity();
        self.link_handle_cardinal_collision();
        self.link_handle_moving_animation_full_long_entry();
        self.follower_link_state_mut().clear_pit_correction();
        self.handle_indoor_camera_and_doors();
    }

    pub(super) fn link_set_the_max_accel(&mut self) {
        if self.game_state.player.follower_link.flag_moving() != 0
            || self.game_state.player.follower_link.hard_swim_stroke() != 0
        {
            return;
        }

        let mut mask = 0x0c;
        for offset in [0, 2] {
            let mode = self.game_state.player.swim_acceleration.mode(offset);
            if self.game_state.player.follower_link.joypad1h_last() & mask != 0 && mode != 2 {
                let speed_active = self
                    .game_state
                    .player
                    .swim_acceleration
                    .speed_active_flag(offset);
                let acceleration = self
                    .game_state
                    .player
                    .swim_acceleration
                    .acceleration(offset);
                let max_speed = self.game_state.player.swim_acceleration.max_speed(offset);
                if speed_active != 0 || (acceleration >= 240 && acceleration >= max_speed) {
                    self.swim_acceleration_mut().set_mode(offset, 0);
                    if acceleration >= 240 {
                        self.swim_acceleration_mut()
                            .set_speed_active_flag(offset, 1);
                        self.swim_acceleration_mut().set_mode(offset, 1);
                    } else {
                        self.swim_acceleration_mut().set_max_speed(offset, 240);
                        self.swim_acceleration_mut()
                            .set_speed_active_flag(offset, 0);
                    }
                }
            } else {
                self.swim_acceleration_mut().set_max_speed(offset, 240);
                self.swim_acceleration_mut()
                    .set_speed_active_flag(offset, 0);
            }
            mask >>= 2;
        }
    }

    pub(in crate::zelda_rtl) fn link_handle_toss(&mut self) -> bool {
        if self.game_state.player.follower_link.y_button_action_flags() & 0x80 == 0
            || self.game_state.player.follower_link.filtered_joypad_l() & 0x80 == 0
            || self.game_state.player.follower_link.is_lift_throw_primed()
        {
            return false;
        }

        self.follower_link_state_mut()
            .clear_lift_throw_scratch_state();
        self.follower_link_state_mut().set_y_button_action_step(0);
        self.follower_link_state_mut().set_y_button_action_flags(0);
        self.follower_link_state_mut().clear_direction_lock_bits(1);
        true
    }

    pub(in crate::zelda_rtl) fn link_cancel_dash(&mut self) {
        if !self.game_state.player.follower_link.is_running() {
            return;
        }
        for i in (0..=4).rev() {
            if self.ancilla_slot_view(i).ancilla_type() == 0x1e {
                self.ancilla_slot_view_mut(i).set_ancilla_type(0);
            }
        }
        self.follower_link_state_mut().cancel_dash_state();
        self.swim_acceleration_mut().clear_mode_low_axis();
    }

    pub(in crate::zelda_rtl) fn repel_dash(&mut self) {
        if self.game_state.player.follower_link.is_running()
            && self.game_state.player.follower_link.dash_counter() != 64
        {
            self.link_reset_swimming_state();
            self.ancilla_add_dash_tremor(29, 1);
            self.prepare_apply_rumble_to_sprites();
            if self.game_state.system_signals.sound_effect_2() & 0x3f != 27
                && self.game_state.system_signals.sound_effect_2() & 0x3f != 50
            {
                self.ancilla_sfx3_near(3);
            }
            self.link_apply_tile_rebound();
        }
    }

    pub(in crate::zelda_rtl) fn sprite_repel_dash(&mut self) {
        self.follower_link_state_mut()
            .set_last_direction_moved_towards_from_facing();
        self.repel_dash();
    }

    pub(super) fn link_apply_tile_rebound(&mut self) {
        let dir = self
            .game_state
            .player
            .follower_link
            .last_direction_moved_towards() as usize;
        self.follower_link_state_mut().set_actual_velocity_xy(
            LINK_APPLY_TILE_REBOUND_DASH_REBOUND_X_VELOCITY_BY_DIRECTION[dir],
            LINK_APPLY_TILE_REBOUND_DASH_REBOUND_Y_VELOCITY_BY_DIRECTION[dir],
        );
        self.follower_link_state_mut().set_incapacitated_timer(24);
        self.follower_link_state_mut()
            .set_actual_z_velocity_and_copy(36);
        if self.game_state.player.follower_link.flag_moving() != 0 {
            self.follower_link_state_mut().set_direction_and_swim_flags(
                LINK_APPLY_TILE_REBOUND_DIRECTION_BITS_BY_FACING[dir],
            );
            self.swim_acceleration_mut().set_acceleration_direction(
                0,
                LINK_APPLY_TILE_REBOUND_DASH_REBOUND_SWIM_Y_DIRECTIONS[dir] as u16,
            );
            self.swim_acceleration_mut().set_acceleration_direction(
                2,
                LINK_APPLY_TILE_REBOUND_DASH_REBOUND_SWIM_X_DIRECTIONS[dir] as u16,
            );
            let i = (self.game_state.player.follower_link.flag_moving() - 1) as usize * 4 + dir;
            self.swim_acceleration_mut().set_acceleration(
                0,
                LINK_APPLY_TILE_REBOUND_DASH_REBOUND_SWIM_Y_ACCELERATIONS[i],
            );
            self.swim_acceleration_mut().set_acceleration(
                2,
                LINK_APPLY_TILE_REBOUND_DASH_REBOUND_SWIM_X_ACCELERATIONS[i],
            );
        }
        self.follower_link_state_mut().set_auxiliary_state(1);
        self.follower_link_state_mut().set_dash_noise_request();
        self.tile_detect_position_mut()
            .clear_interaction_scratch_x_low();
        self.follower_link_state_mut().clear_electrocute_on_touch();
        self.follower_link_state_mut().set_speed_setting(0);
        self.follower_link_state_mut().clear_direction_lock();
        self.follower_link_state_mut()
            .clear_moving_against_diag_tile();
        if self
            .game_state
            .player
            .follower_link
            .last_direction_moved_towards()
            & 2
            != 0
        {
            self.follower_link_state_mut().set_y_velocity(0);
        } else {
            self.follower_link_state_mut().set_x_velocity(0);
        }
    }

    pub(in crate::zelda_rtl) fn link_handle_moving_animation_full_long_entry(&mut self) {
        crate::cycle_ledger::charge_routine(0x07_e6a6,
            crate::cycle_models::player_animation::full_entry_master_cycles(&self.ram));
        if self.game_state.player.follower_link.handler_state() == 4 {
            self.link_handle_moving_animation_swimming();
            return;
        }

        let mut r0 = self.game_state.player.follower_link.last_direction();
        if r0 == 0 {
            return;
        }
        if self.game_state.player.follower_link.flag_moving() != 0 {
            r0 = self.game_state.player.follower_link.swim_direction_flags();
        }
        if self.game_state.player.follower_link.direction_lock() == 0 {
            let mut y;
            if self
                .game_state
                .player
                .follower_link
                .num_orthogonal_directions()
                == 0
            {
                y = if r0 & 0x0c != 0 { 0 } else { 4 };
            } else if self.game_state.player.follower_link.doorway_state() != 0 {
                y = self
                    .game_state
                    .player
                    .follower_link
                    .doorway_state()
                    .wrapping_mul(2)
                    & !3;
            } else if r0
                & LINK_HANDLE_MOVING_ANIMATION_FULL_LONG_ENTRY_FACING_DIRECTION_BITS
                    [self.game_state.player.follower_link.facing_index()]
                != 0
            {
                self.link_handle_moving_animation_start_with_dash();
                return;
            } else {
                y = if r0 & 0x0c != 0 { 0 } else { 4 };
            }
            if y != 4 {
                y = y.wrapping_add(if r0 & 4 != 0 { 2 } else { 0 });
            } else {
                y = y.wrapping_add(if r0 & 1 != 0 { 2 } else { 0 });
            }
            self.follower_link_state_mut().set_facing(y);
        }
        self.link_handle_moving_animation_start_with_dash();
    }

    pub(in crate::zelda_rtl) fn link_handle_moving_animation_start_with_dash(&mut self) {
        if self.game_state.player.follower_link.is_running() {
            self.link_handle_moving_animation_dash();
            return;
        }
        let mut x = self.game_state.player.follower_link.facing() >> 1;
        if self.game_state.player.follower_link.speed_setting() == 6 {
            x = x.wrapping_add(4);
        } else if self.game_state.player.follower_link.flag_moving() != 0 {
            if self.game_state.player.follower_link.joypad1h_last() & 0x0f == 0 {
                self.follower_link_state_mut().clear_animation_step();
                return;
            }
            x = x.wrapping_add(4);
        }

        if self.game_state.player.follower_link.handler_state() == 23
            || (self.game_state.enhanced_features.has(4096)
                && self.game_state.player.follower_link.handler_state() == 28)
        {
            if self.game_state.player.follower_link.animation_step() < 4
                && self.game_state.player.follower_link.on_somaria_platform() != 2
            {
                self.advance_link_animation_step(
                    LINK_HANDLE_MOVING_ANIMATION_START_WITH_DASH_POSE_ANIMATION_DELAYS[x as usize],
                    4,
                    0,
                );
            } else {
                self.follower_link_state_mut().clear_animation_step();
            }
            return;
        }

        if self.game_state.frame.submodule == 18 || self.game_state.frame.submodule == 19 {
            x = 12;
        } else if self.game_state.frame.submodule != 14
            && !self
                .game_state
                .player
                .follower_link
                .is_lifting_or_carrying()
        {
            if self.game_state.player.follower_link.defense_flags() & 0x8d != 0 {
                x = 12;
            } else if self
                .game_state
                .player
                .follower_link
                .water_ripple_or_grass_state()
                == 0
                && self.game_state.player.follower_link.button_b_frames() == 0
            {
                let mut idx = self.game_state.player.follower_link.animation_step();
                if self.game_state.player.follower_link.speed_setting() == 6 {
                    idx = idx.wrapping_add(8);
                }
                if self.game_state.player.follower_link.flag_moving() != 0 {
                    idx = idx.wrapping_add(8);
                }
                if self.game_state.player.follower_link.on_somaria_platform() != 2 {
                    self.advance_link_animation_step(
                        LINK_HANDLE_MOVING_ANIMATION_START_WITH_DASH_WALK_ANIMATION_DELAYS
                            [idx as usize],
                        9,
                        1,
                    );
                }
                return;
            }
        }

        if self.game_state.player.follower_link.animation_step() < 6
            && self.game_state.player.follower_link.on_somaria_platform() != 2
        {
            self.advance_link_animation_step(
                LINK_HANDLE_MOVING_ANIMATION_START_WITH_DASH_POSE_ANIMATION_DELAYS[x as usize],
                6,
                0,
            );
        } else {
            self.follower_link_state_mut().clear_animation_step();
        }
    }

    pub(super) fn link_handle_moving_animation_swimming(&mut self) {
        let r0 = self.game_state.player.follower_link.swim_direction_flags();
        if r0 == 0 || self.game_state.player.follower_link.direction_lock() != 0 {
            return;
        }
        let mut y;
        if self
            .game_state
            .player
            .follower_link
            .num_orthogonal_directions()
            != 0
        {
            if self.game_state.player.follower_link.doorway_state() != 0 {
                y = self
                    .game_state
                    .player
                    .follower_link
                    .doorway_state()
                    .wrapping_mul(2)
                    & !3;
            } else if r0
                & LINK_HANDLE_MOVING_ANIMATION_SWIMMING_FACING_DIRECTION_BITS
                    [self.game_state.player.follower_link.facing_index()]
                != 0
            {
                return;
            } else {
                y = if r0 & 0x0c != 0 { 0 } else { 4 };
            }
        } else {
            y = if r0 & 0x0c != 0 { 0 } else { 4 };
        }
        if y != 4 {
            y = y.wrapping_add(if r0 & 4 != 0 { 2 } else { 0 });
        } else {
            y = y.wrapping_add(if r0 & 1 != 0 { 2 } else { 0 });
        }
        self.follower_link_state_mut().set_facing(y);
    }

    pub(super) fn link_handle_moving_animation_dash(&mut self) {
        let mut t = 6usize;
        while self.game_state.player.follower_link.dash_countdown()
            >= LINK_HANDLE_MOVING_ANIMATION_DASH_DASH_CHARGE_ANIM_THRESHOLDS[t]
            && t != 0
        {
            t -= 1;
        }
        if self.game_state.player.follower_link.button_b_frames() < 9
            && self
                .game_state
                .player
                .follower_link
                .water_ripple_or_grass_state()
                == 0
        {
            self.advance_link_animation_step(
                LINK_HANDLE_MOVING_ANIMATION_DASH_DASH_CHARGE_ANIM_DELAYS[t * 8],
                9,
                1,
            );
        } else {
            self.advance_link_animation_step_at_least(
                LINK_HANDLE_MOVING_ANIMATION_DASH_DASH_CHARGE_MIN_ANIM_STEPS[t],
                6,
                0,
            );
        }
    }

    pub(super) fn link_apply_moving_floor_velocity(&mut self) {
        if self.lag_free {
            // $07:E5CD..E5E3: shared floor-velocity tail, including RTS.
            crate::cycle_ledger::charge_routine(0x07_e5cd, 346);
        }
        self.follower_link_state_mut()
            .clear_orthogonal_direction_count();
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(self.game_state.dungeon.moving_floor.floor_y_velocity());
        let x = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_add(self.game_state.dungeon.moving_floor.floor_x_velocity());
        self.follower_link_state_mut().set_y(y);
        self.follower_link_state_mut().set_x(x);
    }

    pub(super) fn link_apply_conveyor(&mut self) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x07_e5f0));
        if self.lag_free { crate::cycle_ledger::charge(self.link_conveyor_master()); }
        if self.game_state.player.follower_link.conveyor_belt_state() == 0 {
            return;
        }
        if !self
            .game_state
            .player
            .follower_link
            .is_grounded_or_z_sentinel()
        {
            return;
        }
        if self.game_state.player.follower_link.grabbing_wall_has(1)
            || self.game_state.player.follower_link.handler_state() == 19
            || self.game_state.player.follower_link.has_auxiliary_state()
        {
            return;
        }
        let j = self
            .game_state
            .player
            .follower_link
            .conveyor_belt_state()
            .wrapping_sub(1) as usize;
        if j >= LINK_APPLY_CONVEYOR_MOVE_POS_DIR_FLAG.len() {
            return;
        }
        if self.game_state.player.follower_link.is_running()
            && self.game_state.player.follower_link.dash_counter() == 32
            && self.game_state.player.follower_link.direction()
                & LINK_APPLY_CONVEYOR_MOVE_POS_DIR_FLAG[j]
                != 0
        {
            return;
        }
        self.follower_link_state_mut()
            .clear_orthogonal_direction_count();
        self.follower_link_state_mut()
            .add_direction_flags(LINK_APPLY_CONVEYOR_MOVE_POS_DIR_FLAG[j]);
        let y = ((self.game_state.player.follower_link.y() as u32) << 8
            | self.game_state.player.bg1_movement_accumulator.y_subpixel() as u32)
            .wrapping_add(((LINK_APPLY_CONVEYOR_MOVING_BELT_Y[j] as i32) << 4) as u32);
        self.bg1_move_calc_mut().set_y_subpixel(y as u8);
        self.follower_link_state_mut().set_y((y >> 8) as u16);
        let x = ((self.game_state.player.follower_link.x() as u32) << 8
            | self.game_state.player.bg1_movement_accumulator.x_subpixel() as u32)
            .wrapping_add(((LINK_APPLY_CONVEYOR_MOVING_BELT_X[j] as i32) << 4) as u32);
        self.bg1_move_calc_mut().set_x_subpixel(x as u8);
        self.follower_link_state_mut().set_x((x >> 8) as u16);
    }

    pub(in crate::zelda_rtl) fn flag67_with_directions(&mut self) {
        self.follower_link_state_mut()
            .derive_direction_from_actual_velocity();
    }

    pub(super) fn link_add_in_velocity_y_falling(&mut self) {
        let adjust = i16::from(self.game_state.player.tile_detection.y_low() & 7)
            - if self
                .game_state
                .player
                .follower_link
                .y_velocity_signed()
                .is_negative()
            {
                8
            } else {
                0
            };
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_sub(adjust as u16);
        self.follower_link_state_mut().set_y(y);
    }

    pub(super) fn link_add_in_velocity_y(&mut self) {
        let y =
            self.game_state.player.follower_link.y().wrapping_sub(
                self.game_state.player.follower_link.y_velocity() as i8 as i16 as u16,
            );
        self.follower_link_state_mut().set_y(y);
    }

    pub(super) fn player_change_z(&mut self, z_delta: u8) {
        if (self.game_state.player.follower_link.actual_z_velocity() as i8).is_negative() {
            if self.game_state.player.follower_link.z_low() == 0 {
                return;
            }
            if (self.game_state.player.follower_link.z_low() as i8).is_negative() {
                self.follower_link_state_mut().set_z(0xffff);
                self.follower_link_state_mut().set_actual_z_velocity(0xff);
                return;
            }
        }
        self.follower_link_state_mut()
            .decrement_actual_z_velocity(z_delta);
    }

    pub(in crate::zelda_rtl) fn link_move_position(&mut self) {
        if let Some(position_return) = self.link_move_position_until_coordinates_integrated() {
            self.complete_link_move_position_after_coordinates(position_return);
        }
    }

    fn link_move_position_until_coordinates_integrated(
        &mut self,
    ) -> Option<LinkMovePositionReturn> {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x07_e373));
        if self.lag_free {
            // $07:E373..E3DB: save coordinates, axis loop and child call
            // instructions only. Floor/conveyor and E3E0 own their bodies.
            let p = &self.game_state.player.follower_link;
            let mut clocks = 344;
            if p.handler_state() == 10 { clocks += 6; }
            else {
                clocks += 64;
                if p.on_somaria_platform() == 2 {
                    crate::cycle_ledger::charge(clocks + 6);
                    let x = p.x();
                    let y = p.y();
                    self.follower_link_state_mut().store_safe_return_position(x, y);
                    self.link_handle_velocity_and_sand_drag(x, y);
                    return None;
                }
            }
            clocks += 72 + if p.auxiliary_state() != 0 { 6 } else { 32 };
            let velocities = [p.actual_x_velocity(), p.actual_y_velocity(), p.actual_z_velocity()];
            let count = if p.auxiliary_state() != 0 { 3 } else { 2 };
            for (i, velocity) in velocities[..count].iter().enumerate() {
                clocks += 352 + if velocity & 0x80 == 0 { 6 } else { 32 };
                clocks += 248 + if i + 1 < count { 6 } else { 0 };
            }
            crate::cycle_ledger::charge(clocks + 114);
        }
        let x = self.game_state.player.follower_link.x();
        let y = self.game_state.player.follower_link.y();
        self.follower_link_state_mut()
            .store_safe_return_position(x, y);

        if self.game_state.player.follower_link.handler_state() != 10
            && self.game_state.player.follower_link.on_somaria_platform() == 2
        {
            self.link_handle_velocity_and_sand_drag(x, y);
            return None;
        }

        let actual_x_velocity = self.game_state.player.follower_link.actual_x_velocity();
        let actual_y_velocity = self.game_state.player.follower_link.actual_y_velocity();
        let airborne = self.game_state.player.follower_link.auxiliary_state() != 0;
        let actual_z_velocity = self.game_state.player.follower_link.actual_z_velocity();
        // No external observer runs between these stores. Observe legacy writes
        // once and keep the ordinary X/Y/Z publication order within this borrow.
        let mut player = self.follower_link_state_mut();
        player.move_x_by_velocity(actual_x_velocity);
        player.move_y_by_velocity(actual_y_velocity);
        if airborne {
            player.move_z_by_velocity(actual_z_velocity);
        }

        Some(LinkMovePositionReturn { old_x: x, old_y: y })
    }

    pub(in crate::zelda_rtl) fn complete_link_move_position_after_coordinates(
        &mut self,
        position_return: LinkMovePositionReturn,
    ) {
        self.link_handle_moving_floor();
        self.link_apply_conveyor();
        self.link_handle_velocity_and_sand_drag(position_return.old_x, position_return.old_y);
    }

    pub(super) fn link_handle_velocity_and_sand_drag(&mut self, old_x: u16, old_y: u16) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x07_e3e0));
        if self.lag_free {
            // $07:E3E0..E405: drag, coordinate deltas and common return.
            crate::cycle_ledger::charge(546);
        }
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(self.game_state.player.follower_link.drag_player_y());
        let x = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_add(self.game_state.player.follower_link.drag_player_x());
        let mut player = self.follower_link_state_mut();
        player.set_y(y);
        player.set_x(x);
        player.set_movement_velocity_from_position_delta(x, y, old_x, old_y);
    }

    // $07:E595..E5CB, with early RTS at E5E3. Successful paths enter
    // E5CD, whose shared body/return is charged by its native helper.
    fn link_moving_floor_master(&self) -> u64 {
        let p = &self.game_state.player.follower_link;
        let mut clocks = 48;
        if self.game_state.dungeon.room_load.header_collision() == 0 { return clocks + 6 + 42; }
        clocks += 40;
        if p.z_low() == 0 { clocks += 6; }
        else {
            clocks += 32;
            if p.z_low() != 0xff { return clocks + 6 + 42; }
        }
        clocks += 80;
        if !self.has_player_layer_collision(crate::game_state::constants::player::LAYER_COLLISION_BOTH) {
            return clocks + 6 + 42;
        }
        clocks += 56;
        if p.handler_state() == 19 { return clocks + 6 + 42; }
        for velocity in [self.game_state.dungeon.moving_floor.floor_y_velocity(),
            self.game_state.dungeon.moving_floor.floor_x_velocity()] {
            clocks += 64;
            let low = velocity as u8;
            clocks += if low == 0 { 6 } else { 16 + if (low as i8) < 0 { 6 } else { 16 } + 52 };
        }
        clocks
    }

    // $07:E5F0..E670 and the shared early RTS at E5E3. Conveyor table
    // accesses use the actual belt index; the two fixed axis passes are
    // instruction loops, not a duration estimate.
    fn link_conveyor_master(&self) -> u64 {
        let p = &self.game_state.player.follower_link;
        let mut clocks = 48;
        if p.conveyor_belt_state() == 0 { return clocks + 6 + 42; }
        clocks += 40;
        if p.z_low() == 0 { clocks += 6; }
        else {
            clocks += 32;
            if p.z_low() != 0xff { return clocks + 6 + 42; }
        }
        clocks += 64;
        if p.grabbing_wall_has(1) { return clocks + 6 + 42; }
        clocks += 56;
        if p.handler_state() == 19 { return clocks + 6 + 42; }
        clocks += 40;
        if p.has_auxiliary_state() { return clocks + 6 + 42; }
        clocks += 48;
        let j = p.conveyor_belt_state().wrapping_sub(1) as usize;
        // Native rejects invalid table indices rather than emulating foreign
        // ROM data. Keep that unsupported path unpriced beyond its guards.
        if j >= LINK_APPLY_CONVEYOR_MOVE_POS_DIR_FLAG.len() { return clocks; }
        if !p.is_running() { clocks += 6; }
        else {
            clocks += 64;
            if p.dash_counter() != 32 { clocks += 6; }
            else {
                clocks += 118;
                if p.direction() & LINK_APPLY_CONVEYOR_MOVE_POS_DIR_FLAG[j] != 0 {
                    return clocks + 6 + 42;
                }
            }
        }
        clocks += 284;
        for (i, velocity) in [LINK_APPLY_CONVEYOR_MOVING_BELT_X[j],
            LINK_APPLY_CONVEYOR_MOVING_BELT_Y[j]].iter().enumerate() {
            clocks += 360 + if *velocity >= 0 { 6 } else { 30 };
            clocks += 240 + if i == 0 { 6 } else { 0 };
        }
        clocks + 64
    }

    pub(super) fn link_handle_moving_floor(&mut self) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x07_e595));
        if self.lag_free { crate::cycle_ledger::charge(self.link_moving_floor_master()); }
        if self.game_state.dungeon.room_load.header_collision() == 0 {
            return;
        }
        let z = self.game_state.player.follower_link.z_low();
        if z != 0 && z != 0xff {
            return;
        }
        if !self
            .has_player_layer_collision(crate::game_state::constants::player::LAYER_COLLISION_BOTH)
        {
            return;
        }
        if self.game_state.player.follower_link.handler_state() == 19 {
            return;
        }

        let floor_y = self.game_state.dungeon.moving_floor.floor_y_velocity();
        let floor_x = self.game_state.dungeon.moving_floor.floor_x_velocity();
        self.follower_link_state_mut()
            .mark_moving_floor_direction(floor_y, floor_x);

        self.link_apply_moving_floor_velocity();
    }

    pub(super) fn check_if_room_needs_double_layer_check(&mut self) -> bool {
        if self.game_state.dungeon.room_load.header_collision() == 0
            || self.game_state.dungeon.room_load.header_collision() == 4
        {
            return false;
        }

        if self.game_state.dungeon.room_load.header_collision() >= 2 {
            let y = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_add(self.game_state.display.ppu_scroll_copy.bg1_v_copy2())
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
            self.follower_link_state_mut().set_y(y);

            let x = self
                .game_state
                .player
                .follower_link
                .x()
                .wrapping_add(self.game_state.display.ppu_scroll_copy.bg1_h_copy2())
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
            self.follower_link_state_mut().set_x(x);
            self.follower_link_state_mut()
                .cache_moving_floor_position(x, y);
        }
        self.follower_link_state_mut().mark_lower_level();
        true
    }

    pub(super) fn create_velocity_from_moving_background(&mut self) {
        self.create_velocity_from_moving_background_body(crate::cycle_ledger::blocks(false));
    }

    pub(super) fn create_velocity_from_moving_background_marked(&mut self) {
        // $07:B9B3..B9F6; full body, excluding no callees. Partial resumes
        // retain the raw entry because their source cursor already owns work.
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x07b9b3));
        self.create_velocity_from_moving_background_body(crate::cycle_ledger::blocks(self.lag_free));
    }

    fn create_velocity_from_moving_background_body(&mut self, blocks: crate::cycle_ledger::BlockMarkers) {
        blocks.block(0x07b9b3);
        if self.game_state.dungeon.room_load.header_collision() != 1 {
            blocks.block(0x07b9ba);
            let x = self
                .game_state
                .player
                .follower_link
                .x()
                .wrapping_sub(self.game_state.player.follower_link.moving_floor_x());
            let y = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_sub(self.game_state.player.follower_link.moving_floor_y());
            let new_y = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_add(self.game_state.display.ppu_scroll_copy.bg2_v_copy2())
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg1_v_copy2());
            let new_x = self
                .game_state
                .player
                .follower_link
                .x()
                .wrapping_add(self.game_state.display.ppu_scroll_copy.bg2_h_copy2())
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg1_h_copy2());
            self.follower_link_state_mut().set_y(new_y);
            self.follower_link_state_mut().set_x(new_x);
            if self.game_state.player.follower_link.direction() != 0 {
                blocks.block(0x07b9e6);
                self.follower_link_state_mut()
                    .add_movement_velocity_delta(x, y);
            } else { blocks.taken(0x07b9e4); }
        } else { blocks.taken(0x07b9b8); }
        blocks.block(0x07b9f4);
        self.follower_link_state_mut().clear_lower_level();
    }

    pub(super) fn calculate_snap_scratch_y(&mut self) {
        let mut y_vel = self.game_state.player.follower_link.y_velocity() as i8;
        if self.game_state.player.tile_detection.collision_bits() & 4 != 0 {
            if y_vel >= 0 {
                y_vel = y_vel.wrapping_neg();
            }
        } else if y_vel < 0 {
            y_vel = y_vel.wrapping_neg();
        }

        let x = self.game_state.player.follower_link.x();
        let delta = if y_vel < 0 { -1i16 } else { 1i16 };
        self.follower_link_state_mut()
            .set_x(x.wrapping_add(delta as u16));
    }

    pub(super) fn change_axis_of_perpendicular_door_movement_y(&mut self) {
        self.follower_link_state_mut().set_direction_lock_bits(2);
        let r14 = self.game_state.player.tile_detection.collision_bits();
        let t = ((r14 | (r14 >> 4)) & 0x0f) as u8;
        if t & 7 == 0 {
            self.follower_link_state_mut().clear_doorway_state();
            return;
        }

        let mut t = self.game_state.player.follower_link.y_velocity();
        let dir = if self.game_state.player.follower_link.x_low() >= 0x80 {
            if t & 0x80 == 0 {
                t = t.wrapping_neg();
            }
            4
        } else {
            if t & 0x80 != 0 {
                t = t.wrapping_neg();
            }
            6
        };
        let vel = if t & 0x80 != 0 { -1i16 } else { 1i16 };
        if !self.game_state.player.follower_link.direction_lock_has(1) {
            self.follower_link_state_mut().set_facing(dir);
        }
        let x = self.game_state.player.follower_link.x();
        self.follower_link_state_mut()
            .set_x(x.wrapping_add(vel as u16));
    }

    pub(super) fn snap_on_x(&mut self) {
        let x = self.game_state.player.follower_link.x();
        let adjust = (x & 7).wrapping_sub(
            if (self.game_state.player.follower_link.x_velocity() as i8).is_negative() {
                8
            } else {
                0
            },
        );
        self.follower_link_state_mut().set_x(x.wrapping_sub(adjust));
    }

    pub(super) fn calculate_snap_scratch_x(&mut self) {
        let mut x_vel = self.game_state.player.follower_link.x_velocity() as i8;
        if self.game_state.player.tile_detection.collision_bits() & 4 != 0 {
            if x_vel >= 0 {
                x_vel = x_vel.wrapping_neg();
            }
        } else if x_vel < 0 {
            x_vel = x_vel.wrapping_neg();
        }

        let y = self.game_state.player.follower_link.y();
        let delta = if x_vel < 0 { -1i16 } else { 1i16 };
        self.follower_link_state_mut()
            .set_y(y.wrapping_add(delta as u16));
    }

    pub(super) fn change_axis_of_perpendicular_door_movement_x(&mut self) -> i8 {
        self.follower_link_state_mut().set_direction_lock_bits(2);
        let r14 = self.game_state.player.tile_detection.collision_bits();
        let r0 = ((r14 | (r14 >> 4)) & 0x0f) as u8;
        if r0 & 7 == 0 {
            self.follower_link_state_mut().clear_doorway_state();
            return r0 as i8;
        }

        let mut x_vel = self.game_state.player.follower_link.x_velocity_signed();
        let dir = if self.game_state.player.follower_link.y_low() >= 0x80 {
            if x_vel >= 0 {
                x_vel = x_vel.wrapping_neg();
            }
            0
        } else {
            if x_vel < 0 {
                x_vel = x_vel.wrapping_neg();
            }
            2
        };
        if !self.game_state.player.follower_link.direction_lock_has(1) {
            self.follower_link_state_mut().set_facing(dir);
        }
        let y = self.game_state.player.follower_link.y();
        self.follower_link_state_mut()
            .set_y(y.wrapping_add(x_vel as i16 as u16));
        x_vel
    }

    pub(super) fn tile_behavior_handle_item_and_execute(&mut self, x: u16, y: u16) {
        let tile = self.handle_item_tile_action_overworld(x, y);
        self.tile_detect_execute_definition(tile, 0, 1, false);
    }

    pub(in crate::zelda_rtl) fn push_block_target_tile(&self, x: u16, y: u16) -> NativeTile {
        let offset = ((y & !7) as usize) * 8
            + (x & 0x3f) as usize
            + if self.game_state.player.follower_link.is_on_lower_level() {
                0x1000
            } else {
                0
            };
        self.game_state.dungeon.bg2_attributes.bg2_tile(offset)
    }

    pub(super) fn link_handle_change_in_z_velocity(&mut self) {
        self.player_change_z(
            if self.game_state.player.follower_link.handler_state() == 19 {
                1
            } else {
                2
            },
        );
    }

    pub(in crate::zelda_rtl) fn run_slope_collision_checks(&mut self, order: CollisionOrder) {
        for axis in order.axes() {
            // The first axis may change these flags: read again for the second.
            if !axis.blocked_by_slope(
                self.game_state
                    .player
                    .follower_link
                    .moving_against_diag_tile(),
            ) {
                self.start_movement_collision_checks(axis);
            }
        }
    }

    pub(super) fn link_hop_in_or_out_of_water_y(&mut self) {
        let ts = if self.game_state.world.location.is_outdoors() {
            2
        } else if self
            .game_state
            .player
            .follower_link
            .about_to_jump_off_ledge()
            != 0
        {
            0
        } else {
            self.game_state.display.sub_screen_layers
        };

        let mut vel = LINK_HOP_IN_OR_OUT_OF_WATER_Y_RECOIL_VEL_Y[ts as usize];
        if self
            .game_state
            .player
            .follower_link
            .last_direction_moved_towards()
            == 0
        {
            vel = 0u8.wrapping_sub(vel);
        }

        self.follower_link_state_mut()
            .set_actual_velocity_xy(0, vel);
        self.follower_link_state_mut()
            .set_actual_z_velocity_and_copy(
                LINK_HOP_IN_OR_OUT_OF_WATER_Y_RECOIL_VEL_Z[ts as usize],
            );
        self.follower_link_state_mut().set_z(0);
        self.follower_link_state_mut().set_incapacitated_timer(16);
        self.follower_link_state_mut().enter_water_hop_state();
    }

    pub(super) fn link_hop_in_or_out_of_water_x(&mut self) {
        let ts = if self.game_state.world.location.is_outdoors() {
            2
        } else if self
            .game_state
            .player
            .follower_link
            .about_to_jump_off_ledge()
            != 0
        {
            0
        } else {
            self.game_state.display.sub_screen_layers
        };

        let mut vel = LINK_HOP_IN_OR_OUT_OF_WATER_X_RECOIL_VEL_X[ts as usize];
        if self
            .game_state
            .player
            .follower_link
            .last_direction_moved_towards()
            & 1
            == 0
        {
            vel = 0u8.wrapping_sub(vel);
        }
        self.follower_link_state_mut()
            .set_actual_velocity_xy(vel, 0);
        self.follower_link_state_mut()
            .set_actual_z_velocity_and_copy(
                LINK_HOP_IN_OR_OUT_OF_WATER_X_RECOIL_VEL_Z[ts as usize],
            );
        self.follower_link_state_mut().set_incapacitated_timer(16);
        self.follower_link_state_mut().enter_water_hop_state();
    }

    pub(in crate::zelda_rtl) fn run_ledge_hop_timer(&mut self) -> bool {
        let mut rv = false;
        if !self
            .game_state
            .player
            .follower_link
            .is_in_auxiliary_state(1)
        {
            if !self.game_state.player.follower_link.is_running() {
                if self
                    .follower_link_state_mut()
                    .tick_jump_ledge_timer_or_reset()
                {
                    return true;
                }
            } else {
                rv = true;
            }
        }
        self.follower_link_state_mut()
            .restore_position_from_previous();
        self.follower_link_state_mut().clear_movement_subpixels();
        rv
    }

    /// Ledge-hop sound. C plays this via `Ancilla_Sfx2_Near(0x20)`, which routes through
    /// `PlaySfx_SetPanFrom`: it stamps RAW_SFX_PAN_VALUE (0xcf8) = 0x20 AND sets
    /// SOUND_EFFECT_1 = `0x20 | pan`. `ancilla_sfx2_near` mirrors that exactly.
    pub(super) fn link_ledge_hop_sfx(&mut self) {
        self.ancilla_sfx2_near(0x20);
    }

    pub(super) fn flag_moving_into_slopes_y(&mut self) {
        let diag_state = self.game_state.player.tile_detection.diag_state() as usize;
        let x = self.game_state.player.follower_link.x().wrapping_sub(
            if self.game_state.player.tile_detection.slope_collision_bits() & 4 != 0 {
                1
            } else {
                0
            },
        );
        let o = diag_state * 4 + (x & 7) as usize;
        let mut y = (self.game_state.player.tile_detection.y_low() & 7) as i8;

        if self.game_state.player.tile_detection.diagonal_tile() & 5 != 0 {
            let mut ym = (self.game_state.player.tile_detection.y_low() & 7) as i8;
            if self.game_state.enhanced_features.has(4096) {
                if diag_state & 2 != 0 {
                    ym = ym.wrapping_neg();
                } else {
                    ym = FLAG_MOVING_INTO_SLOPES_Y_AVOID_JUDDER[o].wrapping_sub(8 - ym);
                }
            } else {
                if diag_state & 2 == 0 {
                    ym = 8 - ym;
                } else {
                    ym += 8;
                }
                ym = FLAG_MOVING_INTO_SLOPES_Y_AVOID_JUDDER[o].wrapping_sub(ym);
            }

            let y_velocity = self.game_state.player.follower_link.y_velocity_signed();
            if y_velocity == 0 {
                return;
            }
            if y_velocity.is_negative() {
                ym = ym.wrapping_neg();
            }
            y = ym;
        } else {
            y = FLAG_MOVING_INTO_SLOPES_Y_AVOID_JUDDER[o].wrapping_sub(y);
        }

        if self
            .game_state
            .player
            .follower_link
            .y_velocity_signed()
            .is_negative()
        {
            if y <= 0 {
                return;
            }
            let coord = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_add(y as i16 as u16);
            self.follower_link_state_mut().set_y(coord);
            self.follower_link_state_mut()
                .set_moving_against_diag_tile(8);
        } else {
            if y >= 0 {
                return;
            }
            let coord = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_add(y as i16 as u16);
            self.follower_link_state_mut().set_y(coord);
            self.follower_link_state_mut()
                .set_moving_against_diag_tile(4);
        }

        let diag_flags = if self.game_state.player.tile_detection.slope_collision_bits() & 4 != 0 {
            0x12
        } else {
            0x11
        };
        self.follower_link_state_mut()
            .add_moving_against_diag_tile_flags(diag_flags);
    }

    pub(super) fn flag_moving_into_slopes_x(&mut self) {
        let diag_state = self.game_state.player.tile_detection.diag_state();
        let mut x = (self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_sub(if diag_state == 6 { 1 } else { 0 })
            & 7) as i8;
        let which_y_offset =
            if self.game_state.player.tile_detection.slope_collision_bits() & 4 != 0 {
                2
            } else {
                0
            };
        let o = diag_state as usize * 4
            + (self
                .game_state
                .player
                .tile_detection
                .y_low_at(which_y_offset)
                & 7) as usize;

        if self.game_state.player.tile_detection.diagonal_tile() & 5 != 0 {
            let mut xm = (self.game_state.player.follower_link.x() & 7) as i8;
            if diag_state != 4 && diag_state != 6 {
                xm = xm.wrapping_neg();
            } else {
                xm = FLAG_MOVING_INTO_SLOPES_X_AVOID_JUDDER[o].wrapping_sub(8 - xm);
            }
            let x_velocity = self.game_state.player.follower_link.x_velocity_signed();
            if x_velocity == 0 {
                return;
            }
            if x_velocity.is_negative() {
                xm = xm.wrapping_neg();
            }
            x = xm;
        } else {
            x = FLAG_MOVING_INTO_SLOPES_X_AVOID_JUDDER[o].wrapping_sub(x);
        }

        if self
            .game_state
            .player
            .follower_link
            .x_velocity_signed()
            .is_negative()
        {
            if x <= 0 {
                return;
            }
            let coord = self
                .game_state
                .player
                .follower_link
                .x()
                .wrapping_add(x as i16 as u16);
            self.follower_link_state_mut().set_x(coord);
            self.follower_link_state_mut()
                .set_moving_against_diag_tile(2);
        } else {
            if x >= 0 {
                return;
            }
            let coord = self
                .game_state
                .player
                .follower_link
                .x()
                .wrapping_add(x as i16 as u16);
            self.follower_link_state_mut().set_x(coord);
            self.follower_link_state_mut()
                .set_moving_against_diag_tile(1);
        }

        self.follower_link_state_mut()
            .add_moving_against_diag_tile_flags(if diag_state & 2 != 0 { 0x28 } else { 0x24 });
    }

    pub(in crate::zelda_rtl) fn player_something_with_velocity_tired_or_swim(&mut self, xvel: u16, yvel: u16) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x07_e4d3));
        if self.lag_free {
            // $07:E4D3..E53E: two signed subpixel passes. E5CD and E3E0
            // are separate children; include only their call/BRL here.
            let mut clocks = 342;
            for (i, velocity) in [xvel, yvel].iter().enumerate() {
                let high = (velocity >> 8) as u8;
                clocks += 236 + if high < 8 { 6 } else { 32 };
                clocks += 352 + if high < 0x80 { 6 } else { 30 };
                clocks += 182 + if i == 0 { 6 } else { 0 };
            }
            clocks += 64 + if self.game_state.dungeon.room_load.header_collision() == 4 { 46 } else { 6 };
            crate::cycle_ledger::charge(clocks + 78);
        }
        let old_x = self.game_state.player.follower_link.x();
        let old_y = self.game_state.player.follower_link.y();
        self.follower_link_state_mut()
            .store_safe_return_position(old_x, old_y);

        self.follower_link_state_mut()
            .move_x_by_subpixel_delta(xvel);
        let u = (xvel >> 8) as u8;
        let actual_x_velocity = (if (u as i8).is_negative() {
            0u8.wrapping_sub(u)
        } else {
            u
        } << 4)
            | ((xvel as u8) >> 4);

        self.follower_link_state_mut()
            .move_y_by_subpixel_delta(yvel);
        let u = (yvel >> 8) as u8;
        let actual_y_velocity = (if (u as i8).is_negative() {
            0u8.wrapping_sub(u)
        } else {
            u
        } << 4)
            | ((yvel as u8) >> 4);
        self.follower_link_state_mut()
            .set_actual_velocity_xy(actual_x_velocity, actual_y_velocity);

        if self.game_state.dungeon.room_load.header_collision() == 4 {
            self.link_apply_moving_floor_velocity();
        }
        self.follower_link_state_mut().clear_page_movement_deltas();
        self.link_handle_velocity_and_sand_drag(old_x, old_y);
    }

    pub(in crate::zelda_rtl) fn link_check_for_edge_screen_transition(&mut self) -> bool {
        let _scope = crate::cycle_ledger::routine(0x07_f439);
        if !self
            .game_state
            .player
            .follower_link
            .is_edge_transition_blocked_by_handler_state()
            && self.game_state.player.follower_link.incapacitated_timer() == 0
        {
            return false;
        }
        self.follower_link_state_mut().clear_actual_velocity_xy();
        self.follower_link_state_mut().set_recoil_timer(3);
        self.follower_link_state_mut()
            .restore_position_from_previous();
        true
    }

    pub(super) fn player_limit_directions_inner(&mut self) {
        // $07:B660..B666: shared JSR entry or diagonal parent's fallthrough.
        crate::cycle_ledger::blocks(self.lag_free).block(0x07b660);
        self.follower_link_state_mut().reset_direction_limits();
        self.player_limit_directions_after_reset(None);
    }

    pub(super) fn player_limit_directions_after_reset(&mut self, native_vertical_probe: Option<CollisionDirection>) {
        let blocks = crate::cycle_ledger::blocks(self.lag_free);
        blocks.block(0x07b668);
        let direction = self.game_state.player.follower_link.direction();
        if direction & 0x0c != 0 {
            blocks.taken(0x07b66c);
            blocks.block(0x07b671);
            if direction & 8 != 0 { blocks.taken(0x07b677); }
            else { blocks.block(0x07b679); }
            blocks.block(0x07b67b);
            let probe_direction = native_vertical_probe.unwrap_or_else(|| {
                self.follower_link_state_mut().increment_orthogonal_direction_count();
                let probe_direction = CollisionDirection::along(CollisionAxis::Vertical, direction & 8 != 0);
                self.follower_link_state_mut().set_last_direction_moved_towards(probe_direction.legacy_value());
                probe_direction
            });
            if std::mem::take(&mut self.early_vertical_slope_probe) {
                assert!(native_vertical_probe.is_some());
                let y = read_le_u16(&self.ram, 0);
                let low_x = read_le_u16(&self.ram, 2);
                let high_x = read_le_u16(&self.ram, 4);
                use crate::cycle_models::tile_classifier::TileClassifierTail;
                match self.early_tile_classifier_tail.take() {
                    Some(TileClassifierTail::Complete) => {}
                    Some(TileClassifierTail::Effect {tile, offset}) => self.tile_detect_execute_definition(
                        crate::tile_definition::NativeTile::from_cartridge(tile), offset, 1, true),
                    Some(TileClassifierTail::OutdoorAttribute) | None => self.tile_detection_execute(low_x, y, 1),
                }
                match self.early_high_tile_classifier_tail.take() {
                    Some(TileClassifierTail::Complete) => {}
                    Some(TileClassifierTail::Effect {tile, offset}) => self.tile_detect_execute_definition(
                        crate::tile_definition::NativeTile::from_cartridge(tile), offset, 2, true),
                    Some(TileClassifierTail::OutdoorAttribute) | None => self.tile_detection_execute(high_x, y, 2),
                }
            } else {
            self.detect_player_movement(
                CollisionAxis::Vertical,
                probe_direction,
                MovementProbeKind::Slope,
            );
            }

            self.limit_direction_response_markers(true);
            let r14 = self.game_state.player.tile_detection.collision_bits();
            if r14 & 0x30 != 0
                && self.game_state.player.tile_detection.door_direction_flags() as u8 & 2 == 0
                && ((((r14 & 0x30) >> 4) as u8) & self.game_state.player.follower_link.direction())
                    == 0
                && self.game_state.player.follower_link.direction() & 3 != 0
            {
                let mask = PLAYER_LIMIT_DIRECTIONS_INNER_MASKS[if self
                    .game_state
                    .player
                    .follower_link
                    .direction()
                    & 2
                    != 0
                {
                    2
                } else {
                    3
                }];
                self.follower_link_state_mut().set_direction_mask_a(mask);
            } else {
                let mut set_thingy = false;
                if self.game_state.dungeon.room_load.header_collision() == 0
                    && self.game_state.player.follower_link.has_auxiliary_state()
                    && self.game_state.player.tile_detection.slope_collision_bits() & 3 != 0
                {
                    set_thingy = true;
                }

                if r14 & 3 != 0 {
                    self.follower_link_state_mut()
                        .clear_moving_against_diag_tile();
                    if self.game_state.player.follower_link.flag_moving() != 0
                        && self.game_state.player.tile_detection.spike_cactus_tiles() & 3 == 0
                        && self.game_state.player.follower_link.direction() & 3 != 0
                    {
                        self.swim_acceleration_mut().set_speed_active_flag(0, 0);
                        self.swim_acceleration_mut().set_mode(0, 0);
                        self.swim_acceleration_mut().set_acceleration(0, 0);
                        self.swim_acceleration_mut().set_max_speed(0, 0);
                    }
                    set_thingy = true;
                }

                if set_thingy {
                    self.follower_link_state_mut().set_pit_correction_active();
                    let mask = PLAYER_LIMIT_DIRECTIONS_INNER_MASKS[self
                        .game_state
                        .player
                        .follower_link
                        .last_direction_moved_towards()
                        as usize];
                    self.follower_link_state_mut().set_direction_mask_a(mask);
                }
            }
        } else { blocks.block(0x07b66e); }

        self.player_limit_directions_after_vertical(None);
    }

    pub(super) fn player_limit_directions_after_vertical(&mut self, native_horizontal_probe: Option<CollisionDirection>) {
        let blocks = crate::cycle_ledger::blocks(self.lag_free);
        let direction = self.game_state.player.follower_link.direction();
        if direction & 12 != 0 {
            blocks.block(0x07b701);
            if direction & 3 != 0 { blocks.taken(0x07b705); }
            else { blocks.block(0x07b707); }
        }
        if native_horizontal_probe.is_some() || (direction & 0x0c != 0 && direction & 3 != 0) {
            blocks.block(0x07b70a);
            if direction & 2 != 0 { blocks.taken(0x07b710); }
            else { blocks.block(0x07b712); }
            blocks.block(0x07b714);
            let probe_direction = native_horizontal_probe.unwrap_or_else(|| {
                self.follower_link_state_mut().increment_orthogonal_direction_count();
                let direction = CollisionDirection::along(CollisionAxis::Horizontal, direction & 2 != 0);
                self.follower_link_state_mut().set_last_direction_moved_towards(direction.legacy_value());
                direction
            });
            self.detect_player_movement(
                CollisionAxis::Horizontal,
                probe_direction,
                MovementProbeKind::Slope,
            );

            self.limit_direction_response_markers(false);
            let r14 = self.game_state.player.tile_detection.collision_bits();
            if r14 & 0x30 != 0
                && self.game_state.player.tile_detection.door_direction_flags() as u8 & 2 != 0
                && ((((r14 & 0x30) >> 2) as u8) & self.game_state.player.follower_link.direction())
                    == 0
                && self.game_state.player.follower_link.direction() & 0x0c != 0
            {
                let mask = PLAYER_LIMIT_DIRECTIONS_INNER_MASKS[if self
                    .game_state
                    .player
                    .follower_link
                    .direction()
                    & 8
                    != 0
                {
                    0
                } else {
                    1
                }];
                self.follower_link_state_mut().set_direction_mask_b(mask);
            } else {
                let mut set_thingy_b = false;
                if self.game_state.dungeon.room_load.header_collision() == 0
                    && self.game_state.player.follower_link.has_auxiliary_state()
                    && self.game_state.player.tile_detection.slope_collision_bits() & 3 != 0
                {
                    set_thingy_b = true;
                }

                if r14 & 3 != 0 {
                    self.follower_link_state_mut()
                        .clear_moving_against_diag_tile();
                    if self.game_state.player.follower_link.flag_moving() != 0
                        && self.game_state.player.tile_detection.spike_cactus_tiles() & 3 == 0
                        && self.game_state.player.follower_link.direction() & 0x0c != 0
                    {
                        self.swim_acceleration_mut().set_speed_active_flag(2, 0);
                        self.swim_acceleration_mut().set_mode(2, 0);
                        self.swim_acceleration_mut().set_acceleration(2, 0);
                        self.swim_acceleration_mut().set_max_speed(2, 0);
                    }
                    set_thingy_b = true;
                }

                if set_thingy_b {
                    self.follower_link_state_mut().set_pit_correction_active();
                    let mask = PLAYER_LIMIT_DIRECTIONS_INNER_MASKS[self
                        .game_state
                        .player
                        .follower_link
                        .last_direction_moved_towards()
                        as usize];
                    self.follower_link_state_mut().set_direction_mask_b(mask);
                }
            }

            blocks.block(0x07b798);
            self.follower_link_state_mut().apply_direction_masks();
        }

        // $07:B7A0..B7C2: selected direction and orthogonal count return.
        blocks.block(0x07b7a0);
        if self.game_state.player.follower_link.direction() & 15 == 0 {
            blocks.taken(0x07b7a4);
        } else {
            blocks.block(0x07b7a6);
            if self.game_state.player.follower_link.moving_against_diag_tile() & 15 == 0 {
                blocks.taken(0x07b7aa);
            } else { blocks.block(0x07b7ac); }
        }
        blocks.block(0x07b7ae);
        if self.game_state.player.follower_link.num_orthogonal_directions() != 2 {
            blocks.taken(0x07b7b4);
        } else {
            blocks.block(0x07b7b6);
            if self.game_state.player.follower_link.facing() & 4 == 0 {
                blocks.taken(0x07b7bc);
            } else { blocks.block(0x07b7be); }
            blocks.block(0x07b7c0);
        }
        blocks.block(0x07b7c2);
        self.follower_link_state_mut()
            .force_direction_from_diag_tile_if_needed();
        self.follower_link_state_mut()
            .resolve_orthogonal_direction_count_from_facing();
    }

    fn limit_direction_response_markers(&self, vertical: bool) {
        // $07:B682..B700 / B71B..B797: source branches after each real probe.
        // Each tuple names corresponding ROM blocks/branches, not prices.
        if !self.lag_free { return; }
        let b = crate::cycle_ledger::blocks(true);
        let pcs = if vertical {
            [0x07b682,0x07b686,0x07b688,0x07b68c,0x07b68e,0x07b698,
             0x07b69a,0x07b6a0,0x07b6a2,0x07b6a4,0x07b6a6,
             0x07b6aa,0x07b6ad,0x07b6af,0x07b6b3,0x07b6b5,
             0x07b6b7,0x07b6b9,0x07b6bb,0x07b6bf,0x07b6c1,
             0x07b6c3,0x07b6c7,0x07b6c9,0x07b6ce,0x07b6d0,0x07b6d5,
             0x07b6d7,0x07b6db,0x07b6dd,0x07b6f5,0x07b6fc]
        } else {
            [0x07b71b,0x07b71f,0x07b721,0x07b725,0x07b727,0x07b72f,
             0x07b731,0x07b737,0x07b739,0x07b73b,0x07b73d,
             0x07b741,0x07b744,0x07b746,0x07b74a,0x07b74c,
             0x07b74e,0x07b750,0x07b752,0x07b756,0x07b758,
             0x07b75a,0x07b75e,0x07b760,0x07b765,0x07b767,0x07b76c,
             0x07b76e,0x07b772,0x07b774,0x07b78c,0x07b793]
        };
        let bits = self.game_state.player.tile_detection.collision_bits() as u8;
        let direction = self.game_state.player.follower_link.direction();
        let orthogonal = if vertical { 3 } else { 12 };
        b.block(pcs[0]);
        let ordinary = if bits & 0x30 == 0 { b.taken(pcs[1]); true }
        else {
            b.block(pcs[2]);
            let door = self.game_state.player.tile_detection.door_direction_flags() as u8 & 2 != 0;
            if door == vertical { b.taken(pcs[3]); true }
            else {
                b.block(pcs[4]);
                if ((bits & 0x30) >> if vertical { 4 } else { 2 }) & direction != 0 {
                    b.taken(pcs[5]); true
                } else {
                    b.block(pcs[6]);
                    if direction & orthogonal == 0 { b.taken(pcs[7]); true }
                    else {
                        b.block(pcs[8]);
                        if direction & if vertical { 2 } else { 8 } != 0 { b.taken(pcs[9]); }
                        else { b.block(pcs[10]); }
                        b.block(pcs[31]); false
                    }
                }
            }
        };
        if !ordinary { return; }
        b.block(pcs[11]);
        if self.game_state.dungeon.room_load.header_collision() != 0 {
            b.block(pcs[13]);
            if bits & 3 != 0 { b.taken(pcs[14]); }
            else { b.block(pcs[15]); return; }
        } else {
            b.taken(pcs[12]); b.block(pcs[16]);
            let auxiliary = self.game_state.player.follower_link.has_auxiliary_state();
            if !auxiliary { b.taken(pcs[17]); }
            else {
                b.block(pcs[18]);
                if self.game_state.player.tile_detection.slope_collision_bits() & 3 == 0 { b.taken(pcs[19]); }
                else { b.block(pcs[20]); b.block(pcs[30]); b.block(pcs[31]); return; }
            }
        }
        b.block(pcs[21]);
        if bits & 3 == 0 { b.taken(pcs[22]); return; }
        b.block(pcs[23]);
        if self.game_state.player.follower_link.flag_moving() == 0 { b.taken(pcs[24]); }
        else {
            b.block(pcs[25]);
            if self.game_state.player.tile_detection.spike_cactus_tiles() & 3 != 0 { b.taken(pcs[26]); }
            else {
                b.block(pcs[27]);
                if direction & orthogonal == 0 { b.taken(pcs[28]); }
                else { b.block(pcs[29]); }
            }
        }
        b.block(pcs[30]); b.block(pcs[31]);
    }

    pub(in crate::zelda_rtl) fn link_handle_velocity(&mut self) {
        if let Some(position_return) = self.link_handle_velocity_until_position_integrated() {
            self.complete_link_move_position_after_coordinates(position_return);
        }
    }

    pub(in crate::zelda_rtl) fn link_handle_velocity_until_position_integrated(
        &mut self,
    ) -> Option<LinkMovePositionReturn> {
        if !self.link_handle_velocity_before_move_position() {
            return None;
        }
        self.link_move_position_until_coordinates_integrated()
    }

    /// `Link_HandleVelocity` through `Link_MovePosition`'s axis loop, stopping
    /// after the selected axis' subpixel store (the ROM's mid-loop interruption
    /// at route host 179586). `None` when the velocity handler took an early
    /// exit before `Link_MovePosition`.
    pub(in crate::zelda_rtl) fn link_handle_velocity_until_position_partial(
        &mut self,
        axis: PlayerAxis,
    ) -> Option<LinkMovePositionPartialReturn> {
        if !self.link_handle_velocity_before_move_position() {
            return None;
        }
        Some(self.link_move_position_partial_after_subpixel(axis))
    }

    /// `Link_HandleVelocity` through the low coordinate-byte store for
    /// `axis`, retaining the computed high byte and later movement suffix.
    pub(in crate::zelda_rtl) fn link_handle_velocity_until_position_after_coordinate_low(
        &mut self,
        axis: PlayerAxis,
    ) -> Option<LinkMovePositionAfterCoordinateLowReturn> {
        if !self.link_handle_velocity_before_move_position() {
            return None;
        }
        Some(self.link_move_position_after_coordinate_low(axis))
    }

    /// `Link_HandleVelocity` through both coordinate stores for `axis` in
    /// `Link_MovePosition`. `None` when the velocity handler returned before
    /// entering the movement loop.
    pub(in crate::zelda_rtl) fn link_handle_velocity_until_position_after_coordinates(
        &mut self,
        axis: PlayerAxis,
    ) -> Option<LinkMovePositionAfterCoordinatesReturn> {
        if !self.link_handle_velocity_before_move_position() {
            return None;
        }
        Some(self.link_move_position_after_coordinates(axis))
    }

    /// Run speed selection and the source-proven actual-velocity prefix,
    /// retaining whichever components have not yet been published.
    pub(in crate::zelda_rtl) fn link_handle_velocity_until_actual_checkpoint(
        &mut self,
        checkpoint: impl Into<LinkActualVelocityCheckpoint>,
    ) -> Option<LinkActualVelocityReturn> {
        let checkpoint = checkpoint.into();
        let completed_clear_stores = match checkpoint {
            LinkActualVelocityCheckpoint::Clearing { completed } => {
                assert!((1..4).contains(&completed));
                completed
            }
            _ => 4,
        };
        let speed_index = self.link_handle_velocity_before_velocity_clear()?;
        for store in 0..completed_clear_stores {
            self.follower_link_state_mut()
                .clear_velocity_selection_store(store);
        }
        if matches!(
            checkpoint,
            LinkActualVelocityCheckpoint::BeforeSelection
                | LinkActualVelocityCheckpoint::Clearing { .. }
        ) {
            return Some(LinkActualVelocityReturn {
                completed_clear_stores,
                pending_speed_index: Some(speed_index),
                pending_actual_x: None,
                pending_actual_y: None,
            });
        }
        let (direction, velocity) = self.link_handle_velocity_after_velocity_cleared(speed_index);
        let mut pending_actual_x = (direction & 0x03 != 0).then_some(if direction & 0x02 != 0 {
            0u8.wrapping_sub(velocity)
        } else {
            velocity
        });
        if let Some(actual_x) = pending_actual_x.filter(|_| {
            matches!(
                checkpoint,
                LinkActualVelocityCheckpoint::BeforeY | LinkActualVelocityCheckpoint::AfterBoth
            )
        }) {
            self.follower_link_state_mut()
                .set_actual_x_velocity(actual_x);
            pending_actual_x = None;
        }
        let mut pending_actual_y = (direction & 0x0c != 0).then_some(if direction & 0x08 != 0 {
            0u8.wrapping_sub(velocity)
        } else {
            velocity
        });
        if checkpoint == LinkActualVelocityCheckpoint::AfterBoth {
            if let Some(actual_y) = pending_actual_y.take() {
                self.follower_link_state_mut()
                    .set_actual_y_velocity(actual_y);
            }
        }
        Some(LinkActualVelocityReturn {
            completed_clear_stores,
            pending_speed_index: None,
            pending_actual_x,
            pending_actual_y,
        })
    }

    /// Resume pending actual-velocity publication in X-then-Y order, airborne defaults,
    /// and `Link_MovePosition` without replaying speed/modifier selection.
    pub(in crate::zelda_rtl) fn link_handle_velocity_from_actual_checkpoint_until_position_integrated(
        &mut self,
        velocity_return: LinkActualVelocityReturn,
    ) -> Option<LinkMovePositionReturn> {
        for store in velocity_return.completed_clear_stores..4 {
            self.follower_link_state_mut()
                .clear_velocity_selection_store(store);
        }
        if let Some(speed_index) = velocity_return.pending_speed_index {
            let (direction, velocity) =
                self.link_handle_velocity_after_velocity_cleared(speed_index);
            self.follower_link_state_mut()
                .set_actual_velocity_from_direction(direction, velocity);
        }
        if let Some(actual_x) = velocity_return.pending_actual_x {
            self.follower_link_state_mut()
                .set_actual_x_velocity(actual_x);
        }
        if let Some(actual_y) = velocity_return.pending_actual_y {
            self.follower_link_state_mut()
                .set_actual_y_velocity(actual_y);
        }
        self.follower_link_state_mut().prime_airborne_z_velocity();
        self.link_move_position_until_coordinates_integrated()
    }

    /// `Link_HandleVelocity` up to its `Link_MovePosition` call. `false` when
    /// the ROM returned earlier (safe-return/sand-drag, swim, or the
    /// all-blocked collision exit).
    fn link_handle_velocity_before_move_position(&mut self) -> bool {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x07_e245));
        if self.lag_free {
            crate::cycle_ledger::charge(self.link_velocity_selection_master());
        }
        let Some((direction, velocity)) =
            self.link_handle_velocity_before_actual_velocity_resolution()
        else {
            return false;
        };
        self.follower_link_state_mut()
            .set_actual_velocity_from_direction(direction, velocity);
        self.follower_link_state_mut().prime_airborne_z_velocity();
        true
    }

    /// $07:E245..E372, through speed/direction resolution and the branch to
    /// MovePosition. Early BRLs price only their instruction, not the swim,
    /// sand-drag, dash-in-place or common-return tail they enter.
    fn link_velocity_selection_master(&self) -> u64 {
        let p = &self.game_state.player.follower_link;
        let sub = self.game_state.frame.submodule;
        let mut clocks = 128 + if sub == 2 { 56 } else { 6 };
        if sub == 2 && self.game_state.frame.main_module == 14 {
            return clocks + 6 + 318;
        }
        clocks += 48;
        if p.is_prevented_from_moving() { return clocks + 318; }
        clocks += 6 + 56;
        if p.handler_state() == 4 { return clocks + 6 + 30; }
        clocks += 48;
        let mut modifier = p.speed_modifier();
        let mut index;
        if p.flag_moving() != 0 {
            clocks += 48;
            if !p.is_running() { return clocks + 6 + 30; }
            clocks += 62;
            index = 24u8;
        } else {
            clocks += 6 + 48;
            if p.is_running() {
                clocks += 88;
                if p.dash_counter() < 16 { return clocks + 30; }
                clocks += 6;
                modifier = 0;
            } else { clocks += 6; }
            clocks += 96;
            if self.game_state.player.tile_detection.tile_collision_bits_primary()
                | self.game_state.player.tile_detection.tile_collision_bits_secondary() == 0x0f {
                return clocks + 30;
            }
            clocks += 6 + 96;
            index = p.speed_setting();
            if p.water_ripple_or_grass_state() == 0 { clocks += 6; }
            else {
                clocks += 56 + 24;
                if index == 16 { clocks += 38; index = 22; }
                else {
                    clocks += 6 + 72;
                    if index == 12 { clocks += 16; index = 14; }
                    else { clocks += 6; index = 12; }
                }
            }
        }
        // $E2CA..E333: diagonal speed, pit acceleration, deceleration,
        // and the original lookup (including its X8 page crossing).
        let direction = p.direction();
        clocks += 182;
        if direction & 0x0c == 0 { clocks += 6; }
        else {
            clocks += 46;
            if direction & 3 == 0 { clocks += 6; }
            else { clocks += 16; index = index.wrapping_add(1); }
        }
        clocks += 106;
        let mut table = true;
        if p.is_near_pit() {
            clocks += 32;
            if !p.near_pit_state_is(3) { clocks += 6; }
            else { clocks += 56 + if modifier >= 0x30 { 6 + 62 } else { 62 }; }
        } else {
            clocks += 6 + 40;
            if modifier == 0 { clocks += 6; }
            else {
                clocks += 72;
                if sub == 8 { clocks += 6; index = 10; }
                else {
                    clocks += 32;
                    if sub == 16 { clocks += 6; index = 10; }
                    else { clocks += 16; index = 2; }
                }
                clocks += 56 + 6 + 56; // AND #0 makes BEQ unconditional.
                if modifier == 1 { clocks += 6; }
                else {
                    clocks += 32;
                    if modifier >= 16 { clocks += 6 + 48; }
                    else { clocks += 78; table = false; }
                }
            }
        }
        if table { clocks += 32 + if u16::from(index) + 0x27 > 0xff { 6 } else { 0 }; }
        // $E333..E372: scratch setup, two signed component passes, Z setup.
        clocks += 182 + 6 + 134;
        for (mask, negative) in [(3, 2), (12, 8)] {
            clocks += 64 + 182;
            if direction & mask == 0 { clocks += 6; }
            else { clocks += 40 + 60 + if direction & negative == 0 { 6 } else { 90 }; }
        }
        clocks
    }

    /// Run the stateful speed-selection prefix, stopping before either
    /// component of actual velocity is resolved.
    fn link_handle_velocity_before_actual_velocity_resolution(&mut self) -> Option<(u8, u8)> {
        let speed_index = self.link_handle_velocity_until_velocity_cleared()?;
        Some(self.link_handle_velocity_after_velocity_cleared(speed_index))
    }

    fn link_handle_velocity_until_velocity_cleared(&mut self) -> Option<u8> {
        let speed_index = self.link_handle_velocity_before_velocity_clear()?;
        self.follower_link_state_mut()
            .clear_actual_velocity_and_page_movement_deltas();
        Some(speed_index)
    }

    fn link_handle_velocity_before_velocity_clear(&mut self) -> Option<u8> {
        let old_x = self.game_state.player.follower_link.x();
        let old_y = self.game_state.player.follower_link.y();

        if (self.game_state.frame.submodule == 2 && self.game_state.frame.main_module == 14)
            || self
                .game_state
                .player
                .follower_link
                .is_prevented_from_moving()
        {
            self.store_link_safe_return_position(old_x, old_y);
            self.link_handle_velocity_and_sand_drag(old_x, old_y);
            return None;
        }

        if self.game_state.player.follower_link.handler_state() == 4 {
            self.handle_swim_stroke_and_subpixels();
            return None;
        }

        let speed_index = if self.game_state.player.follower_link.flag_moving() != 0 {
            if !self.game_state.player.follower_link.is_running() {
                self.handle_swim_stroke_and_subpixels();
                return None;
            }
            24
        } else {
            if self.game_state.player.follower_link.is_running() {
                if self.lag_free && self.game_state.player.follower_link.dash_counter() < 16 {
                    self.link_dash_in_place();
                    return None;
                }
                self.follower_link_state_mut().clear_speed_modifier();
                assert!(self.game_state.player.follower_link.dash_counter() >= 32);
            }
            if (self
                .game_state
                .player
                .tile_detection
                .tile_collision_bits_primary()
                | self
                    .game_state
                    .player
                    .tile_detection
                    .tile_collision_bits_secondary())
                == 0x0f
            {
                if self.lag_free {
                    // $07:E402..E405: all-blocked BRL's common return only.
                    crate::cycle_ledger::charge_routine(0x07_e402, 94);
                }
                return None;
            }
            if self
                .game_state
                .player
                .follower_link
                .water_ripple_or_grass_state()
                != 0
            {
                match self.game_state.player.follower_link.speed_setting() {
                    16 => 22,
                    12 => 14,
                    _ => 12,
                }
            } else {
                self.game_state.player.follower_link.speed_setting()
            }
        };

        Some(speed_index)
    }

    fn link_handle_velocity_after_velocity_cleared(&mut self, mut speed_index: u8) -> (u8, u8) {
        let direction = self.game_state.player.follower_link.direction();
        if (direction & 0x0c) != 0 && (direction & 0x03) != 0 {
            speed_index = speed_index.wrapping_add(1);
        }

        if self.game_state.player.follower_link.is_near_pit() {
            if self.game_state.player.follower_link.near_pit_state_is(3) {
                self.follower_link_state_mut()
                    .increase_near_pit_speed_modifier();
            }
        } else if self.game_state.player.follower_link.speed_modifier() != 0 {
            speed_index =
                if self.game_state.frame.submodule == 8 || self.game_state.frame.submodule == 16 {
                    10
                } else {
                    2
                };
            let speed_modifier = self.game_state.player.follower_link.speed_modifier();
            if speed_modifier != 1 && speed_modifier < 16 {
                self.follower_link_state_mut().advance_dash_deceleration();
                speed_index = 26;
            } else if speed_modifier != 1 {
                self.follower_link_state_mut().clear_speed_modifier();
                self.follower_link_state_mut().set_speed_setting(0);
            }
        }

        let velocity = self
            .game_state
            .player
            .follower_link
            .speed_modifier()
            .wrapping_add(LINK_HANDLE_VELOCITY_SPEED_MOD[speed_index as usize]);
        (self.game_state.player.follower_link.direction(), velocity)
    }

    /// `Link_MovePosition` up to and including the selected axis' subpixel
    /// store: the ROM's loop runs z (airborne only), then x, then y; earlier
    /// axes complete, the selected axis owes its coordinate delta.
    fn link_move_position_partial_after_subpixel(
        &mut self,
        axis: PlayerAxis,
    ) -> LinkMovePositionPartialReturn {
        let x = self.game_state.player.follower_link.x();
        let y = self.game_state.player.follower_link.y();
        self.follower_link_state_mut()
            .store_safe_return_position(x, y);
        assert!(
            !(self.game_state.player.follower_link.handler_state() != 10
                && self.game_state.player.follower_link.on_somaria_platform() == 2),
            "a mid-loop Link_MovePosition interruption cannot take the Somaria platform exit",
        );
        for &candidate in self.link_move_position_axes() {
            let velocity = self.link_move_position_axis_velocity(candidate);
            if candidate == axis {
                let pending_pixel_delta = self
                    .follower_link_state_mut()
                    .move_axis_subpixel_only_by_velocity(candidate, velocity);
                return LinkMovePositionPartialReturn {
                    old_x: x,
                    old_y: y,
                    partial: LinkMovePositionPartial {
                        axis,
                        pending_pixel_delta,
                    },
                };
            }
            self.link_move_position_full_axis(candidate, velocity);
        }
        panic!("Link_MovePosition interruption named axis {axis:?} outside the ROM's axis loop");
    }

    /// `Link_MovePosition` through both coordinate stores for `axis`.
    fn link_move_position_after_coordinates(
        &mut self,
        axis: PlayerAxis,
    ) -> LinkMovePositionAfterCoordinatesReturn {
        let x = self.game_state.player.follower_link.x();
        let y = self.game_state.player.follower_link.y();
        self.follower_link_state_mut()
            .store_safe_return_position(x, y);
        assert!(
            !(self.game_state.player.follower_link.handler_state() != 10
                && self.game_state.player.follower_link.on_somaria_platform() == 2),
            "a mid-loop Link_MovePosition interruption cannot take the Somaria platform exit",
        );
        for &candidate in self.link_move_position_axes() {
            let velocity = self.link_move_position_axis_velocity(candidate);
            self.link_move_position_full_axis(candidate, velocity);
            if candidate == axis {
                return LinkMovePositionAfterCoordinatesReturn {
                    old_x: x,
                    old_y: y,
                    axis,
                };
            }
        }
        panic!("Link_MovePosition interruption named axis {axis:?} outside the ROM's axis loop");
    }

    /// `Link_MovePosition` through the current axis' low coordinate-byte
    /// store, preserving its high byte and every later axis.
    fn link_move_position_after_coordinate_low(
        &mut self,
        axis: PlayerAxis,
    ) -> LinkMovePositionAfterCoordinateLowReturn {
        let x = self.game_state.player.follower_link.x();
        let y = self.game_state.player.follower_link.y();
        self.follower_link_state_mut()
            .store_safe_return_position(x, y);
        assert!(
            !(self.game_state.player.follower_link.handler_state() != 10
                && self.game_state.player.follower_link.on_somaria_platform() == 2),
            "a mid-loop Link_MovePosition interruption cannot take the Somaria platform exit",
        );
        for &candidate in self.link_move_position_axes() {
            let velocity = self.link_move_position_axis_velocity(candidate);
            if candidate == axis {
                let mut player = self.follower_link_state_mut();
                let pending_pixel_delta =
                    player.move_axis_subpixel_only_by_velocity(candidate, velocity);
                let pending_coordinate_high =
                    player.apply_axis_pixel_delta_low(candidate, pending_pixel_delta);
                return LinkMovePositionAfterCoordinateLowReturn {
                    old_x: x,
                    old_y: y,
                    axis,
                    pending_coordinate_high,
                };
            }
            self.link_move_position_full_axis(candidate, velocity);
        }
        panic!("Link_MovePosition interruption named axis {axis:?} outside the ROM's axis loop");
    }

    /// Resume `Link_MovePosition` after `link_move_position_partial_after_subpixel`.
    pub(in crate::zelda_rtl) fn complete_link_move_position_from_partial(
        &mut self,
        position_return: LinkMovePositionPartialReturn,
    ) {
        let LinkMovePositionPartial {
            axis,
            pending_pixel_delta,
        } = position_return.partial;
        self.follower_link_state_mut()
            .apply_axis_pixel_delta(axis, pending_pixel_delta);
        let passes = self.link_move_position_axes();
        let resume_at = passes
            .iter()
            .position(|candidate| *candidate == axis)
            .expect("a suspended Link_MovePosition axis must belong to the ROM's axis loop");
        for candidate in passes[resume_at + 1..].iter().copied() {
            let velocity = self.link_move_position_axis_velocity(candidate);
            self.link_move_position_full_axis(candidate, velocity);
        }
        self.complete_link_move_position_after_coordinates(LinkMovePositionReturn {
            old_x: position_return.old_x,
            old_y: position_return.old_y,
        });
    }

    /// Resume after the current axis' low coordinate byte was published.
    pub(in crate::zelda_rtl) fn complete_link_move_position_from_after_coordinate_low(
        &mut self,
        position_return: LinkMovePositionAfterCoordinateLowReturn,
    ) {
        self.follower_link_state_mut().apply_axis_coordinate_high(
            position_return.axis,
            position_return.pending_coordinate_high,
        );
        self.complete_link_move_position_from_after_coordinates(
            LinkMovePositionAfterCoordinatesReturn {
                old_x: position_return.old_x,
                old_y: position_return.old_y,
                axis: position_return.axis,
            },
        );
    }

    /// Resume after `link_move_position_after_coordinates`, beginning with
    /// the following axis and then completing the ordinary movement tail.
    pub(in crate::zelda_rtl) fn complete_link_move_position_from_after_coordinates(
        &mut self,
        position_return: LinkMovePositionAfterCoordinatesReturn,
    ) {
        let passes = self.link_move_position_axes();
        let resume_at = passes
            .iter()
            .position(|candidate| *candidate == position_return.axis)
            .expect("a suspended Link_MovePosition axis must belong to the ROM's axis loop");
        for candidate in passes[resume_at + 1..].iter().copied() {
            let velocity = self.link_move_position_axis_velocity(candidate);
            self.link_move_position_full_axis(candidate, velocity);
        }
        self.complete_link_move_position_after_coordinates(LinkMovePositionReturn {
            old_x: position_return.old_x,
            old_y: position_return.old_y,
        });
    }

    /// The ROM's `Link_MovePosition` axes in their original execution order.
    fn link_move_position_axes(&self) -> &'static [PlayerAxis] {
        if self.game_state.player.follower_link.auxiliary_state() != 0 {
            &[PlayerAxis::Z, PlayerAxis::X, PlayerAxis::Y]
        } else {
            &[PlayerAxis::X, PlayerAxis::Y]
        }
    }

    fn link_move_position_axis_velocity(&self, axis: PlayerAxis) -> u8 {
        match axis {
            PlayerAxis::Z => self.game_state.player.follower_link.actual_z_velocity(),
            PlayerAxis::X => self.game_state.player.follower_link.actual_x_velocity(),
            PlayerAxis::Y => self.game_state.player.follower_link.actual_y_velocity(),
        }
    }

    fn link_move_position_full_axis(&mut self, axis: PlayerAxis, velocity: u8) {
        self.follower_link_state_mut()
            .move_axis_by_velocity(axis, velocity);
    }

    fn link_dash_in_place(&mut self) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x07_e545));
        let p = &self.game_state.player.follower_link;
        let mut clocks = 104;
        let speed = if p.joypad1h_last() & 15 == 0 {
            clocks += 6;
            0u16
        } else {
            clocks += 64 + 64;
            if p.water_ripple_or_grass_state() == 0 { clocks += 6; 0x180 }
            else { clocks += 16; 0x150 }
        };
        clocks += 184;
        let direction = p.direction() & 15;
        let mut index = 3;
        let mut bits = direction;
        while bits & 1 == 0 {
            clocks += 30 + 30;
            if index == 0 {
                if self.lag_free {
                    // $07:E545..E575: no direction returns without integration.
                    crate::cycle_ledger::charge(clocks + 72);
                }
                self.follower_link_state_mut().clear_actual_velocity_xy();
                return;
            }
            clocks += 6;
            bits >>= 1;
            index -= 1;
        }
        clocks += 30 + 6 + 100;
        if index == 0 { clocks += 6 + 38; }
        else { clocks += 32 + if index != 2 { 6 } else { 38 }; }
        if self.lag_free {
            // $07:E576..E592: sign/axis selection and BRL only; E4D3 owns
            // integration and E3E0 owns the final drag/common-return tail.
            crate::cycle_ledger::charge(clocks + 208);
        }
        let velocity = if index == 0 || index == 2 { 0u16.wrapping_sub(speed) } else { speed };
        self.follower_link_state_mut().clear_actual_velocity_xy();
        let (x, y) = if index & 2 != 0 { (velocity, 0) } else { (0, velocity) };
        self.player_something_with_velocity_tired_or_swim(x, y);
    }

    // $07:E42A..E4D0: actual timer, acceleration, clamp and direction
    // branches. E4D3 owns integration; E3E0 owns the drag/return tail.
    fn swim_stroke_master(&self) -> u64 {
        let p = &self.game_state.player.follower_link;
        let a = &self.game_state.player.swim_acceleration;
        let mut clocks = 86;
        for offset in [2, 0] {
            let mut mode = a.mode(offset);
            let timer = p.swim_stroke_frame_counter(offset).wrapping_sub(1);
            clocks += 122;
            if (timer as i16) < 0 { clocks += 116; mode = 1; }
            else { clocks += 6; }
            clocks += 148;
            let mut table_index = mode.wrapping_mul(2) as u8;
            if p.flag_moving() == 0 { clocks += 6; }
            else {
                clocks += 148;
                table_index = table_index.wrapping_add(p.flag_moving().wrapping_mul(8));
            }
            clocks += 110 + if u16::from(table_index) + 0x06 > 0xff { 6 } else { 0 };
            let delta = HANDLE_SWIM_STROKE_AND_SUBPIXELS_SWIM_ACCELERATION_DELTAS
                [usize::from(table_index / 2)] as i16 as u16;
            let sum = a.acceleration(offset).wrapping_add(delta);
            if sum == 0 { clocks += 6; }
            else { clocks += 16 + if (sum as i16) > 0 { 6 } else { 0 }; }
            if (sum as i16) <= 0 {
                clocks += 216 + if mode == 2 { 178 } else { 6 + 138 };
            } else {
                clocks += 322;
                let index = (offset as u16).wrapping_add(a.acceleration_direction(offset))
                    .wrapping_mul(2) as u8;
                if u16::from(index) + 0x22 > 0xff { clocks += 6; }
                clocks += if sum < a.max_speed(offset) { 6 } else { 40 };
            }
            clocks += 132 + if p.has_swim_axis_drag() { 220 } else { 6 };
            clocks += 80 + if a.acceleration_direction(offset) as u8 != 0 { 6 } else { 114 };
            clocks += 44 + if offset == 0 { 6 } else { 30 };
        }
        clocks
    }

    pub(in crate::zelda_rtl) fn handle_swim_stroke_and_subpixels(&mut self) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x07_e42a));
        if self.lag_free {
            crate::cycle_ledger::charge(self.swim_stroke_master());
        }
        self.follower_link_state_mut().clear_actual_velocity_xy();

        let mut stroke = [0u16; 2];

        for i in (0..=1).rev() {
            let offset = i * 2;
            let stroke_timer = self
                .game_state
                .player
                .follower_link
                .swim_stroke_frame_counter(offset)
                .wrapping_sub(1);
            self.follower_link_state_mut()
                .set_swim_stroke_frame_counter(offset, stroke_timer);
            if (stroke_timer as i16) < 0 {
                self.follower_link_state_mut()
                    .set_swim_stroke_frame_counter(offset, 0);
                self.swim_acceleration_mut().set_mode(offset, 1);
            }

            let mut table_index = self.game_state.player.swim_acceleration.mode(offset);
            if self.game_state.player.follower_link.flag_moving() != 0 {
                table_index = table_index.wrapping_add(
                    u16::from(self.game_state.player.follower_link.flag_moving()) * 4,
                );
            }

            let delta = HANDLE_SWIM_STROKE_AND_SUBPIXELS_SWIM_ACCELERATION_DELTAS
                [table_index as usize] as i16 as u16;
            let mut sum = self
                .game_state
                .player
                .swim_acceleration
                .acceleration(offset)
                .wrapping_add(delta);
            if (sum as i16) <= 0 {
                self.follower_link_state_mut().mask_direction(
                    HANDLE_SWIM_STROKE_AND_SUBPIXELS_SWIM_AXIS_DIRECTION_CLEAR_MASKS[i],
                );
                self.follower_link_state_mut()
                    .set_last_direction_from_current_direction();
                if self.game_state.player.swim_acceleration.mode(offset) == 2 {
                    self.swim_acceleration_mut().set_mode(offset, 0);
                    self.swim_acceleration_mut().set_max_speed(offset, 240);
                    self.swim_acceleration_mut().set_acceleration(offset, 2);
                } else {
                    self.swim_acceleration_mut().set_mode(offset, 0);
                    self.swim_acceleration_mut().set_max_speed(offset, 0);
                    self.swim_acceleration_mut().set_acceleration(offset, 0);
                }
            } else {
                let dir_index = self
                    .game_state
                    .player
                    .swim_acceleration
                    .acceleration_direction(offset) as usize
                    + i * 2;
                self.follower_link_state_mut().add_direction_flags(
                    HANDLE_SWIM_STROKE_AND_SUBPIXELS_SWIM_DIRECTION_BITS_BY_AXIS[dir_index],
                );
                let max_sum = self.game_state.player.swim_acceleration.max_speed(offset);
                if sum >= max_sum {
                    sum = max_sum;
                }
                self.swim_acceleration_mut().set_acceleration(offset, sum);
            }

            stroke[i] = self
                .game_state
                .player
                .swim_acceleration
                .acceleration(offset);
            if self.game_state.player.follower_link.has_swim_axis_drag() {
                stroke[i] = stroke[i].wrapping_sub(stroke[i] >> 2);
            }
            if self
                .game_state
                .player
                .swim_acceleration
                .acceleration_direction(offset)
                == 0
            {
                stroke[i] = 0u16.wrapping_sub(stroke[i]);
            }
        }

        self.player_something_with_velocity_tired_or_swim(stroke[1], stroke[0]);
    }
}
