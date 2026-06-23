use crate::types::sign8;

pub(super) const IS_IN_DARK_WORLD_PREP: usize = 0x0fff;
pub(super) const DUNG_FLOOR_MOVE_FLAGS_PREP: usize = 0x041a;
pub(super) const ACTIVE_OVERLORD_INDEX_PREP: usize = 0x0fde;
pub(super) const SPRITE_PREP_SHARED_COUNTER: usize = 0x0ff8;
pub(super) const LINK_RUPEES_IN_POND_PREP: usize = 0x0f36a;
pub(super) const ITEM_DROP_LUCK_PREP: usize = 0x0cf9;
pub(super) const LUCK_KILL_COUNTER_PREP: usize = 0x0cfa;
pub(super) const NUM_SPRITES_KILLED_PREP: usize = 0x0cfb;
pub(super) const SPRITE_DELAY_AUX3_PREP: usize = 0x0ee0;
pub(super) const MINIGAME_CREDITS_PREP: usize = 0x04c4;
pub(super) const FLAG_OVERWORLD_AREA_DID_CHANGE_PREP: usize = 0x0abf;
pub(super) const SRAM_PROGRESS_INDICATOR_3_PREP: usize = 0x0f3c9;
pub(super) const SPRCOLL_X_BASE_PREP: usize = 0x0fbc;
pub(super) const SPRCOLL_Y_BASE_PREP: usize = 0x0fbe;
pub(super) const CHAINCHOMP_X_HIST_PREP: usize = 0x1fc00;
pub(super) const CHAINCHOMP_Y_HIST_PREP: usize = 0x1fd00;
pub(super) const FEATURE_MISC_BUG_FIXES_PREP: u32 = 4096;

#[cfg(test)]
pub(super) const ALT_SPRITE_STATE_PREP: usize = 0x1d00;
#[cfg(test)]
pub(super) const ALT_SPRITE_TYPE_PREP: usize = 0x1d10;
#[cfg(test)]
pub(super) const ALT_SPRITE_X_HI_PREP: usize = 0x1d30;
#[cfg(test)]
pub(super) const ALT_SPRITE_Y_HI_PREP: usize = 0x1d50;
#[cfg(test)]
pub(super) const BEAMOS_X_LO_PREP: usize = 0x1fd80;
#[cfg(test)]
pub(super) const BEAMOS_Y_LO_PREP: usize = 0x1fe80;
#[cfg(test)]
pub(super) const BEAMOS_Y_HI_PREP: usize = 0x1ff00;
#[cfg(test)]
pub(super) const MOLDORM_X_LO_PREP: usize = 0x1fc00;
#[cfg(test)]
pub(super) const MOLDORM_X_HI_PREP: usize = 0x1fc80;
#[cfg(test)]
pub(super) const MOLDORM_Y_LO_PREP: usize = 0x1fd00;
#[cfg(test)]
pub(super) const MOLDORM_Y_HI_PREP: usize = 0x1fd80;
pub(super) const OVERLORD_X_HI_PREP: usize = 0x0b10;
#[cfg(test)]
pub(super) const OVERLORD_Y_LO_PREP: usize = 0x0b18;
#[cfg(test)]
pub(super) const OVERLORD_Y_HI_PREP: usize = 0x0b20;
#[cfg(test)]
pub(super) const OVERLORD_GEN1_PREP: usize = 0x0b28;
#[cfg(test)]
pub(super) const OVERLORD_GEN2_PREP: usize = 0x0b30;
#[cfg(test)]
pub(super) const OVERLORD_GEN3_PREP: usize = 0x0b38;
#[cfg(test)]
pub(super) const OVERLORD_FLOOR_PREP: usize = 0x0b40;
#[cfg(test)]
pub(super) const SWAMOLA_X_LO_PREP: usize = 0x1fa5c;
#[cfg(test)]
pub(super) const SWAMOLA_X_HI_PREP: usize = 0x1fb1c;
#[cfg(test)]
pub(super) const SWAMOLA_Y_LO_PREP: usize = 0x1fbdc;
#[cfg(test)]
pub(super) const SWAMOLA_Y_HI_PREP: usize = 0x1fc9c;

