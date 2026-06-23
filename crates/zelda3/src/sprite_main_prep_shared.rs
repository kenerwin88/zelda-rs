use crate::types::sign8;
use crate::zelda_rtl::sprite::DrawMultipleData;

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

// ---------------------------------------------------------------------------
// Promoted sprite prep method-local tables. Names retain the owning helper so
// generic C table names stay readable at callsites.
// ---------------------------------------------------------------------------

pub(super) const SPRITE_PREP_STANDARD_GUARD_GUARD_SUBTYPE_B_REMAP: [u8; 8] =
    [0, 2, 1, 3, 6, 4, 5, 7];

pub(super) const SPRITE_PREP_RAT_BUMP_DAMAGE_VALUES: [u8; 2] = [0, 5];

pub(super) const SPRITE_PREP_RAT_HEALTH_VALUES: [u8; 2] = [2, 8];

pub(super) const SPRITE_PREP_KEESE_BUMP_DAMAGE_VALUES: [u8; 2] = [0x80, 0x85];

pub(super) const SPRITE_PREP_KEESE_HEALTH_VALUES: [u8; 2] = [1, 4];

pub(super) const SPRITE_PREP_KEESE_FLAGS5_VALUES: [u8; 2] = [0, 7];

pub(super) const SPRITE_PREP_ROPE_BUMP_DAMAGE_VALUES: [u8; 2] = [1, 5];

pub(super) const SPRITE_PREP_ROPE_HEALTH_VALUES: [u8; 2] = [4, 8];

pub(super) const SPRITE_PREP_ROPE_FLAGS5_VALUES: [u8; 2] = [1, 7];

pub(super) const SPRITE_PREP_POKEY_INITIAL_X_VELOCITIES: [i8; 4] = [16, -16, 16, -16];

pub(super) const SPRITE_PREP_POKEY_INITIAL_Y_VELOCITIES: [i8; 4] = [16, 16, -16, -16];

pub(super) const SPRITE_PREP_OCTOBALLOON_DELAYS: [u8; 4] = [192, 208, 224, 240];

pub(super) const SPRITE_PREP_RAVEN_BUMP_DAMAGE_VALUES: [u8; 2] = [0x81, 0x88];

pub(super) const SPRITE_PREP_RAVEN_HEALTH_VALUES: [u8; 2] = [4, 8];

pub(super) const SPRITE_PREP_RAVEN_FLAGS5_VALUES: [u8; 2] = [6, 2];

pub(super) const SPRITE_PREP_ARROW_GAME_BOUNCE_X_OFFSETS: [u8; 8] =
    [0, 0x40, 0x80, 0xc0, 0x30, 0x60, 0x90, 0xc0];

pub(super) const SPRITE_PREP_ARROW_GAME_BOUNCE_Y_OFFSETS: [u8; 8] =
    [0, 0x4f, 0x4f, 0x4f, 0x5a, 0x5a, 0x5a, 0x5a];

pub(super) const SPRITE_PREP_ARROW_GAME_BOUNCE_ATTRS: [u8; 8] = [0, 1, 1, 1, 2, 2, 2, 2];

pub(super) const SPRITE_PREP_ARROW_GAME_BOUNCE_X_VELOCITIES: [i8; 2] = [-8, 12];

pub(super) const SPRITE_PREP_ARROW_GAME_BOUNCE_FLAGS4_VALUES: [u8; 2] = [0x1c, 0x15];

pub(super) const ARCHERY_GAME_HOST_PROCTOR_GAME_SPRITE_COUNTS: [u8; 6] = [5, 4, 3, 2, 1, 0];

pub(super) const ARCHERY_GAME_HOST_PROCTOR_GAME_X_OFFSETS: [i8; 18] = [
    0, 0, 0, 0, 48, 48, 48, 48, 8, 8, 16, 16, 24, 24, 32, 32, 40, 40,
];

pub(super) const ARCHERY_GAME_HOST_PROCTOR_GAME_Y_OFFSETS: [i8; 18] =
    [-8, 0, 8, 16, -8, 0, 8, 16, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8];

pub(super) const ARCHERY_GAME_HOST_PROCTOR_GAME_CHARS: [u8; 18] = [
    0x2b, 0x3b, 0x3b, 0x2b, 0x2b, 0x3b, 0x3b, 0x2b, 0x63, 0x73, 0x63, 0x73, 0x63, 0x73, 0x63, 0x73,
    0x63, 0x73,
];

