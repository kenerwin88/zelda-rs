//! Active file-menu owner, including save rendering, input and the live SRAM
//! copy on confirmation. Children share the CPU and real return stack.
use super::{
    file_select_draw::FileSelectDraw,
    intro_clear::Access,
    native_work::Work,
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FileSelectMain {
    pc: u16,
    child: Option<FileSelectDraw>,
    complete: bool,
}
impl Default for FileSelectMain {
    fn default() -> Self {
        Self {
            pc: 0xcebd,
            child: None,
            complete: false,
        }
    }
}
impl FileSelectMain {
    /// Erase-file setup joins the same menu cursor and publication owners.
    pub(crate) fn kill_file_setup() -> Self {
        Self { pc: 0xd49a, ..Default::default() }
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        self.child.as_ref().map_or(
            if matches!(self.pc, 0xd070..=0xd086 | 0xd49a | 0xd49c | 0xc52e..=0xc532) {
                0x0cd49a
            } else if self.pc >= 0xcfbb && self.pc <= 0xd052 {
                0x0ccfbb
            } else {
                0x0ccedc
            },
            |w| w.routine(),
        )
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl FnMut(u64, Access) -> u16,
    ) -> Option<u64> {
        if self.complete {
            return None;
        }
        if let Some(child) = self.child.as_mut() {
            let cycles = child.step(cpu, clock, bus);
            if child.complete() {
                self.child = None;
                assert_eq!((cpu.bank, cpu.pc), (12, self.pc));
            }
            return cycles;
        }
        assert_eq!((cpu.bank, cpu.pc, cpu.d), (12, self.pc, 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let m = if w.cpu.p & 0x20 != 0 { 1 } else { 2 };
        let x = if w.cpu.p & 0x10 != 0 { 1 } else { 2 };
        w.fetch();
        self.pc = match self.pc {
            0xd49a | 0xd073 => {
                w.immediate(if self.pc == 0xd49a { 8 } else { 15 }, 1);
                self.pc + 2
            }
            0xd49c => { w.drain(16); 0xd070 }
            0xd070 => { w.drain(22); w.push(0xd072, 2); 0xc52e }
            0xc52e | 0xd075 | 0xd084 => {
                w.dp_store(match self.pc { 0xc52e => 0x14, 0xd075 => 0x13, _ => 0xc8 }, w.cpu.a & 255, 1);
                self.pc + 2
            }
            0xc530 => { w.dp_increment(0x11, 1, true); 0xc532 }
            0xc532 => { w.rts(); w.cpu.pc }
            0xd077 => { w.drain(16); w.write(w.absolute(0x710), 0, 1); 0xd07a }
            0xd07a => { w.drain(8); w.cpu.x=254; w.flags(w.cpu.x,1); 0xd07c }
            0xd07c | 0xd07d => {
                w.drain(6);
                w.cpu.x=w.cpu.x.wrapping_add(1)&255;
                w.flags(w.cpu.x,1); self.pc+1
            }
            0xd07e => {
                w.drain(14);
                let value=w.read(u32::from(0xbfu16.wrapping_add(w.cpu.x)),1);
                w.load(value,1); 0xd080
            }
            0xd080 => { if w.branch(w.cpu.p & ZERO != 0) { 0xd07c } else { 0xd082 } }
            0xd082 => { w.drain(6); w.load(w.cpu.x,1); 0xd083 }
            0xd083 => {
                w.drain(6);
                w.cpu.p = (w.cpu.p & !CARRY) | (w.cpu.a & 1) as u8;
                w.load((w.cpu.a & 255) >> 1,1); 0xd084
            }
            0xd086 => { w.rtl(); self.complete=true; w.cpu.pc }
            0xcebd | 0xcebe | 0xcfd5 => {
                w.drain(6);
                w.push(
                    u16::from(if self.pc == 0xcebe {
                        w.cpu.bank
                    } else {
                        w.cpu.db
                    }),
                    1,
                );
                self.pc + 1
            }
            0xcebf | 0xcfd9 | 0xd00d | 0xd0a0 => {
                w.drain(12);
                let v = w.pull(1);
                w.cpu.db = v as u8;
                w.flags(v, 1);
                self.pc + 1
            }
            0xcec0 => {
                w.drain(30);
                w.push(u16::from(w.cpu.bank), 1);
                w.push(0xcec3, 2);
                0xcedc
            }
            0xcec4 => {
                w.drain(16);
                0xd09c
            }
            0xcedc | 0xcf59 | 0xcf6a | 0xcf72 | 0xcf89 | 0xcf33 | 0xcfa3 => {
                if self.pc == 0xcf59 {
                    w.dp_increment(0xc8, 1, true);
                } else {
                    let v = w.dp_read(
                        match self.pc {
                            0xcf33 => 0xf6,
                            0xcfa3 => 0xbf,
                            _ => 0xc8,
                        },
                        m,
                    );
                    w.load(v, m);
                }
                self.pc + 2
            }
            0xcf5b => {
                let v = w.dp_read(0xc8, 1);
                w.load(v, 1);
                self.pc + 2
            }
            0xcede | 0xcf5d | 0xcf6c | 0xcef5 => {
                w.compare_immediate(
                    if self.pc == 0xcef5 {
                        0x55aa
                    } else if self.pc == 0xcf5d {
                        5
                    } else {
                        3
                    },
                    m,
                );
                self.pc + 1 + u16::from(m)
            }
            0xcee0 | 0xcef8 | 0xcf1f | 0xcf3b | 0xcf3f | 0xcf43 | 0xcf4c | 0xcf5f | 0xcf6e
            | 0xcf70 | 0xcf78 | 0xcfa9 => {
                let (t, target) = match self.pc {
                    0xcee0 => (w.cpu.p & CARRY != 0, 0xcee5),
                    0xcef8 => (w.cpu.p & ZERO == 0, 0xcf18),
                    0xcf1f => (w.cpu.p & CARRY == 0, 0xceea),
                    0xcf3b => (w.cpu.p & ZERO != 0, 0xcfba),
                    0xcf3f => (w.cpu.p & ZERO != 0, 0xcf65),
                    0xcf43 => (w.cpu.p & ZERO != 0, 0xcf54),
                    0xcf4c => (w.cpu.p & NEGATIVE == 0, 0xcf63),
                    0xcf5f => (w.cpu.p & ZERO == 0, 0xcf63),
                    0xcf6e => (w.cpu.p & ZERO != 0, 0xcf9d),
                    0xcf70 => (w.cpu.p & CARRY != 0, 0xcfa1),
                    0xcf78 => (w.cpu.p & ZERO != 0, 0xcf97),
                    _ => (w.cpu.p & ZERO == 0, 0xcfb2),
                };
                if w.branch(t) {
                    target
                } else {
                    self.pc + 2
                }
            }
            0xcee2 | 0xcf47 | 0xcf56 | 0xcf67 | 0xcf7c | 0xcfad | 0xd040 => {
                let a = match self.pc {
                    0xcee2 => 0xb9d,
                    0xcf47 | 0xcf56 => 0x12f,
                    0xcf67 | 0xcfad => 0x12e,
                    0xcf7c => 0x12c,
                    _ => 0x204,
                };
                w.drain(16);
                w.write(w.absolute(a), w.cpu.a & if m == 1 { 255 } else { 65535 }, m);
                self.pc + 3
            }
            0xcee5 | 0xcf21 | 0xcf81 | 0xcf92 | 0xcfda | 0xd03c => {
                w.width(
                    if matches!(self.pc, 0xcf81 | 0xcf92) {
                        0x20
                    } else {
                        0x30
                    },
                    matches!(self.pc, 0xcee5 | 0xcf81 | 0xcfda),
                );
                self.pc + 2
            }
            0xcee7 | 0xcf31 | 0xcf99 | 0xcf9d | 0xcfa1 | 0xcfbb | 0xcfdc => {
                w.drain(8 * u32::from(x));
                let v = match self.pc {
                    0xcf31 | 0xcf9d => 2,
                    0xcf99 => 4,
                    0xcfa1 => 3,
                    0xcfbb => 15,
                    _ => 0,
                };
                if matches!(self.pc, 0xcee7 | 0xcfbb) {
                    w.cpu.x = v;
                } else {
                    w.cpu.y = v;
                }
                w.flags(v, x);
                self.pc + 1 + u16::from(x)
            }
            0xceea => {
                w.dp_store(0, w.cpu.x, 2);
                self.pc + 2
            }
            0xceec | 0xcef1 | 0xcf83 => {
                w.drain(24);
                let v = w.read(
                    if self.pc == 0xcef1 { 0x7003e5 } else { 0x848c } + u32::from(w.cpu.x),
                    2,
                );
                w.load(v, 2);
                self.pc + 4
            }
            0xcef0 | 0xcf75 => {
                w.drain(6);
                w.cpu.x = w.cpu.a & if x == 1 { 255 } else { 65535 };
                w.flags(w.cpu.x, x);
                self.pc + 1
            }
            0xcefa | 0xcf0d => {
                w.drain(6);
                w.push(w.cpu.x, 2);
                self.pc + 1
            }
            0xcf02 | 0xcf14 => {
                w.drain(12);
                let v = w.pull(2);
                w.cpu.x = v;
                w.flags(v, 2);
                self.pc + 1
            }
            0xcefb | 0xcf18 | 0xcf23 | 0xcfdf => {
                let v = w.dp_read(if self.pc == 0xcf23 { 0xc8 } else { 0 }, x);
                w.cpu.x = v;
                w.flags(v, x);
                self.pc + 2
            }
            0xcefd | 0xcf03 | 0xcf08 | 0xcf25 | 0xcf45 | 0xcf4e | 0xcf54 | 0xcf65 | 0xcf7a
            | 0xcfab | 0xcfbd | 0xcfc7 | 0xcfd6 | 0xd00e | 0xd019 | 0xd024 | 0xd02a | 0xd030
            | 0xd036 | 0xd03e | 0xd043 | 0xd09c => {
                let v = match self.pc {
                    0xcefd | 0xd09c => 1,
                    0xcf03 => 0xd698,
                    0xcf08 => 0xd699,
                    0xcf25 => 0x1c,
                    0xcf45 | 0xcf54 => 0x20,
                    0xcf4e => 4,
                    0xcf65 => 0x2c,
                    0xcf7a => 0xf1,
                    0xcfab => 0x3c,
                    0xcfd6 => 0x7e,
                    0xd00e => 7,
                    0xd024 => 0x6040,
                    0xd02a => 0x4841,
                    0xd030 => 0x7f,
                    0xd036 => 65535,
                    0xd03e => 0x80,
                    0xd043 => 5,
                    _ => 0,
                };
                w.immediate(v, m);
                self.pc + 1 + u16::from(m)
            }
            0xcf00 => {
                w.drain(14);
                w.write(u32::from(0xbf + w.cpu.x), w.cpu.a, 2);
                self.pc + 2
            }
            0xcf76 => {
                w.drain(14);
                let v = w.read(u32::from(0xbf + w.cpu.x), 1);
                w.load(v, 1);
                self.pc + 2
            }
            0xcf06 | 0xcf0b | 0xcf27 | 0xcf50 | 0xcf87 | 0xcfb4 | 0xd045 | 0xd09e => {
                let a = match self.pc {
                    0xcf06 => 4,
                    0xcf0b => 2,
                    0xcf50 => 0xc8,
                    0xcfb4 | 0xd045 => 0x10,
                    0xd09e => 0x14,
                    _ => 0,
                };
                w.dp_store(
                    a,
                    if self.pc == 0xcfb4 {
                        w.cpu.y
                    } else {
                        w.cpu.a & if m == 1 { 255 } else { 65535 }
                    },
                    m,
                );
                self.pc + 2
            }
            0xcf0e | 0xcf11 | 0xcf15 | 0xcf2e => {
                w.drain(22);
                w.push(self.pc + 2, 2);
                let entry = match self.pc {
                    0xcf0e => 0xd6af,
                    0xcf11 => 0xd7db,
                    0xcf15 => 0xd63c,
                    _ => 0xd7a5,
                };
                w.cpu.pc = entry;
                self.child = Some(FileSelectDraw::new(entry));
                self.pc + 3
            }
            0xcf1a | 0xcf1b | 0xd004 | 0xd005 => {
                w.increment_x();
                self.pc + 1
            }
            0xcf1c => {
                w.compare_x_immediate(6, 2);
                self.pc + 3
            }
            0xcf29 => {
                w.drain(16);
                if 0x78 + w.cpu.x > 255 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(0xcd78) + u32::from(w.cpu.x), 1);
                w.load(v, 1);
                self.pc + 3
            }
            0xcf2c => {
                w.dp_store(1, w.cpu.a & 255, 1);
                self.pc + 2
            }
            0xcf35 | 0xcf39 | 0xcf3d | 0xcf41 => {
                w.mask(
                    match self.pc {
                        0xcf35 => 0xc0,
                        0xcf39 => 0xfc,
                        0xcf3d => 0x2c,
                        _ => 8,
                    },
                    1,
                );
                self.pc + 2
            }
            0xcf37 | 0xcfa5 | 0xcfa7 => {
                let v = w.dp_read(
                    match self.pc {
                        0xcf37 => 0xf4,
                        0xcfa5 => 0xc1,
                        _ => 0xc3,
                    },
                    1,
                );
                w.load((w.cpu.a & 255) | v, 1);
                self.pc + 2
            }
            0xcf4a => {
                w.dp_increment(0xc8, 1, false);
                self.pc + 2
            }
            0xcf52 | 0xcf63 | 0xcf9b | 0xcf9f | 0xcfb0 => {
                w.branch(true);
                match self.pc {
                    0xcf52 => 0xcf63,
                    0xcf9b => 0xcfb4,
                    0xcf9f => 0xcfa3,
                    _ => 0xcfba,
                }
            }
            0xcf61 | 0xcf7f | 0xcf97 | 0xcfb2 | 0xcfb6 | 0xcfb8 | 0xd047 => {
                w.dp_store(
                    match self.pc {
                        0xcf61 | 0xcfb2 => 0xc8,
                        0xcf7f | 0xcf97 => 0xc9,
                        0xcfb6 | 0xd047 => 0x11,
                        _ => 0xb0,
                    },
                    0,
                    1,
                );
                self.pc + 2
            }
            0xcf74 | 0xcf8b => {
                w.shift_left(m);
                self.pc + 1
            }
            0xcf8c | 0xcf8d => {
                w.increment_a(2);
                self.pc + 1
            }
            0xcf8e => {
                w.drain(24);
                w.write(0x701ffe, w.cpu.a, 2);
                self.pc + 4
            }
            0xcf94 => {
                w.drain(22);
                0xcfbb
            }
            0xcfba | 0xd052 => {
                w.rtl();
                w.cpu.pc
            }
            0xd0a1 => {
                w.rtl();
                self.complete = true;
                w.cpu.pc
            }
            0xcfbf | 0xcfc3 | 0xcfc9 | 0xcfcd | 0xcfd1 => {
                w.drain(24);
                let a = match self.pc {
                    0xcfbf => 0x1ac0,
                    0xcfc3 => 0x1ae0,
                    0xcfc9 => 0x1ab0,
                    0xcfcd => 0x1ad0,
                    _ => 0x1af0,
                };
                w.write(a + u32::from(w.cpu.x), w.cpu.a & 255, 1);
                self.pc + 4
            }
            0xcfd8 => {
                w.drain(6);
                w.push(w.cpu.a & 255, 1);
                self.pc + 1
            }
            0xcfe1 | 0xcfe8 | 0xcfef | 0xcff6 | 0xcffd => {
                w.drain(24);
                let a = 0x700000 + u32::from((self.pc - 0xcfe1) / 7) * 0x100 + u32::from(w.cpu.x);
                let v = w.read(a, 2);
                w.load(v, 2);
                self.pc + 4
            }
            0xcfe5 | 0xcfec | 0xcff3 | 0xcffa | 0xd001 => {
                w.drain(22);
                let a = 0xf000 + ((self.pc - 0xcfe5) / 7) * 0x100;
                w.write(w.absolute(a) + u32::from(w.cpu.y), w.cpu.a, 2);
                self.pc + 3
            }
            0xd006 | 0xd007 => {
                w.increment_y();
                self.pc + 1
            }
            0xd008 => {
                w.drain(16);
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.y >= 256);
                w.flags(w.cpu.y.wrapping_sub(256), 2);
                self.pc + 3
            }
            0xd00b => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    0xcfe1
                } else {
                    self.pc + 2
                }
            }
            0xd011 | 0xd015 | 0xd01c | 0xd020 => {
                w.drain(24);
                w.write(
                    match self.pc {
                        0xd011 => 0x7ec00d,
                        0xd015 => 0x7ec013,
                        0xd01c => 0x7ec00f,
                        _ => 0x7ec015,
                    },
                    w.cpu.a,
                    2,
                );
                self.pc + 4
            }
            0xd027 | 0xd02d | 0xd033 | 0xd039 => {
                w.drain(16);
                w.write(
                    w.absolute(match self.pc {
                        0xd027 => 0x219,
                        0xd02d => 0x21d,
                        0xd033 => 0x21f,
                        _ => 0x221,
                    }),
                    w.cpu.a,
                    2,
                );
                self.pc + 3
            }
            0xd049 | 0xd04c | 0xd04f => {
                w.drain(16);
                w.write(
                    w.absolute(match self.pc {
                        0xd049 => 0x10e,
                        0xd04c => 0x710,
                        _ => 0xab2,
                    }),
                    0,
                    1,
                );
                self.pc + 3
            }
            _ => unreachable!("file menu continuation {:04x}", self.pc),
        };
        if self.child.is_none() {
            w.cpu.pc = self.pc;
        }
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
    #[test]
    fn active_menu_matches_each_instruction_for_all_slots_and_input_paths() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for present in 0..8 {
            for cursor in 0..5 {
                for input in [0, 0x08, 0x04, 0x10, 0x80] {
                    let mut machine = Snes::new();
                    machine.cart.load(CartType::LoRom, &rom, 0x2000);
                    for (i, b) in machine.ram.iter_mut().enumerate() {
                        *b = (i ^ (i >> 8)) as u8;
                    }
                    machine.cart.ram.fill(0x27);
                    machine.ram[0xbf..0xc5].fill(0);
                    machine.ram[0xc8] = cursor;
                    machine.ram[0xf4] = input;
                    machine.ram[0xf6] = 0;
                    machine.ram[0x10] = 1;
                    machine.ram[0x11] = 5;
                    machine.ram[0x1fb..0x1fe].copy_from_slice(&[0x59, 0x80, 0]);
                    for slot in 0..3 {
                        let s = slot * 0x500;
                        let signature = if present & (1 << slot) != 0 {
                            0x55aau16
                        } else {
                            0
                        };
                        machine.cart.ram[s + 0x3e5..s + 0x3e7]
                            .copy_from_slice(&signature.to_le_bytes());
                        machine.cart.ram[s + 0x359] = ((present + slot) % 5) as u8;
                        machine.cart.ram[s + 0x35a] = ((present + slot) % 4) as u8;
                        machine.cart.ram[s + 0x36c] = (3 + slot * 8) as u8 * 8;
                        machine.cart.ram[s + 0x405..s + 0x407]
                            .copy_from_slice(&[0u16, 42, 999][slot].to_le_bytes());
                        for i in 0..6 {
                            machine.cart.ram[s + 0x3d9 + i * 2..s + 0x3db + i * 2]
                                .copy_from_slice(&((i * 17 + slot) as u16).to_le_bytes());
                        }
                    }
                    let mut native = NativeNmiContext {
                        a: 0xab83,
                        x: 0x71,
                        y: 0x83,
                        d: 0,
                        db: 0,
                        bank: 12,
                        pc: 0xcebd,
                        sp: 0x1fa,
                        p: 0x31,
                    };
                    machine.cpu.a = native.a;
                    machine.cpu.x = native.x;
                    machine.cpu.y = native.y;
                    machine.cpu.k = 12;
                    machine.cpu.pc = native.pc;
                    machine.cpu.sp = native.sp;
                    machine.cpu.e = false;
                    machine.cpu.unpack_flags(native.p);
                    let mut ram = machine.ram.clone();
                    let mut sram = machine.cart.ram.clone();
                    let mut clock = CpuMasterTimeline::at_raster(
                        0,
                        CpuRasterPosition::new(if present & 1 == 0 { 224 } else { 261 }, 1350),
                        CpuBusWorkload::default(),
                        CpuFieldTiming::non_interlace(present & 1 != 0),
                    );
                    let mut source = RomCpuTimingProbe::new(
                        machine,
                        clock.clone(),
                        SourcePpuReadState::snes9x_reset(),
                    )
                    .unwrap();
                    clock.begin_synchronous_timeline().unwrap();
                    let mut work = FileSelectMain::default();
                    let mut count = 0;
                    while !work.complete() {
                        let label = format!(
                            "file menu slots{present} cursor{cursor} input{input:02x} pc{:04x}",
                            native.pc
                        );
                        super::super::native_work::compare_instruction_with_devices(
                            &label,
                            &mut native,
                            &mut clock,
                            &mut source,
                            &mut ram,
                            &rom,
                            None,
                            |_, a| match a {
                                Access::Read(a, width) if (0x700000..0x702000).contains(&a) => {
                                    let i = (a - 0x700000) as usize;
                                    Some(if width == 1 {
                                        u16::from(sram[i])
                                    } else {
                                        u16::from_le_bytes([sram[i], sram[i + 1]])
                                    })
                                }
                                Access::Write(a, value, width)
                                    if (0x700000..0x702000).contains(&a) =>
                                {
                                    let i = (a - 0x700000) as usize;
                                    sram[i..i + usize::from(width)].copy_from_slice(
                                        &value.to_le_bytes()[..usize::from(width)],
                                    );
                                    Some(0)
                                }
                                _ => None,
                            },
                            |cpu, clock, bus| {
                                work.step(cpu, clock, bus).unwrap();
                            },
                        );
                        assert_eq!(ram, source.snes().ram, "{label}");
                        assert_eq!(sram, source.snes().cart.ram, "{label}");
                        work = work.clone();
                        count += 1;
                        assert!(count < 10000);
                    }
                    assert_eq!((native.bank, native.pc, native.sp), (0, 0x805a, 0x1fd));
                }
            }
        }
    }
}
