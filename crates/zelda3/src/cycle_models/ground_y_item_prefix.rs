//! Player caller guards and Y-item dispatch on one retained stack and clock.
//! The tail identifies the first child still owned by the semantic caller.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, ZERO},
    y_item_dispatch::{YItemChild, YItemDispatch},
    y_item_lamp::{LampChild, YItemLamp},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GroundYItemTail {
    SkipYItem,
    AfterYItem,
    YItemChild(YItemChild),
    LampMagic,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GroundYItemPrefix {
    pc: u16,
    dispatch: Option<YItemDispatch>,
    lamp: Option<YItemLamp>,
    tail: Option<GroundYItemTail>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, RomCpuTimingProbe, Snes,
        SourcePpuReadState,
    };
    #[test]
    fn ground_y_item_prefix_matches_source_and_lamp_activation_boundaries() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut instructions = 0;
        let mut saw_lamp_magic = false;
        let mut saw_cape = false;
        for item in 0..=20u8 {
            for variant in 0..16u8 {
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                let r = &mut machine.ram;
                r[0x303] = item;
                r[0x304] = item;
                r[0x308] = u8::from(variant == 1);
                r[0x376] = u8::from(variant == 2);
                r[0x377] = u8::from(variant == 3);
                r[0x5d] = if variant == 4 { 17 } else { 0 };
                r[0x3c] = u8::from(variant == 5);
                r[0x2e0] = u8::from(variant == 6);
                r[0x3fc] = u8::from(variant == 7);
                if variant == 8 {
                    r[0x304] = 8;
                    r[0xf34c] = 2;
                    r[0x3a] = 0x40;
                }
                if variant == 9 {
                    r[0x304] = 19;
                    r[0x55] = 1;
                }
                r[0x6c] = u8::from(variant == 10);
                r[0x46] = u8::from(variant == 11);
                r[0xf4] = if matches!(variant, 12 | 13) { 0x40 } else { 0 };
                r[0xf34a] = u8::from(variant == 13);
                r[0x301] = if variant == 14 { 0x40 } else { 0 };
                r[0x37a] = u8::from(variant == 15);
                let mut cpu = NativeNmiContext {
                    a: 0x240a,
                    x: 10,
                    y: 254,
                    d: 0,
                    db: 7,
                    bank: 7,
                    pc: 0x81b3,
                    sp: 0x1f4,
                    p: 0xf3,
                };
                machine.cpu.a = cpu.a;
                machine.cpu.x = cpu.x;
                machine.cpu.y = cpu.y;
                machine.cpu.db = cpu.db;
                machine.cpu.k = cpu.bank;
                machine.cpu.pc = cpu.pc;
                machine.cpu.sp = cpu.sp;
                machine.cpu.e = false;
                machine.cpu.unpack_flags(cpu.p);
                let mut ram = machine.ram.clone();
                let mut clock = CpuMasterTimeline::at_raster(
                    4586,
                    CpuRasterPosition::new(
                        if variant & 1 == 0 { 21 } else { 261 },
                        if variant & 1 == 0 { 8 } else { 1350 },
                    ),
                    CpuBusWorkload::default(),
                    CpuFieldTiming::non_interlace(variant & 1 != 0),
                );
                let mut source = RomCpuTimingProbe::new(
                    machine,
                    clock.clone(),
                    SourcePpuReadState::snes9x_reset(),
                )
                .unwrap();
                clock.begin_synchronous_timeline().unwrap();
                let mut work = GroundYItemPrefix::default();
                while work.tail().is_none() {
                    let label = format!("ground Y item={item} variant={variant} pc={:04x}", cpu.pc);
                    super::super::native_work::compare_instruction_with_devices(
                        &label,
                        &mut cpu,
                        &mut clock,
                        &mut source,
                        &mut ram,
                        &rom,
                        None,
                        |_, access| {
                            let super::super::intro_clear::Access::Read(address, 2) = access else {
                                return None;
                            };
                            super::super::y_item_dispatch::table_byte(address).map(|low| {
                                u16::from_le_bytes([
                                    low,
                                    super::super::y_item_dispatch::table_byte(address + 1).unwrap(),
                                ])
                            })
                        },
                        |cpu, clock, bus| {
                            work.step(cpu, clock, bus).unwrap();
                        },
                    );
                    work = work.clone();
                    instructions += 1;
                }
                assert_eq!(ram, source.snes().ram);
                match work.tail().unwrap() {
                    GroundYItemTail::AfterYItem => assert_eq!((cpu.pc, cpu.sp), (0x81c9, 0x1f4)),
                    GroundYItemTail::SkipYItem => assert_eq!((cpu.pc, cpu.sp), (0x81df, 0x1f4)),
                    GroundYItemTail::YItemChild(YItemChild::ForceUnequipCape) => {
                        saw_cape = true;
                        assert_eq!((cpu.pc, cpu.sp), (0xae47, 0x1f0));
                    }
                    GroundYItemTail::YItemChild(YItemChild::Item(_)) => assert_eq!(cpu.sp, 0x1f2),
                    GroundYItemTail::LampMagic => {
                        saw_lamp_magic = true;
                        assert_eq!((cpu.pc, cpu.sp, cpu.x), (0xb0ab, 0x1f0, 6));
                        assert_eq!(ram[0x3a] & 0x40, 0x40);
                    }
                }
            }
        }
        assert!(saw_lamp_magic && saw_cape);
        println!("ground Y-item prefix: 336 cases, {instructions} source instructions");
    }
}
impl Default for GroundYItemPrefix {
    fn default() -> Self {
        Self {
            pc: 0x81b3,
            dispatch: None,
            lamp: None,
            tail: None,
        }
    }
}
impl GroundYItemPrefix {
    pub(crate) fn tail(&self) -> Option<GroundYItemTail> {
        self.tail
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.tail.is_some() {
            return None;
        }
        if let Some(lamp) = &mut self.lamp {
            let cycles = lamp.step(cpu, clock, bus);
            if lamp.complete() {
                self.dispatch.as_mut().unwrap().return_from_child(cpu);
                assert_eq!((cpu.bank, cpu.pc), (7, 0x81c9));
                self.tail = Some(GroundYItemTail::AfterYItem);
            } else if let Some(child) = lamp.child() {
                assert_eq!(child, LampChild::MagicPayment);
                self.tail = Some(GroundYItemTail::LampMagic);
            }
            return cycles;
        }
        if let Some(dispatch) = &mut self.dispatch {
            let cycles = dispatch.step(cpu, clock, bus);
            if dispatch.complete() {
                assert_eq!((cpu.bank, cpu.pc), (7, 0x81c9));
                self.tail = Some(GroundYItemTail::AfterYItem);
            } else if let Some(child) = dispatch.child() {
                if child == YItemChild::Item(9) {
                    self.lamp = Some(YItemLamp::default());
                } else {
                    self.tail = Some(GroundYItemTail::YItemChild(child));
                }
            }
            return cycles;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d, cpu.p & 0x38),
            (7, self.pc, 0, 0x30)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0x81b3 | 0x81bb => {
                w.drain(16);
                let value = w.read(w.absolute(if self.pc == 0x81b3 { 0x308 } else { 0x377 }), 1);
                w.load(value, 1);
                self.pc + 3
            }
            0x81b6 => {
                w.drain(16);
                let value = w.read(w.absolute(0x376), 1);
                w.load((w.cpu.a & 255) | value, 1);
                0x81b9
            }
            0x81b9 | 0x81be | 0x81c4 => {
                let taken = if self.pc == 0x81c4 {
                    w.cpu.p & ZERO != 0
                } else {
                    w.cpu.p & ZERO == 0
                };
                if w.branch(taken) {
                    self.tail = Some(GroundYItemTail::SkipYItem);
                    0x81df
                } else {
                    self.pc + 2
                }
            }
            0x81c0 => {
                let value = w.dp_read(0x5d, 1);
                w.load(value, 1);
                0x81c2
            }
            0x81c2 => {
                w.compare_immediate(17, 1);
                0x81c4
            }
            0x81c6 => {
                w.item_call();
                self.dispatch = Some(YItemDispatch::default());
                0x9b0e
            }
            _ => unreachable!("ground Y-item caller {:04x}", self.pc),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}
