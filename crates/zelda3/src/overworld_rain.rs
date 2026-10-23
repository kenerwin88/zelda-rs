//! Native continuation for the rain tail shared by overworld callers.
//! A stopped caller retains its loaded bytes and branch decisions. Sound,
//! color math, the overlay counter and scroll stores stay with their owners.

use super::{ZeldaState, OVERWORLD_RAIN_X_OFFSETS, OVERWORLD_RAIN_Y_OFFSETS};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OverworldRainContinuation {
    pc: u16,
    a: u8,
    x: u8,
    zero: bool,
    carry: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};

    #[test]
    fn rain_continuation_matches_source_instructions_and_every_resume_edge() {
        let path = std::env::var_os("ZELDA3_ROM").map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../saves/zelda3.sfc"));
        let mut rom = std::fs::read(path).expect("source rain test requires the ROM");
        if rom.len() % 0x400 == 0x200 { rom.drain(..0x200); }
        let mut visited = std::collections::BTreeSet::new();
        for (screen, progress, event) in [(0, 0, 0), (0, 2, 0), (0x70, 2, 0), (0x70, 0, 0x20)] {
            for frame in [0, 3, 5, 36, 44, 88, 90, 37] {
                for counter in [0, 1, 2, 3, 255] {
                    let mut state = ZeldaState::new();
                    state.ram[0x8a] = screen;
                    state.write_wram_byte(0xf3c5, progress);
                    state.ram[0xf2f0] = event;
                    state.ram[0x1a] = frame;
                    state.ram[0x494] = counter;
                    state.ram[0xe0..0xe2].copy_from_slice(&0xffaau16.to_le_bytes());
                    state.ram[0xe6..0xe8].copy_from_slice(&0xfebbu16.to_le_bytes());
                    state.sync_native_game_state_from_ram();
                    let initial = state.clone();
                    let checkpoint = RomCpuCheckpoint {
                        entry_pc: 0x02_a4cd, stop_pc: 0x00_805a,
                        a: 0, x: 0, y: 0, sp: 0x01fc, dp: 0, db: 0x02,
                        carry: false, zero: false, overflow: false, negative: false,
                        interrupt_disable: false, decimal: false, accumulator_is_8_bit: true,
                        index_is_8_bit: true, emulation: false, waiting: false,
                        stack_address: 0x01fd, stack_bytes: &[0x59, 0x80, 0x00],
                    };
                    let mut run = RomCpuTimingRun::new(&rom, &state.wram_image(), &state.sram,
                        &state.ppu, &state.dma, [0; 4], checkpoint).unwrap();
                    run.enable_cpu_write_trace();
                    let mut expected = state.wram_image().into_owned();
                    let mut rain = OverworldRainContinuation::new();
                    let mut edges = vec![rain.pc];
                    let mut total = 0;
                    while !run.is_complete() {
                        let pc = run.pc();
                        visited.insert(pc);
                        assert!(OverworldRainContinuation::contains_pc(pc));
                        assert_eq!(pc, 0x02_0000 | u32::from(rain.pc));
                        let cycles = u64::from(run.step().master_cycles);
                        total += cycles;
                        for (address, value) in run.take_cpu_wram_writes() {
                            expected[address] = value;
                        }
                        let start = crate::cycle_ledger::master();
                        rain.step(&mut state);
                        assert_eq!(crate::cycle_ledger::master() - start, cycles, "PC {pc:06x}");
                        assert_eq!(rain.a, run.accumulator() as u8, "PC {pc:06x}");
                        assert_eq!(u16::from(rain.x), run.index_x(), "PC {pc:06x}");
                        assert_eq!(run.pc(), if rain.pc == 0 { 0x00_805a }
                            else { 0x02_0000 | u32::from(rain.pc) });
                        assert_eq!(*state.wram_image(), expected, "stores after PC {pc:06x}");
                        edges.push(rain.pc);
                        assert!(edges.len() <= 50);
                    }
                    for edge in edges {
                        let mut split = initial.clone();
                        let mut cursor = OverworldRainContinuation::new();
                        let start = crate::cycle_ledger::master();
                        cursor.advance_to(&mut split, edge);
                        cursor.finish(&mut split);
                        assert_eq!(crate::cycle_ledger::master() - start, total);
                        assert_eq!(*split.wram_image(), expected, "resume at {edge:04x}");
                        assert_eq!(split.game_state, state.game_state);
                    }
                }
            }
        }
        for pc in 0x02_a4cd..=0x02_a52c {
            if OverworldRainContinuation::contains_pc(pc) {
                assert!(visited.contains(&pc), "missing source branch at {pc:06x}");
            }
        }
    }

    #[test]
    fn rain_continuation_keeps_loaded_progress_and_completed_side_effects() {
        let mut state = ZeldaState::new();
        state.write_wram_byte(0xf3c5, 2);
        state.sync_native_game_state_from_ram();
        let mut rain = OverworldRainContinuation::new();
        rain.advance_to(&mut state, 0xa4d7);
        // The story byte has already been loaded. Changing its owner while
        // suspended cannot change that pending comparison into another path.
        state.save_progress_mut().set_progress_indicator(0);
        rain.finish(&mut state);
        assert_eq!(state.game_state.world.transient.move_overlay_counter, 0);

        let mut state = ZeldaState::new();
        state.game_state.frame.set_frame_counter(36);
        state.set_bg1_x(0xffaa);
        state.world_transient_mut().set_move_overlay_ctr(3);
        let mut rain = OverworldRainContinuation::new();
        rain.advance_to(&mut state, 0xa506);
        assert_eq!(state.game_state.system_signals.sound_effect_1(), 54);
        state.set_sound_effect_1(0); // NMI consumed the pending sound.
        rain.advance_to(&mut state, 0xa521);
        state.set_bg1_x(0x99bb);
        rain.advance_to(&mut state, 0xa523);
        // STA $E1 publishes the retained high byte without replacing the
        // current low byte. Resuming beyond this store cannot replay it.
        assert_eq!(state.game_state.display.ppu_scroll_copy.bg1_h_copy2(), 0x00bb);
        state.set_bg1_x(0x1234);
        rain.finish(&mut state);
        assert_eq!(state.game_state.display.ppu_scroll_copy.bg1_h_copy2(), 0x1234);
        assert_eq!(state.game_state.system_signals.sound_effect_1(), 0);
        assert_eq!(state.game_state.world.transient.move_overlay_counter, 0);
    }
}

