// Methods ported from zelda3/src/sprite.c and included inside ZeldaState.

use super::*;
use crate::types::{sign16, sign8, PairU8, Point16U, PointU8, ProjectSpeedRet, SpriteHitBox};

const SPRITE_C_SPRITE: usize = 0x0db0;
const SPRITE_DELAY_AUX1_SPRITE: usize = 0x0e00;
const SPRITE_HEALTH_SPRITE: usize = 0x0e50;
const SPRITE_DELAY_AUX3_SPRITE: usize = 0x0ee0;
const SPRITE_DELAY_AUX2_SPRITE: usize = 0x0e10;
const SPRITE_INIT_TABLE_LEN: usize = 243;
const SPRITE_INIT_FLAGS2_TABLE: usize = 0;
const SPRITE_INIT_HEALTH_TABLE: usize = 1;
const SPRITE_INIT_BUMP_DAMAGE_TABLE: usize = 2;
const SPRITE_INIT_FLAGS3_TABLE: usize = 3;
const SPRITE_INIT_FLAGS4_TABLE: usize = 4;
const SPRITE_INIT_FLAGS_TABLE: usize = 5;
const SPRITE_INIT_FLAGS5_TABLE: usize = 6;
const SPRITE_INIT_DEFL_BITS_TABLE: usize = 7;
const SPRITE_ANIM_CLOCK_SPRITE: usize = 0x0ec0;
const SPRITE_UNK2_SPRITE: usize = 0x0bb0;

const K_SPRITE_PREP_AND_DRAW_SINGLE_LARGE_TAB1: [u8; 236] = [
    200, 0, 107, 0, 0, 0, 0, 0, 0, 203, 0, 8, 10, 11, 0, 0, 13, 0, 0, 86, 0, 0, 15, 17, 0, 19, 0,
    0, 0, 0, 20, 0, 21, 27, 0, 42, 42, 248, 0, 182, 0, 0, 0, 170, 0, 0, 28, 0, 0, 0, 0, 0, 0, 0, 0,
    243, 243, 0, 187, 39, 0, 0, 66, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 15, 63, 0, 0, 0, 64, 64, 68, 0,
    0, 0, 0, 71, 70, 0, 0, 72, 74, 101, 101, 0, 0, 0, 0, 0, 143, 0, 0, 76, 78, 78, 78, 78, 0, 48,
    36, 50, 56, 60, 129, 0, 82, 0, 0, 0, 0, 0, 0, 92, 0, 98, 94, 0, 0, 0, 101, 102, 0, 0, 0, 0,
    110, 14, 0, 59, 66, 0, 0, 117, 120, 123, 0, 0, 207, 0, 132, 141, 141, 141, 141, 0, 148, 117,
    160, 0, 0, 162, 166, 0, 0, 0, 177, 0, 181, 0, 189, 0, 0, 0, 105, 0, 0, 0, 0, 0, 92, 0, 214,
    230, 0, 0, 0, 219, 218, 233, 0, 0, 190, 192, 106, 0, 249, 215, 0, 0, 0, 216, 0, 0, 222, 227, 0,
    0, 0, 235, 0, 0, 0, 0, 0, 0, 244, 244, 29, 31, 31, 31, 32, 32, 32, 33, 34, 35, 35, 37, 40, 106,
    246, 41, 0, 0, 205, 206,
];

const K_SPRITE_PREP_AND_DRAW_SINGLE_LARGE_TAB2: [u8; 251] = [
    0xa0, 0xa2, 0xa0, 0xa2, 0x80, 0x82, 0x80, 0x82, 0xea, 0xec, 0x84, 0x4e, 0x61, 0xbd, 0x8c, 0x20,
    0x22, 0xc0, 0xc2, 0xe6, 0xe4, 0x82, 0xaa, 0x84, 0xac, 0x80, 0xa0, 0xca, 0xaf, 0x29, 0x39, 0x0b,
    0x6e, 0x60, 0x62, 0x63, 0x4c, 0xea, 0xec, 0x24, 0x6b, 0x24, 0x22, 0x24, 0x26, 0x20, 0x30, 0x21,
    0x2a, 0x24, 0x86, 0x88, 0x8a, 0x8c, 0x8e, 0xa2, 0xa4, 0xa6, 0xa8, 0xaa, 0x84, 0x80, 0x82, 0x6e,
    0x40, 0x42, 0xe6, 0xe8, 0x80, 0x82, 0xc8, 0x8d, 0xe3, 0xe5, 0xc5, 0xe1, 0x04, 0x24, 0x0e, 0x2e,
    0x0c, 0x0a, 0x9c, 0xc7, 0xb6, 0xb7, 0x60, 0x62, 0x64, 0x66, 0x68, 0x6a, 0xe4, 0xf4, 0x02, 0x02,
    0x00, 0x04, 0xc6, 0xcc, 0xce, 0x28, 0x84, 0x82, 0x80, 0xe5, 0x24, 0x00, 0x02, 0x04, 0xa0, 0xaa,
    0xa4, 0xa6, 0xac, 0xa2, 0xa8, 0xa6, 0x88, 0x86, 0x8e, 0xae, 0x8a, 0x42, 0x44, 0x42, 0x44, 0x64,
    0x66, 0xcc, 0xcc, 0xca, 0x87, 0x97, 0x8e, 0xae, 0xac, 0x8c, 0x8e, 0xaa, 0xac, 0xd2, 0xf3, 0x84,
    0xa2, 0x84, 0xa4, 0xe7, 0x8a, 0xa8, 0x8a, 0xa8, 0x88, 0xa0, 0xa4, 0xa2, 0xa6, 0xa6, 0xa6, 0xa6,
    0x7e, 0x7f, 0x8a, 0x88, 0x8c, 0xa6, 0x86, 0x8e, 0xac, 0x86, 0xbb, 0xac, 0xa9, 0xb9, 0xaa, 0xba,
    0xbc, 0x8a, 0x8e, 0x8a, 0x86, 0x0a, 0xc2, 0xc4, 0xe2, 0xe4, 0xc6, 0xea, 0xec, 0xff, 0xe6, 0xc6,
    0xcc, 0xec, 0xce, 0xee, 0x4c, 0x6c, 0x4e, 0x6e, 0xc8, 0xc4, 0xc6, 0x88, 0x8c, 0x24, 0xe0, 0xae,
    0xc0, 0xc8, 0xc4, 0xc6, 0xe2, 0xe0, 0xee, 0xae, 0xa0, 0x80, 0xee, 0xc0, 0xc2, 0xbf, 0x8c, 0xaa,
    0x86, 0xa8, 0xa6, 0x2c, 0x28, 0x06, 0xdf, 0xcf, 0xa9, 0x46, 0x46, 0xea, 0xc0, 0xc2, 0xe0, 0xe8,
    0xe2, 0xe6, 0xe4, 0x0b, 0x8e, 0xa0, 0xec, 0xea, 0xe9, 0x48, 0x58,
];

const SPRITE_INIT_TABLES_HEX: &str = concat!(
    "0102018281848484020f020120030484010504018004a283040282628280808501a503040483020182a2a2a3aaa3a482828382808282a580a4828182",
    "8282810608080808060808080607070202220101208207850f21058302010101010707070700858303a4000000000904a000010000038b86c2828104",
    "8221060301030303000004050503010200000002070001018706008302222222220403050101040102080880210303030202088fa18180808080a180",
    "81818681828280808306000005040605020000050404070b0c0c060603a4048281831010818282828383838182838381828182a0a1a3a1a1a1838583",
    "8383830c06ff0303030303020c04ff00030c020014040400ff0002030800000000000008030802020003ff0003030303030303030003000303030003",
    "000000000302ff02060408060806040808080404020202ff08ff30100808ff020000ffffffffffffffffff0404ffffffff100300020401ff04ff0000",
    "0000ff000060ff18ffffff0304ff10080800ff2020202020080804084030ff02ffffffff10040204040808081040400804080404080c100000000000",
    "0000000000000000000000008030ffffffff08000000200008052828285a10184000040000ffff000000000000000000000000000000000000000000",
    "000000000000838381020202020201130101010108010108050340040002038500010040000006000503010000030000000000000000000000000000",
    "004000000000000002020001010301030101030303010301010101010111140101020500000404080808080400040302020202020301000001800501",
    "000000400004000014040604040404030404040104041505040515150305000515050506060606050306050503030306171515050501858305040000",
    "000000000000000000000000000000171705050504030210000600050717171715070610000303001919000000000000000000000000010000000000",
    "000000000000000000190b1b4b4141414d1d011d198d1b099d3d01091140014d19071d59804d4001491b4103131541181b41470f494b4d4147494d49",
    "404d47494174475b5851491d5d03191b171917191b1717171b0d091919495d5b490d0313411b5b5d43434d4d4d4d490100414d4d4d4d1d09c40d0d",
    "0903034b474749494147368b491d494343430b410d070b1d430d430d1d4d4d1b1b0a0b00050d010101010b050101010717190d0d804d1917190b090d",
    "4a1249c3c3c3c376405941584f735b4441510a0b0b4b00405b0d00000d4b0b59410b0d010d0d00504c44510101f2f8f4f2d4d4d4f8f8f4f4d8f8d8df",
    "c869c1d2d2dcc7c1c7c7c7c10000004343434343000000001c0000020103000003c00700000007454300400d00000000000000000707070707070d07",
    "070707030707074003070d000707000009121212121212121212121200000000801209090040000c000000404010102e2e401e53000a000000001212",
    "400000401900000a0d0a0a800a41004000490000c0004000004000000980c000400000800000185a00d4d4d4d4004000808040404000091d00000000",
    "00000000000a1b1b1b1b4100030707030a00010a0a0900000000090000404000000000898080001c004000001c070303444444444444444444444443",
    "444340c0c0c7c3c3c01b081b1b1b030000000000000000000a0001300000201000000100000000000000082000040000000000000001040000000000",
    "000000000000000000000000000000000000686061616161616161616161610000000000000202020000700000009090000000000000000000000000",
    "0060600000000000000000000000800000020000700000000000000000b000c20020000200000000000200b000000000000a0a000000004020000",
    "000000000000000000000000000000000000000000000c20000000000000400000000000002020202000000000000000a0a101010100000001010",
    "101000100000000000000000000000000000839684808080808002000280a08397808094910700800080929680a00000008004808206060000808080",
    "80808080808080808080808080808080800000808090809191919791959593971491928182828085808080040480918080808080808080008080828a",
    "808080809291808281818081808080808080808080809780808080c28015151706008000c01340000206101400004000000000134611808000000010",
    "000000161616818782008080000000008080000000000000000000000000000000800000001700120000000000101700400100000000000000000000",
    "000000000040000000000000000080000000000000000044202020202000810000480000000000000000000400000000482480000000200000008000",
    "000000000000000080800000000000008000800002000000048000000000000000000000000000000084008105014008a00000000000848484840880",
    "808000808080800008800000004000000000000000000201000004000000008004040000480000040001010000800000004008080808000000808000",
    "000004010500000000000000040200808080808280000080000080800000010140000004000000000000000405050580800000000000000000020202",
    "020202020202020202020202020200828208802080808020",
);

// Regenerated directly from zelda3/src/sprite.c kSpriteInit_* tables.
// The older packed blob above drifted by four bytes after BumpDamage, which
// shifted Flags3/Flags4/Flags/Flags5/DeflBits for later sprite types.
const SPRITE_INIT_TABLES_C_HEX: &str = concat!(
    "0102018281848484020f020120030484010504018004a283040282628280808501a503040483020182a2a2a3aaa3a482828382808282a580a482818282828106",
    "08080808060808080607070202220101208207850f21058302010101010707070700858303a4000000000904a000010000038b86c28281048221060301030303",
    "000004050503010200000002070001018706008302222222220403050101040102080880210303030202088fa18180808080a180818186818282808083060000",
    "05040605020000050404070b0c0c060603a4048281831010818282828383838182838381828182a0a1a3a1a1a18385838383830c06ff0303030303020c04ff00",
    "030c020014040400ff0002030800000000000008030802020003ff0003030303030303030003000303030003000000000302ff02060408060806040808080404",
    "020202ff08ff30100808ff020000ffffffffffffffffff0404ffffffff100300020401ff04ff00000000ff000060ff18ffffff0304ff10080800ff2020202020",
    "080804084030ff02ffffffff10040204040808081040400804080404080c1000000000000000000000000000000000008030ffffffff08000000200008052828",
    "285a10184000040000ffff0000000000000000000000000000000000000000000000000000008383810202020202011301010101080101080503400400020385",
    "00010040000006000503010000030000000000000000000000000000004000000000000002020001010301030101030303010301010101010111140101020500",
    "00040408080808040004030202020202030100000180050100000040000400001404060404040403040404010404150504051515030500051505050606060605",
    "03060505030303061715150505018583050400000000000000000000000000000000001717050505040302100006000507171717150706100003030019190000",
    "00000000000000000010000000000000000000000000000000190b1b4b4141414d1d011d198d1b099d3d01091140014d19071d59804d4001491b410313154118",
    "1b41470f494b4d4147494d49404d47494174475b5851491d5d03191b171917191b1717171b0d091919495d5b490d0313411b5b5d43434d4d4d4d4d490100414d",
    "4d4d4d1d09c40d0d0903034b474749494147368b491d494343430b410d070b1d430d430d1d4d4d1b1b0a0b00050d010101010b050101010717190d0d804d1917",
    "190b090d4a1249c3c3c3c376405941584f735b4441510a0b0b4b00405b0d00000d4b0b59410b0d010d0d00504c44510101f2f8f4f2d4d4d4f8f8f4f4d8f8d8df",
    "c869c1d2d2dcc7c1c7c7c7c10000004343434343000000001c0000020103000003c00700000007454300400d00000000000000000707070707070d0707070703",
    "0707074003070d000707000009121212121212121212121200000000801209090040000c000000404010102e2e401e53000a000000001212400000401900000a",
    "0d0a0a800a41004000490000c0004000004000000980c000400000800000185a00d4d4d4d4004000808040404000091d0000000000000000000a1b1b1b1b4100",
    "030707030a00010a0a0900000000090000404000000000898080001c004000001c070303444444444444444444444443444340c0c0c7c3c3c01b081b1b1b0300",
    "00000000000000000a00013000002010000001000000000000000820000400000000000000010400000000000000000000000000000000000000000000006860",
    "61616161616161616161610000000000000202020000700000009090000000000000000000000000006060000000000000000000000080000002000070000000",
    "0000000000b000c20020000200000000000200b000000000000000a0a000000004020000000000000000000000000000000000000000000000000000c2000000",
    "0000000400000000000002020202000000000000000a0a101010100000001010101000100000000000000000000000000000839684808080808002000280a083",
    "97808094910700800080929680a00000008004808206060000808080808080808080808080808080808080808000008080908091919197919595939714919281",
    "82828085808080040480918080808080808080008080828a808080809291808281818081808080808080808080809780808080c28015151706008000c0134000",
    "02061014000040000000001346118080000000100000001616168187820080800000000080800000000000000000000000000000008000000017001200000000",
    "00101700400100000000000000000000000000000040000000000000000080000000000000000044202020202000810000480000000000000000000400000000",
    "482480000000200000008000000000000000000080800000000000008000800002000000048000000000000000000000000000000084008105014008a0000000",
    "00008484848408808080008080808000088000000040000000000000000002010000040000000080040400004800000400010100008000000040080808080000",
    "00808000000004010500000000000000040200808080808280000080000080800000010140000004000000000000000405050580800000000000000000020202",
    "020202020202020202020202020200828208802080808020",
);

// Additional sprite RAM addresses used by the common helpers ported below.
// These mirror the C variable declarations in zelda3/src/variables.h.
const SPRITE_Y_RECOIL: usize = 0x0f30;
const SPRITE_I_LOC: usize = 0x1f9c2;
const SPRITE_UNK3_LOC: usize = 0x1fa1c;
const SPRITE_UNK4_LOC: usize = 0x1fa2c;
const SPRITE_UNK5_LOC: usize = 0x1fa3c;
const SPRITE_DRAW_PRIORITY_OVERRIDE: usize = 0x0cfe;
const SPRITE_PICKUP_SLOT_CACHE: usize = 0x0fb2;
const SPRITE_F: usize = 0x0ea0;
const SPRITE_AI_STATE_SPRITE: usize = 0x0d80;
const ANCILLA_X_LO_SPRITE: usize = 0x0c04;
const ANCILLA_Y_LO_SPRITE: usize = 0x0bfa;
const SPRITE_BUMP_DAMAGE: usize = 0x0cd2;
const SPRITE_FLAGS_SPRITE: usize = 0x0b6b;
const SPRITE_UNK1_LOC: usize = 0x1fa4c;
const DAMAGE_TYPE_DETERMINER_SPRITE: usize = 0x0cf2;
const DUNG_SECRETS_UNK1_SPRITE: usize = 0x0b9c;
const SPRITE_WALLCOLL: usize = 0x0e70;
const TMP_COUNTER_SPRITE: usize = 0x0fb5;
const SPRITE_GIVE_DAMAGE_SPRITE: usize = 0x0ce2;
const IS_IN_DARK_WORLD_SPRITE: usize = 0x0fff;
const GARNISH_Y_LO_SPRITE: usize = 0x1f81e;
const GARNISH_X_LO_SPRITE: usize = 0x1f83c;
const GARNISH_Y_HI_SPRITE: usize = 0x1f85a;
const GARNISH_X_HI_SPRITE: usize = 0x1f878;
const GARNISH_Y_VEL_SPRITE: usize = 0x1f896;
const GARNISH_X_VEL_SPRITE: usize = 0x1f8b4;
const GARNISH_Y_SUBPIXEL_SPRITE: usize = 0x1f8d2;
const GARNISH_X_SUBPIXEL_SPRITE: usize = 0x1f8f0;
const GARNISH_ACTIVE_SPRITE: usize = 0x0fb4;
const GARNISH_COUNTDOWN_SPRITE: usize = 0x1f90e;
const K_CHECK_DAMAGE_FROM_PLAYER_CARRY: u8 = 1;
const K_CHECK_DAMAGE_FROM_PLAYER_NE: u8 = 2;
const GARNISH_SPRITE_SPRITE: usize = 0x1f92c;
const GARNISH_FLOOR_SPRITE: usize = 0x1f968;
const GARNISH_OAM_FLAGS_SPRITE: usize = 0x1f9fe;
const OVERLORD_GEN3_SPRITE: usize = 0x0b38;
const OVERLORD_FLOOR_SPRITE: usize = 0x0b40;
const OVERLORD_SPAWNED_IN_AREA_SPRITE: usize = 0x0cca;
const OVERWORLD_AREA_INDEX_SPRITE: usize = 0x040a;
const OVERWORLD_SECRET_SUBST_CTR_SPRITE: usize = 0x0cf7;
const REPULSESPARK_FLOOR_STATUS_SPRITE: usize = 0x0b68;
const REPULSESPARK_TIMER_SPRITE: usize = 0x0fac;
const REPULSESPARK_X_LO_SPRITE: usize = 0x0fad;
const REPULSESPARK_Y_LO_SPRITE: usize = 0x0fae;
const SRAM_PROGRESS_INDICATOR_SPRITE: usize = 0x0f3c5;
const SPRITE_RESET_SCRATCH_A: usize = 0x0ff8;
const SPRITE_RESET_SCRATCH_B: usize = 0x0ffb;
const ACTIVATE_BOMB_TRAP_OVERLORD_SPRITE: usize = 0x0cf4;
const OAM_REGION_BASE_SPRITE: usize = 0x0fe0;
const SPR_RANGED_BASED_TOGGLER: usize = 0x0fb7;
const SPRCOLL_X_BASE_SPRITE: usize = 0x0fbc;
const SPRCOLL_Y_BASE_SPRITE: usize = 0x0fbe;
const SPRITE_WHERE_IN_OVERWORLD: usize = 0x1df80;
const OVERLORD_OFFSET_SPRITE_POS_SPRITE: usize = 0x0b48;
const FEATURES0_EXTEND_SCREEN64_SPRITE: u32 = 1;
const FEATURES0_COLLECT_ITEMS_WITH_SWORD_SPRITE: u32 = 16;

const OVERWORLD_AREA_SPRCOLL_SIZES: [u8; 192] = [
    0x04, 0x04, 0x02, 0x04, 0x04, 0x04, 0x04, 0x02, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04,
    0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x04, 0x04, 0x02, 0x04, 0x04, 0x02, 0x04, 0x04,
    0x04, 0x04, 0x02, 0x04, 0x04, 0x02, 0x04, 0x04, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02,
    0x04, 0x04, 0x02, 0x02, 0x02, 0x04, 0x04, 0x02, 0x04, 0x04, 0x02, 0x02, 0x02, 0x04, 0x04, 0x02,
    0x04, 0x04, 0x02, 0x04, 0x04, 0x04, 0x04, 0x02, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04,
    0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x04, 0x04, 0x02, 0x04, 0x04, 0x02, 0x04, 0x04,
    0x04, 0x04, 0x02, 0x04, 0x04, 0x02, 0x04, 0x04, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02,
    0x04, 0x04, 0x02, 0x02, 0x02, 0x04, 0x04, 0x02, 0x04, 0x04, 0x02, 0x02, 0x02, 0x04, 0x04, 0x02,
    0x04, 0x04, 0x02, 0x04, 0x04, 0x04, 0x04, 0x02, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04,
    0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x04, 0x04, 0x02, 0x04, 0x04, 0x02, 0x04, 0x04,
    0x04, 0x04, 0x02, 0x04, 0x04, 0x02, 0x04, 0x04, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02,
    0x04, 0x04, 0x02, 0x02, 0x02, 0x04, 0x04, 0x02, 0x04, 0x04, 0x02, 0x02, 0x02, 0x04, 0x04, 0x02,
];

// Overlord x/y RAM addresses used by Sprite_SpawnDynamicallyEx to
// surface the overlord coordinates back into the SpriteSpawnInfo struct.
// Mirror the overlord.rs declarations (which are private to that module).
const SPRITE_OVERLORD_X_LO: usize = 0x0b08;
const SPRITE_OVERLORD_X_HI: usize = 0x0b10;
const SPRITE_OVERLORD_Y_LO: usize = 0x0b18;
const SPRITE_OVERLORD_Y_HI: usize = 0x0b20;

// Word-wide alias used in Sprite_SpawnDynamicallyEx when player_is_indoors
// is false. `sprite_N_word[j]` is the 16-bit view of `sprite_N[j]`.
// variables.h:1228 sets byte_7FFABC at 0x1fabc; sprite_N lives at 0x0bc0.
const SPRITE_N_WORD: usize = 0x0bc0;

// Dual-layer tile-collision cache referenced by Sprite_CheckTileCollision2.
// variables.h:1228 — `byte_7FFABC` lives at 0x1fabc.
const BYTE_7FFABC: usize = 0x1fabc;

// Single-byte tile-type cache used across Sprite_* tile helpers.
// variables.h:755 — `sprite_tiletype` lives at 0x0fa5.
const SPRITE_TILETYPE_SPR: usize = 0x0fa5;
const ITEM_DROP_LUCK_SPRITE: usize = 0x0cf9;
const LUCK_KILL_COUNTER_SPRITE: usize = 0x0cfa;
const NUM_SPRITES_KILLED_SPRITE: usize = 0x0cfb;
const PRIZES_ARR1_SPRITE: usize = 0x0fc7;
const ALT_SPRITE_STATE_SPRITE: usize = 0x1d00;
const ALT_SPRITE_TYPE_SPRITE: usize = 0x1d10;
const ALT_SPRITE_X_LO_SPRITE: usize = 0x1d20;
const ALT_SPRITE_X_HI_SPRITE: usize = 0x1d30;
const ALT_SPRITE_Y_LO_SPRITE: usize = 0x1d40;
const ALT_SPRITE_Y_HI_SPRITE: usize = 0x1d50;
const ALT_SPRITE_GRAPHICS_SPRITE: usize = 0x1d60;
const ALT_SPRITE_A_SPRITE: usize = 0x1d70;
const ALT_SPRITE_HEAD_DIR_SPRITE: usize = 0x1d80;
const ALT_SPRITE_OAM_FLAGS_SPRITE: usize = 0x1d90;
const ALT_SPRITE_OBJ_PRIO_SPRITE: usize = 0x1da0;
const ALT_SPRITE_D_SPRITE: usize = 0x1db0;
const ALT_SPRITE_FLAGS2_SPRITE: usize = 0x1dc0;
const ALT_SPRITE_FLOOR_SPRITE: usize = 0x1dd0;
const ALT_SPRITE_SPAWNED_FLAG_SPRITE: usize = 0x1de0;
const ALT_SPRITE_FLAGS3_SPRITE: usize = 0x1df0;
const ALT_SPRITE_B_SPRITE: usize = 0x1fa5c;
const ALT_SPRITE_C_SPRITE: usize = 0x1fa6c;
const ALT_SPRITE_E_SPRITE: usize = 0x1fa7c;
const ALT_SPRITE_SUBTYPE2_SPRITE: usize = 0x1fa8c;
const ALT_SPRITE_HEIGHT_ABOVE_SHADOW_SPRITE: usize = 0x1fa9c;
const ALT_SPRITE_DELAY_MAIN_SPRITE: usize = 0x1faac;
const ALT_SPRITE_I_SPRITE: usize = 0x1facc;
const ALT_SPRITE_IGNORE_PROJECTILE_SPRITE: usize = 0x1fadc;

// `Sprite_DrawMultiple` consumes a table of these draws (sprite.h:38-42).
#[derive(Copy, Clone)]
pub(super) struct DrawMultipleData {
    pub x: i8,
    pub y: i8,
    pub char_flags: u16,
    pub ext: u8,
}

// `Sprite_SetSpawnedCoordinates` consumes this struct (sprite.h:28-34).
#[derive(Copy, Clone, Default)]
pub(super) struct SpriteSpawnInfo {
    pub r0_x: u16,
    pub r2_y: u16,
    pub r4_z: u8,
    pub r5_overlord_x: u16,
    pub r7_overlord_y: u16,
}

#[derive(Copy, Clone, Default, serde::Serialize, serde::Deserialize)]
pub(super) struct PrepOamCoordsRet {
    pub x: u16,
    pub y: u16,
    pub r4: u8,
    pub flags: u8,
}

fn hex_nibble(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        b'A'..=b'F' => byte - b'A' + 10,
        _ => 0,
    }
}

fn sprite_init_value(table: usize, ty: u8) -> u8 {
    let idx = (table * SPRITE_INIT_TABLE_LEN + ty as usize) * 2;
    let bytes = SPRITE_INIT_TABLES_C_HEX.as_bytes();
    (hex_nibble(bytes[idx]) << 4) | hex_nibble(bytes[idx + 1])
}

fn empty_sprite_hit_box() -> SpriteHitBox {
    SpriteHitBox {
        r0_xlo: 0,
        r8_xhi: 0,
        r1_ylo: 0,
        r9_yhi: 0,
        r2: 0,
        r3: 0,
        r4_spr_xlo: 0,
        r10_spr_xhi: 0,
        r5_spr_ylo: 0,
        r11_spr_yhi: 0,
        r6_spr_xsize: 0,
        r7_spr_ysize: 0,
    }
}

impl ZeldaState {
    pub(super) fn sprite_where_in_room_mask(&self, room: u16) -> u16 {
        read_le_u16(&self.ram, SPRITE_WHERE_IN_ROOM + usize::from(room) * 2)
    }

    pub(super) fn set_sprite_where_in_room_mask(&mut self, room: u16, mask: u16) {
        write_le_u16(
            &mut self.ram,
            SPRITE_WHERE_IN_ROOM + usize::from(room) * 2,
            mask,
        );
    }

    pub(super) fn prepare_apply_rumble_to_sprites(&mut self) {
        const APPLY_RUMBLE_X: [i8; 4] = [-32, -32, -32, 16];
        const APPLY_RUMBLE_Y: [i8; 4] = [-32, 32, -24, -24];
        const APPLY_RUMBLE_WH: [u8; 6] = [0x50, 0x50, 0x20, 0x20, 0x50, 0x50];

        let j = (self.ram[LINK_DIRECTION_FACING] >> 1) as usize;
        let player = self.player_state_view();
        let x = player.x().wrapping_add(APPLY_RUMBLE_X[j] as i16 as u16);
        let y = player.y().wrapping_add(APPLY_RUMBLE_Y[j] as i16 as u16);
        let mut hb = SpriteHitBox {
            r0_xlo: x as u8,
            r1_ylo: y as u8,
            r2: APPLY_RUMBLE_WH[j],
            r3: APPLY_RUMBLE_WH[j + 2],
            r4_spr_xlo: 0,
            r10_spr_xhi: 0,
            r5_spr_ylo: 0,
            r11_spr_yhi: 0,
            r6_spr_xsize: 0,
            r7_spr_ysize: 0,
            r8_xhi: (x >> 8) as u8,
            r9_yhi: (y >> 8) as u8,
        };
        self.entity_apply_rumble_to_sprites(&mut hb);
    }

    // void Oam_ResetRegionBases() {  // 8683d3
    //   memcpy(oam_region_base, kOam_ResetRegionBases, 12);
    // }
    pub(super) fn oam_reset_region_bases(&mut self) {
        const OAM_RESET_REGION_BASES: [u16; 6] = [0x0030, 0x01d0, 0x0000, 0x0030, 0x0120, 0x0140];
        for (i, value) in OAM_RESET_REGION_BASES.into_iter().enumerate() {
            write_le_u16(&mut self.ram, OAM_REGION_BASE_SPRITE + i * 2, value);
        }
    }

    // void Sprite_SpawnImmediatelySmashedTerrain(uint8 what, uint16 x, uint16 y) {  // 86812d
    //   uint8 bak1 = flag_is_sprite_to_pick_up;
    //   uint8 bak2 = sprite_pickup_slot_cache;
    //   int k = Sprite_SpawnThrowableTerrain_silently(what, x, y);
    //   if (k >= 0)
    //     ThrowableScenery_TransmuteToDebris(k);
    //   sprite_pickup_slot_cache = bak2;
    //   flag_is_sprite_to_pick_up = bak1;
    // }
    pub(super) fn sprite_spawn_immediately_smashed_terrain(&mut self, what: u8, x: u16, y: u16) {
        let bak1 = self.ram[FLAG_IS_SPRITE_TO_PICK_UP];
        let bak2 = self.ram[SPRITE_PICKUP_SLOT_CACHE];
        let k = self.sprite_spawn_throwable_terrain_silently(what, x, y);
        if k >= 0 {
            self.throwable_scenery_transmute_to_debris(k as usize);
        }
        self.ram[SPRITE_PICKUP_SLOT_CACHE] = bak2;
        self.ram[FLAG_IS_SPRITE_TO_PICK_UP] = bak1;
    }

    // void Sprite_SpawnThrowableTerrain(uint8 what, uint16 x, uint16 y) {  // 86814b
    //   sound_effect_1 = Link_CalculateSfxPan() | 29;
    //   Sprite_SpawnThrowableTerrain_silently(what, x, y);
    // }
    pub(super) fn sprite_spawn_throwable_terrain(&mut self, what: u8, x: u16, y: u16) {
        self.ram[SOUND_EFFECT_1] = self.link_calculate_sfx_pan() | 29;
        self.sprite_spawn_throwable_terrain_silently(what, x, y);
    }

    // int Sprite_SpawnThrowableTerrain_silently(uint8 what, uint16 x, uint16 y) {  // 868156
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_spawn_throwable_terrain_silently(
        &mut self,
        what: u8,
        x: u16,
        y: u16,
    ) -> i32 {
        const K_THROWABLE_SCENERY_FLAGS: [u8; 9] = [0x0c, 0x0c, 0x0c, 0, 0, 0, 0xb0, 0x08, 0xb4];

        let Some(k) = (0..16).rev().find(|&k| self.ram[SPRITE_STATE + k] == 0) else {
            return -1;
        };

        self.ram[SPRITE_STATE + k] = 10;
        self.ram[SPRITE_TYPE + k] = 0xec;
        self.sprite_set_x(k, x);
        self.sprite_set_y(k, y);
        self.sprite_prep_load_properties_for_helpers(k);
        self.ram[SPRITE_FLOOR + k] = self.ram[LINK_IS_ON_LOWER_LEVEL];
        self.ram[SPRITE_C_SPRITE + k] = what;
        if what >= 6 {
            self.ram[SPRITE_FLAGS2 + k] = 0xa6;
        }

        let mut flags = K_THROWABLE_SCENERY_FLAGS[what as usize];
        if what == 2 && self.ram[PLAYER_IS_INDOORS] != 0 {
            self.ram[SPRITE_OAM_FLAGS + k] = 0x80;
            flags = 0x50;
        }
        self.ram[SPRITE_OAM_FLAGS + k] = flags;
        self.ram[SPRITE_UNK4_LOC + k] = 9;
        self.ram[FLAG_IS_SPRITE_TO_PICK_UP] = 2;
        self.ram[SPRITE_PICKUP_SLOT_CACHE] = 2;
        self.ram[SPRITE_DELAY_MAIN + k] = 16;
        self.ram[SPRITE_FLOOR + k] = self.ram[LINK_IS_ON_LOWER_LEVEL];
        self.ram[SPRITE_GRAPHICS + k] = 0;

        if self.ram[DUNG_SECRETS_UNK1_SPRITE] != 255 {
            if (self.ram[DUNG_SECRETS_UNK1_SPRITE] | self.ram[PLAYER_IS_INDOORS]) == 0
                && self.ram[SPRITE_C_SPRITE + k].wrapping_sub(2) < 2
            {
                self.overworld_substitute_alternate_secret();
            }
            if self.ram[DUNG_SECRETS_UNK1_SPRITE] & 0x80 != 0 {
                self.ram[SPRITE_GRAPHICS + k] = self.ram[DUNG_SECRETS_UNK1_SPRITE] & 0x7f;
                self.ram[DUNG_SECRETS_UNK1_SPRITE] = 0;
            }
            self.sprite_spawn_secret(k);
        }

        k as i32
    }

    // void Overworld_SubstituteAlternateSecret() {  // 9afbdb
    //   ...see sprite.c...
    // }
    pub(super) fn overworld_substitute_alternate_secret(&mut self) {
        const SECRET_SUBST_TAB0: [u8; 64] = [
            0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 6, 4, 4, 6, 0, 0, 15, 15, 4, 5,
            5, 4, 6, 6, 15, 15, 4, 5, 5, 7, 6, 6, 31, 31, 4, 7, 7, 4, 6, 6, 6, 7, 2, 0, 0, 0, 0, 0,
            6, 6, 2, 0, 0, 0, 0, 0,
        ];
        const SECRET_SUBST_TAB2: [u8; 16] = [1, 1, 1, 1, 15, 1, 1, 18, 16, 1, 1, 1, 17, 1, 1, 3];
        const SECRET_SUBST_TAB1: [u8; 16] = [0, 0, 0, 0, 2, 0, 0, 8, 16, 0, 0, 0, 1, 0, 0, 0];

        if self.get_random_number() & 1 != 0 {
            return;
        }

        let mut n = 0;
        for j in (0..16).rev() {
            if self.ram[SPRITE_STATE + j] != 0 && self.ram[SPRITE_TYPE + j] != 0x6c {
                n += 1;
            }
        }
        if n >= 4 || self.ram[SRAM_PROGRESS_INDICATOR_SPRITE] < 2 {
            return;
        }

        let j = ((self.ram[OVERWORLD_SECRET_SUBST_CTR_SPRITE] & 7)
            + if self.ram[IS_IN_DARK_WORLD_SPRITE] != 0 {
                8
            } else {
                0
            }) as usize;
        self.ram[OVERWORLD_SECRET_SUBST_CTR_SPRITE] =
            self.ram[OVERWORLD_SECRET_SUBST_CTR_SPRITE].wrapping_add(1);
        let area = (self.ram[OVERWORLD_AREA_INDEX_SPRITE] & 0x3f) as usize;
        if SECRET_SUBST_TAB0[area] & SECRET_SUBST_TAB1[j] == 0 {
            self.ram[DUNG_SECRETS_UNK1_SPRITE] = SECRET_SUBST_TAB2[j];
        }
    }

    fn entity_apply_rumble_to_sprites(&mut self, hb: &mut SpriteHitBox) {
        for j in (0..=15).rev() {
            if self.ram[SPRITE_DEFL_BITS + j] & 2 == 0 || self.ram[SPRITE_E + j] == 0 {
                continue;
            }
            if self.ram[SPRITE_CHR_HALFSLOT_STATE] != 0x0e {
                self.sprite_setup_hit_box(j, hb);
                if !self.check_if_hit_boxes_overlap(hb) {
                    continue;
                }
            }
            self.ram[SPRITE_E + j] = 0;
            self.ram[SOUND_EFFECT_2] = 0x30;
            self.ram[SPRITE_Z_VEL + j] = 0x30;
            self.ram[SPRITE_X_VEL + j] = 0x10;
            self.ram[SPRITE_DELAY_AUX3_SPRITE + j] = 0x30;
            self.ram[SPRITE_STUNNED + j] = 255;
            if self.ram[SPRITE_TYPE + j] == 0xd8 {
                self.sprite_transmute_to_bomb_for_sprite(j);
            }
        }
    }

    fn sprite_transmute_to_bomb_for_sprite(&mut self, k: usize) {
        self.ram[SPRITE_TYPE + k] = 0x4a;
        self.ram[SPRITE_C_SPRITE + k] = 1;
        self.ram[SPRITE_DELAY_AUX1_SPRITE + k] = 255;
        self.ram[SPRITE_FLAGS3 + k] = 0x18;
        self.ram[SPRITE_OAM_FLAGS + k] = 8;
        self.ram[SPRITE_HEALTH_SPRITE + k] = 0;
    }

    pub(super) fn sprite_nullify_hookshot_drag(&mut self) {
        for i in (0..5).rev() {
            if self.ram[ANCILLA_TYPE + i] & 0x1f == 0 && self.ram[RELATED_TO_HOOKSHOT] != 0 {
                self.ram[RELATED_TO_HOOKSHOT] = 0;
                break;
            }
        }
        let player = self.player_state_view();
        let link_x_hi = (player.x() >> 8) as u8;
        let link_y_hi = (player.y() >> 8) as u8;
        self.ram[LINK_X_COORD_SAFE_RETURN_HI] = link_x_hi;
        self.ram[LINK_Y_COORD_SAFE_RETURN_HI] = link_y_hi;
        copy_le_u16(&mut self.ram, LINK_X_COORD, LINK_X_COORD_PREV);
        copy_le_u16(&mut self.ram, LINK_Y_COORD, LINK_Y_COORD_PREV);
        self.handle_indoor_camera_and_doors();
    }

    pub(super) fn sprite_prep_reset_properties(&mut self, k: usize) {
        for base in [
            SPRITE_PAUSE,
            SPRITE_E,
            SPRITE_X_VEL,
            SPRITE_Y_VEL,
            SPRITE_Z_VEL,
            SPRITE_X_SUBPIXEL,
            SPRITE_Y_SUBPIXEL,
            SPRITE_Z_SUBPOS,
            SPRITE_AI_STATE,
            SPRITE_GRAPHICS,
            SPRITE_D,
            SPRITE_DELAY_MAIN,
            SPRITE_DELAY_AUX1_SPRITE,
            SPRITE_DELAY_AUX2_SPRITE,
            SPRITE_DELAY_AUX4,
            SPRITE_HEAD_DIR,
            SPRITE_ANIM_CLOCK_SPRITE,
            SPRITE_G,
            SPRITE_HIT_TIMER,
            SPRITE_WALLCOLL,
            SPRITE_Z,
            SPRITE_HEALTH_SPRITE,
            SPRITE_F,
            SPRITE_X_RECOIL,
            SPRITE_Y_RECOIL,
            SPRITE_A,
            SPRITE_B,
            SPRITE_C_SPRITE,
            SPRITE_UNK2_SPRITE,
            SPRITE_SUBTYPE2,
            SPRITE_IGNORE_PROJECTILE,
            SPRITE_OBJ_PRIO,
            SPRITE_OAM_FLAGS,
            SPRITE_STUNNED,
            SPRITE_GIVE_DAMAGE_SPRITE,
            SPRITE_UNK3_LOC,
            SPRITE_UNK4_LOC,
            SPRITE_UNK5_LOC,
            SPRITE_UNK1_LOC,
            SPRITE_I_LOC,
        ] {
            self.ram[base + k] = 0;
        }
    }

    pub(super) fn sprite_prep_load_properties(&mut self, k: usize) {
        self.sprite_prep_reset_properties(k);
        let ty = self.ram[SPRITE_TYPE + k];

        self.ram[SPRITE_FLAGS2 + k] = sprite_init_value(SPRITE_INIT_FLAGS2_TABLE, ty);
        self.ram[SPRITE_HEALTH_SPRITE + k] = sprite_init_value(SPRITE_INIT_HEALTH_TABLE, ty);
        self.ram[SPRITE_FLAGS4 + k] = sprite_init_value(SPRITE_INIT_FLAGS4_TABLE, ty);
        self.ram[SPRITE_FLAGS5 + k] = sprite_init_value(SPRITE_INIT_FLAGS5_TABLE, ty);
        self.ram[SPRITE_DEFL_BITS + k] = sprite_init_value(SPRITE_INIT_DEFL_BITS_TABLE, ty);
        self.ram[SPRITE_BUMP_DAMAGE + k] = sprite_init_value(SPRITE_INIT_BUMP_DAMAGE_TABLE, ty);
        self.ram[SPRITE_FLAGS_SPRITE + k] = sprite_init_value(SPRITE_INIT_FLAGS_TABLE, ty);
        self.ram[SPRITE_ROOM + k] = if self.ram[PLAYER_IS_INDOORS] != 0 {
            read_le_u16(&self.ram, DUNGEON_ROOM_INDEX2) as u8
        } else {
            read_le_u16(&self.ram, OVERWORLD_AREA_INDEX_SPRITE) as u8
        };

        let flags3 = sprite_init_value(SPRITE_INIT_FLAGS3_TABLE, ty);
        self.ram[SPRITE_FLAGS3 + k] = flags3;
        self.ram[SPRITE_OAM_FLAGS + k] = flags3 & 0x0f;
    }

    pub(super) fn sprite_prep_load_palette(&mut self, k: usize) {
        let flags3 = sprite_init_value(SPRITE_INIT_FLAGS3_TABLE, self.ram[SPRITE_TYPE + k]);
        self.ram[SPRITE_FLAGS3 + k] = flags3;
        self.ram[SPRITE_OAM_FLAGS + k] = flags3 & 0x0f;
    }

    pub(super) fn ancilla_spawn_falling_prize(&mut self, item: u8) -> i32 {
        self.ancilla_add_falling_prize(0x29, item, 4)
    }

    pub(super) fn sprite_set_x(&mut self, k: usize, x: u16) {
        self.sprite_slot_view_mut(k).set_x(x);
    }

    pub(super) fn sprite_set_y(&mut self, k: usize, y: u16) {
        self.sprite_slot_view_mut(k).set_y(y);
    }

    // void SpriteAddXY(int k, int xv, int yv) {
    //   Sprite_SetX(k, Sprite_GetX(k) + xv);
    //   Sprite_SetY(k, Sprite_GetY(k) + yv);
    // }
    pub(super) fn sprite_add_xy(&mut self, k: usize, xv: i32, yv: i32) {
        self.sprite_set_x(k, self.sprite_get_x(k).wrapping_add(xv as i16 as u16));
        self.sprite_set_y(k, self.sprite_get_y(k).wrapping_add(yv as i16 as u16));
    }

    // void SpriteFall_AdjustPosition(int k) {  // 86e624
    //   SpriteAddXY(k, dung_floor_x_vel, dung_floor_y_vel);
    // }
    pub(super) fn sprite_fall_adjust_position(&mut self, k: usize) {
        self.sprite_add_xy(
            k,
            read_le_u16(&self.ram, DUNG_FLOOR_X_VEL) as i16 as i32,
            read_le_u16(&self.ram, DUNG_FLOOR_Y_VEL) as i16 as i32,
        );
    }

    pub(super) fn sprite_get_x(&self, k: usize) -> u16 {
        self.sprite_slot_view(k).x()
    }

    pub(super) fn sprite_get_y(&self, k: usize) -> u16 {
        self.sprite_slot_view(k).y()
    }

    pub(super) fn sprite_is_right_of_location(&self, k: usize, x: u16) -> PairU8 {
        let xv = x.wrapping_sub(self.sprite_get_x(k));
        PairU8 {
            a: u8::from((xv as i16).is_negative()),
            b: xv as u8,
        }
    }

    pub(super) fn sprite_is_below_location(&self, k: usize, y: u16) -> PairU8 {
        let yv = y.wrapping_sub(self.sprite_get_y(k));
        PairU8 {
            a: u8::from((yv as i16).is_negative()),
            b: yv as u8,
        }
    }

    // uint8 Sprite_DirectionToFaceLocation(int k, uint16 x, uint16 y) {  // 86eb30
    //   PairU8 below = Sprite_IsBelowLocation(k, y);
    //   PairU8 right = Sprite_IsRightOfLocation(k, x);
    //   uint8 ym = sign8(below.b) ? -below.b : below.b;
    //   tmp_counter = ym;
    //   uint8 xm = sign8(right.b) ? -right.b : right.b;
    //   return (xm >= ym) ? right.a : below.a + 2;
    // }
    pub(super) fn sprite_direction_to_face_location(&mut self, k: usize, x: u16, y: u16) -> u8 {
        let below = self.sprite_is_below_location(k, y);
        let right = self.sprite_is_right_of_location(k, x);
        let ym = if sign8(below.b) {
            below.b.wrapping_neg()
        } else {
            below.b
        };
        self.ram[TMP_COUNTER_SPRITE] = ym;
        let xm = if sign8(right.b) {
            right.b.wrapping_neg()
        } else {
            right.b
        };
        if xm >= ym {
            right.a
        } else {
            below.a + 2
        }
    }

    pub(super) fn sprite_is_right_of_link(&self, k: usize) -> PairU8 {
        let x = self
            .player_state_view()
            .x()
            .wrapping_sub(self.sprite_get_x(k));
        PairU8 {
            a: u8::from(sign16(x)),
            b: x as u8,
        }
    }

    pub(super) fn sprite_is_below_link(&self, k: usize) -> PairU8 {
        let link_y = self.player_state_view().y();
        let t = (link_y as u8) as i32 + 8;
        let u = (t & 0xff) + self.ram[SPRITE_Z + k] as i32;
        let v = (u & 0xff) - self.ram[SPRITE_Y_LO + k] as i32;
        let w = (link_y >> 8) as i32 - self.ram[SPRITE_Y_HI + k] as i32 - i32::from(v < 0);
        let y = ((w & 0xff) + (t >> 8) + (u >> 8)) as u8;
        PairU8 {
            a: u8::from((y as i8).is_negative()),
            b: v as u8,
        }
    }

    pub(super) fn sprite_project_speed_towards_link(
        &self,
        k: usize,
        mut vel: u8,
    ) -> ProjectSpeedRet {
        if vel == 0 {
            return ProjectSpeedRet {
                x: 0,
                y: 0,
                xdiff: 0,
                ydiff: 0,
            };
        }
        let below = self.sprite_is_below_link(k);
        let mut r12 = if (below.b as i8).is_negative() {
            0u8.wrapping_sub(below.b)
        } else {
            below.b
        };

        let right = self.sprite_is_right_of_link(k);
        let mut r13 = if (right.b as i8).is_negative() {
            0u8.wrapping_sub(right.b)
        } else {
            right.b
        };
        let mut swapped = false;
        if r13 < r12 {
            swapped = true;
            std::mem::swap(&mut r12, &mut r13);
        }
        let mut xvel = vel;
        let mut yvel = 0u8;
        let mut t = 0u8;
        loop {
            t = t.wrapping_add(r12);
            if t >= r13 {
                t = t.wrapping_sub(r13);
                yvel = yvel.wrapping_add(1);
            }
            vel = vel.wrapping_sub(1);
            if vel == 0 {
                break;
            }
        }
        if swapped {
            std::mem::swap(&mut xvel, &mut yvel);
        }
        ProjectSpeedRet {
            x: if right.a != 0 {
                0u8.wrapping_sub(xvel)
            } else {
                xvel
            },
            y: if below.a != 0 {
                0u8.wrapping_sub(yvel)
            } else {
                yvel
            },
            xdiff: right.b,
            ydiff: below.b,
        }
    }

    pub(super) fn sprite_project_speed_towards_location(
        &self,
        k: usize,
        x: u16,
        y: u16,
        mut vel: u8,
    ) -> ProjectSpeedRet {
        if vel == 0 {
            return ProjectSpeedRet {
                x: 0,
                y: 0,
                xdiff: 0,
                ydiff: 0,
            };
        }
        let below = self.sprite_is_below_location(k, y);
        let mut r12 = if (below.b as i8).is_negative() {
            0u8.wrapping_sub(below.b)
        } else {
            below.b
        };
        let right = self.sprite_is_right_of_location(k, x);
        let mut r13 = if (right.b as i8).is_negative() {
            0u8.wrapping_sub(right.b)
        } else {
            right.b
        };
        let mut swapped = false;
        if r13 < r12 {
            swapped = true;
            std::mem::swap(&mut r12, &mut r13);
        }
        let mut xvel = vel;
        let mut yvel = 0u8;
        let mut t = 0u8;
        loop {
            t = t.wrapping_add(r12);
            if t >= r13 {
                t = t.wrapping_sub(r13);
                yvel = yvel.wrapping_add(1);
            }
            vel = vel.wrapping_sub(1);
            if vel == 0 {
                break;
            }
        }
        if swapped {
            std::mem::swap(&mut xvel, &mut yvel);
        }
        ProjectSpeedRet {
            x: if right.a != 0 {
                0u8.wrapping_sub(xvel)
            } else {
                xvel
            },
            y: if below.a != 0 {
                0u8.wrapping_sub(yvel)
            } else {
                yvel
            },
            xdiff: right.b,
            ydiff: below.b,
        }
    }

    // void Sprite_ApproachTargetSpeed(int k, uint8 x, uint8 y) {
    //   if (sprite_x_vel[k] - x)
    //     sprite_x_vel[k] += sign8(sprite_x_vel[k] - x) ? 1 : -1;
    //   if (sprite_y_vel[k] - y)
    //     sprite_y_vel[k] += sign8(sprite_y_vel[k] - y) ? 1 : -1;
    // }
    pub(super) fn sprite_approach_target_speed(&mut self, k: usize, x: u8, y: u8) {
        let x_diff = self.ram[SPRITE_X_VEL + k].wrapping_sub(x);
        if x_diff != 0 {
            self.ram[SPRITE_X_VEL + k] =
                self.ram[SPRITE_X_VEL + k].wrapping_add(if sign8(x_diff) { 1 } else { 0xff });
        }
        let y_diff = self.ram[SPRITE_Y_VEL + k].wrapping_sub(y);
        if y_diff != 0 {
            self.ram[SPRITE_Y_VEL + k] =
                self.ram[SPRITE_Y_VEL + k].wrapping_add(if sign8(y_diff) { 1 } else { 0xff });
        }
    }

    pub(super) fn sprite_setup_hit_box(&self, k: usize, hb: &mut SpriteHitBox) {
        const SPRITE_HITBOX_XLO: [i8; 32] = [
            2, 3, 0, -3, -6, 0, 2, -8, 0, -4, -8, 0, -8, -16, 2, 2, 2, 2, 2, -8, 2, 2, -16, -8,
            -12, 4, -4, -12, 5, -32, -2, 4,
        ];
        const SPRITE_HITBOX_XHI: [i8; 32] = [
            0, 0, 0, -1, -1, 0, 0, -1, 0, -1, -1, 0, -1, -1, 0, 0, 0, 0, 0, -1, 0, 0, -1, -1, -1,
            0, -1, -1, 0, -1, -1, 0,
        ];
        const SPRITE_HITBOX_XSIZE: [u8; 32] = [
            12, 1, 16, 20, 20, 8, 4, 32, 48, 24, 32, 32, 32, 48, 12, 12, 60, 124, 12, 32, 4, 12,
            48, 32, 40, 8, 24, 24, 5, 80, 4, 8,
        ];
        const SPRITE_HITBOX_YLO: [i8; 32] = [
            0, 3, 4, -4, -8, 2, 0, -16, 12, -4, -8, 0, -10, -16, 2, 2, 2, 2, -3, -12, 2, 10, 0,
            -12, 16, 4, -4, -12, 3, -16, -8, 10,
        ];
        const SPRITE_HITBOX_YHI: [i8; 32] = [
            0, 0, 0, -1, -1, 0, 0, -1, 0, -1, -1, 0, -1, -1, 0, 0, 0, 0, -1, -1, 0, 0, 0, -1, 0, 0,
            -1, -1, 0, -1, -1, 0,
        ];
        const SPRITE_HITBOX_YSIZE: [u8; 32] = [
            14, 1, 16, 21, 24, 4, 8, 40, 20, 24, 40, 29, 36, 48, 60, 124, 12, 12, 17, 28, 4, 2, 28,
            20, 10, 4, 24, 16, 5, 48, 8, 12,
        ];

        if (self.ram[SPRITE_Z + k] as i8).is_negative() {
            hb.r10_spr_xhi = 0x80;
            return;
        }
        let i = (self.ram[SPRITE_FLAGS4 + k] & 0x1f) as usize;
        let t = u16::from(self.ram[SPRITE_X_LO + k])
            .wrapping_add(u16::from(SPRITE_HITBOX_XLO[i] as u8));
        hb.r4_spr_xlo = t as u8;
        let t_hi = u16::from(self.ram[SPRITE_X_HI + k])
            .wrapping_add(u16::from(SPRITE_HITBOX_XHI[i] as u8))
            .wrapping_add(t >> 8);
        hb.r10_spr_xhi = t_hi as u8;

        let t = u16::from(self.ram[SPRITE_Y_LO + k])
            .wrapping_add(u16::from(SPRITE_HITBOX_YLO[i] as u8));
        let u = t >> 8;
        let ylo = (t as u8).wrapping_sub(self.ram[SPRITE_Z + k]);
        hb.r5_spr_ylo = ylo;
        let t_hi = u16::from(self.ram[SPRITE_Y_HI + k])
            .wrapping_sub(u16::from((t as u8) < self.ram[SPRITE_Z + k]));
        hb.r11_spr_yhi = t_hi
            .wrapping_add(u)
            .wrapping_add(u16::from(SPRITE_HITBOX_YHI[i] as u8)) as u8;

        hb.r6_spr_xsize = SPRITE_HITBOX_XSIZE[i];
        hb.r7_spr_ysize = SPRITE_HITBOX_YSIZE[i];
    }

    // bool Sprite_SetupHitBox00(int k) {  // 86f1f6
    //   return (uint16)(link_x_coord - cur_sprite_x + 11) < 23 &&
    //          (uint16)(link_y_coord - cur_sprite_y + sprite_z[k] + 16) < 24;
    // }
    pub(super) fn sprite_setup_hit_box00(&self, k: usize) -> bool {
        let player = self.player_state_view();
        player
            .x()
            .wrapping_sub(read_le_u16(&self.ram, CUR_SPRITE_X))
            .wrapping_add(11)
            < 23
            && player
                .y()
                .wrapping_sub(read_le_u16(&self.ram, CUR_SPRITE_Y))
                .wrapping_add(self.ram[SPRITE_Z + k] as u16)
                .wrapping_add(16)
                < 24
    }

    // void Sprite_PlaceWeaponTink(int k) {  // 86f6ca
    //   if (repulsespark_timer)
    //     return;
    //   SpriteSfx_QueueSfx2WithPan(k, 5);
    //   Sprite_PlaceRupulseSpark_2(k);
    // }
    pub(super) fn sprite_place_weapon_tink(&mut self, k: usize) {
        if self.ram[REPULSESPARK_TIMER_SPRITE] != 0 {
            return;
        }
        self.sprite_sfx_queue_sfx2_with_pan(k, 5);
        self.sprite_place_rupulse_spark_2(k);
    }

    // void Sprite_PlaceRupulseSpark_2(int k) {  // 86f6d5
    //   uint16 x = Sprite_GetX(k) - BG2HOFS_copy2;
    //   uint16 y = Sprite_GetY(k) - BG2VOFS_copy2;
    //   if (x & ~0xff || y & ~0xff)
    //     return;
    //   repulsespark_x_lo = sprite_x_lo[k];
    //   repulsespark_y_lo = sprite_y_lo[k];
    //   repulsespark_timer = 5;
    //   repulsespark_floor_status = sprite_floor[k];
    // }
    pub(super) fn sprite_place_rupulse_spark_2(&mut self, k: usize) {
        let x = self
            .sprite_get_x(k)
            .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2));
        let y = self
            .sprite_get_y(k)
            .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2));
        if x & !0xff != 0 || y & !0xff != 0 {
            return;
        }
        self.ram[REPULSESPARK_X_LO_SPRITE] = self.ram[SPRITE_X_LO + k];
        self.ram[REPULSESPARK_Y_LO_SPRITE] = self.ram[SPRITE_Y_LO + k];
        self.ram[REPULSESPARK_TIMER_SPRITE] = 5;
        self.ram[REPULSESPARK_FLOOR_STATUS_SPRITE] = self.ram[SPRITE_FLOOR + k];
    }

    // void Link_PlaceWeaponTink() {  // 86f69f
    //   if (repulsespark_timer)
    //     return;
    //   repulsespark_timer = 5;
    //   int t = (uint8)link_x_coord + player_oam_x_offset;
    //   repulsespark_x_lo = t;
    //   t = (uint8)link_y_coord + player_oam_y_offset + (t >> 8);  // carry wtf
    //   repulsespark_y_lo = t;
    //   repulsespark_floor_status = link_is_on_lower_level;
    //   sound_effect_1 = Link_CalculateSfxPan() | 5;
    // }
    pub(super) fn link_place_weapon_tink(&mut self) {
        if self.ram[REPULSESPARK_TIMER_SPRITE] != 0 {
            return;
        }
        self.ram[REPULSESPARK_TIMER_SPRITE] = 5;
        let player = self.player_state_view();
        let t = u16::from(player.x() as u8) + self.ram[PLAYER_OAM_X_OFFSET] as u16;
        let y = u16::from(player.y() as u8) + self.ram[PLAYER_OAM_Y_OFFSET] as u16 + (t >> 8);
        self.ram[REPULSESPARK_X_LO_SPRITE] = t as u8;
        self.ram[REPULSESPARK_Y_LO_SPRITE] = y as u8;
        self.ram[REPULSESPARK_FLOOR_STATUS_SPRITE] = self.ram[LINK_IS_ON_LOWER_LEVEL];
        self.ram[SOUND_EFFECT_1] = self.link_calculate_sfx_pan() | 5;
    }

    // void Sprite_ApplyRecoilToLink(int k, uint8 vel) {  // 86f688
    //   ProjectSpeedRet pt = Sprite_ProjectSpeedTowardsLink(k, vel);
    //   link_actual_vel_x = pt.x;
    //   link_actual_vel_y = pt.y;
    //   g_ram[0xc7] = link_actual_vel_z = vel >> 1;
    //   link_z_coord = 0;
    // }
    pub(super) fn sprite_apply_recoil_to_link(&mut self, k: usize, vel: u8) {
        let pt = self.sprite_project_speed_towards_link(k, vel);
        self.ram[LINK_ACTUAL_VEL_X] = pt.x;
        self.ram[LINK_ACTUAL_VEL_Y] = pt.y;
        self.ram[LINK_ACTUAL_VEL_Z] = vel >> 1;
        self.ram[LINK_RECOIL_Z_VEL] = vel >> 1;
        self.player_state_view_mut().set_z(0);
    }

    fn player_action_hit_box_from_table(&self, hb: &mut SpriteHitBox, t: usize, shrink: bool) {
        const X: [i8; 65] = [
            0, 2, 0, 0, -8, 0, 2, 0, 2, 2, 1, 1, 0, 0, 0, 0, 0, 2, 4, 4, 0, 0, -4, -4, -6, 2, 1, 1,
            0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 4, 4, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, -4, -4, -10, 0,
            2, 2, 0, 0, 0, 0, 0, 0, 0,
        ];
        const W: [u8; 65] = [
            15, 4, 8, 8, 8, 8, 12, 8, 4, 4, 6, 6, 0, 0, 0, 0, 0, 4, 16, 12, 8, 8, 12, 11, 12, 4, 6,
            6, 0, 0, 0, 0, 0, 8, 8, 8, 10, 14, 15, 4, 4, 4, 6, 6, 0, 0, 0, 0, 0, 8, 8, 8, 10, 14,
            15, 4, 4, 4, 6, 6, 0, 0, 0, 0, 0,
        ];
        const Y: [i8; 65] = [
            0, 2, 0, 2, 4, 4, 4, 7, 2, 2, 1, 1, 0, 0, 0, 0, 0, 2, 0, 2, -4, -3, -8, 0, 0, 2, 1, 1,
            0, 0, 0, 0, 0, 0, 0, 0, -2, 0, -4, 1, 2, 2, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, -2, 0, -4, 1,
            2, 2, 1, 1, 0, 0, 0, 0, 0,
        ];
        const H: [u8; 65] = [
            15, 4, 8, 2, 12, 8, 12, 8, 4, 4, 6, 6, 0, 0, 0, 0, 0, 4, 8, 4, 12, 12, 12, 4, 8, 4, 6,
            4, 0, 0, 0, 0, 0, 8, 8, 8, 8, 8, 12, 4, 4, 4, 6, 6, 0, 0, 0, 0, 0, 8, 8, 8, 8, 8, 12,
            4, 4, 4, 6, 6, 0, 0, 0, 0, 0,
        ];

        let player = self.player_state_view();
        let mut x = player
            .x()
            .wrapping_add(X[t].wrapping_add(self.ram[PLAYER_OAM_X_OFFSET] as i8) as i16 as u16);
        let mut y = player
            .y()
            .wrapping_add(Y[t].wrapping_add(self.ram[PLAYER_OAM_Y_OFFSET] as i8) as i16 as u16);
        let mut w = W[t];
        let mut h = H[t];
        if shrink {
            if w >= 2 {
                let r = w.wrapping_sub(2).min(6);
                w = w.wrapping_sub(r);
                x = x.wrapping_add(u16::from(r >> 1));
            }
            if h >= 2 {
                let r = h.wrapping_sub(2).min(6);
                h = h.wrapping_sub(r);
                y = y.wrapping_add(u16::from(r >> 1));
            }
        }
        hb.r0_xlo = x as u8;
        hb.r8_xhi = (x >> 8) as u8;
        hb.r1_ylo = y as u8;
        hb.r9_yhi = (y >> 8) as u8;
        hb.r2 = w;
        hb.r3 = h;
    }

    // void Player_SetupActionHitBox(SpriteHitBox *hb) {  // 86f5e0
    pub(super) fn player_setup_action_hit_box(&self, hb: &mut SpriteHitBox) {
        const RUN_Y_HI: [u8; 4] = [0xff, 0, 0, 0];
        const RUN_Y_LO: [u8; 4] = [0xf8, 16, 8, 8];
        const RUN_X_HI: [u8; 4] = [0, 0, 0xff, 0];
        const RUN_X_LO: [u8; 4] = [0, 0, 0xf8, 8];
        const TAB4: [u8; 13] = [1, 1, 1, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1];

        if self.ram[LINK_IS_RUNNING] != 0 {
            let j = usize::from(self.ram[LINK_DIRECTION_FACING] >> 1);
            let player = self.player_state_view();
            let x = player
                .x()
                .wrapping_add(u16::from(RUN_X_LO[j]) | (u16::from(RUN_X_HI[j]) << 8));
            let y = player
                .y()
                .wrapping_add(u16::from(RUN_Y_LO[j]) | (u16::from(RUN_Y_HI[j]) << 8));
            hb.r0_xlo = x as u8;
            hb.r8_xhi = (x >> 8) as u8;
            hb.r1_ylo = y as u8;
            hb.r9_yhi = (y >> 8) as u8;
            hb.r2 = 16;
            hb.r3 = 16;
            return;
        }

        let mut t = 0usize;
        if (self.ram[LINK_ITEM_IN_HAND] & 10) == 0 && (self.ram[LINK_POSITION_MODE] & 0x10) == 0 {
            if sign8(self.ram[BUTTON_B_FRAMES]) {
                let player = self.player_state_view();
                let x = player.x().wrapping_sub(14);
                let y = player.y().wrapping_sub(10);
                hb.r0_xlo = x as u8;
                hb.r8_xhi = (x >> 8) as u8;
                hb.r1_ylo = y as u8;
                hb.r9_yhi = (y >> 8) as u8;
                hb.r2 = 44;
                hb.r3 = 45;
                return;
            } else if TAB4[usize::from(self.ram[BUTTON_B_FRAMES])] != 0 {
                hb.r8_xhi = 0x80;
                return;
            }
            t = usize::from(self.ram[LINK_DIRECTION_FACING]) * 8
                + usize::from(self.ram[BUTTON_B_FRAMES])
                + 1;
        }
        self.player_action_hit_box_from_table(hb, t, false);
    }

    // void Link_UpdateHitBoxWithSword(SpriteHitBox *hb) {  // new
    pub(super) fn link_update_hit_box_with_sword(&self, hb: &mut SpriteHitBox) {
        const TAB4: [u8; 13] = [1, 1, 1, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1];

        if self.ram[LINK_SPIN_ATTACK_STEP_COUNTER] != 0
            || sign8(self.ram[BUTTON_B_FRAMES])
            || TAB4[usize::from(self.ram[BUTTON_B_FRAMES])] != 0
        {
            return;
        }
        let t = usize::from(self.ram[LINK_DIRECTION_FACING]) * 8
            + usize::from(self.ram[BUTTON_B_FRAMES])
            + 1;
        self.player_action_hit_box_from_table(hb, t, true);
    }

    // void Sprite_DoHitBoxesFast(int k, SpriteHitBox *hb) {  // 86f645
    //   if (HIBYTE(dungmap_var8) == 0x80) {
    //     hb->r10_spr_xhi = 0x80;
    //     return;
    //   }
    //   int t;
    //   t = Sprite_GetX(k) + (int8)HIBYTE(dungmap_var8);
    //   hb->r4_spr_xlo = t;
    //   hb->r10_spr_xhi = t >> 8;
    //   t = Sprite_GetY(k) + (int8)BYTE(dungmap_var8);
    //   hb->r5_spr_ylo = t;
    //   hb->r11_spr_yhi = t >> 8;
    //   hb->r6_spr_xsize = hb->r7_spr_ysize = (sprite_type[k] == 0x6a) ? 16 : 3;
    // }
    pub(super) fn sprite_do_hit_boxes_fast(&self, k: usize, hb: &mut SpriteHitBox) {
        if self.ram[DUNGMAP_VAR8 + 1] == 0x80 {
            hb.r10_spr_xhi = 0x80;
            return;
        }
        let x = self
            .sprite_get_x(k)
            .wrapping_add(self.ram[DUNGMAP_VAR8 + 1] as i8 as i16 as u16);
        hb.r4_spr_xlo = x as u8;
        hb.r10_spr_xhi = (x >> 8) as u8;
        let y = self
            .sprite_get_y(k)
            .wrapping_add(self.ram[DUNGMAP_VAR8] as i8 as i16 as u16);
        hb.r5_spr_ylo = y as u8;
        hb.r11_spr_yhi = (y >> 8) as u8;
        let size = if self.ram[SPRITE_TYPE + k] == 0x6a {
            16
        } else {
            3
        };
        hb.r6_spr_xsize = size;
        hb.r7_spr_ysize = size;
    }

    // void Sprite_CorrectOamEntries(int k, int n, uint8 islarge) {  // 86febc
    //   OamEnt *oam = GetOamCurPtr();
    //   uint8 *extp = &g_ram[oam_ext_cur_ptr];
    //   uint16 spr_x = Sprite_GetX(k);
    //   uint16 spr_y = Sprite_GetY(k);
    //   uint8 scrollx = spr_x - BG2HOFS_copy2;
    //   uint8 scrolly = spr_y - BG2VOFS_copy2;
    //   do {
    //     uint16 x = spr_x + (int8)(oam->x - scrollx);
    //     uint16 y = spr_y + (int8)(oam->y - scrolly);
    //     uint8 ext = sign8(islarge) ? (*extp & 2) : islarge;
    //     *extp = ext + ((uint16)(x - BG2HOFS_copy2) >= 0x100);
    //     if ((uint16)(y + 0x10 - BG2VOFS_copy2) >= 0x100)
    //       oam->y = 0xf0;
    //   } while (oam++, extp++, --n >= 0);
    // }
    pub(super) fn sprite_correct_oam_entries(&mut self, k: usize, n: i32, islarge: u8) {
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let mut extp = read_le_u16(&self.ram, OAM_EXT_CUR_PTR) as usize;
        let spr_x = self.sprite_get_x(k);
        let spr_y = self.sprite_get_y(k);
        let scrollx = spr_x.wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2)) as u8;
        let scrolly = spr_y.wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2)) as u8;
        for _ in 0..=n {
            let x = spr_x.wrapping_add(self.ram[oam].wrapping_sub(scrollx) as i8 as i16 as u16);
            let y = spr_y.wrapping_add(self.ram[oam + 1].wrapping_sub(scrolly) as i8 as i16 as u16);
            let ext = if sign8(islarge) {
                self.ram[extp] & 2
            } else {
                islarge
            };
            self.ram[extp] =
                ext + u8::from(x.wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2)) >= 0x100);
            if y.wrapping_add(0x10)
                .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2))
                >= 0x100
            {
                self.ram[oam + 1] = 0xf0;
            }
            oam += 4;
            extp += 1;
        }
    }

    // void Link_SetupHitBox_conditional(SpriteHitBox *hb) {  // 86f705
    //   if (link_disable_sprite_damage)
    //     hb->r9_yhi = 0x80;
    //   else
    //     Link_SetupHitBox(hb);
    // }
    pub(super) fn link_setup_hit_box_conditional(&self, hb: &mut SpriteHitBox) {
        if self.ram[LINK_DISABLE_SPRITE_DAMAGE] != 0 {
            hb.r9_yhi = 0x80;
        } else {
            self.link_setup_hit_box(hb);
        }
    }

    // void Link_SetupHitBox(SpriteHitBox *hb) {  // 86f70a
    //   hb->r3 = hb->r2 = 8;
    //   uint16 x = link_x_coord + 4;
    //   hb->r0_xlo = x;
    //   hb->r8_xhi = x >> 8;
    //   uint16 y = link_y_coord + 8;
    //   hb->r1_ylo = y;
    //   hb->r9_yhi = y >> 8;
    // }
    pub(super) fn link_setup_hit_box(&self, hb: &mut SpriteHitBox) {
        hb.r2 = 8;
        hb.r3 = 8;
        let player = self.player_state_view();
        let x = player.x().wrapping_add(4);
        hb.r0_xlo = x as u8;
        hb.r8_xhi = (x >> 8) as u8;
        let y = player.y().wrapping_add(8);
        hb.r1_ylo = y as u8;
        hb.r9_yhi = (y >> 8) as u8;
    }

    pub(super) fn check_if_hit_boxes_overlap(&self, hb: &SpriteHitBox) -> bool {
        if hb.r8_xhi == 0x80 || hb.r10_spr_xhi == 0x80 {
            return false;
        }

        let mut t = i32::from(hb.r5_spr_ylo) - i32::from(hb.r1_ylo);
        let r15 = (t + i32::from(hb.r7_spr_ysize)) as u8;
        let r12 = hb
            .r11_spr_yhi
            .wrapping_sub(hb.r9_yhi)
            .wrapping_sub(u8::from(t < 0));
        t = i32::from(r12) + (((t & 0xff) + 0x80) >> 8);
        if (t as u8) != 0 {
            return t >= 0x100;
        }
        if hb.r3.wrapping_add(hb.r7_spr_ysize) < r15 {
            return false;
        }

        t = i32::from(hb.r4_spr_xlo) - i32::from(hb.r0_xlo);
        let r15 = (t + i32::from(hb.r6_spr_xsize)) as u8;
        let r12 = hb
            .r10_spr_xhi
            .wrapping_sub(hb.r8_xhi)
            .wrapping_sub(u8::from(t < 0));
        t = i32::from(r12) + (((t & 0xff) + 0x80) >> 8);
        if (t as u8) != 0 {
            return t >= 0x100;
        }
        if hb.r2.wrapping_add(hb.r6_spr_xsize) < r15 {
            return false;
        }

        true
    }

    pub(super) fn sprite_prep_oam_coord_or_double_ret(
        &mut self,
        k: usize,
    ) -> Option<(u16, u16, u8)> {
        let (ret, out) = self.sprite_prep_oam_coord_or_double_ret_raw(k);
        if out {
            None
        } else {
            Some((ret.x, ret.y, ret.flags))
        }
    }

    // void Sprite_PrepOamCoord(int k, PrepOamCoordsRet *ret) {  // 86e416
    //   Sprite_PrepOamCoordOrDoubleRet(k, ret);
    // }
    pub(super) fn sprite_prep_oam_coord(&mut self, k: usize, ret: &mut PrepOamCoordsRet) {
        let (prepped, _) = self.sprite_prep_oam_coord_or_double_ret_raw(k);
        *ret = prepped;
    }

    // bool Sprite_PrepOamCoordOrDoubleRet(int k, PrepOamCoordsRet *ret) {  // 86e41e
    //   sprite_pause[k] = 0;
    //   uint16 x = cur_sprite_x - BG2HOFS_copy2;
    //   uint16 y = cur_sprite_y - BG2VOFS_copy2;
    //   bool out_of_bounds = false;
    //   R0 = x;
    //   R2 = y - sprite_z[k];
    //   ret->flags = sprite_oam_flags[k] ^ sprite_obj_prio[k];
    //   ret->r4 = 0;
    //   if ((uint16)(x + 0x40 + xt) >= (0x170 + xt * 2) ||
    //       (uint16)(y + 0x40) >= 0x170 && !(sprite_flags4[k] & 0x20)) {
    //     sprite_pause[k]++;
    //     if (!(sprite_defl_bits[k] & 0x80))
    //       Sprite_KillSelf(k);
    //     out_of_bounds = true;
    //   }
    //   ret->x = R0;
    //   ret->y = R2;
    //   BYTE(dungmap_var7) = ret->x;
    //   HIBYTE(dungmap_var7) = ret->y;
    //   return out_of_bounds;
    // }
    fn sprite_prep_oam_coord_or_double_ret_raw(&mut self, k: usize) -> (PrepOamCoordsRet, bool) {
        self.ram[SPRITE_PAUSE + k] = 0;
        let cur_x = read_le_u16(&self.ram, CUR_SPRITE_X);
        let cur_y = read_le_u16(&self.ram, CUR_SPRITE_Y);
        let x = cur_x.wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2));
        let y = cur_y.wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2));
        let r2 = y.wrapping_sub(self.ram[SPRITE_Z + k] as u16);
        write_le_u16(&mut self.ram, R0, x);
        write_le_u16(&mut self.ram, R2, r2);
        let flags = self.ram[SPRITE_OAM_FLAGS + k] ^ self.ram[SPRITE_OBJ_PRIO + k];
        let xt = if self.read_u32_ram(ENHANCED_FEATURES0) & FEATURES0_EXTEND_SCREEN64_SPRITE != 0 {
            0x40
        } else {
            0
        };
        let out = x.wrapping_add(0x40 + xt) >= 0x170 + xt * 2
            || (y.wrapping_add(0x40) >= 0x170 && self.ram[SPRITE_FLAGS4 + k] & 0x20 == 0);
        if out {
            self.ram[SPRITE_PAUSE + k] = self.ram[SPRITE_PAUSE + k].wrapping_add(1);
            if (self.ram[SPRITE_DEFL_BITS + k] & 0x80) == 0 {
                self.sprite_kill_self(k);
            }
        }
        let ret_x = read_le_u16(&self.ram, R0);
        let ret_y = read_le_u16(&self.ram, R2);
        let ret = PrepOamCoordsRet {
            x: ret_x,
            y: ret_y,
            r4: 0,
            flags,
        };
        write_le_u16(
            &mut self.ram,
            DUNGMAP_VAR7,
            ret_x as u8 as u16 | ((ret_y as u8 as u16) << 8),
        );
        (ret, out)
    }

    // void Sprite_InitializeSlots() {  // 89afd6
    //   for (int k = 15; k >= 0; k--) {
    //     uint8 st = sprite_state[k], ty = sprite_type[k];
    //     if (st != 0) {
    //       if (st == 10) {
    //         if (ty != 0xec && ty != 0xd2) {
    //           link_picking_throw_state = 0;
    //           link_state_bits = 0;
    //           sprite_state[k] = 0;
    //         }
    //       } else {
    //         if (ty != 0x6c && sprite_room[k] != BYTE(overworld_area_index))
    //           sprite_state[k] = 0;
    //       }
    //     }
    //   }
    //   for (int k = 7; k >= 0; k--) {
    //     if (overlord_type[k] && overlord_spawned_in_area[k] != BYTE(overworld_area_index))
    //       overlord_type[k] = 0;
    //   }
    // }
    pub(super) fn sprite_initialize_slots(&mut self) {
        let area = self.ram[OVERWORLD_AREA_INDEX_SPRITE];
        for k in (0..=15usize).rev() {
            let st = self.ram[SPRITE_STATE + k];
            let ty = self.ram[SPRITE_TYPE + k];
            if st == 0 {
                continue;
            }
            if st == 10 {
                if ty != 0xec && ty != 0xd2 {
                    self.ram[LINK_PICKING_THROW_STATE] = 0;
                    self.ram[LINK_STATE_BITS] = 0;
                    self.ram[SPRITE_STATE + k] = 0;
                }
            } else if ty != 0x6c && self.ram[SPRITE_ROOM + k] != area {
                self.ram[SPRITE_STATE + k] = 0;
            }
        }
        for k in (0..=7usize).rev() {
            if self.ram[OVERLORD_TYPE + k] != 0
                && self.ram[OVERLORD_SPAWNED_IN_AREA_SPRITE + k] != area
            {
                self.ram[OVERLORD_TYPE + k] = 0;
            }
        }
    }

    // void Sprite_InitializeMirrorPortal() {  // 89af89
    //   for (int k = 15; k >= 0; k--) {
    //     if (sprite_state[k] && sprite_type[k] == 0x6c)
    //       sprite_state[k] = 0;
    //   }
    //   SpriteSpawnInfo info;
    //   int j = Sprite_SpawnDynamically(0xff, 0x6c, &info);
    //   if (j < 0)
    //     j = 0;
    //   Sprite_SetX(j, bird_travel_x_hi[15] << 8 | bird_travel_x_lo[15]);
    //   Sprite_SetY(j, (bird_travel_y_hi[15] << 8 | bird_travel_y_lo[15]) + 8);
    //   sprite_floor[j] = 0;
    //   sprite_ignore_projectile[j] = 1;
    // }
    pub(super) fn sprite_initialize_mirror_portal(&mut self) {
        for k in (0..=15usize).rev() {
            if self.ram[SPRITE_STATE + k] != 0 && self.ram[SPRITE_TYPE + k] == 0x6c {
                self.ram[SPRITE_STATE + k] = 0;
            }
        }

        let mut info = SpriteSpawnInfo::default();
        let mut j = self.sprite_spawn_dynamically(0xff, 0x6c, &mut info);
        if j < 0 {
            j = 0;
        }
        let ju = j as usize;
        let x = (u16::from(self.ram[BIRD_TRAVEL_X_HI + 15]) << 8)
            | u16::from(self.ram[BIRD_TRAVEL_X_LO + 15]);
        let y = ((u16::from(self.ram[BIRD_TRAVEL_Y_HI + 15]) << 8)
            | u16::from(self.ram[BIRD_TRAVEL_Y_LO + 15]))
        .wrapping_add(8);
        self.sprite_set_x(ju, x);
        self.sprite_set_y(ju, y);
        self.ram[SPRITE_FLOOR + ju] = 0;
        self.ram[SPRITE_IGNORE_PROJECTILE + ju] = 1;
    }

    // void Sprite_ResetAll() {  // 89c44e
    //   Sprite_DisableAll();
    //   Sprite_ResetAll_noDisable();
    // }
    pub(super) fn sprite_reset_all(&mut self) {
        self.sprite_disable_all();
        self.sprite_reset_all_no_disable();
    }

    // void Sprite_DisableAll() {  // 89c22f
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_disable_all(&mut self) {
        for k in (0..16).rev() {
            if self.ram[SPRITE_STATE + k] != 0
                && (self.ram[PLAYER_IS_INDOORS] != 0 || self.ram[SPRITE_TYPE + k] != 0x6c)
            {
                self.ram[SPRITE_STATE + k] = 0;
            }
        }
        for k in (0..10).rev() {
            self.ram[ANCILLA_TYPE + k] = 0;
        }

        self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] = 0;
        self.ram[SPRITE_LIMIT_INSTANCE] = 0;
        self.ram[ITEM_DROP_COUNTER] = 0;
        self.ram[ARCHERY_GAME_HIT_COUNTER] = 0;
        self.ram[ARCHERY_GAME_ARROWS_LEFT] = 0;
        self.ram[GARNISH_ACTIVE_SPRITE] = 0;
        self.ram[DUNGEON_TRAP_TRIGGER_LATCH] = 0;
        self.ram[ACTIVATE_BOMB_TRAP_OVERLORD_SPRITE] = 0;
        self.ram[INTRO_TIMES_PAL_FLASH] = 0;
        self.ram[SPRITE_RESET_SCRATCH_A] = 0;
        self.ram[SPRITE_RESET_SCRATCH_B] = 0;
        self.ram[FLAG_BLOCK_LINK_MENU] = 0;
        self.ram[OVERWORLD_BOULDER_TRAP_COUNT] = 0;
        self.ram[SPRITE_CHR_HALFSLOT_STATE] = 0;
        self.ram[IS_ARCHER_OR_SHOVEL_GAME] = 0;
        for k in (0..8).rev() {
            self.ram[OVERLORD_TYPE + k] = 0;
        }
        for k in (0..30).rev() {
            self.ram[GARNISH_TYPE + k] = 0;
        }
    }

    // void Sprite_ResetAll_noDisable() {  // 89c452
    //   haunted_grove_flute_event_latch = 0;
    //   sprite_alert_flag = 0;
    //   overworld_boulder_trap_count = 0;
    //   MESSAGE_OR_SPRITE_STATE_CACHE = 0;
    //   sprite_chr_halfslot_state = 0;
    //   sprite_limit_instance = 0;
    //   sort_sprites_setting = 0;
    //   if (follower_indicator != 13)
    //     super_bomb_indicator_unk2 = 0xfe;
    //   memset(sprite_where_in_room, 0, 0x1000);
    //   memset(overworld_sprite_was_loaded, 0, 0x200);
    //   memset(dungeon_room_history, 0xff, 8);
    // }
    pub(super) fn sprite_reset_all_no_disable(&mut self) {
        self.ram[HAUNTED_GROVE_FLUTE_EVENT_LATCH] = 0;
        self.ram[SPRITE_ALERT_FLAG] = 0;
        self.ram[OVERWORLD_BOULDER_TRAP_COUNT] = 0;
        self.ram[MESSAGE_OR_SPRITE_STATE_CACHE] = 0;
        self.ram[SPRITE_CHR_HALFSLOT_STATE] = 0;
        self.ram[SPRITE_LIMIT_INSTANCE] = 0;
        self.ram[SORT_SPRITES_SETTING] = 0;
        if self.ram[FOLLOWER_INDICATOR] != 13 {
            self.ram[SUPER_BOMB_INDICATOR_TIMER] = 0xfe;
        }
        self.ram[SPRITE_WHERE_IN_ROOM..SPRITE_WHERE_IN_ROOM + 0x1000].fill(0);
        self.ram[OVERWORLD_SPRITE_WAS_LOADED..OVERWORLD_SPRITE_WAS_LOADED + 0x200].fill(0);
        for i in 0..4 {
            write_le_u16(&mut self.ram, DUNGEON_ROOM_HISTORY + i * 2, 0xffff);
        }
    }

    pub(super) fn sprite_reload_all_overworld(&mut self) {
        self.sprite_disable_all();
        self.sprite_overworld_reload_all_just_load();
    }

    pub(super) fn sprite_overworld_reload_all_just_load(&mut self) {
        self.sprite_reset_all_no_disable();
        self.overworld_load_sprites();
        self.sprite_activate_all_proxima();
    }

    pub(super) fn overworld_load_sprites(&mut self) {
        let area = read_le_u16(&self.ram, OVERWORLD_AREA_INDEX_SPRITE);
        let area_lo = self.ram[OVERWORLD_AREA_INDEX_SPRITE] as usize;
        write_le_u16(&mut self.ram, SPRCOLL_X_BASE_SPRITE, (area & 7) << 9);
        write_le_u16(
            &mut self.ram,
            SPRCOLL_Y_BASE_SPRITE,
            (((area & 0x3f) >> 2) & 0x0e) << 8,
        );
        let size = u16::from(OVERWORLD_AREA_SPRCOLL_SIZES[area_lo]) << 8;
        write_le_u16(&mut self.ram, SPRCOLL_X_SIZE, size);
        write_le_u16(&mut self.ram, SPRCOLL_Y_SIZE, size);

        let base = match self.ram[SRAM_PROGRESS_INDICATOR_SPRITE] {
            3 => 2,
            2 => 1,
            _ => 0,
        };
        let Some(offsets) = self.asset_raw(159).map(Vec::from) else {
            return;
        };
        let Some(sprites) = self.asset_raw(160).map(Vec::from) else {
            return;
        };
        let offs_idx = (area as usize + base * 144) * 2;
        if offs_idx + 1 >= offsets.len() {
            return;
        }
        let mut src = read_word_from_slice(&offsets, offs_idx) as usize;
        while src < sprites.len() && sprites[src] != 0xff {
            if src + 2 >= sprites.len() {
                break;
            }
            if sprites[src + 2] == 0xf4 {
                self.ram[OVERWORLD_BOULDER_TRAP_COUNT] =
                    self.ram[OVERWORLD_BOULDER_TRAP_COUNT].wrapping_add(1);
                src += 3;
                continue;
            }

            let r2 = (sprites[src] >> 4) << 2;
            let r6 = (sprites[src + 1] >> 4).wrapping_add(r2);
            let r5 = (sprites[src + 1] & 0x0f) | (sprites[src] << 4);
            let idx = usize::from(r5) | (usize::from(r6) << 8);
            self.ram[SPRITE_WHERE_IN_OVERWORLD + idx] = sprites[src + 2].wrapping_add(1);
            src += 3;
        }
    }

    pub(super) fn sprite_activate_all_proxima(&mut self) {
        let bak0 = read_le_u16(&self.ram, BG2HOFS_COPY2);
        let bak1 = self.ram[OVERWORLD_SCROLL_DELTA + 1];
        self.ram[OVERWORLD_SCROLL_DELTA + 1] = 0xff;

        let xt: u16 =
            if self.read_u32_ram(ENHANCED_FEATURES0) & FEATURES0_EXTEND_SCREEN64_SPRITE != 0 {
                0x40
            } else {
                0
            };
        write_le_u16(&mut self.ram, BG2HOFS_COPY2, bak0.wrapping_sub(xt));
        for _ in (0..=(21 + (xt >> 3))).rev() {
            self.sprite_activate_when_proximal();
            let bg = read_le_u16(&self.ram, BG2HOFS_COPY2).wrapping_add(16);
            write_le_u16(&mut self.ram, BG2HOFS_COPY2, bg);
        }
        self.ram[OVERWORLD_SCROLL_DELTA + 1] = bak1;
        write_le_u16(&mut self.ram, BG2HOFS_COPY2, bak0);
    }

    pub(super) fn sprite_proximity_activation(&mut self) {
        if self.frame_control_view().submodule() != 0 {
            self.sprite_activate_when_proximal();
            self.sprite_activate_when_proximal_big();
        } else {
            if self.ram[SPR_RANGED_BASED_TOGGLER] & 1 == 0 {
                self.sprite_activate_when_proximal();
            }
            if self.ram[SPR_RANGED_BASED_TOGGLER] & 1 != 0 {
                self.sprite_activate_when_proximal_big();
            }
            self.ram[SPR_RANGED_BASED_TOGGLER] = self.ram[SPR_RANGED_BASED_TOGGLER].wrapping_add(1);
        }
    }

    pub(super) fn sprite_activate_when_proximal(&mut self) {
        if self.ram[OVERWORLD_SCROLL_DELTA + 1] == 0 {
            return;
        }
        let xt: u16 =
            if self.read_u32_ram(ENHANCED_FEATURES0) & FEATURES0_EXTEND_SCREEN64_SPRITE != 0 {
                0x40
            } else {
                0
            };
        let x = read_le_u16(&self.ram, BG2HOFS_COPY2).wrapping_add(
            if sign8(self.ram[OVERWORLD_SCROLL_DELTA + 1]) {
                0u16.wrapping_sub(0x10).wrapping_sub(xt)
            } else {
                0x110u16.wrapping_add(xt)
            },
        );
        let mut y = read_le_u16(&self.ram, BG2VOFS_COPY2).wrapping_sub(0x30);
        for _ in (0..=21).rev() {
            self.sprite_overworld_proximity_motivated_load(x, y);
            y = y.wrapping_add(16);
        }
    }

    pub(super) fn sprite_activate_when_proximal_big(&mut self) {
        if self.ram[OVERWORLD_SCROLL_DELTA] == 0 {
            return;
        }
        let xt: u16 =
            if self.read_u32_ram(ENHANCED_FEATURES0) & FEATURES0_EXTEND_SCREEN64_SPRITE != 0 {
                0x40
            } else {
                0
            };
        let mut x = read_le_u16(&self.ram, BG2HOFS_COPY2)
            .wrapping_sub(0x30)
            .wrapping_sub(xt);
        let y = read_le_u16(&self.ram, BG2VOFS_COPY2).wrapping_add(
            if sign8(self.ram[OVERWORLD_SCROLL_DELTA]) {
                0u16.wrapping_sub(0x10)
            } else {
                0x110
            },
        );
        for _ in (0..=(21 + (xt >> 3))).rev() {
            self.sprite_overworld_proximity_motivated_load(x, y);
            x = x.wrapping_add(16);
        }
    }

    pub(super) fn sprite_overworld_proximity_motivated_load(&mut self, x: u16, y: u16) {
        let sprcoll_x_base = read_le_u16(&self.ram, SPRCOLL_X_BASE_SPRITE);
        let sprcoll_y_base = read_le_u16(&self.ram, SPRCOLL_Y_BASE_SPRITE);
        let xt = x.wrapping_sub(sprcoll_x_base);
        let yt = y.wrapping_sub(sprcoll_y_base);
        if xt >= read_le_u16(&self.ram, SPRCOLL_X_SIZE)
            || yt >= read_le_u16(&self.ram, SPRCOLL_Y_SIZE)
        {
            return;
        }

        let r1 = (((yt >> 8) * 4) | (xt >> 8)) as u8;
        let r0 = ((y & 0x00f0) | ((x >> 4) & 0x000f)) as u8;
        self.overworld_load_proxima_sprite_if_alive((u16::from(r1) << 8) | u16::from(r0));
    }

    pub(super) fn overworld_load_proxima_sprite_if_alive(&mut self, blk: u16) {
        let sprite_to_spawn = self.ram[SPRITE_WHERE_IN_OVERWORLD + blk as usize];
        if sprite_to_spawn == 0 {
            return;
        }

        let loadedmask = 0x80u8 >> (blk & 7);
        let loaded_idx = OVERWORLD_SPRITE_WAS_LOADED + ((blk >> 3) as usize);
        if self.ram[loaded_idx] & loadedmask != 0 {
            return;
        }

        if sprite_to_spawn >= 0xf4 {
            let k = self.alloc_overlord();
            if k < 0 {
                return;
            }
            let k = k as usize;
            self.ram[loaded_idx] |= loadedmask;
            write_le_u16(
                &mut self.ram,
                OVERLORD_OFFSET_SPRITE_POS_SPRITE + k * 2,
                blk,
            );
            self.ram[OVERLORD_TYPE + k] = sprite_to_spawn.wrapping_sub(0xf3);
            self.ram[SPRITE_OVERLORD_X_LO + k] = ((blk << 4) & 0x00f0) as u8
                + if self.ram[OVERLORD_TYPE + k] == 1 {
                    8
                } else {
                    0
                };
            self.ram[SPRITE_OVERLORD_Y_LO + k] = (blk & 0x00f0) as u8;
            self.ram[SPRITE_OVERLORD_X_HI + k] = (((blk >> 8) & 3) as u8)
                .wrapping_add((read_le_u16(&self.ram, SPRCOLL_X_BASE_SPRITE) >> 8) as u8);
            self.ram[SPRITE_OVERLORD_Y_HI + k] = ((blk >> 10) as u8)
                .wrapping_add((read_le_u16(&self.ram, SPRCOLL_Y_BASE_SPRITE) >> 8) as u8);
            self.ram[OVERLORD_FLOOR_SPRITE + k] = 0;
            self.ram[OVERLORD_SPAWNED_IN_AREA_SPRITE + k] = self.ram[OVERWORLD_AREA_INDEX_SPRITE];
            self.ram[OVERLORD_GEN2 + k] = 0;
            self.ram[OVERLORD_GEN1 + k] = 0;
            self.ram[OVERLORD_GEN3_SPRITE + k] = 0;
        } else {
            let k = self.overworld_alloc_sprite(sprite_to_spawn);
            if k < 0 {
                return;
            }
            let k = k as usize;
            if std::env::var_os("ZELDA3_REPLAY_SPRITE_LOAD_DUMP").is_some() {
                println!(
                    "ow-load frame={} blk=0x{:04x} raw=0x{:02x} type=0x{:02x} slot={} old_t=0x{:02x} old_st=0x{:02x} old_c=0x{:02x} old_bump=0x{:02x}",
                    self.ram[FRAME_COUNTER],
                    blk,
                    sprite_to_spawn,
                    sprite_to_spawn.wrapping_sub(1),
                    k,
                    self.ram[SPRITE_TYPE + k],
                    self.ram[SPRITE_STATE + k],
                    self.ram[SPRITE_C_SPRITE + k],
                    self.ram[SPRITE_BUMP_DAMAGE + k],
                );
            }
            self.ram[loaded_idx] |= loadedmask;
            write_le_u16(&mut self.ram, SPRITE_N_WORD + k * 2, blk);
            self.ram[SPRITE_TYPE + k] = sprite_to_spawn.wrapping_sub(1);
            self.ram[SPRITE_STATE + k] = 8;
            self.ram[SPRITE_X_LO + k] = ((blk << 4) & 0x00f0) as u8;
            self.ram[SPRITE_Y_LO + k] = (blk & 0x00f0) as u8;
            self.ram[SPRITE_X_HI + k] = (((blk >> 8) & 3) as u8)
                .wrapping_add((read_le_u16(&self.ram, SPRCOLL_X_BASE_SPRITE) >> 8) as u8);
            self.ram[SPRITE_Y_HI + k] = ((blk >> 10) as u8)
                .wrapping_add((read_le_u16(&self.ram, SPRCOLL_Y_BASE_SPRITE) >> 8) as u8);
            self.ram[SPRITE_FLOOR + k] = 0;
            self.ram[SPRITE_SUBTYPE + k] = 0;
            self.ram[SPRITE_DIE_ACTION + k] = 0;
        }
    }

    pub(super) fn dungeon_reset_sprites(&mut self) {
        if self.ram[PLAYER_IS_INDOORS] != 0 {
            self.dungeon_cache_trans_sprites();
        }
        self.ram[LINK_PICKING_THROW_STATE] = 0;
        self.ram[LINK_STATE_BITS] = 0;
        self.sprite_disable_all();
        write_le_u16(&mut self.ram, SPRCOLL_X_SIZE, 0xffff);
        write_le_u16(&mut self.ram, SPRCOLL_Y_SIZE, 0xffff);
        let room = read_le_u16(&self.ram, DUNGEON_ROOM_INDEX2);
        let seen = (0..4).any(|i| read_le_u16(&self.ram, DUNGEON_ROOM_HISTORY + i * 2) == room);
        if !seen {
            let dropped = read_le_u16(&self.ram, DUNGEON_ROOM_HISTORY + 6);
            for i in (1..4).rev() {
                let prev = read_le_u16(&self.ram, DUNGEON_ROOM_HISTORY + (i - 1) * 2);
                write_le_u16(&mut self.ram, DUNGEON_ROOM_HISTORY + i * 2, prev);
            }
            write_le_u16(&mut self.ram, DUNGEON_ROOM_HISTORY, room);
            if dropped != 0xffff {
                self.set_sprite_where_in_room_mask(dropped, 0);
            }
        }
        self.dungeon_load_sprites();
    }

    pub(super) fn dungeon_load_sprites(&mut self) {
        let Some(sprites) = self.asset_raw(58).map(Vec::from) else {
            return;
        };
        let Some(offsets) = self.asset_raw(59).map(Vec::from) else {
            return;
        };
        let room = read_le_u16(&self.ram, DUNGEON_ROOM_INDEX2) as usize;
        let start = read_word_from_slice(&offsets, room * 2) as usize;
        if start >= sprites.len() {
            return;
        }

        self.ram[SPRITE_ROOM_ORIGIN_Y_HI] = ((room >> 3) & 0xfe) as u8;
        self.ram[SPRITE_ROOM_ORIGIN_X_HI] = ((room & 0x0f) << 1) as u8;
        self.ram[SORT_SPRITES_SETTING] = sprites[start];

        let mut k = 0isize;
        let mut src = start + 1;
        while src < sprites.len() && sprites[src] != 0xff {
            if src + 2 >= sprites.len() {
                break;
            }
            k = self.dungeon_load_single_sprite(
                k as usize,
                sprites[src],
                sprites[src + 1],
                sprites[src + 2],
            );
            k += 1;
            src += 3;
        }
    }

    pub(super) fn dungeon_load_single_sprite(
        &mut self,
        k: usize,
        y: u8,
        x: u8,
        sprite_type: u8,
    ) -> isize {
        if sprite_type == 0xe4 {
            if y == 0xfe || y == 0xfd {
                if k != 0 {
                    self.ram[SPRITE_DIE_ACTION + k - 1] = if y == 0xfe { 1 } else { 2 };
                }
                return k as isize - 1;
            }
        } else if x >= 0xe0 {
            self.dungeon_load_single_overlord(&[y, x, sprite_type]);
            return k as isize - 1;
        }

        if sprite_init_value(SPRITE_INIT_DEFL_BITS_TABLE, sprite_type) & 1 == 0
            && self.sprite_where_in_room_mask(read_le_u16(&self.ram, DUNGEON_ROOM_INDEX2))
                & (1 << k)
                != 0
        {
            return k as isize;
        }

        self.ram[SPRITE_STATE + k] = 8;
        self.ram[TMP_COUNTER_SPRITE] = y;
        self.ram[SPRITE_FLOOR + k] = y >> 7;

        let y_coord =
            (((y as u16) << 4) & 0x01ff) + ((self.ram[SPRITE_ROOM_ORIGIN_Y_HI] as u16) << 8);
        self.ram[SPRITE_Y_LO + k] = y_coord as u8;
        self.ram[SPRITE_Y_HI + k] = (y_coord >> 8) as u8;

        self.ram[SPRITE_SHARED_SCRATCH_A] = x;
        let x_coord =
            (((x as u16) << 4) & 0x01ff) + ((self.ram[SPRITE_ROOM_ORIGIN_X_HI] as u16) << 8);
        self.ram[SPRITE_X_LO + k] = x_coord as u8;
        self.ram[SPRITE_X_HI + k] = (x_coord >> 8) as u8;

        self.ram[SPRITE_TYPE + k] = sprite_type;
        self.ram[TMP_COUNTER_SPRITE] = (self.ram[TMP_COUNTER_SPRITE] & 0x60) >> 2;
        self.ram[SPRITE_SUBTYPE + k] = self.ram[TMP_COUNTER_SPRITE] | (x >> 5);
        self.ram[SPRITE_N + k] = k as u8;
        self.ram[SPRITE_DIE_ACTION + k] = 0;
        k as isize
    }

    // void Dungeon_LoadSingleOverlord(const uint8 *src) {  // 89c3e8
    //   int k = AllocOverlord();
    //   if (k < 0)
    //     return;
    //   uint8 y = src[0], x = src[1], type = src[2];
    //   overlord_type[k] = type;
    //   overlord_floor[k] = (y >> 7);
    //   int t = ((y << 4) & 0x1ff) + (SPRITE_ROOM_ORIGIN_Y_HI << 8);
    //   overlord_y_lo[k] = t;
    //   overlord_y_hi[k] = t >> 8;
    //   t = ((x << 4) & 0x1ff) + (SPRITE_ROOM_ORIGIN_X_HI << 8);
    //   overlord_x_lo[k] = t;
    //   overlord_x_hi[k] = t >> 8;
    //   overlord_spawned_in_area[k] = overworld_area_index;
    //   overlord_gen2[k] = 0;
    //   overlord_gen1[k] = 0;
    //   overlord_gen3[k] = 0;
    //   if (overlord_type[k] == 10 || overlord_type[k] == 11) {
    //     overlord_gen2[k] = 160;
    //   } else if (overlord_type[k] == 3) {
    //     overlord_gen2[k] = 255;
    //     overlord_x_lo[k] -= 8;
    //   }
    // }
    pub(super) fn dungeon_load_single_overlord(&mut self, src: &[u8]) {
        let k = self.alloc_overlord();
        if k < 0 || src.len() < 3 {
            return;
        }
        let k = k as usize;
        let y = src[0];
        let x = src[1];
        let type_ = src[2];
        self.ram[OVERLORD_TYPE + k] = type_;
        self.ram[OVERLORD_FLOOR_SPRITE + k] = y >> 7;
        let mut t =
            (((y as u16) << 4) & 0x01ff) + ((self.ram[SPRITE_ROOM_ORIGIN_Y_HI] as u16) << 8);
        self.ram[SPRITE_OVERLORD_Y_LO + k] = t as u8;
        self.ram[SPRITE_OVERLORD_Y_HI + k] = (t >> 8) as u8;
        t = (((x as u16) << 4) & 0x01ff) + ((self.ram[SPRITE_ROOM_ORIGIN_X_HI] as u16) << 8);
        self.ram[SPRITE_OVERLORD_X_LO + k] = t as u8;
        self.ram[SPRITE_OVERLORD_X_HI + k] = (t >> 8) as u8;
        self.ram[OVERLORD_SPAWNED_IN_AREA_SPRITE + k] = self.ram[OVERWORLD_AREA_INDEX_SPRITE];
        self.ram[OVERLORD_GEN2 + k] = 0;
        self.ram[OVERLORD_GEN1 + k] = 0;
        self.ram[OVERLORD_GEN3_SPRITE + k] = 0;
        if self.ram[OVERLORD_TYPE + k] == 10 || self.ram[OVERLORD_TYPE + k] == 11 {
            self.ram[OVERLORD_GEN2 + k] = 160;
        } else if self.ram[OVERLORD_TYPE + k] == 3 {
            self.ram[OVERLORD_GEN2 + k] = 255;
            self.ram[SPRITE_OVERLORD_X_LO + k] = self.ram[SPRITE_OVERLORD_X_LO + k].wrapping_sub(8);
        }
    }

    pub(super) fn sprite_main(&mut self) {
        if self.ram[PLAYER_IS_INDOORS] == 0 {
            self.ram[ANCILLA_FLOOR..ANCILLA_FLOOR + 5].fill(0);
            self.sprite_proximity_activation();
        }
        self.ram[IS_IN_DARK_WORLD_SPRITE] = u8::from(self.ram[SAVEGAME_IS_DARKWORLD] != 0);
        if self.frame_control_view().submodule() == 0 {
            write_le_u16(&mut self.ram, DRAG_PLAYER_X, 0);
            write_le_u16(&mut self.ram, DRAG_PLAYER_Y, 0);
        }
        self.oam_reset_region_bases();
        self.replay_trace_ram_watch("sprite-after-oam-reset");
        self.garnish_execute_upper_slots();
        self.replay_trace_ram_watch("sprite-after-garnish-upper");
        self.follower_main();
        self.replay_trace_ram_watch("sprite-after-follower");
        self.ram[SPRITE_PICKUP_SLOT_CACHE] = self.ram[FLAG_IS_SPRITE_TO_PICK_UP];
        self.ram[FLAG_IS_SPRITE_TO_PICK_UP] = 0;
        self.ram[DUNGMAP_VAR8 + 1] = 0x80;
        if self.ram[SET_WHEN_DAMAGING_ENEMIES] & 0x7f != 0 {
            self.ram[SET_WHEN_DAMAGING_ENEMIES] =
                self.ram[SET_WHEN_DAMAGING_ENEMIES].wrapping_sub(1);
        } else {
            self.ram[SET_WHEN_DAMAGING_ENEMIES] = 0;
        }
        self.ram[PLAYER_POSE_DRAW_COUNTER] = 0;
        self.ram[LINK_PULL_ACTION_STATE] = 0;
        self.ram[LINK_PREVENT_FROM_MOVING] = 0;
        if self.ram[SPRITE_ALERT_FLAG] != 0 {
            self.ram[SPRITE_ALERT_FLAG] = self.ram[SPRITE_ALERT_FLAG].wrapping_sub(1);
        }
        self.ancilla_main();
        self.replay_trace_ram_watch("sprite-after-ancilla");
        self.overlord_main();
        self.replay_trace_ram_watch("sprite-after-overlord");
        self.ram[ARCHERY_GAME_OUT_OF_ARROWS] = 0;
        let trace_sprite_slots = std::env::var_os("ZELDA3_REPLAY_RAM_WATCH_FRAME").is_some();

        for k in (0..16).rev() {
            self.ram[CUR_OBJECT_INDEX] = k as u8;
            if trace_sprite_slots {
                self.replay_trace_ram_watch(&format!("sprite-before-execute-single slot={k}"));
            }
            self.sprite_execute_single(k);
            if trace_sprite_slots {
                self.replay_trace_ram_watch(&format!("sprite-after-execute-single slot={k}"));
            }
        }
        self.garnish_execute_lower_slots();
        self.ram[OVERWORLD_SCROLL_DELTA] = 0;
        self.ram[OVERWORLD_SCROLL_DELTA + 1] = 0;
        self.execute_cached_sprites();
        if self.ram[LOAD_CHR_HALFSLOT_EVEN_ODD] != 0 {
            self.ram[SPRITE_CHR_HALFSLOT_STATE] = self.ram[LOAD_CHR_HALFSLOT_EVEN_ODD];
        }
    }

    // void Sprite_ExecuteSingle(int k) {  // 8684e2
    //   uint8 st = sprite_state[k];
    //   if (st != 0)
    //     Sprite_TimersAndOam(k);
    //   kSprite_ExecuteSingle[st](k);
    // }
    pub(super) fn sprite_execute_single(&mut self, k: usize) {
        let st = self.ram[SPRITE_STATE + k];
        if st != 0 {
            self.sprite_timers_and_oam(k);
        }
        match st {
            0 => self.sprite_inactive_sprite(k),
            1 => self.sprite_module_fall1(k),
            2 => self.sprite_module_poof(k),
            3 => self.sprite_module_drown(k),
            4 => self.sprite_module_explode(k),
            5 => self.sprite_module_fall2(k),
            6 => self.sprite_module_die(k),
            7 => self.sprite_module_burn(k),
            8 => self.sprite_module_initialize(k),
            9 => self.sprite_active_main(k),
            10 => self.sprite_module_carried(k),
            11 => self.sprite_module_stunned(k),
            _ => self.sprite_active_main(k),
        }
    }

    // void ExecuteCachedSprites() {  // 9de9da
    //   ...see sprite.c...
    // }
    pub(super) fn execute_cached_sprites(&mut self) {
        if self.ram[PLAYER_IS_INDOORS] == 0
            || self.frame_control_view().submodule() == 0
            || self.frame_control_view().submodule() == 14
            || self.ram[ALT_SPRITES_FLAG] == 0
        {
            self.ram[ALT_SPRITES_FLAG] = 0;
            return;
        }
        for i in (0..16usize).rev() {
            self.ram[CUR_OBJECT_INDEX] = i as u8;
            if self.ram[ALT_SPRITE_STATE_SPRITE + i] != 0 {
                self.uncache_and_execute_sprite(i);
            }
        }
    }

    // void UncacheAndExecuteSprite(int k) {  // 9dea00
    //   ...see sprite.c...
    // }
    pub(super) fn uncache_and_execute_sprite(&mut self, k: usize) {
        const LIVE: [usize; 24] = [
            SPRITE_STATE,
            SPRITE_TYPE,
            SPRITE_X_LO,
            SPRITE_X_HI,
            SPRITE_Y_LO,
            SPRITE_Y_HI,
            SPRITE_GRAPHICS,
            SPRITE_A,
            SPRITE_HEAD_DIR,
            SPRITE_OAM_FLAGS,
            SPRITE_OBJ_PRIO,
            SPRITE_D,
            SPRITE_FLAGS2,
            SPRITE_FLOOR,
            SPRITE_AI_STATE_SPRITE,
            SPRITE_FLAGS3,
            SPRITE_B,
            SPRITE_C_SPRITE,
            SPRITE_E,
            SPRITE_SUBTYPE2,
            SPRITE_Z,
            SPRITE_DELAY_MAIN,
            SPRITE_I_LOC,
            SPRITE_IGNORE_PROJECTILE,
        ];
        const ALT: [usize; 24] = [
            ALT_SPRITE_STATE_SPRITE,
            ALT_SPRITE_TYPE_SPRITE,
            ALT_SPRITE_X_LO_SPRITE,
            ALT_SPRITE_X_HI_SPRITE,
            ALT_SPRITE_Y_LO_SPRITE,
            ALT_SPRITE_Y_HI_SPRITE,
            ALT_SPRITE_GRAPHICS_SPRITE,
            ALT_SPRITE_A_SPRITE,
            ALT_SPRITE_HEAD_DIR_SPRITE,
            ALT_SPRITE_OAM_FLAGS_SPRITE,
            ALT_SPRITE_OBJ_PRIO_SPRITE,
            ALT_SPRITE_D_SPRITE,
            ALT_SPRITE_FLAGS2_SPRITE,
            ALT_SPRITE_FLOOR_SPRITE,
            ALT_SPRITE_SPAWNED_FLAG_SPRITE,
            ALT_SPRITE_FLAGS3_SPRITE,
            ALT_SPRITE_B_SPRITE,
            ALT_SPRITE_C_SPRITE,
            ALT_SPRITE_E_SPRITE,
            ALT_SPRITE_SUBTYPE2_SPRITE,
            ALT_SPRITE_HEIGHT_ABOVE_SHADOW_SPRITE,
            ALT_SPRITE_DELAY_MAIN_SPRITE,
            ALT_SPRITE_I_SPRITE,
            ALT_SPRITE_IGNORE_PROJECTILE_SPRITE,
        ];

        let mut bak = [0u8; 24];
        for i in 0..LIVE.len() {
            bak[i] = self.ram[LIVE[i] + k];
            self.ram[LIVE[i] + k] = self.ram[ALT[i] + k];
        }
        self.sprite_execute_single(k);
        if self.ram[SPRITE_PAUSE + k] != 0 {
            self.ram[ALT_SPRITE_STATE_SPRITE + k] = 0;
        }
        for i in (0..LIVE.len()).rev() {
            self.ram[LIVE[i] + k] = bak[i];
        }
    }

    // void Dungeon_CacheTransSprites() {  // 89c176
    //   ...see sprite.c...
    // }
    pub(super) fn dungeon_cache_trans_sprites(&mut self) {
        if self.ram[PLAYER_IS_INDOORS] == 0 {
            return;
        }
        self.ram[ALT_SPRITES_FLAG] = self.ram[PLAYER_IS_INDOORS];
        const LIVE: [usize; 24] = [
            SPRITE_STATE,
            SPRITE_TYPE,
            SPRITE_X_LO,
            SPRITE_X_HI,
            SPRITE_Y_LO,
            SPRITE_Y_HI,
            SPRITE_GRAPHICS,
            SPRITE_A,
            SPRITE_HEAD_DIR,
            SPRITE_OAM_FLAGS,
            SPRITE_OBJ_PRIO,
            SPRITE_D,
            SPRITE_FLAGS2,
            SPRITE_FLOOR,
            SPRITE_AI_STATE_SPRITE,
            SPRITE_FLAGS3,
            SPRITE_B,
            SPRITE_C_SPRITE,
            SPRITE_E,
            SPRITE_SUBTYPE2,
            SPRITE_Z,
            SPRITE_DELAY_MAIN,
            SPRITE_I_LOC,
            SPRITE_IGNORE_PROJECTILE,
        ];
        const ALT: [usize; 24] = [
            ALT_SPRITE_STATE_SPRITE,
            ALT_SPRITE_TYPE_SPRITE,
            ALT_SPRITE_X_LO_SPRITE,
            ALT_SPRITE_X_HI_SPRITE,
            ALT_SPRITE_Y_LO_SPRITE,
            ALT_SPRITE_Y_HI_SPRITE,
            ALT_SPRITE_GRAPHICS_SPRITE,
            ALT_SPRITE_A_SPRITE,
            ALT_SPRITE_HEAD_DIR_SPRITE,
            ALT_SPRITE_OAM_FLAGS_SPRITE,
            ALT_SPRITE_OBJ_PRIO_SPRITE,
            ALT_SPRITE_D_SPRITE,
            ALT_SPRITE_FLAGS2_SPRITE,
            ALT_SPRITE_FLOOR_SPRITE,
            ALT_SPRITE_SPAWNED_FLAG_SPRITE,
            ALT_SPRITE_FLAGS3_SPRITE,
            ALT_SPRITE_B_SPRITE,
            ALT_SPRITE_C_SPRITE,
            ALT_SPRITE_E_SPRITE,
            ALT_SPRITE_SUBTYPE2_SPRITE,
            ALT_SPRITE_HEIGHT_ABOVE_SHADOW_SPRITE,
            ALT_SPRITE_DELAY_MAIN_SPRITE,
            ALT_SPRITE_I_SPRITE,
            ALT_SPRITE_IGNORE_PROJECTILE_SPRITE,
        ];
        for k in (0..16usize).rev() {
            self.ram[ALT_SPRITE_STATE_SPRITE + k] = 0;
            self.ram[ALT_SPRITE_TYPE_SPRITE + k] = self.ram[SPRITE_TYPE + k];
            self.ram[ALT_SPRITE_X_LO_SPRITE + k] = self.ram[SPRITE_X_LO + k];
            self.ram[ALT_SPRITE_GRAPHICS_SPRITE + k] = self.ram[SPRITE_GRAPHICS + k];
            self.ram[ALT_SPRITE_X_HI_SPRITE + k] = self.ram[SPRITE_X_HI + k];
            self.ram[ALT_SPRITE_Y_LO_SPRITE + k] = self.ram[SPRITE_Y_LO + k];
            self.ram[ALT_SPRITE_Y_HI_SPRITE + k] = self.ram[SPRITE_Y_HI + k];
            if self.ram[SPRITE_PAUSE + k] != 0
                || self.ram[SPRITE_STATE + k] == 4
                || self.ram[SPRITE_STATE + k] == 10
            {
                continue;
            }
            for i in 0..LIVE.len() {
                self.ram[ALT[i] + k] = self.ram[LIVE[i] + k];
            }
        }
    }

    pub(super) fn oam_allocate_from_region_a(&mut self, num: u8) -> u16 {
        self.oam_get_buffer_position(num, 0)
    }

    pub(super) fn oam_allocate_from_region_b(&mut self, num: u8) -> u16 {
        self.oam_get_buffer_position(num, 2)
    }

    pub(super) fn oam_allocate_from_region_c(&mut self, num: u8) -> u16 {
        self.oam_get_buffer_position(num, 4)
    }

    pub(super) fn oam_allocate_from_region_d(&mut self, num: u8) -> u16 {
        self.oam_get_buffer_position(num, 6)
    }

    pub(super) fn oam_allocate_from_region_e(&mut self, num: u8) -> u16 {
        self.oam_get_buffer_position(num, 8)
    }

    pub(super) fn oam_allocate_from_region_f(&mut self, num: u8) -> u16 {
        self.oam_get_buffer_position(num, 10)
    }

    // void Sprite_TimersAndOam(int k) {  // 8683f2
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_timers_and_oam(&mut self, k: usize) {
        let x = self.sprite_get_x(k);
        let y = self.sprite_get_y(k);
        write_le_u16(&mut self.ram, CUR_SPRITE_X, x);
        write_le_u16(&mut self.ram, CUR_SPRITE_Y, y);

        let num = ((self.ram[SPRITE_FLAGS2 + k] & 0x1f).wrapping_add(1)).wrapping_mul(4);
        if self.ram[SORT_SPRITES_SETTING] != 0 {
            if self.ram[SPRITE_FLOOR + k] != 0 {
                self.oam_allocate_from_region_f(num);
            } else {
                self.oam_allocate_from_region_d(num);
            }
        } else {
            self.oam_allocate_from_region_a(num);
        }

        if (self.frame_control_view().submodule() | self.ram[FLAG_UNK1]) == 0 {
            for base in [
                SPRITE_DELAY_MAIN,
                SPRITE_DELAY_AUX1_SPRITE,
                SPRITE_DELAY_AUX2_SPRITE,
                SPRITE_DELAY_AUX3_SPRITE,
            ] {
                if self.ram[base + k] != 0 {
                    self.ram[base + k] = self.ram[base + k].wrapping_sub(1);
                }
            }

            let timer = self.ram[SPRITE_HIT_TIMER + k] & 0x7f;
            if timer != 0 {
                if self.ram[SPRITE_STATE + k] >= 9 {
                    if timer == 31 {
                        self.sprite_hit_timer31(k);
                    } else if timer == 24 {
                        self.sprite_mini_moldorm_recoil(k);
                    }
                }
                if self.ram[SPRITE_GIVE_DAMAGE_SPRITE + k] < 251 {
                    self.ram[SPRITE_OBJ_PRIO + k] =
                        ((u16::from(self.ram[SPRITE_HIT_TIMER + k]) * 2) & 0x0e) as u8;
                }
                self.ram[SPRITE_HIT_TIMER + k] = self.ram[SPRITE_HIT_TIMER + k].wrapping_sub(1);
            } else {
                self.ram[SPRITE_HIT_TIMER + k] = 0;
                self.ram[SPRITE_OBJ_PRIO + k] = 0;
            }

            if self.ram[SPRITE_DELAY_AUX4 + k] != 0 {
                self.ram[SPRITE_DELAY_AUX4 + k] = self.ram[SPRITE_DELAY_AUX4 + k].wrapping_sub(1);
            }
        }

        const SPRITE_PRIOS: [u8; 4] = [0x20, 0x10, 0x30, 0x30];
        let mut floor = self.ram[LINK_IS_ON_LOWER_LEVEL] as usize;
        if floor != 3 {
            floor = self.ram[SPRITE_FLOOR + k] as usize;
        }
        self.ram[SPRITE_OBJ_PRIO + k] =
            (self.ram[SPRITE_OBJ_PRIO + k] & 0xcf) | SPRITE_PRIOS[floor];
    }

    pub(super) fn oam_get_buffer_position(&mut self, num: u8, y: u8) -> u16 {
        const LIMITS: [u16; 6] = [0x0171, 0x0201, 0x0031, 0x00c1, 0x0141, 0x01d1];
        const FALLBACKS: [u16; 48] = [
            0x0030, 0x0050, 0x0080, 0x00b0, 0x00e0, 0x0110, 0x0140, 0x0170, 0x01d0, 0x01d4, 0x01dc,
            0x01e0, 0x01e4, 0x01ec, 0x01f0, 0x01f8, 0x0000, 0x0004, 0x0008, 0x000c, 0x0010, 0x0014,
            0x0018, 0x001c, 0x0030, 0x0038, 0x0050, 0x0068, 0x0080, 0x0098, 0x00b0, 0x00c8, 0x0120,
            0x0124, 0x0128, 0x012c, 0x0130, 0x0134, 0x0138, 0x013c, 0x0140, 0x0150, 0x0160, 0x0170,
            0x0180, 0x0190, 0x01a0, 0x01b8,
        ];

        let region = (y >> 1) as usize;
        let base_addr = 0x0fe0 + region * 2;
        let mut pstart = read_le_u16(&self.ram, base_addr);
        let p = pstart.wrapping_add(num as u16);
        if p >= LIMITS[region] {
            let alloc_addr = OAM_ALLOC_ARR1 + region * 2;
            let alloc = read_le_u16(&self.ram, alloc_addr);
            let j = alloc & 7;
            write_le_u16(&mut self.ram, alloc_addr, alloc.wrapping_add(1));
            pstart = FALLBACKS[region * 8 + j as usize];
        } else {
            write_le_u16(&mut self.ram, base_addr, p);
        }
        let oam = 0x0800 + pstart;
        write_le_u16(&mut self.ram, OAM_EXT_CUR_PTR, 0x0a20 + (pstart >> 2));
        write_le_u16(&mut self.ram, OAM_CUR_PTR, oam);
        oam
    }

    pub(super) fn sprite_move_xy(&mut self, k: usize) {
        self.sprite_move_x(k);
        self.sprite_move_y(k);
    }

    // void Sprite_MoveXYZ(int k) {
    //   Sprite_MoveZ(k);
    //   Sprite_MoveX(k);
    //   Sprite_MoveY(k);
    // }
    pub(super) fn sprite_move_xyz(&mut self, k: usize) {
        self.sprite_move_z(k);
        self.sprite_move_x(k);
        self.sprite_move_y(k);
    }

    pub(super) fn sprite_move_x(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).move_x();
    }

    pub(super) fn sprite_move_y(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).move_y();
    }

    pub(super) fn sprite_draw_shadow(&mut self, k: usize, x: u16) {
        if self.ram[SPRITE_PAUSE + k] != 0
            || (self.ram[SPRITE_STATE + k] == 10 && self.ram[SPRITE_UNK3_LOC + k] == 3)
        {
            return;
        }
        let y = self
            .sprite_y(k)
            .wrapping_add(10)
            .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2));
        if y.wrapping_add(0x10) >= 0x100 {
            return;
        }
        let oam = (read_le_u16(&self.ram, OAM_CUR_PTR) as usize)
            + usize::from(self.ram[SPRITE_FLAGS2 + k] & 0x1f) * 4;
        let flags = (self.ram[SPRITE_OAM_FLAGS + k] ^ self.ram[SPRITE_OBJ_PRIO + k]) & 0x30;
        if self.ram[SPRITE_FLAGS3 + k] & 0x20 != 0 {
            self.set_oam_helper1_at(oam, x, y.wrapping_add(1) as u8, 0x38, flags | 8, 0);
        } else {
            self.set_oam_helper1_at(oam, x, y as u8, 0x6c, flags | 8, 2);
        }
    }

    // ----- Common sprite-AI helpers (round-3 agent) -----
    // Direct 1:1 ports of the small Sprite_* helpers that are widely shared by
    // sprite-AI handlers. Bodies preserved verbatim modulo Rust-isms.

    // static int AllocOverlord() {
    //   int i = 7;
    //   while (i >= 0 && overlord_type[i] != 0)
    //     i--;
    //   return i;
    // }
    pub(super) fn alloc_overlord(&self) -> i32 {
        let mut i = 7i32;
        while i >= 0 && self.ram[OVERLORD_TYPE + i as usize] != 0 {
            i -= 1;
        }
        i
    }

    // static int Overworld_AllocSprite(uint8 type) {
    //   int i = (type == 0x58) ? 4 :
    //           (type == 0xd0) ? 5 :
    //           (type == 0xeb || type == 0x53 || type  == 0xf3) ? 14 : 13;
    //   for (; i >= 0; i--) {
    //     if (sprite_state[i] == 0 || sprite_type[i] == 0x41 && sprite_C[i] != 0)
    //       break;
    //   }
    //   return i;
    // }
    pub(super) fn overworld_alloc_sprite(&self, type_: u8) -> i32 {
        let mut i = if type_ == 0x58 {
            4
        } else if type_ == 0xd0 {
            5
        } else if type_ == 0xeb || type_ == 0x53 || type_ == 0xf3 {
            14
        } else {
            13
        };
        while i >= 0 {
            let k = i as usize;
            if self.ram[SPRITE_STATE + k] == 0
                || (self.ram[SPRITE_TYPE + k] == 0x41 && self.ram[SPRITE_C_SPRITE + k] != 0)
            {
                break;
            }
            i -= 1;
        }
        i
    }

    // uint16 Garnish_GetX(int k) {
    //   return garnish_x_lo[k] | garnish_x_hi[k] << 8;
    // }
    pub(super) fn garnish_get_x(&self, k: usize) -> u16 {
        u16::from(self.ram[GARNISH_X_LO_SPRITE + k])
            | (u16::from(self.ram[GARNISH_X_HI_SPRITE + k]) << 8)
    }

    // uint16 Garnish_GetY(int k) {
    //   return garnish_y_lo[k] | garnish_y_hi[k] << 8;
    // }
    pub(super) fn garnish_get_y(&self, k: usize) -> u16 {
        u16::from(self.ram[GARNISH_Y_LO_SPRITE + k])
            | (u16::from(self.ram[GARNISH_Y_HI_SPRITE + k]) << 8)
    }

    // bool Garnish_ReturnIfPrepFails(int k, Point16U *pt) {  // 86e75e
    //   uint16 x = Garnish_GetX(k) - BG2HOFS_copy2;
    //   uint16 y = Garnish_GetY(k) - BG2VOFS_copy2;
    //   if (x >= 256 || y >= 256) {
    //     garnish_type[k] = 0;
    //     return true;
    //   }
    //   pt->x = x;
    //   pt->y = y - 16;
    //   return false;
    // }
    pub(super) fn garnish_return_if_prep_fails(&mut self, k: usize, pt: &mut Point16U) -> bool {
        let x = self
            .garnish_get_x(k)
            .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2));
        let y = self
            .garnish_get_y(k)
            .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2));
        if x >= 256 || y >= 256 {
            self.ram[GARNISH_TYPE + k] = 0;
            return true;
        }
        pt.x = x;
        pt.y = y.wrapping_sub(16);
        false
    }

    fn set_oam_plain_at_for_sprite(
        &mut self,
        oam: usize,
        x: u8,
        y: u8,
        charnum: u8,
        flags: u8,
        big: u8,
    ) {
        self.ram[oam] = x;
        self.ram[oam + 1] = y;
        self.ram[oam + 2] = charnum;
        self.ram[oam + 3] = flags;
        let ext_index = (oam - OAM_BUF) / 4;
        self.ram[BYTEWISE_EXTENDED_OAM + ext_index] = big;
    }

    // void Garnish_SparkleCommon(int k, uint8 shift) {  // 86dfb1
    //   static const uint8 kGarnishSparkle_Char[4] = {0x83, 0xc7, 0x80, 0xb7};
    //   uint8 t = garnish_countdown[k] >> shift;
    //   Point16U pt;
    //   if (Garnish_ReturnIfPrepFails(k, &pt))
    //     return;
    //   OamEnt *oam = GetOamCurPtr();
    //   int j = garnish_sprite[k];
    //   SetOamPlain(oam, pt.x, pt.y, kGarnishSparkle_Char[t],
    //                (sprite_oam_flags[j] | sprite_obj_prio[j]) & 0xf0 | 4, 0);
    // }
    pub(super) fn garnish_sparkle_common(&mut self, k: usize, shift: u8) {
        const GARNISH_SPARKLE_CHAR: [u8; 4] = [0x83, 0xc7, 0x80, 0xb7];

        let t = usize::from(self.ram[GARNISH_COUNTDOWN_SPRITE + k] >> shift);
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let j = usize::from(self.ram[GARNISH_SPRITE_SPRITE + k]);
        let flags = (self.ram[SPRITE_OAM_FLAGS + j] | self.ram[SPRITE_OBJ_PRIO + j]) & 0xf0 | 4;
        self.set_oam_plain_at_for_sprite(
            oam,
            pt.x as u8,
            pt.y as u8,
            GARNISH_SPARKLE_CHAR[t],
            flags,
            0,
        );
    }

    // void Garnish_DustCommon(int k, uint8 shift) {  // 86dfdc
    //   static const uint8 kRunningManDust_Char[3] = {0xdf, 0xcf, 0xa9};
    //   tmp_counter = garnish_countdown[k] >> shift;
    //   Point16U pt;
    //   if (Garnish_ReturnIfPrepFails(k, &pt))
    //     return;
    //   OamEnt *oam = GetOamCurPtr();
    //   SetOamPlain(oam, pt.x, pt.y, kRunningManDust_Char[tmp_counter], 0x24, 0);
    // }
    pub(super) fn garnish_dust_common(&mut self, k: usize, shift: u8) {
        const RUNNING_MAN_DUST_CHAR: [u8; 3] = [0xdf, 0xcf, 0xa9];

        self.ram[TMP_COUNTER_SPRITE] = self.ram[GARNISH_COUNTDOWN_SPRITE + k] >> shift;
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        self.set_oam_plain_at_for_sprite(
            oam,
            pt.x as u8,
            pt.y as u8,
            RUNNING_MAN_DUST_CHAR[usize::from(self.ram[TMP_COUNTER_SPRITE])],
            0x24,
            0,
        );
    }

    // void Garnish12_Sparkle(int k) { Garnish_SparkleCommon(k, 2); }
    pub(super) fn garnish12_sparkle(&mut self, k: usize) {
        self.garnish_sparkle_common(k, 2);
    }

    // void Garnish_SimpleSparkle(int k) { Garnish_SparkleCommon(k, 3); }
    pub(super) fn garnish_simple_sparkle(&mut self, k: usize) {
        self.garnish_sparkle_common(k, 3);
    }

    // void Garnish14_KakKidDashDust(int k) { Garnish_DustCommon(k, 2); }
    pub(super) fn garnish14_kak_kid_dash_dust(&mut self, k: usize) {
        self.garnish_dust_common(k, 2);
    }

    // void Garnish_WaterTrail(int k) { Garnish_DustCommon(k, 3); }
    pub(super) fn garnish_water_trail(&mut self, k: usize) {
        self.garnish_dust_common(k, 3);
    }

    // void Garnish04_LaserTrail(int k) {  // 86e000
    //   static const uint8 kLaserBeamTrail_Char[2] = {0xd2, 0xf3};
    //   Point16U pt;
    //   if (Garnish_ReturnIfPrepFails(k, &pt))
    //     return;
    //   SetOamPlain(GetOamCurPtr(), pt.x, pt.y, kLaserBeamTrail_Char[garnish_oam_flags[k]], 0x25, 0);
    // }
    pub(super) fn garnish04_laser_trail(&mut self, k: usize) {
        const LASER_BEAM_TRAIL_CHAR: [u8; 2] = [0xd2, 0xf3];

        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        self.set_oam_plain_at_for_sprite(
            oam,
            pt.x as u8,
            pt.y as u8,
            LASER_BEAM_TRAIL_CHAR[usize::from(self.ram[GARNISH_OAM_FLAGS_SPRITE + k])],
            0x25,
            0,
        );
    }

    // void Garnish06_ZoroTrail(int k) {  // 86e025
    //   Point16U pt;
    //   if (Garnish_ReturnIfPrepFails(k, &pt))
    //     return;
    //   int j = garnish_sprite[k];
    //   SetOamPlain(GetOamCurPtr(), pt.x, pt.y, 0x75, sprite_oam_flags[j] | sprite_obj_prio[j], 0);
    // }
    pub(super) fn garnish06_zoro_trail(&mut self, k: usize) {
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let j = usize::from(self.ram[GARNISH_SPRITE_SPRITE + k]);
        self.set_oam_plain_at_for_sprite(
            oam,
            pt.x as u8,
            pt.y as u8,
            0x75,
            self.ram[SPRITE_OAM_FLAGS + j] | self.ram[SPRITE_OBJ_PRIO + j],
            0,
        );
    }

    // void Garnish01_FireSnakeTail(int k) {  // 86e03e
    //   Point16U pt;
    //   if (Garnish_ReturnIfPrepFails(k, &pt))
    //     return;
    //   int j = garnish_sprite[k];
    //   SetOamPlain(GetOamCurPtr(), pt.x, pt.y, 0x28, sprite_oam_flags[j] | sprite_obj_prio[j], 2);
    // }
    pub(super) fn garnish01_fire_snake_tail(&mut self, k: usize) {
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let j = usize::from(self.ram[GARNISH_SPRITE_SPRITE + k]);
        self.set_oam_plain_at_for_sprite(
            oam,
            pt.x as u8,
            pt.y as u8,
            0x28,
            self.ram[SPRITE_OAM_FLAGS + j] | self.ram[SPRITE_OBJ_PRIO + j],
            2,
        );
    }

    // void Garnish02_MothulaBeamTrail(int k) {  // 86e057
    //   int j = garnish_sprite[k];
    //   SetOamPlain(GetOamCurPtr(), garnish_x_lo[k] - BG2HOFS_copy2, garnish_y_lo[k] - BG2VOFS_copy2, 0xaa,
    //                sprite_oam_flags[j] | sprite_obj_prio[j], 2);
    // }
    pub(super) fn garnish02_mothula_beam_trail(&mut self, k: usize) {
        let oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let j = usize::from(self.ram[GARNISH_SPRITE_SPRITE + k]);
        self.set_oam_plain_at_for_sprite(
            oam,
            self.ram[GARNISH_X_LO_SPRITE + k].wrapping_sub(self.ram[BG2HOFS_COPY2]),
            self.ram[GARNISH_Y_LO_SPRITE + k].wrapping_sub(self.ram[BG2VOFS_COPY2]),
            0xaa,
            self.ram[SPRITE_OAM_FLAGS + j] | self.ram[SPRITE_OBJ_PRIO + j],
            2,
        );
    }

    // void Garnish_CheckPlayerCollision(int k, int x, int y) {  // 89b459
    //   if ((k ^ frame_counter) & 7 | countdown_for_blink | link_disable_sprite_damage)
    //     return;
    //
    //   if ((uint8)(link_x_coord - BG2HOFS_copy2 - x + 12) < 24 &&
    //       (uint8)(link_y_coord - BG2VOFS_copy2 - y + 22) < 28) {
    //     link_auxiliary_state = 1;
    //     link_incapacitated_timer = 16;
    //     link_give_damage = 16;
    //     link_actual_vel_x ^= 255;
    //     link_actual_vel_y ^= 255;
    //   }
    // }
    pub(super) fn garnish_check_player_collision(&mut self, k: usize, x: i32, y: i32) {
        if (((k as u8) ^ self.ram[FRAME_COUNTER]) & 7)
            | self.ram[COUNTDOWN_FOR_BLINK]
            | self.ram[LINK_DISABLE_SPRITE_DAMAGE]
            != 0
        {
            return;
        }

        let link_x = self.player_state_view().x();
        let link_y = self.player_state_view().y();
        let bg_x = read_le_u16(&self.ram, BG2HOFS_COPY2);
        let bg_y = read_le_u16(&self.ram, BG2VOFS_COPY2);
        if (link_x
            .wrapping_sub(bg_x)
            .wrapping_sub(x as u16)
            .wrapping_add(12) as u8)
            < 24
            && (link_y
                .wrapping_sub(bg_y)
                .wrapping_sub(y as u16)
                .wrapping_add(22) as u8)
                < 28
        {
            self.ram[LINK_AUXILIARY_STATE] = 1;
            self.ram[LINK_INCAPACITATED_TIMER] = 16;
            self.ram[LINK_GIVE_DAMAGE] = 16;
            self.ram[LINK_ACTUAL_VEL_X] ^= 255;
            self.ram[LINK_ACTUAL_VEL_Y] ^= 255;
        }
    }

    // void Garnish15_ArrghusSplash(int k) {  // 89b178
    //   ...see sprite.c...
    // }
    pub(super) fn garnish15_arrghus_splash(&mut self, k: usize) {
        const ARRGHUS_SPLASH_X: [i8; 8] = [-12, 20, -10, 10, -8, 8, -4, 4];
        const ARRGHUS_SPLASH_Y: [i8; 8] = [-4, -4, -2, -2, 0, 0, 0, 0];
        const ARRGHUS_SPLASH_CHAR: [u8; 8] = [0xae, 0xae, 0xae, 0xae, 0xae, 0xae, 0xac, 0xac];
        const ARRGHUS_SPLASH_FLAGS: [u8; 8] = [0x34, 0x74, 0x34, 0x74, 0x34, 0x74, 0x34, 0x74];
        const ARRGHUS_SPLASH_EXT: [u8; 8] = [0, 0, 2, 2, 2, 2, 2, 2];

        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let g = usize::from((self.ram[GARNISH_COUNTDOWN_SPRITE + k] >> 1) & 6);
        for i in (0..=1).rev() {
            let j = i + g;
            self.set_oam_plain_at_for_sprite(
                oam,
                pt.x.wrapping_add(ARRGHUS_SPLASH_X[j] as i16 as u16) as u8,
                pt.y.wrapping_add(ARRGHUS_SPLASH_Y[j] as i16 as u16) as u8,
                ARRGHUS_SPLASH_CHAR[j],
                ARRGHUS_SPLASH_FLAGS[j],
                ARRGHUS_SPLASH_EXT[j],
            );
            oam += 4;
        }
    }

    // void Garnish13_PyramidDebris(int k) {  // 89b216
    //   ...see sprite.c...
    // }
    pub(super) fn garnish13_pyramid_debris(&mut self, k: usize) {
        let oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;

        let y = (i32::from(self.ram[GARNISH_Y_LO_SPRITE + k]) << 8)
            + i32::from(self.ram[GARNISH_Y_SUBPIXEL_SPRITE + k])
            + ((self.ram[GARNISH_Y_VEL_SPRITE + k] as i8 as i32) << 4);
        self.ram[GARNISH_Y_SUBPIXEL_SPRITE + k] = y as u8;
        self.ram[GARNISH_Y_LO_SPRITE + k] = (y >> 8) as u8;

        let x = (i32::from(self.ram[GARNISH_X_LO_SPRITE + k]) << 8)
            + i32::from(self.ram[GARNISH_X_SUBPIXEL_SPRITE + k])
            + ((self.ram[GARNISH_X_VEL_SPRITE + k] as i8 as i32) << 4);
        self.ram[GARNISH_X_SUBPIXEL_SPRITE + k] = x as u8;
        self.ram[GARNISH_X_LO_SPRITE + k] = (x >> 8) as u8;

        self.ram[GARNISH_Y_VEL_SPRITE + k] = self.ram[GARNISH_Y_VEL_SPRITE + k].wrapping_add(3);
        let t = self.ram[GARNISH_X_LO_SPRITE + k].wrapping_sub(self.ram[BG2HOFS_COPY2]);
        if t >= 248 {
            self.ram[GARNISH_TYPE + k] = 0;
            return;
        }
        self.ram[oam] = t;
        let t = self.ram[GARNISH_Y_LO_SPRITE + k].wrapping_sub(self.ram[BG2VOFS_COPY2]);
        if t >= 240 {
            self.ram[GARNISH_TYPE + k] = 0;
            return;
        }
        self.ram[oam + 1] = t;
        self.ram[oam + 2] = 0x5c;
        self.ram[oam + 3] = (self.ram[FRAME_COUNTER] << 3) & 0xc0 | 0x34;
        let ext_index = (oam - OAM_BUF) / 4;
        self.ram[BYTEWISE_EXTENDED_OAM + ext_index] = 0;
    }

    // void Garnish11_WitheringGanonBatFlame(int k) {  // 89b2b2
    //   ...see sprite.c...
    // }
    pub(super) fn garnish11_withering_ganon_bat_flame(&mut self, k: usize) {
        if (self.frame_control_view().submodule() | self.ram[FLAG_UNK1]) == 0 {
            let y = self.garnish_get_y(k).wrapping_sub(1);
            self.garnish_set_y(k, y);
        }
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        self.set_oam_plain_at_for_sprite(oam, pt.x as u8, pt.y as u8, 0xa4, 0x22, 0);
        self.set_oam_plain_at_for_sprite(
            oam + 4,
            pt.x.wrapping_add(8) as u8,
            pt.y as u8,
            0xa5,
            0x22,
            0,
        );
    }

    // void Garnish10_GanonBatFlame(int k) {  // 89b306
    //   ...see sprite.c...
    // }
    pub(super) fn garnish10_ganon_bat_flame(&mut self, k: usize) {
        const GANON_BAT_FLAME_IDX: [u8; 32] = [
            7, 6, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5,
            4, 5, 4,
        ];
        const GANON_BAT_FLAME_CHAR: [u8; 7] = [0xac, 0xac, 0x66, 0x66, 0x8e, 0xa0, 0xa2];
        const GANON_BAT_FLAME_FLAGS: [u8; 7] = [1, 0x41, 1, 0x41, 0, 0, 0];

        if self.ram[GARNISH_COUNTDOWN_SPRITE + k] == 8 {
            self.ram[GARNISH_TYPE + k] = 0x11;
        }
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let j = usize::from(
            GANON_BAT_FLAME_IDX[usize::from(self.ram[GARNISH_COUNTDOWN_SPRITE + k] >> 3)],
        );
        self.set_oam_plain_at_for_sprite(
            read_le_u16(&self.ram, OAM_CUR_PTR) as usize,
            pt.x as u8,
            pt.y as u8,
            GANON_BAT_FLAME_CHAR[j],
            GANON_BAT_FLAME_FLAGS[j] | 0x22,
            2,
        );
        self.garnish_check_player_collision(k, i32::from(pt.x), i32::from(pt.y));
    }

    // void Garnish0A_CannonSmoke(int k) {  // 89b3ee
    //   ...see sprite.c...
    // }
    pub(super) fn garnish0_a_cannon_smoke(&mut self, k: usize) {
        const GARNISH_CANNON_POOF_CHAR: [u8; 2] = [0x8a, 0x86];
        const GARNISH_CANNON_POOF_FLAGS: [u8; 4] = [0x20, 0x10, 0x30, 0x30];

        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let j = usize::from(self.ram[GARNISH_SPRITE_SPRITE + k]);
        self.set_oam_plain_at_for_sprite(
            read_le_u16(&self.ram, OAM_CUR_PTR) as usize,
            pt.x as u8,
            pt.y as u8,
            GARNISH_CANNON_POOF_CHAR[usize::from(self.ram[GARNISH_COUNTDOWN_SPRITE + k] >> 3)],
            GARNISH_CANNON_POOF_FLAGS[j] | 4,
            2,
        );
    }

    fn dungeon_update_tile_map_with_common_tile_for_garnish(&mut self, x: u16, y: u16, v: u8) {
        self.Dungeon_UpdateTileMapWithCommonTile(i32::from(x), i32::from(y), v);
    }

    // void Garnish0C_TrinexxIceBreath(int k) {  // 89b34f
    pub(super) fn garnish0_c_trinexx_ice_breath(&mut self, k: usize) {
        const TRINEXX_ICE_CHAR: [u8; 12] = [
            0xe8, 0xe8, 0xe6, 0xe6, 0xe4, 0xe4, 0xe4, 0xe4, 0xe4, 0xe4, 0xe4, 0xe4,
        ];
        const TRINEXX_ICE_FLAGS: [u8; 4] = [0, 0x40, 0xc0, 0x80];

        if self.ram[GARNISH_COUNTDOWN_SPRITE + k] == 0x50
            && (self.frame_control_view().submodule() | self.ram[FLAG_UNK1]) == 0
        {
            self.dungeon_update_tile_map_with_common_tile_for_garnish(
                self.garnish_get_x(k),
                self.garnish_get_y(k).wrapping_sub(16),
                18,
            );
        }
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        self.set_oam_plain_at_for_sprite(
            read_le_u16(&self.ram, OAM_CUR_PTR) as usize,
            pt.x as u8,
            pt.y as u8,
            TRINEXX_ICE_CHAR[usize::from(self.ram[GARNISH_COUNTDOWN_SPRITE + k] >> 4)],
            TRINEXX_ICE_FLAGS[usize::from((self.ram[GARNISH_COUNTDOWN_SPRITE + k] >> 2) & 3)]
                | 0x35,
            2,
        );
    }

    // void Garnish09_LightningTrail(int k) {  // 89b429
    //   ...see sprite.c...
    // }
    pub(super) fn garnish09_lightning_trail(&mut self, k: usize) {
        const LIGHTNING_TRAIL_CHAR: [u8; 8] = [0xcc, 0xec, 0xce, 0xee, 0xcc, 0xec, 0xce, 0xee];
        const LIGHTNING_TRAIL_FLAGS: [u8; 8] = [0x31, 0x31, 0x31, 0x31, 0x71, 0x71, 0x71, 0x71];

        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let j = usize::from(self.ram[GARNISH_SPRITE_SPRITE + k]);
        let room_offset = if self.ram[DUNGEON_ROOM_INDEX2] == 0x20 {
            0x80
        } else {
            0
        };
        self.set_oam_plain_at_for_sprite(
            read_le_u16(&self.ram, OAM_CUR_PTR) as usize,
            pt.x as u8,
            pt.y as u8,
            LIGHTNING_TRAIL_CHAR[j].wrapping_sub(room_offset),
            (self.ram[FRAME_COUNTER] << 1) & 0x0e | LIGHTNING_TRAIL_FLAGS[j],
            2,
        );
        self.garnish_check_player_collision(k, i32::from(pt.x), i32::from(pt.y));
    }

    // void Garnish03_FallingTile(int k) {  // 89b627
    pub(super) fn garnish03_falling_tile(&mut self, k: usize) {
        const CRUMBLE_TILE_XY: [u8; 5] = [4, 0, 0, 0, 0];
        const CRUMBLE_TILE_CHAR: [u8; 5] = [0x80, 0xcc, 0xcc, 0xea, 0xca];
        const CRUMBLE_TILE_FLAGS: [u8; 5] = [0x30, 0x31, 0x31, 0x31, 0x31];
        const CRUMBLE_TILE_EXT: [u8; 5] = [0, 2, 2, 2, 2];

        let mut j = self.ram[GARNISH_COUNTDOWN_SPRITE + k];
        if j == 0x1e {
            j = self.frame_control_view().submodule() | self.ram[FLAG_UNK1];
            if j == 0 {
                self.dungeon_update_tile_map_with_common_tile_for_garnish(
                    self.garnish_get_x(k),
                    self.garnish_get_y(k).wrapping_sub(16),
                    4,
                );
            }
        }
        let j = usize::from(j >> 3);
        let x = self
            .garnish_get_x(k)
            .wrapping_add(u16::from(CRUMBLE_TILE_XY[j]))
            .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2));
        let y = self
            .garnish_get_y(k)
            .wrapping_add(u16::from(CRUMBLE_TILE_XY[j]))
            .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2));
        if x < 256 && y < 256 {
            self.set_oam_plain_at_for_sprite(
                read_le_u16(&self.ram, OAM_CUR_PTR) as usize,
                x as u8,
                y.wrapping_sub(16) as u8,
                CRUMBLE_TILE_CHAR[j],
                CRUMBLE_TILE_FLAGS[j],
                CRUMBLE_TILE_EXT[j],
            );
        }
    }

    // void Garnish07_BabasuFlash(int k) {  // 89b49e
    pub(super) fn garnish07_babasu_flash(&mut self, k: usize) {
        const BABUSU_FLASH_CHAR: [u8; 4] = [0xa8, 0x8a, 0x86, 0x86];
        const BABUSU_FLASH_FLAGS: [u8; 4] = [0x2d, 0x2c, 0x2c, 0x2c];

        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let j = usize::from(self.ram[GARNISH_COUNTDOWN_SPRITE + k] >> 3);
        self.set_oam_plain_at_for_sprite(
            read_le_u16(&self.ram, OAM_CUR_PTR) as usize,
            pt.x as u8,
            pt.y as u8,
            BABUSU_FLASH_CHAR[j],
            BABUSU_FLASH_FLAGS[j],
            2,
        );
    }

    // void Garnish08_KholdstareTrail(int k) {  // 89b4c6
    pub(super) fn garnish08_kholdstare_trail(&mut self, k: usize) {
        const GARNISH_NEBULE_XY: [i8; 3] = [-1, -1, 0];
        const GARNISH_NEBULE_CHAR: [u8; 3] = [0x9c, 0x9d, 0x8d];

        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let i = usize::from(self.ram[GARNISH_COUNTDOWN_SPRITE + k] >> 2);
        let j = usize::from(self.ram[GARNISH_SPRITE_SPRITE + k]);
        self.set_oam_plain_at_for_sprite(
            read_le_u16(&self.ram, OAM_CUR_PTR) as usize,
            pt.x.wrapping_add(GARNISH_NEBULE_XY[i] as i16 as u16) as u8,
            pt.y.wrapping_add(GARNISH_NEBULE_XY[i] as i16 as u16) as u8,
            GARNISH_NEBULE_CHAR[i],
            (self.ram[SPRITE_OAM_FLAGS + j] | self.ram[SPRITE_OBJ_PRIO + j]) & !1,
            0,
        );
    }

    // void Garnish0E_TrinexxFireBreath(int k) {  // 89b55d
    pub(super) fn garnish0_e_trinexx_fire_breath(&mut self, k: usize) {
        const TRINEXX_LAVA_BUBBLE_CHAR: [u8; 4] = [0x83, 0xc7, 0x80, 0x9d];

        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let j = usize::from(self.ram[GARNISH_SPRITE_SPRITE + k]);
        self.set_oam_plain_at_for_sprite(
            read_le_u16(&self.ram, OAM_CUR_PTR) as usize,
            pt.x as u8,
            pt.y as u8,
            TRINEXX_LAVA_BUBBLE_CHAR[usize::from(self.ram[GARNISH_COUNTDOWN_SPRITE + k] >> 3)],
            (self.ram[SPRITE_OAM_FLAGS + j] | self.ram[SPRITE_OBJ_PRIO + j]) & 0xf0 | 0x0e,
            0,
        );
    }

    // void Garnish0F_BlindLaserTrail(int k) {  // 89b591
    pub(super) fn garnish0_f_blind_laser_trail(&mut self, k: usize) {
        const BLIND_LASER_TRAIL_CHAR: [u8; 4] = [0x61, 0x71, 0x70, 0x60];

        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let j = usize::from(self.ram[GARNISH_SPRITE_SPRITE + k]);
        self.set_oam_plain_at_for_sprite(
            read_le_u16(&self.ram, OAM_CUR_PTR) as usize,
            pt.x as u8,
            pt.y as u8,
            BLIND_LASER_TRAIL_CHAR
                [usize::from(self.ram[GARNISH_OAM_FLAGS_SPRITE + k].wrapping_sub(7))],
            self.ram[SPRITE_OAM_FLAGS + j] | self.ram[SPRITE_OBJ_PRIO + j],
            0,
        );
    }

    // void Garnish_ExecuteUpperSlots() {  // 89b08c
    //   HandleScreenFlash();
    //
    //   if (garnish_active) {
    //     for (int i = 29; i >= 15; i--)
    //       Garnish_ExecuteSingle(i);
    //   }
    // }
    pub(super) fn garnish_execute_upper_slots(&mut self) {
        self.handle_screen_flash();

        if self.ram[GARNISH_ACTIVE_SPRITE] != 0 {
            for i in (15..=29).rev() {
                self.garnish_execute_single(i);
            }
        }
    }

    // void Garnish_ExecuteLowerSlots() {  // 89b097
    //   if (garnish_active) {
    //     for (int i = 14; i >= 0; i--)
    //       Garnish_ExecuteSingle(i);
    //   }
    // }
    pub(super) fn garnish_execute_lower_slots(&mut self) {
        if self.ram[GARNISH_ACTIVE_SPRITE] != 0 {
            for i in (0..=14).rev() {
                self.garnish_execute_single(i);
            }
        }
    }

    // void Garnish_ExecuteSingle(int k) {  // 89b0b6
    //   ...see sprite.c...
    // }
    pub(super) fn garnish_execute_single(&mut self, k: usize) {
        const GARNISH_OAM_MEM_SIZE: [u8; 23] = [
            0, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 8, 16,
        ];

        self.ram[CUR_OBJECT_INDEX] = k as u8;
        let type_ = self.ram[GARNISH_TYPE + k];
        if type_ == 0 {
            return;
        }
        if (type_ == 5 || (self.frame_control_view().submodule() | self.ram[FLAG_UNK1]) == 0)
            && self.ram[GARNISH_COUNTDOWN_SPRITE + k] != 0
        {
            self.ram[GARNISH_COUNTDOWN_SPRITE + k] =
                self.ram[GARNISH_COUNTDOWN_SPRITE + k].wrapping_sub(1);
            if self.ram[GARNISH_COUNTDOWN_SPRITE + k] == 0 {
                self.ram[GARNISH_TYPE + k] = 0;
                return;
            }
        }

        let sprsize = GARNISH_OAM_MEM_SIZE[usize::from(type_)];
        if self.ram[SORT_SPRITES_SETTING] != 0 {
            if self.ram[GARNISH_FLOOR_SPRITE + k] != 0 {
                self.oam_allocate_from_region_f(sprsize);
            } else {
                self.oam_allocate_from_region_d(sprsize);
            }
        } else {
            self.oam_allocate_from_region_a(sprsize);
        }

        match type_ {
            1 => self.garnish01_fire_snake_tail(k),
            2 => self.garnish02_mothula_beam_trail(k),
            3 => self.garnish03_falling_tile(k),
            4 => self.garnish04_laser_trail(k),
            5 => self.garnish_simple_sparkle(k),
            6 => self.garnish06_zoro_trail(k),
            7 => self.garnish07_babasu_flash(k),
            8 => self.garnish08_kholdstare_trail(k),
            9 => self.garnish09_lightning_trail(k),
            10 => self.garnish0_a_cannon_smoke(k),
            11 => self.garnish_water_trail(k),
            12 => self.garnish0_c_trinexx_ice_breath(k),
            13 => {}
            14 => self.garnish0_e_trinexx_fire_breath(k),
            15 => self.garnish0_f_blind_laser_trail(k),
            16 => self.garnish10_ganon_bat_flame(k),
            17 => self.garnish11_withering_ganon_bat_flame(k),
            18 => self.garnish12_sparkle(k),
            19 => self.garnish13_pyramid_debris(k),
            20 => self.garnish14_kak_kid_dash_dust(k),
            21 => self.garnish15_arrghus_splash(k),
            22 => self.garnish16_thrown_item_debris(k),
            _ => {}
        }
    }

    // void Sprite_Get16BitCoords(int k) {
    //   cur_sprite_x = sprite_x_lo[k] | sprite_x_hi[k] << 8;
    //   cur_sprite_y = sprite_y_lo[k] | sprite_y_hi[k] << 8;
    // }
    pub(super) fn sprite_get16_bit_coords(&mut self, k: usize) {
        let x = self.sprite_get_x(k);
        let y = self.sprite_get_y(k);
        write_le_u16(&mut self.ram, CUR_SPRITE_X, x);
        write_le_u16(&mut self.ram, CUR_SPRITE_Y, y);
    }

    // void Sprite_inactiveSprite(int k) {  // 868510
    //   if (!player_is_indoors) {
    //     sprite_N_word[k] = 0xffff;
    //   } else {
    //     sprite_N[k] = 0xff;
    //   }
    // }
    pub(super) fn sprite_inactive_sprite(&mut self, k: usize) {
        if self.ram[PLAYER_IS_INDOORS] == 0 {
            write_le_u16(&mut self.ram, SPRITE_N_WORD + k * 2, 0xffff);
        } else {
            self.ram[SPRITE_N + k] = 0xff;
        }
    }

    // void Sprite_KillSelf(int k) {  // 89f1f8
    //   if (!(sprite_defl_bits[k] & 0x40) && player_is_indoors)
    //     return;
    //   sprite_state[k] = 0;
    //   uint16 blk = sprite_N_word[k];
    //   g_ram[0] = blk;
    //   WORD(g_ram[1]) = (blk >> 3) + 0xef80;
    //   uint8 loadedmask = (0x80 >> (blk & 7));
    //   uint16 addr = 0xEF80 + (blk >> 3);
    //   uint8 *loadedp = &g_ram[addr + 0x10000];
    //   if (blk < 0xffff)
    //     *loadedp &= ~loadedmask;
    //   if (!player_is_indoors)
    //     sprite_N_word[k] = 0xffff;
    //   else
    //     sprite_N[k] = 0xff;
    // }
    pub(super) fn sprite_kill_self(&mut self, k: usize) {
        if (self.ram[SPRITE_DEFL_BITS + k] & 0x40) == 0 && self.ram[PLAYER_IS_INDOORS] != 0 {
            return;
        }
        self.ram[SPRITE_STATE + k] = 0;
        let blk = read_le_u16(&self.ram, SPRITE_N_WORD + k * 2);
        self.ram[R0] = blk as u8;
        write_le_u16(&mut self.ram, R0 + 1, (blk >> 3).wrapping_add(0xef80));
        let loadedmask = 0x80 >> (blk & 7);
        if blk < 0xffff {
            let loaded_idx = (OVERWORLD_SPRITE_WAS_LOADED + ((blk >> 3) as usize)) & 0x1ffff;
            self.ram[loaded_idx] &= !loadedmask as u8;
        }
        if self.ram[PLAYER_IS_INDOORS] == 0 {
            write_le_u16(&mut self.ram, SPRITE_N_WORD + k * 2, 0xffff);
        } else {
            self.ram[SPRITE_N + k] = 0xff;
        }
    }

    // void Sprite_HitTimer31(int k) {
    //   if (sprite_type[k] != 0x7a || is_in_dark_world)
    //     return;
    //   if (sprite_health[k] <= sprite_give_damage[k]) {
    //     dialogue_message_index = 0x140;
    //     Sprite_ShowMessageMinimal();
    //   }
    // }
    pub(super) fn sprite_hit_timer31(&mut self, k: usize) {
        if self.ram[SPRITE_TYPE + k] != 0x7a || self.ram[IS_IN_DARK_WORLD_SPRITE] != 0 {
            return;
        }
        if self.ram[SPRITE_HEALTH + k] <= self.ram[SPRITE_GIVE_DAMAGE_SPRITE + k] {
            write_le_u16(&mut self.ram, DIALOGUE_MESSAGE_INDEX, 0x140);
            self.sprite_show_message_minimal_c();
        }
    }

    // bool Sprite_TrackBodyToHead(int k) {  // 85dca2
    //   if (sprite_head_dir[k] != sprite_D[k]) {
    //     if (frame_counter & 0x1f)
    //       return false;
    //     if (!((sprite_head_dir[k] ^ sprite_D[k]) & 2)) {
    //       sprite_D[k] = (((k ^ frame_counter) >> 5 | 2) & 3) ^ (sprite_head_dir[k] & 2);
    //       return false;
    //     }
    //   }
    //   sprite_D[k] = sprite_head_dir[k];
    //   return true;
    // }
    pub(super) fn sprite_track_body_to_head(&mut self, k: usize) -> bool {
        if self.ram[SPRITE_HEAD_DIR + k] != self.ram[SPRITE_D + k] {
            if (self.ram[FRAME_COUNTER] & 0x1f) != 0 {
                return false;
            }
            if ((self.ram[SPRITE_HEAD_DIR + k] ^ self.ram[SPRITE_D + k]) & 2) == 0 {
                self.ram[SPRITE_D + k] = ((((k as u8) ^ self.ram[FRAME_COUNTER]) >> 5) | 2) & 3
                    ^ (self.ram[SPRITE_HEAD_DIR + k] & 2);
                return false;
            }
        }
        self.ram[SPRITE_D + k] = self.ram[SPRITE_HEAD_DIR + k];
        true
    }

    // bool Sprite_CheckIfLinkIsBusy() {  // 87f4d0
    //   if (link_auxiliary_state | link_pose_for_item | (link_state_bits & 0x80))
    //     return true;
    //   for (int i = 4; i >= 0; i--) {
    //     if (ancilla_type[i] == 0x27)
    //       return true;
    //   }
    //   return false;
    // }
    pub(super) fn sprite_check_if_link_is_busy(&self) -> bool {
        if (self.ram[LINK_AUXILIARY_STATE] | self.ram[LINK_POSE_FOR_ITEM])
            | (self.ram[LINK_STATE_BITS] & 0x80)
            != 0
        {
            return true;
        }
        for i in (0..=4usize).rev() {
            if self.ram[ANCILLA_TYPE + i] == 0x27 {
                return true;
            }
        }
        false
    }

    // bool Sprite_ReturnIfInactive(int k) {
    //   return (sprite_state[k] != 9 || flag_unk1 || submodule_index
    //           || !(sprite_defl_bits[k] & 0x80) && sprite_pause[k]);
    // }
    // Note: in C this returns true when the caller should bail. Same here.
    pub(super) fn sprite_return_if_inactive(&self, k: usize) -> bool {
        if self.ram[SPRITE_STATE + k] != 9 {
            return true;
        }
        if self.ram[FLAG_UNK1] != 0 || self.frame_control_view().submodule() != 0 {
            return true;
        }
        (self.ram[SPRITE_DEFL_BITS + k] & 0x80) == 0 && self.ram[SPRITE_PAUSE + k] != 0
    }

    // bool Sprite_ReturnIfPaused(int k) {  // 86d9f3
    //   return (flag_unk1 || submodule_index || !(sprite_defl_bits[k] & 0x80) && sprite_pause[k]);
    // }
    pub(super) fn sprite_return_if_paused(&self, k: usize) -> bool {
        self.ram[FLAG_UNK1] != 0
            || self.frame_control_view().submodule() != 0
            || ((self.ram[SPRITE_DEFL_BITS + k] & 0x80) == 0 && self.ram[SPRITE_PAUSE + k] != 0)
    }

    // bool Sprite_ReturnIfPhasingOut(int k) {  // 86d0ed
    //   if (!sprite_stunned[k] || (submodule_index | flag_unk1))
    //     return false;
    //   if (!(frame_counter & 1))
    //     sprite_stunned[k]--;
    //   uint8 a = sprite_stunned[k];
    //   if (a == 0)
    //     sprite_state[k] = 0;
    //   else if (a >= 0x28 || (a & 1) != 0)
    //     return false;
    //   PrepOamCoordsRet info;
    //   Sprite_PrepOamCoordOrDoubleRet(k, &info);
    //   return true;
    // }
    pub(super) fn sprite_return_if_phasing_out(&mut self, k: usize) -> bool {
        if self.ram[SPRITE_STUNNED + k] == 0
            || (self.frame_control_view().submodule() | self.ram[FLAG_UNK1]) != 0
        {
            return false;
        }
        if (self.ram[FRAME_COUNTER] & 1) == 0 {
            self.ram[SPRITE_STUNNED + k] = self.ram[SPRITE_STUNNED + k].wrapping_sub(1);
        }
        let a = self.ram[SPRITE_STUNNED + k];
        if a == 0 {
            self.ram[SPRITE_STATE + k] = 0;
        } else if a >= 0x28 || (a & 1) != 0 {
            return false;
        }
        let _ = self.sprite_prep_oam_coord_or_double_ret(k);
        true
    }

    // bool SpriteDraw_AbsorbableTransient(int k, bool transient) {  // 86d22f
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_draw_absorbable_transient(&mut self, k: usize, transient: bool) -> bool {
        const ABSORBABLE_TAB1: [u8; 15] = [0, 1, 1, 1, 2, 2, 2, 0, 1, 1, 2, 2, 1, 2, 2];
        const ABSORBABLE_TAB2: [u8; 19] = [0, 0, 0, 0, 1, 2, 3, 0, 0, 4, 5, 0, 0, 0, 0, 2, 4, 6, 2];

        if transient && self.sprite_return_if_phasing_out(k) {
            return false;
        }
        if self.ram[SORT_SPRITES_SETTING] == 0 && self.ram[PLAYER_IS_INDOORS] != 0 {
            self.ram[SPRITE_OBJ_PRIO + k] = 0x30;
        }
        if self.ram[SPRITE_CHR_HALFSLOT_STATE] >= 3 {
            return false;
        }
        if self.ram[SPRITE_DELAY_AUX2_SPRITE + k] != 0 {
            self.oam_allocate_from_region_c(12);
        }
        if self.ram[SPRITE_E + k] != 0 {
            if self.read_u32_ram(ENHANCED_FEATURES0) & 4096 != 0 {
                self.ram[SPRITE_B + k] = 0;
            }
            return true;
        }

        let j = self.ram[SPRITE_TYPE + k].wrapping_sub(0xd8) as usize;
        let a = ABSORBABLE_TAB2[j];
        if a != 0 {
            self.sprite_draw_numbered_absorbable(k, i32::from(a));
            return false;
        }

        let t = ABSORBABLE_TAB1[j];
        if t == 0 {
            self.sprite_draw_single_small(k);
            return false;
        }
        if t == 2 {
            if self.ram[SPRITE_TYPE + k] == 0xe6 {
                if self.ram[SPRITE_SUBTYPE + k] == 1 {
                    self.sprite_draw_thin_and_tall(k);
                    return false;
                }
                self.ram[SPRITE_GRAPHICS + k] = 1;
            }
            self.sprite_draw_single_large(k);
            return false;
        }
        self.sprite_draw_thin_and_tall(k);
        false
    }

    // void Sprite_DrawNumberedAbsorbable(int k, int a) {  // 86d2fa
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_draw_numbered_absorbable(&mut self, k: usize, a: i32) {
        const X: [i16; 18] = [0, 0, 8, 0, 0, 8, 0, 0, 8, 0, 0, 2, 0, 0, 2, 0, 0, 0];
        const Y: [i16; 18] = [0, 0, 8, 0, 0, 8, 0, 0, 8, 0, 8, 8, 0, 8, 8, 0, 8, 8];
        const CHR: [u8; 18] = [
            0x6e, 0x6e, 0x68, 0x6e, 0x6e, 0x78, 0x6e, 0x6e, 0x79, 0x63, 0x73, 0x69, 0x63, 0x73,
            0x6a, 0x63, 0x73, 0x73,
        ];
        const EXT: [u8; 18] = [2, 2, 0, 2, 2, 0, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let base = ((a - 1) * 3).max(0) as usize;
        let n = if self.ram[SPRITE_HEAD_DIR + k] < 1 {
            2
        } else {
            1
        };
        for i in (0..=n).rev() {
            let j = (base + i).min(CHR.len() - 1);
            self.set_oam_helper0_at(
                oam,
                x.wrapping_add(X[j] as u16),
                y.wrapping_add(Y[j] as u16),
                CHR[j],
                flags,
                EXT[j],
            );
            oam += 4;
        }
        let mut info = PrepOamCoordsRet { x, y, r4: 0, flags };
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // void Sprite_HalveSpeed_XY(int k) {
    //   sprite_x_vel[k] = (int8)sprite_x_vel[k] >> 1;
    //   sprite_y_vel[k] = (int8)sprite_y_vel[k] >> 1;
    // }
    pub(super) fn sprite_halve_speed_xy(&mut self, k: usize) {
        self.ram[SPRITE_X_VEL + k] = ((self.ram[SPRITE_X_VEL + k] as i8) >> 1) as u8;
        self.ram[SPRITE_Y_VEL + k] = ((self.ram[SPRITE_Y_VEL + k] as i8) >> 1) as u8;
    }

    // void Sprite_ApplyRicochet(int k) {  // 86e229
    //   Sprite_InvertSpeed_XY(k);
    //   Sprite_HalveSpeed_XY(k);
    //   ThrowableScenery_TransmuteIfValid(k);
    // }
    pub(super) fn sprite_apply_ricochet(&mut self, k: usize) {
        self.sprite_invert_speed_xy(k);
        self.sprite_halve_speed_xy(k);
        self.throwable_scenery_transmute_if_valid(k);
    }

    // void ThrowableScenery_TransmuteIfValid(int k) {  // 86e22f
    //   if (sprite_type[k] != 0xec)
    //     return;
    //   repulsespark_timer = 0;
    //   ThrowableScenery_TransmuteToDebris(k);
    // }
    pub(super) fn throwable_scenery_transmute_if_valid(&mut self, k: usize) {
        if self.ram[SPRITE_TYPE + k] != 0xec {
            return;
        }
        self.ram[REPULSESPARK_TIMER_SPRITE] = 0;
        self.throwable_scenery_transmute_to_debris(k);
    }

    // void ThrowableScenery_TransmuteToDebris(int k) {  // 86e239
    //   uint8 a = sprite_graphics[k];
    //   if (a != 0) {
    //     BYTE(dung_secrets_unk1) = a;
    //     Sprite_SpawnSecret(k);
    //     BYTE(dung_secrets_unk1) = 0;
    //   }
    //   a = player_is_indoors ? 0 : sprite_C[k];
    //   sound_effect_1 = 0;
    //   SpriteSfx_QueueSfx2WithPan(k, kSprite_Func21_Sfx[a]);
    //   Sprite_ScheduleForBreakage(k);
    // }
    pub(super) fn throwable_scenery_transmute_to_debris(&mut self, k: usize) {
        const K_SPRITE_FUNC21_SFX: [u8; 9] = [0x1f, 0x1f, 0x1e, 0x1e, 0x1e, 0x1f, 0x1f, 0x1f, 0x1f];
        let mut a = self.ram[SPRITE_GRAPHICS + k];
        if a != 0 {
            self.ram[DUNG_SECRETS_UNK1_SPRITE] = a;
            self.sprite_spawn_secret(k);
            self.ram[DUNG_SECRETS_UNK1_SPRITE] = 0;
        }
        a = if self.ram[PLAYER_IS_INDOORS] != 0 {
            0
        } else {
            self.ram[SPRITE_C_SPRITE + k]
        };
        self.ram[SOUND_EFFECT_1] = 0;
        self.sprite_sfx_queue_sfx2_with_pan(k, K_SPRITE_FUNC21_SFX[a as usize]);
        self.sprite_schedule_for_breakage(k);
    }

    // void Sprite_Func18(int k, uint8 new_type) {  // 86edcb
    //   sprite_type[k] = new_type;
    //   SpritePrep_LoadProperties(k);
    //   Sprite_SpawnPoofGarnish(k);
    //   sound_effect_2 = 0;
    //   SpriteSfx_QueueSfx3WithPan(k, 0x32);
    //   sprite_hit_timer[k] = 0;
    //   sprite_give_damage[k] = 0;
    // }
    pub(super) fn sprite_func18(&mut self, k: usize, new_type: u8) {
        self.ram[SPRITE_TYPE + k] = new_type;
        self.sprite_prep_load_properties_for_helpers(k);
        self.sprite_spawn_poof_garnish(k);
        self.ram[SOUND_EFFECT_2] = 0;
        self.sprite_sfx_queue_sfx3_with_pan(k, 0x32);
        self.ram[SPRITE_HIT_TIMER + k] = 0;
        self.ram[SPRITE_GIVE_DAMAGE_SPRITE + k] = 0;
    }

    // void Sprite_Func15(int k, int a) {  // 86ed25
    //   damage_type_determiner = a;
    //   Sprite_ApplyCalculatedDamage(k, a == 8 ? 0x35 : 0x20);
    // }
    pub(super) fn sprite_func15(&mut self, k: usize, a: u8) {
        self.ram[DAMAGE_TYPE_DETERMINER_SPRITE] = a;
        self.sprite_apply_calculated_damage(k, if a == 8 { 0x35 } else { 0x20 });
    }

    // void Sprite_CalculateSwordDamage(int k) {  // 86ed3f
    //   if (sprite_flags3[k] & 0x40)
    //     return;
    //   sprite_unk1[k] = link_is_running;
    //   uint8 a = link_sword_type - 1;
    //   if (!link_is_running)
    //     a |= sign8(button_b_frames) ? 4 : sign8(button_b_frames - 9) ? 0 : 8;
    //   damage_type_determiner = kSprite_Func14_Damage[a];
    //   if (link_item_in_hand & 10)
    //     damage_type_determiner = 3;
    //   link_sword_delay_timer = 4;
    //   set_when_damaging_enemies = 16;
    //   Sprite_ApplyCalculatedDamage(k, 0x9d);
    // }
    pub(super) fn sprite_calculate_sword_damage(&mut self, k: usize) {
        const K_SPRITE_FUNC14_DAMAGE: [u8; 12] = [1, 2, 3, 4, 2, 3, 4, 5, 1, 1, 2, 3];
        if self.ram[SPRITE_FLAGS3 + k] & 0x40 != 0 {
            return;
        }
        self.ram[SPRITE_UNK1_LOC + k] = self.ram[LINK_IS_RUNNING];
        let mut a = self.ram[LINK_SWORD_TYPE].wrapping_sub(1);
        if self.ram[LINK_IS_RUNNING] == 0 {
            a |= if sign8(self.ram[BUTTON_B_FRAMES]) {
                4
            } else if sign8(self.ram[BUTTON_B_FRAMES].wrapping_sub(9)) {
                0
            } else {
                8
            };
        }
        self.ram[DAMAGE_TYPE_DETERMINER_SPRITE] = K_SPRITE_FUNC14_DAMAGE[a as usize];
        if self.ram[LINK_ITEM_IN_HAND] & 10 != 0 {
            self.ram[DAMAGE_TYPE_DETERMINER_SPRITE] = 3;
        }
        self.ram[LINK_SWORD_DELAY_TIMER] = 4;
        self.ram[SET_WHEN_DAMAGING_ENEMIES] = 16;
        self.sprite_apply_calculated_damage(k, 0x9d);
    }

    // void Sprite_ApplyCalculatedDamage(int k, int a) {  // 86ed89
    //   if ((sprite_flags3[k] & 0x40) || sprite_type[k] >= 0xD8)
    //     return;
    //   uint8 dmg = kEnemyDamages[damage_type_determiner * 8 | enemy_damage_data[sprite_type[k] * 16 | damage_type_determiner]];
    //   Sprite_GiveDamage(k, dmg, a);
    // }
    pub(super) fn sprite_apply_calculated_damage(&mut self, k: usize, a: u8) {
        const K_ENEMY_DAMAGES: [u8; 128] = [
            0, 1, 32, 255, 252, 251, 0, 0, 0, 2, 64, 4, 0, 0, 0, 0, 0, 4, 64, 2, 3, 0, 0, 0, 0, 8,
            64, 4, 0, 0, 0, 0, 0, 16, 64, 8, 0, 0, 0, 0, 0, 16, 64, 8, 0, 0, 0, 0, 0, 4, 64, 16, 0,
            0, 0, 0, 0, 255, 64, 255, 252, 251, 0, 0, 0, 4, 64, 255, 252, 251, 32, 0, 0, 100, 24,
            100, 0, 0, 0, 0, 0, 249, 250, 255, 100, 0, 0, 0, 0, 8, 64, 253, 4, 16, 0, 0, 0, 8, 64,
            254, 4, 0, 0, 0, 0, 16, 64, 253, 0, 0, 0, 0, 0, 254, 64, 16, 0, 0, 0, 0, 0, 32, 64,
            255, 0, 0, 0, 250,
        ];
        if self.ram[SPRITE_FLAGS3 + k] & 0x40 != 0 || self.ram[SPRITE_TYPE + k] >= 0xd8 {
            return;
        }
        let damage_type = self.ram[DAMAGE_TYPE_DETERMINER_SPRITE] as usize;
        let enemy_damage_index = self.ram[SPRITE_TYPE + k] as usize * 16 + damage_type;
        let dmg = K_ENEMY_DAMAGES
            [damage_type * 8 | self.ram[ENEMY_DAMAGE_DATA + enemy_damage_index] as usize];
        self.sprite_give_damage(k, dmg, a);
    }

    // void Sprite_GiveDamage(int k, uint8 dmg, uint8 r0_hit_timer) {  // 86edc5
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_give_damage(&mut self, k: usize, dmg: u8, r0_hit_timer: u8) {
        if std::env::var_os("ZELDA3_TRACE_GIVE_DAMAGE").is_some()
            && self.world_state_view().dungeon_room() == 0x00a8
            && self.ram[SPRITE_TYPE + k] == 0xa7
            && k == 2
        {
            eprintln!(
                "R give-damage entry fc={} k={} dmg=0x{:02x} hit=0x{:02x} type=0x{:02x} dmgtype=0x{:02x} x=0x{:04x} y=0x{:04x} f=0x{:02x} health=0x{:02x} give=0x{:02x} item=0x{:02x}",
                self.ram[FRAME_COUNTER],
                k,
                dmg,
                r0_hit_timer,
                self.ram[SPRITE_TYPE + k],
                self.ram[DAMAGE_TYPE_DETERMINER_SPRITE],
                self.sprite_get_x(k),
                self.sprite_get_y(k),
                self.ram[SPRITE_F + k],
                self.ram[SPRITE_HEALTH_SPRITE + k],
                self.ram[SPRITE_GIVE_DAMAGE_SPRITE + k],
                self.ram[LINK_ITEM_IN_HAND],
            );
        }
        if dmg == 249 {
            self.sprite_func18(k, 0xe3);
            return;
        }
        if dmg == 250 {
            self.sprite_func18(k, 0x8f);
            self.ram[SPRITE_AI_STATE_SPRITE + k] = 2;
            self.ram[SPRITE_Z_VEL + k] = 32;
            self.ram[SPRITE_OAM_FLAGS + k] = 8;
            self.ram[SPRITE_F + k] = 0;
            self.ram[SPRITE_HIT_TIMER + k] = 0;
            self.ram[SPRITE_HEALTH_SPRITE + k] = 0;
            self.ram[SPRITE_BUMP_DAMAGE + k] = 1;
            self.ram[SPRITE_FLAGS5 + k] = 1;
            return;
        }
        if dmg >= self.ram[SPRITE_GIVE_DAMAGE_SPRITE + k] {
            self.ram[SPRITE_GIVE_DAMAGE_SPRITE + k] = dmg;
        }
        if dmg == 0 {
            if self.ram[DAMAGE_TYPE_DETERMINER_SPRITE] != 10 {
                if self.ram[SPRITE_FLAGS_SPRITE + k] & 4 != 0 {
                    self.sprite_set_damage_stun(k);
                    return;
                }
                self.ram[LINK_SWORD_DELAY_TIMER] = 0;
            }
            self.ram[SPRITE_HIT_TIMER + k] = 0;
            self.ram[SPRITE_GIVE_DAMAGE_SPRITE + k] = 0;
            return;
        }
        if dmg >= 254 && self.ram[SPRITE_STATE + k] == 11 {
            self.ram[SPRITE_HIT_TIMER + k] = 0;
            self.ram[SPRITE_GIVE_DAMAGE_SPRITE + k] = 0;
            return;
        }
        if self.ram[SPRITE_TYPE + k] == 0x9a && self.ram[SPRITE_GIVE_DAMAGE_SPRITE + k] < 0xf0 {
            self.ram[SPRITE_STATE + k] = 9;
            self.ram[SPRITE_AI_STATE_SPRITE + k] = 4;
            self.ram[SPRITE_DELAY_MAIN + k] = 15;
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x28);
            return;
        }
        if self.ram[SPRITE_TYPE + k] == 0x1b {
            self.sprite_sfx_queue_sfx2_with_pan(k, 5);
            self.sprite_schedule_for_breakage(k);
            self.sprite_place_weapon_tink(k);
            return;
        }
        self.ram[SPRITE_HIT_TIMER + k] = r0_hit_timer;
        if self.ram[SPRITE_TYPE + k] != 0x92 || self.ram[SPRITE_C_SPRITE + k] >= 3 {
            let sfx = if self.ram[SPRITE_FLAGS_SPRITE + k] & 2 != 0 {
                0x21
            } else if self.ram[SPRITE_FLAGS5 + k] & 0x10 != 0 {
                0x1c
            } else {
                8
            };
            self.ram[SOUND_EFFECT_2] = sfx | self.sprite_calculate_sfx_pan(k);
        }
        self.sprite_set_damage_stun(k);
        if std::env::var_os("ZELDA3_TRACE_GIVE_DAMAGE").is_some()
            && self.world_state_view().dungeon_room() == 0x00a8
            && self.ram[SPRITE_TYPE + k] == 0xa7
            && k == 2
        {
            eprintln!(
                "R give-damage set-f fc={} k={} f=0x{:02x} dmg=0x{:02x} hit=0x{:02x} dmgtype=0x{:02x} xr=0x{:02x} yr=0x{:02x}",
                self.ram[FRAME_COUNTER],
                k,
                self.ram[SPRITE_F + k],
                dmg,
                r0_hit_timer,
                self.ram[DAMAGE_TYPE_DETERMINER_SPRITE],
                self.ram[SPRITE_X_RECOIL + k],
                self.ram[SPRITE_Y_RECOIL + k],
            );
        }
    }

    fn sprite_set_damage_stun(&mut self, k: usize) {
        let ty = self.ram[SPRITE_TYPE + k];
        self.ram[SPRITE_F + k] = if self.ram[DAMAGE_TYPE_DETERMINER_SPRITE] >= 13 {
            0
        } else if ty == 9 {
            20
        } else if ty == 0x53 || ty == 0x18 {
            11
        } else {
            15
        };
    }

    // void Sprite_ScheduleForBreakage(int k) {  // 86e25a
    //   sprite_delay_main[k] = 31;
    //   sprite_state[k] = 6;
    //   sprite_flags2[k] += 4;
    // }
    pub(super) fn sprite_schedule_for_breakage(&mut self, k: usize) {
        self.ram[SPRITE_DELAY_MAIN + k] = 31;
        self.ram[SPRITE_STATE + k] = 6;
        self.ram[SPRITE_FLAGS2 + k] = self.ram[SPRITE_FLAGS2 + k].wrapping_add(4);
    }

    // void Sprite_ZeroVelocity_XY(int k) {  // 86cf5d
    //   sprite_x_vel[k] = sprite_y_vel[k] = 0;
    // }
    pub(super) fn sprite_zero_velocity_xy(&mut self, k: usize) {
        self.ram[SPRITE_Y_VEL + k] = 0;
        self.ram[SPRITE_X_VEL + k] = self.ram[SPRITE_Y_VEL + k];
    }

    // void Sprite_Invert_XY_Speeds(int k) {
    //   sprite_x_vel[k] = -sprite_x_vel[k];
    //   sprite_y_vel[k] = -sprite_y_vel[k];
    // }
    pub(super) fn sprite_invert_xy_speeds(&mut self, k: usize) {
        self.ram[SPRITE_X_VEL + k] = self.ram[SPRITE_X_VEL + k].wrapping_neg();
        self.ram[SPRITE_Y_VEL + k] = self.ram[SPRITE_Y_VEL + k].wrapping_neg();
    }

    // void Sprite_BounceOffWall(int k) {  // 86d9c0
    //   if (sprite_wallcoll[k] & 3)
    //     sprite_x_vel[k] = -sprite_x_vel[k];
    //   if (sprite_wallcoll[k] & 12)
    //     sprite_y_vel[k] = -sprite_y_vel[k];
    // }
    pub(super) fn sprite_bounce_off_wall(&mut self, k: usize) {
        if (self.ram[SPRITE_WALLCOLL + k] & 3) != 0 {
            self.ram[SPRITE_X_VEL + k] = self.ram[SPRITE_X_VEL + k].wrapping_neg();
        }
        if (self.ram[SPRITE_WALLCOLL + k] & 12) != 0 {
            self.ram[SPRITE_Y_VEL + k] = self.ram[SPRITE_Y_VEL + k].wrapping_neg();
        }
    }

    // void Sprite_InvertSpeed_XY(int k) {  // 86d9d5
    //   sprite_x_vel[k] = -sprite_x_vel[k];
    //   sprite_y_vel[k] = -sprite_y_vel[k];
    // }
    pub(super) fn sprite_invert_speed_xy(&mut self, k: usize) {
        self.ram[SPRITE_X_VEL + k] = self.ram[SPRITE_X_VEL + k].wrapping_neg();
        self.ram[SPRITE_Y_VEL + k] = self.ram[SPRITE_Y_VEL + k].wrapping_neg();
    }

    // void Sprite_MoveZ(int k) {
    //   uint16 z = (sprite_z[k] << 8 | sprite_z_subpos[k]) + ((int8)sprite_z_vel[k] << 4);
    //   sprite_z_subpos[k] = z;
    //   sprite_z[k] = z >> 8;
    // }
    pub(super) fn sprite_move_z(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).move_z();
    }

    // void Sprite_ApplySpeedTowardsLink(int k, uint8 vel) {
    //   ProjectSpeedRet pt = Sprite_ProjectSpeedTowardsLink(k, vel);
    //   sprite_x_vel[k] = pt.x;
    //   sprite_y_vel[k] = pt.y;
    // }
    pub(super) fn sprite_apply_speed_towards_link(&mut self, k: usize, vel: u8) {
        let pt = self.sprite_project_speed_towards_link(k, vel);
        self.ram[SPRITE_X_VEL + k] = pt.x;
        self.ram[SPRITE_Y_VEL + k] = pt.y;
    }

    // void Sprite_SetSpawnedCoordinates(int k, SpriteSpawnInfo *info) {
    //   sprite_x_lo[k] = info->r0_x;
    //   sprite_x_hi[k] = info->r0_x >> 8;
    //   sprite_y_lo[k] = info->r2_y;
    //   sprite_y_hi[k] = info->r2_y >> 8;
    //   sprite_z[k] = info->r4_z;
    // }
    pub(super) fn sprite_set_spawned_coordinates(&mut self, k: usize, info: &SpriteSpawnInfo) {
        let mut sprite = self.sprite_slot_view_mut(k);
        sprite.set_x(info.r0_x);
        sprite.set_y(info.r2_y);
        sprite.set_z(info.r4_z);
    }

    pub(super) fn sprite_explode_spawn_ea(&mut self, k: usize) {
        self.ram[TMP_COUNTER_SPRITE] = self.ram[SPRITE_TYPE + k];
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically_ex(k, 0xea, &mut info, 14);
        if j < 0 {
            return;
        }
        let j = j as usize;
        self.sprite_set_spawned_coordinates(j, &info);
        self.ram[SPRITE_Z_VEL + j] = 32;
        self.ram[SPRITE_FLOOR + j] = self.ram[LINK_IS_ON_LOWER_LEVEL];
        self.ram[SPRITE_A + j] = if j == 9 { 2 } else { 6 };
        self.sprite_set_y(j, info.r2_y.wrapping_add(3));
        if self.ram[TMP_COUNTER_SPRITE] == 0xce {
            self.sprite_set_y(j, info.r2_y.wrapping_add(16));
            return;
        }
        if self.ram[TMP_COUNTER_SPRITE] == 0xcb {
            let player = self.player_state_view();
            let link_x_hi = (player.x() >> 8) as u8;
            let link_y_hi = (player.y() >> 8) as u8;
            self.ram[SPRITE_Y_LO + j] = 0x78;
            self.ram[SPRITE_X_LO + j] = 0x78;
            self.ram[SPRITE_X_HI + j] = link_x_hi;
            self.ram[SPRITE_Y_HI + j] = link_y_hi;
        }
    }

    // void SpriteModule_Die(int k) {  // 86f8a2
    //   SpriteDeath_MainEx(k, false);
    // }
    pub(super) fn sprite_module_die(&mut self, k: usize) {
        self.sprite_death_main_ex(k, false);
    }

    // void SpriteDeath_MainEx(int k, bool second_entry) {  // 86823a
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_death_main_ex(&mut self, k: usize, second_entry: bool) {
        if !second_entry {
            let type_ = self.ram[SPRITE_TYPE + k];
            if type_ == 0xec {
                self.throwable_scenery_scatter_into_debris(k);
                return;
            }
            if type_ == 0x53
                || type_ == 0x54
                || type_ == 0x92
                || (type_ == 0x4a && self.ram[SPRITE_C_SPRITE + k] >= 2)
            {
                self.sprite_active_main_for_death(k);
                return;
            }
            if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                self.sprite_do_the_death(k);
                return;
            }
        }
        if sign8(self.ram[SPRITE_FLAGS3 + k]) {
            self.sprite_active_main_for_death(k);
            return;
        }
        if ((self.ram[FRAME_COUNTER] & 3)
            | self.frame_control_view().submodule()
            | self.ram[FLAG_UNK1])
            == 0
        {
            self.ram[SPRITE_DELAY_MAIN + k] = self.ram[SPRITE_DELAY_MAIN + k].wrapping_add(1);
        }
        self.sprite_death_draw_poof(k);

        if self.ram[SPRITE_TYPE + k] != 0x40 && self.ram[SPRITE_DELAY_MAIN + k] < 10 {
            return;
        }
        let oam = read_le_u16(&self.ram, OAM_CUR_PTR).wrapping_add(16);
        let ext = read_le_u16(&self.ram, OAM_EXT_CUR_PTR).wrapping_add(4);
        write_le_u16(&mut self.ram, OAM_CUR_PTR, oam);
        write_le_u16(&mut self.ram, OAM_EXT_CUR_PTR, ext);
        let bak = self.ram[SPRITE_FLAGS2 + k];
        self.ram[SPRITE_FLAGS2 + k] = self.ram[SPRITE_FLAGS2 + k].wrapping_sub(4);
        self.sprite_active_main_for_death(k);
        self.ram[SPRITE_FLAGS2 + k] = bak;
    }

    fn sprite_active_main_for_death(&mut self, k: usize) {
        self.sprite_active_main(k);
    }

    // void Sprite_DoTheDeath(int k) {  // 86f923
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_do_the_death(&mut self, k: usize) {
        const PIKIT_DROP_ITEMS: [u8; 4] = [0xdc, 0xe1, 0xd9, 0xe6];
        const PRIZE_MASKS: [u8; 7] = [1, 1, 1, 0, 1, 1, 1];

        let type_ = self.ram[SPRITE_TYPE + k];
        if type_ == 0xbe {
            self.ram[SPRITE_G] = self.ram[SPRITE_G].wrapping_sub(1);
        }

        if type_ == 0xaa && self.ram[SPRITE_E + k] != 0 {
            let bak = self.ram[SPRITE_SUBTYPE + k];
            let item = PIKIT_DROP_ITEMS[usize::from(self.ram[SPRITE_E + k].wrapping_sub(1))];
            self.prepare_enemy_drop(k, item);
            self.ram[SPRITE_SUBTYPE + k] = bak;
            if bak == 1 {
                self.ram[SPRITE_OAM_FLAGS + k] = 9;
                self.ram[SPRITE_FLAGS3 + k] = 0xf0;
            }
            self.ram[SPRITE_HEAD_DIR + k] = self.ram[SPRITE_HEAD_DIR + k].wrapping_add(1);
            return;
        }

        if type_ == 0x45
            && self.ram[SRAM_PROGRESS_INDICATOR_SPRITE] == 2
            && self.ram[OVERWORLD_AREA_INDEX_SPRITE] == 0x18
        {
            self.ram[MUSIC_CONTROL] = 7;
        }

        let drop_item = self.ram[SPRITE_DIE_ACTION + k];
        if drop_item != 0 {
            self.ram[SPRITE_SUBTYPE + k] = self.ram[SPRITE_N + k];
            self.ram[SPRITE_N + k] = 255;
            let arg = if drop_item == 1 {
                0xe4
            } else if drop_item == 3 {
                0xd9
            } else {
                0xe5
            };
            self.prepare_enemy_drop(k, arg);
            return;
        }

        let mut prize = self.ram[SPRITE_FLAGS5 + k] & 0x0f;
        if prize != 0 {
            prize = prize.wrapping_sub(1);
            let luck = self.ram[ITEM_DROP_LUCK_SPRITE];
            if luck != 0 {
                self.ram[LUCK_KILL_COUNTER_SPRITE] =
                    self.ram[LUCK_KILL_COUNTER_SPRITE].wrapping_add(1);
                if self.ram[LUCK_KILL_COUNTER_SPRITE] >= 10 {
                    self.ram[ITEM_DROP_LUCK_SPRITE] = 0;
                }
                if luck == 1 {
                    self.force_prize_drop(k, prize, 1);
                    return;
                }
            } else if (self.get_random_number() & PRIZE_MASKS[usize::from(prize)]) == 0 {
                self.force_prize_drop(k, prize, prize);
                return;
            }
        }
        self.ram[SPRITE_STATE + k] = 0;
        self.sprite_death_func4(k);
    }

    // void ForcePrizeDrop(int k, uint8 prize, uint8 slot) {  // 86f9bc
    //   prize = prize * 8 | prizes_arr1[slot];
    //   prizes_arr1[slot] = (prizes_arr1[slot] + 1) & 7;
    //   PrepareEnemyDrop(k, kPrizeItems[prize]);
    // }
    pub(super) fn force_prize_drop(&mut self, k: usize, prize: u8, slot: u8) {
        const PRIZE_ITEMS: [u8; 56] = [
            0xd8, 0xd8, 0xd8, 0xd8, 0xd9, 0xd8, 0xd8, 0xd9, 0xda, 0xd9, 0xda, 0xdb, 0xda, 0xd9,
            0xda, 0xda, 0xe0, 0xdf, 0xdf, 0xda, 0xe0, 0xdf, 0xd8, 0xdf, 0xdc, 0xdc, 0xdc, 0xdd,
            0xdc, 0xdc, 0xde, 0xdc, 0xe1, 0xd8, 0xe1, 0xe2, 0xe1, 0xd8, 0xe1, 0xe2, 0xdf, 0xd9,
            0xd8, 0xe1, 0xdf, 0xdc, 0xd9, 0xd8, 0xd8, 0xe3, 0xe0, 0xdb, 0xde, 0xd8, 0xdb, 0xe2,
        ];
        let slot = usize::from(slot);
        let prize = usize::from(prize) * 8 | usize::from(self.ram[PRIZES_ARR1_SPRITE + slot]);
        self.ram[PRIZES_ARR1_SPRITE + slot] =
            self.ram[PRIZES_ARR1_SPRITE + slot].wrapping_add(1) & 7;
        self.prepare_enemy_drop(k, PRIZE_ITEMS[prize]);
    }

    // void PrepareEnemyDrop(int k, uint8 item) {  // 86f9d1
    //   ...see sprite.c...
    // }
    pub(super) fn prepare_enemy_drop(&mut self, k: usize, item: u8) {
        const PRIZE_Z: [u8; 15] = [
            0, 0x24, 0x24, 0x24, 0x20, 0x20, 0x20, 0x24, 0x24, 0x24, 0x24, 0, 0x24, 0x20, 0x20,
        ];

        self.ram[SPRITE_TYPE + k] = item;
        if item == 0xe5 {
            self.sprite_prep_big_key_load_graphics(k);
        } else if item == 0xe4 {
            self.sprite_prep_key_set_item_drop(k);
        }

        self.ram[SPRITE_STATE + k] = 9;
        let zbak = self.ram[SPRITE_Z + k];
        self.sprite_prep_load_properties_for_helpers(k);
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);

        let pz = PRIZE_Z[usize::from(self.ram[SPRITE_TYPE + k].wrapping_sub(0xd8))];
        self.ram[SPRITE_Z_VEL + k] = pz & 0xf0;
        self.sprite_set_x(k, self.sprite_get_x(k).wrapping_add(u16::from(pz & 0x0f)));
        self.ram[SPRITE_Z + k] = zbak;
        self.ram[SPRITE_DELAY_AUX4 + k] = 21;
        self.ram[SPRITE_STUNNED + k] = 255;
        self.sprite_death_func4(k);
    }

    // void SpriteDeath_Func4(int k) {  // 86fa25
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_death_func4(&mut self, k: usize) {
        if self.ram[SPRITE_TYPE + k] == 0xa2 && self.sprite_check_if_screen_is_clear() {
            self.ancilla_spawn_falling_prize(4);
        }
        self.sprite_manually_set_death_flag_uw(k);
        self.ram[NUM_SPRITES_KILLED_SPRITE] = self.ram[NUM_SPRITES_KILLED_SPRITE].wrapping_add(1);
        if self.ram[SPRITE_TYPE + k] == 0x40 {
            self.ram[SPRITE_STATE + k] = 9;
            self.ram[SPRITE_GRAPHICS + k] = 4;
            self.sprite_death_main_ex(k, true);
        }
    }

    pub(super) fn sprite_death_draw_poof(&mut self, k: usize) {
        const X: [i8; 32] = [
            0, 0, 0, 8, 0, 8, 0, 8, 8, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, -3, 11, -3, 11,
            -6, 14, -6, 14,
        ];
        const Y: [i8; 32] = [
            0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8, -3, -3, 11, 11,
            -6, -6, 14, 14,
        ];
        const CHR: [u8; 32] = [
            0, 0xb9, 0, 0, 0xb4, 0xb5, 0xb5, 0xb4, 0xb9, 0, 0, 0, 0xb5, 0xb4, 0xb4, 0xb5, 0xa8,
            0xa8, 0xb8, 0xb8, 0xa8, 0xa8, 0xb8, 0xb8, 0xa9, 0xa9, 0xa9, 0xa9, 0x9b, 0x9b, 0x9b,
            0x9b,
        ];
        const FLAGS: [u8; 32] = [
            4, 4, 4, 4, 4, 4, 0xc4, 0xc4, 0x44, 4, 4, 4, 0x44, 0x44, 0x84, 0x84, 4, 0x44, 4, 0x44,
            4, 0x44, 4, 0x44, 0x44, 4, 0xc4, 0x84, 4, 0x44, 0x84, 0xc4,
        ];

        if self.ram[DUNG_HDR_COLLISION] == 4 {
            self.ram[SPRITE_OBJ_PRIO + k] = 0x30;
        }
        let Some((_x, _y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let r12 = (self.ram[SPRITE_FLAGS3 + k] & 0x20) >> 3;
        let dungmap_x = self.ram[DUNGMAP_VAR7];
        let dungmap_y = self.ram[DUNGMAP_VAR7 + 1];
        let mut i = usize::from((self.ram[SPRITE_DELAY_MAIN + k] & 0x1c) ^ 0x1c) + 3;
        for _ in 0..4 {
            if CHR[i] != 0 {
                self.ram[oam + 2] = CHR[i];
                self.ram[oam + 1] = dungmap_y.wrapping_sub(r12).wrapping_add(Y[i] as u8);
                self.ram[oam] = dungmap_x.wrapping_sub(r12).wrapping_add(X[i] as u8);
                self.ram[oam + 3] = (flags & 0x30) | FLAGS[i];
            }
            oam += 4;
            i = i.wrapping_sub(1);
        }
        self.sprite_correct_oam_entries(k, 3, 0);
    }

    // void SpriteModule_Fall1(int k) {  // 86852e
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_module_fall1(&mut self, k: usize) {
        if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
            self.ram[SPRITE_STATE + k] = 0;
            self.sprite_manually_set_death_flag_uw(k);
            return;
        }
        let (mut info, out) = self.sprite_prep_oam_coord_or_double_ret_raw(k);
        if !out {
            self.sprite_fall_draw(k, &mut info);
        }
    }

    // void SpriteModule_Burn(int k) {  // sprite.c:747
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_module_burn(&mut self, k: usize) {
        const FLAME_GFX: [u8; 32] = [
            5, 4, 3, 1, 2, 0, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1,
            2, 3, 0,
        ];
        self.ram[SPRITE_HIT_TIMER + k] = 0;
        let j = i16::from(self.ram[SPRITE_DELAY_MAIN + k]) - 1;
        if j == 0 {
            self.sprite_do_the_death(k);
            return;
        }
        let bak_graphics = self.ram[SPRITE_GRAPHICS + k];
        let bak_oam = self.ram[SPRITE_OAM_FLAGS + k];
        self.ram[SPRITE_GRAPHICS + k] = FLAME_GFX[(j >> 3) as usize];
        self.ram[SPRITE_OAM_FLAGS + k] = 3;
        self.flame_draw(k);
        self.ram[SPRITE_OAM_FLAGS + k] = bak_oam;
        self.ram[SPRITE_GRAPHICS + k] = bak_graphics;
        let next_oam = read_le_u16(&self.ram, OAM_CUR_PTR).wrapping_add(8);
        let next_ext = read_le_u16(&self.ram, OAM_EXT_CUR_PTR).wrapping_add(2);
        write_le_u16(&mut self.ram, OAM_CUR_PTR, next_oam);
        write_le_u16(&mut self.ram, OAM_EXT_CUR_PTR, next_ext);
        if self.ram[SPRITE_DELAY_MAIN + k] >= 0x10 {
            let bak = self.ram[SPRITE_FLAGS2 + k];
            self.ram[SPRITE_FLAGS2 + k] = self.ram[SPRITE_FLAGS2 + k].wrapping_sub(2);
            self.sprite_active_main_for_death(k);
            self.ram[SPRITE_FLAGS2 + k] = bak;
        }
    }

    // void SpriteModule_Poof(int k) {  // 86e393
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_module_poof(&mut self, k: usize) {
        const X: [i8; 16] = [-6, 10, 1, 13, -6, 10, 1, 13, -7, 4, -5, 6, -1, 1, -2, 0];
        const Y: [i8; 16] = [-6, -4, 10, 9, -6, -4, 10, 9, -8, -10, 4, 3, -1, -2, 0, 1];
        const CHR: [u8; 16] = [
            0x9b, 0x9b, 0x9b, 0x9b, 0xb3, 0xb3, 0xb3, 0xb3, 0x8a, 0x8a, 0x8a, 0x8a, 0x8a, 0x8a,
            0x8a, 0x8a,
        ];
        const FLAGS: [u8; 16] = [
            0x24, 0xa4, 0x24, 0xa4, 0xe4, 0x64, 0xa4, 0x24, 0x24, 0xe4, 0xe4, 0xe4, 0x24, 0xe4,
            0xe4, 0xe4,
        ];
        const EXT: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2];

        if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
            if self.ram[SPRITE_TYPE + k] == 0x0d && self.ram[SPRITE_HEAD_DIR + k] != 0 {
                let bakx = self.sprite_get_x(k);
                self.prepare_enemy_drop(k, 0x0d);
                self.sprite_set_x(k, bakx);
                self.ram[SPRITE_Z_VEL + k] = 0;
                self.ram[SPRITE_IGNORE_PROJECTILE + k] = 0;
            } else if self.ram[SPRITE_DIE_ACTION + k] == 0 {
                self.force_prize_drop(k, 2, 2);
            } else {
                self.sprite_do_the_death(k);
            }
            return;
        }

        let Some((_x, _y, _flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let mut j = usize::from(((self.ram[SPRITE_DELAY_MAIN + k] >> 1) & !3) + 3).min(15);
        let base_x = self.ram[DUNGMAP_VAR7];
        let base_y = self.ram[DUNGMAP_VAR7 + 1];
        for _ in 0..4 {
            self.set_oam_plain_at_for_sprite(
                oam,
                base_x.wrapping_add(X[j] as u8),
                base_y.wrapping_add(Y[j] as u8),
                CHR[j],
                FLAGS[j],
                EXT[j],
            );
            oam += 4;
            j = j.saturating_sub(1);
        }
        self.sprite_correct_oam_entries(k, 3, 0xff);
    }

    // void SpriteModule_Drown(int k) {  // 86859c
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_module_drown(&mut self, k: usize) {
        const DROWN_DMD: [DrawMultipleData; 8] = [
            DrawMultipleData {
                x: -7,
                y: -7,
                char_flags: 0x0480,
                ext: 0,
            },
            DrawMultipleData {
                x: 14,
                y: -6,
                char_flags: 0x0483,
                ext: 0,
            },
            DrawMultipleData {
                x: -6,
                y: -6,
                char_flags: 0x04cf,
                ext: 0,
            },
            DrawMultipleData {
                x: 13,
                y: -5,
                char_flags: 0x04df,
                ext: 0,
            },
            DrawMultipleData {
                x: -4,
                y: -4,
                char_flags: 0x04ae,
                ext: 0,
            },
            DrawMultipleData {
                x: 12,
                y: -4,
                char_flags: 0x44af,
                ext: 0,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x04e7,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x04e7,
                ext: 2,
            },
        ];
        const OAM_FLAGS: [u8; 4] = [0, 0x40, 0xc0, 0x80];
        const OAM_CHAR: [u8; 11] = [
            0xc0, 0xc0, 0xc0, 0xc0, 0xcd, 0xcd, 0xcd, 0xcb, 0xcb, 0xcb, 0xcb,
        ];

        if self.ram[SPRITE_AI_STATE_SPRITE + k] != 0 {
            if self.ram[SPRITE_A + k] == 6 {
                self.oam_allocate_from_region_c(8);
            }
            self.ram[SPRITE_FLAGS3 + k] ^= 16;
            self.sprite_draw_single_large(k);
            let oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
            let j = self.ram[SPRITE_DELAY_MAIN + k];
            if j == 1 {
                self.ram[SPRITE_STATE + k] = 0;
            }
            if j != 0 {
                self.ram[oam + 2] = OAM_CHAR[usize::from((j >> 1).min(10))];
                self.ram[oam + 3] = 0x24;
                return;
            }
            self.ram[oam + 2] = 0x8a;
            self.ram[oam + 3] =
                OAM_FLAGS[usize::from((self.ram[SPRITE_SUBTYPE2 + k] >> 2) & 3)] | 0x24;
            if self.sprite_return_if_paused(k) {
                return;
            }
            self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
            self.sprite_move_xy(k);
            self.sprite_move_z(k);
            self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_Z_VEL + k].wrapping_sub(2);
            if sign8(self.ram[SPRITE_Z + k]) {
                self.ram[SPRITE_Z + k] = 0;
                self.ram[SPRITE_DELAY_MAIN + k] = 18;
                self.ram[SPRITE_FLAGS3 + k] &= !0x10;
            }
        } else {
            if self.sprite_return_if_paused(k) {
                return;
            }
            if self.ram[FRAME_COUNTER] & 1 == 0 {
                self.ram[SPRITE_DELAY_MAIN + k] = self.ram[SPRITE_DELAY_MAIN + k].wrapping_add(1);
            }
            self.ram[SPRITE_OAM_FLAGS + k] = 0;
            self.ram[SPRITE_HIT_TIMER + k] = 0;
            if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                self.ram[SPRITE_STATE + k] = 0;
            }
            let base = usize::from(((self.ram[SPRITE_DELAY_MAIN + k] << 1) & 0xf8) >> 2).min(6);
            self.sprite_draw_multiple(k, &DROWN_DMD[base..base + 2], None);
        }
    }

    // void SpriteModule_Explode(int k) {  // sprite.c:616
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_module_explode(&mut self, k: usize) {
        const SPRITE_EXPLODE_DMD: [DrawMultipleData; 32] = [
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x0060,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x0060,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x0060,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x0060,
                ext: 2,
            },
            DrawMultipleData {
                x: -5,
                y: -5,
                char_flags: 0x0062,
                ext: 2,
            },
            DrawMultipleData {
                x: 5,
                y: -5,
                char_flags: 0x4062,
                ext: 2,
            },
            DrawMultipleData {
                x: -5,
                y: 5,
                char_flags: 0x8062,
                ext: 2,
            },
            DrawMultipleData {
                x: 5,
                y: 5,
                char_flags: 0xc062,
                ext: 2,
            },
            DrawMultipleData {
                x: -8,
                y: -8,
                char_flags: 0x0062,
                ext: 2,
            },
            DrawMultipleData {
                x: 8,
                y: -8,
                char_flags: 0x4062,
                ext: 2,
            },
            DrawMultipleData {
                x: -8,
                y: 8,
                char_flags: 0x8062,
                ext: 2,
            },
            DrawMultipleData {
                x: 8,
                y: 8,
                char_flags: 0xc062,
                ext: 2,
            },
            DrawMultipleData {
                x: -8,
                y: -8,
                char_flags: 0x0064,
                ext: 2,
            },
            DrawMultipleData {
                x: 8,
                y: -8,
                char_flags: 0x4064,
                ext: 2,
            },
            DrawMultipleData {
                x: -8,
                y: 8,
                char_flags: 0x8064,
                ext: 2,
            },
            DrawMultipleData {
                x: 8,
                y: 8,
                char_flags: 0xc064,
                ext: 2,
            },
            DrawMultipleData {
                x: -8,
                y: -8,
                char_flags: 0x0066,
                ext: 2,
            },
            DrawMultipleData {
                x: 8,
                y: -8,
                char_flags: 0x4066,
                ext: 2,
            },
            DrawMultipleData {
                x: -8,
                y: 8,
                char_flags: 0x8066,
                ext: 2,
            },
            DrawMultipleData {
                x: 8,
                y: 8,
                char_flags: 0xc066,
                ext: 2,
            },
            DrawMultipleData {
                x: -8,
                y: -8,
                char_flags: 0x0068,
                ext: 2,
            },
            DrawMultipleData {
                x: 8,
                y: -8,
                char_flags: 0x0068,
                ext: 2,
            },
            DrawMultipleData {
                x: -8,
                y: 8,
                char_flags: 0x0068,
                ext: 2,
            },
            DrawMultipleData {
                x: 8,
                y: 8,
                char_flags: 0x0068,
                ext: 2,
            },
            DrawMultipleData {
                x: -8,
                y: -8,
                char_flags: 0x006a,
                ext: 2,
            },
            DrawMultipleData {
                x: 8,
                y: -8,
                char_flags: 0x406a,
                ext: 2,
            },
            DrawMultipleData {
                x: -8,
                y: 8,
                char_flags: 0x806a,
                ext: 2,
            },
            DrawMultipleData {
                x: 8,
                y: 8,
                char_flags: 0xc06a,
                ext: 2,
            },
            DrawMultipleData {
                x: -8,
                y: -8,
                char_flags: 0x004e,
                ext: 2,
            },
            DrawMultipleData {
                x: 8,
                y: -8,
                char_flags: 0x404e,
                ext: 2,
            },
            DrawMultipleData {
                x: -8,
                y: 8,
                char_flags: 0x804e,
                ext: 2,
            },
            DrawMultipleData {
                x: 8,
                y: 8,
                char_flags: 0xc04e,
                ext: 2,
            },
        ];

        if self.ram[SPRITE_A + k] != 0 {
            if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                self.ram[SPRITE_STATE + k] = 0;
                if !(0..16).any(|j| self.ram[SPRITE_STATE + j] == 4) {
                    self.ram[LOAD_CHR_HALFSLOT_EVEN_ODD] = 1;
                    if !self.sprite_check_if_screen_is_clear() {
                        self.ram[FLAG_BLOCK_LINK_MENU] = 0;
                    }
                }
            } else {
                let base = usize::from((self.ram[SPRITE_DELAY_MAIN + k] >> 2) ^ 7) * 4;
                self.sprite_draw_multiple(k, &SPRITE_EXPLODE_DMD[base..base + 4], None);
            }
            return;
        }
        self.ram[SPRITE_FLOOR + k] = 2;
        if self.ram[SPRITE_DELAY_MAIN + k] == 32 {
            self.ram[SPRITE_STATE + k] = 0;
            self.ram[FLAG_IS_LINK_IMMOBILIZED] = 0;
            if self.ram[PLAYER_NEAR_PIT_STATE] != 2 && self.sprite_check_if_screen_is_clear() {
                if self.ram[SPRITE_TYPE + k] >= 0xd6 {
                    self.ram[MUSIC_CONTROL] = 0x13;
                } else if self.ram[SPRITE_TYPE + k] == 0x7a {
                    self.prepare_dungeon_exit_from_boss_fight();
                } else {
                    self.sprite_explode_spawn_ea(k);
                    return;
                }
            }
        }
        if self.ram[SPRITE_DELAY_MAIN + k] >= 64
            && (self.ram[SPRITE_DELAY_MAIN + k] >= 0x70
                || (self.ram[SPRITE_DELAY_MAIN + k] & 1) == 0)
        {
            self.sprite_active_main_for_death(k);
        }

        let type_ = self.ram[SPRITE_TYPE + k];
        let delay = self.ram[SPRITE_DELAY_MAIN + k];
        if delay >= 0xc0 {
            return;
        }
        if delay & 3 == 0 {
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x0c);
        }
        if delay & if type_ == 0x92 { 3 } else { 7 } != 0 {
            return;
        }

        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x1c, &mut info);
        if j >= 0 {
            const SPRITE_EXPLODE_RANDOM_XY: [i8; 16] =
                [0, 4, 8, 12, -4, -8, -12, 0, 0, 8, 16, 24, -24, -16, -8, 0];
            let j = j as usize;
            self.ram[LOAD_CHR_HALFSLOT_EVEN_ODD] = 11;
            self.ram[SPRITE_STATE + j] = 4;
            self.ram[SPRITE_FLAGS2 + j] = 3;
            self.ram[SPRITE_OAM_FLAGS + j] = 0x0c;
            let random_base = if type_ == 0x92 { 8 } else { 0 };
            let xoff =
                SPRITE_EXPLODE_RANDOM_XY[usize::from(self.get_random_number() & 7) | random_base];
            let yoff =
                SPRITE_EXPLODE_RANDOM_XY[usize::from(self.get_random_number() & 7) | random_base];
            self.sprite_set_x(j, info.r0_x.wrapping_add(xoff as i16 as u16));
            self.sprite_set_y(
                j,
                info.r2_y
                    .wrapping_add(yoff as i16 as u16)
                    .wrapping_sub(u16::from(info.r4_z)),
            );
            self.ram[SPRITE_DELAY_MAIN + j] = 31;
            self.ram[SPRITE_A + j] = 31;
        }
    }

    // void SpriteModule_Fall2(int k) {  // 86fbea
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_module_fall2(&mut self, k: usize) {
        const TAB1: [u8; 32] = [
            13, 13, 13, 13, 13, 13, 13, 12, 12, 12, 12, 12, 3, 3, 3, 3, 3, 2, 2, 2, 2, 1, 1, 1, 1,
            0, 0, 0, 0, 0, 0, 0,
        ];
        const TAB2: [u8; 32] = [
            5, 5, 5, 5, 5, 5, 5, 4, 4, 4, 4, 4, 3, 3, 3, 3, 3, 2, 2, 2, 2, 1, 1, 1, 1, 0, 0, 0, 0,
            0, 0, 0,
        ];
        const TAB3: [u8; 16] = [
            0xff, 0x3f, 0x1f, 0x0f, 0x0f, 7, 3, 1, 0xff, 0x3f, 0x1f, 0x0f, 7, 3, 1, 0,
        ];
        const TAB4: [u8; 4] = [0, 4, 8, 0];

        let mut delay = self.ram[SPRITE_DELAY_MAIN + k];
        if delay == 0 {
            self.ram[SPRITE_STATE + k] = 0;
            self.sprite_manually_set_death_flag_uw(k);
            return;
        }
        if delay >= 0x40 {
            if self.ram[SPRITE_OAM_FLAGS + k] != 5 {
                if ((delay & 7) | self.frame_control_view().submodule() | self.ram[FLAG_UNK1]) == 0
                {
                    self.sprite_sfx_queue_sfx3_with_pan(k, 0x31);
                }
                self.sprite_active_main_for_death(k);
                let Some((x, y, _flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
                    return;
                };
                self.sprite_draw_distress_custom(x, y.wrapping_sub(8), delay.wrapping_add(20));
                return;
            }
            self.ram[SPRITE_DELAY_MAIN + k] = 63;
            delay = 63;
        }
        if delay == 61 {
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x20);
        }
        let j = usize::from(delay >> 1);
        if self.ram[SPRITE_TYPE + k] == 0x26 || self.ram[SPRITE_TYPE + k] == 0x13 {
            self.ram[SPRITE_GRAPHICS + k] = TAB2[j];
            self.sprite_draw_falling_helma_beetle(k);
        } else {
            let mut t = TAB1[j];
            if t < 12 {
                t = t.wrapping_add(TAB4[usize::from(self.ram[SPRITE_D + k] & 3)]);
            }
            self.ram[SPRITE_GRAPHICS + k] = t;
            self.sprite_draw_falling_humanoid(k);
        }
        if (self.ram[FRAME_COUNTER] & TAB3[usize::from(self.ram[SPRITE_DELAY_MAIN + k] >> 3)])
            | self.frame_control_view().submodule()
            != 0
        {
            return;
        }
        self.sprite_check_tile_property(k, 0x68);
        if self.ram[SPRITE_TILETYPE_SPR] != 0x20 {
            self.ram[SPRITE_Y_RECOIL + k] = 0;
            self.ram[SPRITE_X_RECOIL + k] = 0;
        }
        self.ram[SPRITE_Y_VEL + k] = ((self.ram[SPRITE_Y_RECOIL + k] as i8) >> 2) as u8;
        self.ram[SPRITE_X_VEL + k] = ((self.ram[SPRITE_X_RECOIL + k] as i8) >> 2) as u8;
        self.sprite_move_xy(k);
    }

    // bool Sprite_CheckDamageToAndFromLink(int k) {  // 85ab93
    //   Sprite_CheckDamageFromLink(k);
    //   return Sprite_CheckDamageToLink(k);
    // }
    pub(super) fn sprite_check_damage_to_and_from_link(&mut self, k: usize) -> bool {
        self.sprite_check_damage_from_link(k);
        self.sprite_check_damage_to_link(k)
    }

    // void SpriteModule_Carried(int k) {  // 86de83
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_module_carried(&mut self, k: usize) {
        const SPRITE_HELD_Z_FOR_FRAME: [u8; 6] = [3, 2, 1, 3, 2, 1];
        const SPRITE_HELD_X: [i8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, -13, -10, -5, 0, 13, 10, 5, 0];
        const SPRITE_HELD_Z: [u8; 16] =
            [13, 14, 15, 16, 0, 10, 22, 16, 8, 11, 14, 16, 8, 11, 14, 16];

        self.ram[SPRITE_ROOM + k] = self.ram[OVERWORLD_AREA_INDEX_SPRITE];
        if self.ram[SPRITE_UNK3_LOC + k] != 3 {
            if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                self.ram[SPRITE_DELAY_MAIN + k] = if self.ram[SPRITE_C_SPRITE + k] == 6 {
                    8
                } else {
                    4
                };
                self.ram[SPRITE_UNK3_LOC + k] = self.ram[SPRITE_UNK3_LOC + k].wrapping_add(1);
            }
        } else {
            self.ram[SPRITE_FLAGS3 + k] &= !0x10;
        }

        let t = self.ram[SPRITE_DELAY_AUX4 + k].wrapping_sub(1);
        let r0 = u16::from(t < 63 && (t & 2) != 0);
        let j = usize::from(
            self.ram[LINK_DIRECTION_FACING]
                .wrapping_mul(2)
                .wrapping_add(self.ram[SPRITE_UNK3_LOC + k])
                & 0x0f,
        );
        let link_x = self.player_state_view().x();
        let offset = SPRITE_HELD_X[j] as i16 as u16;
        let t0 = u16::from(link_x as u8) + u16::from(offset as u8);
        let t1 = u16::from(t0 as u8) + ((t0 >> 8) & 1) + r0;
        let t2 = u16::from((link_x >> 8) as u8)
            + ((t1 >> 8) & 1)
            + ((t0 >> 8) & 1)
            + u16::from((offset >> 8) as u8);
        self.ram[SPRITE_X_LO + k] = t1 as u8;
        self.ram[SPRITE_X_HI + k] = t2 as u8;
        self.ram[SPRITE_Z + k] = SPRITE_HELD_Z[j];
        let an = if self.ram[LINK_ANIMATION_STEPS] < 6 {
            usize::from(self.ram[LINK_ANIMATION_STEPS])
        } else {
            0
        };
        let z = self
            .player_state_view()
            .z()
            .wrapping_add(1)
            .wrapping_add(u16::from(SPRITE_HELD_Z_FOR_FRAME[an]));
        self.sprite_set_y(
            k,
            self.player_state_view().y().wrapping_add(8).wrapping_sub(z),
        );
        self.ram[SPRITE_FLOOR + k] = self.ram[LINK_IS_ON_LOWER_LEVEL] & 1;

        self.carried_sprite_check_for_throw(k);
        self.sprite_get16_bit_coords(k);
        if self.ram[SPRITE_UNK4_LOC + k] != 11 {
            self.sprite_active_main_for_death(k);
            if self.ram[SPRITE_DELAY_AUX4 + k] == 1 {
                self.ram[SPRITE_STATE + k] = 9;
                self.ram[SPRITE_B + k] = 0;
                self.ram[SPRITE_DELAY_AUX4 + k] = 96;
                self.ram[SPRITE_Z_VEL + k] = 32;
                self.ram[SPRITE_FLAGS3 + k] |= 0x10;
                self.ram[LINK_PICKING_THROW_STATE] = 2;
            }
        } else {
            self.sprite_stunned_main_func1(k);
        }
    }

    // void CarriedSprite_CheckForThrow(int k) {  // 86df6d
    //   ...see sprite.c...
    // }
    pub(super) fn carried_sprite_check_for_throw(&mut self, k: usize) {
        const SPRITE_HELD_THROW_XVEL: [u8; 4] = [0, 0, (-62i8) as u8, 63];
        const SPRITE_HELD_THROW_YVEL: [u8; 4] = [(-62i8) as u8, 63, 0, 0];
        const SPRITE_HELD_THROW_ZVEL: [u8; 4] = [4, 4, 4, 4];

        if self.frame_control_view().main_module() == 14 {
            return;
        }
        if self.ram[PLAYER_NEAR_PIT_STATE] != 2 {
            let t = (self.ram[LINK_AUXILIARY_STATE] & 1)
                | self.ram[LINK_IS_IN_DEEP_WATER]
                | self.ram[LINK_IS_BUNNY_MIRROR]
                | self.ram[LINK_POSE_FOR_ITEM]
                | if self.ram[LINK_DISABLE_SPRITE_DAMAGE] != 0 {
                    0
                } else {
                    self.ram[LINK_INCAPACITATED_TIMER]
                };
            if t == 0 {
                if self.ram[SPRITE_UNK3_LOC + k] != 3
                    || ((self.ram[FILTERED_JOYPAD_H] | self.ram[FILTERED_JOYPAD_L]) & 0x80) == 0
                {
                    return;
                }
                self.ram[FILTERED_JOYPAD_L] &= 0x7f;
            }
        }

        self.sprite_sfx_queue_sfx3_with_pan(k, 0x13);
        self.ram[LINK_PICKING_THROW_STATE] = 2;
        self.ram[SPRITE_STATE + k] = self.ram[SPRITE_UNK4_LOC + k];
        self.ram[SPRITE_Z_VEL + k] = 0;
        self.ram[SPRITE_UNK3_LOC + k] = 0;
        self.ram[SPRITE_FLAGS3 + k] = (self.ram[SPRITE_FLAGS3 + k] & !0x10)
            | (sprite_init_value(SPRITE_INIT_FLAGS3_TABLE, self.ram[SPRITE_TYPE + k]) & 0x10);
        let j = usize::from((self.ram[LINK_DIRECTION_FACING] >> 1) & 3);
        self.ram[SPRITE_X_VEL + k] = SPRITE_HELD_THROW_XVEL[j];
        self.ram[SPRITE_Y_VEL + k] = SPRITE_HELD_THROW_YVEL[j];
        self.ram[SPRITE_Z_VEL + k] = SPRITE_HELD_THROW_ZVEL[j];
        self.ram[SPRITE_DELAY_AUX4 + k] = 0;
    }

    // void SpriteModule_Stunned(int k) {  // 86dffa
    //   SpriteStunned_MainEx(k, false);
    // }
    pub(super) fn sprite_module_stunned(&mut self, k: usize) {
        self.sprite_stunned_main_ex(k, false);
    }

    // void ThrownSprite_TileAndSpriteInteraction(int k) {  // 86e02a
    //   SpriteStunned_MainEx(k, true);
    // }
    pub(super) fn thrown_sprite_tile_and_sprite_interaction(&mut self, k: usize) {
        self.sprite_stunned_main_ex(k, true);
    }

    // void ThrowableScenery_InteractWithSpritesAndTiles(int k) {  // 86e164
    //   Sprite_MoveXY(k);
    //   if (!sprite_E[k])
    //     Sprite_CheckTileCollision(k);
    //   ThrownSprite_TileAndSpriteInteraction(k);
    // }
    pub(super) fn throwable_scenery_interact_with_sprites_and_tiles(&mut self, k: usize) {
        self.sprite_move_xy(k);
        if self.ram[SPRITE_E + k] == 0 {
            self.sprite_check_tile_collision(k);
        }
        self.thrown_sprite_tile_and_sprite_interaction(k);
    }

    // void SpriteStunned_MainEx(int k, bool second_entry) {  // 86dfcf
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_stunned_main_ex(&mut self, k: usize, second_entry: bool) {
        if !second_entry {
            self.sprite_draw_ripple_if_in_water(k);
            self.sprite_stunned_main_func1(k);
            if self.sprite_return_if_paused(k) {
                return;
            }
            if self.ram[SPRITE_F + k] != 0 {
                if sign8(self.ram[SPRITE_F + k]) {
                    self.ram[SPRITE_F + k] = 0;
                }
                self.ram[SPRITE_Y_VEL + k] = 0;
                self.ram[SPRITE_X_VEL + k] = 0;
            }
            if self.ram[SPRITE_DELAY_MAIN + k] < 0x20 {
                self.sprite_check_damage_from_link(k);
            }
            if self.sprite_return_if_recoiling(k) {
                return;
            }
            self.sprite_move_xy(k);
            if self.ram[SPRITE_E + k] == 0 {
                self.sprite_check_tile_collision(k);
                if self.ram[SPRITE_STATE + k] == 0 {
                    return;
                }
            }
        }

        if (second_entry || self.ram[SPRITE_E + k] == 0)
            && (self.ram[SPRITE_WALLCOLL + k] & 0x0f) != 0
        {
            self.sprite_apply_ricochet(k);
            if self.ram[SPRITE_STATE + k] == 11 {
                self.sprite_sfx_queue_sfx2_with_pan(k, 5);
            }
        }
        self.sprite_check_tile_property(k, 0x68);

        if sprite_init_value(SPRITE_INIT_FLAGS3_TABLE, self.ram[SPRITE_TYPE + k]) & 0x10 != 0 {
            self.ram[SPRITE_FLAGS3 + k] |= 0x10;
            if self.ram[SPRITE_TILETYPE_SPR] == 32 {
                self.ram[SPRITE_FLAGS3 + k] &= !0x10;
            }
        }
        self.sprite_move_z(k);
        self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_Z_VEL + k].wrapping_sub(2);
        if self.ram[SPRITE_Z + k].wrapping_sub(1) >= 0xf0 {
            self.ram[SPRITE_Z + k] = 0;
            if self.ram[SPRITE_TYPE + k] == 0xe8
                && (self.ram[SPRITE_Z_VEL + k].wrapping_sub(0xe8) as i8).is_negative()
            {
                self.ram[SPRITE_STATE + k] = 6;
                self.ram[SPRITE_DELAY_MAIN + k] = 8;
                self.ram[SPRITE_FLAGS2 + k] = 3;
                return;
            }

            self.throwable_scenery_transmute_if_valid(k);
            let mut tile = self.ram[SPRITE_TILETYPE_SPR];
            if self.ram[SPRITE_TILETYPE_SPR] == 32 {
                tile = self.ram[SPRITE_FLAGS_SPRITE + k] >> 1;
                if self.ram[SPRITE_FLAGS_SPRITE + k] & 1 == 0 {
                    self.sprite_func8(k);
                    return;
                }
            }
            if tile == 9 {
                let z_vel = self.ram[SPRITE_Z_VEL + k];
                self.ram[SPRITE_Z_VEL + k] = 0;
                if (z_vel.wrapping_sub(0xf0) as i8).is_negative() {
                    let mut info = SpriteSpawnInfo::default();
                    let j = self.sprite_spawn_dynamically(k, 0xec, &mut info);
                    if j >= 0 {
                        let j = j as usize;
                        self.sprite_set_spawned_coordinates(j, &info);
                        self.sprite_func22(j);
                    }
                }
            } else if tile == 8 {
                if self.ram[SPRITE_TYPE + k] == 0xd2 || (self.get_random_number() & 1) != 0 {
                    self.sprite_spawn_leaping_fish(k);
                }
                self.sprite_func22(k);
                return;
            }

            let z_vel = self.ram[SPRITE_Z_VEL + k];
            if (z_vel as i8).is_negative() {
                let bounced = z_vel.wrapping_neg() >> 1;
                self.ram[SPRITE_Z_VEL + k] = if bounced < 9 { 0 } else { bounced };
            }
            self.ram[SPRITE_X_VEL + k] = ((self.ram[SPRITE_X_VEL + k] as i8) >> 1) as u8;
            if self.ram[SPRITE_X_VEL + k] == 0xff {
                self.ram[SPRITE_X_VEL + k] = 0;
            }
            self.ram[SPRITE_Y_VEL + k] = ((self.ram[SPRITE_Y_VEL + k] as i8) >> 1) as u8;
            if self.ram[SPRITE_Y_VEL + k] == 0xff {
                self.ram[SPRITE_Y_VEL + k] = 0;
            }
        }
        if self.ram[SPRITE_STATE + k] != 11 || self.ram[SPRITE_UNK5_LOC + k] != 0 {
            if self.sprite_return_if_lifted(k) {
                return;
            }
            if self.ram[SPRITE_TYPE + k] != 0x4a {
                self.thrown_sprite_check_damage_to_sprites(k);
            }
        }
    }

    // void SpriteStunned_Main_Func1(int k) {  // 86e2ba
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_stunned_main_func1(&mut self, k: usize) {
        const SPRITE_STUNNED_MAIN_FUNC1_MASKS: [u8; 7] = [0x7f, 0x0f, 3, 1, 0, 0, 0];
        const SPARKLE_GARNISH_XY: [i8; 4] = [-4, 12, 3, 8];

        if std::env::var_os("ZELDA3_REPLAY_GARNISH_TRACE").is_some() {
            eprintln!(
                "R stunned-before fc=0x{:02x} rng=0x{:02x} k={} type=0x{:02x} state=0x{:02x} unk5=0x{:02x} delay=0x{:02x} stunned=0x{:02x} give=0x{:02x} z=0x{:02x} zv=0x{:02x} ai=0x{:02x}",
                self.ram[FRAME_COUNTER],
                self.ram[RNG_SEED],
                k,
                self.ram[SPRITE_TYPE + k],
                self.ram[SPRITE_STATE + k],
                self.ram[SPRITE_UNK5_LOC + k],
                self.ram[SPRITE_DELAY_MAIN + k],
                self.ram[SPRITE_STUNNED + k],
                self.ram[SPRITE_GIVE_DAMAGE_SPRITE + k],
                self.ram[SPRITE_Z + k],
                self.ram[SPRITE_Z_VEL + k],
                self.ram[SPRITE_AI_STATE + k],
            );
        }
        self.sprite_active_main_for_death(k);
        if std::env::var_os("ZELDA3_REPLAY_GARNISH_TRACE").is_some() {
            eprintln!(
                "R stunned-after-active fc=0x{:02x} rng=0x{:02x} k={} type=0x{:02x} state=0x{:02x} unk5=0x{:02x} delay=0x{:02x} stunned=0x{:02x} give=0x{:02x} z=0x{:02x} zv=0x{:02x} ai=0x{:02x}",
                self.ram[FRAME_COUNTER],
                self.ram[RNG_SEED],
                k,
                self.ram[SPRITE_TYPE + k],
                self.ram[SPRITE_STATE + k],
                self.ram[SPRITE_UNK5_LOC + k],
                self.ram[SPRITE_DELAY_MAIN + k],
                self.ram[SPRITE_STUNNED + k],
                self.ram[SPRITE_GIVE_DAMAGE_SPRITE + k],
                self.ram[SPRITE_Z + k],
                self.ram[SPRITE_Z_VEL + k],
                self.ram[SPRITE_AI_STATE + k],
            );
        }
        if self.ram[SPRITE_UNK5_LOC + k] != 0 {
            if self.ram[SPRITE_DELAY_MAIN + k] < 32 {
                self.ram[SPRITE_OAM_FLAGS + k] = (self.ram[SPRITE_OAM_FLAGS + k] & 0xf1) | 4;
            }
            let t = (((k as u8) << 4) ^ self.ram[FRAME_COUNTER])
                | self.frame_control_view().submodule();
            let mask =
                SPRITE_STUNNED_MAIN_FUNC1_MASKS[usize::from(self.ram[SPRITE_DELAY_MAIN + k] >> 4)];
            if std::env::var_os("ZELDA3_REPLAY_GARNISH_TRACE").is_some() {
                eprintln!(
                    "R stunned-sparkle-check fc=0x{:02x} k={} t=0x{:02x} mask=0x{:02x} delay=0x{:02x}",
                    self.ram[FRAME_COUNTER],
                    k,
                    t,
                    mask,
                    self.ram[SPRITE_DELAY_MAIN + k],
                );
            }
            if t & mask != 0 {
                return;
            }
            let x = SPARKLE_GARNISH_XY[usize::from(self.get_random_number() & 3)] as i16 as u16;
            let y = SPARKLE_GARNISH_XY[usize::from(self.get_random_number() & 3)] as i16 as u16;
            self.sprite_garnish_spawn_sparkle(k, x, y);
            return;
        }

        if (self.ram[FRAME_COUNTER] & 1)
            | self.frame_control_view().submodule()
            | self.ram[FLAG_UNK1]
            != 0
        {
            return;
        }
        let t = self.ram[SPRITE_STUNNED + k];
        if t != 0 {
            self.ram[SPRITE_STUNNED + k] = self.ram[SPRITE_STUNNED + k].wrapping_sub(1);
            if t < 0x38 {
                self.ram[SPRITE_X_VEL + k] = if (t & 1) != 0 { (-8i8) as u8 } else { 8 };
                self.sprite_move_x(k);
            }
            return;
        }
        self.ram[SPRITE_STATE + k] = 9;
        self.ram[SPRITE_X_RECOIL + k] = 0;
        self.ram[SPRITE_Y_RECOIL + k] = 0;
    }

    // void Sprite_SpawnLeapingFish(int k) {  // 86e286
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_spawn_leaping_fish(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xd2, &mut info);
        if j < 0 {
            return;
        }
        let j = j as usize;
        self.sprite_set_spawned_coordinates(j, &info);
        self.ram[SPRITE_AI_STATE_SPRITE + j] = 2;
        self.ram[SPRITE_DELAY_MAIN + j] = 48;
        if self.ram[SPRITE_TYPE + k] == 0xd2 {
            self.ram[SPRITE_A + j] = 0xd2;
        }
    }

    // bool Sprite_HandleDraggingByAncilla(int k) {  // 86cf64
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_handle_dragging_by_ancilla(&mut self, k: usize) -> bool {
        let mut j = self.ram[SPRITE_B + k];
        if j == 0 {
            return false;
        }
        j = j.wrapping_sub(1);
        let j = usize::from(j);
        if self.ram[ANCILLA_TYPE + j] == 0 {
            self.sprite_handle_absorption_by_player(k);
        } else {
            self.ram[SPRITE_X_LO + k] = self.ram[ANCILLA_X_LO_SPRITE + j];
            self.ram[SPRITE_X_HI + k] = self.ram[ANCILLA_X_HI + j];
            self.ram[SPRITE_Y_LO + k] = self.ram[ANCILLA_Y_LO_SPRITE + j];
            self.ram[SPRITE_Y_HI + k] = self.ram[ANCILLA_Y_HI + j];
            self.ram[SPRITE_Z + k] = 0;
        }
        true
    }

    // void Sprite_CheckAbsorptionByPlayer(int k) {  // 86d116
    //   if (!sprite_delay_aux4[k] && Sprite_CheckDamageToPlayer_1(k))
    //     Sprite_HandleAbsorptionByPlayer(k);
    // }
    pub(super) fn sprite_check_absorption_by_player(&mut self, k: usize) {
        if self.ram[SPRITE_DELAY_AUX4 + k] == 0 && self.sprite_check_damage_to_player_1(k) {
            self.sprite_handle_absorption_by_player(k);
        }
    }

    // void Sprite_HandleAbsorptionByPlayer(int k) {  // 86d13c
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_handle_absorption_by_player(&mut self, k: usize) {
        const ABSORPTION_SFX: [u8; 15] = [
            0x0b, 0x0a, 0x0a, 0x0a, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x2f, 0x2f,
            0x0b,
        ];
        const RUPEES_ABSORPTION: [u16; 3] = [1, 5, 20];
        const BOMBS_ABSORPTION: [u8; 3] = [1, 4, 8];
        const ABSORB_BIG_KEY: [u16; 2] = [0x4000, 0x2000];

        self.ram[SPRITE_STATE + k] = 0;
        let t = self.ram[SPRITE_TYPE + k].wrapping_sub(0xd8);
        if usize::from(t) < ABSORPTION_SFX.len() {
            self.sprite_sfx_queue_sfx3_with_pan(k, ABSORPTION_SFX[usize::from(t)]);
        }
        match t {
            0 => self.ram[LINK_HEARTS_FILLER] = self.ram[LINK_HEARTS_FILLER].wrapping_add(8),
            1..=3 => {
                let rupees = read_le_u16(&self.ram, LINK_RUPEES_GOAL)
                    .wrapping_add(RUPEES_ABSORPTION[usize::from(t - 1)]);
                write_le_u16(&mut self.ram, LINK_RUPEES_GOAL, rupees);
            }
            4..=6 => {
                self.ram[LINK_BOMB_FILLER] =
                    self.ram[LINK_BOMB_FILLER].wrapping_add(BOMBS_ABSORPTION[usize::from(t - 4)]);
            }
            7 => self.ram[LINK_MAGIC_FILLER] = self.ram[LINK_MAGIC_FILLER].wrapping_add(0x10),
            8 => self.ram[LINK_MAGIC_FILLER] = 0x80,
            9 => {
                let arrows = if self.ram[SPRITE_HEAD_DIR + k] == 0 {
                    5
                } else {
                    self.ram[SPRITE_HEAD_DIR + k]
                };
                self.ram[LINK_ARROW_FILLER] = self.ram[LINK_ARROW_FILLER].wrapping_add(arrows);
            }
            10 => self.ram[LINK_ARROW_FILLER] = self.ram[LINK_ARROW_FILLER].wrapping_add(10),
            11 => {
                self.sprite_sfx_queue_sfx2_with_pan(k, 0x31);
                self.ram[LINK_HEARTS_FILLER] = self.ram[LINK_HEARTS_FILLER].wrapping_add(56);
            }
            12 => {
                self.ram[LINK_NUM_KEYS] = self.ram[LINK_NUM_KEYS].wrapping_add(1);
                self.finish_absorbed_key_or_big_key(k, &ABSORB_BIG_KEY);
            }
            13 => {
                self.ram[ITEM_RECEIPT_METHOD] = 0;
                self.link_receive_item(0x32, 0);
                self.finish_absorbed_key_or_big_key(k, &ABSORB_BIG_KEY);
            }
            14 => {
                self.ram[LINK_SHIELD_TYPE] = self.ram[SPRITE_SUBTYPE + k];
                if self.read_u32_ram(ENHANCED_FEATURES0) & 4096 != 0 {
                    self.Palette_Load_Shield();
                }
            }
            _ => {}
        }
    }

    fn finish_absorbed_key_or_big_key(&mut self, k: usize, absorb_big_key: &[u16; 2]) {
        self.ram[SPRITE_N + k] = self.ram[SPRITE_SUBTYPE + k];
        let idx = usize::from(self.ram[SPRITE_DIE_ACTION + k]);
        let bits = read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) | absorb_big_key[idx];
        write_le_u16(&mut self.ram, DUNG_SAVEGAME_STATE_BITS, bits);
        self.sprite_manually_set_death_flag_uw(k);
    }

    // uint8 Sprite_CheckDamageFromLink(int k) {  // 86f2b4
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_check_damage_from_link(&mut self, k: usize) -> u8 {
        if (self.ram[SPRITE_HIT_TIMER + k] & 0x80) != 0
            || self.ram[SPRITE_FLOOR + k] != self.ram[LINK_IS_ON_LOWER_LEVEL]
            || self.ram[PLAYER_OAM_Y_OFFSET] == 0x80
        {
            return 0;
        }

        let mut hb = empty_sprite_hit_box();
        self.player_setup_action_hit_box(&mut hb);
        self.sprite_setup_hit_box(k, &mut hb);
        let overlap = self.check_if_hit_boxes_overlap(&hb);
        if std::env::var_os("ZELDA3_TRACE_DAMAGE_FROM_LINK").is_some()
            && self.world_state_view().dungeon_room() == 0x00a8
            && self.ram[SPRITE_TYPE + k] == 0xa7
            && k == 2
        {
            eprintln!(
                "R damage-from-link fc={} k={} overlap={} type=0x{:02x} dmgtype=0x{:02x} link=0x{:04x},0x{:04x} spr=0x{:04x},0x{:04x} hb={:02x}/{:02x},{:02x}/{:02x} sz={:02x},{:02x} sprhb={:02x}/{:02x},{:02x}/{:02x} sprsz={:02x},{:02x} item=0x{:02x} sword_delay=0x{:02x}",
                self.ram[FRAME_COUNTER],
                k,
                overlap,
                self.ram[SPRITE_TYPE + k],
                self.ram[DAMAGE_TYPE_DETERMINER_SPRITE],
                self.player_state_view().x(),
                self.player_state_view().y(),
                self.sprite_get_x(k),
                self.sprite_get_y(k),
                hb.r0_xlo,
                hb.r8_xhi,
                hb.r1_ylo,
                hb.r9_yhi,
                hb.r2,
                hb.r3,
                hb.r4_spr_xlo,
                hb.r10_spr_xhi,
                hb.r5_spr_ylo,
                hb.r11_spr_yhi,
                hb.r6_spr_xsize,
                hb.r7_spr_ysize,
                self.ram[LINK_ITEM_IN_HAND],
                self.ram[LINK_SWORD_DELAY_TIMER],
            );
        }
        if !overlap {
            return 0;
        }

        self.ram[SET_WHEN_DAMAGING_ENEMIES] = 0;
        if (self.ram[LINK_POSITION_MODE] & 0x10) != 0 {
            return K_CHECK_DAMAGE_FROM_PLAYER_CARRY | K_CHECK_DAMAGE_FROM_PLAYER_NE;
        }

        if (self.ram[LINK_ITEM_IN_HAND] & 10) != 0 {
            if self.ram[SPRITE_TYPE + k] >= 0xd6 {
                return 0;
            }
            if self.ram[SPRITE_STATE + k] == 11 && self.ram[SPRITE_UNK5_LOC + k] != 0 {
                self.ram[SPRITE_STATE + k] = 2;
                self.ram[SPRITE_DELAY_MAIN + k] = 32;
                self.ram[SPRITE_FLAGS2 + k] = (self.ram[SPRITE_FLAGS2 + k] & 0xe0) | 3;
                self.sprite_sfx_queue_sfx2_with_pan(k, 0x1f);
                return K_CHECK_DAMAGE_FROM_PLAYER_CARRY | K_CHECK_DAMAGE_FROM_PLAYER_NE;
            }
        }

        let ty = self.ram[SPRITE_TYPE + k];
        if ty == 0x7b {
            if !sign8(self.ram[BUTTON_B_FRAMES].wrapping_sub(9)) {
                return 0;
            }
        } else if ty == 9 {
            if self.ram[SPRITE_A + k] == 0 {
                self.sprite_apply_recoil_to_link(k, 48);
                self.ram[SET_WHEN_DAMAGING_ENEMIES] = 144;
                self.ram[LINK_INCAPACITATED_TIMER] = 16;
                self.sprite_sfx_queue_sfx2_with_pan(k, 0x21);
                self.ram[SPRITE_DELAY_AUX1_SPRITE + k] = 48;
                self.ram[SOUND_EFFECT_2] = self.sprite_calculate_sfx_pan(k)
                    | if self.read_u32_ram(ENHANCED_FEATURES0) & 4096 != 0 {
                        0x32
                    } else {
                        0
                    };
                self.link_place_weapon_tink();
                return K_CHECK_DAMAGE_FROM_PLAYER_CARRY;
            }
        } else if ty == 0x92 {
            if self.ram[SPRITE_C + k] >= 3 {
                self.sprite_apply_recoil_to_link(k, 32);
                self.ram[SET_WHEN_DAMAGING_ENEMIES] = 144;
                self.ram[LINK_INCAPACITATED_TIMER] = 16;
            } else {
                return self.sprite_check_damage_from_link_getting_out(k);
            }
        } else if ty == 0x26 || ty == 0x13 || ty == 2 {
            const SPRITE_DAMAGE_TAB3: [u8; 4] = [4, 6, 0, 2];
            let cond = (ty == 0x13
                && SPRITE_DAMAGE_TAB3[usize::from(self.ram[SPRITE_D + k] & 3)]
                    == self.ram[LINK_DIRECTION_FACING])
                || ty == 2;
            self.sprite_attempt_zap_damage(k);
            self.sprite_apply_recoil_to_link(k, 32);
            self.ram[SET_WHEN_DAMAGING_ENEMIES] = 16;
            self.ram[LINK_INCAPACITATED_TIMER] = 16;
            if cond {
                self.ram[SPRITE_HIT_TIMER + k] = 0;
                self.link_place_weapon_tink();
            }
            return 0;
        } else if matches!(ty, 0xcb | 0xcd | 0xcc | 0xd6 | 0xd7 | 0xce | 0x54) {
            self.sprite_apply_recoil_to_link(k, 32);
            self.ram[SET_WHEN_DAMAGING_ENEMIES] = 144;
            self.ram[LINK_INCAPACITATED_TIMER] = 16;
        }

        if (self.ram[SPRITE_DEFL_BITS + k] & 4) == 0 {
            self.sprite_attempt_zap_damage(k);
            return K_CHECK_DAMAGE_FROM_PLAYER_CARRY;
        }

        self.sprite_check_damage_from_link_getting_out(k)
    }

    fn sprite_check_damage_from_link_getting_out(&mut self, k: usize) -> u8 {
        if self.ram[SET_WHEN_DAMAGING_ENEMIES] == 0 {
            self.sprite_apply_recoil_to_link(k, 4);
            self.ram[LINK_INCAPACITATED_TIMER] = 16;
            self.ram[SET_WHEN_DAMAGING_ENEMIES] = 16;
        }
        self.link_place_weapon_tink();
        K_CHECK_DAMAGE_FROM_PLAYER_CARRY
    }

    // bool Sprite_CheckDamageToLink(int k) {  // 86f145
    //   if (link_disable_sprite_damage)
    //     return false;
    //   return Sprite_CheckDamageToPlayer_1(k);
    // }
    pub(super) fn sprite_check_damage_to_link(&mut self, k: usize) -> bool {
        self.ram[LINK_DISABLE_SPRITE_DAMAGE] == 0 && self.sprite_check_damage_to_player_1(k)
    }

    // bool Sprite_CheckDamageToPlayer_1(int k) {  // 86f14a
    //   if ((k ^ frame_counter) & 3 | sprite_hit_timer[k])
    //     return false;
    //   return Sprite_CheckDamageToLink_same_layer(k);
    // }
    pub(super) fn sprite_check_damage_to_player_1(&mut self, k: usize) -> bool {
        (((k as u8) ^ self.ram[FRAME_COUNTER]) & 3) == 0
            && self.ram[SPRITE_HIT_TIMER + k] == 0
            && self.sprite_check_damage_to_link_same_layer(k)
    }

    // bool Sprite_CheckDamageToLink_same_layer(int k) {  // 86f154
    //   if (link_is_on_lower_level != sprite_floor[k])
    //     return false;
    //   return Sprite_CheckDamageToLink_ignore_layer(k);
    // }
    pub(super) fn sprite_check_damage_to_link_same_layer(&mut self, k: usize) -> bool {
        self.ram[LINK_IS_ON_LOWER_LEVEL] == self.ram[SPRITE_FLOOR + k]
            && self.sprite_check_damage_to_link_ignore_layer(k)
    }

    // bool Sprite_CheckDamageToLink_ignore_layer(int k) {  // 86f15c
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_check_damage_to_link_ignore_layer(&mut self, k: usize) -> bool {
        let carry = if self.ram[SPRITE_FLAGS4 + k] != 0 {
            let mut hitbox = empty_sprite_hit_box();
            self.link_setup_hit_box(&mut hitbox);
            if (0xd8..=0xe6).contains(&self.ram[SPRITE_TYPE + k])
                && self.read_u32_ram(ENHANCED_FEATURES0) & FEATURES0_COLLECT_ITEMS_WITH_SWORD_SPRITE
                    != 0
            {
                self.link_update_hit_box_with_sword(&mut hitbox);
            }
            self.sprite_setup_hit_box(k, &mut hitbox);
            self.check_if_hit_boxes_overlap(&hitbox)
        } else {
            self.sprite_setup_hit_box00(k)
        };

        if sign8(self.ram[SPRITE_FLAGS2 + k]) {
            return carry;
        }
        if std::env::var_os("ZELDA3_TRACE_SPRITE_DAMAGE").is_some()
            && self.ram[PLAYER_IS_INDOORS] != 0
            && self.world_state_view().dungeon_room() == 0x00a8
        {
            eprintln!(
                "R sprite-ignore-layer fc={} k={} type=0x{:02x} carry={} flags2=0x{:02x} flags4=0x{:02x} flags5=0x{:02x} shield=0x{:02x} bunny=0x{:02x} statebits=0x{:02x} facing=0x{:02x} d=0x{:02x} aux=0x{:02x} link=0x{:04x},0x{:04x} cur=0x{:04x},0x{:04x} z=0x{:02x}",
                self.ram[FRAME_COUNTER],
                k,
                self.ram[SPRITE_TYPE + k],
                carry,
                self.ram[SPRITE_FLAGS2 + k],
                self.ram[SPRITE_FLAGS4 + k],
                self.ram[SPRITE_FLAGS5 + k],
                self.ram[LINK_SHIELD_TYPE],
                self.ram[LINK_IS_BUNNY_MIRROR],
                self.ram[LINK_STATE_BITS],
                self.ram[LINK_DIRECTION_FACING],
                self.ram[SPRITE_D + k],
                self.ram[LINK_AUXILIARY_STATE],
                self.player_state_view().x(),
                self.player_state_view().y(),
                read_le_u16(&self.ram, CUR_SPRITE_X),
                read_le_u16(&self.ram, CUR_SPRITE_Y),
                self.ram[SPRITE_Z + k],
            );
        }
        if !carry || self.ram[LINK_AUXILIARY_STATE] != 0 {
            return false;
        }

        const SPRITE_DAMAGE_TAB2: [u8; 4] = [6, 4, 0, 0];
        const SPRITE_DAMAGE_TAB3: [u8; 4] = [4, 6, 0, 2];
        if self.ram[LINK_IS_BUNNY_MIRROR] == 0
            && !sign8(self.ram[LINK_STATE_BITS])
            && (self.ram[SPRITE_FLAGS5 + k] & 0x20) != 0
            && self.ram[LINK_SHIELD_TYPE] != 0
        {
            self.ram[SPRITE_STATE + k] = 0;
            let t = if self.ram[BUTTON_B_FRAMES] != 0 {
                SPRITE_DAMAGE_TAB2[usize::from((self.ram[LINK_DIRECTION_FACING] >> 1) & 3)]
            } else {
                self.ram[LINK_DIRECTION_FACING]
            };
            if t == SPRITE_DAMAGE_TAB3[usize::from(self.ram[SPRITE_D + k] & 3)] {
                self.sprite_sfx_queue_sfx2_with_pan(k, 6);
                self.sprite_place_rupulse_spark_2(k);
                match self.ram[SPRITE_TYPE + k] {
                    0x95 => {
                        self.sprite_sfx_queue_sfx3_with_pan(k, 0x26);
                        return false;
                    }
                    0x9b => {
                        self.sprite_invert_xy_speeds(k);
                        self.ram[SPRITE_D + k] ^= 1;
                        self.ram[SPRITE_AI_STATE_SPRITE + k] =
                            self.ram[SPRITE_AI_STATE_SPRITE + k].wrapping_add(1);
                        self.ram[SPRITE_STATE + k] = 9;
                        return false;
                    }
                    0x1b => {
                        self.sprite_schedule_for_breakage(k);
                        return false;
                    }
                    0x0c => {
                        self.sprite_func3(k);
                        return true;
                    }
                    _ => return false,
                }
            }
        }

        self.sprite_attempt_damage_to_link_plus_recoil(k);
        if self.ram[SPRITE_TYPE + k] == 0x0c {
            self.sprite_func3(k);
        }
        true
    }

    // void Sprite_AttemptDamageToLinkWithCollisionCheck(int k) {  // 86f3ca
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_attempt_damage_to_link_with_collision_check(&mut self, k: usize) {
        if (((k as u8) ^ self.ram[FRAME_COUNTER]) & 1) != 0 {
            return;
        }
        let mut hb = empty_sprite_hit_box();
        self.sprite_do_hit_boxes_fast(k, &mut hb);
        self.link_setup_hit_box_conditional(&mut hb);
        let overlap = self.check_if_hit_boxes_overlap(&hb);
        if std::env::var_os("ZELDA3_TRACE_SPRITE_DAMAGE").is_some()
            && self.ram[PLAYER_IS_INDOORS] != 0
            && self.world_state_view().dungeon_room() == 0x00a8
        {
            eprintln!(
                "R sprite-damage-check fc={} k={} type=0x{:02x} st=0x{:02x} bump=0x{:02x} link=0x{:04x},0x{:04x} overlap={} blink=0x{:02x} disable=0x{:02x} aux=0x{:02x} incap=0x{:02x} hp=0x{:02x}",
                self.ram[FRAME_COUNTER],
                k,
                self.ram[SPRITE_TYPE + k],
                self.ram[SPRITE_STATE + k],
                self.ram[SPRITE_BUMP_DAMAGE + k],
                self.player_state_view().x(),
                self.player_state_view().y(),
                overlap,
                self.ram[COUNTDOWN_FOR_BLINK],
                self.ram[LINK_DISABLE_SPRITE_DAMAGE],
                self.ram[LINK_AUXILIARY_STATE],
                self.ram[LINK_INCAPACITATED_TIMER],
                self.ram[LINK_HEALTH_CURRENT],
            );
        }
        if overlap {
            self.sprite_attempt_damage_to_link_plus_recoil(k);
        }
    }

    // void Guard_ParrySwordAttacks(int k) {  // 86eb5e
    pub(super) fn guard_parry_sword_attacks(&mut self, k: usize) {
        const FUNC1_TAB: [u8; 8] = [15, 15, 24, 15, 15, 19, 15, 15];
        const FUNC1_TAB2: [u8; 8] = [6, 6, 6, 12, 6, 6, 6, 15];

        if self.ram[LINK_IS_ON_LOWER_LEVEL] != self.ram[SPRITE_FLOOR + k]
            || (self.ram[LINK_INCAPACITATED_TIMER] | self.ram[LINK_AUXILIARY_STATE]) != 0
            || sign8(self.ram[SPRITE_HIT_TIMER + k])
        {
            return;
        }
        let mut hb = empty_sprite_hit_box();
        self.sprite_do_hit_boxes_fast(k, &mut hb);
        if (self.ram[LINK_POSITION_MODE] & 0x10) != 0 || self.ram[PLAYER_OAM_Y_OFFSET] == 0x80 {
            self.sprite_attempt_damage_to_link_with_collision_check(k);
            return;
        }
        self.player_setup_action_hit_box(&mut hb);
        let button_neg = sign8(self.ram[BUTTON_B_FRAMES]);
        let action_overlap = self.check_if_hit_boxes_overlap(&hb);
        if std::env::var_os("ZELDA3_TRACE_GUARD_PARRY").is_some()
            && self.world_state_view().dungeon_room() == 0x00a8
            && self.ram[SPRITE_TYPE + k] == 0xa7
            && k == 2
        {
            eprintln!(
                "R guard-parry action fc={} k={} button=0x{:02x} neg={} overlap={} link=0x{:04x},0x{:04x} spr=0x{:04x},0x{:04x} hb={:02x}/{:02x},{:02x}/{:02x} sz={:02x},{:02x} sprhb={:02x}/{:02x},{:02x}/{:02x} sprsz={:02x},{:02x}",
                self.ram[FRAME_COUNTER],
                k,
                self.ram[BUTTON_B_FRAMES],
                button_neg,
                action_overlap,
                self.player_state_view().x(),
                self.player_state_view().y(),
                self.sprite_get_x(k),
                self.sprite_get_y(k),
                hb.r0_xlo,
                hb.r8_xhi,
                hb.r1_ylo,
                hb.r9_yhi,
                hb.r2,
                hb.r3,
                hb.r4_spr_xlo,
                hb.r10_spr_xhi,
                hb.r5_spr_ylo,
                hb.r11_spr_yhi,
                hb.r6_spr_xsize,
                hb.r7_spr_ysize,
            );
        }
        if button_neg || !action_overlap {
            self.sprite_setup_hit_box(k, &mut hb);
            let body_overlap = self.check_if_hit_boxes_overlap(&hb);
            if std::env::var_os("ZELDA3_TRACE_GUARD_PARRY").is_some()
                && self.world_state_view().dungeon_room() == 0x00a8
                && self.ram[SPRITE_TYPE + k] == 0xa7
                && k == 2
            {
                eprintln!(
                    "R guard-parry body fc={} k={} overlap={} hb={:02x}/{:02x},{:02x}/{:02x} sz={:02x},{:02x} sprhb={:02x}/{:02x},{:02x}/{:02x} sprsz={:02x},{:02x}",
                    self.ram[FRAME_COUNTER],
                    k,
                    body_overlap,
                    hb.r0_xlo,
                    hb.r8_xhi,
                    hb.r1_ylo,
                    hb.r9_yhi,
                    hb.r2,
                    hb.r3,
                    hb.r4_spr_xlo,
                    hb.r10_spr_xhi,
                    hb.r5_spr_ylo,
                    hb.r11_spr_yhi,
                    hb.r6_spr_xsize,
                    hb.r7_spr_ysize,
                );
            }
            if !body_overlap {
                self.sprite_attempt_damage_to_link_with_collision_check(k);
            } else {
                self.sprite_attempt_zap_damage(k);
            }
            return;
        }
        if self.ram[SPRITE_TYPE + k] != 0x6a {
            let j = usize::from(self.get_random_number() & 7);
            self.ram[SPRITE_F + k] = FUNC1_TAB[j];
        }
        let j = usize::from(self.get_random_number() & 7);
        self.ram[LINK_INCAPACITATED_TIMER] = FUNC1_TAB2[j];
        let fast_sword = sign8(self.ram[BUTTON_B_FRAMES].wrapping_sub(9));
        let pt = self.sprite_project_speed_towards_link(k, if fast_sword { 32 } else { 24 });
        self.ram[SPRITE_X_RECOIL + k] = 0u8.wrapping_sub(pt.x);
        self.ram[SPRITE_Y_RECOIL + k] = 0u8.wrapping_sub(pt.y);
        self.sprite_apply_recoil_to_link(k, if fast_sword { 8 } else { 16 });
        self.link_place_weapon_tink();
        self.ram[SET_WHEN_DAMAGING_ENEMIES] = 0x90;
    }

    // void Sprite_AttemptDamageToLinkPlusRecoil(int k) {  // 86f3db
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_attempt_damage_to_link_plus_recoil(&mut self, k: usize) {
        if std::env::var_os("ZELDA3_TRACE_SPRITE_DAMAGE").is_some()
            && self.ram[PLAYER_IS_INDOORS] != 0
            && self.world_state_view().dungeon_room() == 0x00a8
        {
            eprintln!(
                "R sprite-damage-plus entry fc={} k={} type=0x{:02x} blink=0x{:02x} disable=0x{:02x} aux=0x{:02x} incap=0x{:02x} vx=0x{:02x} vy=0x{:02x}",
                self.ram[FRAME_COUNTER],
                k,
                self.ram[SPRITE_TYPE + k],
                self.ram[COUNTDOWN_FOR_BLINK],
                self.ram[LINK_DISABLE_SPRITE_DAMAGE],
                self.ram[LINK_AUXILIARY_STATE],
                self.ram[LINK_INCAPACITATED_TIMER],
                self.ram[LINK_ACTUAL_VEL_X],
                self.ram[LINK_ACTUAL_VEL_Y],
            );
        }
        if (self.ram[COUNTDOWN_FOR_BLINK] | self.ram[LINK_DISABLE_SPRITE_DAMAGE]) != 0 {
            return;
        }
        const PLAYER_DAMAGES: [u8; 30] = [
            2, 1, 1, 4, 4, 4, 0, 0, 0, 8, 4, 2, 8, 8, 8, 16, 8, 4, 32, 16, 8, 32, 24, 16, 24, 16,
            8, 64, 48, 24,
        ];
        self.ram[LINK_INCAPACITATED_TIMER] = 19;
        self.sprite_apply_recoil_to_link(k, 24);
        self.ram[LINK_AUXILIARY_STATE] = 1;
        let idx = 3 * usize::from(self.ram[SPRITE_BUMP_DAMAGE + k] & 0x0f)
            + usize::from(self.ram[LINK_ARMOR]);
        self.ram[LINK_GIVE_DAMAGE] = PLAYER_DAMAGES[idx];
        if self.ram[SPRITE_TYPE + k] == 0x61 && self.ram[SPRITE_C_SPRITE + k] != 0 {
            self.ram[LINK_ACTUAL_VEL_X] = self.ram[SPRITE_X_VEL + k].wrapping_mul(2);
            self.ram[LINK_ACTUAL_VEL_Y] = self.ram[SPRITE_Y_VEL + k].wrapping_mul(2);
        }
    }

    // void Sprite_AttemptZapDamage(int k) {  // 86ec02
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_attempt_zap_damage(&mut self, k: usize) {
        let ty = self.ram[SPRITE_TYPE + k];
        let electric = (ty == 0x7a
            || (ty == 0x0d && self.ram[LINK_SWORD_TYPE] < 4)
            || ((ty == 0x24 || ty == 0x23) && self.ram[SPRITE_DELAY_MAIN + k] != 0))
            && self.ram[SPRITE_STATE + k] == 9;
        if electric {
            if self.ram[COUNTDOWN_FOR_BLINK] == 0 {
                self.ram[SPRITE_DELAY_AUX1_SPRITE + k] = 64;
                self.ram[LINK_ELECTROCUTE_ON_TOUCH] = 64;
                self.sprite_attempt_damage_to_link_plus_recoil(k);
            }
        } else {
            let vel = if sign8(self.ram[BUTTON_B_FRAMES].wrapping_sub(9)) {
                0x50
            } else {
                0x40
            };
            let pt = self.sprite_project_speed_towards_link(k, vel);
            self.ram[SPRITE_X_RECOIL + k] = 0u8.wrapping_sub(pt.x);
            self.ram[SPRITE_Y_RECOIL + k] = 0u8.wrapping_sub(pt.y);
            self.sprite_calculate_sword_damage(k);
        }
    }

    // bool Sprite_CheckTileProperty(int k, int j) {  // 86e73c
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_check_tile_property(&mut self, k: usize, j: i32) -> bool {
        let orig_j = j;
        let mut trace_tile_matches = std::env::var_os("ZELDA3_TRACE_TILE_COLLISION").is_some()
            && std::env::var("ZELDA3_TRACE_TILE_COLLISION_FRAME")
                .ok()
                .and_then(|s| {
                    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
                        u8::from_str_radix(hex, 16).ok()
                    } else {
                        s.parse::<u8>().ok()
                    }
                })
                .map_or(true, |target| self.ram[FRAME_COUNTER] == target);
        if trace_tile_matches {
            if let Ok(value) = std::env::var("ZELDA3_TRACE_TILE_COLLISION_TYPE") {
                let target = value
                    .strip_prefix("0x")
                    .or_else(|| value.strip_prefix("0X"))
                    .and_then(|hex| u8::from_str_radix(hex, 16).ok())
                    .or_else(|| value.parse::<u8>().ok());
                if target != Some(self.ram[SPRITE_TYPE + k]) {
                    trace_tile_matches = false;
                }
            }
        }
        if trace_tile_matches {
            if let Ok(value) = std::env::var("ZELDA3_TRACE_TILE_COLLISION_SLOT") {
                if value.parse::<usize>().ok() != Some(k) {
                    trace_tile_matches = false;
                }
            }
        }
        let j = (j >> 1) as usize;
        const FUNC5_X: [i8; 54] = [
            8, 8, 2, 14, 8, 8, -2, 10, 8, 8, 1, 14, 4, 4, 4, 4, 4, 4, -2, 10, 8, 8, -25, 40, 8, 8,
            2, 14, 8, 8, -8, 23, 8, 8, -20, 36, 8, 8, -1, 16, 8, 8, -1, 16, 8, 8, -8, 24, 8, 8, -8,
            24, 8, 3,
        ];
        const FUNC5_Y: [i8; 54] = [
            6, 20, 13, 13, 0, 8, 4, 4, 1, 14, 8, 8, 4, 4, 4, 4, -2, 10, 4, 4, -25, 40, 8, 8, 3, 16,
            10, 10, -8, 25, 8, 8, -20, 36, 8, 8, -1, 16, 8, 8, 14, 3, 8, 8, -8, 24, 8, 8, -8, 32,
            8, 8, 12, 4,
        ];
        const SIMPLIFIED_TILE_ATTR: [u8; 256] = [
            0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0,
            3, 3, 3, 0, 0, 0, 0, 0, 0, 1, 1, 4, 4, 4, 4, 4, 4, 4, 4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        ];
        const FUNC5_TAB3: [i8; 256] = [
            0, 1, 2, 3, 2, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0,
            1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 2, -1, -1, -1, -1, -1, -1, -1, -1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 0, 0, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 0, 1, 0, 1, 0, 0, -1, -1, -1, -1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 2, 0, 0, 0, 0, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        ];
        let mut x;
        let y;
        let in_bounds;
        if self.ram[PLAYER_IS_INDOORS] != 0 {
            x = (read_le_u16(&self.ram, CUR_SPRITE_X).wrapping_add(8) & 0x01ff)
                .wrapping_add(FUNC5_X[j] as i16 as u16)
                .wrapping_sub(8);
            y = (read_le_u16(&self.ram, CUR_SPRITE_Y).wrapping_add(8) & 0x01ff)
                .wrapping_add(FUNC5_Y[j] as i16 as u16)
                .wrapping_sub(8);
            in_bounds = x < 0x0200 && y < 0x0200;
        } else {
            x = read_le_u16(&self.ram, CUR_SPRITE_X).wrapping_add(FUNC5_X[j] as i16 as u16);
            y = read_le_u16(&self.ram, CUR_SPRITE_Y).wrapping_add(FUNC5_Y[j] as i16 as u16);
            in_bounds = x.wrapping_sub(read_le_u16(&self.ram, SPRCOLL_X_BASE_SPRITE))
                < read_le_u16(&self.ram, SPRCOLL_X_SIZE)
                && y.wrapping_sub(read_le_u16(&self.ram, SPRCOLL_Y_BASE_SPRITE))
                    < read_le_u16(&self.ram, SPRCOLL_Y_SIZE);
        }
        if !in_bounds {
            if trace_tile_matches {
                eprintln!(
                    "R tile fc={} k={} orig={} j={} x=0x{:04x} y=0x{:04x} in=0 floor=0x{:02x} flags2=0x{:02x} ret={}",
                    self.ram[FRAME_COUNTER],
                    k,
                    orig_j,
                    j,
                    x,
                    y,
                    self.ram[SPRITE_FLOOR + k],
                    self.ram[SPRITE_FLAGS2 + k],
                    if self.ram[SPRITE_FLAGS2 + k] & 0x40 != 0 {
                        0
                    } else {
                        1
                    }
                );
            }
            if self.ram[SPRITE_FLAGS2 + k] & 0x40 != 0 {
                self.ram[SPRITE_STATE + k] = 0;
                return false;
            }
            return true;
        }
        let b = self.sprite_get_tile_attribute(k, &mut x, y);
        if trace_tile_matches {
            eprintln!(
                "R tile fc={} k={} orig={} j={} x=0x{:04x} y=0x{:04x} in=1 floor=0x{:02x} b=0x{:02x} tile=0x{:02x} defl=0x{:02x} flags5=0x{:02x} tab3=0x{:02x}",
                self.ram[FRAME_COUNTER],
                k,
                orig_j,
                j,
                x,
                y,
                self.ram[SPRITE_FLOOR + k],
                b,
                self.ram[SPRITE_TILETYPE_SPR],
                self.ram[SPRITE_DEFL_BITS + k],
                self.ram[SPRITE_FLAGS5 + k],
                FUNC5_TAB3[usize::from(b)] as u8
            );
        }
        if self.ram[SPRITE_DEFL_BITS + k] & 8 != 0 {
            let a = SIMPLIFIED_TILE_ATTR[usize::from(b)];
            if a == 4 {
                if self.ram[PLAYER_IS_INDOORS] == 0 {
                    self.ram[SPRITE_E + k] = 4;
                }
            } else if a >= 1 {
                return if (0x10..0x14).contains(&self.ram[SPRITE_TILETYPE_SPR]) {
                    self.entity_check_sloped_tile_collision(x, y)
                } else {
                    true
                };
            }
            return false;
        }

        if self.ram[SPRITE_FLAGS5 + k] & 0x40 != 0 {
            let typ = self.ram[SPRITE_TYPE + k];
            if (typ == 0xd2 || typ == 0x8a) && b == 9 {
                return false;
            }
            if (typ == 0x94 && self.ram[SPRITE_E + k] == 0)
                || typ == 0xe3
                || typ == 0x8c
                || typ == 0x9a
                || typ == 0x81
            {
                return b != 8 && b != 9;
            }
        }

        if FUNC5_TAB3[usize::from(b)] == 0 {
            return false;
        }
        if (0x10..0x14).contains(&self.ram[SPRITE_TILETYPE_SPR]) {
            return self.entity_check_sloped_tile_collision(x, y);
        }
        if self.ram[SPRITE_TILETYPE_SPR] == 0x44 {
            if self.ram[SPRITE_F + k] != 0 && !sign8(self.ram[SPRITE_GIVE_DAMAGE_SPRITE + k]) {
                if self.ram[SPRITE_TYPE + k] == 0x88
                    && self.read_u32_ram(ENHANCED_FEATURES0) & 4096 != 0
                {
                    if self.ram[SPRITE_HIT_TIMER + k] == 0 {
                        self.ancilla_check_damage_to_sprite_preset(k, 6);
                    }
                } else {
                    self.ancilla_check_damage_to_sprite_preset(k, 4);
                }
                if self.ram[SPRITE_HIT_TIMER + k] != 0 {
                    self.ram[SPRITE_HIT_TIMER + k] = 153;
                    self.ram[SPRITE_F + k] = 0;
                }
            }
        } else if self.ram[SPRITE_TILETYPE_SPR] == 0x20 {
            return self.ram[SPRITE_FLAGS_SPRITE + k] & 1 == 0 || self.ram[SPRITE_F + k] == 0;
        }
        true
    }

    // void Sprite_CheckForTileInDirection_horizontal(int k, int yy) {  // 86e5b8
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_check_for_tile_in_direction_horizontal(&mut self, k: usize, yy: i32) {
        if !self.sprite_check_tile_in_direction(k, yy) {
            return;
        }
        const SPRITE_FUNC7_TAB: [u8; 4] = [8, 4, 2, 1];
        let idx = (yy as usize) & 3;
        self.ram[SPRITE_WALLCOLL + k] |= SPRITE_FUNC7_TAB[idx];
        if (self.ram[SPRITE_SUBTYPE + k] & 7) < 5 {
            let n = if self.ram[SPRITE_F + k] != 0 { 3 } else { 1 };
            self.sprite_add_xy(k, if (yy & 1) != 0 { -n } else { n }, 0);
        }
    }

    // void Sprite_CheckForTileInDirection_vertical(int k, int yy) {  // 86e5ee
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_check_for_tile_in_direction_vertical(&mut self, k: usize, yy: i32) {
        if !self.sprite_check_tile_in_direction(k, yy) {
            return;
        }
        const SPRITE_FUNC7_TAB: [u8; 4] = [8, 4, 2, 1];
        let idx = (yy as usize) & 3;
        self.ram[SPRITE_WALLCOLL + k] |= SPRITE_FUNC7_TAB[idx];
        if (self.ram[SPRITE_SUBTYPE + k] & 7) < 5 {
            let n = if self.ram[SPRITE_F + k] != 0 { 3 } else { 1 };
            self.sprite_add_xy(k, 0, if (yy & 1) != 0 { -n } else { n });
        }
    }

    // bool Sprite_CheckTileInDirection(int k, int yy) {  // 86e72f
    //   uint8 t = (sprite_flags[k] & 0xf0);
    //   yy = 2 * ((t >> 2) + yy);
    //   return Sprite_CheckTileProperty(k, yy);
    // }
    pub(super) fn sprite_check_tile_in_direction(&mut self, k: usize, yy: i32) -> bool {
        let t = i32::from(self.ram[SPRITE_FLAGS_SPRITE + k] & 0xf0);
        self.sprite_check_tile_property(k, 2 * ((t >> 2) + yy))
    }

    // bool Entity_CheckSlopedTileCollision(uint16 x, uint16 y) {  // 86e8fe
    //   ...see sprite.c...
    // }
    pub(super) fn entity_check_sloped_tile_collision(&mut self, x: u16, y: u16) -> bool {
        const SLOPED_TILE: [u8; 32] = [
            7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 7, 6, 5, 4, 3,
            2, 1, 0,
        ];
        let a = (y & 7) as u8;
        let r6 = self.ram[SPRITE_TILETYPE_SPR].wrapping_sub(0x10);
        let b = SLOPED_TILE[usize::from(r6) * 8 + usize::from(x & 7)];
        if r6 < 2 {
            b >= a
        } else {
            a >= b
        }
    }

    // void Sprite_DrawRippleIfInWater(int k) {  // 9eff8d
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_draw_ripple_if_in_water(&mut self, k: usize) {
        if self.ram[SPRITE_I_LOC + k] != 8 && self.ram[SPRITE_I_LOC + k] != 9 {
            return;
        }
        if self.ram[SPRITE_FLAGS3 + k] & 0x20 != 0 {
            let x = read_le_u16(&self.ram, CUR_SPRITE_X).wrapping_sub(4);
            write_le_u16(&mut self.ram, CUR_SPRITE_X, x);
            if self.ram[SPRITE_TYPE + k] == 0xdf {
                let y = read_le_u16(&self.ram, CUR_SPRITE_Y).wrapping_sub(7);
                write_le_u16(&mut self.ram, CUR_SPRITE_Y, y);
            }
        }
        self.sprite_draw_water_ripple(k);
        self.sprite_get16_bit_coords(k);
        self.oam_allocate_from_region_a(((self.ram[SPRITE_FLAGS2 + k] & 0x1f) + 1) * 4);
    }

    // void ThrownSprite_CheckDamageToSprites(int k) {  // 86e172
    //   ...see sprite.c...
    // }
    pub(super) fn thrown_sprite_check_damage_to_sprites(&mut self, k: usize) {
        if self.ram[SPRITE_DELAY_AUX4 + k] != 0
            || (self.ram[SPRITE_X_VEL + k] | self.ram[SPRITE_Y_VEL + k]) == 0
        {
            return;
        }
        for i in (0..=15usize).rev() {
            if i != self.ram[CUR_OBJECT_INDEX] as usize
                && self.ram[SPRITE_TYPE + k] != 0xd2
                && self.ram[SPRITE_STATE + i] >= 9
                && ((((i as u8) ^ self.ram[FRAME_COUNTER]) & 3)
                    | self.ram[SPRITE_IGNORE_PROJECTILE + i]
                    | self.ram[SPRITE_HIT_TIMER + i])
                    == 0
                && self.ram[SPRITE_FLOOR + k] == self.ram[SPRITE_FLOOR + i]
            {
                self.thrown_sprite_check_damage_to_single_sprite(k, i as i32);
            }
        }
    }

    // void ThrownSprite_CheckDamageToSingleSprite(int k, int j) {  // 86e1b2
    //   ...see sprite.c...
    // }
    pub(super) fn thrown_sprite_check_damage_to_single_sprite(&mut self, k: usize, j: i32) {
        let j = j as usize;
        let t = i32::from(self.ram[SPRITE_Y_LO + k]) - i32::from(self.ram[SPRITE_Z + k]);
        let u = ((t & 0xff) + 8) as u8;
        let mut hb = SpriteHitBox {
            r0_xlo: self.ram[SPRITE_X_LO + k],
            r8_xhi: self.ram[SPRITE_X_HI + k],
            r1_ylo: u,
            r9_yhi: self.ram[SPRITE_Y_HI + k]
                .wrapping_add(u8::from((t & 0xff) + 8 >= 0x100))
                .wrapping_sub(u8::from(t < 0)),
            r2: 15,
            r3: 8,
            r4_spr_xlo: 0,
            r10_spr_xhi: 0,
            r5_spr_ylo: 0,
            r11_spr_yhi: 0,
            r6_spr_xsize: 0,
            r7_spr_ysize: 0,
        };
        self.sprite_setup_hit_box(j, &mut hb);
        if !self.check_if_hit_boxes_overlap(&hb) {
            return;
        }
        if self.ram[SPRITE_TYPE + j] == 0x3f {
            self.sprite_place_weapon_tink(k);
        } else {
            let a = if self.ram[SPRITE_TYPE + k] == 0xec
                && self.ram[SPRITE_C_SPRITE + k] == 2
                && self.ram[PLAYER_IS_INDOORS] == 0
            {
                1
            } else {
                3
            };
            self.ancilla_check_damage_to_sprite_preset(j, a);
            self.ram[SPRITE_X_RECOIL + j] = self.ram[SPRITE_X_VEL + k].wrapping_mul(2);
            self.ram[SPRITE_Y_RECOIL + j] = self.ram[SPRITE_Y_VEL + k].wrapping_mul(2);
            self.ram[SPRITE_DELAY_AUX4 + k] = 16;
        }
        self.sprite_apply_ricochet(k);
    }

    // void Sprite_KillFriends() {
    //   for(int j = 15; j >= 0; j--) {
    //     if (j != cur_object_index && sprite_state[j] && !(sprite_defl_bits[j] & 2)
    //         && sprite_type[j] != 0x7a) {
    //       sprite_state[j] = 6;
    //       sprite_delay_main[j] = 15;
    //       sprite_flags3[j] = 0;
    //       sprite_flags5[j] = 0;
    //       sprite_flags2[j] = 3;
    //     }
    //   }
    // }
    pub(super) fn sprite_kill_friends(&mut self) {
        let cur = self.ram[CUR_OBJECT_INDEX] as usize;
        for j in (0..=15usize).rev() {
            if j == cur {
                continue;
            }
            if self.ram[SPRITE_STATE + j] == 0
                || (self.ram[SPRITE_DEFL_BITS + j] & 2) != 0
                || self.ram[SPRITE_TYPE + j] == 0x7a
            {
                continue;
            }
            self.ram[SPRITE_STATE + j] = 6;
            self.ram[SPRITE_DELAY_MAIN + j] = 15;
            self.ram[SPRITE_FLAGS3 + j] = 0;
            self.ram[SPRITE_FLAGS5 + j] = 0;
            self.ram[SPRITE_FLAGS2 + j] = 3;
        }
    }

    // void Sprite_Func8(int k) {  // 86e0ab
    //   sprite_state[k] = 1;
    //   sprite_delay_main[k] = 0x1f;
    //   sound_effect_1 = 0;
    //   SpriteSfx_QueueSfx2WithPan(k, 0x20);
    // }
    pub(super) fn sprite_func8(&mut self, k: usize) {
        self.ram[SPRITE_STATE + k] = 1;
        self.ram[SPRITE_DELAY_MAIN + k] = 0x1f;
        self.ram[SOUND_EFFECT_1] = 0;
        self.sprite_sfx_queue_sfx2_with_pan(k, 0x20);
    }

    // void Sprite_Func22(int k) {  // 86e0f6
    //   sound_effect_1 = Sprite_CalculateSfxPan(k) | 0x28;
    //   sprite_state[k] = 3;
    //   sprite_delay_main[k] = 15;
    //   sprite_ai_state[k] = 0;
    //   GetRandomNumber(); // wtf
    //   sprite_flags2[k] = 3;
    // }
    pub(super) fn sprite_func22(&mut self, k: usize) {
        self.ram[SOUND_EFFECT_1] = self.sprite_calculate_sfx_pan(k) | 0x28;
        self.ram[SPRITE_STATE + k] = 3;
        self.ram[SPRITE_DELAY_MAIN + k] = 15;
        self.ram[SPRITE_AI_STATE_SPRITE + k] = 0;
        self.get_random_number();
        self.ram[SPRITE_FLAGS2 + k] = 3;
    }

    // void Sprite_Func3(int k) {  // 86efda
    //   sprite_state[k] = 6;
    //   sprite_delay_main[k] = 31;
    //   sprite_flags2[k] = 3;
    // }
    pub(super) fn sprite_func3(&mut self, k: usize) {
        self.ram[SPRITE_STATE + k] = 6;
        self.ram[SPRITE_DELAY_MAIN + k] = 31;
        self.ram[SPRITE_FLAGS2 + k] = 3;
    }

    // void Sprite_SpawnSecret(int k) {  // 868264
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_spawn_secret(&mut self, k: usize) {
        const K_SPAWN_SECRET_ITEMS: [u8; 22] = [
            0xd9, 0x3e, 0x79, 0xd9, 0xdc, 0xd8, 0xda, 0xe4, 0xe1, 0xdc, 0xd8, 0xdf, 0xe0, 0x0b,
            0x42, 0xd3, 0x41, 0xd4, 0xd9, 0xe3, 0xd8, 0,
        ];
        const K_SPAWN_SECRET_ITEM_SPAWN_FLAG: [u8; 22] = [
            0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        const K_SPAWN_SECRET_ITEM_X_LO: [u8; 22] = [
            4, 0, 4, 4, 0, 4, 4, 4, 4, 0, 4, 4, 4, 0, 0, 0, 0, 0, 4, 0, 4, 4,
        ];
        const K_SPAWN_SECRET_ITEM_IGNORE_PROJ: [u8; 22] = [
            1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 1,
        ];
        const K_SPAWN_SECRET_ITEM_Z_VEL: [u8; 22] = [
            16, 0, 0, 16, 0, 0, 16, 16, 16, 16, 0, 16, 10, 16, 0, 0, 0, 0, 16, 0, 0, 0,
        ];

        if self.ram[PLAYER_IS_INDOORS] == 0 {
            let before_rng = self.ram[RNG_SEED];
            let roll = self.get_random_number();
            if std::env::var_os("ZELDA3_REPLAY_SPRITE_LOAD_DUMP").is_some() {
                println!(
                    "secret-spawn frame={} parent={} before=0x{:02x} roll=0x{:02x} b=0x{:02x} indoors={}",
                    self.ram[FRAME_COUNTER],
                    k,
                    before_rng,
                    roll,
                    self.ram[DUNG_SECRETS_UNK1_SPRITE],
                    self.ram[PLAYER_IS_INDOORS],
                );
            }
            if (roll & 8) != 0 {
                return;
            }
        }
        let mut b = self.ram[DUNG_SECRETS_UNK1_SPRITE];
        if b == 0 {
            return;
        }
        if b == 4 {
            b = 19 + (self.get_random_number() & 3);
        }
        let i = b.wrapping_sub(1) as usize;
        if i >= K_SPAWN_SECRET_ITEMS.len() || K_SPAWN_SECRET_ITEMS[i] == 0 {
            return;
        }

        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, K_SPAWN_SECRET_ITEMS[i], &mut info);
        if j < 0 {
            return;
        }
        let ju = j as usize;
        self.ram[SPRITE_AI_STATE_SPRITE + ju] = K_SPAWN_SECRET_ITEM_SPAWN_FLAG[i];
        self.ram[SPRITE_IGNORE_PROJECTILE + ju] = K_SPAWN_SECRET_ITEM_IGNORE_PROJ[i];
        self.ram[SPRITE_Z_VEL + ju] = K_SPAWN_SECRET_ITEM_Z_VEL[i];
        self.sprite_set_x(
            ju,
            info.r0_x
                .wrapping_add(u16::from(K_SPAWN_SECRET_ITEM_X_LO[i])),
        );
        self.sprite_set_y(ju, info.r2_y);
        self.ram[SPRITE_Z + ju] = info.r4_z;
        self.ram[SPRITE_GRAPHICS + ju] = 0;
        self.ram[SPRITE_DELAY_AUX4 + ju] = 32;
        self.ram[SPRITE_DELAY_AUX2_SPRITE + ju] = 48;

        let ty = self.ram[SPRITE_TYPE + ju];
        if ty == 0xe4 {
            self.sprite_prep_small_key(ju);
            self.ram[SPRITE_STUNNED + ju] = 255;
        } else if ty == 0x0b {
            self.ram[SOUND_EFFECT_1] = 0x30;
            if self.ram[DUNGEON_ROOM_INDEX2] == 1 {
                self.ram[SPRITE_SUBTYPE + ju] = 1;
            }
            self.ram[SPRITE_STUNNED + ju] = 255;
        } else if ty == 0x41 || ty == 0x42 {
            self.ram[SOUND_EFFECT_2] = 4;
            self.ram[SPRITE_GIVE_DAMAGE_SPRITE + ju] = 0;
            self.ram[SPRITE_HIT_TIMER + ju] = 160;
        } else if ty == 0x3e {
            self.ram[SPRITE_OAM_FLAGS + ju] = 9;
        } else {
            self.ram[SPRITE_STUNNED + ju] = 255;
            if ty == 0x79 {
                self.ram[SPRITE_A + ju] = 32;
            }
        }
    }

    // void Ancilla_CheckDamageToSprite_preset(int k, int a) {  // 86ece0
    //   if (a == 15 && sprite_z[k] != 0)
    //     return;
    //   if (a != 0 && a != 7) {
    //     Sprite_Func15(k, a);
    //     return;
    //   }
    //   Sprite_Func15(k, a);
    //   if (sprite_give_damage[k] || repulsespark_timer)
    //     return;
    //   repulsespark_timer = 5;
    //   int j = SPRITE_SHARED_SCRATCH_A;
    //   repulsespark_x_lo = ancilla_x_lo[j] + 4;
    //   repulsespark_y_lo = ancilla_y_lo[j];
    //   repulsespark_floor_status = link_is_on_lower_level;
    //   sound_effect_1 = 0;
    //   SpriteSfx_QueueSfx2WithPan(k, 5);
    // }
    pub(super) fn ancilla_check_damage_to_sprite_preset(&mut self, k: usize, a: u8) {
        if a == 15 && self.ram[SPRITE_Z + k] != 0 {
            return;
        }

        self.sprite_func15(k, a);
        if a != 0 && a != 7 {
            return;
        }
        if self.ram[SPRITE_GIVE_DAMAGE_SPRITE + k] != 0 || self.ram[REPULSESPARK_TIMER_SPRITE] != 0
        {
            return;
        }
        self.ram[REPULSESPARK_TIMER_SPRITE] = 5;
        let j = self.ram[SPRITE_SHARED_SCRATCH_A] as usize;
        self.ram[REPULSESPARK_X_LO_SPRITE] = self.ram[ANCILLA_X_LO_SPRITE + j].wrapping_add(4);
        self.ram[REPULSESPARK_Y_LO_SPRITE] = self.ram[ANCILLA_Y_LO_SPRITE + j];
        self.ram[REPULSESPARK_FLOOR_STATUS_SPRITE] = self.ram[LINK_IS_ON_LOWER_LEVEL];
        self.ram[SOUND_EFFECT_1] = 0;
        self.sprite_sfx_queue_sfx2_with_pan(k, 5);
    }

    // void Sprite_MiniMoldorm_Recoil(int k) {  // 86eec8
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_mini_moldorm_recoil(&mut self, k: usize) {
        if self.ram[SPRITE_STATE + k] < 9 {
            return;
        }
        self.ram[TMP_COUNTER_SPRITE] = self.ram[SPRITE_STATE + k];

        let dmg = self.ram[SPRITE_GIVE_DAMAGE_SPRITE + k];
        if dmg == 253 {
            self.ram[SPRITE_GIVE_DAMAGE_SPRITE + k] = 0;
            self.sprite_sfx_queue_sfx3_with_pan(k, 9);
            self.ram[SPRITE_STATE + k] = 7;
            self.ram[SPRITE_DELAY_MAIN + k] = 0x70;
            self.ram[SPRITE_FLAGS2 + k] = self.ram[SPRITE_FLAGS2 + k].wrapping_add(2);
            self.ram[SPRITE_GIVE_DAMAGE_SPRITE + k] = 0;
            return;
        }

        if dmg >= 251 {
            self.ram[SPRITE_GIVE_DAMAGE_SPRITE + k] = 0;
            if self.ram[SPRITE_STATE + k] == 11 {
                return;
            }
            self.ram[SPRITE_UNK5_LOC + k] = u8::from(dmg == 254);
            if self.ram[SPRITE_UNK5_LOC + k] != 0 {
                self.ram[SPRITE_DEFL_BITS + k] |= 8;
                self.ram[SPRITE_FLAGS5 + k] &= !0x80;
                self.sprite_sfx_queue_sfx2_with_pan(k, 15);
                self.ram[SPRITE_Z_VEL + k] = 24;
                self.ram[SPRITE_BUMP_DAMAGE + k] &= !0x80;
                self.sprite_zero_velocity_xy(k);
            }
            self.ram[SPRITE_STATE + k] = 11;
            self.ram[SPRITE_DELAY_MAIN + k] = 64;
            const HIT_TIMER24_STUN_VALUES: [u8; 5] = [0x20, 0x80, 0, 0, 0xff];
            self.ram[SPRITE_STUNNED + k] = HIT_TIMER24_STUN_VALUES[dmg.wrapping_add(5) as usize];
            if self.ram[SPRITE_TYPE + k] == 0x23 {
                self.ram[SPRITE_TYPE + k] = 0x24;
            }
            return;
        }

        let t = i32::from(self.ram[SPRITE_HEALTH + k]) - i32::from(dmg);
        self.ram[SPRITE_HEALTH + k] = t as u8;
        self.ram[SPRITE_GIVE_DAMAGE_SPRITE + k] = 0;
        if t > 0 {
            return;
        }

        if self.ram[SPRITE_DIE_ACTION + k] == 0 {
            if self.ram[SPRITE_STATE + k] == 11 {
                self.ram[SPRITE_DIE_ACTION + k] = 3;
            }
            if self.ram[SPRITE_UNK1_LOC + k] != 0 {
                self.ram[SPRITE_UNK1_LOC + k] = 0;
                self.ram[SPRITE_FLAGS5 + k] = 0;
            }
        }

        let ty = self.ram[SPRITE_TYPE + k];
        if ty != 0x1b {
            self.sprite_sfx_queue_sfx3_with_pan(k, 9);
        }

        if ty == 0x40 {
            let screen = self.ram[OVERWORLD_SCREEN_INDEX] as usize;
            self.ram[SAVE_OW_EVENT_INFO + screen] |= 0x40;
        } else if ty == 0xec {
            if self.ram[SPRITE_C_SPRITE + k] == 2 {
                self.throwable_scenery_transmute_to_debris(k);
            }
            return;
        }

        if self.ram[SPRITE_STATE + k] == 10 {
            self.ram[LINK_STATE_BITS] = 0;
            self.ram[LINK_PICKING_THROW_STATE] = 0;
        }
        self.ram[SPRITE_STATE + k] = 6;

        if ty == 0x0c {
            self.sprite_func3(k);
        } else if ty == 0x92 {
            self.sprite_kill_friends();
            self.ram[SPRITE_DELAY_MAIN + k] = 255;
            self.sprite_mini_moldorm_recoil_out_common(k);
        } else if ty == 0xcb {
            self.ram[SPRITE_AI_STATE_SPRITE + k] = 128;
            self.ram[SPRITE_DELAY_MAIN + k] = 128;
            self.ram[SPRITE_STATE + k] = 9;
            self.sprite_mini_moldorm_recoil_out_common(k);
        } else if ty == 0xcc || ty == 0xcd {
            self.ram[SPRITE_AI_STATE_SPRITE + k] = 128;
            self.ram[SPRITE_DELAY_MAIN + k] = 96;
            self.ram[SPRITE_STATE + k] = 9;
            self.sprite_mini_moldorm_recoil_out_common(k);
        } else if ty == 0x53 {
            self.ram[SPRITE_DELAY_MAIN + k] = 35;
            self.ram[SPRITE_HIT_TIMER + k] = 0;
            self.sprite_mini_moldorm_recoil_out_common2(k);
        } else if ty == 0x54 {
            self.ram[SPRITE_AI_STATE_SPRITE + k] = 5;
            self.ram[SPRITE_DELAY_MAIN + k] = 0xc0;
            self.ram[SPRITE_HIT_TIMER + k] = 0xc0;
            self.sprite_mini_moldorm_recoil_out_common(k);
        } else if ty == 0x09 {
            self.ram[SPRITE_AI_STATE_SPRITE + k] = 3;
            self.ram[SPRITE_DELAY_AUX4 + k] = 160;
            self.ram[SPRITE_STATE + k] = 9;
            self.sprite_mini_moldorm_recoil_out_common(k);
        } else if ty == 0x7a {
            self.sprite_kill_friends();
            self.ram[SPRITE_STATE + k] = 9;
            self.ram[SPRITE_IGNORE_PROJECTILE + k] = 9;
            if self.ram[IS_IN_DARK_WORLD_SPRITE] == 0 {
                self.ram[SPRITE_AI_STATE_SPRITE + k] = 10;
                self.ram[SPRITE_DELAY_MAIN + k] = 255;
                self.ram[SPRITE_Z_VEL + k] = 32;
            } else {
                self.ram[SPRITE_DELAY_MAIN + k] = 255;
                self.ram[SPRITE_AI_STATE_SPRITE + k] = 8;
                self.ram[SPRITE_AI_STATE_SPRITE + 1] = 9;
                self.ram[SPRITE_AI_STATE_SPRITE + 2] = 9;
                self.ram[SPRITE_GRAPHICS + 1] = 0;
                self.ram[SPRITE_GRAPHICS + 2] = 0;
            }
            self.sprite_mini_moldorm_recoil_out_common(k);
        } else if ty == 0x23 && self.ram[SPRITE_C_SPRITE + k] == 0 {
            self.ram[SPRITE_AI_STATE_SPRITE + k] = 2;
            self.ram[SPRITE_DELAY_MAIN + k] = 32;
            self.ram[SPRITE_STATE + k] = 9;
            self.ram[SPRITE_HIT_TIMER + k] = 0;
        } else if ty == 0x0f {
            self.ram[SPRITE_HIT_TIMER + k] = 0;
            self.ram[SPRITE_DELAY_MAIN + k] = 15;
        } else if self.ram[SPRITE_FLAGS_SPRITE + k] & 2 == 0 {
            self.ram[SPRITE_DELAY_MAIN + k] = if self.ram[SPRITE_HIT_TIMER + k] & 0x80 != 0 {
                31
            } else {
                15
            };
            self.ram[SPRITE_FLAGS2 + k] = self.ram[SPRITE_FLAGS2 + k].wrapping_add(4);
            if self.ram[TMP_COUNTER_SPRITE] == 11 {
                self.ram[SPRITE_FLAGS5 + k] = 1;
            }
        } else {
            if ty != 0xa2 {
                self.sprite_kill_friends();
            }
            self.ram[SPRITE_STATE + k] = 4;
            self.ram[SPRITE_A + k] = 0;
            self.ram[SPRITE_DELAY_MAIN + k] = 255;
            self.ram[SPRITE_HIT_TIMER + k] = 255;
            self.sprite_mini_moldorm_recoil_out_common(k);
        }
    }

    fn sprite_mini_moldorm_recoil_out_common(&mut self, k: usize) {
        self.ram[FLAG_BLOCK_LINK_MENU] = self.ram[FLAG_BLOCK_LINK_MENU].wrapping_add(1);
        self.sprite_mini_moldorm_recoil_out_common2(k);
    }

    fn sprite_mini_moldorm_recoil_out_common2(&mut self, k: usize) {
        self.ram[SOUND_EFFECT_2] = 0;
        self.sprite_sfx_queue_sfx3_with_pan(k, 0x22);
    }

    // bool Sprite_ReturnIfRecoiling(int k) {
    //   ...see sprite.c:3143...
    // }
    pub(super) fn sprite_return_if_recoiling(&mut self, k: usize) -> bool {
        const K_SPRITE2_RETURN_IF_RECOILING_MASKS: [u8; 6] = [3, 1, 0, 0, 0xc, 3];
        let trace_recoil_matches = std::env::var_os("ZELDA3_TRACE_RECOIL").is_some()
            && std::env::var("ZELDA3_TRACE_RECOIL_FRAME")
                .ok()
                .and_then(|value| {
                    let trimmed = value.trim();
                    if let Some(hex) = trimmed.strip_prefix("0x") {
                        u8::from_str_radix(hex, 16).ok()
                    } else {
                        trimmed.parse::<u8>().ok()
                    }
                })
                .is_none_or(|frame| frame == self.ram[FRAME_COUNTER]);
        if self.ram[SPRITE_F + k] == 0 {
            return false;
        }
        if self.ram[SPRITE_F + k] & 0x7f == 0 {
            self.ram[SPRITE_F + k] = 0;
            return false;
        }
        let yvbak = self.ram[SPRITE_Y_VEL + k];
        let xvbak = self.ram[SPRITE_X_VEL + k];
        if trace_recoil_matches {
            eprintln!(
                "R recoil fc={} entry k={} f=0x{:02x} xr=0x{:02x} yr=0x{:02x} xv=0x{:02x} yv=0x{:02x} bump=0x{:02x} x=0x{:04x} y=0x{:04x}",
                self.ram[FRAME_COUNTER],
                k,
                self.ram[SPRITE_F + k],
                self.ram[SPRITE_X_RECOIL + k],
                self.ram[SPRITE_Y_RECOIL + k],
                self.ram[SPRITE_X_VEL + k],
                self.ram[SPRITE_Y_VEL + k],
                self.ram[SPRITE_BUMP_DAMAGE + k],
                self.sprite_get_x(k),
                self.sprite_get_y(k),
            );
        }
        let new_f = self.ram[SPRITE_F + k].wrapping_sub(1);
        self.ram[SPRITE_F + k] = new_f;
        if new_f == 0
            && (self.ram[SPRITE_X_RECOIL + k].wrapping_add(0x20) >= 0x40
                || self.ram[SPRITE_Y_RECOIL + k].wrapping_add(0x20) >= 0x40)
        {
            self.ram[SPRITE_F + k] = 144;
        }
        let i = self.ram[SPRITE_F + k];
        // !sign8(i) -> top bit clear
        if (i & 0x80) == 0
            && (self.ram[FRAME_COUNTER] & K_SPRITE2_RETURN_IF_RECOILING_MASKS[(i >> 2) as usize])
                == 0
        {
            self.ram[SPRITE_Y_VEL + k] = self.ram[SPRITE_Y_RECOIL + k];
            self.ram[SPRITE_X_VEL + k] = self.ram[SPRITE_X_RECOIL + k];
            let bump = self.ram[SPRITE_BUMP_DAMAGE + k];
            let t = if (bump as i8) >= 0 {
                self.sprite_check_tile_collision(k) & 0xf
            } else {
                0
            };
            if trace_recoil_matches {
                eprintln!(
                    "R recoil fc={} collide k={} i=0x{:02x} mask=0x{:02x} t=0x{:02x} xv=0x{:02x} yv=0x{:02x} x=0x{:04x} y=0x{:04x}",
                    self.ram[FRAME_COUNTER],
                    k,
                    i,
                    K_SPRITE2_RETURN_IF_RECOILING_MASKS[(i >> 2) as usize],
                    t,
                    self.ram[SPRITE_X_VEL + k],
                    self.ram[SPRITE_Y_VEL + k],
                    self.sprite_get_x(k),
                    self.sprite_get_y(k),
                );
            }
            if (bump as i8) >= 0 && t != 0 {
                if t < 4 {
                    self.ram[SPRITE_X_RECOIL + k] = 0;
                    self.ram[SPRITE_X_VEL + k] = 0;
                } else {
                    self.ram[SPRITE_Y_RECOIL + k] = 0;
                    self.ram[SPRITE_Y_VEL + k] = 0;
                }
            } else {
                self.sprite_move_xy(k);
                if trace_recoil_matches {
                    eprintln!(
                        "R recoil fc={} move k={} x=0x{:04x} y=0x{:04x}",
                        self.ram[FRAME_COUNTER],
                        k,
                        self.sprite_get_x(k),
                        self.sprite_get_y(k),
                    );
                }
            }
        }
        self.ram[SPRITE_Y_VEL + k] = yvbak;
        self.ram[SPRITE_X_VEL + k] = xvbak;
        if trace_recoil_matches {
            eprintln!(
                "R recoil fc={} exit k={} ret={} f=0x{:02x} xr=0x{:02x} yr=0x{:02x} x=0x{:04x} y=0x{:04x}",
                self.ram[FRAME_COUNTER],
                k,
                self.ram[SPRITE_TYPE + k] != 0x7a,
                self.ram[SPRITE_F + k],
                self.ram[SPRITE_X_RECOIL + k],
                self.ram[SPRITE_Y_RECOIL + k],
                self.sprite_get_x(k),
                self.sprite_get_y(k),
            );
        }
        self.ram[SPRITE_TYPE + k] != 0x7a
    }

    // void Sprite_DrawMultiple(int k, const DrawMultipleData *src, int n,
    //                          PrepOamCoordsRet *info)
    //   See sprite.c:900.
    // Mirrors C: if `info` is None we use a local buffer; otherwise the
    // caller's out-pointer is populated.
    pub(super) fn sprite_draw_multiple(
        &mut self,
        k: usize,
        src: &[DrawMultipleData],
        info: Option<&mut PrepOamCoordsRet>,
    ) {
        let Some(prepped) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        if let Some(out) = info {
            out.x = prepped.0;
            out.y = prepped.1;
            out.r4 = 0;
            out.flags = prepped.2;
        }
        self.sprite_draw_multiple_with_info(k, src, prepped);
    }

    // Variant that takes a precomputed PrepOamCoord triple (x, y, flags). The
    // C version mutates a caller-supplied PrepOamCoordsRet pointer; Rust
    // callers pass the triple directly so the API stays panic-free.
    pub(super) fn sprite_draw_multiple_with_info(
        &mut self,
        k: usize,
        src: &[DrawMultipleData],
        info: (u16, u16, u8),
    ) {
        let (info_x, info_y, info_flags) = info;
        // r4 is always 0 in C's Sprite_PrepOamCoordOrDoubleRet (sprite.c:1843).
        let info_r4: u8 = 0;
        write_le_u16(&mut self.ram, SPRITE_DRAW_PRIORITY_OVERRIDE, 0);
        let mut a = self.ram[SPRITE_STATE + k];
        if a == 10 {
            a = self.ram[SPRITE_UNK4_LOC + k];
        }
        if a == 11 {
            self.ram[SPRITE_DRAW_PRIORITY_OVERRIDE] = self.ram[SPRITE_UNK5_LOC + k];
        }
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let combined_flags = (u16::from(info_flags) << 8) | u16::from(info_r4);
        for entry in src {
            let mut d = entry.char_flags ^ combined_flags;
            if read_le_u16(&self.ram, SPRITE_DRAW_PRIORITY_OVERRIDE) >= 1 {
                d = (d & !0x0e00) | 0x0400;
            }
            let x = info_x.wrapping_add(entry.x as i8 as i16 as u16);
            let y = info_y.wrapping_add(entry.y as i8 as i16 as u16);
            self.set_oam_helper0_at(oam, x, y, d as u8, (d >> 8) as u8, entry.ext);
            oam += 4;
        }
    }

    // void Sprite_DrawMultiplePlayerDeferred(int k, ...) {
    //   Oam_AllocateDeferToPlayer(k);
    //   Sprite_DrawMultiple(k, src, n, info);
    // }
    pub(super) fn sprite_draw_multiple_player_deferred(
        &mut self,
        k: usize,
        src: &[DrawMultipleData],
        info: Option<&mut PrepOamCoordsRet>,
    ) {
        self.oam_allocate_defer_to_player(k);
        self.sprite_draw_multiple(k, src, info);
    }

    fn sprite_single_draw_char(&self, k: usize) -> u8 {
        let base = K_SPRITE_PREP_AND_DRAW_SINGLE_LARGE_TAB1
            .get(usize::from(self.ram[SPRITE_TYPE + k]))
            .copied()
            .unwrap_or(0);
        K_SPRITE_PREP_AND_DRAW_SINGLE_LARGE_TAB2
            .get(usize::from(base) + usize::from(self.ram[SPRITE_GRAPHICS + k]))
            .copied()
            .unwrap_or(0)
    }

    // void SpriteDraw_SingleLarge(int k) {  // 86dc10
    //   PrepOamCoordsRet info;
    //   if (Sprite_PrepOamCoordOrDoubleRet(k, &info))
    //     return;
    //   Sprite_PrepAndDrawSingleLargeNoPrep(k, &info);
    // }
    pub(super) fn sprite_draw_single_large(&mut self, k: usize) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let mut info = PrepOamCoordsRet { x, y, r4: 0, flags };
        self.sprite_prep_and_draw_single_large_no_prep(k, &mut info);
    }

    // void Sprite_PrepAndDrawSingleLargeNoPrep(int k, PrepOamCoordsRet *info) {  // 86dc13
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_prep_and_draw_single_large_no_prep(
        &mut self,
        k: usize,
        info: &mut PrepOamCoordsRet,
    ) {
        let oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let chr = self.sprite_single_draw_char(k);
        self.ram[oam] = info.x as u8;
        if info.y.wrapping_add(0x10) < 0x100 {
            self.ram[oam + 1] = info.y as u8;
            self.ram[oam + 2] = chr;
            self.ram[oam + 3] = info.flags;
        }
        let ext_index = (oam - OAM_BUF) / 4;
        self.ram[BYTEWISE_EXTENDED_OAM + ext_index] = 2 | u8::from(info.x >= 256);
        if self.ram[SPRITE_FLAGS3 + k] & 0x10 != 0 {
            self.sprite_draw_shadow_custom(k, info, 10);
        }
    }

    // void SpriteDraw_Shadow_custom(int k, PrepOamCoordsRet *info, uint8 a) {  // 86dc5c
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_draw_shadow_custom(
        &mut self,
        k: usize,
        info: &mut PrepOamCoordsRet,
        a: u8,
    ) {
        let mut y = self.sprite_get_y(k).wrapping_add(u16::from(a));
        info.y = y;
        if self.ram[SPRITE_PAUSE + k] != 0
            || (self.ram[SPRITE_STATE + k] == 10 && self.ram[SPRITE_UNK3_LOC + k] == 3)
        {
            return;
        }
        y = y.wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2));
        info.y = y;
        if y.wrapping_add(0x10) >= 0x100 {
            return;
        }
        let oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize
            + usize::from(self.ram[SPRITE_FLAGS2 + k] & 0x1f) * 4;
        if self.ram[SPRITE_FLAGS3 + k] & 0x20 != 0 {
            self.set_oam_helper1_at(
                oam,
                info.x,
                y.wrapping_add(1) as u8,
                0x38,
                (info.flags & 0x30) | 8,
                0,
            );
        } else {
            self.set_oam_helper1_at(oam, info.x, y as u8, 0x6c, (info.flags & 0x30) | 8, 2);
        }
    }

    // void SpriteDraw_SingleSmall(int k) {  // 86dcef
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_draw_single_small(&mut self, k: usize) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        self.ram[oam] = x as u8;
        if y.wrapping_add(0x10) < 0x100 {
            self.ram[oam + 1] = y as u8;
            self.ram[oam + 2] = self.sprite_single_draw_char(k);
            self.ram[oam + 3] = flags;
        }
        let ext_index = (oam - OAM_BUF) / 4;
        self.ram[BYTEWISE_EXTENDED_OAM + ext_index] = u8::from(x >= 256);
        if self.ram[SPRITE_FLAGS3 + k] & 0x10 != 0 {
            let mut info = PrepOamCoordsRet { x, y, r4: 0, flags };
            self.sprite_draw_shadow_custom(k, &mut info, 2);
        }
    }

    // void Sprite_DrawThinAndTall(int k) {  // 86dd40
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_draw_thin_and_tall(&mut self, k: usize) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let chr = self.sprite_single_draw_char(k);
        self.set_oam_helper0_at(oam, x, y, chr, flags, 0);
        self.set_oam_helper0_at(
            oam + 4,
            x,
            y.wrapping_add(8),
            chr.wrapping_add(0x10),
            flags,
            0,
        );
        if self.ram[SPRITE_FLAGS3 + k] & 0x10 != 0 {
            let mut info = PrepOamCoordsRet { x, y, r4: 0, flags };
            self.sprite_draw_shadow_custom(k, &mut info, 10);
        }
    }

    // void SpriteFall_Draw(int k, PrepOamCoordsRet *info) {  // 9dffc5
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_fall_draw(&mut self, k: usize, info: &mut PrepOamCoordsRet) {
        const SPRITE_FALL_CHAR: [u8; 8] = [0x83, 0x83, 0x83, 0x80, 0x80, 0x80, 0xb7, 0xb7];
        let oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let idx = usize::from(self.ram[SPRITE_DELAY_MAIN + k] >> 2);
        self.ram[oam] = info.x.wrapping_add(4) as u8;
        self.ram[oam + 1] = info.y.wrapping_add(4) as u8;
        self.ram[oam + 2] = SPRITE_FALL_CHAR[idx];
        self.ram[oam + 3] = (info.flags & 0x30) | 0x04;
        self.sprite_correct_oam_entries(k, 0, 0);
    }

    // void Sprite_DrawDistress_custom(uint16 xin, uint16 yin, uint8 time) {  // 86a733
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_draw_distress_custom(&mut self, xin: u16, yin: u16, time: u8) {
        const X: [i8; 4] = [-3, 2, 7, 11];
        const Y: [i8; 4] = [-5, -7, -7, -5];
        self.oam_allocate_from_region_a(0x10);
        if (time & 0x18) == 0 {
            return;
        }
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        for i in (0..4).rev() {
            self.set_oam_helper0_at(
                oam,
                xin.wrapping_add(X[i] as i16 as u16),
                yin.wrapping_add(Y[i] as i16 as u16),
                0x83,
                0x22,
                0,
            );
            oam += 4;
        }
    }

    // void SpriteDraw_FallingHelmaBeetle(int k) {  // 86fd17
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_draw_falling_helma_beetle(&mut self, k: usize) {
        const FALL0: [DrawMultipleData; 12] = [
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x0146,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x0148,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x014a,
                ext: 2,
            },
            DrawMultipleData {
                x: 4,
                y: 4,
                char_flags: 0x014c,
                ext: 0,
            },
            DrawMultipleData {
                x: 4,
                y: 4,
                char_flags: 0x00b7,
                ext: 0,
            },
            DrawMultipleData {
                x: 4,
                y: 4,
                char_flags: 0x0080,
                ext: 0,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x016c,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x016e,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 0,
                char_flags: 0x014e,
                ext: 2,
            },
            DrawMultipleData {
                x: 4,
                y: 4,
                char_flags: 0x015c,
                ext: 0,
            },
            DrawMultipleData {
                x: 4,
                y: 4,
                char_flags: 0x00b7,
                ext: 0,
            },
            DrawMultipleData {
                x: 4,
                y: 4,
                char_flags: 0x0080,
                ext: 0,
            },
        ];
        let mut base = usize::from(self.ram[SPRITE_GRAPHICS + k]).min(5);
        if self.ram[SPRITE_TYPE + k] == 0x13 {
            base += 6;
        }
        self.sprite_draw_multiple(k, &FALL0[base..base + 1], None);
    }

    // void SpriteDraw_FallingHumanoid(int k) {  // 86fe5b
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_draw_falling_humanoid(&mut self, k: usize) {
        const X: [i8; 56] = [
            -4, 4, -4, 12, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, -4, 12, -4, 4, 0, 0, 0, 0, 0, 0, 0,
            0, 4, 0, 0, 0, -4, 12, -4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 4, 0, 0, 0, 4, 0, 0,
            0,
        ];
        const Y: [i8; 56] = [
            -4, -4, 4, 12, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, -4, -4, 12, 4, 0, 0, 0, 0, 0, 0, 0,
            0, 4, 0, 0, 0, -4, -4, 12, 4, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 4, 0, 0, 0, 4, 0, 0,
            0,
        ];
        const CHR: [u8; 56] = [
            0xae, 0xa8, 0xa6, 0xaf, 0xaa, 0, 0, 0, 0xac, 0, 0, 0, 0xbe, 0, 0, 0, 0xa8, 0xae, 0xaf,
            0xa6, 0xaa, 0, 0, 0, 0xac, 0, 0, 0, 0xbe, 0, 0, 0, 0xa6, 0xaf, 0xae, 0xa8, 0xaa, 0, 0,
            0, 0xac, 0, 0, 0, 0xbe, 0, 0, 0, 0xb6, 0, 0, 0, 0x80, 0, 0, 0,
        ];
        const FLAGS: [u8; 56] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0, 0, 0,
            0x40, 0, 0, 0, 0x40, 0, 0, 0, 0x80, 0x80, 0x80, 0x80, 0x80, 0, 0, 0, 0x80, 0, 0, 0,
            0x80, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0,
        ];
        const EXT: [u8; 56] = [
            0, 2, 2, 0, 2, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 2, 2, 0, 0, 0, 2, 0, 0, 0, 0,
            0, 0, 0, 2, 0, 0, 2, 2, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];

        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let q = usize::from(self.ram[SPRITE_GRAPHICS + k]);
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let n = if q < 12 && (q & 3) == 0 { 3 } else { 0 };
        for n_cur in (0..=n).rev() {
            let i = q * 4 + n_cur;
            self.set_oam_plain_at_for_sprite(
                oam,
                x.wrapping_add(X[i] as i16 as u16) as u8,
                y.wrapping_add(Y[i] as i16 as u16) as u8,
                CHR[i],
                flags ^ FLAGS[i],
                EXT[i],
            );
            oam += 4;
        }
        self.sprite_correct_oam_entries(k, n as i32, 0xff);
    }

    // void ScatterDebris_Draw(int k, Point16U pt) {  // 89f198
    //   ...see sprite.c...
    // }
    pub(super) fn scatter_debris_draw(&mut self, k: usize, pt: Point16U) {
        const X: [i8; 12] = [-8, 8, 16, -5, 8, 15, -1, 7, 11, 1, 3, 8];
        const Y: [i8; 12] = [7, 2, 12, 9, 2, 10, 11, 2, 11, 7, 3, 8];
        const CHR: [u8; 12] = [
            0xe2, 0xe2, 0xe2, 0xe2, 0xf2, 0xf2, 0xf2, 0xe2, 0xe2, 0xf2, 0xe2, 0xe2,
        ];
        const FLAGS: [u8; 12] = [0, 0, 0, 0, 0x80, 0x40, 0, 0x80, 0x40, 0, 0, 0];

        if self.ram[GARNISH_COUNTDOWN_SPRITE + k] == 16 {
            self.ram[GARNISH_TYPE + k] = 0;
        }
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let base = usize::from(((self.ram[GARNISH_COUNTDOWN_SPRITE + k] & 0x0f) >> 2) * 3);
        for i in (0..=2usize).rev() {
            let j = base + i;
            self.set_oam_helper1_at(
                oam,
                pt.x.wrapping_add(X[j] as i16 as u16),
                pt.y.wrapping_add(Y[j] as i16 as u16) as u8,
                CHR[j],
                FLAGS[j] | 0x22,
                0,
            );
            oam += 4;
        }
    }

    // void Garnish16_ThrownItemDebris(int k) {  // 89f0cb
    pub(super) fn garnish16_thrown_item_debris(&mut self, k: usize) {
        const X: [i16; 64] = [
            0, 8, 0, 8, -2, 9, -1, 9, -4, 9, -1, 10, -6, 9, -1, 12, -7, 9, -2, 13, -9, 9, -3, 14,
            -4, -4, 9, 15, -3, -3, -3, 9, -4, 4, 6, 10, -1, 4, 6, 7, 0, 2, 4, 7, 1, 1, 5, 7, 0, -2,
            8, 9, -1, -6, 9, 10, -2, -7, 12, 11, -3, -9, 4, 6,
        ];
        const Y: [i8; 64] = [
            0, 0, 8, 8, 0, -1, 10, 10, 0, -3, 11, 7, 1, -4, 12, 8, 1, -4, 13, 9, 2, -4, 16, 10, 14,
            14, -4, 11, 16, 16, 16, -1, 2, -5, 5, 1, 3, -7, 8, 2, 4, -8, 4, 10, -9, 4, 4, 12, -10,
            4, 8, 14, -12, 4, 8, 15, -15, 3, 8, 17, -17, 1, 18, 15,
        ];
        const CHR: [u8; 64] = [
            0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58,
            0x58, 0x58, 0x48, 0x58, 0x58, 0x58, 0x48, 0x58, 0x58, 0x48, 0x48, 0x48, 0x58, 0x48,
            0x48, 0x48, 0x48, 0x48, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59,
            0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59,
            0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59,
        ];
        const FLAGS: [u8; 64] = [
            0x80, 0, 0x80, 0x40, 0x80, 0x40, 0x80, 0, 0, 0xc0, 0, 0x80, 0x80, 0x40, 0x80, 0, 0x80,
            0xc0, 0, 0x80, 0, 0, 0x80, 0, 0x80, 0x80, 0x80, 0x80, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0,
            0x40, 0x40, 0x40, 0, 0x40, 0x40, 0, 0, 0x80, 0, 0x40, 0x40, 0x40, 0, 0x40, 0x40, 0x40,
            0x40, 0x40, 0x40, 0x40, 0x40, 0, 0, 0x40, 0, 0, 0,
        ];

        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let r5 = self.ram[GARNISH_OAM_FLAGS_SPRITE + k];
        if self.ram[SPRITE_CHR_HALFSLOT_STATE] >= 3 {
            return;
        }
        if self.ram[GARNISH_SPRITE_SPRITE + k] == 3 {
            self.scatter_debris_draw(k, pt);
            return;
        }
        self.ram[TMP_COUNTER_SPRITE] = self.ram[GARNISH_SPRITE_SPRITE + k];
        let mut base = ((self.ram[GARNISH_COUNTDOWN_SPRITE + k] >> 2) ^ 7) << 2;
        if self.ram[TMP_COUNTER_SPRITE] == 4
            || (self.ram[TMP_COUNTER_SPRITE] == 2 && self.ram[PLAYER_IS_INDOORS] == 0)
        {
            base = base.wrapping_add(0x20);
        }
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        for i in (0..=3usize).rev() {
            let j = usize::from(base) + i;
            let chr = if self.ram[TMP_COUNTER_SPRITE] == 0 {
                0x4e
            } else if self.ram[TMP_COUNTER_SPRITE] >= 0x80 {
                0xf2
            } else {
                CHR[j]
            };
            self.set_oam_helper1_at(
                oam,
                pt.x.wrapping_add(X[j] as u16),
                pt.y.wrapping_add(Y[j] as i16 as u16) as u8,
                chr,
                FLAGS[j] | r5,
                0,
            );
            oam += 4;
        }
    }

    // void Oam_AllocateDeferToPlayer(int k) — sprite.c:2920
    pub(super) fn oam_allocate_defer_to_player(&mut self, k: usize) {
        if self.ram[SPRITE_FLOOR + k] != self.ram[LINK_IS_ON_LOWER_LEVEL] {
            return;
        }
        let right = self.sprite_is_right_of_link(k);
        if right.b.wrapping_add(0x10) >= 0x20 {
            return;
        }
        let below = self.sprite_is_below_link(k);
        if below.b.wrapping_add(0x20) >= 0x48 {
            return;
        }
        let nslots = ((self.ram[SPRITE_FLAGS2 + k] & 0x1f) + 1) << 2;
        if below.a != 0 {
            self.oam_allocate_from_region_c(nslots);
        } else {
            self.oam_allocate_from_region_b(nslots);
        }
    }

    // bool Sprite_ReturnIfLifted(int k) — sprite.c:2602
    pub(super) fn sprite_return_if_lifted(&mut self, k: usize) -> bool {
        if self.frame_control_view().submodule() != 0
            || self.ram[BUTTON_B_FRAMES] != 0
            || self.ram[FLAG_UNK1] != 0
            || self.ram[SPRITE_FLOOR + k] != self.ram[LINK_IS_ON_LOWER_LEVEL]
        {
            return false;
        }
        for j in (0..=15usize).rev() {
            if self.ram[SPRITE_STATE + j] == 10 {
                return false;
            }
        }
        if self.ram[SPRITE_TYPE + k] != 0xb
            && self.ram[SPRITE_TYPE + k] != 0x4a
            && (self.ram[SPRITE_X_VEL + k] | self.ram[SPRITE_Y_VEL + k]) != 0
        {
            return false;
        }
        if self.ram[LINK_IS_RUNNING] != 0 {
            return false;
        }
        self.sprite_return_if_lifted_permissive(k)
    }

    // bool Sprite_ReturnIfLiftedPermissive(int k) — sprite.c:2615
    pub(super) fn sprite_return_if_lifted_permissive(&mut self, k: usize) -> bool {
        const K_SPRITE_RETURN_IF_LIFTED_DIRS: [u8; 4] = [4, 6, 0, 2];
        if self.ram[LINK_IS_RUNNING] != 0 {
            return false;
        }
        if self.ram[FLAG_IS_SPRITE_TO_PICK_UP_CACHED].wrapping_sub(1) != self.ram[CUR_OBJECT_INDEX]
        {
            let mut hb = SpriteHitBox {
                r0_xlo: 0,
                r8_xhi: 0,
                r1_ylo: 0,
                r9_yhi: 0,
                r2: 0,
                r3: 0,
                r4_spr_xlo: 0,
                r10_spr_xhi: 0,
                r5_spr_ylo: 0,
                r11_spr_yhi: 0,
                r6_spr_xsize: 0,
                r7_spr_ysize: 0,
            };
            self.link_setup_hit_box_conditional(&mut hb);
            self.sprite_setup_hit_box(k, &mut hb);
            if self.check_if_hit_boxes_overlap(&hb) {
                let v = (k as u8).wrapping_add(1);
                self.ram[SPRITE_PICKUP_SLOT_CACHE] = v;
                self.ram[FLAG_IS_SPRITE_TO_PICK_UP] = v;
            }
            false
        } else {
            self.ram[FILTERED_JOYPAD_L] = 0;
            self.ram[SPRITE_E + k] = 0;
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x1d);
            self.ram[SPRITE_UNK4_LOC + k] = self.ram[SPRITE_STATE + k];
            self.ram[SPRITE_STATE + k] = 10;
            self.ram[SPRITE_DELAY_MAIN + k] = 16;
            self.ram[SPRITE_UNK3_LOC + k] = 0;
            self.ram[SPRITE_I_LOC + k] = 0;
            let dir = self.sprite_direction_to_face_link(k, None) as usize;
            self.ram[LINK_DIRECTION_FACING] = K_SPRITE_RETURN_IF_LIFTED_DIRS[dir & 3];
            true
        }
    }

    // void Sprite_CheckIfLifted_permissive(int k) {  // 86aa0c
    //   Sprite_ReturnIfLiftedPermissive(k);
    // }
    pub(super) fn sprite_check_if_lifted_permissive(&mut self, k: usize) {
        let _ = self.sprite_return_if_lifted_permissive(k);
    }

    // uint8 Sprite_DirectionToFaceLink(int k, PointU8 *coords_out) {  // 86eaa4
    //   PairU8 below = Sprite_IsBelowLink(k);
    //   PairU8 right = Sprite_IsRightOfLink(k);
    //   uint8 ym = sign8(below.b) ? -below.b : below.b;
    //   tmp_counter = ym;
    //   uint8 xm = sign8(right.b) ? -right.b : right.b;
    //   if (coords_out)
    //     coords_out->x = right.b, coords_out->y = below.b;
    //   return (xm >= ym) ? right.a : below.a + 2;
    // }
    pub(super) fn sprite_direction_to_face_link(
        &mut self,
        k: usize,
        coords_out: Option<&mut PointU8>,
    ) -> u8 {
        let below = self.sprite_is_below_link(k);
        let right = self.sprite_is_right_of_link(k);
        let ym = if sign8(below.b) {
            0u8.wrapping_sub(below.b)
        } else {
            below.b
        };
        self.ram[TMP_COUNTER_SPRITE] = ym;
        let xm = if sign8(right.b) {
            0u8.wrapping_sub(right.b)
        } else {
            right.b
        };
        if let Some(coords) = coords_out {
            coords.x = right.b;
            coords.y = below.b;
        }
        if xm >= ym {
            right.a
        } else {
            below.a + 2
        }
    }

    // int Sprite_SpawnDynamically(int k, uint8 what, SpriteSpawnInfo *info) { // 9df65d
    //   return Sprite_SpawnDynamicallyEx(k, what, info, 15);
    // }
    //
    // Canonical 1:1 port. Returns -1 if no slot was found, otherwise the
    // 0..15 slot index. `info` is populated with the spawn coordinates the
    // caller will consume via `Sprite_SetSpawnedCoordinates`.
    pub(super) fn sprite_spawn_dynamically(
        &mut self,
        k: usize,
        what: u8,
        info: &mut SpriteSpawnInfo,
    ) -> i32 {
        self.sprite_spawn_dynamically_ex(k, what, info, 15)
    }

    // int Sprite_SpawnDynamicallyEx(int k, uint8 what, SpriteSpawnInfo *info, int j) { // 9df65f
    //   do {
    //     if (sprite_state[j] == 0) {
    //       sprite_type[j] = what;
    //       sprite_state[j] = 9;
    //       info->r0_x = Sprite_GetX(k);
    //       info->r2_y = Sprite_GetY(k);
    //       info->r4_z = sprite_z[k];
    //       info->r5_overlord_x = overlord_x_lo[k] | overlord_x_hi[k] << 8;
    //       info->r7_overlord_y = overlord_y_lo[k] | overlord_y_hi[k] << 8;
    //       SpritePrep_LoadProperties(j);
    //       if (!player_is_indoors) {
    //         sprite_N_word[j] = 0xffff;
    //       } else {
    //         sprite_N[j] = 0xff;
    //       }
    //       sprite_floor[j] = sprite_floor[k];
    //       sprite_D[j] = sprite_D[k];
    //       sprite_die_action[j] = 0;
    //       sprite_subtype[j] = 0;
    //       break;
    //     }
    //   } while (--j >= 0);
    //   return j;
    // }
    //
    // Canonical 1:1 port. The do/while loop walks j down from the caller's
    // starting bound (15 from `Sprite_SpawnDynamically`, or 13 / 14 for
    // narrower variants); the first slot with `sprite_state[j] == 0` wins.
    pub(super) fn sprite_spawn_dynamically_ex(
        &mut self,
        k: usize,
        what: u8,
        info: &mut SpriteSpawnInfo,
        j_in: i32,
    ) -> i32 {
        let mut j = j_in;
        loop {
            if j >= 0 && self.ram[SPRITE_STATE + j as usize] == 0 {
                let ju = j as usize;
                if std::env::var_os("ZELDA3_REPLAY_SPRITE_SPAWN_SCAN_DUMP").is_some() {
                    println!(
                        "dyn-scan frame={} parent={} what=0x{:02x} slot={} old_t=0x{:02x} old_st=0x{:02x} old_c=0x{:02x} old_bump=0x{:02x}",
                        self.ram[FRAME_COUNTER],
                        k,
                        what,
                        ju,
                        self.ram[SPRITE_TYPE + ju],
                        self.ram[SPRITE_STATE + ju],
                        self.ram[SPRITE_C_SPRITE + ju],
                        self.ram[SPRITE_BUMP_DAMAGE + ju],
                    );
                }
                if std::env::var_os("ZELDA3_REPLAY_SPRITE_LOAD_DUMP").is_some() {
                    println!(
                        "dyn-spawn frame={} parent={} what=0x{:02x} slot={} old_t=0x{:02x} old_st=0x{:02x} old_c=0x{:02x} old_bump=0x{:02x}",
                        self.ram[FRAME_COUNTER],
                        k,
                        what,
                        ju,
                        self.ram[SPRITE_TYPE + ju],
                        self.ram[SPRITE_STATE + ju],
                        self.ram[SPRITE_C_SPRITE + ju],
                        self.ram[SPRITE_BUMP_DAMAGE + ju],
                    );
                }
                self.ram[SPRITE_TYPE + ju] = what;
                self.ram[SPRITE_STATE + ju] = 9;
                info.r0_x = self.sprite_get_x(k);
                info.r2_y = self.sprite_get_y(k);
                info.r4_z = self.ram[SPRITE_Z + k];
                info.r5_overlord_x = u16::from(self.ram[SPRITE_OVERLORD_X_LO + k])
                    | (u16::from(self.ram[SPRITE_OVERLORD_X_HI + k]) << 8);
                info.r7_overlord_y = u16::from(self.ram[SPRITE_OVERLORD_Y_LO + k])
                    | (u16::from(self.ram[SPRITE_OVERLORD_Y_HI + k]) << 8);
                self.sprite_prep_load_properties_for_helpers(ju);
                if self.ram[PLAYER_IS_INDOORS] == 0 {
                    write_le_u16(&mut self.ram, SPRITE_N_WORD + ju * 2, 0xffff);
                } else {
                    self.ram[SPRITE_N + ju] = 0xff;
                }
                self.ram[SPRITE_FLOOR + ju] = self.ram[SPRITE_FLOOR + k];
                self.ram[SPRITE_D + ju] = self.ram[SPRITE_D + k];
                self.ram[SPRITE_DIE_ACTION + ju] = 0;
                self.ram[SPRITE_SUBTYPE + ju] = 0;
                break;
            }
            if j >= 0 && std::env::var_os("ZELDA3_REPLAY_SPRITE_SPAWN_SCAN_DUMP").is_some() {
                let ju = j as usize;
                println!(
                    "dyn-scan frame={} parent={} what=0x{:02x} slot={} old_t=0x{:02x} old_st=0x{:02x} old_c=0x{:02x} old_bump=0x{:02x}",
                    self.ram[FRAME_COUNTER],
                    k,
                    what,
                    ju,
                    self.ram[SPRITE_TYPE + ju],
                    self.ram[SPRITE_STATE + ju],
                    self.ram[SPRITE_C_SPRITE + ju],
                    self.ram[SPRITE_BUMP_DAMAGE + ju],
                );
            }
            j -= 1;
            if j < 0 {
                break;
            }
        }
        j
    }

    // int ReleaseFairy() {  // 9efe33
    //   SpriteSpawnInfo info;
    //   int j = Sprite_SpawnDynamically(0, 0xe3, &info);
    //   if (j >= 0) {
    //     sprite_floor[j] = link_is_on_lower_level;
    //     Sprite_SetX(j, link_x_coord + 8);
    //     Sprite_SetY(j, link_y_coord + 16);
    //     sprite_D[j] = 0;
    //     sprite_delay_aux4[j] = 96;
    //   }
    //   return j;
    // }
    pub(super) fn release_fairy(&mut self) -> i32 {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(0, 0xe3, &mut info);
        if j >= 0 {
            let ju = j as usize;
            self.ram[SPRITE_FLOOR + ju] = self.ram[LINK_IS_ON_LOWER_LEVEL];
            self.sprite_set_x(ju, self.player_state_view().x().wrapping_add(8));
            self.sprite_set_y(ju, self.player_state_view().y().wrapping_add(16));
            self.ram[SPRITE_D + ju] = 0;
            self.ram[SPRITE_DELAY_AUX4 + ju] = 96;
        }
        j
    }

    // uint8 Sprite_CheckTileCollision(int k) {  // 85b88d
    //   Sprite_CheckTileCollision2(k);
    //   return sprite_wallcoll[k];
    // }
    //
    // Canonical 1:1 port. Runs the 2-layer collision pass and returns the
    // cached `sprite_wallcoll[k]` byte.
    pub(super) fn sprite_check_tile_collision(&mut self, k: usize) -> u8 {
        self.sprite_check_tile_collision2(k);
        self.ram[SPRITE_WALLCOLL + k]
    }

    // void Sprite_CheckTileCollision2(int k) {  // 86e4ab
    //   sprite_wallcoll[k] = 0;
    //   if (sign8(sprite_flags4[k]) || !dung_hdr_collision) {
    //     Sprite_CheckTileCollisionSingleLayer(k);
    //     return;
    //   }
    //   SPRITE_SHARED_SCRATCH_A = sprite_floor[k];
    //   sprite_floor[k] = 1;
    //   Sprite_CheckTileCollisionSingleLayer(k);
    //   if (dung_hdr_collision == 4) {
    //     sprite_floor[k] = SPRITE_SHARED_SCRATCH_A;
    //     return;
    //   }
    //   sprite_floor[k] = 0;
    //   Sprite_CheckTileCollisionSingleLayer(k);
    //   byte_7FFABC[k] = sprite_tiletype;
    // }
    //
    // 1:1 port of the dispatcher around `Sprite_CheckTileCollisionSingleLayer`.
    pub(super) fn sprite_check_tile_collision2(&mut self, k: usize) {
        self.ram[SPRITE_WALLCOLL + k] = 0;
        let f4 = self.ram[SPRITE_FLAGS4 + k];
        let dung_coll = self.ram[DUNG_HDR_COLLISION];
        // sign8: top bit set.
        if (f4 & 0x80) != 0 || dung_coll == 0 {
            self.sprite_check_tile_collision_single_layer(k);
            return;
        }
        self.ram[SPRITE_SHARED_SCRATCH_A] = self.ram[SPRITE_FLOOR + k];
        self.ram[SPRITE_FLOOR + k] = 1;
        self.sprite_check_tile_collision_single_layer(k);
        if dung_coll == 4 {
            self.ram[SPRITE_FLOOR + k] = self.ram[SPRITE_SHARED_SCRATCH_A];
            return;
        }
        self.ram[SPRITE_FLOOR + k] = 0;
        self.sprite_check_tile_collision_single_layer(k);
        // byte_7FFABC[k] = sprite_tiletype — write-through to the
        // dual-layer cache so the next iteration sees the lower-layer tile.
        let tt = self.ram[SPRITE_TILETYPE_SPR];
        self.ram[BYTE_7FFABC + k] = tt;
    }

    // void Sprite_CheckTileCollisionSingleLayer(int k) {  // 86e4db
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_check_tile_collision_single_layer(&mut self, k: usize) {
        if self.ram[SPRITE_FLAGS2 + k] & 0x20 != 0 {
            if self.sprite_check_tile_property(k, 0x6a) {
                self.ram[SPRITE_WALLCOLL + k] = self.ram[SPRITE_WALLCOLL + k].wrapping_add(1);
            }
            return;
        }

        if sign8(self.ram[SPRITE_FLAGS4 + k]) || self.ram[DUNG_HDR_COLLISION] == 0 {
            if self.ram[SPRITE_Y_VEL + k] != 0 {
                self.sprite_check_for_tile_in_direction_vertical(
                    k,
                    if sign8(self.ram[SPRITE_Y_VEL + k]) {
                        0
                    } else {
                        1
                    },
                );
            }
            if self.ram[SPRITE_X_VEL + k] != 0 {
                self.sprite_check_for_tile_in_direction_horizontal(
                    k,
                    if sign8(self.ram[SPRITE_X_VEL + k]) {
                        2
                    } else {
                        3
                    },
                );
            }
        } else {
            self.sprite_check_for_tile_in_direction_vertical(k, 1);
            self.sprite_check_for_tile_in_direction_vertical(k, 0);
            self.sprite_check_for_tile_in_direction_horizontal(k, 3);
            self.sprite_check_for_tile_in_direction_horizontal(k, 2);
        }

        if sign8(self.ram[SPRITE_FLAGS5 + k]) || self.ram[SPRITE_Z + k] != 0 {
            return;
        }

        self.sprite_check_tile_property(k, 0x68);
        self.ram[SPRITE_I_LOC + k] = self.ram[SPRITE_TILETYPE_SPR];
        match self.ram[SPRITE_TILETYPE_SPR] {
            0x1c => {
                if self.ram[SORT_SPRITES_SETTING] != 0 && self.ram[SPRITE_STATE + k] == 11 {
                    self.ram[SPRITE_FLOOR + k] = 1;
                }
            }
            0x20 => {
                if self.ram[SPRITE_FLAGS_SPRITE + k] & 1 != 0 {
                    if self.ram[PLAYER_IS_INDOORS] == 0 {
                        self.sprite_func8(k);
                    } else {
                        self.ram[SPRITE_STATE + k] = 5;
                        if self.ram[SPRITE_TYPE + k] == 0x13 || self.ram[SPRITE_TYPE + k] == 0x26 {
                            self.ram[SPRITE_OAM_FLAGS + k] &= !1;
                            self.ram[SPRITE_DELAY_MAIN + k] = 63;
                        } else {
                            self.ram[SPRITE_DELAY_MAIN + k] = 95;
                        }
                    }
                }
            }
            0x0c => {
                if self.ram[BYTE_7FFABC + k] == 0x1c {
                    self.sprite_fall_adjust_position(k);
                    self.ram[SPRITE_WALLCOLL + k] |= 0x20;
                }
            }
            0x68..=0x6b => self.sprite_apply_conveyor(k, i32::from(self.ram[SPRITE_TILETYPE_SPR])),
            8 => {
                if self.ram[DUNG_HDR_COLLISION] == 4 {
                    self.sprite_apply_conveyor(k, 0x6a);
                }
            }
            _ => {}
        }
    }

    // uint8 GetTileAttribute(uint8 floor, uint16 *x, uint16 y) {  // 86e87b
    //   uint8 tiletype;
    //   if (player_is_indoors) {
    //     int t = (floor >= 1) ? 0x1000 : 0;
    //     t += (*x & 0x1f8) >> 3;
    //     t += (y & 0x1f8) << 3;
    //     tiletype = dung_bg2_attr_table[t];
    //   } else {
    //     tiletype = Overworld_GetTileAttributeAtLocation(*x >>= 3, y);
    //   }
    //   sprite_tiletype = tiletype;
    //   return tiletype;
    // }
    #[allow(non_snake_case)]
    pub(super) fn GetTileAttribute(&mut self, floor: u8, x: &mut u16, y: u16) -> u8 {
        let tiletype = if self.ram[PLAYER_IS_INDOORS] != 0 {
            let mut t = if floor >= 1 { 0x1000 } else { 0 };
            t += ((*x & 0x01f8) >> 3) as usize;
            t += ((y & 0x01f8) << 3) as usize;
            self.ram[DUNG_BG2_ATTR_TABLE + t]
        } else {
            *x >>= 3;
            self.overworld_get_tile_attribute_at_location(*x, y)
        };
        self.ram[SPRITE_TILETYPE_SPR] = tiletype;
        tiletype
    }

    // uint8 Sprite_GetTileAttribute(int k, uint16 *x, uint16 y) {  // 86e883
    //   return GetTileAttribute(sprite_floor[k], x, y);
    // }
    pub(super) fn sprite_get_tile_attribute(&mut self, k: usize, x: &mut u16, y: u16) -> u8 {
        self.GetTileAttribute(self.ram[SPRITE_FLOOR + k], x, y)
    }

    // int Sprite_ShowSolicitedMessage(int k, uint16 msg) {  // 85e1a7
    //   static const uint8 kShowMessageFacing_Tab0[4] = {4, 6, 0, 2};
    //   dialogue_message_index = msg;
    //   if (!Sprite_CheckDamageToLink_same_layer(k) ||
    //       Sprite_CheckIfLinkIsBusy() ||
    //       !(filtered_joypad_L & 0x80) ||
    //       sprite_delay_aux4[k] || link_auxiliary_state == 2)
    //     return sprite_D[k];
    //   uint8 dir = Sprite_DirectionToFaceLink(k, NULL);
    //   if (link_direction_facing != kShowMessageFacing_Tab0[dir])
    //     return sprite_D[k];
    //   Sprite_ShowMessageUnconditional(dialogue_message_index);
    //   sprite_delay_aux4[k] = 64;
    //   return dir ^ 0x103;
    // }
    //
    // Canonical 1:1 port. Returns either `sprite_D[k]` (low byte; caller
    // typically gates on the 0x100 bit) or `dir ^ 0x103` once dialogue is
    // actually triggered. Mirrors the C `int` return.
    pub(super) fn sprite_show_solicited_message(&mut self, k: usize, msg: u16) -> u16 {
        const K_SHOW_MESSAGE_FACING_TAB0: [u8; 4] = [4, 6, 0, 2];
        write_le_u16(&mut self.ram, DIALOGUE_MESSAGE_INDEX, msg);
        if !self.sprite_check_damage_to_link_same_layer_for_helpers(k)
            || self.sprite_check_if_link_is_busy_for_helpers()
            || (self.ram[FILTERED_JOYPAD_L] & 0x80) == 0
            || self.ram[SPRITE_DELAY_AUX4 + k] != 0
            || self.ram[LINK_AUXILIARY_STATE] == 2
        {
            return u16::from(self.ram[SPRITE_D + k]);
        }
        let dir = self.sprite_direction_to_face_link_for_helpers(k);
        if self.ram[LINK_DIRECTION_FACING] != K_SHOW_MESSAGE_FACING_TAB0[(dir & 3) as usize] {
            return u16::from(self.ram[SPRITE_D + k]);
        }
        let msg_index = read_le_u16(&self.ram, DIALOGUE_MESSAGE_INDEX);
        self.sprite_show_message_unconditional(msg_index);
        self.ram[SPRITE_DELAY_AUX4 + k] = 64;
        u16::from(dir) ^ 0x103
    }

    // int Sprite_ShowMessageOnContact(int k, uint16 msg) {  // 85e1f0
    //   dialogue_message_index = msg;
    //   if (!Sprite_CheckDamageToLink_same_layer(k) || link_auxiliary_state == 2)
    //     return sprite_D[k];
    //   Sprite_ShowMessageUnconditional(dialogue_message_index);
    //   return Sprite_DirectionToFaceLink(k, NULL) ^ 0x103;
    // }
    pub(super) fn sprite_show_message_on_contact(&mut self, k: usize, msg: u16) -> u16 {
        write_le_u16(&mut self.ram, DIALOGUE_MESSAGE_INDEX, msg);
        if !self.sprite_check_damage_to_link_same_layer(k) || self.ram[LINK_AUXILIARY_STATE] == 2 {
            return u16::from(self.ram[SPRITE_D + k]);
        }
        let msg_index = read_le_u16(&self.ram, DIALOGUE_MESSAGE_INDEX);
        self.sprite_show_message_unconditional(msg_index);
        u16::from(self.sprite_direction_to_face_link(k, None)) ^ 0x103
    }

    // bool Sprite_TutorialGuard_ShowMessageOnContact(int k, uint16 msg) {  // 85fa59
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_tutorial_guard_show_message_on_contact(
        &mut self,
        k: usize,
        msg: u16,
    ) -> bool {
        write_le_u16(&mut self.ram, DIALOGUE_MESSAGE_INDEX, msg);
        let bak2 = self.ram[SPRITE_FLAGS2 + k];
        let bak4 = self.ram[SPRITE_FLAGS4 + k];
        self.ram[SPRITE_FLAGS2 + k] = 0x80;
        self.ram[SPRITE_FLAGS4 + k] = 0x07;
        let rv = self.sprite_check_damage_to_link_same_layer(k);
        self.ram[SPRITE_FLAGS2 + k] = bak2;
        self.ram[SPRITE_FLAGS4 + k] = bak4;
        if !rv {
            return false;
        }
        self.sprite_nullify_hookshot_drag();
        self.ram[LINK_IS_RUNNING] = 0;
        self.ram[LINK_SPEED_SETTING] = 0;
        if self.ram[LINK_AUXILIARY_STATE] == 0 {
            self.sprite_show_message_minimal_c();
        }
        true
    }

    // void Sprite_ShowMessageUnconditional(uint16 msg) {  // 85e219
    //   dialogue_message_index = msg;
    //   TILE_INTERACTION_SHARED_FLAG = 0;
    //   messaging_module = 0;
    //   submodule_index = 2;
    //   saved_module_for_menu = main_module_index;
    //   main_module_index = 14;
    //   Sprite_NullifyHookshotDrag();
    //   link_speed_setting = 0;
    //   Link_CancelDash();
    //   link_auxiliary_state = 0;
    //   link_incapacitated_timer = 0;
    //   if (link_player_handler_state == kPlayerState_RecoilWall)
    //     link_player_handler_state = kPlayerState_Ground;
    // }
    //
    // Canonical 1:1 port. `kPlayerState_RecoilWall == 13`, `kPlayerState_Ground == 0`
    // (see player.h).
    pub(super) fn sprite_show_message_unconditional(&mut self, msg: u16) {
        const K_PLAYER_STATE_RECOIL_WALL_LOCAL: u8 = 13;
        const K_PLAYER_STATE_GROUND_LOCAL: u8 = 0;
        write_le_u16(&mut self.ram, DIALOGUE_MESSAGE_INDEX, msg);
        self.ram[TILE_INTERACTION_SHARED_FLAG] = 0;
        self.ram[MESSAGING_MODULE] = 0;
        let main_module = self.frame_control_view().main_module();
        self.frame_control_view_mut().set_submodule(2);
        self.ram[SAVED_MODULE_FOR_MENU] = main_module;
        self.frame_control_view_mut().set_main_module(14);
        self.sprite_nullify_hookshot_drag();
        self.ram[LINK_SPEED_SETTING] = 0;
        self.link_cancel_dash();
        self.ram[LINK_AUXILIARY_STATE] = 0;
        self.ram[LINK_INCAPACITATED_TIMER] = 0;
        if self.ram[LINK_PLAYER_HANDLER_STATE] == K_PLAYER_STATE_RECOIL_WALL_LOCAL {
            self.ram[LINK_PLAYER_HANDLER_STATE] = K_PLAYER_STATE_GROUND_LOCAL;
        }
    }

    // void Sprite_ShowMessageMinimal() {  // 85fa8e
    //   TILE_INTERACTION_SHARED_FLAG = 0;
    //   messaging_module = 0;
    //   submodule_index = 2;
    //   saved_module_for_menu = main_module_index;
    //   main_module_index = 14;
    // }
    pub(super) fn sprite_show_message_minimal_c(&mut self) {
        self.ram[TILE_INTERACTION_SHARED_FLAG] = 0;
        self.ram[MESSAGING_MODULE] = 0;
        let main_module = self.frame_control_view().main_module();
        self.frame_control_view_mut().set_submodule(2);
        self.ram[SAVED_MODULE_FOR_MENU] = main_module;
        self.frame_control_view_mut().set_main_module(14);
    }

    // void Sprite_ApplyConveyor(int k, int j) {  // 9d8010
    //   if (!(frame_counter & 1))
    //     return;
    //   static const int8 kConveyorAdjustment_X[] = {0, 0, -1, 1};
    //   static const int8 kConveyorAdjustment_Y[] = {-1, 1, 0, 0};
    //   Sprite_SetX(k, Sprite_GetX(k) + kConveyorAdjustment_X[j - 0x68]);
    //   Sprite_SetY(k, Sprite_GetY(k) + kConveyorAdjustment_Y[j - 0x68]);
    // }
    pub(super) fn sprite_apply_conveyor(&mut self, k: usize, j: i32) {
        const K_CONVEYOR_ADJUSTMENT_X: [i8; 4] = [0, 0, -1, 1];
        const K_CONVEYOR_ADJUSTMENT_Y: [i8; 4] = [-1, 1, 0, 0];

        if (self.ram[FRAME_COUNTER] & 1) == 0 {
            return;
        }
        let idx = (j - 0x68) as usize;
        self.sprite_set_x(
            k,
            self.sprite_get_x(k)
                .wrapping_add(K_CONVEYOR_ADJUSTMENT_X[idx] as i16 as u16),
        );
        self.sprite_set_y(
            k,
            self.sprite_get_y(k)
                .wrapping_add(K_CONVEYOR_ADJUSTMENT_Y[idx] as i16 as u16),
        );
    }

    // uint8 Sprite_BounceFromTileCollision(int k) {  // 9dc751
    //   int j = Sprite_CheckTileCollision(k);
    //   if (j & 3) {
    //     sprite_x_vel[k] = -sprite_x_vel[k];
    //     sprite_G[k]++;
    //   }
    //   if (j & 12) {
    //     sprite_y_vel[k] = -sprite_y_vel[k];
    //     sprite_G[k]++;
    //     return sprite_G[k]; // wtf
    //   }
    //   return 0;
    // }
    pub(super) fn sprite_bounce_from_tile_collision(&mut self, k: usize) -> u8 {
        let j = self.sprite_check_tile_collision(k);
        if (j & 3) != 0 {
            self.ram[SPRITE_X_VEL + k] = self.ram[SPRITE_X_VEL + k].wrapping_neg();
            self.ram[SPRITE_G + k] = self.ram[SPRITE_G + k].wrapping_add(1);
        }
        if (j & 12) != 0 {
            self.ram[SPRITE_Y_VEL + k] = self.ram[SPRITE_Y_VEL + k].wrapping_neg();
            self.ram[SPRITE_G + k] = self.ram[SPRITE_G + k].wrapping_add(1);
            return self.ram[SPRITE_G + k];
        }
        0
    }

    // int Sprite_SpawnSimpleSparkleGarnishEx(int k, uint16 x, uint16 y, int limit) {
    //   int j = GarnishAllocLimit(limit);
    //   if (j >= 0) {
    //     garnish_type[j] = 5;
    //     garnish_active = 5;
    //     Garnish_SetX(j, Sprite_GetX(k) + x);
    //     Garnish_SetY(j, Sprite_GetY(k) + y - sprite_z[k] + 16);
    //     garnish_countdown[j] = 31;
    //     garnish_sprite[j] = k;
    //     garnish_floor[j] = sprite_floor[k];
    //   }
    //   g_ram[15] = j;
    //   return j;
    // }
    pub(super) fn sprite_spawn_simple_sparkle_garnish_ex(
        &mut self,
        k: usize,
        x: u16,
        y: u16,
        limit: i32,
    ) -> i32 {
        let j = self.garnish_alloc_limit(limit as usize);
        if std::env::var_os("ZELDA3_REPLAY_GARNISH_TRACE").is_some() {
            eprintln!(
                "R garnish-spawn fc=0x{:02x} rng=0x{:02x} room=0x{:04x} k={} type=0x{:02x} state=0x{:02x} delay=0x{:02x} xarg=0x{:04x} yarg=0x{:04x} limit={} slot={} sx=0x{:04x} sy=0x{:04x} z=0x{:02x} r12=0x{:04x} r14=0x{:04x}",
                self.ram[FRAME_COUNTER],
                self.ram[RNG_SEED],
                self.world_state_view().dungeon_room(),
                k,
                self.ram[SPRITE_TYPE + k],
                self.ram[SPRITE_STATE + k],
                self.ram[SPRITE_DELAY_MAIN + k],
                x,
                y,
                limit,
                j,
                self.sprite_get_x(k),
                self.sprite_get_y(k),
                self.ram[SPRITE_Z + k],
                read_le_u16(&self.ram, R12),
                read_le_u16(&self.ram, R14),
            );
        }
        if j >= 0 {
            let j = j as usize;
            self.ram[GARNISH_TYPE + j] = 5;
            self.ram[GARNISH_ACTIVE_SPRITE] = 5;
            self.garnish_set_x(j, self.sprite_get_x(k).wrapping_add(x));
            self.garnish_set_y(
                j,
                self.sprite_get_y(k)
                    .wrapping_add(y)
                    .wrapping_sub(self.ram[SPRITE_Z + k] as u16)
                    .wrapping_add(16),
            );
            self.ram[GARNISH_COUNTDOWN_SPRITE + j] = 31;
            self.ram[GARNISH_SPRITE_SPRITE + j] = k as u8;
            self.ram[GARNISH_FLOOR_SPRITE + j] = self.ram[SPRITE_FLOOR + k];
        }
        self.ram[R15] = j as u8;
        j
    }

    // void Sprite_GarnishSpawn_Sparkle_limited(int k, uint16 x, uint16 y) {  // 9ea001
    //   Sprite_SpawnSimpleSparkleGarnishEx(k, x, y, 14);
    // }
    pub(super) fn sprite_garnish_spawn_sparkle_limited(&mut self, k: usize, x: u16, y: u16) {
        self.sprite_spawn_simple_sparkle_garnish_ex(k, x, y, 14);
    }

    // int Sprite_GarnishSpawn_Sparkle(int k, uint16 x, uint16 y) {  // 9ea007
    //   return Sprite_SpawnSimpleSparkleGarnishEx(k, x, y, 29);
    // }
    pub(super) fn sprite_garnish_spawn_sparkle(&mut self, k: usize, x: u16, y: u16) -> i32 {
        self.sprite_spawn_simple_sparkle_garnish_ex(k, x, y, 29)
    }

    // void Sprite_HaltAllMovement() {  // 9ef508
    //   Sprite_NullifyHookshotDrag();
    //   link_speed_setting = 0;
    //   Link_CancelDash();
    // }
    pub(super) fn sprite_halt_all_movement(&mut self) {
        self.sprite_nullify_hookshot_drag();
        self.ram[LINK_SPEED_SETTING] = 0;
        self.link_cancel_dash();
    }

    // void Sprite_BehaveAsBarrier(int k) {  // 9ef4f3
    //   uint8 bak = sprite_flags4[k];
    //   sprite_flags4[k] = 0;
    //   if (Sprite_CheckDamageToLink_same_layer(k))
    //     Sprite_HaltAllMovement();
    //   sprite_flags4[k] = bak;
    // }
    pub(super) fn sprite_behave_as_barrier(&mut self, k: usize) {
        let bak = self.ram[SPRITE_FLAGS4 + k];
        self.ram[SPRITE_FLAGS4 + k] = 0;
        if self.sprite_check_damage_to_link_same_layer(k) {
            self.sprite_halt_all_movement();
        }
        self.ram[SPRITE_FLAGS4 + k] = bak;
    }

    // bool Sprite_CheckIfScreenIsClear() {  // 89af32
    //   for (int i = 15; i >= 0; i--) {
    //     if (sprite_state[i] && !(sprite_flags4[i] & 0x40)) {
    //       uint16 x = Sprite_GetX(i) - BG2HOFS_copy2;
    //       uint16 y = Sprite_GetY(i) - BG2VOFS_copy2;
    //       if (x < 256 && y < 256)
    //         return false;
    //     }
    //   }
    //   return Sprite_CheckIfOverlordsClear();
    // }
    pub(super) fn sprite_check_if_screen_is_clear(&self) -> bool {
        for i in (0..=15usize).rev() {
            if self.ram[SPRITE_STATE + i] != 0 && (self.ram[SPRITE_FLAGS4 + i] & 0x40) == 0 {
                let x = self
                    .sprite_get_x(i)
                    .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2));
                let y = self
                    .sprite_get_y(i)
                    .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2));
                if x < 256 && y < 256 {
                    return false;
                }
            }
        }
        self.sprite_check_if_overlords_clear()
    }

    // bool Sprite_CheckIfRoomIsClear() {  // 89af61
    //   for (int i = 15; i >= 0; i--) {
    //     if (sprite_state[i] && !(sprite_flags4[i] & 0x40))
    //       return false;
    //   }
    //   return Sprite_CheckIfOverlordsClear();
    // }
    pub(super) fn sprite_check_if_room_is_clear(&self) -> bool {
        for i in (0..=15usize).rev() {
            if self.ram[SPRITE_STATE + i] != 0 && (self.ram[SPRITE_FLAGS4 + i] & 0x40) == 0 {
                return false;
            }
        }
        self.sprite_check_if_overlords_clear()
    }

    // bool Sprite_CheckIfOverlordsClear() {  // 89af76
    //   for (int i = 7; i >= 0; i--) {
    //     if (overlord_type[i] == 0x14 || overlord_type[i] == 0x18)
    //       return false;
    //   }
    //   return true;
    // }
    pub(super) fn sprite_check_if_overlords_clear(&self) -> bool {
        for i in (0..=7usize).rev() {
            if self.ram[OVERLORD_TYPE + i] == 0x14 || self.ram[OVERLORD_TYPE + i] == 0x18 {
                return false;
            }
        }
        true
    }

    // void Sprite_ManuallySetDeathFlagUW(int k) {  // 89c2f5
    //   if (!player_is_indoors || sprite_defl_bits[k] & 1 || sign8(sprite_N[k]))
    //     return;
    //   sprite_where_in_room[dungeon_room_index2] |= 1 << sprite_N[k];
    // }
    pub(super) fn sprite_manually_set_death_flag_uw(&mut self, k: usize) {
        if self.ram[PLAYER_IS_INDOORS] == 0
            || (self.ram[SPRITE_DEFL_BITS + k] & 1) != 0
            || sign8(self.ram[SPRITE_N + k])
        {
            return;
        }
        let room = read_le_u16(&self.ram, DUNGEON_ROOM_INDEX2);
        let bit = 1u16 << self.ram[SPRITE_N + k];
        let mask = self.sprite_where_in_room_mask(room) | bit;
        self.set_sprite_where_in_room_mask(room, mask);
    }

    // uint8 Sprite_ConvertVelocityToAngle(uint8 x, uint8 y) {  // 9df614
    //   static const uint8 kConvertVelocityToAngle_Tab0[32] = {
    //     0, 0, 1, 1, 1, 2, 2, 2, 0, 0, 15, 15, 15, 14, 14, 14,
    //     8, 8, 7, 7, 7, 6, 6, 6, 8, 8,  9,  9,  9, 10, 10, 10,
    //   };
    //   static const uint8 kConvertVelocityToAngle_Tab1[32] = {
    //     4, 4, 3, 3, 3, 2, 2, 2, 12, 12, 13, 13, 13, 14, 14, 14,
    //     4, 4, 5, 5, 5, 6, 6, 6, 12, 12, 11, 11, 11, 10, 10, 10,
    //   };
    //   int s = ((y >> 7) + (x >> 7) * 2) * 8;
    //   if (sign8(x)) x = -x;
    //   if (sign8(y)) y = -y;
    //   if (x >= y) {
    //     return kConvertVelocityToAngle_Tab0[(y >> 2) + s];
    //   } else {
    //     return kConvertVelocityToAngle_Tab1[(x >> 2) + s];
    //   }
    // }
    pub(super) fn sprite_convert_velocity_to_angle(x: u8, y: u8) -> u8 {
        const K_CONVERT_VELOCITY_TO_ANGLE_TAB0: [u8; 32] = [
            0, 0, 1, 1, 1, 2, 2, 2, 0, 0, 15, 15, 15, 14, 14, 14, 8, 8, 7, 7, 7, 6, 6, 6, 8, 8, 9,
            9, 9, 10, 10, 10,
        ];
        const K_CONVERT_VELOCITY_TO_ANGLE_TAB1: [u8; 32] = [
            4, 4, 3, 3, 3, 2, 2, 2, 12, 12, 13, 13, 13, 14, 14, 14, 4, 4, 5, 5, 5, 6, 6, 6, 12, 12,
            11, 11, 11, 10, 10, 10,
        ];

        let mut x = x;
        let mut y = y;
        let s = (((y >> 7) + (x >> 7) * 2) * 8) as usize;
        if sign8(x) {
            x = x.wrapping_neg();
        }
        if sign8(y) {
            y = y.wrapping_neg();
        }
        if x >= y {
            K_CONVERT_VELOCITY_TO_ANGLE_TAB0[((y >> 2) as usize) + s]
        } else {
            K_CONVERT_VELOCITY_TO_ANGLE_TAB1[((x >> 2) as usize) + s]
        }
    }

    // -----------------------------------------------------------------
    // `_for_helpers` adapters used by the canonical message helpers above.
    // -----------------------------------------------------------------

    fn sprite_prep_load_properties_for_helpers(&mut self, k: usize) {
        self.sprite_prep_load_properties(k);
    }

    fn sprite_check_damage_to_link_same_layer_for_helpers(&mut self, k: usize) -> bool {
        self.sprite_check_damage_to_link_same_layer(k)
    }

    fn sprite_check_if_link_is_busy_for_helpers(&self) -> bool {
        self.sprite_check_if_link_is_busy()
    }

    fn sprite_direction_to_face_link_for_helpers(&mut self, k: usize) -> u8 {
        self.sprite_direction_to_face_link(k, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_state() -> Box<ZeldaState> {
        Box::new(ZeldaState::new())
    }

    fn empty_hit_box() -> SpriteHitBox {
        SpriteHitBox {
            r0_xlo: 0,
            r8_xhi: 0,
            r1_ylo: 0,
            r9_yhi: 0,
            r2: 0,
            r3: 0,
            r4_spr_xlo: 0,
            r10_spr_xhi: 0,
            r5_spr_ylo: 0,
            r11_spr_yhi: 0,
            r6_spr_xsize: 0,
            r7_spr_ysize: 0,
        }
    }

    #[test]
    fn sprite_func3_sets_death_delay_and_flags() {
        let mut s = fresh_state();
        let k = 5;
        s.ram[SPRITE_STATE + k] = 9;
        s.ram[SPRITE_DELAY_MAIN + k] = 0xaa;
        s.ram[SPRITE_FLAGS2 + k] = 0xbb;

        s.sprite_func3(k);

        assert_eq!(s.ram[SPRITE_STATE + k], 6);
        assert_eq!(s.ram[SPRITE_DELAY_MAIN + k], 31);
        assert_eq!(s.ram[SPRITE_FLAGS2 + k], 3);
    }

    #[test]
    fn sprite_func8_resets_sound_then_queues_panned_sfx2() {
        let mut s = fresh_state();
        let k = 4;
        s.ram[SOUND_EFFECT_1] = 0xff;
        s.ram[SPRITE_STATE + k] = 9;
        s.ram[SPRITE_DELAY_MAIN + k] = 0;
        s.sprite_set_x(k, 0x0170);
        write_le_u16(&mut s.ram, BG2HOFS_COPY2, 0x0100);
        let expected_sfx = s.sprite_calculate_sfx_pan(k) | 0x20;

        s.sprite_func8(k);

        assert_eq!(s.ram[SPRITE_STATE + k], 1);
        assert_eq!(s.ram[SPRITE_DELAY_MAIN + k], 0x1f);
        assert_eq!(s.ram[SOUND_EFFECT_1], expected_sfx);
    }

    #[test]
    fn sprite_func22_sets_transition_state_and_advances_rng() {
        let mut s = fresh_state();
        let k = 6;
        s.ram[SPRITE_STATE + k] = 9;
        s.ram[SPRITE_DELAY_MAIN + k] = 0xaa;
        s.ram[SPRITE_AI_STATE_SPRITE + k] = 0xbb;
        s.ram[SPRITE_FLAGS2 + k] = 0xcc;
        s.sprite_set_x(k, 0x0040);
        write_le_u16(&mut s.ram, BG2HOFS_COPY2, 0x0000);
        let expected_sfx = s.sprite_calculate_sfx_pan(k) | 0x28;

        s.sprite_func22(k);

        assert_eq!(s.ram[SOUND_EFFECT_1], expected_sfx);
        assert_eq!(s.ram[SPRITE_STATE + k], 3);
        assert_eq!(s.ram[SPRITE_DELAY_MAIN + k], 15);
        assert_eq!(s.ram[SPRITE_AI_STATE_SPRITE + k], 0);
        assert_eq!(s.ram[SPRITE_FLAGS2 + k], 3);
    }

    #[test]
    fn throwable_scenery_transmute_if_valid_only_transmutes_throwable_scenery() {
        let k = 5;

        let mut other = fresh_state();
        other.ram[SPRITE_TYPE + k] = 0x12;
        other.ram[REPULSESPARK_TIMER_SPRITE] = 7;
        other.ram[SPRITE_DELAY_MAIN + k] = 0xaa;
        other.ram[SPRITE_STATE + k] = 9;
        other.ram[SPRITE_FLAGS2 + k] = 0x20;
        other.throwable_scenery_transmute_if_valid(k);
        assert_eq!(other.ram[REPULSESPARK_TIMER_SPRITE], 7);
        assert_eq!(other.ram[SPRITE_DELAY_MAIN + k], 0xaa);
        assert_eq!(other.ram[SPRITE_STATE + k], 9);
        assert_eq!(other.ram[SPRITE_FLAGS2 + k], 0x20);

        let mut scenery = fresh_state();
        scenery.ram[SPRITE_TYPE + k] = 0xec;
        scenery.ram[REPULSESPARK_TIMER_SPRITE] = 7;
        scenery.ram[SPRITE_DELAY_MAIN + k] = 0xaa;
        scenery.ram[SPRITE_STATE + k] = 9;
        scenery.ram[SPRITE_FLAGS2 + k] = 0x20;
        scenery.throwable_scenery_transmute_if_valid(k);
        assert_eq!(scenery.ram[REPULSESPARK_TIMER_SPRITE], 0);
        assert_eq!(scenery.ram[SOUND_EFFECT_1] & 0x3f, 0x1f);
        assert_eq!(scenery.ram[SPRITE_DELAY_MAIN + k], 31);
        assert_eq!(scenery.ram[SPRITE_STATE + k], 6);
        assert_eq!(scenery.ram[SPRITE_FLAGS2 + k], 0x24);
    }

    #[test]
    fn sprite_apply_ricochet_inverts_halves_and_transmutes_if_valid() {
        let k = 5;
        let mut s = fresh_state();
        s.ram[SPRITE_TYPE + k] = 0xec;
        s.ram[SPRITE_X_VEL + k] = 0x10;
        s.ram[SPRITE_Y_VEL + k] = 0xf0;
        s.ram[REPULSESPARK_TIMER_SPRITE] = 9;
        s.ram[SPRITE_FLAGS2 + k] = 0x03;

        s.sprite_apply_ricochet(k);

        assert_eq!(s.ram[SPRITE_X_VEL + k], 0xf8);
        assert_eq!(s.ram[SPRITE_Y_VEL + k], 0x08);
        assert_eq!(s.ram[REPULSESPARK_TIMER_SPRITE], 0);
        assert_eq!(s.ram[SPRITE_DELAY_MAIN + k], 31);
        assert_eq!(s.ram[SPRITE_STATE + k], 6);
        assert_eq!(s.ram[SPRITE_FLAGS2 + k], 0x07);
    }

    #[test]
    fn sprite_func18_changes_type_resets_damage_and_spawns_poof_garnish() {
        let mut s = fresh_state();
        let k = 4;
        s.ram[SPRITE_TYPE + k] = 0x12;
        s.ram[SPRITE_SUBTYPE + k] = 0xaa;
        s.ram[SPRITE_DIE_ACTION + k] = 0xbb;
        s.ram[SOUND_EFFECT_2] = 0xff;
        s.ram[SPRITE_HIT_TIMER + k] = 0xcc;
        s.ram[SPRITE_GIVE_DAMAGE_SPRITE + k] = 0xdd;
        s.sprite_set_x(k, 0x0123);
        s.sprite_set_y(k, 0x0340);
        s.ram[SPRITE_FLOOR + k] = 2;

        s.sprite_func18(k, 0xe3);

        assert_eq!(s.ram[SPRITE_TYPE + k], 0xe3);
        assert_eq!(s.ram[SPRITE_SUBTYPE + k], 0xaa);
        assert_eq!(s.ram[SPRITE_DIE_ACTION + k], 0xbb);
        assert_eq!(s.ram[SOUND_EFFECT_2] & 0x3f, 0x32);
        assert_eq!(s.ram[SPRITE_HIT_TIMER + k], 0);
        assert_eq!(s.ram[SPRITE_GIVE_DAMAGE_SPRITE + k], 0);

        assert_eq!(s.ram[GARNISH_ACTIVE_SPRITE], 10);
        assert_eq!(s.ram[GARNISH_TYPE + 29], 10);
        assert_eq!(s.ram[GARNISH_X_LO_SPRITE + 29], 0x23);
        assert_eq!(s.ram[GARNISH_X_HI_SPRITE + 29], 0x01);
        assert_eq!(s.ram[GARNISH_Y_LO_SPRITE + 29], 0x50);
        assert_eq!(s.ram[GARNISH_Y_HI_SPRITE + 29], 0x03);
        assert_eq!(s.ram[GARNISH_SPRITE_SPRITE + 29], 2);
        assert_eq!(s.ram[GARNISH_COUNTDOWN_SPRITE + 29], 15);
    }

    #[test]
    fn sprite_apply_conveyor_skips_even_frames() {
        let mut s = fresh_state();
        let k = 3;
        s.ram[FRAME_COUNTER] = 0;
        s.sprite_set_x(k, 0x0100);
        s.sprite_set_y(k, 0x0200);

        s.sprite_apply_conveyor(k, 0x68);

        assert_eq!(s.sprite_get_x(k), 0x0100);
        assert_eq!(s.sprite_get_y(k), 0x0200);
    }

    #[test]
    fn sprite_apply_conveyor_moves_by_direction_table_on_odd_frames() {
        for (j, expected_x, expected_y) in [
            (0x68, 0x0100, 0x01ff),
            (0x69, 0x0100, 0x0201),
            (0x6a, 0x00ff, 0x0200),
            (0x6b, 0x0101, 0x0200),
        ] {
            let mut s = fresh_state();
            let k = 3;
            s.ram[FRAME_COUNTER] = 1;
            s.sprite_set_x(k, 0x0100);
            s.sprite_set_y(k, 0x0200);

            s.sprite_apply_conveyor(k, j);

            assert_eq!(s.sprite_get_x(k), expected_x);
            assert_eq!(s.sprite_get_y(k), expected_y);
        }
    }

    #[test]
    fn sprite_add_xy_applies_signed_offsets_to_16_bit_coords() {
        let mut s = fresh_state();
        let k = 2;
        s.sprite_set_x(k, 0x0100);
        s.sprite_set_y(k, 0x0200);

        s.sprite_add_xy(k, -3, 5);

        assert_eq!(s.sprite_get_x(k), 0x00fd);
        assert_eq!(s.sprite_get_y(k), 0x0205);
    }

    #[test]
    fn sprite_fall_adjust_position_adds_signed_floor_velocity() {
        let mut s = fresh_state();
        let k = 2;
        s.sprite_set_x(k, 0x0100);
        s.sprite_set_y(k, 0x0200);
        write_le_u16(&mut s.ram, DUNG_FLOOR_X_VEL, 0xfffe);
        write_le_u16(&mut s.ram, DUNG_FLOOR_Y_VEL, 0x0003);

        s.sprite_fall_adjust_position(k);

        assert_eq!(s.sprite_get_x(k), 0x00fe);
        assert_eq!(s.sprite_get_y(k), 0x0203);
    }

    #[test]
    fn sprite_move_xyz_updates_z_then_x_then_y_subpixels() {
        let mut s = fresh_state();
        let k = 4;
        s.sprite_set_x(k, 0x0100);
        s.sprite_set_y(k, 0x0200);
        s.ram[SPRITE_X_SUBPIXEL + k] = 0xf0;
        s.ram[SPRITE_Y_SUBPIXEL + k] = 0x10;
        s.ram[SPRITE_Z + k] = 0x03;
        s.ram[SPRITE_Z_SUBPOS + k] = 0xf0;
        s.ram[SPRITE_X_VEL + k] = 0x02;
        s.ram[SPRITE_Y_VEL + k] = 0xfe;
        s.ram[SPRITE_Z_VEL + k] = 0x02;

        s.sprite_move_xyz(k);

        assert_eq!(s.sprite_get_x(k), 0x0101);
        assert_eq!(s.ram[SPRITE_X_SUBPIXEL + k], 0x10);
        assert_eq!(s.sprite_get_y(k), 0x01ff);
        assert_eq!(s.ram[SPRITE_Y_SUBPIXEL + k], 0xf0);
        assert_eq!(s.ram[SPRITE_Z + k], 0x04);
        assert_eq!(s.ram[SPRITE_Z_SUBPOS + k], 0x10);
    }

    #[test]
    fn alloc_overlord_returns_highest_free_slot_or_negative_one() {
        let mut s = fresh_state();
        assert_eq!(s.alloc_overlord(), 7);

        s.ram[OVERLORD_TYPE + 7] = 1;
        s.ram[OVERLORD_TYPE + 6] = 1;
        assert_eq!(s.alloc_overlord(), 5);

        for i in 0..8 {
            s.ram[OVERLORD_TYPE + i] = 1;
        }
        assert_eq!(s.alloc_overlord(), -1);
    }

    #[test]
    fn overworld_alloc_sprite_matches_start_slots_and_reuse_rule() {
        let mut s = fresh_state();
        s.ram[SPRITE_STATE..SPRITE_STATE + 16].fill(1);
        s.ram[SPRITE_STATE + 13] = 0;
        assert_eq!(s.overworld_alloc_sprite(0x01), 13);

        s.ram[SPRITE_STATE + 13] = 1;
        s.ram[SPRITE_TYPE + 12] = 0x41;
        s.ram[SPRITE_C_SPRITE + 12] = 2;
        assert_eq!(s.overworld_alloc_sprite(0x01), 12);

        let mut special = fresh_state();
        special.ram[SPRITE_STATE..SPRITE_STATE + 16].fill(1);
        special.ram[SPRITE_STATE + 4] = 0;
        assert_eq!(special.overworld_alloc_sprite(0x58), 4);

        let mut full = fresh_state();
        full.ram[SPRITE_STATE..SPRITE_STATE + 16].fill(1);
        assert_eq!(full.overworld_alloc_sprite(0xd0), -1);
    }

    #[test]
    fn dungeon_load_single_overlord_allocates_and_initializes_coords() {
        let mut s = fresh_state();
        s.ram[OVERLORD_TYPE + 7] = 1;
        s.ram[SPRITE_ROOM_ORIGIN_Y_HI] = 0x20;
        s.ram[SPRITE_ROOM_ORIGIN_X_HI] = 0x10;
        write_le_u16(&mut s.ram, OVERWORLD_AREA_INDEX_SPRITE, 0x1234);

        s.dungeon_load_single_overlord(&[0x83, 0xe4, 10]);

        assert_eq!(s.ram[OVERLORD_TYPE + 6], 10);
        assert_eq!(s.ram[OVERLORD_FLOOR_SPRITE + 6], 1);
        assert_eq!(s.ram[SPRITE_OVERLORD_Y_LO + 6], 0x30);
        assert_eq!(s.ram[SPRITE_OVERLORD_Y_HI + 6], 0x20);
        assert_eq!(s.ram[SPRITE_OVERLORD_X_LO + 6], 0x40);
        assert_eq!(s.ram[SPRITE_OVERLORD_X_HI + 6], 0x10);
        assert_eq!(s.ram[OVERLORD_SPAWNED_IN_AREA_SPRITE + 6], 0x34);
        assert_eq!(s.ram[OVERLORD_GEN1 + 6], 0);
        assert_eq!(s.ram[OVERLORD_GEN2 + 6], 160);
        assert_eq!(s.ram[OVERLORD_GEN3_SPRITE + 6], 0);

        let mut trap = fresh_state();
        trap.ram[SPRITE_ROOM_ORIGIN_X_HI] = 0x10;
        trap.dungeon_load_single_overlord(&[0x00, 0xe0, 3]);
        assert_eq!(trap.ram[OVERLORD_TYPE + 7], 3);
        assert_eq!(trap.ram[OVERLORD_GEN2 + 7], 255);
        assert_eq!(trap.ram[SPRITE_OVERLORD_X_LO + 7], 0xf8);
    }

    #[test]
    fn sprite_initialize_slots_clears_stale_sprite_and_overlord_slots() {
        let mut s = fresh_state();
        s.ram[OVERWORLD_AREA_INDEX_SPRITE] = 0x34;
        s.ram[LINK_PICKING_THROW_STATE] = 7;
        s.ram[LINK_STATE_BITS] = 0x80;

        s.ram[SPRITE_STATE + 1] = 10;
        s.ram[SPRITE_TYPE + 1] = 0x20;
        s.ram[SPRITE_STATE + 2] = 10;
        s.ram[SPRITE_TYPE + 2] = 0xec;
        s.ram[SPRITE_STATE + 3] = 9;
        s.ram[SPRITE_TYPE + 3] = 0x20;
        s.ram[SPRITE_ROOM + 3] = 0x12;
        s.ram[SPRITE_STATE + 4] = 9;
        s.ram[SPRITE_TYPE + 4] = 0x20;
        s.ram[SPRITE_ROOM + 4] = 0x34;
        s.ram[SPRITE_STATE + 5] = 9;
        s.ram[SPRITE_TYPE + 5] = 0x6c;
        s.ram[SPRITE_ROOM + 5] = 0x12;
        s.ram[OVERLORD_TYPE + 1] = 0x14;
        s.ram[OVERLORD_SPAWNED_IN_AREA_SPRITE + 1] = 0x12;
        s.ram[OVERLORD_TYPE + 2] = 0x14;
        s.ram[OVERLORD_SPAWNED_IN_AREA_SPRITE + 2] = 0x34;

        s.sprite_initialize_slots();

        assert_eq!(s.ram[SPRITE_STATE + 1], 0);
        assert_eq!(s.ram[LINK_PICKING_THROW_STATE], 0);
        assert_eq!(s.ram[LINK_STATE_BITS], 0);
        assert_eq!(s.ram[SPRITE_STATE + 2], 10);
        assert_eq!(s.ram[SPRITE_STATE + 3], 0);
        assert_eq!(s.ram[SPRITE_STATE + 4], 9);
        assert_eq!(s.ram[SPRITE_STATE + 5], 9);
        assert_eq!(s.ram[OVERLORD_TYPE + 1], 0);
        assert_eq!(s.ram[OVERLORD_TYPE + 2], 0x14);
    }

    #[test]
    fn sprite_initialize_mirror_portal_replaces_existing_portal_and_sets_travel_coords() {
        let mut s = fresh_state();
        s.ram[SPRITE_STATE + 4] = 9;
        s.ram[SPRITE_TYPE + 4] = 0x6c;
        s.ram[SPRITE_STATE + 15] = 1;
        s.ram[BIRD_TRAVEL_X_LO + 15] = 0x34;
        s.ram[BIRD_TRAVEL_X_HI + 15] = 0x12;
        s.ram[BIRD_TRAVEL_Y_LO + 15] = 0xf8;
        s.ram[BIRD_TRAVEL_Y_HI + 15] = 0x01;

        s.sprite_initialize_mirror_portal();

        assert_eq!(s.ram[SPRITE_STATE + 4], 0);
        assert_eq!(s.ram[SPRITE_TYPE + 14], 0x6c);
        assert_eq!(s.ram[SPRITE_STATE + 14], 9);
        assert_eq!(s.sprite_get_x(14), 0x1234);
        assert_eq!(s.sprite_get_y(14), 0x0200);
        assert_eq!(s.ram[SPRITE_FLOOR + 14], 0);
        assert_eq!(s.ram[SPRITE_IGNORE_PROJECTILE + 14], 1);

        let mut full = fresh_state();
        full.ram[SPRITE_STATE..SPRITE_STATE + 16].fill(9);
        full.ram[SPRITE_STATE] = 7;
        full.ram[BIRD_TRAVEL_X_LO + 15] = 0xcd;
        full.ram[BIRD_TRAVEL_X_HI + 15] = 0xab;
        full.ram[BIRD_TRAVEL_Y_LO + 15] = 0x01;
        full.ram[BIRD_TRAVEL_Y_HI + 15] = 0x02;
        full.sprite_initialize_mirror_portal();
        assert_eq!(full.sprite_get_x(0), 0xabcd);
        assert_eq!(full.sprite_get_y(0), 0x0209);
        assert_eq!(full.ram[SPRITE_FLOOR], 0);
        assert_eq!(full.ram[SPRITE_IGNORE_PROJECTILE], 1);
    }

    #[test]
    fn sprite_reset_all_no_disable_clears_reset_state_without_disabling_sprites() {
        let mut s = fresh_state();
        s.ram[HAUNTED_GROVE_FLUTE_EVENT_LATCH] = 1;
        s.ram[SPRITE_ALERT_FLAG] = 2;
        s.ram[OVERWORLD_BOULDER_TRAP_COUNT] = 3;
        s.ram[MESSAGE_OR_SPRITE_STATE_CACHE] = 4;
        s.ram[SPRITE_CHR_HALFSLOT_STATE] = 5;
        s.ram[SPRITE_LIMIT_INSTANCE] = 6;
        s.ram[SORT_SPRITES_SETTING] = 7;
        s.ram[FOLLOWER_INDICATOR] = 12;
        s.ram[SUPER_BOMB_INDICATOR_TIMER] = 0x55;
        s.ram[SPRITE_STATE + 3] = 9;
        s.ram[ANCILLA_TYPE + 2] = 0x27;
        write_le_u16(&mut s.ram, SPRITE_WHERE_IN_ROOM + 0x123 * 2, 0x55aa);
        s.ram[OVERWORLD_SPRITE_WAS_LOADED + 0x42] = 0xbb;
        write_le_u16(&mut s.ram, DUNGEON_ROOM_HISTORY + 2, 0x1234);

        s.sprite_reset_all_no_disable();

        assert_eq!(s.ram[HAUNTED_GROVE_FLUTE_EVENT_LATCH], 0);
        assert_eq!(s.ram[SPRITE_ALERT_FLAG], 0);
        assert_eq!(s.ram[OVERWORLD_BOULDER_TRAP_COUNT], 0);
        assert_eq!(s.ram[MESSAGE_OR_SPRITE_STATE_CACHE], 0);
        assert_eq!(s.ram[SPRITE_CHR_HALFSLOT_STATE], 0);
        assert_eq!(s.ram[SPRITE_LIMIT_INSTANCE], 0);
        assert_eq!(s.ram[SORT_SPRITES_SETTING], 0);
        assert_eq!(s.ram[SUPER_BOMB_INDICATOR_TIMER], 0xfe);
        assert_eq!(read_le_u16(&s.ram, SPRITE_WHERE_IN_ROOM + 0x123 * 2), 0);
        assert_eq!(s.ram[OVERWORLD_SPRITE_WAS_LOADED + 0x42], 0);
        assert_eq!(read_le_u16(&s.ram, DUNGEON_ROOM_HISTORY + 2), 0xffff);
        assert_eq!(s.ram[SPRITE_STATE + 3], 9);
        assert_eq!(s.ram[ANCILLA_TYPE + 2], 0x27);

        let mut follower = fresh_state();
        follower.ram[FOLLOWER_INDICATOR] = 13;
        follower.ram[SUPER_BOMB_INDICATOR_TIMER] = 0x55;
        follower.sprite_reset_all_no_disable();
        assert_eq!(follower.ram[SUPER_BOMB_INDICATOR_TIMER], 0x55);
    }

    #[test]
    fn dungeon_load_single_sprite_preserves_c_tmp_counter_side_effect() {
        let mut s = fresh_state();
        write_le_u16(&mut s.ram, DUNGEON_ROOM_INDEX2, 0x004a);
        s.ram[SPRITE_ROOM_ORIGIN_Y_HI] = 0x08;
        s.ram[SPRITE_ROOM_ORIGIN_X_HI] = 0x04;

        let next = s.dungeon_load_single_sprite(3, 0xa0, 0x60, 0x2f);

        assert_eq!(next, 3);
        assert_eq!(s.ram[SPRITE_STATE + 3], 8);
        assert_eq!(s.ram[SPRITE_FLOOR + 3], 1);
        assert_eq!(s.ram[SPRITE_SHARED_SCRATCH_A], 0x60);
        assert_eq!(s.ram[TMP_COUNTER_SPRITE], 0x08);
        assert_eq!(s.ram[SPRITE_SUBTYPE + 3], 0x0b);
    }

    #[test]
    fn garnish_get_x_and_y_read_16_bit_coords() {
        let mut s = fresh_state();
        let k = 7;
        s.ram[GARNISH_X_LO_SPRITE + k] = 0x34;
        s.ram[GARNISH_X_HI_SPRITE + k] = 0x12;
        s.ram[GARNISH_Y_LO_SPRITE + k] = 0xcd;
        s.ram[GARNISH_Y_HI_SPRITE + k] = 0xab;

        assert_eq!(s.garnish_get_x(k), 0x1234);
        assert_eq!(s.garnish_get_y(k), 0xabcd);
    }

    #[test]
    fn sprite_inactive_sprite_invalidates_room_or_overworld_slot_marker() {
        let mut outdoor = fresh_state();
        let k = 5;
        outdoor.ram[PLAYER_IS_INDOORS] = 0;
        outdoor.ram[SPRITE_N_WORD + k * 2] = 0x34;
        outdoor.ram[SPRITE_N_WORD + k * 2 + 1] = 0x12;
        outdoor.sprite_inactive_sprite(k);
        assert_eq!(read_le_u16(&outdoor.ram, SPRITE_N_WORD + k * 2), 0xffff);

        let mut indoor = fresh_state();
        indoor.ram[PLAYER_IS_INDOORS] = 1;
        indoor.ram[SPRITE_N + k] = 0x34;
        indoor.sprite_inactive_sprite(k);
        assert_eq!(indoor.ram[SPRITE_N + k], 0xff);
    }

    #[test]
    fn sprite_get_tile_attribute_reads_indoor_floor_table_and_caches_type() {
        let mut s = fresh_state();
        let k = 5;
        s.ram[PLAYER_IS_INDOORS] = 1;
        s.ram[SPRITE_FLOOR + k] = 1;
        let mut x = 0x0128;
        let y = 0x0030;
        let offset = 0x1000 + (((x & 0x01f8) >> 3) as usize) + (((y & 0x01f8) << 3) as usize);
        s.ram[DUNG_BG2_ATTR_TABLE + offset] = 0x72;

        assert_eq!(s.sprite_get_tile_attribute(k, &mut x, y), 0x72);

        assert_eq!(x, 0x0128);
        assert_eq!(s.ram[SPRITE_TILETYPE_SPR], 0x72);

        let mut floor0_x = 0x0008;
        s.ram[DUNG_BG2_ATTR_TABLE + 1] = 0x34;
        assert_eq!(s.GetTileAttribute(0, &mut floor0_x, 0), 0x34);
        assert_eq!(floor0_x, 0x0008);
        assert_eq!(s.ram[SPRITE_TILETYPE_SPR], 0x34);

        s.ram[PLAYER_IS_INDOORS] = 0;
        let mut outdoor_x = 0x0128;
        let outdoor_y = 0x0040;
        let expected = s.overworld_get_tile_attribute_at_location(outdoor_x >> 3, outdoor_y);
        assert_eq!(s.GetTileAttribute(0, &mut outdoor_x, outdoor_y), expected);
        assert_eq!(outdoor_x, 0x0025);
        assert_eq!(s.ram[SPRITE_TILETYPE_SPR], expected);
    }

    #[test]
    fn link_setup_hit_box_matches_c_offsets_and_disabled_sentinel() {
        let mut s = fresh_state();
        write_le_u16(&mut s.ram, LINK_X_COORD, 0x12fc);
        write_le_u16(&mut s.ram, LINK_Y_COORD, 0x34f9);
        let mut hb = empty_hit_box();

        s.link_setup_hit_box(&mut hb);

        assert_eq!(hb.r2, 8);
        assert_eq!(hb.r3, 8);
        assert_eq!(hb.r0_xlo, 0x00);
        assert_eq!(hb.r8_xhi, 0x13);
        assert_eq!(hb.r1_ylo, 0x01);
        assert_eq!(hb.r9_yhi, 0x35);

        s.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
        hb.r0_xlo = 0xaa;
        hb.r8_xhi = 0xbb;
        hb.r1_ylo = 0xcc;
        hb.r9_yhi = 0xdd;
        s.link_setup_hit_box_conditional(&mut hb);
        assert_eq!(hb.r0_xlo, 0xaa);
        assert_eq!(hb.r8_xhi, 0xbb);
        assert_eq!(hb.r1_ylo, 0xcc);
        assert_eq!(hb.r9_yhi, 0x80);

        s.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
        s.link_setup_hit_box_conditional(&mut hb);
        assert_eq!(hb.r0_xlo, 0x00);
        assert_eq!(hb.r8_xhi, 0x13);
        assert_eq!(hb.r1_ylo, 0x01);
        assert_eq!(hb.r9_yhi, 0x35);
    }

    #[test]
    fn sprite_setup_hit_box00_uses_current_sprite_link_bounds_and_z() {
        let mut s = fresh_state();
        let k = 5;
        write_le_u16(&mut s.ram, CUR_SPRITE_X, 0x0100);
        write_le_u16(&mut s.ram, CUR_SPRITE_Y, 0x0200);
        write_le_u16(&mut s.ram, LINK_X_COORD, 0x0100);
        write_le_u16(&mut s.ram, LINK_Y_COORD, 0x0200);

        assert!(s.sprite_setup_hit_box00(k));

        write_le_u16(&mut s.ram, LINK_X_COORD, 0x010c);
        assert!(!s.sprite_setup_hit_box00(k));

        write_le_u16(&mut s.ram, LINK_X_COORD, 0x0100);
        write_le_u16(&mut s.ram, LINK_Y_COORD, 0x0208);
        assert!(!s.sprite_setup_hit_box00(k));

        write_le_u16(&mut s.ram, LINK_Y_COORD, 0x01f9);
        s.ram[SPRITE_Z + k] = 7;
        assert!(s.sprite_setup_hit_box00(k));
    }

    #[test]
    fn sprite_place_rupulse_spark_2_sets_visible_sprite_position() {
        let mut s = fresh_state();
        let k = 5;
        write_le_u16(&mut s.ram, BG2HOFS_COPY2, 0x0100);
        write_le_u16(&mut s.ram, BG2VOFS_COPY2, 0x0200);
        s.sprite_set_x(k, 0x0184);
        s.sprite_set_y(k, 0x027f);
        s.ram[SPRITE_FLOOR + k] = 2;

        s.sprite_place_rupulse_spark_2(k);

        assert_eq!(s.ram[REPULSESPARK_X_LO_SPRITE], 0x84);
        assert_eq!(s.ram[REPULSESPARK_Y_LO_SPRITE], 0x7f);
        assert_eq!(s.ram[REPULSESPARK_TIMER_SPRITE], 5);
        assert_eq!(s.ram[REPULSESPARK_FLOOR_STATUS_SPRITE], 2);

        let mut offscreen = fresh_state();
        offscreen.sprite_set_x(k, 0x0200);
        offscreen.sprite_set_y(k, 0x0000);
        offscreen.sprite_place_rupulse_spark_2(k);
        assert_eq!(offscreen.ram[REPULSESPARK_TIMER_SPRITE], 0);
    }

    #[test]
    fn sprite_place_weapon_tink_respects_active_repulsespark_timer() {
        let mut active = fresh_state();
        let k = 5;
        active.ram[REPULSESPARK_TIMER_SPRITE] = 3;
        active.ram[SOUND_EFFECT_1] = 0;
        active.sprite_place_weapon_tink(k);
        assert_eq!(active.ram[REPULSESPARK_TIMER_SPRITE], 3);
        assert_eq!(active.ram[SOUND_EFFECT_1], 0);

        let mut s = fresh_state();
        s.sprite_set_x(k, 0x0050);
        s.sprite_set_y(k, 0x0060);
        s.ram[SPRITE_FLOOR + k] = 1;
        s.sprite_place_weapon_tink(k);
        assert_eq!(s.ram[REPULSESPARK_TIMER_SPRITE], 5);
        assert_eq!(s.ram[REPULSESPARK_X_LO_SPRITE], 0x50);
        assert_eq!(s.ram[REPULSESPARK_Y_LO_SPRITE], 0x60);
        assert_eq!(s.ram[REPULSESPARK_FLOOR_STATUS_SPRITE], 1);
        assert_eq!(s.ram[SOUND_EFFECT_1], 5);
    }

    #[test]
    fn link_place_weapon_tink_uses_link_oam_offsets_and_x_carry() {
        let mut active = fresh_state();
        active.ram[REPULSESPARK_TIMER_SPRITE] = 3;
        active.ram[LINK_X_COORD] = 0xf0;
        active.ram[PLAYER_OAM_X_OFFSET] = 0x20;
        active.link_place_weapon_tink();
        assert_eq!(active.ram[REPULSESPARK_TIMER_SPRITE], 3);
        assert_eq!(active.ram[REPULSESPARK_X_LO_SPRITE], 0);
        assert_eq!(active.ram[SOUND_EFFECT_1], 0);

        let mut s = fresh_state();
        s.ram[LINK_X_COORD] = 0xf0;
        s.ram[LINK_X_COORD + 1] = 0x01;
        s.ram[LINK_Y_COORD] = 0x20;
        s.ram[PLAYER_OAM_X_OFFSET] = 0x20;
        s.ram[PLAYER_OAM_Y_OFFSET] = 0x30;
        s.ram[LINK_IS_ON_LOWER_LEVEL] = 2;

        s.link_place_weapon_tink();

        assert_eq!(s.ram[REPULSESPARK_TIMER_SPRITE], 5);
        assert_eq!(s.ram[REPULSESPARK_X_LO_SPRITE], 0x10);
        assert_eq!(s.ram[REPULSESPARK_Y_LO_SPRITE], 0x51);
        assert_eq!(s.ram[REPULSESPARK_FLOOR_STATUS_SPRITE], 2);
        assert_eq!(s.ram[SOUND_EFFECT_1], s.link_calculate_sfx_pan() | 5);
    }

    #[test]
    fn sprite_apply_recoil_to_link_projects_speed_and_resets_z_coord() {
        let mut s = fresh_state();
        let k = 4;
        s.sprite_set_x(k, 0x0100);
        s.sprite_set_y(k, 0x0200);
        write_le_u16(&mut s.ram, LINK_X_COORD, 0x0140);
        write_le_u16(&mut s.ram, LINK_Y_COORD, 0x01d0);
        s.ram[SPRITE_Z + k] = 4;
        write_le_u16(&mut s.ram, LINK_Z_COORD, 0x1234);

        let expected = s.sprite_project_speed_towards_link(k, 0x30);
        s.sprite_apply_recoil_to_link(k, 0x30);

        assert_eq!(s.ram[LINK_ACTUAL_VEL_X], expected.x);
        assert_eq!(s.ram[LINK_ACTUAL_VEL_Y], expected.y);
        assert_eq!(s.ram[LINK_ACTUAL_VEL_Z], 0x18);
        assert_eq!(s.ram[LINK_RECOIL_Z_VEL], 0x18);
        assert_eq!(read_le_u16(&s.ram, LINK_Z_COORD), 0);
    }

    #[test]
    fn sprite_direction_to_face_link_matches_c_axis_and_coords_output() {
        let mut s = fresh_state();
        let k = 4;
        s.sprite_set_x(k, 0x0100);
        s.sprite_set_y(k, 0x0200);
        write_le_u16(&mut s.ram, LINK_X_COORD, 0x0120);
        write_le_u16(&mut s.ram, LINK_Y_COORD, 0x0204);
        let mut coords = PointU8 { x: 0, y: 0 };

        assert_eq!(s.sprite_direction_to_face_link(k, Some(&mut coords)), 0);
        assert_eq!(coords, PointU8 { x: 0x20, y: 0x0c });
        assert_eq!(s.ram[TMP_COUNTER_SPRITE], 0x0c);

        write_le_u16(&mut s.ram, LINK_X_COORD, 0x00f8);
        write_le_u16(&mut s.ram, LINK_Y_COORD, 0x0240);
        s.ram[SPRITE_Z + k] = 0;
        assert_eq!(s.sprite_direction_to_face_link(k, None), 2);
        assert_eq!(s.ram[TMP_COUNTER_SPRITE], 0x48);
    }

    #[test]
    fn sprite_do_hit_boxes_fast_uses_dungmap_offsets_and_large_type_size() {
        let mut s = fresh_state();
        let k = 4;
        s.sprite_set_x(k, 0x0120);
        s.sprite_set_y(k, 0x0202);
        s.ram[DUNGMAP_VAR8] = 0xfc;
        s.ram[DUNGMAP_VAR8 + 1] = 0x08;
        let mut hb = empty_hit_box();

        s.sprite_do_hit_boxes_fast(k, &mut hb);

        assert_eq!(hb.r4_spr_xlo, 0x28);
        assert_eq!(hb.r10_spr_xhi, 0x01);
        assert_eq!(hb.r5_spr_ylo, 0xfe);
        assert_eq!(hb.r11_spr_yhi, 0x01);
        assert_eq!(hb.r6_spr_xsize, 3);
        assert_eq!(hb.r7_spr_ysize, 3);

        s.ram[SPRITE_TYPE + k] = 0x6a;
        s.ram[DUNGMAP_VAR8] = 0x02;
        s.ram[DUNGMAP_VAR8 + 1] = 0xfe;
        s.sprite_do_hit_boxes_fast(k, &mut hb);
        assert_eq!(hb.r4_spr_xlo, 0x1e);
        assert_eq!(hb.r10_spr_xhi, 0x01);
        assert_eq!(hb.r5_spr_ylo, 0x04);
        assert_eq!(hb.r11_spr_yhi, 0x02);
        assert_eq!(hb.r6_spr_xsize, 16);
        assert_eq!(hb.r7_spr_ysize, 16);

        hb.r10_spr_xhi = 0x12;
        s.ram[DUNGMAP_VAR8 + 1] = 0x80;
        s.sprite_do_hit_boxes_fast(k, &mut hb);
        assert_eq!(hb.r10_spr_xhi, 0x80);
    }

    #[test]
    fn sprite_correct_oam_entries_recomputes_ext_bits_and_hides_offscreen_y() {
        let mut s = fresh_state();
        let k = 4;
        s.sprite_set_x(k, 0x0120);
        s.sprite_set_y(k, 0x0200);
        write_le_u16(&mut s.ram, BG2HOFS_COPY2, 0x0100);
        write_le_u16(&mut s.ram, BG2VOFS_COPY2, 0x0200);
        write_le_u16(&mut s.ram, OAM_CUR_PTR, OAM_BUF as u16);
        write_le_u16(&mut s.ram, OAM_EXT_CUR_PTR, BYTEWISE_EXTENDED_OAM as u16);
        s.ram[OAM_BUF] = 0x30;
        s.ram[OAM_BUF + 1] = 0x04;
        s.ram[OAM_BUF + 4] = 0xf0;
        s.ram[OAM_BUF + 5] = 0xef;
        s.ram[BYTEWISE_EXTENDED_OAM] = 2;
        s.ram[BYTEWISE_EXTENDED_OAM + 1] = 0;

        s.sprite_correct_oam_entries(k, 1, 0xff);

        assert_eq!(s.ram[BYTEWISE_EXTENDED_OAM], 2);
        assert_eq!(s.ram[BYTEWISE_EXTENDED_OAM + 1], 1);
        assert_eq!(s.ram[OAM_BUF + 1], 0x04);
        assert_eq!(s.ram[OAM_BUF + 5], 0xf0);

        s.ram[OAM_BUF] = 0x30;
        s.ram[OAM_BUF + 1] = 0x04;
        s.ram[BYTEWISE_EXTENDED_OAM] = 2;
        s.sprite_correct_oam_entries(k, 0, 0);
        assert_eq!(s.ram[BYTEWISE_EXTENDED_OAM], 0);
    }

    #[test]
    fn sprite_kill_self_matches_indoor_guard_and_loaded_bit_clear() {
        let k = 5;

        let mut guarded = fresh_state();
        guarded.ram[PLAYER_IS_INDOORS] = 1;
        guarded.ram[SPRITE_STATE + k] = 9;
        guarded.ram[SPRITE_N + k] = 0x12;
        guarded.sprite_kill_self(k);
        assert_eq!(guarded.ram[SPRITE_STATE + k], 9);
        assert_eq!(guarded.ram[SPRITE_N + k], 0x12);

        let mut indoor_allowed = fresh_state();
        indoor_allowed.ram[PLAYER_IS_INDOORS] = 1;
        indoor_allowed.ram[SPRITE_DEFL_BITS + k] = 0x40;
        indoor_allowed.ram[SPRITE_STATE + k] = 9;
        indoor_allowed.ram[SPRITE_N + k] = 0x12;
        indoor_allowed.sprite_kill_self(k);
        assert_eq!(indoor_allowed.ram[SPRITE_STATE + k], 0);
        assert_eq!(indoor_allowed.ram[SPRITE_N + k], 0xff);

        let mut outdoor = fresh_state();
        outdoor.ram[SPRITE_STATE + k] = 9;
        write_le_u16(&mut outdoor.ram, SPRITE_N_WORD + k * 2, 0x0012);
        outdoor.ram[OVERWORLD_SPRITE_WAS_LOADED + 2] = 0xff;
        outdoor.sprite_kill_self(k);
        assert_eq!(outdoor.ram[SPRITE_STATE + k], 0);
        assert_eq!(outdoor.ram[0], 0x12);
        assert_eq!(read_le_u16(&outdoor.ram, 1), 0xef82);
        assert_eq!(outdoor.ram[OVERWORLD_SPRITE_WAS_LOADED + 2], 0xdf);
        assert_eq!(read_le_u16(&outdoor.ram, SPRITE_N_WORD + k * 2), 0xffff);

        let mut wrapped = fresh_state();
        wrapped.ram[SPRITE_STATE + k] = 9;
        write_le_u16(&mut wrapped.ram, SPRITE_N_WORD + k * 2, 0xff00);
        wrapped.ram[(OVERWORLD_SPRITE_WAS_LOADED + (0xff00 >> 3)) & 0x1ffff] = 0xff;
        wrapped.sprite_kill_self(k);
        assert_eq!(
            wrapped.ram[(OVERWORLD_SPRITE_WAS_LOADED + (0xff00 >> 3)) & 0x1ffff],
            0x7f
        );
    }

    #[test]
    fn stunned_sprite_sparkle_gate_uses_reference_masks() {
        let k = 12;
        let mut s = fresh_state();
        write_le_u16(&mut s.ram, OAM_CUR_PTR, OAM_BUF as u16);
        write_le_u16(&mut s.ram, OAM_EXT_CUR_PTR, BYTEWISE_EXTENDED_OAM as u16);
        s.ram[FRAME_COUNTER] = 0x94;
        s.ram[0x0fa1] = 0x48;
        s.ram[SPRITE_TYPE + k] = 0x22;
        s.ram[SPRITE_STATE + k] = 11;
        s.sprite_set_x(k, 0x0d0b);
        s.sprite_set_y(k, 0x056a);
        s.ram[SPRITE_UNK5_LOC + k] = 1;
        s.ram[SPRITE_DELAY_MAIN + k] = 0x18;
        s.ram[SPRITE_AI_STATE + k] = 1;
        s.ram[SPRITE_Z + k] = 3;
        s.ram[SPRITE_Z_VEL + k] = 0x0b;
        s.sprite_stunned_main_func1(k);

        assert_eq!(s.ram[0x0fa1], 0x48);
        assert_eq!(s.ram[GARNISH_ACTIVE_SPRITE], 0);
        assert_eq!(s.ram[GARNISH_TYPE + 28], 0);
    }

    #[test]
    fn sprite_prep_oam_coord_fills_ret_and_out_of_bounds_side_effects() {
        let k = 4;
        let mut visible = fresh_state();
        write_le_u16(&mut visible.ram, CUR_SPRITE_X, 0x0120);
        write_le_u16(&mut visible.ram, CUR_SPRITE_Y, 0x0230);
        write_le_u16(&mut visible.ram, BG2HOFS_COPY2, 0x0100);
        write_le_u16(&mut visible.ram, BG2VOFS_COPY2, 0x0200);
        visible.ram[SPRITE_Z + k] = 3;
        visible.ram[SPRITE_OAM_FLAGS + k] = 0x0a;
        visible.ram[SPRITE_OBJ_PRIO + k] = 0x03;
        let mut ret = PrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0xff,
            flags: 0,
        };

        visible.sprite_prep_oam_coord(k, &mut ret);

        assert_eq!(ret.x, 0x20);
        assert_eq!(ret.y, 0x2d);
        assert_eq!(ret.r4, 0);
        assert_eq!(ret.flags, 0x09);
        assert_eq!(read_le_u16(&visible.ram, DUNGMAP_VAR7), 0x2d20);
        assert_eq!(visible.ram[SPRITE_PAUSE + k], 0);

        let mut out = fresh_state();
        out.ram[SPRITE_STATE + k] = 9;
        write_le_u16(&mut out.ram, SPRITE_N_WORD + k * 2, 0x0012);
        out.ram[OVERWORLD_SPRITE_WAS_LOADED + 2] = 0xff;
        write_le_u16(&mut out.ram, CUR_SPRITE_X, 0x0130);
        let mut out_ret = PrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0xff,
            flags: 0,
        };

        out.sprite_prep_oam_coord(k, &mut out_ret);

        assert_eq!(out_ret.x, 0x8212);
        assert_eq!(out_ret.y, 0x00ef);
        assert_eq!(out_ret.r4, 0);
        assert_eq!(out.ram[SPRITE_PAUSE + k], 1);
        assert_eq!(out.ram[SPRITE_STATE + k], 0);
        assert_eq!(out.ram[OVERWORLD_SPRITE_WAS_LOADED + 2], 0xdf);
    }

    #[test]
    fn sprite_spawn_simple_sparkle_garnish_ex_initializes_allocated_slot() {
        let mut s = fresh_state();
        let k = 4;
        s.ram[GARNISH_TYPE + 29] = 1;
        s.sprite_set_x(k, 0x0100);
        s.sprite_set_y(k, 0x0200);
        s.ram[SPRITE_Z + k] = 3;
        s.ram[SPRITE_FLOOR + k] = 2;

        assert_eq!(s.sprite_garnish_spawn_sparkle(k, 0x12, 0x20), 28);

        assert_eq!(s.ram[GARNISH_TYPE + 28], 5);
        assert_eq!(s.ram[GARNISH_ACTIVE_SPRITE], 5);
        assert_eq!(s.garnish_get_x(28), 0x0112);
        assert_eq!(s.garnish_get_y(28), 0x022d);
        assert_eq!(s.ram[GARNISH_COUNTDOWN_SPRITE + 28], 31);
        assert_eq!(s.ram[GARNISH_SPRITE_SPRITE + 28], k as u8);
        assert_eq!(s.ram[GARNISH_FLOOR_SPRITE + 28], 2);
        assert_eq!(s.ram[15], 28);

        let mut full = fresh_state();
        full.ram[GARNISH_TYPE..GARNISH_TYPE + 15].fill(1);
        full.sprite_garnish_spawn_sparkle_limited(k, 0, 0);
        assert_eq!(full.ram[15], 0xff);
        assert_eq!(full.ram[GARNISH_ACTIVE_SPRITE], 0);
    }

    #[test]
    fn release_fairy_spawns_fairy_at_link_position_or_returns_negative_one() {
        let mut s = fresh_state();
        write_le_u16(&mut s.ram, LINK_X_COORD, 0x0100);
        write_le_u16(&mut s.ram, LINK_Y_COORD, 0x0200);
        s.ram[LINK_IS_ON_LOWER_LEVEL] = 1;
        s.ram[SPRITE_D] = 3;

        assert_eq!(s.release_fairy(), 15);
        assert_eq!(s.ram[SPRITE_TYPE + 15], 0xe3);
        assert_eq!(s.ram[SPRITE_STATE + 15], 9);
        assert_eq!(s.ram[SPRITE_FLOOR + 15], 1);
        assert_eq!(s.sprite_get_x(15), 0x0108);
        assert_eq!(s.sprite_get_y(15), 0x0210);
        assert_eq!(s.ram[SPRITE_D + 15], 0);
        assert_eq!(s.ram[SPRITE_DELAY_AUX4 + 15], 96);

        let mut full = fresh_state();
        full.ram[SPRITE_STATE..SPRITE_STATE + 16].fill(9);
        assert_eq!(full.release_fairy(), -1);
    }

    #[test]
    fn sprite_convert_velocity_to_angle_matches_c_tables() {
        for (x, y, expected) in [
            (16, 0, 0),
            (16, 8, 1),
            (0, 16, 4),
            (8, 16, 3),
            (0xf0, 0, 8),
            (0, 0xf0, 12),
        ] {
            assert_eq!(ZeldaState::sprite_convert_velocity_to_angle(x, y), expected);
        }
    }

    #[test]
    fn sprite_zero_and_invert_velocity_helpers_match_c() {
        let mut s = fresh_state();
        let k = 5;
        s.ram[SPRITE_X_VEL + k] = 0x12;
        s.ram[SPRITE_Y_VEL + k] = 0xf0;
        s.sprite_zero_velocity_xy(k);
        assert_eq!(s.ram[SPRITE_X_VEL + k], 0);
        assert_eq!(s.ram[SPRITE_Y_VEL + k], 0);

        s.ram[SPRITE_X_VEL + k] = 0x12;
        s.ram[SPRITE_Y_VEL + k] = 0xf0;
        s.sprite_invert_xy_speeds(k);
        assert_eq!(s.ram[SPRITE_X_VEL + k], 0xee);
        assert_eq!(s.ram[SPRITE_Y_VEL + k], 0x10);

        s.sprite_invert_speed_xy(k);
        assert_eq!(s.ram[SPRITE_X_VEL + k], 0x12);
        assert_eq!(s.ram[SPRITE_Y_VEL + k], 0xf0);
    }

    #[test]
    fn sprite_bounce_off_wall_inverts_only_colliding_axes() {
        let k = 5;

        let mut x_only = fresh_state();
        x_only.ram[SPRITE_X_VEL + k] = 0x08;
        x_only.ram[SPRITE_Y_VEL + k] = 0x09;
        x_only.ram[SPRITE_WALLCOLL + k] = 0x01;
        x_only.sprite_bounce_off_wall(k);
        assert_eq!(x_only.ram[SPRITE_X_VEL + k], 0xf8);
        assert_eq!(x_only.ram[SPRITE_Y_VEL + k], 0x09);

        let mut y_only = fresh_state();
        y_only.ram[SPRITE_X_VEL + k] = 0x08;
        y_only.ram[SPRITE_Y_VEL + k] = 0x09;
        y_only.ram[SPRITE_WALLCOLL + k] = 0x04;
        y_only.sprite_bounce_off_wall(k);
        assert_eq!(y_only.ram[SPRITE_X_VEL + k], 0x08);
        assert_eq!(y_only.ram[SPRITE_Y_VEL + k], 0xf7);

        let mut both = fresh_state();
        both.ram[SPRITE_X_VEL + k] = 0x08;
        both.ram[SPRITE_Y_VEL + k] = 0x09;
        both.ram[SPRITE_WALLCOLL + k] = 0x0f;
        both.sprite_bounce_off_wall(k);
        assert_eq!(both.ram[SPRITE_X_VEL + k], 0xf8);
        assert_eq!(both.ram[SPRITE_Y_VEL + k], 0xf7);
    }

    #[test]
    fn sprite_return_if_paused_matches_c_boolean_gates() {
        let k = 2;

        let mut active = fresh_state();
        active.ram[SPRITE_PAUSE + k] = 0;
        assert!(!active.sprite_return_if_paused(k));

        let mut global_pause = fresh_state();
        global_pause.ram[FLAG_UNK1] = 1;
        assert!(global_pause.sprite_return_if_paused(k));

        let mut submodule = fresh_state();
        submodule.ram[SUBMODULE_INDEX] = 2;
        assert!(submodule.sprite_return_if_paused(k));

        let mut sprite_pause = fresh_state();
        sprite_pause.ram[SPRITE_PAUSE + k] = 1;
        sprite_pause.ram[SPRITE_DEFL_BITS + k] = 0;
        assert!(sprite_pause.sprite_return_if_paused(k));

        sprite_pause.ram[SPRITE_DEFL_BITS + k] = 0x80;
        assert!(!sprite_pause.sprite_return_if_paused(k));
    }

    #[test]
    fn sprite_return_if_phasing_out_matches_stun_countdown_and_draw_gate() {
        let k = 2;

        let mut idle = fresh_state();
        assert!(!idle.sprite_return_if_phasing_out(k));

        let mut blocked = fresh_state();
        blocked.ram[SPRITE_STUNNED + k] = 4;
        blocked.ram[SUBMODULE_INDEX] = 1;
        assert!(!blocked.sprite_return_if_phasing_out(k));
        assert_eq!(blocked.ram[SPRITE_STUNNED + k], 4);

        let mut high_timer = fresh_state();
        high_timer.ram[FRAME_COUNTER] = 1;
        high_timer.ram[SPRITE_STUNNED + k] = 0x28;
        assert!(!high_timer.sprite_return_if_phasing_out(k));
        assert_eq!(high_timer.ram[SPRITE_STUNNED + k], 0x28);

        let mut odd_after_tick = fresh_state();
        odd_after_tick.ram[SPRITE_STUNNED + k] = 2;
        assert!(!odd_after_tick.sprite_return_if_phasing_out(k));
        assert_eq!(odd_after_tick.ram[SPRITE_STUNNED + k], 1);

        let mut expired = fresh_state();
        expired.ram[SPRITE_STATE + k] = 9;
        expired.ram[SPRITE_STUNNED + k] = 1;
        expired.ram[SPRITE_PAUSE + k] = 7;
        assert!(expired.sprite_return_if_phasing_out(k));
        assert_eq!(expired.ram[SPRITE_STUNNED + k], 0);
        assert_eq!(expired.ram[SPRITE_STATE + k], 0);
        assert_eq!(expired.ram[SPRITE_PAUSE + k], 0);

        let mut even_visible = fresh_state();
        even_visible.ram[FRAME_COUNTER] = 1;
        even_visible.ram[SPRITE_STUNNED + k] = 2;
        even_visible.ram[SPRITE_PAUSE + k] = 7;
        assert!(even_visible.sprite_return_if_phasing_out(k));
        assert_eq!(even_visible.ram[SPRITE_STUNNED + k], 2);
        assert_eq!(even_visible.ram[SPRITE_PAUSE + k], 0);
    }

    #[test]
    fn sprite_check_if_lifted_permissive_delegates_to_lifted_helper_side_effects() {
        let mut s = fresh_state();
        let k = 3;
        s.ram[CUR_OBJECT_INDEX] = k as u8;
        s.ram[FLAG_IS_SPRITE_TO_PICK_UP_CACHED] = (k as u8).wrapping_add(1);
        s.ram[SPRITE_STATE + k] = 9;
        s.ram[FILTERED_JOYPAD_L] = 0xff;
        s.ram[SPRITE_E + k] = 5;
        s.ram[SPRITE_UNK3_LOC + k] = 6;
        s.ram[SPRITE_I_LOC + k] = 7;

        s.sprite_check_if_lifted_permissive(k);

        assert_eq!(s.ram[FILTERED_JOYPAD_L], 0);
        assert_eq!(s.ram[SPRITE_E + k], 0);
        assert_eq!(s.ram[SPRITE_UNK4_LOC + k], 9);
        assert_eq!(s.ram[SPRITE_STATE + k], 10);
        assert_eq!(s.ram[SPRITE_DELAY_MAIN + k], 16);
        assert_eq!(s.ram[SPRITE_UNK3_LOC + k], 0);
        assert_eq!(s.ram[SPRITE_I_LOC + k], 0);

        let mut running = fresh_state();
        running.ram[LINK_IS_RUNNING] = 1;
        running.ram[SPRITE_STATE + k] = 9;
        running.sprite_check_if_lifted_permissive(k);
        assert_eq!(running.ram[SPRITE_STATE + k], 9);
    }

    #[test]
    fn sprite_hit_timer31_shows_message_only_for_light_world_good_bee_death() {
        let mut s = fresh_state();
        let k = 3;
        s.ram[SPRITE_TYPE + k] = 0x7a;
        s.ram[SPRITE_HEALTH + k] = 4;
        s.ram[SPRITE_GIVE_DAMAGE_SPRITE + k] = 4;
        s.ram[MAIN_MODULE_INDEX] = 7;

        s.sprite_hit_timer31(k);

        assert_eq!(read_le_u16(&s.ram, DIALOGUE_MESSAGE_INDEX), 0x0140);
        assert_eq!(s.ram[SUBMODULE_INDEX], 2);
        assert_eq!(s.ram[SAVED_MODULE_FOR_MENU], 7);
        assert_eq!(s.ram[MAIN_MODULE_INDEX], 14);

        let mut dark = fresh_state();
        dark.ram[SPRITE_TYPE + k] = 0x7a;
        dark.ram[SPRITE_HEALTH + k] = 1;
        dark.ram[SPRITE_GIVE_DAMAGE_SPRITE + k] = 1;
        dark.ram[IS_IN_DARK_WORLD_SPRITE] = 1;
        dark.sprite_hit_timer31(k);
        assert_eq!(read_le_u16(&dark.ram, DIALOGUE_MESSAGE_INDEX), 0);
    }

    #[test]
    fn sprite_track_body_to_head_matches_frame_gated_turning() {
        let mut equal = fresh_state();
        let k = 6;
        equal.ram[SPRITE_HEAD_DIR + k] = 2;
        equal.ram[SPRITE_D + k] = 2;
        assert!(equal.sprite_track_body_to_head(k));
        assert_eq!(equal.ram[SPRITE_D + k], 2);

        let mut waiting = fresh_state();
        waiting.ram[FRAME_COUNTER] = 1;
        waiting.ram[SPRITE_HEAD_DIR + k] = 0;
        waiting.ram[SPRITE_D + k] = 1;
        assert!(!waiting.sprite_track_body_to_head(k));
        assert_eq!(waiting.ram[SPRITE_D + k], 1);

        let mut same_axis = fresh_state();
        same_axis.ram[FRAME_COUNTER] = 0x20;
        same_axis.ram[SPRITE_HEAD_DIR + k] = 0;
        same_axis.ram[SPRITE_D + k] = 1;
        assert!(!same_axis.sprite_track_body_to_head(k));
        assert_eq!(same_axis.ram[SPRITE_D + k], 3);

        let mut opposite_axis = fresh_state();
        opposite_axis.ram[FRAME_COUNTER] = 0x20;
        opposite_axis.ram[SPRITE_HEAD_DIR + k] = 2;
        opposite_axis.ram[SPRITE_D + k] = 0;
        assert!(opposite_axis.sprite_track_body_to_head(k));
        assert_eq!(opposite_axis.ram[SPRITE_D + k], 2);
    }

    #[test]
    fn sprite_direction_to_face_location_uses_larger_axis_and_caches_y_distance() {
        let mut s = fresh_state();
        let k = 6;
        s.sprite_set_x(k, 0x0100);
        s.sprite_set_y(k, 0x0100);

        assert_eq!(s.sprite_direction_to_face_location(k, 0x0120, 0x0108), 0);
        assert_eq!(s.ram[TMP_COUNTER_SPRITE], 0x08);

        assert_eq!(s.sprite_direction_to_face_location(k, 0x0104, 0x00e0), 3);
        assert_eq!(s.ram[TMP_COUNTER_SPRITE], 0x20);
    }

    #[test]
    fn sprite_approach_target_speed_steps_one_toward_targets() {
        let mut s = fresh_state();
        let k = 4;
        s.ram[SPRITE_X_VEL + k] = 0x10;
        s.ram[SPRITE_Y_VEL + k] = 0x20;

        s.sprite_approach_target_speed(k, 0x20, 0x10);

        assert_eq!(s.ram[SPRITE_X_VEL + k], 0x11);
        assert_eq!(s.ram[SPRITE_Y_VEL + k], 0x1f);

        s.sprite_approach_target_speed(k, 0x11, 0x1f);

        assert_eq!(s.ram[SPRITE_X_VEL + k], 0x11);
        assert_eq!(s.ram[SPRITE_Y_VEL + k], 0x1f);
    }

    #[test]
    fn sprite_halt_all_movement_nullifies_hookshot_drag_and_speed() {
        let mut s = fresh_state();
        s.ram[ANCILLA_TYPE + 4] = 0;
        s.ram[RELATED_TO_HOOKSHOT] = 1;
        s.ram[LINK_X_COORD] = 0x34;
        s.ram[LINK_X_COORD + 1] = 0x12;
        s.ram[LINK_Y_COORD] = 0x78;
        s.ram[LINK_Y_COORD + 1] = 0x56;
        s.ram[LINK_X_COORD_PREV] = 0xbc;
        s.ram[LINK_X_COORD_PREV + 1] = 0x9a;
        s.ram[LINK_Y_COORD_PREV] = 0xf0;
        s.ram[LINK_Y_COORD_PREV + 1] = 0xde;
        s.ram[LINK_SPEED_SETTING] = 7;

        s.sprite_halt_all_movement();

        assert_eq!(s.ram[RELATED_TO_HOOKSHOT], 0);
        assert_eq!(s.ram[LINK_X_COORD_SAFE_RETURN_HI], 0x12);
        assert_eq!(s.ram[LINK_Y_COORD_SAFE_RETURN_HI], 0x56);
        assert_eq!(s.ram[LINK_X_COORD], 0xbc);
        assert_eq!(s.ram[LINK_X_COORD + 1], 0x9a);
        assert_eq!(s.ram[LINK_Y_COORD], 0xf0);
        assert_eq!(s.ram[LINK_Y_COORD + 1], 0xde);
        assert_eq!(s.ram[LINK_SPEED_SETTING], 0);
    }

    #[test]
    fn sprite_check_if_link_is_busy_matches_link_and_hookshot_gates() {
        assert!(!fresh_state().sprite_check_if_link_is_busy());

        let mut aux = fresh_state();
        aux.ram[LINK_AUXILIARY_STATE] = 1;
        assert!(aux.sprite_check_if_link_is_busy());

        let mut item_pose = fresh_state();
        item_pose.ram[LINK_POSE_FOR_ITEM] = 2;
        assert!(item_pose.sprite_check_if_link_is_busy());

        let mut lifted = fresh_state();
        lifted.ram[LINK_STATE_BITS] = 0x80;
        assert!(lifted.sprite_check_if_link_is_busy());

        let mut hookshot = fresh_state();
        hookshot.ram[ANCILLA_TYPE + 4] = 0x27;
        assert!(hookshot.sprite_check_if_link_is_busy());
    }

    #[test]
    fn sprite_schedule_for_breakage_sets_state_delay_and_flags() {
        let mut s = fresh_state();
        let k = 5;
        s.ram[SPRITE_FLAGS2 + k] = 0xfe;

        s.sprite_schedule_for_breakage(k);

        assert_eq!(s.ram[SPRITE_DELAY_MAIN + k], 31);
        assert_eq!(s.ram[SPRITE_STATE + k], 6);
        assert_eq!(s.ram[SPRITE_FLAGS2 + k], 2);
    }

    #[test]
    fn sprite_check_if_overlords_clear_rejects_active_overlord_types() {
        let mut s = fresh_state();

        assert!(s.sprite_check_if_overlords_clear());

        s.ram[OVERLORD_TYPE + 3] = 0x14;
        assert!(!s.sprite_check_if_overlords_clear());

        s.ram[OVERLORD_TYPE + 3] = 0x18;
        assert!(!s.sprite_check_if_overlords_clear());

        s.ram[OVERLORD_TYPE + 3] = 0x13;
        assert!(s.sprite_check_if_overlords_clear());
    }

    #[test]
    fn sprite_check_if_room_is_clear_ignores_inactive_and_ignored_sprites() {
        let mut s = fresh_state();
        let k = 5;

        assert!(s.sprite_check_if_room_is_clear());

        s.ram[SPRITE_STATE + k] = 9;
        assert!(!s.sprite_check_if_room_is_clear());

        s.ram[SPRITE_FLAGS4 + k] = 0x40;
        assert!(s.sprite_check_if_room_is_clear());

        s.ram[SPRITE_STATE + k] = 0;
        s.ram[SPRITE_FLAGS4 + k] = 0;
        s.ram[OVERLORD_TYPE + 2] = 0x18;
        assert!(!s.sprite_check_if_room_is_clear());
    }

    #[test]
    fn sprite_check_if_screen_is_clear_uses_camera_bounds_and_overlords() {
        let mut s = fresh_state();
        let k = 5;

        write_le_u16(&mut s.ram, BG2HOFS_COPY2, 0);
        write_le_u16(&mut s.ram, BG2VOFS_COPY2, 0);
        s.ram[SPRITE_STATE + k] = 9;
        s.sprite_set_x(k, 0x00f0);
        s.sprite_set_y(k, 0x00f0);
        assert!(!s.sprite_check_if_screen_is_clear());

        s.sprite_set_x(k, 0x0100);
        assert!(s.sprite_check_if_screen_is_clear());

        s.sprite_set_x(k, 0x00f0);
        s.ram[SPRITE_FLAGS4 + k] = 0x40;
        assert!(s.sprite_check_if_screen_is_clear());

        s.ram[SPRITE_STATE + k] = 0;
        s.ram[SPRITE_FLAGS4 + k] = 0;
        s.ram[OVERLORD_TYPE + 1] = 0x14;
        assert!(!s.sprite_check_if_screen_is_clear());
    }

    #[test]
    fn sprite_manually_set_death_flag_uw_sets_room_bit_only_when_allowed() {
        let mut s = fresh_state();
        let k = 8;
        s.ram[PLAYER_IS_INDOORS] = 1;
        s.ram[SPRITE_N + k] = 8;
        write_le_u16(&mut s.ram, DUNGEON_ROOM_INDEX2, 0x0123);

        s.sprite_manually_set_death_flag_uw(k);

        assert_eq!(
            read_le_u16(&s.ram, SPRITE_WHERE_IN_ROOM + 0x0123 * 2),
            0x0100
        );

        let mut outdoors = fresh_state();
        outdoors.ram[SPRITE_N + k] = 8;
        write_le_u16(&mut outdoors.ram, DUNGEON_ROOM_INDEX2, 0x0123);
        outdoors.sprite_manually_set_death_flag_uw(k);
        assert_eq!(
            read_le_u16(&outdoors.ram, SPRITE_WHERE_IN_ROOM + 0x0123 * 2),
            0
        );

        let mut ignored = fresh_state();
        ignored.ram[PLAYER_IS_INDOORS] = 1;
        ignored.ram[SPRITE_DEFL_BITS + k] = 1;
        ignored.ram[SPRITE_N + k] = 8;
        write_le_u16(&mut ignored.ram, DUNGEON_ROOM_INDEX2, 0x0123);
        ignored.sprite_manually_set_death_flag_uw(k);
        assert_eq!(
            read_le_u16(&ignored.ram, SPRITE_WHERE_IN_ROOM + 0x0123 * 2),
            0
        );

        let mut signed = fresh_state();
        signed.ram[PLAYER_IS_INDOORS] = 1;
        signed.ram[SPRITE_N + k] = 0x80;
        write_le_u16(&mut signed.ram, DUNGEON_ROOM_INDEX2, 0x0123);
        signed.sprite_manually_set_death_flag_uw(k);
        assert_eq!(
            read_le_u16(&signed.ram, SPRITE_WHERE_IN_ROOM + 0x0123 * 2),
            0
        );
    }

    #[test]
    fn sprite_bounce_from_tile_collision_returns_zero_without_collision() {
        let mut s = fresh_state();
        let k = 2;
        s.ram[SPRITE_X_VEL + k] = 0x12;
        s.ram[SPRITE_Y_VEL + k] = 0xf0;
        s.ram[SPRITE_G + k] = 7;
        s.ram[SPRITE_FLAGS2 + k] = 0x60;

        assert_eq!(s.sprite_bounce_from_tile_collision(k), 0);
        assert_eq!(s.ram[SPRITE_X_VEL + k], 0x12);
        assert_eq!(s.ram[SPRITE_Y_VEL + k], 0xf0);
        assert_eq!(s.ram[SPRITE_G + k], 7);
    }
}
