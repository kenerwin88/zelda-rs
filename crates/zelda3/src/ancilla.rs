//! Ancilla dispatch, shared tables, subsystem modules, and regression tests.
// Methods ported from zelda3/src/ancilla.c and included inside ZeldaState.

use super::*;

#[macro_use]
mod ancilla_shared;
use ancilla_shared::*;

#[path = "ancilla/spawn.rs"]
mod spawn;
#[path = "ancilla/bombs.rs"]
mod bombs;
#[path = "ancilla/weapons.rs"]
mod weapons;
#[path = "ancilla/magic.rs"]
mod magic;
#[path = "ancilla/items.rs"]
mod items;
#[path = "ancilla/effects.rs"]
mod effects;
#[path = "ancilla/helpers.rs"]
mod helpers;

use crate::types::{sign8};

impl ZeldaState {
    pub(crate) fn native_receipt_animation_table_byte(address:u32)->Option<u8> {
        match address {
            0x08c375..=0x08c377=>Some(RECEIVE_ITEM_MILESTONE_GFX_SOURCES[(address-0x08c375) as usize]),
            0x08c378..=0x08c37a=>Some(RECEIVE_ITEM_MILESTONE_FRAME_TIMERS[(address-0x08c378) as usize]),
            0x08c37b..=0x08c37d=>Some(RECEIVE_ITEM_CRYSTAL_FRAME_SEQUENCE[(address-0x08c37b) as usize]),
            _=>None,
        }
    }

    pub(crate) fn native_ancilla_oam_table_byte(address: u32) -> Option<u8> {
        match address {
            0x09849c..=0x0984e7=>Some(WISH_POND_ITEM_OAM_FLAGS[(address-0x09849c) as usize]),
            0x08c003..=0x08c00a => Some(ANCILLA20_BLANKET_BEDSPREAD_CHAR[(address - 0x08c003) as usize]),
            0x08c00b..=0x08c012 => Some(ANCILLA20_BLANKET_BEDSPREAD_FLAGS[(address - 0x08c00b) as usize]),
            0x08f66d..=0x08f670 => Some(ANCILLA_PREP_OAM_COORD_TAGALONG_LAYER_BITS[(address - 0x08f66d) as usize]),
            _ => None,
        }
    }

    pub(crate) fn native_ancilla_sprite_count(kind: usize) -> Option<u8> {
        ANCILLA_DRAW_SPRITE_COUNTS.get(kind).copied()
    }

    pub(super) fn ancilla_main(&mut self) {
        // Cycle ledger: Ancilla_Main $08:8242 (m8 x8, JSL target): PHB PHK PLB
        // JSR Ancilla_WeaponTink JSR Ancilla_ExecuteAll PLB RTL (236; the
        // callees charge their own bodies).
        let _scope = crate::cycle_ledger::routine(0x08_8242);
        crate::cycle_ledger::charge(236);
        self.ancilla_weapon_tink();
        self.ancilla_execute_all();
    }

    fn ancilla_weapon_tink(&mut self) {
        // Cycle ledger: Ancilla_WeaponTink $08:8F89 (m8 x8, JSR target).
        let _scope = crate::cycle_ledger::routine(0x08_8f89);
        // $08:8F89-8F8C LDA $0fac : BEQ (48), taken (+6) to $08:8F81 RTS (42).
        if self.game_state.sprites.garnish_runtime.repulsespark_timer() == 0 {
            crate::cycle_ledger::charge(48 + 6 + 42);
            return;
        }
        crate::cycle_ledger::charge(48);
        self.ancilla_weapon_tink_after_guard();
    }

    fn ancilla_weapon_tink_after_guard(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x08_8f89);
        self.sprite_system_mut().set_alert_flag(2);
        let anim_delay = self.garnish_state_mut().decrement_repulsespark_anim_delay();
        // $08:8F8E-8F96 LDA #$02 STA $0fdc DEC $0faf BPL (110): a negative
        // delay falls into $08:8F98-8F9D DEC $0fac LDA #$01 STA $0faf (94),
        // else the BPL is taken (+6).
        if sign8(anim_delay) {
            crate::cycle_ledger::charge(110 + 94);
            self.garnish_state_mut().decrement_repulsespark_timer();
            self.garnish_state_mut().set_repulsespark_anim_delay(1);
        } else {
            crate::cycle_ledger::charge(110 + 6);
        }

