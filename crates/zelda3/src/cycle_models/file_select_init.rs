//! Retained file-selection initializer. Calls share the live CPU and stack;
//! palette and graphics children retire their own bus work across interrupts.
use super::{
    default_graphics::DefaultGraphics,
    intro_clear::Access,
    intro_palettes::{IntroPalettes, PaletteSource},
    native_tilesets::NativeTilesets,
    native_work::Work,
    nmi_stack::NativeNmiContext,
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Child {
    Palette(IntroPalettes),
    Default(DefaultGraphics),
    Tilesets(NativeTilesets),
    Graphics(super::file_select_graphics::FileSelectGraphics),
    Sram(super::sram_validate::SramValidate),
    Enemy(super::enemy_damage::EnemyDamage),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FileSelectInit {
    pc: u16,
    child: Option<Child>,
    complete: bool,
}
impl Default for FileSelectInit {
    fn default() -> Self {
        Self {
            pc: 0xcda1,
            child: None,
            complete: false,
        }
    }
}
impl FileSelectInit {
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        match &self.child {
            Some(Child::Palette(w)) => w.routine(),
            Some(Child::Default(w)) => w.routine(),
            Some(Child::Tilesets(w)) => w.routine(),
            Some(Child::Graphics(w)) => w.routine(),
            Some(Child::Sram(_)) => 0x028054,
            Some(Child::Enemy(w)) => w.routine(),
            None => 0x0ccd9d,
        }
    }
    pub(crate) fn palette_source(&self, cpu: &NativeNmiContext) -> PaletteSource {
        match &self.child {
            Some(Child::Palette(w)) => w.palette_source(cpu),
            _ => PaletteSource::Constant,
        }
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
            let tail = matches!(child, Child::Enemy(_));
            let (cycles, complete) = match child {
                Child::Palette(w) => {
                    let c = w.step(cpu, clock, bus);
                    (c, w.complete())
                }
                Child::Default(w) => {
                    let c = w.step(cpu, clock, bus);
                    (c, w.complete())
                }
                Child::Tilesets(w) => {
                    let c = w.step(cpu, clock, bus);
                    (c, w.complete())
                }
                Child::Graphics(w) => {
                    let c = w.step(cpu, clock, bus);
                    (c, w.complete())
                }
                Child::Sram(w) => {
                    let c = w.step(cpu, clock, bus);
                    (c, w.complete())
                }
                Child::Enemy(w) => {
                    let c = w.step(cpu, clock, bus);
                    (c, w.complete())
                }
            };
            if complete {
                self.child = None;
                if tail {
                    self.complete = true;
                } else {
                    assert_eq!((cpu.bank, cpu.pc), (12, self.pc));
                }
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
        w.fetch();
        self.pc = match self.pc {
            0xcda1 | 0xcda4 | 0xcdcc => {
                w.drain(16);
                w.write(
                    w.absolute(match self.pc {
                        0xcda1 => 0x12a,
                        0xcda4 => 0x1f0c,
                        _ => 0x202,
                    }),
                    0,
                    1,
                );
                self.pc + 3
            }
            0xcda7 | 0xcdae | 0xcdb3 | 0xcdc3 | 0xcdcf | 0xcdd4 | 0xcdd9 => {
                w.immediate(
                    match self.pc {
                        0xcda7 => 11,
                        0xcdae => 2,
                        0xcdb3 => 6,
                        0xcdc3 => 0,
                        0xcdcf => 1,
                        0xcdd4 => 35,
                        _ => 81,
                    },
                    1,
                );
                self.pc + 2
            }
            0xcda9 | 0xcdb0 | 0xcdb5 | 0xcdb8 | 0xcdc5 | 0xcdd1 | 0xcdd6 | 0xcddb => {
                let address = match self.pc {
                    0xcda9 => 0x12c,
                    0xcdb0 => 0xaa9,
                    0xcdb5 => 0xab6,
                    0xcdb8 => 0x710,
                    0xcdc5 => 0xab2,
                    0xcdd1 => 0xaa4,
                    0xcdd6 => 0xaa1,
                    _ => 0xaa2,
                };
                w.drain(16);
                w.write(w.absolute(address), w.cpu.a & 255, 1);
                self.pc + 3
            }
            0xcdac => {
                w.dp_increment(0x11, 1, true);
                self.pc + 2
            }
            0xcdbb | 0xcdbf | 0xcdc8 | 0xcdde | 0xcde2 | 0xcde6 | 0xcdea => {
                let target = match self.pc {
                    0xcdbb => 0x1bee74,
                    0xcdbf => 0x1beea8,
                    0xcdc8 => 0x1bee52,
                    0xcdde => 0x00e2d0,
                    0xcde2 => 0x00e19b,
                    0xcde6 => 0x00e4e9,
                    _ => 0x028054,
                };
                w.drain(30);
                w.push(u16::from(w.cpu.bank), 1);
                w.push(self.pc + 3, 2);
                w.cpu.bank = (target >> 16) as u8;
                w.cpu.pc = target as u16;
                self.child = Some(match target {
                    0x00e2d0 => Child::Default(Default::default()),
                    0x00e19b => Child::Tilesets(Default::default()),
                    0x00e4e9 => Child::Graphics(Default::default()),
                    0x028054 => Child::Sram(Default::default()),
                    _ => Child::Palette(IntroPalettes::from_entry(target)),
                });
                self.pc + 4
            }
            0xcdee => {
                w.drain(24);
                w.cpu.bank = 2;
                w.cpu.pc = 0xfe71;
                self.child = Some(Child::Enemy(Default::default()));
                self.pc
            }
            _ => unreachable!("file-selection continuation {:04x}", self.pc),
        };
        if self.child.is_none() {
            w.cpu.pc = self.pc;
        }
        Some(w.cycles)
    }
}
