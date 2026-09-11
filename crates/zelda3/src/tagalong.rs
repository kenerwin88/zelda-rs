// Methods ported from zelda3/src/tagalong.c and included inside ZeldaState.

use super::*;
use crate::types::{abs16, sign16, sign8, ProjectSpeedRet};

const OAM_BUF_TAGALONG: usize = 0x0800;
const FEATURES0_MISC_BUG_FIXES_TAGALONG: u32 = 4096;
const FEATURES0_TURN_WHILE_DASHING_TAGALONG: u32 = 4;

const TAGALONG_SLOWDOWN_INDICATOR_BY_FOLLOWER: [u8; 15] =
    [0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const TAGALONG_RELEASE_INDICATOR_BY_FOLLOWER: [u8; 15] =
    [0, 0, 3, 3, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const TAGALONG_MESSAGE_FOLLOWER_INDICATORS: [u8; 3] = [5, 9, 0x0a];
const TAGALONG_MESSAGE_TIMERS: [u16; 3] = [0x0df3, 0x06f9, 0x0df3];
const TAGALONG_MSG: [u16; 3] = [0x20, 0x108, 0x11d];

#[derive(Clone, Copy)]
pub(super) struct TagalongMessageInfo {
    y: u16,
    x: u16,
    bit: u8,
    msg: u16,
    tagalong: u8,
}

#[derive(Clone, Copy)]
struct TagalongSprXY {
    y1: i8,
    x1: i8,
    y2: i8,
    x2: i8,
}

#[derive(Clone, Copy)]
struct TagalongDmaFlags {
    dma6: u8,
    dma7: u8,
    flags: u8,
}

const TAGALONG_INDOOR_INFOS: [TagalongMessageInfo; 12] = [
    TagalongMessageInfo {
        y: 0x1ef0,
        x: 0x288,
        bit: 1,
        msg: 0x99,
        tagalong: 4,
    },
    TagalongMessageInfo {
        y: 0x1e58,
        x: 0x2f0,
        bit: 2,
        msg: 0x9a,
        tagalong: 4,
    },
    TagalongMessageInfo {
        y: 0x1ea8,
        x: 0x3b8,
        bit: 4,
        msg: 0x9b,
        tagalong: 4,
    },
    TagalongMessageInfo {
        y: 0x0cf8,
        x: 0x25b,
        bit: 1,
        msg: 0x21,
        tagalong: 1,
    },
    TagalongMessageInfo {
        y: 0x0cf8,
        x: 0x39d,
        bit: 2,
        msg: 0x21,
        tagalong: 1,
    },
    TagalongMessageInfo {
        y: 0x0c78,
        x: 0x238,
        bit: 4,
        msg: 0x21,
        tagalong: 1,
    },
    TagalongMessageInfo {
        y: 0x0a30,
        x: 0x2f8,
        bit: 1,
        msg: 0x22,
        tagalong: 1,
    },
    TagalongMessageInfo {
        y: 0x0178,
        x: 0x550,
        bit: 1,
        msg: 0x23,
        tagalong: 1,
    },
    TagalongMessageInfo {
        y: 0x0168,
        x: 0x4f8,
        bit: 2,
        msg: 0x2a,
        tagalong: 1,
    },
    TagalongMessageInfo {
        y: 0x1bd8,
        x: 0x16fc,
        bit: 1,
        msg: 0x124,
        tagalong: 6,
    },
    TagalongMessageInfo {
        y: 0x1520,
        x: 0x167c,
        bit: 1,
        msg: 0x124,
        tagalong: 6,
    },
    TagalongMessageInfo {
        y: 0x05ac,
        x: 0x4fc,
        bit: 1,
        msg: 0x29,
        tagalong: 1,
    },
];

const TAGALONG_OUTDOOR_INFOS: [TagalongMessageInfo; 5] = [
    TagalongMessageInfo {
        y: 0x03c0,
        x: 0x0730,
        bit: 1,
        msg: 0x9d,
        tagalong: 4,
    },
    TagalongMessageInfo {
        y: 0x0648,
        x: 0x0f50,
        bit: 0,
        msg: 0xffff,
        tagalong: 0x0a,
    },
    TagalongMessageInfo {
        y: 0x06c8,
        x: 0x0d78,
        bit: 1,
        msg: 0xffff,
        tagalong: 0x0a,
    },
    TagalongMessageInfo {
        y: 0x0688,
        x: 0x0c78,
        bit: 2,
        msg: 0xffff,
        tagalong: 0x0a,
    },
    TagalongMessageInfo {
        y: 0x00e8,
        x: 0x0090,
        bit: 0,
        msg: 0x28,
        tagalong: 0x0e,
    },
];

const TAGALONG_INDOOR_OFFSETS: [u8; 8] = [0, 3, 6, 7, 9, 10, 11, 12];
const TAGALONG_OUTDOOR_OFFSETS: [u8; 4] = [0, 1, 4, 5];
const TAGALONG_INDOOR_ROOMS: [u16; 7] = [0xf1, 0x61, 0x51, 2, 0xdb, 0xab, 0x22];
const TAGALONG_OUTDOOR_ROOMS: [u16; 3] = [3, 0x5e, 0];

const TAGALONG_DRAW_SPR_XY: [TagalongSprXY; 56] = [
    TagalongSprXY {
        y1: -2,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: -2,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: -2,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: -2,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: -1,
        x1: 0,
        y2: 1,
        x2: 0,
    },
    TagalongSprXY {
        y1: -1,
        x1: 0,
        y2: 1,
        x2: 0,
    },
    TagalongSprXY {
        y1: -1,
        x1: 0,
        y2: 1,
        x2: 0,
    },
    TagalongSprXY {
        y1: -1,
        x1: 0,
        y2: 1,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 1,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 1,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 1,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 1,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: -3,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 3,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 1,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 1,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 1,
        x1: -3,
        y2: 1,
        x2: 0,
    },
    TagalongSprXY {
        y1: 1,
        x1: 3,
        y2: 1,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 1,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 1,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 1,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 1,
        x2: 0,
    },
    TagalongSprXY {
        y1: -1,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: -1,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: -1,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: -1,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 1,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 1,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 1,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 1,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 2,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 2,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 2,
        x1: -1,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 2,
        x1: 1,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 3,
        x1: 0,
        y2: 1,
        x2: 0,
    },
    TagalongSprXY {
        y1: 3,
        x1: 0,
        y2: 1,
        x2: 0,
    },
    TagalongSprXY {
        y1: 3,
        x1: -1,
        y2: 1,
        x2: 0,
    },
    TagalongSprXY {
        y1: 3,
        x1: 1,
        y2: 1,
        x2: 0,
    },
];

const TAGALONG_DMA_AND_FLAGS: [TagalongDmaFlags; 16] = [
    TagalongDmaFlags {
        dma6: 0x20,
        dma7: 0xc0,
        flags: 0x00,
    },
    TagalongDmaFlags {
        dma6: 0x00,
        dma7: 0xa0,
        flags: 0x00,
    },
    TagalongDmaFlags {
        dma6: 0x40,
        dma7: 0x60,
        flags: 0x00,
    },
    TagalongDmaFlags {
        dma6: 0x40,
        dma7: 0x60,
        flags: 0x44,
    },
    TagalongDmaFlags {
        dma6: 0x20,
        dma7: 0xc0,
        flags: 0x04,
    },
    TagalongDmaFlags {
        dma6: 0x00,
        dma7: 0xa0,
        flags: 0x04,
    },
    TagalongDmaFlags {
        dma6: 0x40,
        dma7: 0x80,
        flags: 0x00,
    },
    TagalongDmaFlags {
        dma6: 0x40,
        dma7: 0x80,
        flags: 0x44,
    },
    TagalongDmaFlags {
        dma6: 0x20,
        dma7: 0xe0,
        flags: 0x00,
    },
    TagalongDmaFlags {
        dma6: 0x00,
        dma7: 0xe0,
        flags: 0x00,
    },
    TagalongDmaFlags {
        dma6: 0x40,
        dma7: 0xe0,
        flags: 0x00,
    },
    TagalongDmaFlags {
        dma6: 0x40,
        dma7: 0xe0,
        flags: 0x44,
    },
    TagalongDmaFlags {
        dma6: 0x20,
        dma7: 0xe0,
        flags: 0x04,
    },
    TagalongDmaFlags {
        dma6: 0x00,
        dma7: 0xe0,
        flags: 0x04,
    },
    TagalongDmaFlags {
        dma6: 0x40,
        dma7: 0xe0,
        flags: 0x04,
    },
    TagalongDmaFlags {
        dma6: 0x40,
        dma7: 0xe0,
        flags: 0x40,
    },
];

const TAGALONG_DRAW_PALS: [u8; 14] = [0, 4, 4, 4, 4, 0, 7, 4, 4, 3, 4, 4, 4, 4];
const TAGALONG_DRAW_OFFS: [u16; 14] = [
    0, 0, 0x80, 0x80, 0x80, 0, 0, 0xc0, 0xc0, 0x100, 0x180, 0x180, 0x140, 0x140,
];
const TAGALONG_DRAW_SPR_INFO0: [u8; 24] = [
    0xd8, 0x24, 0xd8, 0x64, 0xd9, 0x24, 0xd9, 0x64, 0xda, 0x24, 0xda, 0x64, 0xc8, 0x22, 0xc8, 0x62,
    0xc9, 0x22, 0xc9, 0x62, 0xca, 0x22, 0xca, 0x62,
];
const TAGALONG_DRAW_SPR_OFFS0: [u16; 2] = [0x170, 0xc0];
const TAGALONG_DRAW_SPR_OFFS1: [u16; 2] = [0x1c0, 0x110];

impl ZeldaState {
    pub(super) fn tagalong_is_following(&self) -> bool {
        let main = self.game_state.frame.main_module;
        let sub = self.game_state.frame.submodule;
        !self.game_state.player.follower_link.is_immobilized()
            && sub != 10
            && !(main == 9 && sub == 0x23)
            && !(main == 14 && (sub == 1 || sub == 2))
    }

    pub(super) fn follower_validate_message_freedom(&self) -> bool {
        self.game_state
            .player
            .follower_link
            .can_open_follower_message()
    }

    pub(super) fn follower_move_towards_link(&mut self) {
        loop {
            let k = 9;
            let j = self.game_state.sprites.follower_runtime.tail_write_index() as usize;
            let x = self.tagalong_x(j);
            let y = self.tagalong_y(j);
            self.ancilla_slot_view_mut(k).set_x(x);
            self.ancilla_slot_view_mut(k).set_y(y);

            let pt = self.Ancilla_ProjectSpeedTowardsPlayer(k, 24);
            let mut ancilla = self.ancilla_slot_view_mut(k);
            ancilla.set_x_velocity(pt.x);
            ancilla.set_y_velocity(pt.y);
            self.Ancilla_MoveY(k);
            self.Ancilla_MoveX(k);

            let x = self.Ancilla_GetX(k);
            let y = self.Ancilla_GetY(k);
            let link = self.game_state.player.follower_link;
            if abs16(x.wrapping_sub(link.x())) < 2 && abs16(y.wrapping_sub(link.y())) < 2 {
                return;
            }
            self.follower_state_mut().increment_tail_write_index();
            let k = self.game_state.sprites.follower_runtime.tail_write_index() as usize;
            if k == 18 {
                return;
            }
            let layer_bits = self.game_state.player.follower_link.floor_layer_bits() | 1;
            let mut follower = self.tagalong_slot_mut(k);
            follower.set_position(x, y);
            follower.set_layer_bits(layer_bits);
        }
    }

    pub(super) fn follower_check_blind_trigger(&self) -> bool {
        let k = self.game_state.sprites.follower_runtime.data_index() as usize;
        let mut x = self.tagalong_x(k);
        let mut y = self.tagalong_y(k);
        let z = self.tagalong_slot(k).z_signed() as i16 as u16;
        y = y.wrapping_add(z).wrapping_add(12);
        x = x.wrapping_add(8);
        abs16(0x1568u16.wrapping_sub(y)) < 24 && abs16(0x1980u16.wrapping_sub(x)) < 24
    }

    pub(super) fn follower_initialize(&mut self) {
        let link = self.game_state.player.follower_link;
        let y = link.y();
        let x = link.x();
        let layer_bits = link.floor_layer_bits() | link.facing_layer_bits();
        let mut follower = self.tagalong_slot_mut(0);
        follower.set_position(x, y);
        follower.set_layer_bits(layer_bits);
        self.follower_state_mut().set_reacquire_timer_low(64);
        self.follower_state_mut().set_data_index(0);
        self.follower_state_mut().set_tail_write_index(0);
        self.follower_state_mut().clear_hookshot_interlock();
        self.follower_state_mut().clear_jump_timer();
        self.follower_link_state_mut().set_speed_setting(0);
        if self
            .game_state
            .enhanced_features
            .has(FEATURES0_TURN_WHILE_DASHING_TAGALONG)
        {
            let mut link = self.follower_link_state_mut();
            link.set_ground_state();
            link.clear_running();
        }
    }

    pub(super) fn sprite_become_follower(&mut self, k: usize) {
        self.follower_state_mut().set_appearance_none_flag(0);
        let y = self.Tagalong_Sprite_GetY(k).wrapping_sub(6);
        let x = self.Tagalong_Sprite_GetX(k).wrapping_add(1);
        let layer_bits = self.game_state.player.follower_link.floor_layer_bits() | 1;
        let mut follower = self.tagalong_slot_mut(0);
        follower.set_position(x, y);
        follower.set_layer_bits(layer_bits);
        self.follower_state_mut().set_reacquire_timer_low(64);
        self.follower_state_mut().set_tail_write_index(0);
        self.follower_state_mut().set_data_index(0);
        self.follower_state_mut().clear_hookshot_interlock();
        self.follower_state_mut().clear_jump_timer();
        self.follower_link_state_mut().set_speed_setting(0);
        self.follower_state_mut().set_appearance_none_flag(0);
        self.follower_state_mut().set_dropped(0);
        self.follower_move_towards_link();
    }

    pub(super) fn follower_main(&mut self) {
        // Cycle ledger: Follower_Main $09:9FC4 (m8 x8, JSR target of the
        // Tagalong_Main_ wrapper). $09:9FC4-9FC8 LDA long $7ef3cc : BNE
        // (56): no follower falls into $09:9FCA RTS (42).
        let _scope = crate::cycle_ledger::routine(0x09_9fc4);
        if self.game_state.sprites.follower_runtime.indicator() == 0 {
            crate::cycle_ledger::charge(56 + 42);
            return;
        }
        // BNE taken (+6), $09:9FCB-9FCD CMP #$0e : BNE (32): the trigger
        // follower falls into $09:9FCF BRL Follower_HandleTrigger (30; the
        // handler is a branch target and is not priced here).
        if self.game_state.sprites.follower_runtime.indicator() == 0x0e {
            crate::cycle_ledger::charge(56 + 6 + 32 + 30);
            self.follower_handle_trigger();
            return;
        }
        let j = TAGALONG_MESSAGE_FOLLOWER_INDICATORS
            .iter()
            .position(|&v| v == self.game_state.sprites.follower_runtime.indicator())
            .map(|v| v as i32)
            .unwrap_or(-1);
        // $09:9FD2 LDY #$02 (16), then the table search from Y = 2 down:
        // $09:9FD4-9FDB LDA long CMP $9fb5,y BEQ (88, +6 on the match) and
        // $09:9FDD-9FDE DEY : BPL (30, +6 taken) per miss; three misses end
        // with $09:9FE0 BRL Follower_NoTimedMessage (30).
        crate::cycle_ledger::charge(56 + 6 + 32 + 6 + 16);
        crate::cycle_ledger::charge(if j >= 0 {
            (2 - j as u64) * (88 + 30 + 6) + 88 + 6
        } else {
            3 * 88 + 3 * 30 + 2 * 6 + 30
        });
        if j >= 0 {
            // $09:9FE3-9FE5 LDA $11 : BNE Tagalong_5_14 (40, +6 taken); then
            // $09:9FE7-9FE9 CPY #$02 : BNE (32, +6 taken for j != 2) and for
            // j == 2 $09:9FEB-9FEF LDA $8a AND #$40 BNE Tagalong_5_14 (56,
            // +6 taken).
            if self.game_state.frame.submodule != 0 {
                crate::cycle_ledger::charge(40 + 6);
            } else if j == 2 {
                crate::cycle_ledger::charge(40 + 32 + 56);
                if self.game_state.world.location.overworld_screen_index() & 0x40 != 0 {
                    crate::cycle_ledger::charge(6);
                }
            } else {
                crate::cycle_ledger::charge(40 + 32 + 6);
            }
        }
        if j >= 0
            && self.game_state.frame.submodule == 0
            && !(j == 2 && self.game_state.world.location.overworld_screen_index() & 0x40 != 0)
        {
            let timer = self.tick_shared_message_timer();
            // $09:9FF1-9FF6 REP #$20 DEC $02cd BPL Tagalong_5_14 (100, +6
            // taken while non-negative); a negative timer runs $09:9FF8-9FFE
            // SEP JSL Follower_ValidateMessageFreedom BCS (100): blocked
            // falls into $09:A000-A006 STZ STZ BRA (86), free takes the BCS
            // (+6) into $09:A008-A023 (382, JSL Main_ShowTextMessage
            // included).
            if sign16(timer) {
                crate::cycle_ledger::charge(100 + 100);
                if !self.follower_validate_message_freedom() {
                    crate::cycle_ledger::charge(86);
                    self.clear_shared_message_timer();
                } else {
                    crate::cycle_ledger::charge(6 + 382);
                    let j = j as usize;
                    self.start_shared_message_timer(TAGALONG_MESSAGE_TIMERS[j]);
                    self.dialogue_message_index_mut().set_value(TAGALONG_MSG[j]);
                    self.Tagalong_Main_ShowTextMessage();
                }
            } else {
                crate::cycle_ledger::charge(100 + 6);
            }
        }
        if j >= 0 {
            // Tagalong_5_14 $09:A024-A028 SEP CPY #$00 BNE (54): j == 0 falls
            // into $09:A02A RTS (42), else the BNE is taken (+6) into
            // Follower_NoTimedMessage.
            crate::cycle_ledger::charge(if j != 0 { 54 + 6 } else { 54 + 42 });
        }
        if j != 0 {
            self.follower_no_timed_message();
        }
    }

    pub(super) fn follower_no_timed_message(&mut self) {
        // Cycle ledger: Follower_NoTimedMessage $09:A02B is a branch target
        // (charges into Follower_Main's scope). $09:A02B-A031 SEP LDA long
        // dropped BEQ (78): a dropped follower runs $09:A033 BRL and
        // $09:A0DE BRL Follower_NotFollowing (60; NotFollowing itself is not
        // priced here).
        if self.game_state.sprites.follower_runtime.dropped() != 0 {
            crate::cycle_ledger::charge(78 + 30 + 30);
            self.follower_not_following();
            return;
        }
        // BEQ taken (+6), $09:A036-A03C LDA long CMP #$0c BNE (72).
        crate::cycle_ledger::charge(78 + 6 + 72);
        if self.game_state.sprites.follower_runtime.indicator() == 12 {
            // $09:A03E-A040 LDA $4d : BNE (40): an auxiliary state takes the
            // BNE (+6) into $09:A04C BRL Follower_CheckGameMode (30); else
            // $09:A042 BRA (22) into the drop-condition chain, which is not
            // priced (the translation evaluates it in a different order).
            if self.game_state.player.follower_link.has_auxiliary_state() {
                crate::cycle_ledger::charge(40 + 6 + 30);
            } else {
                crate::cycle_ledger::charge(40 + 22);
            }
            if !self.game_state.player.follower_link.has_auxiliary_state()
                && self.follower_can_drop()
            {
                self.follower_drop();
                return;
            }
        } else if self.game_state.sprites.follower_runtime.indicator() == 13 {
            // BNE taken (+6), $09:A044-A04A LDA long CMP #$0d BEQ taken (72
            // + 6), $09:A04F-A053 LDA $4d CMP #$02 BEQ (56, +6 taken) and
            // $09:A055-A059 LDA $5b CMP #$02 BEQ (56, +6 taken) into the
            // drop; otherwise the drop-condition chain (not priced).
            crate::cycle_ledger::charge(6 + 72 + 6);
            if self.game_state.player.follower_link.auxiliary_state() == 2 {
                crate::cycle_ledger::charge(56 + 6);
            } else if self.game_state.player.follower_link.near_pit_state_is(2) {
                crate::cycle_ledger::charge(56 + 56 + 6);
            } else {
                crate::cycle_ledger::charge(56 + 56);
            }
            if self.game_state.player.follower_link.auxiliary_state() == 2
                || self.game_state.player.follower_link.near_pit_state_is(2)
            {
                self.follower_drop();
                return;
            }
            if self.follower_can_drop() {
                self.follower_drop();
                return;
            }
        } else {
            // BNE taken (+6), $09:A044-A04A not taken (72), $09:A04C BRL
            // Follower_CheckGameMode (30).
            crate::cycle_ledger::charge(6 + 72 + 30);
        }
        self.follower_check_game_mode();
    }

    fn follower_can_drop(&self) -> bool {
        self.game_state.frame.submodule == 0
            && self.game_state.player.follower_link.can_drop_follower()
            && self
                .game_state
                .sprites
                .follower_runtime
                .appearance_none_flag()
                == 0
            && self
                .game_state
                .sprites
                .follower_runtime
                .hookshot_interlock_is_clear()
            && self
                .tagalong_slot(self.game_state.sprites.follower_runtime.data_index() as usize)
                .z_signed()
                <= 0
            && self.game_state.player.follower_link.filtered_joypad_l() & 0x80 != 0
    }

    fn follower_drop(&mut self) {
        // Cycle ledger ($09:A084 onward, inside Follower_Main's scope):
        // $09:A084-A08A LDA long CMP #$0d BNE (72, +6 taken for other
        // followers); the super bomb runs $09:A08C-A08E LDA $1b : BNE (40,
        // +6 taken indoors) and outdoors $09:A090-A094 CMP #$08 BEQ (56),
        // $09:A096 CMP #$09 BEQ (32), $09:A09A CMP #$0a BEQ (32) with the
        // matching medallion state taking its branch (+6) to
        // Follower_CheckGameMode, else $09:A09E-A0A5 (96). Then
        // $09:A0A8-A0DA (552) and $09:A0DE BRL Follower_NotFollowing (30).
        if self.game_state.sprites.follower_runtime.indicator() == 13 {
            if self.game_state.world.location.indoor_flag() != 0 {
                crate::cycle_ledger::charge(72 + 40 + 6);
            } else {
                let handler = self.game_state.player.follower_link.handler_state();
                crate::cycle_ledger::charge(match handler {
                    0x08 => 72 + 40 + 56 + 6,
                    0x09 => 72 + 40 + 56 + 32 + 6,
                    0x0a => 72 + 40 + 56 + 32 + 32 + 6,
                    _ => 72 + 40 + 56 + 32 + 32 + 96,
                });
            }
        } else {
            crate::cycle_ledger::charge(72 + 6);
        }
        if self.game_state.sprites.follower_runtime.indicator() == 13
            && self.game_state.world.location.indoor_flag() == 0
        {
            if self.game_state.player.follower_link.is_using_medallion() {
                self.follower_check_game_mode();
                return;
            }
            self.set_super_bomb_indicator_timer(3);
            self.set_super_bomb_indicator_counter(0xbb);
        }
        crate::cycle_ledger::charge(552 + 30);
        self.follower_state_mut().set_dropped(128);
        self.follower_state_mut().set_reacquire_timer_low(64);
        let k = self.game_state.sprites.follower_runtime.data_index() as usize;
        let y = self.tagalong_y(k);
        let x = self.tagalong_x(k);
        let floor = self.game_state.player.follower_link.floor();
        let indoor = self.game_state.world.location.indoor_flag();
        self.follower_state_mut().set_saved_y(y);
        self.follower_state_mut().set_saved_x(x);
        self.follower_state_mut().set_saved_floor(floor);
        self.follower_state_mut().set_saved_indoor_flag(indoor);
        self.follower_not_following();
    }

    pub(super) fn follower_check_game_mode(&mut self) {
        // Cycle ledger: Follower_CheckGameMode $09:A0E1 is a branch target
        // (charges into Follower_Main's scope). Tagalong_IsFollowing is the
        // test chain $09:A0E1-A102: SEP LDA $02e4 BNE (70), LDX $10 LDY $11
        // CPY #$0a BEQ (80), CPX #$09 BNE (32) [CPY #$23 BEQ (32)], CPX #$0e
        // BNE (32) [CPY #$01 BEQ (32), CPY #$02 BNE (32)]; a taken exit (+6)
        // lands on $09:A104 BRL $09:A18C (30). Then $09:A107-A10B LDA $30 ORA
        // $31 BEQ (64, +6 taken when Link is still).
        {
            let main = self.game_state.frame.main_module;
            let sub = self.game_state.frame.submodule;
            let link = self.game_state.player.follower_link;
            let cost = if link.is_immobilized() {
                70 + 6 + 30
            } else if sub == 10 {
                70 + 80 + 6 + 30
            } else if main == 9 {
                if sub == 0x23 {
                    70 + 80 + 32 + 32 + 6 + 30
                } else {
                    70 + 80 + 32 + 32 + 32 + 6
                }
            } else if main == 14 {
                if sub == 1 {
                    70 + 80 + 32 + 6 + 32 + 32 + 6 + 30
                } else if sub == 2 {
                    70 + 80 + 32 + 6 + 32 + 32 + 32 + 30
                } else {
                    70 + 80 + 32 + 6 + 32 + 32 + 32 + 6
                }
            } else {
                70 + 80 + 32 + 6 + 32 + 6
            };
            crate::cycle_ledger::charge(cost);
            if self.tagalong_is_following() {
                crate::cycle_ledger::charge(if link.is_moving() { 64 } else { 64 + 6 });
            }
        }
        if self.tagalong_is_following() && self.game_state.player.follower_link.is_moving() {
            let mut k = self
                .game_state
                .sprites
                .follower_runtime
                .tail_write_index()
                .wrapping_add(1);
            // $09:A10D-A113 LDX $02d3 INX CPX #$14 BNE (78, +6 taken below
            // 20, else $09:A115 LDX #$00 16); $09:A117-A11E STX LDA $24 CMP
            // #$f0 BCC (88, +6 taken below $f0, else $09:A120 LDA #$00 16);
            // $09:A122-A161 (798, its final BNE taken +6 unless swimming).
            crate::cycle_ledger::charge(if k == 20 { 78 + 16 } else { 78 + 6 });
            crate::cycle_ledger::charge(
                if self.game_state.player.follower_link.z_low() >= 0xf0 {
                    88 + 16
                } else {
                    88 + 6
                },
            );
            crate::cycle_ledger::charge(798);
            if k == 20 {
                k = 0;
            }
            self.follower_state_mut().set_tail_write_index(k);
            let link = self.game_state.player.follower_link;
            let z = link.z_for_follow();
            let k = k as usize;
            let y = link.y().wrapping_sub(z as u16);
            let x = link.x();
            let mut layerbits = link.facing_layer_bits() | link.floor_layer_bits();
            // Swimming: $09:A163 LDY #$20 : BRA (38) into $09:A185 (84).
            // Else $09:A167-A169 CMP #$13 BNE (32, +6 taken unless the
            // hookshot), hookshot: $09:A16B-A16E LDA $037e BEQ (48, +6 taken
            // when clear) else $09:A170 (86); then $09:A178-A17D LDY #$80 LDA
            // $0351 BEQ (64, +6 taken with no surface effect), else CMP #$01
            // BEQ (32, +6 taken for ripples) or $09:A183 LDY #$40 (16), then
            // $09:A185-A189 TYA ORA STA (84).
            if link.is_swimming() {
                crate::cycle_ledger::charge(38 + 84);
                layerbits |= 0x20;
            } else {
                crate::cycle_ledger::charge(6 + 32);
                if link.is_hookshot() {
                    crate::cycle_ledger::charge(if link.has_hookshot_interlock() {
                        48 + 86
                    } else {
                        48 + 6
                    });
                } else {
                    crate::cycle_ledger::charge(6);
                }
                if link.is_hookshot()
                    && self
                        .game_state
                        .player
                        .follower_link
                        .has_hookshot_interlock()
                {
                    layerbits |= 0x10;
                }
                let surface_effect = self
                    .game_state
                    .player
                    .follower_link
                    .water_ripple_or_grass_state();
                crate::cycle_ledger::charge(match surface_effect {
                    0 => 64 + 6,
                    1 => 64 + 32 + 6 + 84,
                    _ => 64 + 32 + 16 + 84,
                });
                if surface_effect != 0 {
                    layerbits |= if surface_effect == 1 { 0x80 } else { 0x40 };
                }
            }
            let mut follower = self.tagalong_slot_mut(k);
            follower.set_z(z);
            follower.set_position(x, y);
            follower.set_layer_bits(layerbits);
        }
        // $09:A18C-A193 LDA long DEC ASL TAX JMP (abs,x) (128): the
        // per-follower handler runs inside Follower_Main's scope (unpriced).
        crate::cycle_ledger::charge(128);
        match self.game_state.sprites.follower_runtime.indicator() {
            2 | 4 => self.follower_old_man(),
            3 | 11 => self.follower_old_man_unused(),
            5 | 14 => self.follower_basic_mover(),
            _ => self.follower_basic_mover(),
        }
    }

    pub(super) fn follower_basic_mover(&mut self) {
        if !self.tagalong_is_following() {
            self.tagalong_draw();
            return;
        }
        self.follower_handle_trigger();
        if self.game_state.sprites.follower_runtime.indicator() == 10
            && self.game_state.player.follower_link.has_auxiliary_state()
            && self.game_state.player.follower_link.blink_countdown() != 0
        {
            let k = if self
                .game_state
                .sprites
                .follower_runtime
                .data_index()
                .wrapping_add(1)
                == 20
            {
                0
            } else {
                self.game_state
                    .sprites
                    .follower_runtime
                    .data_index()
                    .wrapping_add(1)
            };
            self.kiki_spawn_handler_b(k as usize);
            self.follower_state_mut().set_indicator(0);
            return;
        }
        if self.game_state.sprites.follower_runtime.indicator() == 6
            && self.game_state.world.location.dungeon_room() == 0x0ac
            && self.saved_room_flags(101) & 0x100 != 0
            && self.follower_check_blind_trigger()
        {
            let k = self.game_state.sprites.follower_runtime.data_index() as usize;
            let x = self.tagalong_x(k);
            let y = self.tagalong_y(k);
            self.follower_state_mut().set_indicator(0);
            self.blind_spawn_from_maiden(x, y);
            self.dungeon_environment_mut()
                .increment_trapdoors_down_low();
            self.dungeon_doors_mut().clear_current_door_pos();
            self.dungeon_doors_mut().clear_door_animation_step();
            self.set_submodule(5);
            self.set_music_control(21);
            return;
        }
        if self
            .game_state
            .sprites
            .follower_runtime
            .hookshot_interlock_is_clear()
        {
            if self.game_state.player.follower_link.is_hookshot()
                && self
                    .game_state
                    .player
                    .follower_link
                    .has_hookshot_interlock()
            {
                self.follower_state_mut().set_hookshot_interlock();
                self.advance_follower_tail();
                self.tagalong_draw();
                return;
            }
        } else {
            if self.game_state.player.follower_link.is_hookshot() {
                self.advance_follower_tail();
                self.tagalong_draw();
                return;
            }
            if self
                .game_state
                .sprites
                .follower_runtime
                .hookshot_release_tail_index()
                != self.game_state.sprites.follower_runtime.data_index()
            {
                self.follower_state_mut()
                    .advance_data_index_wrapping_at_20();
                self.tagalong_draw();
                return;
            }
            self.follower_state_mut().clear_hookshot_interlock();
        }
        let k = self.game_state.sprites.follower_runtime.data_index() as usize;
        if self.tagalong_slot(k).is_above_ground() {
            if self.game_state.sprites.follower_runtime.tail_write_index() != k as u8 {
                self.follower_state_mut()
                    .advance_data_index_wrapping_at_20();
                self.tagalong_draw();
                return;
            }
            let link = self.game_state.player.follower_link;
            let y = link.y();
            let x = link.x();
            let mut follower = self.tagalong_slot_mut(k);
            follower.set_z(0);
            follower.set_position(x, y);
        }
        if self.game_state.player.follower_link.is_moving() {
            self.advance_follower_tail();
        }
        self.tagalong_draw();
    }

    fn advance_follower_tail(&mut self) {
        let mut t = self
            .game_state
            .sprites
            .follower_runtime
            .tail_write_index()
            .wrapping_sub(15);
        if sign8(t) {
            t = t.wrapping_add(20);
        }
        if t == self.game_state.sprites.follower_runtime.data_index() {
            self.follower_state_mut()
                .advance_data_index_wrapping_at_20();
        }
    }

    pub(super) fn follower_not_following(&mut self) {
        if self.game_state.sprites.follower_runtime.saved_indoor_flag()
            != self.game_state.world.location.indoor_flag()
        {
            return;
        }
        if !self.game_state.player.follower_link.is_running()
            && !self.follower_check_proximity_to_link()
        {
            self.follower_initialize();
            let indoor = self.game_state.world.location.indoor_flag();
            self.follower_state_mut().set_saved_indoor_flag(indoor);
            if self.game_state.sprites.follower_runtime.indicator() == 13 {
                self.set_super_bomb_indicator_timer(254);
                self.set_super_bomb_indicator_counter(0);
            }
            self.follower_state_mut().set_dropped(0);
            self.tagalong_draw();
        } else {
            if self.game_state.sprites.follower_runtime.indicator() == 13
                && self.game_state.world.location.indoor_flag() == 0
                && self.hud_state().super_bomb_indicator_timer() == 0
            {
                if self.AncillaAdd_SuperBombExplosion(0x3a, 0).is_some() {
                    self.follower_state_mut().set_dropped(0);
                    if self
                        .game_state
                        .enhanced_features
                        .has(FEATURES0_MISC_BUG_FIXES_TAGALONG)
                    {
                        self.follower_state_mut().set_indicator(0);
                        return;
                    }
                } else {
                    self.set_super_bomb_indicator_counter(1);
                }
            }
            self.follower_do_layers();
        }
    }

    pub(super) fn follower_old_man(&mut self) {
        if !self.tagalong_is_following() {
            self.tagalong_draw();
            return;
        }
        if self.game_state.player.follower_link.speed_setting() != 4 {
            self.follower_link_state_mut().set_speed_setting(12);
        }
        self.follower_handle_trigger();
        if self.game_state.sprites.follower_runtime.indicator() == 0 {
            return;
        } else if self.game_state.sprites.follower_runtime.indicator() == 4 {
            let k = self.game_state.sprites.follower_runtime.data_index() as usize;
            if self.tagalong_slot(k).is_above_ground()
                && self.game_state.sprites.follower_runtime.tail_write_index()
                    != self.game_state.sprites.follower_runtime.data_index()
            {
                self.follower_state_mut()
                    .advance_data_index_wrapping_at_20();
                self.tagalong_draw();
                return;
            }
        } else {
            if self
                .game_state
                .player
                .follower_link
                .should_transform_old_man_from_recoil()
            {
                if self.game_state.sprites.follower_runtime.tail_write_index()
                    == self.game_state.sprites.follower_runtime.data_index()
                {
                    // C asserts here because the follower X value is undefined.
                    panic!("follower_old_man assert");
                }
                self.transform_old_man();
                return;
            }
            if self
                .game_state
                .player
                .follower_link
                .should_transform_old_man_from_auxiliary_state()
            {
                self.transform_old_man();
                return;
            }
        }
        if self.game_state.player.follower_link.is_moving() {
            let mut t = self
                .game_state
                .sprites
                .follower_runtime
                .tail_write_index()
                .wrapping_sub(20);
            if sign8(t) {
                t = t.wrapping_add(20);
            }
            if t == self.game_state.sprites.follower_runtime.data_index() {
                self.follower_state_mut()
                    .advance_data_index_wrapping_at_20();
            }
        } else if self.game_state.frame.frame_counter & 3 == 0
            && self.game_state.sprites.follower_runtime.tail_write_index()
                != self.game_state.sprites.follower_runtime.data_index()
        {
            let mut t = self
                .game_state
                .sprites
                .follower_runtime
                .tail_write_index()
                .wrapping_sub(9);
            if sign8(t) {
                t = t.wrapping_add(20);
            }
            if t != self.game_state.sprites.follower_runtime.data_index() {
                self.follower_state_mut()
                    .advance_data_index_wrapping_at_20();
            }
        }
        self.tagalong_draw();
    }

    fn transform_old_man(&mut self) {
        let indicator = TAGALONG_RELEASE_INDICATOR_BY_FOLLOWER
            [self.game_state.sprites.follower_runtime.indicator() as usize];
        self.follower_state_mut().set_indicator(indicator);
        self.follower_state_mut().set_reacquire_timer_low(64);
        let k = self.game_state.sprites.follower_runtime.data_index() as usize;
        let y = self.tagalong_y(k);
        let x = self.tagalong_x(k);
        let floor = self.game_state.player.follower_link.floor();
        self.follower_state_mut().set_saved_y(y);
        self.follower_state_mut().set_saved_x(x);
        self.follower_state_mut().set_saved_floor(floor);
        self.follower_old_man_unused();
    }

    pub(super) fn follower_old_man_unused(&mut self) {
        self.follower_link_state_mut().set_speed_setting(16);
        if self.game_state.player.follower_link.can_reacquire_old_man() {
            self.follower_link_state_mut().set_speed_setting(0);
            if !self.game_state.player.follower_link.is_hookshot()
                && !self.follower_check_proximity_to_link()
            {
                self.follower_initialize();
                let indicator = TAGALONG_SLOWDOWN_INDICATOR_BY_FOLLOWER
                    [self.game_state.sprites.follower_runtime.indicator() as usize];
                self.follower_state_mut().set_indicator(indicator);
                return;
            }
        }
        self.follower_do_layers();
    }

    pub(super) fn follower_do_layers(&mut self) {
        let priority = FollowerLinkState::oam_priority_for_floor_value(
            self.game_state.sprites.follower_runtime.saved_floor(),
        );
        self.oam_state_mut()
            .set_priority_word((priority as u16) << 8);
        let a = if self.game_state.sprites.follower_runtime.indicator() == 12
            || self.game_state.sprites.follower_runtime.indicator() == 13
        {
            2
        } else {
            1
        };
        self.follower_animate_movement_preserved(
            a,
            self.game_state.sprites.follower_runtime.saved_x(),
            self.game_state.sprites.follower_runtime.saved_y(),
        );
    }

    pub(super) fn follower_check_proximity_to_link(&mut self) -> bool {
        self.follower_state_mut().decrement_reacquire_timer_low();
        if !sign8(
            self.game_state
                .sprites
                .follower_runtime
                .reacquire_timer_low(),
        ) {
            return true;
        }
        self.follower_state_mut().set_reacquire_timer_low(0);
        let y = self.game_state.sprites.follower_runtime.saved_y();
        let x = self.game_state.sprites.follower_runtime.saved_x();
        let link = self.game_state.player.follower_link;
        let ly = link.y();
        let lx = link.x();
        y.wrapping_sub(1) >= ly
            || y.wrapping_add(19) < ly
            || x.wrapping_sub(1) >= lx
            || x.wrapping_add(19) < lx
    }

    pub(super) fn follower_handle_trigger(&mut self) {
        if self.game_state.frame.submodule != 0 {
            return;
        }
        let (infos, start, end) = if self.game_state.world.location.is_indoors() {
            let room = self.game_state.world.location.dungeon_room();
            let Some(j) = TAGALONG_INDOOR_ROOMS.iter().position(|&v| v == room) else {
                return;
            };
            (
                &TAGALONG_INDOOR_INFOS[..],
                TAGALONG_INDOOR_OFFSETS[j] as usize,
                TAGALONG_INDOOR_OFFSETS[j + 1] as usize,
            )
        } else {
            let screen = self.game_state.world.location.overworld_screen();
            let Some(j) = TAGALONG_OUTDOOR_ROOMS.iter().position(|&v| v == screen) else {
                return;
            };
            (
                &TAGALONG_OUTDOOR_INFOS[..],
                TAGALONG_OUTDOOR_OFFSETS[j] as usize,
                TAGALONG_OUTDOOR_OFFSETS[j + 1] as usize,
            )
        };
        let st = if self
            .game_state
            .sprites
            .follower_runtime
            .data_index()
            .wrapping_add(1)
            >= 20
        {
            0
        } else {
            self.game_state
                .sprites
                .follower_runtime
                .data_index()
                .wrapping_add(1)
        };
        for info in &infos[start..end] {
            if info.tagalong == self.game_state.sprites.follower_runtime.indicator()
                && self.follower_check_for_trigger(info)
            {
                if info.bit & self.game_state.sprites.follower_runtime.event_flags() != 0 {
                    return;
                }
                self.follower_state_mut().or_event_flags(info.bit);
                self.dialogue_message_index_mut().set_value(info.msg);
                self.stage_rescue_follower_message_obj_scanout(info.bit, info.msg);
                if info.msg == 0xffff {
                    if info.bit & 3 == 0 {
                        self.kiki_revert_to_sprite(st as usize);
                    } else if self.game_state.world.overworld.event_info.event_info(self.game_state.world.location.overworld_screen_index() as usize)
                        & 1
                        == 0
                    {
                        self.kiki_spawn_handler_a(st as usize);
                    }
                    return;
                }
                if info.msg == 0x9d {
                    self.OldMan_RevertToSprite(st as usize);
                } else if info.msg == 0x28 {
                    self.follower_state_mut().set_indicator(0);
                }
                self.Tagalong_Main_ShowTextMessage();
                return;
            }
        }
    }

    pub(super) fn tagalong_draw(&mut self) {
        if self
            .game_state
            .sprites
            .follower_runtime
            .appearance_none_flag()
            != 0
        {
            return;
        }
        let current_follower =
            self.tagalong_slot(self.game_state.sprites.follower_runtime.data_index() as usize);
        let priority = if current_follower.z() != 0 && !self.game_state.world.location.is_indoors()
        {
            0x20
        } else if self.game_state.frame.submodule == 14 {
            self.game_state
                .player
                .follower_link
                .oam_priority_for_floor()
        } else {
            (current_follower.layer_bits() & 0x0c) << 2
        };
        self.oam_state_mut()
            .set_priority_word((priority as u16) << 8);
        let k = if sign8(self.game_state.sprites.follower_runtime.data_index()) {
            0
        } else {
            self.game_state.sprites.follower_runtime.data_index() as usize
        };
        let x = self.tagalong_x(k);
        let y = self.tagalong_y(k);
        let a = self.tagalong_slot(k).layer_bits();
        self.follower_animate_movement_preserved(a, x, y);
    }

    #[rustfmt::skip]
    fn set_oam_follower_at(&mut self, oam: usize, x: u16, y: u16, charnum: u8, flags: u8, mut big: u8) {
        let visible = x.wrapping_add(0x80) < 0x180 && {
            big |= ((x >> 8) & 1) as u8;
            y.wrapping_add(0x10) < 0x100
        };
        self.oam_state_mut()
            .write_entry(oam, x as u8, if visible { y as u8 } else { 0xf0 }, charnum, flags);
        self.oam_state_mut()
            .set_extended_byte((oam - OAM_BUF_TAGALONG) / 4, big);
    }

    pub(super) fn follower_animate_movement_preserved(&mut self, ain: u8, xin: u16, yin: u16) {
        let mut yt = 0;
        let av;
        let mut sc = 0;
        if (ain >> 2 & 8) != 0
            && (self.game_state.sprites.follower_runtime.indicator() == 6
                || self.game_state.sprites.follower_runtime.indicator() == 1)
        {
            yt = 8;
            av = if self.game_state.player.swim_acceleration.acceleration(0) != 0 {
                (self.game_state.frame.frame_counter >> 1) & 4
            } else {
                (self.game_state.frame.frame_counter >> 2) & 4
            };
        } else if self.game_state.frame.submodule == 8
            || self.game_state.frame.submodule == 14
            || self.game_state.frame.submodule == 16
        {
            av = if self.game_state.player.follower_link.is_running() {
                self.game_state.frame.frame_counter & 4
            } else {
                (self.game_state.frame.frame_counter >> 1) & 4
            };
        } else if self.game_state.sprites.follower_runtime.indicator() == 11 {
            av = (self.game_state.frame.frame_counter >> 1) & 4;
        } else if ((self.game_state.sprites.follower_runtime.indicator() == 12
            || self.game_state.sprites.follower_runtime.indicator() == 13)
            && self.game_state.sprites.follower_runtime.dropped() != 0)
            || self.game_state.player.follower_link.is_immobilized()
            || self.game_state.frame.submodule == 10
            || (self.game_state.frame.main_module == 9 && self.game_state.frame.submodule == 0x23)
            || (self.game_state.frame.main_module == 14
                && (self.game_state.frame.submodule == 1 || self.game_state.frame.submodule == 2))
            || !self.game_state.player.follower_link.is_moving()
        {
            av = 4;
            sc = 4;
        } else {
            av = if self.game_state.player.follower_link.is_running() {
                self.game_state.frame.frame_counter & 4
            } else {
                (self.game_state.frame.frame_counter >> 1) & 4
            };
        }
        let frame = (ain & 3).wrapping_add(av).wrapping_add(yt) as usize;
        let link_y = self.game_state.player.follower_link.y();
        let spr_offs = if (link_y == yin && (ain & 3) == 0) || link_y < yin {
            TAGALONG_DRAW_SPR_OFFS0[self.game_state.oam.sprite_sorting_offset_index()] >> 2
        } else {
            TAGALONG_DRAW_SPR_OFFS1[self.game_state.oam.sprite_sorting_offset_index()] >> 2
        } as usize;
        self.oam_state_mut()
            .set_current_extended_pointer(0x0a20 + spr_offs as u16);
        self.oam_state_mut()
            .set_current_pointer(0x0800 + (spr_offs as u16) * 4);
        let mut oam = self.game_state.oam.current_pointer_usize();
        let scrolly = yin.wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        let scrollx = xin.wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
        let mut skip_first_sprites = false;
        let mut sk_index = 0usize;
        if self.game_state.sprites.follower_runtime.indicator() == 1
            || self.game_state.sprites.follower_runtime.indicator() == 6
            || (ain & 0x20) == 0
        {
            if ain & 0xc0 == 0 {
                skip_first_sprites = true;
            } else if (ain & 0x80) == 0 {
                sk_index += 12;
                if sc != 0 {
                    self.follower_state_mut().clear_draw_anim_frame();
                } else if self.game_state.frame.frame_counter & 7 == 0 {
                    self.follower_state_mut()
                        .increment_and_cycle_draw_anim_frame();
                }
            } else if self.game_state.frame.frame_counter & 7 == 0 {
                self.follower_state_mut()
                    .increment_and_cycle_draw_anim_frame();
            }
        } else if self.game_state.frame.frame_counter & 7 == 0 {
            self.follower_state_mut()
                .increment_and_cycle_draw_anim_frame();
        }
        if !skip_first_sprites {
            sk_index += self.game_state.sprites.follower_runtime.draw_anim_frame() as usize * 4;
            self.set_oam_follower_at(
                oam,
                scrollx,
                scrolly.wrapping_add(16),
                TAGALONG_DRAW_SPR_INFO0[sk_index],
                TAGALONG_DRAW_SPR_INFO0[sk_index + 1],
                0,
            );
            self.set_oam_follower_at(
                oam + 4,
                scrollx.wrapping_add(8),
                scrolly.wrapping_add(16),
                TAGALONG_DRAW_SPR_INFO0[sk_index + 2],
                TAGALONG_DRAW_SPR_INFO0[sk_index + 3],
                0,
            );
            oam += 8;
        }
        let mut pal =
            TAGALONG_DRAW_PALS[self.game_state.sprites.follower_runtime.indicator() as usize];
        if pal == 7 && self.game_state.sprites.follower_runtime.palette_swap_flag() != 0 {
            pal = 0;
        }
        if self.game_state.sprites.follower_runtime.indicator() == 13 {
            let colorful = if self
                .game_state
                .enhanced_features
                .has(FEATURES0_MISC_BUG_FIXES_TAGALONG)
            {
                self.hud_state().super_bomb_indicator_timer() <= 1
            } else {
                self.hud_state().super_bomb_indicator_timer() == 1
            };
            if colorful {
                pal = self.game_state.frame.frame_counter & 7;
            }
        }
        let sprd = TAGALONG_DRAW_SPR_XY[frame
            + (TAGALONG_DRAW_OFFS[self.game_state.sprites.follower_runtime.indicator() as usize]
                >> 3) as usize];
        let sprf = TAGALONG_DMA_AND_FLAGS[frame];
        if self.game_state.sprites.follower_runtime.indicator() != 12
            && self.game_state.sprites.follower_runtime.indicator() != 13
        {
            self.set_oam_follower_at(
                oam,
                scrollx.wrapping_add(sprd.x1 as i16 as u16),
                scrolly.wrapping_add(sprd.y1 as i16 as u16),
                0x20,
                (sprf.flags & 0xf0) | (pal << 1) | (self.game_state.oam.priority_word() >> 8) as u8,
                2,
            );
            oam += 4;
            self.set_sprite_dma_head_pointer(sprf.dma6);
        }
        self.set_oam_follower_at(
            oam,
            scrollx.wrapping_add(sprd.x2 as i16 as u16),
            scrolly.wrapping_add(sprd.y2 as i16 as u16).wrapping_add(8),
            0x22,
            ((sprf.flags & 0x0f) << 4)
                | (pal << 1)
                | (self.game_state.oam.priority_word() >> 8) as u8,
            2,
        );
        self.set_sprite_dma_body_pointer(sprf.dma7);
    }

    pub(super) fn follower_check_for_trigger(&self, info: &TagalongMessageInfo) -> bool {
        let link = self.game_state.player.follower_link;
        let mut x = link
            .x()
            .wrapping_add(12)
            .wrapping_sub(info.x.wrapping_add(8));
        let mut y = link
            .y()
            .wrapping_add(12)
            .wrapping_sub(info.y.wrapping_add(8));
        if sign16(x) {
            x = 0u16.wrapping_sub(x);
        }
        if sign16(y) {
            y = 0u16.wrapping_sub(y);
        }
        x < 24 && y < 28
    }

    pub(super) fn follower_disable(&mut self) {
        if self.game_state.sprites.follower_runtime.indicator() == 9
            || self.game_state.sprites.follower_runtime.indicator() == 10
        {
            self.follower_state_mut().set_indicator(0);
        }
    }

    pub(super) fn blind_spawn_from_maiden(&mut self, x: u16, y: u16) {
        let k = 0;
        let mut maiden = self.sprite_slot_view_mut(k);
        maiden.set_state(9);
        maiden.set_sprite_type(206);
        self.Tagalong_Sprite_SetX(k, x);
        self.Tagalong_Sprite_SetY(k, y.wrapping_sub(16));
        self.SpritePrep_LoadProperties(k);
        let mut maiden = self.sprite_slot_view_mut(k);
        maiden.set_delay_aux2(192);
        maiden.set_graphics(21);
        maiden.set_direction(2);
        maiden.set_ignore_projectile(2);
        self.dungeon_savegame_state_mut()
            .or_savegame_state_bits(0x2000);
        self.sprite_system_mut().set_blind_head_anim_counter(0);
    }

    pub(super) fn kiki_revert_to_sprite(&mut self, k: usize) {
        if let Some(j) = self.kiki_spawn_handler_monke(k) {
            self.sprite_slot_view_mut(j).set_subtype2(1);
        }
        self.follower_state_mut().set_indicator(0);
    }

    pub(super) fn kiki_spawn_handler_monke(&mut self, k: usize) -> Option<usize> {
        let j = self.Tagalong_Sprite_SpawnDynamically(k, 0xb6)?;
        let layer = self.tagalong_slot(k).direction();
        let mut monke = self.sprite_slot_view_mut(j);
        monke.set_head_direction(layer);
        monke.set_direction(layer);
        let x = self.tagalong_x(k);
        let y = self.tagalong_y(k);
        self.Tagalong_Sprite_SetX(j, x.wrapping_add(2));
        self.Tagalong_Sprite_SetY(j, y.wrapping_add(2));
        let floor = self.game_state.player.follower_link.floor();
        let mut monke = self.sprite_slot_view_mut(j);
        monke.set_floor(floor);
        monke.set_ignore_projectile(1);
        monke.set_floor(2);
        self.follower_link_state_mut().set_speed_setting(0);
        Some(j)
    }

    pub(super) fn kiki_spawn_handler_a(&mut self, k: usize) {
        if let Some(j) = self.kiki_spawn_handler_monke(k) {
            self.sprite_slot_view_mut(j).set_subtype2(2);
        }
    }

    pub(super) fn kiki_spawn_handler_b(&mut self, k: usize) {
        if let Some(j) = self.kiki_spawn_handler_monke(k) {
            let mut monke = self.sprite_slot_view_mut(j);
            monke.set_z(1);
            monke.set_z_velocity(16);
            monke.set_subtype2(3);
        }
        self.follower_state_mut().set_indicator(0);
    }

    fn tagalong_x(&self, k: usize) -> u16 {
        self.tagalong_slot(k).x()
    }

    fn tagalong_y(&self, k: usize) -> u16 {
        self.tagalong_slot(k).y()
    }

    fn Tagalong_Main_ShowTextMessage(&mut self) {
        if self.game_state.frame.main_module != 14 {
            self.clear_tile_interaction_shared_flag();
            self.messaging_state_mut().clear_module();
            self.set_submodule(2);
            self.save_main_module_for_menu();
            self.set_main_module(14);
        }
    }

    fn Ancilla_GetX(&self, k: usize) -> u16 {
        self.ancilla_slot_view(k).x()
    }

    fn Ancilla_GetY(&self, k: usize) -> u16 {
        self.ancilla_slot_view(k).y()
    }

    fn Ancilla_SetXY(&mut self, k: usize, x: u16, y: u16) {
        let mut ancilla = self.ancilla_slot_view_mut(k);
        ancilla.set_x(x);
        ancilla.set_y(y);
    }

    fn Ancilla_MoveX(&mut self, k: usize) {
        self.ancilla_slot_view_mut(k).move_x();
    }

    fn Ancilla_MoveY(&mut self, k: usize) {
        self.ancilla_slot_view_mut(k).move_y();
    }

    fn Ancilla_ProjectSpeedTowardsPlayer(&self, k: usize, vel: u8) -> ProjectSpeedRet {
        if vel == 0 {
            return ProjectSpeedRet {
                x: 0,
                y: 0,
                xdiff: 0,
                ydiff: 0,
            };
        }
        let link = self.game_state.player.follower_link;
        let below = link.y().wrapping_sub(self.Ancilla_GetY(k));
        let right = link.x().wrapping_sub(self.Ancilla_GetX(k));
        let below_b = below as u8;
        let below_a = sign16(below) as u8;
        let right_b = right as u8;
        let right_a = sign16(right) as u8;
        let mut r12 = if sign8(below_b) {
            0u8.wrapping_sub(below_b)
        } else {
            below_b
        };
        let mut r13 = if sign8(right_b) {
            0u8.wrapping_sub(right_b)
        } else {
            right_b
        };
        let mut swapped = false;
        if r13 < r12 {
            swapped = true;
            core::mem::swap(&mut r12, &mut r13);
        }
        let mut xvel = vel;
        let mut yvel = 0u8;
        let mut t = 0u8;
        let mut n = vel;
        loop {
            t = t.wrapping_add(r12);
            if t >= r13 {
                t = t.wrapping_sub(r13);
                yvel = yvel.wrapping_add(1);
            }
            n = n.wrapping_sub(1);
            if n == 0 {
                break;
            }
        }
        if swapped {
            core::mem::swap(&mut xvel, &mut yvel);
        }
        ProjectSpeedRet {
            x: if right_a != 0 {
                0u8.wrapping_sub(xvel)
            } else {
                xvel
            },
            y: if below_a != 0 {
                0u8.wrapping_sub(yvel)
            } else {
                yvel
            },
            xdiff: right_b,
            ydiff: below_b,
        }
    }

    fn AncillaAdd_SuperBombExplosion(&mut self, a: u8, y: u8) -> Option<usize> {
        let k = self.ancilla_add_simple(a, y)?;
        let mut explosion = self.ancilla_slot_view_mut(k);
        explosion.set_r(0);
        explosion.set_step(0);
        explosion.set_work_byte_25(0);
        explosion.set_l(0);
        explosion.set_work_byte_3(6);
        explosion.set_item_to_link(1);
        let j = self.game_state.sprites.follower_runtime.data_index_word() as usize;
        let y = self.tagalong_y(j);
        let x = self.tagalong_x(j);
        self.Ancilla_SetXY(k, x.wrapping_add(8), y.wrapping_add(16));
        Some(k)
    }

    fn Tagalong_Sprite_GetX(&self, k: usize) -> u16 {
        self.sprite_slot_view(k).x()
    }

    fn Tagalong_Sprite_GetY(&self, k: usize) -> u16 {
        self.sprite_slot_view(k).y()
    }

    fn Tagalong_Sprite_SetX(&mut self, k: usize, x: u16) {
        self.sprite_slot_view_mut(k).set_x(x);
    }

    fn Tagalong_Sprite_SetY(&mut self, k: usize, y: u16) {
        self.sprite_slot_view_mut(k).set_y(y);
    }

    fn set_sprite_room_marker_word(&mut self, k: usize, value: u16) {
        self.sprite_workspace_mut().set_room_marker_word(k, value);
    }

    fn Tagalong_Sprite_SpawnDynamically(&mut self, k: usize, sprite: u8) -> Option<usize> {
        let j = (0..16)
            .rev()
            .find(|&j| self.sprite_slot_view(j).state() == 0)?;
        {
            let mut spawned = self.sprite_slot_view_mut(j);
            spawned.set_state(9);
            spawned.set_sprite_type(sprite);
        }
        self.SpritePrep_LoadProperties(j);
        if !self.game_state.world.location.is_indoors() {
            self.set_sprite_room_marker_word(j, 0xffff);
        } else {
            self.sprite_slot_view_mut(j).set_n(0xff);
        }
        let source = self.sprite_slot_view(k);
        let floor = source.floor();
        let direction = source.direction();
        let mut spawned = self.sprite_slot_view_mut(j);
        spawned.set_floor(floor);
        spawned.set_direction(direction);
        spawned.set_die_action(0);
        spawned.set_subtype(0);
        Some(j)
    }

    fn SpritePrep_LoadProperties(&mut self, k: usize) {
        self.sprite_prep_load_properties(k);
    }

    fn OldMan_RevertToSprite(&mut self, k: usize) {
        if let Some(j) = self.Tagalong_Sprite_SpawnDynamically(k, 0xad) {
            let layer = self.tagalong_slot(k).direction();
            let mut old_man = self.sprite_slot_view_mut(j);
            old_man.set_direction(layer);
            old_man.set_head_direction(layer);
            self.Tagalong_Sprite_SetY(j, self.tagalong_y(k).wrapping_add(2));
            self.Tagalong_Sprite_SetX(j, self.tagalong_x(k).wrapping_add(2));
            let floor = self.game_state.player.follower_link.floor();
            let mut old_man = self.sprite_slot_view_mut(j);
            old_man.set_floor(floor);
            old_man.set_ignore_projectile(1);
            old_man.set_subtype2(1);
        }
        self.OldMan_EnableCutscene();
        self.follower_state_mut().set_indicator(0);
        self.follower_link_state_mut().set_speed_setting(0);
    }

    fn OldMan_EnableCutscene(&mut self) {
        self.follower_link_state_mut().immobilize();
        self.follower_link_state_mut().enable_cutscene_immunity();
    }
}
