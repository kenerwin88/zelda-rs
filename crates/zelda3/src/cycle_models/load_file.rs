//! Complete load-game caller with retained native children and real returns.
//! The common prefix ends at the destination decision without popping the
//! module's far frame. Children share the live CPU, DMA and typed device bus.
use super::animated_dungeon_graphics::AnimatedDungeonGraphics;
use super::dungeon_attributes::CustomAttributes;
use super::dungeon_attributes::DungeonAttributes;
use super::dungeon_entrance::DungeonEntrance;
use super::dungeon_palettes::DungeonPalettes;
use super::dungeon_upload::DungeonLoadAndDraw;
use super::dungeon_visited::VisitedFlags;
use super::load_file_common::LoadFileCommon;
use super::load_file_hud::LoadFileHud;
use super::load_file_player::LoadFilePlayer;
use super::native_tilesets::NativeTilesets;
use super::song_bank::SongBank;
use super::sprite_reset::SpriteReset;
use super::{
    intro_palettes::PaletteSource,
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::{CpuMasterTimeline, DmaState};
#[derive(Clone, Debug, PartialEq, Eq)]
enum Child {
    Common(LoadFileCommon),
    Hud(LoadFileHud),
    Entrance(DungeonEntrance),
    Room(DungeonLoadAndDraw),
    Custom(CustomAttributes),
    Animated(AnimatedDungeonGraphics),
    Attributes(DungeonAttributes),
    Tilesets(NativeTilesets),
    Palettes(DungeonPalettes),
    Visited(VisitedFlags),
    Player(LoadFilePlayer),
    Sprite(SpriteReset),
    Song(SongBank),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LoadFile {
    pc: u32,
    child: Option<Child>,
    message_return: Option<u32>,
    complete: bool,
}
impl Default for LoadFile {
    fn default() -> Self {
        Self {
            pc: 0x028192,
            child: Some(Child::Common(Default::default())),
            message_return: None,
            complete: false,
        }
    }
}
impl LoadFile {
    pub(crate) fn opening_player_active(&self) -> bool {
        matches!(&self.child, Some(Child::Player(work)) if work.opening_active())
    }

    pub(crate) fn sprite_records_active(&self) -> bool {
        matches!(&self.child, Some(Child::Sprite(work)) if work.records_active())
    }

    pub(crate) fn sprite_reset_parent_active(&self) -> bool {
        matches!(&self.child, Some(Child::Sprite(work)) if work.reset_parent_active())
    }

    pub(crate) fn sprite_cache_active(&self) -> bool {
        matches!(&self.child, Some(Child::Sprite(work)) if work.cache_active())
    }

    pub(crate) fn sprite_reset_suffix_active(&self) -> bool {
        matches!(&self.child, Some(Child::Sprite(work)) if work.reset_suffix_active())
    }

    pub(crate) fn sprite_disable_active(&self) -> bool {
        matches!(&self.child, Some(Child::Sprite(work)) if work.disable_active())
    }

    pub(crate) fn follower_initialization_active(&self) -> bool {
        matches!(&self.child, Some(Child::Player(work)) if work.follower_initialization_active())
    }

    pub(crate) fn room_entry_cache_active(&self) -> bool {
        matches!(&self.child, Some(Child::Player(work)) if work.room_entry_cache_active())
    }

    pub(crate) fn interactive_cleanup_active(&self) -> bool {
        matches!(&self.child, Some(Child::Player(work)) if work.interactive_cleanup_active())
    }

    pub(crate) fn visited_active(&self) -> bool {
        matches!(self.child, Some(Child::Visited(_)))
    }

    pub(crate) fn attributes_active(&self) -> bool {
        matches!(self.child, Some(Child::Attributes(_) | Child::Custom(_)))
    }

    pub(crate) fn room_upload_active(&self) -> bool {
        matches!(&self.child, Some(Child::Room(work)) if work.upload_active())
    }

    pub(crate) fn object_tiles_active(&self) -> bool {
        matches!(&self.child, Some(Child::Room(work)) if work.object_tiles_active())
    }

    pub(crate) fn saved_objects_active(&self) -> bool {
        matches!(&self.child, Some(Child::Room(work)) if work.saved_objects_active())
    }

    pub(crate) fn door_state_active(&self) -> bool {
        matches!(&self.child, Some(Child::Room(work)) if work.door_state_active())
    }

    pub(crate) fn object_parser_active(&self) -> bool {
        matches!(&self.child, Some(Child::Room(work)) if work.object_parser_active())
    }

    pub(crate) fn floors_active(&self) -> bool {
        matches!(&self.child, Some(Child::Room(work)) if work.floors_active())
    }

    pub(crate) fn room_parent_active(&self) -> bool {
        matches!(&self.child, Some(Child::Room(work)) if work.room_parent_active())
    }

    pub(crate) fn header_active(&self) -> bool {
        matches!(&self.child, Some(Child::Room(work)) if work.header_active())
    }

    pub(crate) fn parent_active(&self) -> bool {
        self.child.is_none()
    }

    pub(crate) fn entrance_active(&self) -> bool {
        matches!(self.child, Some(Child::Entrance(_)))
    }

    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        match &self.child {
            Some(Child::Common(w)) => w.routine(),
            Some(Child::Hud(w)) => w.routine(),
            Some(Child::Entrance(w)) => w.routine(),
            Some(Child::Room(w)) => w.routine(),
            Some(Child::Custom(w)) => w.routine(),
            Some(Child::Animated(w)) => w.routine(),
            Some(Child::Attributes(w)) => w.routine(),
            Some(Child::Tilesets(w)) => w.routine(),
            Some(Child::Palettes(w)) => w.routine(),
            Some(Child::Visited(w)) => w.routine(),
            Some(Child::Player(w)) => w.routine(),
            Some(Child::Sprite(w)) => w.routine(),
            Some(Child::Song(w)) => w.routine(),
            None => {
                if self.message_return.is_some() {
                    0x0ffdaa
                } else {
                    0x028136
                }
            }
        }
    }
    pub(crate) fn palette_source(&self, cpu: &NativeNmiContext) -> PaletteSource {
        match &self.child {
            Some(Child::Palettes(w)) => w.palette_source(cpu),
            _ => PaletteSource::Constant,
        }
    }
    pub(crate) fn take_dma_end(&mut self) -> Option<u64> {
        match &mut self.child {
            Some(Child::Common(w)) => w.take_dma_end(),
            Some(Child::Room(w)) => w.take_dma_end(),
            _ => None,
        }
    }
    fn child(target: u32) -> Child {
        match target {
            0x2d8b3 => Child::Entrance(Default::default()),
            0x2c57b => Child::Room(Default::default()),
            0xe942a => Child::Custom(Default::default()),
            0xd337 => Child::Animated(Default::default()),
            0x1b8bf => Child::Attributes(Default::default()),
            0xe19b => Child::Tilesets(Default::default()),
            0x2b8cb => Child::Visited(Default::default()),
            0x0de395 | 0x0dfa6c | 0x0ddd32 | 0x0dfa68 => {
                Child::Hud(LoadFileHud::new(target as u16))
            }
            0x02c630 | 0x0ed6dd => Child::Palettes(DungeonPalettes::from_entry(target)),
            0x01fec1 | 0x028aef | 0x0ffd22 | 0x028d71 | 0x0afd90 | 0x099efc | 0x079a2c => {
                Child::Player(LoadFilePlayer::from_entry(target))
            }
            0x09c44e | 0x09c114 => Child::Sprite(SpriteReset::from_entry(target)),
            0x029bd7 => Child::Song(SongBank::from_entry(target)),
            _ => unreachable!("load-game child {target:06x}"),
        }
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        dma: &mut DmaState,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.complete {
            return None;
        }
        if let Some(child) = &mut self.child {
            let (cycles, complete) = match child {
                Child::Common(w) => {
                    let c = w.step(cpu, clock, dma, bus);
                    (c, w.complete())
                }
                Child::Hud(w) => {
                    let c = w.step(cpu, clock, bus);
                    (c, w.complete())
                }
                Child::Entrance(w) => {
                    let c = w.step(cpu, clock, bus);
                    (c, w.complete())
                }
                Child::Room(w) => {
                    let c = w.step(cpu, clock, dma, bus);
                    (c, w.complete())
                }
                Child::Custom(w) => {
                    let c = w.step(cpu, clock, bus);
                    (c, w.complete())
                }
                Child::Animated(w) => {
                    let c = w.step(cpu, clock, bus);
                    (c, w.complete())
                }
                Child::Attributes(w) => {
                    let c = w.step(cpu, clock, bus);
                    (c, w.complete())
                }
                Child::Tilesets(w) => {
                    let c = w.step(cpu, clock, bus);
                    (c, w.complete())
                }
                Child::Palettes(w) => {
                    let c = w.step(cpu, clock, bus);
                    (c, w.complete())
                }
                Child::Visited(w) => {
                    let c = w.step(cpu, clock, bus);
                    (c, w.complete())
                }
                Child::Player(w) => {
                    let c = w.step(cpu, clock, bus);
                    (c, w.complete())
                }
                Child::Sprite(w) => {
                    let c = w.step(cpu, clock, bus);
                    (c, w.complete())
                }
                Child::Song(w) => {
                    let c = w.step(cpu, clock, bus);
                    (c, w.complete())
                }
            };
            if complete {
                assert_eq!((u32::from(cpu.bank) << 16) | u32::from(cpu.pc), self.pc);
                self.child = None;
            }
            return cycles;
        }
        assert_eq!(
            (
                (u32::from(cpu.bank) << 16) | u32::from(cpu.pc),
                cpu.d,
                cpu.p & 8
            ),
            (self.pc, 0, 0)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let m = if w.cpu.p & 0x20 != 0 { 1 } else { 2 };
        let x = if w.cpu.p & 0x10 != 0 { 1 } else { 2 };
        let mask = if m == 1 { 255 } else { 65535 };
        let index_mask = if x == 1 { 255 } else { 65535 };
        w.fetch();
        self.pc = match self.pc {
            0x28192 | 0x281bd | 0x281cd | 0x281d5 | 0x281e2 | 0x282bc | 0x282dd | 0x2832b
            | 0x28356 | 0x2835c | 0x2838c => {
                w.drain(24);
                let a = match self.pc {
                    0x28192 => 0x7ef3ca,
                    0x281bd => 0x7ec011,
                    0x281cd | 0x28356 | 0x2838c => 0x7ef3c5,
                    0x281d5 => 0x7ef3c8,
                    0x281e2 => 0x7ef353,
                    0x282bc | 0x2835c => 0x7ef3c6,
                    0x282dd => 0x7ec005,
                    0x2832b => 0x7ef3cc,
                    _ => unreachable!(),
                };
                let v = w.read(a, m);
                w.load(v, m);
                self.pc + 4
            }
            0x28196 | 0x281c6 | 0x281cb | 0x281db | 0x281e8 | 0x28252 | 0x28297 | 0x282c3
            | 0x282e8 | 0x282ee | 0x282f4 | 0x283a4 | 0x283a9 | 0xffdae => {
                if w.branch(w.cpu.p & ZERO != 0) {
                    match self.pc {
                        0x28196 => 0x281bd,
                        0x281c6 => 0x281cd,
                        0x281cb | 0x281db => 0x28208,
                        0x281e8 => 0x281ed,
                        0x28252 => 0x28260,
                        0x28297 => 0x2829d,
                        0x282c3 => 0x282cc,
                        0x282e8 => 0x282f6,
                        0x282ee | 0x282f4 => 0x282f8,
                        0x283a4 | 0x283a9 => 0x283b2,
                        0xffdae => 0xffdc3,
                        _ => unreachable!(),
                    }
                } else {
                    self.pc + 2
                }
            }
            0x28198 | 0x2829f | 0x282a9 | 0x282b5 | 0x28399 | 0x2839f | 0xffdaa | 0xffdba => {
                let v = w.dp_read(
                    match self.pc {
                        0x28198 => 0x1b,
                        0x2829f | 0x282a9 | 0x282b5 | 0x2839f => 0xa0,
                        0x28399 => 0xa4,
                        0xffdaa | 0xffdba => 0x10,
                        _ => unreachable!(),
                    },
                    m,
                );
                w.load(v, m);
                self.pc + 2
            }
            0x2819a | 0x281c1 | 0x28256 | 0x282ba | 0x282e1 | 0x28331 | 0x2835a | 0x28362 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    match self.pc {
                        0x2819a | 0x281c1 => 0x28208,
                        0x28256 => 0x2825a,
                        0x282ba => 0x282cc,
                        0x282e1 => 0x282fa,
                        0x28331 => 0x28340,
                        0x2835a | 0x28362 => 0x2837e,
                        _ => unreachable!(),
                    }
                } else {
                    self.pc + 2
                }
            }
            0x2819c | 0x281a0 | 0x281a4 | 0x281f2 | 0x28212 | 0x28216 | 0x2821a | 0x28260
            | 0x2826d | 0x28279 | 0x2827d | 0x28286 | 0x28299 | 0x282ce | 0x28302 | 0x28324
            | 0x2833c | 0x28344 | 0x28348 | 0x2834c | 0x2837a => {
                let target = match self.pc {
                    0x2819c | 0x28212 => 0xde395,
                    0x281a0 | 0x28216 => 0xdfa6c,
                    0x281a4 | 0x2821a => 0xddd32,
                    0x281f2 => 0xffdaa,
                    0x28260 => 0xdfa68,
                    0x2826d => 0xe942a,
                    0x28279 => 0xd337,
                    0x2827d => 0x1b8bf,
                    0x28286 => 0xe19b,
                    0x28299 => 0xed6dd,
                    0x282ce => 0x2b8cb,
                    0x28302 => 0x1fec1,
                    0x28324 => 0xffd22,
                    0x2833c => 0xafd90,
                    0x28344 => 0x99efc,
                    0x28348 => 0x9c44e,
                    0x2834c => 0x9c114,
                    0x2837a => 0x79a2c,
                    _ => unreachable!(),
                };
                w.operand_then_internal(3);
                w.push(u16::from(w.cpu.bank), 1);
                w.push(w.cpu.pc + 3, 2);
                w.cpu.bank = (target >> 16) as u8;
                w.cpu.pc = target as u16;
                if target == 0x0ffdaa {
                    assert!(self.message_return.is_none());
                    self.message_return = Some(self.pc + 4);
                    target
                } else {
                    self.child = Some(Self::child(target));
                    self.pc + 4
                }
            }
            0x281a8 | 0x281b9 | 0x2822a | 0x28264 | 0x28267 | 0x28318 | 0x28339 | 0x28350
            | 0xffdb0 | 0xffdb3 => {
                w.drain(16);
                let a = w.absolute(match self.pc {
                    0x281a8 => 0x10a,
                    0x281b9 => 0x4aa,
                    0x2822a => 0x402,
                    0x28264 => 0x45a,
                    0x28267 => 0x458,
                    0x28318 => 0xaa9,
                    0x28339 => 0x4b4,
                    0x28350 => 0x2f0,
                    0xffdb0 => 0x223,
                    0xffdb3 => 0x1cd8,
                    _ => unreachable!(),
                });
                w.write(a, 0, m);
                self.pc + 3
            }
            0x281ab | 0x281b1 | 0x281f9 | 0x281fd | 0x28203 | 0x28208 | 0x28220 | 0x2822d
            | 0x28258 | 0x28281 | 0x2828a | 0x282c5 | 0x282d2 | 0x282d6 | 0x282ea | 0x282f0
            | 0x282f6 | 0x28306 | 0x2830c | 0x28312 | 0x28333 | 0x28340 | 0x28364 | 0x28368
            | 0x2836c | 0x28370 | 0x2837e | 0x28385 | 0x28394 | 0x283ad | 0xffdb6 | 0xffdbf => {
                w.drain(8 * u32::from(m));
                let v: u16 = match self.pc {
                    0x281ab | 0x282f6 => 0x20,
                    0x281b1 => 0x8,
                    0x281f9 | 0x28385 => 0xf,
                    0x281fd => 0x4,
                    0x28203 => 0x1b,
                    0x28208 | 0x2822d | 0x28258 | 0x282c5 | 0x2830c | 0x28333 | 0x28370 => 0x0,
                    0x28220 | 0x28394 => 0x5,
                    0x28281 | 0x2828a => 0xa,
                    0x282d2 | 0x28312 | 0xffdb6 => 0x2,
                    0x282d6 => 0xb3,
                    0x282ea => 0x32,
                    0x282f0 => 0x62,
                    0x28306 => 0x1f,
                    0x28340 => 0x9,
                    0x28364 => 0x30,
                    0x28368 => 0x50,
                    0x2836c => 0x80,
                    0x2837e => 0x7,
                    0x283ad => 0x3,
                    0xffdbf => 0xe,
                    _ => unreachable!(),
                };
                w.load(v, m);
                self.pc + 1 + u32::from(m)
            }
            0x281ad | 0x281b3 | 0x281fb | 0x281ff | 0x28205 | 0x28210 | 0x282d4 | 0x282d8
            | 0x282f8 | 0x28342 | 0x28366 | 0x2836a | 0x2836e | 0x28383 | 0x28387 | 0xffdb8
            | 0xffdc1 => {
                w.dp_store(
                    match self.pc {
                        0x281ad => 0xa0,
                        0x281b3 | 0x28205 | 0x28383 | 0xffdc1 => 0x10,
                        0x281fb => 0x13,
                        0x281ff => 0x1c,
                        0x28210 => 0x95,
                        0x282d4 => 0x99,
                        0x282d8 | 0x282f8 => 0x9a,
                        0x28342 => 0x94,
                        0x28366 => 0x9c,
                        0x2836a => 0x9d,
                        0x2836e => 0x9e,
                        0x28387 | 0xffdb8 => 0x11,
                        _ => unreachable!(),
                    },
                    w.cpu.a & mask,
                    m,
                );
                self.pc + 2
            }
            0x281af | 0x281b5 | 0x281b7 | 0x28201 | 0x28226 | 0x28228 | 0x2831b | 0x2831d
            | 0x2831f => {
                w.dp_store(
                    match self.pc {
                        0x281af => 0xa1,
                        0x281b5 => 0x11,
                        0x281b7 => 0xb0,
                        0x28201 => 0x1d,
                        0x28226 => 0xa0,
                        0x28228 => 0xa2,
                        0x2831b => 0x57,
                        0x2831d => 0x3a,
                        0x2831f => 0x3c,
                        _ => unreachable!(),
                    },
                    0,
                    m,
                );
                self.pc + 2
            }
            0x281bc | 0x28207 | 0x283b4 | 0xffdc3 => {
                w.rtl();
                let returned = (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc);
                if let Some(expected) = self.message_return.take() {
                    assert_eq!(returned, expected);
                } else {
                    self.complete = true;
                }
                returned
            }
            0x281c3 | 0x281c8 | 0x2824d | 0x28292 => {
                w.drain(16);
                let a = w.absolute(match self.pc {
                    0x281c3 => 0x10a,
                    0x281c8 => 0x4aa,
                    0x2824d => 0x40c,
                    0x28292 => 0x2e0,
                    _ => unreachable!(),
                });
                let v = w.read(a, m);
                w.load(v, m);
                self.pc + 3
            }
            0x281d1 | 0x281d9 | 0x281e6 | 0x28250 | 0x28254 | 0x282b7 | 0x2832f | 0x28390
            | 0x283a1 | 0x283a6 | 0xffdac => {
                w.drain(8 * u32::from(m));
                let v: u16 = match self.pc {
                    0x281d1 | 0x281e6 | 0x28254 | 0x28390 | 0x283a1 => 0x2,
                    0x281d9 => 0x5,
                    0x28250 => 0xff,
                    0x282b7 => 0x104,
                    0x2832f => 0xd,
                    0x283a6 => 0x12,
                    0xffdac => 0xe,
                    _ => unreachable!(),
                };
                let a = w.cpu.a & mask;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(a >= v);
                w.flags(a.wrapping_sub(v) & mask, m);
                self.pc + 1 + u32::from(m)
            }
            0x281d3 => {
                if w.branch(w.cpu.p & CARRY == 0) {
                    0x28208
                } else {
                    self.pc + 2
                }
            }
            0x281dd | 0x2821e | 0x2829d | 0x2839d => {
                w.width(
                    match self.pc {
                        0x281dd => 0x10,
                        0x2821e | 0x2839d => 0x20,
                        0x2829d => 0x30,
                        _ => unreachable!(),
                    },
                    true,
                );
                self.pc + 2
            }
            0x281df | 0x281ea | 0x282e3 => {
                w.drain(8 * u32::from(x));
                let v: u16 = match self.pc {
                    0x281df => 0x185,
                    0x281ea => 0x184,
                    0x282e3 => 0x3,
                    _ => unreachable!(),
                };
                w.cpu.x = v;
                w.flags(v, x);
                self.pc + 1 + u32::from(x)
            }
            0x281ed => {
                w.drain(16);
                let a = w.absolute(0x1cf0);
                w.write(a, w.cpu.x & index_mask, x);
                self.pc + 3
            }
            0x281f0 | 0x28248 | 0x282cc | 0x283ab | 0x283b2 => {
                w.width(
                    match self.pc {
                        0x281f0 => 0x10,
                        0x28248 | 0x283ab | 0x283b2 => 0x20,
                        0x282cc => 0x30,
                        _ => unreachable!(),
                    },
                    false,
                );
                self.pc + 2
            }
            0x281f6 | 0x2824a | 0x2826a | 0x2828f | 0x28321 | 0x28328 | 0x28389 => {
                let target = match self.pc {
                    0x281f6 | 0x2828f => 0x2c630,
                    0x2824a => 0x2d8b3,
                    0x2826a => 0x2c57b,
                    0x28321 => 0x28aef,
                    0x28328 => 0x28d71,
                    0x28389 => 0x29bd7,
                    _ => unreachable!(),
                };
                w.operand_then_internal(2);
                w.push(w.cpu.pc + 2, 2);
                w.cpu.bank = (target >> 16) as u8;
                w.cpu.pc = target as u16;
                if target == 0x0ffdaa {
                    assert!(self.message_return.is_none());
                    self.message_return = Some(self.pc + 3);
                    target
                } else {
                    self.child = Some(Self::child(target));
                    self.pc + 3
                }
            }
            0x2820a | 0x28230 | 0x28234 | 0x28238 | 0x2823c | 0x28240 | 0x28244 | 0x282c8
            | 0x282fe | 0x28308 | 0x2830e | 0x28314 | 0x28335 | 0x28372 | 0x28376 => {
                w.drain(24);
                let a = match self.pc {
                    0x2820a => 0x7ec011,
                    0x28230 => 0x7ec019,
                    0x28234 => 0x7ec01b,
                    0x28238 => 0x7ec01d,
                    0x2823c => 0x7ec01f,
                    0x28240 => 0x7ec021,
                    0x28244 => 0x7ec023,
                    0x282c8 | 0x28372 => 0x7ec005,
                    0x282fe => 0x7ec017,
                    0x28308 => 0x7ec007,
                    0x2830e => 0x7ec00b,
                    0x28314 => 0x7ec009,
                    0x28335 => 0x7ef3cc,
                    0x28376 => 0x7ec006,
                    _ => unreachable!(),
                };
                w.write(a, w.cpu.a & mask, m);
                self.pc + 4
            }
            0x2820e => {
                w.drain(8 * u32::from(m));
                let v: u16 = 0x7;
                w.load((w.cpu.a | v) & mask, m);
                self.pc + 1 + u32::from(m)
            }
            0x28223 | 0x28283 | 0x2828c | 0x282a6 | 0x282b2 | 0x28380 | 0x28396 | 0x283af
            | 0xffdbc => {
                w.drain(16);
                let a = w.absolute(match self.pc {
                    0x28223 | 0x28396 | 0x283af => 0x12d,
                    0x28283 => 0xaa4,
                    0x2828c => 0xab1,
                    0x282a6 => 0x62c,
                    0x282b2 => 0x62e,
                    0x28380 | 0xffdbc => 0x10c,
                    _ => unreachable!(),
                });
                w.write(a, w.cpu.a & mask, m);
                self.pc + 3
            }
            0x2825a | 0x282ae | 0x282af | 0x282b0 => {
                w.drain(6);
                let a = w.cpu.a & mask;
                w.cpu.p = (w.cpu.p & !CARRY) | (a & 1) as u8;
                w.load(a >> 1, m);
                self.pc + 1
            }
            0x2825b => {
                w.drain(6);
                w.cpu.x = w.cpu.a & index_mask;
                w.flags(w.cpu.x, x);
                self.pc + 1
            }
            0x2825c | 0x28274 | 0x282fa => {
                w.drain(24);
                let a = (match self.pc {
                    0x2825c => 0x7ef37c,
                    0x28274 => 0x2811e,
                    0x282fa => 0x2a1e5,
                    _ => unreachable!(),
                }) + u32::from(w.cpu.x);
                let v = w.read(a, m);
                w.load(v, m);
                self.pc + 4
            }
            0x28271 | 0x282da => {
                w.drain(16);
                let a = w.absolute(match self.pc {
                    0x28271 => 0xaa1,
                    0x282da => 0x45a,
                    _ => unreachable!(),
                });
                let v = w.read(a, x);
                w.cpu.x = v;
                w.flags(v, x);
                self.pc + 3
            }
            0x28278 => {
                w.drain(6);
                w.cpu.y = w.cpu.a & index_mask;
                w.flags(w.cpu.y, x);
                self.pc + 1
            }
            0x28295 => {
                let v = w.dp_read(0x56, m);
                w.load((w.cpu.a | v) & mask, m);
                self.pc + 2
            }
            0x282a1 | 0x282ab | 0x282c0 | 0x28360 => {
                w.drain(8 * u32::from(m));
                let v: u16 = match self.pc {
                    0x282a1 => 0xf,
                    0x282ab => 0xff0,
                    0x282c0 | 0x28360 => 0x10,
                    _ => unreachable!(),
                };
                w.load((w.cpu.a & v) & mask, m);
                self.pc + 1 + u32::from(m)
            }
            0x282a4 => {
                w.shift_left(m);
                self.pc + 1
            }
            0x282a5 | 0x282b1 => {
                w.swap_bytes();
                self.pc + 1
            }
            0x282e5 => {
                w.drain(16);
                let a = w.absolute(0x414);
                let v = w.read(a, x);
                w.cpu.y = v;
                w.flags(v, x);
                self.pc + 3
            }
            0x282ec | 0x282f2 => {
                w.drain(8 * u32::from(x));
                let v: u16 = match self.pc {
                    0x282ec => 0x7,
                    0x282f2 => 0x4,
                    _ => unreachable!(),
                };
                let a = w.cpu.y & index_mask;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(a >= v);
                w.flags(a.wrapping_sub(v) & index_mask, x);
                self.pc + 1 + u32::from(x)
            }
            0x28353 => {
                w.drain(16);
                let a = w.absolute(0x4c7);
                let v = w.read(a, m).wrapping_add(1) & mask;
                w.drain(6);
                w.flags(v, m);
                w.write(a, v, m);
                self.pc + 3
            }
            0x28392 => {
                if w.branch(w.cpu.p & CARRY != 0) {
                    0x283b2
                } else {
                    self.pc + 2
                }
            }
            0x2839b => {
                if w.branch(w.cpu.p & NEGATIVE != 0) {
                    0x283b2
                } else {
                    self.pc + 2
                }
            }
            _ => unreachable!("load-game continuation {:06x}", self.pc),
        };
        if self.child.is_none() {
            w.cpu.pc = self.pc as u16;
            w.cpu.bank = (self.pc >> 16) as u8;
        }
        Some(w.cycles)
    }
}
#[cfg(test)]
#[path = "load_file_tests.rs"]
mod tests;
#[cfg(test)]
pub(super) const OWNED_POSITIONS: &[u32] = &[
    0x28192, 0x28196, 0x28198, 0x2819a, 0x2819c, 0x281a0, 0x281a4, 0x281a8, 0x281ab, 0x281ad,
    0x281af, 0x281b1, 0x281b3, 0x281b5, 0x281b7, 0x281b9, 0x281bc, 0x281bd, 0x281c1, 0x281c3,
    0x281c6, 0x281c8, 0x281cb, 0x281cd, 0x281d1, 0x281d3, 0x281d5, 0x281d9, 0x281db, 0x281dd,
    0x281df, 0x281e2, 0x281e6, 0x281e8, 0x281ea, 0x281ed, 0x281f0, 0x281f2, 0x281f6, 0x281f9,
    0x281fb, 0x281fd, 0x281ff, 0x28201, 0x28203, 0x28205, 0x28207, 0x28208, 0x2820a, 0x2820e,
    0x28210, 0x28212, 0x28216, 0x2821a, 0x2821e, 0x28220, 0x28223, 0x28226, 0x28228, 0x2822a,
    0x2822d, 0x28230, 0x28234, 0x28238, 0x2823c, 0x28240, 0x28244, 0x28248, 0x2824a, 0x2824d,
    0x28250, 0x28252, 0x28254, 0x28256, 0x28258, 0x2825a, 0x2825b, 0x2825c, 0x28260, 0x28264,
    0x28267, 0x2826a, 0x2826d, 0x28271, 0x28274, 0x28278, 0x28279, 0x2827d, 0x28281, 0x28283,
    0x28286, 0x2828a, 0x2828c, 0x2828f, 0x28292, 0x28295, 0x28297, 0x28299, 0x2829d, 0x2829f,
    0x282a1, 0x282a4, 0x282a5, 0x282a6, 0x282a9, 0x282ab, 0x282ae, 0x282af, 0x282b0, 0x282b1,
    0x282b2, 0x282b5, 0x282b7, 0x282ba, 0x282bc, 0x282c0, 0x282c3, 0x282c5, 0x282c8, 0x282cc,
    0x282ce, 0x282d2, 0x282d4, 0x282d6, 0x282d8, 0x282da, 0x282dd, 0x282e1, 0x282e3, 0x282e5,
    0x282e8, 0x282ea, 0x282ec, 0x282ee, 0x282f0, 0x282f2, 0x282f4, 0x282f6, 0x282f8, 0x282fa,
    0x282fe, 0x28302, 0x28306, 0x28308, 0x2830c, 0x2830e, 0x28312, 0x28314, 0x28318, 0x2831b,
    0x2831d, 0x2831f, 0x28321, 0x28324, 0x28328, 0x2832b, 0x2832f, 0x28331, 0x28333, 0x28335,
    0x28339, 0x2833c, 0x28340, 0x28342, 0x28344, 0x28348, 0x2834c, 0x28350, 0x28353, 0x28356,
    0x2835a, 0x2835c, 0x28360, 0x28362, 0x28364, 0x28366, 0x28368, 0x2836a, 0x2836c, 0x2836e,
    0x28370, 0x28372, 0x28376, 0x2837a, 0x2837e, 0x28380, 0x28383, 0x28385, 0x28387, 0x28389,
    0x2838c, 0x28390, 0x28392, 0x28394, 0x28396, 0x28399, 0x2839b, 0x2839d, 0x2839f, 0x283a1,
    0x283a4, 0x283a6, 0x283a9, 0x283ab, 0x283ad, 0x283af, 0x283b2, 0x283b4, 0xffdaa, 0xffdac,
    0xffdae, 0xffdb0, 0xffdb3, 0xffdb6, 0xffdb8, 0xffdba, 0xffdbc, 0xffdbf, 0xffdc1, 0xffdc3,
];
