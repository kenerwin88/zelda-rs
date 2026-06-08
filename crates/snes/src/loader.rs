//! ROM loading + cartridge header heuristic.
//! Port of `zelda3/snes/snes_other.c`.

use crate::cart::CartType;
use crate::snes::Snes;

#[derive(Default, Debug, Clone)]
pub struct CartHeader {
    pub header_version: u8,
    pub name: String,
    pub speed: u8,
    pub kind: u8,
    pub coprocessor: u8,
    pub chips: u8,
    pub rom_size: u32,
    pub ram_size: u32,
    pub region: u8,
    pub maker: u8,
    pub version: u8,
    pub checksum_complement: u16,
    pub checksum: u16,
    pub maker_code: String,
    pub game_code: String,
    pub flash_size: u32,
    pub ex_ram_size: u32,
    pub special_version: u8,
    pub ex_coprocessor: u8,
    pub score: i16,
    pub pal: bool,
    pub cart_type: u8, // 1 = LoROM, 2 = HiROM
}

/// Replicates `snes_loadRom`. Identifies LoROM vs HiROM by header
/// heuristic, copies the ROM into the cart, and resets the system.
pub fn load_rom(snes: &mut Snes, mut data: &[u8]) -> Result<(), LoadRomError> {
    if data.len() < 0x8000 {
        return Err(LoadRomError::TooSmall(data.len()));
    }

    let mut headers: [CartHeader; 4] = Default::default();
    for h in &mut headers {
        h.score = -50;
    }
    if data.len() >= 0x8000 {
        read_header(data, 0x7fc0, &mut headers[0]);
    }
    if data.len() >= 0x8200 {
        read_header(data, 0x81c0, &mut headers[1]);
    }
    if data.len() >= 0x10000 {
        read_header(data, 0xffc0, &mut headers[2]);
    }
    if data.len() >= 0x10200 {
        read_header(data, 0x101c0, &mut headers[3]);
    }

    let mut max_score = 0;
    let mut used = 0;
    for (i, h) in headers.iter().enumerate() {
        if h.score > max_score {
            max_score = h.score;
            used = i;
        }
    }

    // odd-numbered slots are for ROMs with a 512-byte copier header
    if used & 1 != 0 {
        data = &data[0x200..];
    }
    if headers[used].cart_type > 2 {
        return Err(LoadRomError::UnsupportedType(headers[used].cart_type));
    }

    // expand to a power of two by mirroring the tail upwards
    let mut new_length = 0x8000usize;
    while data.len() > new_length {
        new_length *= 2;
    }
    let mut new_data = vec![0u8; new_length];
    new_data[..data.len()].copy_from_slice(data);
    let mut length = data.len();
    let mut test = 1usize;
    while length != new_length {
        if length & test != 0 {
            let (src_start, src_end) = (length - test, length);
            let dst_start = length;
            for i in 0..test {
                new_data[dst_start + i] = new_data[src_start + i];
            }
            let _ = src_end; // satisfy borrow checker / clarity
            length += test;
        }
        test *= 2;
    }

    let h = &headers[used];
    let cart_type = match h.cart_type {
        1 => CartType::LoRom,
        2 => CartType::HiRom,
        _ => CartType::Empty,
    };
    let ram_size = if h.chips > 0 { h.ram_size as usize } else { 0 };
    snes.cart.load(cart_type, &new_data, ram_size);
    snes.reset(true);
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum LoadRomError {
    #[error("rom too small ({0} bytes; need >= 0x8000)")]
    TooSmall(usize),
    #[error("unsupported cart type {0}")]
    UnsupportedType(u8),
}

fn ascii_or_dot(b: u8) -> char {
    if (0x20..0x7f).contains(&b) {
        b as char
    } else {
        '.'
    }
}

fn header_size(shift: u8) -> u32 {
    0x400u32.checked_shl(shift as u32).unwrap_or(0)
}

fn read_header(data: &[u8], location: usize, h: &mut CartHeader) {
    // ── 21-byte name ────────────────────────────────────────────────
    h.name = (0..21).map(|i| ascii_or_dot(data[location + i])).collect();
    h.speed = data[location + 0x15] >> 4;
    h.kind = data[location + 0x15] & 0xf;
    h.coprocessor = data[location + 0x16] >> 4;
    h.chips = data[location + 0x16] & 0xf;
    h.rom_size = header_size(data[location + 0x17]);
    h.ram_size = header_size(data[location + 0x18]);
    h.region = data[location + 0x19];
    h.maker = data[location + 0x1a];
    h.version = data[location + 0x1b];
    h.checksum_complement = ((data[location + 0x1d] as u16) << 8) | data[location + 0x1c] as u16;
    h.checksum = ((data[location + 0x1f] as u16) << 8) | data[location + 0x1e] as u16;

    h.header_version = 1;
    if h.maker == 0x33 {
        h.header_version = 3;
        h.maker_code = (0..2)
            .map(|i| ascii_or_dot(data[location - 0x10 + i]))
            .collect();
        h.game_code = (0..4)
            .map(|i| ascii_or_dot(data[location - 0xe + i]))
            .collect();
        h.flash_size = header_size(data[location - 4]);
        h.ex_ram_size = header_size(data[location - 3]);
        h.special_version = data[location - 2];
        h.ex_coprocessor = data[location - 1];
    } else if data[location + 0x14] == 0 {
        h.header_version = 2;
        h.ex_coprocessor = data[location - 1];
    }

    h.pal = (0x2..=0xc).contains(&h.region) || h.region == 0x11;
    h.cart_type = if location < 0x9000 { 1 } else { 2 };

    let mut score = 0i16;
    score += if h.speed == 2 || h.speed == 3 { 5 } else { -4 };
    score += if h.kind <= 3 || h.kind == 5 { 5 } else { -2 };
    score += if h.coprocessor <= 5 || h.coprocessor >= 0xe {
        5
    } else {
        -2
    };
    score += if h.chips <= 6 || h.chips == 9 || h.chips == 0xa {
        5
    } else {
        -2
    };
    score += if h.region <= 0x14 { 5 } else { -2 };
    score += if h.checksum.wrapping_add(h.checksum_complement) == 0xffff {
        8
    } else {
        -6
    };

    let reset_vector = data[location + 0x3c] as u16 | ((data[location + 0x3d] as u16) << 8);
    score += if reset_vector >= 0x8000 { 8 } else { -20 };

    let opcode_idx = location + 0x40 - 0x8000 + (reset_vector & 0x7fff) as usize;
    let opcode = data.get(opcode_idx).copied().unwrap_or(0);
    if opcode == 0x78 || opcode == 0x18 {
        score += 6; // sei, clc
    }
    if opcode == 0x4c || opcode == 0x5c || opcode == 0x9c {
        score += 3; // jmp abs, jml abl, stz abs
    }
    if opcode == 0x00 || opcode == 0xff || opcode == 0xdb {
        score -= 6; // brk, sbc alx, stp
    }

    h.score = score;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Construct the minimum valid LoROM image so the loader recognises
    /// it (good speed, type, region, reset vector, opcode, and a
    /// matching checksum/complement).
    fn fake_lorom(size: usize) -> Vec<u8> {
        let mut rom = vec![0u8; size];
        let header = 0x7fc0;
        rom[header..header + 21].copy_from_slice(b"TEST ROM             ");
        rom[header + 0x15] = 0x20; // LoROM (low nibble), speed 2 (high)
        rom[header + 0x16] = 0x02;
        rom[header + 0x17] = 0x05; // 32 KiB rom
        rom[header + 0x18] = 0x03;
        rom[header + 0x19] = 0x01; // USA
        rom[header + 0x1a] = 0x00;
        rom[header + 0x1b] = 0x00;
        rom[header + 0x1c] = 0x55;
        rom[header + 0x1d] = 0x55;
        rom[header + 0x1e] = 0xaa;
        rom[header + 0x1f] = 0xaa;
        // reset vector
        rom[header + 0x3c] = 0x00;
        rom[header + 0x3d] = 0x80;
        // first opcode at $008000 maps to byte 0
        rom[0x0000] = 0x78; // sei
        rom
    }

    #[test]
    fn loader_identifies_lorom() {
        let rom = fake_lorom(0x8000);
        let mut snes = Snes::new();
        load_rom(&mut snes, &rom).expect("load");
        assert_eq!(snes.cart.kind, CartType::LoRom);
    }

    #[test]
    fn loader_rejects_tiny_rom() {
        let mut snes = Snes::new();
        let result = load_rom(&mut snes, &[0u8; 0x100]);
        assert!(matches!(result, Err(LoadRomError::TooSmall(_))));
    }
}