pub(super) const ARCHERY_GAME_HOST_PROCTOR_GAME_OAM_FLAGS: [u8; 18] = [
    0x33, 0x33, 0xb3, 0xb3, 0x73, 0x73, 0xf3, 0xf3, 0x32, 0x32, 0x32, 0x32, 0x32, 0x32, 0x32, 0x32,
    0x32, 0x32,
];

pub(super) const SPRITE_GOOD_OR_BAD_ARCHERY_TARGET_CASH_PRIZE: [u8; 10] =
    [4, 8, 16, 32, 64, 99, 99, 99, 99, 99];

pub(super) const ARCHERY_GAME_HOST_IDLE_GRAPHICS: [u8; 4] = [3, 4, 3, 2];

pub(super) const ARCHERY_TARGET_RESET_X_LOWS: [u8; 2] = [(-24i8) as u8, 8];

pub(super) const SPRITE_SPAWN_SPARKLE_GARNISH_COORD_OFFSETS: [i8; 4] = [-4, 0, 4, 8];

pub(super) const SPRITE_MAGIC_BAT_SPAWN_LIGHTNING_X_VELOCITIES: [i8; 4] = [-8, -4, 4, 8];

pub(super) const SPRITE_MAGIC_BAT_SPAWN_LIGHTNING_STATE2_VALUES: [u8; 4] = [0, 0x11, 0x22, 0x33];

pub(super) const MAGIC_BAT_RISING_UP_X_ACCELERATIONS: [i8; 2] = [-8, 7];

pub(super) const MAGIC_BAT_LIGHTNING_OAM_FLAG_SEQUENCE: [u8; 8] = [0x0a, 4, 2, 4, 2, 0x0a, 4, 2];

pub(super) const KHOLDSTARE_SPAWN_PUFF_CLOUD_GARNISH_XY_OFFSETS: [i8; 8] =
    [-8, -6, -4, -2, 0, 2, 4, 6];

pub(super) const FIRE_BAT_ANIMATE_GRAPHICS: [u8; 4] = [4, 5, 6, 5];

pub(super) const SPRITE_SPAWN_FIRE_PHLEGM_X_OFFSETS: [i8; 4] = [16, -8, 4, 4];

pub(super) const SPRITE_SPAWN_FIRE_PHLEGM_Y_OFFSETS: [i8; 4] = [-2, -2, 8, -20];

pub(super) const SPRITE_SPAWN_FIRE_PHLEGM_X_VELOCITIES: [i8; 4] = [48, -48, 0, 0];

pub(super) const SPRITE_SPAWN_FIRE_PHLEGM_Y_VELOCITIES: [i8; 4] = [0, 0, 48, -48];

pub(super) const OCTOROK_FIRE_LOOGIE_X_OFFSETS: [i8; 4] = [12, -12, 0, 0];

pub(super) const OCTOROK_FIRE_LOOGIE_Y_OFFSETS: [i8; 4] = [4, 4, 12, -12];

pub(super) const OCTOROK_FIRE_LOOGIE_X_VELOCITIES: [i8; 4] = [44, -44, 0, 0];

pub(super) const OCTOROK_FIRE_LOOGIE_Y_VELOCITIES: [i8; 4] = [0, 0, 44, -44];

pub(super) const MOBLIN_MATERIALIZE_SPEAR_X_OFFSETS: [i8; 4] = [11, -2, -3, 11];

pub(super) const MOBLIN_MATERIALIZE_SPEAR_Y_OFFSETS: [i8; 4] = [-3, -3, 3, -11];

pub(super) const MOBLIN_MATERIALIZE_SPEAR_X_VELOCITIES: [i8; 4] = [32, -32, 0, 0];

pub(super) const MOBLIN_MATERIALIZE_SPEAR_Y_VELOCITIES: [i8; 4] = [0, 0, 32, -32];

pub(super) const SNITCH_SPAWN_GUARD_X_OFFSETS: [u16; 3] = [0x0120, 0x0340, 0x02e0];

pub(super) const SNITCH_SPAWN_GUARD_Y_OFFSETS: [u16; 3] = [0x0100, 0x03b0, 0x0160];

