//! Small timing models for translated CPU work that crosses raster events.
//!
//! Native game routines normally run atomically. A routine that outlives
//! vblank needs an explicit bus schedule when the PPU consumes memory while
//! the CPU is still authoring it.

const MASTER_CYCLES_PER_SCANLINE: u32 = 1364;
const NTSC_SCANLINES_PER_FIELD: u32 = 262;
const WRAM_REFRESH_CYCLE: u32 = 538;
const WRAM_REFRESH_STALL_CYCLES: u32 = 40;
const HDMA_INIT_CYCLE: u32 = 20;
const HDMA_START_CYCLE: u32 = 1106;

const ATTRACT_MAP_HDMA_BUS_STALL_CYCLES: u32 = 66;
const ATTRACT_MAP_PROJECTION_WORD_CYCLES: u32 = 504;
pub(crate) const ATTRACT_MAP_PROJECTION_WORDS: usize = 224;

const SPRITE_TUTORIAL_GUARD_OR_BARRIER: u8 = 0x3f;
const SPRITE_BLUE_GUARD: u8 = 0x41;
const SPRITE_MIRROR_PORTAL: u8 = 0x6c;

// Snes9x PC/raster traces around Sprite_Main ($06:8328) and the eventual
// WorldMap_FadeOut $2100 write ($00:8942). The fixed term includes the common
// mirror-portal Sprite_Main path and the Module0E suffix through that write.
const WORLD_MAP_FADE_FIXED_MASTER_CYCLES: u32 = 34_460;
const EMPTY_GARNISH_TABLE_SCAN_MASTER_CYCLES: u32 = 7_488;
const TUTORIAL_GUARD_OR_BARRIER_MASTER_CYCLES: u32 = 12_608;

/// Semantic parts of a `Sprite_Main` call that materially change when a
/// following CPU write reaches active scanout.
///
/// This is intentionally not a general cycle estimator. A consumer may use a
/// workload only when every active routine in it has a measured timing model.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SpriteMainTimingWorkload {
    tutorial_guard_or_barrier_count: u8,
    blue_guard_count: u8,
    mirror_portal_count: u8,
    other_active_sprite_count: u8,
    scans_all_garnish_slots: bool,
    active_garnish_count: u8,
}

impl SpriteMainTimingWorkload {
    pub(crate) fn record_active_sprite(&mut self, sprite_type: u8) {
        match sprite_type {
            SPRITE_TUTORIAL_GUARD_OR_BARRIER => {
                self.tutorial_guard_or_barrier_count += 1;
            }
            SPRITE_BLUE_GUARD => self.blue_guard_count += 1,
            SPRITE_MIRROR_PORTAL => self.mirror_portal_count += 1,
            _ => self.other_active_sprite_count += 1,
        }
    }

    pub(crate) fn record_garnish_table(&mut self, scans_all_slots: bool, active_garnish_count: u8) {
        self.scans_all_garnish_slots = scans_all_slots;
        self.active_garnish_count = active_garnish_count;
    }

    /// Raster boundary of the force-blank write in `WorldMap_FadeOut`.
    ///
    /// The canonical route exercises three fades. Full-route Snes9x tracing
    /// records V=43 for two tutorial barriers and V=30 for the enabled-but-
    /// empty garnish scan. Both use the same ROM call path. Unknown active
    /// sprite/garnish routines return `None` rather than borrowing either
    /// measured timing.
    pub(crate) fn world_map_fade_force_blank_output_scanline(self) -> Option<u8> {
        if self.mirror_portal_count != 1
            || self.blue_guard_count != 0
            || self.other_active_sprite_count != 0
            || self.active_garnish_count != 0
        {
            return None;
        }

        let mut clock = WORLD_MAP_FADE_FIXED_MASTER_CYCLES
            + u32::from(self.tutorial_guard_or_barrier_count)
                * TUTORIAL_GUARD_OR_BARRIER_MASTER_CYCLES;
        if self.scans_all_garnish_slots {
            clock += EMPTY_GARNISH_TABLE_SCAN_MASTER_CYCLES;
        }
        // The S-PPU V counter is one-based relative to the renderer's output
        // row index (a write observed at V=49 blanks output row 48).
        Some(((clock / MASTER_CYCLES_PER_SCANLINE) as u8).saturating_sub(1))
    }

    /// Raster boundary of the force-blank write in `DungMap_Backup` for
    /// measured dungeon sprite workloads.
    pub(crate) fn dungeon_map_backup_force_blank_output_scanline(self) -> Option<u8> {
        // The sanctuary map transition runs one active blue-guard routine and
        // no garnish work. Patched-core raster tracing places its INIDISP write
        // at the boundary which blanks output row 35.
        (self.blue_guard_count == 1
            && self.tutorial_guard_or_barrier_count == 0
            && self.mirror_portal_count == 0
            && self.other_active_sprite_count == 0
            && !self.scans_all_garnish_slots
            && self.active_garnish_count == 0)
            .then_some(35)
    }
}