impl OverworldRainContinuation {
    const RETURNED: u16 = 0;

    pub(super) fn new() -> Self {
        Self { pc: 0xa4cd, a: 0, x: 0, zero: false, carry: false }
    }

    pub(crate) fn contains_pc(pc: u32) -> bool {
        pc >> 16 == 2 && matches!(pc as u16,
            0xa4cd | 0xa4cf | 0xa4d1 | 0xa4d3 | 0xa4d7 | 0xa4d9 |
            0xa4db | 0xa4df | 0xa4e1 | 0xa4e3 | 0xa4e5 | 0xa4e7 |
            0xa4e9 | 0xa4eb | 0xa4ed | 0xa4ef | 0xa4f1 | 0xa4f3 |
            0xa4f5 | 0xa4f7 | 0xa4f9 | 0xa4fb | 0xa4fd | 0xa4ff |
            0xa501 | 0xa503 | 0xa506 | 0xa508 | 0xa50a | 0xa50c |
            0xa50e | 0xa510 | 0xa513 | 0xa514 | 0xa516 | 0xa519 |
            0xa51a | 0xa51c | 0xa51d | 0xa521 | 0xa523 | 0xa525 |
            0xa526 | 0xa52a | 0xa52c)
    }

    pub(super) fn advance_to(&mut self, state: &mut ZeldaState, pc: u16) {
        assert!(pc == Self::RETURNED || Self::contains_pc(0x02_0000 | u32::from(pc)));
        while self.pc != pc {
            assert_ne!(self.pc, Self::RETURNED, "rain caller did not reach {pc:04x}");
            self.step(state);
        }
    }

    pub(crate) fn finish(&mut self, state: &mut ZeldaState) {
        let _scope = crate::cycle_ledger::routine(0x02_a4cd);
        self.advance_to(state, Self::RETURNED);
    }

    fn load(&mut self, value: u8) {
        self.a = value;
        self.zero = value == 0;
    }

    fn compare(&mut self, value: u8) {
        self.zero = self.a == value;
        self.carry = self.a >= value;
    }

    fn add(&mut self, value: u8) {
        let result = u16::from(self.a) + u16::from(value) + u16::from(self.carry);
        self.carry = result > 255;
        self.load(result as u8);
    }

    fn branch(taken: bool, target: u16, next: u16) -> (u16, u64) {
        if taken { (target, 22) } else { (next, 16) }
    }

