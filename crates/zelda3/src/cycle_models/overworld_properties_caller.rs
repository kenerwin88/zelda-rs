//! Module08 properties caller and native callee composition on one live clock.
//! The original indirect near call and final RTL consume their actual frames.
//! Special-exit work exposes an explicit handoff until its native owner is
//! connected; this owner never decodes cartridge instructions.
use super::{
    animated_item_graphics::AnimatedItemGraphics,
    animated_overworld_graphics::AnimatedOverworldGraphics,
    dungeon_palettes::DungeonPalettes,
    hud_update::HudUpdate,
    load_file_player::LoadFilePlayer,
    mirror_portal::MirrorPortal,
    native_tilesets::NativeTilesets,
    native_work::{Work, WorkBus},
    nmi_stack::NativeNmiContext,
    overworld_area_palettes::OverworldAreaPalettes,
    overworld_dungeon_exit::OverworldDungeonExit,
    overworld_fixed_scroll::OverworldFixedScroll,
    overworld_screen_palettes::OverworldScreenPalettes,
    overworld_song_list::OverworldSongList,
    overworld_sprite_reload::OverworldSpriteReload,
    pre_overworld_properties::{BunnyStatus, PreOverworldProperties, PropertiesChild},
    song_bank::SongBank,
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Child {
    Bunny(BunnyStatus),
    Exit(OverworldDungeonExit),
    Songs(OverworldSongList),
    Hud(HudUpdate),
    Animated(AnimatedOverworldGraphics),
    Tilesets(NativeTilesets),
    Screen(OverworldScreenPalettes),
    Area(OverworldAreaPalettes),
    Fixed(OverworldFixedScroll),
    Player(LoadFilePlayer),
    PlayerGraphics(AnimatedItemGraphics),
    Reload(OverworldSpriteReload),
    Mirror(MirrorPortal),
    BunnyPalettes(DungeonPalettes),
    Bank(SongBank),
}
impl Child {
    fn new(kind: PropertiesChild, cpu: &NativeNmiContext) -> Option<Self> {
        use PropertiesChild::*;
        Some(match kind {
            BunnyStatus => Self::Bunny(Default::default()),
            DungeonExit => Self::Exit(Default::default()),
            SongList => Self::Songs(Default::default()),
            HudRefill => Self::Hud(Default::default()),
            AnimatedTiles => Self::Animated(Default::default()),
            Tilesets => Self::Tilesets(Default::default()),
            ScreenPalette => Self::Screen(Default::default()),
            OverworldPalettes | BackgroundColor | PaletteCache => Self::Area(
                OverworldAreaPalettes::from_entry((u32::from(cpu.bank) << 16) | u32::from(cpu.pc)),
            ),
            FixedColorAndScroll => Self::Fixed(Default::default()),
            Follower | ResetPlayer => Self::Player(LoadFilePlayer::from_entry(
                (u32::from(cpu.bank) << 16) | u32::from(cpu.pc),
            )),
            PlayerGraphics => Self::PlayerGraphics(Default::default()),
            SpriteReload => Self::Reload(Default::default()),
            MirrorPortal => Self::Mirror(Default::default()),
            BunnyPalettes => Self::BunnyPalettes(DungeonPalettes::from_entry(0x0ed6dd)),
            SongBankTail => Self::Bank(SongBank::from_entry(0x02854c)),
            SpecialExit => return None,
        })
    }
    fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> (Option<u64>, bool) {
        macro_rules! step {
            ($work:expr) => {{
                let work = $work;
                let cycles = work.step(cpu, clock, bus);
                (cycles, work.complete())
            }};
        }
        match self {
            Self::Bunny(w) => step!(w),
            Self::Exit(w) => step!(w),
            Self::Songs(w) => step!(w),
            Self::Hud(w) => step!(w),
            Self::Animated(w) => step!(w),
            Self::Tilesets(w) => step!(w),
            Self::Screen(w) => step!(w),
            Self::Area(w) => step!(w),
            Self::Fixed(w) => step!(w),
            Self::Player(w) => step!(w),
            Self::PlayerGraphics(w) => step!(w),
            Self::Reload(w) => step!(w),
            Self::Mirror(w) => step!(w),
            Self::BunnyPalettes(w) => step!(w),
            Self::Bank(w) => step!(w),
        }
    }
}

/// Immutable Module08 indirect-call table, also consumed by the source fixture.
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    let offset = (address & 0x7fffff).checked_sub(0x0283b9)? as usize;
    [0xc7, 0x83, 0x19, 0xaf, 0xb9, 0xed].get(offset).copied()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OverworldPropertiesCaller {
    pc: u16,
    properties: Option<PreOverworldProperties>,
    child: Option<Child>,
    complete: bool,
}
impl Default for OverworldPropertiesCaller {
    fn default() -> Self {
        Self {
            pc: 0x83bf,
            properties: None,
            child: None,
            complete: false,
        }
    }
}
impl OverworldPropertiesCaller {
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn active_child(&self) -> Option<PropertiesChild> {
        self.properties
            .as_ref()
            .and_then(PreOverworldProperties::child)
    }
    pub(crate) fn external_child(&self) -> Option<PropertiesChild> {
        if self.child.is_none() {
            self.active_child()
        } else {
            None
        }
    }
    pub(crate) fn return_from_external_child(&mut self, cpu: &NativeNmiContext) {
        assert!(
            self.external_child().is_some(),
            "no external properties child"
        );
        self.properties.as_mut().unwrap().return_from_child(cpu);
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
        if let Some(child) = self.child.as_mut() {
            let (cycles, complete) = child.step(cpu, clock, bus);
            if complete {
                self.properties.as_mut().unwrap().return_from_child(cpu);
                self.child = None;
            }
            return cycles;
        }
        if let Some(properties) = self.properties.as_mut() {
            if properties.complete() {
                self.properties = None;
                self.pc = 0x83c6;
            } else {
                if properties.child().is_some() {
                    return None;
                }
                let cycles = properties.step(cpu, clock, bus);
                if let Some(kind) = properties.child() {
                    self.child = Child::new(kind, cpu);
                }
                return cycles;
            }
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d, cpu.p & 0x38),
            (2, self.pc, 0, 0x30)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0x83bf => {
                let value = w.dp_read(0x11, 1);
                w.load(value, 1);
                0x83c1
            }
            0x83c1 => {
                w.shift_left(1);
                0x83c2
            }
            0x83c2 => {
                w.drain(6);
                w.cpu.x = w.cpu.a & 255;
                w.flags(w.cpu.x, 1);
                0x83c3
            }
            0x83c3 => {
                // JSR(abs,X) fetches its operand bytes around the internal
                // cycle, then reads the target before its two stack writes.
                w.drain(8);
                w.drain(6);
                w.drain(8);
                let target = w.read(0x0283b9 + u32::from(w.cpu.x), 2);
                assert_eq!(
                    target, 0x83c7,
                    "Module08 properties owner reached another submodule"
                );
                let sp = w.cpu.sp;
                w.push(0x83c5, 2);
                self.properties = Some(PreOverworldProperties::new(0x83c6, sp));
                target
            }
            0x83c6 => {
                w.rtl();
                self.complete = true;
                w.cpu.pc
            }
            _ => unreachable!("Module08 properties caller PC {:04x}", self.pc),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}
