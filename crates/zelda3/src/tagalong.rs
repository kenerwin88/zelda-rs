// Methods ported from zelda3/src/tagalong.c and included inside ZeldaState.

use super::*;
use crate::types::{abs16, sign16, sign8, OamEnt, ProjectSpeedRet};

const MAIN_MODULE_INDEX_TAGALONG: usize = 0x10;
const SUBMODULE_INDEX_TAGALONG: usize = 0x11;
const PLAYER_IS_INDOORS_TAGALONG: usize = 0x1b;
const FILTERED_JOYPAD_L_TAGALONG: usize = 0xf6;
const FRAME_COUNTER_TAGALONG: usize = 0x1a;
const TAGALONG_MESSAGE_TIMER: usize = 0x2cd;
const TAGALONG_JUMP_TIMER_TAGALONG: usize = 0x2d6;
const TAGALONG_DRAW_ANIM_FRAME: usize = 0x2d7;
const COUNTDOWN_FOR_BLINK_TAGALONG: usize = 0x31f;
const OAM_PRIORITY_VALUE_TAGALONG: usize = 0x64;
const OAM_CUR_PTR_TAGALONG: usize = 0x90;
const OAM_EXT_CUR_PTR_TAGALONG: usize = 0x92;
const TAGALONG_MIRROR_BGM_COMMAND: usize = 0x12c;
const DIALOGUE_MESSAGE_INDEX_TAGALONG: usize = 0x1cf0;
const SAVED_MODULE_FOR_MENU_TAGALONG: usize = 0x10c;
const TAGALONG_MESSAGE_RESET_FLAG: usize = 0x223;
const MESSAGING_MODULE_TAGALONG: usize = 0x1cd8;
const PALETTE_SWAP_FLAG_TAGALONG: usize = 0x0abd;
const SUPER_BOMB_INDICATOR_TIMER_TAGALONG: usize = 0x4b4;
const SUPER_BOMB_INDICATOR_COUNTER_TAGALONG: usize = 0x4b5;
const DUNGEON_ROOM_INDEX_TAGALONG: usize = 0xa0;
const DUNG_FLAG_TRAPDOORS_DOWN_TAGALONG: usize = 0x468;
const DUNG_CUR_DOOR_POS_TAGALONG: usize = 0x68e;
const DOOR_ANIMATION_STEP_INDICATOR_TAGALONG: usize = 0x690;
const KIKI_ANIM_COUNTER_TAGALONG: usize = 0x0b69;
const SPRITE_IGNORE_PROJECTILE_TAGALONG: usize = 0x0ba0;
const SPRITE_STATE_TAGALONG: usize = 0x0dd0;
const SPRITE_TYPE_TAGALONG: usize = 0x0e20;
const SPRITE_Y_LO_TAGALONG: usize = 0x0d00;
const SPRITE_X_LO_TAGALONG: usize = 0x0d10;
const SPRITE_Y_HI_TAGALONG: usize = 0x0d20;
const SPRITE_X_HI_TAGALONG: usize = 0x0d30;
const SPRITE_Y_VEL_TAGALONG: usize = 0x0d40;
const SPRITE_X_VEL_TAGALONG: usize = 0x0d50;
const SPRITE_AI_STATE_TAGALONG: usize = 0x0d80;
const SPRITE_HEAD_DIR_TAGALONG: usize = 0x0eb0;
const SPRITE_D_TAGALONG: usize = 0x0de0;
const SPRITE_DELAY_AUX2_TAGALONG: usize = 0x0e10;
const SPRITE_GRAPHICS_TAGALONG: usize = 0x0dc0;
const SPRITE_FLOOR_TAGALONG: usize = 0x0f20;
const SPRITE_Z_TAGALONG: usize = 0x0f70;
const SPRITE_Z_VEL_TAGALONG: usize = 0x0f80;
const SPRITE_SUBTYPE2_TAGALONG: usize = 0x0e80;
const SPRITE_N_TAGALONG: usize = 0x0bc0;
const SPRITE_DIE_ACTION_TAGALONG: usize = 0x0cba;
const SPRITE_SUBTYPE_TAGALONG: usize = 0x0e30;
const ANCILLA_Y_LO_TAGALONG: usize = 0x0bfa;
const ANCILLA_X_LO_TAGALONG: usize = 0x0c04;
const ANCILLA_Y_HI_TAGALONG: usize = 0x0c0e;
const ANCILLA_X_HI_TAGALONG: usize = 0x0c18;
const ANCILLA_Y_VEL_TAGALONG: usize = 0x0c22;
const ANCILLA_X_VEL_TAGALONG: usize = 0x0c2c;
const ANCILLA_Y_SUBPIXEL_TAGALONG: usize = 0x0c36;
const ANCILLA_X_SUBPIXEL_TAGALONG: usize = 0x0c40;
const ANCILLA_STEP_TAGALONG: usize = 0x0c54;
const ANCILLA_ITEM_TO_LINK_TAGALONG: usize = 0x0c5e;
const ANCILLA_L_TAGALONG: usize = 0x385;
const ANCILLA_R_TAGALONG: usize = 0x03ea;
const OAM_BUF_TAGALONG: usize = 0x0800;
const SAVE_DUNG_INFO_TAGALONG: usize = 0x0f000;
const SAVED_TAGALONG_Y_TAGALONG: usize = 0x0f3cd;
const SAVED_TAGALONG_X_TAGALONG: usize = 0x0f3cf;
const SAVED_TAGALONG_INDOORS_TAGALONG: usize = 0x0f3d1;
const SAVED_TAGALONG_FLOOR_TAGALONG: usize = 0x0f3d2;
const OVERWORLD_SCREEN_INDEX_TAGALONG: usize = 0x8a;
const ENHANCED_FEATURES0_TAGALONG: usize = 0x064c;

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

