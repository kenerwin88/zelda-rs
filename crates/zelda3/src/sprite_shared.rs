use super::sprite::DrawMultipleData;

pub(super) const SPRITE_C_SPRITE: usize = 0x0db0;
pub(super) const SPRITE_DELAY_AUX1_SPRITE: usize = 0x0e00;
pub(super) const SPRITE_HEALTH_SPRITE: usize = 0x0e50;
pub(super) const SPRITE_DELAY_AUX3_SPRITE: usize = 0x0ee0;
pub(super) const SPRITE_DELAY_AUX2_SPRITE: usize = 0x0e10;
pub(super) const SPRITE_INIT_TABLE_LEN: usize = 243;
pub(super) const SPRITE_INIT_FLAGS2_TABLE: usize = 0;
pub(super) const SPRITE_INIT_HEALTH_TABLE: usize = 1;
pub(super) const SPRITE_INIT_BUMP_DAMAGE_TABLE: usize = 2;
pub(super) const SPRITE_INIT_FLAGS3_TABLE: usize = 3;
pub(super) const SPRITE_INIT_FLAGS4_TABLE: usize = 4;
pub(super) const SPRITE_INIT_FLAGS_TABLE: usize = 5;
pub(super) const SPRITE_INIT_FLAGS5_TABLE: usize = 6;
pub(super) const SPRITE_INIT_DEFL_BITS_TABLE: usize = 7;

