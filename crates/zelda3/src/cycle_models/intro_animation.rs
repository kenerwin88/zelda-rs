//! Native intro animation caller, scene loop and title-screen actor dispatch.
//! Calls push their real return addresses. Nested work keeps the same CPU and
//! bus; no semantic callee fabricates the caller's continuation.
use super::{
    intro_clear::Access,
    intro_sprite_init::{Initializer, IntroSpriteInit},
    intro_sprite_work::{AddObjectsToOam, MoveTriangle},
    native_work::Work,
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
    triforce_animation::TriforceAnimation,
};
use snes::CpuMasterTimeline;
#[derive(Clone, Debug, PartialEq, Eq)]
enum Child {
    Triforce(TriforceAnimation),
    Init(IntroSpriteInit),
    Draw(AddObjectsToOam),
    Move(MoveTriangle),
}
impl Child {
    fn complete(&self) -> bool {
        match self {
            Self::Triforce(w) => w.complete(),
            Self::Init(w) => w.complete(),
            Self::Draw(w) => w.complete(),
            Self::Move(w) => w.complete(),
        }
    }
    fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl FnMut(u64, Access) -> u16,
    ) -> Option<u64> {
        match self {
            Self::Triforce(w) => w.step(cpu, clock, bus),
            Self::Init(w) => w.step(cpu, clock, bus),
            Self::Draw(w) => w.step(cpu, clock, bus),
            Self::Move(w) => w.step(cpu, clock, bus),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct IntroAnimation {
    pc: u32,
    child: Option<Child>,
    complete: bool,
}
impl Default for IntroAnimation {
    fn default() -> Self {
        Self {
            pc: 0x0cc404,
            child: None,
            complete: false,
        }
    }
}
impl IntroAnimation {
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        0x0cc404
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
        assert_eq!((u32::from(cpu.bank) << 16) | u32::from(cpu.pc), self.pc);
        assert_eq!(cpu.d, 0);
        if let Some(child) = self.child.as_mut() {
            let cycles = child.step(cpu, clock, bus);
            if child.complete() {
                self.child = None;
            }
            self.pc = (u32::from(cpu.bank) << 16) | u32::from(cpu.pc);
            return cycles;
        }
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        let next = match self.pc {
            0x0cc404 | 0x0cc405 => {
                w.drain(6);
                w.push(
                    u16::from(if self.pc == 0x0cc404 {
                        w.cpu.db
                    } else {
                        w.cpu.bank
                    }),
                    1,
                );
                self.pc + 1
            }
            0x0cc406 | 0x0cc410 => {
                w.drain(12);
                w.cpu.db = w.pull(1) as u8;
                w.flags(u16::from(w.cpu.db), 1);
                self.pc + 1
            }
            0x0cc407 => {
                w.drain(16);
                let a = w.absolute(0x1e0a);
                let value = w.read(a, 1).wrapping_add(1) & 255;
                w.drain(6);
                w.flags(value, 1);
                w.write(a, value, 1);
                0x0cc40a
            }
            0x0cc40a => {
                call(&mut w, 0xc435);
                self.child = Some(Child::Triforce(Default::default()));
                0x0cc435
            }
            0x0cc40d => {
                call(&mut w, 0xc412);
                0x0cc412
            }
            0x0cc411 => {
                w.rtl();
                self.complete = true;
                (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc)
            }
            0x0cc412 | 0x0cc417 => {
                w.immediate(if self.pc == 0x0cc412 { 0 } else { 8 }, 1);
                self.pc + 2
            }
            0x0cc414 | 0x0cc419 => {
                store(&mut w, if self.pc == 0x0cc414 { 0x1e08 } else { 0x1e09 });
                self.pc + 3
            }
            0x0cc41c => {
                w.drain(8);
                w.cpu.x = 7;
                w.flags(7, 1);
                0x0cc41e
            }
            0x0cc41e => {
                call(&mut w, 0xc534);
                0x0cc534
            }
            0x0cc421 => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_sub(1) & 255;
                w.flags(w.cpu.x, 1);
                0x0cc422
            }
            0x0cc422 => {
                if w.branch(w.cpu.p & NEGATIVE == 0) {
                    0x0cc41e
                } else {
                    0x0cc424
                }
            }
            0x0cc424 | 0x0cc543 | 0x0cc84f | 0x0cc867 | 0x0cc935 | 0x0cc971 | 0x0cc72e
            | 0x0cc8e1 | 0x0cc607 | 0x0cc60e => {
                w.rts();
                0x0c0000 | u32::from(w.cpu.pc)
            }
            0x0cc534 | 0x0cc544 | 0x0cc55b => {
                indexed_load(
                    &mut w,
                    if self.pc == 0x0cc534 { 0x1e10 } else { 0x1e18 },
                    false,
                );
                self.pc + 3
            }
            0x0cc537 => {
                if w.branch(w.cpu.p & ZERO != 0) {
                    0x0cc543
                } else {
                    0x0cc539
                }
            }
            0x0cc539 | 0x0cc547 | 0x0cc55e | 0x0cc5ba => {
                w.drain(30);
                w.push(u16::from(w.cpu.bank), 1);
                w.push(w.cpu.pc + 3, 2);
                0x008781
            }
            // JumpTableLocal consumes its JSL frame and reads the actual target.
            0x008781 => {
                w.dp_store(3, w.cpu.y, 1);
                0x008783
            }
            0x008783 => {
                w.drain(12);
                w.cpu.y = w.pull(1);
                w.flags(w.cpu.y, 1);
                0x008784
            }
            0x008784 => {
                w.dp_store(0, w.cpu.y, 1);
                0x008786
            }
            0x008786 => {
                w.width(0x30, true);
                0x008788
            }
            0x008788 => {
                w.mask(255, 2);
                0x00878b
            }
            0x00878b => {
                w.shift_left(2);
                0x00878c
            }
            0x00878c => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                0x00878d
            }
            0x00878d => {
                w.drain(12);
                let value = w.pull(2);
                w.load(value, 2);
                0x00878e
            }
            0x00878e => {
                w.dp_store(1, w.cpu.a, 2);
                0x008790
            }
            0x008790 => {
                w.increment_y();
                0x008791
            }
            0x008791 => {
                let a = w.indirect_y_address(0);
                let value = w.read(a, 2);
                w.load(value, 2);
                0x008793
            }
            0x008793 => {
                w.dp_store(0, w.cpu.a, 2);
                0x008795
            }
            0x008795 => {
                w.width(0x30, false);
                0x008797
            }
            0x008797 => {
                w.cpu.y = w.dp_read(3, 1);
                w.flags(w.cpu.y, 1);
                0x008799
            }
            0x008799 => {
                w.drain(16);
                let pc = w.read(0, 2);
                let bank = w.read(2, 1);
                let target = (u32::from(bank) << 16) | u32::from(pc);
                self.child = match target {
                    0x0cc57e => Some(Child::Init(IntroSpriteInit::new(Initializer::Triangle))),
                    0x0cc850 => Some(Child::Init(IntroSpriteInit::new(Initializer::Copyright))),
                    0x0cc8e2 => Some(Child::Init(IntroSpriteInit::new(Initializer::Sparkle))),
                    _ => None,
                };
                target
            }
            // Active title triangle: drawing, movement, then acceleration/stop.
            0x0cc5b1 => {
                call(&mut w, 0xc70f);
                0x0cc70f
            }
            0x0cc5b4 => {
                call(&mut w, 0xc9f1);
                self.child = Some(Child::Move(Default::default()));
                0x0cc9f1
            }
            0x0cc5b7 => {
                load(&mut w, 0x1e00);
                0x0cc5ba
            }
            0x0cc5d6 => {
                load(&mut w, 0x1e0a);
                0x0cc5d9
            }
            0x0cc5d9 => {
                w.mask(31, 1);
                0x0cc5db
            }
            0x0cc5db => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    0x0cc5f1
                } else {
                    0x0cc5dd
                }
            }
            0x0cc5dd | 0x0cc5e7 => {
                indexed_load(
                    &mut w,
                    if self.pc == 0x0cc5dd { 0xc5ca } else { 0xc5cd },
                    false,
                );
                self.pc + 3
            }
            0x0cc5e0 | 0x0cc5ea => {
                w.clear_carry();
                self.pc + 1
            }
            0x0cc5e1 | 0x0cc5eb => {
                let rhs = indexed_read(
                    &mut w,
                    if self.pc == 0x0cc5e1 { 0x1e58 } else { 0x1e60 },
                    false,
                );
                w.adc_byte(rhs);
                self.pc + 3
            }
            0x0cc5e4 | 0x0cc5ee => {
                let a = w.cpu.a;
                indexed_store(&mut w, if self.pc == 0x0cc5e4 { 0x1e58 } else { 0x1e60 }, a);
                self.pc + 3
            }
            0x0cc5f1 | 0x0cc5fc => {
                indexed_load(
                    &mut w,
                    if self.pc == 0x0cc5f1 { 0xc5d0 } else { 0xc5d3 },
                    false,
                );
                self.pc + 3
            }
            0x0cc5f4 | 0x0cc5ff => {
                let rhs = indexed_read(
                    &mut w,
                    if self.pc == 0x0cc5f4 { 0x1e30 } else { 0x1e48 },
                    false,
                );
                let a = w.cpu.a & 255;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(a >= rhs);
                w.flags(a.wrapping_sub(rhs) & 255, 1);
                self.pc + 3
            }
            0x0cc5f7 | 0x0cc602 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    self.pc + 5
                } else {
                    self.pc + 2
                }
            }
            0x0cc5f9 | 0x0cc604 | 0x0cc608 | 0x0cc60b => {
                indexed_store(
                    &mut w,
                    if matches!(self.pc, 0x0cc5f9 | 0x0cc608) {
                        0x1e58
                    } else {
                        0x1e60
                    },
                    0,
                );
                self.pc + 3
            }
            // Shared drawing callers publish the count and data pointer before JSR.
            0x0cc70f | 0x0cc8d0 | 0x0cc956 => {
                w.immediate(
                    match self.pc {
                        0x0cc70f => 16,
                        0x0cc8d0 => 13,
                        _ => 1,
                    },
                    1,
                );
                self.pc + 2
            }
            0x0cc711 | 0x0cc8d2 | 0x0cc958 => {
                w.dp_store(6, w.cpu.a & 255, 1);
                self.pc + 2
            }
            0x0cc713 | 0x0cc8d4 | 0x0cc95a => {
                w.dp_store(7, 0, 1);
                self.pc + 2
            }
            0x0cc715 => {
                w.compare_x_immediate(2, 1);
                0x0cc717
            }
            0x0cc717 => {
                if w.branch(w.cpu.p & ZERO != 0) {
                    0x0cc723
                } else {
                    0x0cc719
                }
            }
            0x0cc719 | 0x0cc723 | 0x0cc71d | 0x0cc727 | 0x0cc8d6 | 0x0cc8da | 0x0cc968 => {
                w.immediate(
                    match self.pc {
                        0x0cc719 => 0x0f,
                        0x0cc723 => 0x8f,
                        0x0cc71d | 0x0cc727 => 0xc6,
                        0x0cc8d6 => 0x68,
                        0x0cc8da => 0xc8,
                        _ => 0xc9,
                    },
                    1,
                );
                self.pc + 2
            }
            0x0cc71b | 0x0cc725 | 0x0cc8d8 | 0x0cc966 => {
                w.dp_store(8, w.cpu.a & 255, 1);
                self.pc + 2
            }
            0x0cc71f | 0x0cc729 | 0x0cc8dc | 0x0cc96c => {
                w.dp_store(9, w.cpu.a & 255, 1);
                self.pc + 2
            }
            0x0cc721 => {
                w.branch(true);
                0x0cc72b
            }
            0x0cc72b | 0x0cc8de | 0x0cc96e => {
                call(&mut w, 0xc972);
                self.child = Some(Child::Draw(Default::default()));
                0x0cc972
            }
            0x0cc864 => {
                call(&mut w, 0xc8d0);
                0x0cc8d0
            }
            // Sparkle state and low-coordinate updates retain their live fields.
            0x0cc90d => {
                call(&mut w, 0xc956);
                0x0cc956
            }
            0x0cc910 | 0x0cc91e => {
                load(&mut w, 0x1e0a);
                self.pc + 3
            }
            0x0cc913 | 0x0cc914 | 0x0cc921..=0x0cc925 => {
                shift_right(&mut w);
                self.pc + 1
            }
            0x0cc915 | 0x0cc926 => {
                w.mask(if self.pc == 0x0cc915 { 7 } else { 3 }, 1);
                self.pc + 2
            }
            0x0cc917 | 0x0cc928 => {
                w.drain(6);
                w.cpu.y = w.cpu.a & 255;
                w.flags(w.cpu.y, 1);
                self.pc + 1
            }
            0x0cc918 | 0x0cc929 | 0x0cc92f => {
                indexed_load(
                    &mut w,
                    match self.pc {
                        0x0cc918 => 0xc905,
                        0x0cc929 => 0xc8fd,
                        _ => 0xc901,
                    },
                    true,
                );
                self.pc + 3
            }
            0x0cc91b | 0x0cc92c | 0x0cc932 => {
                let a = w.cpu.a;
                indexed_store(
                    &mut w,
                    match self.pc {
                        0x0cc91b => 0x1e20,
                        0x0cc92c => 0x1e30,
                        _ => 0x1e48,
                    },
                    a,
                );
                self.pc + 3
            }
            0x0cc95c => {
                indexed_load(&mut w, 0x1e20, false);
                0x0cc95f
            }
            0x0cc95f => {
                if w.branch(w.cpu.p & NEGATIVE != 0) {
                    0x0cc971
                } else {
                    0x0cc961
                }
            }
            0x0cc961..=0x0cc963 => {
                w.shift_left(1);
                self.pc + 1
            }
            0x0cc964 | 0x0cc96a => {
                w.drain(8);
                w.adc_byte(if self.pc == 0x0cc964 { 0x36 } else { 0 });
                self.pc + 2
            }
            _ => panic!("unowned intro animation continuation {:06x}", self.pc),
        };
        self.pc = next;
        w.cpu.bank = (next >> 16) as u8;
        w.cpu.pc = next as u16;
        Some(w.cycles)
    }
}
fn call<B: FnMut(u64, Access) -> u16>(w: &mut Work<'_, B>, target: u16) {
    w.drain(22);
    w.push(w.cpu.pc + 2, 2);
    w.cpu.pc = target;
}
fn load<B: FnMut(u64, Access) -> u16>(w: &mut Work<'_, B>, a: u16) {
    w.drain(16);
    let value = w.read(w.absolute(a), 1);
    w.load(value, 1);
}
fn store<B: FnMut(u64, Access) -> u16>(w: &mut Work<'_, B>, a: u16) {
    w.drain(16);
    w.write(w.absolute(a), w.cpu.a & 255, 1);
}
fn indexed_read<B: FnMut(u64, Access) -> u16>(w: &mut Work<'_, B>, a: u16, y: bool) -> u16 {
    w.drain(16);
    let index = if y { w.cpu.y } else { w.cpu.x };
    if (a & 255) + index > 255 {
        w.drain(6);
    }
    w.read(w.absolute(a.wrapping_add(index)), 1)
}
fn indexed_load<B: FnMut(u64, Access) -> u16>(w: &mut Work<'_, B>, a: u16, y: bool) {
    let value = indexed_read(w, a, y);
    w.load(value, 1);
}
fn indexed_store<B: FnMut(u64, Access) -> u16>(w: &mut Work<'_, B>, a: u16, value: u16) {
    w.drain(22);
    w.write(w.absolute(a.wrapping_add(w.cpu.x)), value & 255, 1);
}
fn shift_right<B: FnMut(u64, Access) -> u16>(w: &mut Work<'_, B>) {
    w.drain(6);
    let a = w.cpu.a & 255;
    w.cpu.p = (w.cpu.p & !CARRY) | (a & 1) as u8;
    w.load(a >> 1, 1);
}

