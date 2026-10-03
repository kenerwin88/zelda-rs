//! File menu initialization owns the flag clear, tilemap DMA, palette calls,
//! and final update requests on one interruptible native CPU stack.
use super::{
    file_select_palettes::FileSelectPalettes,
    intro_clear::Access,
    intro_palettes::PaletteSource,
    native_tilemap_erase::NativeTilemapErase,
    native_work::Work,
    nmi_stack::{NativeNmiContext, NEGATIVE},
    tilemap_erase::Pattern,
};
use snes::{CpuMasterTimeline, DmaState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FileSelectErase {
    pc: u16,
    erase: Option<NativeTilemapErase>,
    palettes: Option<FileSelectPalettes>,
    complete: bool,
}
impl Default for FileSelectErase {
    fn default() -> Self {
        Self {
            pc: 0xcdf2,
            erase: None,
            palettes: None,
            complete: false,
        }
    }
}
impl FileSelectErase {
    pub(crate) fn erase_triforce() -> Self {
        Self { pc:0xcdf9, ..Default::default() }
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn take_dma_end(&mut self) -> Option<u64> {
        self.erase
            .as_mut()
            .and_then(NativeTilemapErase::take_dma_end)
    }
    pub(crate) fn palette_source(&self) -> PaletteSource {
        if self.palettes.is_some() {
            PaletteSource::Asset
        } else {
            PaletteSource::Constant
        }
    }
    pub(crate) fn routine(&self) -> u32 {
        if let Some(w) = &self.erase {
            w.routine()
        } else if self.palettes.is_some() {
            0x1bef96
        } else if self.pc < 0xc000 {
            0x893d
        } else {
            0x0ccdf2
        }
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        dma: &mut DmaState,
        bus: impl FnMut(u64, Access) -> u16,
    ) -> Option<u64> {
        if self.complete {
            return None;
        }
        if let Some(w) = &mut self.erase {
            let c = w.step(cpu, clock, dma, bus);
            if w.complete() {
                self.erase = None;
                assert_eq!((cpu.bank, cpu.pc), (12, self.pc));
            }
            return c;
        }
        if let Some(w) = &mut self.palettes {
            let c = w.step(cpu, clock, bus);
            if w.complete() {
                self.palettes = None;
                assert_eq!((cpu.bank, cpu.pc), (12, self.pc));
            }
            return c;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.db, cpu.d),
            (if self.pc < 0xc000 { 0 } else { 12 }, self.pc, 0, 0)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0xcdf2 => {
                w.drain(8);
                w.cpu.x = 5;
                w.flags(5, 1);
                0xcdf4
            }
            0xcdf4 => {
                w.drain(14);
                w.write(u32::from(0xbfu16.wrapping_add(w.cpu.x)), 0, 1);
                0xcdf6
            }
            0xcdf6 => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_sub(1) & 255;
                w.flags(w.cpu.x, 1);
                0xcdf7
            }
            0xcdf7 => {
                if w.branch(w.cpu.p & NEGATIVE == 0) {
                    0xcdf4
                } else {
                    0xcdf9
                }
            }
            0xcdf9 | 0x893d => {
                w.immediate(0x80, 1);
                self.pc + 2
            }
            0xcdfb => {
                w.drain(16);
                w.write(w.absolute(0x710), w.cpu.a & 255, 1);
                0xcdfe
            }
            0xcdfe | 0xce02 | 0xce06 => {
                w.drain(30);
                w.push(u16::from(w.cpu.bank), 1);
                w.push(self.pc + 3, 2);
                w.cpu.bank = if self.pc == 0xce06 { 27 } else { 0 };
                match self.pc {
                    0xcdfe => 0x893d,
                    0xce02 => {
                        w.cpu.pc = 0x8333;
                        self.erase = Some(NativeTilemapErase::new(Pattern::Triforce));
                        0xce06
                    }
                    _ => {
                        w.cpu.pc = 0xef96;
                        self.palettes = Some(Default::default());
                        0xce0a
                    }
                }
            }
            0x893f => {
                w.drain(16);
                w.write_io(0x2100, w.cpu.a & 255, 1);
                0x8942
            }
            0x8942 => {
                w.dp_store(0x13, w.cpu.a & 255, 1);
                0x8944
            }
            0x8944 => {
                w.drain(16);
                w.write_io(0x420c, 0, 1);
                for ch in &mut dma.channel {
                    ch.hdma_active = false;
                }
                0x8947
            }
            0x8947 => {
                w.dp_store(0x9b, 0, 1);
                0x8949
            }
            0x8949 => {
                w.rtl();
                w.cpu.pc
            }
            0xce0a | 0xce0c => {
                w.dp_increment(if self.pc == 0xce0a { 0x15 } else { 0x11 }, 1, true);
                self.pc + 2
            }
            0xce0e => {
                w.rtl();
                self.complete = true;
                w.cpu.pc
            }
            _ => unreachable!("file menu continuation {:04x}", self.pc),
        };
        if self.erase.is_none() && self.palettes.is_none() {
            w.cpu.pc = self.pc;
        }
        Some(w.cycles)
    }
}
