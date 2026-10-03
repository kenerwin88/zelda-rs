//! Native locals for the shared magic/rupee refill prefix ($0D:DB92..DC18).
//! Each edge follows a source instruction so an interrupt retains loaded
//! values and branch decisions, while later reads see the post-NMI state.
//! Stores go through the resource and sound owners; no source RAM is imported.

use super::ZeldaState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct HudRefillPrefix {
    pc: u16,
    a: u16,
    zero: bool,
    negative: bool,
    carry: bool,
    clamped: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{HudUpdateInterruption, HudUpdateResume};
    use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};

    #[test]
    fn refill_prefix_matches_source_instructions_and_every_resume_edge() {
        let path = std::env::var_os("ZELDA3_ROM").map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../saves/zelda3.sfc"));
        let mut rom = std::fs::read(path).expect("source prefix test requires the ROM");
        if rom.len() % 0x400 == 0x200 { rom.drain(..0x200); }
        let mut visited = std::collections::BTreeSet::new();
        for (filler, magic, frame, sound) in [
            (0, 0, 0, 0), (1, 128, 0, 0), (2, 255, 0, 1),
            (2, 127, 0, 0), (2, 1, 0, 9), (2, 1, 3, 0),
        ] {
            for (actual, goal, delay) in [
                (5u16, 5u16, 7), (5, 6, 0), (5, 9, 1),
                (5, 4, 255), (999, 1001, 8), (0, 65535, 0),
                (65535, 0, 0),
            ] {
                let mut state = ZeldaState::new();
                state.ram[0xf373] = filler;
                state.ram[0xf36e] = magic;
                state.ram[0x1a] = frame;
                state.ram[0x12e] = sound;
                state.ram[0xcfd] = delay;
                state.ram[0xf362..0xf364].copy_from_slice(&actual.to_le_bytes());
                state.ram[0xf360..0xf362].copy_from_slice(&goal.to_le_bytes());
                state.sync_native_game_state_from_ram();
                let initial = state.clone();
                let checkpoint = RomCpuCheckpoint {
                    entry_pc: 0x0d_db92, stop_pc: 0x0d_dc18,
                    a: 0, x: 0, y: 0, sp: 0x01fd, dp: 0, db: 0x0d,
                    carry: false, zero: false, overflow: false, negative: false,
                    interrupt_disable: false, decimal: false, accumulator_is_8_bit: true,
                    index_is_8_bit: true, emulation: false, waiting: false,
                    stack_address: 0x01fe, stack_bytes: &[],
                };
                let mut run = RomCpuTimingRun::new(&rom, &state.ram, &state.sram,
                    &state.ppu, &state.dma, [0; 4], checkpoint).unwrap();
                run.enable_cpu_write_trace();
                let mut expected_ram = state.ram.clone();
                let mut prefix = HudRefillPrefix::new();
                let mut edges = vec![prefix.pc];
                let mut total = 0;
                while !run.is_complete() {
                    let pc = run.pc();
                    visited.insert(pc);
                    assert!(HudRefillPrefix::contains_pc(pc));
                    assert_eq!(pc, 0x0d_0000 | u32::from(prefix.pc));
                    let cycles = u64::from(run.step().master_cycles);
                    total += cycles;
                    for (address, value) in run.take_cpu_wram_writes() {
                        expected_ram[address] = value;
                    }
                    let start = crate::cycle_ledger::master();
                    prefix.step(&mut state);
                    assert_eq!(crate::cycle_ledger::master() - start, cycles, "PC {pc:06x}");
                    assert_eq!(prefix.a, run.accumulator(), "retained value after PC {pc:06x}");
                    assert_eq!(run.pc(), 0x0d_0000 | u32::from(prefix.pc));
                    assert_eq!(state.ram, expected_ram, "stores after PC {pc:06x}");
                    edges.push(prefix.pc);
                    assert!(edges.len() <= 60);
                }
                for edge in edges {
                    let mut split = initial.clone();
                    let mut cursor = HudRefillPrefix::new();
                    let start = crate::cycle_ledger::master();
                    cursor.advance_to(&mut split, edge);
                    cursor.finish(&mut split);
                    assert_eq!(crate::cycle_ledger::master() - start, total);
                    assert_eq!(split.ram, expected_ram, "resume at {edge:04x}");
                    assert_eq!(split.game_state, state.game_state);
                }
            }
        }
        // Every classified instruction (and the terminal boundary) is covered.
        for pc in 0x0d_db92..0x0d_dc18 {
            if HudRefillPrefix::contains_pc(pc) {
                assert!(visited.contains(&pc), "missing source branch at {pc:06x}");
            }
        }
    }

    #[test]
    fn refill_prefix_resume_preserves_loaded_values_and_observes_later_reads() {
        let mut state = ZeldaState::new();
        state.player_resources_mut().set_rupees_actual(5);
        state.player_resources_mut().set_rupees_goal(5);
        let mut cursor = HudRefillPrefix::new();
        cursor.advance_to(&mut state, 0xdbd6);
        // A survives NMI, but CMP must read the current goal on resumption.
        state.player_resources_mut().set_rupees_actual(90);
        state.player_resources_mut().set_rupees_goal(8);
        cursor.finish(&mut state);
        assert_eq!(state.game_state.inventory.player_resources.rupees_actual(), 6);

        let mut state = ZeldaState::new();
        state.player_resources_mut().set_magic_filler(2);
        state.set_sound_effect_1(9);
        let mut cursor = HudRefillPrefix::new();
        cursor.advance_to(&mut state, 0xdbc9);
        // NMI consumes the sound port, but the pending BNE uses the earlier LDA.
        state.set_sound_effect_1(0);
        cursor.finish(&mut state);
        assert_eq!(state.game_state.system_signals.sound_effect_1(), 0);
        assert_eq!(state.game_state.inventory.player_resources.magic.amount(), 1);
        assert_eq!(state.game_state.inventory.player_resources.magic_filler(), 1);
    }

    #[test]
    fn refill_prefix_preserves_expanded_wallet_cap() {
        for (actual, goal, expected_actual, expected_goal) in [
            (999, 1500, 1000, 1500), (9998, 10001, 9999, 10001),
            (9999, 10001, 9999, 9999),
        ] {
            let mut state = ZeldaState::new();
            state.enhanced_features_mut().set_bits(super::super::FEATURE_CARRY_MORE_RUPEES);
            state.player_resources_mut().set_rupees_actual(actual);
            state.player_resources_mut().set_rupees_goal(goal);
            let mut cursor = HudRefillPrefix::new();
            cursor.advance_to(&mut state, 0xdbf3);
            cursor.finish(&mut state);
            assert_eq!(state.game_state.inventory.player_resources.rupees_actual(), expected_actual);
            assert_eq!(state.game_state.inventory.player_resources.rupees_goal(), expected_goal);
        }
    }

    #[test]
    fn refill_prefix_hud_caller_resumes_without_repeating_resources() {
        for pc in [0xdb92, 0xdbb3, 0xdbbc, 0xdbd6, 0xdbf7, 0xdc05, 0xdc18] {
            let mut atomic = ZeldaState::new();
            atomic.player_resources_mut().set_magic_filler(2);
            atomic.player_resources_mut().set_rupees_goal(1);
            atomic.game_state.frame.frame_counter = 3;
            atomic.ram[0x1a] = 3;
            let mut split = atomic.clone();
            let start = crate::cycle_ledger::master();
            atomic.hud_refill_logic_until_interruption();
            let total = crate::cycle_ledger::master() - start;
            split.native_overworld_hud_interruption =
                Some(HudUpdateInterruption::RefillPrefix { pc });
            let start = crate::cycle_ledger::master();
            let resume = split.hud_refill_logic_until_interruption().unwrap();
            assert!(matches!(resume, HudUpdateResume::RefillPrefix(_)));
            assert_eq!(split.ram[0x16], 0);
            split.resume_overworld_hud_update(resume);
            assert_eq!(crate::cycle_ledger::master() - start, total);
            assert_eq!(split.ram, atomic.ram);
            assert_eq!(split.game_state, atomic.game_state);
        }
    }
}

