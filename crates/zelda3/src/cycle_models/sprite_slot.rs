//! Active slot caller, timers/OAM, initialization properties, and Uncle/Priest
//! preparation. Every nested return consumes the live native stack. Bodies not
//! owned here remain explicit children, including damage and priest helpers.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, ZERO},
    oam_allocate::OamAllocate,
    sprite_drawing::{DrawingChild, SpriteDrawing},
    sprite_slot_data,
};
use snes::CpuMasterTimeline;
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    let entry = match address {
        0x05db95..=0x05db9a => Some((&[0xde2c, 0xdce6, 0xdb9b][..], 0x05db95)),
        0x05de3a..=0x05de3d => Some((&[0xde3e, 0xdf19][..], 0x05de3a)),
        0x0684f4..=0x06850b => Some((&sprite_slot_data::STATES[..], 0x0684f4)),
        0x06865b..=0x068840 => Some((&sprite_slot_data::PREP[..], 0x06865b)),
        0x069283..=0x069468 => Some((&sprite_slot_data::ACTIVE[..], 0x069283)),
        _ => None,
    };
    if let Some((words, base)) = entry {
        let i = (address - base) as usize;
        return Some(words[i / 2].to_le_bytes()[i & 1]);
    }
    crate::ZeldaState::native_uncle_prep_table_byte(address)
        .or_else(|| super::sprite_drawing::table_byte(address))
        .or_else(|| crate::ZeldaState::native_sprite_property_table_byte(address))
        .or_else(|| super::ancilla_main::table_byte(address))
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SlotChild {
    AgahnimBalls,
    Recoil,
    State(u32),
    Prep(u32),
    Active(u32),
    PriestMantle,
    FaceLink,
    Drawing(DrawingChild),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Dispatch {
    State,
    Prep,
    Active,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Call {
    returned: u32,
    far: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SpriteSlot {
    pc: u32,
    calls: Vec<Call>,
    allocator: Option<OamAllocate>,
    dispatch: Dispatch,
    drawing: Option<SpriteDrawing>,
    child: Option<SlotChild>,
    complete: bool,
}
impl Default for SpriteSlot {
    fn default() -> Self {
        Self::from_entry(0x0684e7)
    }
}
impl SpriteSlot {
    fn from_entry(pc: u32) -> Self {
        Self {
            pc,
            calls: Vec::new(),
            allocator: None,
            dispatch: Dispatch::State,
            drawing: None,
            child: None,
            complete: false,
        }
    }
    pub(crate) fn child(&self) -> Option<SlotChild> {
        self.child
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn return_from_child(&mut self, cpu: &NativeNmiContext) {
        let child = self.child.take().expect("slot child must be pending");
        if let Some(drawing) = self.drawing.as_mut() {
            assert_eq!(
                child,
                SlotChild::Drawing(drawing.child().expect("drawing child"))
            );
            drawing.return_from_child(cpu);
            return;
        }
        self.accept_return(cpu, None);
    }
    fn accept_return(&mut self, cpu: &NativeNmiContext, far: Option<bool>) {
        let returned = (u32::from(cpu.bank) << 16) | u32::from(cpu.pc);
        if let Some(call) = self.calls.pop() {
            assert_eq!(call.returned, returned);
            if let Some(far) = far {
                assert_eq!(call.far, far);
            }
        } else {
            self.complete = true;
        }
        self.pc = returned;
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
        assert!(self.child.is_none(), "sprite slot child still owns the CPU");
        if let Some(drawing) = self.drawing.as_mut() {
            let cycles = drawing.step(cpu, clock, bus);
            if drawing.complete() {
                self.drawing = None;
                self.accept_return(cpu, Some(true));
            } else if let Some(child) = drawing.child() {
                self.child = Some(SlotChild::Drawing(child));
            }
            return cycles;
        }
        if let Some(allocator) = self.allocator.as_mut() {
            let cycles = allocator.step(cpu, clock, bus);
            if allocator.complete() {
                assert_eq!((u32::from(cpu.bank) << 16) | u32::from(cpu.pc), self.pc);
                self.allocator = None;
            }
            return cycles;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d, cpu.p & 8),
            ((self.pc >> 16) as u8, self.pc as u16, 0, 0)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let pc = self.pc;
        w.fetch();
        self.pc = match pc {
            0x0684e7 | 0x068453 | 0x06927f => {
                w.drain(6);
                let m = if pc == 0x06927f { 2 } else { 1 };
                w.push(w.cpu.a & if m == 1 { 255 } else { 65535 }, m);
                pc + 1
            }
            0x0684eb | 0x068479 => {
                w.drain(12);
                let v = w.pull(1);
                w.load(v, 1);
                pc + 1
            }
            0x0684ec | 0x06844f | 0x068457 | 0x06847a | 0x068484 | 0x05da3f | 0x05daf7
            | 0x05da4d | 0x05da7f | 0x05da9f => {
                let v = match pc {
                    0x0684ec => 9,
                    0x06844f => 31,
                    0x068457 => 0x7a,
                    0x06847a => 24,
                    0x068484 => 251,
                    0x05da3f => 18,
                    0x05daf7 => 4,
                    0x05da4d => 3,
                    0x05da7f => 2,
                    _ => 1,
                };
                w.compare_immediate(v, 1);
                pc + 2
            }
            0x068400 | 0x068405 | 0x06840b | 0x068411 | 0x06841c | 0x068424 | 0x06842c
            | 0x068434 | 0x06843c | 0x068446 | 0x06844d | 0x068451 | 0x068459 | 0x06845e
            | 0x068467 | 0x068469 | 0x06847c | 0x068486 | 0x068494 | 0x06849f | 0x0684a8
            | 0x0684ee | 0x0db852 | 0x05da41 | 0x05daf9 | 0x05db01 | 0x05db13 | 0x05da4f
            | 0x05da61 | 0x05da81 | 0x05da8d | 0x05daa1 | 0x05dabe => {
                let z = w.cpu.p & ZERO != 0;
                let c = w.cpu.p & CARRY != 0;
                let (take, target) = match pc {
                    0x068400 => (z, 0x068413),
                    0x068405 => (z, 0x06840d),
                    0x06840b | 0x068411 => (true, 0x068417),
                    0x06841c => (z, 0x068421),
                    0x068424 => (z, 0x068429),
                    0x06842c => (z, 0x068431),
                    0x068434 => (z, 0x068439),
                    0x06843c => (z, 0x068441),
                    0x068446 => (z, 0x068496),
                    0x06844d => (!c, 0x068481),
                    0x068451 => (!z, 0x06847a),
                    0x068459 | 0x06845e => (!z, 0x068479),
                    0x068467 => (z, 0x06846b),
                    0x068469 => (c, 0x068479),
                    0x06847c => (!z, 0x068481),
                    0x068486 => (c, 0x068491),
                    0x068494 => (true, 0x06849c),
                    0x06849f => (z, 0x0684a4),
                    0x0684a8 => (z, 0x0684ad),
                    0x0684ee => (z, 0x06850c),
                    0x0db852 => (z, 0x0db857),
                    0x05da41 => (z, 0x05da46),
                    0x05daf9 => (!z, 0x05db0d),
                    0x05db01 | 0x05db13 => (!z, 0x05db1f),
                    0x05da4f => (!c, 0x05da5b),
                    0x05da61 => (z, 0x05da67),
                    0x05da81 => (!c, 0x05da8f),
                    0x05da8d | 0x05dabe => (true, 0x05dac2),
                    _ => (!z, 0x05dac0),
                };
                if w.branch(take) {
                    target
                } else {
                    pc + 2
                }
            }
            0x06841e | 0x06850c | 0x05da43 => {
                w.drain(16);
                match pc {
                    0x06841e => 0x0684a4,
                    0x06850c => 0x069271,
                    _ => 0x05daf7,
                }
            }
            0x0684e8 | 0x0683f2 | 0x068475 | 0x06847e | 0x06864d | 0x0db818 | 0x06bfe5
            | 0x06bfe0 | 0x05da2c | 0x05da46 | 0x05da8f | 0x05db89 | 0x05de30 | 0x05de2c => {
                let (target, far, child) = match pc {
                    0x0684e8 => (0x0683f2, false, None),
                    0x0683f2 => (0x0684c1, false, None),
                    0x068475 => (0x05fa8e, true, Some(SlotChild::AgahnimBalls)),
                    0x06847e => (0x06eec8, false, Some(SlotChild::Recoil)),
                    0x06864d => (0x0db818, true, None),
                    0x0db818 => (0x0db871, true, None),
                    0x06bfe5 => (0x05da29, true, None),
                    0x06bfe0 => (0x05db86, true, None),
                    0x05db89 => (0x05db8e, false, None),
                    0x05de30 => (0x05f94e, false, None),
                    0x05de2c => (0x0dd391, true, None),
                    0x05da2c => (0x05da3d, false, None),
                    0x05da46 => (0x05db27, false, Some(SlotChild::PriestMantle)),
                    _ => (0x05f93f, false, Some(SlotChild::FaceLink)),
                };
                let width = if far { 3 } else { 2 };
                w.operand_then_internal(width);
                if far {
                    w.push(u16::from(w.cpu.bank), 1);
                }
                w.push(pc as u16 + u16::from(width), 2);
                self.calls.push(Call {
                    returned: pc + 1 + u32::from(width),
                    far,
                });
                self.child = child;
                if pc == 0x05de2c {
                    self.drawing = Some(SpriteDrawing::new(target));
                }
                target
            }
            0x068407 | 0x06840d | 0x068413 => {
                let target = match pc {
                    0x068407 => 0xba94,
                    0x06840d => 0xba8c,
                    _ => 0xba80,
                };
                w.operand_then_internal(3);
                w.push(6, 1);
                w.push(pc as u16 + 3, 2);
                w.cpu.bank = 13;
                w.cpu.pc = target;
                self.allocator = Some(OamAllocate::new(target));
                pc + 4
            }
            0x0683f5 | 0x068402 | 0x068421 | 0x068429 | 0x068431 | 0x068439 | 0x068441
            | 0x068448 | 0x068454 | 0x068460 | 0x068464 | 0x068481 | 0x068488 | 0x06849c
            | 0x0684aa | 0x0684ad | 0x0684b2 | 0x0684c1 | 0x0684c7 | 0x0684cd | 0x0684d3
            | 0x068654 | 0x069271 | 0x0db820 | 0x0db823 | 0x0db829 | 0x0db82f | 0x0db835
            | 0x0db83b | 0x0db841 | 0x0db847 | 0x0db860 | 0x0db863 | 0x05db03 | 0x05da6c
            | 0x05dac7 | 0x05dacb | 0x05dad1 | 0x05dad4 | 0x05dada | 0x05dade | 0x05dae4
            | 0x05dae7 => {
                let (a, idx, ldy) = match pc {
                    0x0683f5 | 0x05da6c => (0xe40, w.cpu.x, false),
                    0x068402 | 0x0684aa => (0xf20, w.cpu.x, true),
                    0x068421 => (0xdf0, w.cpu.x, false),
                    0x068429 => (0xe00, w.cpu.x, false),
                    0x068431 => (0xe10, w.cpu.x, false),
                    0x068439 => (0xee0, w.cpu.x, false),
                    0x068441 | 0x068488 => (0xef0, w.cpu.x, false),
                    0x068448 => (0xdd0, w.cpu.x, true),
                    0x068454 | 0x068654 | 0x069271 => (0xe20, w.cpu.x, false),
                    0x068460 => (0xe50, w.cpu.x, false),
                    0x068464 | 0x068481 => (0xce2, w.cpu.x, false),
                    0x06849c => (0xf10, w.cpu.x, false),
                    0x0684ad => (0xb89, w.cpu.x, false),
                    0x0684b2 => (0x84b9, w.cpu.y, false),
                    0x0684c1 | 0x05db03 | 0x05dacb => (0xd10, w.cpu.x, false),
                    0x0684c7 | 0x05dad4 => (0xd30, w.cpu.x, false),
                    0x0684cd | 0x05dade => (0xd00, w.cpu.x, false),
                    0x0684d3 | 0x05dae7 => (0xd20, w.cpu.x, false),
                    0x0db820 | 0x0db860 => (0xe20, w.cpu.x, true),
                    0x0db823 => (0xb080, w.cpu.y, false),
                    0x0db829 => (0xb173, w.cpu.y, false),
                    0x0db82f => (0xb44c, w.cpu.y, false),
                    0x0db835 => (0xb632, w.cpu.y, false),
                    0x0db83b => (0xb725, w.cpu.y, false),
                    0x0db841 => (0xb266, w.cpu.y, false),
                    0x0db847 => (0xb53f, w.cpu.y, false),
                    0x0db863 => (0xb359, w.cpu.y, false),
                    0x05dac7 => (0xda31, w.cpu.y, false),
                    0x05dad1 => (0xda32, w.cpu.y, false),
                    0x05dada => (0xda37, w.cpu.y, false),
                    _ => (0xda38, w.cpu.y, false),
                };
                w.drain(16);
                if (a & 255) + idx > 255 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(a) + u32::from(idx), 1);
                if ldy {
                    w.cpu.y = v;
                    w.flags(v, 1);
                } else if pc == 0x068464 {
                    w.sbc(v, 1);
                } else if pc == 0x0684b2 {
                    w.load((w.cpu.a | v) & 255, 1);
                } else if matches!(pc, 0x05dacb | 0x05dad4 | 0x05dade | 0x05dae7) {
                    w.adc_byte(v);
                } else {
                    w.load(v, 1);
                }
                pc + 3
            }
            0x0683fd | 0x068419 | 0x06845b | 0x0db84d | 0x0db854 => {
                let a = match pc {
                    0x0683fd => 0xfb3,
                    0x068419 => 0xfc1,
                    0x06845b => 0xfff,
                    0x0db84d => 0x40a,
                    _ => 0x48e,
                };
                w.drain(16);
                let v = w.read(w.absolute(a), 1);
                if pc == 0x0683fd {
                    w.cpu.y = v;
                    w.flags(v, 1);
                } else {
                    w.load(
                        if pc == 0x068419 {
                            (w.cpu.a | v) & 255
                        } else {
                            v
                        },
                        1,
                    );
                }
                pc + 3
            }
            0x068417 | 0x0684a4 | 0x0db850 | 0x05da3d => {
                let a = match pc {
                    0x068417 => 0x11,
                    0x0684a4 => 0xee,
                    0x0db850 => 0x1b,
                    _ => 0xa0,
                };
                let v = w.dp_read(a, 1);
                if matches!(pc, 0x0684a4 | 0x0db850) {
                    w.cpu.y = v;
                    w.flags(v, 1);
                } else {
                    w.load(v, 1);
                }
                pc + 2
            }
            0x0683f8 | 0x068444 | 0x06848c | 0x0684b0 | 0x0db869 | 0x05daff | 0x05db11
            | 0x05da5f | 0x05da6f => {
                w.mask(
                    match pc {
                        0x0683f8 => 31,
                        0x068444 => 127,
                        0x06848c => 14,
                        0x0684b0 => 0xcf,
                        0x0db869 => 15,
                        0x05daff => 16,
                        0x05db11 => 1,
                        0x05da5f => 2,
                        _ => 0xf0,
                    },
                    1,
                );
                pc + 2
            }
            0x0683fa => {
                w.increment_a(1);
                pc + 1
            }
            0x0683fb | 0x0683fc | 0x06848b | 0x05dac5 => {
                w.shift_left(1);
                pc + 1
            }
            0x06844b | 0x0684a6 => {
                w.drain(8);
                let rhs = if pc == 0x06844b { 9 } else { 3 };
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.y >= rhs);
                w.flags(w.cpu.y.wrapping_sub(rhs) & 255, 1);
                pc + 2
            }
            0x068463 => {
                w.drain(6);
                w.cpu.p |= CARRY;
                pc + 1
            }
            0x06846b | 0x068470 | 0x0db8da | 0x05db15 | 0x05da67 | 0x05da76 | 0x05da83
            | 0x05da8b | 0x05dab7 | 0x05dabc | 0x05dac0 | 0x05daf0 => {
                w.immediate(
                    match pc {
                        0x06846b => 64,
                        0x068470 | 0x05da67 | 0x05dabc => 1,
                        0x05db15 | 0x05da76 => 3,
                        0x05da83 => 4,
                        0x05dab7 => 170,
                        0x05dac0 => 2,
                        _ => 0,
                    },
                    1,
                );
                pc + 2
            }
            0x06846d | 0x068472 | 0x0684c4 | 0x0684ca | 0x0684d0 | 0x0684d6 => {
                let a = match pc {
                    0x06846d => 0x1cf0,
                    0x068472 => 0x1cf1,
                    _ => 0xfd8 + (pc as u16 - 0x84c4) / 6,
                };
                w.drain(16);
                w.write(w.absolute(a), w.cpu.a & 255, 1);
                pc + 3
            }
            0x06848e | 0x068496 | 0x068499 | 0x0684b5 | 0x0db826 | 0x0db82c | 0x0db832
            | 0x0db838 | 0x0db83e | 0x0db844 | 0x0db84a | 0x0db857 | 0x0db866 | 0x0db86b
            | 0x05db09 | 0x05db17 | 0x05db1b | 0x05db1f | 0x05da63 | 0x05da69 | 0x05da73
            | 0x05da78 | 0x05da85 | 0x05da88 | 0x05da95 | 0x05da98 | 0x05dab9 | 0x05dac2
            | 0x05dace | 0x05dad7 | 0x05dae1 | 0x05daea => {
                let a = match pc {
                    0x06848e | 0x068499 | 0x0684b5 => 0xb89,
                    0x068496 => 0xef0,
                    0x0db826 | 0x05da73 => 0xe40,
                    0x0db82c => 0xe50,
                    0x0db832 | 0x05da78 => 0xf60,
                    0x0db838 => 0xbe0,
                    0x0db83e => 0xcaa,
                    0x0db844 => 0xcd2,
                    0x0db84a => 0xb6b,
                    0x0db857 => 0xc9a,
                    0x0db866 => 0xe60,
                    0x0db86b => 0xf50,
                    0x05db09 | 0x05dace => 0xd10,
                    0x05db17 | 0x05da85 | 0x05da98 => 0xde0,
                    0x05db1b | 0x05dac2 => 0xe80,
                    0x05db1f | 0x05da63 => 0xdd0,
                    0x05da69 => 0xe90,
                    0x05da88 => 0xdc0,
                    0x05da95 => 0xeb0,
                    0x05dab9 => 0xdf0,
                    0x05dad7 => 0xd30,
                    0x05dae1 => 0xd00,
                    _ => 0xd20,
                };
                let zero = matches!(pc, 0x068496 | 0x068499 | 0x05db1f | 0x05da63 | 0x05da88);
                w.operand_then_internal(2);
                w.write(
                    w.absolute(a) + u32::from(w.cpu.x),
                    if zero { 0 } else { w.cpu.a & 255 },
                    1,
                );
                pc + 3
            }
            0x068426 | 0x06842e | 0x068436 | 0x06843e | 0x068491 | 0x0684a1 | 0x068651
            | 0x05daed => {
                let a = match pc {
                    0x068426 => 0xdf0,
                    0x06842e => 0xe00,
                    0x068436 => 0xe10,
                    0x06843e => 0xee0,
                    0x068491 => 0xef0,
                    0x0684a1 => 0xf10,
                    0x068651 => 0xdd0,
                    _ => 0xba0,
                };
                w.operand_then_internal(2);
                let a = w.absolute(a) + u32::from(w.cpu.x);
                let old = w.read(a, 1);
                let v = if matches!(pc, 0x068651 | 0x05daed) {
                    old.wrapping_add(1)
                } else {
                    old.wrapping_sub(1)
                } & 255;
                w.drain(6);
                w.flags(v, 1);
                w.write(a, v, 1);
                pc + 3
            }
            0x0db871..=0x0db8d7 => {
                let a = [
                    0xf00, 0xe90, 0xd50, 0xd40, 0xf80, 0xd70, 0xd60, 0xf90, 0xd80, 0xdc0, 0xde0,
                    0xdf0, 0xe00, 0xe10, 0xf10, 0xeb0, 0xec0, 0xed0, 0xef0, 0xe70, 0xf70, 0xe50,
                    0xea0, 0xf40, 0xf30, 0xd90, 0xda0, 0xdb0, 0xbb0, 0xe80, 0xba0, 0xb89, 0xf50,
                    0xb58, 0xce2,
                ][((pc - 0x0db871) / 3) as usize];
                w.operand_then_internal(2);
                w.write(w.absolute(a) + u32::from(w.cpu.x), 0, 1);
                pc + 3
            }
            0x0db8dc | 0x0db8e0 | 0x0db8e4 | 0x0db8e8 | 0x0db8ec => {
                let a = if pc == 0x0db8ec {
                    0x7ff9c2
                } else {
                    0x7ffa1c + (pc - 0x0db8dc) * 4
                };
                w.drain(24);
                w.write(a + u32::from(w.cpu.x), w.cpu.a & 255, 1);
                pc + 4
            }
            0x0db81c | 0x0db85c | 0x0db81d | 0x0db85d | 0x05da29 | 0x0db81e | 0x0db85e
            | 0x05da2a => {
                w.drain(6);
                let v = match pc {
                    0x0db81c | 0x0db85c => w.cpu.y,
                    0x0db81d | 0x0db85d | 0x05da29 => u16::from(w.cpu.db),
                    _ => u16::from(w.cpu.bank),
                };
                w.push(v, 1);
                pc + 1
            }
            0x0db81f | 0x0db85a | 0x0db85f | 0x0db86e | 0x05da2b | 0x05da2f => {
                w.drain(12);
                w.cpu.db = w.pull(1) as u8;
                w.flags(u16::from(w.cpu.db), 1);
                pc + 1
            }
            0x0db85b | 0x0db86f => {
                w.drain(12);
                w.cpu.y = w.pull(1);
                w.flags(w.cpu.y, 1);
                pc + 1
            }
            // The shared JSL dispatcher removes its own far frame and jumps
            // through the live DP pointer, leaving the slot's near return.
            0x0684f0 | 0x068657 | 0x05db91 | 0x05de36 => {
                w.operand_then_internal(3);
                w.push(u16::from(w.cpu.bank), 1);
                w.push(pc as u16 + 3, 2);
                self.dispatch = match pc {
                    0x0684f0 => Dispatch::State,
                    0x068657 => Dispatch::Prep,
                    _ => Dispatch::Active,
                };
                0x008781
            }
            0x008781 | 0x008784 => {
                w.dp_store(if pc == 0x008781 { 3 } else { 0 }, w.cpu.y, 1);
                pc + 2
            }
            0x008783 => {
                w.drain(12);
                w.cpu.y = w.pull(1);
                w.flags(w.cpu.y, 1);
                pc + 1
            }
            0x008786 | 0x008795 | 0x069274 | 0x069280 => {
                w.width(0x30, matches!(pc, 0x008786 | 0x069274));
                pc + 2
            }
            0x008788 | 0x069276 => {
                w.mask(255, 2);
                pc + 3
            }
            0x00878b | 0x069279 => {
                w.shift_left(2);
                pc + 1
            }
            0x00878c | 0x06927a => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                pc + 1
            }
            0x00878d => {
                w.drain(12);
                let v = w.pull(2);
                w.load(v, 2);
                pc + 1
            }
            0x00878e | 0x008793 => {
                w.dp_store(if pc == 0x00878e { 1 } else { 0 }, w.cpu.a, 2);
                pc + 2
            }
            0x008790 => {
                w.increment_y();
                pc + 1
            }
            0x008791 => {
                let a = w.indirect_y_address(0);
                let v = w.read(a, 2);
                w.load(v, 2);
                pc + 2
            }
            0x008797 => {
                w.cpu.y = w.dp_read(3, 1);
                w.flags(w.cpu.y, 1);
                pc + 2
            }
            0x008799 => {
                w.drain(16);
                let low = w.read(0, 2);
                let bank = w.read(2, 1);
                let target = (u32::from(bank) << 16) | u32::from(low);
                if !matches!(target, 0x06864d | 0x06bfe5 | 0x05de2c) {
                    self.child = Some(match self.dispatch {
                        Dispatch::Prep => SlotChild::Prep(target),
                        Dispatch::State => SlotChild::State(target),
                        Dispatch::Active => SlotChild::Active(target),
                    });
                }
                target
            }
            0x06927b => {
                w.drain(16);
                w.drain(6);
                let v = w.read(w.absolute(0x9283) + u32::from(w.cpu.y), 2);
                w.load(v, 2);
                pc + 3
            }
            0x06927e => {
                w.drain(6);
                w.load(w.cpu.a.wrapping_sub(1), 2);
                pc + 1
            }
            0x069282 => {
                w.rts();
                let target = (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc);
                if target != 0x06bfe0 {
                    self.child = Some(SlotChild::Active(target));
                }
                target
            }
            0x05dafb | 0x05db0d | 0x05da49 | 0x05da51 | 0x05da5b | 0x05da7b | 0x05da9b
            | 0x05daa3 | 0x05daad => {
                let a = match pc {
                    0x05da49 => 0x7ef3c5,
                    0x05da7b => 0x7ef359,
                    0x05da9b => 0x7ef3cc,
                    0x05daad => 0x7ef29b,
                    _ => 0x7ef3c6,
                };
                w.drain(24);
                let v = w.read(a, 1);
                w.load(v, 1);
                pc + 4
            }
            0x05da57 | 0x05daa9 | 0x05dab3 | 0x05daf2 => {
                w.drain(24);
                w.write(
                    match pc {
                        0x05dab3 => 0x7ef29b,
                        0x05daf2 => 0x7ffe01,
                        _ => 0x7ef3c6,
                    },
                    w.cpu.a & 255,
                    1,
                );
                pc + 4
            }
            0x05da55 | 0x05da71 | 0x05daa7 | 0x05dab1 | 0x05da93 => {
                w.drain(8);
                let v = if pc == 0x05da93 {
                    (w.cpu.a ^ 3) & 255
                } else {
                    (w.cpu.a
                        | match pc {
                            0x05daa7 => 4,
                            0x05dab1 => 32,
                            _ => 2,
                        })
                        & 255
                };
                w.load(v, 1);
                pc + 2
            }
            0x05db06 | 0x05daca | 0x05dadd => {
                w.clear_carry();
                pc + 1
            }
            0x05db07 => {
                w.drain(8);
                w.adc_byte(8);
                pc + 2
            }
            0x05db1a => {
                w.drain(6);
                let v = w.cpu.a & 255;
                w.cpu.p = (w.cpu.p & !CARRY) | (v & 1) as u8;
                w.load(v >> 1, 1);
                pc + 1
            }
            0x05da92 => {
                w.drain(6);
                w.load(w.cpu.y, 1);
                pc + 1
            }
            0x05dac6 => {
                w.drain(6);
                w.cpu.y = w.cpu.a & 255;
                w.flags(w.cpu.y, 1);
                pc + 1
            }
            0x0684d9 | 0x0684b8 | 0x0db8f0 | 0x0db870 | 0x06bfe9 | 0x06bfe4 | 0x05da30
            | 0x05da66 | 0x05daf6 | 0x05db0c | 0x05db1e | 0x05db22 => {
                let far = matches!(pc, 0x0db8f0 | 0x0db870 | 0x05da30);
                if far {
                    w.rtl();
                } else {
                    w.rts();
                }
                self.accept_return(w.cpu, Some(far));
                self.pc
            }
            0x05db86 | 0x05db87 => {
                w.drain(6);
                let bank = if pc == 0x05db86 { w.cpu.db } else { w.cpu.bank };
                w.push(u16::from(bank), 1);
                pc + 1
            }
            0x05db88 | 0x05db8c => {
                w.drain(12);
                w.cpu.db = w.pull(1) as u8;
                w.flags(u16::from(w.cpu.db), 1);
                pc + 1
            }
            0x05db8d => {
                w.rtl();
                self.accept_return(w.cpu, Some(true));
                self.pc
            }
            0x05db8e | 0x05de33 | 0x05f94e | 0x05f95e | 0x05f963 => {
                w.drain(16);
                let a = match pc {
                    0x05db8e => 0xe90,
                    0x05de33 => 0xe80,
                    0x05f94e => 0xdd0,
                    0x05f95e => 0xcaa,
                    _ => 0xf00,
                };
                let v = w.read(w.absolute(a) + u32::from(w.cpu.x), 1);
                w.load(v, 1);
                pc + 3
            }
            0x05f951 => {
                w.compare_immediate(9, 1);
                pc + 2
            }
            0x05f955 => {
                w.drain(16);
                let v = w.read(w.absolute(0xfc1), 1);
                w.load(v, 1);
                pc + 3
            }
            0x05f95a => {
                let v = w.dp_read(0x11, 1);
                w.load(v, 1);
                pc + 2
            }
            0x05f953 | 0x05f958 | 0x05f95c | 0x05f961 | 0x05f966 => {
                let (take, target) = match pc {
                    0x05f961 => (w.cpu.p & 0x80 != 0, 0x05f96a),
                    0x05f966 => (w.cpu.p & ZERO != 0, 0x05f96a),
                    _ => (w.cpu.p & ZERO == 0, 0x05f968),
                };
                if w.branch(take) { target } else { pc + 2 }
            }
            0x05f968 | 0x05f969 => {
                w.drain(12);
                let v = w.pull(1);
                w.load(v, 1);
                if pc == 0x05f969 {
                    assert!(!self.calls.pop().expect("inactive near frame").far);
                }
                pc + 1
            }
            0x05f96a => {
                w.rts();
                self.accept_return(w.cpu, Some(false));
                self.pc
            }
            _ => unreachable!("sprite slot {pc:06x}"),
        };
        if self.allocator.is_none() {
            w.cpu.bank = (self.pc >> 16) as u8;
            w.cpu.pc = self.pc as u16;
        }
        Some(w.cycles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rom() -> Vec<u8> {
        std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap()
    }
    fn run(
        rom: &[u8],
        entry: u32,
        slot: usize,
        kind: u8,
        state: u8,
        case: usize,
        field: u64,
    ) -> (usize, Option<SlotChild>) {
        let mut machine = snes::Snes::new();
        machine.cart.load(snes::CartType::LoRom, rom, 0x2000);
        for (i, v) in machine.ram.iter_mut().enumerate() {
            *v = (i as u8).wrapping_mul(17).wrapping_add(0x35);
        }
        let ram = &mut machine.ram;
        ram[0xdd0 + slot] = state;
        ram[0xe20 + slot] = kind;
        ram[0xe40 + slot] = (case * 7) as u8;
        ram[0xf20 + slot] = ((case / 2) % 4) as u8;
        ram[0x11] = if case & 8 == 0 { 0 } else { 15 };
        ram[0xfc1] = u8::from(case & 32 != 0);
        ram[0xfb3] = (case & 1) as u8;
        ram[0xee] = ((case / 4) % 4) as u8;
        ram[0x1b] = (case & 1) as u8;
        ram[0x40a] = (case * 19) as u8;
        ram[0x48e] = (case * 37) as u8;
        for (i, a) in [0xdf0, 0xe00, 0xe10, 0xee0, 0xf10].into_iter().enumerate() {
            ram[a + slot] = [0, 1, 128, 255][(case + i) % 4];
        }
        ram[0xef0 + slot] = [0, 1, 24, 31, 0x80, 0x98, 0x9f, 255][case % 8];
        ram[0xfff] = ((case / 16) % 2) as u8;
        ram[0xe50 + slot] = [0, 10, 20, 255][(case / 4) % 4];
        ram[0xce2 + slot] = [0, 10, 20, 250, 251, 252, 254, 255][(case / 8) % 8];
        ram[0xa0] = [0, 4, 18, 255][(case / 4) % 4];
        ram[0xa1] = ((case / 8) % 2) as u8;
        ram[0xf3c5] = ((case / 4) % 5) as u8;
        ram[0xf3c6] = (case & 31) as u8;
        ram[0xf359] = ((case / 8) % 4) as u8;
        ram[0xf3cc] = ((case / 16) % 3) as u8;
        ram[0xe90 + slot] = (case % 3) as u8;
        ram[0xe80 + slot] = (case / 3 % 2) as u8;
        ram[0xdc0 + slot] = (case / 8 % 2) as u8;
        ram[0xde0 + slot] = (case / 16 % 4) as u8;
        for region in 0..6 {
            let base = [0u16, 0x1f0, 0x200, 0x300][case % 4];
            ram[0xfe0 + region * 2..0xfe2 + region * 2].copy_from_slice(&base.to_le_bytes());
            ram[0xfec + region * 2..0xfee + region * 2]
                .copy_from_slice(&(case as u16).wrapping_add(0xfff8).to_le_bytes());
        }
        let far = matches!(entry, 0x0db818 | 0x05da29 | 0x05db86);
        let mut cpu = NativeNmiContext {
            a: 0xab00 | u16::from(state),
            x: slot as u16,
            y: (case % 4) as u16,
            d: 0,
            db: if matches!(entry, 0x05da49 | 0x05da92) {
                5
            } else {
                6
            },
            bank: (entry >> 16) as u8,
            pc: entry as u16,
            sp: if far { 0x1fc } else { 0x1fd },
            p: 0x30 | ((case as u8) & 0xc3),
        };
        if far {
            ram[0x1fd..0x200].copy_from_slice(&[0x59, 0x80, 6]);
        } else {
            ram[0x1fe..0x200].copy_from_slice(&[0x59, 0x80]);
        }
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
        let clock = CpuMasterTimeline::at_raster(
            field,
            snes::CpuRasterPosition::new(224, 1340),
            snes::CpuBusWorkload::default(),
            snes::CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        let mut source =
            snes::RomCpuTimingProbe::new(machine, clock, snes::SourcePpuReadState::snes9x_reset())
                .unwrap();
        let mut clock = source.timeline().clone();
        let mut work = SpriteSlot::from_entry(entry);
        let mut instructions = 0;
        for _ in 0..2000 {
            if work.complete() || work.child().is_some() {
                break;
            }
            let label=format!("entry={entry:06x} slot={slot} type={kind:02x} state={state} case={case} field={field} pc={:06x}",work.pc);
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
            instructions += 1;
        }
        assert!(work.complete() || work.child().is_some());
        assert_eq!(ram, source.snes().ram);
        if work.complete() {
            assert_eq!((cpu.pc, cpu.sp), (0x805a, 0x1ff));
        }
        (instructions, work.child())
    }
    #[test]
    fn sprite_slot_timers_and_dispatch_match_source_for_every_slot_and_state() {
        let rom = rom();
        let mut instructions = 0;
        let mut damage = 0;
        let mut recoil = 0;
        for slot in 0..16 {
            for case in 0..128 {
                let state = [8, 9, 10, 11][(case / 2) % 4];
                let kind = if case & 16 == 0 { 0x7a } else { 0x73 };
                let (n, child) = run(
                    &rom,
                    0x0683f2,
                    slot,
                    kind,
                    state,
                    case,
                    [0, 1, 30002, 30003][case % 4],
                );
                instructions += n;
                damage += usize::from(child == Some(SlotChild::AgahnimBalls));
                recoil += usize::from(child == Some(SlotChild::Recoil));
            }
        }
        for state in 1..=11 {
            for slot in 0..16 {
                for case in 0..16 {
                    instructions += run(
                        &rom,
                        0x0684e7,
                        slot,
                        0x73,
                        state,
                        case,
                        [0, 1, 30002, 30003][case % 4],
                    )
                    .0;
                }
            }
        }
        assert!(damage > 0 && recoil > 0, "damage={damage} recoil={recoil}");
        eprintln!("4864 slot/timer fixtures, {instructions} instruction boundaries, damage={damage}, recoil={recoil}");
    }
    #[test]
    fn sprite_initialization_matches_source_all_property_and_dispatch_tables() {
        let rom = rom();
        for range in [
            0x0684f4..0x06850c,
            0x06865b..0x068841,
            0x069283..0x069469,
            0x0db080..0x0db818,
            0x0684b9..0x0684bd,
            0x05da31..0x05da3d,
            0x05db95..0x05db9b,
            0x05de3a..0x05de3e,
        ] {
            for a in range {
                assert_eq!(
                    table_byte(a),
                    Some(rom[(a >> 16) as usize * 0x8000 + (a as usize & 0x7fff)]),
                    "table {a:06x}"
                );
            }
        }
        let mut instructions = 0;
        for kind in 0..243 {
            for slot in 0..16 {
                for case in 0..4 {
                    instructions += run(
                        &rom,
                        0x0db818,
                        slot,
                        kind,
                        8,
                        case,
                        [0, 1, 30002, 30003][case],
                    )
                    .0;
                }
                instructions += run(
                    &rom,
                    0x06864d,
                    slot,
                    kind,
                    8,
                    slot,
                    [0, 1, 30002, 30003][slot % 4],
                )
                .0;
                instructions += run(
                    &rom,
                    0x069271,
                    slot,
                    kind,
                    9,
                    slot,
                    [0, 1, 30002, 30003][slot % 4],
                )
                .0;
            }
        }
        eprintln!("23328 property and dispatch fixtures, {instructions} instruction boundaries");
    }
    #[test]
    fn uncle_prep_matches_source_rooms_progress_and_live_helper_handoffs() {
        let rom = rom();
        let mut instructions = 0;
        let mut mantle = 0;
        let mut facing = 0;
        for entry in [0x05da29, 0x05da49, 0x05da92] {
            for slot in 0..16 {
                for case in 0..128 {
                    let (n, child) = run(
                        &rom,
                        entry,
                        slot,
                        0x73,
                        9,
                        case,
                        [0, 1, 30002, 30003][case % 4],
                    );
                    instructions += n;
                    mantle += usize::from(child == Some(SlotChild::PriestMantle));
                    facing += usize::from(child == Some(SlotChild::FaceLink));
                }
            }
        }
        assert!(mantle > 0 && facing > 0);
        eprintln!("6144 Uncle/Priest prep fixtures, {instructions} instruction boundaries, mantle={mantle}, facing={facing}");
    }

    #[test]
    fn uncle_active_wrapper_matches_source_drawing_and_inactive_double_return() {
        let rom = rom();
        let mut instructions = 0;
        let mut returns = 0;
        let mut children = 0;
        for slot in 0..16 {
            for case in 0..128 {
                let (n, child) = run(
                    &rom, 0x05db86, slot, 0x73,
                    [8, 9, 10, 11][case / 4 % 4], case,
                    [0, 1, 30002, 30003][case % 4],
                );
                instructions += n;
                if child.is_none() {
                    returns += 1;
                } else {
                    children += 1;
                }
            }
        }
        assert!(returns > 0 && children > 0);
        eprintln!("2048 active Uncle/Priest fixtures, {instructions} instructions, returns={returns}, pending={children}");
    }
}
