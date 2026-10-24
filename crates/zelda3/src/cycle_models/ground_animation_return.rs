//! Ground animation, camera and ancilla returns through the dungeon push call.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, ZERO},
    player_animation::PlayerAnimation,
};
use snes::CpuMasterTimeline;
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct GroundAnimationReturn {
    pc: Option<u16>,
    animation: Option<PlayerAnimation>,
    complete: bool,
    camera: Option<super::ground_camera_return::GroundCameraReturn>,
    skip_camera: bool,
    ancilla: Option<super::player_ancilla_return::PlayerAncillaReturn>,
    push: Option<super::dungeon_push_caller_return::DungeonPushCallerReturn>,
    native_push_handler: bool,
    native_after_push: bool,
    native_room_tags: bool,
    native_interactions: bool,
    native_edge: bool,
    native_lamp: bool,
    native_sprite_scroll: bool,
    native_drawing: bool,
    native_sprite_prefix: bool,
    native_garnish: bool,
    native_follower: bool,
    native_ancilla: bool,
    inactive_slot_bodies: u16,
    live_inactive_slot_bodies: bool,
    after_push: Option<super::dungeon_after_push::DungeonAfterPush>,
}
impl GroundAnimationReturn {
    pub(crate) fn with_live_inactive_slot_bodies(mut self, enabled:bool)->Self {self.live_inactive_slot_bodies=enabled;self}
    #[cfg(feature = "rom-timing")]
    pub(crate) fn with_inactive_slot_bodies(mut self, mask:u16)->Self {self.inactive_slot_bodies=mask;self}
    pub(crate) fn with_camera_skip(skip_camera: bool) -> Self {Self {skip_camera,..Default::default()}}
    pub(crate) fn with_native_push_handler(mut self, enabled: bool) -> Self {self.native_push_handler=enabled;self}
    pub(crate) fn with_native_ancilla(mut self,enabled:bool)->Self {self.native_ancilla=enabled;self}
    pub(crate) fn with_native_follower(mut self,enabled:bool)->Self {self.native_follower=enabled;self}
    pub(crate) fn with_native_garnish(mut self,enabled:bool)->Self {self.native_garnish=enabled;self}
    pub(crate) fn with_native_sprite_prefix(mut self,enabled:bool)->Self {self.native_sprite_prefix=enabled;self}
    pub(crate) fn sprite_prefix_active(&self)->bool {self.after_push.as_ref().is_some_and(super::dungeon_after_push::DungeonAfterPush::sprite_prefix_active)}
    pub(crate) fn with_native_drawing(mut self,enabled:bool)->Self {self.native_drawing=enabled;self}
    pub(crate) fn drawing_active(&self)->bool {self.after_push.as_ref().is_some_and(super::dungeon_after_push::DungeonAfterPush::drawing_active)}
    pub(crate) fn with_native_sprite_scroll(mut self,enabled:bool)->Self {self.native_sprite_scroll=enabled;self}
    pub(crate) fn with_native_lamp(mut self,enabled:bool)->Self {self.native_lamp=enabled;self}
    pub(crate) fn with_native_edge(mut self,enabled:bool)->Self {self.native_edge=enabled;self}
    pub(crate) fn with_native_interactions(mut self,enabled:bool)->Self {self.native_interactions=enabled;self}
    pub(crate) fn with_native_room_tags(mut self,enabled:bool)->Self {self.native_room_tags=enabled;self}
    pub(crate) fn with_native_after_push(mut self, enabled: bool) -> Self {self.native_after_push=enabled;self}
    pub(crate) fn after_push_tail(&self) -> Option<super::dungeon_after_push::DungeonAfterPushTail> {self.after_push.as_ref().and_then(super::dungeon_after_push::DungeonAfterPush::tail)}
    pub(crate) fn after_push_active(&self) -> bool {self.after_push.is_some()}
    pub(crate) fn push_tail(&self) -> Option<super::dungeon_push_caller_return::PushCallerTail> {self.push.as_ref().and_then(super::dungeon_push_caller_return::DungeonPushCallerReturn::tail)}
    pub(crate) fn push_active(&self) -> bool {self.push.is_some()}
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn camera_tail(&self) -> Option<super::ground_camera_return::GroundCameraTail> {
        self.camera.as_ref().and_then(super::ground_camera_return::GroundCameraReturn::tail)
    }
    pub(crate) fn ancilla_tail(&self) -> Option<super::player_ancilla_return::PlayerAncillaTail> {self.ancilla.as_ref().and_then(super::player_ancilla_return::PlayerAncillaReturn::tail)}
    pub(crate) fn routine(&self) -> u32 {
        if let Some(after_push)=&self.after_push {after_push.routine()} else if self.push.is_some() {
            0x01d81b
        } else if self.ancilla.is_some() {
            0x078689
        } else if self.camera.is_some() {
            0x07e8f0
        } else if self.animation.is_some() {
            0x07e6a6
        } else {
            0x0782c5
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
        if let Some(after_push)=&mut self.after_push {
            let cycles=after_push.step(cpu,clock,bus);self.complete=after_push.tail().is_some();return cycles;
        }
        if let Some(push)=&mut self.push {
            let cycles=push.step(cpu,clock,bus);
            if self.native_after_push && push.tail()==Some(super::dungeon_push_caller_return::PushCallerTail::Returned) {self.after_push=Some(super::dungeon_after_push::DungeonAfterPush::default().with_native_room_tags(self.native_room_tags).with_native_interactions(self.native_interactions).with_native_edge(self.native_edge).with_native_lamp(self.native_lamp).with_native_sprite_scroll(self.native_sprite_scroll).with_native_drawing(self.native_drawing).with_native_sprite_prefix(self.native_sprite_prefix).with_native_garnish(self.native_garnish).with_native_follower(self.native_follower).with_native_ancilla(self.native_ancilla).with_inactive_slot_bodies(self.inactive_slot_bodies).with_live_inactive_slot_bodies(self.live_inactive_slot_bodies));}
            else {self.complete=push.tail().is_some();}
            return cycles;
        }
        if let Some(ancilla)=&mut self.ancilla {
            let cycles=ancilla.step(cpu,clock,bus);
            if ancilla.tail()==Some(super::player_ancilla_return::PlayerAncillaTail::Returned) {
                self.push=Some(super::dungeon_push_caller_return::DungeonPushCallerReturn::with_handler_deferred(!self.native_push_handler));
            } else {self.complete=ancilla.tail().is_some();}
            return cycles;
        }
        if let Some(camera) = &mut self.camera {
            let cycles=camera.step(cpu,clock,bus);
            if camera.tail()==Some(super::ground_camera_return::GroundCameraTail::Returned) {self.ancilla=Some(Default::default());} else {self.complete=camera.tail().is_some();}
            return cycles;
        }
        if let Some(animation) = &mut self.animation {
            let cycles = animation.step(cpu, clock, bus);
            if animation.complete() {
                assert_eq!((cpu.bank, cpu.pc, cpu.sp), (7, 0x82c9, 0x1f4));
                self.animation = None;
            }
            return cycles;
        }
        let pc = self.pc.unwrap_or(0x82c5);
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d, cpu.db, cpu.p & 0x38),
            (7, pc, 0, 7, 0x30)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        let next = match pc {
            0x82c5 => {
                w.operand_then_internal(3);
                w.push(w.cpu.bank as u16, 1);
                w.push(pc + 3, 2);
                w.cpu.bank = 7;
                self.animation = Some(PlayerAnimation::default());
                0xe6a6
            }
            0x82c9 => {
                w.drain(16);
                let v = w.read(w.absolute(0x377), 1);
                w.load(v, 1);
                0x82cc
            }
            0x82cc => {
                if w.branch(w.cpu.p & ZERO != 0) {
                    0x82d2
                } else {
                    0x82ce
                }
            }
            0x82ce | 0x82d0 => {
                w.dp_store(if pc == 0x82ce { 0x30 } else { 0x31 }, 0, 1);
                pc + 2
            }
            0x82d2 => {
                w.drain(16);
                w.write(w.absolute(0x302), 0, 1);
                0x82d5
            }
            0x82d5 => {
                w.item_call();
                if self.skip_camera {self.complete=true;} else {self.camera=Some(Default::default());}
                assert_eq!(w.cpu.sp, 0x1f2);
                0xe8f0
            }
            _ => panic!("ground animation return PC {pc:04x}"),
        };
        w.cpu.pc = next;
        self.pc = Some(if pc == 0x82c5 { 0x82c9 } else { next });
        Some(w.cycles)
    }
}

