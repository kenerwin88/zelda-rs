//! Link OAM's live pose selection and initial sprite-bank publications.
//! The equipment suffix is an explicit tail handoff, with the original PHB,
//! optional stair Y backup and caller return still on the native stack.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    crate::ZeldaState::native_link_oam_table_byte(address)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LinkOamChild {
    Equipment,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LinkOam {
    pc: u16,
    near_returns: Vec<u16>,
    complete: bool,
}
impl Default for LinkOam {
    fn default() -> Self {
        Self {
            pc: 0xa18e,
            near_returns: Vec::new(),
            complete: false,
        }
    }
}
impl LinkOam {
    pub(crate) fn child(&self) -> Option<LinkOamChild> {
        (self.pc == 0xa5ce && !self.complete).then_some(LinkOamChild::Equipment)
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    /// The remaining tail must consume the real PLB/RTL before the parent can
    /// resume. No synthetic call/return is introduced at the linear boundary.
    pub(crate) fn return_from_child(&mut self, cpu: &NativeNmiContext) {
        assert_eq!(self.child(), Some(LinkOamChild::Equipment));
        assert!(self.near_returns.is_empty());
        self.pc = cpu.pc;
        self.complete = true;
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
        assert!(self.child().is_none(), "Link OAM tail still owns the CPU");
        assert_eq!((cpu.bank, cpu.pc, cpu.d, cpu.p & 8), (13, self.pc, 0, 0));
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
        w.fetch();
        self.pc = match pc {
            0xa18e | 0xa18f | 0xa1a3 | 0xa1a6 => {
                w.drain(6);
                w.push(
                    match pc {
                        0xa18e => u16::from(w.cpu.db),
                        0xa18f => 13,
                        _ => w.cpu.a & mask,
                    },
                    1,
                );
                pc + 1
            }
            0xa190 => {
                w.drain(12);
                w.cpu.db = w.pull(1) as u8;
                w.flags(u16::from(w.cpu.db), 1);
                pc + 1
            }
            0xa191 | 0xa199 | 0xa1a7 | 0xa1b0 | 0xa1be | 0xa1df | 0xa1e5 | 0xa1ec | 0xa215
            | 0xa228 | 0xa239 | 0xa249 | 0xa250 | 0xa25e | 0xa275 | 0xa27e | 0xa289 | 0xa298
            | 0xa2b0 | 0xa2bc | 0xa2dc | 0xa2eb | 0xa2f7 | 0xa2fd | 0xa30d | 0xa31d | 0xa33b
            | 0xa34e | 0xa355 | 0xa387 | 0xa398 | 0xa3b3 | 0xa3cc | 0xa3fd | 0xa409 | 0xa41e
            | 0xa422 | 0xa433 | 0xa478 | 0xa48c | 0xa4ed | 0xa57a | 0xaac3 => {
                let v = match pc {
                    0xa199 | 0xa289 => 24,
                    0xa1b0 => 12,
                    0xa1df => 0x80,
                    0xa1ec => 1,
                    0xa215 => 31,
                    0xa228 => 36,
                    0xa239 => 33,
                    0xa250 | 0xa25e => 10,
                    0xa275 => 25,
                    0xa27e => 26,
                    0xa298 => 22,
                    0xa2b0 => 0x20,
                    0xa2bc => 17,
                    0xa2dc | 0xa31d => 19,
                    0xa2eb => 18,
                    0xa2f7 => 30,
                    0xa2fd => 29,
                    0xa30d => 27,
                    0xa33b => 0x30,
                    0xa34e => 5,
                    0xa355 => 20,
                    0xa387 => 4,
                    0xa398 | 0xa41e => 2,
                    0xa3b3 => 16,
                    0xa3cc => 23,
                    0xa3fd => 21,
                    0xa409 => 15,
                    0xa422 => 39,
                    0xa433 => 3,
                    0xa478 => 0xe00,
                    0xa48c => 12,
                    0xaac3 => 7,
                    _ => 0,
                };
                if matches!(
                    pc,
                    0xa1be | 0xa1df | 0xa2b0 | 0xa33b | 0xa398 | 0xa478 | 0xa4ed | 0xa57a
                ) {
                    w.immediate(v, m);
                    pc + 1 + u16::from(m)
                } else {
                    w.drain(8 * u32::from(x));
                    if matches!(pc, 0xa1e5 | 0xa1ec | 0xa48c | 0xaac3) {
                        w.cpu.x = v;
                    } else {
                        w.cpu.y = v;
                    }
                    w.flags(v, x);
                    pc + 1 + u16::from(x)
                }
            }
            0xa193 | 0xa1a1 | 0xa1a4 | 0xa1b8 | 0xa1d1 | 0xa1d8 | 0xa1f6 | 0xa20f | 0xa22c
            | 0xa23b | 0xa241 | 0xa252 | 0xa258 | 0xa260 | 0xa264 | 0xa26e | 0xa292 | 0xa29a
            | 0xa2a2 | 0xa2a6 | 0xa2b6 | 0xa2be | 0xa2c4 | 0xa2c8 | 0xa311 | 0xa31f | 0xa330
            | 0xa344 | 0xa34a | 0xa361 | 0xa379 | 0xa381 | 0xa3ef | 0xa412 | 0xa418 | 0xa424
            | 0xa42c | 0xa43c | 0xa441 | 0xa448 | 0xa44e | 0xa4a9 | 0xa4b4 | 0xa4c8 | 0xa4cf
            | 0xa4db | 0xa4df | 0xa4e3 | 0xa4e7 | 0xa518 | 0xa541 | 0xa555 | 0xa55c | 0xa568
            | 0xa56c | 0xa570 | 0xa574 | 0xa5a5 => {
                let a = match pc {
                    0xa193 | 0xa252 | 0xa2c4 => 0x11,
                    0xa1a1 | 0xa1d1 => 0x20,
                    0xa1a4 => 0x21,
                    0xa1d8 => 0x22,
                    0xa1b8 | 0xa23b | 0xa26e | 0xa29a | 0xa2a2 => 0x2e,
                    0xa1f6 => 0xee,
                    0xa20f | 0xa2b6 | 0xa330 | 0xa344 | 0xa3ef => 0x5d,
                    0xa22c | 0xa241 | 0xa264 | 0xa2a6 | 0xa44e => 0x2f,
                    0xa258 => 0x10,
                    0xa260 => 0x28,
                    0xa292 => 0x48,
                    0xa2be | 0xa42c | 0xa448 => 2,
                    0xa2c8 => 0xf0,
                    0xa311 => 0x4d,
                    0xa31f => 0x1a,
                    0xa34a => 0x55,
                    0xa361 => 0x5b,
                    0xa379 => 0x5a,
                    0xa381 | 0xa441 => 0x65,
                    0xa412 => 0x3a,
                    0xa418 | 0xa424 => 0x3c,
                    0xa43c => 0x64,
                    0xa4a9 => 0x76,
                    0xa4b4 | 0xa4db | 0xa541 | 0xa568 => 0x74,
                    0xa4c8 | 0xa555 => 0x72,
                    0xa4cf | 0xa55c => 4,
                    0xa4df | 0xa56c => 0x25,
                    0xa4e3 | 0xa4e7 | 0xa570 | 0xa574 => 0x24,
                    _ => 6,
                };
                let index = matches!(
                    pc,
                    0xa264
                        | 0xa26e
                        | 0xa4b4
                        | 0xa4c8
                        | 0xa4cf
                        | 0xa4db
                        | 0xa541
                        | 0xa555
                        | 0xa55c
                        | 0xa568
                );
                let v = w.dp_read(a, if index { x } else { m });
                if index {
                    if matches!(pc, 0xa264 | 0xa26e | 0xa4c8 | 0xa555) {
                        w.cpu.x = v;
                    } else {
                        w.cpu.y = v;
                    }
                    w.flags(v, x);
                } else {
                    w.load(v, m);
                }
                pc + 2
            }
            0xa195 | 0xa19b | 0xa1ba | 0xa211 | 0xa21a | 0xa254 | 0xa25a | 0xa29c | 0xa2b8
            | 0xa2f9 | 0xa315 | 0xa319 | 0xa332 | 0xa346 | 0xa365 | 0xa369 | 0xa37d | 0xa3f1
            | 0xa3f5 | 0xa3f9 | 0xa401 | 0xa405 | 0xa41a | 0xa428 | 0xa4bc | 0xa4e9 | 0xa549
            | 0xa576 => {
                w.compare_immediate(
                    match pc {
                        0xa195 => 18,
                        0xa19b | 0xa346 => 19,
                        0xa1ba | 0xa37d => 6,
                        0xa211 => 22,
                        0xa21a | 0xa2f9 => 2,
                        0xa254 => 14,
                        0xa25a => 18,
                        0xa29c | 0xa332 => 5,
                        0xa2b8 | 0xa319 => 4,
                        0xa315 | 0xa365 => 1,
                        0xa369 | 0xa405 => 3,
                        0xa3f1 => 10,
                        0xa3f5 => 8,
                        0xa3f9 | 0xa41a | 0xa428 => 9,
                        0xa401 => 30,
                        0xa4bc | 0xa549 => 255,
                        _ => 0xf0,
                    },
                    m,
                );
                pc + 1 + u16::from(m)
            }
            0xa197 | 0xa19d | 0xa1ae | 0xa1bc | 0xa1ea | 0xa213 | 0xa21c | 0xa226 | 0xa237
            | 0xa24e | 0xa256 | 0xa25c | 0xa262 | 0xa268 | 0xa26c | 0xa27c | 0xa280 | 0xa287
            | 0xa290 | 0xa296 | 0xa29e | 0xa2ae | 0xa2ba | 0xa2c6 | 0xa2cc | 0xa2da | 0xa2e6
            | 0xa2f3 | 0xa2fb | 0xa2ff | 0xa306 | 0xa30f | 0xa313 | 0xa317 | 0xa31b | 0xa334
            | 0xa339 | 0xa348 | 0xa34c | 0xa353 | 0xa363 | 0xa367 | 0xa36b | 0xa37f | 0xa38f
            | 0xa396 | 0xa3a2 | 0xa3a9 | 0xa3b1 | 0xa3b5 | 0xa3bc | 0xa3c4 | 0xa3c9 | 0xa3ce
            | 0xa3d3 | 0xa3db | 0xa3e0 | 0xa3ed | 0xa3f3 | 0xa3f7 | 0xa3fb | 0xa3ff | 0xa403
            | 0xa407 | 0xa410 | 0xa416 | 0xa41c | 0xa420 | 0xa42a | 0xa43a | 0xa481 | 0xa498
            | 0xa49c | 0xa4bf | 0xa4e1 | 0xa4e5 | 0xa4eb | 0xa51d | 0xa54c | 0xa56e | 0xa572
            | 0xa578 | 0xa5aa | 0xaac6 | 0xaac9 => {
                let z = w.cpu.p & ZERO != 0;
                let c = w.cpu.p & CARRY != 0;
                let n = w.cpu.p & NEGATIVE != 0;
                let (take, target) = match pc {
                    0xa197 => (z, 0xa19f),
                    0xa19d => (!z, 0xa1d1),
                    0xa1ae => (z, 0xa1b2),
                    0xa1bc => (!c, 0xa1c0),
                    0xa1ea => (z, 0xa1ee),
                    0xa213 => (!z, 0xa223),
                    0xa21c => (z, 0xa223),
                    0xa226 => (z, 0xa234),
                    0xa237 => (z, 0xa249),
                    0xa24e => (z, 0xa252),
                    0xa256 => (!z, 0xa282),
                    0xa25c | 0xa262 => (z, 0xa282),
                    0xa268 | 0xa26c => (z, 0xa2a2),
                    0xa27c => (z, 0xa2a6),
                    0xa280 | 0xa290 => (true, 0xa2a6),
                    0xa287 => (z, 0xa292),
                    0xa296 => (z, 0xa2a2),
                    0xa29e => (!c, 0xa2a2),
                    0xa2ae => (z, 0xa2b6),
                    0xa2ba => (!z, 0xa2f0),
                    0xa2c6 => (!z, 0xa2ce),
                    0xa2cc => (!z, 0xa2dc),
                    0xa2da => (z, 0xa2e3),
                    0xa2e6 => (z, 0xa2ed),
                    0xa2f3 => (z, 0xa301),
                    0xa2fb => (z, 0xa2ff),
                    0xa2ff | 0xa30f => (true, 0xa2ed),
                    0xa306 => (z, 0xa311),
                    0xa313 => (z, 0xa361),
                    0xa317 => (z, 0xa330),
                    0xa31b => (!z, 0xa361),
                    0xa334 => (!z, 0xa344),
                    0xa339 => (!z, 0xa341),
                    0xa348 => (z, 0xa361),
                    0xa34c => (!z, 0xa361),
                    0xa353 => (z, 0xa35c),
                    0xa363 | 0xa367 => (z, 0xa38c),
                    0xa36b => (!z, 0xa379),
                    0xa37f => (!c, 0xa387),
                    0xa38f => (z, 0xa3c6),
                    0xa396 => (c, 0xa39d),
                    0xa3a2 => (!c, 0xa3c1),
                    0xa3a9 => (z, 0xa3ac),
                    0xa3b1 => (z, 0xa3b7),
                    0xa3b5 => (true, 0xa3c1),
                    0xa3bc => (z, 0xa3c1),
                    0xa3c4 | 0xa3ce => (true, 0xa3eb),
                    0xa3c9 => (z, 0xa3d0),
                    0xa3d3 => (z, 0xa3dd),
                    0xa3db => (true, 0xa3e8),
                    0xa3e0 => (z, 0xa3ef),
                    0xa3ed | 0xa410 | 0xa420 => (true, 0xa435),
                    0xa3f3 | 0xa3f7 => (z, 0xa3fd),
                    0xa3fb => (!z, 0xa401),
                    0xa3ff => (true, 0xa40b),
                    0xa403 => (z, 0xa409),
                    0xa407 => (!z, 0xa412),
                    0xa416 => (z, 0xa435),
                    0xa41c => (!z, 0xa422),
                    0xa42a => (!c, 0xa435),
                    0xa43a => (z, 0xa446),
                    0xa481 => (z, 0xa486),
                    0xa498 => (z, 0xa4a1),
                    0xa49c => (!n, 0xa48f),
                    0xa4bf => (!z, 0xa4c4),
                    0xa4e1 => (n, 0xa4e7),
                    0xa4e5 => (true, 0xa4ef),
                    0xa4eb => (!c, 0xa4ef),
                    0xa51d => (z, 0xa525),
                    0xa54c => (!z, 0xa551),
                    0xa56e => (n, 0xa574),
                    0xa572 => (true, 0xa57c),
                    0xa578 => (!c, 0xa57c),
                    0xa5aa => (z, 0xa5b2),
                    0xaac6 => (c, 0xaacb),
                    _ => (!n, 0xaac5),
                };
                if w.branch(take) {
                    target
                } else {
                    pc + 2
                }
            }
            0xa19f | 0xa1b6 | 0xa1cd | 0xa1d6 | 0xa1dd | 0xa1e1 | 0xa1e3 | 0xa1f0 | 0xa1f2
            | 0xa200 | 0xa21e | 0xa22a | 0xa23f | 0xa273 | 0xa28e | 0xa2a0 | 0xa2a4 | 0xa2b2
            | 0xa2b4 | 0xa2c2 | 0xa2e1 | 0xa2e9 | 0xa2f5 | 0xa30b | 0xa32b | 0xa33d | 0xa33f
            | 0xa35c | 0xa37b | 0xa385 | 0xa3eb | 0xa40e | 0xa426 | 0xa431 | 0xa446 | 0xa44a
            | 0xa457 | 0xa45c | 0xa46d | 0xa476 | 0xa4b2 | 0xa4cd | 0xa4ef | 0xa4f1 | 0xa510
            | 0xa55a | 0xa57c | 0xa57e | 0xa59d => {
                let a = match pc {
                    0xa19f | 0xa1b6 | 0xa1dd => 0,
                    0xa1cd => 0x20,
                    0xa1d6 => 1,
                    0xa1e1 => 0x45,
                    0xa1e3 => 0x44,
                    0xa1f0 => 0x72,
                    0xa1f2 => 0x73,
                    0xa200 | 0xa2b4 | 0xa33f => 0x64,
                    0xa2a0 => 0x2e,
                    0xa2b2 | 0xa33d | 0xa385 => 0x65,
                    0xa446 => 3,
                    0xa44a => 0x76,
                    0xa457 | 0xa4b2 => 0x74,
                    0xa45c | 0xa476 => 4,
                    0xa4cd | 0xa55a => 10,
                    0xa4ef | 0xa57c => 15,
                    0xa4f1 | 0xa57e => 14,
                    0xa510 | 0xa59d => 6,
                    _ => 2,
                };
                let v = if pc == 0xa19f {
                    w.cpu.y
                } else if matches!(
                    pc,
                    0xa1f2 | 0xa22a | 0xa2a0 | 0xa2b4 | 0xa2f5 | 0xa33f | 0xa446 | 0xa4f1 | 0xa57e
                ) {
                    0
                } else {
                    w.cpu.a & mask
                };
                w.dp_store(a, v, if pc == 0xa19f { x } else { m });
                pc + 2
            }
            0xa1a9 | 0xa1e7 | 0xa202 | 0xa217 | 0xa223 | 0xa234 | 0xa24b | 0xa277 | 0xa282 | 0xa28b
            | 0xa2ab | 0xa2ce | 0xa2d1 | 0xa2d4 | 0xa2d7 | 0xa2de | 0xa2e3 | 0xa2f0 | 0xa301
            | 0xa308 | 0xa336 | 0xa350 | 0xa357 | 0xa36d | 0xa373 | 0xa38c | 0xa3a4 | 0xa3ac
            | 0xa3b7 | 0xa3c1 | 0xa3c6 | 0xa3d0 | 0xa3dd | 0xa3e8 | 0xa40b | 0xa47e | 0xa48f
            | 0xa4d7 | 0xa52b | 0xa564 | 0xa5b8 => {
                let a = match pc {
                    0xa1a9 | 0xa277 => 0x462,
                    0xa202 => 0xfb3,
                    0xa217 => 0x37d,
                    0xa223 => 0x3ef,
                    0xa234 => 0x2e0,
                    0xa1e7 | 0xa24b => 0x351,
                    0xa282 => 0x376,
                    0xa28b | 0xa308 | 0xa3c1 => 0x30a,
                    0xa2ab => 0x345,
                    0xa2ce => 0x33c,
                    0xa2d1 => 0x33d,
                    0xa2d4 => 0x33e,
                    0xa2d7 => 0x33f,
                    0xa2de => 0x2cc,
                    0xa2e3 => 0x32a,
                    0xa2f0 => 0x2da,
                    0xa301 => 0x36b,
                    0xa336 => 0x34e,
                    0xa350 => 0x360,
                    0xa357 | 0xa3e8 => 0x300,
                    0xa36d => 0xa124,
                    0xa373 => 0xa125,
                    0xa38c | 0xa3b7 => 0x308,
                    0xa3a4 | 0xa3ac => 0x309,
                    0xa3c6 => 0x377,
                    0xa3d0 => 0x301,
                    0xa3dd => 0x37a,
                    0xa40b => 0x31c,
                    0xa47e => 0xabd,
                    0xa48f => 0x354,
                    0xa4d7 | 0xa564 => 0x352,
                    _ => 0x346,
                };
                w.drain(16);
                let v = w.read(w.absolute(a), m);
                if matches!(pc, 0xa2d1 | 0xa2d4 | 0xa2d7 | 0xa52b | 0xa5b8) {
                    w.load((w.cpu.a | v) & mask, m);
                } else if matches!(pc, 0xa4d7 | 0xa564) {
                    w.adc(v);
                } else {
                    w.load(v, m);
                }
                pc + 3
            }
            0xa1ac | 0xa1f8 | 0xa23d | 0xa27a | 0xa285 | 0xa294 | 0xa2c0 | 0xa2ca | 0xa304
            | 0xa321 | 0xa35a | 0xa3a7 | 0xa3af | 0xa3ba | 0xa414 | 0xa450 | 0xa45f | 0xa473
            | 0xa492 | 0xa4a2 | 0xa4ab | 0xa4b9 | 0xa4d3 | 0xa50d | 0xa51a | 0xa526 | 0xa53b
            | 0xa546 | 0xa560 | 0xa59a | 0xa5a7 | 0xa5b3 | 0xa5c8 => {
                w.mask(
                    match pc {
                        0xa1ac | 0xa27a => 4,
                        0xa23d | 0xa285 | 0xa35a => 3,
                        0xa294 => 13,
                        0xa2c0 | 0xa304 | 0xa3af | 0xa51a | 0xa5a7 => 1,
                        0xa2ca => 15,
                        0xa321 => 24,
                        0xa3a7 => 2,
                        0xa3ba | 0xa414 => 0x80,
                        0xa526 | 0xa5b3 => 0xc000,
                        0xa53b | 0xa5c8 => 0xff00,
                        _ => 255,
                    },
                    m,
                );
                pc + 1 + u16::from(m)
            }
            0xa1b2 | 0xa1c4 | 0xa1ee | 0xa1fc | 0xa206 | 0xa326 | 0xa453 | 0xa45e | 0xa466
            | 0xa46f | 0xa4a1 | 0xa4a8 | 0xa4da | 0xa513 | 0xa517 | 0xa51f | 0xa524 | 0xa525
            | 0xa534 | 0xa537 | 0xa567 | 0xa5a0 | 0xa5a4 | 0xa5ac | 0xa5b1 | 0xa5b2 | 0xa5c1
            | 0xa5c4 => {
                w.drain(6);
                match pc {
                    0xa1c4 | 0xa206 | 0xa466 | 0xa46f | 0xa513 | 0xa517 | 0xa524 | 0xa5a0
                    | 0xa5a4 | 0xa5b1 => {
                        w.cpu.y = w.cpu.a & if x == 1 { 255 } else { 65535 };
                        w.flags(w.cpu.y, x);
                    }
                    0xa1fc | 0xa326 | 0xa453 | 0xa4a8 | 0xa4da | 0xa537 | 0xa567 | 0xa5c4 => {
                        w.cpu.x = w.cpu.a & if x == 1 { 255 } else { 65535 };
                        w.flags(w.cpu.x, x);
                    }
                    _ => w.load(
                        if matches!(pc, 0xa1ee | 0xa4a1 | 0xa534 | 0xa5c1) {
                            w.cpu.x
                        } else {
                            w.cpu.y
                        },
                        m,
                    ),
                }
                pc + 1
            }
            0xa1b3 | 0xa1c1 | 0xa1ca | 0xa463 | 0xa46a | 0xa4a5 | 0xa4ae | 0xa4d6 | 0xa4f6
            | 0xa502 | 0xa563 | 0xa583 | 0xa58f => {
                w.clear_carry();
                pc + 1
            }
            0xa1d3 | 0xa1da | 0xa42e | 0xa4f9 | 0xa586 => {
                w.drain(6);
                w.cpu.p |= CARRY;
                pc + 1
            }
            0xa1b4 | 0xa1c2 | 0xa1cb | 0xa1d4 | 0xa1db | 0xa464 | 0xa46b | 0xa4a6 | 0xa4f7
            | 0xa4fa | 0xa503 | 0xa529 | 0xa584 | 0xa587 | 0xa590 | 0xa5b6 => {
                let a = match pc {
                    0xa1cb => 0x20,
                    0xa1d4 => 0xe8,
                    0xa1db => 0xe2,
                    0xa464 => 4,
                    0xa46b => 2,
                    0xa4a6 => 0x74,
                    0xa4f7 | 0xa584 => 1,
                    0xa4fa | 0xa587 => 15,
                    0xa529 | 0xa5b6 => 0x64,
                    _ => 0,
                };
                let v = w.dp_read(a, m);
                if matches!(pc, 0xa1d4 | 0xa1db | 0xa4fa | 0xa587) {
                    w.sbc(v, m);
                } else if matches!(pc, 0xa529 | 0xa5b6) {
                    w.load((w.cpu.a | v) & mask, m);
                } else if m == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                pc + 2
            }
            0xa1c0
            | 0xa1ef
            | 0xa1fb
            | 0xa205
            | 0xa462
            | 0xa4c4
            | 0xa520..=0xa523
            | 0xa551
            | 0xa5ad..=0xa5b0
            | 0xaac5 => {
                w.shift_left(m);
                pc + 1
            }
            0xa1c5 | 0xa1cf | 0xa1f4 | 0xa20d | 0xa44c | 0xa4dd | 0xa508 | 0xa56a | 0xa595 => {
                w.width(
                    if pc == 0xa44c { 0x30 } else { 0x20 },
                    matches!(pc, 0xa1c5 | 0xa1f4 | 0xa44c | 0xa508 | 0xa595),
                );
                pc + 2
            }
            0xa1c7 | 0xa1fd | 0xa207 | 0xa270 | 0xa39d | 0xa3d8 | 0xa3e5 | 0xa454 | 0xa459
            | 0xa467 | 0xa470 | 0xa495 | 0xa4af | 0xa4b6 | 0xa4ca | 0xa4f3 | 0xa4ff | 0xa50a
            | 0xa514 | 0xa538 | 0xa543 | 0xa557 | 0xa580 | 0xa58c | 0xa597 | 0xa5a1 | 0xa5c5 => {
                let (base, index) = match pc {
                    0xa1c7 => (0xa15e, w.cpu.y),
                    0xa1fd => (0xa126, w.cpu.x),
                    0xa207 => (0xa120, w.cpu.y),
                    0xa270 => (0xa131, w.cpu.x),
                    0xa39d => (0xa148, w.cpu.x),
                    0xa3d8 => (0xa13a, w.cpu.x),
                    0xa3e5 => (0xa142, w.cpu.x),
                    0xa454 => (0xa070, w.cpu.x),
                    0xa459 => (0xa030, w.cpu.x),
                    0xa467 => (0x9ef0, w.cpu.y),
                    0xa470 => (0x9cf1, w.cpu.y),
                    0xa495 => (0xa150, w.cpu.x),
                    0xa4af => (0xa038, w.cpu.x),
                    0xa4b6 | 0xa50a => (0x89f9, w.cpu.y),
                    0xa4ca => (0xa108, w.cpu.x),
                    0xa4f3 => (0x92ed, w.cpu.y),
                    0xa4ff => (0x9369, w.cpu.y),
                    0xa514 | 0xa5a1 => (0x838c, w.cpu.y),
                    0xa538 | 0xa5c5 => (0xa20, w.cpu.x),
                    0xa543 | 0xa597 => (0x8a75, w.cpu.y),
                    0xa557 => (0xa10c, w.cpu.x),
                    0xa580 => (0x93e5, w.cpu.y),
                    _ => (0x9461, w.cpu.y),
                };
                let ldy = matches!(pc, 0xa39d | 0xa3d8 | 0xa3e5);
                w.drain(16);
                if x == 2 || (base & 255) + index > 255 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(base) + u32::from(index), if ldy { x } else { m });
                if ldy {
                    w.cpu.y = v;
                    w.flags(v, x);
                } else if pc == 0xa495 {
                    let a = w.cpu.a & mask;
                    w.cpu.p = (w.cpu.p & !CARRY) | u8::from(a >= v);
                    w.flags(a.wrapping_sub(v) & mask, m);
                } else if pc == 0xa4af {
                    w.adc(v);
                } else {
                    w.load(v, m);
                }
                pc + 3
            }
            0xa20a | 0xa22e | 0xa243 | 0xa2a8 | 0xa370 | 0xa376 | 0xa39a | 0xa435 | 0xa43e
            | 0xa443 | 0xa47b | 0xa483 | 0xa486 | 0xa489 | 0xa4c5 | 0xa552 => {
                let a = match pc {
                    0xa20a | 0xa370 => 0x352,
                    0xa376 => 0x353,
                    0xa435 => 0x354,
                    0xa43e => 0x35d,
                    0xa443 => 0x35e,
                    0xa47b | 0xa483 => 0x346,
                    0xa486 | 0xa4c5 => 0x102,
                    0xa489 | 0xa552 => 0x104,
                    _ => 0x323,
                };
                w.drain(16);
                w.write(
                    w.absolute(a),
                    if pc == 0xa435 {
                        w.cpu.y
                    } else if matches!(pc, 0xa483 | 0xa486 | 0xa489) {
                        0
                    } else {
                        w.cpu.a & mask
                    },
                    if pc == 0xa435 { x } else { m },
                );
                pc + 3
            }
            0xa220 | 0xa231 | 0xa246 | 0xa2ed | 0xa32d | 0xa341 | 0xa35e | 0xa389 | 0xa3be
            | 0xa49e | 0xa4c1 | 0xa54e => {
                w.operand_then_internal(2);
                match pc {
                    0xa341 => 0xa3c6,
                    0xa49e | 0xa54e => 0xa5ce,
                    0xa4c1 => 0xa541,
                    _ => 0xa435,
                }
            }
            0xa266 | 0xa26a | 0xa394 => {
                w.compare_x_immediate(if pc == 0xa266 { 4 } else { 6 }, x);
                pc + 1 + u16::from(x)
            }
            0xa3a0 | 0xa438 => {
                w.drain(8 * u32::from(x));
                let v = if pc == 0xa3a0 { 13 } else { 5 };
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.y >= v);
                w.flags(
                    w.cpu.y.wrapping_sub(v) & if x == 1 { 255 } else { 65535 },
                    x,
                );
                pc + 1 + u16::from(x)
            }
            0xa2e8 | 0xa3cb => {
                w.drain(6);
                w.load(w.cpu.a.wrapping_sub(1) & mask, m);
                pc + 1
            }
            0xa323 | 0xa324 | 0xa325 | 0xa512 | 0xa535 | 0xa536 | 0xa59f | 0xa5c2 | 0xa5c3 => {
                w.drain(6);
                let v = w.cpu.a & mask;
                w.cpu.p = (w.cpu.p & !CARRY) | (v & 1) as u8;
                w.load(v >> 1, m);
                pc + 1
            }
            0xa327 => {
                w.drain(24);
                let v = w.read(0x079635 + u32::from(w.cpu.x), m);
                w.load(v, m);
                pc + 4
            }
            0xa383 | 0xa42f | 0xa52e | 0xa5bb => {
                w.drain(8 * u32::from(m));
                if pc == 0xa42f {
                    w.sbc(10, m);
                } else {
                    w.load(
                        (w.cpu.a
                            | match pc {
                                0xa383 => 0x30,
                                0xa52e => 4,
                                _ => 0x14,
                            })
                            & mask,
                        m,
                    );
                }
                pc + 1 + u16::from(m)
            }
            0xa391 | 0xa3d5 | 0xa3e2 => {
                w.operand_then_internal(2);
                w.push(pc + 2, 2);
                self.near_returns.push(pc + 3);
                0xaac3
            }
            0xa3ab => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_add(1) & if x == 1 { 255 } else { 65535 };
                w.flags(w.cpu.y, x);
                pc + 1
            }
            0xa49a | 0xa49b | 0xaac8 => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_sub(1) & if x == 1 { 255 } else { 65535 };
                w.flags(w.cpu.x, x);
                pc + 1
            }
            0xa4d1 | 0xa55e => {
                w.drain(8);
                let pointer = w.read(10, 2);
                if x == 2 || (pointer & 255) + w.cpu.y > 255 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(pointer) + u32::from(w.cpu.y), m);
                w.load(v, m);
                pc + 2
            }
            0xa4fc | 0xa505 | 0xa531 | 0xa53e | 0xa589 | 0xa592 | 0xa5be | 0xa5cb => {
                let a = match pc {
                    0xa4fc | 0xa589 => 0x801,
                    0xa505 | 0xa592 => 0x800,
                    0xa531 | 0xa5be => 0x802,
                    _ => 0xa20,
                };
                w.operand_then_internal(2);
                w.write(w.absolute(a) + u32::from(w.cpu.x), w.cpu.a & mask, m);
                pc + 3
            }
            0xaacb => {
                w.rts();
                assert_eq!(self.near_returns.pop(), Some(w.cpu.pc));
                w.cpu.pc
            }
            _ => unreachable!("Link OAM {pc:04x}"),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn run(
        rom: &[u8],
        mode: usize,
        case: usize,
        banks: bool,
        visited: &mut std::collections::BTreeSet<u16>,
    ) -> usize {
        let mut machine = snes::Snes::new();
        machine.cart.load(snes::CartType::LoRom, rom, 0x2000);
        let ram = &mut machine.ram;
        ram.fill(0);
        for (i, byte) in ram[0x800..0xb00].iter_mut().enumerate() {
            *byte = (i as u8).wrapping_mul(37).wrapping_add(case as u8);
        }
        for (a, v) in [
            (0x20, [0u16, 1, 255, 0xffff][case % 4]),
            (0x22, [0xfffeu16, 0, 255, 0x100][case / 4 % 4]),
            (0xe2, [0u16, 0x100, 0xffff, 16][case % 4]),
            (0xe8, [0u16, 255, 0x8000, 0xffff][case / 4 % 4]),
            (0x24, [0u16, 1, 0xffef, 0xfff0, 0xffff, 0x8000][case % 6]),
        ] {
            ram[a..a + 2].copy_from_slice(&v.to_le_bytes());
        }
        ram[0x10] = if case & 32 == 0 { 7 } else { 18 };
        ram[0x11] = [0, 14, 15, 18, 19][case / 8 % 5];
        let stair = matches!(ram[0x11], 18 | 19);
        ram[0x2e] = (case / 4 % 9) as u8;
        ram[0x2f] = (case % 4 * 2) as u8;
        ram[0x462] = (case & 4) as u8;
        ram[0xee] = (case % 4) as u8;
        ram[0xfb3] = (case & 1) as u8;
        ram[0x351] = (case / 4 % 2) as u8;
        ram[0x345] = (case / 8 % 2) as u8;
        ram[0xabd] = (case / 16 % 2) as u8;
        ram[0x300] = (case / 4 % 3) as u8;
        ram[0x30a] = (case / 4 % 3) as u8;
        ram[0x31c] = (case / 4 % 3) as u8;
        match if banks { 0 } else { mode } {
            0 => {}
            1 => {
                ram[0x5d] = 22;
                ram[0x37d] = (case % 3) as u8;
            }
            2 => ram[0x3ef] = 1,
            3 => ram[0x2e0] = 1,
            4 => {
                ram[0x11] = 14;
                ram[0x28] = 1;
            }
            5 => ram[0x376] = 1,
            6 => ram[0x48] = 1,
            7 => {
                ram[0x5d] = 4;
                ram[0xf0] = (case & 15) as u8;
                ram[0x33c] = (case & 1) as u8;
                ram[0x2cc] = (case / 4 % 3) as u8;
                ram[0x32a] = (case / 8 % 4) as u8;
            }
            8 => ram[0x2da] = (case % 2 + 1) as u8,
            9 => ram[0x36b] = 1,
            10 => {
                ram[0x4d] = 4;
                ram[0x1a] = (case * 8) as u8;
            }
            11 => {
                ram[0x4d] = 1;
                ram[0x5d] = 5;
                ram[0x34e] = (case & 1) as u8;
            }
            12 => {
                ram[0x4d] = 1;
                ram[0x360] = (case & 1) as u8;
            }
            13 => {
                ram[0x5b] = (case % 2 + 2) as u8;
                ram[0x5a] = (case / 4 % 10) as u8;
            }
            14 => {
                ram[0x308] = 1 << (case % 8);
                ram[0x309] = (case / 8 % 4) as u8;
            }
            15 => ram[0x301] = 1 << (case % 8),
            16 => ram[0x37a] = 1 << (case % 8),
            17 => ram[0x377] = (case / 4 % 4 + 1) as u8,
            18 => ram[0x5d] = [8, 9, 10][case % 3],
            19 => ram[0x5d] = [3, 30][case % 2],
            20 => {
                ram[0x3a] = 0x80;
                ram[0x3c] = (case / 4 % 18) as u8;
            }
            _ => unreachable!(),
        }
        let stair = stair && mode != 4;
        ram[0x1fd..0x200].copy_from_slice(&[0x2b, 0x93, 2]);
        let mut cpu = NativeNmiContext {
            a: 0xab80,
            x: 0x47,
            y: 0x51,
            d: 0,
            db: (case % 4) as u8,
            bank: 13,
            pc: 0xa18e,
            sp: 0x1fc,
            p: 0x30 | (case as u8 & 0xc3),
        };
        let mut work = LinkOam::default();
        if banks {
            cpu.pc = 0xa47e;
            cpu.db = 13;
            cpu.sp = 0x1fb;
            cpu.p &= !0x30;
            work.pc = 0xa47e;
            ram[0x354] = 4;
            ram[0x74..0x76].copy_from_slice(&0u16.to_le_bytes());
            ram[0x76..0x78].copy_from_slice(&(mode as u16).to_le_bytes());
            ram[4..6].copy_from_slice(&(case as u16 % 12).to_le_bytes());
            ram[0x72..0x74].copy_from_slice(&(if case & 1 == 0 { 0u16 } else { 2 }).to_le_bytes());
            ram[0x352..0x354]
                .copy_from_slice(&(if case & 2 == 0 { 0x190u16 } else { 0xe0 }).to_le_bytes());
            ram[0x346..0x348].copy_from_slice(&0xe00u16.to_le_bytes());
            ram[0x64..0x66].copy_from_slice(&[0, 0x30]);
            ram[0] = case as u8;
            ram[1] = (case as u8).wrapping_mul(31);
            ram[0x1fc] = 0;
        }
        machine.cpu.a = cpu.a;
        machine.cpu.x = cpu.x;
        machine.cpu.y = cpu.y;
        machine.cpu.k = cpu.bank;
        machine.cpu.pc = cpu.pc;
        machine.cpu.db = cpu.db;
        machine.cpu.sp = cpu.sp;
        machine.cpu.e = false;
        machine.cpu.unpack_flags(cpu.p);
        let mut ram = machine.ram.clone();
        let clock = CpuMasterTimeline::at_raster(
            [0, 1, 30002, 30003][case % 4],
            snes::CpuRasterPosition::new(224, 1340),
            snes::CpuBusWorkload::default(),
            snes::CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        let mut source =
            snes::RomCpuTimingProbe::new(machine, clock, snes::SourcePpuReadState::snes9x_reset())
                .unwrap();
        let mut clock = source.timeline().clone();
        let mut instructions = 0;
        for _ in 0..2000 {
            if work.child().is_some() {
                break;
            }
            visited.insert(work.pc);
            let label = format!(
                "LinkOam mode={mode} case={case} banks={banks} pc={:04x}",
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
                    work.step(cpu, clock, |at, access| {
                        if let super::super::intro_clear::Access::Read(a, width) = access {
                            if a & 0x8000 != 0 && a < 0x7e0000 {
                                for byte in 0..width {
                                    let a = a + u32::from(byte);
                                    assert_eq!(
                                        table_byte(a),
                                        Some(
                                            rom[(a >> 16) as usize * 0x8000
                                                + (a as usize & 0x7fff)]
                                        ),
                                        "{label}: native table {a:06x}"
                                    );
                                }
                            }
                        }
                        bus(at, access)
                    })
                    .unwrap();
                },
            );
            instructions += 1;
        }
        assert_eq!(work.child(), Some(LinkOamChild::Equipment));
        assert!(!work.complete());
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.sp),
            (13, 0xa5ce, if !banks && stair { 0x1f9 } else { 0x1fb })
        );
        assert_eq!(ram, source.snes().ram);
        instructions
    }
    #[test]
    fn link_oam_pose_and_initial_banks_match_source_live_stack_tables_and_publications() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut visited = std::collections::BTreeSet::new();
        let mut instructions = 0;
        for mode in 0..21 {
            for case in 0..256 {
                instructions += run(&rom, mode, case, false, &mut visited);
            }
        }
        for bank_index in 0..124 {
            for case in 0..48 {
                instructions += run(&rom, bank_index, case, true, &mut visited);
            }
        }
        for pc in [
            0xa1c7, 0xa2dc, 0xa32b, 0xa391, 0xa3d5, 0xa3e2, 0xa467, 0xa495, 0xa4d1, 0xa531, 0xa53e,
            0xa55e, 0xa5be, 0xa5cb, 0xaacb,
        ] {
            assert!(visited.contains(&pc), "uncovered OAM path {pc:04x}");
        }
        eprintln!("11328 LinkOam pose/bank fixtures, {instructions} instruction boundaries, {} distinct PCs",visited.len());
    }
}