/// Advance translated CPU work through the raster bus events that preempt the
/// attract-map projection loop.
///
/// These values mirror the Snes9x scheduler. WRAM refresh consumes 40 master
/// cycles. The two active mode-2 HDMA channels consume 66 cycles at HDMA init
/// and at each visible-line transfer.
fn advance_attract_map_projection_work(mut clock: u32, mut work: u32) -> u32 {
    while work != 0 {
        let scanline = clock / MASTER_CYCLES_PER_SCANLINE;
        let cycle = clock % MASTER_CYCLES_PER_SCANLINE;
        let field_scanline = scanline % NTSC_SCANLINES_PER_FIELD;
        let mut next_cycle = MASTER_CYCLES_PER_SCANLINE;
        let mut stall = 0;

        for (event_cycle, event_stall, enabled) in [
            (
                HDMA_INIT_CYCLE,
                ATTRACT_MAP_HDMA_BUS_STALL_CYCLES,
                field_scanline == 0,
            ),
            (WRAM_REFRESH_CYCLE, WRAM_REFRESH_STALL_CYCLES, true),
            (
                HDMA_START_CYCLE,
                ATTRACT_MAP_HDMA_BUS_STALL_CYCLES,
                field_scanline <= 224,
            ),
        ] {
            if enabled && cycle < event_cycle && event_cycle < next_cycle {
                next_cycle = event_cycle;
                stall = event_stall;
            }
        }

        let available = next_cycle - cycle;
        if work <= available {
            return clock + work;
        }
        clock += available + stall;
        work -= available;
    }
    clock
}

/// Whether the word authored by the ROM's descending Mode 7 projection loop
/// wins its race with the ascending HDMA consumer for this scanline.
pub(crate) fn attract_map_projection_current_word_is_visible(scanline: usize) -> bool {
    debug_assert!(scanline < ATTRACT_MAP_PROJECTION_WORDS);

    // Snes9x WRAM tracing at PCs $0C:F7AF/$0C:F7B7 records the first completed
    // word at V=237, HC=616. Each following word takes 504 master cycles of CPU
    // work; raster bus stalls are inserted by the scheduler above.
    let mut completion = 237 * MASTER_CYCLES_PER_SCANLINE + 616;
    for _ in scanline + 1..ATTRACT_MAP_PROJECTION_WORDS {
        completion =
            advance_attract_map_projection_work(completion, ATTRACT_MAP_PROJECTION_WORD_CYCLES);
    }

    let consumption = (NTSC_SCANLINES_PER_FIELD + scanline as u32) * MASTER_CYCLES_PER_SCANLINE
        + HDMA_START_CYCLE;
    completion <= consumption
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world_map_workload(
        tutorial_barriers: u8,
        scans_all_garnish_slots: bool,
    ) -> SpriteMainTimingWorkload {
        let mut workload = SpriteMainTimingWorkload::default();
        workload.record_active_sprite(SPRITE_MIRROR_PORTAL);
        for _ in 0..tutorial_barriers {
            workload.record_active_sprite(SPRITE_TUTORIAL_GUARD_OR_BARRIER);
        }
        workload.record_garnish_table(scans_all_garnish_slots, 0);
        workload
    }

    #[test]
    fn world_map_force_blank_follows_measured_sprite_main_work() {
        assert_eq!(
            world_map_workload(2, false).world_map_fade_force_blank_output_scanline(),
            Some(42)
        );
        assert_eq!(
            world_map_workload(0, true).world_map_fade_force_blank_output_scanline(),
            Some(29)
        );
    }

    #[test]
    fn world_map_force_blank_rejects_unmeasured_active_routines() {
        let mut workload = world_map_workload(0, true);
        workload.record_active_sprite(0x01);
        assert_eq!(workload.world_map_fade_force_blank_output_scanline(), None);

        let mut workload = world_map_workload(0, true);
        workload.record_garnish_table(true, 1);
        assert_eq!(workload.world_map_fade_force_blank_output_scanline(), None);
    }

    #[test]
    fn sanctuary_blue_guard_dungeon_map_blanks_after_the_hud_prefix() {
        let mut workload = SpriteMainTimingWorkload::default();
        workload.record_active_sprite(SPRITE_BLUE_GUARD);
        workload.record_garnish_table(false, 0);

        assert_eq!(
            workload.dungeon_map_backup_force_blank_output_scanline(),
            Some(35)
        );
        assert_eq!(workload.world_map_fade_force_blank_output_scanline(), None);
    }
}
