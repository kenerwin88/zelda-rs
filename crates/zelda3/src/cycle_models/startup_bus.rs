//! Device events from retained startup callers share their memory bus owner.
use super::{intro_clear::Access, intro_palettes::PaletteSource, native_work::WorkBus};
use snes::CpuMasterTimeline;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CallerOwner {
    Startup,
    Overlays,
    ScreenBuild,
    Properties(Option<super::pre_overworld_properties::PropertiesChild>),
    LoadFileParent,
    DungeonEntrance,
    DungeonHeader,
    DungeonRoomParent,
    DungeonFloors,
    DungeonObjectParser,
    DungeonObjectTiles,
    DungeonSavedObjects,
    DungeonDoorState,
    DungeonRoomUpload,
    DungeonAttributes,
    DungeonVisited,
    InteractiveCleanup,
    RoomEntryCache,
    FollowerInitialization,
    SpriteDisable,
    SpriteResetSuffix,
    SpriteCache,
    SpriteResetParent,
    SpriteRecords,
    OpeningPlayer,
}

pub(crate) enum StartupAccess {
    Memory(Access, PaletteSource, CallerOwner),
    EndScanline,
    InactiveSpriteBodies,
    EnterSpritePreparation,
}

pub(super) struct CallerBus<'a, F> {
    pub(super) callback: &'a mut F,
    pub(super) palette: PaletteSource,
    pub(super) owner: CallerOwner,
    pub(super) vblank: &'a mut Option<(u64, u64)>,
    pub(super) nmitimen: &'a mut Option<(u8, bool)>,
    pub(super) hdma: &'a mut Option<u8>,
    pub(super) in_vblank: bool,
}

impl<F: FnMut(u64, StartupAccess) -> u16> WorkBus for CallerBus<'_, F> {
    fn inactive_sprite_body_mask(&mut self) -> u16 {
        (self.callback)(0, StartupAccess::InactiveSpriteBodies)
    }
    fn observe_clock(&mut self, clock: &CpuMasterTimeline) {
        self.in_vblank = clock.synchronous_beam_position().unwrap().scanline >= 225;
    }
    fn access(&mut self, at: u64, access: Access) -> u16 {
        match access {
            Access::Vblank { field, at } => *self.vblank = Some((field, at)),
            Access::Write(0x4200, value, 1) => *self.nmitimen = Some((value as u8, self.in_vblank)),
            Access::Write(0x420c, value, 1) => *self.hdma = Some(value as u8),
            _ => {}
        }
        (self.callback)(at, StartupAccess::Memory(access, self.palette, self.owner))
    }
    fn end_scanline(&mut self, at: u64) {
        (self.callback)(at, StartupAccess::EndScanline);
    }
}