pub(crate) fn table_byte(address: u32) -> Option<u8> {
    let (base, entries): (u32, &[u16]) = match address {
        0x0cc53d..=0x0cc542 => (0x0cc53d, &[0xc543, 0xc544, 0xc55b]),
        0x0cc54b..=0x0cc55a => (
            0x0cc54b,
            &[
                0xc57e, 0xc84f, 0xc850, 0xc8e2, 0xcbe8, 0xcbe8, 0xcbe8, 0xcd19,
            ],
        ),
        0x0cc562..=0x0cc571 => (
            0x0cc562,
            &[
                0xc5b1, 0xc84f, 0xc864, 0xc90d, 0xcc13, 0xcc13, 0xcc13, 0xcd3e,
            ],
        ),
        0x0cc5be..=0x0cc5c9 => (0x0cc5be, &[0xc5d6, 0xc5d6, 0xc5d6, 0xc5d6, 0xc5d6, 0xc608]),
        0x0cc5d0..=0x0cc5d5 => {
            return Some([75, 95, 117, 88, 48, 88][(address - 0x0cc5d0) as usize])
        }
        0x0cc905..=0x0cc90c => {
            return Some([0, 1, 2, 3, 2, 1, 0xff, 0xff][(address - 0x0cc905) as usize])
        }
        _ => return None,
    };
    let offset = (address - base) as usize;
    Some(entries[offset / 2].to_le_bytes()[offset % 2])
}

