//! Small timing models for translated CPU work that crosses raster events.
//!
//! Native game routines normally run atomically. A routine that outlives
//! vblank needs an explicit bus schedule when the PPU consumes memory while
//! the CPU is still authoring it.

const MASTER_CYCLES_PER_SCANLINE: u32 = 1364;
const NTSC_SCANLINES_PER_FIELD: u32 = 262;
// The pinned Snes9x core defaults to M1SNES (`_5A22 == 1`). Its CPU reset
// selects SNES_WRAM_REFRESH_HC_v1; 538 is the M2-only v2 position.
const WRAM_REFRESH_CYCLE: u32 = 530;
const WRAM_REFRESH_STALL_CYCLES: u32 = 40;
const HDMA_INIT_CYCLE: u32 = 20;
const HDMA_START_CYCLE: u32 = 1106;

const ATTRACT_MAP_HDMA_BUS_STALL_CYCLES: u32 = 66;
const ATTRACT_MAP_PROJECTION_WORD_CYCLES: u32 = 504;
pub(crate) const ATTRACT_MAP_PROJECTION_WORDS: usize = 224;

/// Whether the caller suffix after `Module07_11_01_FadeOut` crosses vblank.
///
/// This is deliberately limited to the fully measured straight-stair workload.
/// A pinned Snes9x PC/raster trace records the palette-countdown write at
/// `$00:E9AB`; a missing write on the following host frame means the 65816 is
/// still returning through the Module 7 caller rather than starting another
/// translated module iteration. Unknown rooms and staircase workloads stay
/// atomic instead of borrowing this cadence.
pub(crate) const fn straight_interroom_fadeout_suffix_crosses_vblank(
    main_module: u8,
    submodule: u8,
    subsubmodule: u8,
    dungeon_room: u8,
    staircase_index: u8,
    palette_countdown: u8,
) -> bool {
    main_module == 7
        && submodule == 0x12
        && subsubmodule == 1
        && match (dungeon_room, staircase_index) {
            (0x51, 0x30) => {
                matches!(palette_countdown, 1 | 3 | 5 | 6 | 7 | 9 | 11 | 13 | 17 | 20)
            }
            // The crystal-4 II route reaches vblank while returning from the
            // first fadeout step on this staircase. Keep the new observation
            // local until the remaining palette phases are measured.
            (0x32, 0x35) => palette_countdown == 1,
            _ => false,
        }
}

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
    blue_guard_full_animation_count: u8,
    unmeasured_blue_guard_count: u8,
    mirror_portal_count: u8,
    other_active_sprite_count: u8,
    scans_all_garnish_slots: bool,
    active_garnish_count: u8,
}

impl SpriteMainTimingWorkload {
    pub(crate) fn record_active_sprite(&mut self, sprite_type: u8, sprite_c: u8) {
        match sprite_type {
            SPRITE_TUTORIAL_GUARD_OR_BARRIER => {
                self.tutorial_guard_or_barrier_count += 1;
            }
            SPRITE_BLUE_GUARD if sprite_c == 0 => self.blue_guard_count += 1,
            SPRITE_BLUE_GUARD => self.unmeasured_blue_guard_count += 1,
            SPRITE_MIRROR_PORTAL => self.mirror_portal_count += 1,
            _ => self.other_active_sprite_count += 1,
        }
    }