    fn step(&mut self, state: &mut ZeldaState) {
        let (next, cycles) = match self.pc {
            0xa4cd => { self.load(state.game_state.world.location.overworld_screen_index()); (0xa4cf, 24) }
            0xa4cf => { self.compare(0x70); (0xa4d1, 16) }
            0xa4d1 => Self::branch(self.zero, 0xa4db, 0xa4d3),
            0xa4d3 => { self.load(state.game_state.inventory.save_progress.progress_indicator()); (0xa4d7, 40) }
            0xa4d7 => { self.compare(2); (0xa4d9, 16) }
            0xa4d9 => Self::branch(self.carry, 0xa52c, 0xa4db),
            0xa4db => { self.load(state.game_state.world.overworld.event_info.event_info(0x70)); (0xa4df, 40) }
            0xa4df => { self.load(self.a & 0x20); (0xa4e1, 16) }
            0xa4e1 => Self::branch(!self.zero, 0xa52c, 0xa4e3),
            0xa4e3 => { self.load(state.game_state.frame.frame_counter); (0xa4e5, 24) }
            0xa4e5 => { self.compare(3); (0xa4e7, 16) }
            0xa4e7 => Self::branch(self.zero, 0xa506, 0xa4e9),
            0xa4e9 => { self.compare(5); (0xa4eb, 16) }
            0xa4eb => Self::branch(self.zero, 0xa4fd, 0xa4ed),
            0xa4ed => { self.compare(36); (0xa4ef, 16) }
            0xa4ef => Self::branch(self.zero, 0xa501, 0xa4f1),
            0xa4f1 => { self.compare(44); (0xa4f3, 16) }
            0xa4f3 => Self::branch(self.zero, 0xa4fd, 0xa4f5),
            0xa4f5 => { self.compare(88); (0xa4f7, 16) }
            0xa4f7 => Self::branch(self.zero, 0xa506, 0xa4f9),
            0xa4f9 => { self.compare(90); (0xa4fb, 16) }
            0xa4fb => Self::branch(!self.zero, 0xa50a, 0xa4fd),
            0xa4fd => { self.load(0x72); (0xa4ff, 16) }
            0xa4ff => (0xa508, 22),
            0xa501 => { self.x = 0x36; self.zero = false; (0xa503, 16) }
            0xa503 => { state.set_sound_effect_1(self.x); (0xa506, 32) }
            0xa506 => { self.load(0x32); (0xa508, 16) }
            0xa508 => { state.set_color_math_control(self.a); (0xa50a, 24) }
            0xa50a => { self.load(state.game_state.frame.frame_counter); (0xa50c, 24) }
            0xa50c => { self.load(self.a & 3); (0xa50e, 16) }
            0xa50e => Self::branch(!self.zero, 0xa52c, 0xa510),
            0xa510 => { self.load(state.game_state.world.transient.move_overlay_counter); (0xa513, 32) }
            0xa513 => { self.load(self.a.wrapping_add(1)); (0xa514, 14) }
            0xa514 => { self.load(self.a & 3); (0xa516, 16) }
            0xa516 => { state.world_transient_mut().set_move_overlay_ctr(self.a); (0xa519, 32) }
            0xa519 => { self.x = self.a; self.zero = self.x == 0; (0xa51a, 14) }
            0xa51a => { self.load((state.game_state.display.ppu_scroll_copy.bg1_h_copy2() >> 8) as u8); (0xa51c, 24) }
            0xa51c => { self.carry = false; (0xa51d, 14) }
            0xa51d => { self.add(OVERWORLD_RAIN_X_OFFSETS[usize::from(self.x)]); (0xa521, 40) }
            0xa521 => {
                let low = state.game_state.display.ppu_scroll_copy.bg1_h_copy2() & 0xff;
                state.set_bg1_x(low | (u16::from(self.a) << 8));
                (0xa523, 24)
            }
            0xa523 => { self.load((state.game_state.display.ppu_scroll_copy.bg1_v_copy2() >> 8) as u8); (0xa525, 24) }
            0xa525 => { self.carry = false; (0xa526, 14) }
            0xa526 => { self.add(OVERWORLD_RAIN_Y_OFFSETS[usize::from(self.x)]); (0xa52a, 40) }
            0xa52a => {
                let low = state.game_state.display.ppu_scroll_copy.bg1_v_copy2() & 0xff;
                state.set_bg1_y(low | (u16::from(self.a) << 8));
                (0xa52c, 24)
            }
            0xa52c => (Self::RETURNED, 44),
            _ => unreachable!("invalid native rain continuation edge {:04x}", self.pc),
        };
        crate::cycle_ledger::charge(cycles);
        self.pc = next;
    }
}
