//! Main-only HDMA stalls for a leased immutable descriptor program.
//! No payload values, scene identities or source lag counts price the bus.
use super::*;

const FIELD: u64 = 1364 * 262;
const NMI: u64 = 225 * 1364 + 86;
type ChannelProgram = (u8, u8, u16, u8, bool, u8, bool);

fn leading_handler_may_miss_init(handler: u64) -> bool {
    // $00:8135 disables HDMA until $00:8222 (or $82C4). If that interval
    // reaches V0's initialization, enabling later retains the old cursor;
    // it cannot use this composer's freshly initialized per-field program.
    // Ceil refresh is a conservative placement envelope; the full handler
    // includes its short post-$420C return suffix, so ambiguous edges decline.
    let init = FIELD - NMI + u64::from(snes::cpu_timeline::HDMA_INIT_CYCLE);
    handler + handler.div_ceil(1324) * 40 >= init
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Programming {
    mask: u8,
    channels: [Option<ChannelProgram>; 8],
}

impl Programming {
    pub(super) fn read(game: &ZeldaState) -> Self {
        let mask = game.game_state.display.hdma_enable_mask;
        Self { mask, channels: std::array::from_fn(|i| {
            let c = &game.dma.channel[i];
            (mask & (1 << i) != 0).then_some((c.mode, c.b_adr, c.a_adr,
                c.a_bank, c.indirect, c.ind_bank, c.from_b))
        }) }
    }
}

#[derive(Clone, Debug)]
pub(super) struct Program {
    pub(super) programming: Programming,
    events: Vec<(u64, u64)>,
    total: u64,
}

impl Program {
    pub(super) fn read(game: &ZeldaState, programming: Programming) -> Option<Self> {
        if programming.mask == 0 || programming.channels.iter().flatten().any(|c| c.0 > 7) {
            return None;
        }
        let dma = game.dma_with_native_hdma_enable();
        let mut bus = snes::HdmaTimingState::new(&dma);
        let mut events = Vec::new();
        // $420C / $43x0..$43xA: use the hardware's real line counters,
        // transfer modes, indirect reloads and termination rules. Only the
        // existing immutable ROM descriptor byte owners are admitted.
        for (line, event, cycle) in std::iter::once((0u64, snes::CpuBusEvent::HdmaInit,
                u64::from(snes::cpu_timeline::HDMA_INIT_CYCLE)))
            .chain((0..225).map(|line| (line, snes::CpuBusEvent::HdmaStart,
                u64::from(snes::cpu_timeline::HDMA_START_CYCLE)))) {
            let stall = bus.stall(event, |address|
                ZeldaState::native_hdma_descriptor_byte(address & 0x7fffff))?;
            if stall != 0 {
                events.push(((line * 1364 + cycle + FIELD - NMI) % FIELD, u64::from(stall)));
            }
        }
        events.sort_unstable();
        let total = events.iter().map(|(_, clocks)| clocks).sum();
        Some(Self { programming, events, total })
    }

    fn occurrences(end: u64, first: u64) -> u64 {
        if end < first { 0 } else { 1 + (end - first) / FIELD }
    }

    fn main_stalls(&self, start: u64, end: u64, handler: u64, worker: u64, after_irq: bool) -> u64 {
        let repeated = (handler + worker).min(FIELD - 1);
        let offset = if after_irq && worker != 0 { FIELD - worker } else { 0 };
        let mut next = start / FIELD * FIELD + offset;
        if next <= start { next += FIELD; }
        self.events.iter().map(|&(at, clocks)| {
            let mut count = Self::occurrences(end, at) - Self::occurrences(start, at);
            let in_block = (at + FIELD - offset) % FIELD;
            // A handler/worker owns its ending event drain too. Events in
            // that wall interval must never be charged again to main.
            if repeated != 0 && in_block <= repeated {
                count -= Self::occurrences(end, next + in_block);
            }
            count * clocks
        }).sum()
    }

    pub(super) fn advance(&self, main: u64, start: u64, handler: u64,
        worker: u64, after_irq: bool) -> Option<(u64, u64)> {
        self.advance_segment(main, start, handler, worker, after_irq, 0)
    }

    fn advance_segment(&self, main: u64, start: u64, handler: u64,
        worker: u64, after_irq: bool, previous_work: u64) -> Option<(u64, u64)> {
        // A generic capacity proof, not an iteration/time cap: even the
        // whole field's bus work plus its induced refresh must leave main
        // positive capacity under the retained interrupt ownership.
        if (self.total * 1364).div_ceil(1324) >= FIELD - (handler + worker).min(FIELD - 1) {
            return None;
        }
        let mut hdma = 0;
        loop {
            let work = main + hdma;
            let refresh = ((previous_work + work).div_ceil(1324)
                - previous_work.div_ceil(1324)) * 40;
            let end = advance_main_cost(work + refresh,
                start, handler, worker, after_irq);
            let next = self.main_stalls(start, end, handler, worker, after_irq);
            if next == hdma { return Some((end, hdma)); }
            debug_assert!(next > hdma, "positive-capacity HDMA closure is monotone");
            hdma = next;
        }
    }

    pub(super) fn compose(&self, main: u64, start: u64, handler: u64,
        worker: u64, after_irq: bool, barriers: &[(u64, u16, u16, u64, u64)]) -> Option<(u64, u64, u64)> {
        let (mut wall, mut previous, mut hdma, mut waits) = (start, 0, 0, 0);
        // $00:F3A3..F3B2 polls the V-counter until its target. The barrier's
        // wall interval already includes HDMA and refresh. Only the CPU-work
        // intervals on either side lease bus events; preserve refresh's
        // cumulative work counter when splitting them.
        for &(end, line, mask, sample_master, loop_master) in barriers {
            if end > main { break; }
            let (next, bus) = self.advance_segment(end - previous, wall,
                handler, worker, after_irq, previous + hdma)?;
            hdma += bus;
            wall = next;
            let wait = raster_poll_wait(wall, line, mask, sample_master, loop_master);
            wall += wait;
            waits += wait;
            previous = end;
        }
        let (wall, bus) = self.advance_segment(main - previous, wall,
            handler, worker, after_irq, previous + hdma)?;
        Some((wall, hdma + bus, waits))
    }
}

impl ZeldaState {
    pub(super) fn native_begin_bus_lease(&mut self) {
        let programming = Programming::read(self);
        if self.native_pacing.bus_program.as_ref().map(|p| &p.programming) != Some(&programming) {
            self.native_pacing.bus_program = Program::read(self, programming);
        }
        self.native_pacing.bus_epoch = crate::cycle_ledger::hdma_program_epoch();
        self.native_pacing.finished_hdma = 0;
        self.native_pacing.bus_admitted = false;
    }

    pub(super) fn native_bus_lease_valid(&self) -> bool {
        self.native_bus_lease_decline() == 0
    }

    /// Diagnostic reasons only. Bits may overlap: one iteration can have
    /// mutable descriptors, a program write and a separate timed owner.
    /// No source lag count or structural residual participates in admission.
    pub(super) fn native_bus_lease_decline(&self) -> u16 {
        // VWF's $0E:C9FD instruction work already enters native_main_work once.
        // DMA, uploads, worker changes and program changes still decline.
        // Retained raster barriers compose with the immutable lease.
        let programming = Programming::read(self);
        (u16::from(self.native_pacing.main_dma != 0) << 1)
            | (u16::from(self.native_pacing.song_upload) << 2)
            | (u16::from(!self.native_pacing.worker_segments.is_empty()) << 4)
            | (u16::from(self.game_execution_scheduler.current_work().is_some()) << 5)
            | (u16::from(self.native_pacing.bus_epoch != crate::cycle_ledger::hdma_program_epoch()) << 6)
            | (u16::from(self.native_pacing.bus_program.is_none() && programming.mask != 0) << 7)
            | (u16::from(self.native_pacing.bus_program.as_ref().is_some_and(|program|
                program.programming != programming)) << 8)
            | (u16::from(programming.mask == 0 && self.native_pacing.bus_program.is_none()) << 9)
            | (u16::from(leading_handler_may_miss_init(self.native_pacing.entry_handler)) << 10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game() -> ZeldaState {
        let mut game = ZeldaState::default();
        game.set_lag_free(true);
        game.hdma_setup(0x00f2fb, 0x00f2fb, 0x42, 0x26, 0x28, 0x7e);
        game.set_hdma_enable_mask(0xc0);
        game
    }

    #[test]
    fn leading_nmi_must_establish_the_field_initialization() {
        let mut game = game();
        game.native_begin_main_cost();
        game.native_pacing.entry_handler = 94_000;
        assert_eq!(game.native_bus_lease_decline(), 1 << 10);
        // Even an instruction-only handler ending before V0 can span init
        // once refresh is included. The guard follows the hardware event.
        game.native_pacing.entry_handler = 49_500;
        assert_eq!(game.native_bus_lease_decline(), 1 << 10);
        game.native_pacing.entry_handler = 28_520;
        assert_eq!(game.native_bus_lease_decline(), 0);
        // Replay the actual DMA device's disable/init/late-enable sequence.
        // No descriptors are fetched at the disabled init; late HDMAEN does
        // not reset A2A/NTRL/DAS or perform the missing first transfer.
        let mut late = game.dma_with_native_hdma_enable();
        for c in &mut late.channel { c.hdma_active = false; }
        assert_eq!(late.run_synchronous_hdma(snes::CpuBusEvent::HdmaInit,
            |_| panic!("disabled initialization fetched a descriptor")), 0);
        for i in [6, 7] { late.channel[i].hdma_active = true; late.channel[i].rep_count = 73; }
        assert_eq!(late.run_synchronous_hdma(snes::CpuBusEvent::HdmaStart,
            |_| panic!("late enable unexpectedly reset the descriptor cursor")), 34);
        for i in [6, 7] { assert_eq!(late.channel[i].rep_count, 72); }
        let mut fresh = game.dma_with_native_hdma_enable();
        let read = |access| match access {
            snes::dma::HdmaBusAccess::Read(address) => ZeldaState::native_hdma_descriptor_byte(address).unwrap_or(0),
            snes::dma::HdmaBusAccess::Write(_, _) => 0,
        };
        assert_eq!(fresh.run_synchronous_hdma(snes::CpuBusEvent::HdmaInit, read), 66);
        assert_eq!(fresh.run_synchronous_hdma(snes::CpuBusEvent::HdmaStart, read), 66);
    }

    #[test]
    fn window_retarget_preserves_the_executed_rom_descriptor_owner() {
        use snes::{CartType, CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline,
            CpuRasterPosition, RomCpuTimingProbe, Snes, SourcePpuReadState};
        let rom = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc")).unwrap();
        for table in [0x0abddd, 0x00f2fb] {
          for entry in [0xb260, 0x9d5d] {
            let mut game = game();
            game.hdma_setup(0, table, 0x42, 0, 0x1e, 0);
            game.set_hdma_enable_mask(0x80);
            // Retargeting mid-field must preserve the cursor as well as A1T.
            game.dma.channel[7].table_adr = table as u16 + 3;
            game.dma.channel[7].rep_count = 73;
            game.dma.channel[7].size = 0x1b42;
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.dma = game.dma.clone();
            machine.cpu.e = false;
            machine.cpu.unpack_flags(0x30);
            machine.cpu.k = 2;
            machine.cpu.db = 0;
            machine.cpu.pc = entry;
            let clock = CpuMasterTimeline::at_raster(0, CpuRasterPosition::new(230, 0),
                CpuBusWorkload::with_dynamic_hdma(), CpuFieldTiming::non_interlace(false));
            let mut source = RomCpuTimingProbe::new(machine, clock,
                SourcePpuReadState::snes9x_reset()).unwrap();
            // REP, LDA #$2641, STA $4370 in each real producer.
            for _ in 0..3 { source.step().unwrap(); }
            game.hdma_retarget_channel7_window();
            assert_eq!(game.dma.save_c_saveload(), source.snes().dma.save_c_saveload());
            let program = Program::read(&game, Programming::read(&game)).unwrap();
            // F0 descriptors terminate after 224 transfers. F8 descriptors
            // remain live for line 224: 42 transfer clocks replace an 8-clock
            // terminal indirect fetch, a net 34 clocks per complete field.
            assert_eq!(program.total, if table == 0x0abddd { 9474 } else { 9508 });
          }
        }
    }

    #[test]
    fn stable_program_stalls_follow_executed_rom_bus_events() {
        use snes::{CartType, CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline,
            CpuRasterPosition, CpuSynchronousTimelineEvent, RomCpuTimingProbe, Snes, SourcePpuReadState};
        let rom = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc")).unwrap();
        for pointers in [(0x00f2fb, 0x00f2fb), (0x0abdcf, 0x0abdd6)] {
          let mut game = game();
          game.hdma_setup(pointers.0, pointers.1, 0x42, 0x26, 0x28, 0x7e);
          let program = Program::read(&game, Programming::read(&game)).unwrap();
          let largest_event = program.events.iter().map(|r| r.1).max().unwrap();
          for target in [1324, 50_000, 200_000, 400_000, 2 * FIELD + 100_000] {
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.dma = game.dma_with_native_hdma_enable();
            machine.ram[0x12] = 0; // $00:8034's real LDA/BEQ wait loop.
            machine.cpu.e = false;
            machine.cpu.unpack_flags(0x30);
            machine.cpu.k = 0;
            machine.cpu.db = 0;
            machine.cpu.pc = 0x8034;
            machine.cpu.sp = 0x1fc;
            let clock = CpuMasterTimeline::at_raster(0, CpuRasterPosition::new(225, 86),
                CpuBusWorkload::with_dynamic_hdma(), CpuFieldTiming::non_interlace(false));
            let mut source = RomCpuTimingProbe::new(machine, clock,
                SourcePpuReadState::snes9x_reset()).unwrap();
            let mut observer = source.timeline().synchronous_event_observer();
            let (mut cpu, mut refresh, mut wall) = (0u64, 0u64, 0u64);
            while cpu < target {
                let step = source.step().unwrap();
                assert!(step.accepted_interrupt.is_none());
                cpu += step.transactions.iter().map(|t| u64::from(t.duration_master_cycles)).sum::<u64>();
                wall += step.ended_at.master_cycles() - step.started_at.master_cycles();
                source.timeline().observe_processed_synchronous_events(&mut observer, |event| {
                    if event == CpuSynchronousTimelineEvent::Bus(snes::CpuBusEvent::WramRefresh) {
                        refresh += 40;
                    }
                });
            }
            let source_hdma = wall - cpu - refresh;
            let (model_wall, model_hdma) = program.advance(cpu, 0, 0, 0, false).unwrap();
            // P2 retains nominal fields and its phase-agnostic ceil refresh.
            // Across these fields the resulting endpoint uncertainty can
            // move at most one HDMA event, plus one refresh and short lines.
            assert!(model_hdma.abs_diff(source_hdma) <= largest_event,
                "work={cpu} model={model_hdma} source={source_hdma}");
            assert!(model_wall.abs_diff(wall) <= largest_event + 40 + 4 * (wall / FIELD + 2));
            // Exercise the completed native lease with the same executed
            // instruction work, independently of the pure bus helper.
            game.lag_free_frame_master_start = crate::cycle_ledger::master();
            game.lag_free_vwf_master = 0;
            game.native_begin_main_cost();
            game.native_pacing.entry_handler = 0;
            crate::cycle_ledger::charge(cpu);
            game.native_finish_main_cost();
            assert_eq!(game.native_cost_main_hdma(), (model_hdma, true));
            assert_eq!(game.native_cost_shortfall(0).0, model_wall);
            assert_eq!(game.native_pacing.finished_main, cpu);
          }
        }
    }

    #[test]
    fn masked_vcounter_poll_matches_rom_across_the_physical_field_wrap() {
        use snes::{CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline,
            CpuRasterPosition, RomCpuTimingProbe, SourcePpuReadState};
        let rom = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc")).unwrap();
        for line in [0u16, 191, 192, 225, 255, 256, 261] {
          for h in [0u16, 1200, 1320, 1339, 1340, 1363] {
            let mut machine = snes::Snes9xColdCpuExecutor::from_lorom_reset(&rom)
                .unwrap().machine().snes().clone();
            machine.cpu.e = false;
            machine.cpu.unpack_flags(0);
            machine.cpu.db = 0;
            machine.cpu.k = 0;
            machine.cpu.pc = 0xf3a3;
            let clock = CpuMasterTimeline::at_raster(0, CpuRasterPosition::new(line, h),
                CpuBusWorkload::default(), CpuFieldTiming::non_interlace(false));
            let mut source = RomCpuTimingProbe::new(machine, clock,
                SourcePpuReadState::snes9x_reset()).unwrap();
            // M16 LDA $213F also reads $2140; supply its APU bus owner.
            let mut apu = snes::apu::ApuState::new();
            apu.reset_snes9x_coroutine();
            source.attach_apu_port_owner(snes::ApuHostPortTiming::new(
                snes::ApuHostPortProbe::from_snes9x_coroutine(apu).unwrap(),
                snes::Snes9xApuClockCheckpoint::new(source.timeline().clock_master_cycles(), 0, 0).unwrap(),
            ).unwrap()).unwrap();
            let mut wall = 0;
            while source.snes().cpu.pc != 0xf3b4 {
                let step = source.step().unwrap();
                assert!(step.accepted_interrupt.is_none());
                wall += step.ended_at.master_cycles() - step.started_at.master_cycles();
            }
            let elapsed = (u64::from(line) + 262 - 225) % 262 * 1364 + u64::from(h);
            let model = raster_poll_wait(elapsed, 192, 0xff, 24, 178) + 172;
            assert!(model.abs_diff(wall) <= 178 + 40 + 4,
                "V={line} H={h} model={model} source={wall}");
            if line >= 256 { assert!(wall > 192 * 1364); }
          }
        }
    }

    #[test]
    fn raster_barrier_composes_with_executed_rom_without_charging_poll_bus_twice() {
        use snes::{CartType, CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline,
            CpuRasterPosition, CpuSynchronousTimelineEvent, RomCpuTimingProbe, SourcePpuReadState};
        let rom = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc")).unwrap();
        for radius in [0u16, 28, 100, 119] {
            let mut game = game();
            let program = Program::read(&game, Programming::read(&game)).unwrap();
            let event_max = program.events.iter().map(|e| e.1).max().unwrap();
            // Install the CPU-only source device owners, including OAM's
            // byte read at $2138 in the 16-bit LDA $2137 polling pass.
            let mut machine = snes::Snes9xColdCpuExecutor::from_lorom_reset(&rom)
                .unwrap().machine().snes().clone();
            machine.ram.fill(0);
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.dma = game.dma_with_native_hdma_enable();
            machine.ram[0x67c..0x67e].copy_from_slice(&radius.to_le_bytes());
            machine.ram[0x20] = 96;
            machine.ram[0x22] = 128;
            machine.ram[0x10c] = 9; // Avoid the unrelated sprite-reset callee.
            machine.ram[0x1fd..0x200].copy_from_slice(&[0x59, 0x80, 0]);
            machine.cpu.e = false;
            machine.cpu.unpack_flags(0x30);
            machine.cpu.pc = 0xf312;
            machine.cpu.sp = 0x1fc;
            let clock = CpuMasterTimeline::at_raster(0, CpuRasterPosition::new(225, 86),
                CpuBusWorkload::with_dynamic_hdma(), CpuFieldTiming::non_interlace(false));
            let mut source = RomCpuTimingProbe::new(machine, clock,
                SourcePpuReadState::snes9x_reset()).unwrap();
            let mut apu = snes::apu::ApuState::new();
            apu.reset_snes9x_coroutine();
            source.attach_apu_port_owner(snes::ApuHostPortTiming::new(
                snes::ApuHostPortProbe::from_snes9x_coroutine(apu).unwrap(),
                snes::Snes9xApuClockCheckpoint::new(source.timeline().clock_master_cycles(), 0, 0).unwrap(),
            ).unwrap()).unwrap();
            let mut observer = source.timeline().synchronous_event_observer();
            let (mut cpu, mut refresh, mut wall, mut prefix, mut poll_cpu, mut poll_bus) =
                (0u64, 0u64, 0u64, None, 0u64, 0u64);
            for _ in 0..200_000 {
                let pc = source.snes().cpu.pc;
                if pc == 0x805a { break; }
                if pc == 0xf3a3 && prefix.is_none() { prefix = Some(cpu); }
                let polling = (0xf3a3..=0xf3b2).contains(&pc);
                let step = source.step().unwrap();
                assert!(step.accepted_interrupt.is_none());
                let instructions = step.transactions.iter()
                    .map(|t| u64::from(t.duration_master_cycles)).sum::<u64>();
                let duration = step.ended_at.master_cycles() - step.started_at.master_cycles();
                let mut step_refresh = 0;
                source.timeline().observe_processed_synchronous_events(&mut observer, |event| {
                    if event == CpuSynchronousTimelineEvent::Bus(snes::CpuBusEvent::WramRefresh) {
                        step_refresh += 40;
                    }
                });
                cpu += instructions;
                refresh += step_refresh;
                wall += duration;
                if polling { poll_cpu += instructions; poll_bus += duration - instructions - step_refresh; }
            }
            assert_eq!(source.snes().cpu.pc, 0x805a);
            let prefix = prefix.unwrap();
            assert!(poll_cpu >= 172);
            game.lag_free_frame_master_start = crate::cycle_ledger::master();
            game.native_begin_main_cost();
            game.native_pacing.entry_handler = 0;
            crate::cycle_ledger::charge(prefix);
            game.native_wait_until_line(192, 0xff, 24, 178);
            // The final untaken polling pass remains ordinary CPU work.
            crate::cycle_ledger::charge(cpu - prefix - poll_cpu + 172);
            game.native_finish_main_cost();
            let (bus, admitted) = game.native_cost_main_hdma();
            assert!(admitted);
            let outside_poll_bus = wall - cpu - refresh - poll_bus;
            assert!(bus.abs_diff(outside_poll_bus) <= 2 * event_max,
                "radius={radius} bus={bus} source={outside_poll_bus}");
            let model_wall = game.native_cost_shortfall(0).0;
            assert!(model_wall.abs_diff(wall) <= 178 + 2 * event_max + 80,
                "radius={radius} wall={model_wall} source={wall}");
            assert_eq!(game.native_pacing.finished_main, cpu - poll_cpu + 172);
        }
    }

    #[test]
    fn worker_owned_bus_events_are_never_charged_to_main() {
        let mut program = Program::read(&game(), Programming::read(&game())).unwrap();
        program.events = vec![(60_000, 66), (200_000, 66)];
        program.total = 132;
        assert_eq!(program.main_stalls(81_000, 2 * FIELD + 81_000, 1000, 80_000, false), 2 * 66);
        assert_eq!(program.main_stalls(1000, 2 * FIELD + 100_000, 1000, FIELD - 120_000, true), 3 * 66);
    }

    #[test]
    fn lease_declines_program_changes_and_mixed_clock_owners() {
        let mut game = game();
        game.native_begin_main_cost();
        assert!(game.native_bus_lease_valid());
        game.ram[0x1b00] = 0x7f; // Indirect payload, not a descriptor.
        assert!(game.native_bus_lease_valid());
        game.clear_hdma_enable_mask();
        game.set_hdma_enable_mask(0xc0);
        assert!(!game.native_bus_lease_valid()); // A restored mask still changed during main.
        game.native_begin_main_cost();
        game.native_pacing.raster_barriers.push((0, 192, 0xff, 24, 178));
        assert!(game.native_bus_lease_valid());
        game.native_begin_main_cost();
        game.lag_free_vwf_master = 1;
        assert!(game.native_bus_lease_valid());
        game.lag_free_vwf_master = 0;
        game.dma.channel[6].a_bank = 0x7e; // Mutable descriptors are unsupported.
        game.native_begin_main_cost();
        assert!(!game.native_bus_lease_valid());
    }

    #[test]
    fn decline_reasons_overlap_and_freeze_before_the_closing_nmi() {
        let mut game = game();
        game.native_begin_main_cost();
        game.lag_free_vwf_master = 1;
        game.native_pacing.main_dma = 26;
        game.native_pacing.raster_barriers.push((0, 192, 0xff, 24, 178));
        game.clear_hdma_enable_mask();
        game.set_hdma_enable_mask(0xc0);
        assert_eq!(game.native_bus_lease_decline(), 2 | 64);
        game.native_finish_main_cost();
        assert_eq!(game.native_cost_main_hdma_decline(), 2 | 64);
        game.lag_free_vwf_master = 0;
        game.native_pacing.main_dma = 0;
        game.native_pacing.raster_barriers.clear();
        game.clear_hdma_enable_mask();
        assert_ne!(game.native_bus_lease_decline(), game.native_cost_main_hdma_decline());
        assert_eq!(game.native_cost_main_hdma_decline(), 2 | 64);
    }
}
