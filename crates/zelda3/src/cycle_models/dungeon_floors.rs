//! Two-layer floor construction with live line pointers and nested returns.
//! Every tile write retires separately; the second layer falls through into
//! the shared chunk routine under the caller's original return frame.
#[cfg(test)]
use super::intro_clear::Access;
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonFloors {
    entry: u16,
    pc: u16,
    depth: u8,
    complete: bool,
}
impl Default for DungeonFloors {
    fn default() -> Self {
        Self {
            entry: 0x89dc,
            pc: 0x89dc,
            depth: 0,
            complete: false,
        }
    }
}
impl DungeonFloors {
    /// Full room chunks entered by an object tail jump on the caller's frame.
    pub(crate) fn chunks() -> Self {
        Self {
            entry: 0x8a1f,
            pc: 0x8a1f,
            depth: 0,
            complete: false,
        }
    }
    /// Shared block writer entered by object drawing under its own JSR frame.
    pub(crate) fn blocks() -> Self {
        Self {
            entry: 0x8a44,
            pc: 0x8a44,
            depth: 0,
            complete: false,
        }
    }
    pub(crate) fn routine(&self) -> u32 {
        0x010000 | u32::from(self.entry)
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
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
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.db, cpu.d, cpu.p & 0x38),
            (1, self.pc, 0, 0, 0)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0x89dc | 0x8a03 => {
                w.drain(16);
                w.cpu.x = 30;
                w.flags(30, 2);
                self.pc + 3
            }
            0x89df | 0x89e5 | 0x8a06 => {
                w.drain(24);
                let base = match self.pc {
                    0x89df => 0x018719,
                    0x89e5 => 0x01871a,
                    _ => 0x0186f9,
                };
                let value = w.read(base + u32::from(w.cpu.x), 2);
                w.load(value, 2);
                self.pc + 4
            }
            0x89e3 | 0x89e9 | 0x8a0a => {
                w.operand_then_internal(1);
                let base = if self.pc == 0x89e3 { 0xbf } else { 0xc0 };
                w.write(u32::from(w.cpu.x.wrapping_add(base)), w.cpu.a, 2);
                self.pc + 2
            }
            0x89eb..=0x89ed | 0x8a0c..=0x8a0e => {
                w.decrement_x();
                self.pc + 1
            }
            0x89ee | 0x8a0f => {
                if w.branch(w.cpu.p & NEGATIVE == 0) {
                    if self.pc == 0x89ee {
                        0x89df
                    } else {
                        0x8a06
                    }
                } else {
                    self.pc + 2
                }
            }
            0x89f0 | 0x8a1f => {
                let v = w.dp_read(if self.pc == 0x89f0 { 0xba } else { 0x0c }, 2);
                w.cpu.y = v;
                w.flags(v, 2);
                self.pc + 2
            }
            0x89f2 | 0x8a38 | 0x8a3a | 0x8a34 | 0x8a79 | 0x8a83 => {
                let (a, inc) = match self.pc {
                    0x89f2 => (0xba, true),
                    0x8a38 | 0x8a3a => (0x0c, true),
                    0x8a34 => (0x0e, false),
                    0x8a79 => (4, false),
                    _ => (0x0a, false),
                };
                w.dp_increment(a, 2, inc);
                self.pc + 2
            }
            0x89f4 | 0x8a11 => {
                w.dp_store(0x0c, 0, 2);
                self.pc + 2
            }
            0x89f6 => {
                let a = w.indirect_y_address(0xb7);
                let v = w.read(a, 2);
                w.load(v, 2);
                0x89f8
            }
            0x89f8 => {
                w.drain(6);
                w.push(w.cpu.a, 2);
                0x89f9
            }
            0x8a13 => {
                w.drain(12);
                let v = w.pull(2);
                w.load(v, 2);
                0x8a14
            }
            0x89f9 | 0x8a14 => {
                w.mask(if self.pc == 0x89f9 { 0xf0 } else { 15 }, 2);
                self.pc + 3
            }
            0x89fc | 0x8a1b => {
                w.drain(16);
                w.write(if self.pc == 0x89fc { 0x0490 } else { 0x046a }, w.cpu.a, 2);
                self.pc + 3
            }
            0x89ff | 0x8a1e => {
                w.tax_word();
                self.pc + 1
            }
            0x8a00 | 0x8a2d => {
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                self.depth += 1;
                if self.pc == 0x8a00 {
                    0x8a1f
                } else {
                    0x8a44
                }
            }
            0x8a17..=0x8a1a => {
                w.shift_left(2);
                self.pc + 1
            }
            0x8a21 => {
                w.operand_then_internal(2);
                let v = w.read(0x009b02 + u32::from(w.cpu.y), 2);
                w.load(v, 2);
                0x8a24
            }
            0x8a24 | 0x8a33 | 0x8a78 | 0x8a82 => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0x8a25 | 0x8a2a | 0x8a46 => {
                w.immediate(if self.pc == 0x8a46 { 2 } else { 8 }, 2);
                self.pc + 3
            }
            0x8a28 | 0x8a44 | 0x8a49 => {
                w.dp_store(
                    match self.pc {
                        0x8a28 => 0x0e,
                        0x8a44 => 0x0a,
                        _ => 4,
                    },
                    w.cpu.a,
                    2,
                );
                self.pc + 2
            }
            0x8a30 | 0x8a75 => {
                w.drain(16);
                w.adc(if self.pc == 0x8a30 { 0x01c0 } else { 0x0100 });
                self.pc + 3
            }
            0x8a36 | 0x8a41 | 0x8a7b | 0x8a85 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    match self.pc {
                        0x8a36 => 0x8a2a,
                        0x8a41 => 0x8a1f,
                        0x8a7b => 0x8a4b,
                        _ => 0x8a46,
                    }
                } else {
                    self.pc + 2
                }
            }
            0x8a3c => {
                let v = w.dp_read(0x0c, 2);
                w.load(v, 2);
                0x8a3e
            }
            0x8a3e => {
                w.compare_immediate(8, 2);
                0x8a41
            }
            0x8a43 | 0x8a88 => {
                w.rts();
                if self.depth == 0 {
                    self.complete = true;
                } else {
                    self.depth -= 1;
                }
                w.cpu.pc
            }
            0x8a4b | 0x8a50 | 0x8a55 | 0x8a5a | 0x8a5f | 0x8a64 | 0x8a69 | 0x8a6e => {
                w.operand_then_internal(2);
                let index = (self.pc - 0x8a4b) / 5;
                let v = w.read(0x009b52 + u32::from(w.cpu.x) + u32::from(index * 2), 2);
                w.load(v, 2);
                self.pc + 3
            }
            0x8a4e | 0x8a53 | 0x8a58 | 0x8a5d | 0x8a62 | 0x8a67 | 0x8a6c | 0x8a71 => {
                let index = (self.pc - 0x8a4e) / 5;
                let a = w.indirect_y_address(0xbf + index * 3);
                w.write(a, w.cpu.a, 2);
                self.pc + 2
            }
            0x8a73 | 0x8a7d => {
                w.drain(6);
                w.load(w.cpu.y, 2);
                self.pc + 1
            }
            0x8a74 | 0x8a87 => {
                w.clear_carry();
                self.pc + 1
            }
            0x8a7e => {
                w.drain(6);
                w.cpu.p |= CARRY;
                0x8a7f
            }
            0x8a7f => {
                w.drain(16);
                w.sbc(0x01f8, 2);
                0x8a82
            }
            _ => unreachable!("floor continuation {:04x}", self.pc),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, RomCpuTimingProbe, Snes,
        SourcePpuReadState,
    };

    fn native_data() -> std::collections::BTreeMap<u32, u8> {
        let mut state = crate::ZeldaState::new();
        state
            .set_assets(
                &std::fs::read(
                    std::env::var_os("ZELDA3_ASSETS").expect("floor tests require native assets"),
                )
                .unwrap(),
            )
            .unwrap();
        state.native_dungeon_floor_data().unwrap()
    }
    fn rom() -> Vec<u8> {
        std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap()
    }
    #[test]
    fn dungeon_floor_native_tables_match_source() {
        let data = native_data();
        let rom = rom();
        for (&a, &v) in &data {
            assert_eq!(
                v,
                rom[(a as usize >> 16) * 0x8000 + (a as usize & 0x7fff)],
                "floor table {a:06x}"
            );
        }
        assert_eq!(data.len(), 330);
        eprintln!("330 native floor tile/pointer/quadrant bytes match source");
    }
    #[test]
    fn dungeon_floors_match_source_all_patterns_and_continuations() {
        let data = native_data();
        let rom = rom();
        let mut visited = std::collections::BTreeSet::new();
        let mut total = 0;
        let mut cases = 0;
        for floor in 0..256u16 {
            for variant in 0..2 {
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                for (i, b) in machine.ram.iter_mut().enumerate() {
                    *b = (i.wrapping_mul(13) ^ (i >> 8)) as u8;
                }
                // The floor word is mutable room input, supplied independently
                // to both machines. The second case crosses a 4 KiB read edge.
                let offset = if variant == 0 { 0 } else { 0x0fffu16 };
                machine.ram[0xb7..0xba].copy_from_slice(&[0x00, 0x80, 0x7f]);
                machine.ram[0xba..0xbc].copy_from_slice(&offset.to_le_bytes());
                let address = 0x18000 + usize::from(offset);
                machine.ram[address..address + 2]
                    .copy_from_slice(&(floor | ((!floor & 255) << 8)).to_le_bytes());
                machine.ram[0x1fc..0x1fe].copy_from_slice(&[0x11, 0x88]);
                let mut cpu = NativeNmiContext {
                    a: 0x1283,
                    x: 0x8142,
                    y: 0x1999,
                    d: 0,
                    db: 0,
                    bank: 1,
                    pc: 0x89dc,
                    sp: 0x1fb,
                    p: if variant == 0 { 0x41 } else { 0x84 },
                };
                machine.cpu.a = cpu.a;
                machine.cpu.x = cpu.x;
                machine.cpu.y = cpu.y;
                machine.cpu.k = cpu.bank;
                machine.cpu.pc = cpu.pc;
                machine.cpu.sp = cpu.sp;
                machine.cpu.e = false;
                machine.cpu.unpack_flags(cpu.p);
                let mut ram = machine.ram.clone();
                let mut state = crate::ZeldaState::new();
                state.ram.copy_from_slice(&ram);
                state.sync_native_game_state_from_ram();
                let (v, h) = if variant == 0 {
                    (224, 1338)
                } else {
                    (261, 1350)
                };
                let mut clock = CpuMasterTimeline::at_raster(
                    0,
                    CpuRasterPosition::new(v, h),
                    CpuBusWorkload::default(),
                    CpuFieldTiming::non_interlace(v == 261),
                );
                let mut source = RomCpuTimingProbe::new(
                    machine,
                    clock.clone(),
                    SourcePpuReadState::snes9x_reset(),
                )
                .unwrap();
                clock.begin_synchronous_timeline().unwrap();
                let mut work = DungeonFloors::default();
                assert_eq!(work.routine(), 0x0189dc);
                let mut count = 0;
                let mut writes = std::collections::BTreeSet::new();
                let mut write_count = 0;
                while !work.complete() {
                    visited.insert(cpu.pc);
                    let label = format!("floor {floor:02x} variant{variant} pc{:04x}", cpu.pc);
                    let mut tile_write = None;
                    let mut published = false;
                    let mut wrote = false;
                    super::super::native_work::compare_instruction_with_devices(
                        &label,
                        &mut cpu,
                        &mut clock,
                        &mut source,
                        &mut ram,
                        &rom,
                        None,
                        |_, access| match access {
                            Access::Read(a, width) if a & 0x8000 != 0 && a < 0x7e0000 => {
                                let mut value = 0;
                                for b in 0..width {
                                    let a = a + u32::from(b);
                                    value |=
                                        u16::from(*data.get(&a).unwrap_or_else(|| {
                                            panic!("unbound floor data {a:06x}")
                                        })) << (b * 8);
                                }
                                Some(value)
                            }
                            Access::Write(a, value, width) => {
                                wrote = true;
                                if state.native_dungeon_floor_store(a, value, width) {
                                    if (0x7e2000..0x7e6000).contains(&a) {
                                        assert_eq!(width, 2);
                                        writes.insert(a);
                                        write_count += 1;
                                        tile_write = Some(a as usize - 0x7e0000);
                                    } else {
                                        published = true;
                                    }
                                } else {
                                    assert!(matches!(a, 0..=0xf | 0x100..=0x1ff),
                                        "{label}: unowned floor store {a:06x}");
                                    state.ram[a as usize..a as usize + usize::from(width)]
                                        .copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
                                }
                                None
                            }
                            _ => None,
                        },
                        |cpu, clock, bus| {
                            work.step(cpu, clock, bus).unwrap();
                        },
                    );
                    assert_eq!(ram, source.snes().ram, "{label}");
                    if wrote {
                        assert_eq!(state.ram, ram, "{label}: published native RAM");
                    }
                    if let Some(offset) = tile_write {
                        assert_eq!(state.game_state.dungeon.room_tilemaps.bg2_tile((offset - 0x2000) / 2),
                            crate::types::read_le_u16(&ram, offset), "{label}: floor tile owner");
                    }
                    if published || work.complete() {
                        let incoherent = state.game_state.dungeon.report_incoherent_with_ram(&ram);
                        assert!(incoherent.is_empty(), "{label}: {incoherent:?}");
                    }
                    work = work.clone();
                    count += 1;
                    assert!(count < 40000, "{label}");
                }
                assert_eq!((cpu.pc, cpu.sp, work.depth), (0x8812, 0x1fd, 0));
                assert_eq!(u16::from_le_bytes([ram[0xba], ram[0xbb]]), offset + 1);
                assert_eq!(
                    (write_count, writes.len()),
                    (8192, 8192),
                    "exactly one store per tile word"
                );
                total += count;
                cases += 1;
            }
        }
        assert_eq!(visited.len(), 84);
        eprintln!("{cases} floor fixtures matched {total} instructions; {} continuation positions; 8192 unique tile writes per fixture", visited.len());
        eprintln!("floor native publications: {} tile stores, pointer/selector checks after each store, full dungeon owner checks at each return", cases * 8192);
    }
}
