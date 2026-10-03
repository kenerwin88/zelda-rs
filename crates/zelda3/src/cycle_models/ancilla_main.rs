//! Ancilla_ExecuteOne and the blanket's shared coordinate/OAM work. The
//! dispatch header owns the real allocation and timer work before handing
//! unsupported bodies back to its caller; no active body is silently skipped.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
    oam_allocate::OamAllocate,
};
use snes::CpuMasterTimeline;

// Immutable near-call targets at $08:837F, indexed by (kind - 1) * 2.
const HANDLERS: [u16; 67] = [
    0x851b, 0x86d2, 0x8852, 0x8d19, 0x90fc, 0x93e8, 0x955a, 0x9fb6, 0xa131, 0xa45b, 0xa4dd, 0xddc5,
    0xddca, 0xa60e, 0xa60e, 0xa60e, 0xa536, 0xa60e, 0x8435, 0xa80d, 0xa80f, 0xa8e5, 0xa9a9, 0xaaa0,
    0xb0ce, 0xbab0, 0x93ff, 0xb66a, 0xbbbc, 0xbc92, 0xbd74, 0xc013, 0xc094, 0xc38a, 0xd3bc, 0xee01,
    0xa8e3, 0xd65a, 0xdde8, 0xc6f2, 0xca8c, 0xd7b2, 0xd8fd, 0xe365, 0xe9e8, 0xeb3e, 0xec13, 0xdb24,
    0xdc70, 0xaa35, 0xa60e, 0xef9a, 0xc25f, 0xcfaa, 0xd03d, 0xd1d8, 0xea83, 0xf18d, 0xc167, 0xc1ea,
    0xca01, 0xcbf2, 0xd519, 0xd49a, 0xecaf, 0xc7de, 0xcca0,
];
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    if (0x08837f..0x088405).contains(&address) {
        let offset = (address - 0x08837f) as usize;
        Some(HANDLERS[offset / 2].to_le_bytes()[offset & 1])
    } else {
        crate::ZeldaState::native_ancilla_oam_table_byte(address)
            .or_else(|| crate::ZeldaState::native_oam_table_byte(address))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AncillaMain {
    pc: u16,
    allocator: Option<OamAllocate>,
    near_returns: Vec<u16>,
    child: Option<u16>,
    complete: bool,
}
impl Default for AncillaMain {
    fn default() -> Self {
        Self::from_entry(0x833c)
    }
}
impl AncillaMain {
    fn from_entry(pc: u16) -> Self {
        assert!(matches!(pc, 0x833c | 0xc013 | 0xf671 | 0xf6e1));
        Self {
            pc,
            allocator: None,
            near_returns: Vec::new(),
            child: None,
            complete: false,
        }
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn child(&self) -> Option<u16> {
        self.child
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.complete {
            return None;
        }
        assert!(
            self.child.is_none(),
            "active ancilla body still owns the CPU"
        );
        if let Some(allocator) = self.allocator.as_mut() {
            let cycles = allocator.step(cpu, clock, bus);
            if allocator.complete() {
                assert_eq!((cpu.bank, cpu.pc), (8, self.pc));
                self.allocator = None;
            }
            return cycles;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.db, cpu.d, cpu.p & 0x18),
            (8, self.pc, 8, 0, 0x10)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let pc = self.pc;
        let m = if w.cpu.p & 0x20 != 0 { 1 } else { 2 };
        w.fetch();
        self.pc = match pc {
            0x833c | 0xc024 | 0xf6e1 | 0xc059 => {
                w.drain(6);
                let v = match pc {
                    0x833c => w.cpu.a & 255,
                    0xc059 => w.cpu.y,
                    _ => w.cpu.x,
                };
                w.push(v, 1);
                pc + 1
            }
            0x833d => {
                w.compare_x_immediate(6, 1);
                pc + 2
            }
            0x833f | 0x8347 | 0x834c | 0x8352 | 0x8358 | 0x8364 | 0x8369 | 0xc028 | 0xc030
            | 0xc03b | 0xc075 | 0xc07b | 0xc08d | 0xf6e6 | 0xf6ea | 0xf6f4 => {
                let z = w.cpu.p & ZERO != 0;
                let (take, target) = match pc {
                    0x833f => (w.cpu.p & CARRY != 0, 0x8362),
                    0x8347 => (z, 0x835a),
                    0x834c => (!z, 0x8354),
                    0x8352 | 0x8358 => (true, 0x835e),
                    0x8364 => (!z, 0x836e),
                    0x8369 => (z, 0x836e),
                    0xc028 => (!z, 0xc032),
                    0xc030 => (true, 0xc038),
                    0xc03b => (z, 0xc03f),
                    0xc075 => (w.cpu.p & NEGATIVE != 0, 0xc08f),
                    0xc07b => (!z, 0xc046),
                    0xc08d => (true, 0xc046),
                    0xf6e6 | 0xf6ea => (!z, 0xf6f7),
                    _ => (w.cpu.p & CARRY != 0, 0xf6f7),
                };
                if w.branch(take) {
                    target
                } else {
                    pc + 2
                }
            }
            0x8341 | 0x8349 | 0x8366 | 0x8372 | 0x8377 | 0xc049 | 0xc04f | 0xf671 | 0xf674
            | 0xf67b | 0xf680 | 0xf685 | 0xf68a => {
                let (base, index, ldy) = match pc {
                    0x8341 => (0xc90, w.cpu.x, false),
                    0x8349 | 0xf671 => (0xc7c, w.cpu.x, true),
                    0x8366 => (0xc68, w.cpu.x, true),
                    0x8372 => (0x837f, w.cpu.y, false),
                    0x8377 => (0x8380, w.cpu.y, false),
                    0xc049 => (0xc003, w.cpu.x, false),
                    0xc04f => (0xc00b, w.cpu.x, false),
                    0xf674 => (0xf66d, w.cpu.y, false),
                    0xf67b => (0xbfa, w.cpu.x, false),
                    0xf680 => (0xc0e, w.cpu.x, false),
                    0xf685 => (0xc04, w.cpu.x, false),
                    _ => (0xc18, w.cpu.x, false),
                };
                w.drain(16);
                if (base & 255) + index > 255 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(base) + u32::from(index), 1);
                if ldy {
                    w.cpu.y = v;
                    w.flags(v, 1);
                } else {
                    w.load(v, 1);
                }
                pc + 3
            }
            0x8344 | 0xc025 | 0xc038 => {
                w.drain(16);
                let v = w.read(w.absolute(if pc == 0x8344 { 0xfb3 } else { 0x37d }), 1);
                if pc == 0x8344 {
                    w.cpu.y = v;
                    w.flags(v, 1);
                } else {
                    w.load(v, 1);
                }
                pc + 3
            }
            0x834e | 0x8354 | 0x835a | 0xc02c | 0xc034 => {
                let target = match pc {
                    0x834e => 0xba8c,
                    0x8354 => 0xba94,
                    0xc02c => 0xba84,
                    _ => 0xba80,
                };
                w.operand_then_internal(3);
                w.push(8, 1);
                w.push(pc + 3, 2);
                w.cpu.bank = 13;
                w.cpu.pc = target;
                self.allocator = Some(OamAllocate::new(target));
                pc + 4
            }
            0x835e | 0xc05a | 0xf6f8 => {
                w.drain(6);
                w.load(if pc == 0xf6f8 { w.cpu.x } else { w.cpu.y }, 1);
                pc + 1
            }
            0x835f => {
                w.operand_then_internal(2);
                w.write(w.absolute(0xc86) + u32::from(w.cpu.x), w.cpu.a & 255, 1);
                pc + 3
            }
            0x8362 => {
                w.cpu.y = w.dp_read(0x11, 1);
                w.flags(w.cpu.y, 1);
                pc + 2
            }
            0x836b => {
                w.operand_then_internal(2);
                let a = w.absolute(0xc68) + u32::from(w.cpu.x);
                let v = w.read(a, 1).wrapping_sub(1) & 255;
                w.drain(6);
                w.flags(v, 1);
                w.write(a, v, 1);
                pc + 3
            }
            0x836e | 0xc065 | 0xc08f | 0xf6fc => {
                w.drain(12);
                let v = w.pull(1);
                if pc == 0x836e {
                    w.load(v, 1);
                } else {
                    if pc == 0xc065 {
                        w.cpu.y = v;
                    } else {
                        w.cpu.x = v;
                    }
                    w.flags(v, 1);
                }
                pc + 1
            }
            0x836f => {
                w.drain(6);
                w.load(w.cpu.a.wrapping_sub(1) & 255, 1);
                pc + 1
            }
            0x8370 => {
                w.shift_left(1);
                pc + 1
            }
            0x8371 | 0xc060 => {
                w.drain(6);
                w.cpu.y = w.cpu.a & 255;
                w.flags(w.cpu.y, 1);
                pc + 1
            }
            0x8375 | 0x837a | 0xc01a | 0xc01e | 0xc020 | 0xc042 | 0xc06f | 0xc081 | 0xc089
            | 0xf677 | 0xf679 | 0xf67e | 0xf683 | 0xf688 | 0xf68d | 0xf696 | 0xf69d | 0xf69f => {
                let a = match pc {
                    0x8375 | 0xc089 | 0xf67e | 0xf696 => 0,
                    0x837a | 0xf683 => 1,
                    0xc06f | 0xc081 | 0xf688 | 0xf69d => 2,
                    0xf68d => 3,
                    0xc01a | 0xf69f => 4,
                    0xc01e => 6,
                    0xc020 => 8,
                    0xc042 => 10,
                    0xf677 => 0x65,
                    _ => 0x64,
                };
                w.dp_store(
                    a,
                    if pc == 0xf679 {
                        0
                    } else {
                        w.cpu.a & if m == 1 { 255 } else { 65535 }
                    },
                    m,
                );
                pc + 2
            }
            0x837c => {
                w.drain(16);
                let target = w.read(0, 2);
                if target != 0xc013 {
                    self.child = Some(target);
                }
                target
            }
            0xc013 | 0xc046 => {
                w.operand_then_internal(2);
                w.push(pc + 2, 2);
                self.near_returns.push(pc + 3);
                if pc == 0xc013 {
                    0xf671
                } else {
                    0xf6e1
                }
            }
            0xc016 | 0xc022 | 0xc067 | 0xc071 | 0xc07d | 0xc08b | 0xf68f | 0xf6a1 => {
                w.width(0x20, matches!(pc, 0xc016 | 0xc067 | 0xc07d | 0xf68f));
                pc + 2
            }
            0xc018 | 0xc01c | 0xc069 | 0xc077 | 0xc07f | 0xc083 | 0xf691 | 0xf698 | 0xf6e4
            | 0xf6e8 | 0xf6ec | 0xf6f0 => {
                let a = match pc {
                    0xc018 | 0xf691 | 0xf6f0 => 0,
                    0xf6e4 => 1,
                    0xc01c | 0xf698 | 0xf6ec => 2,
                    0xf6e8 => 3,
                    0xc083 => 4,
                    0xc069 | 0xc07f => 6,
                    _ => 10,
                };
                let v = w.dp_read(a, m);
                w.load(v, m);
                pc + 2
            }
            0xc02a | 0xc032 | 0xc03d | 0xc040 | 0xc061 => {
                w.immediate(
                    match pc {
                        0xc02a | 0xc032 => 16,
                        0xc03d => 4,
                        0xc040 => 3,
                        _ => 2,
                    },
                    1,
                );
                pc + 2
            }
            0xc03f | 0xf6f6 => {
                w.drain(6);
                w.cpu.x = w.cpu.a & 255;
                w.flags(w.cpu.x, 1);
                pc + 1
            }
            0xc044 | 0xf6e2 => {
                w.drain(8);
                if pc == 0xc044 {
                    w.cpu.y = 0;
                    w.flags(0, 1);
                } else {
                    w.cpu.x = 0xf0;
                    w.flags(0xf0, 1);
                }
                pc + 2
            }
            0xc04c | 0xc056 | 0xc063 | 0xf6ee | 0xf6f9 => {
                w.drain(8);
                let pointer = w.read(if pc == 0xc063 { 0x92 } else { 0x90 }, 2);
                w.drain(6);
                w.write(w.absolute(pointer) + u32::from(w.cpu.y), w.cpu.a & 255, 1);
                pc + 2
            }
            0xc04e | 0xc058 | 0xf6f7 | 0xf6fb | 0xc066 => {
                w.drain(6);
                let v = if pc == 0xc066 {
                    &mut w.cpu.x
                } else {
                    &mut w.cpu.y
                };
                *v = (*v + 1) & 255;
                let value = *v;
                w.flags(value, 1);
                pc + 1
            }
            0xc052 => {
                w.drain(8);
                w.load((w.cpu.a | 13) & 255, 1);
                pc + 2
            }
            0xc054 => {
                let v = w.dp_read(0x65, 1);
                w.load((w.cpu.a | v) & 255, 1);
                pc + 2
            }
            0xc05b | 0xf693 | 0xf69a => {
                w.drain(6);
                w.cpu.p |= CARRY;
                pc + 1
            }
            0xc05c => {
                w.drain(8);
                w.sbc(4, 1);
                pc + 2
            }
            0xc05e | 0xc05f => {
                w.drain(6);
                let v = w.cpu.a & 255;
                w.cpu.p = (w.cpu.p & !CARRY) | (v & 1) as u8;
                w.load(v >> 1, 1);
                pc + 1
            }
            0xc06b | 0xc085 => {
                w.clear_carry();
                pc + 1
            }
            0xc06c | 0xc086 => {
                w.drain(16);
                w.adc(if pc == 0xc06c { 16 } else { 8 });
                pc + 3
            }
            0xc073 => {
                w.dp_increment(10, 1, false);
                pc + 2
            }
            0xc079 | 0xf6f2 => {
                w.compare_immediate(if pc == 0xc079 { 1 } else { 0xf0 }, 1);
                pc + 2
            }
            0xf694 | 0xf69b => {
                let v = w.dp_read(if pc == 0xf694 { 0xe8 } else { 0xe2 }, 2);
                w.sbc(v, 2);
                pc + 2
            }
            0xc090 | 0xf6a3 | 0xf6fd => {
                w.rts();
                if let Some(returned) = self.near_returns.pop() {
                    assert_eq!(w.cpu.pc, returned);
                } else {
                    self.complete = true;
                }
                w.cpu.pc
            }
            _ => unreachable!("ancilla continuation {pc:04x}"),
        };
        if self.allocator.is_none() {
            w.cpu.pc = self.pc;
        }
        Some(w.cycles)
    }
}

#[cfg(test)]
mod tests {
    use super::super::sprite_main::SpriteMain;
    use super::*;

    enum Fixture {
        Ancilla(AncillaMain),
        Sprites(SpriteMain),
    }
    impl Fixture {
        fn finished(&self) -> bool {
            match self {
                Self::Ancilla(w) => w.complete() || w.child().is_some(),
                Self::Sprites(w) => w.complete() || w.child().is_some(),
            }
        }
        fn step(
            &mut self,
            cpu: &mut NativeNmiContext,
            clock: &mut CpuMasterTimeline,
            bus: impl WorkBus,
        ) {
            match self {
                Self::Ancilla(w) => w.step(cpu, clock, bus),
                Self::Sprites(w) => w.step(cpu, clock, bus),
            }
            .unwrap();
        }
    }

    fn run(rom: &[u8], kind: u8, slot: usize, case: usize, field: u64, composed: bool) -> usize {
        let mut machine = snes::Snes::new();
        machine.cart.load(snes::CartType::LoRom, rom, 0x2000);
        machine.ram.fill(0);
        let ram = &mut machine.ram;
        ram[0x11] = if case & 1 == 0 { 0 } else { 15 };
        ram[0xfb3] = ((case / 2) % 2) as u8;
        ram[0x37d] = [0, 1, 0x80, 255][case % 4];
        let scroll_x = [0u16, 0x100, 0xffff, 0x80][case % 4];
        let scroll_y = [0u16, 0x80, 0x100, 0xffff][(case / 4) % 4];
        ram[0xe2..0xe4].copy_from_slice(&scroll_x.to_le_bytes());
        ram[0xe8..0xea].copy_from_slice(&scroll_y.to_le_bytes());
        for k in 0..10 {
            let x = scroll_x
                .wrapping_add([0u16, 239, 240, 255, 256, 0xffff, 0x7fff, 0x8000][(case + k) % 8]);
            let y = scroll_y
                .wrapping_add([0u16, 231, 239, 240, 248, 255, 256, 0xffff][(case / 2 + k) % 8]);
            ram[0xc04 + k] = x as u8;
            ram[0xc18 + k] = (x >> 8) as u8;
            ram[0xbfa + k] = y as u8;
            ram[0xc0e + k] = (y >> 8) as u8;
            ram[0xc7c + k] = ((case + k) % 4) as u8;
            ram[0xc68 + k] = [0, 1, 128, 255][(case / 4 + k) % 4];
            ram[0xc90 + k] = [0, 1, 16, 255][(case / 8 + k) % 4];
        }
        for (region, limit) in [0x171u16, 0x201, 0x31, 0xc1, 0x141, 0x1d1]
            .into_iter()
            .enumerate()
        {
            // The allocator's isolated tests also cover u16 wrap. Drawing
            // through those invalid extended pointers would hit unmapped I/O;
            // these composed fixtures keep the OAM regions in live RAM.
            let base = [
                0,
                limit - 17,
                limit - 16,
                limit - 1,
                limit,
                limit + 1,
                0x1ff,
                0x200,
            ][case % 8];
            ram[0xfe0 + region * 2..0xfe2 + region * 2].copy_from_slice(&base.to_le_bytes());
            ram[0xfec + region * 2..0xfee + region * 2]
                .copy_from_slice(&(case as u16).wrapping_add(0xfff8).to_le_bytes());
        }
        ram[0x800..0xa40].fill(0x47);
        let mut cpu = NativeNmiContext {
            a: 0xab00 | u16::from(kind),
            x: slot as u16,
            y: 0x51,
            d: 0,
            db: 8,
            bank: 8,
            pc: 0x833c,
            sp: 0x1fd,
            p: 0x30 | ((case as u8) & 0xc3),
        };
        let mut work = if composed {
            ram[0x1b] = 1;
            for k in 0..=slot {
                ram[0xc4a + k] = 0x20;
            }
            ram[0xdd0] = if case & 1 == 0 { 8 } else { 9 };
            ram[0xe20] = 0x73;
            // Compose both the frozen active-body double return and the
            // unfrozen house-state return after drawing. Blanket clipping still
            // varies independently across the full coordinate matrix above.
            if case & 1 != 0 {
                ram[0x11] = if case & 2 == 0 { 15 } else { 0 };
                let x = scroll_x.wrapping_add(80);
                let y = scroll_y.wrapping_add(64);
                ram[0xd10] = x as u8;
                ram[0xd30] = (x >> 8) as u8;
                ram[0xd00] = y as u8;
                ram[0xd20] = (y >> 8) as u8;
                ram[0xdc0] = (case / 8 % 2) as u8;
                ram[0xde0] = (case / 16 % 4) as u8;
            }
            cpu.bank = 6;
            cpu.pc = 0x8328;
            cpu.sp = 0x1fc;
            cpu.db = 0;
            ram[0x1fd..0x200].copy_from_slice(&[0x37, 0x93, 2]);
            Fixture::Sprites(SpriteMain::default())
        } else {
            ram[0xc4a + slot] = kind;
            ram[0x1fe..0x200].copy_from_slice(&[0x37, 0x83]);
            Fixture::Ancilla(AncillaMain::default())
        };
        machine.cpu.a = cpu.a;
        machine.cpu.x = cpu.x;
        machine.cpu.y = cpu.y;
        machine.cpu.pc = cpu.pc;
        machine.cpu.k = cpu.bank;
        machine.cpu.db = cpu.db;
        machine.cpu.sp = cpu.sp;
        machine.cpu.e = false;
        machine.cpu.unpack_flags(cpu.p);
        let mut ram = machine.ram.clone();
        let timeline = CpuMasterTimeline::at_raster(
            field,
            snes::CpuRasterPosition::new(224, 1340),
            snes::CpuBusWorkload::default(),
            snes::CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        let mut source = snes::RomCpuTimingProbe::new(
            machine,
            timeline,
            snes::SourcePpuReadState::snes9x_reset(),
        )
        .unwrap();
        let mut clock = source.timeline().clone();
        let mut instructions = 0;
        for _ in 0..6000 {
            if work.finished() {
                break;
            }
            let label=format!("kind={kind:02x} slot={slot} case={case} field={field} composed={composed} pc={:02x}:{:04x}",cpu.bank,cpu.pc);
            super::super::native_work::compare_instruction(
                &label,
                &mut cpu,
                &mut clock,
                &mut source,
                &mut ram,
                rom,
                |cpu, clock, bus| work.step(cpu, clock, bus),
            );
            instructions += 1;
        }
        assert!(work.finished());
        assert_eq!(ram, source.snes().ram);
        match work {
            Fixture::Ancilla(w) if kind == 0x20 => {
                assert!(w.complete());
                assert_eq!(
                    (cpu.bank, cpu.pc, cpu.sp, cpu.x),
                    (8, 0x8338, 0x1ff, slot as u16)
                );
            }
            Fixture::Ancilla(w) => {
                assert!(!w.complete());
                assert_eq!(w.child(), Some(HANDLERS[usize::from(kind - 1)]));
                assert_eq!((cpu.bank, cpu.sp), (8, 0x1fd));
            }
            Fixture::Sprites(w) => {
                assert!(w.complete());
                assert_eq!(w.child(), None);
                assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x9338, 0x1ff));
            }
        }
        instructions
    }
    fn rom() -> Vec<u8> {
        std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap()
    }
    #[test]
    fn ancilla_dispatch_matches_source_all_targets_slots_and_allocation_paths() {
        let rom = rom();
        for a in (0x08837f..0x088405)
            .chain(0x08c003..0x08c013)
            .chain(0x08f66d..0x08f671)
        {
            assert_eq!(
                table_byte(a),
                Some(rom[(a >> 16) as usize * 0x8000 + (a as usize & 0x7fff)])
            );
        }
        let mut instructions = 0;
        for kind in 1..=67 {
            for slot in 0..10 {
                for case in 0..4 {
                    instructions += run(&rom, kind, slot, case, [0, 1, 30002, 30003][case], false);
                }
            }
        }
        eprintln!("2680 ancilla dispatch fixtures, {instructions} instruction boundaries");
    }
    #[test]
    fn blanket_matches_source_clipping_regions_timers_and_sprite_parent_returns() {
        let rom = rom();
        let mut instructions = 0;
        for slot in 0..10 {
            for case in 0..64 {
                for field in [0, 1, 30002, 30003] {
                    instructions += run(&rom, 0x20, slot, case, field, false);
                }
                instructions += run(&rom, 0x20, slot, case, [0, 1, 30002, 30003][case % 4], true);
            }
        }
        eprintln!("2560 blanket and 640 composed SpriteMain fixtures, {instructions} instruction boundaries");
    }
}