impl HudRefillPrefix {
    pub(super) fn new() -> Self {
        Self { pc: 0xdb92, a: 0, zero: false, negative: false, carry: false, clamped: false }
    }

    pub(crate) fn contains_pc(pc: u32) -> bool {
        pc >> 16 == 0x0d && matches!(pc as u16,
            0xdb92 | 0xdb96 | 0xdb98 | 0xdb9c | 0xdb9e | 0xdba0 | 0xdba2 |
            0xdba6 | 0xdba8 | 0xdbac | 0xdbae | 0xdbb2 | 0xdbb3 | 0xdbb7 |
            0xdbbb | 0xdbbc | 0xdbc0 | 0xdbc2 | 0xdbc4 | 0xdbc6 | 0xdbc9 |
            0xdbcb | 0xdbcd | 0xdbd0 | 0xdbd2 | 0xdbd6 | 0xdbda | 0xdbdc |
            0xdbde | 0xdbdf | 0xdbe1 | 0xdbe4 | 0xdbe8 | 0xdbea | 0xdbeb |
            0xdbee | 0xdbf0 | 0xdbf3 | 0xdbf7 | 0xdbfb | 0xdbfd | 0xdc00 |
            0xdc02 | 0xdc05 | 0xdc08 | 0xdc0a | 0xdc0c | 0xdc0e | 0xdc11 |
            0xdc13 | 0xdc15 | 0xdc18)
    }

