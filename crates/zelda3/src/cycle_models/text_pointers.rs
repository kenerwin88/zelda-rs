//! Native encoded-message pointer scan. A phase retires one source instruction;
//! the live CPU context and WRAM scratch pointer survive interrupt boundaries.
use super::{
    intro_clear::Access,
    native_work::Work,
    nmi_stack::{NativeNmiContext, CARRY, ZERO},
};
use snes::CpuMasterTimeline;

// The original indexed word load reads this whole window before checking
// whether Y is a command. Adjacent routine bytes are read as DATA only.
// No opcode decoder or source-PC dispatch is used by this continuation.
const LENGTH_READ_WINDOW: [u8; 257] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1c, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0xf3, 0x28, 0xf4, 0x28, 0xf3, 0x68, 0xc8, 0x28, 0x7f, 0x38, 0xc8, 0x68,
    0xf3, 0xa8, 0xf4, 0xa8, 0xf3, 0xe8, 0x25, 0x61, 0x44, 0x62, 0x42, 0x2e, 0x7f, 0x38, 0x00, 0x00,
    0x28, 0x00, 0x50, 0x00, 0xb8, 0x24, 0xba, 0x24, 0xbc, 0x24, 0xbe, 0x24, 0xb8, 0x24, 0xba, 0x24,
    0xbc, 0x24, 0xbe, 0x24, 0x1f, 0x00, 0x3f, 0x00, 0x5e, 0x00, 0x7d, 0x00, 0x9c, 0x00, 0xbc, 0x00,
    0xdb, 0x00, 0xfa, 0x00, 0x19, 0x01, 0x39, 0x01, 0x58, 0x01, 0x77, 0x01, 0x96, 0x01, 0xb6, 0x01,
    0xd5, 0x01, 0xf4, 0x01, 0x08, 0x03, 0x01, 0x01, 0x01, 0x01, 0x01, 0x02, 0x02, 0x02, 0x02, 0x01,
    0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x02, 0x02, 0x02, 0x02, 0x01, 0x01, 0x01, 0x01, 0x01,
    0x8b, 0x4b, 0xab, 0xa9, 0x1c, 0x85, 0x02, 0xc2, 0x30, 0xa9, 0x00, 0x80, 0x85, 0x00, 0xa2, 0x00,
    0x00, 0xa5, 0x00, 0x9f, 0xc0, 0x71, 0x7f, 0xa5, 0x01, 0x9f, 0xc1, 0x71, 0x7f, 0xe8, 0xe8, 0xe8,
    0xa7, 0x00, 0x29, 0xff, 0x00, 0xa8, 0xb9, 0x6b, 0xd3, 0x29, 0xff, 0x00, 0xc0, 0x67, 0x00, 0x90,
    0x05, 0xc0, 0x80, 0x00, 0x90, 0x03, 0xa9, 0x01, 0x00, 0x18, 0x65, 0x00, 0x85, 0x00, 0xc0, 0x7f,
    0x00, 0xf0, 0xce, 0xc0, 0x80, 0x00, 0xd0, 0x0f, 0xca, 0xca, 0xca, 0xa9, 0x40, 0xdf, 0x85, 0x00,
    0xa9, 0x0e, 0x00, 0x85, 0x02, 0x80, 0xba, 0xc0, 0xff, 0x00, 0xd0, 0xc4, 0xe2, 0x30, 0xab, 0x6b,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x03, 0x01, 0x02, 0x00, 0x03, 0x01, 0x04, 0x00, 0x0a,
    0x01,
];

pub(crate) fn table_byte(address: u32) -> Option<u8> {
    address
        .checked_sub(0x0e_d36b)
        .and_then(|i| LENGTH_READ_WINDOW.get(i as usize))
        .copied()
}