pub(super) const KODONGO_SET_DIRECTION_X_VELOCITIES: [i8; 4] = [16, -16, 0, 0];

pub(super) const KODONGO_SET_DIRECTION_Y_VELOCITIES: [i8; 4] = [0, 0, 16, -16];

pub(super) const KODONGO_SPAWN_FIRE_X_OFFSETS: [i8; 4] = [8, -8, 0, 0];

pub(super) const KODONGO_SPAWN_FIRE_Y_OFFSETS: [i8; 4] = [0, 0, 8, -8];

pub(super) const KODONGO_SPAWN_FIRE_X_VELOCITIES: [i8; 4] = [24, -24, 0, 0];

pub(super) const KODONGO_SPAWN_FIRE_Y_VELOCITIES: [i8; 4] = [0, 0, 24, -24];

pub(super) const CREATE_SIX_BLUE_BALLS_X_VELOCITIES: [i8; 6] = [0, 24, 24, 0, -24, -24];

pub(super) const CREATE_SIX_BLUE_BALLS_Y_VELOCITIES: [i8; 6] = [-32, -16, 16, 32, 16, -16];

pub(super) const LANMOLA_SPAWN_SHRAPNEL_Y_VELOCITIES: [i8; 8] = [28, -28, 28, -28, 0, 36, 0, -36];

pub(super) const LANMOLA_SPAWN_SHRAPNEL_X_VELOCITIES: [i8; 8] = [-28, -28, 28, 28, -36, 0, 36, 0];

pub(super) const OCTOBALLOON_FORM_BABBY_X_VELOCITIES: [i8; 6] = [16, 11, -11, -16, -11, 11];

pub(super) const OCTOBALLOON_FORM_BABBY_Y_VELOCITIES: [i8; 6] = [0, 11, 11, 0, -11, -11];

pub(super) const RUPEE_PULL_SPAWN_PRIZE_X_VELOCITIES: [i8; 4] = [-18, -12, 12, 18];

pub(super) const RUPEE_PULL_SPAWN_PRIZE_Y_VELOCITIES: [i8; 4] = [16, 24, 24, 16];

pub(super) const RUPEE_PULL_SPAWN_PRIZE_TYPES: [u8; 3] = [0xd9, 0xda, 0xdb];

pub(super) const PIROGUSU_SPAWN_SPLASH_SPLASH_JITTER_OFFSETS: [u8; 4] = [3, 4, 5, 4];

pub(super) const LASER_EYE_FIRE_BEAM_SPAWN_XY: [i8; 6] = [12, -4, 4, 4, 12, -4];

pub(super) const LASER_EYE_FIRE_BEAM_SPAWN_XYVEL: [i8; 6] = [112, -112, 0, 0, 112, -112];

pub(super) const GET_POSITION_RELATIVE_TO_THE_GREAT_OVERLORD_GANON_X_OFFSETS: [i8; 2] = [20, -18];

pub(super) const GET_POSITION_RELATIVE_TO_THE_GREAT_OVERLORD_GANON_Y_OFFSETS: [i8; 2] = [-20, -20];

pub(super) const SPRITE_AD_OLD_MAN_OLD_MOUNTAIN_MAN_MSGS: [u16; 3] = [0x9e, 0x9f, 0xa0];

pub(super) const SPRITE_HAPPINESS_POND_COSTS: [u8; 4] = [5, 20, 25, 50];

pub(super) const SPRITE_HAPPINESS_POND_COST_HEX_VALUES: [u8; 4] = [5, 0x20, 0x25, 0x50];

pub(super) const HAPPINESS_POND_REWARD_MESSAGES: [u16; 5] = [0x8f, 0x90, 0x92, 0x91, 0x93];

pub(super) const HAPPINESS_POND_MAX_BOMBS_HEX: [u8; 8] =
    [0x10, 0x15, 0x20, 0x25, 0x30, 0x35, 0x40, 0x50];

pub(super) const HAPPINESS_POND_ARROW_REFILL_AMOUNTS: [u8; 8] =
    [0x30, 0x35, 0x40, 0x45, 0x50, 0x55, 0x60, 0x70];

pub(super) const HAPPINESS_POND_LUCK_MESSAGES: [u16; 4] = [0x150, 0x151, 0x152, 0x153];

