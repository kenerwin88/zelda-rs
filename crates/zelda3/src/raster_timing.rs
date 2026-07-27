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
