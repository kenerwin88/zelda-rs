//! Live dungeon caller after the push-block handler, through camera return.
use super::{
    dungeon_camera::DungeonCamera,
    half_slot::HalfSlot,
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DungeonAfterPushTail {
    /// The complete dungeon and main caller retired their real returns.
    MainWait,
    /// The submodule branch retired; room maintenance is skipped.
    Lamp,
    AfterLamp,
    BeforeDrawing(super::dungeon_sprite_scroll::SavedSpriteScroll),
    AfterDrawing(super::dungeon_sprite_scroll::SavedSpriteScroll),
    SpritePrefix {
        saved: super::dungeon_sprite_scroll::SavedSpriteScroll,
        tail: super::dungeon_sprite_prefix::SpritePrefixTail,
    },
    /// The request check retired; conversion starts at $00:E3FF.
    HalfSlotBody,
    /// The camera retired its real RTS; the caller resumes at $02:87C1.
    AfterCamera,
    RoomTags(super::dungeon_room_tags::RoomTagTail),
    Interactions(super::dungeon_interaction_caller::InteractionTail),
    Edge(super::dungeon_edge_caller::EdgeTail),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonAfterPush {
    pc: u16,
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
    sprite_prefix: Option<super::dungeon_sprite_prefix::DungeonSpritePrefix>,
    drawing: Option<super::dungeon_push_drawing::DungeonPushDrawing>,
    sprite_scroll: Option<super::dungeon_sprite_scroll::DungeonSpriteScroll>,
    lamp: Option<super::dungeon_lamp_caller::DungeonLampCaller>,
    edge: Option<super::dungeon_edge_caller::DungeonEdgeCaller>,
    interactions: Option<super::dungeon_interaction_caller::DungeonInteractionCaller>,
    room_tags: Option<super::dungeon_room_tags::DungeonRoomTags>,
    half_slot: Option<HalfSlot>,
    camera: Option<DungeonCamera>,
    tail: Option<DungeonAfterPushTail>,
}

impl Default for DungeonAfterPush {
    fn default() -> Self {
        Self {
            pc: 0x87b6,
            native_room_tags: false,
            native_interactions: false,
            native_edge: false,
            native_lamp: false,
            native_sprite_scroll: false,
            native_drawing: false,
            native_sprite_prefix: false,
            native_garnish: false,
            native_follower: false,
            native_ancilla: false,
            inactive_slot_bodies: 0,
            live_inactive_slot_bodies: false,
            sprite_prefix: None,
            drawing: None,
            sprite_scroll: None,
            lamp: None,
            edge: None,
            interactions: None,
            room_tags: None,
            half_slot: None,
            camera: None,
            tail: None,
        }
    }
}

impl DungeonAfterPush {
    pub(crate) fn with_live_inactive_slot_bodies(mut self, enabled:bool)->Self {self.live_inactive_slot_bodies=enabled;self}
    pub(crate) fn with_inactive_slot_bodies(mut self, mask:u16)->Self {self.inactive_slot_bodies=mask;self}
    pub(crate) fn with_native_ancilla(mut self,enabled:bool)->Self {self.native_ancilla=enabled;self}
    pub(crate) fn with_native_follower(mut self,enabled:bool)->Self {self.native_follower=enabled;self}
    pub(crate) fn with_native_garnish(mut self, enabled: bool) -> Self {
        self.native_garnish = enabled;
        self
    }
    pub(crate) fn with_native_sprite_prefix(mut self, enabled: bool) -> Self {
        self.native_sprite_prefix = enabled;
        self
    }
    pub(crate) fn sprite_prefix_active(&self) -> bool {
        self.sprite_prefix.is_some()
    }
    pub(crate) fn with_native_drawing(mut self, enabled: bool) -> Self {
        self.native_drawing = enabled;
        self
    }
    pub(crate) fn drawing_active(&self) -> bool {
        self.drawing.is_some()
    }
    pub(crate) fn with_native_sprite_scroll(mut self, enabled: bool) -> Self {
        self.native_sprite_scroll = enabled;
        self
    }
    pub(crate) fn with_native_lamp(mut self, enabled: bool) -> Self {
        self.native_lamp = enabled;
        self
    }
    pub(crate) fn begin_native_lamp(&mut self) {
        if self.native_lamp {
            self.tail = None;
            self.lamp = Some(Default::default());
        }
    }
    pub(crate) fn with_native_edge(mut self, enabled: bool) -> Self {
        self.native_edge = enabled;
        self
    }
    pub(crate) fn with_native_interactions(mut self, enabled: bool) -> Self {
        self.native_interactions = enabled;
        self
    }

    pub(crate) fn with_native_room_tags(mut self, enabled: bool) -> Self {
        self.native_room_tags = enabled;
        self
    }

    pub(crate) fn tail(&self) -> Option<DungeonAfterPushTail> {
        self.tail
    }
    pub(crate) fn routine(&self) -> u32 {
        if let Some(prefix) = &self.sprite_prefix {
            prefix.routine()
        } else if let Some(drawing) = &self.drawing {
            drawing.routine()
        } else if self.sprite_scroll.is_some() {
            0x0287e5
        } else if let Some(work) = &self.lamp {
            work.routine()
        } else if let Some(work) = &self.edge {
            work.routine()
        } else if let Some(work) = &self.interactions {
            work.routine()
        } else if self.room_tags.is_some() {
            0x01c2fd
        } else if self.camera.is_some() {
            0x02ba31
        } else if self.half_slot.is_some() {
            0x00e3fa
        } else {
            0x0287b6
        }
    }

    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.tail.is_some() {
            return None;
        }
        if let Some(work) = &mut self.sprite_prefix {
            let cycles = work.step(cpu, clock, bus);
            if let Some(tail) = work.tail() {
                self.tail = Some(DungeonAfterPushTail::SpritePrefix {
                    saved: self.sprite_scroll.as_ref().unwrap().saved().unwrap(),
                    tail,
                });
            }
            return cycles;
        }
        if let Some(work) = &mut self.drawing {
            let cycles = work.step(cpu, clock, bus);
            if work.complete() {
                if self.native_sprite_prefix {
                    self.sprite_prefix = Some(
                        super::dungeon_sprite_prefix::DungeonSpritePrefix::default()
                            .with_native_garnish(self.native_garnish).with_native_follower(self.native_follower).with_native_ancilla(self.native_ancilla).with_inactive_slot_bodies(self.inactive_slot_bodies).with_live_inactive_slot_bodies(self.live_inactive_slot_bodies),
                    );
                } else {
                    self.tail = Some(DungeonAfterPushTail::AfterDrawing(
                        self.sprite_scroll.as_ref().unwrap().saved().unwrap(),
                    ));
                }
            }
            return cycles;
        }
        if let Some(work) = &mut self.sprite_scroll {
            let cycles = work.step(cpu, clock, bus);
            if work.saved().is_some() && self.native_drawing {
                self.drawing = Some(Default::default());
            } else {
                self.tail = work.saved().map(DungeonAfterPushTail::BeforeDrawing);
            }
            return cycles;
        }
        if let Some(work) = &mut self.lamp {
            let cycles = work.step(cpu, clock, bus);
            if work.complete() {
                if self.native_sprite_scroll {
                    self.sprite_scroll = Some(Default::default());
                } else {
                    self.tail = Some(DungeonAfterPushTail::AfterLamp);
                }
            }
            return cycles;
        }
        if let Some(work) = &mut self.edge {
            let cycles = work.step(cpu, clock, bus);
            self.tail = work.tail().map(DungeonAfterPushTail::Edge);
            if work.tail() == Some(super::dungeon_edge_caller::EdgeTail::Lamp) {
                self.begin_native_lamp();
            }
            return cycles;
        }
        if let Some(work) = &mut self.interactions {
            let cycles = work.step(cpu, clock, bus);
            if self.native_edge
                && work.tail() == Some(super::dungeon_interaction_caller::InteractionTail::Returned)
            {
                self.edge = Some(Default::default());
            } else {
                self.tail = work.tail().map(DungeonAfterPushTail::Interactions);
                if work.tail() == Some(super::dungeon_interaction_caller::InteractionTail::Lamp) {
                    self.begin_native_lamp();
                }
            }
            return cycles;
        }
        if let Some(room_tags) = &mut self.room_tags {
            let cycles = room_tags.step(cpu, clock, bus);
            if self.native_interactions
                && room_tags.tail() == Some(super::dungeon_room_tags::RoomTagTail::Returned)
            {
                self.interactions = Some(Default::default());
            } else {
                self.tail = room_tags.tail().map(DungeonAfterPushTail::RoomTags);
                if room_tags.tail() == Some(super::dungeon_room_tags::RoomTagTail::Lamp) {
                    self.begin_native_lamp();
                }
            }
            return cycles;
        }
        if let Some(camera) = &mut self.camera {
            let cycles = camera.step(cpu, clock, bus);
            if camera.complete() {
                assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x87c1, 0x1fc));
                if self.native_room_tags {
                    self.room_tags = Some(Default::default());
                } else {
                    self.tail = Some(DungeonAfterPushTail::AfterCamera);
                }
            }
            return cycles;
        }
        if let Some(half_slot) = &mut self.half_slot {
            let cycles = half_slot.step(cpu, clock, bus);
            if half_slot.body_pending() {
                self.tail = Some(DungeonAfterPushTail::HalfSlotBody);
            } else if half_slot.complete() {
                assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x87be, 0x1fc));
                self.half_slot = None;
            }
            return cycles;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d, cpu.db, cpu.p & 0x38),
            (2, self.pc, 0, 0, 0x30)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0x87b6 => {
                let value = w.dp_read(0x11, 1);
                w.load(value, 1);
                0x87b8
            }
            0x87b8 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    self.tail = Some(DungeonAfterPushTail::Lamp);
                    0x87e1
                } else {
                    0x87ba
                }
            }
            0x87ba => {
                w.operand_then_internal(3);
                w.push(2, 1);
                w.push(0x87bd, 2);
                w.cpu.bank = 0;
                w.cpu.pc = 0xe3fa;
                self.half_slot = Some(Default::default());
                0x87be
            }
            0x87be => {
                w.operand_then_internal(2);
                w.push(0x87c0, 2);
                w.cpu.pc = 0xba31;
                self.camera = Some(Default::default());
                0x87c1
            }
            _ => unreachable!("dungeon post-push caller {:04x}", self.pc),
        };
        if self.half_slot.is_none() && self.camera.is_none() {
            w.cpu.pc = self.pc;
        }
        let cycles = w.cycles;
        if self.tail == Some(DungeonAfterPushTail::Lamp) {
            self.begin_native_lamp();
        }
        Some(cycles)
    }
}

