//! Publications and retained devices for the spotlight's physical caller.
use super::native_item_graphics::DungeonReturnBus;
use super::*;
use crate::game_state::constants::{BIG_ROCK_STARTING_ADDRESS};
use crate::cycle_models::{intro_clear::Access, native_work::WorkBus};
use snes::{cpu_timeline::CpuSynchronousTimelineEvent, CpuMasterTimeline, HdmaTimingState};

/// The table builder reads both PPU counters and APUI bytes. Its caller must
/// retain these devices across instructions and NMI suspension, without
/// resampling a presentation PPU or replaying a host audio window.
pub(super) struct SpotlightDeviceBus<'a> {
    pub(super) caller: SpotlightCallerBus<'a>,
    beam: Option<snes::cpu_timeline::CpuSynchronousBeamPosition>,
}
impl<'a> SpotlightDeviceBus<'a> {
    pub(super) fn new(caller: SpotlightCallerBus<'a>) -> Self {
        assert!(caller.state.native_ppu.is_some(), "spotlight needs its retained PPU");
        Self { caller, beam: None }
    }
}
impl WorkBus for SpotlightDeviceBus<'_> {
    fn begin_main_iteration(&mut self, clock: &CpuMasterTimeline) {
        let state = &mut self.caller.state;
        if state.game_state.frame.main_module == 8 && state.game_state.frame.submodule == 0 {
            // The properties translation's existing upload owner starts at
            // INC $1A, before OAM clearing and module dispatch. Retain that
            // exact boundary from the live caller rather than losing its
            // provenance when the native closing caller returns.
            let at = clock.raster_position();
            state.spotlight_successor_cpu_entry_envelope = Some((at, at));
        }
    }
    fn inactive_sprite_body_mask(&mut self) -> u16 { self.caller.inactive_sprite_body_mask() }
    fn observe_clock(&mut self, clock: &CpuMasterTimeline) {
        self.caller.state.native_ppu.as_mut().unwrap().synchronize(clock);
        self.beam = clock.synchronous_beam_position();
    }
    fn access(&mut self, at: u64, access: Access) -> u16 {
        if let Some(value) = self.caller.state.native_cpu_math.access(access) {
            return value;
        }
        match access {
            Access::Read(address @ 0x2137..=0x213f, 1) => {
                return u16::from(self.caller.state.native_ppu.as_mut().unwrap()
                    .read(address as u16, self.beam.expect("clocked spotlight PPU read")).unwrap());
            }
            Access::Read(address @ 0x2140..=0x2143, 1) => {
                return u16::from(self.caller.state.native_main_port_at(at, (address - 0x2140) as u8, None));
            }
            Access::Write(address @ 0x2100..=0x213f, value, 1) => {
                self.caller.state.native_ppu.as_mut().unwrap()
                    .write(address as u16, value as u8, self.beam.expect("clocked spotlight PPU write"));
                if address != 0x2100 {
                    self.caller.state.zelda_ppu_write(address, value as u8);
                    return 0;
                }
            }
            _ => {}
        }
        self.caller.access(at, access)
    }
    fn retire_event(&mut self, event: CpuSynchronousTimelineEvent, clock: &CpuMasterTimeline) -> u32 {
        use snes::cpu_timeline::CpuBusEvent;
        if matches!(event, CpuSynchronousTimelineEvent::Bus(CpuBusEvent::HdmaInit | CpuBusEvent::HdmaStart)) {
            let state = &mut self.caller.state;
            self.caller.hdma.install_into(&mut state.dma);
            let cycles = crate::cycle_models::native_hdma::wait_event(event, clock,
                &state.ram, state.native_ppu.as_mut().unwrap(), &mut state.dma, &mut state.ppu);
            *self.caller.hdma = HdmaTimingState::new(&state.dma);
            return cycles;
        }
        self.timeline_event(event, clock.clock_master_cycles());
        0
    }
    fn end_scanline(&mut self, at: u64) {
        self.caller.state.native_cpu_end_scanline_at(at);
    }
}

