use zelda3::ZeldaState;

use crate::read_le_u16;

pub(crate) fn replay_sram_checksum_ok(bytes: &[u8], base: usize) -> bool {
    let mut sum = 0u16;
    for i in 0..0x280 {
        sum = sum.wrapping_add(read_le_u16(bytes, base + i * 2));
    }
    sum == 0x5a5a
}

pub(crate) fn replay_checksum_bytes(bytes: &[u8]) -> u32 {
    let mut hash = 2166136261u32;
    for &byte in bytes {
        hash = (hash ^ u32::from(byte)).wrapping_mul(16777619);
    }
    hash
}

pub(crate) fn replay_checksum_ram_range(ram: &[u8], start: usize, size: usize) -> u32 {
    let mut hash = 2166136261u32;
    for index in start..start + size {
        let byte = if parity::fingerprint_mask_contains(index) {
            0
        } else {
            ram[index]
        };
        hash = (hash ^ u32::from(byte)).wrapping_mul(16777619);
    }
    hash
}

pub(crate) fn replay_save_ancilla_dump(game: &ZeldaState) -> String {
    let mut out = String::from("ancilla");
    for k in 0..10 {
        if game.ram[0x0c4a + k] == 0 && game.ram[0x0c5e + k] == 0 && game.ram[0x03b1 + k] == 0 {
            continue;
        }
        out.push_str(&format!(
            " [{k}:t=0x{:02x} x=0x{:04x} y=0x{:04x} xv=0x{:02x} yv=0x{:02x} step=0x{:02x} aux=0x{:02x} item=0x{:02x} arr3=0x{:02x} floor=0x{:02x} floor2=0x{:02x}]",
            game.ram[0x0c4a + k],
            u16::from_le_bytes([game.ram[0x0c04 + k], game.ram[0x0c18 + k]]),
            u16::from_le_bytes([game.ram[0x0bfa + k], game.ram[0x0c0e + k]]),
            game.ram[0x0c2c + k],
            game.ram[0x0c22 + k],
            game.ram[0x0c54 + k],
            game.ram[0x03b1 + k],
            game.ram[0x0c5e + k],
            game.ram[0x039f + k],
            game.ram[0x0c7c + k],
            game.ram[0x03ca + k],
        ));
    }
    out
}

pub(crate) fn replay_save_ram_page_dump(game: &ZeldaState) -> String {
    let mut out = String::from("ram-pages");
    for page in 0..128usize {
        let start = page * 0x400;
        out.push_str(&format!(
            " [{start:05x}=0x{:08x}]",
            replay_checksum_ram_range(&game.ram, start, 0x400)
        ));
    }
    out
}

pub(crate) fn replay_save_ram0400_dump(game: &ZeldaState) -> String {
    let mut out = String::from("ram0400");
    for index in 0x400..0x800 {
        let byte = game.ram[index];
        if byte != 0 {
            out.push_str(&format!(" [{index:04x}=0x{byte:02x}]"));
        }
    }
    out
}

pub(crate) fn replay_save_ram0000_dump(game: &ZeldaState) -> String {
    let mut out = String::from("ram0000");
    for index in 0x000..0x400 {
        let byte = game.ram[index];
        if byte != 0 {
            out.push_str(&format!(" [{index:04x}=0x{byte:02x}]"));
        }
    }
    out
}

pub(crate) fn replay_save_requested_ram_page_dump(game: &ZeldaState) -> Option<String> {
    let raw = std::env::var("ZELDA3_REPLAY_RAM_DUMP_PAGE").ok()?;
    let parsed = raw
        .strip_prefix("0x")
        .or_else(|| raw.strip_prefix("0X"))
        .map_or_else(
            || raw.parse::<usize>().ok(),
            |hex| usize::from_str_radix(hex, 16).ok(),
        )?;
    let start = parsed & !0x3ff;
    if start >= game.ram.len() {
        return None;
    }
    let end = (start + 0x400).min(game.ram.len());
    let mut out = format!("ram-page-bytes page=0x{start:05x}");
    for index in start..end {
        let byte = game.ram[index];
        if byte != 0 {
            out.push_str(&format!(" [{index:05x}=0x{byte:02x}]"));
        }
    }
    Some(out)
}

