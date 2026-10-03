//! Native Startup_InitializeMemory instruction owner. NMI is disabled by
//! its reset caller; the caller retains the hardware clock between steps.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Width,
    SaveReturn,
    Index,
    Zero,
    Store(u8),
    Dex,
    DexAgain,
    Loop,
    Palette,
    SaveSlot,
    LoadSignature(u8),
    CompareSignature(u8),
    BranchSignature(u8),
    ZeroSignature(u8),
    StoreSignature(u8),
    RestoreReturn,
    WindowMask,
    ByteWidth,
    Brightness,
    StoreBrightness,
    Cgram,
    Return,
    Complete,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StartupMemoryWork {
    phase: Phase,
    index: u16,
    saved_return: u16,
    signature: u16,
    accumulator: u16,
    carry: bool,
    nz: u8,
}

impl Default for StartupMemoryWork {
    fn default() -> Self {
        Self {
            phase: Phase::Width,
            index: 0,
            saved_return: 0,
            signature: 0,
            accumulator: 0,
            carry: false,
            nz: 0,
        }
    }
}

fn word(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}
fn store(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}

impl StartupMemoryWork {
    pub(crate) fn clears_window_mask(&self) -> bool {
        self.phase == Phase::WindowMask
    }

    /// Retire one instruction. Values loaded before a pause remain owned by
    /// this continuation; branches do not reread mutable RAM.
    pub(crate) fn step(&mut self, ram: &mut [u8], sram: &mut [u8]) -> Option<u32> {
        use Phase::*;
        let (next, cycles) = match self.phase {
            Width => (SaveReturn, 22),
            SaveReturn => {
                self.saved_return = word(ram, 0x1fe);
                (Index, 40)
            }
            Index => {
                self.index = 0x3fe;
                (Zero, 24)
            }
            Zero => {
                self.accumulator = 0;
                (Store(0), 24)
            }
            Store(bank) => {
                store(ram, usize::from(self.index) + usize::from(bank) * 0x400, 0);
                (if bank == 7 { Dex } else { Store(bank + 1) }, 46)
            }
            Dex => {
                self.index -= 1;
                (DexAgain, 14)
            }
            DexAgain => {
                self.index -= 1;
                (Loop, 14)
            }
            Loop => {
                if self.index == 0 {
                    (Palette, 16)
                } else {
                    (Store(0), 22)
                }
            }
            Palette => {
                store(ram, 0xc500, 0);
                (SaveSlot, 48)
            }
            SaveSlot => {
                store(sram, 0x1ffe, 0);
                (LoadSignature(0), 48)
            }
            LoadSignature(slot) => {
                self.signature = word(sram, 0x3e5 + usize::from(slot) * 0x500);
                self.accumulator = self.signature;
                (CompareSignature(slot), 48)
            }
            CompareSignature(slot) => {
                self.carry = self.accumulator >= 0x55aa;
                (BranchSignature(slot), 24)
            }
            BranchSignature(slot) => {
                if self.signature == 0x55aa {
                    (Self::next_signature(slot), 22)
                } else {
                    (ZeroSignature(slot), 16)
                }
            }
            ZeroSignature(slot) => {
                self.accumulator = 0;
                (StoreSignature(slot), 24)
            }
            StoreSignature(slot) => {
                store(sram, 0x3e5 + usize::from(slot) * 0x500, 0);
                (Self::next_signature(slot), 48)
            }
            RestoreReturn => {
                store(ram, 0x1fe, self.saved_return);
                (WindowMask, 40)
            }
            WindowMask => (ByteWidth, 36),
            ByteWidth => (Brightness, 22),
            Brightness => {
                self.accumulator = (self.accumulator & 0xff00) | 0x80;
                (StoreBrightness, 16)
            }
            StoreBrightness => {
                ram[0x13] = 0x80;
                (Cgram, 24)
            }
            Cgram => {
                ram[0x15] = ram[0x15].wrapping_add(1);
                self.nz = (ram[0x15] & 0x80) | if ram[0x15] == 0 { 2 } else { 0 };
                (Return, 38)
            }
            Return => (Complete, 42),
            Complete => return None,
        };
        self.phase = next;
        Some(cycles)
    }

    fn next_signature(slot: u8) -> Phase {
        if slot == 2 {
            Phase::RestoreReturn
        } else {
            Phase::LoadSignature(slot + 1)
        }
    }

    pub(crate) fn returned_registers(&self) -> super::nmi_stack::NativeNmiContext {
        use super::nmi_stack::*;
        assert_eq!(self.phase, Phase::Complete);
        NativeNmiContext {
            a: self.accumulator,
            x: self.index,
            y: self.saved_return & 0xff, // LDY saved the return; SEP truncates Y.
            // Reset's LDA #0/TCD and LDA #$01FF/TCS establish D and S.
            // Sound_LoadIntroSongBank preserves bank/D/S and ends with CLI.
            d: 0,
            sp: 0x01ff,
            db: 0,
            bank: 0,
            pc: self.saved_return.wrapping_add(1),
            // Reset REP #$28 clears decimal, hardware reset clears overflow;
            // upload PHP/PLP preserves them. SEP #$30 sets both byte widths.
            // CMP owns carry and the final INC $15 owns N/Z.
            p: ACCUMULATOR_BYTE | INDEX_BYTE | self.nz | if self.carry { CARRY } else { 0 },
        }
    }