        // $08:8FA0-8FA5 LDA #$10 LDY $0fb3 BEQ (64): sorting off takes the
        // BEQ (+6) into $08:8FB8 JSL RegionA (62); sorting on runs
        // $08:8FA7-8FAA LDY $0b68 : BNE (48) then $08:8FAC JSL RegionD : BRA
        // (84) or the taken BNE (+6) into $08:8FB2 JSL RegionF : BRA (84).
        if self.game_state.oam.has_sprite_sorting() {
            if self
                .game_state
                .sprites
                .garnish_runtime
                .repulsespark_floor_status()
                != 0
            {
                crate::cycle_ledger::charge(64 + 48 + 6 + 84);
                self.oam_allocate_from_region_f(0x10);
            } else {
                crate::cycle_ledger::charge(64 + 48 + 84);
                self.oam_allocate_from_region_d(0x10);
            }
        } else {
            crate::cycle_ledger::charge(64 + 6 + 62);
            self.oam_allocate_from_region_a(0x10);
        }

        let x = self
            .game_state
            .sprites
            .garnish_runtime
            .repulsespark_x_lo()
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2_low());
        let y = self
            .game_state
            .sprites
            .garnish_runtime
            .repulsespark_y_lo()
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2_low());
        // $08:8FBC-8FC5 LDA $0fad SEC SBC $00e2 CMP #$f8 BCS (110), then
        // $08:8FC7-8FD2 STA $00 LDA $0fae SEC SBC $00e8 CMP #$f0 BCS (134);
        // either taken BCS (+6) lands on $08:9009 STZ $0fac : RTS (74).
        if x >= 0xf8 {
            crate::cycle_ledger::charge(110 + 6 + 74);
        } else if y >= 0xf0 {
            crate::cycle_ledger::charge(110 + 134 + 6 + 74);
        } else {
            crate::cycle_ledger::charge(110 + 134);
        }
        if x >= 0xf8 || y >= 0xf0 {
            self.garnish_state_mut().clear_repulsespark_timer();
            return;
        }

        let oam = self.game_state.oam.current_pointer_usize();
        let oam_idx = (oam - OAM_BUF) / 4;

        let flags = ANCILLA_WEAPON_TINK_REPULSE_SPARK_FLAGS[self
            .game_state
            .sprites
            .garnish_runtime
            .repulsespark_floor_status()
            as usize];
        // $08:8FD4-8FDB STA $01 LDA $0fac CMP #$03 BCC (88): a timer of 3 or
        // more runs $08:8FDD-8FEF (250; its CPX #$09 BCS is taken, +6, for
        // 9 and up, else $08:8FF1 LDA #$92, 16) and $08:8FF3-9008 (352, RTS
        // included); below 3 the BCC is taken (+6) into the four-sprite
        // block $08:900D-907F (1,644, RTS included).
        if self.game_state.sprites.garnish_runtime.repulsespark_timer() >= 3 {
            crate::cycle_ledger::charge(
                88 + 250
                    + if self.game_state.sprites.garnish_runtime.repulsespark_timer() < 9 {
                        16
                    } else {
                        6
                    }
                    + 352,
            );
            self.set_oam_plain(
                oam_idx,
                x,
                y,
                if self.game_state.sprites.garnish_runtime.repulsespark_timer() < 9 {
                    0x92
                } else {
                    0x80
                },
                flags,
                0,
            );
            return;
        }

        crate::cycle_ledger::charge(88 + 6 + 1_644);
        let c = ANCILLA_WEAPON_TINK_REPULSE_SPARK_CHAR
            [self.game_state.sprites.garnish_runtime.repulsespark_timer() as usize];
        self.set_oam_plain(oam_idx, x.wrapping_sub(4), y.wrapping_sub(4), c, flags, 0);
        self.set_oam_plain(
            oam_idx + 1,
            x.wrapping_add(4),
            y.wrapping_sub(4),
            c,
            flags | 0x40,
            0,
        );
        self.set_oam_plain(
            oam_idx + 2,
            x.wrapping_sub(4),
            y.wrapping_add(4),
            c,
            flags | 0x80,
            0,
        );
        self.set_oam_plain(
            oam_idx + 3,
            x.wrapping_add(4),
            y.wrapping_add(4),
            c,
            flags | 0xc0,
            0,
        );
    }

    fn ancilla_empty(&mut self, _k: usize) {}

    fn ancilla_unused_14(&mut self, _k: usize) {
        // Ancilla_Unused_14 is an assert-only dispatch slot in the C port.
        panic!("Ancilla_Unused_14");
    }

    fn ancilla_unused_25(&mut self, _k: usize) {
        // Ancilla_Unused_25 is an assert-only dispatch slot in the C port.
        panic!("Ancilla_Unused_25");
    }

    fn ancilla_execute_all(&mut self) {
        // Cycle ledger: Ancilla_ExecuteAll $08:832B (m8 x8, JSR target):
        // $08:832B LDX #$09 (16), then the slot loop below.
        let _scope = crate::cycle_ledger::routine(0x08_832b);
        crate::cycle_ledger::charge(16);
        self.ancilla_execute_slots_below(10);
    }

    /// Run the descending ancilla loop for slots below `from`, stopping when
    /// a slot's synchronous item receipt suspended the translated call stack
    /// (the falling milestone item, route host 1142850); the receipt's
    /// completion resumes the remaining slots through this same helper.
    pub(super) fn ancilla_execute_slots_below(&mut self, from: usize) {
        // Cycle ledger (Ancilla_ExecuteAll's loop, charged into the caller's
        // scope): per slot $08:832D-8333 STX $0fa0 LDA $c4a,x BEQ (80), then
        // $08:8335 JSR Ancilla_ExecuteOne (46) or the taken BEQ (+6), and
        // $08:8338-8339 DEX : BPL (30, +6 taken for slots 9..1); $08:833B
        // RTS (42) once the loop ran to slot 0. A resumed call charges the
        // remaining slots and the RTS.
        for i in (0..from).rev() {
            if self
                .game_execution_scheduler
                .work_suspends_translated_call_stack()
            {
                return;
            }
            self.sprite_system_mut().set_cur_object_index(i as u8);
            let ty = self.ancilla_slot_view(i).ancilla_type();
            crate::cycle_ledger::charge(if ty != 0 { 80 + 46 } else { 80 + 6 });
            crate::cycle_ledger::charge(if i > 0 { 30 + 6 } else { 30 });
            if ty != 0 {
                let ancilla = self.ancilla_slot_view(i);
                self.replay_trace_ram_watch(&format!(
                    "ancilla-before-execute-one ancilla={i} type=0x{:02x} timer=0x{:02x} item=0x{:02x} work3=0x{:02x} floor=0x{:02x} num={}",
                    ty,
                    ancilla.timer(),
                    ancilla.item_to_link(),
                    ancilla.work_byte_3(),
                    ancilla.floor(),
                    ancilla.num_sprites(),
                ));
                self.ancilla_execute_one(ty, i);
                let ancilla = self.ancilla_slot_view(i);
                self.replay_trace_ram_watch(&format!(
                    "ancilla-after-execute-one ancilla={i} type=0x{:02x} timer=0x{:02x} item=0x{:02x} work3=0x{:02x} floor=0x{:02x} num={}",
                    self.ancilla_slot_view(i).ancilla_type(),
                    ancilla.timer(),
                    ancilla.item_to_link(),
                    ancilla.work_byte_3(),
                    ancilla.floor(),
                    ancilla.num_sprites(),
                ));
            }
        }
        // $08:833B RTS (42).
        crate::cycle_ledger::charge(42);
    }

    fn ancilla_execute_one(&mut self, ty: u8, k: usize) {
        // Cycle ledger: Ancilla_ExecuteOne $08:833C (m8 x8, JSR target; the
        // handler is entered by JMP (abs) and runs inside this scope).
        // $08:833C-833F PHA CPX #$06 BCS (54): slots 6+ take the BCS (+6);
        // below that the ROM inlines the OAM allocation: $08:8341-8347 LDA
        // $c90,x LDY $0fb3 BEQ (80), sorting off taking the BEQ (+6) into
        // $08:835A JSL RegionA (62), sorting on running $08:8349-834C LDY
        // $c7c,x : BNE (48) then $08:834E JSL RegionD : BRA (84) or the taken
        // BNE (+6) into $08:8354 JSL RegionF : BRA (84); then $08:835E-835F
        // TYA STA $c86,x (52).
        let _scope = crate::cycle_ledger::routine(0x08_833c);
        if k < 6 {
            crate::cycle_ledger::charge(54);
            crate::cycle_ledger::charge(if self.game_state.oam.has_sprite_sorting() {
                if self.ancilla_slot_view(k).floor() != 0 {
                    80 + 48 + 6 + 84
                } else {
                    80 + 48 + 84
                }
            } else {
                80 + 6 + 62
            });
            crate::cycle_ledger::charge(52);
            let num_sprites = self.ancilla_slot_view(k).num_sprites();
            let idx = self.ancilla_allocate_oam_from_region_a_or_d_or_f(k, num_sprites);
            self.ancilla_slot_view_mut(k).set_oam_index(idx as u8);
        } else {
            crate::cycle_ledger::charge(54 + 6);
        }

        // $08:8362-8364 LDY $11 : BNE (40), taken (+6) outside submodule 0;
        // else $08:8366-8369 LDY $c68,x : BEQ (48) and $08:836B DEC $c68,x
        // (52) or the taken BEQ (+6).
        if self.game_state.frame.submodule != 0 {
            crate::cycle_ledger::charge(40 + 6);
        } else if self.ancilla_slot_view(k).timer() != 0 {
            crate::cycle_ledger::charge(40 + 48 + 52);
        } else {
            crate::cycle_ledger::charge(40 + 48 + 6);
        }
        if self.game_state.frame.submodule == 0 && self.ancilla_slot_view(k).timer() != 0 {
            self.ancilla_slot_view_mut(k).tick_timer();
        }

        // $08:836E-837C PLA DEC ASL TAY LDA $837f,y STA $00 LDA $8380,y STA
        // $01 JMP ($00) (222); the two abs,y reads cross a page (+6 each)
        // from type $42 on, the second one already at type $41.
        crate::cycle_ledger::charge(match ty {
            0x42.. => 222 + 12,
            0x41 => 222 + 6,
            _ => 222,
        });
        self.ancilla_execute_body(ty,k);
    }

    fn ancilla_execute_body(&mut self, ty: u8, k: usize) {
        match ty {
            0x01 => self.ancilla01_somaria_bullet(k),
            0x02 => self.ancilla02_fire_rod_shot(k),
            0x03 => self.ancilla_empty(k),
            0x04 => self.ancilla04_beam_hit(k),
            0x05 => self.ancilla05_boomerang(k),
            0x06 => self.ancilla06_wall_hit(k),
            0x07 => self.ancilla07_bomb(k),
            0x08 => self.ancilla08_door_debris(k),
            0x09 => self.ancilla09_arrow(k),
            0x0a => self.ancilla0_a_arrow_in_the_wall(k),
            0x0b => self.ancilla0_b_ice_rod_shot(k),
            0x0c => self.ancilla_sword_beam(k),
            0x0d => self.ancilla0_d_spin_attack_full_charge_spark(k),
            0x0e..=0x10 => self.ancilla33_blast_wall_explosion(k),
            0x11 => self.ancilla11_ice_rod_wall_hit(k),
            0x12 => self.ancilla33_blast_wall_explosion(k),
            0x13 => self.ancilla13_ice_rod_sparkle(k),
            0x14 => self.ancilla_unused_14(k),
            0x15 => self.ancilla15_jump_splash(k),
            0x16 => self.ancilla16_hit_stars(k),
            0x17 => self.ancilla17_shovel_dirt(k),
            0x18 => self.ancilla18_ether_spell(k),
            0x19 => self.ancilla19_bombos_spell(k),
            0x1a => self.ancilla1_a_powder_dust(k),
            0x1b => self.ancilla_sword_wall_hit(k),
            0x1c => self.ancilla1_c_quake_spell(k),
            0x1d => self.ancilla1_d_screen_shake(k),
            0x1e => self.ancilla1_e_dash_dust(k),
            0x1f => self.ancilla1_f_hookshot(k),
            0x20 => self.ancilla20_blanket(k),
            0x21 => self.ancilla21_snore(k),
            0x22 => self.ancilla22_item_receipt(k),
            0x23 => self.ancilla23_link_poof(k),
            0x24 => self.ancilla24_gravestone(k),
            0x25 => self.ancilla_unused_25(k),
            0x26 => self.ancilla26_sword_swing_sparkle(k),
            0x27 => self.ancilla27_duck(k),
            0x28 => self.ancilla28_wish_pond_item(k),
            0x29 => self.ancilla29_milestone_item_receipt(k),
            0x2a => self.ancilla2_a_spin_attack_sparkle_a(k),
            0x2b => self.ancilla2_b_spin_attack_sparkle_b(k),
            0x2c => self.ancilla2_c_somaria_block(k),
            0x2d => self.ancilla2_d_somaria_block_fizz(k),
            0x2e => self.ancilla2_e_somaria_block_fission(k),
            0x2f => self.ancilla2_f_lamp_flame(k),
            0x30 => self.ancilla30_byrna_windup_spark(k),
            0x31 => self.ancilla31_byrna_spark(k),
            0x32 => self.ancilla32_blast_wall_fireball(k),
            0x33 => self.ancilla33_blast_wall_explosion(k),
            0x34 => self.ancilla34_skull_woods_fire(k),
            0x35 => self.ancilla35_master_sword_receipt(k),
            0x36 => self.ancilla36_flute(k),
            0x37 => self.ancilla37_weathervane_explosion(k),
            0x39 => self.ancilla39_somaria_platform_poof(k),
            0x38 => self.ancilla38_cutscene_duck(k),
            0x3a => self.ancilla3_a_big_bomb_explosion(k),
            0x3b => self.ancilla3_b_sword_up_sparkle(k),
            0x3c => self.ancilla3_c_spin_attack_charge_sparkle(k),
            0x3d => self.ancilla3_d_item_splash(k),
            0x3e => self.ancilla_rising_crystal(k),
            0x3f => self.ancilla3_f_bush_poof(k),
            0x40 => self.ancilla40_dwarf_poof(k),
            0x41 => self.ancilla41_waterfall_splash(k),
            0x42 => self.ancilla42_happiness_pond_rupees(k),
            0x43 => self.ancilla43_ganons_tower_cutscene(k),
            _ => {}
        }
    }

}

