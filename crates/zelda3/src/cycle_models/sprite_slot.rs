//! Active slot caller, timers/OAM, initialization properties, and Uncle/Priest
//! preparation. Every nested return consumes the live native stack. Bodies not
//! owned here remain explicit children, including damage and priest helpers.
use super::{
    indoor_camera::IndoorCamera,
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
    oam_allocate::OamAllocate,
    sprite_drawing::{DrawingChild, SpriteDrawing},
    sprite_slot_data,
};
use snes::CpuMasterTimeline;
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    let entry = match address {
        0x05db95..=0x05db9a => Some((&[0xde2c, 0xdce6, 0xdb9b][..], 0x05db95)),
        0x05de3a..=0x05de3d => Some((&[0xde3e, 0xdf19][..], 0x05de3a)),
        0x05de48..=0x05de51 => Some((&[0xde52, 0xde72, 0xde9a, 0xdeb4, 0xdef8][..], 0x05de48)),
        0x05df20..=0x05df25 => Some((&[0xdf26, 0xdf44, 0xdf43][..], 0x05df20)),
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
        .or_else(|| super::indoor_camera::table_byte(address))
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
    UncleCall(u32),
    RoomTransition(u32),
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
    camera: Option<super::indoor_camera::IndoorCamera>,
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
            camera: None,
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
        if let Some(camera) = self.camera.as_mut() {
            assert_eq!(child, SlotChild::RoomTransition(camera.child().unwrap()));
            camera.return_from_child(cpu);
            return;
        }
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
        if let Some(camera) = self.camera.as_mut() {
            let cycles = camera.step(cpu, clock, bus);
            self.child = camera.child().map(SlotChild::RoomTransition);
            if camera.complete() {
                self.camera = None;
                self.accept_return(cpu, Some(false));
            }
            return cycles;
        }
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
        let m = if w.cpu.p & 0x20 != 0 { 1 } else { 2 };
        let x = if w.cpu.p & 0x10 != 0 { 1 } else { 2 };
        let mask = if m == 1 { 255 } else { 65535 };
        let xmask = if x == 1 { 255 } else { 65535 };
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
                if !matches!(target, 0x06864d | 0x06bfe5 | 0x05de2c
                    | 0x05de3e | 0x05df19 | 0x05de52 | 0x05de72 | 0x05de9a
                    | 0x05deb4 | 0x05def8 | 0x05df26 | 0x05df44 | 0x05df43) {
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
            0x5de3e | 0x5f9ed | 0x5f9f0 | 0x5f9f9 | 0x791b9 => {
                let (operand, next) = match pc {
                    0x5de3e => (0x5f9ed, 0x5de41),
                    0x5f9ed => (0x5f9f4, 0x5f9f0),
                    0x5f9f0 => (0x5fa00, 0x5f9f3),
                    0x5f9f9 => (0x5fa00, 0x5f9fc),
                    0x791b9 => (0x79195, 0x791bc),
                    _ => unreachable!(),
                };
                w.operand_then_internal(2);
                w.push((next - 1) as u16, 2);
                self.calls.push(Call {
                    returned: next,
                    far: false,
                });
                operand
            }
            0x5de41 | 0x5deb4 | 0x5debe | 0x5dee3 | 0x5df19 | 0x5fa00 | 0x5fa10 | 0x5e214 | 0x7919d
            | 0xff546 => {
                let (operand, next) = match pc {
                    0x5de41 => (0x0d80, 0x5de44),
                    0x5deb4 => (0x0df0, 0x5deb7),
                    0x5debe => (0x0d00, 0x5dec1),
                    0x5dee3 => (0x0d90, 0x5dee6),
                    0x5df19 => (0x0d80, 0x5df1c),
                    0x5fa00 => (0x0d40, 0x5fa03),
                    0x5fa10 => (0x0d40, 0x5fa13),
                    0x5e214 => (0x0de0, 0x5e217),
                    0x7919d => (0x0c4a, 0x791a0),
                    0xff546 => (0x0c4a, 0xff549),
                    _ => unreachable!(),
                };
                w.drain(16);
                if x == 2 || (operand & 255) + w.cpu.x > 255 {
                    w.drain(6);
                }
                let value = w.read(w.absolute(operand) + u32::from(w.cpu.x), m);
                w.load(value, m);
                next
            }
            0x5de44 | 0x5df1c => {
                let (operand, next) = match pc {
                    0x5de44 => (0x8781, 0x5de48),
                    0x5df1c => (0x8781, 0x5df20),
                    _ => unreachable!(),
                };
                w.operand_then_internal(3);
                w.push(u16::from(w.cpu.bank), 1);
                w.push((next - 1) as u16, 2);
                self.dispatch = Dispatch::Active;
                operand
            }
            0x5de52 | 0x5de57 | 0x5de5c | 0x5de61 | 0x5de66 | 0x5de8c | 0x5de90 | 0x5de94 | 0x5de9a
            | 0x5dea5 | 0x5deaa | 0x5def8 | 0x5defe | 0x5df03 | 0x5df30 | 0x5df3a | 0x5df50
            | 0x5df55 | 0x5df65 | 0x5e225 | 0x5e22e | 0x5e247 => {
                let (operand, next) = match pc {
                    0x5de52 => (0x0040, 0x5de54),
                    0x5de57 => (0x0009, 0x5de59),
                    0x5de5c => (0x005a, 0x5de5e),
                    0x5de61 => (0x0021, 0x5de63),
                    0x5de66 => (0x001f, 0x5de68),
                    0x5de8c => (0x0057, 0x5de8e),
                    0x5de90 => (0x0021, 0x5de92),
                    0x5de94 => (0x0001, 0x5de96),
                    0x5de9a => (0x000d, 0x5de9c),
                    0x5dea5 => (0x0003, 0x5dea7),
                    0x5deaa => (0x0001, 0x5deac),
                    0x5def8 => (0x0005, 0x5defa),
                    0x5defe => (0x00f3, 0x5df00),
                    0x5df03 => (0x000d, 0x5df05),
                    0x5df30 => (0x000e, 0x5df32),
                    0x5df3a => (0x0000, 0x5df3c),
                    0x5df50 => (0x0001, 0x5df52),
                    0x5df55 => (0x0003, 0x5df57),
                    0x5df65 => (0x0001, 0x5df67),
                    0x5e225 => (0x0002, 0x5e227),
                    0x5e22e => (0x000e, 0x5e230),
                    0x5e247 => (0x0000, 0x5e249),
                    _ => unreachable!(),
                };
                w.drain(8 * u32::from(m));
                let value = operand;
                w.load(value, m);
                next
            }
            0x5de54 | 0x5de59 | 0x5de5e | 0x5de63 | 0x5de96 | 0x5dea7 | 0x5df00 | 0x5df05 | 0x5e1f0
            | 0x5e219 | 0x5e22b => {
                let (operand, next) = match pc {
                    0x5de54 => (0x0fc2, 0x5de57),
                    0x5de59 => (0x0fc3, 0x5de5c),
                    0x5de5e => (0x0fc4, 0x5de61),
                    0x5de63 => (0x0fc5, 0x5de66),
                    0x5de96 => (0x02e4, 0x5de99),
                    0x5dea7 => (0x012c, 0x5deaa),
                    0x5df00 => (0x02cd, 0x5df03),
                    0x5df05 => (0x02ce, 0x5df08),
                    0x5e1f0 => (0x1cf0, 0x5e1f3),
                    0x5e219 => (0x1cf0, 0x5e21c),
                    0x5e22b => (0x010c, 0x5e22e),
                    _ => unreachable!(),
                };
                w.drain(16);
                w.write(w.absolute(operand), w.cpu.a & mask, m);
                next
            }
            0x5de68 | 0x5de9c | 0x5df32 | 0x5df44 | 0x5fa18 => {
                let (operand, next) = match pc {
                    0x5de68 => (0x0000, 0x5de6a),
                    0x5de9c => (0x0000, 0x5de9e),
                    0x5df32 => (0x0000, 0x5df34),
                    0x5df44 => (0x0000, 0x5df46),
                    0x5fa18 => (0x0000, 0x5fa1a),
                    _ => unreachable!(),
                };
                w.drain(8 * u32::from(x));
                let value = operand;
                w.cpu.y = value;
                w.flags(value, x);
                next
            }
            0x5de6a | 0x5de9e | 0x5df2c | 0x5df34 | 0x5e208 | 0x5e233 | 0x5e239 | 0xff570 => {
                let (operand, next) = match pc {
                    0x5de6a => (0x5e219, 0x5de6e),
                    0x5de9e => (0x5e219, 0x5dea2),
                    0x5df2c => (0x791b9, 0x5df30),
                    0x5df34 => (0x5e1f0, 0x5df38),
                    0x5e208 => (0x5e219, 0x5e20c),
                    0x5e233 => (0xff540, 0x5e237),
                    0x5e239 => (0x791b9, 0x5e23d),
                    0xff570 => (0x7f42f, 0xff574),
                    _ => unreachable!(),
                };
                w.operand_then_internal(3);
                w.push(u16::from(w.cpu.bank), 1);
                w.push((next - 1) as u16, 2);
                self.calls.push(Call {
                    returned: next,
                    far: true,
                });
                operand
            }
            0x5de6e | 0x5de83 | 0x5dea2 | 0x5dee0 | 0x5deea | 0x5df40 | 0x5df4d => {
                let (operand, next) = match pc {
                    0x5de6e => (0x0d80, 0x5de71),
                    0x5de83 => (0x0d80, 0x5de86),
                    0x5dea2 => (0x0d80, 0x5dea5),
                    0x5dee0 => (0x0d90, 0x5dee3),
                    0x5deea => (0x0d80, 0x5deed),
                    0x5df40 => (0x0d80, 0x5df43),
                    0x5df4d => (0x0d80, 0x5df50),
                    _ => unreachable!(),
                };
                w.operand_then_internal(2);
                let address = w.absolute(operand) + u32::from(w.cpu.x);
                let old = w.read(address, m);
                w.drain(6);
                let value = old.wrapping_add(1) & mask;
                w.flags(value, m);
                w.write_high_low(address, value, m);
                next
            }
            0x5de71 | 0x5de82 | 0x5de99 | 0x5deaf | 0x5def7 | 0x5df18 | 0x5df43 | 0x5df6b | 0x5f9f3
            | 0x5f9ff | 0x5fa2d | 0x791b8 => {
                w.rts();
                self.accept_return(w.cpu, Some(false));
                self.pc
            }
            0x5de72 | 0x5de78 | 0x5deed | 0x5e1fc | 0x5e229 | 0x5e241 | 0xff55a | 0xff55e => {
                let (operand, next) = match pc {
                    0x5de72 => (0x001a, 0x5de74),
                    0x5de78 => (0x009c, 0x5de7a),
                    0x5deed => (0x001a, 0x5deef),
                    0x5e1fc => (0x004d, 0x5e1fe),
                    0x5e229 => (0x0010, 0x5e22b),
                    0x5e241 => (0x005d, 0x5e243),
                    0xff55a => (0x0023, 0xff55c),
                    0xff55e => (0x0021, 0xff560),
                    _ => unreachable!(),
                };
                let value = w.dp_read(operand, m);
                w.load(value, m);
                next
            }
            0x5de74 | 0x5def2 => {
                let (operand, next) = match pc {
                    0x5de74 => (0x0003, 0x5de76),
                    0x5def2 => (0x0001, 0x5def4),
                    _ => unreachable!(),
                };
                w.drain(8 * u32::from(m));
                let value = operand;
                w.load(w.cpu.a & value & mask, m);
                next
            }
            0x5de76 | 0x5deb7 | 0x5debc | 0x5dee8 | 0x5e245 | 0x791a2 | 0xff54b => {
                let (operand, next) = match pc {
                    0x5de76 => (0x5de82, 0x5de78),
                    0x5deb7 => (0x5deed, 0x5deb9),
                    0x5debc => (0x5dec7, 0x5debe),
                    0x5dee8 => (0x5deed, 0x5deea),
                    0x5e245 => (0x5e24b, 0x5e247),
                    0x791a2 => (0x791a7, 0x791a4),
                    0xff54b => (0xff557, 0xff54d),
                    _ => unreachable!(),
                };
                if w.branch(w.cpu.p & ZERO == 0) {
                    operand
                } else {
                    next
                }
            }
            0x5de7a | 0x5dee6 | 0x5e1fe | 0x5e243 | 0x791a0 | 0xff549 => {
                let (operand, next) = match pc {
                    0x5de7a => (0x0020, 0x5de7c),
                    0x5dee6 => (0x0003, 0x5dee8),
                    0x5e1fe => (0x0002, 0x5e200),
                    0x5e243 => (0x0002, 0x5e245),
                    0x791a0 => (0x001e, 0x791a2),
                    0xff549 => (0x001f, 0xff54b),
                    _ => unreachable!(),
                };
                w.drain(8 * u32::from(m));
                let value = operand;
                let left = w.cpu.a & mask;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(left >= value);
                w.flags(left.wrapping_sub(value) & mask, m);
                next
            }
            0x5de7c | 0x5fa03 | 0x5e200 | 0x79198 | 0xff550 => {
                let (operand, next) = match pc {
                    0x5de7c => (0x5de83, 0x5de7e),
                    0x5fa03 => (0x5fa2d, 0x5fa05),
                    0x5e200 => (0x5e214, 0x5e202),
                    0x79198 => (0x791b8, 0x7919a),
                    0xff550 => (0xff557, 0xff552),
                    _ => unreachable!(),
                };
                if w.branch(w.cpu.p & ZERO != 0) {
                    operand
                } else {
                    next
                }
            }
            0x5de7e | 0x5de80 => {
                let (operand, next) = match pc {
                    0x5de7e => (0x009c, 0x5de80),
                    0x5de80 => (0x009d, 0x5de82),
                    _ => unreachable!(),
                };
                w.dp_increment(operand, m, false);
                next
            }
            0x5de86 | 0x5de89 => {
                let (operand, next) = match pc {
                    0x5de86 => (0x037d, 0x5de89),
                    0x5de89 => (0x037c, 0x5de8c),
                    _ => unreachable!(),
                };
                w.drain(16);
                let address = w.absolute(operand);
                let old = w.read(address, m);
                w.drain(6);
                let value = old.wrapping_add(1) & mask;
                w.flags(value, m);
                w.write_high_low(address, value, m);
                next
            }
            0x5de8e | 0x5de92 | 0x5e227 | 0x5e230 | 0x5e249 | 0xff55c | 0xff560 | 0xff567 | 0xff56c => {
                let (operand, next) = match pc {
                    0x5de8e => (0x0020, 0x5de90),
                    0x5de92 => (0x0021, 0x5de94),
                    0x5e227 => (0x0011, 0x5e229),
                    0x5e230 => (0x0010, 0x5e232),
                    0x5e249 => (0x005d, 0x5e24b),
                    0xff55c => (0x0041, 0xff55e),
                    0xff560 => (0x0040, 0xff562),
                    0xff567 => (0x0022, 0xff569),
                    0xff56c => (0x0020, 0xff56e),
                    _ => unreachable!(),
                };
                w.dp_store(operand, w.cpu.a & mask, m);
                next
            }
            0x5deac | 0x5dec4 | 0x5deca | 0x5ded0 | 0x5ded7 | 0x5dedd | 0x5def4 | 0x5df52 | 0x5fa0d
            | 0x5fa23 | 0x5fa2a => {
                let (operand, next) = match pc {
                    0x5deac => (0x0dc0, 0x5deaf),
                    0x5dec4 => (0x0d00, 0x5dec7),
                    0x5deca => (0x0df0, 0x5decd),
                    0x5ded0 => (0x0de0, 0x5ded3),
                    0x5ded7 => (0x0d50, 0x5deda),
                    0x5dedd => (0x0d40, 0x5dee0),
                    0x5def4 => (0x0dc0, 0x5def7),
                    0x5df52 => (0x0dc0, 0x5df55),
                    0x5fa0d => (0x0d60, 0x5fa10),
                    0x5fa23 => (0x0d00, 0x5fa26),
                    0x5fa2a => (0x0d20, 0x5fa2d),
                    _ => unreachable!(),
                };
                w.operand_then_internal(2);
                w.write(w.absolute(operand) + u32::from(w.cpu.x), w.cpu.a & mask, m);
                next
            }
            0x5deb9 => {
                let (operand, next) = (0x0d90, 0x5debc);
                w.drain(16);
                if x == 2 || (operand & 255) + w.cpu.x > 255 {
                    w.drain(6);
                }
                let value = w.read(w.absolute(operand) + u32::from(w.cpu.x), x);
                w.cpu.y = value;
                w.flags(value, x);
                next
            }
            0x5dec1 | 0x5e212 => {
                let next = match pc {
                    0x5dec1 => 0x5dec2,
                    0x5e212 => 0x5e213,
                    _ => unreachable!(),
                };
                w.drain(6);
                w.cpu.p |= CARRY;
                next
            }
            0x5dec2 => {
                let (operand, next) = (0x0002, 0x5dec4);
                w.drain(8 * u32::from(m));
                let value = operand;
                w.sbc(value, m);
                next
            }
            0x5dec7 | 0x5decd | 0x5ded4 | 0x5deda => {
                let (operand, next) = match pc {
                    0x5dec7 => (0xdeb0, 0x5deca),
                    0x5decd => (0xdeb2, 0x5ded0),
                    0x5ded4 => (0xde24, 0x5ded7),
                    0x5deda => (0xde28, 0x5dedd),
                    _ => unreachable!(),
                };
                w.drain(16);
                if x == 2 || (operand & 255) + w.cpu.y > 255 {
                    w.drain(6);
                }
                let value = w.read(w.absolute(operand) + u32::from(w.cpu.y), m);
                w.load(value, m);
                next
            }
            0x5ded3 => {
                let next = 0x5ded4;
                w.drain(6);
                w.cpu.y = w.cpu.a & xmask;
                w.flags(w.cpu.y, x);
                next
            }
            0x5deef | 0x5def0 | 0x5def1 | 0x5fa14 | 0x5fa15 | 0x5fa16 | 0x5fa17 => {
                let next = match pc {
                    0x5deef => 0x5def0,
                    0x5def0 => 0x5def1,
                    0x5def1 => 0x5def2,
                    0x5fa14 => 0x5fa15,
                    0x5fa15 => 0x5fa16,
                    0x5fa16 => 0x5fa17,
                    0x5fa17 => 0x5fa18,
                    _ => unreachable!(),
                };
                w.drain(6);
                let value = w.cpu.a & mask;
                w.cpu.p = (w.cpu.p & !CARRY) | (value & 1) as u8;
                w.load(value >> 1, m);
                next
            }
            0x5defa | 0x5df0e | 0x5df3c | 0x5df57 | 0x5df61 | 0x5df67 => {
                let (operand, next) = match pc {
                    0x5defa => (0x7ef3cc, 0x5defe),
                    0x5df0e => (0x7ef3c6, 0x5df12),
                    0x5df3c => (0x7ef3cc, 0x5df40),
                    0x5df57 => (0x7ef3c8, 0x5df5b),
                    0x5df61 => (0x7ef3c6, 0x5df65),
                    0x5df67 => (0x7ef3c5, 0x5df6b),
                    _ => unreachable!(),
                };
                w.drain(24);
                w.write(operand, w.cpu.a & mask, m);
                next
            }
            0x5df08 | 0x5df5b => {
                let (operand, next) = match pc {
                    0x5df08 => (0x7ef3c6, 0x5df0c),
                    0x5df5b => (0x7ef3c6, 0x5df5f),
                    _ => unreachable!(),
                };
                w.drain(24);
                let value = w.read(operand, m);
                w.load(value, m);
                next
            }
            0x5df0c | 0x5df5f | 0x5fa1d => {
                let (operand, next) = match pc {
                    0x5df0c => (0x0010, 0x5df0e),
                    0x5df5f => (0x0001, 0x5df61),
                    0x5fa1d => (0x00f0, 0x5fa1f),
                    _ => unreachable!(),
                };
                w.drain(8 * u32::from(m));
                let value = operand;
                w.load((w.cpu.a | value) & mask, m);
                next
            }
            0x5df12 | 0x791a4 => {
                let (operand, next) = match pc {
                    0x5df12 => (0x0dd0, 0x5df15),
                    0x791a4 => (0x0c4a, 0x791a7),
                    _ => unreachable!(),
                };
                w.operand_then_internal(2);
                w.write(w.absolute(operand) + u32::from(w.cpu.x), 0, m);
                next
            }
            0x5df15 | 0x5df46 | 0x5e21f | 0x5e222 | 0x791ab | 0x791b0 | 0x791b5 | 0xff552 => {
                let (operand, next) = match pc {
                    0x5df15 => (0x02e4, 0x5df18),
                    0x5df46 => (0x02e9, 0x5df49),
                    0x5e21f => (0x0223, 0x5e222),
                    0x5e222 => (0x1cd8, 0x5e225),
                    0x791ab => (0x0374, 0x791ae),
                    0x791b0 => (0x0372, 0x791b3),
                    0x791b5 => (0x032b, 0x791b8),
                    0xff552 => (0x037e, 0xff555),
                    _ => unreachable!(),
                };
                w.drain(16);
                w.write(w.absolute(operand), 0, m);
                next
            }
            0x5df26 | 0x5df49 | 0x5e1f6 => {
                let (operand, next) = match pc {
                    0x5df26 => (0x6f129, 0x5df2a),
                    0x5df49 => (0x799ad, 0x5df4d),
                    0x5e1f6 => (0x6f129, 0x5e1fa),
                    _ => unreachable!(),
                };
                w.operand_then_internal(3);
                w.push(u16::from(w.cpu.bank), 1);
                w.push((next - 1) as u16, 2);
                self.calls.push(Call {
                    returned: next,
                    far: true,
                });
                self.child = Some(SlotChild::UncleCall(operand));
                operand
            }
            0x5df2a | 0x5df38 | 0x5e1fa => {
                let (operand, next) = match pc {
                    0x5df2a => (0x5df30, 0x5df2c),
                    0x5df38 => (0x5df43, 0x5df3a),
                    0x5e1fa => (0x5e214, 0x5e1fc),
                    _ => unreachable!(),
                };
                if w.branch(w.cpu.p & CARRY == 0) {
                    operand
                } else {
                    next
                }
            }
            0x5f9f4 => {
                let next = 0x5f9f5;
                w.drain(6);
                w.load(w.cpu.x, m);
                next
            }
            0x5f9f5 | 0x5fa09 | 0x5e217 => {
                let next = match pc {
                    0x5f9f5 => 0x5f9f6,
                    0x5fa09 => 0x5fa0a,
                    0x5e217 => 0x5e218,
                    _ => unreachable!(),
                };
                w.clear_carry();
                next
            }
            0x5f9f6 => {
                let (operand, next) = (0x0010, 0x5f9f8);
                w.drain(8 * u32::from(m));
                let value = operand;
                if m == 1 {
                    w.adc_byte(value);
                } else {
                    w.adc(value);
                }
                next
            }
            0x5f9f8 => {
                let next = 0x5f9f9;
                w.drain(6);
                w.cpu.x = w.cpu.a & xmask;
                w.flags(w.cpu.x, x);
                next
            }
            0x5f9fc => {
                let (operand, next) = (0x0fa0, 0x5f9ff);
                w.drain(16);
                let value = w.read(w.absolute(operand), x);
                w.cpu.x = value;
                w.flags(value, x);
                next
            }
            0x5fa05 | 0x5fa06 | 0x5fa07 | 0x5fa08 => {
                let next = match pc {
                    0x5fa05 => 0x5fa06,
                    0x5fa06 => 0x5fa07,
                    0x5fa07 => 0x5fa08,
                    0x5fa08 => 0x5fa09,
                    _ => unreachable!(),
                };
                w.shift_left(m);
                next
            }
            0x5fa0a | 0x5fa20 | 0x5fa27 => {
                let (operand, next) = match pc {
                    0x5fa0a => (0x0d60, 0x5fa0d),
                    0x5fa20 => (0x0d00, 0x5fa23),
                    0x5fa27 => (0x0d20, 0x5fa2a),
                    _ => unreachable!(),
                };
                w.drain(16);
                if x == 2 || (operand & 255) + w.cpu.x > 255 {
                    w.drain(6);
                }
                let value = w.read(w.absolute(operand) + u32::from(w.cpu.x), m);
                if m == 1 {
                    w.adc_byte(value);
                } else {
                    w.adc(value);
                }
                next
            }
            0x5fa13 => {
                let next = 0x5fa14;
                w.drain(6);
                w.push(u16::from(w.cpu.p), 1);
                next
            }
            0x5fa1a => {
                let next = 0x5fa1b;
                w.drain(12);
                w.cpu.p = w.pull(1) as u8;
                if w.cpu.p & 0x10 != 0 {
                    w.cpu.x &= 255;
                    w.cpu.y &= 255;
                }
                next
            }
            0x5fa1b | 0x791a8 | 0xff558 => {
                let (operand, next) = match pc {
                    0x5fa1b => (0x5fa20, 0x5fa1d),
                    0x791a8 => (0x7919d, 0x791aa),
                    0xff558 => (0xff546, 0xff55a),
                    _ => unreachable!(),
                };
                if w.branch(w.cpu.p & NEGATIVE == 0) {
                    operand
                } else {
                    next
                }
            }
            0x5fa1f => {
                let next = 0x5fa20;
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_sub(1) & xmask;
                w.flags(w.cpu.y, x);
                next
            }
            0x5fa26 | 0x5e20f => {
                let next = match pc {
                    0x5fa26 => 0x5fa27,
                    0x5e20f => 0x5e210,
                    _ => unreachable!(),
                };
                w.drain(6);
                w.load(w.cpu.y, m);
                next
            }
            0x5e1f3 | 0x5e21c => {
                let (operand, next) = match pc {
                    0x5e1f3 => (0x1cf1, 0x5e1f6),
                    0x5e21c => (0x1cf1, 0x5e21f),
                    _ => unreachable!(),
                };
                w.drain(16);
                w.write(w.absolute(operand), w.cpu.y, x);
                next
            }
            0x5e202 | 0x79195 | 0xff54d | 0xff564 | 0xff569 => {
                let (operand, next) = match pc {
                    0x5e202 => (0x1cf0, 0x5e205),
                    0x79195 => (0x0372, 0x79198),
                    0xff54d => (0x037e, 0xff550),
                    0xff564 => (0x0fc2, 0xff567),
                    0xff569 => (0x0fc4, 0xff56c),
                    _ => unreachable!(),
                };
                w.drain(16);
                let value = w.read(w.absolute(operand), m);
                w.load(value, m);
                next
            }
            0x5e205 => {
                let (operand, next) = (0x1cf1, 0x5e208);
                w.drain(16);
                let value = w.read(w.absolute(operand), x);
                w.cpu.y = value;
                w.flags(value, x);
                next
            }
            0x5e20c => {
                let (operand, next) = (0x5f93f, 0x5e20f);
                w.operand_then_internal(2);
                w.push((next - 1) as u16, 2);
                self.calls.push(Call {
                    returned: next,
                    far: false,
                });
                self.child = Some(SlotChild::FaceLink);
                operand
            }
            0x5e210 => {
                let (operand, next) = (0x0003, 0x5e212);
                w.drain(8 * u32::from(m));
                let value = operand;
                w.load((w.cpu.a ^ value) & mask, m);
                next
            }
            0x5e213 | 0x5e218 | 0x5e24c | 0x791bc | 0x7f438 | 0xff576 => {
                w.rtl();
                self.accept_return(w.cpu, Some(true));
                self.pc
            }
            0x5e232 | 0x7919a | 0x7f432 | 0xff543 => {
                let next = match pc {
                    0x5e232 => 0x5e233,
                    0x7919a => 0x7919b,
                    0x7f432 => 0x7f433,
                    0xff543 => 0xff544,
                    _ => unreachable!(),
                };
                w.drain(6);
                w.push(w.cpu.x, x);
                next
            }
            0x5e237 | 0x5e23d | 0x5e23f | 0x791ae | 0x791b3 => {
                let (operand, next) = match pc {
                    0x5e237 => (0x005e, 0x5e239),
                    0x5e23d => (0x004d, 0x5e23f),
                    0x5e23f => (0x0046, 0x5e241),
                    0x791ae => (0x005e, 0x791b0),
                    0x791b3 => (0x0050, 0x791b5),
                    _ => unreachable!(),
                };
                w.dp_store(operand, 0, m);
                next
            }
            0x5e24b | 0x791aa | 0x7f436 | 0xff574 => {
                let next = match pc {
                    0x5e24b => 0x5e24c,
                    0x791aa => 0x791ab,
                    0x7f436 => 0x7f437,
                    0xff574 => 0xff575,
                    _ => unreachable!(),
                };
                w.drain(12);
                let value = w.pull(x);
                w.cpu.x = value;
                w.flags(value, x);
                next
            }
            0x7919b | 0xff544 => {
                let (operand, next) = match pc {
                    0x7919b => (0x0004, 0x7919d),
                    0xff544 => (0x0004, 0xff546),
                    _ => unreachable!(),
                };
                w.drain(8 * u32::from(x));
                let value = operand;
                w.cpu.x = value;
                w.flags(value, x);
                next
            }
            0x791a7 | 0xff557 => {
                let next = match pc {
                    0x791a7 => 0x791a8,
                    0xff557 => 0xff558,
                    _ => unreachable!(),
                };
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_sub(1) & xmask;
                w.flags(w.cpu.x, x);
                next
            }
            0x7f42f | 0xff540 => {
                let next = match pc {
                    0x7f42f => 0x7f430,
                    0xff540 => 0xff541,
                    _ => unreachable!(),
                };
                w.drain(6);
                w.push(u16::from(w.cpu.db), 1);
                next
            }
            0x7f430 | 0xff541 => {
                let next = match pc {
                    0x7f430 => 0x7f431,
                    0xff541 => 0xff542,
                    _ => unreachable!(),
                };
                w.drain(6);
                w.push(u16::from(w.cpu.bank), 1);
                next
            }
            0x7f431 | 0x7f437 | 0xff542 | 0xff575 => {
                let next = match pc {
                    0x7f431 => 0x7f432,
                    0x7f437 => 0x7f438,
                    0xff542 => 0xff543,
                    0xff575 => 0xff576,
                    _ => unreachable!(),
                };
                w.drain(12);
                let value = w.pull(1);
                w.cpu.db = value as u8;
                w.flags(value, 1);
                next
            }
            0x7f433 => {
                let (operand, next) = (0x7e8f0, 0x7f436);
                w.operand_then_internal(2);
                w.push((next - 1) as u16, 2);
                self.calls.push(Call {
                    returned: next,
                    far: false,
                });
                self.camera = Some(IndoorCamera::default());
                operand
            }
            0xff555 => {
                let (operand, next) = (0xff55a, 0xff557);
                if w.branch(true) {
                    operand
                } else {
                    next
                }
            }
            0xff562 => {
                let (operand, next) = (0x0020, 0xff564);
                w.width(operand as u8, true);
                next
            }
            0xff56e => {
                let (operand, next) = (0x0020, 0xff570);
                w.width(operand as u8, false);
                next
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
    #[test]
    fn uncle_states_match_source_movement_dialogue_camera_and_real_returns() {
        use super::super::intro_clear::Access;
        use std::collections::BTreeSet;
        let rom = rom();
        let mut visited = BTreeSet::new();
        let mut instructions = 0;
        let mut complete = 0;
        let mut children = BTreeSet::new();
        for entry in [0x05de3e, 0x05df19] {
            for slot in 0..16usize {
                for case in 0..256usize {
                    let mut machine = snes::Snes::new();
                    machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                    let r = &mut machine.ram;
                    r[0xfa0] = slot as u8;
                    r[0xd80 + slot] = (case % if entry == 0x05de3e { 5 } else { 3 }) as u8;
                    r[0xd90 + slot] = (case % 3) as u8;
                    r[0xdf0 + slot] = [0, 1, 128, 255][case % 4];
                    for offset in [0, 16] {
                        let k = slot + offset;
                        r[0xd40 + k] = [0, 1, 12, 127, 128, 244, 255, 16][case / 4 % 8];
                        r[0xd60 + k] = (case as u8).wrapping_mul(37);
                        r[0xd00 + k] = (case as u8).wrapping_mul(19);
                        r[0xd20 + k] = [0, 1, 127, 255][case % 4];
                    }
                    r[0x1a] = (case % 8) as u8;
                    r[0x9c] = [0x20, 0x21, 0, 255][case / 8 % 4];
                    r[0x9d] = case as u8;
                    r[0x10] = 7;
                    r[0x1b] = 1;
                    r[0x6c] = 0;
                    r[0x20] = 0x50;
                    r[0x21] = 0x21;
                    r[0x22] = 0x40;
                    r[0x23] = 9;
                    r[0x40] = r[0x21];
                    r[0x41] = r[0x23];
                    r[0xa0] = case as u8;
                    r[0xa1] = 0;
                    r[0xa9] = (case % 2) as u8;
                    r[0xaa] = (case / 2 % 2 * 2) as u8;
                    r[0xa6] = (case / 4 % 2 * 2) as u8;
                    r[0xa7] = (case / 8 % 2 * 2) as u8;
                    r[0x5d] = [0, 2, 3, 8][case / 32 % 4];
                    r[0x372] = (case % 2) as u8;
                    r[0x374] = 1;
                    r[0x50] = 3;
                    r[0x32b] = 4;
                    r[0x37e] = (case / 2 % 2) as u8;
                    for k in 0..5 {
                        r[0xc4a + k] = [0, 0x1e, 0x1f][(case + k) % 3];
                    }
                    for (at, value) in [(0xfc2, 0x0940u16), (0xfc4, 0x215a)] {
                        r[at..at + 2].copy_from_slice(&value.to_le_bytes());
                    }
                    r[0x1fe..0x200].copy_from_slice(&[0x59, 0x80]);
                    let mut cpu = NativeNmiContext {
                        a: 0xabcd,
                        x: slot as u16,
                        y: 7,
                        d: 0,
                        db: 5,
                        bank: 5,
                        pc: entry as u16,
                        sp: 0x1fd,
                        p: 0x30 | (case as u8 & 0xc3),
                    };
                    machine.cpu.a = cpu.a;
                    machine.cpu.x = cpu.x;
                    machine.cpu.y = cpu.y;
                    machine.cpu.k = cpu.bank;
                    machine.cpu.db = cpu.db;
                    machine.cpu.pc = cpu.pc;
                    machine.cpu.sp = cpu.sp;
                    machine.cpu.e = false;
                    machine.cpu.unpack_flags(cpu.p);
                    let mut ram = machine.ram.clone();
                    let clock = CpuMasterTimeline::at_raster(
                        4000 + case as u64,
                        snes::CpuRasterPosition::new(if case & 1 == 0 { 224 } else { 261 }, 1300),
                        snes::CpuBusWorkload::default(),
                        snes::CpuFieldTiming::NON_INTERLACE_EVEN,
                    );
                    let mut source = snes::RomCpuTimingProbe::new(
                        machine,
                        clock,
                        snes::SourcePpuReadState::snes9x_reset(),
                    )
                    .unwrap();
                    let mut clock = source.timeline().clone();
                    let mut work = SpriteSlot::from_entry(entry);
                    for _ in 0..2000 {
                        if work.complete() || work.child().is_some() {
                            break;
                        }
                        visited.insert((u32::from(cpu.bank) << 16) | u32::from(cpu.pc));
                        super::super::native_work::compare_instruction(
                            &format!(
                                "Uncle entry={entry:06x} slot={slot} case={case} pc={:02x}:{:04x}",
                                cpu.bank, cpu.pc
                            ),
                            &mut cpu,
                            &mut clock,
                            &mut source,
                            &mut ram,
                            &rom,
                            |cpu, clock, bus| {
                                work.step(cpu, clock, |at, access| {
                                    if let Access::Read(a, width) = access {
                                        if a & 0x408000 == 0x8000 {
                                            for b in 0..width {
                                                let a = a + u32::from(b);
                                                assert_eq!(
                                                    table_byte(a),
                                                    Some(
                                                        rom[(a >> 16) as usize * 0x8000
                                                            + (a as usize & 0x7fff)]
                                                    ),
                                                    "Uncle table {a:06x}"
                                                );
                                            }
                                        }
                                    }
                                    bus(at, access)
                                })
                                .unwrap();
                            },
                        );
                        work = work.clone();
                        instructions += 1;
                    }
                    assert!(work.complete() || work.child().is_some());
                    assert_eq!(ram, source.snes().ram);
                    if work.complete() {
                        complete += 1;
                        assert_eq!((cpu.bank, cpu.pc, cpu.sp, cpu.db), (5, 0x805a, 0x1ff, 5));
                        assert!(work.calls.is_empty());
                    } else {
                        children.insert((u32::from(cpu.bank) << 16) | u32::from(cpu.pc));
                    }
                }
            }
        }
        for pc in [
            0x05de52, 0x05de72, 0x05de9a, 0x05deb4, 0x05def8, 0x05df26, 0x05df44, 0x05df43,
            0x05fa1d, 0x05fa27, 0x05e24c, 0x0ff54d, 0x0791a4, 0x07e8f0,
        ] {
            assert!(visited.contains(&pc), "uncovered Uncle path {pc:06x}");
        }
        assert_eq!(children, BTreeSet::from([0x06f129, 0x0799ad]));
        eprintln!("8192 active Uncle fixtures, {instructions} instruction boundaries, {complete} full returns, {} PCs; remaining passage calls {children:x?}",visited.len());
    }
}