pub(super) const WISH_POND_SPARKLE_X_OFFSETS: [u8; 8] = [0, 4, 8, 12, 16, 20, 24, 0];
pub(super) const WISH_POND_SPARKLE_Y_OFFSETS: [u8; 8] = [0, 8, 16, 24, 32, 40, 4, 36];
pub(super) const WISH_POND_ITEM_OAM_FLAGS: [u8; 76] = [
    5, 0xff, 5, 5, 5, 5, 5, 1, 2, 1, 1, 1, 2, 2, 2, 4, 4, 4, 1, 1, 2, 1, 1, 1, 2, 1, 2, 1, 4, 4, 2,
    1, 6, 1, 2, 1, 2, 2, 1, 2, 2, 4, 1, 1, 4, 2, 1, 4, 2, 2, 4, 4, 4, 2, 1, 4, 1, 2, 2, 1, 2, 2, 1,
    1, 4, 4, 1, 2, 2, 4, 4, 4, 2, 5, 2, 1,
];
pub(super) const RECEIVE_ITEM_PREP_DRAW_FRAME_START_BYTES: [u8; 76] = [
    0, 0, 0, 0, 0, 2, 2, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 0, 2, 0, 2, 2, 0, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 0, 0, 2, 0, 2, 2, 2, 0, 2, 2,
];
pub(super) const WISH_POND_ITEM_DATA_OFFSETS: [u8; 32] = [
    0, 4, 6, 7, 8, 10, 11, 12, 13, 14, 15, 16, 17, 20, 21, 22, 22, 23, 24, 25, 28, 30, 31, 32, 33,
    33, 37, 40, 42, 42, 42, 42,
];
pub(super) const WISH_POND_ITEM_DATA: [u8; 50] = [
    0x3a, 0x3a, 0x3b, 0x3b, 0x0c, 0x2a, 0x0a, 0x27, 0x29, 0x0d, 0x07, 0x08, 0x0f, 0x10, 0x11, 0x12,
    0x09, 0x13, 0x14, 0x4a, 0x21, 0x1d, 0x15, 0x18, 0x19, 0x31, 0x1a, 0x1a, 0x1b, 0x1c, 0x4b, 0x1e,
    0x1f, 0x49, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x22, 0x23, 0x29, 0x16, 0x2b, 0x2c, 0x2d, 0x3d,
    0x3c, 0x48,
];

pub(super) const SPRITE_INITIAL_BUMP_DAMAGE: [u8; 243] = [
    0x83, 0x83, 0x81, 2, 2, 2, 2, 2, 1, 0x13, 1, 1, 1, 1, 8, 1, 1, 8, 5, 3, 0x40, 4, 0, 2, 3, 0x85,
    0, 1, 0, 0x40, 0, 0, 6, 0, 5, 3, 1, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x40,
    0, 0, 0, 0, 0, 0, 2, 2, 0, 1, 1, 3, 1, 3, 1, 1, 3, 3, 3, 1, 3, 1, 1, 1, 1, 1, 1, 0x11, 0x14, 1,
    1, 2, 5, 0, 0, 4, 4, 8, 8, 8, 8, 4, 0, 4, 3, 2, 2, 2, 2, 2, 3, 1, 0, 0, 1, 0x80, 5, 1, 0, 0, 0,
    0x40, 0, 4, 0, 0, 0x14, 4, 6, 4, 4, 4, 4, 3, 4, 4, 4, 1, 4, 4, 0x15, 5, 4, 5, 0x15, 0x15, 3, 5,
    0, 5, 0x15, 5, 5, 6, 6, 6, 6, 5, 3, 6, 5, 5, 3, 3, 3, 6, 0x17, 0x15, 0x15, 5, 5, 1, 0x85, 0x83,
    5, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x17, 0x17, 5, 5, 5, 4, 3, 2, 0x10, 0,
    6, 0, 5, 7, 0x17, 0x17, 0x17, 0x15, 7, 6, 0x10, 0, 3, 3, 0, 0x19, 0x19, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0x10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub(super) fn chain_chomp_one_mult_prep(a: u8, b: u8) -> i32 {
    let at = if sign8(a) { 0u8.wrapping_sub(a) } else { a };
    let prod = (((at as u16) * (b as u16)) >> 8) as u8;
    if sign8(a) {
        !(prod as i32)
    } else {
        prod as i32
    }
}