impl ZeldaState {
    pub(super) fn finish_native_ancilla_caller(
        &mut self,
        tail: crate::cycle_models::dungeon_ancilla::AncillaTail,
    ) {
        use crate::cycle_models::dungeon_ancilla::AncillaTail;
        let _scope = crate::cycle_ledger::routine(0x08_8242);
        match tail {
            AncillaTail::ReceiptOamReturned {slot} => {
                // The body and its real RTS have retired; retain the caller's slot loop and wrapper return.
                crate::cycle_ledger::charge(72);
                let _loop=crate::cycle_ledger::routine(0x08_832b);
                crate::cycle_ledger::charge(30+if slot!=0 {6}else{0});
                self.ancilla_execute_slots_below(usize::from(slot));
            }
            AncillaTail::ReceiptDrawing {slot,tail} => {
                crate::cycle_ledger::charge(72);
                let _loop=crate::cycle_ledger::routine(0x08_832b);
                crate::cycle_ledger::charge(30+if slot!=0 {6}else{0});
                self.finish_native_receipt_drawing(usize::from(slot),tail);
                if !self.game_execution_scheduler.work_suspends_translated_call_stack() {
                    self.ancilla_execute_slots_below(usize::from(slot));
                }
            }
            AncillaTail::Receipt {slot,tail} => {
                crate::cycle_ledger::charge(72);
                let _loop=crate::cycle_ledger::routine(0x08_832b);
                crate::cycle_ledger::charge(30+if slot!=0 {6}else{0});
                self.finish_native_receipt_prefix(usize::from(slot),tail);
                if !self.game_execution_scheduler.work_suspends_translated_call_stack() {
                    self.ancilla_execute_slots_below(usize::from(slot));
                }
            }
            AncillaTail::Returned => {}
            AncillaTail::Spark => {
                // Retain the wrapper return and next JSR charges, as the ordinary receipt continuation expects.
                crate::cycle_ledger::charge(46 + 72);
                self.ancilla_weapon_tink_after_guard();
                self.ancilla_execute_all();
            }
            AncillaTail::Body { slot, kind, .. } => {
                crate::cycle_ledger::charge(72);
                let _loop = crate::cycle_ledger::routine(0x08_832b);
                // The retained slot loop charges DEX/BPL before a body that can suspend into an item receipt.
                crate::cycle_ledger::charge(30 + if slot != 0 { 6 } else { 0 });
                {
                    let _body = crate::cycle_ledger::routine(0x08_833c);
                    self.ancilla_execute_body(kind, usize::from(slot));
                }
                if !self
                    .game_execution_scheduler
                    .work_suspends_translated_call_stack()
                {
                    self.ancilla_execute_slots_below(usize::from(slot));
                }
            }
        }
    }

}

#[cfg(test)]
#[path = "ancilla_tests.rs"]
mod tests;