pub(super) const SINGLE_LARGE_SPRITE_CHAR_BASE_BY_TYPE: [u8; 236] = [
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

pub(super) const SINGLE_LARGE_SPRITE_CHAR_BY_BASE_AND_GFX: [u8; 251] = [
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

pub(super) const SPRITE_INIT_TABLES_HEX: &str = concat!(
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
pub(super) const SPRITE_INIT_TABLES_C_HEX: &str = concat!(
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
pub(super) const SPRITE_Y_RECOIL: usize = 0x0f30;
pub(super) const SPRITE_DRAW_PRIORITY_OVERRIDE: usize = 0x0cfe;
pub(super) const SPRITE_PICKUP_SLOT_CACHE: usize = 0x0fb2;
pub(super) const SPRITE_F: usize = 0x0ea0;
pub(super) const SPRITE_AI_STATE_SPRITE: usize = 0x0d80;
pub(super) const ANCILLA_X_LO_SPRITE: usize = 0x0c04;
pub(super) const ANCILLA_Y_LO_SPRITE: usize = 0x0bfa;
pub(super) const SPRITE_BUMP_DAMAGE: usize = 0x0cd2;
pub(super) const SPRITE_FLAGS_SPRITE: usize = 0x0b6b;
pub(super) const DAMAGE_TYPE_DETERMINER_SPRITE: usize = 0x0cf2;
pub(super) const SPRITE_WALLCOLL: usize = 0x0e70;
pub(super) const SPRITE_GIVE_DAMAGE_SPRITE: usize = 0x0ce2;
pub(super) const IS_IN_DARK_WORLD_SPRITE: usize = 0x0fff;
pub(super) const GARNISH_Y_LO_SPRITE: usize = 0x1f81e;
pub(super) const GARNISH_X_LO_SPRITE: usize = 0x1f83c;
pub(super) const GARNISH_Y_HI_SPRITE: usize = 0x1f85a;
pub(super) const GARNISH_X_HI_SPRITE: usize = 0x1f878;
pub(super) const GARNISH_Y_VEL_SPRITE: usize = 0x1f896;
pub(super) const GARNISH_X_VEL_SPRITE: usize = 0x1f8b4;
pub(super) const GARNISH_Y_SUBPIXEL_SPRITE: usize = 0x1f8d2;
pub(super) const GARNISH_X_SUBPIXEL_SPRITE: usize = 0x1f8f0;
pub(super) const GARNISH_ACTIVE_SPRITE: usize = 0x0fb4;
pub(super) const GARNISH_COUNTDOWN_SPRITE: usize = 0x1f90e;
pub(super) const CHECK_DAMAGE_FROM_PLAYER_CARRY: u8 = 1;
pub(super) const CHECK_DAMAGE_FROM_PLAYER_NON_ELEMENTAL: u8 = 2;
pub(super) const GARNISH_SPRITE_SPRITE: usize = 0x1f92c;
pub(super) const GARNISH_FLOOR_SPRITE: usize = 0x1f968;
pub(super) const GARNISH_OAM_FLAGS_SPRITE: usize = 0x1f9fe;
pub(super) const OVERLORD_GEN3_SPRITE: usize = 0x0b38;
pub(super) const OVERLORD_FLOOR_SPRITE: usize = 0x0b40;
pub(super) const OVERLORD_SPAWNED_IN_AREA_SPRITE: usize = 0x0cca;
pub(super) const OVERWORLD_AREA_INDEX_SPRITE: usize = 0x040a;
pub(super) const REPULSESPARK_FLOOR_STATUS_SPRITE: usize = 0x0b68;
pub(super) const REPULSESPARK_TIMER_SPRITE: usize = 0x0fac;
pub(super) const REPULSESPARK_X_LO_SPRITE: usize = 0x0fad;
pub(super) const REPULSESPARK_Y_LO_SPRITE: usize = 0x0fae;
pub(super) const SRAM_PROGRESS_INDICATOR_SPRITE: usize = 0x0f3c5;
pub(super) const SPRITE_RESET_WORK_A: usize = 0x0ff8;
pub(super) const SPRITE_RESET_WORK_B: usize = 0x0ffb;
pub(super) const ACTIVATE_BOMB_TRAP_OVERLORD_SPRITE: usize = 0x0cf4;
pub(super) const OAM_REGION_BASE_SPRITE: usize = 0x0fe0;
pub(super) const SPR_RANGED_BASED_TOGGLER: usize = 0x0fb7;
pub(super) const SPRCOLL_X_BASE_SPRITE: usize = 0x0fbc;
pub(super) const SPRCOLL_Y_BASE_SPRITE: usize = 0x0fbe;
pub(super) const SPRITE_WHERE_IN_OVERWORLD: usize = 0x1df80;
pub(super) const OVERLORD_OFFSET_SPRITE_POS_SPRITE: usize = 0x0b48;
pub(super) const FEATURES0_EXTEND_SCREEN64_SPRITE: u32 = 1;
pub(super) const FEATURES0_COLLECT_ITEMS_WITH_SWORD_SPRITE: u32 = 16;

pub(super) const OVERWORLD_AREA_SPRCOLL_SIZES: [u8; 192] = [
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
pub(super) const SPRITE_OVERLORD_X_HI: usize = 0x0b10;
pub(super) const SPRITE_OVERLORD_Y_LO: usize = 0x0b18;
pub(super) const SPRITE_OVERLORD_Y_HI: usize = 0x0b20;

// Word-wide alias used in Sprite_SpawnDynamicallyEx when player_is_indoors
// is false. `sprite_N_word[j]` is the 16-bit view of `sprite_N[j]`.
// variables.h:1228 sets byte_7FFABC at 0x1fabc; sprite_N lives at 0x0bc0.
pub(super) const SPRITE_N_WORD: usize = 0x0bc0;

// Dual-layer tile-collision cache referenced by Sprite_CheckTileCollision2.
// variables.h:1228 — `byte_7FFABC` lives at 0x1fabc.
pub(super) const BYTE_7FFABC: usize = 0x1fabc;

// Single-byte tile-type cache used across Sprite_* tile helpers.
// variables.h:755 — `sprite_tiletype` lives at 0x0fa5.
pub(super) const SPRITE_TILETYPE_SPR: usize = 0x0fa5;
pub(super) const ITEM_DROP_LUCK_SPRITE: usize = 0x0cf9;
pub(super) const LUCK_KILL_COUNTER_SPRITE: usize = 0x0cfa;
pub(super) const NUM_SPRITES_KILLED_SPRITE: usize = 0x0cfb;
pub(super) const ALT_SPRITE_STATE_SPRITE: usize = 0x1d00;
pub(super) const ALT_SPRITE_TYPE_SPRITE: usize = 0x1d10;
pub(super) const ALT_SPRITE_X_LO_SPRITE: usize = 0x1d20;
pub(super) const ALT_SPRITE_X_HI_SPRITE: usize = 0x1d30;
pub(super) const ALT_SPRITE_Y_LO_SPRITE: usize = 0x1d40;
pub(super) const ALT_SPRITE_Y_HI_SPRITE: usize = 0x1d50;
pub(super) const ALT_SPRITE_GRAPHICS_SPRITE: usize = 0x1d60;
pub(super) const ALT_SPRITE_A_SPRITE: usize = 0x1d70;
pub(super) const ALT_SPRITE_HEAD_DIR_SPRITE: usize = 0x1d80;
pub(super) const ALT_SPRITE_OAM_FLAGS_SPRITE: usize = 0x1d90;
pub(super) const ALT_SPRITE_OBJ_PRIO_SPRITE: usize = 0x1da0;
pub(super) const ALT_SPRITE_D_SPRITE: usize = 0x1db0;
pub(super) const ALT_SPRITE_FLAGS2_SPRITE: usize = 0x1dc0;
pub(super) const ALT_SPRITE_FLOOR_SPRITE: usize = 0x1dd0;
pub(super) const ALT_SPRITE_SPAWNED_FLAG_SPRITE: usize = 0x1de0;
pub(super) const ALT_SPRITE_FLAGS3_SPRITE: usize = 0x1df0;
pub(super) const ALT_SPRITE_B_SPRITE: usize = 0x1fa5c;
pub(super) const ALT_SPRITE_C_SPRITE: usize = 0x1fa6c;
pub(super) const ALT_SPRITE_E_SPRITE: usize = 0x1fa7c;
pub(super) const ALT_SPRITE_SUBTYPE2_SPRITE: usize = 0x1fa8c;
pub(super) const ALT_SPRITE_HEIGHT_ABOVE_SHADOW_SPRITE: usize = 0x1fa9c;
pub(super) const ALT_SPRITE_DELAY_MAIN_SPRITE: usize = 0x1faac;
pub(super) const ALT_SPRITE_I_SPRITE: usize = 0x1facc;
pub(super) const ALT_SPRITE_IGNORE_PROJECTILE_SPRITE: usize = 0x1fadc;

// ---------------------------------------------------------------------------
// Promoted sprite method-local tables. Names retain the owning helper so
// generic C table names stay readable at callsites.
// ---------------------------------------------------------------------------

pub(super) const PREPARE_APPLY_RUMBLE_TO_SPRITES_APPLY_RUMBLE_X: [i8; 4] = [-32, -32, -32, 16];

pub(super) const PREPARE_APPLY_RUMBLE_TO_SPRITES_APPLY_RUMBLE_Y: [i8; 4] = [-32, 32, -24, -24];

pub(super) const PREPARE_APPLY_RUMBLE_TO_SPRITES_APPLY_RUMBLE_WH: [u8; 6] =
    [0x50, 0x50, 0x20, 0x20, 0x50, 0x50];

pub(super) const OAM_RESET_REGION_BASES_OAM_RESET_REGION_BASES: [u16; 6] =
    [0x0030, 0x01d0, 0x0000, 0x0030, 0x0120, 0x0140];

pub(super) const SPRITE_SPAWN_THROWABLE_TERRAIN_SILENTLY_THROWABLE_SCENERY_OAM_FLAGS: [u8; 9] =
    [0x0c, 0x0c, 0x0c, 0, 0, 0, 0xb0, 0x08, 0xb4];

pub(super) const OVERWORLD_SUBSTITUTE_ALTERNATE_SECRET_SECRET_SUBSTITUTION_ITEMS: [u8; 64] = [
    0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 6, 4, 4, 6, 0, 0, 15, 15, 4, 5, 5, 4, 6,
    6, 15, 15, 4, 5, 5, 7, 6, 6, 31, 31, 4, 7, 7, 4, 6, 6, 6, 7, 2, 0, 0, 0, 0, 0, 6, 6, 2, 0, 0,
    0, 0, 0,
];

pub(super) const OVERWORLD_SUBSTITUTE_ALTERNATE_SECRET_SECRET_SUBSTITUTION_HORIZONTAL_OFFSETS:
    [u8; 16] = [1, 1, 1, 1, 15, 1, 1, 18, 16, 1, 1, 1, 17, 1, 1, 3];

pub(super) const OVERWORLD_SUBSTITUTE_ALTERNATE_SECRET_SECRET_SUBSTITUTION_VERTICAL_OFFSETS: [u8;
    16] = [0, 0, 0, 0, 2, 0, 0, 8, 16, 0, 0, 0, 1, 0, 0, 0];

pub(super) const SPRITE_SETUP_HIT_BOX_SPRITE_HITBOX_XLO: [i8; 32] = [
    2, 3, 0, -3, -6, 0, 2, -8, 0, -4, -8, 0, -8, -16, 2, 2, 2, 2, 2, -8, 2, 2, -16, -8, -12, 4, -4,
    -12, 5, -32, -2, 4,
];

pub(super) const SPRITE_SETUP_HIT_BOX_SPRITE_HITBOX_XHI: [i8; 32] = [
    0, 0, 0, -1, -1, 0, 0, -1, 0, -1, -1, 0, -1, -1, 0, 0, 0, 0, 0, -1, 0, 0, -1, -1, -1, 0, -1,
    -1, 0, -1, -1, 0,
];

pub(super) const SPRITE_SETUP_HIT_BOX_SPRITE_HITBOX_XSIZE: [u8; 32] = [
    12, 1, 16, 20, 20, 8, 4, 32, 48, 24, 32, 32, 32, 48, 12, 12, 60, 124, 12, 32, 4, 12, 48, 32,
    40, 8, 24, 24, 5, 80, 4, 8,
];

pub(super) const SPRITE_SETUP_HIT_BOX_SPRITE_HITBOX_YLO: [i8; 32] = [
    0, 3, 4, -4, -8, 2, 0, -16, 12, -4, -8, 0, -10, -16, 2, 2, 2, 2, -3, -12, 2, 10, 0, -12, 16, 4,
    -4, -12, 3, -16, -8, 10,
];

pub(super) const SPRITE_SETUP_HIT_BOX_SPRITE_HITBOX_YHI: [i8; 32] = [
    0, 0, 0, -1, -1, 0, 0, -1, 0, -1, -1, 0, -1, -1, 0, 0, 0, 0, -1, -1, 0, 0, 0, -1, 0, 0, -1, -1,
    0, -1, -1, 0,
];

pub(super) const SPRITE_SETUP_HIT_BOX_SPRITE_HITBOX_YSIZE: [u8; 32] = [
    14, 1, 16, 21, 24, 4, 8, 40, 20, 24, 40, 29, 36, 48, 60, 124, 12, 12, 17, 28, 4, 2, 28, 20, 10,
    4, 24, 16, 5, 48, 8, 12,
];

pub(super) const PLAYER_ACTION_HIT_BOX_FROM_TABLE_X_OFFSETS: [i8; 65] = [
    0, 2, 0, 0, -8, 0, 2, 0, 2, 2, 1, 1, 0, 0, 0, 0, 0, 2, 4, 4, 0, 0, -4, -4, -6, 2, 1, 1, 0, 0,
    0, 0, 0, 0, 0, 0, 2, 2, 4, 4, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, -4, -4, -10, 0, 2, 2, 0, 0,
    0, 0, 0, 0, 0,
];

pub(super) const PLAYER_ACTION_HIT_BOX_FROM_TABLE_WIDTHS: [u8; 65] = [
    15, 4, 8, 8, 8, 8, 12, 8, 4, 4, 6, 6, 0, 0, 0, 0, 0, 4, 16, 12, 8, 8, 12, 11, 12, 4, 6, 6, 0,
    0, 0, 0, 0, 8, 8, 8, 10, 14, 15, 4, 4, 4, 6, 6, 0, 0, 0, 0, 0, 8, 8, 8, 10, 14, 15, 4, 4, 4, 6,
    6, 0, 0, 0, 0, 0,
];

pub(super) const PLAYER_ACTION_HIT_BOX_FROM_TABLE_Y_OFFSETS: [i8; 65] = [
    0, 2, 0, 2, 4, 4, 4, 7, 2, 2, 1, 1, 0, 0, 0, 0, 0, 2, 0, 2, -4, -3, -8, 0, 0, 2, 1, 1, 0, 0, 0,
    0, 0, 0, 0, 0, -2, 0, -4, 1, 2, 2, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, -2, 0, -4, 1, 2, 2, 1, 1, 0,
    0, 0, 0, 0,
];

pub(super) const PLAYER_ACTION_HIT_BOX_FROM_TABLE_HEIGHTS: [u8; 65] = [
    15, 4, 8, 2, 12, 8, 12, 8, 4, 4, 6, 6, 0, 0, 0, 0, 0, 4, 8, 4, 12, 12, 12, 4, 8, 4, 6, 4, 0, 0,
    0, 0, 0, 8, 8, 8, 8, 8, 12, 4, 4, 4, 6, 6, 0, 0, 0, 0, 0, 8, 8, 8, 8, 8, 12, 4, 4, 4, 6, 6, 0,
    0, 0, 0, 0,
];

pub(super) const PLAYER_SETUP_ACTION_HIT_BOX_RUN_Y_HI: [u8; 4] = [0xff, 0, 0, 0];

pub(super) const PLAYER_SETUP_ACTION_HIT_BOX_RUN_Y_LO: [u8; 4] = [0xf8, 16, 8, 8];

pub(super) const PLAYER_SETUP_ACTION_HIT_BOX_RUN_X_HI: [u8; 4] = [0, 0, 0xff, 0];

pub(super) const PLAYER_SETUP_ACTION_HIT_BOX_RUN_X_LO: [u8; 4] = [0, 0, 0xf8, 8];

pub(super) const PLAYER_SETUP_ACTION_HIT_BOX_SWORD_ACTION_INACTIVE_FRAMES: [u8; 13] =
    [1, 1, 1, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1];

pub(super) const LINK_UPDATE_HIT_BOX_WITH_SWORD_SWORD_ACTION_INACTIVE_FRAMES: [u8; 13] =
    [1, 1, 1, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1];

pub(super) const SPRITE_TIMERS_AND_OAM_SPRITE_PRIOS: [u8; 4] = [0x20, 0x10, 0x30, 0x30];

pub(super) const OAM_GET_BUFFER_POSITION_LIMITS: [u16; 6] =
    [0x0171, 0x0201, 0x0031, 0x00c1, 0x0141, 0x01d1];

pub(super) const OAM_GET_BUFFER_POSITION_FALLBACKS: [u16; 48] = [
    0x0030, 0x0050, 0x0080, 0x00b0, 0x00e0, 0x0110, 0x0140, 0x0170, 0x01d0, 0x01d4, 0x01dc, 0x01e0,
    0x01e4, 0x01ec, 0x01f0, 0x01f8, 0x0000, 0x0004, 0x0008, 0x000c, 0x0010, 0x0014, 0x0018, 0x001c,
    0x0030, 0x0038, 0x0050, 0x0068, 0x0080, 0x0098, 0x00b0, 0x00c8, 0x0120, 0x0124, 0x0128, 0x012c,
    0x0130, 0x0134, 0x0138, 0x013c, 0x0140, 0x0150, 0x0160, 0x0170, 0x0180, 0x0190, 0x01a0, 0x01b8,
];

pub(super) const GARNISH_SPARKLE_COMMON_GARNISH_SPARKLE_CHAR: [u8; 4] = [0x83, 0xc7, 0x80, 0xb7];

pub(super) const GARNISH_DUST_COMMON_RUNNING_MAN_DUST_CHAR: [u8; 3] = [0xdf, 0xcf, 0xa9];

pub(super) const GARNISH04_LASER_TRAIL_LASER_BEAM_TRAIL_CHAR: [u8; 2] = [0xd2, 0xf3];

pub(super) const GARNISH15_ARRGHUS_SPLASH_ARRGHUS_SPLASH_X: [i8; 8] =
    [-12, 20, -10, 10, -8, 8, -4, 4];

pub(super) const GARNISH15_ARRGHUS_SPLASH_ARRGHUS_SPLASH_Y: [i8; 8] = [-4, -4, -2, -2, 0, 0, 0, 0];

pub(super) const GARNISH15_ARRGHUS_SPLASH_ARRGHUS_SPLASH_CHAR: [u8; 8] =
    [0xae, 0xae, 0xae, 0xae, 0xae, 0xae, 0xac, 0xac];

pub(super) const GARNISH15_ARRGHUS_SPLASH_ARRGHUS_SPLASH_FLAGS: [u8; 8] =
    [0x34, 0x74, 0x34, 0x74, 0x34, 0x74, 0x34, 0x74];

pub(super) const GARNISH15_ARRGHUS_SPLASH_ARRGHUS_SPLASH_EXT: [u8; 8] = [0, 0, 2, 2, 2, 2, 2, 2];

pub(super) const GARNISH10_GANON_BAT_FLAME_GANON_BAT_FLAME_IDX: [u8; 32] = [
    7, 6, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4,
];

pub(super) const GARNISH10_GANON_BAT_FLAME_GANON_BAT_FLAME_CHAR: [u8; 7] =
    [0xac, 0xac, 0x66, 0x66, 0x8e, 0xa0, 0xa2];

pub(super) const GARNISH10_GANON_BAT_FLAME_GANON_BAT_FLAME_FLAGS: [u8; 7] =
    [1, 0x41, 1, 0x41, 0, 0, 0];

pub(super) const GARNISH0_A_CANNON_SMOKE_GARNISH_CANNON_POOF_CHAR: [u8; 2] = [0x8a, 0x86];

pub(super) const GARNISH0_A_CANNON_SMOKE_GARNISH_CANNON_POOF_FLAGS: [u8; 4] =
    [0x20, 0x10, 0x30, 0x30];

pub(super) const GARNISH0_C_TRINEXX_ICE_BREATH_TRINEXX_ICE_CHAR: [u8; 12] = [
    0xe8, 0xe8, 0xe6, 0xe6, 0xe4, 0xe4, 0xe4, 0xe4, 0xe4, 0xe4, 0xe4, 0xe4,
];

pub(super) const GARNISH0_C_TRINEXX_ICE_BREATH_TRINEXX_ICE_FLAGS: [u8; 4] = [0, 0x40, 0xc0, 0x80];

pub(super) const GARNISH09_LIGHTNING_TRAIL_LIGHTNING_TRAIL_CHAR: [u8; 8] =
    [0xcc, 0xec, 0xce, 0xee, 0xcc, 0xec, 0xce, 0xee];

pub(super) const GARNISH09_LIGHTNING_TRAIL_LIGHTNING_TRAIL_FLAGS: [u8; 8] =
    [0x31, 0x31, 0x31, 0x31, 0x71, 0x71, 0x71, 0x71];

pub(super) const GARNISH03_FALLING_TILE_CRUMBLE_TILE_XY: [u8; 5] = [4, 0, 0, 0, 0];

pub(super) const GARNISH03_FALLING_TILE_CRUMBLE_TILE_CHAR: [u8; 5] = [0x80, 0xcc, 0xcc, 0xea, 0xca];

pub(super) const GARNISH03_FALLING_TILE_CRUMBLE_TILE_FLAGS: [u8; 5] =
    [0x30, 0x31, 0x31, 0x31, 0x31];

pub(super) const GARNISH03_FALLING_TILE_CRUMBLE_TILE_EXT: [u8; 5] = [0, 2, 2, 2, 2];

pub(super) const GARNISH07_BABASU_FLASH_BABUSU_FLASH_CHAR: [u8; 4] = [0xa8, 0x8a, 0x86, 0x86];

pub(super) const GARNISH07_BABASU_FLASH_BABUSU_FLASH_FLAGS: [u8; 4] = [0x2d, 0x2c, 0x2c, 0x2c];

pub(super) const GARNISH08_KHOLDSTARE_TRAIL_GARNISH_NEBULE_XY: [i8; 3] = [-1, -1, 0];

pub(super) const GARNISH08_KHOLDSTARE_TRAIL_GARNISH_NEBULE_CHAR: [u8; 3] = [0x9c, 0x9d, 0x8d];

pub(super) const GARNISH0_E_TRINEXX_FIRE_BREATH_TRINEXX_LAVA_BUBBLE_CHAR: [u8; 4] =
    [0x83, 0xc7, 0x80, 0x9d];

pub(super) const GARNISH0_F_BLIND_LASER_TRAIL_BLIND_LASER_TRAIL_CHAR: [u8; 4] =
    [0x61, 0x71, 0x70, 0x60];

pub(super) const GARNISH_EXECUTE_SINGLE_GARNISH_OAM_MEM_SIZE: [u8; 23] = [
    0, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 8, 16,
];

pub(super) const SPRITE_DRAW_ABSORBABLE_TRANSIENT_ABSORBABLE_OAM_EXT_SIZE_BY_TYPE: [u8; 15] =
    [0, 1, 1, 1, 2, 2, 2, 0, 1, 1, 2, 2, 1, 2, 2];

pub(super) const SPRITE_DRAW_ABSORBABLE_TRANSIENT_ABSORBABLE_GFX_BY_TYPE: [u8; 19] =
    [0, 0, 0, 0, 1, 2, 3, 0, 0, 4, 5, 0, 0, 0, 0, 2, 4, 6, 2];

pub(super) const SPRITE_DRAW_NUMBERED_ABSORBABLE_X_OFFSETS: [i16; 18] =
    [0, 0, 8, 0, 0, 8, 0, 0, 8, 0, 0, 2, 0, 0, 2, 0, 0, 0];

pub(super) const SPRITE_DRAW_NUMBERED_ABSORBABLE_Y_OFFSETS: [i16; 18] =
    [0, 0, 8, 0, 0, 8, 0, 0, 8, 0, 8, 8, 0, 8, 8, 0, 8, 8];

pub(super) const SPRITE_DRAW_NUMBERED_ABSORBABLE_CHARS: [u8; 18] = [
    0x6e, 0x6e, 0x68, 0x6e, 0x6e, 0x78, 0x6e, 0x6e, 0x79, 0x63, 0x73, 0x69, 0x63, 0x73, 0x6a, 0x63,
    0x73, 0x73,
];

pub(super) const SPRITE_DRAW_NUMBERED_ABSORBABLE_EXT_SIZES: [u8; 18] =
    [2, 2, 0, 2, 2, 0, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

pub(super) const THROWABLE_SCENERY_TRANSMUTE_TO_DEBRIS_THROWN_SPRITE_IMPACT_SFX: [u8; 9] =
    [0x1f, 0x1f, 0x1e, 0x1e, 0x1e, 0x1f, 0x1f, 0x1f, 0x1f];

pub(super) const SPRITE_CALCULATE_SWORD_DAMAGE_SPRITE_DAMAGE_BY_PLAYER_WEAPON: [u8; 12] =
    [1, 2, 3, 4, 2, 3, 4, 5, 1, 1, 2, 3];

pub(super) const SPRITE_APPLY_CALCULATED_DAMAGE_ENEMY_CONTACT_DAMAGE_BY_TYPE: [u8; 128] = [
    0, 1, 32, 255, 252, 251, 0, 0, 0, 2, 64, 4, 0, 0, 0, 0, 0, 4, 64, 2, 3, 0, 0, 0, 0, 8, 64, 4,
    0, 0, 0, 0, 0, 16, 64, 8, 0, 0, 0, 0, 0, 16, 64, 8, 0, 0, 0, 0, 0, 4, 64, 16, 0, 0, 0, 0, 0,
    255, 64, 255, 252, 251, 0, 0, 0, 4, 64, 255, 252, 251, 32, 0, 0, 100, 24, 100, 0, 0, 0, 0, 0,
    249, 250, 255, 100, 0, 0, 0, 0, 8, 64, 253, 4, 16, 0, 0, 0, 8, 64, 254, 4, 0, 0, 0, 0, 16, 64,
    253, 0, 0, 0, 0, 0, 254, 64, 16, 0, 0, 0, 0, 0, 32, 64, 255, 0, 0, 0, 250,
];

pub(super) const SPRITE_DO_THE_DEATH_PIKIT_DROP_ITEMS: [u8; 4] = [0xdc, 0xe1, 0xd9, 0xe6];

pub(super) const SPRITE_DO_THE_DEATH_PRIZE_MASKS: [u8; 7] = [1, 1, 1, 0, 1, 1, 1];

pub(super) const FORCE_PRIZE_DROP_PRIZE_ITEMS: [u8; 56] = [
    0xd8, 0xd8, 0xd8, 0xd8, 0xd9, 0xd8, 0xd8, 0xd9, 0xda, 0xd9, 0xda, 0xdb, 0xda, 0xd9, 0xda, 0xda,
    0xe0, 0xdf, 0xdf, 0xda, 0xe0, 0xdf, 0xd8, 0xdf, 0xdc, 0xdc, 0xdc, 0xdd, 0xdc, 0xdc, 0xde, 0xdc,
    0xe1, 0xd8, 0xe1, 0xe2, 0xe1, 0xd8, 0xe1, 0xe2, 0xdf, 0xd9, 0xd8, 0xe1, 0xdf, 0xdc, 0xd9, 0xd8,
    0xd8, 0xe3, 0xe0, 0xdb, 0xde, 0xd8, 0xdb, 0xe2,
];

pub(super) const PREPARE_ENEMY_DROP_PRIZE_Z: [u8; 15] = [
    0, 0x24, 0x24, 0x24, 0x20, 0x20, 0x20, 0x24, 0x24, 0x24, 0x24, 0, 0x24, 0x20, 0x20,
];

pub(super) const SPRITE_DEATH_DRAW_POOF_X_OFFSETS: [i8; 32] = [
    0, 0, 0, 8, 0, 8, 0, 8, 8, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, -3, 11, -3, 11, -6, 14,
    -6, 14,
];

pub(super) const SPRITE_DEATH_DRAW_POOF_Y_OFFSETS: [i8; 32] = [
    0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8, -3, -3, 11, 11, -6, -6,
    14, 14,
];

pub(super) const SPRITE_DEATH_DRAW_POOF_CHARS: [u8; 32] = [
    0, 0xb9, 0, 0, 0xb4, 0xb5, 0xb5, 0xb4, 0xb9, 0, 0, 0, 0xb5, 0xb4, 0xb4, 0xb5, 0xa8, 0xa8, 0xb8,
    0xb8, 0xa8, 0xa8, 0xb8, 0xb8, 0xa9, 0xa9, 0xa9, 0xa9, 0x9b, 0x9b, 0x9b, 0x9b,
];

pub(super) const SPRITE_DEATH_DRAW_POOF_FLAGS: [u8; 32] = [
    4, 4, 4, 4, 4, 4, 0xc4, 0xc4, 0x44, 4, 4, 4, 0x44, 0x44, 0x84, 0x84, 4, 0x44, 4, 0x44, 4, 0x44,
    4, 0x44, 0x44, 4, 0xc4, 0x84, 4, 0x44, 0x84, 0xc4,
];

pub(super) const SPRITE_MODULE_BURN_FLAME_GFX: [u8; 32] = [
    5, 4, 3, 1, 2, 0, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0,
];

pub(super) const SPRITE_MODULE_POOF_X_OFFSETS: [i8; 16] =
    [-6, 10, 1, 13, -6, 10, 1, 13, -7, 4, -5, 6, -1, 1, -2, 0];

pub(super) const SPRITE_MODULE_POOF_Y_OFFSETS: [i8; 16] =
    [-6, -4, 10, 9, -6, -4, 10, 9, -8, -10, 4, 3, -1, -2, 0, 1];

pub(super) const SPRITE_MODULE_POOF_CHARS: [u8; 16] = [
    0x9b, 0x9b, 0x9b, 0x9b, 0xb3, 0xb3, 0xb3, 0xb3, 0x8a, 0x8a, 0x8a, 0x8a, 0x8a, 0x8a, 0x8a, 0x8a,
];

pub(super) const SPRITE_MODULE_POOF_FLAGS: [u8; 16] = [
    0x24, 0xa4, 0x24, 0xa4, 0xe4, 0x64, 0xa4, 0x24, 0x24, 0xe4, 0xe4, 0xe4, 0x24, 0xe4, 0xe4, 0xe4,
];

pub(super) const SPRITE_MODULE_POOF_EXT_SIZES: [u8; 16] =
    [0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2];

pub(super) const SPRITE_MODULE_DROWN_DROWN_DRAW_FRAMES: [DrawMultipleData; 8] = [
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

pub(super) const SPRITE_MODULE_DROWN_OAM_FLAGS: [u8; 4] = [0, 0x40, 0xc0, 0x80];

pub(super) const SPRITE_MODULE_DROWN_OAM_CHARS: [u8; 11] = [
    0xc0, 0xc0, 0xc0, 0xc0, 0xcd, 0xcd, 0xcd, 0xcb, 0xcb, 0xcb, 0xcb,
];

pub(super) const SPRITE_MODULE_EXPLODE_SPRITE_EXPLODE_DRAW_FRAMES: [DrawMultipleData; 32] = [
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

pub(super) const SPRITE_MODULE_FALL2_FALLING_HUMANOID_GFX_BY_DELAY: [u8; 32] = [
    13, 13, 13, 13, 13, 13, 13, 12, 12, 12, 12, 12, 3, 3, 3, 3, 3, 2, 2, 2, 2, 1, 1, 1, 1, 0, 0, 0,
    0, 0, 0, 0,
];

pub(super) const SPRITE_MODULE_FALL2_FALLING_HELMA_BEETLE_GFX_BY_DELAY: [u8; 32] = [
    5, 5, 5, 5, 5, 5, 5, 4, 4, 4, 4, 4, 3, 3, 3, 3, 3, 2, 2, 2, 2, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0,
];

pub(super) const SPRITE_MODULE_FALL2_FALLING_TILE_CHECK_FRAME_MASKS: [u8; 16] = [
    0xff, 0x3f, 0x1f, 0x0f, 0x0f, 7, 3, 1, 0xff, 0x3f, 0x1f, 0x0f, 7, 3, 1, 0,
];

pub(super) const SPRITE_MODULE_FALL2_FALLING_DIRECTION_GFX_OFFSETS: [u8; 4] = [0, 4, 8, 0];

pub(super) const SPRITE_MODULE_CARRIED_SPRITE_HELD_Z_FOR_FRAME: [u8; 6] = [3, 2, 1, 3, 2, 1];

pub(super) const SPRITE_MODULE_CARRIED_SPRITE_HELD_X: [i8; 16] =
    [0, 0, 0, 0, 0, 0, 0, 0, -13, -10, -5, 0, 13, 10, 5, 0];

pub(super) const SPRITE_MODULE_CARRIED_SPRITE_HELD_Z: [u8; 16] =
    [13, 14, 15, 16, 0, 10, 22, 16, 8, 11, 14, 16, 8, 11, 14, 16];

pub(super) const CARRIED_SPRITE_CHECK_FOR_THROW_SPRITE_HELD_THROW_XVEL: [u8; 4] =
    [0, 0, (-62i8) as u8, 63];

pub(super) const CARRIED_SPRITE_CHECK_FOR_THROW_SPRITE_HELD_THROW_YVEL: [u8; 4] =
    [(-62i8) as u8, 63, 0, 0];

pub(super) const CARRIED_SPRITE_CHECK_FOR_THROW_SPRITE_HELD_THROW_ZVEL: [u8; 4] = [4, 4, 4, 4];

pub(super) const SPRITE_STUNNED_MAIN_FUNC1_SPRITE_STUNNED_MAIN_FUNC1_MASKS: [u8; 7] =
    [0x7f, 0x0f, 3, 1, 0, 0, 0];

pub(super) const SPRITE_STUNNED_MAIN_FUNC1_SPARKLE_GARNISH_XY: [i8; 4] = [-4, 12, 3, 8];

pub(super) const SPRITE_HANDLE_ABSORPTION_BY_PLAYER_ABSORPTION_SFX: [u8; 15] = [
    0x0b, 0x0a, 0x0a, 0x0a, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x2f, 0x2f, 0x0b,
];

pub(super) const SPRITE_HANDLE_ABSORPTION_BY_PLAYER_RUPEES_ABSORPTION: [u16; 3] = [1, 5, 20];

pub(super) const SPRITE_HANDLE_ABSORPTION_BY_PLAYER_BOMBS_ABSORPTION: [u8; 3] = [1, 4, 8];

pub(super) const SPRITE_HANDLE_ABSORPTION_BY_PLAYER_ABSORB_BIG_KEY: [u16; 2] = [0x4000, 0x2000];

pub(super) const SPRITE_CHECK_DAMAGE_TO_LINK_IGNORE_LAYER_SHIELD_BLOCK_FACING_TO_DIRECTION: [u8;
    4] = [6, 4, 0, 0];

pub(super) const SPRITE_CHECK_DAMAGE_TO_LINK_IGNORE_LAYER_SPRITE_DAMAGE_FACING_BY_DIRECTION: [u8;
    4] = [4, 6, 0, 2];

pub(super) const GUARD_PARRY_SWORD_ATTACKS_GUARD_PARRY_HITBOX_SIZE_BY_DIRECTION: [u8; 8] =
    [15, 15, 24, 15, 15, 19, 15, 15];

pub(super) const GUARD_PARRY_SWORD_ATTACKS_GUARD_PARRY_SWORD_STEP_BY_DIRECTION: [u8; 8] =
    [6, 6, 6, 12, 6, 6, 6, 15];

pub(super) const SPRITE_ATTEMPT_DAMAGE_TO_LINK_PLUS_RECOIL_PLAYER_DAMAGES: [u8; 30] = [
    2, 1, 1, 4, 4, 4, 0, 0, 0, 8, 4, 2, 8, 8, 8, 16, 8, 4, 32, 16, 8, 32, 24, 16, 24, 16, 8, 64,
    48, 24,
];

pub(super) const SPRITE_CHECK_TILE_PROPERTY_FUNC5_X: [i8; 54] = [
    8, 8, 2, 14, 8, 8, -2, 10, 8, 8, 1, 14, 4, 4, 4, 4, 4, 4, -2, 10, 8, 8, -25, 40, 8, 8, 2, 14,
    8, 8, -8, 23, 8, 8, -20, 36, 8, 8, -1, 16, 8, 8, -1, 16, 8, 8, -8, 24, 8, 8, -8, 24, 8, 3,
];

pub(super) const SPRITE_CHECK_TILE_PROPERTY_FUNC5_Y: [i8; 54] = [
    6, 20, 13, 13, 0, 8, 4, 4, 1, 14, 8, 8, 4, 4, 4, 4, -2, 10, 4, 4, -25, 40, 8, 8, 3, 16, 10, 10,
    -8, 25, 8, 8, -20, 36, 8, 8, -1, 16, 8, 8, 14, 3, 8, 8, -8, 24, 8, 8, -8, 32, 8, 8, 12, 4,
];

pub(super) const SPRITE_CHECK_TILE_PROPERTY_SIMPLIFIED_TILE_ATTR: [u8; 256] = [
    0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 3, 3, 3,
    0, 0, 0, 0, 0, 0, 1, 1, 4, 4, 4, 4, 4, 4, 4, 4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];

pub(super) const SPRITE_CHECK_TILE_PROPERTY_SPRITE_TILE_ATTR_SIMPLIFIED: [i8; 256] = [
    0, 1, 2, 3, 2, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1,
    1, 1, 1, 0, 0, 0, 1, 2, -1, -1, -1, -1, -1, -1, -1, -1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1,
    1, 1, 1, 0, 1, 1, 1, 1, 1, 0, 1, 0, 1, 0, 0, -1, -1, -1, -1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 2, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1,
];

pub(super) const SPRITE_CHECK_FOR_TILE_IN_DIRECTION_HORIZONTAL_SPRITE_TILE_DIRECTION_BITS: [u8; 4] =
    [8, 4, 2, 1];

pub(super) const SPRITE_CHECK_FOR_TILE_IN_DIRECTION_VERTICAL_SPRITE_TILE_DIRECTION_BITS: [u8; 4] =
    [8, 4, 2, 1];

pub(super) const ENTITY_CHECK_SLOPED_TILE_COLLISION_SLOPED_TILE: [u8; 32] = [
    7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 7, 6, 5, 4, 3, 2, 1, 0,
];

pub(super) const SPRITE_SPAWN_SECRET_SECRET_SPAWN_ITEMS_BY_TILE: [u8; 22] = [
    0xd9, 0x3e, 0x79, 0xd9, 0xdc, 0xd8, 0xda, 0xe4, 0xe1, 0xdc, 0xd8, 0xdf, 0xe0, 0x0b, 0x42, 0xd3,
    0x41, 0xd4, 0xd9, 0xe3, 0xd8, 0,
];

pub(super) const SPRITE_SPAWN_SECRET_SECRET_ITEM_SPAWN_FLAGS: [u8; 22] = [
    0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub(super) const SPRITE_SPAWN_SECRET_SECRET_ITEM_X_LOW_OFFSETS: [u8; 22] = [
    4, 0, 4, 4, 0, 4, 4, 4, 4, 0, 4, 4, 4, 0, 0, 0, 0, 0, 4, 0, 4, 4,
];

pub(super) const SPRITE_SPAWN_SECRET_SECRET_ITEM_IGNORE_PROJECTILE_FLAGS: [u8; 22] = [
    1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 1,
];

pub(super) const SPRITE_SPAWN_SECRET_SECRET_ITEM_Z_VELOCITIES: [u8; 22] = [
    16, 0, 0, 16, 0, 0, 16, 16, 16, 16, 0, 16, 10, 16, 0, 0, 0, 0, 16, 0, 0, 0,
];

pub(super) const SPRITE_RETURN_IF_RECOILING_SPRITE_RECOIL_DIRECTION_MASKS: [u8; 6] =
    [3, 1, 0, 0, 0xc, 3];

pub(super) const SPRITE_FALL_DRAW_SPRITE_FALL_CHAR: [u8; 8] =
    [0x83, 0x83, 0x83, 0x80, 0x80, 0x80, 0xb7, 0xb7];

pub(super) const SPRITE_DRAW_DISTRESS_CUSTOM_X_OFFSETS: [i8; 4] = [-3, 2, 7, 11];

pub(super) const SPRITE_DRAW_DISTRESS_CUSTOM_Y_OFFSETS: [i8; 4] = [-5, -7, -7, -5];

pub(super) const SPRITE_DRAW_FALLING_HELMA_BEETLE_FALL0: [DrawMultipleData; 12] = [
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

pub(super) const SPRITE_DRAW_FALLING_HUMANOID_X_OFFSETS: [i8; 56] = [
    -4, 4, -4, 12, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, -4, 12, -4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0,
    0, 0, -4, 12, -4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 4, 0, 0, 0, 4, 0, 0, 0,
];

pub(super) const SPRITE_DRAW_FALLING_HUMANOID_Y_OFFSETS: [i8; 56] = [
    -4, -4, 4, 12, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, -4, -4, 12, 4, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0,
    0, 0, -4, -4, 12, 4, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 4, 0, 0, 0, 4, 0, 0, 0,
];

pub(super) const SPRITE_DRAW_FALLING_HUMANOID_CHARS: [u8; 56] = [
    0xae, 0xa8, 0xa6, 0xaf, 0xaa, 0, 0, 0, 0xac, 0, 0, 0, 0xbe, 0, 0, 0, 0xa8, 0xae, 0xaf, 0xa6,
    0xaa, 0, 0, 0, 0xac, 0, 0, 0, 0xbe, 0, 0, 0, 0xa6, 0xaf, 0xae, 0xa8, 0xaa, 0, 0, 0, 0xac, 0, 0,
    0, 0xbe, 0, 0, 0, 0xb6, 0, 0, 0, 0x80, 0, 0, 0,
];

pub(super) const SPRITE_DRAW_FALLING_HUMANOID_FLAGS: [u8; 56] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0, 0, 0, 0x40, 0,
    0, 0, 0x40, 0, 0, 0, 0x80, 0x80, 0x80, 0x80, 0x80, 0, 0, 0, 0x80, 0, 0, 0, 0x80, 0, 0, 0, 1, 0,
    0, 0, 1, 0, 0, 0,
];

pub(super) const SPRITE_DRAW_FALLING_HUMANOID_EXT_SIZES: [u8; 56] = [
    0, 2, 2, 0, 2, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 2, 2, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0,
    2, 0, 0, 2, 2, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub(super) const SCATTER_DEBRIS_DRAW_X_OFFSETS: [i8; 12] =
    [-8, 8, 16, -5, 8, 15, -1, 7, 11, 1, 3, 8];

pub(super) const SCATTER_DEBRIS_DRAW_Y_OFFSETS: [i8; 12] = [7, 2, 12, 9, 2, 10, 11, 2, 11, 7, 3, 8];

pub(super) const SCATTER_DEBRIS_DRAW_CHARS: [u8; 12] = [
    0xe2, 0xe2, 0xe2, 0xe2, 0xf2, 0xf2, 0xf2, 0xe2, 0xe2, 0xf2, 0xe2, 0xe2,
];

pub(super) const SCATTER_DEBRIS_DRAW_FLAGS: [u8; 12] =
    [0, 0, 0, 0, 0x80, 0x40, 0, 0x80, 0x40, 0, 0, 0];

pub(super) const GARNISH16_THROWN_ITEM_DEBRIS_X_OFFSETS: [i16; 64] = [
    0, 8, 0, 8, -2, 9, -1, 9, -4, 9, -1, 10, -6, 9, -1, 12, -7, 9, -2, 13, -9, 9, -3, 14, -4, -4,
    9, 15, -3, -3, -3, 9, -4, 4, 6, 10, -1, 4, 6, 7, 0, 2, 4, 7, 1, 1, 5, 7, 0, -2, 8, 9, -1, -6,
    9, 10, -2, -7, 12, 11, -3, -9, 4, 6,
];

pub(super) const GARNISH16_THROWN_ITEM_DEBRIS_Y_OFFSETS: [i8; 64] = [
    0, 0, 8, 8, 0, -1, 10, 10, 0, -3, 11, 7, 1, -4, 12, 8, 1, -4, 13, 9, 2, -4, 16, 10, 14, 14, -4,
    11, 16, 16, 16, -1, 2, -5, 5, 1, 3, -7, 8, 2, 4, -8, 4, 10, -9, 4, 4, 12, -10, 4, 8, 14, -12,
    4, 8, 15, -15, 3, 8, 17, -17, 1, 18, 15,
];

pub(super) const GARNISH16_THROWN_ITEM_DEBRIS_CHARS: [u8; 64] = [
    0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58,
    0x48, 0x58, 0x58, 0x58, 0x48, 0x58, 0x58, 0x48, 0x48, 0x48, 0x58, 0x48, 0x48, 0x48, 0x48, 0x48,
    0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59,
    0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59,
];

pub(super) const GARNISH16_THROWN_ITEM_DEBRIS_FLAGS: [u8; 64] = [
    0x80, 0, 0x80, 0x40, 0x80, 0x40, 0x80, 0, 0, 0xc0, 0, 0x80, 0x80, 0x40, 0x80, 0, 0x80, 0xc0, 0,
    0x80, 0, 0, 0x80, 0, 0x80, 0x80, 0x80, 0x80, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0, 0x40, 0x40, 0x40,
    0, 0x40, 0x40, 0, 0, 0x80, 0, 0x40, 0x40, 0x40, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
    0x40, 0, 0, 0x40, 0, 0, 0,
];

pub(super) const SPRITE_RETURN_IF_LIFTED_PERMISSIVE_LIFTED_SPRITE_PLAYER_FACING_BY_DIRECTION: [u8;
    4] = [4, 6, 0, 2];

pub(super) const SPRITE_SHOW_SOLICITED_MESSAGE_MESSAGE_FACING_BY_DIRECTION: [u8; 4] = [4, 6, 0, 2];

pub(super) const SPRITE_SHOW_MESSAGE_UNCONDITIONAL_PLAYER_HANDLER_STATE_RECOIL_WALL_LOCAL: u8 = 13;

pub(super) const SPRITE_SHOW_MESSAGE_UNCONDITIONAL_PLAYER_HANDLER_STATE_GROUND_LOCAL: u8 = 0;

pub(super) const SPRITE_APPLY_CONVEYOR_CONVEYOR_TILE_X_ADJUSTMENTS: [i8; 4] = [0, 0, -1, 1];

pub(super) const SPRITE_APPLY_CONVEYOR_CONVEYOR_TILE_Y_ADJUSTMENTS: [i8; 4] = [-1, 1, 0, 0];

pub(super) const SPRITE_CONVERT_VELOCITY_TO_ANGLE_VELOCITY_TO_ANGLE_X_DOMINANT: [u8; 32] = [
    0, 0, 1, 1, 1, 2, 2, 2, 0, 0, 15, 15, 15, 14, 14, 14, 8, 8, 7, 7, 7, 6, 6, 6, 8, 8, 9, 9, 9,
    10, 10, 10,
];

pub(super) const SPRITE_CONVERT_VELOCITY_TO_ANGLE_VELOCITY_TO_ANGLE_Y_DOMINANT: [u8; 32] = [
    4, 4, 3, 3, 3, 2, 2, 2, 12, 12, 13, 13, 13, 14, 14, 14, 4, 4, 5, 5, 5, 6, 6, 6, 12, 12, 11, 11,
    11, 10, 10, 10,
];
