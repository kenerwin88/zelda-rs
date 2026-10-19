//! Ground velocity selection, position integration, floor/conveyor movement and return.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    if let Some(i) = address.checked_sub(0x07e227) {
        if let Some(v) = [
            24, 16, 10, 24, 16, 8, 8, 4, 12, 16, 9, 25, 20, 13, 16, 8, 64, 42, 16, 8, 4, 2, 48, 24,
            32, 21, 240, 0, 240, 1,
        ]
        .get(i as usize)
        {
            return Some(*v);
        }
    }
    [8, 4, 2, 1, 248, 8, 0, 0, 0, 0, 248, 8]
        .get(address.checked_sub(0x07e5e4)? as usize)
        .copied()
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GroundVelocityTail {
    Complete,
    Swim,
    Subpixels { x: u16, y: u16 },
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct GroundVelocity {
    pc: Option<u16>,
    tail: Option<GroundVelocityTail>,
    subpixels: [u16; 2],
}
impl GroundVelocity {
    pub(crate) fn tail(&self) -> Option<GroundVelocityTail> {
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
        let pc = self.pc.unwrap_or(0x82be);
        assert_eq!((cpu.bank, cpu.pc, cpu.d, cpu.p & 0x18), (7, pc, 0, 0x10));
        let width = if cpu.p & 0x20 != 0 { 1 } else { 2 };
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        let next = match pc {
            0x82be => {
                w.operand_then_internal(3);
                w.push(w.cpu.bank as u16, 1);
                w.push(pc + 3, 2);
                0xe245
            }
            0xe245 => {
                w.drain(6);
                w.push(w.cpu.db as u16, 1);
                0xe246
            }
            0xe246 => {
                w.drain(6);
                w.push(w.cpu.bank as u16, 1);
                0xe247
            }
            0xe247 => {
                w.drain(12);
                let v = w.pull(1);
                w.cpu.db = v as u8;
                w.flags(v, 1);
                0xe248
            }
            0xe248 => {
                w.drain(8);
                let v = w.read(u32::from(0x11u16), width);
                w.load(v, width);
                0xe24a
            }
            0xe24a => {
                w.compare_immediate(2, width);
                0xe24c
            }
            0xe24c => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xe254
                } else {
                    0xe24e
                }
            }
            0xe24e => {
                w.drain(8);
                let v = w.read(u32::from(0x10u16), width);
                w.load(v, width);
                0xe250
            }
            0xe250 => {
                w.compare_immediate(14, width);
                0xe252
            }
            0xe252 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xe259
                } else {
                    0xe254
                }
            }
            0xe254 => {
                w.drain(16);
                let v = w.read(w.absolute(0xb7bu16), width);
                w.load(v, width);
                0xe257
            }
            0xe257 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xe274
                } else {
                    0xe259
                }
            }
            0xe259 => {
                w.drain(8);
                let v = w.read(u32::from(0x20u16), width);
                w.load(v, width);
                0xe25b
            }
            0xe25b => {
                w.drain(8);
                let address = u32::from(0x0u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe25d
            }
            0xe25d => {
                w.drain(8);
                let address = u32::from(0x3eu16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe25f
            }
            0xe25f => {
                w.drain(8);
                let v = w.read(u32::from(0x22u16), width);
                w.load(v, width);
                0xe261
            }
            0xe261 => {
                w.drain(8);
                let address = u32::from(0x1u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe263
            }
            0xe263 => {
                w.drain(8);
                let address = u32::from(0x3fu16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe265
            }
            0xe265 => {
                w.drain(8);
                let v = w.read(u32::from(0x21u16), width);
                w.load(v, width);
                0xe267
            }
            0xe267 => {
                w.drain(8);
                let address = u32::from(0x2u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe269
            }
            0xe269 => {
                w.drain(8);
                let address = u32::from(0x40u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe26b
            }
            0xe26b => {
                w.drain(8);
                let v = w.read(u32::from(0x23u16), width);
                w.load(v, width);
                0xe26d
            }
            0xe26d => {
                w.drain(8);
                let address = u32::from(0x3u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe26f
            }
            0xe26f => {
                w.drain(8);
                let address = u32::from(0x41u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe271
            }
            0xe271 => {
                w.operand_then_internal(2);
                0xe3e0
            }
            0xe274 => {
                w.drain(8);
                let v = w.read(u32::from(0x5du16), width);
                w.load(v, width);
                0xe276
            }
            0xe276 => {
                w.compare_immediate(4, width);
                0xe278
            }
            0xe278 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xe28a
                } else {
                    0xe27a
                }
            }
            0xe27a => {
                w.drain(16);
                let v = w.read(w.absolute(0x34au16), width);
                w.load(v, width);
                0xe27d
            }
            0xe27d => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xe28d
                } else {
                    0xe27f
                }
            }
            0xe27f => {
                w.drain(16);
                let v = w.read(w.absolute(0x372u16), width);
                w.load(v, width);
                0xe282
            }
            0xe282 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xe28a
                } else {
                    0xe284
                }
            }
            0xe284 => {
                w.drain(8 * u32::from(width));
                let v = 24;
                w.load(v, width);
                0xe286
            }
            0xe286 => {
                w.drain(8);
                let address = u32::from(0x0u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe288
            }
            0xe288 => {
                w.branch(true);
                0xe2ca
            }
            0xe28a => {
                w.operand_then_internal(2);
                self.tail = Some(GroundVelocityTail::Swim);
                0xe42a
            }
            0xe28d => {
                w.drain(16);
                let v = w.read(w.absolute(0x372u16), width);
                w.load(v, width);
                0xe290
            }
            0xe290 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xe29e
                } else {
                    0xe292
                }
            }
            0xe292 => {
                w.drain(8);
                let address = u32::from(0x57u16);
                w.write(address, 0, width);
                0xe294
            }
            0xe294 => {
                w.drain(16);
                let v = w.read(w.absolute(0x2f1u16), width);
                w.load(v, width);
                0xe297
            }
            0xe297 => {
                w.compare_immediate(16, width);
                0xe299
            }
            0xe299 => {
                let taken = w.branch(w.cpu.p & CARRY != 0);
                if taken {
                    0xe29e
                } else {
                    0xe29b
                }
            }
            0xe29b => {
                w.operand_then_internal(2);
                0xe545
            }
            0xe29e => {
                w.drain(16);
                let v = w.read(w.absolute(0x316u16), width);
                w.load(v, width);
                0xe2a1
            }
            0xe2a1 => {
                w.drain(16);
                let v = w.read(w.absolute(0x317u16), width);
                w.load((w.cpu.a | v) & if width == 1 { 255 } else { 65535 }, width);
                0xe2a4
            }
            0xe2a4 => {
                w.compare_immediate(15, width);
                0xe2a6
            }
            0xe2a6 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xe2ab
                } else {
                    0xe2a8
                }
            }
            0xe2a8 => {
                w.operand_then_internal(2);
                0xe402
            }
            0xe2ab => {
                w.drain(8);
                let v = w.read(u32::from(0x5eu16), width);
                w.load(v, width);
                0xe2ad
            }
            0xe2ad => {
                w.drain(8);
                let address = u32::from(0x0u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe2af
            }
            0xe2af => {
                w.drain(16);
                let v = w.read(w.absolute(0x351u16), width);
                w.load(v, width);
                0xe2b2
            }
            0xe2b2 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xe2ca
                } else {
                    0xe2b4
                }
            }
            0xe2b4 => {
                w.drain(8);
                let v = w.read(u32::from(0x5eu16), width);
                w.load(v, width);
                0xe2b6
            }
            0xe2b6 => {
                w.compare_immediate(16, width);
                0xe2b8
            }
            0xe2b8 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xe2be
                } else {
                    0xe2ba
                }
            }
            0xe2ba => {
                w.item_x(22, 1);
                0xe2bc
            }
            0xe2bc => {
                w.branch(true);
                0xe2c8
            }
            0xe2be => {
                w.item_x(12, 1);
                0xe2c0
            }
            0xe2c0 => {
                w.drain(8);
                let v = w.read(u32::from(0x5eu16), width);
                w.load(v, width);
                0xe2c2
            }
            0xe2c2 => {
                w.compare_immediate(12, width);
                0xe2c4
            }
            0xe2c4 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xe2c8
                } else {
                    0xe2c6
                }
            }
            0xe2c6 => {
                w.item_x(14, 1);
                0xe2c8
            }
            0xe2c8 => {
                w.drain(8);
                let address = u32::from(0x0u16);
                w.write(address, w.cpu.x, 1);
                0xe2ca
            }
            0xe2ca => {
                w.drain(8);
                let address = u32::from(0x27u16);
                w.write(address, 0, width);
                0xe2cc
            }
            0xe2cc => {
                w.drain(8);
                let address = u32::from(0x28u16);
                w.write(address, 0, width);
                0xe2ce
            }
            0xe2ce => {
                w.drain(8);
                let address = u32::from(0x68u16);
                w.write(address, 0, width);
                0xe2d0
            }
            0xe2d0 => {
                w.drain(8);
                let address = u32::from(0x69u16);
                w.write(address, 0, width);
                0xe2d2
            }
            0xe2d2 => {
                w.item_x(0, 1);
                0xe2d4
            }
            0xe2d4 => {
                w.drain(8);
                let v = w.read(u32::from(0x67u16), width);
                w.load(v, width);
                0xe2d6
            }
            0xe2d6 => {
                w.drain(6);
                w.cpu.y = w.cpu.a & 255;
                w.flags(w.cpu.y, 1);
                0xe2d7
            }
            0xe2d7 => {
                w.mask(12, width);
                0xe2d9
            }
            0xe2d9 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xe2e2
                } else {
                    0xe2db
                }
            }
            0xe2db => {
                w.drain(6);
                w.load(w.cpu.y, width);
                0xe2dc
            }
            0xe2dc => {
                w.mask(3, width);
                0xe2de
            }
            0xe2de => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xe2e2
                } else {
                    0xe2e0
                }
            }
            0xe2e0 => {
                w.item_x(1, 1);
                0xe2e2
            }
            0xe2e2 => {
                w.drain(6);
                w.load(w.cpu.x, width);
                0xe2e3
            }
            0xe2e3 => {
                w.clear_carry();
                0xe2e4
            }
            0xe2e4 => {
                w.drain(8);
                let v = w.read(u32::from(0x0u16), width);
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xe2e6
            }
            0xe2e6 => {
                w.drain(6);
                w.cpu.x = w.cpu.a & 255;
                w.flags(w.cpu.x, 1);
                0xe2e7
            }
            0xe2e7 => {
                w.drain(8);
                let v = w.read(u32::from(0x5bu16), width);
                w.load(v, width);
                0xe2e9
            }
            0xe2e9 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xe301
                } else {
                    0xe2eb
                }
            }
            0xe2eb => {
                w.compare_immediate(3, width);
                0xe2ed
            }
            0xe2ed => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xe330
                } else {
                    0xe2ef
                }
            }
            0xe2ef => {
                w.drain(8);
                let v = w.read(u32::from(0x57u16), width);
                w.load(v, width);
                0xe2f1
            }
            0xe2f1 => {
                w.compare_immediate(48, width);
                0xe2f3
            }
            0xe2f3 => {
                let taken = w.branch(w.cpu.p & CARRY != 0);
                if taken {
                    0xe2fb
                } else {
                    0xe2f5
                }
            }
            0xe2f5 => {
                w.drain(8 * u32::from(width));
                let v = 8;
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xe2f7
            }
            0xe2f7 => {
                w.drain(8);
                let address = u32::from(0x57u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe2f9
            }
            0xe2f9 => {
                w.branch(true);
                0xe330
            }
            0xe2fb => {
                w.drain(8 * u32::from(width));
                let v = 32;
                w.load(v, width);
                0xe2fd
            }
            0xe2fd => {
                w.drain(8);
                let address = u32::from(0x57u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe2ff
            }
            0xe2ff => {
                w.branch(true);
                0xe330
            }
            0xe301 => {
                w.drain(8);
                let v = w.read(u32::from(0x57u16), width);
                w.load(v, width);
                0xe303
            }
            0xe303 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xe330
                } else {
                    0xe305
                }
            }
            0xe305 => {
                w.item_x(10, 1);
                0xe307
            }
            0xe307 => {
                w.drain(8);
                let v = w.read(u32::from(0x11u16), width);
                w.load(v, width);
                0xe309
            }
            0xe309 => {
                w.compare_immediate(8, width);
                0xe30b
            }
            0xe30b => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xe313
                } else {
                    0xe30d
                }
            }
            0xe30d => {
                w.compare_immediate(16, width);
                0xe30f
            }
            0xe30f => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xe313
                } else {
                    0xe311
                }
            }
            0xe311 => {
                w.item_x(2, 1);
                0xe313
            }
            0xe313 => {
                w.drain(8);
                let v = w.read(u32::from(0x67u16), width);
                w.load(v, width);
                0xe315
            }
            0xe315 => {
                w.mask(0, width);
                0xe317
            }
            0xe317 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xe31a
                } else {
                    0xe319
                }
            }
            0xe319 => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_add(1) & 255;
                w.flags(w.cpu.x, 1);
                0xe31a
            }
            0xe31a => {
                w.drain(8);
                let v = w.read(u32::from(0x57u16), width);
                w.load(v, width);
                0xe31c
            }
            0xe31c => {
                w.compare_immediate(1, width);
                0xe31e
            }
            0xe31e => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xe330
                } else {
                    0xe320
                }
            }
            0xe320 => {
                w.compare_immediate(16, width);
                0xe322
            }
            0xe322 => {
                let taken = w.branch(w.cpu.p & CARRY != 0);
                if taken {
                    0xe32c
                } else {
                    0xe324
                }
            }
            0xe324 => {
                w.drain(8 * u32::from(width));
                let v = 1;
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xe326
            }
            0xe326 => {
                w.drain(8);
                let address = u32::from(0x57u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe328
            }
            0xe328 => {
                w.drain(8 * u32::from(width));
                let v = 0;
                w.load(v, width);
                0xe32a
            }
            0xe32a => {
                w.branch(true);
                0xe333
            }
            0xe32c => {
                w.drain(8);
                let address = u32::from(0x57u16);
                w.write(address, 0, width);
                0xe32e
            }
            0xe32e => {
                w.drain(8);
                let address = u32::from(0x5eu16);
                w.write(address, 0, width);
                0xe330
            }
            0xe330 => {
                w.drain(16);
                if (0xe227u16 + w.cpu.x) & 0xff00 != 0xe200 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(0xe227u16 + w.cpu.x), width);
                w.load(v, width);
                0xe333
            }
            0xe333 => {
                w.clear_carry();
                0xe334
            }
            0xe334 => {
                w.drain(8);
                let v = w.read(u32::from(0x57u16), width);
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xe336
            }
            0xe336 => {
                w.drain(8);
                let address = u32::from(0xau16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe338
            }
            0xe338 => {
                w.drain(8);
                let address = u32::from(0xbu16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe33a
            }
            0xe33a => {
                w.drain(8 * u32::from(width));
                let v = 3;
                w.load(v, width);
                0xe33c
            }
            0xe33c => {
                w.drain(8);
                let address = u32::from(0xcu16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe33e
            }
            0xe33e => {
                w.drain(8 * u32::from(width));
                let v = 2;
                w.load(v, width);
                0xe340
            }
            0xe340 => {
                w.drain(8);
                let address = u32::from(0xdu16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe342
            }
            0xe342 => {
                w.item_x(1, 1);
                0xe344
            }
            0xe344 => {
                w.drain(8);
                let v = w.read(u32::from(0x67u16), width);
                w.load(v, width);
                0xe346
            }
            0xe346 => {
                w.drain(8);
                let v = w.read(u32::from(0xcu16), width);
                w.load((w.cpu.a & v) & if width == 1 { 255 } else { 65535 }, width);
                0xe348
            }
            0xe348 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xe359
                } else {
                    0xe34a
                }
            }
            0xe34a => {
                w.drain(8);
                let v = w.read(u32::from(0xdu16), width);
                w.load((w.cpu.a & v) & if width == 1 { 255 } else { 65535 }, width);
                0xe34c
            }
            0xe34c => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xe355
                } else {
                    0xe34e
                }
            }
            0xe34e => {
                w.drain(8);
                w.drain(6);
                let v = w.read(u32::from(0xau16 + w.cpu.x), width);
                w.load(v, width);
                0xe350
            }
            0xe350 => {
                w.drain(8 * u32::from(width));
                let v = 255;
                w.load((w.cpu.a ^ v) & if width == 1 { 255 } else { 65535 }, width);
                0xe352
            }
            0xe352 => {
                w.increment_a(width);
                0xe353
            }
            0xe353 => {
                w.drain(8);
                w.drain(6);
                let address = u32::from(0xau16 + w.cpu.x);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe355
            }
            0xe355 => {
                w.drain(8);
                w.drain(6);
                let v = w.read(u32::from(0xau16 + w.cpu.x), width);
                w.load(v, width);
                0xe357
            }
            0xe357 => {
                w.drain(8);
                w.drain(6);
                let address = u32::from(0x27u16 + w.cpu.x);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe359
            }
            0xe359 => {
                w.drain(8);
                let address = u32::from(0xcu16);
                let old = w.read(address, width);
                let mask = if width == 1 { 255 } else { 65535 };
                let v = (old << 1) & mask;
                w.cpu.p =
                    (w.cpu.p & !CARRY) | u8::from(old & if width == 1 { 128 } else { 32768 } != 0);
                w.flags(v, width);
                w.drain(6);
                w.write_high_low(address, v, width);
                0xe35b
            }
            0xe35b => {
                w.drain(8);
                let address = u32::from(0xcu16);
                let old = w.read(address, width);
                let mask = if width == 1 { 255 } else { 65535 };
                let v = (old << 1) & mask;
                w.cpu.p =
                    (w.cpu.p & !CARRY) | u8::from(old & if width == 1 { 128 } else { 32768 } != 0);
                w.flags(v, width);
                w.drain(6);
                w.write_high_low(address, v, width);
                0xe35d
            }
            0xe35d => {
                w.drain(8);
                let address = u32::from(0xdu16);
                let old = w.read(address, width);
                let mask = if width == 1 { 255 } else { 65535 };
                let v = (old << 1) & mask;
                w.cpu.p =
                    (w.cpu.p & !CARRY) | u8::from(old & if width == 1 { 128 } else { 32768 } != 0);
                w.flags(v, width);
                w.drain(6);
                w.write_high_low(address, v, width);
                0xe35f
            }
            0xe35f => {
                w.drain(8);
                let address = u32::from(0xdu16);
                let old = w.read(address, width);
                let mask = if width == 1 { 255 } else { 65535 };
                let v = (old << 1) & mask;
                w.cpu.p =
                    (w.cpu.p & !CARRY) | u8::from(old & if width == 1 { 128 } else { 32768 } != 0);
                w.flags(v, width);
                w.drain(6);
                w.write_high_low(address, v, width);
                0xe361
            }
            0xe361 => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_sub(1) & 255;
                w.flags(w.cpu.x, 1);
                0xe362
            }
            0xe362 => {
                let taken = w.branch(w.cpu.p & NEGATIVE == 0);
                if taken {
                    0xe344
                } else {
                    0xe364
                }
            }
            0xe364 => {
                w.drain(8 * u32::from(width));
                let v = 255;
                w.load(v, width);
                0xe366
            }
            0xe366 => {
                w.drain(8);
                let address = u32::from(0x29u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe368
            }
            0xe368 => {
                w.drain(8);
                let address = u32::from(0x24u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe36a
            }
            0xe36a => {
                w.drain(8);
                let address = u32::from(0x25u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe36c
            }
            0xe36c => {
                w.drain(8);
                let address = u32::from(0x2cu16);
                w.write(address, 0, width);
                0xe36e
            }
            0xe36e => {
                w.branch(true);
                0xe373
            }
            0xe373 => {
                w.drain(8);
                let v = w.read(u32::from(0x20u16), width);
                w.load(v, width);
                0xe375
            }
            0xe375 => {
                w.drain(8);
                let address = u32::from(0x0u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe377
            }
            0xe377 => {
                w.drain(8);
                let address = u32::from(0x3eu16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe379
            }
            0xe379 => {
                w.drain(8);
                let v = w.read(u32::from(0x22u16), width);
                w.load(v, width);
                0xe37b
            }
            0xe37b => {
                w.drain(8);
                let address = u32::from(0x1u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe37d
            }
            0xe37d => {
                w.drain(8);
                let address = u32::from(0x3fu16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe37f
            }
            0xe37f => {
                w.drain(8);
                let v = w.read(u32::from(0x21u16), width);
                w.load(v, width);
                0xe381
            }
            0xe381 => {
                w.drain(8);
                let address = u32::from(0x2u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe383
            }
            0xe383 => {
                w.drain(8);
                let address = u32::from(0x40u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe385
            }
            0xe385 => {
                w.drain(8);
                let v = w.read(u32::from(0x23u16), width);
                w.load(v, width);
                0xe387
            }
            0xe387 => {
                w.drain(8);
                let address = u32::from(0x3u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe389
            }
            0xe389 => {
                w.drain(8);
                let address = u32::from(0x41u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe38b
            }
            0xe38b => {
                w.drain(8);
                let v = w.read(u32::from(0x5du16), width);
                w.load(v, width);
                0xe38d
            }
            0xe38d => {
                w.compare_immediate(10, width);
                0xe38f
            }
            0xe38f => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xe398
                } else {
                    0xe391
                }
            }
            0xe391 => {
                w.drain(16);
                let v = w.read(w.absolute(0x2f5u16), width);
                w.load(v, width);
                0xe394
            }
            0xe394 => {
                w.compare_immediate(2, width);
                0xe396
            }
            0xe396 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xe3e0
                } else {
                    0xe398
                }
            }
            0xe398 => {
                w.item_y(2, 1);
                0xe39a
            }
            0xe39a => {
                w.item_x(4, 1);
                0xe39c
            }
            0xe39c => {
                w.drain(8);
                let v = w.read(u32::from(0x4du16), width);
                w.load(v, width);
                0xe39e
            }
            0xe39e => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xe3a4
                } else {
                    0xe3a0
                }
            }
            0xe3a0 => {
                w.item_y(1, 1);
                0xe3a2
            }
            0xe3a2 => {
                w.item_x(2, 1);
                0xe3a4
            }
            0xe3a4 => {
                w.drain(16);
                if (0x27u16 + w.cpu.y) & 0xff00 != 0x0 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(0x27u16 + w.cpu.y), width);
                w.load(v, width);
                0xe3a7
            }
            0xe3a7 => {
                w.shift_left(width);
                0xe3a8
            }
            0xe3a8 => {
                w.shift_left(width);
                0xe3a9
            }
            0xe3a9 => {
                w.shift_left(width);
                0xe3aa
            }
            0xe3aa => {
                w.shift_left(width);
                0xe3ab
            }
            0xe3ab => {
                w.clear_carry();
                0xe3ac
            }
            0xe3ac => {
                w.drain(16);
                if (0x2au16 + w.cpu.y) & 0xff00 != 0x0 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(0x2au16 + w.cpu.y), width);
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xe3af
            }
            0xe3af => {
                w.drain(16);
                w.drain(6);
                let address = w.absolute(0x2au16 + w.cpu.y);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe3b2
            }
            0xe3b2 => {
                w.drain(6);
                w.push(w.cpu.y, 1);
                0xe3b3
            }
            0xe3b3 => {
                w.drain(6);
                w.push(w.cpu.p as u16, 1);
                0xe3b4
            }
            0xe3b4 => {
                w.drain(16);
                if (0x27u16 + w.cpu.y) & 0xff00 != 0x0 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(0x27u16 + w.cpu.y), width);
                w.load(v, width);
                0xe3b7
            }
            0xe3b7 => {
                w.drain(6);
                let old = w.cpu.a & if width == 1 { 255 } else { 65535 };
                w.cpu.p = (w.cpu.p & !CARRY) | (old & 1) as u8;
                w.load(old >> 1, width);
                0xe3b8
            }
            0xe3b8 => {
                w.drain(6);
                let old = w.cpu.a & if width == 1 { 255 } else { 65535 };
                w.cpu.p = (w.cpu.p & !CARRY) | (old & 1) as u8;
                w.load(old >> 1, width);
                0xe3b9
            }
            0xe3b9 => {
                w.drain(6);
                let old = w.cpu.a & if width == 1 { 255 } else { 65535 };
                w.cpu.p = (w.cpu.p & !CARRY) | (old & 1) as u8;
                w.load(old >> 1, width);
                0xe3ba
            }
            0xe3ba => {
                w.drain(6);
                let old = w.cpu.a & if width == 1 { 255 } else { 65535 };
                w.cpu.p = (w.cpu.p & !CARRY) | (old & 1) as u8;
                w.load(old >> 1, width);
                0xe3bb
            }
            0xe3bb => {
                w.compare_immediate(8, width);
                0xe3bd
            }
            0xe3bd => {
                w.item_y(0, 1);
                0xe3bf
            }
            0xe3bf => {
                let taken = w.branch(w.cpu.p & CARRY == 0);
                if taken {
                    0xe3c5
                } else {
                    0xe3c1
                }
            }
            0xe3c1 => {
                w.drain(8 * u32::from(width));
                let v = 240;
                w.load((w.cpu.a | v) & if width == 1 { 255 } else { 65535 }, width);
                0xe3c3
            }
            0xe3c3 => {
                w.item_y(255, 1);
                0xe3c5
            }
            0xe3c5 => {
                w.drain(12);
                let v = w.pull(1);
                w.cpu.p = v as u8;
                if w.cpu.p & 0x10 != 0 {
                    w.cpu.x &= 255;
                    w.cpu.y &= 255;
                }
                0xe3c6
            }
            0xe3c6 => {
                w.drain(8);
                w.drain(6);
                let v = w.read(u32::from(0x20u16 + w.cpu.x), width);
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xe3c8
            }
            0xe3c8 => {
                w.drain(8);
                w.drain(6);
                let address = u32::from(0x20u16 + w.cpu.x);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe3ca
            }
            0xe3ca => {
                w.drain(6);
                w.load(w.cpu.y, width);
                0xe3cb
            }
            0xe3cb => {
                w.drain(8);
                w.drain(6);
                let v = w.read(u32::from(0x21u16 + w.cpu.x), width);
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xe3cd
            }
            0xe3cd => {
                w.drain(8);
                w.drain(6);
                let address = u32::from(0x21u16 + w.cpu.x);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe3cf
            }
            0xe3cf => {
                w.drain(12);
                let v = w.pull(1);
                w.cpu.y = v;
                w.flags(v, 1);
                0xe3d0
            }
            0xe3d0 => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_sub(1) & 255;
                w.flags(w.cpu.y, 1);
                0xe3d1
            }
            0xe3d1 => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_sub(1) & 255;
                w.flags(w.cpu.x, 1);
                0xe3d2
            }
            0xe3d2 => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_sub(1) & 255;
                w.flags(w.cpu.x, 1);
                0xe3d3
            }
            0xe3d3 => {
                let taken = w.branch(w.cpu.p & NEGATIVE == 0);
                if taken {
                    0xe3a4
                } else {
                    0xe3d5
                }
            }
            0xe3d5 => {
                w.item_call();
                0xe595
            }
            0xe3d8 => {
                w.item_call();
                0xe5f0
            }
            0xe3db => {
                w.branch(true);
                0xe3e0
            }
            0xe3dd => {
                w.drain(6);
                w.push(w.cpu.db as u16, 1);
                0xe3de
            }
            0xe3de => {
                w.drain(6);
                w.push(w.cpu.bank as u16, 1);
                0xe3df
            }
            0xe3df => {
                w.drain(12);
                let v = w.pull(1);
                w.cpu.db = v as u8;
                w.flags(v, 1);
                0xe3e0
            }
            0xe3e0 => {
                w.width(32, true);
                0xe3e2
            }
            0xe3e2 => {
                w.drain(8);
                let v = w.read(u32::from(0x20u16), width);
                w.load(v, width);
                0xe3e4
            }
            0xe3e4 => {
                w.clear_carry();
                0xe3e5
            }
            0xe3e5 => {
                w.drain(16);
                let v = w.read(w.absolute(0xb7eu16), width);
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xe3e8
            }
            0xe3e8 => {
                w.drain(8);
                let address = u32::from(0x20u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe3ea
            }
            0xe3ea => {
                w.drain(8);
                let v = w.read(u32::from(0x22u16), width);
                w.load(v, width);
                0xe3ec
            }
            0xe3ec => {
                w.clear_carry();
                0xe3ed
            }
            0xe3ed => {
                w.drain(16);
                let v = w.read(w.absolute(0xb7cu16), width);
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xe3f0
            }
            0xe3f0 => {
                w.drain(8);
                let address = u32::from(0x22u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe3f2
            }
            0xe3f2 => {
                w.width(32, false);
                0xe3f4
            }
            0xe3f4 => {
                w.drain(8);
                let v = w.read(u32::from(0x20u16), width);
                w.load(v, width);
                0xe3f6
            }
            0xe3f6 => {
                w.drain(6);
                w.cpu.p |= CARRY;
                0xe3f7
            }
            0xe3f7 => {
                w.drain(8);
                let v = w.read(u32::from(0x0u16), width);
                w.sbc(v, width);
                0xe3f9
            }
            0xe3f9 => {
                w.drain(8);
                let address = u32::from(0x30u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe3fb
            }
            0xe3fb => {
                w.drain(8);
                let v = w.read(u32::from(0x22u16), width);
                w.load(v, width);
                0xe3fd
            }
            0xe3fd => {
                w.drain(6);
                w.cpu.p |= CARRY;
                0xe3fe
            }
            0xe3fe => {
                w.drain(8);
                let v = w.read(u32::from(0x1u16), width);
                w.sbc(v, width);
                0xe400
            }
            0xe400 => {
                w.drain(8);
                let address = u32::from(0x31u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe402
            }
            0xe402 => {
                w.width(32, false);
                0xe404
            }
            0xe404 => {
                w.drain(12);
                let v = w.pull(1);
                w.cpu.db = v as u8;
                w.flags(v, 1);
                0xe405
            }
            0xe405 => {
                w.rtl();
                self.tail = Some(GroundVelocityTail::Complete);
                assert_eq!((w.cpu.bank, w.cpu.pc), (7, 0x82c2));
                w.cpu.pc
            }
            0xe545 => {
                w.drain(8);
                let address = u32::from(0x0u16);
                w.write(address, 0, width);
                0xe547
            }
            0xe547 => {
                w.drain(8);
                let address = u32::from(0x1u16);
                w.write(address, 0, width);
                0xe549
            }
            0xe549 => {
                w.drain(8);
                let v = w.read(u32::from(0xf0u16), width);
                w.load(v, width);
                0xe54b
            }
            0xe54b => {
                w.mask(15, width);
                0xe54d
            }
            0xe54d => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xe55e
                } else {
                    0xe54f
                }
            }
            0xe54f => {
                w.item_x(128, 1);
                0xe551
            }
            0xe551 => {
                w.drain(16);
                let v = w.read(w.absolute(0x351u16), width);
                w.load(v, width);
                0xe554
            }
            0xe554 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xe558
                } else {
                    0xe556
                }
            }
            0xe556 => {
                w.item_x(80, 1);
                0xe558
            }
            0xe558 => {
                w.drain(8);
                let address = u32::from(0x0u16);
                w.write(address, w.cpu.x, 1);
                0xe55a
            }
            0xe55a => {
                w.drain(8 * u32::from(width));
                let v = 1;
                w.load(v, width);
                0xe55c
            }
            0xe55c => {
                w.drain(8);
                let address = u32::from(0x1u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe55e
            }
            0xe55e => {
                w.drain(8);
                let address = u32::from(0x27u16);
                w.write(address, 0, width);
                0xe560
            }
            0xe560 => {
                w.drain(8);
                let address = u32::from(0x28u16);
                w.write(address, 0, width);
                0xe562
            }
            0xe562 => {
                w.drain(8);
                let address = u32::from(0x8u16);
                w.write(address, 0, width);
                0xe564
            }
            0xe564 => {
                w.drain(8);
                let address = u32::from(0x9u16);
                w.write(address, 0, width);
                0xe566
            }
            0xe566 => {
                w.drain(8);
                let address = u32::from(0xau16);
                w.write(address, 0, width);
                0xe568
            }
            0xe568 => {
                w.drain(8);
                let address = u32::from(0xbu16);
                w.write(address, 0, width);
                0xe56a
            }
            0xe56a => {
                w.item_x(3, 1);
                0xe56c
            }
            0xe56c => {
                w.drain(8);
                let v = w.read(u32::from(0x67u16), width);
                w.load(v, width);
                0xe56e
            }
            0xe56e => {
                w.drain(6);
                let old = w.cpu.a & if width == 1 { 255 } else { 65535 };
                w.cpu.p = (w.cpu.p & !CARRY) | (old & 1) as u8;
                w.load(old >> 1, width);
                0xe56f
            }
            0xe56f => {
                let taken = w.branch(w.cpu.p & CARRY != 0);
                if taken {
                    0xe576
                } else {
                    0xe571
                }
            }
            0xe571 => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_sub(1) & 255;
                w.flags(w.cpu.x, 1);
                0xe572
            }
            0xe572 => {
                let taken = w.branch(w.cpu.p & NEGATIVE == 0);
                if taken {
                    0xe56e
                } else {
                    0xe574
                }
            }
            0xe574 => {
                w.drain(12);
                let v = w.pull(1);
                w.cpu.db = v as u8;
                w.flags(v, 1);
                0xe575
            }
            0xe575 => {
                w.rtl();
                self.tail = Some(GroundVelocityTail::Complete);
                assert_eq!((w.cpu.bank, w.cpu.pc), (7, 0x82c2));
                w.cpu.pc
            }
            0xe576 => {
                w.drain(6);
                w.cpu.y = w.cpu.x & 255;
                w.flags(w.cpu.y, 1);
                0xe577
            }
            0xe577 => {
                w.width(32, true);
                0xe579
            }
            0xe579 => {
                w.drain(8);
                let v = w.read(u32::from(0x0u16), width);
                w.load(v, width);
                0xe57b
            }
            0xe57b => {
                w.drain(8);
                let old = w.cpu.y;
                w.cpu.p = (w.cpu.p & !CARRY) | CARRY;
                w.flags(old.wrapping_sub(0) & 255, 1);
                0xe57d
            }
            0xe57d => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xe583
                } else {
                    0xe57f
                }
            }
            0xe57f => {
                w.drain(8);
                let old = w.cpu.y;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(old >= 2);
                w.flags(old.wrapping_sub(2) & 255, 1);
                0xe581
            }
            0xe581 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xe587
                } else {
                    0xe583
                }
            }
            0xe583 => {
                w.drain(8 * u32::from(width));
                let v = 65535;
                w.load((w.cpu.a ^ v) & if width == 1 { 255 } else { 65535 }, width);
                0xe586
            }
            0xe586 => {
                w.increment_a(width);
                0xe587
            }
            0xe587 => {
                w.drain(6);
                w.push(w.cpu.a, width);
                0xe588
            }
            0xe588 => {
                w.drain(6);
                w.load(w.cpu.y, width);
                0xe589
            }
            0xe589 => {
                w.mask(2, width);
                0xe58c
            }
            0xe58c => {
                w.drain(6);
                w.cpu.x = w.cpu.a & 255;
                w.flags(w.cpu.x, 1);
                0xe58d
            }
            0xe58d => {
                w.drain(12);
                let v = w.pull(width);
                w.load(v, width);
                0xe58e
            }
            0xe58e => {
                w.drain(8);
                w.drain(6);
                let address = u32::from(0x8u16 + w.cpu.x);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                self.subpixels[usize::from(w.cpu.x / 2)] = w.cpu.a;
                0xe590
            }
            0xe590 => {
                w.width(32, false);
                0xe592
            }
            0xe592 => {
                w.operand_then_internal(2);
                self.tail = Some(GroundVelocityTail::Subpixels {
                    x: self.subpixels[1],
                    y: self.subpixels[0],
                });
                0xe4d3
            }
            0xe595 => {
                w.drain(16);
                let v = w.read(w.absolute(0x46cu16), width);
                w.load(v, width);
                0xe598
            }
            0xe598 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xe5e3
                } else {
                    0xe59a
                }
            }
            0xe59a => {
                w.drain(8);
                let v = w.read(u32::from(0x24u16), width);
                w.load(v, width);
                0xe59c
            }
            0xe59c => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xe5a2
                } else {
                    0xe59e
                }
            }
            0xe59e => {
                w.compare_immediate(255, width);
                0xe5a0
            }
            0xe5a0 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xe5e3
                } else {
                    0xe5a2
                }
            }
            0xe5a2 => {
                w.drain(16);
                let v = w.read(w.absolute(0x322u16), width);
                w.load(v, width);
                0xe5a5
            }
            0xe5a5 => {
                w.mask(3, width);
                0xe5a7
            }
            0xe5a7 => {
                w.compare_immediate(3, width);
                0xe5a9
            }
            0xe5a9 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xe5e3
                } else {
                    0xe5ab
                }
            }
            0xe5ab => {
                w.drain(8);
                let v = w.read(u32::from(0x5du16), width);
                w.load(v, width);
                0xe5ad
            }
            0xe5ad => {
                w.compare_immediate(19, width);
                0xe5af
            }
            0xe5af => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xe5e3
                } else {
                    0xe5b1
                }
            }
            0xe5b1 => {
                w.item_y(8, 1);
                0xe5b3
            }
            0xe5b3 => {
                w.drain(16);
                let v = w.read(w.absolute(0x310u16), width);
                w.load(v, width);
                0xe5b6
            }
            0xe5b6 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xe5bf
                } else {
                    0xe5b8
                }
            }
            0xe5b8 => {
                let taken = w.branch(w.cpu.p & NEGATIVE != 0);
                if taken {
                    0xe5bc
                } else {
                    0xe5ba
                }
            }
            0xe5ba => {
                w.item_y(4, 1);
                0xe5bc
            }
            0xe5bc => {
                w.drain(6);
                w.load(w.cpu.y, width);
                0xe5bd
            }
            0xe5bd => {
                w.drain(8);
                let address = u32::from(0x67u16);
                let old = w.read(address, width);
                w.cpu.p = (w.cpu.p & !ZERO) | if old & w.cpu.a == 0 { ZERO } else { 0 };
                w.drain(6);
                w.write_high_low(
                    address,
                    old | (w.cpu.a & if width == 1 { 255 } else { 65535 }),
                    width,
                );
                0xe5bf
            }
            0xe5bf => {
                w.item_y(2, 1);
                0xe5c1
            }
            0xe5c1 => {
                w.drain(16);
                let v = w.read(w.absolute(0x312u16), width);
                w.load(v, width);
                0xe5c4
            }
            0xe5c4 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xe5cd
                } else {
                    0xe5c6
                }
            }
            0xe5c6 => {
                let taken = w.branch(w.cpu.p & NEGATIVE != 0);
                if taken {
                    0xe5ca
                } else {
                    0xe5c8
                }
            }
            0xe5c8 => {
                w.item_y(1, 1);
                0xe5ca
            }
            0xe5ca => {
                w.drain(6);
                w.load(w.cpu.y, width);
                0xe5cb
            }
            0xe5cb => {
                w.drain(8);
                let address = u32::from(0x67u16);
                let old = w.read(address, width);
                w.cpu.p = (w.cpu.p & !ZERO) | if old & w.cpu.a == 0 { ZERO } else { 0 };
                w.drain(6);
                w.write_high_low(
                    address,
                    old | (w.cpu.a & if width == 1 { 255 } else { 65535 }),
                    width,
                );
                0xe5cd
            }
            0xe5cd => {
                w.drain(8);
                let address = u32::from(0x6au16);
                w.write(address, 0, width);
                0xe5cf
            }
            0xe5cf => {
                w.width(32, true);
                0xe5d1
            }
            0xe5d1 => {
                w.drain(8);
                let v = w.read(u32::from(0x20u16), width);
                w.load(v, width);
                0xe5d3
            }
            0xe5d3 => {
                w.clear_carry();
                0xe5d4
            }
            0xe5d4 => {
                w.drain(16);
                let v = w.read(w.absolute(0x310u16), width);
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xe5d7
            }
            0xe5d7 => {
                w.drain(8);
                let address = u32::from(0x20u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe5d9
            }
            0xe5d9 => {
                w.drain(8);
                let v = w.read(u32::from(0x22u16), width);
                w.load(v, width);
                0xe5db
            }
            0xe5db => {
                w.clear_carry();
                0xe5dc
            }
            0xe5dc => {
                w.drain(16);
                let v = w.read(w.absolute(0x312u16), width);
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xe5df
            }
            0xe5df => {
                w.drain(8);
                let address = u32::from(0x22u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe5e1
            }
            0xe5e1 => {
                w.width(32, false);
                0xe5e3
            }
            0xe5e3 => {
                w.rts();
                w.cpu.pc
            }
            0xe5f0 => {
                w.drain(16);
                let v = w.read(w.absolute(0x3f3u16), width);
                w.load(v, width);
                0xe5f3
            }
            0xe5f3 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xe5e3
                } else {
                    0xe5f5
                }
            }
            0xe5f5 => {
                w.drain(8);
                let v = w.read(u32::from(0x24u16), width);
                w.load(v, width);
                0xe5f7
            }
            0xe5f7 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xe5fd
                } else {
                    0xe5f9
                }
            }
            0xe5f9 => {
                w.compare_immediate(255, width);
                0xe5fb
            }
            0xe5fb => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xe5e3
                } else {
                    0xe5fd
                }
            }
            0xe5fd => {
                w.drain(16);
                let v = w.read(w.absolute(0x376u16), width);
                w.load(v, width);
                0xe600
            }
            0xe600 => {
                w.mask(1, width);
                0xe602
            }
            0xe602 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xe5e3
                } else {
                    0xe604
                }
            }
            0xe604 => {
                w.drain(8);
                let v = w.read(u32::from(0x5du16), width);
                w.load(v, width);
                0xe606
            }
            0xe606 => {
                w.compare_immediate(19, width);
                0xe608
            }
            0xe608 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xe5e3
                } else {
                    0xe60a
                }
            }
            0xe60a => {
                w.drain(8);
                let v = w.read(u32::from(0x4du16), width);
                w.load(v, width);
                0xe60c
            }
            0xe60c => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xe5e3
                } else {
                    0xe60e
                }
            }
            0xe60e => {
                w.drain(16);
                let v = w.read(w.absolute(0x372u16), width);
                w.load(v, width);
                0xe611
            }
            0xe611 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xe625
                } else {
                    0xe613
                }
            }
            0xe613 => {
                w.drain(16);
                let v = w.read(w.absolute(0x2f1u16), width);
                w.load(v, width);
                0xe616
            }
            0xe616 => {
                w.compare_immediate(32, width);
                0xe618
            }
            0xe618 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xe625
                } else {
                    0xe61a
                }
            }
            0xe61a => {
                w.drain(16);
                let v = w.read(w.absolute(0x3f3u16), 1);
                w.cpu.y = v;
                w.flags(v, 1);
                0xe61d
            }
            0xe61d => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_sub(1) & 255;
                w.flags(w.cpu.y, 1);
                0xe61e
            }
            0xe61e => {
                w.drain(16);
                if (0xe5e4u16 + w.cpu.y) & 0xff00 != 0xe500 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(0xe5e4u16 + w.cpu.y), width);
                w.load(v, width);
                0xe621
            }
            0xe621 => {
                w.drain(8);
                let v = w.read(u32::from(0x67u16), width);
                w.load((w.cpu.a & v) & if width == 1 { 255 } else { 65535 }, width);
                0xe623
            }
            0xe623 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xe5e3
                } else {
                    0xe625
                }
            }
            0xe625 => {
                w.drain(8);
                let address = u32::from(0x6au16);
                w.write(address, 0, width);
                0xe627
            }
            0xe627 => {
                w.drain(16);
                let v = w.read(w.absolute(0x3f3u16), 1);
                w.cpu.y = v;
                w.flags(v, 1);
                0xe62a
            }
            0xe62a => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_sub(1) & 255;
                w.flags(w.cpu.y, 1);
                0xe62b
            }
            0xe62b => {
                w.drain(16);
                if (0xe5e4u16 + w.cpu.y) & 0xff00 != 0xe500 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(0xe5e4u16 + w.cpu.y), width);
                w.load(v, width);
                0xe62e
            }
            0xe62e => {
                w.drain(8);
                let address = u32::from(0x67u16);
                let old = w.read(address, width);
                w.cpu.p = (w.cpu.p & !ZERO) | if old & w.cpu.a == 0 { ZERO } else { 0 };
                w.drain(6);
                w.write_high_low(
                    address,
                    old | (w.cpu.a & if width == 1 { 255 } else { 65535 }),
                    width,
                );
                0xe630
            }
            0xe630 => {
                w.drain(16);
                if (0xe5e8u16 + w.cpu.y) & 0xff00 != 0xe500 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(0xe5e8u16 + w.cpu.y), width);
                w.load(v, width);
                0xe633
            }
            0xe633 => {
                w.drain(8);
                let address = u32::from(0x72u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe635
            }
            0xe635 => {
                w.drain(16);
                if (0xe5ecu16 + w.cpu.y) & 0xff00 != 0xe500 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(0xe5ecu16 + w.cpu.y), width);
                w.load(v, width);
                0xe638
            }
            0xe638 => {
                w.drain(8);
                let address = u32::from(0x73u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe63a
            }
            0xe63a => {
                w.item_x(1, 1);
                0xe63c
            }
            0xe63c => {
                w.item_y(2, 1);
                0xe63e
            }
            0xe63e => {
                w.drain(6);
                w.push(w.cpu.x, 1);
                0xe63f
            }
            0xe63f => {
                w.drain(8);
                w.drain(6);
                let v = w.read(u32::from(0x72u16 + w.cpu.x), width);
                w.load(v, width);
                0xe641
            }
            0xe641 => {
                w.shift_left(width);
                0xe642
            }
            0xe642 => {
                w.shift_left(width);
                0xe643
            }
            0xe643 => {
                w.shift_left(width);
                0xe644
            }
            0xe644 => {
                w.shift_left(width);
                0xe645
            }
            0xe645 => {
                w.clear_carry();
                0xe646
            }
            0xe646 => {
                w.drain(16);
                if (0x41cu16 + w.cpu.x) & 0xff00 != 0x400 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(0x41cu16 + w.cpu.x), width);
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xe649
            }
            0xe649 => {
                w.drain(16);
                w.drain(6);
                let address = w.absolute(0x41cu16 + w.cpu.x);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe64c
            }
            0xe64c => {
                w.drain(8);
                w.drain(6);
                let v = w.read(u32::from(0x72u16 + w.cpu.x), width);
                w.load(v, width);
                0xe64e
            }
            0xe64e => {
                w.drain(6);
                w.push(w.cpu.p as u16, 1);
                0xe64f
            }
            0xe64f => {
                w.item_x(0, 1);
                0xe651
            }
            0xe651 => {
                w.drain(6);
                let old = w.cpu.a & if width == 1 { 255 } else { 65535 };
                w.cpu.p = (w.cpu.p & !CARRY) | (old & 1) as u8;
                w.load(old >> 1, width);
                0xe652
            }
            0xe652 => {
                w.drain(6);
                let old = w.cpu.a & if width == 1 { 255 } else { 65535 };
                w.cpu.p = (w.cpu.p & !CARRY) | (old & 1) as u8;
                w.load(old >> 1, width);
                0xe653
            }
            0xe653 => {
                w.drain(6);
                let old = w.cpu.a & if width == 1 { 255 } else { 65535 };
                w.cpu.p = (w.cpu.p & !CARRY) | (old & 1) as u8;
                w.load(old >> 1, width);
                0xe654
            }
            0xe654 => {
                w.drain(6);
                let old = w.cpu.a & if width == 1 { 255 } else { 65535 };
                w.cpu.p = (w.cpu.p & !CARRY) | (old & 1) as u8;
                w.load(old >> 1, width);
                0xe655
            }
            0xe655 => {
                w.drain(12);
                let v = w.pull(1);
                w.cpu.p = v as u8;
                if w.cpu.p & 0x10 != 0 {
                    w.cpu.x &= 255;
                    w.cpu.y &= 255;
                }
                0xe656
            }
            0xe656 => {
                let taken = w.branch(w.cpu.p & NEGATIVE == 0);
                if taken {
                    0xe65b
                } else {
                    0xe658
                }
            }
            0xe658 => {
                w.drain(8 * u32::from(width));
                let v = 240;
                w.load((w.cpu.a | v) & if width == 1 { 255 } else { 65535 }, width);
                0xe65a
            }
            0xe65a => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_sub(1) & 255;
                w.flags(w.cpu.x, 1);
                0xe65b
            }
            0xe65b => {
                w.drain(16);
                if (0x20u16 + w.cpu.y) & 0xff00 != 0x0 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(0x20u16 + w.cpu.y), width);
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xe65e
            }
            0xe65e => {
                w.drain(16);
                w.drain(6);
                let address = w.absolute(0x20u16 + w.cpu.y);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe661
            }
            0xe661 => {
                w.drain(6);
                w.load(w.cpu.x, width);
                0xe662
            }
            0xe662 => {
                w.drain(16);
                if (0x21u16 + w.cpu.y) & 0xff00 != 0x0 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(0x21u16 + w.cpu.y), width);
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xe665
            }
            0xe665 => {
                w.drain(16);
                w.drain(6);
                let address = w.absolute(0x21u16 + w.cpu.y);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xe668
            }
            0xe668 => {
                w.drain(12);
                let v = w.pull(1);
                w.cpu.x = v;
                w.flags(v, 1);
                0xe669
            }
            0xe669 => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_sub(1) & 255;
                w.flags(w.cpu.y, 1);
                0xe66a
            }
            0xe66a => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_sub(1) & 255;
                w.flags(w.cpu.y, 1);
                0xe66b
            }
            0xe66b => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_sub(1) & 255;
                w.flags(w.cpu.x, 1);
                0xe66c
            }
            0xe66c => {
                let taken = w.branch(w.cpu.p & NEGATIVE == 0);
                if taken {
                    0xe63e
                } else {
                    0xe66e
                }
            }
            0xe66e => {
                w.width(32, false);
                0xe670
            }
            0xe670 => {
                w.rts();
                w.cpu.pc
            }
            _ => panic!("ground velocity PC {pc:04x}"),
        };
        w.cpu.pc = next;
        self.pc = Some(next);
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
    fn ground_velocity_matches_source_selection_position_floor_and_conveyor() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for (start, end) in [(0x07e227, 0x07e245), (0x07e5e4, 0x07e5f0)] {
            for address in start..end {
                assert_eq!(
                    table_byte(address),
                    Some(rom[7 * 0x8000 + (address as usize & 0x7fff)])
                );
            }
        }
        let mut instructions = 0;

        for timer in 0..=255u8 {
            for variant in 0..32u8 {
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                let r = &mut machine.ram;
                for (i, byte) in r.iter_mut().enumerate() {
                    *byte = (i ^ (i >> 7) ^ timer as usize) as u8;
                }
                r[0x1f5..0x1f7].copy_from_slice(&[0x1b, 0x80]);
                r[0x10] = 7;
                r[0x11] = 0;
                r[0xb7b] = 0;
                r[0x5d] = 0;
                r[0x34a] = 0;
                r[0x372] = 0;
                r[0x316] = 0;
                r[0x317] = 0;
                match variant {
                    0 => r[0xb7b] = 1,
                    1 => {
                        r[0x10] = 14;
                        r[0x11] = 2;
                    }
                    2 => r[0x5d] = 4,
                    3 => r[0x34a] = 1,
                    4 => {
                        r[0x372] = 1;
                        r[0x2f1] = 15;
                    }
                    5 => {
                        r[0x316] = 7;
                        r[0x317] = 8;
                    }
                    _ => {
                        r[0x372] = u8::from(variant >= 8 && timer & 128 != 0);
                        r[0x2f1] = 32 + (timer & 31);
                        r[0x5d] = [0, 10, 19][(variant as usize) % 3];
                    }
                }
                r[0x5e] = (timer % 15) * 2;
                r[0x351] = u8::from(timer & 32 != 0);
                r[0x5b] = [0, 1, 2, 3][(variant & 3) as usize];
                r[0x57] = [0, 1, 2, 15, 16, 32, 48, 255][(timer & 7) as usize];
                r[0x67] = timer & 15;
                r[0xf0] = timer & 15;
                r[0x2f5] = if timer & 16 != 0 { 2 } else { 0 };
                r[0x4d] = u8::from(variant & 8 != 0);
                let values = [0u16, 0xffff, 0x7fff, 0x8000, 0xff00, 0xff, 0x5555, 0xaaaa];
                r[0x20..0x22].copy_from_slice(&values[((timer >> 3) & 7) as usize].to_le_bytes());
                r[0x22..0x24].copy_from_slice(&values[(timer & 7) as usize].to_le_bytes());
                r[0x24..0x26]
                    .copy_from_slice(&[0u16, 0xffff, 1, 255][(variant & 3) as usize].to_le_bytes());
                r[0x46c] = [0, 1, 2, 4][(timer >> 6) as usize];
                r[0x322] = timer & 3;
                r[0x310..0x312].copy_from_slice(&values[(timer & 7) as usize].to_le_bytes());
                r[0x312..0x314].copy_from_slice(&values[((timer >> 3) & 7) as usize].to_le_bytes());
                r[0x3f3] = timer % 5;
                r[0x376] = (timer >> 5) & 1;
                let pc = 0x82be;
                let mut cpu = NativeNmiContext {
                    a: 0x2400,
                    x: 16,
                    y: variant as u16,
                    d: 0,
                    db: variant,
                    bank: 7,
                    pc,
                    sp: 0x1f4,
                    p: 0x30 | (timer & 0xc3),
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
                        if variant & 1 == 0 { 22 } else { 261 },
                        if variant & 1 == 0 { 26 } else { 1350 },
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
                let mut work = GroundVelocity::default();
                while work.tail().is_none() {
                    let label = format!(
                        "velocity controls={timer} variant={variant} pc={:04x}",
                        cpu.pc
                    );
                    super::super::native_work::compare_instruction_with_devices(
                        &label,
                        &mut cpu,
                        &mut clock,
                        &mut source,
                        &mut ram,
                        &rom,
                        None,
                        |_, access| {
                            if let super::super::intro_clear::Access::Read(a, width) = access {
                                let low = table_byte(a)?;
                                Some(if width == 1 {
                                    u16::from(low)
                                } else {
                                    u16::from_le_bytes([low, table_byte(a + 1)?])
                                })
                            } else {
                                None
                            }
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
                    GroundVelocityTail::Complete => assert_eq!(
                        (cpu.bank, cpu.pc, cpu.sp, cpu.db),
                        (7, 0x82c2, 0x1f4, variant)
                    ),
                    GroundVelocityTail::Swim => {
                        assert_eq!((cpu.bank, cpu.pc, cpu.sp), (7, 0xe42a, 0x1f0))
                    }
                    GroundVelocityTail::Subpixels { x, y } => {
                        assert_eq!((cpu.bank, cpu.pc, cpu.sp), (7, 0xe4d3, 0x1f0));
                        assert_eq!(x, u16::from_le_bytes([ram[0xa], ram[0xb]]));
                        assert_eq!(y, u16::from_le_bytes([ram[8], ram[9]]));
                    }
                }
            }
        }

        println!("ground velocity: 8192 cases, {instructions} source instructions, speed selection, axis carry, floor/conveyor, bank/stack and special tails");
    }
}
