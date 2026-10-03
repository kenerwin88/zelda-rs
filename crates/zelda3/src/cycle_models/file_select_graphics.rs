//! File-selection CHR uploads, with live decompressor, planar loops and font
//! reads retained on the caller's CPU stack.
use super::{
    native_decompress::NativeDecompression,
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, NEGATIVE},
    ppu_plane_conversion::{PlaneLoop, PpuPlaneConversion},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Child {
    Decompress(NativeDecompression),
    Planes(PpuPlaneConversion),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FileSelectGraphics {
    pc: u16,
    child: Option<Child>,
    complete: bool,
}
impl Default for FileSelectGraphics {
    fn default() -> Self {
        Self {
            pc: 0xe4e9,
            child: None,
            complete: false,
        }
    }
}
impl FileSelectGraphics {
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        match &self.child {
            Some(Child::Decompress(w)) => w.routine(),
            Some(Child::Planes(w)) => w.routine(),
            None => 0xe4e9,
        }
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
        if let Some(child) = self.child.as_mut() {
            let (cycles, complete) = match child {
                Child::Decompress(w) => {
                    let c = w.step(cpu, clock, bus);
                    (c, w.complete())
                }
                Child::Planes(w) => {
                    let c = w.step(cpu, clock, bus);
                    (c, w.complete())
                }
            };
            if complete {
                self.child = None;
                self.pc = cpu.pc;
            }
            return cycles;
        }
        assert_eq!((cpu.bank, cpu.pc, cpu.d), (0, self.pc, 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0xe4e9 | 0xe4ee | 0xe4f6 | 0xe517 => {
                w.immediate(
                    match self.pc {
                        0xe4e9 => 2,
                        0xe4ee => 0x80,
                        0xe4f6 => 0x50,
                        _ => 0x0e,
                    },
                    1,
                );
                self.pc + 2
            }
            0xe51d | 0xe523 => {
                w.immediate(if self.pc == 0xe51d { 0x7000 } else { 0x8000 }, 2);
                self.pc + 3
            }
            0xe4eb | 0xe4f0 | 0xe4f3 | 0xe4f8 | 0xe520 | 0xe52d | 0xe548 => {
                let target = match self.pc {
                    0xe4eb => 0x2101,
                    0xe4f0 => 0x2115,
                    0xe4f3 | 0xe520 => 0x2116,
                    0xe4f8 => 0x2117,
                    _ => 0x2118,
                };
                w.drain(16);
                w.write_io(
                    target,
                    if self.pc == 0xe4f3 { 0 } else { w.cpu.a },
                    if self.pc >= 0xe520 { 2 } else { 1 },
                );
                self.pc + 3
            }
            0xe4fb | 0xe4fc | 0xe539 | 0xe53a => {
                w.drain(6);
                w.push(
                    u16::from(if matches!(self.pc, 0xe4fb | 0xe539) {
                        w.cpu.db
                    } else {
                        w.cpu.bank
                    }),
                    1,
                );
                self.pc + 1
            }
            0xe4fd | 0xe516 | 0xe53b | 0xe554 => {
                w.drain(12);
                let v = w.pull(1);
                w.cpu.db = v as u8;
                w.flags(v, 1);
                self.pc + 1
            }
            0xe4fe | 0xe505 | 0xe50a | 0xe511 | 0xe53c => {
                w.drain(8);
                w.cpu.y = match self.pc {
                    0xe4fe => 0x5e,
                    0xe50a => 0x5f,
                    0xe53c => 0x6b,
                    _ => 0x3f,
                };
                w.flags(w.cpu.y, 1);
                self.pc + 2
            }
            0xe500 | 0xe50c | 0xe53e | 0xe507 | 0xe513 => {
                w.drain(22);
                w.push(self.pc + 2, 2);
                let planes = matches!(self.pc, 0xe507 | 0xe513);
                w.cpu.pc = if planes { 0xe5af } else { 0xe766 };
                self.child = Some(if planes {
                    Child::Planes(PpuPlaneConversion::new(PlaneLoop::HighSheet))
                } else {
                    Child::Decompress(NativeDecompression::new(true))
                });
                self.pc + 3
            }
            0xe503 | 0xe50f | 0xe51b | 0xe541 | 0xe537 | 0xe552 => {
                w.width(
                    if matches!(self.pc, 0xe503 | 0xe50f) {
                        0x20
                    } else {
                        0x30
                    },
                    !matches!(self.pc, 0xe537 | 0xe552),
                );
                self.pc + 2
            }
            0xe519 | 0xe526 => {
                w.dp_store(
                    if self.pc == 0xe519 { 2 } else { 0 },
                    w.cpu.a,
                    if self.pc == 0xe519 { 1 } else { 2 },
                );
                self.pc + 2
            }
            0xe528 | 0xe543 => {
                w.drain(16);
                w.cpu.x = if self.pc == 0xe528 { 0x7ff } else { 0x2ff };
                w.flags(w.cpu.x, 2);
                self.pc + 3
            }
            0xe52b | 0xe546 => {
                let v = w.indirect_read(0, 2);
                w.load(v, 2);
                self.pc + 2
            }
            0xe530 | 0xe532 | 0xe54b | 0xe54d => {
                w.dp_increment(0, 2, true);
                self.pc + 2
            }
            0xe534 | 0xe54f => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_sub(1);
                w.flags(w.cpu.x, 2);
                self.pc + 1
            }
            0xe535 | 0xe550 => {
                if w.branch(w.cpu.p & NEGATIVE == 0) {
                    if self.pc == 0xe535 {
                        0xe52b
                    } else {
                        0xe546
                    }
                } else {
                    self.pc + 2
                }
            }
            0xe608 => {
                w.rts();
                w.cpu.pc
            }
            0xe555 => {
                w.rtl();
                self.complete = true;
                w.cpu.pc
            }
            _ => unreachable!("file CHR continuation {:04x}", self.pc),
        };
        if self.child.is_none() {
            w.cpu.pc = self.pc;
        }
        Some(w.cycles)
    }
}
