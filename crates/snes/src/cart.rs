//! Cartridge: ROM + on-cart SRAM, LoROM/HiROM bus mapping.
//!
//! Port of `zelda3/snes/cart.c`. The C version holds a back-pointer to
//! `Snes` only to fetch `openBus` on unmapped reads; we take that as a
//! parameter so the cart owns no foreign references.

#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CartType {
    Empty = 0,
    LoRom = 1,
    HiRom = 2,
}

impl CartType {
    pub fn from_raw(v: u8) -> Self {
        match v {
            1 => CartType::LoRom,
            2 => CartType::HiRom,
            _ => CartType::Empty,
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Cart {
    pub kind: CartType,
    pub rom: Vec<u8>,
    pub ram: Vec<u8>,
}

impl Cart {
    pub const C_SAVELOAD_SIZE: usize = 0x2000;

    pub fn new() -> Self {
        Self {
            kind: CartType::Empty,
            rom: Vec::new(),
            ram: vec![0; 0x2000],
        }
    }

    pub fn reset(&mut self) {
        for b in &mut self.ram {
            *b = 0;
        }
    }

    pub fn load(&mut self, kind: CartType, rom: &[u8], ram_size: usize) {
        assert_eq!(ram_size, self.ram.len(), "ramSize must match cart ramSize");
        self.kind = kind;
        self.rom = rom.to_vec();
        for b in &mut self.ram {
            *b = 0;
        }
    }

    pub fn read(&self, bank: u8, adr: u16, open_bus: u8) -> u8 {
        match self.kind {
            CartType::Empty => open_bus,
            CartType::LoRom => self.read_lorom(bank, adr, open_bus),
            CartType::HiRom => self.read_hirom(bank, adr, open_bus),
        }
    }

    pub fn write(&mut self, bank: u8, adr: u16, val: u8) {
        match self.kind {
            CartType::Empty => {}
            CartType::LoRom => self.write_lorom(bank, adr, val),
            CartType::HiRom => self.write_hirom(bank, adr, val),
        }
    }

    fn rom_byte(&self, idx: usize) -> u8 {
        let mask = self.rom.len().wrapping_sub(1);
        self.rom[idx & mask]
    }

    fn ram_idx(&self, idx: usize) -> usize {
        idx & self.ram.len().wrapping_sub(1)
    }

    fn read_lorom(&self, bank: u8, adr: u16, open_bus: u8) -> u8 {
        let bank32 = bank as usize;
        let adr32 = adr as usize;

        if adr >= 0x8000 {
            return self.rom_byte((bank32 << 15) | (adr32 & 0x7fff));
        }
        if ((0x70..0x7e).contains(&bank) || bank >= 0xf0) && adr < 0x8000 && !self.ram.is_empty() {
            return self.ram[self.ram_idx(((bank32 & 0xf) << 15) | adr32)];
        }
        if bank & 0x40 != 0 {
            return self.rom_byte((bank32 << 15) | (adr32 & 0x7fff));
        }
        open_bus
    }

    fn write_lorom(&mut self, bank: u8, adr: u16, val: u8) {
        // NOTE: C uses `bank > 0xf0` for the upper SRAM range (cart.c:87).
        // That excludes bank 0xf0 itself, which looks like a bug compared
        // to the read path's `bank >= 0xf0`. Preserve the C behavior so
        // the byte-for-byte oracle compare still matches.
        if ((0x70..0x7e).contains(&bank) || bank > 0xf0) && adr < 0x8000 && !self.ram.is_empty() {
            let bank32 = bank as usize;
            let adr32 = adr as usize;
            let idx = self.ram_idx(((bank32 & 0xf) << 15) | adr32);
            self.ram[idx] = val;
        }
    }

    fn read_hirom(&self, bank: u8, adr: u16, open_bus: u8) -> u8 {
        let bank = bank & 0x7f;
        let bank32 = bank as usize;
        let adr32 = adr as usize;

        if bank < 0x40 && (0x6000..0x8000).contains(&adr) && !self.ram.is_empty() {
            return self.ram[self.ram_idx(((bank32 & 0x3f) << 13) | (adr32 & 0x1fff))];
        }
        if adr >= 0x8000 || bank >= 0x40 {
            return self.rom_byte(((bank32 & 0x3f) << 16) | adr32);
        }
        open_bus
    }

    fn write_hirom(&mut self, bank: u8, adr: u16, val: u8) {
        let bank = bank & 0x7f;
        let bank32 = bank as usize;
        let adr32 = adr as usize;

        if bank < 0x40 && (0x6000..0x8000).contains(&adr) && !self.ram.is_empty() {
            let idx = self.ram_idx(((bank32 & 0x3f) << 13) | (adr32 & 0x1fff));
            self.ram[idx] = val;
        }
    }

    /// Byte layout used by C `cart_saveload`.
    pub fn save_c_saveload(&self) -> Vec<u8> {
        let mut out = vec![0; Self::C_SAVELOAD_SIZE];
        let len = self.ram.len().min(Self::C_SAVELOAD_SIZE);
        out[..len].copy_from_slice(&self.ram[..len]);
        out
    }

    pub fn load_c_saveload(&mut self, data: &[u8]) -> Result<(), String> {
        if data.len() != Self::C_SAVELOAD_SIZE {
            return Err(format!(
                "invalid cart saveload size {}, expected {}",
                data.len(),
                Self::C_SAVELOAD_SIZE
            ));
        }
        if self.ram.len() != Self::C_SAVELOAD_SIZE {
            self.ram.resize(Self::C_SAVELOAD_SIZE, 0);
        }
        self.ram.copy_from_slice(data);
        Ok(())
    }
}

impl Default for Cart {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fill_rom(size: usize) -> Vec<u8> {
        (0..size).map(|i| (i & 0xff) as u8).collect()
    }

    #[test]
    fn empty_cart_returns_open_bus() {
        let cart = Cart::new();
        assert_eq!(cart.read(0, 0x8000, 0xaa), 0xaa);
    }

    #[test]
    fn lorom_reads_rom_at_8000() {
        let mut cart = Cart::new();
        let rom = fill_rom(0x10000);
        cart.load(CartType::LoRom, &rom, 0x2000);
        // bank 0, $8000 maps to rom[0]
        assert_eq!(cart.read(0, 0x8000, 0), rom[0]);
        // bank 1, $8000 maps to rom[0x8000]
        assert_eq!(cart.read(1, 0x8000, 0), rom[0x8000]);
    }

    #[test]
    fn lorom_sram_roundtrip() {
        let mut cart = Cart::new();
        let rom = fill_rom(0x10000);
        cart.load(CartType::LoRom, &rom, 0x2000);
        cart.write(0x70, 0x0123, 0x5a);
        assert_eq!(cart.read(0x70, 0x0123, 0), 0x5a);
    }

    #[test]
    fn hirom_reads_rom_at_8000() {
        let mut cart = Cart::new();
        let rom = fill_rom(0x10000);
        cart.load(CartType::HiRom, &rom, 0x2000);
        assert_eq!(cart.read(0xc0, 0x8000, 0), rom[0x8000]);
    }

    #[test]
    fn hirom_sram_roundtrip() {
        let mut cart = Cart::new();
        let rom = fill_rom(0x10000);
        cart.load(CartType::HiRom, &rom, 0x2000);
        cart.write(0x20, 0x6000, 0x33);
        assert_eq!(cart.read(0x20, 0x6000, 0), 0x33);
    }
}
