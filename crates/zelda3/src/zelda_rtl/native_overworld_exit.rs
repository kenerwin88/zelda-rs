//! Live exit-walk chain and publications. The surrounding development caller
//! still owns dispatch and the sprites/HUD/rain return.
use super::native_item_graphics::scroll_hdma_descriptor_byte;
use super::*;
use crate::game_state::constants::{OVERWORLD_SCREEN_TRANSITION, OVERWORLD_SCREEN_TRANS_DIR_BITS2};
use crate::cycle_models::{
    intro_clear::Access,
    native_work::WorkBus,
    nmi_stack::NativeNmiContext,
    overworld_exit_walk::ExitWalkDirection,
    overworld_exit_chain::ExitChain,
};
use snes::{cpu_timeline::CpuSynchronousTimelineEvent, CpuMasterTimeline, HdmaTimingState};

/// Main stack bytes $01FB..$01FF: the exit caller's return frames and Main's
/// $8059 return, directly below the main loop's initial SP of $01FF.
const EXIT_CALLER_RETURN_STACK: usize = 0x1fb;

#[derive(Clone, Debug)]
pub(super) struct NativeOverworldExitEntry {
    pub(super) host: u32,
    pub(super) input: u16,
    pub(super) retain_return_phase: bool,
    pub(super) context: NativeNmiContext,
    pub(super) timeline: CpuMasterTimeline,
    pub(super) hdma: HdmaTimingState,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn production_exit_bus_matches_source_without_runtime_rom() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let rom = std::fs::read(root.join("saves/zelda3.sfc")).unwrap();
        let assets = std::fs::read(std::env::var("ZELDA3_ASSETS").unwrap()).unwrap();
        let mut instructions = 0;
        for direction in [ExitWalkDirection::Down, ExitWalkDirection::Up] {
            for case in 0..64usize {
                let mut actual = ZeldaState::new();
                actual.assets = Some(AssetPack::parse(&assets).unwrap());
                actual.rom.clear();
                actual.write_wram_byte(0x10, 9);
                actual.write_wram_byte(0x11, if direction == ExitWalkDirection::Down {
                    10
                } else {
                    11
                });
                actual.ram[0x20..0x22]
                    .copy_from_slice(&[0u16, 1, 255, 65535][case % 4].to_le_bytes());
                actual.ram[0x69a] = [0, 1, 2, 255][case / 4 % 4];
                actual.ram[0x416] = [0, 8, 5][case % 3];
                actual.ram[0x67] = [0, 4, 8, 12][case / 16 % 4];
                actual.ram[0x2f] = (case / 16 % 4 * 2) as u8;
                actual.ram[0x2e] = (case % 8) as u8;
                actual.ram[0x2b] = (case % 4) as u8;
                actual.ram[0x1fb..0x200].copy_from_slice(&[0x7d, 0xa4, 0x59, 0x80, 0]);
                actual.sync_native_game_state_from_ram();
                let mut cpu = NativeNmiContext {
                    a: 0x1200,
                    x: 3,
                    y: 7,
                    d: 0,
                    db: [0, 2, 0x80, 0x82][case / 16],
                    bank: 2,
                    pc: if direction == ExitWalkDirection::Down {
                        0xac8f
                    } else {
                        0xacc2
                    },
                    sp: 0x1fa,
                    p: 0x30 | [0, 1, 0x41, 0xc3][case % 4],
                };
                let mut clock = CpuMasterTimeline::at_raster(
                    4937,
                    snes::CpuRasterPosition::new(224, 1300),
                    snes::CpuBusWorkload::default(),
                    snes::CpuFieldTiming::NON_INTERLACE_EVEN,
                );
                let mut machine = snes::Snes::new();
                machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                machine.ram.copy_from_slice(&actual.wram_image());
                machine.cpu.a = cpu.a;
                machine.cpu.x = cpu.x;
                machine.cpu.y = cpu.y;
                machine.cpu.db = cpu.db;
                machine.cpu.k = cpu.bank;
                machine.cpu.pc = cpu.pc;
                machine.cpu.sp = cpu.sp;
                machine.cpu.e = false;
                machine.cpu.unpack_flags(cpu.p);
                let mut source = snes::RomCpuTimingProbe::new(
                    machine,
                    clock.clone(),
                    snes::SourcePpuReadState::snes9x_reset(),
                )
                .unwrap();
                clock.begin_synchronous_timeline().unwrap();
                let mut hdma = HdmaTimingState::new(&actual.dma);
                let mut work = ExitChain::new(direction);
                for _ in 0..20000 {
                    if work.complete() {
                        break;
                    }
                    let pc = cpu.pc;
                    source.step().unwrap();
                    work.step(
                        &mut cpu,
                        &mut clock,
                        ExitBus {
                            state: &mut actual,
                            hdma: &mut hdma,
                        },
                    );
                    let c = &source.snes().cpu;
                    assert_eq!(
                        cpu,
                        NativeNmiContext {
                            a: c.a,
                            x: c.x,
                            y: c.y,
                            d: c.dp,
                            db: c.db,
                            bank: c.k,
                            pc: c.pc,
                            sp: c.sp,
                            p: c.pack_flags()
                        },
                        "case={case} pc={pc:04x}"
                    );
                    assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                    assert!(
                        *actual.wram_image() == source.snes().ram,
                        "case={case} pc={pc:04x} RAM"
                    );
                    instructions += 1;
                    work = work.clone();
                }
                assert!(work.complete());
                let mut projected = actual.clone();
                projected.sync_native_game_state_from_ram();
                assert_eq!(
                    actual.game_state.player.follower_link,
                    projected.game_state.player.follower_link,
                    "case={case} player publication"
                );
                // Project active exit owners. Ending/attract keeps inactive
                // aliases of Link's coordinates and animation in its mirror;
                // this caller does not author that other mode's state.
                let mut a = actual.ram.clone();
                let mut b = a.clone();
                let publish = |state: &GameState, ram: &mut Vec<u8>| {
                    state.frame.encode_into_image(ram);
                    state.player.write_to_ram(ram);
                    state.world.write_to_ram(ram);
                    state.display.write_to_ram(ram);
                };
                publish(&actual.game_state, &mut a);
                publish(&projected.game_state, &mut b);
                let differences = a
                    .iter()
                    .zip(&b)
                    .enumerate()
                    .filter_map(|(i, (a, b))| (i >= 0x10 && a != b).then_some((i, *a, *b)))
                    .take(16)
                    .collect::<Vec<_>>();
                assert_eq!(differences, vec![], "case={case} typed projection");
                assert!(actual.rom.is_empty());
            }
        }
        eprintln!("128 production exit publishers, {instructions} Source instruction boundaries; empty runtime ROM");
    }
}