/// Reconstitute the encoded asset's physical stream, including the two
/// control markers read by the scan. Message lengths are not pointer outputs:
/// the continuation below still reads and interprets every command itself.
pub(crate) fn message_streams(messages: &[&[u8]]) -> Option<[Vec<u8>; 2]> {
    if messages.len() != 397 {
        return None;
    }
    let mut streams = [Vec::new(), Vec::new()];
    for (i, message) in messages.iter().enumerate() {
        let bank = usize::from(i >= 359);
        streams[bank].extend_from_slice(message);
        streams[bank].push(0x7f);
    }
    streams[0].extend_from_slice(&[0x80, 0xff]);
    streams[1].extend_from_slice(&[0xff, 0xff]);
    // The original scan increments a 16-bit pointer without a bank carry.
    if streams[0].len() > 0x8000 || streams[1].len() > 0x20c0 {
        return None;
    }
    Some(streams)
}

/// `$0E:D3EB..D44A`, M8/X8 entry, then M16/X16. Prices the actual encoded
/// tokens used to produce the pointer table; command arguments are skipped by
/// the ROM's length table. Derived with scripts/rom_block_costs.py, excluding
/// the caller's JSL. No raster, interrupt or duration measurement enters here.
pub(crate) fn scan_master_cycles(streams: &[Vec<u8>; 2]) -> Option<u64> {
    let mut master = 214 + 202; // Entry and first pointer publication.
    for (bank, stream) in streams.iter().enumerate() {
        let mut offset = 0;
        loop {
            let token = *stream.get(offset)?;
            let length = if (0x67..0x80).contains(&token) {
                usize::from(table_byte(0x0ed36b + u32::from(token))?)
            } else { 1 };
            if length == 0 { return None; }
            master += 204 + if token < 0x67 { 6 + 24 }
                else { 40 + if token < 0x80 { 6 } else { 24 } };
            master += 118;
            offset = offset.checked_add(length)?;
            if offset > stream.len() { return None; }
            if token == 0x7f {
                master += 6 + 202; // BEQ and next pointer.
                continue;
            }
            master += 40;
            if token == 0x80 {
                if bank != 0 || stream.get(offset) != Some(&0xff) { return None; }
                master += 176 + 202; // Switch physical bank; overwrite sentinel pointer.
                break;
            }
            master += 6 + 40;
            if token == 0xff {
                if bank != 1 || stream.get(offset) != Some(&0xff) { return None; }
                return Some(master + 94);
            }
            master += 6;
        }
    }
    None
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Call,
    SaveBank,
    ProgramBank,
    SetBank,
    LoadBank,
    StoreBank,
    Width,
    LoadStart,
    StoreStart,
    Index,
    PointerLow,
    WriteLow,
    PointerHigh,
    WriteHigh,
    Increment(u8),
    ReadToken,
    MaskToken,
    TokenIndex,
    CommandLength,
    MaskLength,
    CompareCommand,
    BranchCommand,
    CompareLimit,
    BranchLimit,
    SingleByte,
    ClearCarry,
    AddLength,
    SavePointer,
    CompareEnd,
    BranchEnd,
    CompareBank,
    BranchBank,
    Decrement(u8),
    SecondStart,
    SaveSecondStart,
    SecondBank,
    SaveSecondBank,
    Restart,
    CompareStop,
    BranchStop,
    ByteWidth,
    RestoreBank,
    Return,
    Complete,
    PaletteCall,
    PaletteWidth,
    PaletteIndex,
    PaletteZero,
    PaletteStore(u8),
    PaletteIncrement(u8),
    PaletteCompare,
    PaletteBranch,
    PaletteComplete,
    PaletteLoading,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TextPointers {
    phase: Phase,
    palettes: super::intro_palettes::IntroPalettes,
}
impl Default for TextPointers {
    fn default() -> Self {
        Self {
            phase: Phase::Call,
            palettes: Default::default(),
        }
    }
}
impl TextPointers {
    pub(crate) fn complete(&self) -> bool {
        matches!(self.phase, Phase::Complete | Phase::PaletteComplete) || self.caller_complete()
    }
    pub(crate) fn begin_palette_clear(&mut self) {
        assert_eq!(self.phase, Phase::Complete);
        self.phase = Phase::PaletteCall;
    }
    pub(crate) fn palette_clear_complete(&self) -> bool {
        self.phase == Phase::PaletteComplete
    }
    pub(crate) fn begin_palette_loads(&mut self) {
        assert_eq!(self.phase, Phase::PaletteComplete);
        self.phase = Phase::PaletteLoading;
    }
    pub(crate) fn caller_complete(&self) -> bool {
        self.phase == Phase::PaletteLoading && self.palettes.complete()
    }
    pub(crate) fn palette_source(
        &self,
        cpu: &NativeNmiContext,
    ) -> super::intro_palettes::PaletteSource {
        if self.phase == Phase::PaletteLoading {
            self.palettes.palette_source(cpu)
        } else {
            super::intro_palettes::PaletteSource::Constant
        }
    }
    pub(crate) fn routine(&self) -> u32 {
        use Phase::*;
        match self.phase {
            PaletteLoading => self.palettes.routine(),
            Call => 0x028116,
            PaletteCall => 0x02811a,
            PaletteWidth | PaletteIndex | PaletteZero | PaletteStore(_) | PaletteIncrement(_)
            | PaletteCompare | PaletteBranch | PaletteComplete => 0x02c5b2,
            _ => 0x0ed3eb,
        }
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl FnMut(u64, Access) -> u16,
    ) -> Option<u64> {
        use Phase::*;
        if self.complete() {
            return None;
        }
        if self.phase == PaletteLoading {
            return self.palettes.step(cpu, clock, bus);
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d),
            ((self.routine() >> 16) as u8, self.pc(), 0)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.phase = match self.phase {
            Call => {
                // Source bus groups the operand fetch before the stack writes.
                w.drain(24);
                w.drain(6);
                w.push(u16::from(w.cpu.bank), 1);
                w.push(0x8119, 2);
                w.cpu.bank = 14;
                SaveBank
            }
            SaveBank | ProgramBank => {
                w.drain(6);
                w.push(
                    u16::from(if self.phase == SaveBank {
                        w.cpu.db
                    } else {
                        w.cpu.bank
                    }),
                    1,
                );
                if self.phase == SaveBank {
                    ProgramBank
                } else {
                    SetBank
                }
            }
            SetBank | RestoreBank => {
                w.drain(12);
                w.cpu.db = w.pull(1) as u8;
                w.flags(u16::from(w.cpu.db), 1);
                if self.phase == SetBank {
                    LoadBank
                } else {
                    Return
                }
            }
            LoadBank => {
                w.drain(8);
                w.cpu.load_a_byte(0x1c);
                StoreBank
            }
            StoreBank => {
                w.drain(8);
                w.write(2, w.cpu.a & 255, 1);
                Width
            }
            Width => {
                w.drain(8);
                w.drain(6);
                w.cpu.p &= !0x30;
                LoadStart
            }
            LoadStart | SecondStart => {
                w.drain(16);
                w.cpu.a = if self.phase == LoadStart {
                    0x8000
                } else {
                    0xdf40
                };
                w.flags(w.cpu.a, 2);
                if self.phase == LoadStart {
                    StoreStart
                } else {
                    SaveSecondStart
                }
            }
            StoreStart | SaveSecondStart | SavePointer => {
                w.drain(8);
                w.write(0, w.cpu.a, 2);
                match self.phase {
                    StoreStart => Index,
                    SaveSecondStart => SecondBank,
                    _ => CompareEnd,
                }
            }
            Index => {
                w.drain(16);
                w.cpu.x = 0;
                w.flags(0, 2);
                PointerLow
            }
            PointerLow | PointerHigh => {
                w.drain(8);
                w.cpu.a = w.read(u32::from(self.phase == PointerHigh), 2);
                w.flags(w.cpu.a, 2);
                if self.phase == PointerLow {
                    WriteLow
                } else {
                    WriteHigh
                }
            }
            WriteLow | WriteHigh => {
                w.drain(24);
                w.write(
                    0x7f71c0 + u32::from(w.cpu.x) + u32::from(self.phase == WriteHigh),
                    w.cpu.a,
                    2,
                );
                if self.phase == WriteLow {
                    PointerHigh
                } else {
                    Increment(0)
                }
            }
            Increment(i) | Decrement(i) => {
                w.drain(6);
                let increment = matches!(self.phase, Increment(_));
                w.cpu.x = if increment {
                    w.cpu.x.wrapping_add(1)
                } else {
                    w.cpu.x.wrapping_sub(1)
                };
                w.flags(w.cpu.x, 2);
                if i < 2 {
                    if increment {
                        Increment(i + 1)
                    } else {
                        Decrement(i + 1)
                    }
                } else if increment {
                    ReadToken
                } else {
                    SecondStart
                }
            }
            ReadToken => {
                w.drain(8);
                let low = w.read(0, 2);
                let bank = w.read(2, 1);
                w.cpu.a = w.read((u32::from(bank) << 16) | u32::from(low), 2);
                w.flags(w.cpu.a, 2);
                MaskToken
            }
            MaskToken | MaskLength => {
                w.drain(16);
                w.cpu.a &= 255;
                w.flags(w.cpu.a, 2);
                if self.phase == MaskToken {
                    TokenIndex
                } else {
                    CompareCommand
                }
            }
            TokenIndex => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                CommandLength
            }
            CommandLength => {
                w.drain(16);
                w.drain(6);
                w.cpu.a = w.read((u32::from(w.cpu.db) << 16) + 0xd36b + u32::from(w.cpu.y), 2);
                w.flags(w.cpu.a, 2);
                MaskLength
            }
            CompareCommand | CompareLimit | CompareEnd | CompareBank | CompareStop => {
                w.drain(16);
                let (value, next) = match self.phase {
                    CompareCommand => (0x67, BranchCommand),
                    CompareLimit => (0x80, BranchLimit),
                    CompareEnd => (0x7f, BranchEnd),
                    CompareBank => (0x80, BranchBank),
                    _ => (0xff, BranchStop),
                };
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.y >= value);
                w.flags(w.cpu.y.wrapping_sub(value), 2);
                next
            }
            BranchCommand | BranchLimit | BranchEnd | BranchBank | BranchStop => {
                w.drain(8);
                let (taken, yes, no) = match self.phase {
                    BranchCommand => (w.cpu.p & CARRY == 0, SingleByte, CompareLimit),
                    BranchLimit => (w.cpu.p & CARRY == 0, ClearCarry, SingleByte),
                    BranchEnd => (w.cpu.p & ZERO != 0, PointerLow, CompareBank),
                    BranchBank => (w.cpu.p & ZERO == 0, CompareStop, Decrement(0)),
                    _ => (w.cpu.p & ZERO == 0, ReadToken, ByteWidth),
                };
                if taken {
                    w.drain(6);
                    yes
                } else {
                    no
                }
            }
            SingleByte => {
                w.drain(16);
                w.cpu.a = 1;
                w.flags(1, 2);
                ClearCarry
            }
            ClearCarry => {
                w.drain(6);
                w.cpu.p &= !CARRY;
                AddLength
            }
            AddLength => {
                w.drain(8);
                let rhs = w.read(0, 2);
                let old = w.cpu.a;
                let sum = u32::from(old) + u32::from(rhs) + u32::from(w.cpu.p & CARRY != 0);
                w.cpu.a = sum as u16;
                w.cpu.p = (w.cpu.p & !(CARRY | 0x40))
                    | u8::from(sum > 0xffff)
                    | if (!(old ^ rhs) & (old ^ w.cpu.a) & 0x8000) != 0 {
                        0x40
                    } else {
                        0
                    };
                w.flags(w.cpu.a, 2);
                SavePointer
            }
            SecondBank => {
                w.drain(16);
                w.cpu.a = 14;
                w.flags(14, 2);
                SaveSecondBank
            }
            SaveSecondBank => {
                w.drain(8);
                w.write(2, w.cpu.a, 2);
                Restart
            }
            Restart => {
                w.drain(8);
                w.drain(6);
                PointerLow
            }
            ByteWidth => {
                w.drain(8);
                w.drain(6);
                w.cpu.p |= 0x30;
                w.cpu.x &= 255;
                w.cpu.y &= 255;
                RestoreBank
            }
            Return => {
                w.drain(12);
                w.cpu.pc = w.pull(2).wrapping_add(1);
                w.cpu.bank = w.pull(1) as u8;
                Complete
            }
            PaletteCall => {
                w.drain(16);
                w.drain(6);
                w.push(0x811c, 2);
                PaletteWidth
            }
            PaletteWidth => {
                w.drain(8);
                w.drain(6);
                w.cpu.p &= !0x20;
                PaletteIndex
            }
            PaletteIndex => {
                w.drain(8);
                w.cpu.x = 0;
                w.flags(0, 1);
                PaletteZero
            }
            PaletteZero => {
                w.drain(16);
                w.cpu.a = 0;
                w.flags(0, 2);
                PaletteStore(0)
            }
            PaletteStore(i) => {
                w.drain(24);
                w.write(
                    0x7ec480 + u32::from(i) * 0x80 + u32::from(w.cpu.x),
                    w.cpu.a,
                    2,
                );
                if i < 4 {
                    PaletteStore(i + 1)
                } else {
                    PaletteIncrement(0)
                }
            }
            PaletteIncrement(i) => {
                w.drain(6);
                w.cpu.x = u16::from((w.cpu.x as u8).wrapping_add(1));
                w.flags(w.cpu.x, 1);
                if i == 0 {
                    PaletteIncrement(1)
                } else {
                    PaletteCompare
                }
            }
            PaletteCompare => {
                w.drain(8);
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.x >= 0x80);
                w.flags(u16::from((w.cpu.x as u8).wrapping_sub(0x80)), 1);
                PaletteBranch
            }
            PaletteBranch => {
                w.drain(8);
                if w.cpu.p & ZERO == 0 {
                    w.drain(6);
                    PaletteStore(0)
                } else {
                    PaletteComplete
                }
            }
            Complete | PaletteComplete | PaletteLoading => unreachable!(),
        };
        if self.phase != Complete {
            w.cpu.pc = self.pc();
        }
        Some(w.cycles)
    }
    fn pc(&self) -> u16 {
        use Phase::*;
        match self.phase {
            Call => 0x8116,
            SaveBank => 0xd3eb,
            ProgramBank => 0xd3ec,
            SetBank => 0xd3ed,
            LoadBank => 0xd3ee,
            StoreBank => 0xd3f0,
            Width => 0xd3f2,
            LoadStart => 0xd3f4,
            StoreStart => 0xd3f7,
            Index => 0xd3f9,
            PointerLow => 0xd3fc,
            WriteLow => 0xd3fe,
            PointerHigh => 0xd402,
            WriteHigh => 0xd404,
            Increment(i) => 0xd408 + u16::from(i),
            ReadToken => 0xd40b,
            MaskToken => 0xd40d,
            TokenIndex => 0xd410,
            CommandLength => 0xd411,
            MaskLength => 0xd414,
            CompareCommand => 0xd417,
            BranchCommand => 0xd41a,
            CompareLimit => 0xd41c,
            BranchLimit => 0xd41f,
            SingleByte => 0xd421,
            ClearCarry => 0xd424,
            AddLength => 0xd425,
            SavePointer => 0xd427,
            CompareEnd => 0xd429,
            BranchEnd => 0xd42c,
            CompareBank => 0xd42e,
            BranchBank => 0xd431,
            Decrement(i) => 0xd433 + u16::from(i),
            SecondStart => 0xd436,
            SaveSecondStart => 0xd439,
            SecondBank => 0xd43b,
            SaveSecondBank => 0xd43e,
            Restart => 0xd440,
            CompareStop => 0xd442,
            BranchStop => 0xd445,
            ByteWidth => 0xd447,
            RestoreBank => 0xd449,
            Return => 0xd44a,
            PaletteCall => 0x811a,
            PaletteWidth => 0xc5b2,
            PaletteIndex => 0xc5b4,
            PaletteZero => 0xc5b6,
            PaletteStore(i) => 0xc5b9 + u16::from(i) * 4,
            PaletteIncrement(i) => 0xc5cd + u16::from(i),
            PaletteCompare => 0xc5cf,
            PaletteBranch => 0xc5d1,
            PaletteComplete => 0xc5d3,
            Complete | PaletteLoading => unreachable!(),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, RomCpuTimingProbe, Snes,
        SourceCpuBusAccessKind as Kind, SourcePpuReadState,
    };
    #[test]
    fn text_scan_matches_every_source_instruction_for_corpus_and_command_edges() {
        let Ok(original) = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        ) else {
            return;
        };
        assert_eq!(&LENGTH_READ_WINDOW[..], &original[0x7536b..0x7546c]);
        for synthetic in [false, true] {
            let mut rom = original.clone();
            if synthetic {
                let mut bytes = Vec::new();
                for token in 0u8..=254 {
                    if token == 0x80 {
                        continue;
                    }
                    bytes.push(token);
                    if (0x67..0x80).contains(&token) && LENGTH_READ_WINDOW[usize::from(token)] == 2
                    {
                        // Parameters that look like stop/bank markers must be skipped.
                        bytes.push(if token & 1 == 0 { 0xff } else { 0x80 });
                    }
                }
                bytes.extend_from_slice(&[0x7f, 0x80, 0xff]);
                rom[0xe0000..0xe0000 + bytes.len()].copy_from_slice(&bytes);
                rom[0x75f40..0x75f46].copy_from_slice(&[0x67, 0x6c, 0xff, 0x7f, 0xff, 0xff]);
            }
            for (v, h) in [(238, 850), (224, 1338), (261, 1350)] {
                for odd in [false, true] {
                    let mut machine = Snes::new();
                    machine.cart.load(CartType::LoRom, &rom, 0x2000);
                    for (i, b) in machine.ram.iter_mut().enumerate() {
                        *b = (i ^ (i >> 8)) as u8;
                    }
                    let mut native = NativeNmiContext {
                        a: 0x0281,
                        x: 0xfc,
                        y: 0xff,
                        d: 0,
                        db: 0,
                        bank: 2,
                        pc: 0x8116,
                        sp: 0x1fc,
                        p: 0xb0,
                    };
                    machine.cpu.a = native.a;
                    machine.cpu.x = native.x;
                    machine.cpu.y = native.y;
                    machine.cpu.pc = native.pc;
                    machine.cpu.k = native.bank;
                    machine.cpu.sp = native.sp;
                    machine.cpu.e = false;
                    machine.cpu.unpack_flags(native.p);
                    let mut ram = machine.ram.clone();
                    let mut clock = CpuMasterTimeline::at_raster(
                        0,
                        CpuRasterPosition::new(v, h),
                        CpuBusWorkload::default(),
                        CpuFieldTiming::non_interlace(odd),
                    );
                    let mut source = RomCpuTimingProbe::new(
                        machine,
                        clock.clone(),
                        SourcePpuReadState::snes9x_reset(),
                    )
                    .unwrap();
                    clock.begin_synchronous_timeline().unwrap();
                    let mut work = TextPointers::default();
                    let mut palette = false;
                    let mut steps = 0;
                    let mut vblanks = 0;
                    let mut scan_master = 0;
                    while !work.complete() {
                        let phase = work.phase;
                        let operand_address =
                            (u32::from(native.bank) << 16) | u32::from(native.pc + 1);
                        let mut first_read = true;
                        let expected = source
                            .step()
                            .unwrap()
                            .accesses
                            .into_iter()
                            .filter(|a| match a.kind {
                                Kind::Write { .. } => true,
                                // Immediate operand fetches share the ROM read
                                // event kind. They are priced by the native
                                // phase; retain all actual data-bus reads here.
                                Kind::Read { .. } => {
                                    let operand = first_read && a.address == operand_address;
                                    first_read = false;
                                    !operand
                                        && matches!(
                                            phase,
                                            Phase::PointerLow
                                                | Phase::PointerHigh
                                                | Phase::ReadToken
                                                | Phase::CommandLength
                                                | Phase::AddLength
                                                | Phase::SetBank
                                                | Phase::RestoreBank
                                                | Phase::Return
                                        )
                                }
                                _ => false,
                            })
                            .map(|a| {
                                let address =
                                    if a.address >> 16 < 0x40 && a.address & 65535 < 0x2000 {
                                        a.address & 65535
                                    } else {
                                        a.address
                                    };
                                let (access, value) = match a.kind {
                                    Kind::Read { value, width } => {
                                        (Access::Read(address, width), value)
                                    }
                                    Kind::Write { value, width } => {
                                        (Access::Write(address, value, width), 0)
                                    }
                                    _ => panic!("unexpected scan access {a:?}"),
                                };
                                (a.timestamp.master_cycles(), access, value)
                            })
                            .collect::<Vec<_>>();
                        let mut actual = Vec::new();
                        let scan_instruction = work.routine() == 0x0ed3eb;
                        let instruction_master = work.step(&mut native, &mut clock, |at, access| {
                            let read = |address: u32| {
                                if address < 0x2000 {
                                    ram[address as usize]
                                } else if address >= 0x7e0000 {
                                    ram[address as usize - 0x7e0000]
                                } else {
                                    rom[((address >> 16) as usize & 127) * 0x8000
                                        + (address as usize & 0x7fff)]
                                }
                            };
                            let value = match access {
                                Access::Read(a, w) => {
                                    if w == 1 {
                                        u16::from(read(a))
                                    } else {
                                        u16::from_le_bytes([read(a), read(a + 1)])
                                    }
                                }
                                Access::Write(a, value, w) => {
                                    let offset = if a >= 0x7e0000 {
                                        a as usize - 0x7e0000
                                    } else {
                                        a as usize
                                    };
                                    ram[offset..offset + usize::from(w)]
                                        .copy_from_slice(&value.to_le_bytes()[..usize::from(w)]);
                                    0
                                }
                                Access::Vblank { .. } => {
                                    vblanks += 1;
                                    return 0;
                                }
                            };
                            actual.push((at, access, value));
                            value
                        });
                        if scan_instruction { scan_master += instruction_master.unwrap(); }
                        assert_eq!(
                            actual, expected,
                            "synthetic={synthetic} phase={phase:?} v={v} h={h} odd={odd}"
                        );
                        let c = &source.snes().cpu;
                        assert_eq!(
                            native,
                            NativeNmiContext {
                                a: c.a,
                                x: c.x,
                                y: c.y,
                                d: c.dp,
                                db: c.db,
                                bank: c.k,
                                pc: c.pc,
                                sp: c.sp,
                                p: c.pack_flags()
                            },
                            "phase={phase:?}"
                        );
                        assert_eq!(clock, *source.timeline(), "phase={phase:?}");
                        // Host suspension retains the native phase, including saved flags.
                        if work.complete() && !palette {
                            assert_eq!(scan_master_cycles(&[
                                rom[0xe0000..0xe8000].to_vec(),
                                rom[0x75f40..0x78000].to_vec(),
                            ]), Some(scan_master), "ROM token prices, synthetic={synthetic}");
                            assert_eq!((native.bank, native.pc, native.sp), (2, 0x811a, 0x1fc));
                            work.begin_palette_clear();
                            palette = true;
                        }
                        work = work.clone();
                        steps += 1;
                        assert!(steps < 1_000_000);
                    }
                    assert_eq!(ram, source.snes().ram);
                    assert_eq!((native.bank, native.pc, native.sp), (2, 0xc5d3, 0x1fa));
                    if !synthetic {
                        assert!(vblanks > 20);
                    }
                }
            }
        }
    }
}