pub(super) const HAPPINESS_POND_LUCK_VALUES: [u8; 4] = [1, 0, 0, 2];

pub(super) const WISH_POND2_DRAW_WISH_POND_ITEM_DRAW_FRAMES: [DrawMultipleData; 8] = [
    DrawMultipleData {
        x: 32,
        y: -64,
        char_flags: 0x0024,
        ext: 0,
    },
    DrawMultipleData {
        x: 32,
        y: -56,
        char_flags: 0x0034,
        ext: 0,
    },
    DrawMultipleData {
        x: 32,
        y: -64,
        char_flags: 0x0024,
        ext: 0,
    },
    DrawMultipleData {
        x: 32,
        y: -56,
        char_flags: 0x0034,
        ext: 0,
    },
    DrawMultipleData {
        x: 32,
        y: -64,
        char_flags: 0x0024,
        ext: 2,
    },
    DrawMultipleData {
        x: 32,
        y: -64,
        char_flags: 0x0024,
        ext: 2,
    },
    DrawMultipleData {
        x: 32,
        y: -64,
        char_flags: 0x0024,
        ext: 2,
    },
    DrawMultipleData {
        x: 32,
        y: -64,
        char_flags: 0x0024,
        ext: 2,
    },
];

pub(super) const BUZZBLOB_SELECT_NEW_DIRECTION_X_VELOCITIES: [i8; 8] = [3, 2, -2, -3, -2, 2, 0, 0];

pub(super) const BUZZBLOB_SELECT_NEW_DIRECTION_Y_VELOCITIES: [i8; 8] = [0, 2, 2, 0, -2, -2, 0, 0];

pub(super) const BUZZBLOB_SELECT_NEW_DIRECTION_DELAYS: [u8; 8] = [48, 48, 48, 48, 48, 48, 64, 64];

pub(super) const LUMBERJACK_CHECK_PROXIMITY_X_OFFSETS: [u16; 2] = [48, 52];

pub(super) const LUMBERJACK_CHECK_PROXIMITY_Y_OFFSETS: [u16; 2] = [19, 20];

pub(super) const LUMBERJACK_CHECK_PROXIMITY_WIDTHS: [u16; 2] = [98, 106];

pub(super) const LUMBERJACK_CHECK_PROXIMITY_HEIGHTS: [u16; 2] = [37, 40];

pub(super) const SPRITE_PREP_DEBIRANDO_PIT_DEBIRANDO_OAM_FLAGS: [u8; 2] = [6, 8];

pub(super) const SPRITE_PREP_BONK_ITEM_DASH_ITEM_MASK: [u16; 2] = [0x4000, 0x2000];

pub(super) const SPRITE_PREP_SHOPKEEPER_SHOP_KEEPER_WHERE: [u8; 13] = [
    0x0f, 0x10, 0x00, 0x06, 0x18, 0x12, 0x1e, 0xff, 0x1f, 0x23, 0x24, 0x25, 0x27,
];

pub(super) const SHOP_KEEPER_SPAWN_SHOP_ITEM_SHOP_KEEPER_ITEM_X: [i16; 3] = [-44, 8, 60];

pub(super) const SPRITE_PREP_STORYTELLER_ROOMS: [u8; 5] = [0x0e, 0x0e, 0x12, 0x1a, 0x14];

pub(super) const SPRITE_PREP_ADULTS_HUMAN_MULTI_TYPES: [u8; 3] = [3, 0xe1, 0x19];

pub(super) const SPRITE_PREP_TEKTITE_OAM_FLAGS: [u8; 2] = [9, 7];

pub(super) const SPRITE_PREP_TEKTITE_HEALTH_VALUES: [u8; 2] = [8, 12];

pub(super) const SPRITE_PREP_TEKTITE_BUMP_DAMAGE_VALUES: [u8; 2] = [3, 5];

pub(super) const CHAIN_CHOMP_MOVE_CHAIN_MULS: [u8; 6] = [205, 154, 102, 51, 8, 0xbd];

pub(super) const SPRITE_PREP_FAIRY_POND_OAM_FLAGS: [u8; 2] = [10, 2];

pub(super) const SPRITE_PREP_ANTIFAIRY_X_VELOCITIES: [i8; 2] = [16, -16];