struct ExitBus<'a> {
    state: &'a mut ZeldaState,
    hdma: &'a mut HdmaTimingState,
}
impl ExitBus<'_> {
    fn read(&self, address: u32) -> u8 {
        if address & 0x400000 == 0 && address & 65535 < 0x2000 {
            return self.state.read_wram_byte((address & 65535) as usize);
        }
        if (0x7e0000..0x800000).contains(&address) {
            return self.state.read_wram_byte((address - 0x7e0000) as usize);
        }
        crate::cycle_models::player_animation::table_byte(address)
            .or_else(|| crate::cycle_models::overworld_camera::table_byte(address))
            .or_else(|| crate::cycle_models::overworld_map_scroll::dispatch_byte(address))
            .or_else(|| {
                crate::cycle_models::overworld_stripes::table_byte(
                    address,
                    self.state.asset_raw(70).unwrap_or(&[]),
                )
            })
            .or_else(|| {
                address.checked_sub(0x02f88d).and_then(|offset| {
                    self.state
                        .asset_raw(107)
                        .and_then(|data| data.get(offset as usize).copied())
                })
            })
            .or_else(|| scroll_hdma_descriptor_byte(address))
            .unwrap_or_else(|| panic!("native overworld exit read outside assets: {address:06x}"))
    }
}
impl WorkBus for ExitBus<'_> {
    fn access(&mut self, _at: u64, access: Access) -> u16 {
        match access {
            Access::Read(a, width) => {
                if width == 1 {
                    u16::from(self.read(a))
                } else {
                    u16::from_le_bytes([self.read(a), self.read(a + 1)])
                }
            }
            Access::Write(a, value, width) => {
                let offset = if a & 0x400000 == 0 && a & 65535 < 0x2000 {
                    (a & 65535) as usize
                } else if (0x7e0000..0x800000).contains(&a) {
                    (a - 0x7e0000) as usize
                } else {
                    panic!("exit store {a:06x}")
                };
                for (byte, value) in value.to_le_bytes().into_iter().take(usize::from(width)).enumerate() {
                    self.state.write_wram_byte(offset + byte, value);
                }
                self.state.native_overworld_exit_store(offset, value, width);
                0
            }
            Access::Vblank { .. } => 0,
        }
    }
    fn retire_event(
        &mut self,
        event: CpuSynchronousTimelineEvent,
        clock: &CpuMasterTimeline,
    ) -> u32 {
        match event {
            CpuSynchronousTimelineEvent::Bus(
                event @ (CpuBusEvent::HdmaInit | CpuBusEvent::HdmaStart),
            ) => {
                let state = &self.state;
                self.hdma
                    .stall(event, |a| {
                        if a & 0x400000 == 0 && a & 65535 < 0x2000 {
                            Some(state.ram[(a & 65535) as usize])
                        } else if (0x7e0000..0x800000).contains(&a) {
                            Some(state.ram[(a - 0x7e0000) as usize])
                        } else {
                            scroll_hdma_descriptor_byte(a)
                        }
                    })
                    .expect("exit HDMA operands")
            }
            _ => {
                self.timeline_event(event, clock.clock_master_cycles());
                0
            }
        }
    }
}
impl ZeldaState {
    pub(super) fn native_overworld_exit_store(&mut self, offset: usize, value: u16, width: u8) {
        match offset {
            0..=15 | 0x1c0..=0x1ff => {}
            0x11 => self.set_submodule(value as u8),
            0x17 => self.set_pending_nmi_subroutine(value as u8),
            0x20 | 0xe0 | 0xe2 | 0xe6 | 0xe8 | 0x618..=0x623 => {
                self.native_dungeon_camera_store(offset, value, width)
            }
            0x2b => self
                .follower_link_state_mut()
                .write_ground_movement_store(offset, value as u8),
            0x2d | 0x2e | 0x2f => {
                assert!(self.native_player_drawing_store(offset as u32, value, width));
            }
            0x30 => self.follower_link_state_mut().set_y_velocity(value as u8),
            0x26 | 0x67 => {
                assert!(self.native_load_player_initialization_store(offset, value, width));
            }
            0x84 | 0x86 | 0x88 => {
                let mut load = self.game_state.world.overworld.map16.active_load;
                match offset {
                    0x84 => load.src_off = value,
                    0x86 => load.dst_off = value,
                    _ => load.y_unit = value,
                }
                self.set_overworld_map16_load_state(load);
            }
            0x416 => self.set_screen_transition_direction_bits_word(read_le_u16(&self.ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2)),
            0x418 => self.set_screen_transition_word(read_le_u16(&self.ram, OVERWORLD_SCREEN_TRANSITION)),
            0x500..=0x53e => {
                assert_eq!(width, 2);
                self.set_overworld_map16_stripe_word((offset - 0x500) / 2, value);
            }
            0x624..=0x62a => {
                self.set_overworld_scroll_counter_for_axis((offset - 0x624) / 2, value)
            }
            0x69a => self.set_overworld_transition_countdown(value as u8),
            0x69e => self.set_overworld_vertical_scroll_delta(value),
            0x69f => self.set_overworld_horizontal_scroll_delta(value),
            0x1100..=0x13ff => {} // The upload buffer is canonically RAM-backed.
            _ => panic!("unowned native exit store {offset:05x}"),
        }
    }
    fn execute_native_overworld_exit(
        &mut self,
        entry: &NativeOverworldExitEntry,
        direction: ExitWalkDirection,
        charge: bool,
    ) -> Option<(NativeNmiContext, CpuCycleBudget, HdmaTimingState)> {
        let mut cpu = entry.context;
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.sp, cpu.d),
            (
                2,
                if direction == ExitWalkDirection::Down {
                    0xac8f
                } else {
                    0xacc2
                },
                0x1fa,
                0
            ),
            "exit caller admission"
        );
        assert_eq!(
            cpu.p & 0x38,
            0x30,
            "exit caller widths and binary arithmetic"
        );
        let mut hdma = entry.hdma.clone();
        let mut chain = ExitChain::new(direction);
        let mut budget = CpuCycleBudget::from_native_timeline(&entry.timeline);
        self.ram[EXIT_CALLER_RETURN_STACK..EXIT_CALLER_RETURN_STACK + 5].copy_from_slice(&[0x7d, 0xa4, 0x59, 0x80, 0]);
        for _ in 0..20000 {
            if chain.complete() {
                assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0xa47e, 0x1fc));
                return Some((cpu, budget, hdma));
            }
            let ((cycles, routine), advance) = budget.retire_native_instruction(|clock| {
                chain.step(
                    &mut cpu,
                    clock,
                    ExitBus {
                        state: self,
                        hdma: &mut hdma,
                    },
                )
            });
            if charge {
                crate::cycle_ledger::charge_routine(routine, cycles);
            }
            if advance != CpuWorkAdvance::Complete {
                return None;
            }
        }
        panic!("native exit did not return");
    }
    pub(super) fn begin_native_overworld_exit(&mut self, direction: ExitWalkDirection) -> bool {
        let Some(entry) = self.native_overworld_exit_entry.take() else {
            return false;
        };
        if entry.host != self.frame_ctr_dbg || self.game_state.enhanced_features.has(u32::MAX) {
            return false;
        }
        // Admission uses native execution on a private trial, not a guessed
        // cycle allowance. Suspended exits remain with the existing caller.
        let mut trial = self.clone();
        trial.rom.clear();
        if trial
            .execute_native_overworld_exit(&entry, direction, false)
            .is_none()
        {
            return false;
        }
        let (cpu, budget, hdma) = self
            .execute_native_overworld_exit(&entry, direction, true)
            .expect("admitted exit crossed NMI");
        if nmi::debug_frame_selection_env_matches(
            "ZELDA3_DEBUG_DIALOGUE_CPU_PHASE",
            self.frame_ctr_dbg,
        ) {
            eprintln!(
                "native-overworld-exit-return host={} retained_caller={} cpu={cpu:?} field={} raster={:?}",
                self.frame_ctr_dbg,
                entry.retain_return_phase,
                budget.field_index(),
                budget.raster_position()
            );
        }
        // A development caller seed can execute and validate the leaf, but
        // cannot establish a retained main-wait phase. Preserve clock
        // ownership only when dispatch started from a retained caller.
        if !entry.retain_return_phase {
            return true;
        }
        let checkpoint = RomCpuCheckpoint {
            entry_pc: 0x02a47e,
            stop_pc: 0x00805d,
            a: cpu.a,
            x: cpu.x,
            y: cpu.y,
            sp: cpu.sp,
            dp: cpu.d,
            db: cpu.db,
            carry: cpu.p & 1 != 0,
            zero: cpu.p & 2 != 0,
            overflow: cpu.p & 0x40 != 0,
            negative: cpu.p & 0x80 != 0,
            interrupt_disable: cpu.p & 4 != 0,
            decimal: false,
            accumulator_is_8_bit: cpu.p & 0x20 != 0,
            index_is_8_bit: cpu.p & 0x10 != 0,
            emulation: false,
            waiting: false,
            stack_address: 0,
            stack_bytes: &[],
        };
        let dma = self.dma_with_native_hdma_enable();
        // The retained source successor receives an exported cartridge image.
        // Its speculative WRAM is not a replacement for native gameplay.
        let mut run = RomCpuTimingRun::new(
            &self.rom,
            &self.wram_image(),
            &self.sram,
            &self.ppu,
            &dma,
            self.zelda_audio_apu_output_ports(),
            checkpoint,
        )
        .unwrap();
        run.install_native_callee_hdma(&hdma);
        run.set_joypad_input(entry.input);
        run.enable_cpu_write_trace();
        (
            self.native_overworld_packing_progress,
            self.native_overworld_hud_interruption,
        ) = overworld_packing_from_run(self, false, run, budget);
        true
    }
}