#[cfg(test)]
pub(crate) fn seed_case(ram: &mut [u8], case: usize, mode: usize) {
    let mode = mode & 15;
    ram[0x26] = [0, 1, 2, 4, 8, 12, 15, 255][case % 8];
    ram[0x5d] = match mode {
        2 => 4,
        3 => 23,
        _ => 0,
    };
    ram[0x50] = if mode == 14 { 0x80 } else { 0 };
    ram[0x6a] = (case & 1) as u8;
    ram[0x6c] = [0, 1, 2][case / 4 % 3];
    ram[0x2f] = (case % 4 * 2) as u8;
    ram[0x2d] = [0, 1, 2, 3, 7, 15, 254, 255][case / 4 % 8];
    ram[0x2e] = (case % 9) as u8;
    ram[0x5e] = if mode == 12 { 6 } else { 0 };
    ram[0x34a] = u8::from(matches!(mode, 11 | 13));
    ram[0x340] = [0, 1, 2, 4, 8, 12, 15, 255][case / 8 % 8];
    ram[0x372] = u8::from(mode == 1 || mode == 15);
    ram[0x374] = case as u8;
    ram[0xf0] = [0, 1, 15, 0x80][case / 4 % 4];
    ram[0x11] = match mode {
        4 => 14,
        5 => 18,
        6 => 19,
        _ => 15,
    };
    ram[0x308] = if mode == 7 { 0x80 } else { 0 };
    ram[0x48] = if mode == 8 { case as u8 } else { 0 };
    ram[0x351] = u8::from(mode == 9 || (mode == 1 && case & 32 != 0));
    ram[0x3c] = if mode == 10 || mode == 1 {
        (case / 8) as u8
    } else {
        0
    };
    ram[0x2f5] = [0, 1, 2, 255][case / 16 % 4];
    ram[0x377] = (case & 1) as u8;
    ram[0x30] = 0x95;
    ram[0x31] = 0x63;
    ram[0x1f5..0x1fd].copy_from_slice(&[0x1b, 0x80, 0, 0x4a, 0x89, 2, 0xae, 0x87]);
}
#[cfg(test)]
mod tests {
    use super::*;
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, RomCpuTimingProbe, Snes,
        SourcePpuReadState,
    };
    #[test]
    fn ground_animation_return_matches_source_animation_and_cleanup() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for address in (0x07e671..0x07e69d).chain(0x07e842..0x07e888) {
            assert_eq!(
                super::super::player_animation::table_byte(address),
                Some(rom[7 * 0x8000 + (address as usize & 0x7fff)])
            );
        }
        assert_eq!(super::super::player_animation::table_byte(0x07e69d), None);
        let mut instructions = 0;

        for timer in 0..=255u8 {
            for variant in 0..32u8 {
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                let r = &mut machine.ram;
                for (i, byte) in r.iter_mut().enumerate() {
                    *byte = (i ^ (i >> 7) ^ timer as usize) as u8;
                }
                seed_case(r, timer as usize, variant as usize);
                let pc = 0x82c5;
                let mut cpu = NativeNmiContext {
                    a: 0x2400,
                    x: 16,
                    y: variant as u16,
                    d: 0,
                    db: 7,
                    bank: 7,
                    pc,
                    sp: 0x1f4,
                    p: 0x30 | (timer & 0xc3),
                };
                machine.cpu.a = cpu.a;
                machine.cpu.x = cpu.x;
                machine.cpu.y = cpu.y;
                machine.cpu.db = cpu.db;
                machine.cpu.k = cpu.bank;
                machine.cpu.pc = cpu.pc;
                machine.cpu.sp = cpu.sp;
                machine.cpu.e = false;
                machine.cpu.unpack_flags(cpu.p);
                let mut ram = machine.ram.clone();
                let mut clock = CpuMasterTimeline::at_raster(
                    4586,
                    CpuRasterPosition::new(
                        if variant & 1 == 0 { 22 } else { 261 },
                        if variant & 1 == 0 { 26 } else { 1350 },
                    ),
                    CpuBusWorkload::default(),
                    CpuFieldTiming::non_interlace(variant & 1 != 0),
                );
                let mut source = RomCpuTimingProbe::new(
                    machine,
                    clock.clone(),
                    SourcePpuReadState::snes9x_reset(),
                )
                .unwrap();
                clock.begin_synchronous_timeline().unwrap();
                let mut work = GroundAnimationReturn::default();
                while !work.complete() {
                    let label = format!(
                        "velocity controls={timer} variant={variant} pc={:04x}",
                        cpu.pc
                    );
                    super::super::native_work::compare_instruction_with_devices(
                        &label,
                        &mut cpu,
                        &mut clock,
                        &mut source,
                        &mut ram,
                        &rom,
                        None,
                        |_, access| {
                            if let super::super::intro_clear::Access::Read(a, width) = access {
                                let byte =
                                    |address| super::super::player_animation::table_byte(address);
                                let low = byte(a)?;
                                Some(if width == 1 {
                                    u16::from(low)
                                } else {
                                    u16::from_le_bytes([low, byte(a + 1)?])
                                })
                            } else {
                                None
                            }
                        },
                        |cpu, clock, bus| {
                            work.step(cpu, clock, bus).unwrap();
                        },
                    );
                    work = work.clone();
                    instructions += 1;
                }
                assert_eq!(ram, source.snes().ram);
                assert!(work.camera_tail().is_some());
                if work.ancilla_tail().is_none() {assert_eq!((cpu.bank,cpu.db),(7,7));}
                assert_eq!(ram[0x302], 0);
                if timer & 1 != 0 {
                    assert_eq!(&ram[0x30..0x32], &[0, 0]);
                } else {
                    assert_eq!(&ram[0x30..0x32], &[0x95, 0x63]);
                }
            }
        }
        println!("ground animation return: 8192 cases, {instructions} source instructions, full animation, legal immutable tables, cleanup, byte neighbors and camera call");
    }
}