#[cfg(test)]
mod tests {
    use super::*;
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, RomCpuTimingProbe, Snes,
        SourcePpuReadState,
    };
    #[test]
    fn retained_intro_animation_parent_and_title_actors_match_source_instructions() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for a in 0x0cc000..0x0cd000 {
            if let Some(value) = table_byte(a) {
                assert_eq!(value, rom[0x60000 + (a as usize & 0x7fff)], "table {a:06x}");
            }
        }
        let mut instructions = 0;
        for stage in 0..6 {
            for seed in 0..32u8 {
                for active in [false, true] {
                    compare_case(&rom, stage, seed, active, &mut instructions);
                }
            }
        }
        eprintln!("384 parent animation cases, {instructions} independently checked instruction boundaries");
    }
    fn compare_case(rom: &[u8], stage: u8, seed: u8, active: bool, instructions: &mut usize) {
        let mut cpu = NativeNmiContext {
            a: 0xab00 | u16::from(seed),
            x: 0x45,
            y: 0x93,
            d: 0,
            db: seed,
            bank: 12,
            pc: 0xc404,
            sp: 0x1fc,
            p: 0x30 | (seed & 0xc3),
        };
        let mut machine = Snes::new();
        machine.cart.load(CartType::LoRom, rom, 0x2000);
        for (i, b) in machine.ram.iter_mut().enumerate() {
            *b = (i ^ (i >> 8) ^ usize::from(seed)) as u8;
        }
        machine.ram[0x1e00] = stage;
        machine.ram[0x1e01] = seed.wrapping_mul(8).wrapping_add(7);
        machine.ram[0x1e0a] = seed.wrapping_mul(8).wrapping_sub(1);
        machine.ram[0x1f00] = if seed & 1 == 0 { 0 } else { 1 };
        for slot in 0..8 {
            machine.ram[0x1e10 + slot] = if seed == 0 {
                0
            } else if active {
                2
            } else {
                1
            };
            machine.ram[0x1e18 + slot] = if slot < 3 { 0 } else { 1 + (slot % 3) as u8 };
            machine.ram[0x1e20 + slot] = [0, 1, 2, 3, 0xff][usize::from(seed) % 5];
        }
        machine.ram[0x1fd..0x200].copy_from_slice(&[0x33, 0x80, 0]);
        machine.cpu.a = cpu.a;
        machine.cpu.x = cpu.x;
        machine.cpu.y = cpu.y;
        machine.cpu.dp = cpu.d;
        machine.cpu.db = cpu.db;
        machine.cpu.k = cpu.bank;
        machine.cpu.pc = cpu.pc;
        machine.cpu.sp = cpu.sp;
        machine.cpu.e = false;
        machine.cpu.unpack_flags(cpu.p);
        let mut ram = machine.ram.clone();
        let (v, h) = match seed % 3 {
            0 => (100, 510),
            1 => (224, 1340),
            _ => (261, 1320),
        };
        let mut clock = CpuMasterTimeline::at_raster(
            0,
            CpuRasterPosition::new(v, h),
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        let mut source =
            RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                .unwrap();
        clock.begin_synchronous_timeline().unwrap();
        let mut work = IntroAnimation::default();
        let mut count = 0;
        while !work.complete() {
            let label = format!(
                "stage={stage} seed={seed} active={active} pc={:06x}",
                work.pc
            );
            super::super::native_work::compare_instruction(
                &label,
                &mut cpu,
                &mut clock,
                &mut source,
                &mut ram,
                rom,
                |cpu, clock, bus| {
                    work.step(cpu, clock, bus).unwrap();
                },
            );
            assert_eq!(ram, source.snes().ram, "{label}");
            work = work.clone();
            count += 1;
            *instructions += 1;
            assert!(count < 10000, "{label}");
        }
        assert_eq!((cpu.bank, cpu.pc, cpu.sp, cpu.db), (0, 0x8034, 0x1ff, seed));
        assert!(work
            .step(&mut cpu, &mut clock, |_, _| panic!(
                "completed parent touched bus"
            ))
            .is_none());
    }
}