pub(crate) fn replay_save_room_mask(game: &ZeldaState, room: u16) -> u16 {
    let offset = 0x1df80 + usize::from(room) * 2;
    u16::from_le_bytes([game.ram[offset], game.ram[offset + 1]])
}

fn replay_save_room_history(game: &ZeldaState) -> [u16; 4] {
    [
        u16::from_le_bytes([game.ram[0xb80], game.ram[0xb81]]),
        u16::from_le_bytes([game.ram[0xb82], game.ram[0xb83]]),
        u16::from_le_bytes([game.ram[0xb84], game.ram[0xb85]]),
        u16::from_le_bytes([game.ram[0xb86], game.ram[0xb87]]),
    ]
}

pub(crate) fn replay_save_garnish_dump(game: &ZeldaState) -> String {
    const GARNISH_TYPE: usize = 0x1f800;
    const GARNISH_Y_LO: usize = 0x1f81e;
    const GARNISH_X_LO: usize = 0x1f83c;
    const GARNISH_Y_HI: usize = 0x1f85a;
    const GARNISH_X_HI: usize = 0x1f878;
    const GARNISH_Y_VEL: usize = 0x1f896;
    const GARNISH_X_VEL: usize = 0x1f8b4;
    const GARNISH_COUNTDOWN: usize = 0x1f90e;
    const GARNISH_SPRITE: usize = 0x1f92c;
    const GARNISH_FLOOR: usize = 0x1f968;
    const GARNISH_OAM_FLAGS: usize = 0x1f9fe;

    let mut out = String::from("garnish");
    for k in 0..30 {
        if game.ram[GARNISH_TYPE + k] == 0 && game.ram[GARNISH_COUNTDOWN + k] == 0 {
            continue;
        }
        out.push_str(&format!(
            " [{k}:t=0x{:02x} cd=0x{:02x} x=0x{:04x} y=0x{:04x} xv=0x{:02x} yv=0x{:02x} spr=0x{:02x} floor=0x{:02x} oam=0x{:02x}]",
            game.ram[GARNISH_TYPE + k],
            game.ram[GARNISH_COUNTDOWN + k],
            u16::from_le_bytes([game.ram[GARNISH_X_LO + k], game.ram[GARNISH_X_HI + k]]),
            u16::from_le_bytes([game.ram[GARNISH_Y_LO + k], game.ram[GARNISH_Y_HI + k]]),
            game.ram[GARNISH_X_VEL + k],
            game.ram[GARNISH_Y_VEL + k],
            game.ram[GARNISH_SPRITE + k],
            game.ram[GARNISH_FLOOR + k],
            game.ram[GARNISH_OAM_FLAGS + k],
        ));
    }
    out
}

pub(crate) fn replay_save_room_history_dump(game: &ZeldaState) -> String {
    let mut out = String::from("room-history");
    for (k, room) in replay_save_room_history(game).into_iter().enumerate() {
        let mask = if room == 0xffff {
            0
        } else {
            replay_save_room_mask(game, room)
        };
        out.push_str(&format!(" [{k}:room=0x{room:04x} mask=0x{mask:04x}]"));
    }
    out
}

pub(crate) fn replay_save_room_mask_dump(game: &ZeldaState) -> String {
    let current_room = u16::from_le_bytes([game.ram[0x48e], game.ram[0x48f]]);
    let mut out = format!(
        "room-masks current=0x{:04x} current_room=0x{:04x}",
        replay_save_room_mask(game, current_room),
        current_room
    );
    for room in replay_save_room_history(game) {
        if room != 0xffff {
            out.push_str(&format!(
                " [room=0x{room:04x} mask=0x{:04x}]",
                replay_save_room_mask(game, room)
            ));
        }
    }
    out
}

pub(crate) fn replay_save_overlord_dump(game: &ZeldaState) -> String {
    let mut out = String::from("overlords");
    for k in 0..8 {
        if game.ram[0x0b00 + k] == 0 {
            continue;
        }
        out.push_str(&format!(
            " [{k}:t=0x{:02x} x=0x{:04x} y=0x{:04x} floor=0x{:02x} gen1=0x{:02x} gen2=0x{:02x}]",
            game.ram[0x0b00 + k],
            u16::from_le_bytes([game.ram[0x0b08 + k], game.ram[0x0b10 + k]]),
            u16::from_le_bytes([game.ram[0x0b18 + k], game.ram[0x0b20 + k]]),
            game.ram[0x0b40 + k],
            game.ram[0x0b28 + k],
            game.ram[0x0b30 + k],
        ));
    }
    out
}