    #[cfg(test)]
    pub(crate) fn next_pc(&self) -> u32 {
        use Phase::*;
        match self.phase {
            Width => 0x87c0,
            SaveReturn => 0x87c2,
            Index => 0x87c5,
            Zero => 0x87c8,
            Store(bank) => 0x87cb + u32::from(bank) * 3,
            Dex => 0x87e3,
            DexAgain => 0x87e4,
            Loop => 0x87e5,
            Palette => 0x87e7,
            SaveSlot => 0x87eb,
            LoadSignature(slot) => 0x87ef + u32::from(slot) * 16,
            CompareSignature(slot) => 0x87f3 + u32::from(slot) * 16,
            BranchSignature(slot) => 0x87f6 + u32::from(slot) * 16,
            ZeroSignature(slot) => 0x87f8 + u32::from(slot) * 16,
            StoreSignature(slot) => 0x87fb + u32::from(slot) * 16,
            RestoreReturn => 0x881f,
            WindowMask => 0x8822,
            ByteWidth => 0x8825,
            Brightness => 0x8827,
            StoreBrightness => 0x8829,
            Cgram => 0x882b,
            Return => 0x882d,
            Complete => 0x802f,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_startup_registers_follow_signature_and_saved_return() {
        use snes::{
            CartType, CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline, CpuRasterPosition,
            RomCpuTimingProbe, Snes, SourcePpuReadState,
        };
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let Ok(rom) = std::fs::read(root.join("saves/zelda3.sfc")) else {
            return;
        };
        for signature in [0, 0x55aa, 0xffff] {
            for saved_return in [0x802e, 0x91ff, 0xffff] {
                let mut ram = vec![0x55; 0x20000];
                let mut sram = vec![0xff; 0x2000];
                store(&mut ram, 0x1fe, saved_return);
                store(&mut sram, 0xde5, signature);
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                machine.ram.copy_from_slice(&ram);
                machine.cart.ram.copy_from_slice(&sram);
                machine.cpu.e = false;
                machine.cpu.unpack_flags(0x30);
                machine.cpu.a = 0xabcd;
                machine.cpu.x = 0x45;
                machine.cpu.y = 0x67;
                machine.cpu.sp = 0x1fd;
                machine.cpu.dp = 0;
                machine.cpu.db = 0;
                machine.cpu.k = 0;
                machine.cpu.pc = 0x87c0;
                let timeline = CpuMasterTimeline::at_raster(
                    0,
                    CpuRasterPosition::new(100, 100),
                    CpuBusWorkload::default(),
                    CpuFieldTiming::non_interlace(false),
                );
                let mut source =
                    RomCpuTimingProbe::new(machine, timeline, SourcePpuReadState::snes9x_reset())
                        .unwrap();
                let mut work = StartupMemoryWork::default();
                while work.step(&mut ram, &mut sram).is_some() {
                    source.step().unwrap();
                }
                let native = work.returned_registers();
                let cpu = &source.snes().cpu;
                assert_eq!(
                    (
                        native.a,
                        native.x,
                        native.y,
                        native.d,
                        native.db,
                        native.bank,
                        native.pc,
                        native.sp,
                        native.p
                    ),
                    (
                        cpu.a,
                        cpu.x,
                        cpu.y,
                        cpu.dp,
                        cpu.db,
                        cpu.k,
                        cpu.pc,
                        cpu.sp,
                        cpu.pack_flags()
                    ),
                    "signature={signature:04x} return={saved_return:04x}"
                );
                assert_eq!(ram, source.snes().ram);
                assert_eq!(sram, source.snes().cart.ram);
            }
        }
    }

    #[test]
    fn startup_preserves_unwritten_words_and_loaded_return() {
        for valid_slots in 0..8 {
            let mut ram = vec![0x55; 0x20000];
            let mut sram = vec![0x77; 0x2000];
            store(&mut ram, 0x1fe, 0x802e);
            for slot in 0..3 {
                if valid_slots & (1 << slot) != 0 {
                    store(&mut sram, 0x3e5 + slot * 0x500, 0x55aa);
                }
            }
            let mut work = StartupMemoryWork::default();
            let mut instructions = 0;
            while work.step(&mut ram, &mut sram).is_some() {
                instructions += 1;
                // Pausing/restarting ownership must preserve the saved return
                // after the clearing loop overwrites its original RAM slot.
                work = work.clone();
            }
            assert!(instructions > 5600);
            for bank in 0..8 {
                assert_eq!(word(&ram, bank * 0x400), 0x5555);
            }
            for at in 2..0x2000 {
                let expected = match at {
                    0x13 => 0x80,
                    0x15 => 1,
                    0x1fe => 0x2e,
                    0x1ff => 0x80,
                    _ if at % 0x400 < 2 => 0x55,
                    _ => 0,
                };
                assert_eq!(ram[at], expected, "offset={at:04x}");
            }
            assert_eq!(word(&ram, 0xc500), 0);
            assert_eq!(word(&sram, 0x1ffe), 0);
            for slot in 0..3 {
                assert_eq!(
                    word(&sram, 0x3e5 + slot * 0x500),
                    if valid_slots & (1 << slot) != 0 {
                        0x55aa
                    } else {
                        0
                    }
                );
            }
        }
    }
}