    pub(super) fn advance_to(&mut self, state: &mut ZeldaState, stop: u16) {
        assert!(Self::contains_pc(0x0d_0000 | u32::from(stop)));
        while self.pc != stop {
            assert_ne!(self.pc, 0xdc18, "refill prefix did not reach {stop:04x}");
            self.step(state);
        }
    }

    pub(super) fn finish(&mut self, state: &mut ZeldaState) {
        self.advance_to(state, 0xdc18);
    }

    fn flags(&mut self, value: u16, sign: u16) {
        self.zero = value == 0;
        self.negative = value & sign != 0;
    }

    fn load8(&mut self, value: u8) {
        self.a = (self.a & 0xff00) | u16::from(value);
        self.flags(u16::from(value), 0x80);
    }

    fn load16(&mut self, value: u16) {
        self.a = value;
        self.flags(value, 0x8000);
    }

    fn branch(taken: bool, target: u16, next: u16) -> (u16, u64) {
        if taken { (target, 22) } else { (next, 16) }
    }

    fn step(&mut self, state: &mut ZeldaState) {
        let (next, cycles) = match self.pc {
            0xdb92 => { self.load8(state.game_state.inventory.player_resources.magic_filler()); (0xdb96, 40) }
            0xdb96 => Self::branch(self.zero, 0xdbd0, 0xdb98),
            0xdb98 => { self.load8(state.game_state.inventory.player_resources.magic.amount()); (0xdb9c, 40) }
            0xdb9c => {
                self.carry = self.a as u8 >= 128;
                self.flags(u16::from((self.a as u8).wrapping_sub(128)), 0x80);
                (0xdb9e, 16)
            }
            0xdb9e => Self::branch(!self.carry, 0xdbae, 0xdba0),
            0xdba0 => { self.load8(128); (0xdba2, 16) }
            0xdba2 => { state.player_magic_mut().set_magic_power(self.a as u8); (0xdba6, 40) }
            0xdba6 => { self.load8(0); (0xdba8, 16) }
            0xdba8 => { state.player_resources_mut().set_magic_filler(self.a as u8); (0xdbac, 40) }
            0xdbac => (0xdbd0, 22),
            0xdbae => { self.load8(state.game_state.inventory.player_resources.magic_filler()); (0xdbb2, 40) }
            0xdbb2 => { self.load8((self.a as u8).wrapping_sub(1)); (0xdbb3, 14) }
            0xdbb3 => { state.player_resources_mut().set_magic_filler(self.a as u8); (0xdbb7, 40) }
            0xdbb7 => { self.load8(state.game_state.inventory.player_resources.magic.amount()); (0xdbbb, 40) }
            0xdbbb => { self.load8((self.a as u8).wrapping_add(1)); (0xdbbc, 14) }
            0xdbbc => { state.player_magic_mut().set_magic_power(self.a as u8); (0xdbc0, 40) }
            0xdbc0 => { self.load8(state.game_state.frame.frame_counter); (0xdbc2, 24) }
            0xdbc2 => { self.load8(self.a as u8 & 3); (0xdbc4, 16) }
            0xdbc4 => Self::branch(!self.zero, 0xdbd0, 0xdbc6),
            0xdbc6 => { self.load8(state.game_state.system_signals.sound_effect_1()); (0xdbc9, 32) }
            0xdbc9 => Self::branch(!self.zero, 0xdbd0, 0xdbcb),
            0xdbcb => { self.load8(45); (0xdbcd, 16) }
            0xdbcd => { state.set_sound_effect_1(self.a as u8); (0xdbd0, 32) }
            0xdbd0 => (0xdbd2, 22), // REP #$30
            0xdbd2 => { self.load16(state.game_state.inventory.player_resources.rupees_actual()); (0xdbd6, 48) }
            0xdbd6 => {
                let goal = state.game_state.inventory.player_resources.rupees_goal();
                self.carry = self.a >= goal;
                self.flags(self.a.wrapping_sub(goal), 0x8000);
                (0xdbda, 48)
            }
            0xdbda => Self::branch(self.zero, 0xdc13, 0xdbdc),
            0xdbdc => Self::branch(self.negative, 0xdbea, 0xdbde),
            0xdbde => { self.load16(self.a.wrapping_sub(1)); (0xdbdf, 14) }
            0xdbdf => Self::branch(!self.negative, 0xdbf7, 0xdbe1),
            0xdbe1 => { self.load16(0); (0xdbe4, 24) }
            0xdbe4 => { state.player_resources_mut().set_rupees_goal(self.a); (0xdbe8, 48) }
            0xdbe8 => (0xdbf7, 22),
            0xdbea => { self.load16(self.a.wrapping_add(1)); (0xdbeb, 14) }
            0xdbeb => {
                self.carry = self.a >= 1000;
                self.flags(self.a.wrapping_sub(1000), 0x8000);
                (0xdbee, 24)
            }
            0xdbee => Self::branch(!self.carry, 0xdbf7, 0xdbf0),
            0xdbf0 => {
                // Preserve the optional expanded wallet's native semantics;
                // the canonical cap is the ROM's immediate 999.
                self.clamped = self.a > state.max_rupees();
                if self.clamped { self.load16(state.max_rupees()); }
                (0xdbf3, 24)
            }
            0xdbf3 => {
                if self.clamped { state.player_resources_mut().set_rupees_goal(self.a); }
                (0xdbf7, 48)
            }
            0xdbf7 => { state.player_resources_mut().set_rupees_actual(self.a); (0xdbfb, 48) }
            0xdbfb => (0xdbfd, 22), // SEP #$30 preserves the high accumulator byte.
            0xdbfd => { self.load8(state.game_state.system_signals.sound_effect_1()); (0xdc00, 32) }
            0xdc00 => Self::branch(!self.zero, 0xdc13, 0xdc02),
            0xdc02 => { self.load8(state.hud_state().rupee_sfx_sound_delay()); (0xdc05, 32) }
            0xdc05 => {
                let delay = state.hud_state().rupee_sfx_sound_delay().wrapping_add(1);
                state.set_rupee_sfx_sound_delay(delay);
                self.flags(u16::from(delay), 0x80);
                (0xdc08, 46)
            }
            0xdc08 => { self.load8(self.a as u8 & 7); (0xdc0a, 16) }
            0xdc0a => Self::branch(!self.zero, 0xdc18, 0xdc0c),
            0xdc0c => { self.load8(41); (0xdc0e, 16) }
            0xdc0e => { state.set_sound_effect_1(self.a as u8); (0xdc11, 32) }
            0xdc11 => (0xdc18, 22),
            0xdc13 => (0xdc15, 22), // SEP #$30
            0xdc15 => { state.set_rupee_sfx_sound_delay(0); (0xdc18, 32) }
            _ => unreachable!("invalid native refill prefix edge {:04x}", self.pc),
        };
        crate::cycle_ledger::charge(cycles);
        self.pc = next;
    }
}