#[cfg(test)]
pub(crate) fn seed_case(ram: &mut [u8], case: usize) {
    let velocities = [0, 1, 3, 127, 128, 253, 255, 0];
    super::dungeon_camera::seed_case(
        ram,
        velocities[case % 8],
        velocities[case / 8 % 8],
        case / 64 % 24,
    );
    ram[0x11] = if case % 4 == 0 { 15 } else { 0 };
    ram[0xaaa] = if case % 4 == 1 {
        (case / 4 % 255 + 1) as u8
    } else {
        0
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dungeon_after_push_matches_source_half_slot_camera_and_real_returns() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        let mut tails = [0; 3];
        for case in 0..8192usize {
            let mut machine = snes::Snes::new();
            machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
            for (i, b) in machine.ram.iter_mut().enumerate() {
                *b = (i as u8).wrapping_mul(13).wrapping_add(case as u8);
            }
            seed_case(&mut machine.ram, case);
            let mut cpu = NativeNmiContext {
                a: 0x1111,
                x: 255,
                y: 6,
                d: 0,
                db: 0,
                bank: 2,
                pc: 0x87b6,
                sp: 0x1fc,
                p: 0x30 | (case as u8 & 0xc3),
            };
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut ram = machine.ram.clone();
            let timeline = CpuMasterTimeline::at_raster(
                4586,
                snes::CpuRasterPosition::new(47, 894),
                snes::CpuBusWorkload::default(),
                snes::CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source = snes::RomCpuTimingProbe::new(
                machine,
                timeline,
                snes::SourcePpuReadState::snes9x_reset(),
            )
            .unwrap();
            let mut clock = source.timeline().clone();
            let mut work = DungeonAfterPush::default();
            while work.tail().is_none() {
                let pc = cpu.pc;
                super::super::native_work::compare_instruction(
                    &format!("dungeon after push case={case} pc={pc:04x}"),
                    &mut cpu,
                    &mut clock,
                    &mut source,
                    &mut ram,
                    &rom,
                    |cpu, clock, bus| {
                        work.step(cpu, clock, bus).unwrap();
                    },
                );
                work = work.clone();
                count += 1;
            }
            match work.tail().unwrap() {
                DungeonAfterPushTail::Lamp => {
                    tails[0] += 1;
                    assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x87e1, 0x1fc));
                }
                DungeonAfterPushTail::HalfSlotBody => {
                    tails[1] += 1;
                    assert_eq!((cpu.bank, cpu.pc, cpu.sp), (0, 0xe3ff, 0x1f9));
                }
                DungeonAfterPushTail::MainWait
                | DungeonAfterPushTail::SpritePrefix { .. }
                | DungeonAfterPushTail::AfterDrawing(_)
                | DungeonAfterPushTail::BeforeDrawing(_)
                | DungeonAfterPushTail::AfterLamp
                | DungeonAfterPushTail::RoomTags(_)
                | DungeonAfterPushTail::Interactions(_)
                | DungeonAfterPushTail::Edge(_) => {
                    unreachable!()
                }
                DungeonAfterPushTail::AfterCamera => {
                    tails[2] += 1;
                    assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x87c1, 0x1fc));
                }
            }
            assert_eq!(ram, source.snes().ram);
        }
        assert_eq!(tails, [2048, 2048, 4096]);
        println!(
            "native dungeon after push: 8192 cases, {count} source instructions, tails={tails:?}"
        );
    }
}