    pub(crate) fn record_blue_guard_full_animation(&mut self) {
        self.blue_guard_full_animation_count += 1;
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
            || self.blue_guard_full_animation_count != 0
            || self.unmeasured_blue_guard_count != 0
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
        if self.tutorial_guard_or_barrier_count != 0
            || self.mirror_portal_count != 0
            || self.other_active_sprite_count != 0
            || self.scans_all_garnish_slots
            || self.active_garnish_count != 0
        {
            return None;
        }

        // Direct Snes9x traces at the common $00:8942 INIDISP write show that
        // Guard_Main's animation workload materially changes when
        // DungMap_Backup reaches active scanout. The offscreen room-$72 guard
        // returns from OAM preparation and writes at V=27 (output row 27); the
        // visible room-$71 guard draws the full animation and writes at V=36
        // (output row 35).
        match (
            self.blue_guard_count,
            self.blue_guard_full_animation_count,
            self.unmeasured_blue_guard_count,
        ) {
            (1, 0, 0) => Some(27),
            (1, 1, 0) => Some(35),
            _ => None,
        }
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
            // Snes9x processes a scheduled bus event while CPU.Cycles is at or
            // beyond NextEvent. Preserve that ownership when a work unit starts
            // exactly on the event boundary as well as when it crosses it.
            if enabled && cycle <= event_cycle && event_cycle < next_cycle {
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

    #[test]
    fn m1_wram_refresh_stalls_work_that_starts_or_crosses_cycle_530() {
        // Pinned Snes9x 1.63: globals.cpp selects M1SNES, cpu.cpp selects
        // SNES_WRAM_REFRESH_HC_v1, and snes9x.h defines it as 530 with a
        // 40-master-cycle refresh. cpuexec.cpp processes NextEvent at equality.
        assert_eq!(advance_attract_map_projection_work(530, 6), 576);
        assert_eq!(advance_attract_map_projection_work(524, 12), 576);
    }

    fn world_map_workload(
        tutorial_barriers: u8,
        scans_all_garnish_slots: bool,
    ) -> SpriteMainTimingWorkload {
        let mut workload = SpriteMainTimingWorkload::default();
        workload.record_active_sprite(SPRITE_MIRROR_PORTAL, 0);
        for _ in 0..tutorial_barriers {
            workload.record_active_sprite(SPRITE_TUTORIAL_GUARD_OR_BARRIER, 0);
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
        workload.record_active_sprite(0x01, 0);
        assert_eq!(workload.world_map_fade_force_blank_output_scanline(), None);

        let mut workload = world_map_workload(0, true);
        workload.record_garnish_table(true, 1);
        assert_eq!(workload.world_map_fade_force_blank_output_scanline(), None);
    }

    #[test]
    fn blue_guard_dungeon_map_blanking_follows_the_measured_animation_work() {
        let mut workload = SpriteMainTimingWorkload::default();
        workload.record_active_sprite(SPRITE_BLUE_GUARD, 0);
        workload.record_garnish_table(false, 0);

        assert_eq!(
            workload.dungeon_map_backup_force_blank_output_scanline(),
            Some(27)
        );
        assert_eq!(workload.world_map_fade_force_blank_output_scanline(), None);

        let mut workload = SpriteMainTimingWorkload::default();
        workload.record_active_sprite(SPRITE_BLUE_GUARD, 0);
        workload.record_blue_guard_full_animation();
        workload.record_garnish_table(false, 0);
        assert_eq!(
            workload.dungeon_map_backup_force_blank_output_scanline(),
            Some(35)
        );

        let mut workload = SpriteMainTimingWorkload::default();
        workload.record_active_sprite(SPRITE_BLUE_GUARD, 1);
        workload.record_garnish_table(false, 0);
        assert_eq!(
            workload.dungeon_map_backup_force_blank_output_scanline(),
            None
        );
    }

    #[test]
    fn straight_interroom_fadeout_uses_only_measured_caller_return_boundaries() {
        for countdown in 1..=23 {
            assert_eq!(
                straight_interroom_fadeout_suffix_crosses_vblank(7, 0x12, 1, 0x51, 0x30, countdown,),
                matches!(countdown, 1 | 3 | 5 | 6 | 7 | 9 | 11 | 13 | 17 | 20),
                "unexpected caller-return timing for palette step {countdown}",
            );
        }

        for countdown in 1..=23 {
            assert_eq!(
                straight_interroom_fadeout_suffix_crosses_vblank(7, 0x12, 1, 0x32, 0x35, countdown,),
                countdown == 1,
                "unexpected room-$32 staircase-$35 caller-return timing for palette step {countdown}",
            );
        }

        for unmeasured in [
            (6, 0x12, 1, 0x51, 0x30),
            (7, 0x11, 1, 0x51, 0x30),
            (7, 0x12, 2, 0x51, 0x30),
            (7, 0x12, 1, 0x52, 0x30),
            (7, 0x12, 1, 0x51, 0x31),
        ] {
            assert!(!straight_interroom_fadeout_suffix_crosses_vblank(
                unmeasured.0,
                unmeasured.1,
                unmeasured.2,
                unmeasured.3,
                unmeasured.4,
                1,
            ));
        }
    }
}