pub(crate) fn replay_save_sprite_dump(game: &ZeldaState) -> String {
    let mut out = String::from("sprites");
    for k in 0..16 {
        if game.ram[0x0dd0 + k] == 0 && game.ram[0x0e20 + k] == 0 {
            continue;
        }
        out.push_str(&format!(
            " [{k}:t=0x{:02x} st=0x{:02x} ai=0x{:02x} head=0x{:02x} sub=0x{:02x} x=0x{:04x} y=0x{:04x} d=0x{:02x} c=0x{:02x} e=0x{:02x} f=0x{:02x} n=0x{:04x} delay=0x{:02x} bump=0x{:02x} hp=0x{:02x} hit=0x{:02x} give=0x{:02x}]",
            game.ram[0x0e20 + k],
            game.ram[0x0dd0 + k],
            game.ram[0x0d80 + k],
            game.ram[0x0eb0 + k],
            game.ram[0x0e80 + k],
            u16::from_le_bytes([game.ram[0x0d10 + k], game.ram[0x0d30 + k]]),
            u16::from_le_bytes([game.ram[0x0d00 + k], game.ram[0x0d20 + k]]),
            game.ram[0x0de0 + k],
            game.ram[0x0db0 + k],
            game.ram[0x0e90 + k],
            game.ram[0x0ea0 + k],
            u16::from_le_bytes([game.ram[0x0bc0 + k * 2], game.ram[0x0bc0 + k * 2 + 1]]),
            game.ram[0x0df0 + k],
            game.ram[0x0cd2 + k],
            game.ram[0x0e50 + k],
            game.ram[0x0ef0 + k],
            game.ram[0x0ce2 + k],
        ));
    }
    out
}

pub(crate) fn replay_save_door_dump(game: &ZeldaState) -> String {
    let mut out = format!(
        "doors opened=0x{:04x} opened_adj=0x{:04x} cur=0x{:04x} toggles={:04x}/{:04x} floor={:04x},{:04x} palace={:04x},{:04x} exit_count=0x{:04x} exits={:04x},{:04x},{:04x},{:04x}",
        u16::from_le_bytes([game.ram[0x400], game.ram[0x401]]),
        u16::from_le_bytes([game.ram[0x68c], game.ram[0x68d]]),
        u16::from_le_bytes([game.ram[0x68e], game.ram[0x68f]]),
        u16::from_le_bytes([game.ram[0x44e], game.ram[0x44f]]),
        u16::from_le_bytes([game.ram[0x450], game.ram[0x451]]),
        u16::from_le_bytes([game.ram[0x6c0], game.ram[0x6c1]]),
        u16::from_le_bytes([game.ram[0x6c2], game.ram[0x6c3]]),
        u16::from_le_bytes([game.ram[0x6d0], game.ram[0x6d1]]),
        u16::from_le_bytes([game.ram[0x6d2], game.ram[0x6d3]]),
        u16::from_le_bytes([game.ram[0x19e0], game.ram[0x19e1]]),
        u16::from_le_bytes([game.ram[0x19e2], game.ram[0x19e3]]),
        u16::from_le_bytes([game.ram[0x19e4], game.ram[0x19e5]]),
        u16::from_le_bytes([game.ram[0x19e6], game.ram[0x19e7]]),
        u16::from_le_bytes([game.ram[0x19e8], game.ram[0x19e9]]),
    );
    for k in 0..16 {
        let addr = u16::from_le_bytes([game.ram[0x19a0 + k * 2], game.ram[0x19a1 + k * 2]]);
        let kind = u16::from_le_bytes([game.ram[0x1980 + k * 2], game.ram[0x1981 + k * 2]]);
        let dir = u16::from_le_bytes([game.ram[0x19c0 + k * 2], game.ram[0x19c1 + k * 2]]);
        if addr != 0 || kind != 0 || dir != 0 {
            out.push_str(&format!(
                " [{k}:type=0x{kind:04x} addr=0x{addr:04x} dir=0x{dir:04x}]"
            ));
        }
    }
    out
}