pub(super) struct SpotlightCallerBus<'a> {
    pub(super) state: &'a mut ZeldaState,
    pub(super) hdma: &'a mut HdmaTimingState,
    pub(super) nmi_deadline: &'a mut Option<u64>,
}
impl WorkBus for SpotlightCallerBus<'_> {
    fn inactive_sprite_body_mask(&mut self) -> u16 {
        DungeonReturnBus::new(self.state, self.hdma, self.nmi_deadline).inactive_sprite_body_mask()
    }
    fn access(&mut self, at: u64, access: Access) -> u16 {
        if let Access::Read(address, width) = access {
            if address <= 0xffff {
                if let Some(low) = crate::cycle_models::main_entry::module_table_byte(address as u16) {
                    return if width == 1 { u16::from(low) } else {
                        u16::from_le_bytes([low, crate::cycle_models::main_entry::module_table_byte(address as u16 + 1).unwrap()])
                    };
                }
            }
            if let Some(low) = self.state.native_dungeon_entrance_table_byte(address) {
                return if width == 1 { u16::from(low) } else {
                    u16::from_le_bytes([low, self.state.native_dungeon_entrance_table_byte(address + 1).unwrap()])
                };
            }
            if let Some(low) = crate::cycle_models::spotlight_caller_return::table_byte(address) {
                return if width == 1 { u16::from(low) } else {
                    u16::from_le_bytes([low, crate::cycle_models::spotlight_caller_return::table_byte(address + 1).unwrap()])
                };
            }
        }
        if let Access::Write(address, value, width) = access {
            if (0x7f7000..=0x7f72ff).contains(&address) {
                let offset = (address - 0x7e0000) as usize;
                self.state.ram[offset..offset + usize::from(width)]
                    .copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
                return 0;
            }
            let address = if address & 0x400000 == 0 && address & 65535 < 0x2000 {
                address & 65535
            } else {
                address
            };
            if self.state.native_dungeon_spotlight_store(address, value, width) {
                let offset = address as usize;
                self.state.ram[offset..offset + usize::from(width)]
                    .copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
                return 0;
            }
            if (!matches!(address, 0xae8 | 0xaea) || width == 1)
                && self.state.native_sprite_main_store(address, value, width) {
                let offset = address as usize;
                self.state.ram[offset..offset + usize::from(width)]
                    .copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
                return 0;
            }
            if self.state.native_closing_caller_store(address, value, width) { return 0; }
            match address {
                0x1a => {
                    assert_eq!(width, 1);
                    self.state.frame_state_mut().set_frame_counter(value as u8);
                    return 0;
                }
                0x2100 => {
                    assert_eq!(width, 1);
                    self.state.native_main_inidisp_write(at, value as u8);
                    return 0;
                }
                0x420c => {
                    assert_eq!((value, width), (0, 1));
                    self.hdma.install_into(&mut self.state.dma);
                    for channel in &mut self.state.dma.channel {
                        channel.hdma_active = false;
                    }
                    *self.hdma = HdmaTimingState::new(&self.state.dma);
                    return 0;
                }
                _ => {}
            }
        }
        DungeonReturnBus::new(self.state, self.hdma, self.nmi_deadline).access(at, access)
    }
    fn retire_event(
        &mut self,
        event: CpuSynchronousTimelineEvent,
        clock: &CpuMasterTimeline,
    ) -> u32 {
        DungeonReturnBus::new(self.state, self.hdma, self.nmi_deadline).retire_event(event, clock)
    }
}
impl ZeldaState {
    pub(super) fn native_closing_caller_store(&mut self, address: u32, value: u16, width: u8) -> bool {
        if !matches!(address, 0x10
                | 0x11
                | 0x13
                | 0x1e
                | 0x1f
                | 0x20
                | 0x26
                | 0x3b
                | 0x50
                | 0x67
                | 0x96..=0x98
                | 0x9b..=0x9e
                | 0xb0
                | 0x12a
                | 0x12c
                | 0x12f
                | 0x308..=0x30a
                | 0x376
                | 0x3ef
                | 0x4a0
                | 0x690
                | 0x692
                | 0x699
                | 0x69a
                | 0x1f0c) { return false; }
        let offset = address as usize;
        self.ram[offset..offset + usize::from(width)]
            .copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
        self.native_spotlight_caller_store(address, value, width);
        true
    }
    fn native_spotlight_caller_store(&mut self, address: u32, value: u16, width: u8) {
        if self.native_dungeon_spotlight_store(address, value, width) {
            return;
        }
        match address {
            0x10 => self.set_main_module(value as u8),
            0x11 => self.set_submodule(value as u8),
            0xb0 => self.set_subsubmodule(value as u8),
            0x13 => self.set_screen_brightness(value as u8),
            0x9b => {
                assert_eq!(width, 1);
                self.set_hdma_enable_mask(value as u8);
            }
            0x12a => {
                assert_eq!(value, 0);
                self.deactivate_nmi_thread();
            }
            0x1f0c => {
                assert_eq!(value, 0);
                self.clear_pending_polyhedral_update();
            }
            0x20 | 0x26 => assert!(self.native_player_drawing_store(address, value, width)),
            0x67 => assert!(self.native_load_player_initialization_store(address as usize, value, width)),
            0x3b | 0x50 | 0x308..=0x30a | 0x376 => assert!(
                self.native_load_player_initialization_store(address as usize, value, width)
            ),
            0x3ef => {
                assert_eq!(value, 0);
                self.follower_link_state_mut().clear_force_hold_sword_up();
            }
            0x690 => self
                .dungeon_doors_mut()
                .set_door_animation_step_low(value as u8),
            0x692 => self
                .dungeon_doors_mut()
                .set_door_open_counter_low(value as u8),
            0x699 => {
                let word = read_le_u16(&self.ram, BIG_ROCK_STARTING_ADDRESS);
                self.dungeon_object_tracking_mut()
                    .set_big_rock_starting_address(word);
            }
            0x69a => self.set_overworld_transition_countdown(value as u8),
            0x12f => self.set_sound_effect_2(value as u8),
            0x12c => self.set_music_control(value as u8),
            0x4a0 => {
                assert_eq!((value, width), (0, 1));
                self.clear_floor_changed_timer_low();
            }
            0x9c..=0x9e => {
                assert!(matches!(width,1|2) && address+u32::from(width)<=0x9f);
                for byte in 0..width {
                    self.palette_filter_mut().set_fixed_color_component(
                        (address+u32::from(byte)-0x9c) as usize,(value>>(byte*8)) as u8);
                }
            }
            _ => unreachable!("spotlight caller publication {address:06x}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cycle_models::{
        nmi_stack::NativeNmiContext, spotlight_caller_return::{ClosingSpotlight, SpotlightCallerReturn},
    };
    #[test]
    fn closing_runtime_and_main_wait_handoffs_preserve_source_boundaries() {
        spotlight_runtime_and_main_wait_handoffs(false);
    }

    #[test]
    fn opening_runtime_and_main_wait_handoffs_preserve_source_boundaries() {
        spotlight_runtime_and_main_wait_handoffs(true);
    }

    fn spotlight_runtime_and_main_wait_handoffs(opening:bool) {
        let module=if opening {16} else {15};
        let module_pc=if opening {0x9ad7} else {0x9982};
        use crate::cycle_models::{native_ppu::NativePpuState, startup::Startup};
        let rom = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc")).unwrap();
        let assets = std::fs::read(std::env::var("ZELDA3_ASSETS").unwrap()).unwrap();
        for address in 0x0abddd..=0x0abde3 {
            let offset = ((address >> 16) & 0x7f) * 0x8000 + (address & 0x7fff);
            assert_eq!(ZeldaState::native_hdma_descriptor_byte(address), Some(rom[offset as usize]));
        }
        for (submodule, radius, paused) in [(0, 28u16, false), (0, 28, true), (0, 118, false), (0, 118, true),
            (1, 28, false), (1, 28, true), (1, 118, false), (1, 118, true)] {
            let mut actual = ZeldaState::new();
            actual.set_rom_startup_timing(true);
            actual.set_assets(&assets).unwrap();
            actual.rom.clear();
            actual.ram.fill(0);
            actual.ram[0x10] = module;
            actual.ram[0x11] = submodule;
            actual.ram[0x1b] = u8::from(!opening);
            actual.ram[0x10c] = 9;
            actual.ram[0x2cd..0x2cf].copy_from_slice(&16u16.to_le_bytes());
            actual.ram[0x20..0x24].copy_from_slice(&[0x70, 1, 0x40, 2]);
            actual.ram[0x67c..0x67e].copy_from_slice(&radius.to_le_bytes());
            actual.ram[0x67e..0x680].copy_from_slice(&2u16.to_le_bytes());
            if paused {
                // Exercise the source's per-actor pause and submodule gates
                // with the global modal-pause flag clear.
                for slot in [0, 14, 15] {
                    actual.ram[0xdd0 + slot] = 9;
                    actual.ram[0xe20 + slot] = if slot == 15 { 0x76 } else { 0x73 };
                    actual.ram[0xe90 + slot] = if slot == 14 { 2 } else if slot == 0 { 1 } else { 0 };
                    actual.ram[0xf00 + slot] = if submodule == 0 { 2 } else { 0 };
                    // On-screen drawing clears actor pause before the inactive
                    // return helper. Off-screen drawing sets it again, which
                    // is the per-actor gate exercised by this fixture.
                    if submodule == 0 { actual.ram[0xd30 + slot] = 2; }
                }
            }
            actual.sync_native_game_state_from_ram();
            if paused {
                for (address, value) in [(0x4370, 0x42), (0x4371, 0x1b),
                    (0x4372, 0xdd), (0x4373, 0xbd), (0x4374, 10), (0x4377, 0)] {
                    actual.dma.write_register(address, value);
                }
                let channel = &mut actual.dma.channel[7];
                channel.hdma_active = true;
                channel.table_adr = 0xbde0;
                channel.size = 0x1b00;
                channel.rep_count = 0xf0;
                channel.do_transfer = true;
                channel.terminated = false;
            }
            let cpu = actual.native_startup_cpu.as_mut().unwrap();
            cpu.timeline.begin_synchronous_timeline().unwrap();
            cpu.startup = Startup::from_main_caller_for_test(NativeNmiContext {
                a: 0x1200, x: 3, y: 7, d: 0, db: 0, bank: 0, pc: 0x8038,
                sp: 0x1ff, p: 0x30,
            }, &cpu.timeline);
            actual.native_ppu = Some(NativePpuState::reset(&cpu.timeline));
            actual.frame_ctr_dbg = (cpu.timeline.field_index() + 1) as u32;
            assert!(actual.try_native_startup_main_entry());
            actual.main_loop_caller_phase = crate::cycle_models::main_wait::MainLoopCallerPhase::ModuleRouting;
            let retained = actual.native_startup_cpu.as_ref().unwrap();
            let context = retained.startup.entered_module_context().unwrap();
            assert_eq!((context.bank, context.pc, context.sp), (2, module_pc, 0x1fc));
            let clock = retained.timeline.clone();
            actual.native_main_port_at(clock.clock_master_cycles(), 0, None);
            let mut machine = snes::Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap().machine().snes().clone();
            machine.ram.copy_from_slice(&actual.ram);
            machine.dma = actual.dma.clone();
            machine.nmi_enabled = true;
            machine.cpu.a = context.a;
            machine.cpu.x = context.x;
            machine.cpu.y = context.y;
            machine.cpu.dp = context.d;
            machine.cpu.db = context.db;
            machine.cpu.k = context.bank;
            machine.cpu.pc = context.pc;
            machine.cpu.sp = context.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(context.p);
            let mut source = snes::RomCpuTimingProbe::new(machine, clock,
                actual.native_ppu.as_ref().unwrap().read_state()).unwrap();
            source.attach_apu_port_owner(actual.native_apu_port_owner_for_test()).unwrap();
            if opening {actual.module10_spotlight_open();} else {actual.module0f_spotlight_close();}
            if actual.native_startup_cpu.as_ref().unwrap().startup.intro_clear_accepted_context().is_none()
                && actual.native_startup_cpu.as_ref().unwrap().startup.intro_tail_returned_context().is_none() {
                // A host exit can precede the NMI deadline by one instruction.
                // Retire that instruction on the same clock before comparing
                // the hardware entry; do not advance or run its handler early.
                actual.run_native_intro_clear_until_boundary();
            }
            let retained = actual.native_startup_cpu.as_ref().unwrap();
            assert!(if opening {retained.startup.opening_active()} else {retained.startup.closing_active()});
            let accepted = retained.startup.intro_clear_accepted_context();
            let boundary = accepted.or_else(|| retained.startup.intro_tail_returned_context())
                .expect("spotlight must expose its NMI or actual main return");
            let before_entry = retained.timeline.clock_master_cycles() - if accepted.is_some() {62} else {0};
            while source.timeline().clock_master_cycles() < before_entry {
                source.step().unwrap();
            }
            assert_eq!(source.timeline().clock_master_cycles(), before_entry);
            if accepted.is_some() {
                assert!(source.pending_nmi_acceptance().is_some());
                source.accept_native_nmi().unwrap();
            }
            let c = &source.snes().cpu;
            assert_eq!(boundary, NativeNmiContext { a: c.a, x: c.x, y: c.y, d: c.dp,
                db: c.db, bank: c.k, pc: c.pc, sp: c.sp, p: c.pack_flags() });
            assert_eq!(retained.timeline, *source.timeline());
            let start = crate::game_state::constants::HDMA_TABLE_DYNAMIC;
            let end = start + 0x1c0;
            assert_eq!(&actual.ram[..start], &source.snes().ram[..start]);
            assert_eq!(&actual.ram[end..], &source.snes().ram[end..]);
            assert_eq!(&actual.ram[start..end], &source.snes().ram[0x1b00..0x1cc0]);
            let source_state=crate::game_state::GameState::load_from_ram(&source.snes().ram);
            assert_eq!(actual.game_state.sprites.ancilla_slots,source_state.sprites.ancilla_slots);
            assert_eq!(actual.native_ppu.as_ref().unwrap().read_state(), *source.ppu_reads());
            assert_eq!(bincode::serialize(&actual.dma.channel).unwrap(),
                bincode::serialize(&source.snes().dma.channel).unwrap());
            assert_eq!(actual.native_apu_port_owner_for_test().clock().checkpoint(),
                source.apu_ports().unwrap().clock().checkpoint());
            for _ in 0..8 {
                if actual.native_startup_cpu.as_ref().unwrap().startup.intro_tail_returned_context().is_some() { break; }
                actual.frame_ctr_dbg += 1;
                actual.resume_native_intro_clear_host(0);
                if actual.native_startup_cpu.as_ref().unwrap().startup.intro_tail_returned_context().is_some() {
                    break;
                }
            }
            let returned = actual.native_startup_cpu.as_ref().unwrap().startup
                .intro_tail_returned_context().expect("spotlight must return after its retained NMI");
            assert_eq!((returned.bank, returned.pc, returned.sp), (0, 0x8034, 0x1ff));
            assert_eq!(actual.main_loop_caller_phase,
                crate::cycle_models::main_wait::MainLoopCallerPhase::Waiting);
            assert!(!actual.game_state.display.nmi_update_is_latched());
            assert!(actual.main_loop_sprite_preparation_completed);
            if actual.game_state.frame.main_module == module {
                let clock = actual.native_startup_cpu.as_ref().unwrap().timeline.clone();
                let mut context = returned;
                let mut hdma = HdmaTimingState::new(&actual.dma);
                actual.retain_native_dungeon_main_wait(&mut context, &clock, &mut hdma, &mut None);
                let next = actual.native_main_wait_cpu_phase.as_ref().unwrap();
                let retained_clock = next.budget.retained_native_timeline().clock_master_cycles();
                let next_host = next.host;
                // Exercise the gameplay handoff contract: its old owner has
                // returned, while the native main-wait context stays live.
                actual.native_startup_cpu.as_mut().unwrap().startup.complete_externally();
                actual.frame_ctr_dbg = next_host;
                actual.adopt_spotlight_main_wait_context();
                assert!(actual.native_main_wait_cpu_phase.is_none());
                assert_eq!(actual.native_startup_cpu.as_ref().unwrap().timeline.clock_master_cycles(), retained_clock);
                assert_eq!(actual.advance_native_startup_host(0), Some(true));
                assert!(actual.try_native_startup_main_entry());
                let context = actual.native_startup_cpu.as_ref().unwrap().startup.entered_module_context().unwrap();
                assert_eq!((context.bank, context.pc, context.sp), (2, module_pc, 0x1fc));
            }
            assert!(actual.rom.is_empty());
        }
    }

    #[test]
    fn production_spotlight_caller_bus_matches_source_without_runtime_rom() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let template = std::env::var_os("ZELDA3_ASSETS").map(|path| {
            let mut state = ZeldaState::new();
            state.set_rom_startup_timing(true);
            state.set_assets(&std::fs::read(path).unwrap()).unwrap();
            state
        });
        let mut instructions = 0;
        let mut device_instructions = 0;
        let mut device_cases = 0;
        let mut root_cases = 0;
        for module in [6, 9, 15, 16] {
            for submodule in [0, 1] {
                for case in 0..32usize {
                  for full_table in [false, true] {
                    if full_table && (module != 15 || submodule != 1 || template.is_none()) { continue; }
                   for from_entry in [false, true] {
                    if from_entry && !full_table { continue; }
                    let mut actual = if full_table { template.as_ref().unwrap().clone() }
                        else { ZeldaState::new() };
                    actual.rom.clear();
                    actual.ram.fill(0);
                    let r = &mut actual.ram;
                    r[0x10] = module;
                    r[0x11] = submodule;
                    r[0x1b] = 0;
                    r[0xa0..0xa2]
                        .copy_from_slice(&[0x20u16, 0x120, 0x21, 0xffff][case % 4].to_le_bytes());
                    r[0x8a..0x8c].copy_from_slice(
                        &[3u16, 5, 7, 0x43, 0x45, 0x47, 0, 0xffff][case / 4 % 8].to_le_bytes(),
                    );
                    r[0x696] = u8::from(case & 16 != 0);
                    r[0x698..0x69a].copy_from_slice(
                        &[0u16, 1, 0x100, 0x101, 0x7f01, 0x8000, 0x8001, 0xff00][case / 4 % 8]
                            .to_le_bytes(),
                    );
                    r[0x50] = [0, 1, 0xfe, 0xff][case % 4];
                    r[0x10a] = 0xa5;
                    r[0x3b] = 3;
                    r[0x308] = 1 << (case % 8);
                    r[0x309] = (case / 8 % 4) as u8;
                    r[0x30a] = (case / 4 % 3) as u8;
                    r[0x376] = 1;
                    r[0x3ef] = 1;
                    r[0x96..0x99].fill(0xff);
                    r[0x1e] = 0xff;
                    r[0x1f] = 0xff;
                    r[0x12a] = 1;
                    r[0x1f0c] = 1;
                    r[0xc148..0xc14a]
                        .copy_from_slice(&(case as u16).wrapping_mul(997).to_le_bytes());
                    r[0x20..0x24].copy_from_slice(&[0x70, 1, 0x40, 2]);
                    r[0x2f] = (case / 8 % 4 * 2) as u8;
                    r[0x2e] = (case / 4 % 6) as u8;
                    r[0x67] = [0, 4, 8, 12][case % 4];
                    r[0xf359] = (case / 8 % 5) as u8;
                    r[0xf35a] = (case / 16 % 4) as u8;
                    r[0x1fb..0x200].copy_from_slice(&[0xe1, 0x9a, 0x59, 0x80, 0]);
                    for (i, b) in r[0xa20..0xaa0].iter_mut().enumerate() {
                        *b = (i as u8).wrapping_mul(37) ^ case as u8;
                    }
                    if module == 15 {
                        actual.ram[0x1b] = 1;
                        actual.ram[0x1fb..0x1fd].copy_from_slice(&[0x8c, 0x99]);
                    }
                    if full_table {
                        actual.ram[0x10c] = 9;
                        if from_entry && case & 8 != 0 {
                            // Closing suspends actors without changing their
                            // real Sprite_Main call frame or descending loop.
                            actual.ram[0xfc1] = 1;
                            for slot in [0, 14, 15] {
                                actual.ram[0xdd0 + slot] = 9;
                                actual.ram[0xe20 + slot] = if slot == 15 { 0x76 } else { 0x73 };
                                actual.ram[0xe90 + slot] = if slot == 14 { 2 } else if slot == 0 { 1 } else { 0 };
                                actual.ram[0xde0 + slot] = (case % 2) as u8;
                                actual.ram[0xdc0 + slot] = (case / 4 % 2) as u8;
                                actual.ram[0xd10 + slot] = (case as u8).wrapping_mul(7);
                                actual.ram[0xd00 + slot] = (case as u8).wrapping_mul(11);
                            }
                        }
                        actual.ram[0x67c..0x67e].copy_from_slice(&[0u16, 28, 118, 119][case % 4].to_le_bytes());
                        actual.ram[0x67e..0x680].copy_from_slice(&2u16.to_le_bytes());
                        actual.ram[0x1fd..0x200].copy_from_slice(&[0x59, 0x80, 0]);
                        if case & 4 != 0 {
                            // Channel7 reads the changing physical window table.
                            actual.ram[0x1800..0x1804].copy_from_slice(&[0x81, 0, 0x1b, 0]);
                            for (address, value) in [(0x4370, 0x41), (0x4371, 0x26),
                                (0x4372, 0), (0x4373, 0x18), (0x4374, 0), (0x4377, 0)] {
                                actual.dma.write_register(address, value);
                            }
                            actual.dma.channel[7].hdma_active = true;
                            // This fixture enters mid-field, after initialization.
                            let channel = &mut actual.dma.channel[7];
                            channel.table_adr = 0x1803;
                            channel.size = 0x1b00;
                            channel.rep_count = 0x81;
                            channel.do_transfer = true;
                            channel.terminated = false;
                        }
                    }
                    actual.sync_native_game_state_from_ram();
                    let mut cpu = NativeNmiContext {
                        a: 0x1200,
                        x: 3,
                        y: 7,
                        d: 0,
                        db: 0,
                        bank: 2,
                        pc: 0x9a1d,
                        sp: 0x1fa,
                        p: 0x30 | [0, 1, 0x41, 0xc3][case % 4],
                    };
                    if full_table { cpu.pc = if from_entry { 0x9982 } else { 0x9986 }; cpu.sp = 0x1fc; }
                    let mut machine = if full_table {
                        snes::Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap().machine().snes().clone()
                    } else {
                        let mut machine = snes::Snes::new();
                        machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                        machine
                    };
                    machine.ram.copy_from_slice(&actual.ram);
                    if full_table { machine.dma = actual.dma.clone(); }
                    machine.cpu.a = cpu.a;
                    machine.cpu.x = cpu.x;
                    machine.cpu.y = cpu.y;
                    machine.cpu.k = cpu.bank;
                    machine.cpu.pc = cpu.pc;
                    machine.cpu.sp = cpu.sp;
                    machine.cpu.e = false;
                    machine.cpu.unpack_flags(cpu.p);
                    let mut clock = CpuMasterTimeline::at_raster(
                        4921 + case as u64,
                        snes::CpuRasterPosition::new(
                            [20, 224, 261, 170][case % 4],
                            [0, 1300, 1350, 468][case % 4],
                        ),
                        snes::CpuBusWorkload::default(),
                        snes::CpuFieldTiming::non_interlace(case & 1 == 0),
                    );
                    if full_table {
                        clock = actual.native_startup_cpu.as_ref().unwrap().timeline.clone();
                        actual.native_ppu = Some(crate::cycle_models::native_ppu::NativePpuState::reset(&clock));
                        actual.native_main_port_at(clock.clock_master_cycles(), 0, None);
                    }
                    let mut source = snes::RomCpuTimingProbe::new(
                        machine,
                        clock.clone(),
                        snes::SourcePpuReadState::snes9x_reset(),
                    )
                    .unwrap();
                    if full_table { source.attach_apu_port_owner(actual.native_apu_port_owner_for_test()).unwrap(); }
                    clock.begin_synchronous_timeline().unwrap();
                    let mut hdma = HdmaTimingState::new(&actual.dma);
                    let mut deadline = None;
                    let mut work = if module == 15 { SpotlightCallerReturn::closing() }
                        else { SpotlightCallerReturn::default() };
                    let mut closing = full_table.then(|| if from_entry { ClosingSpotlight::from_module_entry() }
                        else { ClosingSpotlight::after_sprite_return() });
                    for _ in 0..200000 {
                        if closing.as_ref().map_or_else(|| work.complete(), |work| work.complete()) {
                            break;
                        }
                        let pc = (u32::from(cpu.bank) << 16) | u32::from(cpu.pc);
                        let receipt = source.step().unwrap();
                        if let Some(closing) = &mut closing {
                            closing.step(&mut cpu, &mut clock, SpotlightDeviceBus::new(SpotlightCallerBus {
                                state: &mut actual, hdma: &mut hdma, nmi_deadline: &mut deadline,
                            })).unwrap();
                            device_instructions += 1;
                        } else { work.step(
                            &mut cpu,
                            &mut clock,
                            SpotlightCallerBus {
                                state: &mut actual,
                                hdma: &mut hdma,
                                nmi_deadline: &mut deadline,
                            },
                        )
                        .unwrap(); }
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
                            "module={module} submodule={submodule} case={case} pc={pc:06x}"
                        );
                        assert_eq!(clock, *source.timeline(), "pc={pc:06x}");
                        if full_table {
                            assert_eq!(actual.native_apu_port_owner_for_test().clock().checkpoint(),
                                source.apu_ports().unwrap().clock().checkpoint(),
                                "case={case} pc={pc:06x} APU clock; source transactions={:?}", receipt.transactions);
                        }
                        let reference = &source.snes().ram;
                        let ram_matches = if full_table {
                            // The port's typed table also publishes its documented
                            // relocated companion. Compare that companion to the
                            // source's physical table, and every other byte directly.
                            let start = crate::game_state::constants::HDMA_TABLE_DYNAMIC;
                            let end = start + 0x1c0;
                            actual.ram[..start] == reference[..start]
                                && actual.ram[end..] == reference[end..]
                                && actual.ram[start..end] == reference[0x1b00..0x1cc0]
                        } else { actual.ram == *reference };
                        assert!(ram_matches,
                            "module={module} submodule={submodule} case={case} pc={pc:06x} RAM");
                        if full_table {
                            let reference = &source.snes().ppu;
                            assert_eq!((actual.ppu.window1_left, actual.ppu.window1_right,
                                actual.ppu.window2_left, actual.ppu.window2_right),
                                (reference.window1_left, reference.window1_right,
                                reference.window2_left, reference.window2_right), "pc={pc:06x} HDMA windows");
                        }
                        work = work.clone();
                        closing = closing.clone();
                        instructions += 1;
                    }
                    assert!(closing.as_ref().map_or_else(|| work.complete(), |work| work.complete()));
                    if full_table {
                        device_cases += 1;
                        root_cases += usize::from(from_entry);
                        let native = actual.native_apu_port_owner_for_test();
                        let reference = source.take_apu_port_owner().unwrap();
                        assert_eq!(native.clock().checkpoint(), reference.clock().checkpoint());
                        assert_eq!(native.machine().save_c_saveload_prefix(), reference.machine().save_c_saveload_prefix());
                        assert_eq!(bincode::serialize(&native.machine().spc).unwrap(), bincode::serialize(&reference.machine().spc).unwrap());
                        assert_eq!(bincode::serialize(&actual.dma.channel).unwrap(),
                            bincode::serialize(&source.snes().dma.channel).unwrap());
                    }
                    assert!(actual.rom.is_empty());
                    let mut fresh = actual.clone();
                    fresh.sync_native_game_state_from_ram();
                    assert_eq!(
                        actual.game_state.player.follower_link,
                        fresh.game_state.player.follower_link
                    );
                    let publish = |s: &GameState, r: &mut Vec<u8>| {
                        s.frame.write_to_ram(r);
                        s.system_signals.write_to_ram(r);
                        s.player.write_to_ram(r);
                        s.world.write_to_ram(r);
                        s.display.write_to_ram(r);
                        s.oam.write_to_ram(r);
                    };
                    let mut a = actual.ram.clone();
                    let mut b = a.clone();
                    publish(&actual.game_state, &mut a);
                    publish(&fresh.game_state, &mut b);
                    let differences = a
                        .iter()
                        .zip(&b)
                        .enumerate()
                        .filter_map(|(i, (a, b))| (i >= 0x10 && a != b).then_some((i, *a, *b)))
                        .take(16)
                        .collect::<Vec<_>>();
                    assert_eq!(
                        differences,
                        vec![],
                        "module={module} submodule={submodule} case={case} typed projection"
                    );
                    assert_eq!(
                        actual
                            .game_state
                            .dungeon
                            .object_tracking
                            .big_rock_starting_address(),
                        read_le_u16(&actual.ram, 0x698)
                    );
                  }
                  }
                }
            }
        }
        eprintln!("256 production spotlight caller returns, including64 closing, {} Source instruction boundaries; empty runtime ROM", instructions - device_instructions);
        eprintln!("{device_cases} complete closing device-bus cases, {root_cases} from module entry, {device_instructions} Source instruction boundaries");
        if template.is_some() { assert_eq!((device_cases, root_cases), (64, 32)); }
    }
}