pub(super) const SPRITE_PREP_ANTIFAIRY_CIRCLE_X_OFFSETS: [i16; 3] = [10, 20, 10];

pub(super) const SPRITE_PREP_ANTIFAIRY_CIRCLE_Y_OFFSETS: [i16; 3] = [-10, 0, 10];

pub(super) const SPRITE_PREP_ANTIFAIRY_CIRCLE_X_VELOCITIES: [i8; 3] = [18, 0, -18];

pub(super) const SPRITE_PREP_ANTIFAIRY_CIRCLE_Y_VELOCITIES: [i8; 3] = [0, 18, 0];

pub(super) const SPRITE_PREP_ANTIFAIRY_CIRCLE_ATTRS: [u8; 3] = [1, 1, 0];

pub(super) const SPRITE_PREP_ANTIFAIRY_CIRCLE_B: [u8; 3] = [0, 1, 1];

pub(super) const SPRITE_PREP_OCTOROK_BUMP_DAMAGE_VALUES: [u8; 2] = [3, 5];

pub(super) const SPRITE_PREP_OCTOROK_HEALTH_VALUES: [u8; 2] = [2, 4];

pub(super) const SPRITE_PREP_TALKING_TREE_SPAWN_EYEBALL_TALKING_TREE_SPAWN_X: [i16; 2] = [-4, 14];

pub(super) const SPRITE_PREP_SWAMOLA_INITIALIZE_SEGMENTS_BUGGY_SWAMOLA_LOOKUP: [usize; 6] =
    [0x1c, 0xa9, 0x03, 0x9d, 0x90, 0x0d];

pub(super) const SPRITE_PREP_LANMOLAS_INIT_DELAY: [u8; 3] = [128, 192, 255];

pub(super) const SPRITE_PREP_HARDHAT_BEETLE_OAM_FLAGS: [u8; 2] = [6, 8];

pub(super) const SPRITE_PREP_HARDHAT_BEETLE_HEALTH_VALUES: [u8; 2] = [32, 6];

pub(super) const SPRITE_PREP_HARDHAT_BEETLE_ATTRS: [u8; 2] = [16, 12];

pub(super) const SPRITE_PREP_HARDHAT_BEETLE_STATE: [u8; 2] = [1, 3];

pub(super) const SPRITE_PREP_HARDHAT_BEETLE_FLAGS5_VALUES: [u8; 2] = [2, 6];

pub(super) const SPRITE_PREP_HARDHAT_BEETLE_BUMP_DAMAGE_VALUES: [u8; 2] = [5, 3];

pub(super) const SPRITE_PREP_CRYSTAL_SWITCH_CRYSTAL_SWITCH_PAL: [u8; 2] = [2, 4];

pub(super) const SPRITE_PREP_AGAHNIM_OAM_FLAGS: [u8; 2] = [11, 7];

pub(super) const ARRGHUS_HANDLE_PUFFS_PUFF_ORBIT_BASE_ANGLES: [u16; 13] = [
    0, 0x40, 0x80, 0xc0, 0x100, 0x140, 0x180, 0x1c0, 0, 0x66, 0xcc, 0x132, 0x198,
];

pub(super) const ARRGHUS_HANDLE_PUFFS_PUFF_ORBIT_ANGLE_XOR_MASKS: [u16; 13] =
    [0, 0, 0, 0, 0, 0, 0, 0, 0x1ff, 0x1ff, 0x1ff, 0x1ff, 0x1ff];

pub(super) const ARRGHUS_HANDLE_PUFFS_PUFF_ORBIT_PHASE_OFFSETS: [u8; 13] = [
    0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x0c, 0x0c, 0x0c, 0x0c, 0x0c,
];

pub(super) const ARRGHUS_HANDLE_PUFFS_PUFF_ORBIT_WAVE_SHIFTS: [i8; 52] = [
    0, -1, -2, -3, -4, -5, -6, -6, -5, -4, -3, -2, -1, 0, -1, -2, -3, -4, -5, -6, -6, -5, -4, -3,
    -2, -1, 0, -1, -2, -3, -4, -5, -6, -6, -5, -4, -3, -2, -1, 0, -1, -2, -3, -4, -5, -6, -6, -5,
    -4, -3, -2, -1,
];
