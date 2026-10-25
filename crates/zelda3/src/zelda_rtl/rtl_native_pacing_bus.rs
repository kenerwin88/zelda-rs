//! Main-only HDMA stalls for a leased immutable descriptor program.
//! No payload values, scene identities or source lag counts price the bus.
use super::*;

const FIELD: u64 = 1364 * 262;
const NMI: u64 = 225 * 1364 + 86;
type ChannelProgram = (u8, u8, u16, u8, bool, u8, bool);

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
        // A generic capacity proof, not an iteration/time cap: even the
        // whole field's bus work plus its induced refresh must leave main
        // positive capacity under the retained interrupt ownership.
        if (self.total * 1364).div_ceil(1324) >= FIELD - (handler + worker).min(FIELD - 1) {
            return None;
        }
        let mut hdma = 0;
        loop {
            let work = main + hdma;
            let end = advance_main_cost(work + work.div_ceil(1324) * 40,
                start, handler, worker, after_irq);
            let next = self.main_stalls(start, end, handler, worker, after_irq);
            if next == hdma { return Some((end, hdma)); }
            debug_assert!(next > hdma, "positive-capacity HDMA closure is monotone");
            hdma = next;
        }
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
        self.native_pacing.raster_wait_seen = false;
    }

    pub(super) fn native_bus_lease_valid(&self) -> bool {
        // Mixed timed owners and program changes decline the new term. The
        // original pacing model remains authoritative for those calls.
        self.lag_free_vwf_master == 0 && self.native_pacing.main_dma == 0
            && !self.native_pacing.song_upload && !self.native_pacing.raster_wait_seen
            && self.native_pacing.worker_segments.is_empty()
            && self.game_execution_scheduler.current_work().is_none()
            && self.native_pacing.bus_epoch == crate::cycle_ledger::hdma_program_epoch()
            && self.native_pacing.bus_program.as_ref().is_some_and(|program|
                program.programming == Programming::read(self))
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
          }
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
        game.native_pacing.raster_wait_seen = true;
        assert!(!game.native_bus_lease_valid());
        game.native_begin_main_cost();
        game.lag_free_vwf_master = 1;
        assert!(!game.native_bus_lease_valid());
        game.lag_free_vwf_master = 0;
        game.dma.channel[6].a_bank = 0x7e; // Mutable descriptors are unsupported.
        game.native_begin_main_cost();
        assert!(!game.native_bus_lease_valid());
    }
}