struct SpriteSpawnInfo {
    r0_x: u16,
    r2_y: u16,
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
        tagalong: 2,
    },
    TagalongMessageInfo {
        y: 0x0c78,
        x: 0x238,
        bit: 4,
        msg: 0x21,
        tagalong: 4,
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
        !self.follower_link_state().is_immobilized()
            && sub != 10
            && !(main == 9 && sub == 0x23)
            && !(main == 14 && (sub == 1 || sub == 2))
    }

    pub(super) fn follower_validate_message_freedom(&self) -> bool {
        self.follower_link_state().can_open_follower_message()
    }

    pub(super) fn follower_move_towards_link(&mut self) {
        loop {
            let k = 9;
            let j = self.follower_state().tail_write_index() as usize;
            let x = self.tagalong_x(j);
            let y = self.tagalong_y(j);
            self.ancilla_slot_mut(k).set_x(x);
            self.ancilla_slot_mut(k).set_y(y);

            let pt = self.Ancilla_ProjectSpeedTowardsPlayer(k, 24);
            let mut ancilla = self.ancilla_slot_mut(k);
            ancilla.set_x_velocity(pt.x);
            ancilla.set_y_velocity(pt.y);
            self.Ancilla_MoveY(k);
            self.Ancilla_MoveX(k);

            let x = self.Ancilla_GetX(k);
            let y = self.Ancilla_GetY(k);
            let link = self.follower_link_state();
            if abs16(x.wrapping_sub(link.x())) < 2 && abs16(y.wrapping_sub(link.y())) < 2 {
                return;
            }
            self.follower_state_mut().increment_tail_write_index();
            let k = self.follower_state().tail_write_index() as usize;
            if k == 18 {
                return;
            }
            let layer_bits = self.follower_link_state().floor_layer_bits() | 1;
            let mut follower = self.tagalong_slot_mut(k);
            follower.set_position(x, y);
            follower.set_layer_bits(layer_bits);
        }
    }

    pub(super) fn follower_check_blind_trigger(&self) -> bool {
        let k = self.follower_state().data_index() as usize;
        let mut x = self.tagalong_x(k);
        let mut y = self.tagalong_y(k);
        let z = self.tagalong_slot(k).z_signed() as i16 as u16;
        y = y.wrapping_add(z).wrapping_add(12);
        x = x.wrapping_add(8);
        abs16(0x1568u16.wrapping_sub(y)) < 24 && abs16(0x1980u16.wrapping_sub(x)) < 24
    }

    pub(super) fn follower_initialize(&mut self) {
        let link = self.follower_link_state();
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
            .enhanced_features()
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
        let layer_bits = self.follower_link_state().floor_layer_bits() | 1;
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
        if self.follower_state().indicator() == 0 {
            return;
        }
        if self.follower_state().indicator() == 0x0e {
            self.follower_handle_trigger();
            return;
        }
        let j = TAGALONG_MESSAGE_FOLLOWER_INDICATORS
            .iter()
            .position(|&v| v == self.follower_state().indicator())
            .map(|v| v as i32)
            .unwrap_or(-1);
        if j >= 0
            && self.game_state.frame.submodule == 0
            && !(j == 2 && self.game_state.world.location.overworld_screen_index() & 0x40 != 0)
        {
            let timer = self.tick_shared_message_timer();
            if sign16(timer) {
                if !self.follower_validate_message_freedom() {
                    self.clear_shared_message_timer();
                } else {
                    let j = j as usize;
                    self.start_shared_message_timer(TAGALONG_MESSAGE_TIMERS[j]);
                    self.dialogue_message_index_mut().set_value(TAGALONG_MSG[j]);
                    self.Tagalong_Main_ShowTextMessage();
                }
            }
        }
        if j != 0 {
            self.follower_no_timed_message();
        }
    }

    pub(super) fn follower_no_timed_message(&mut self) {
        if self.follower_state().dropped() != 0 {
            self.follower_not_following();
            return;
        }
        if self.follower_state().indicator() == 12 {
            if !self.follower_link_state().has_auxiliary_state() {
                if self.follower_can_drop() {
                    self.follower_drop();
                    return;
                }
            }
        } else if self.follower_state().indicator() == 13 {
            if self.follower_link_state().auxiliary_state() == 2
                || self.player_state().near_pit_state_is(2)
            {
                self.follower_drop();
                return;
            }
            if self.follower_can_drop() {
                self.follower_drop();
                return;
            }
        }
        self.follower_check_game_mode();
    }

    fn follower_can_drop(&self) -> bool {
        self.game_state.frame.submodule == 0
            && self.follower_link_state().can_drop_follower()
            && self.follower_state().appearance_none_flag() == 0
            && self.follower_state().hookshot_interlock_is_clear()
            && self
                .tagalong_slot(self.follower_state().data_index() as usize)
                .z_signed()
                <= 0
            && self.player_state().filtered_joypad_l() & 0x80 != 0
    }

    fn follower_drop(&mut self) {
        if self.follower_state().indicator() == 13
            && self.game_state.world.location.indoor_flag() == 0
        {
            if self.follower_link_state().is_using_medallion() {
                self.follower_check_game_mode();
                return;
            }
            self.hud_state_mut().set_super_bomb_indicator_timer(3);
            self.hud_state_mut().set_super_bomb_indicator_counter(0xbb);
        }
        self.follower_state_mut().set_dropped(128);
        self.follower_state_mut().set_reacquire_timer_low(64);
        let k = self.follower_state().data_index() as usize;
        let y = self.tagalong_y(k);
        let x = self.tagalong_x(k);
        let floor = self.follower_link_state().floor();
        let indoor = self.game_state.world.location.indoor_flag();
        self.follower_state_mut().set_saved_y(y);
        self.follower_state_mut().set_saved_x(x);
        self.follower_state_mut().set_saved_floor(floor);
        self.follower_state_mut().set_saved_indoor_flag(indoor);
        self.follower_not_following();
    }

    pub(super) fn follower_check_game_mode(&mut self) {
        if self.tagalong_is_following() && self.follower_link_state().is_moving() {
            let mut k = self.follower_state().tail_write_index().wrapping_add(1);
            if k == 20 {
                k = 0;
            }
            self.follower_state_mut().set_tail_write_index(k);
            let link = self.follower_link_state();
            let z = link.z_for_follow();
            let k = k as usize;
            let y = link.y().wrapping_sub(z as u16);
            let x = link.x();
            let mut layerbits = link.facing_layer_bits() | link.floor_layer_bits();
            if link.is_swimming() {
                layerbits |= 0x20;
            } else {
                if link.is_hookshot() && self.player_state().has_hookshot_interlock() {
                    layerbits |= 0x10;
                }
                let surface_effect = self.player_state().water_ripple_or_grass_state();
                if surface_effect != 0 {
                    layerbits |= if surface_effect == 1 { 0x80 } else { 0x40 };
                }
            }
            let mut follower = self.tagalong_slot_mut(k);
            follower.set_z(z);
            follower.set_position(x, y);
            follower.set_layer_bits(layerbits);
        }
        match self.follower_state().indicator() {
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
        if self.follower_state().indicator() == 10
            && self.follower_link_state().has_auxiliary_state()
            && self.player_state().blink_countdown() != 0
        {
            let k = if self.follower_state().data_index().wrapping_add(1) == 20 {
                0
            } else {
                self.follower_state().data_index().wrapping_add(1)
            };
            self.kiki_spawn_handler_b(k as usize);
            self.follower_state_mut().set_indicator(0);
            return;
        }
        if self.follower_state().indicator() == 6
            && self.game_state.world.location.dungeon_room() == 0x0ac
            && self.save_progress().dungeon_info_word(101) & 0x100 != 0
            && self.follower_check_blind_trigger()
        {
            let k = self.follower_state().data_index() as usize;
            let x = self.tagalong_x(k);
            let y = self.tagalong_y(k);
            self.follower_state_mut().set_indicator(0);
            self.blind_spawn_from_maiden(x, y);
            self.dungeon_environment_mut()
                .increment_trapdoors_down_low();
            self.dungeon_doors_mut().clear_current_door_pos();
            self.dungeon_doors_mut().clear_door_animation_step();
            self.set_submodule(5);
            self.system_signals_mut().set_music_control(21);
            return;
        }
        if self.follower_state().hookshot_interlock_is_clear() {
            if self.follower_link_state().is_hookshot()
                && self.player_state().has_hookshot_interlock()
            {
                self.follower_state_mut().set_hookshot_interlock();
                self.advance_follower_tail();
                self.tagalong_draw();
                return;
            }
        } else {
            if self.follower_link_state().is_hookshot() {
                self.advance_follower_tail();
                self.tagalong_draw();
                return;
            }
            if self.follower_state().hookshot_release_tail_index()
                != self.follower_state().data_index()
            {
                self.follower_state_mut()
                    .advance_data_index_wrapping_at_20();
                self.tagalong_draw();
                return;
            }
            self.follower_state_mut().clear_hookshot_interlock();
        }
        let k = self.follower_state().data_index() as usize;
        if self.tagalong_slot(k).is_above_ground() {
            if self.follower_state().tail_write_index() != k as u8 {
                self.follower_state_mut()
                    .advance_data_index_wrapping_at_20();
                self.tagalong_draw();
                return;
            }
            let link = self.follower_link_state();
            let y = link.y();
            let x = link.x();
            let mut follower = self.tagalong_slot_mut(k);
            follower.set_z(0);
            follower.set_position(x, y);
        }
        if self.follower_link_state().is_moving() {
            self.advance_follower_tail();
        }
        self.tagalong_draw();
    }

    fn advance_follower_tail(&mut self) {
        let mut t = self.follower_state().tail_write_index().wrapping_sub(15);
        if sign8(t) {
            t = t.wrapping_add(20);
        }
        if t == self.follower_state().data_index() {
            self.follower_state_mut()
                .advance_data_index_wrapping_at_20();
        }
    }

    pub(super) fn follower_not_following(&mut self) {
        if self.follower_state().saved_indoor_flag() != self.game_state.world.location.indoor_flag()
        {
            return;
        }
        if !self.follower_link_state().is_running() && !self.follower_check_proximity_to_link() {
            self.follower_initialize();
            let indoor = self.game_state.world.location.indoor_flag();
            self.follower_state_mut().set_saved_indoor_flag(indoor);
            if self.follower_state().indicator() == 13 {
                self.hud_state_mut().set_super_bomb_indicator_timer(254);
                self.hud_state_mut().set_super_bomb_indicator_counter(0);
            }
            self.follower_state_mut().set_dropped(0);
            self.tagalong_draw();
        } else {
            if self.follower_state().indicator() == 13
                && self.game_state.world.location.indoor_flag() == 0
                && self.hud_state().super_bomb_indicator_timer() == 0
            {
                if self.AncillaAdd_SuperBombExplosion(0x3a, 0).is_some() {
                    self.follower_state_mut().set_dropped(0);
                    if self
                        .enhanced_features()
                        .has(FEATURES0_MISC_BUG_FIXES_TAGALONG)
                    {
                        self.follower_state_mut().set_indicator(0);
                        return;
                    }
                } else {
                    self.hud_state_mut().set_super_bomb_indicator_counter(1);
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
        if self.follower_link_state().speed_setting() != 4 {
            self.follower_link_state_mut().set_speed_setting(12);
        }
        self.follower_handle_trigger();
        if self.follower_state().indicator() == 0 {
            return;
        } else if self.follower_state().indicator() == 4 {
            let k = self.follower_state().data_index() as usize;
            if self.tagalong_slot(k).is_above_ground()
                && self.follower_state().tail_write_index() != self.follower_state().data_index()
            {
                self.follower_state_mut()
                    .advance_data_index_wrapping_at_20();
                self.tagalong_draw();
                return;
            }
        } else {
            if self
                .follower_link_state()
                .should_transform_old_man_from_recoil()
            {
                if self.follower_state().tail_write_index() == self.follower_state().data_index() {
                    // C asserts here because the follower X value is undefined.
                    panic!("follower_old_man assert");
                }
                self.transform_old_man();
                return;
            }
            if self
                .follower_link_state()
                .should_transform_old_man_from_auxiliary_state()
            {
                self.transform_old_man();
                return;
            }
        }
        if self.follower_link_state().is_moving() {
            let mut t = self.follower_state().tail_write_index().wrapping_sub(20);
            if sign8(t) {
                t = t.wrapping_add(20);
            }
            if t == self.follower_state().data_index() {
                self.follower_state_mut()
                    .advance_data_index_wrapping_at_20();
            }
        } else if self.game_state.frame.frame_counter & 3 == 0
            && self.follower_state().tail_write_index() != self.follower_state().data_index()
        {
            let mut t = self.follower_state().tail_write_index().wrapping_sub(9);
            if sign8(t) {
                t = t.wrapping_add(20);
            }
            if t != self.follower_state().data_index() {
                self.follower_state_mut()
                    .advance_data_index_wrapping_at_20();
            }
        }
        self.tagalong_draw();
    }

    fn transform_old_man(&mut self) {
        let indicator =
            TAGALONG_RELEASE_INDICATOR_BY_FOLLOWER[self.follower_state().indicator() as usize];
        self.follower_state_mut().set_indicator(indicator);
        self.follower_state_mut().set_reacquire_timer_low(64);
        let k = self.follower_state().data_index() as usize;
        let y = self.tagalong_y(k);
        let x = self.tagalong_x(k);
        let floor = self.follower_link_state().floor();
        self.follower_state_mut().set_saved_y(y);
        self.follower_state_mut().set_saved_x(x);
        self.follower_state_mut().set_saved_floor(floor);
        self.follower_old_man_unused();
    }

    pub(super) fn follower_old_man_unused(&mut self) {
        self.follower_link_state_mut().set_speed_setting(16);
        if self.follower_link_state().can_reacquire_old_man() {
            self.follower_link_state_mut().set_speed_setting(0);
            if !self.follower_link_state().is_hookshot() && !self.follower_check_proximity_to_link()
            {
                self.follower_initialize();
                let indicator = TAGALONG_SLOWDOWN_INDICATOR_BY_FOLLOWER
                    [self.follower_state().indicator() as usize];
                self.follower_state_mut().set_indicator(indicator);
                return;
            }
        }
        self.follower_do_layers();
    }

    pub(super) fn follower_do_layers(&mut self) {
        let priority =
            FollowerLinkState::oam_priority_for_floor_value(self.follower_state().saved_floor());
        self.oam_state_mut()
            .set_priority_word((priority as u16) << 8);
        let a =
            if self.follower_state().indicator() == 12 || self.follower_state().indicator() == 13 {
                2
            } else {
                1
            };
        self.follower_animate_movement_preserved(
            a,
            self.follower_state().saved_x(),
            self.follower_state().saved_y(),
        );
    }

    pub(super) fn follower_check_proximity_to_link(&mut self) -> bool {
        self.follower_state_mut().decrement_reacquire_timer_low();
        if !sign8(self.follower_state().reacquire_timer_low()) {
            return true;
        }
        self.follower_state_mut().set_reacquire_timer_low(0);
        let y = self.follower_state().saved_y();
        let x = self.follower_state().saved_x();
        let link = self.follower_link_state();
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
        let st = if self.follower_state().data_index().wrapping_add(1) >= 20 {
            0
        } else {
            self.follower_state().data_index().wrapping_add(1)
        };
        for info in &infos[start..end] {
            if info.tagalong == self.follower_state().indicator()
                && self.follower_check_for_trigger(info)
            {
                if info.bit & self.follower_state().event_flags() != 0 {
                    return;
                }
                self.follower_state_mut().or_event_flags(info.bit);
                self.dialogue_message_index_mut().set_value(info.msg);
                if info.msg == 0xffff {
                    if info.bit & 3 == 0 {
                        self.kiki_revert_to_sprite(st as usize);
                    } else if self
                        .overworld_event_info()
                        .event_info(self.game_state.world.location.overworld_screen_index() as usize)
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
        if self.follower_state().appearance_none_flag() != 0 {
            return;
        }
        let current_follower = self.tagalong_slot(self.follower_state().data_index() as usize);
        let priority = if current_follower.z() != 0 && !self.game_state.world.location.is_indoors()
        {
            0x20
        } else if self.game_state.frame.submodule == 14 {
            self.follower_link_state().oam_priority_for_floor()
        } else {
            (current_follower.layer_bits() & 0x0c) << 2
        };
        self.oam_state_mut()
            .set_priority_word((priority as u16) << 8);
        let k = if sign8(self.follower_state().data_index()) {
            0
        } else {
            self.follower_state().data_index() as usize
        };
        let x = self.tagalong_x(k);
        let y = self.tagalong_y(k);
        let a = self.tagalong_slot(k).layer_bits();
        self.follower_animate_movement_preserved(a, x, y);
    }

    #[rustfmt::skip]
    pub(super) fn set_oam_follower(&mut self, oam: &mut OamEnt, x: u16, y: u16, charnum: u8, flags: u8, mut big: u8) {
        oam.x = x as u8;
        let visible = x.wrapping_add(0x80) < 0x180 && {
            big |= ((x >> 8) & 1) as u8;
            y.wrapping_add(0x10) < 0x100
        };
        oam.y = if visible { y as u8 } else { 0xf0 };
        oam.charnum = charnum;
        oam.flags = flags;
        let _ = big;
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
            && (self.follower_state().indicator() == 6 || self.follower_state().indicator() == 1)
        {
            yt = 8;
            av = if self.swim_acceleration().acceleration(0) != 0 {
                (self.game_state.frame.frame_counter >> 1) & 4
            } else {
                (self.game_state.frame.frame_counter >> 2) & 4
            };
        } else if self.game_state.frame.submodule == 8
            || self.game_state.frame.submodule == 14
            || self.game_state.frame.submodule == 16
        {
            av = if self.follower_link_state().is_running() {
                self.game_state.frame.frame_counter & 4
            } else {
                (self.game_state.frame.frame_counter >> 1) & 4
            };
        } else if self.follower_state().indicator() == 11 {
            av = (self.game_state.frame.frame_counter >> 1) & 4;
        } else if ((self.follower_state().indicator() == 12
            || self.follower_state().indicator() == 13)
            && self.follower_state().dropped() != 0)
            || self.follower_link_state().is_immobilized()
            || self.game_state.frame.submodule == 10
            || (self.game_state.frame.main_module == 9 && self.game_state.frame.submodule == 0x23)
            || (self.game_state.frame.main_module == 14
                && (self.game_state.frame.submodule == 1 || self.game_state.frame.submodule == 2))
            || !self.follower_link_state().is_moving()
        {
            av = 4;
            sc = 4;
        } else {
            av = if self.follower_link_state().is_running() {
                self.game_state.frame.frame_counter & 4
            } else {
                (self.game_state.frame.frame_counter >> 1) & 4
            };
        }
        let frame = (ain & 3).wrapping_add(av).wrapping_add(yt) as usize;
        let link_y = self.follower_link_state().y();
        let spr_offs = if (link_y == yin && (ain & 3) == 0) || link_y < yin {
            TAGALONG_DRAW_SPR_OFFS0[self.oam_state().sprite_sorting_offset_index()] >> 2
        } else {
            TAGALONG_DRAW_SPR_OFFS1[self.oam_state().sprite_sorting_offset_index()] >> 2
        } as usize;
        self.oam_state_mut()
            .set_current_extended_pointer(0x0a20 + spr_offs as u16);
        self.oam_state_mut()
            .set_current_pointer(0x0800 + (spr_offs as u16) * 4);
        let mut oam = self.oam_state().current_pointer_usize();
        let scrolly = yin.wrapping_sub(self.ppu_scroll_copy().bg2_v_copy2());
        let scrollx = xin.wrapping_sub(self.ppu_scroll_copy().bg2_h_copy2());
        let mut skip_first_sprites = false;
        let mut sk_index = 0usize;
        if self.follower_state().indicator() == 1
            || self.follower_state().indicator() == 6
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
            sk_index += self.follower_state().draw_anim_frame() as usize * 4;
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
        let mut pal = TAGALONG_DRAW_PALS[self.follower_state().indicator() as usize];
        if pal == 7 && self.follower_state().palette_swap_flag() != 0 {
            pal = 0;
        }
        if self.follower_state().indicator() == 13 {
            let colorful = if self
                .enhanced_features()
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
            + (TAGALONG_DRAW_OFFS[self.follower_state().indicator() as usize] >> 3) as usize];
        let sprf = TAGALONG_DMA_AND_FLAGS[frame];
        if self.follower_state().indicator() != 12 && self.follower_state().indicator() != 13 {
            self.set_oam_follower_at(
                oam,
                scrollx.wrapping_add(sprd.x1 as i16 as u16),
                scrolly.wrapping_add(sprd.y1 as i16 as u16),
                0x20,
                (sprf.flags & 0xf0) | (pal << 1) | (self.oam_state().priority_word() >> 8) as u8,
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
            ((sprf.flags & 0x0f) << 4) | (pal << 1) | (self.oam_state().priority_word() >> 8) as u8,
            2,
        );
        self.set_sprite_dma_body_pointer(sprf.dma7);
    }

    pub(super) fn follower_check_for_trigger(&self, info: &TagalongMessageInfo) -> bool {
        let link = self.follower_link_state();
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
        if self.follower_state().indicator() == 9 || self.follower_state().indicator() == 10 {
            self.follower_state_mut().set_indicator(0);
        }
    }

    pub(super) fn blind_spawn_from_maiden(&mut self, x: u16, y: u16) {
        let k = 0;
        let mut maiden = self.sprite_slot_mut(k);
        maiden.set_state(9);
        maiden.set_sprite_type(206);
        self.Tagalong_Sprite_SetX(k, x);
        self.Tagalong_Sprite_SetY(k, y.wrapping_sub(16));
        self.SpritePrep_LoadProperties(k);
        let mut maiden = self.sprite_slot_mut(k);
        maiden.set_delay_aux2(192);
        maiden.set_graphics(21);
        maiden.set_direction(2);
        maiden.set_ignore_projectile(2);
        self.dungeon_savegame_state_mut()
            .or_savegame_state_bits(0x2000);
        self.follower_state_mut().clear_kiki_anim_counter();
    }

    pub(super) fn kiki_revert_to_sprite(&mut self, k: usize) {
        if let Some(j) = self.kiki_spawn_handler_monke(k) {
            self.sprite_slot_mut(j).set_subtype2(1);
        }
        self.follower_state_mut().set_indicator(0);
    }

    pub(super) fn kiki_spawn_handler_monke(&mut self, k: usize) -> Option<usize> {
        let Some((j, _info)) = self.Tagalong_Sprite_SpawnDynamically(k, 0xb6) else {
            return None;
        };
        let layer = self.tagalong_slot(k).direction();
        let mut monke = self.sprite_slot_mut(j);
        monke.set_head_direction(layer);
        monke.set_direction(layer);
        let x = self.tagalong_x(k);
        let y = self.tagalong_y(k);
        self.Tagalong_Sprite_SetX(j, x.wrapping_add(2));
        self.Tagalong_Sprite_SetY(j, y.wrapping_add(2));
        let floor = self.follower_link_state().floor();
        let mut monke = self.sprite_slot_mut(j);
        monke.set_floor(floor);
        monke.set_ignore_projectile(1);
        monke.set_floor(2);
        self.follower_link_state_mut().set_speed_setting(0);
        Some(j)
    }

    pub(super) fn kiki_spawn_handler_a(&mut self, k: usize) {
        if let Some(j) = self.kiki_spawn_handler_monke(k) {
            self.sprite_slot_mut(j).set_subtype2(2);
        }
    }

    pub(super) fn kiki_spawn_handler_b(&mut self, k: usize) {
        if let Some(j) = self.kiki_spawn_handler_monke(k) {
            let mut monke = self.sprite_slot_mut(j);
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

    fn set_tagalong_x(&mut self, k: usize, x: u16) {
        self.tagalong_slot_mut(k).set_x(x);
    }

    fn set_tagalong_y(&mut self, k: usize, y: u16) {
        self.tagalong_slot_mut(k).set_y(y);
    }

    fn Tagalong_Main_ShowTextMessage(&mut self) {
        if self.game_state.frame.main_module != 14 {
            self.world_transient_mut()
                .clear_tile_interaction_shared_flag();
            self.messaging_state_mut().clear_module();
            self.set_submodule(2);
            self.save_main_module_for_menu();
            self.set_main_module(14);
        }
    }

    fn Ancilla_GetX(&self, k: usize) -> u16 {
        self.ancilla_slot(k).x()
    }

    fn Ancilla_GetY(&self, k: usize) -> u16 {
        self.ancilla_slot(k).y()
    }

    fn Ancilla_SetXY(&mut self, k: usize, x: u16, y: u16) {
        let mut ancilla = self.ancilla_slot_mut(k);
        ancilla.set_x(x);
        ancilla.set_y(y);
    }

    fn Ancilla_MoveX(&mut self, k: usize) {
        self.ancilla_slot_mut(k).move_x();
    }

    fn Ancilla_MoveY(&mut self, k: usize) {
        self.ancilla_slot_mut(k).move_y();
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
        let link = self.follower_link_state();
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
        let mut explosion = self.ancilla_slot_mut(k);
        explosion.set_r(0);
        explosion.set_step(0);
        explosion.set_work_byte_25(0);
        explosion.set_l(0);
        explosion.set_work_byte_3(6);
        explosion.set_item_to_link(1);
        let j = self.follower_state().data_index_word() as usize;
        let y = self.tagalong_y(j);
        let x = self.tagalong_x(j);
        self.Ancilla_SetXY(k, x.wrapping_add(8), y.wrapping_add(16));
        Some(k)
    }

    fn Tagalong_Sprite_GetX(&self, k: usize) -> u16 {
        self.sprite_slot(k).x()
    }

    fn Tagalong_Sprite_GetY(&self, k: usize) -> u16 {
        self.sprite_slot(k).y()
    }

    fn Tagalong_Sprite_SetX(&mut self, k: usize, x: u16) {
        self.sprite_slot_mut(k).set_x(x);
    }

    fn Tagalong_Sprite_SetY(&mut self, k: usize, y: u16) {
        self.sprite_slot_mut(k).set_y(y);
    }

    fn set_sprite_room_marker_word(&mut self, k: usize, value: u16) {
        self.sprite_workspace_mut().set_room_marker_word(k, value);
    }

    fn Tagalong_Sprite_SpawnDynamically(
        &mut self,
        k: usize,
        sprite: u8,
    ) -> Option<(usize, SpriteSpawnInfo)> {
        let j = (0..16).rev().find(|&j| self.sprite_slot(j).state() == 0)?;
        {
            let mut spawned = self.sprite_slot_mut(j);
            spawned.set_state(9);
            spawned.set_sprite_type(sprite);
        }
        self.SpritePrep_LoadProperties(j);
        if !self.game_state.world.location.is_indoors() {
            self.set_sprite_room_marker_word(j, 0xffff);
        } else {
            self.sprite_slot_mut(j).set_n(0xff);
        }
        let source = self.sprite_slot(k);
        let floor = source.floor();
        let direction = source.direction();
        let mut spawned = self.sprite_slot_mut(j);
        spawned.set_floor(floor);
        spawned.set_direction(direction);
        spawned.set_die_action(0);
        spawned.set_subtype(0);
        Some((
            j,
            SpriteSpawnInfo {
                r0_x: self.tagalong_x(k),
                r2_y: self.tagalong_y(k),
            },
        ))
    }

    fn SpritePrep_LoadProperties(&mut self, k: usize) {
        self.sprite_prep_load_properties(k);
    }

    fn OldMan_RevertToSprite(&mut self, k: usize) {
        if let Some((j, _info)) = self.Tagalong_Sprite_SpawnDynamically(k, 0xad) {
            let layer = self.tagalong_slot(k).direction();
            let mut old_man = self.sprite_slot_mut(j);
            old_man.set_direction(layer);
            old_man.set_head_direction(layer);
            self.Tagalong_Sprite_SetY(j, self.tagalong_y(k).wrapping_add(2));
            self.Tagalong_Sprite_SetX(j, self.tagalong_x(k).wrapping_add(2));
            let floor = self.follower_link_state().floor();
            let mut old_man = self.sprite_slot_mut(j);
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