pub(crate) fn replay_save_dungeon_attr_dump(game: &ZeldaState) -> String {
    const DUNG_BG2_ATTR_TABLE: usize = 0x12000;
    let target = std::env::var("ZELDA3_REPLAY_DUNGEON_ATTR_POS")
        .ok()
        .and_then(|value| {
            value
                .strip_prefix("0x")
                .or_else(|| value.strip_prefix("0X"))
                .and_then(|hex| usize::from_str_radix(hex, 16).ok())
                .or_else(|| value.parse::<usize>().ok())
        })
        .unwrap_or(0x05fb);
    let base = target.saturating_sub(2);
    let mut out = format!("dungeon-attrs target=0x{target:04x}");
    for pos in base..(base + 5).min(0x2000) {
        out.push_str(&format!(
            " [0x{pos:04x}=0x{:02x}]",
            game.ram[DUNG_BG2_ATTR_TABLE + pos]
        ));
    }
    out
}

pub(crate) fn replay_save_dungmap_dump(game: &ZeldaState) -> String {
    const DUNG_MAP_TAB5: [u16; 14] = [
        0x21, 0x23, 0x20, 0x21, 0x70, 0x12, 0x11, 0x212, 2, 0x217, 0x160, 0x12, 0x113, 0x171,
    ];
    const DUNG_MAP_TAB21: [u16; 3] = [137, 167, 79];
    const DUNG_MAP_TAB22: [u16; 3] = [169, 119, 190];

    let palace = read_le_u16(&game.ram, 0x040c);
    let raw_dung = usize::from(palace >> 1);
    let valid_dung = raw_dung < DUNG_MAP_TAB5.len();
    let dung = if valid_dung {
        raw_dung
    } else {
        DUNG_MAP_TAB5.len() - 1
    };
    let t5 = if valid_dung {
        (DUNG_MAP_TAB5[dung] & 0x0f) as u8
    } else {
        0
    };
    let floor1 = t5.wrapping_add(game.ram[0x00a4]);
    let mut room = read_le_u16(&game.ram, 0x00a0);
    for i in 0..3 {
        if room == DUNG_MAP_TAB21[i] {
            room = DUNG_MAP_TAB22[i];
        }
    }
    let layout = if valid_dung {
        game.replay_asset_memblk_bytes(97, raw_dung)
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let base = usize::from(floor1) * 25;
    let mut found = -1i32;
    let mut layout_bytes = String::new();
    for i in 0..25 {
        let value = layout.get(base + i).copied().unwrap_or(0x0f);
        if i != 0 {
            layout_bytes.push(',');
        }
        layout_bytes.push_str(&format!("{value:02x}"));
        if found < 0 && value == room as u8 {
            found = i as i32;
        }
    }
    format!(
        "dungmap state={} init={} floor=0x{:04x} idx=0x{:04x} palace=0x{:04x} room=0x{:04x} link=0x{:04x}/0x{:04x} dung={} t5=0x{:02x} floor1=0x{:02x} found={} vars=0x{:02x},0x{:04x},0x{:04x},0x{:04x},0x{:04x},0x{:04x},0x{:04x} layout={}",
        game.ram[0x0200],
        game.ram[0x020d],
        read_le_u16(&game.ram, 0x020e),
        read_le_u16(&game.ram, 0x0211),
        read_le_u16(&game.ram, 0x040c),
        read_le_u16(&game.ram, 0x00a0),
        read_le_u16(&game.ram, 0x0022),
        read_le_u16(&game.ram, 0x0020),
        raw_dung,
        t5,
        floor1,
        found,
        game.ram[0x0210],
        read_le_u16(&game.ram, 0x0215),
        read_le_u16(&game.ram, 0x0213),
        read_le_u16(&game.ram, 0x0217),
        read_le_u16(&game.ram, 0x0cf5),
        read_le_u16(&game.ram, 0x0fa8),
        read_le_u16(&game.ram, 0x0faa),
        layout_bytes,
    )
}

pub(crate) fn replay_save_message_dump(game: &ZeldaState) -> String {
    let read_pos = read_le_u16(&game.ram, 0x1cd9) as usize;
    let mut bytes = String::new();
    for k in 0..8 {
        if k != 0 {
            bytes.push(',');
        }
        let index = 0x11200 + read_pos + k;
        let byte = game.ram.get(index).copied().unwrap_or(0);
        bytes.push_str(&format!("{byte:02x}"));
    }
    format!(
        "message msgmod={} msg=0x{:04x} read=0x{:04x} state=0x{:02x} wait=0x{:04x}/0x{:02x} speed=0x{:02x}/0x{:02x} bytes={}",
        game.ram[0x1cd8],
        read_le_u16(&game.ram, 0x1cf0),
        read_pos,
        game.ram[0x1cd4],
        read_le_u16(&game.ram, 0x1ce0),
        game.ram[0x1ce9],
        game.ram[0x1cd5],
        game.ram[0x1cd6],
        bytes,
    )
}

pub(crate) fn replay_save_palette_dump(game: &ZeldaState) -> String {
    let mut words = String::new();
    for k in 0..8 {
        if k != 0 {
            words.push(',');
        }
        words.push_str(&format!(
            "{:04x}/{:04x}",
            read_le_u16(&game.ram, 0x0c300 + k * 2),
            read_le_u16(&game.ram, 0x0c500 + k * 2),
        ));
    }
    let armor = game.ram[0x0f35b];
    let gloves = game.ram[0x0f354];
    let armor_word = usize::from(armor) * 15 + 12;
    let armorfd = game.replay_asset_word(81, armor_word).unwrap_or(0xffff);
    let gloveclr0 = game.replay_gloves_color(0);
    let gloveclr1 = game.replay_gloves_color(1);
    format!(
        "palette aux=0x{:08x} main=0x{:08x} flag=0x{:02x} filter=0x{:04x} auxmain=0x{:04x} mainind=0x{:02x} sp0=0x{:02x} sp5=0x{:02x} sp6=0x{:02x} sp6r=0x{:02x} hud=0x{:02x} owmode=0x{:02x} sword=0x{:02x} shield=0x{:02x} armor=0x{:02x} gloves=0x{:02x} palfd={:04x}/{:04x} armorfd={:04x} gloveclr={:04x}/{:04x} words={}",
        replay_checksum_ram_range(&game.ram, 0x0c300, 0x200),
        replay_checksum_ram_range(&game.ram, 0x0c500, 0x200),
        game.ram[0x0015],
        read_le_u16(&game.ram, 0x0c007),
        read_le_u16(&game.ram, 0x0aa8),
        game.ram[0x0ab6],
        game.ram[0x0aac],
        game.ram[0x0aad],
        game.ram[0x0aae],
        game.ram[0x0ab1],
        game.ram[0x0ab2],
        game.ram[0x0ab3],
        game.ram[0x0f359],
        game.ram[0x0f35a],
        armor,
        gloves,
        read_le_u16(&game.ram, 0x0c300 + 0xfd * 2),
        read_le_u16(&game.ram, 0x0c500 + 0xfd * 2),
        armorfd,
        gloveclr0,
        gloveclr1,
        words,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fnv1a(bytes: &[u8]) -> u32 {
        let mut hash = 2166136261u32;
        for &byte in bytes {
            hash = (hash ^ u32::from(byte)).wrapping_mul(16777619);
        }
        hash
    }

    #[test]
    fn checksum_bytes_matches_fnv1a() {
        assert_eq!(replay_checksum_bytes(&[]), 2166136261);
        assert_eq!(replay_checksum_bytes(&[1, 2, 3, 4]), fnv1a(&[1, 2, 3, 4]));
    }

    #[test]
    fn ram_checksum_masks_volatile_fingerprint_bytes() {
        let mut ram = vec![0u8; 0x800];
        ram[0x653] = 0x11;
        ram[0x654] = 0xaa;
        ram[0x655] = 0x22;
        let masked_hash = replay_checksum_ram_range(&ram, 0x600, 0x100);

        ram[0x654] = 0x55;
        assert_eq!(replay_checksum_ram_range(&ram, 0x600, 0x100), masked_hash);

        ram[0x655] = 0x33;
        assert_ne!(replay_checksum_ram_range(&ram, 0x600, 0x100), masked_hash);
    }

    #[test]
    fn sram_checksum_accepts_complement_marker_sum() {
        let mut sram = vec![0u8; 0x500];
        sram[0] = 0x5a;
        sram[1] = 0x5a;

        assert!(replay_sram_checksum_ok(&sram, 0));

        sram[0] = 0x59;
        assert!(!replay_sram_checksum_ok(&sram, 0));
    }
}
