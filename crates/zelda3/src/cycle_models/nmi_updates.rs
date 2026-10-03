//! NMI core graphics programming, with the caller's live CPU and bus clock.
//! Constants are the routine's operands; no ROM instructions are decoded.

use super::nmi_stack::{NativeNmiContext, ACCUMULATOR_BYTE, CARRY, INDEX_BYTE, NEGATIVE, ZERO};
use snes::CpuMasterTimeline;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CoreAccess {
    ReadByte(u16),
    ReadWord(u16),
    WriteByte(u16, u8),
    WriteWord(u16, u16),
    /// Owns the nested hardware transfer and enclosing six-clock CPU write.
    Dma(u8),
    AnimatedDma(u8),
    MaintenanceDma(u8, MaintenanceKind),
    Stripe(super::nmi_stripes::Access),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MaintenanceKind {
    Hud,
    Palette,
    Oam,
    BackgroundCharacters,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct MaintenanceResult {
    pub hud: bool,
    pub palette: bool,
}

impl MaintenanceResult {
    pub fn cpu_master_cycles(self) -> u64 {
        (if self.hud { 40 + 288 } else { 46 }) + (if self.palette { 40 + 302 } else { 46 }) + 374
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct CoreResult {
    pub enabled: bool,
    pub travel_bird: bool,
}

impl CoreResult {
    pub fn cpu_master_cycles(self) -> u64 {
        if self.enabled {
            122 + 3580 + if self.travel_bird { 420 } else { 6 } + 258
        } else {
            116 + 24
        }
    }
}

impl NativeNmiContext {
    /// Continue the same updates call through HUD, CGRAM and OAM DMA.
    /// Stop before the stripe-load gate, with M16/X8 and the live stack intact.
    pub(crate) fn display_maintenance_updates(
        &mut self,
        clock: &mut CpuMasterTimeline,
        bus: impl FnMut(&mut CpuMasterTimeline, CoreAccess) -> u16,
    ) -> MaintenanceResult {
        assert_eq!(
            (self.bank, self.pc, self.db, self.d, self.p & 0x30),
            (0, 0x8b67, 0, 0, ACCUMULATOR_BYTE)
        );
        let mut work = CoreUpdates {
            cpu: self,
            clock,
            bus,
        };
        let hud = work.read_dp(0x16) != 0;
        work.branch(!hud);
        if hud {
            work.read_x(0x219);
            work.stx(0x2116);
            work.ldx(0xc700);
            work.stx(0x4302);
            work.lda(0x7e);
            work.store(0x4304, 0x7e);
            work.ldx(0x14a);
            work.stx(0x4305);
            work.lda(1);
            work.maintenance_dma(1, MaintenanceKind::Hud);
        }
        let palette = work.read_dp(0x15) != 0;
        work.branch(!palette);
        if palette {
            work.store(0x2121, 0);
            work.ldy(0x2200);
            work.sty(0x4310);
            work.ldy(0xc500);
            work.sty(0x4312);
            work.lda(0x7e);
            work.store(0x4314, 0x7e);
            work.ldy(0x200);
            work.sty(0x4315);
            work.lda(2);
            work.maintenance_dma(2, MaintenanceKind::Palette);
        }
        work.fetch();
        work.drain(8);
        work.drain(6); // REP #$20
        work.cpu.p &= !ACCUMULATOR_BYTE;
        work.fetch();
        work.drain(8);
        work.drain(6); // SEP #$10
        work.cpu.p |= INDEX_BYTE;
        work.cpu.x &= 0xff;
        work.cpu.y &= 0xff;
        work.fetch();
        work.drain(8); // STZ $15, M16: one WRAM word transaction
        (work.bus)(work.clock, CoreAccess::WriteWord(0x15, 0));
        work.drain(16);
        work.store_word(0x2102, 0);
        work.lda_word(0x400);
        work.store_word(0x4300, work.cpu.a);
        work.lda_word(0x800);
        work.store_word(0x4302, work.cpu.a);
        work.store_word(0x4304, 0);
        work.lda_word(0x220);
        work.store_word(0x4305, work.cpu.a);
        work.fetch();
        work.drain(8); // LDY #$01, X8
        work.cpu.y = 1;
        work.byte_flags(1);
        work.maintenance_dma(work.cpu.y as u8, MaintenanceKind::Oam);
        work.cpu.pc = 0x8bcd;
        MaintenanceResult { hud, palette }
    }

    /// Follow live update requests and return through the caller's stack.
    /// Tilemap/packet requests and other dispatch callees still require owners.
    pub(crate) fn initial_updates_return(
        &mut self,
        clock: &mut CpuMasterTimeline,
        bus: impl FnMut(&mut CpuMasterTimeline, CoreAccess) -> u16,
    ) -> u64 {
        assert_eq!(
            (self.bank, self.pc, self.db, self.d, self.p & 0x30),
            (0, 0x8bcd, 0, 0, INDEX_BYTE)
        );
        let mut work = CoreUpdates {
            cpu: self,
            clock,
            bus,
        };
        work.fetch();
        work.drain(8);
        work.drain(6); // SEP #$30
        work.cpu.p |= ACCUMULATOR_BYTE | INDEX_BYTE;
        let stripe = work.read_dp_operand(0x14);
        work.cpu.y = u16::from(stripe);
        work.byte_flags(stripe);
        work.branch(stripe == 0);
        let mut stripe_cycles = 0;
        if stripe != 0 {
            work.cpu.pc = 0x8bd3;
            let mut upload = super::nmi_stripes::StripeUpload::default();
            while let Some(cycles) = upload.step(work.cpu, work.clock,
                |clock, access| (work.bus)(clock, CoreAccess::Stripe(access))) {
                stripe_cycles += cycles;
            }
        }
        assert_eq!(
            work.read_dp(0x19),
            0,
            "native initial NMI has no tilemap continuation"
        );
        work.branch(true);
        let packets = work.read_dp_operand(0x18);
        work.cpu.x = u16::from(packets);
        work.byte_flags(packets);
        assert_eq!(packets, 0, "native initial NMI has no packet continuation");
        work.branch(true);
        let dispatch = work.read_dp(0x17);
        assert!(matches!(dispatch, 0 | 11), "native NMI has no owner for dispatch {dispatch}");
        work.fetch();
        work.drain(6); // ASL A
        work.cpu.p = (work.cpu.p & !CARRY) | (dispatch >> 7);
        work.cpu.load_a_byte(dispatch.wrapping_shl(1));
        work.fetch();
        work.drain(6); // TAX, X8
        work.cpu.x = u16::from(work.cpu.a as u8);
        work.byte_flags(work.cpu.x as u8);
        work.fetch();
        work.drain(8); // STZ $17
        (work.bus)(work.clock, CoreAccess::WriteByte(0x17, 0));
        work.drain(8);
        // JMP ($8C7E,X), through the routine's immutable dispatch table.
        work.fetch();
        work.drain(16);
        work.drain(6);
        work.drain(16);
        if dispatch == 11 {
            // NMI_UpdateBGCharHalf ($8F45): live target page, WRAM source.
            work.read_a(0x116);
            work.store(0x2117, work.cpu.a as u8);
            work.fetch();
            work.drain(8);
            work.drain(6); // REP #$10
            work.cpu.p &= !INDEX_BYTE;
            work.ldx(0x80);
            work.stx(0x2115);
            work.ldx(0x1801);
            work.stx(0x4300);
            work.ldx(0x1000);
            work.stx(0x4302);
            work.lda(0x7f);
            work.store(0x4304, work.cpu.a as u8);
            work.ldx(0x400);
            work.stx(0x4305);
            work.lda(1);
            work.maintenance_dma(1, MaintenanceKind::BackgroundCharacters);
            work.fetch();
            work.drain(8);
            work.drain(6); // SEP #$10
            work.cpu.p |= INDEX_BYTE;
            work.cpu.x &= 0xff;
            work.cpu.y &= 0xff;
        }
        work.fetch();
        work.drain(12); // RTS
        let address = work.cpu.sp.wrapping_add(1);
        let returned = (work.bus)(work.clock, CoreAccess::ReadWord(address));
        work.drain(16);
        work.cpu.sp = work.cpu.sp.wrapping_add(2);
        work.drain(6);
        work.cpu.pc = returned.wrapping_add(1);
        // The previous fixed return cost included the taken stripe-gate BEQ.
        302 + if stripe == 0 { 22 } else { 16 + stripe_cycles }
            + if dispatch == 11 { 480 - 42 } else { 0 }
    }

    /// Execute $89E0 through $8B67. DMA consumes the live programmed channel
    /// operands before any later instruction can change them. The continuation
    /// at $8B67 still owns this same CPU context and timeline.
    pub(crate) fn core_graphics_updates(
        &mut self,
        clock: &mut CpuMasterTimeline,
        bus: impl FnMut(&mut CpuMasterTimeline, CoreAccess) -> u16,
    ) -> CoreResult {
        assert_eq!(
            (
                self.bank,
                self.pc,
                self.db,
                self.d,
                self.p & ACCUMULATOR_BYTE
            ),
            (0, 0x89e0, 0, 0, ACCUMULATOR_BYTE)
        );
        let mut work = CoreUpdates {
            cpu: self,
            clock,
            bus,
        };
        work.fetch();
        work.drain(8);
        work.drain(6); // REP #$10
        work.cpu.p &= !INDEX_BYTE;
        work.lda(0x80);
        work.store(0x2115, 0x80);
        let enabled = work.read_a(0x710) == 0;
        work.branch(enabled);
        if !enabled {
            work.fetch();
            work.drain(16); // JMP $8B67
            work.cpu.pc = 0x8b67;
            return CoreResult {
                enabled,
                travel_bird: false,
            };
        }
        work.ldx(0x1801);
        for address in [0x4300, 0x4310, 0x4320, 0x4330, 0x4340] {
            work.stx(address);
        }
        work.lda(0x10);
        for address in [0x4304, 0x4314, 0x4324] {
            work.store(address, work.cpu.a as u8);
        }
        work.ldy(0x4100);
        work.sty(0x2116);
        work.read_y(0xace);
        work.sty(0x4302);
        work.ldx(0x40);
        work.stx(0x4305);
        work.read_y(0xad2);
        work.sty(0x4312);
        work.stx(0x4315);
        work.read_y(0xad6);
        work.sty(0x4322);
        work.ldy(0x20);
        work.sty(0x4325);
        work.lda(7);
        work.dma();
        work.sty(0x4325);
        work.ldy(0x4000);
        work.sty(0x2116);
        work.read_y(0xacc);
        work.sty(0x4302);
        work.stx(0x4305);
        work.read_y(0xad0);
        work.sty(0x4312);
        work.stx(0x4315);
        work.read_y(0xad4);
        work.sty(0x4322);
        work.dma();
        work.lda(0x7e);
        for address in [0x4304, 0x4314, 0x4324, 0x4334, 0x4344] {
            work.store(address, 0x7e);
        }
        work.read_y(0xac0);
        work.sty(0x4302);
        work.stx(0x4305);
        work.read_y(0xac4);
        work.sty(0x4312);
        work.stx(0x4315);
        work.read_y(0xac8);
        work.sty(0x4322);
        work.stx(0x4325);
        work.read_y(0xae0);
        work.sty(0x4332);
        work.ldy(0x20);
        work.sty(0x4335);
        work.read_y(0xad8);
        work.sty(0x4342);
        work.stx(0x4345);
        work.lda(0x1f);
        work.dma();
        work.ldy(0x4150);
        work.sty(0x2116);
        work.read_y(0xac2);
        work.sty(0x4302);
        work.stx(0x4305);
        work.read_y(0xac6);
        work.sty(0x4312);
        work.stx(0x4315);
        work.read_y(0xaca);
        work.sty(0x4322);
        work.stx(0x4325);
        work.read_y(0xae2);
        work.sty(0x4332);
        work.ldy(0x20);
        work.sty(0x4335);
        work.read_y(0xada);
        work.sty(0x4342);
        work.stx(0x4345);
        work.dma();
        work.ldy(0x4200);
        work.sty(0x2116);
        work.read_y(0xaec);
        work.sty(0x4302);
        work.stx(0x4305);
        work.read_y(0xaf0);
        work.sty(0x4312);
        work.stx(0x4315);
        work.ldy(0xbd40);
        work.sty(0x4322);
        work.stx(0x4325);
        work.lda(7);
        work.dma();
        work.ldy(0x4300);
        work.sty(0x2116);
        work.read_y(0xaee);
        work.sty(0x4302);
        work.stx(0x4305);
        work.read_y(0xaf2);
        work.sty(0x4312);
        work.stx(0x4315);
        work.ldy(0xbd80);
        work.sty(0x4322);
        work.stx(0x4325);
        work.dma();
        let travel_bird = work.read_a(0xaf4) != 0;
        work.branch(!travel_bird);
        if travel_bird {
            work.ldy(0x40e0);
            work.sty(0x2116);
            work.read_y(0xaf6);
            work.sty(0x4302);
            work.stx(0x4305);
            work.lda(1);
            work.dma();
            work.ldy(0x41e0);
            work.sty(0x2116);
            work.read_y(0xaf8);
            work.sty(0x4302);
            work.stx(0x4305);
            work.dma();
        }
        work.read_x(0xadc);
        work.stx(0x4302);
        work.read_x(0x134);
        work.stx(0x2116);
        work.ldx(0x400);
        work.stx(0x4305);
        work.lda(1);
        work.fetch();
        work.drain(16);
        (work.bus)(work.clock, CoreAccess::AnimatedDma(1));
        work.cpu.pc = 0x8b67;
        CoreResult {
            enabled,
            travel_bird,
        }
    }
}

struct CoreUpdates<'a, F> {
    cpu: &'a mut NativeNmiContext,
    clock: &'a mut CpuMasterTimeline,
    bus: F,
}

#[cfg(test)]
mod tests {
    use super::*;
    use snes::{
        CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, Snes9xColdCpuExecutor,
        SourceCpuBusAccessKind,
    };

    fn rom() -> Option<Vec<u8>> {
        std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .ok()
    }

    #[test]
    fn core_graphics_updates_match_rom_bus_hardware_and_clock() {
        let Some(rom) = rom() else { return };
        for disabled in [0, 1, 0xff] {
            for bird in [0, 0x80] {
                for position in [(225, 1146), (225, 510), (240, 1350), (261, 1300)] {
                    for odd in [false, true] {
                        compare_case(&rom, position, odd, Some((disabled, bird)), 0, 0, (1, 0x18), 0, 0);
                    }
                }
            }
        }
    }

    #[test]
    fn maintenance_and_initial_return_match_rom_bus_hardware_and_clock() {
        let Some(rom) = rom() else { return };
        for hud in [0, 1, 0xff] {
            for palette in [0, 1, 0x80] {
                for channel in [(1, 0x18), (0, 0x22), (0, 0x04)] {
                    for position in [(242, 1346), (225, 510), (261, 1300)] {
                        for odd in [false, true] {
                            compare_case(&rom, position, odd, None, hud, palette, channel, 0, 0);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn background_half_dispatch_matches_rom_bus_hardware_and_clock() {
        let Some(rom) = rom() else { return };
        for page in [0, 0x20, 0x3e, 0xff] {
            for palette in [0, 1] {
                for position in [(225, 510), (240, 1350), (261, 1300)] {
                    for odd in [false, true] {
                        compare_case(&rom, position, odd, None, 0, palette, (1, 0x18), 11, page);
                    }
                }
            }
        }
    }

    fn compare_case(
        rom: &[u8],
        (v, h): (u16, u16),
        odd: bool,
        core: Option<(u8, u8)>,
        hud: u8,
        palette: u8,
        (mode, target): (u8, u8),
        dispatch: u8,
        page: u8,
    ) {
        let mut rom = rom.to_vec();
        // Test-only setup establishes actual I/O state and parks the CPU.
        // The original NMI routine and its table are left untouched.
        let program = [
            0x18, 0xfb, 0xa9, 0x80, 0x8d, 0, 0x21, 0xa9, mode, 0x8d, 0, 0x43, 0xa9, target, 0x8d,
            1, 0x43, 0xa9, 0x80, 0x8d, 0x15, 0x21, 0x80, 0xfe,
        ];
        rom[..program.len()].copy_from_slice(&program);
        rom[0x7ffc..0x7ffe].copy_from_slice(&0x8000u16.to_le_bytes());
        let mut source = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        let target = CpuMasterTimeline::at_raster(
            u64::from(odd),
            CpuRasterPosition::new(v, h),
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        )
        .clock_master_cycles();
        while source.machine().timestamp().master_cycles() < target {
            source.step().unwrap();
        }
        let mut ram = source.machine().snes().ram.clone();
        for (i, b) in ram.iter_mut().enumerate() {
            *b = (i ^ (i >> 8)) as u8;
        }
        let (disabled, bird) = core.unwrap_or((1, 0));
        ram[0x710] = disabled;
        ram[0xaf4] = bird;
        for address in (0xac0..0xafa).step_by(2).filter(|a| *a != 0xaf4) {
            let value: u16 = if odd { 0x9000 + address as u16 } else { 0x100 };
            ram[address..address + 2].copy_from_slice(&value.to_le_bytes());
        }
        ram[0x134..0x136].copy_from_slice(&0x3c00u16.to_le_bytes());
        ram[0x219..0x21b].copy_from_slice(&0x6000u16.to_le_bytes());
        ram[0x14..0x1a].fill(0);
        ram[0x15] = palette;
        ram[0x16] = hud;
        ram[0x17] = dispatch;
        ram[0x116] = page;
        ram[0x1f1..0x1f3].copy_from_slice(&0x8140u16.to_le_bytes());
        let mut native = NativeNmiContext {
            a: 0xab00,
            x: 23,
            y: 47,
            d: 0,
            db: 0,
            bank: 0,
            pc: if core.is_some() { 0x89e0 } else { 0x8b67 },
            sp: 0x1f0,
            p: if core.is_some() { 0x75 } else { 0x65 },
        };
        let mut registers = source.machine().snes().cpu.clone();
        registers.a = native.a;
        registers.x = native.x;
        registers.y = native.y;
        registers.pc = native.pc;
        registers.sp = native.sp;
        registers.dp = 0;
        registers.db = 0;
        registers.k = 0;
        registers.e = false;
        registers.unpack_flags(native.p);
        let at = source.machine().timestamp();
        let sram = source.machine().snes().cart.ram.clone();
        source
            .rebase_private_timing_state_at(at, &registers, &ram, &sram)
            .unwrap();
        let mut ppu = source.machine().snes().ppu.clone();
        let mut dma = source.machine().snes().dma.clone();
        let mut clock = source.machine().timeline().clone();
        let end = if core.is_some() { 0x8b67 } else { 0x8141 };
        let mut expected = Vec::new();
        while source.machine().snes().cpu.pc != end {
            for access in source.step().unwrap().accesses {
                let a = access.address as u16;
                let op = match access.kind {
                    SourceCpuBusAccessKind::Read { width: 1, .. } if access.address < 0x2000 => {
                        CoreAccess::ReadByte(a)
                    }
                    SourceCpuBusAccessKind::Read { width: 2, .. } if access.address < 0x2000 => {
                        CoreAccess::ReadWord(a)
                    }
                    SourceCpuBusAccessKind::Write { value, width: 2 }
                        if access.address < 0x2000 =>
                    {
                        CoreAccess::WriteWord(a, value)
                    }
                    SourceCpuBusAccessKind::Write { value, width: 1 } if a == 0x420b => {
                        CoreAccess::Dma(value as u8)
                    }
                    SourceCpuBusAccessKind::Write { value, width: 1 }
                        if access.address < 0x2000
                            || (0x2100..0x2140).contains(&a)
                            || (0x4300..0x4380).contains(&a) =>
                    {
                        CoreAccess::WriteByte(a, value as u8)
                    }
                    _ => continue,
                };
                expected.push((access.timestamp.master_cycles(), op));
            }
        }
        let mut actual = Vec::new();
        let mut bus = |clock: &mut CpuMasterTimeline, access| {
            actual.push((
                clock.clock_master_cycles(),
                match access {
                    CoreAccess::AnimatedDma(mask) | CoreAccess::MaintenanceDma(mask, _) => {
                        CoreAccess::Dma(mask)
                    }
                    other => other,
                },
            ));
            match access {
                CoreAccess::Stripe(_) => unreachable!("this fixture has no stripe request"),
                CoreAccess::ReadByte(a) => u16::from(ram[usize::from(a)]),
                CoreAccess::ReadWord(a) => {
                    u16::from_le_bytes([ram[usize::from(a)], ram[usize::from(a) + 1]])
                }
                CoreAccess::WriteWord(a, value) => {
                    ram[usize::from(a)..usize::from(a) + 2].copy_from_slice(&value.to_le_bytes());
                    0
                }
                CoreAccess::WriteByte(a, value) => {
                    if a < 0x2000 {
                        ram[usize::from(a)] = value
                    } else if a >= 0x4300 {
                        dma.write_register(a, value)
                    } else {
                        ppu.write(a as u8, value)
                    };
                    0
                }
                CoreAccess::Dma(mask)
                | CoreAccess::AnimatedDma(mask)
                | CoreAccess::MaintenanceDma(mask, _) => {
                    dma.run_synchronous_forward_ppu(clock, mask, |_, _, a, b| {
                        let bank = a >> 16;
                        let low = a as u16;
                        let value = if bank == 0x7e || bank == 0x7f {
                            ram[(a - 0x7e0000) as usize]
                        } else if bank & 0x40 == 0 && low < 0x2000 {
                            ram[usize::from(low)]
                        } else {
                            rom[(((bank & 0x7f) << 15) | u32::from(low & 0x7fff)) as usize]
                        };
                        ppu.write(b, value);
                    });
                    0
                }
            }
        };
        if core.is_some() {
            native.core_graphics_updates(&mut clock, &mut bus);
        } else {
            let result = native.display_maintenance_updates(&mut clock, &mut bus);
            assert_eq!((result.hud, result.palette), (hud != 0, palette != 0));
            assert_eq!(native.initial_updates_return(&mut clock, &mut bus),
                324 + if dispatch == 11 { 438 } else { 0 });
        }
        let label =
            format!("core={core:?} hud={hud} palette={palette} mode={mode} dispatch={dispatch} page={page} v={v} h={h} odd={odd}");
        assert_eq!(actual, expected, "{label}");
        assert_eq!(clock, *source.machine().timeline(), "{label}");
        assert_eq!(ram, source.machine().snes().ram, "{label}");
        assert_eq!(ppu.vram, source.machine().snes().ppu.vram, "{label}");
        assert_eq!(ppu.cgram, source.machine().snes().ppu.cgram, "{label}");
        assert_eq!(ppu.oam, source.machine().snes().ppu.oam, "{label}");
        assert_eq!(
            (
                ppu.vram_pointer,
                ppu.cgram_pointer,
                ppu.cgram_second_write,
                ppu.oam_adr,
                ppu.oam_second_write
            ),
            (
                source.machine().snes().ppu.vram_pointer,
                source.machine().snes().ppu.cgram_pointer,
                source.machine().snes().ppu.cgram_second_write,
                source.machine().snes().ppu.oam_adr,
                source.machine().snes().ppu.oam_second_write
            ),
            "{label}"
        );
        assert_eq!(
            dma.save_c_saveload(),
            source.machine().snes().dma.save_c_saveload(),
            "{label}"
        );
        let cpu = &source.machine().snes().cpu;
        assert_eq!(
            (
                native.a,
                native.x,
                native.y,
                native.p,
                native.pc,
                native.sp,
                native.d,
                native.db,
                native.bank
            ),
            (
                cpu.a,
                cpu.x,
                cpu.y,
                cpu.pack_flags(),
                cpu.pc,
                cpu.sp,
                cpu.dp,
                cpu.db,
                cpu.k
            ),
            "{label}"
        );
    }
}

impl<F: FnMut(&mut CpuMasterTimeline, CoreAccess) -> u16> CoreUpdates<'_, F> {
    fn byte_flags(&mut self, value: u8) {
        self.cpu.p = (self.cpu.p & !(ZERO | NEGATIVE))
            | if value == 0 { ZERO } else { 0 }
            | (value & NEGATIVE);
    }
    fn read_dp_operand(&mut self, address: u16) -> u8 {
        self.fetch();
        self.drain(8);
        let value = (self.bus)(self.clock, CoreAccess::ReadByte(address)) as u8;
        self.drain(8);
        value
    }
    fn read_dp(&mut self, address: u16) -> u8 {
        let value = self.read_dp_operand(address);
        self.cpu.load_a_byte(value);
        value
    }
    fn lda_word(&mut self, value: u16) {
        self.fetch();
        self.drain(16);
        self.cpu.a = value;
        self.index_flags(value);
    }
    fn maintenance_dma(&mut self, mask: u8, kind: MaintenanceKind) {
        self.fetch();
        self.drain(16);
        (self.bus)(self.clock, CoreAccess::MaintenanceDma(mask, kind));
    }
    fn fetch(&mut self) {
        self.clock.advance_synchronous_pcbase_opcode_fetch(8);
    }
    fn drain(&mut self, cycles: u32) {
        self.clock
            .advance_synchronous_after_semantics_with(cycles, |_, _| Ok::<_, String>(0))
            .unwrap();
    }
    fn branch(&mut self, taken: bool) {
        self.fetch();
        self.drain(8);
        if taken {
            self.drain(6);
        }
    }
    fn lda(&mut self, value: u8) {
        self.fetch();
        self.drain(8);
        self.cpu.load_a_byte(value);
    }
    fn index_flags(&mut self, value: u16) {
        self.cpu.p = (self.cpu.p & !(ZERO | NEGATIVE))
            | if value == 0 { ZERO } else { 0 }
            | ((value >> 8) as u8 & NEGATIVE);
    }
    fn ldx(&mut self, value: u16) {
        self.fetch();
        self.drain(16);
        self.cpu.x = value;
        self.index_flags(value);
    }
    fn ldy(&mut self, value: u16) {
        self.fetch();
        self.drain(16);
        self.cpu.y = value;
        self.index_flags(value);
    }
    fn read_a(&mut self, address: u16) -> u8 {
        self.fetch();
        self.drain(16);
        let value = (self.bus)(self.clock, CoreAccess::ReadByte(address)) as u8;
        self.drain(8);
        self.cpu.load_a_byte(value);
        value
    }
    fn read_word(&mut self, address: u16) -> u16 {
        self.fetch();
        self.drain(16);
        let value = (self.bus)(self.clock, CoreAccess::ReadWord(address));
        self.drain(16);
        self.index_flags(value);
        value
    }
    fn read_x(&mut self, address: u16) {
        self.cpu.x = self.read_word(address);
    }
    fn read_y(&mut self, address: u16) {
        self.cpu.y = self.read_word(address);
    }
    fn store(&mut self, address: u16, value: u8) {
        self.fetch();
        self.drain(16);
        (self.bus)(self.clock, CoreAccess::WriteByte(address, value));
        self.drain(6);
    }
    fn store_word(&mut self, address: u16, value: u16) {
        self.fetch();
        self.drain(16);
        for (i, byte) in value.to_le_bytes().into_iter().enumerate() {
            (self.bus)(self.clock, CoreAccess::WriteByte(address + i as u16, byte));
            self.drain(6);
        }
    }
    fn stx(&mut self, address: u16) {
        self.store_word(address, self.cpu.x);
    }
    fn sty(&mut self, address: u16) {
        self.store_word(address, self.cpu.y);
    }
    fn dma(&mut self) {
        self.fetch();
        self.drain(16);
        (self.bus)(self.clock, CoreAccess::Dma(self.cpu.a as u8));
    }
}
