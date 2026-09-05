// Methods ported from zelda3/src/messaging.c and included inside ZeldaState.

use super::*;
use crate::types::{sign16, Pair16U, Point16U};

// Snes9x defines a scanline as 341 dots at four master cycles per dot.
const SNES_MASTER_CYCLES_PER_SCANLINE: u32 = 341 * 4;
const SNES_NTSC_MASTER_CYCLES_PER_FRAME: u32 = 262 * SNES_MASTER_CYCLES_PER_SCANLINE;
// Measured at the VWF handler entry in Snes9x PC traces. The first line enters
// later because message setup precedes it; subsequent lines enter directly.
const VWF_FIRST_LINE_ENTRY_MASTER_CYCLES: u32 = 239_000;
// PC traces place an ordinary later-line handler entry 260,598 master cycles
// before NMI. When the prior handler's caller suffix returned in its own host
// slice, the next module iteration enters 271,344 cycles before NMI.
const VWF_LATER_LINE_ENTRY_MASTER_CYCLES: u32 = 260_598;
const VWF_AFTER_CALLER_SUFFIX_ENTRY_MASTER_CYCLES: u32 = 271_344;
const VWF_RESUMED_FRAME_MASTER_CYCLES: u32 = SNES_NTSC_MASTER_CYCLES_PER_FRAME;
// Successive glyph-loop entries were 488-530 master cycles apart in the
// oracle trace. Use their midpoint as the fixed handler-loop overhead.
const VWF_GLYPH_TRANSITION_MASTER_CYCLES: u32 = 510;
// Fixed work from the message-loop dispatch through VWF_RenderSingle's entry
// effects and drawing setup. The dialogue-click store is much earlier within
// that work: Snes9x PC traces place $0E:CACC 1,952-2,036 master cycles after
// the $0E:C984 message-loop restart. Keeping the post-click setup separate is
// observable when vblank lands after the click but before drawing begins.
const VWF_GLYPH_ENTRY_MASTER_CYCLES: u32 = 18_000;
const VWF_GLYPH_CLICK_MASTER_CYCLES: u32 = 2_000;
const VWF_GLYPH_POST_CLICK_ENTRY_MASTER_CYCLES: u32 =
    VWF_GLYPH_ENTRY_MASTER_CYCLES - VWF_GLYPH_CLICK_MASTER_CYCLES;
// Two oracle traces bracket the click-store boundary for an ordinary resumed
// glyph: 13,844 remaining master cycles publishes APUI03 before vblank, while
// 3,798 publishes it after. Six scanlines plus the traced click prefix is the
// smallest whole-scanline separator between those observations. A resume from
// PreparingDrawing has the longer measured caller suffix below; that shifts
// the same publication boundary by the suffix delta. Frame 26,512 is the
// regression witness: 19,334 cycles retains the click with the 28,000-cycle
// suffix, while the ordinary 13,844-cycle case still clears it.
const VWF_GLYPH_CLICK_VBLANK_MARGIN_MASTER_CYCLES: u32 = 6 * SNES_MASTER_CYCLES_PER_SCANLINE;
// From the RenderText handler epilogue through Module0E's scroll-register
// copies and NMI_PrepareSprites. Oracle PC traces measure about 16,300 master
// cycles for this caller suffix; a completion with less headroom is resumed
// after the intervening vblank instead of being folded into the same callback.
const VWF_CALLER_SUFFIX_MASTER_CYCLES: u32 = 16_500;
// A big-key receipt slice resumed while VWF_RenderSingle was still preparing
// its drawing loops, completed its final handler stores at V=223, reached
// Module0E's scroll-register suffix at V=224, and was interrupted by NMI at
// V=225. The semantic loop model reported 27,442 cycles of headroom for that
// trace. A map-receipt slice that resumed from the Drawing phase instead
// returned before NMI, so this calibration belongs to the suspended PC phase,
// not to every resumed glyph slice.
const VWF_PREPARING_DRAWING_CALLER_SUFFIX_MASTER_CYCLES: u32 = 28_000;
// A 262,662-cycle entry still returns after vblank in the Snes9x PC trace,
// while the 283,400-cycle entry returns before it. Six scanlines is the
// smallest whole-scanline return cost consistent with both measurements.
const VWF_SCROLL_RETURN_VBLANK_MARGIN_MASTER_CYCLES: u32 = 6 * SNES_MASTER_CYCLES_PER_SCANLINE;
const VWF_SCROLL_COMPLETES_BEFORE_NEXT_VBLANK_MASTER_CYCLES: u32 =
    VWF_LATER_LINE_ENTRY_MASTER_CYCLES + VWF_SCROLL_RETURN_VBLANK_MARGIN_MASTER_CYCLES;

impl DialogueScrollCompletionTiming {
    pub(crate) const fn at_scroll_entry(cycles_before_vblank: u32) -> Self {
        if cycles_before_vblank >= VWF_SCROLL_COMPLETES_BEFORE_NEXT_VBLANK_MASTER_CYCLES {
            Self::BeforeNextVblank
        } else {
            Self::AfterReturnBoundary
        }
    }
}

fn vwf_render_loop_cycle_budget(
    resuming: bool,
    current_line: u16,
    entry_phase: VwfHandlerEntryPhase,
) -> u32 {
    if resuming {
        VWF_RESUMED_FRAME_MASTER_CYCLES
    } else if current_line == 0 {
        VWF_FIRST_LINE_ENTRY_MASTER_CYCLES
    } else if entry_phase == VwfHandlerEntryPhase::AfterDeferredCallerSuffix {
        VWF_AFTER_CALLER_SUFFIX_ENTRY_MASTER_CYCLES
    } else {
        VWF_LATER_LINE_ENTRY_MASTER_CYCLES
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum VwfHandlerEntryPhase {
    #[default]
    OrdinaryModuleIteration,
    AfterDeferredCallerSuffix,
}

fn vwf_render_glyph_master_cycles(width: u8, x: u8) -> u32 {
    // ROM $0E:CBF2/$0E:CC90 renders columns only until the current 8-pixel
    // tile boundary; any remaining font bits are stored as the next tile's
    // seed word. The exact traced cost therefore follows this inner-loop
    // iteration count rather than raw glyph width.
    let columns = u32::from(width.min(8 - (x & 7)));
    VWF_GLYPH_ENTRY_MASTER_CYCLES + columns * 8_000
}

fn vwf_render_glyph_drawing_master_cycles(width: u8, x: u8) -> u32 {
    vwf_render_glyph_master_cycles(width, x) - VWF_GLYPH_ENTRY_MASTER_CYCLES
}

const fn vwf_new_glyph_click_requires_boundary_retention(
    cycles_left: u32,
    handler_entry_glyph_phase: VwfGlyphCpuPhase,
) -> bool {
    // The suspended entry work shifts which side of Snes9x's next NMI the
    // final click lands on. Entering still owes the complete post-click setup;
    // PreparingDrawing uses the independently measured longer caller-return
    // boundary. A Drawing resume has no entry-phase shift.
    let resume_phase_shift = match handler_entry_glyph_phase {
        VwfGlyphCpuPhase::Entering { .. } => VWF_GLYPH_POST_CLICK_ENTRY_MASTER_CYCLES,
        VwfGlyphCpuPhase::PreparingDrawing { .. } => {
            VWF_PREPARING_DRAWING_CALLER_SUFFIX_MASTER_CYCLES - VWF_CALLER_SUFFIX_MASTER_CYCLES
        }
        VwfGlyphCpuPhase::Ready | VwfGlyphCpuPhase::Drawing { .. } => 0,
    };
    cycles_left
        < VWF_GLYPH_CLICK_MASTER_CYCLES
            + VWF_GLYPH_CLICK_VBLANK_MARGIN_MASTER_CYCLES
            + resume_phase_shift
}

fn vwf_interrupted_click_marks_boundary(
    phase: VwfGlyphCpuPhase,
    retain_incomplete_click: bool,
) -> bool {
    retain_incomplete_click || phase.vblank_follows_click_before_drawing()
}

fn debug_vwf_budget_for_frame(host_frame: u32) -> bool {
    if std::env::var_os("ZELDA3_DEBUG_VWF_BUDGET").is_some() {
        return true;
    }
    std::env::var("ZELDA3_DEBUG_VWF_BUDGET_FRAME")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        == Some(host_frame)
}

fn vwf_glyph_cursor_after_pending_line_transition(
    current_cursor: usize,
    current_line: u16,
    next_line_requested: bool,
) -> usize {
    if next_line_requested {
        VWF_RENDER_CHARACTER_LINE_POSITIONS[(current_line >> 1) as usize] as usize
    } else {
        current_cursor
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum VwfGlyphCpuPhase {
    #[default]
    Ready,
    Entering {
        remaining_master_cycles: u32,
        post_click_master_cycles: u32,
        drawing_master_cycles: u32,
    },
    PreparingDrawing {
        remaining_master_cycles: u32,
        drawing_master_cycles: u32,
    },
    Drawing {
        remaining_master_cycles: u32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct VwfGlyphCpuAdvance {
    next_phase: VwfGlyphCpuPhase,
    consumed_master_cycles: u32,
    entered_function: bool,
    completed: bool,
}

impl VwfGlyphCpuPhase {
    pub(crate) fn is_ready(self) -> bool {
        matches!(self, Self::Ready)
    }

    pub(crate) fn vblank_follows_click_before_drawing(self) -> bool {
        matches!(self, Self::PreparingDrawing { .. })
    }

    fn advance(self, available: u32, drawing_master_cycles: u32) -> VwfGlyphCpuAdvance {
        match self {
            Self::Ready => Self::advance_click(
                VWF_GLYPH_CLICK_MASTER_CYCLES,
                VWF_GLYPH_POST_CLICK_ENTRY_MASTER_CYCLES,
                drawing_master_cycles,
                available,
            ),
            Self::Entering {
                remaining_master_cycles,
                post_click_master_cycles,
                drawing_master_cycles,
            } => Self::advance_click(
                remaining_master_cycles,
                post_click_master_cycles,
                drawing_master_cycles,
                available,
            ),
            Self::PreparingDrawing {
                remaining_master_cycles,
                drawing_master_cycles,
            } => {
                Self::advance_post_click(remaining_master_cycles, drawing_master_cycles, available)
            }
            Self::Drawing {
                remaining_master_cycles,
            } if remaining_master_cycles > available => VwfGlyphCpuAdvance {
                next_phase: Self::Drawing {
                    remaining_master_cycles: remaining_master_cycles - available,
                },
                consumed_master_cycles: available,
                entered_function: false,
                completed: false,
            },
            Self::Drawing {
                remaining_master_cycles,
            } => VwfGlyphCpuAdvance {
                next_phase: Self::Ready,
                consumed_master_cycles: remaining_master_cycles,
                entered_function: false,
                completed: true,
            },
        }
    }

    fn advance_click(
        click_master_cycles: u32,
        post_click_master_cycles: u32,
        drawing_master_cycles: u32,
        available: u32,
    ) -> VwfGlyphCpuAdvance {
        if click_master_cycles > available {
            return VwfGlyphCpuAdvance {
                next_phase: Self::Entering {
                    remaining_master_cycles: click_master_cycles - available,
                    post_click_master_cycles,
                    drawing_master_cycles,
                },
                consumed_master_cycles: available,
                entered_function: false,
                completed: false,
            };
        }
        let after_click = Self::advance_post_click(
            post_click_master_cycles,
            drawing_master_cycles,
            available - click_master_cycles,
        );
        VwfGlyphCpuAdvance {
            next_phase: after_click.next_phase,
            consumed_master_cycles: click_master_cycles + after_click.consumed_master_cycles,
            entered_function: true,
            completed: after_click.completed,
        }
    }

    fn advance_post_click(
        post_click_master_cycles: u32,
        drawing_master_cycles: u32,
        available: u32,
    ) -> VwfGlyphCpuAdvance {
        if post_click_master_cycles > available {
            return VwfGlyphCpuAdvance {
                next_phase: Self::PreparingDrawing {
                    remaining_master_cycles: post_click_master_cycles - available,
                    drawing_master_cycles,
                },
                consumed_master_cycles: available,
                entered_function: false,
                completed: false,
            };
        }
        let after_entry = available - post_click_master_cycles;
        if drawing_master_cycles > after_entry {
            return VwfGlyphCpuAdvance {
                next_phase: Self::Drawing {
                    remaining_master_cycles: drawing_master_cycles - after_entry,
                },
                consumed_master_cycles: available,
                entered_function: false,
                completed: false,
            };
        }
        VwfGlyphCpuAdvance {
            next_phase: Self::Ready,
            consumed_master_cycles: post_click_master_cycles + drawing_master_cycles,
            entered_function: false,
            completed: true,
        }
    }

    const fn remaining_master_cycles(self) -> u32 {
        match self {
            Self::Ready => 0,
            Self::Entering {
                remaining_master_cycles,
                post_click_master_cycles,
                drawing_master_cycles,
            } => remaining_master_cycles + post_click_master_cycles + drawing_master_cycles,
            Self::PreparingDrawing {
                remaining_master_cycles,
                drawing_master_cycles,
            } => remaining_master_cycles + drawing_master_cycles,
            Self::Drawing {
                remaining_master_cycles,
            } => remaining_master_cycles,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VwfCpuSliceOutcome {
    InterruptedMidGlyph {
        retain_incomplete_click: bool,
    },
    /// The temporary source authority returned from this host interval after
    /// the native C decoder reached the same semantic message endpoint. The
    /// handler remains suspended; no caller suffix has executed yet.
    AuthorityBoundaryReached,
    HandlerComplete {
        master_cycles_before_vblank: u32,
        caller_suffix_master_cycles: u32,
    },
}

/// Deterministic native work needed to bring one already-suspended VWF caller
/// to the decoder endpoint published by the temporary Live timing authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SuspendedVwfEndpointTransition {
    start_read_position: u16,
    target_read_position: u16,
    slice_count: u32,
    current_glyph_started: bool,
}

/// Deterministic native work needed to finish one already-suspended VWF
/// character handler after its last source-published decoder endpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SuspendedVwfCompletionTransition {
    start_read_position: u16,
    end_read_position: u16,
    slice_count: u32,
    caller_suffix_crossed_vblank: bool,
    /// The completed handler consumed a line-feed command and armed the
    /// message-line scroll continuation; the following hosts own its
    /// per-frame pixel cadence.
    begins_message_line_scroll: bool,
}

impl SuspendedVwfCompletionTransition {
    pub(super) const fn caller_suffix_crossed_vblank(self) -> bool {
        self.caller_suffix_crossed_vblank
    }

    pub(super) const fn begins_message_line_scroll(self) -> bool {
        self.begins_message_line_scroll
    }

    #[cfg(test)]
    pub(super) const fn slice_count(self) -> u32 {
        self.slice_count
    }
}

impl SuspendedVwfEndpointTransition {
    pub(super) const fn advanced_native_vwf(self) -> bool {
        self.slice_count != 0 || self.current_glyph_started
    }

    #[cfg(test)]
    pub(super) const fn slice_count(self) -> u32 {
        self.slice_count
    }
}

impl VwfCpuSliceOutcome {
    const fn caller_suffix_crosses_vblank(self) -> bool {
        matches!(
            self,
            Self::HandlerComplete {
                master_cycles_before_vblank,
                caller_suffix_master_cycles,
            } if master_cycles_before_vblank < caller_suffix_master_cycles
        )
    }
}

#[cfg(test)]
mod fast_forward_cycle_tests {
    use super::{
        vwf_glyph_cursor_after_pending_line_transition, vwf_interrupted_click_marks_boundary,
        vwf_new_glyph_click_requires_boundary_retention, vwf_render_glyph_drawing_master_cycles,
        vwf_render_glyph_master_cycles, vwf_render_loop_cycle_budget, VwfCpuSliceOutcome,
        VwfGlyphCpuPhase, VwfHandlerEntryPhase, VWF_AFTER_CALLER_SUFFIX_ENTRY_MASTER_CYCLES,
        VWF_CALLER_SUFFIX_MASTER_CYCLES, VWF_FIRST_LINE_ENTRY_MASTER_CYCLES,
        VWF_GLYPH_CLICK_MASTER_CYCLES, VWF_GLYPH_ENTRY_MASTER_CYCLES,
        VWF_GLYPH_POST_CLICK_ENTRY_MASTER_CYCLES, VWF_LATER_LINE_ENTRY_MASTER_CYCLES,
        VWF_RENDER_CHARACTER_LINE_POSITIONS, VWF_RESUMED_FRAME_MASTER_CYCLES,
    };

    #[test]
    fn render_loop_budget_tracks_rom_entry_and_resume_phases() {
        assert_eq!(
            vwf_render_loop_cycle_budget(false, 0, VwfHandlerEntryPhase::OrdinaryModuleIteration),
            VWF_FIRST_LINE_ENTRY_MASTER_CYCLES
        );
        assert_eq!(
            vwf_render_loop_cycle_budget(false, 2, VwfHandlerEntryPhase::OrdinaryModuleIteration),
            VWF_LATER_LINE_ENTRY_MASTER_CYCLES
        );
        assert_eq!(
            vwf_render_loop_cycle_budget(true, 0, VwfHandlerEntryPhase::AfterDeferredCallerSuffix),
            VWF_RESUMED_FRAME_MASTER_CYCLES
        );
        assert_eq!(
            vwf_render_loop_cycle_budget(false, 2, VwfHandlerEntryPhase::AfterDeferredCallerSuffix),
            VWF_AFTER_CALLER_SUFFIX_ENTRY_MASTER_CYCLES
        );
    }

    #[test]
    fn glyph_cost_stops_at_the_current_tile_boundary() {
        assert_eq!(vwf_render_glyph_master_cycles(6, 0), 66_000);
        assert_eq!(vwf_render_glyph_master_cycles(6, 5), 42_000);
        assert_eq!(vwf_render_glyph_master_cycles(7, 7), 26_000);
    }

    #[test]
    fn glyph_click_precedes_remaining_entry_work_at_vblank() {
        let drawing = vwf_render_glyph_drawing_master_cycles(6, 1);
        let before_click =
            VwfGlyphCpuPhase::Ready.advance(VWF_GLYPH_CLICK_MASTER_CYCLES - 1, drawing);
        assert!(!before_click.entered_function);
        assert!(!before_click.completed);
        assert!(matches!(
            before_click.next_phase,
            VwfGlyphCpuPhase::Entering { .. }
        ));
        assert!(!before_click
            .next_phase
            .vblank_follows_click_before_drawing());
        assert!(!before_click.next_phase.is_ready());

        let at_click = before_click.next_phase.advance(1, drawing);
        assert!(at_click.entered_function);
        assert!(!at_click.completed);
        assert!(matches!(
            at_click.next_phase,
            VwfGlyphCpuPhase::PreparingDrawing { .. }
        ));
        assert!(at_click.next_phase.vblank_follows_click_before_drawing());

        let after_entry = at_click
            .next_phase
            .advance(VWF_GLYPH_POST_CLICK_ENTRY_MASTER_CYCLES, drawing);
        assert!(!after_entry.entered_function, "the click must not repeat");
        assert!(!after_entry.completed);
        assert!(matches!(
            after_entry.next_phase,
            VwfGlyphCpuPhase::Drawing { .. }
        ));
        assert!(!after_entry.next_phase.vblank_follows_click_before_drawing());
        assert!(!after_entry.next_phase.is_ready());
        assert_eq!(
            before_click.consumed_master_cycles
                + at_click.consumed_master_cycles
                + after_entry.consumed_master_cycles,
            VWF_GLYPH_ENTRY_MASTER_CYCLES
        );
    }

    #[test]
    fn glyph_click_boundary_uses_the_oracle_bracket_not_the_semantic_phase_alone() {
        assert!(vwf_new_glyph_click_requires_boundary_retention(
            3_798,
            VwfGlyphCpuPhase::Ready,
        ));
        assert!(!vwf_new_glyph_click_requires_boundary_retention(
            13_844,
            VwfGlyphCpuPhase::Drawing {
                remaining_master_cycles: 5_444,
            },
        ));
        assert!(vwf_new_glyph_click_requires_boundary_retention(
            19_334,
            VwfGlyphCpuPhase::PreparingDrawing {
                remaining_master_cycles: 7_954,
                drawing_master_cycles: 24_000,
            },
        ));
        assert!(vwf_new_glyph_click_requires_boundary_retention(
            19_798,
            VwfGlyphCpuPhase::Entering {
                remaining_master_cycles: 2_000,
                post_click_master_cycles: 16_000,
                drawing_master_cycles: 48_000,
            },
        ));
        assert!(vwf_interrupted_click_marks_boundary(
            VwfGlyphCpuPhase::Drawing {
                remaining_master_cycles: 30_666,
            },
            true,
        ));
        assert!(!vwf_interrupted_click_marks_boundary(
            VwfGlyphCpuPhase::Drawing {
                remaining_master_cycles: 28_156,
            },
            false,
        ));
    }

    #[test]
    fn pending_line_transition_selects_the_rom_render_cursor() {
        assert_eq!(
            vwf_glyph_cursor_after_pending_line_transition(24, 2, true),
            VWF_RENDER_CHARACTER_LINE_POSITIONS[1] as usize
        );
        assert_eq!(
            vwf_glyph_cursor_after_pending_line_transition(24, 2, false),
            24
        );
    }

    #[test]
    fn caller_suffix_continues_only_when_vblank_owns_the_boundary() {
        assert!(VwfCpuSliceOutcome::HandlerComplete {
            master_cycles_before_vblank: VWF_CALLER_SUFFIX_MASTER_CYCLES - 1,
            caller_suffix_master_cycles: VWF_CALLER_SUFFIX_MASTER_CYCLES,
        }
        .caller_suffix_crosses_vblank());
        assert!(!VwfCpuSliceOutcome::HandlerComplete {
            master_cycles_before_vblank: VWF_CALLER_SUFFIX_MASTER_CYCLES,
            caller_suffix_master_cycles: VWF_CALLER_SUFFIX_MASTER_CYCLES,
        }
        .caller_suffix_crosses_vblank());
        assert!(VwfCpuSliceOutcome::HandlerComplete {
            master_cycles_before_vblank: super::VWF_PREPARING_DRAWING_CALLER_SUFFIX_MASTER_CYCLES
                - 1,
            caller_suffix_master_cycles: super::VWF_PREPARING_DRAWING_CALLER_SUFFIX_MASTER_CYCLES,
        }
        .caller_suffix_crosses_vblank());
        assert!(!VwfCpuSliceOutcome::HandlerComplete {
            master_cycles_before_vblank: super::VWF_PREPARING_DRAWING_CALLER_SUFFIX_MASTER_CYCLES,
            caller_suffix_master_cycles: super::VWF_PREPARING_DRAWING_CALLER_SUFFIX_MASTER_CYCLES,
        }
        .caller_suffix_crosses_vblank());
        assert!(!VwfCpuSliceOutcome::InterruptedMidGlyph {
            retain_incomplete_click: false,
        }
        .caller_suffix_crosses_vblank());
    }
}

mod messaging_shared;
use messaging_shared::*;

fn text_decode_cmd(a: u8, src: *const u8) -> u32 {
    let decoded = crate::dialogue_ir::decode_dialogue_byte(0, a, unsafe { src.as_ref().copied() });
    ((decoded.param as u32) << 16) | ((decoded.command as u32) << 8)
}

impl ZeldaState {
    pub(super) fn Module0E_Interface(&mut self) {
        // A fast-forward message render slice: the ROM's NMI skips the core
        // game update (sprites/Link) while the main thread finishes the render.
        let mut skip_run = self.dialogue_fast_forward_hold_active;
        if self.game_state.world.location.is_indoors() {
            if self.game_state.frame.submodule == 3 {
                skip_run = self.overworld_map_state() != 0 && self.overworld_map_state() != 7;
            } else {
                self.dungeon_push_block_handler();
            }
        } else {
            skip_run |= (self.game_state.frame.submodule == 7
                || self.game_state.frame.submodule == 10)
                && self.overworld_map_state() != 0;
        }
        if !skip_run {
            self.arm_dialogue_initialization_schedule_if_needed();
            self.sprite_main();
            if self
                .game_execution_scheduler
                .work_suspends_translated_call_stack()
            {
                return;
            }
            self.complete_module0e_after_sprite_main();
            return;
        }
        self.complete_module0e_run_interface();
    }

    /// Measure the ROM's Text_Initialize call at a dialogue module's entry
    /// (Module0E_Interface, Module1B_SpawnSelect) and arm the schedule the
    /// translated `Text_Initialize` consumes.
    fn arm_dialogue_initialization_schedule_if_needed(&mut self) {
        if self.rom_startup_timing()
            && self.game_state.frame.submodule == 2
            && self.game_state.messaging.runtime.module() == 0
            && self.pending_dialogue_initialization_schedule.is_none()
        {
            // The module enters in this measured raster interval. Execute
            // both endpoints against the live ROM/state: the host only
            // consumes the result when the interval has one schedule.
            let earliest = super::dialogue_initialization_cpu_plan(self, (255, 528));
            let latest = super::dialogue_initialization_cpu_plan(self, (255, 700));
            assert_eq!(
                earliest.schedule_key(),
                latest.schedule_key(),
                "dialogue CPU schedule varies across the measured module entry interval"
            );
            self.pending_dialogue_initialization_schedule = Some((
                earliest.prefix_nmi_crossings(),
                earliest.caller_nmi_crossings(),
                Some(earliest.following_main_nmi_uses_host_animated_bg_operands()),
            ));
            if std::env::var_os("ZELDA3_DEBUG_DIALOGUE_CPU_PLAN").is_some() {
                eprintln!(
                    "dialogue_cpu_plan host={} module={:#04x} msg={:#06x} speed={} earliest={:?} latest={:?} prefix_crossings={} caller_crossings={} following_main_animated_bg={:?}",
                    self.frame_ctr_dbg,
                    self.game_state.frame.main_module,
                    self.game_state.messaging.dialogue_message_index.value(),
                    self.game_state.messaging.runtime.vwf_line_speed(),
                    earliest.diagnostic(),
                    latest.diagnostic(),
                    earliest.prefix_nmi_crossings(),
                    earliest.caller_nmi_crossings(),
                    earliest.following_main_nmi_uses_host_animated_bg_operands(),
                );
            }
        }
    }

    /// Resume `Module0E_Interface` immediately after `Sprite_Main` returns.
    /// A synchronous sprite item receipt can suspend at that call boundary;
    /// this method preserves the exact remaining C statement order without
    /// replaying Sprite_Main or publishing the caller suffix early.
    pub(super) fn complete_module0e_after_sprite_main(&mut self) {
        self.link_oam_main();
        if self.game_state.world.location.is_outdoors() {
            self.OverworldOverlay_HandleRain();
        }
        self.hud_refill_logic();
        if self.game_state.frame.submodule != 2 {
            self.orient_lamp_light_cone();
        }
        self.complete_module0e_run_interface();
    }

    fn complete_module0e_run_interface(&mut self) {
        self.replay_trace_ram_watch("module0e-before-run-interface");
        self.RunInterface();
        self.replay_trace_ram_watch("module0e-after-run-interface");
        if self.rom_startup_timing()
            && (self.game_execution_scheduler.work_is_pending()
                // RenderText_Draw_Scroll is an interruptible caller frame, not
                // an atomic buffer rewrite. Instrumented Snes9x runs remain in
                // its $0e:cfe2..$0e:d088 copy loop across the next vblanks, so
                // Module0E_Interface has not reached the scroll-register suffix
                // at $00:f873 while these continuation slices are pending.
                || !self.dialogue_scroll_cpu_is_idle()
                || self.pre_main_caller_continuation_is(
                    PreMainCallerContinuation::DialogueVwfReturn,
                ))
        {
            return;
        }
        self.complete_module0e_interface_after_run();
    }

    pub(super) fn complete_module0e_interface_after_run(&mut self) {
        let bg1_x_offset = self.game_state.world.scroll.bg1_x_offset();
        let bg1_y_offset = self.game_state.world.scroll.bg1_y_offset();
        let bg2x = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg2_h_copy2()
            .wrapping_add(bg1_x_offset);
        let bg2y = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg2_v_copy2()
            .wrapping_add(bg1_y_offset);
        let bg1x = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg1_h_copy2()
            .wrapping_add(bg1_x_offset);
        let bg1y = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg1_v_copy2()
            .wrapping_add(bg1_y_offset);
        self.set_bg2_h_copy(bg2x);
        self.set_bg2_v_copy(bg2y);
        self.set_bg1_h_copy(bg1x);
        self.set_bg1_v_copy(bg1y);
        self.replay_trace_ram_watch("module0e-after-scroll-copy");
    }

    pub(super) fn Module_Messaging_0(&mut self) {
        // C Module_Messaging_0 is an assert(0) dispatch slot.
        panic!("Module_Messaging_0 hit unsupported C assert(0) path");
    }

    pub(super) fn RunInterface(&mut self) {
        match self.game_state.frame.submodule {
            0 => self.Module_Messaging_0(),
            1 => self.hud_module_run(),
            2 => self.RenderText(),
            3 => self.Module0E_03_DungeonMap(),
            4 => self.Module0E_04_RedPotion(),
            5 => self.Module0E_05_DesertPrayer(),
            6 => self.Module_Messaging_6(),
            7 => self.Messaging_OverworldMap(),
            8 => self.Module0E_08_GreenPotion(),
            9 => self.Module0E_09_BluePotion(),
            10 => self.Module0E_0A_FluteMenu(),
            11 => self.Module0E_0B_SaveMenu(),
            // Master Sword item receipt sets submodule 43. C dispatches through
            // the 12-entry messaging table without bounds checks, landing on
            // kModule_BossVictory[3] in the adjacent static table.
            43 => self.dungeon_close_victory_spin(),
            // C indexes kMessagingSubmodules directly; this is the Rust
            // bounds guard for the same dispatch table.
            submodule => panic!("RunInterface invalid submodule {submodule}"),
        }
    }

    pub(super) fn Module_Messaging_6(&mut self) {
        // C Module_Messaging_6 is an assert(0) dispatch slot.
        panic!("Module_Messaging_6 hit unsupported C assert(0) path");
    }

    pub(super) fn GetDungmapFloorLayout(&self) -> Vec<u8> {
        let idx = (self.game_state.inventory.save_progress.palace_index_x2() >> 1) as usize;
        self.asset_memblk(97, idx)
            .map(|blk| blk.ptr.to_vec())
            .unwrap_or_default()
    }

    pub(super) fn GetOtherDungmapInfo(&self, count: usize) -> u8 {
        let idx = (self.game_state.inventory.save_progress.palace_index_x2() >> 1) as usize;
        self.asset_memblk(98, idx)
            .and_then(|blk| blk.ptr.get(count).copied())
            .unwrap_or(0)
    }

    pub(super) fn GetLightOverworldTilemap(&self) -> Vec<u8> {
        self.asset_raw(67)
            .map(|tilemap| tilemap.to_vec())
            .unwrap_or_default()
    }

    pub(super) fn Module0E_05_DesertPrayer(&mut self) {
        match self.game_state.frame.subsubmodule {
            0 => self.ResetTransitionPropsAndAdvance_ResetInterface(),
            1 => self.ApplyPaletteFilter_bounce(),
            2 => {
                if let Some(
                    interruption @ crate::MainLoopInterruption::DesertPrayerIris {
                        source_subsubmodule,
                        palette_countdown,
                        radius,
                        progress,
                    },
                ) = self.original_timing_main_loop_interruption()
                {
                    assert_eq!(source_subsubmodule, 2);
                    assert_eq!(
                        u16::from(palette_countdown),
                        self.game_state.display.palette_filter.countdown_word(),
                    );
                    assert!(
                        self.take_original_timing_main_loop_interruption(interruption),
                        "Desert Prayer iris interruption disappeared before its source caller consumed it",
                    );
                    self.CleanUpAndPrepDesertPrayerHDMA();
                    self.set_spotlight_window_radius_byte(0x26);
                    self.set_spotlight_window_state_byte(0);
                    assert_eq!(
                        u16::from(self.game_state.display.spotlight_hdma.window_radius_byte(),),
                        radius,
                    );
                    self.begin_desert_prayer_iris(
                        progress,
                        crate::zelda_rtl::DesertPrayerIrisCaller::InitializeCase2,
                    );
                    return;
                }
                self.DesertPrayer_InitializeIrisHDMA();
                self.complete_desert_prayer_case2_after_iris();
            }
            3 => {
                if let Some(
                    interruption @ crate::MainLoopInterruption::DesertPrayerIris {
                        source_subsubmodule,
                        palette_countdown,
                        radius,
                        progress,
                    },
                ) = self.original_timing_main_loop_interruption()
                {
                    assert!(
                        self.take_original_timing_main_loop_interruption(interruption),
                        "Desert Prayer iris interruption disappeared before its source caller consumed it",
                    );
                    self.ApplyPaletteFilter_bounce();
                    assert_eq!(
                        self.game_state.display.palette_filter.countdown_word(),
                        u16::from(palette_countdown),
                    );
                    assert_eq!(self.game_state.frame.subsubmodule, source_subsubmodule);
                    assert_eq!(
                        u16::from(self.game_state.display.spotlight_hdma.window_radius_byte(),),
                        radius,
                    );
                    self.begin_desert_prayer_iris(
                        progress,
                        crate::zelda_rtl::DesertPrayerIrisCaller::PaletteFilterCase3,
                    );
                    return;
                }
                if let Some(
                    interruption @ crate::MainLoopInterruption::DesertPrayerPaletteFilterBeforeColor {
                        countdown,
                        next_color,
                    },
                ) = self.original_timing_main_loop_interruption()
                {
                    assert!(
                        self.take_original_timing_main_loop_interruption(interruption),
                        "Desert Prayer palette interruption disappeared before its source caller consumed it",
                    );
                    self.apply_palette_filter_bounce_prefix(next_color);
                    self.game_execution_scheduler.schedule_work(
                        crate::zelda_rtl::GameWorkContinuation::FinishDesertPrayerPaletteFilter {
                            countdown,
                            next_color,
                        },
                        1,
                    );
                    return;
                }
                self.ApplyPaletteFilter_bounce();
                self.DesertPrayer_BuildIrisHDMATable();
            }
            4 => {
                if let Some(
                    interruption @ crate::MainLoopInterruption::DesertPrayerIris {
                        source_subsubmodule,
                        palette_countdown,
                        radius,
                        progress,
                    },
                ) = self.original_timing_main_loop_interruption()
                {
                    assert_eq!(source_subsubmodule, 4);
                    assert_eq!(
                        u16::from(palette_countdown),
                        self.game_state.display.palette_filter.countdown_word(),
                    );
                    assert_eq!(
                        u16::from(self.game_state.display.spotlight_hdma.window_radius_byte(),),
                        radius,
                    );
                    assert!(
                        self.take_original_timing_main_loop_interruption(interruption),
                        "Desert Prayer iris interruption disappeared before its source caller consumed it",
                    );
                    self.begin_desert_prayer_iris(
                        progress,
                        crate::zelda_rtl::DesertPrayerIrisCaller::RecurringCase4,
                    );
                    return;
                }
                self.DesertPrayer_BuildIrisHDMATable();
            }
            _ => {}
        }
    }

    pub(super) fn Module0E_04_RedPotion(&mut self) {
        if self.hud_refill_health() {
            self.finish_potion_refill();
        }
    }

    pub(super) fn Module0E_08_GreenPotion(&mut self) {
        if self.hud_refill_magic_power() {
            self.finish_potion_refill();
        }
    }

    pub(super) fn Module0E_09_BluePotion(&mut self) {
        if self.hud_refill_health() {
            self.set_submodule(8);
        }
        if self.hud_refill_magic_power() {
            self.set_submodule(4);
        }
    }

    fn finish_potion_refill(&mut self) {
        self.follower_link_state_mut()
            .clear_button_mask_b_y_bits(0x40);
        self.increment_hud_update_flag();
        self.set_submodule(0);
        let saved_module = self.game_state.frame.saved_module_for_menu;
        self.set_main_module(saved_module);
    }

    pub(super) fn Module0E_0B_SaveMenu(&mut self) {
        if self.rom_startup_timing()
            && self.game_state.messaging.runtime.module() == 0
            && matches!(
                self.original_timing_owner(),
                crate::zelda_rtl::OriginalTimingOwner::Live
            )
        {
            match self.take_original_timing_save_menu_initialization_progress() {
                Some(crate::SaveMenuInitializationProgress::Completed) => {}
                Some(crate::SaveMenuInitializationProgress::InProgress) => return,
                None => {
                    assert!(
                        std::mem::take(&mut self.save_menu_initialization_completed_pending),
                        "live timing authority omitted save-menu initialization progress",
                    );
                }
            }
        }
        if self.game_state.world.location.is_outdoors() {
            self.Overworld_DwDeathMountainPaletteAnimation();
        }
        self.RenderText();
        self.clear_hud_update_flag();
        self.clear_core_update_disable_flag();
        if self.game_state.frame.subsubmodule < 3 {
            self.increment_subsubmodule();
        } else {
            self.clear_bg_vram_load_mode();
        }
        if self.game_state.frame.submodule == 0 {
            self.set_subsubmodule(0);
            self.set_bg_vram_load_mode(1);
            if self.multiselect_choice().value() != 0 {
                self.set_ambient_sound_effect(15);
                self.set_main_module(23);
                self.set_submodule(1);
                self.dungeon_object_tracking_mut()
                    .clear_changeable_object_index(0);
                self.dungeon_object_tracking_mut()
                    .clear_changeable_object_index(1);
            } else {
                self.multiselect_choice_mut().restore_backup();
            }
        }
    }

    pub(super) fn Module1B_SpawnSelect(&mut self) {
        // The spawn-select prompt hosts the shared RenderText machinery; its
        // Text_Initialize is measured like Module0E's (route host 160304).
        self.arm_dialogue_initialization_schedule_if_needed();
        self.RenderText();
        if self.game_state.frame.submodule != 0 {
            return;
        }
        self.clear_bg_vram_load_mode();
        self.EnableForceBlank();
        self.EraseTileMaps_normal();
        let bak = self
            .game_state
            .inventory
            .save_progress
            .which_starting_point();
        let choice = self.multiselect_choice().value();
        self.save_progress_mut()
            .set_which_starting_point(LOCATION_MENU_START_POSITIONS[choice as usize]);
        self.set_subsubmodule(0);
        self.load_dungeon_room_rebuild_hud();
        self.save_progress_mut().set_which_starting_point(bak);
    }

    pub(super) fn CleanUpAndPrepDesertPrayerHDMA(&mut self) {
        self.hdma_setup(0, 0x02c80c, 0x41, 0, 0x26, 0);
        let main_layers = self.game_state.display.main_screen_layers;
        let sub_layers = self.game_state.display.sub_screen_layers;
        self.set_window_layer_masks(0x33, 3, 0x33, main_layers, sub_layers);
        self.set_hdma_enable_mask(0x80);
        self.clear_spotlight_hdma_table_dynamic(240);
    }

    pub(super) fn DesertPrayer_InitializeIrisHDMA(&mut self) {
        self.CleanUpAndPrepDesertPrayerHDMA();
        self.set_spotlight_window_radius_byte(0x26);
        self.set_spotlight_window_state_byte(0);
        self.DesertPrayer_BuildIrisHDMATable();
        self.increment_subsubmodule();
    }

    fn complete_desert_prayer_case2_after_iris(&mut self) {
        let countdown = self.game_state.display.mosaic_target_level.wrapping_sub(1);
        self.set_countdown(countdown);
        self.clear_mosaic_target_level();
        self.set_darkening_or_lightening_screen(2);
    }

    pub(super) fn begin_desert_prayer_iris(
        &mut self,
        progress: crate::DesertPrayerIrisProgress,
        caller: crate::zelda_rtl::DesertPrayerIrisCaller,
    ) {
        assert!(
            self.desert_prayer_build_iris_hdma_table(Some(progress)),
            "Desert Prayer iris builder did not reach source progress {progress:?}",
        );
        self.game_execution_scheduler.schedule_work(
            crate::zelda_rtl::GameWorkContinuation::FinishDesertPrayerIris { progress, caller },
            1,
        );
    }

    pub(super) fn complete_desert_prayer_iris(
        &mut self,
        caller: crate::zelda_rtl::DesertPrayerIrisCaller,
    ) {
        self.DesertPrayer_BuildIrisHDMATable();
        if caller == crate::zelda_rtl::DesertPrayerIrisCaller::InitializeCase2 {
            self.increment_subsubmodule();
            self.complete_desert_prayer_case2_after_iris();
        }
        self.complete_module0e_interface_after_run();
    }

    pub(super) fn desert_prayer_iris_completion_closes_dialogue(&self) -> bool {
        self.game_state.frame.subsubmodule == 4
            && self.game_state.display.spotlight_hdma.window_state_byte() != 0
            && self
                .game_state
                .display
                .spotlight_hdma
                .window_radius_byte()
                .wrapping_add(8)
                >= 0xc0
    }

    pub(super) fn complete_desert_prayer_palette_filter_before_iris(
        &mut self,
        countdown: u8,
        next_color: u8,
    ) {
        assert_eq!(
            self.game_state.display.palette_filter.countdown_word(),
            u16::from(countdown),
            "Desert Prayer palette continuation lost its source countdown",
        );
        self.complete_apply_palette_filter_bounce_from(next_color);
    }

    pub(super) fn complete_desert_prayer_palette_filter_after_iris(&mut self) {
        self.DesertPrayer_BuildIrisHDMATable();
        self.complete_module0e_interface_after_run();
    }

    pub(super) fn DesertPrayer_BuildIrisHDMATable(&mut self) {
        assert!(!self.desert_prayer_build_iris_hdma_table(None));
    }

    /// Execute the source builder through an optional persistent-statement
    /// checkpoint. Returning `true` means the named prefix was published and
    /// every later source statement remains pending.
    fn desert_prayer_build_iris_hdma_table(
        &mut self,
        stop_at: Option<crate::DesertPrayerIrisProgress>,
    ) -> bool {
        let r14 = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2())
            .wrapping_add(12);
        let radius = self.game_state.display.spotlight_hdma.window_radius_byte();
        let mut spotlight_y_lower = r14.wrapping_sub(u16::from(radius));
        if stop_at
            == Some(crate::DesertPrayerIrisProgress::Setup {
                completed_writes: 0,
            })
        {
            return true;
        }
        self.set_spotlight_y_lower(spotlight_y_lower);
        if stop_at
            == Some(crate::DesertPrayerIrisProgress::Setup {
                completed_writes: 1,
            })
        {
            return true;
        }
        let mut r4 = if sign16(spotlight_y_lower) {
            spotlight_y_lower
        } else {
            0
        };
        let spotlight_y_upper = spotlight_y_lower.wrapping_add(u16::from(radius) * 2);
        self.set_spotlight_y_upper(spotlight_y_upper);
        if stop_at
            == Some(crate::DesertPrayerIrisProgress::Setup {
                completed_writes: 2,
            })
        {
            return true;
        }
        let spotlight_x_center = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2())
            .wrapping_add(8);
        self.set_spotlight_window_x_center(spotlight_x_center);
        if stop_at
            == Some(crate::DesertPrayerIrisProgress::Setup {
                completed_writes: 3,
            })
        {
            return true;
        }
        self.set_spotlight_window_y_buffer_byte(1);
        if stop_at
            == Some(crate::DesertPrayerIrisProgress::Setup {
                completed_writes: 4,
            })
        {
            return true;
        }

        loop {
            if stop_at == Some(crate::DesertPrayerIrisProgress::BeforeIteration { scanline: r4 }) {
                return true;
            }
            let mut r0 = 0x0100u16;
            let mut r2 = 0x0100u16;
            let in_window =
                sign16(spotlight_y_lower) || (r4 >= spotlight_y_lower && r4 < spotlight_y_upper);
            let radius = self.game_state.display.spotlight_hdma.window_radius_byte();
            let y_buffer = self
                .game_state
                .display
                .spotlight_hdma
                .window_y_buffer_byte();
            let k = if !in_window {
                r4.wrapping_sub(1)
            } else if radius < y_buffer {
                self.set_spotlight_window_y_buffer_byte(1);
                spotlight_y_lower = 0;
                self.set_spotlight_y_lower(0);
                r4 = spotlight_y_upper;
                if r4 >= 225 {
                    break;
                }
                r4.wrapping_sub(1)
            } else {
                let pair = self.DesertHDMA_CalculateIrisShapeLine();
                if pair.a == 0 {
                    spotlight_y_lower = 0;
                    self.set_spotlight_y_lower(0);
                } else {
                    r2 = spotlight_x_center.wrapping_add(pair.b);
                    r0 = spotlight_x_center.wrapping_sub(pair.b);
                }
                r14.wrapping_sub(u16::from(
                    self.game_state
                        .display
                        .spotlight_hdma
                        .window_y_buffer_byte(),
                ))
                .wrapping_sub(1)
            };

            if stop_at
                == Some(crate::DesertPrayerIrisProgress::BeforePrimaryTableWrite {
                    table_word: k,
                    y_buffer,
                })
            {
                return true;
            }

            let t6 = if r0 < 256 {
                r0 as u8
            } else if r0 < 512 {
                255
            } else {
                0
            };
            let t7 = if r2 < 256 { r2 as u8 } else { 255 };
            let r6 = (u16::from(t7) << 8) | u16::from(t6);
            if k < 240 {
                self.set_spotlight_hdma_table_dynamic_entry(
                    k as usize,
                    if r6 == 0xffff { 0x00ff } else { r6 },
                );
            }
            if stop_at
                == Some(crate::DesertPrayerIrisProgress::AfterPrimaryTableWrite {
                    table_word: k,
                    y_buffer,
                })
            {
                return true;
            }

            if sign16(spotlight_y_lower) || (r4 >= spotlight_y_lower && r4 < spotlight_y_upper) {
                let k = u16::from(
                    self.game_state
                        .display
                        .spotlight_hdma
                        .window_y_buffer_byte(),
                )
                .wrapping_sub(2)
                .wrapping_add(r14);
                if k < 240 {
                    self.set_spotlight_hdma_table_dynamic_entry(
                        k as usize,
                        if r6 == 0xffff { 0x00ff } else { r6 },
                    );
                }
                self.increment_spotlight_window_y_buffer_byte();
            }

            r4 = r4.wrapping_add(1);
            if stop_at
                == Some(crate::DesertPrayerIrisProgress::AfterIteration {
                    next_scanline: r4,
                    y_buffer: self
                        .game_state
                        .display
                        .spotlight_hdma
                        .window_y_buffer_byte(),
                })
            {
                return true;
            }
            if !sign16(r4) && r4 >= 225 {
                break;
            }
        }

        if stop_at == Some(crate::DesertPrayerIrisProgress::LoopComplete) {
            return true;
        }

        if self.game_state.frame.subsubmodule != 4 {
            return false;
        }
        if self.game_state.display.spotlight_hdma.window_state_byte() != 1
            && (self.game_state.player.follower_link.filtered_joypad_h()
                | self.game_state.player.follower_link.filtered_joypad_l())
                & 0xc0
                != 0
        {
            self.set_spotlight_window_state_byte(1);
            self.shr_spotlight_window_radius_byte(1);
        }
        if self.game_state.display.spotlight_hdma.window_state_byte() != 0 {
            self.add_spotlight_window_radius_byte(8);
            if self.game_state.display.spotlight_hdma.window_radius_byte() >= 0xc0 {
                self.messaging_state_mut()
                    .xor_message_or_sprite_state_cache(1);
                self.set_music_control(0xf3);
                self.set_ambient_sound_effect(0);
                self.clear_modal_pause_flag();
                self.follower_link_state_mut().set_y_button_action_step(0);
                self.follower_link_state_mut().set_button_mask_b_y(0);
                self.follower_link_state_mut().clear_state_bits();
                self.follower_link_state_mut().clear_direction_lock_bits(1);
                self.set_subsubmodule(0);
                self.set_submodule(0);
                let saved_module = self.game_state.frame.saved_module_for_menu;
                self.set_main_module(saved_module);
                self.clear_window_layer_masks();
                self.IrisSpotlight_ResetTable();
                return false;
            }
        }
        if (self
            .follower_link_state_mut()
            .decrement_spin_attack_delay_timer() as i8)
            .is_negative()
        {
            let i = self
                .game_state
                .player
                .follower_link
                .y_button_action_step()
                .wrapping_add(1);
            if i != 4 {
                self.follower_link_state_mut().set_y_button_action_step(i);
            }
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(PRAYING_SCENE_DELAYS[i as usize]);
        }
        false
    }

    pub(super) fn DesertHDMA_CalculateIrisShapeLine(&self) -> Pair16U {
        let y_buffer = self
            .game_state
            .display
            .spotlight_hdma
            .window_y_buffer_byte();
        let radius = self
            .game_state
            .display
            .spotlight_hdma
            .window_radius_byte()
            .max(1);
        let t = (self.snes_divide(u16::from(y_buffer) << 8, radius) >> 1) as usize;
        let r6 = if self.game_state.display.spotlight_hdma.window_state_byte() != 0 {
            PRAYING_IRIS_OPEN_RADIUS_LOOKUP[t.min(128)]
        } else {
            PRAYING_IRIS_CLOSED_RADIUS_LOOKUP[t.min(128)]
        };
        let mut r8 = (u16::from(r6) * u16::from(radius)) >> 8;
        if self.game_state.display.spotlight_hdma.window_state_byte() != 0 {
            r8 <<= 1;
        }
        Pair16U {
            a: u16::from(r6),
            b: r8,
        }
    }

    pub(super) fn OverworldMap_SetupHdma(&mut self) {
        let addr = PRAYING_IRIS_HDMA_SOURCE_ADDRS[self.overworld_map_flags() as usize];
        self.hdma_setup(addr, addr, 0x42, 0x1b, 0x1e, 10);
    }

    pub(super) fn SaveGameFile(&mut self) {
        let accumulated_word_sum = self.save_game_file_copy_blocks_and_sum_prefix(0);
        self.save_game_file_finish_checksum(0, accumulated_word_sum);
    }

    /// Copy the two source-ordered SRAM mirrors, then accumulate exactly the
    /// requested prefix of the live WRAM checksum input.
    ///
    /// The ROM finishes both copies before entering its checksum loop. Keeping
    /// the sum outside WRAM lets a suspended native caller retain the same
    /// source value while an intervening NMI may mutate later save-block words.
    pub(super) fn save_game_file_copy_blocks_and_sum_prefix(
        &mut self,
        completed_checksum_words: u16,
    ) -> u16 {
        assert!(
            completed_checksum_words <= 0x4fe / 2,
            "SaveGameFile checksum prefix exceeds the source save block",
        );
        let offs = self.selected_save_slot_offset();
        // C copies the LIVE save block from WRAM (ram[SAVE_DUNG_INFO..+0x500]) and
        // checksums it from WRAM. The SaveProgress native model keeps a `dungeon_info`
        // shadow of this block, but that shadow goes stale for bytes owned by OTHER
        // native states (e.g. LINK_HEALTH_CURRENT 0xf36d, owned by player_resources,
        // whose set_current_health write-throughs ram but not this shadow). Reading the
        // shadow saved a stale health to SRAM (and a checksum over the stale block),
        // surfacing on the next LoadFile. Mirror C: copy + checksum from live ram.
        let dung_info = self.ram[SAVE_DUNG_INFO..SAVE_DUNG_INFO + 0x500].to_vec();
        if offs + 0x500 <= self.sram.len() {
            self.sram[offs..offs + 0x500].copy_from_slice(&dung_info);
        }
        if offs + 0xf00 + 0x500 <= self.sram.len() {
            self.sram[offs + 0xf00..offs + 0xf00 + 0x500].copy_from_slice(&dung_info);
        }

        let mut accumulated_word_sum = 0u16;
        for word in 0..usize::from(completed_checksum_words) {
            accumulated_word_sum = accumulated_word_sum
                .wrapping_add(read_le_u16(&self.ram, SAVE_DUNG_INFO + word * 2));
        }
        accumulated_word_sum
    }

    /// Resume `SaveGameFile` after a source checksum-loop boundary.
    pub(super) fn save_game_file_finish_checksum(
        &mut self,
        completed_checksum_words: u16,
        mut accumulated_word_sum: u16,
    ) {
        let total_checksum_words = 0x4fe / 2;
        assert!(
            completed_checksum_words <= total_checksum_words,
            "SaveGameFile checksum continuation exceeds the source save block",
        );
        for word in usize::from(completed_checksum_words)..usize::from(total_checksum_words) {
            accumulated_word_sum = accumulated_word_sum
                .wrapping_add(read_le_u16(&self.ram, SAVE_DUNG_INFO + word * 2));
        }
        let checksum = 0x5a5au16.wrapping_sub(accumulated_word_sum);
        let offs = self.selected_save_slot_offset();
        // Keep the shadow + ram[SAVE_DUNG_INFO+0x4fe] coherent so the frame-end bulk
        // projection of dungeon_info doesn't re-stamp a stale checksum over ram.
        self.save_progress_mut().set_dungeon_info_checksum(checksum);
        if offs + 0x500 <= self.sram.len() {
            write_le_u16(&mut self.sram, offs + 0x4fe, checksum);
        }
        if offs + 0xf00 + 0x500 <= self.sram.len() {
            write_le_u16(&mut self.sram, offs + 0x4fe + 0xf00, checksum);
        }
        self.zelda_write_sram();
    }

    pub(super) fn TransferMode7Characters(&mut self) {
        self.transfer_mode7_characters();
    }

    pub(super) fn Animate_GAMEOVER_Letters(&mut self) {
        match self.ancilla_slot_view(0).ancilla_type() {
            0 => self.increment_submodule(),
            1 => self.GameOverText_SweepLeft(),
            2 => self.GameOverText_UnfurlRight(),
            3 => self.GameOverText_Draw(),
            _ => {}
        }
    }

    pub(super) fn GameOverText_SweepLeft(&mut self) {
        let mut k = self.game_state.minigame.flag_boomerang_in_place() as usize;
        self.sprite_system_mut().set_cur_object_index(k as u8);
        self.ancilla_slot_view_mut(k).set_x_velocity(0x80);
        self.ancilla_move_x(k);
        if self.ancilla_get_x(k) < u16::from(GAME_OVER_SWEEP_LEFT_X_TARGETS[k]) {
            self.ancilla_slot_view_mut(k)
                .set_x_low(GAME_OVER_SWEEP_LEFT_X_TARGETS[k]);
            k += 1;
            self.minigame_state_mut()
                .set_flag_boomerang_in_place(k as u8);
            if k == 8 {
                self.minigame_state_mut().set_flag_boomerang_in_place(7);
                self.ancilla_slot_view_mut(0).increment_ancilla_type();
                self.messaging_state_mut().clear_game_over_letter_cursor();
                self.set_sound_effect_2(38);
                self.GameOverText_Draw();
                return;
            }
        }
        if k == 7 {
            let mut j = 6i32;
            let x = self.ancilla_slot_view(k).x_low();
            while j != i32::from(self.game_state.messaging.runtime.game_over_letter_cursor()) {
                self.ancilla_slot_view_mut(j as usize).set_x_low(x);
                j -= 1;
            }
            let hookshot = self.game_state.messaging.runtime.game_over_letter_cursor() as usize;
            if self.ancilla_get_x(k) < u16::from(GAME_OVER_SWEEP_LEFT_X_TARGETS[hookshot]) {
                self.messaging_state_mut()
                    .decrement_game_over_letter_cursor();
            }
        }
        self.GameOverText_Draw();
    }

    pub(super) fn GameOverText_UnfurlRight(&mut self) {
        let mut k = self.game_state.minigame.flag_boomerang_in_place() as usize;
        self.sprite_system_mut().set_cur_object_index(k as u8);
        self.ancilla_slot_view_mut(k).set_x_velocity(0x60);
        self.ancilla_move_x(k);
        let j = self.game_state.messaging.runtime.game_over_letter_cursor() as usize;
        if self.ancilla_slot_view(k).x() >= u16::from(GAME_OVER_UNFURL_RIGHT_X_TARGETS[j]) {
            self.ancilla_slot_view_mut(j)
                .set_x_low(GAME_OVER_UNFURL_RIGHT_X_TARGETS[j]);
            self.messaging_state_mut()
                .increment_game_over_letter_cursor();
            if self.game_state.messaging.runtime.game_over_letter_cursor() == 8 {
                self.increment_submodule();
                self.ancilla_slot_view_mut(0).increment_ancilla_type();
                self.GameOverText_Draw();
                return;
            }
        }
        let end =
            i32::from(self.game_state.messaging.runtime.game_over_letter_cursor()).wrapping_sub(1);
        k = self.game_state.minigame.flag_boomerang_in_place() as usize;
        let mut j = k as i32;
        let x = self.ancilla_slot_view(k).x_low();
        loop {
            self.ancilla_slot_view_mut(j as usize).set_x_low(x);
            j -= 1;
            if j == end {
                break;
            }
        }
        self.GameOverText_Draw();
    }

    pub(super) fn GameOverText_Draw(&mut self) {
        // ROM $08:F5C4 is the OAM author for the animated GAME OVER letters.
        // The similarly numbered NMI request uploads OBJ character data; it is
        // not this routine. Keeping the two operations separate matters once
        // the initial character upload has completed and the letters move on
        // every subsequent main-thread iteration.
        self.game_over_text_draw();
    }

    pub(super) fn Module12_GameOver(&mut self) {
        let entry_submodule = self.game_state.frame.submodule;
        let spotlight_radius = self.game_state.display.spotlight_hdma.window_radius();
        let live_goal_palette_fill =
            match self.original_timing_main_loop_interruption() {
                Some(crate::MainLoopInterruption::GameOverIrisGoalPaletteFill {
                    completed_stores,
                }) if entry_submodule == 3 => Some(completed_stores),
                _ => None,
            };
        let live_table_progress = matches!(entry_submodule, 2 | 3)
            .then(|| self.take_original_timing_spotlight_table_build_progress())
            .flatten();
        let game_over_spotlight_entry_suspended = entry_submodule == 2
            && live_table_progress
                .is_some_and(|progress| self.begin_game_over_spotlight_entry(progress.progress));
        let game_over_goal_palette_suspended = live_goal_palette_fill.is_some_and(|completed| {
            self.begin_game_over_iris_goal_palette_fill(spotlight_radius, completed)
        });
        let game_over_spotlight_build_suspended = entry_submodule == 3
            && !game_over_goal_palette_suspended
            && self.begin_game_over_spotlight_build(
                spotlight_radius,
                live_table_progress.map(|progress| progress.progress),
            );
        match entry_submodule {
            0 => self.GameOver_AdvanceImmediately(),
            1 => self.Death_Func1(),
            2 if !game_over_spotlight_entry_suspended => self.GameOver_DelayBeforeIris(),
            2 => {}
            3 if !game_over_spotlight_build_suspended && !game_over_goal_palette_suspended => {
                self.GameOver_IrisWipe()
            }
            3 => {}
            4 => self.Death_Func4(),
            5 => self.GameOver_SplatAndFade(),
            6 => self.Death_Func6(),
            7 => self.Animate_GAMEOVER_Letters_bounce(),
            8 => self.GameOver_Finalize_GAMEOVR(),
            9 => self.GameOver_SaveAndOrContinue(),
            10 => self.GameOver_InitializeRevivalFairy(),
            11 => self.RevivalFairy_Main_bounce(),
            12 => self.GameOver_RiseALittle(),
            13 => self.GameOver_Restore0D(),
            14 => self.GameOver_Restore0E(),
            15 => self.GameOver_ResituateLink(),
            _ => {}
        }
        let spotlight_suspended = game_over_spotlight_entry_suspended
            || game_over_spotlight_build_suspended
            || game_over_goal_palette_suspended;
        if self.game_state.frame.submodule != 9 && !spotlight_suspended {
            self.link_oam_main();
        }
        let link_oam_suspended = self
            .take_forwarded_original_timing_main_loop_interruption(
                crate::MainLoopInterruption::LinkOam,
            )
            .is_some();
        // Consuming the wire's LinkOam interruption here (route host 150252)
        // leaves the shared ZeldaRunGameLoop suffix to the generic body: the
        // translated LinkOam_Main call already completed atomically, and the
        // body arms the pending suffix for the resumed stack's next host.
        let _ = link_oam_suspended;
    }

    fn begin_game_over_spotlight_entry(
        &mut self,
        progress: crate::SpotlightTableBuildProgress,
    ) -> bool {
        self.messaging_state_mut().decrement_menu_animation_timer();
        assert_eq!(
            self.game_state.messaging.runtime.menu_animation_timer(),
            0,
            "a game-over entry table checkpoint requires the delay timer to expire",
        );
        self.Death_InitializeGameOverLetters();
        self.spotlight_internal_before_table(0x7e, 0);
        let table_build = self.begin_iris_spotlight_configure_table_at_progress(progress);
        let iteration = SpotlightIteration::game_over_closing(
            SpotlightIterationPhase::CloseEntryBeforeTablePublication,
            true,
        );
        self.schedule_game_over_spotlight_build(table_build, true, iteration);
        true
    }

    fn begin_game_over_spotlight_build(
        &mut self,
        spotlight_radius: u16,
        live_progress: Option<crate::SpotlightTableBuildProgress>,
    ) -> bool {
        if !self.rom_startup_timing() {
            return false;
        }
        if spotlight_radius == 7
            && live_progress.is_none()
            && self
                .game_execution_scheduler
                .current_main_iteration_follows_leading_nmi()
        {
            // At the final radius the complete table, radius-zero goal
            // transition, and Module-12 restore all fit between the leading
            // Open NMI and the following Held NMI. The latter lands at
            // $09:f35d, before the first palette store, so retain an exact
            // zero-store palette continuation instead of scheduling another
            // table-build slice.
            return self.begin_game_over_iris_goal_palette_fill(spotlight_radius, 0);
        }
        self.game_over_iris_wipe_before_table();
        let phase = SpotlightIterationPhase::for_game_over_close_iteration(spotlight_radius);
        let table_build = match live_progress {
            Some(progress) => self.begin_iris_spotlight_configure_table_at_progress(progress),
            None => self.begin_iris_spotlight_configure_table(0),
        };
        self.schedule_game_over_spotlight_build(
            table_build,
            false,
            SpotlightIteration::game_over_closing(phase, false),
        );
        true
    }

    fn begin_game_over_iris_goal_palette_fill(
        &mut self,
        spotlight_radius: u16,
        completed_stores: u8,
    ) -> bool {
        assert_eq!(
            spotlight_radius, 7,
            "the game-over goal palette fill requires the final closing-iris radius",
        );
        assert!(completed_stores <= 96);
        self.game_over_iris_wipe_before_table();
        let table_build = self.begin_iris_spotlight_configure_table(usize::MAX);
        self.complete_iris_spotlight_table_projection(table_build);
        assert!(
            self.complete_iris_spotlight_configure_table_after_projection_deferring_goal(true),
            "the game-over palette interruption requires the closing iris to reach radius zero",
        );
        self.complete_iris_spotlight_goal_transition();
        self.set_main_module(0x12);
        self.game_over_iris_goal_palette_stores(0, usize::from(completed_stores));
        self.game_execution_scheduler.schedule_work(
            crate::zelda_rtl::GameWorkContinuation::FinishGameOverIrisGoalPaletteFill {
                completed_stores,
            },
            1,
        );
        true
    }

    pub(super) fn complete_game_over_spotlight_build(
        &mut self,
        table_build: crate::zelda_rtl::SpotlightTableBuildContinuation,
        entry: bool,
        iteration: SpotlightIteration,
        wire_defers_caller_return: bool,
        wire_completes_caller_return: bool,
    ) {
        if entry {
            self.complete_iris_spotlight_configure_table(table_build);
            self.spotlight_internal_after_table();
            self.set_object_color_window_selection(0x30);
            self.set_bg34_window_selection(0);
            self.increment_submodule();
        } else {
            let main_module = self.game_state.frame.main_module;
            self.complete_iris_spotlight_configure_table(table_build);
            self.game_over_iris_wipe_after_completed_table(main_module);
        }
        self.link_oam_main();
        // The live wire decides whether the caller return crosses vblank: a
        // whole-table build whose LinkOam_Main the ROM's held NMI still
        // interrupted returns on the next host (route host 589119:
        // [NmiAccepted(LatchHeld), NmiHandlerCompleted, NmiAccepted(LatchHeld),
        // MainLoopInterrupted(LinkOam), CallStackContinued]); a terminal
        // return in this host proves the caller and its suffix completed here
        // even when the estimate wanted another slice (route host 943804).
        if (iteration.game_over_build_needs_deferred_caller_return() || wire_defers_caller_return)
            && !wire_completes_caller_return
        {
            self.schedule_spotlight_iteration_return(iteration.after_game_over_build());
        } else {
            // Once the shrinking table finishes before vblank, its caller and
            // normal game-loop suffix also return in this host slice. Folding
            // that suffix here lets the next host begin a fresh main iteration
            // instead of spending a synthetic third frame returning again.
            if self.pending_main_loop_common_suffix.is_some() {
                // The suspended iteration's shared suffix stayed pending
                // while the build was scheduled; retire its one owner (route
                // host 150261).
                self.complete_pending_main_loop_common_suffix_after_module_return();
            } else {
                self.nmi_prepare_sprites();
                self.clear_nmi_update_latch();
            }
            self.next_display_obj_scanout_generation = Some(ObjScanoutGenerations::coherent(
                GraphicsDmaGeneration::HostBoundaryBeforeMain,
            ));
        }
    }

    pub(super) fn GameOver_AdvanceImmediately(&mut self) {
        self.increment_submodule();
        self.Death_Func1();
    }

    pub(super) fn Death_Func1(&mut self) {
        let current_music = self.game_state.system_signals.current_music_control();
        let ambient_sound = self.game_state.system_signals.last_ambient_sound_effect();
        self.set_death_backup_current_music(current_music);
        self.set_death_backup_ambient_sound(ambient_sound);
        self.set_music_control(0xf1);
        self.set_ambient_sound_effect(5);
        self.set_overworld_map_state(5);
        self.follower_link_state_mut().clear_conveyor_belt_state();
        self.tile_detect_position_mut().set_layer_collision_flags(0);
        self.follower_link_state_mut().clear_cape_mode();
        let palette_filter_countdown = self.game_state.display.palette_filter.countdown_word();
        let darkening_or_lightening_screen = self
            .game_state
            .display
            .palette_filter
            .darkening_or_lightening_screen_word();
        self.set_mapbak_bg1_x_offset(palette_filter_countdown);
        self.set_mapbak_bg1_y_offset(darkening_or_lightening_screen);
        let palette = self
            .game_state
            .display
            .palette_buffer
            .aux_visible_slice()
            .to_vec();
        self.copy_mapbak_palette_from(
            &palette[..palette.len().min(256)],
            crate::game_state::PaletteSliceSource::MirrorBank(zelda3_palette::Bank::Aux),
        );
        self.clear_aux_visible_subpalettes();
        self.set_countdown_word(0);
        self.set_darkening_or_lightening_screen_word(0);
        self.set_bg1_x_offset(0);
        self.set_bg1_y_offset(0);
        let cgwsel = self
            .game_state
            .display
            .palette_filter
            .color_window_and_math_word();
        self.set_mapbak_cgwsel_word(cgwsel);
        self.messaging_state_mut().set_menu_animation_timer(32);
        self.clear_floor_changed_timer_low();
        self.hud_floor_indicator();
        self.increment_hud_update_flag();
        self.set_ambient_sound_effect(5);
        self.increment_submodule();
    }

    pub(super) fn GameOver_DelayBeforeIris(&mut self) {
        self.messaging_state_mut().decrement_menu_animation_timer();
        if self.game_state.messaging.runtime.menu_animation_timer() != 0 {
            return;
        }
        self.Death_InitializeGameOverLetters();
        self.IrisSpotlight_close();
        self.set_object_color_window_selection(0x30);
        self.set_bg34_window_selection(0);
        self.increment_submodule();
    }

    pub(super) fn GameOver_IrisWipe(&mut self) {
        self.game_over_iris_wipe_before_table();
        self.game_over_iris_wipe_after_table();
    }

    fn game_over_iris_wipe_before_table(&mut self) {
        self.PaletteFilter_RestoreBGSubstractiveStrict();
        self.copy_color(
            (zelda3_palette::Bank::Main, 32),
            (zelda3_palette::Bank::Main, 0),
        );
    }

    fn game_over_iris_wipe_after_table(&mut self) {
        let bak = self.game_state.frame.main_module;
        self.IrisSpotlight_ConfigureTable();
        self.game_over_iris_wipe_after_completed_table(bak);
    }

    fn game_over_iris_wipe_after_completed_table(&mut self, bak: u8) {
        self.set_main_module(bak);
        if self.game_state.frame.submodule != 0 {
            return;
        }
        self.game_over_iris_goal_palette_stores(0, 96);
        self.game_over_iris_goal_after_palette_fill();
    }

    fn game_over_iris_goal_palette_stores(&mut self, start: usize, end: usize) {
        debug_assert!(start <= end && end <= 96);
        let bases = [0x20usize, 0x30, 0x40, 0x50, 0x60, 0x70];
        for store in start..end {
            let color = store / bases.len();
            let base = bases[store % bases.len()];
            self.set_main_color_constant(base + color, 0x18);
        }
    }

    fn game_over_iris_goal_after_palette_fill(&mut self) {
        self.set_main_color_constant(0, 0x18);
        self.set_main_color_constant(32, 0x18);
        self.IrisSpotlight_ResetTable();
        self.set_fixed_color_red(32);
        self.set_fixed_color_green(64);
        self.set_fixed_color_blue(128);
        self.set_bg12_window_selection(0);
        self.set_bg34_window_selection(0);
        self.set_object_color_window_selection(0);
        self.set_submodule(4);
        // The table reached radius zero after active scanout began. Preserve
        // that fully closed (brightness-zero) image while the live submodule-4
        // initializer prepares the following visible generation.
        self.game_over_iris_goal_scanout_closed_pending = true;
        self.increment_cgram_update_flag();
        self.set_screen_brightness(15);
        self.set_main_screen_layers(20);
        self.set_sub_screen_layers(0);
        self.set_color_math_control(32);
        self.messaging_state_mut().set_menu_animation_timer(64);
        self.set_countdown(0);
        self.set_darkening_or_lightening_screen(0);
        self.Death_PrepFaint();
    }

    pub(super) fn complete_game_over_iris_goal_palette_fill(&mut self, completed_stores: u8) {
        self.game_over_iris_goal_palette_stores(usize::from(completed_stores), 96);
        self.game_over_iris_goal_after_palette_fill();
        self.link_oam_main();
    }

    pub(super) fn GameOver_SplatAndFade(&mut self) {
        if self.game_state.messaging.runtime.menu_animation_timer() != 0 {
            self.messaging_state_mut().decrement_menu_animation_timer();
            return;
        }
        self.PaletteFilter_RestoreBGSubstractiveStrict();
        self.copy_color(
            (zelda3_palette::Bank::Main, 32),
            (zelda3_palette::Bank::Main, 0),
        );
        if self
            .game_state
            .display
            .palette_filter
            .darkening_or_lightening_screen()
            != 0xff
        {
            return;
        }
        self.clear_mosaic_level();
        self.clear_mosaic_direction();
        self.set_mosaic_copy(3);
        for i in 0..4 {
            if self.game_state.inventory.items.bottle(i) == 6 {
                let value = 2;
                self.inventory_items_mut().set_bottle(i, value);
                self.messaging_state_mut().set_menu_animation_timer(12);
                self.set_chr_halfslot_request(15);
                self.Graphics_LoadChrHalfSlot();
                self.clear_chr_halfslot_request();
                self.set_submodule(10);
                return;
            }
        }
        self.dungeon_object_tracking_mut()
            .clear_changeable_object_index(0);
        self.dungeon_object_tracking_mut()
            .clear_changeable_object_index(1);
        self.set_pending_nmi_subroutine(22);
        self.set_core_update_disable_flag(22);
        self.increment_submodule();
    }

    pub(super) fn Death_Func6(&mut self) {
        self.messaging_state_mut().set_menu_animation_timer(12);
        self.set_chr_halfslot_request(15);
        self.Graphics_LoadChrHalfSlot();
        self.clear_chr_halfslot_request();
        self.set_sp6r_indoors(5);
        self.select_overworld_aux_palette_offset();
        self.Palette_Load_SpriteEnvironment_Dungeon();
        self.Palette_Load_SpriteMain();
        self.increment_cgram_update_flag();
        self.increment_submodule();
        self.Death_PlayerSwoon();
    }

    pub(super) fn Death_Func4(&mut self) {
        self.Death_PlayerSwoon();
    }

    pub(super) fn Animate_GAMEOVER_Letters_bounce(&mut self) {
        self.Animate_GAMEOVER_Letters();
    }

    pub(super) fn GameOver_Finalize_GAMEOVR(&mut self) {
        if self.rom_startup_timing() {
            self.prepare_game_over_text_oam();
            if self.game_over_text_render_calls_remaining == 0
                && !self.game_over_text_render_call_in_flight
            {
                self.messaging_state_mut().set_module(2);
                self.dialogue_message_index_mut().set_value(3);
                self.Text_Initialize_initModuleStateLoop();
                self.messaging_state_mut().set_text_msgbox_topleft(0x61e8);
                self.messaging_state_mut().set_text_render_state(2);
                self.game_over_text_render_calls_remaining = 5;
            }
            self.resume_game_over_text_render_loop();
            return;
        }
        self.Animate_GAMEOVER_Letters();
        let bak1 = self.game_state.frame.main_module;
        let bak2 = self.game_state.frame.submodule;
        self.messaging_state_mut().set_module(2);
        self.RenderText();
        self.set_submodule(bak2.wrapping_add(1));
        self.set_main_module(bak1);
        self.messaging_state_mut().set_menu_animation_timer(2);
        self.set_music_control(11);
    }

    pub(crate) fn prepare_game_over_text_oam(&mut self) {
        // The ordinary main-loop prefix hides the prior gameplay table before
        // the post-death dialogue call begins. The translated VWF hold can
        // enter with its generic core-hold flag already set, so make this
        // program-counter boundary explicit and preserve it on every resumed
        // CPU slice. Then author only the sixteen GAME OVER letter sprites,
        // exactly as the suspended ROM stack has done by its first vblank.
        self.clear_oam_buffer();
        self.Animate_GAMEOVER_Letters();
    }

    pub(crate) fn game_over_text_render_loop_active(&self) -> bool {
        self.game_over_text_render_calls_remaining != 0 || self.game_over_text_render_call_in_flight
    }

    /// Resume the exact five-call loop in `RenderText_PostDeathSaveOptions`.
    /// A call is retired only after its VWF handler and caller suffix return;
    /// a vblank-interrupted call remains the same outer-loop iteration.
    pub(crate) fn resume_game_over_text_render_loop(&mut self) {
        while self.game_over_text_render_calls_remaining != 0 {
            let resumed_interrupted_call = self.game_over_text_render_call_in_flight;
            self.game_over_text_render_call_in_flight = true;
            self.Text_Render();
            if self.dialogue_fast_forward_hold_pending
                || !self.dialogue_scroll_cpu_is_idle()
                || self
                    .pre_main_caller_continuation_is(PreMainCallerContinuation::DialogueVwfReturn)
            {
                return;
            }
            self.finish_game_over_text_render_call();
            if resumed_interrupted_call && self.game_over_text_render_calls_remaining != 0 {
                // The VWF handler returned near the end of the CPU slice that
                // resumed it. Until dialogue CPU accounting becomes a shared
                // call-stack budget, do not grant the next outer invocation a
                // fresh full-frame budget on this same host call.
                self.dialogue_fast_forward_hold_pending = true;
                return;
            }
        }
    }

    pub(crate) fn finish_game_over_text_render_call(&mut self) {
        debug_assert!(self.game_over_text_render_call_in_flight);
        debug_assert_ne!(self.game_over_text_render_calls_remaining, 0);
        self.game_over_text_render_call_in_flight = false;
        self.game_over_text_render_calls_remaining -= 1;
        if self.game_over_text_render_calls_remaining == 0 {
            self.increment_submodule();
            self.messaging_state_mut().set_menu_animation_timer(2);
            self.set_music_control(11);
        }
    }

    pub(super) fn GameOver_SaveAndOrContinue(&mut self) {
        self.GameOver_AnimateChoiceFairy();
        self.Animate_GAMEOVER_Letters();

        if self.game_state.player.follower_link.filtered_joypad_h() & 0x20 != 0 {
            self.increment_subsubmodule();
            if self.game_state.frame.subsubmodule >= 3 {
                self.set_subsubmodule(0);
            }
            self.messaging_state_mut().set_menu_animation_timer(12);
            self.set_sound_effect_2(32);
        } else {
            self.messaging_state_mut().decrement_menu_animation_timer();
            if self.game_state.messaging.runtime.menu_animation_timer() == 0 {
                self.messaging_state_mut().set_menu_animation_timer(1);
                if self.game_state.player.follower_link.joypad1h_last() & 12 != 0 {
                    if self.game_state.player.follower_link.joypad1h_last() & 4 != 0 {
                        self.increment_subsubmodule();
                        if self.game_state.frame.subsubmodule >= 3 {
                            self.set_subsubmodule(0);
                        }
                    } else {
                        self.decrement_subsubmodule();
                        if (self.game_state.frame.subsubmodule as i8).is_negative() {
                            self.set_subsubmodule(2);
                        }
                    }
                    self.messaging_state_mut().set_menu_animation_timer(12);
                    self.set_sound_effect_2(32);
                }
            }
        }

        if ((self.game_state.player.follower_link.filtered_joypad_l() & 0xc0)
            | self.game_state.player.follower_link.filtered_joypad_h())
            & 0xd0
            == 0
        {
            return;
        }
        self.set_sound_effect_1(44);
        self.Death_Func15(self.game_state.frame.subsubmodule != 2);
    }

    pub(super) fn Death_Func15(&mut self, count_as_death: bool) {
        self.death_func15_common_prefix(count_as_death);
        self.death_func15_after_common_prefix();
    }

    /// `Death_Func15`'s shared fast prefix through `Sprite_ResetAll` and the
    /// death counters — the ROM completes this before the save-quit reset
    /// hold begins (route host 159333).
    pub(super) fn death_func15_common_prefix(&mut self, count_as_death: bool) {
        self.set_music_control(0xf1);
        if self.game_state.world.location.is_indoors() {
            self.Dungeon_FlagRoomData_Quadrants();
        }
        self.AdjustLinkBunnyStatus();
        if self.game_state.inventory.save_progress.progress_indicator() < 3 {
            self.save_progress_mut().set_dark_world_state(0);
            if !self.game_state.inventory.items.has_moon_pearl() {
                self.ForceNonbunnyStatus();
            }
        }
        if self.game_state.world.location.dungeon_room() == 0 {
            self.set_indoor_flag(0);
        }

        self.reset_some_things_after_death(self.game_state.world.location.dungeon_room() as u8);
        if matches!(
            self.game_state.sprites.follower_runtime.indicator(),
            6 | 9 | 10 | 13
        ) {
            self.follower_state_mut().set_indicator(0);
        }

        let health = POST_DEATH_HEALTH_BY_CAPACITY
            [(self.game_state.inventory.player_resources.health_capacity() >> 3) as usize];
        self.set_restart_check_flag(health);
        self.player_resources_mut().set_current_health(health);
        let palace = self.game_state.inventory.save_progress.palace_index_x2();
        if palace != 0xff {
            let slot = if palace == 2 { 0 } else { palace } >> 1;
            let keys = self.game_state.inventory.player_resources.keys();
            self.dungeon_key_slots_mut()
                .set_keys_earned_slot(slot as usize, keys);
        }
        self.sprite_reset_all();
        if self
            .game_state
            .inventory
            .save_progress
            .total_death_save_counter_is_uninitialized()
            && (!self.game_state.enhanced_features.has(4096) || count_as_death)
        {
            self.save_progress_mut()
                .increment_pending_death_save_counter();
        }
        self.increment_game_over_check_flag();
    }

    fn death_func15_after_common_prefix(&mut self) {
        if self.game_state.frame.subsubmodule != 1 {
            if self.game_state.world.location.is_indoors() {
                if self.game_state.sprites.follower_runtime.indicator() != 1
                    && self.game_state.inventory.save_progress.palace_index_x2() != 0xff
                {
                    self.clear_restart_check_flag();
                } else {
                    self.set_queued_music_control(0);
                    self.set_indoor_flag(0);
                    if self.game_state.inventory.save_progress.dark_world_state() != 0 {
                        self.set_dungeon_room(32);
                    }
                }
            } else if self.game_state.inventory.save_progress.dark_world_state() != 0 {
                self.set_dungeon_room(32);
            }

            if self.game_state.inventory.save_progress.progress_indicator() != 0 {
                if self.game_state.frame.subsubmodule == 0 {
                    self.SaveGameFile();
                }
                self.set_main_module(5);
                self.set_submodule(0);
                self.clear_bg_vram_load_mode();
            } else {
                let offset = self.selected_save_slot_source_offset();
                self.save_load_scratch_mut().set_source_offset(offset);
                self.clear_game_over_check_flag();
                self.CopySaveToWRAM();
            }
        } else {
            self.death_func15_save_quit_pre_hold();
            self.death_func15_save_quit_post_hold();
        }
    }

    /// The fast statements of `Death_Func15`'s save-quit branch which the ROM
    /// completes before its long reset hold begins (route host 159333: the
    /// oracle's sprite slots are already cleared at the hold's first
    /// boundary while the module byte and scroll registers are not).
    pub(super) fn death_func15_save_quit_pre_hold(&mut self) {
        if self.game_state.inventory.save_progress.progress_indicator() != 0 {
            self.SaveGameFile();
        }
        self.set_main_screen_layers(16);
        self.set_indoor_flag(0);
    }

    /// The slow remainder of the save-quit branch: `Death_Func31` (whose
    /// `Intro_InitializeMemory_darken` WRAM clear plus the overworld
    /// song-bank upload hold the wire for tens of slices) and the register
    /// resets the ROM performs only after that hold completes.
    pub(super) fn death_func15_save_quit_post_hold(&mut self) {
        self.death_func15_save_quit_reset_writes();
        self.death_func15_save_quit_song_upload();
    }

    /// The reset's WRAM-observable writes (`Death_Func31`'s module/scroll
    /// stores and the intro WRAM clear) — the oracle completes these as the
    /// reset's NMI masking begins, before the song upload (route run
    /// 159378).
    pub(super) fn death_func15_save_quit_reset_writes(&mut self) {
        self.death_func15_save_quit_reset_state_before_dungeon_info_clear();
        self.death_func15_save_quit_finish_dungeon_info_clear();
    }

    /// The save-quit reset prefix through the last scroll store immediately
    /// before the source begins clearing `save_dung_info`.
    pub(super) fn death_func15_save_quit_reset_state_before_dungeon_info_clear(&mut self) {
        self.death_func31();
        self.clear_restart_check_flag();
        self.clear_game_over_check_flag();
        self.set_queued_music_control(0);
        self.set_bg1_x(0);
        self.set_bg2_x(0);
        self.set_bg3_h_copy2(0);
        self.set_bg1_y(0);
        self.set_bg2_y(0);
        self.set_bg3_v_copy2(0);
        self.set_bg1_h_copy(0);
        self.set_bg2_h_copy(0);
        self.set_bg1_v_copy(0);
        self.set_bg2_v_copy(0);
    }

    /// Complete the source-ordered dungeon-info clear after its reset-state
    /// prefix has already been published.
    pub(super) fn death_func15_save_quit_finish_dungeon_info_clear(&mut self) {
        self.save_progress_mut().clear_dungeon_info();
    }

    /// The overworld song-bank upload — the ROM's NMI-masked tail of the
    /// save-quit reset.
    pub(super) fn death_func15_save_quit_song_upload(&mut self) {
        self.select_overworld_song_bank();
        self.load_overworld_songs();
    }

    pub(super) fn GameOver_AnimateChoiceFairy(&mut self) {
        self.set_oam_plain(
            0x14,
            0x34,
            DEATH_SPR_Y0[self.game_state.frame.subsubmodule as usize],
            DEATH_SPR_CHAR0[(self.game_state.frame.frame_counter >> 3 & 1) as usize],
            0x78,
            2,
        );
    }

    pub(super) fn GameOver_InitializeRevivalFairy(&mut self) {
        self.configure_revival_ancillae();
        self.player_resources_mut().set_heart_filler(56);
        self.increment_submodule();
        self.set_overworld_map_state(0);
    }

    pub(super) fn RevivalFairy_Main_bounce(&mut self) {
        self.revival_fairy_main();
    }

    pub(super) fn GameOver_RiseALittle(&mut self) {
        if self.game_state.inventory.player_resources.heart_filler() == 0 {
            let palette = self
                .game_state
                .display
                .ppu_scroll_copy
                .mapbak_palette_slice()[..256]
                .to_vec();
            self.copy_aux_visible_from_tagged(
                &palette,
                crate::game_state::PaletteSliceSource::MirrorBank(zelda3_palette::Bank::Backup),
            );
            self.clear_main_visible_subpalettes();
            self.set_main_color_constant(0, 0);
            self.set_countdown_word(0);
            self.set_darkening_or_lightening_screen_word(2);
            let cgwsel = self.game_state.display.ppu_scroll_copy.mapbak_cgwsel_word();
            self.set_color_window_and_math_word(cgwsel);
            self.increment_submodule();
        }
        self.revival_fairy_main();
        self.hud_refill_logic();
    }

    pub(super) fn GameOver_Restore0D(&mut self) {
        if !self.hud_state().is_doing_heart_animation() {
            self.set_chr_halfslot_request(1);
            self.Graphics_LoadChrHalfSlot();
            let fixed_color = self.game_state.dungeon.room_effects.fixed_color_plusminus();
            self.Dungeon_ApproachFixedColor_variable(fixed_color);
            self.increment_submodule();
        }
        self.revival_fairy_main();
        self.hud_refill_logic();
    }

    pub(super) fn GameOver_Restore0E(&mut self) {
        self.Graphics_LoadChrHalfSlot();
        let sub_screen_layers = self.game_state.display.ppu_scroll_copy.mapbak_ts();
        self.set_sub_screen_layers(sub_screen_layers);
        self.increment_submodule();
    }

    pub(super) fn GameOver_ResituateLink(&mut self) {
        self.PaletteFilter_RestoreBGAdditiveStrict();
        self.copy_color(
            (zelda3_palette::Bank::Main, 32),
            (zelda3_palette::Bank::Main, 0),
        );
        if self.game_state.display.palette_filter.countdown() != 32 {
            return;
        }
        if self.game_state.world.location.is_outdoors() {
            self.Overworld_SetFixedColAndScroll();
        }
        let sub_screen_layers = self.game_state.display.ppu_scroll_copy.mapbak_ts();
        self.set_sub_screen_layers(sub_screen_layers);
        let saved_module = self.game_state.frame.saved_module_for_menu;
        self.set_main_module(saved_module);
        self.set_submodule(0);
        self.follower_link_state_mut().set_blink_countdown(144);
        let music = self.game_state.system_signals.death_backup_current_music();
        self.set_music_control(music);
        let ambient = self.game_state.system_signals.death_backup_ambient_sound();
        self.set_ambient_sound_effect(ambient);
        let palette_filter_countdown = self
            .game_state
            .display
            .ppu_scroll_copy
            .mapbak_bg1_x_offset();
        let darkening_or_lightening_screen = self
            .game_state
            .display
            .ppu_scroll_copy
            .mapbak_bg1_y_offset();
        self.set_countdown_word(palette_filter_countdown);
        self.set_darkening_or_lightening_screen_word(darkening_or_lightening_screen);
    }

    pub(super) fn Module0E_0A_FluteMenu(&mut self) {
        match self.overworld_map_state() {
            0 => self.WorldMap_FadeOut(),
            1 => {
                self.set_birdtravel_status(0);
                self.WorldMap_LoadLightWorldMap();
            }
            2 => self.WorldMap_LoadSpriteGFX(),
            3 => self.WorldMap_Brighten(),
            4 => {
                self.messaging_state_mut().set_menu_animation_timer(0x10);
                self.increment_overworld_map_state();
            }
            5 => self.FluteMenu_HandleSelection(),
            6 => self.WorldMap_RestoreGraphics(),
            7 => self.FluteMenu_LoadSelectedScreen(),
            8 => self.Overworld_LoadOverlayAndMap(),
            9 => self.FluteMenu_FadeInAndQuack(),
            // C Module0E_0A_FluteMenu asserts outside states 0..=9.
            state => panic!("Module0E_0A_FluteMenu invalid overworld_map_state {state}"),
        }
    }

    pub(super) fn FluteMenu_HandleSelection(&mut self) {
        if self.game_state.messaging.runtime.menu_animation_timer() == 0 {
            if (self.game_state.player.follower_link.joypad1l_last()
                | self.game_state.player.follower_link.joypad1h_last())
                & 0xc0
                != 0
            {
                if self
                    .game_state
                    .enhanced_features
                    .has(FEATURE_CANCEL_BIRD_TRAVEL)
                {
                    let joypad = self.game_state.player.follower_link.joypad1l_last();
                    self.messaging_state_mut().set_menu_animation_timer(joypad);
                }
                self.increment_overworld_map_state();
                return;
            }
        } else {
            self.messaging_state_mut().decrement_menu_animation_timer();
        }

        if self.game_state.player.follower_link.filtered_joypad_h() & 10 != 0 {
            self.decrement_birdtravel_status();
            self.set_sound_effect_2(32);
        }
        if self.game_state.player.follower_link.filtered_joypad_h() & 5 != 0 {
            self.increment_birdtravel_status();
            self.set_sound_effect_2(32);
        }
        self.and_birdtravel_status(7);

        let mut pt = Point16U { x: 0, y: 0 };
        if self.game_state.frame.frame_counter & 0x10 != 0
            && self.WorldMap_CalculateOamCoordinates(&mut pt)
        {
            self.WorldMap_AddSprite(16, 2, 0x3e, 0, pt.x.wrapping_sub(4), pt.y.wrapping_sub(4));
        }

        let ybak = self.game_state.player.special_exit_position.y();
        let xbak = self.game_state.player.special_exit_position.x();
        for i in (0..8).rev() {
            let bird_x = u16::from(BIRD_TRAVEL_X_HIGH[i]) << 8 | u16::from(BIRD_TRAVEL_X_LOW[i]);
            let bird_y = u16::from(BIRD_TRAVEL_Y_HIGH[i]) << 8 | u16::from(BIRD_TRAVEL_Y_LOW[i]);
            self.set_bird_travel_destination(i, bird_x, bird_y);
            self.special_exit_position_mut()
                .set_position(bird_x, bird_y);

            if self.WorldMap_CalculateOamCoordinates(&mut pt) {
                self.WorldMap_AddSprite(
                    i,
                    0,
                    if i == usize::from(self.birdtravel_status()) {
                        0x30 + (self.game_state.frame.frame_counter & 6)
                    } else {
                        0x32
                    },
                    BIRD_TRAVEL_OVERWORLD_SCREEN_BY_STOP[i],
                    pt.x,
                    pt.y,
                );
            }
        }
        self.special_exit_position_mut().set_position(xbak, ybak);
    }

    pub(super) fn FluteMenu_LoadSelectedScreen(&mut self) {
        self.FluteMenu_LoadSelectedScreenPrefix();

        if self.game_state.messaging.runtime.menu_animation_timer() & 0x40 == 0 {
            if self.begin_original_timing_flute_menu_selected_screen() {
                return;
            }
            self.FluteMenu_LoadTransport();
        }

        self.FluteMenu_LoadSelectedScreenAfterTransport();
    }

    pub(super) fn FluteMenu_LoadSelectedScreenPrefix(&mut self) {
        self.clear_overworld_event_bits(0x3b, 0x20);
        self.clear_overworld_event_bits(0x7b, 0x20);
        let dung_267 = self
            .game_state
            .inventory
            .save_progress
            .dungeon_info_word(267)
            & !0x0080;
        let dung_40 = self
            .game_state
            .inventory
            .save_progress
            .dungeon_info_word(40)
            & !0x0100;
        self.save_progress_mut()
            .set_dungeon_info_word(267, dung_267);
        self.save_progress_mut().set_dungeon_info_word(40, dung_40);
    }

    pub(super) fn FluteMenu_LoadSelectedScreenAfterTransport(&mut self) {
        self.FluteMenu_LoadSelectedScreenPalettes();
        let t = self.game_state.world.location.overworld_screen_index() & 0xbf;
        self.DecompressAnimatedOverworldTiles(if t == 3 || t == 5 || t == 7 {
            0x58
        } else {
            0x5a
        });
        self.Overworld_SetFixedColAndScroll();
        self.clear_overworld_aux_or_main_offset();
        self.set_hud_palette(0);
        self.InitializeTilesets();
        self.increment_overworld_map_state();
        self.dungeon_room_load_mut().set_draw_width_indicator(0);
        self.Overworld_LoadOverlays2();
        self.decrement_submodule();
        self.set_sound_effect_2(16);
        let m = self.overworld_config_table().current_music();
        self.set_ambient_sound_effect(m >> 4);
        let track = m & 0x0f;
        let music = if self.zelda_is_playing_music_track(track) {
            0xf3
        } else {
            track
        };
        self.set_music_control(music);
    }

    pub(super) fn Overworld_LoadOverlayAndMap(&mut self) {
        let bak1 = self.game_state.frame.main_module_word();
        let bak2 = self.overworld_map_state_word();
        self.Overworld_LoadAndBuildScreen();
        self.set_overworld_map_state_word(bak2.wrapping_add(1));
        self.set_main_module_word(bak1);
    }

    pub(super) fn FluteMenu_FadeInAndQuack(&mut self) {
        self.increment_screen_brightness();
        if self.game_state.display.screen_brightness == 15 {
            self.BirdTravel_Finish_Doit();
        } else {
            self.sprite_main();
        }
    }

    pub(super) fn BirdTravel_Finish_Doit(&mut self) {
        self.set_overworld_map_state(0);
        self.set_subsubmodule(0);
        let saved_module = self.game_state.frame.saved_module_for_menu;
        self.set_main_module(saved_module);
        self.set_submodule(0);
        let hdma_enable_mask = self.game_state.display.ppu_scroll_copy.mapbak_hdmaen();
        self.set_hdma_enable_mask(hdma_enable_mask);
        self.add_bird_travel_something(0x27, 4);
        self.sprite_main();
    }

    pub(super) fn Messaging_OverworldMap(&mut self) {
        match self.overworld_map_state() {
            0 => self.WorldMap_FadeOut(),
            1 => self.WorldMap_LoadLightWorldMap(),
            2 => self.WorldMap_LoadDarkWorldMap(),
            3 => self.WorldMap_LoadSpriteGFX(),
            4 => self.WorldMap_Brighten(),
            5 => self.WorldMap_PlayerControl(),
            6 => self.WorldMap_RestoreGraphics(),
            7 => self.WorldMap_ExitMap(),
            _ => {}
        }
    }

    pub(super) fn WorldMap_FadeOut(&mut self) {
        self.decrement_screen_brightness();
        if self.game_state.display.screen_brightness != 0 {
            return;
        }
        let hdmaen = self.game_state.display.hdma_enable_mask;
        self.set_mapbak_hdmaen(hdmaen);
        // Module0E runs Sprite_Main before this write. Its variable CPU work
        // decides how much of the active field has already scanned out.
        if let Some(scanline) = self
            .last_sprite_main_timing_workload
            .and_then(SpriteMainTimingWorkload::world_map_fade_force_blank_output_scanline)
        {
            self.enable_force_blank_during_active_scanout(scanline);
        } else {
            // Do not project an unmeasured workload onto a known route case.
            self.enable_force_blank();
        }
        self.set_mosaic_copy(3);
        self.increment_overworld_map_state();
        // C backs up MAPBAK_TM:MAPBAK_TS from the RAM word TM_COPY:TS_COPY (the last-projected
        // values), not the live native layer masks. The native sub_screen_layers can be
        // transiently ahead of RAM TS_COPY this frame (dark-room sub-screen path), so reading
        // layer_masks_word() backed up a stale 1 into MAPBAK_TS (0xc212) vs old-rust's 0.
        let tm_ts = read_le_u16(&self.ram, crate::game_state::constants::TM_COPY);
        self.set_mapbak_tm_word(tm_ts);
        let bg1hofs = self.game_state.display.ppu_scroll_copy.bg1_h_copy2();
        let bg2hofs = self.game_state.display.ppu_scroll_copy.bg2_h_copy2();
        let bg1vofs = self.game_state.display.ppu_scroll_copy.bg1_v_copy2();
        let bg2vofs = self.game_state.display.ppu_scroll_copy.bg2_v_copy2();
        self.set_map_backup_scrolls(bg1hofs, bg2hofs, bg1vofs, bg2vofs);
        self.set_bg1_x(0);
        self.set_bg2_x(0);
        self.set_bg3_h_copy2(0);
        self.set_bg1_y(0);
        self.set_bg2_y(0);
        self.set_bg3_v_copy2(0);
        let cgwsel_cgadsub = self
            .game_state
            .display
            .palette_filter
            .color_window_and_math_word();
        self.set_mapbak_cgwsel_word(cgwsel_cgadsub);
        self.follower_link_state_mut()
            .set_link_dma_graphics_index_word(0x01fc);
        if self.game_state.world.location.overworld_screen_index() < 0x80 {
            self.special_exit_position_mut().store_from_player();
        }
        if self.game_state.inventory.save_progress.progress_indicator() < 2 {
            self.set_color_window_selection(0x80);
            self.set_color_math_control(0x61);
        }
        self.set_sound_effect_2(16);
        self.set_ambient_sound_effect(5);
        self.set_music_control(0xf2);
        self.set_bg_mode(7);
    }

    pub(super) fn WorldMap_LoadLightWorldMap(&mut self) {
        if self.begin_world_map_light_load_work() {
            return;
        }
        self.world_map_load_light_world_map();
    }

    pub(super) fn WorldMap_LoadDarkWorldMap(&mut self) {
        if u16::from(self.game_state.world.location.overworld_screen_index()) & 0x40 != 0 {
            if let Some(tilemap) = self.asset_raw(68).map(Vec::from) {
                let len = tilemap.len().min(1024);
                self.copy_tilemap_upload_stripe_bytes(&tilemap[..len]);
            }
            self.set_pending_nmi_subroutine(21);
        }
        self.increment_overworld_map_state();
    }

    pub(super) fn WorldMap_LoadSpriteGFX(&mut self) {
        self.set_chr_halfslot_request(0x10);
        self.Graphics_LoadChrHalfSlot();
        self.clear_chr_halfslot_request();
        self.increment_overworld_map_state();
    }

    pub(super) fn WorldMap_Brighten(&mut self) {
        self.increment_screen_brightness();
        if self.game_state.display.screen_brightness == 15 {
            self.increment_overworld_map_state();
        }
    }

    pub(super) fn DidPressButtonForMap(&self) -> bool {
        if self.game_state.world.transient.hud_cur_item_x() != 0 {
            self.game_state.player.follower_link.filtered_joypad_h() & 0x20 != 0
        } else {
            self.game_state.player.follower_link.filtered_joypad_l() & 0x40 != 0
        }
    }

    pub(super) fn WorldMap_PlayerControl(&mut self) {
        if self.overworld_map_flags() & 0x80 != 0 {
            self.and_overworld_map_flags(!0x80);
            self.OverworldMap_SetupHdma();
        }

        if self.overworld_map_flags() == 0 && self.DidPressButtonForMap() {
            self.increment_overworld_map_state();
            return;
        }

        if self.game_state.dungeon.room_load.draw_width_indicator() != 0 {
            let draw_width = self
                .game_state
                .dungeon
                .room_load
                .draw_width_indicator()
                .wrapping_sub(1);
            self.dungeon_room_load_mut()
                .set_draw_width_indicator(draw_width);
        } else if self.game_state.player.follower_link.filtered_joypad_l() & 0x30 != 0
            || self.DidPressButtonForMap()
        {
            self.set_sound_effect_2(36);
            self.dungeon_room_load_mut().set_draw_width_indicator(8);
            let t = (self.overworld_map_flags() ^ 1) & 1;
            self.set_overworld_map_flags(t | 0x80);
            self.set_mode7_zoom_timer(OVERWORLD_MAP_ZOOM_TIMERS[t as usize]);
            if self.mode7_zoom_timer() == 12 {
                let y = self.game_state.player.special_exit_position.map_zoom_y();
                self.set_bg1_y(y);
                self.set_mode7_center_y(y.wrapping_add(0x100));
                let t0 = self
                    .game_state
                    .player
                    .special_exit_position
                    .map_zoom_x_offset();
                let abs_t0 = if (t0 as i16) < 0 {
                    0u16.wrapping_sub(t0)
                } else {
                    t0
                };
                let t1 = abs_t0.wrapping_mul(5) >> 1;
                let t2 = if (t0 as i16) < 0 {
                    0u16.wrapping_sub(t1)
                } else {
                    t1
                };
                self.set_bg1_x(t2.wrapping_add(0x80) & !1);
            } else {
                self.set_bg1_y(200);
                self.set_mode7_center_y(200 + 256);
                self.set_bg1_x(128);
            }
        }

        if self.overworld_map_flags() != 0 {
            let k = ((self.game_state.player.follower_link.joypad1h_last() & 12) >> 1) as usize;
            let bg1vofs = self.game_state.display.ppu_scroll_copy.bg1_v_copy2();
            if bg1vofs != OVERWORLD_MAP_SCROLL_TARGETS[k] {
                let next = bg1vofs.wrapping_add(OVERWORLD_MAP_SCROLL_DELTAS[k] as u16);
                self.set_bg1_y(next);
                self.set_mode7_center_y(next.wrapping_add(0x100));
            }
            let k = ((self.game_state.player.follower_link.joypad1h_last() & 3) * 2 + 1) as usize;
            let bg1hofs = self.game_state.display.ppu_scroll_copy.bg1_h_copy2();
            if bg1hofs != OVERWORLD_MAP_SCROLL_TARGETS[k] {
                self.set_bg1_h_copy2(bg1hofs.wrapping_add(OVERWORLD_MAP_SCROLL_DELTAS[k] as u16));
            }
        }

        self.WorldMap_HandleSprites();
    }

    pub(super) fn WorldMap_RestoreGraphics(&mut self) {
        self.decrement_screen_brightness();
        if self.game_state.display.screen_brightness != 0 {
            return;
        }
        self.EnableForceBlank();
        self.increment_overworld_map_state();
        let aux = self
            .game_state
            .display
            .palette_buffer
            .aux_full_slice()
            .to_vec();
        self.copy_main_full_from_tagged(
            &aux,
            crate::game_state::PaletteSliceSource::MirrorBank(zelda3_palette::Bank::Aux),
        );
        let cgwsel_cgadsub = self.game_state.display.ppu_scroll_copy.mapbak_cgwsel_word();
        self.set_color_window_and_math_word(cgwsel_cgadsub);
        self.set_bg3_h_copy2(0);
        self.set_bg3_v_copy2(0);
        let bg1hofs = self
            .game_state
            .display
            .ppu_scroll_copy
            .map_backup_bg1_h_copy2();
        let bg2hofs = self
            .game_state
            .display
            .ppu_scroll_copy
            .map_backup_bg2_h_copy2();
        let bg1vofs = self
            .game_state
            .display
            .ppu_scroll_copy
            .map_backup_bg1_v_copy2();
        let bg2vofs = self
            .game_state
            .display
            .ppu_scroll_copy
            .map_backup_bg2_v_copy2();
        self.set_bg1_x(bg1hofs);
        self.set_bg2_x(bg2hofs);
        self.set_bg1_y(bg1vofs);
        self.set_bg2_y(bg2vofs);
        let tm_ts = self.game_state.display.ppu_scroll_copy.mapbak_tm_word();
        self.set_layer_masks_word(tm_ts);
        self.Attract_SetUpConclusionHDMA();
    }

    pub(super) fn Attract_SetUpConclusionHDMA(&mut self) {
        self.hdma_setup(0x0abddd, 0x0abddd, 0x42, 0x1b, 0x1e, 0);
        self.set_hdma_enable_mask(0x80);
        self.set_bg_mode(9);
        self.clear_core_update_disable_flag();
    }

    pub(super) fn WorldMap_ExitMap(&mut self) {
        self.clear_overworld_aux_or_main_offset();
        self.set_hud_palette(0);
        if self.rom_startup_timing() {
            // InitializeTilesets is a long 65816 decompression/conversion
            // path. The ROM remains in module $0e/$07 under forced blank
            // while vblank interrupts it; do not expose its caller suffix as
            // an atomic main-thread operation.
            self.game_execution_scheduler.schedule_work(
                GameWorkContinuation::FinishWorldMapExitTilesets,
                WORLD_MAP_EXIT_TILESET_LOAD_NMI_SLICES,
            );
            return;
        }
        self.complete_world_map_exit_after_tileset_load();
    }

    pub(super) fn complete_world_map_exit_after_tileset_load(&mut self) {
        self.InitializeTilesets();
        self.increment_cgram_update_flag();
        self.dungeon_room_load_mut().set_draw_width_indicator(0);
        self.set_overworld_map_state(0);
        self.set_subsubmodule(0);
        let saved_module = self.game_state.frame.saved_module_for_menu;
        self.set_main_module(saved_module);
        self.set_submodule(32);
        self.clear_vram_upload_cursor();
        let hdma_enable_mask = self.game_state.display.ppu_scroll_copy.mapbak_hdmaen();
        self.set_hdma_enable_mask(hdma_enable_mask);
        let music = self.overworld_config_table().current_music();
        self.set_ambient_sound_effect(music >> 4);
        self.set_sound_effect_2(0x10);
        self.set_music_control(0xf3);
    }

    pub(super) fn WorldMap_SetUpHDMA(&mut self) {
        self.world_map_setup_hdma();
    }

    pub(super) fn WorldMap_FillTilemapWithEF(&mut self) {
        self.world_map_fill_tilemap_with_ef();
    }

    pub(super) fn WorldMap_HandleSprites(&mut self) {
        let ybak = self.game_state.player.special_exit_position.y();
        let xbak = self.game_state.player.special_exit_position.x();

        if self.game_state.frame.frame_counter & 0x10 != 0 {
            if let Some((x, y)) = self.WorldMap_CalculateCurrentOamCoordinates() {
                self.WorldMap_AddSprite(0, 2, 0x3e, 0, x.wrapping_sub(4), y.wrapping_sub(4));
            }
        }

        let k = 15;
        if self.game_state.world.location.overworld_screen_index() < 0x40
            && !self
                .game_state
                .world
                .overworld
                .bird_travel_destinations
                .destination(k)
                .is_empty()
        {
            if self.game_state.frame.frame_counter == 0 {
                self.increment_bird_travel_stop_status(k);
            }
            let bird = self
                .game_state
                .world
                .overworld
                .bird_travel_destinations
                .destination(k);
            let bird_x = bird.x;
            let bird_y = bird.y;
            self.special_exit_position_mut()
                .set_position(bird_x, bird_y);
            if let Some((x, y)) = self.WorldMap_CalculateCurrentOamCoordinates() {
                self.WorldMap_AddSprite(
                    15,
                    2,
                    OVERWORLD_MAP_BIRD_FRAME_CHARS
                        [(self.game_state.frame.frame_counter >> 1 & 3) as usize],
                    0x6a,
                    x,
                    y,
                );
            }
        }

        if self.game_state.world.overworld.event_info.event_info(0x5b) & 0x20 == 0
            && (((self
                .game_state
                .inventory
                .save_progress
                .map_icons_indicator()
                >= 6) as u8
                ^ self.game_state.world.region.is_in_dark_world() as u8)
                & 1)
                == 0
        {
            self.WorldMap_HandleCrystalSprites();
        }

        self.special_exit_position_mut().set_position(xbak, ybak);
    }

    fn WorldMap_HandleCrystalSprites(&mut self) {
        let k = self
            .game_state
            .inventory
            .save_progress
            .map_icons_indicator() as usize;
        if k >= 9 {
            return;
        }
        for crystal in 0..7 {
            let have_marker = if crystal < 3 {
                self.OverworldMap_CheckForPendant(crystal)
                    || self.OverworldMap_CheckForCrystal(crystal)
            } else {
                self.OverworldMap_CheckForCrystal(crystal)
            };
            if have_marker || (OVERWORLD_MAP_CRYSTAL_ICON_X_POSITIONS[crystal][k] as i16) < 0 {
                continue;
            }
            self.special_exit_position_mut().set_position(
                OVERWORLD_MAP_CRYSTAL_ICON_X_POSITIONS[crystal][k],
                OVERWORLD_MAP_CRYSTAL_ICON_Y_POSITIONS[crystal][k],
            );
            let mut info = OVERWORLD_MAP_CRYSTAL_ICON_INFO_TILES[crystal][k];
            let t = (info >> 8) as u8;
            if t != 0 {
                if t != 100 && self.game_state.frame.frame_counter & 0x10 != 0 {
                    continue;
                }
                self.special_exit_position_mut()
                    .offset_position(0u16.wrapping_sub(4), 0u16.wrapping_sub(4));
            }
            if let Some((x, y)) = self.WorldMap_CalculateCurrentOamCoordinates() {
                let mut ext = 2;
                if info >> 8 == 0 {
                    info = u16::from(
                        OVERWORLD_MAP_CRYSTAL_ICON_FRAMES
                            [(self.game_state.frame.frame_counter >> 3 & 3) as usize],
                    ) << 8
                        | 0x32;
                    ext = 0;
                }
                self.WorldMap_AddSprite(14 - crystal, ext, info as u8, (info >> 8) as u8, x, y);
            }
        }
    }

    pub(super) fn WorldMap_CalculateOamCoordinates(&mut self, pt: &mut Point16U) -> bool {
        if let Some((x, y)) = self.WorldMap_CalculateCurrentOamCoordinates() {
            pt.x = x;
            pt.y = y;
            true
        } else {
            false
        }
    }

    fn WorldMap_CalculateCurrentOamCoordinates(&self) -> Option<(u16, u16)> {
        let spexit = &self.game_state.player.special_exit_position;
        let y_spexit = spexit.y();
        let x_spexit = spexit.x();
        if self.overworld_map_flags() == 0 {
            let j = (-(i32::from(y_spexit >> 4))
                + i32::from(self.game_state.display.ppu_scroll_copy.mode7_center_y())
                + i32::from((y_spexit >> 3) & 1)
                - 0xc0) as usize;
            let yval = 13u16.wrapping_mul(*OVERWORLD_MAP_PROJECTION_CURVE.get(j)? as u16) >> 4;
            let mut at = (x_spexit >> 4) as u8;
            let below = at < 0x80;
            at = at.wrapping_sub(0x80);
            if (at as i8) < 0 {
                at = !at;
            }
            let t1 = (((if yval < 224 { yval } else { 0 }) * 0x54) >> 8) as u8 + 0xb2;
            let t2 = ((u16::from(at) * u16::from(t1)) >> 8) as u8;
            let x = if below {
                0x80u16.wrapping_sub(u16::from(t2))
            } else {
                u16::from(t2).wrapping_add(0x80)
            };
            Some((
                x.wrapping_sub(self.game_state.display.ppu_scroll_copy.bg1_h_copy2())
                    .wrapping_add(0x80),
                yval + 12,
            ))
        } else {
            let t0 = (-(i32::from(y_spexit >> 4))
                + i32::from(self.game_state.display.ppu_scroll_copy.mode7_center_y())
                - 0x80) as u16;
            if t0 >= 0x100 {
                return None;
            }
            let t1 = (t0 * 37) >> 4;
            let yval = *OVERWORLD_MAP_PROJECTION_CURVE.get(t1 as usize)? as u16;
            let mut t2 = x_spexit;
            let below = t2 < 0x7f8;
            t2 = t2.wrapping_sub(0x7f8);
            if (t2 as i16) < 0 {
                t2 = (!t2).wrapping_add(1);
            }
            let t3 = if yval < 226 { yval } else { 0 };
            let t4 = ((t3 * 84) >> 8) + 178;
            let t5 = (((t2 as u8 as u16) * t4) >> 8) as u16;
            let t6 = ((t2 >> 8) * t4).wrapping_add(t5);
            let mut t7 = if below {
                0x800u16.wrapping_sub(t6)
            } else {
                t6.wrapping_add(0x800)
            };
            let below2 = t7 < 0x800;
            t7 = t7.wrapping_sub(0x800);
            let t8 = if below2 { (!t7).wrapping_add(1) } else { t7 };
            let t9 = (((t8 as u8 as u16) * 45) >> 8) as u16;
            let t10 = ((t8 >> 8) * 45).wrapping_add(t9);
            let t11 = if below2 {
                0x80u16.wrapping_sub(t10)
            } else {
                t10.wrapping_add(0x80)
            };
            let xval = t11.wrapping_sub(self.game_state.display.ppu_scroll_copy.bg1_h_copy2());
            let xt = if self
                .game_state
                .enhanced_features
                .has(FEATURE_EXTEND_SCREEN64_MAP)
            {
                0x48
            } else {
                0
            };
            if xval.wrapping_add(0x80).wrapping_add(xt) >= 0x100 + xt * 2 {
                return None;
            }
            Some((xval.wrapping_add(0x81), yval.wrapping_add(16)))
        }
    }

    pub(super) fn WorldMap_AddSprite(
        &mut self,
        spr: usize,
        big: u8,
        flags: u8,
        ch: u8,
        x: u16,
        y: u16,
    ) {
        let mut big = big;
        let mut flags = flags;
        let mut ch = ch;
        let mut x = x;
        let mut y = y;

        if self.game_state.frame.frame_counter & 0x10 == 0 && ch == 100 {
            assert!(spr >= 8);
            ch = OVERWORLD_MAP_ICON_TILES[spr - 8];
            flags = 0x32;
            big = 0;
        } else {
            x = x.wrapping_sub(4);
            y = y.wrapping_sub(4);
        }
        if self
            .game_state
            .enhanced_features
            .has(FEATURE_EXTEND_SCREEN64_MAP)
        {
            big |= ((x >> 8) as u8) & 1;
        }
        self.set_oam_plain(spr, x as u8, y as u8, ch, flags, big);
    }

    pub(super) fn OverworldMap_CheckForPendant(&self, k: usize) -> bool {
        self.game_state
            .inventory
            .save_progress
            .map_icons_indicator()
            == 3
            && self.game_state.inventory.player_resources.pendant_flags()
                & OVERWORLD_MAP_PENDANT_BIT_MASKS[k]
                != 0
    }

    pub(super) fn OverworldMap_CheckForCrystal(&self, k: usize) -> bool {
        self.game_state
            .inventory
            .save_progress
            .map_icons_indicator()
            == 7
            && self.game_state.inventory.player_resources.crystal_flags()
                & OVERWORLD_MAP_CRYSTAL_BIT_MASKS[k]
                != 0
    }

    pub(super) fn Module0E_03_DungeonMap(&mut self) {
        self.replay_trace_ram_watch("dungmap-before-submodule");
        match self.overworld_map_state() {
            0 => self.DungMap_Backup(),
            1 => self.Module0E_03_01_DrawMap(),
            2 => self.DungMap_LightenUpMap(),
            3 => self.DungeonMap_HandleInputAndSprites(),
            4 => self.DungMap_4(),
            5 => self.DungMap_FadeMapToBlack(),
            6 => self.DungeonMap_RecoverGFX(),
            7 => self.ToggleStarTilesAndAdvance(),
            _ => self.DungMap_RestoreOld(),
        }
        self.replay_trace_ram_watch("dungmap-after-submodule");
    }

    pub(super) fn Module0E_03_01_DrawMap(&mut self) {
        self.replay_trace_ram_watch("dungmap-draw-before-init");
        match self.game_state.dungeon_map_display.dungmap_init_state() {
            0 => self.Module0E_03_01_00_PrepMapGraphics(),
            1 => self.Module0E_03_01_01_DrawLEVEL(),
            2 => self.Module0E_03_01_02_DrawFloorsBackdrop(),
            3 => self.Module0E_03_01_03_DrawRooms(),
            4 => self.DungeonMap_DrawRoomMarkers(),
            // C dispatches through kDungMapInit and asserts in debug if this
            // state is outside the initialized table.
            state => panic!("Module0E_03_01_DrawMap invalid dungmap_init_state {state}"),
        }
        self.replay_trace_ram_watch("dungmap-draw-after-init");
    }

    pub(super) fn Module0E_03_01_00_PrepMapGraphics(&mut self) {
        if self.rom_startup_timing() {
            self.game_execution_scheduler.schedule_work(
                GameWorkContinuation::FinishDungeonMapGraphicsPreparation,
                DUNGEON_MAP_GRAPHICS_PREPARATION_NMI_SLICES,
            );
            return;
        }
        self.complete_dungeon_map_graphics_preparation();
    }

    pub(super) fn complete_dungeon_map_graphics_preparation(&mut self) {
        self.replay_trace_ram_watch("dungmap-prep-entry");
        let hdmaen_bak = self.game_state.display.hdma_enable_mask;
        self.clear_hdma_enable_mask();
        let main_tile_theme = self.game_state.world.palette_theme.main_tile_theme_index();
        let sprite_gfx = self.game_state.sprites.system.graphics_index();
        let aux_tile_theme = self.game_state.world.palette_theme.aux_tile_theme_index();
        let main_layers = self.game_state.display.main_screen_layers;
        let sub_layers = self.game_state.display.sub_screen_layers;
        self.set_mapbak_main_tile_theme_index(main_tile_theme);
        self.set_mapbak_sprite_graphics_index(sprite_gfx);
        self.set_mapbak_aux_tile_theme_index(aux_tile_theme);
        self.set_mapbak_tm(main_layers);
        self.set_mapbak_ts(sub_layers);
        self.world_palette_theme_mut().set_main_tile_theme_index(32);
        let graphics_index =
            0x80 | (self.game_state.inventory.save_progress.palace_index_x2() >> 1);
        self.sprite_system_mut().set_graphics_index(graphics_index);
        self.world_palette_theme_mut().set_aux_tile_theme_index(64);
        self.set_main_screen_layers(0x16);
        self.set_sub_screen_layers(1);
        self.EraseTileMaps_dungeonmap();
        self.InitializeTilesets();
        self.select_overworld_aux_palette_offset();
        self.replay_trace_ram_watch("dungmap-prep-before-bg-palette");
        self.Palette_Load_DungeonMapBG();
        self.replay_trace_ram_watch("dungmap-prep-after-bg-palette");
        self.Palette_Load_DungeonMapSprite();
        self.replay_trace_ram_watch("dungmap-prep-after-sprite-palette");
        self.set_hud_palette(1);
        self.Palette_Load_HUD();
        self.replay_trace_ram_watch("dungmap-prep-after-hud-palette");
        self.LoadActualGearPalettes();
        self.replay_trace_ram_watch("dungmap-prep-after-gear-palette");
        self.increment_cgram_update_flag();
        self.increment_dungeon_map_init_state();
        self.set_hdma_enable_mask(hdmaen_bak);
        self.set_bg_vram_load_mode(9);
        self.set_core_update_disable_flag(9);
        self.replay_trace_ram_watch("dungmap-prep-exit");
    }

    pub(super) fn Module0E_03_01_01_DrawLEVEL(&mut self) {
        let dung = usize::from(self.game_state.inventory.save_progress.palace_index_x2() >> 1)
            .min(DUNGEON_MAP_LEVEL_LABEL_INDEX_BY_DUNGEON.len() - 1);
        let i = DUNGEON_MAP_LEVEL_LABEL_INDEX_BY_DUNGEON[dung] >> 1;
        if i >= 0 {
            let i = i as usize;
            self.write_vram_upload_level_label_tiles(
                &DUNGEON_MAP_LEVEL_LABEL_TOP_STRIPE,
                &DUNGEON_MAP_LEVEL_LABEL_BOTTOM_STRIPE,
            );
            self.write_vram_upload_buffer_word(14, DUNGEON_MAP_LEVEL_LABEL_TOP_TILES[i]);
            self.write_vram_upload_buffer_word(30, DUNGEON_MAP_LEVEL_LABEL_BOTTOM_TILES[i]);
            self.set_bg_vram_load_mode(1);
        }
        self.increment_dungeon_map_init_state();
    }

    pub(super) fn Module0E_03_01_02_DrawFloorsBackdrop(&mut self) {
        let dung = usize::from(self.game_state.inventory.save_progress.palace_index_x2() >> 1)
            .min(DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON.len() - 1);
        let t5 = DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON[dung];
        let mut offs = 0usize;

        if t5 & 0x0100 != 0 {
            for &tile in &DUNGEON_MAP_FLOOR_LIST_HEADER_STRIPE {
                self.write_vram_upload_buffer_word(offs * 2, tile);
                offs += 1;
            }
            let mut t = 0x1123u16;
            for _ in 0..16 {
                self.write_vram_upload_buffer_word(offs * 2, t.swap_bytes());
                self.write_vram_upload_buffer_word((offs + 1) * 2, 0x0e40);
                self.write_vram_upload_buffer_word((offs + 2) * 2, 0x1b2e);
                t = t.wrapping_add(0x20);
                offs += 3;
            }
        }

        let t5_low = t5 as u8;
        let tab7_index = if t5_low >= 0x50 {
            usize::from((t5_low >> 4).wrapping_sub(4))
        } else if t5 & 0x0f >= 5 {
            usize::from(t5 & 0x0f)
        } else {
            0
        };
        let mut t7 = DUNGEON_MAP_FLOOR_LIST_VRAM_STARTS[tab7_index];
        let t7_org = t7;
        let mut j = 0usize;
        loop {
            self.write_vram_upload_buffer_word(offs * 2, t7.swap_bytes());
            offs += 1;
            self.write_vram_upload_buffer_word(offs * 2, 0x0e40);
            offs += 1;
            self.write_vram_upload_buffer_word(
                offs * 2,
                DUNGEON_MAP_FLOOR_LIST_LABEL_TILES[j] + if t5 & 0x0200 != 0 { 0x0400 } else { 0 },
            );
            offs += 1;
            if j != 6 {
                j += 1;
            }
            t7 = t7.wrapping_add(0x20);
            if t7 >= 0x1360 {
                break;
            }
        }
        self.set_vram_upload_cursor((offs * 2) as u16);
        self.DungeonMap_BuildFloorListBoxes(t5 as u8, t7_org);
        let offset = self.game_state.display.vram_upload_cursor_usize();
        self.terminate_vram_upload_buffer_at(offset);
        self.increment_dungeon_map_init_state();
        self.set_bg_vram_load_mode(1);
    }

    pub(super) fn DungeonMap_BuildFloorListBoxes(&mut self, t5: u8, mut r14: u16) {
        let n = usize::from((t5 & 0x0f).wrapping_add(t5 >> 4)).max(1);
        r14 = r14
            .wrapping_sub(0x40 - 2)
            .wrapping_add(u16::from(t5 & 0x0f) * 0x40);
        let mut offs = self.game_state.display.vram_upload_cursor_usize() >> 1;
        for _ in 0..n {
            self.write_vram_upload_buffer_word(offs * 2, r14.swap_bytes());
            offs += 1;
            self.write_vram_upload_buffer_word(offs * 2, 0x0700);
            offs += 1;
            for (x, &tile) in DUNGEON_MAP_FLOOR_LIST_BOX_TILES.iter().enumerate() {
                self.write_vram_upload_buffer_word(offs * 2, tile);
                offs += 1;
                if x == 3 {
                    r14 = r14.wrapping_add(0x20);
                    self.write_vram_upload_buffer_word(offs * 2, r14.swap_bytes());
                    offs += 1;
                    self.write_vram_upload_buffer_word(offs * 2, 0x0700);
                    offs += 1;
                }
            }
            r14 = r14.wrapping_sub(0x40 + 0x20);
        }
        self.set_vram_upload_cursor((offs * 2) as u16);
    }

    pub(super) fn Module0E_03_01_03_DrawRooms(&mut self) {
        if self.rom_startup_timing() {
            if !self.take_original_timing_main_loop_iteration_returned_to_wait() {
                self.game_execution_scheduler.schedule_work(
                    GameWorkContinuation::FinishDungeonMapRoomDrawing,
                    DUNGEON_MAP_ROOM_DRAWING_NMI_SLICES,
                );
                return;
            }
        }
        self.complete_dungeon_map_room_drawing();
    }

    pub(super) fn complete_dungeon_map_room_drawing(&mut self) {
        self.clear_dungeon_map_floor_scroll_step();
        self.clear_dungeon_map_idx();
        let dung = usize::from(self.game_state.inventory.save_progress.palace_index_x2() >> 1)
            .min(DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON.len() - 1);
        let t =
            (-(i16::from((DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON[dung] & 0x0f) as u8)) as u16) as u8;
        if self.game_state.dungeon.stair_movement.current_floor_word() != u16::from(t) {
            let dung_cur_floor = u16::from(self.game_state.dungeon.stair_movement.current_floor());
            self.set_dungeon_map_current_floor(dung_cur_floor);
        } else {
            let dung_cur_floor = self
                .game_state
                .dungeon
                .stair_movement
                .current_floor_word()
                .wrapping_add(1);
            let dungmap_idx = self
                .game_state
                .dungeon_map_display
                .dungmap_idx()
                .wrapping_add(2);
            self.set_dungeon_map_current_floor(dung_cur_floor);
            self.set_dungeon_map_idx(dungmap_idx);
        }
        self.DungeonMap_DrawFloorNumbersByRoom(0, !0x1000);
        self.DungeonMap_DrawBorderForRooms(0, !0x1000);
        self.DungeonMap_DrawDungeonLayout(0);
        self.decrement_dungeon_map_current_floor_byte();
        self.DungeonMap_DrawFloorNumbersByRoom(0x0300, !0x1000);
        self.DungeonMap_DrawBorderForRooms(0x0300, !0x1000);
        self.DungeonMap_DrawDungeonLayout(0x0300);
        let dungmap_cur_floor = self
            .game_state
            .dungeon_map_display
            .dungmap_cur_floor()
            .wrapping_add(1);
        self.set_dungeon_map_current_floor(dungmap_cur_floor);
        self.clear_dungeon_map_scroll_state();
        self.set_pending_nmi_subroutine(8);
        self.set_nmi_load_target_page(0x22);
        self.increment_dungeon_map_init_state();
    }

    pub(super) fn DungeonMap_DrawBorderForRooms(&mut self, pd: u16, mask: u16) {
        for i in 0..4 {
            let idx = (((DUNGEON_MAP_ROOM_BORDER_CORNER_POSITIONS[i].wrapping_add(pd)) & 0x0fff)
                >> 1) as usize;
            self.set_messaging_render_buffer_word(
                idx,
                DUNGEON_MAP_ROOM_BORDER_CORNER_TILES[i] & mask,
            );
        }
        for i in 0..2 {
            let r4 = DUNGEON_MAP_ROOM_BORDER_HORIZONTAL_POSITIONS[i].wrapping_add(pd);
            for j in (0..20u16).step_by(2) {
                let idx = (((r4.wrapping_add(j)) & 0x0fff) >> 1) as usize;
                self.set_messaging_render_buffer_word(
                    idx,
                    DUNGEON_MAP_ROOM_BORDER_HORIZONTAL_TILES[i] & mask,
                );
            }
        }
        for i in 0..2 {
            let r4 = DUNGEON_MAP_ROOM_BORDER_VERTICAL_POSITIONS[i].wrapping_add(pd);
            for j in (0..0x280u16).step_by(0x40) {
                let idx = (((r4.wrapping_add(j)) & 0x0fff) >> 1) as usize;
                self.set_messaging_render_buffer_word(
                    idx,
                    DUNGEON_MAP_ROOM_BORDER_VERTICAL_TILES[i] & mask,
                );
            }
        }
    }

    pub(super) fn DungeonMap_DrawFloorNumbersByRoom(&mut self, pd: u16, r8: u16) {
        let mut p = 0x00deu16;
        loop {
            let t = (((p.wrapping_add(pd)) & 0x0fff) >> 1) as usize;
            self.set_messaging_render_buffer_word(t, 0x0f00);
            self.set_messaging_render_buffer_word(t + 1, 0x0f00);
            p = p.wrapping_add(0x40);
            if p == 0x039e {
                break;
            }
        }
        let t = (((0x035eu16.wrapping_add(pd)) & 0x0fff) >> 1) as usize;
        let floor = self.game_state.dungeon_map_display.dungmap_cur_floor();
        let (q1, q2) = if (floor & 0x80) != 0 {
            (
                0x1f1c,
                DUNGEON_MAP_FLOOR_NUMBER_TILES[usize::from((!(floor as u8)) & 0x07)],
            )
        } else {
            (
                DUNGEON_MAP_FLOOR_NUMBER_TILES[usize::from(floor & 0x0f)],
                0x1f1d,
            )
        };
        self.set_messaging_render_buffer_word(t, q1 & r8);
        self.set_messaging_render_buffer_word(t + 1, q2 & r8);
    }

    pub(super) fn DungeonMap_DrawDungeonLayout(&mut self, pd: i32) {
        for i in 0..5 {
            let arg_x = ((292 + 128 * i + pd) & 0x0fff) >> 1;
            self.DungeonMap_DrawSingleRowOfRooms(i, arg_x);
        }
    }

    pub(super) fn DungeonMap_DrawSingleRowOfRooms(&mut self, i: i32, mut arg_x: i32) {
        let dung = usize::from(self.game_state.inventory.save_progress.palace_index_x2() >> 1)
            .min(DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON.len() - 1);
        let t5 = DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON[dung];
        let dungmask = DUNG_MAP_UPPER_BITMASKS[dung & 0x0f];
        let curp = self.GetDungmapFloorLayout();
        let has_map = self
            .game_state
            .inventory
            .player_resources
            .has_dungeon_map_mask(dungmask);

        for j in 0..5 {
            let mut r14 = self
                .game_state
                .dungeon_map_display
                .dungmap_cur_floor_byte()
                .wrapping_add((t5 & 0x0f) as u8);
            let room_index = usize::from(r14) * 25 + (i as usize) * 5 + j as usize;
            let v = curp.get(room_index).copied().unwrap_or(0x0f);
            let yv = if v == 0x0f {
                0x51
            } else {
                r14 = (self
                    .game_state
                    .inventory
                    .save_progress
                    .dungeon_info_word(usize::from(v))
                    & 0x0f) as u8;
                let mut k = 0usize;
                let mut count = 0usize;
                while k < curp.len() && curp[k] != v {
                    count += usize::from(curp[k] != 0x0f);
                    k += 1;
                }
                self.GetOtherDungmapInfo(count)
            };

            let base = usize::from(yv) * 4;
            let av0 =
                self.dungeon_map_room_tile(DUNGEON_MAP_ROOM_QUADRANT_TILES[base], r14, 8, has_map);
            let av1 = self.dungeon_map_room_tile(
                DUNGEON_MAP_ROOM_QUADRANT_TILES[base + 1],
                r14,
                4,
                has_map,
            );
            let av2 = self.dungeon_map_room_tile(
                DUNGEON_MAP_ROOM_QUADRANT_TILES[base + 2],
                r14,
                2,
                has_map,
            );
            let av3 = self.dungeon_map_room_tile(
                DUNGEON_MAP_ROOM_QUADRANT_TILES[base + 3],
                r14,
                1,
                has_map,
            );
            let idx = arg_x as usize;
            self.set_messaging_render_buffer_word(idx, av0);
            self.set_messaging_render_buffer_word(idx + 1, av1);
            self.set_messaging_render_buffer_word(idx + 32, av2);
            self.set_messaging_render_buffer_word(idx + 33, av3);
            arg_x += 2;
        }
    }

    fn dungeon_map_room_tile(&self, mut r12: u16, r14: u8, bit: u8, has_map: bool) -> u16 {
        let r12_org = r12;
        if r12 != 0x0b00 && (r14 & bit) == 0 {
            if (r12 & 0x1000) == 0 {
                r12 = 0x0400;
            } else if has_map {
                return (r12 & !0x1c00) | 0x0c00;
            } else {
                r12 = 0;
            }
        } else {
            r12 = 0;
        }
        if has_map || (r14 & bit) != 0 {
            r12.wrapping_add(r12_org)
        } else {
            0x0b00
        }
    }

    pub(super) fn DungeonMap_DrawRoomMarkers(&mut self) {
        let dung = usize::from(self.game_state.inventory.save_progress.palace_index_x2() >> 1)
            .min(DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON.len() - 1);
        let t5 = (DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON[dung] & 0x0f) as u8;
        let floor1 = t5.wrapping_add(self.game_state.dungeon.stair_movement.current_floor());

        let mut room = self.game_state.world.location.dungeon_room();
        for i in 0..3 {
            if room == DUNGEON_MAP_ROOM_REMAP_FROM[i] {
                room = DUNGEON_MAP_ROOM_REMAP_TO[i];
            }
        }

        let roomp = self.GetDungmapFloorLayout();
        let mut xcoord = 0u8;
        let mut ycoord = 0u8;
        let base = usize::from(floor1) * 25;
        for i in 0..25 {
            if roomp.get(base + i).copied().unwrap_or(0x0f) == room as u8 {
                break;
            }
            if xcoord < 64 {
                xcoord = xcoord.wrapping_add(16);
            } else {
                xcoord = 0;
                ycoord = ycoord.wrapping_add(16);
            }
        }

        let marker_x = u16::from(xcoord)
            .wrapping_add(0x90)
            .wrapping_add((self.game_state.player.follower_link.x() & 0x01e0) >> 5);
        self.set_dungeon_map_player_marker_x(marker_x);
        self.set_dungeon_map_location_marker_base_y(ycoord);

        let idx = usize::from((self.game_state.dungeon_map_display.dungmap_idx() >> 1) & 1);
        let marker_y = u16::from(ycoord)
            .wrapping_add(DUNGEON_MAP_MARKER_Y_BASES[idx])
            .wrapping_add((self.game_state.player.follower_link.y() & 0x01e0) >> 5);
        self.set_dungeon_map_player_marker_y(marker_y);

        let floor2 = t5.wrapping_add(DUNGEON_MAP_BOSS_FLOOR_OFFSETS[dung] as u8);
        let marker_base = usize::from(floor2) * 25;
        self.reset_dungeon_map_marker_offsets();

        let lookfor = DUNGEON_MAP_BOSS_ROOM_BY_DUNGEON[dung];
        for j in (0..25).rev() {
            let value = roomp.get(marker_base + j).copied().unwrap_or(0x0f);
            if value != 0x0f && value == lookfor {
                break;
            }
            let marker_x_offset = self.shift_dungeon_map_marker_x_left();
            if (marker_x_offset as i16) < 0 {
                self.reset_dungeon_map_marker_x_and_shift_marker_y_low_up();
            }
        }

        let floor3 = (self.game_state.dungeon_map_display.dungmap_cur_floor_byte() as i8)
            .wrapping_sub(DUNGEON_MAP_BOSS_FLOOR_OFFSETS[dung] as i8);
        let marker_y_offset = self
            .game_state
            .dungeon_map_display
            .marker_y_offset()
            .wrapping_add_signed(i16::from(floor3) * 0x60)
            .wrapping_add(DUNGEON_MAP_MARKER_Y_BASES[0]);
        self.set_dungeon_map_marker_y_offset(marker_y_offset);
        self.increment_overworld_map_state();
        self.set_screen_brightness(0);
        self.clear_dungeon_map_init_state();
    }

    pub(super) fn DungeonMap_HandleInputAndSprites(&mut self) {
        self.DungeonMap_HandleInput();
        self.DungeonMap_DrawSprites();
    }

    pub(super) fn DungeonMap_HandleInput(&mut self) {
        if self.WantExitDungeonMap() {
            let overworld_map_state = self.overworld_map_state().wrapping_add(2);
            self.set_overworld_map_state(overworld_map_state);
            self.clear_dungeon_map_init_state();
        } else {
            self.DungeonMap_HandleMovementInput();
        }
    }

    fn WantExitDungeonMap(&self) -> bool {
        if self.game_state.world.transient.hud_cur_item_x() != 0 {
            self.game_state.player.follower_link.filtered_joypad_h() & 0x20 != 0
        } else {
            self.game_state.player.follower_link.filtered_joypad_l() & 0x40 != 0
        }
    }

    pub(super) fn DungeonMap_HandleMovementInput(&mut self) {
        self.DungeonMap_HandleFloorSelect();
        if self
            .game_state
            .dungeon_map_display
            .dungmap_floor_scroll_step()
            != 0
        {
            self.DungeonMap_ScrollFloors();
        }
    }

    pub(super) fn DungeonMap_HandleFloorSelect(&mut self) {
        let t5 = DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON[usize::from(
            self.game_state.inventory.save_progress.palace_index_x2() >> 1,
        )
        .min(DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON.len() - 1)];
        let r2 = ((t5 >> 4) & 0x0f) as u8;
        let r3 = (t5 & 0x0f) as u8;
        if r2.wrapping_add(r3) < 3
            || self
                .game_state
                .dungeon_map_display
                .dungmap_floor_scroll_step()
                != 0
            || (self.game_state.player.follower_link.joypad1h_last() & 0x0c) == 0
        {
            return;
        }

        self.dungeon_map_mut().clear_current_floor_high();
        let mut scroll_draw_offset = self.game_state.dungeon_map_display.scroll_draw_offset();
        if (self.game_state.player.follower_link.joypad1h_last() & 8) != 0 {
            if r2.wrapping_sub(1) == self.game_state.dungeon_map_display.dungmap_cur_floor_byte() {
                return;
            }
            self.increment_dungeon_map_current_floor_byte();
            scroll_draw_offset = scroll_draw_offset.wrapping_sub(0x300) & 0x0fff;
        } else {
            if (!r3).wrapping_add(1) == self.game_state.dungeon_map_display.dungmap_cur_floor_byte()
            {
                return;
            }
            let new_floor = self
                .game_state
                .dungeon_map_display
                .dungmap_cur_floor()
                .wrapping_sub(2);
            self.set_dungeon_map_current_floor(new_floor);
            scroll_draw_offset = scroll_draw_offset.wrapping_add(0x600) & 0x0fff;
        }

        self.DungeonMap_DrawFloorNumbersByRoom(scroll_draw_offset, !0x1000);
        self.DungeonMap_DrawBorderForRooms(scroll_draw_offset, !0x1000);
        self.DungeonMap_DrawDungeonLayout(scroll_draw_offset as i32);
        self.increment_dungeon_map_floor_scroll_step();
        let joypad_h = self.game_state.player.follower_link.joypad1h_last();
        self.set_dungeon_map_scroll_input(u16::from(joypad_h));
        let x = usize::from((joypad_h >> 3) & 1);
        let target = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg2_v_copy2()
            .wrapping_add_signed(DUNGEON_MAP_FLOOR_SCROLL_TARGET_DELTAS[x]);
        self.set_dungeon_map_scroll_target_y(target);
        if x == 0 {
            scroll_draw_offset = scroll_draw_offset.wrapping_sub(0x300) & 0x0fff;
            self.increment_dungeon_map_current_floor_byte();
        }
        self.set_dungeon_map_scroll_draw_offset(scroll_draw_offset);
        self.set_pending_nmi_subroutine(8);
    }

    pub(super) fn DungeonMap_ScrollFloors(&mut self) {
        let x = self
            .game_state
            .dungeon_map_display
            .scroll_input_direction_index();
        let marker_y = self
            .game_state
            .dungeon_map_display
            .dungmap_player_marker_y()
            .wrapping_add_signed(i16::from(DUNGEON_MAP_SCROLL_MARKER_Y_DELTAS[x]));
        self.set_dungeon_map_player_marker_y(marker_y);
        self.add_dungeon_map_marker_y_offset_signed(i16::from(
            DUNGEON_MAP_SCROLL_MARKER_Y_DELTAS[x],
        ));
        let bg2 = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg2_v_copy2()
            .wrapping_add_signed(i16::from(DUNGEON_MAP_SCROLL_BG_Y_DELTAS[x]));
        self.set_bg2_y(bg2);
        if bg2
            == self
                .game_state
                .dungeon_map_display
                .dungmap_scroll_target_y()
        {
            self.clear_dungeon_map_floor_scroll_step();
        }
    }

    pub(super) fn DungeonMap_DrawSprites(&mut self) {
        let dung = usize::from(self.game_state.inventory.save_progress.palace_index_x2() >> 1)
            .min(DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON.len() - 1);
        let r2 = (DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON[dung] & 0x0f) as u8;
        let floor = r2.wrapping_add(self.game_state.dungeon.stair_movement.current_floor());

        let mut spr_pos = 0usize;
        let mut r14 = 0u16;
        self.DungeonMap_DrawLinkPointing(spr_pos, r2, floor);
        spr_pos += 1;
        loop {
            spr_pos = self.DungeonMap_DrawLocationMarker(spr_pos, r14);
            r14 = r14.wrapping_add(1);
            if spr_pos == 9 {
                break;
            }
        }
        spr_pos = self.DungeonMap_DrawBlinkingIndicator(spr_pos);
        spr_pos = self.DungeonMap_DrawBossIcon(spr_pos);
        let _ = self.DungeonMap_DrawFloorNumberObjects(spr_pos);
        self.DungeonMap_DrawFloorBlinker();
    }

    pub(super) fn DungeonMap_DrawLinkPointing(&mut self, spr_pos: usize, r2: u8, mut r3: u8) {
        let dung = usize::from(self.game_state.inventory.save_progress.palace_index_x2() >> 1)
            .min(DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON.len() - 1);
        let t5 = DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON[dung] as u8;
        if 4i8.wrapping_sub(r2 as i8) >= 0 {
            r3 = r3.wrapping_add(4u8.wrapping_sub(r2));
            let a = ((t5 >> 4) as i8).wrapping_sub(4);
            if a >= 0 {
                r3 = r3.wrapping_sub(a as u8);
            }
        }
        let y = DUNGEON_MAP_FLOOR_Y_POSITIONS[usize::from(r3)].wrapping_sub(4);
        let flags = if self.palette_swap_enabled() {
            0x30
        } else {
            0x3e
        };
        self.set_oam_plain(spr_pos, 0x19, y, 0, flags, 2);
    }

    pub(super) fn DungeonMap_DrawBlinkingIndicator(&mut self, spr_pos: usize) -> usize {
        let marker_y = self
            .game_state
            .dungeon_map_display
            .dungmap_player_marker_y();
        let y = if marker_y < 256 { marker_y as u8 } else { 0xf0 }.wrapping_sub(3);
        self.set_oam_plain(
            spr_pos,
            self.game_state
                .dungeon_map_display
                .dungmap_player_marker_x_byte()
                .wrapping_sub(3),
            y,
            0x34,
            DUNGEON_MAP_PLAYER_MARKER_OAM_FLAGS
                [usize::from((self.game_state.frame.frame_counter >> 2) & 3)],
            0,
        );
        spr_pos + 1
    }

    pub(super) fn DungeonMap_DrawLocationMarker(&mut self, mut spr_pos: usize, r14: u16) -> usize {
        for i in (0..4).rev() {
            let r15 = self
                .game_state
                .dungeon_map_display
                .location_marker_base_y()
                .wrapping_add(DUNGEON_MAP_MARKER_Y_BASES[usize::from(r14)] as u8);
            let mut fr = (self.game_state.frame.frame_counter >> 2) & 1;
            let marker_y = self
                .game_state
                .dungeon_map_display
                .dungmap_player_marker_y();
            if ((marker_y.wrapping_add(1)) & 0x00f0) == u16::from(r15.wrapping_add(1))
                && marker_y < 256
            {
                fr = fr.wrapping_add(2);
            }
            let x = (self
                .game_state
                .dungeon_map_display
                .dungmap_player_marker_x()
                & 0x00f0)
                .wrapping_add_signed(i16::from(DUNGEON_MAP_LOCATION_MARKER_X_OFFSETS[i]))
                as u8;
            let y = u16::from(r15)
                .wrapping_add_signed(i16::from(DUNGEON_MAP_LOCATION_MARKER_Y_OFFSETS[i]))
                as u8;
            self.set_oam_plain(
                spr_pos,
                x,
                y,
                0,
                DUNGEON_MAP_LOCATION_MARKER_CHARS[usize::from(fr)]
                    | DUNGEON_MAP_LOCATION_MARKER_OAM_FLAGS[i],
                2,
            );
            spr_pos += 1;
        }
        spr_pos
    }

    pub(super) fn DungeonMap_DrawFloorNumberObjects(&mut self, spr_pos: usize) -> usize {
        let t5 = DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON[usize::from(
            self.game_state.inventory.save_progress.palace_index_x2() >> 1,
        )
        .min(DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON.len() - 1)];
        let mut r2 = ((t5 >> 4) & 0x0f) as u8;
        let mut r3 = (t5 & 0x0f) as u8;
        let mut yv = 7u8;
        if r2.wrapping_add(r3) != 8 && r2 < 4 {
            yv = 6;
            let mut i = 3u8;
            while i != 0 && i != r2 {
                yv = yv.wrapping_sub(1);
                i = i.wrapping_sub(1);
            }
            if r3 >= 5 {
                let mut i = 5u8;
                while i != r3 && r3 != 8 {
                    yv = yv.wrapping_add(1);
                    i = i.wrapping_add(1);
                }
            }
        }

        let mut r4 = DUNGEON_MAP_FLOOR_Y_POSITIONS[usize::from(yv)].wrapping_add(1);
        r2 = r2.wrapping_sub(1);
        r3 = 0u8.wrapping_sub(r3);
        let mut pos = spr_pos;
        loop {
            let left = if (r2 as i8) < 0 {
                0x1c
            } else {
                DUNGEON_MAP_FLOOR_DIGIT_CHARS[usize::from(r2)]
            };
            let right = if (r2 as i8) < 0 {
                DUNGEON_MAP_FLOOR_DIGIT_CHARS[usize::from(r2 ^ 0xff)]
            } else {
                0x1d
            };
            self.set_oam_plain(pos, 0x30, r4, left, 0x3d, 0);
            self.set_oam_plain(pos + 1, 0x38, r4, right, 0x3d, 0);
            r4 = r4.wrapping_add(16);
            let done = r2 == r3;
            pos += 2;
            r2 = r2.wrapping_sub(1);
            if done {
                break;
            }
        }
        pos
    }

    pub(super) fn DungeonMap_DrawFloorBlinker(&mut self) {
        let mut floor = self.game_state.dungeon_map_display.dungmap_cur_floor_byte();
        let t5 = DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON[usize::from(
            self.game_state.inventory.save_progress.palace_index_x2() >> 1,
        )
        .min(DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON.len() - 1)] as u8;
        let mut flag = u8::from(((t5 >> 4) & 0x0f).wrapping_add(t5 & 0x0f) != 1);
        floor = floor.wrapping_sub(flag);
        let mut r0;
        let mut i = flag;
        loop {
            r0 = floor.wrapping_add(t5 & 0x0f);
            let a = 4i8.wrapping_sub((t5 & 0x0f) as i8);
            if a >= 0 {
                r0 = r0.wrapping_add(a as u8);
                let a = ((t5 >> 4) as i8).wrapping_sub(4);
                if a >= 0 {
                    r0 = r0.wrapping_sub(a as u8);
                }
            }
            floor = floor.wrapping_add(1);
            if i == 0 {
                break;
            }
            i = i.wrapping_sub(1);
        }
        if (self.game_state.frame.frame_counter & 0x10) == 0 {
            return;
        }
        let y = DUNGEON_MAP_FLOOR_Y_POSITIONS[usize::from(r0)].wrapping_sub(4);
        loop {
            let mut x = 40u8;
            let mut spr_pos =
                0x40 + usize::from(DUNGEON_MAP_FLOOR_BLINKER_SPRITE_OFFSETS[usize::from(flag)]);
            for i in (0..4).rev() {
                let t = 0x3d | if i != 0 { 0 } else { 0x40 };
                self.set_oam_plain(
                    spr_pos,
                    x,
                    y.wrapping_add(flag.wrapping_mul(16)),
                    DUNGEON_MAP_FLOOR_BLINKER_CHARS[i],
                    t,
                    0,
                );
                self.set_oam_plain(
                    spr_pos + 4,
                    x,
                    y.wrapping_add(flag.wrapping_mul(16)).wrapping_add(8),
                    DUNGEON_MAP_FLOOR_BLINKER_CHARS[i],
                    t | 0x80,
                    0,
                );
                x = x.wrapping_add(8);
                spr_pos += 1;
            }
            if flag == 0 {
                break;
            }
            flag = flag.wrapping_sub(1);
        }
    }

    pub(super) fn DungeonMap_DrawBossIcon(&mut self, spr_pos: usize) -> usize {
        let dung = usize::from(self.game_state.inventory.save_progress.palace_index_x2() >> 1)
            .min(DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON.len() - 1);
        if (self
            .game_state
            .inventory
            .save_progress
            .dungeon_info_word(usize::from(DUNGEON_MAP_BOSS_ROOM_BY_DUNGEON[dung]))
            & 0x0800)
            != 0
            || !self
                .game_state
                .inventory
                .player_resources
                .has_compass_mask(DUNG_MAP_UPPER_BITMASKS[dung & 0x0f])
            || DUNGEON_MAP_BOSS_FLOOR_OFFSETS[dung] < 0
        {
            return spr_pos;
        }
        let spr_pos = self.DungeonMap_DrawBossIconByFloor(spr_pos);
        if (self.game_state.frame.frame_counter & 0x0f) >= 10 {
            return spr_pos;
        }
        let xy = DUNGEON_MAP_BOSS_ICON_XY_BY_DUNGEON[dung];
        let x = (xy >> 8)
            .wrapping_add(self.game_state.dungeon_map_display.marker_x_offset())
            .wrapping_add(0x90) as u8;
        let marker_y_offset = self.game_state.dungeon_map_display.marker_y_offset();
        let y = if marker_y_offset < 256 {
            xy.wrapping_add(marker_y_offset) as u8
        } else {
            0xf0
        };
        self.set_oam_plain(spr_pos, x, y, 0x31, 0x33, 0);
        spr_pos + 1
    }

    pub(super) fn DungeonMap_DrawBossIconByFloor(&mut self, spr_pos: usize) -> usize {
        let dung = usize::from(self.game_state.inventory.save_progress.palace_index_x2() >> 1)
            .min(DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON.len() - 1);
        let t5 = DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON[dung] as u8;
        let r2 = t5 & 0x0f;
        let mut r3 = r2.wrapping_add(DUNGEON_MAP_BOSS_FLOOR_OFFSETS[dung] as u8);
        if 4i8.wrapping_sub(r2 as i8) >= 0 {
            r3 = r3.wrapping_add(4u8.wrapping_sub(r2));
            let a = ((t5 >> 4) as i8).wrapping_sub(4);
            if a >= 0 {
                r3 = r3.wrapping_sub(a as u8);
            }
        }
        if (self.game_state.frame.frame_counter & 0x0f) >= 10 {
            return spr_pos;
        }
        self.set_oam_plain(
            spr_pos,
            0x4c,
            DUNGEON_MAP_FLOOR_Y_POSITIONS[usize::from(r3)],
            0x31,
            0x33,
            0,
        );
        spr_pos + 1
    }

    pub(super) fn DungeonMap_RecoverGFX(&mut self) {
        if self.rom_startup_timing() {
            self.game_execution_scheduler.schedule_work(
                GameWorkContinuation::FinishDungeonMapRecovery,
                DUNGEON_MAP_RECOVERY_NMI_SLICES,
            );
            return;
        }
        self.complete_dungeon_map_recovery();
    }

    pub(super) fn complete_dungeon_map_recovery(&mut self) {
        let hdmaen_bak = self.game_state.display.hdma_enable_mask;
        self.clear_hdma_enable_mask();
        self.EraseTileMaps_normal();

        let main_screen_layers = self.game_state.display.ppu_scroll_copy.mapbak_tm();
        let sub_screen_layers = self.game_state.display.ppu_scroll_copy.mapbak_ts();
        self.set_main_screen_layers(main_screen_layers);
        self.set_sub_screen_layers(sub_screen_layers);
        let main_tile_theme = self
            .game_state
            .display
            .ppu_scroll_copy
            .mapbak_main_tile_theme_index();
        self.world_palette_theme_mut()
            .set_main_tile_theme_index(main_tile_theme);
        let graphics_index = self
            .game_state
            .display
            .ppu_scroll_copy
            .mapbak_sprite_graphics_index();
        self.sprite_system_mut().set_graphics_index(graphics_index);
        let aux_tile_theme = self
            .game_state
            .display
            .ppu_scroll_copy
            .mapbak_aux_tile_theme_index();
        self.world_palette_theme_mut()
            .set_aux_tile_theme_index(aux_tile_theme);
        self.InitializeTilesets();
        self.clear_overworld_aux_or_main_offset();
        self.set_hud_palette(0);
        self.hud_rebuild();

        self.clear_screen_transition();
        self.dungeon_room_load_mut().clear_quadrant_upload_index();
        loop {
            self.WaterFlood_BuildOneQuadrantForVRAM();
            self.upload_tilemap_now();
            self.Dungeon_PrepareNextRoomQuadrantUpload();
            self.upload_tilemap_now();
            if self.game_state.dungeon.room_load.quadrant_upload_index() == 0x10 {
                break;
            }
        }

        self.clear_pending_nmi_subroutine();
        self.set_subsubmodule(0);
        self.set_hdma_enable_mask(hdmaen_bak);
        let mapbak_palette = self
            .game_state
            .display
            .ppu_scroll_copy
            .mapbak_palette_slice()
            .to_vec();
        self.copy_main_full_from_tagged(
            &mapbak_palette,
            crate::game_state::PaletteSliceSource::MirrorBank(zelda3_palette::Bank::Backup),
        );
        let fixed_color_plusminus = self.game_state.dungeon.room_effects.fixed_color_plusminus();
        self.or_fixed_color_red(fixed_color_plusminus);
        self.or_fixed_color_green(fixed_color_plusminus);
        self.or_fixed_color_blue(fixed_color_plusminus);

        self.set_sound_effect_2(16);
        self.set_music_control(0xf3);
        self.RecoverPegGFXFromMapping();
        self.increment_cgram_update_flag();
        self.increment_overworld_map_state();
        self.set_screen_brightness(0);
        self.clear_core_update_disable_flag();
    }

    pub(super) fn ToggleStarTilesAndAdvance(&mut self) {
        self.ResetStarTileGraphics();
        self.increment_overworld_map_state();
    }

    pub(super) fn DungMap_4(&mut self) {
        let scroll_target = self
            .game_state
            .dungeon_map_display
            .dungmap_scroll_target_y();
        let y = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg2_v_copy2()
            .wrapping_add(scroll_target);
        self.set_bg2_y(y);
        let marker_y = self
            .game_state
            .dungeon_map_display
            .dungmap_player_marker_y()
            .wrapping_sub(scroll_target);
        self.set_dungeon_map_player_marker_y(marker_y);
        let new_row = self.decrement_bottle_menu_row();
        if new_row == 0 {
            let overworld_map_state = self.overworld_map_state().wrapping_sub(1);
            self.set_overworld_map_state(overworld_map_state);
        }
    }

    pub(super) fn DungMap_LightenUpMap(&mut self) {
        self.increment_screen_brightness();
        if self.game_state.display.screen_brightness == 0x0f {
            self.increment_overworld_map_state();
        }
    }

    pub(super) fn DungMap_Backup(&mut self) {
        self.decrement_screen_brightness();
        if self.game_state.display.screen_brightness != 0 {
            return;
        }
        self.set_mosaic_copy(3);
        let hdmaen = self.game_state.display.hdma_enable_mask;
        self.set_mapbak_hdmaen(hdmaen);
        if let Some(scanline) = self
            .last_sprite_main_timing_workload
            .and_then(SpriteMainTimingWorkload::dungeon_map_backup_force_blank_output_scanline)
        {
            self.enable_force_blank_during_active_scanout(scanline);
        } else {
            self.EnableForceBlank();
        }
        self.increment_overworld_map_state();
        self.clear_dungeon_map_init_state();
        self.set_fixed_color_red(0x20);
        self.set_fixed_color_green(0x40);
        self.set_fixed_color_blue(0x80);
        self.follower_link_state_mut()
            .set_link_dma_graphics_index_word(0x0250);
        let palette = self
            .game_state
            .display
            .palette_buffer
            .main_full_slice()
            .to_vec();
        self.copy_mapbak_palette_from(
            &palette,
            crate::game_state::PaletteSliceSource::MirrorBank(zelda3_palette::Bank::Main),
        );
        let bg1_x_offset = self.game_state.world.scroll.bg1_x_offset();
        let bg1_y_offset = self.game_state.world.scroll.bg1_y_offset();
        self.set_mapbak_bg1_x_offset(bg1_x_offset);
        self.set_mapbak_bg1_y_offset(bg1_y_offset);
        self.set_bg1_x_offset(0);
        self.set_bg1_y_offset(0);
        let bg1hofs = self.game_state.display.ppu_scroll_copy.bg1_h_copy2();
        let bg2hofs = self.game_state.display.ppu_scroll_copy.bg2_h_copy2();
        let bg1vofs = self.game_state.display.ppu_scroll_copy.bg1_v_copy2();
        let bg2vofs = self.game_state.display.ppu_scroll_copy.bg2_v_copy2();
        self.set_map_backup_scrolls(bg1hofs, bg2hofs, bg1vofs, bg2vofs);
        self.set_bg1_x(0);
        self.set_bg1_y(0);
        self.set_bg2_x(0);
        self.set_bg2_y(0);
        self.set_bg3_h_copy2(0);
        self.set_bg3_v_copy2(0);
        let cgwsel = self
            .game_state
            .display
            .palette_filter
            .color_window_and_math_word();
        self.set_mapbak_cgwsel_word(cgwsel);
        self.set_color_window_selection(0x02);
        self.set_color_math_control(0x20);
        self.fill_messaging_render_buffer_word_range(0, 2048, 0x0300);
        self.set_sound_effect_2(16);
        self.set_music_control(0xf2);
    }

    pub(super) fn DungMap_FadeMapToBlack(&mut self) {
        self.decrement_screen_brightness();
        if self.game_state.display.screen_brightness != 0 {
            return;
        }
        self.EnableForceBlank();
        self.increment_overworld_map_state();
        let cgwsel = self.game_state.display.ppu_scroll_copy.mapbak_cgwsel_word();
        let bg1hofs = self
            .game_state
            .display
            .ppu_scroll_copy
            .map_backup_bg1_h_copy2();
        let bg2hofs = self
            .game_state
            .display
            .ppu_scroll_copy
            .map_backup_bg2_h_copy2();
        let bg1vofs = self
            .game_state
            .display
            .ppu_scroll_copy
            .map_backup_bg1_v_copy2();
        let bg2vofs = self
            .game_state
            .display
            .ppu_scroll_copy
            .map_backup_bg2_v_copy2();
        self.set_color_window_and_math_word(cgwsel);
        self.set_bg1_x(bg1hofs);
        self.set_bg2_x(bg2hofs);
        self.set_bg1_y(bg1vofs);
        self.set_bg2_y(bg2vofs);
        self.set_bg3_v_copy2(0);
        self.set_bg3_h_copy2(0);
        let bg1_x_offset = self
            .game_state
            .display
            .ppu_scroll_copy
            .mapbak_bg1_x_offset();
        let bg1_y_offset = self
            .game_state
            .display
            .ppu_scroll_copy
            .mapbak_bg1_y_offset();
        self.set_bg1_x_offset(bg1_x_offset);
        self.set_bg1_y_offset(bg1_y_offset);
        self.increment_cgram_update_flag();
    }

    pub(super) fn DungMap_RestoreOld(&mut self) {
        self.OrientLampLightCone();
        self.increment_screen_brightness();
        if self.game_state.display.screen_brightness != 0x0f {
            return;
        }
        let saved_module = self.game_state.frame.saved_module_for_menu;
        self.set_main_module(saved_module);
        self.set_submodule(0);
        self.set_overworld_map_state(0);
        self.set_subsubmodule(0);
        self.set_screen_brightness(0x0f);
        let hdma_enable_mask = self.game_state.display.ppu_scroll_copy.mapbak_hdmaen();
        self.set_hdma_enable_mask(hdma_enable_mask);
    }

    pub(super) fn Death_InitializeGameOverLetters(&mut self) {
        self.minigame_state_mut().set_flag_boomerang_in_place(0);
        for i in 0..8 {
            self.ancilla_slot_view_mut(i).set_x(0xb0);
        }
        self.ancilla_slot_view_mut(0).set_ancilla_type(1);
        self.messaging_state_mut().set_game_over_letter_cursor(6);
    }

    pub(super) fn CopySaveToWRAM(&mut self) {
        let k = 0x0f;
        self.clear_bird_travel_destination(k);
        self.clear_bird_travel_stop_status(k);

        let save_offset = self.game_state.save_load_transfer.source_offset_usize();
        if save_offset + 0x500 <= self.sram.len() {
            let save = self.sram[save_offset..save_offset + 0x500].to_vec();
            self.save_progress_mut().copy_dungeon_info_from(&save);
        }

        self.set_bg_tile_animation_countdown(7);
        self.follower_link_state_mut()
            .reset_link_dma_animation_cycle(7);
        self.set_message_dma_destination_address(0x6040);
        self.set_message_dma_tile_base(0x4841);
        self.set_message_dma_tile_limit(0x007f);
        self.set_message_dma_tile_sentinel(0xffff);
        if self
            .game_state
            .enhanced_features
            .has(SAVE_LOAD_MISC_BUG_FIXES_FLAG)
        {
            self.clear_mosaic_level();
        }

        self.save_progress_mut().request_post_message_refresh();
        self.set_main_module(5);
        self.set_submodule(0);
        self.set_which_entrance(0);
        self.clear_core_update_disable_flag();
        self.set_hud_palette(0);
    }

    pub(super) fn RenderText(&mut self) {
        match self.game_state.messaging.runtime.module() {
            0 => self.Text_Initialize(),
            1 => self.Text_Render(),
            2 => self.RenderText_PostDeathSaveOptions(),
            _ => {}
        }
    }

    pub(super) fn RenderText_PostDeathSaveOptions(&mut self) {
        self.dialogue_message_index_mut().set_value(3);
        self.Text_Initialize_initModuleStateLoop();
        self.messaging_state_mut().set_text_msgbox_topleft(0x61e8);
        self.messaging_state_mut().set_text_render_state(2);
        for _ in 0..5 {
            self.Text_Render();
        }
    }

    pub(super) fn Text_Initialize(&mut self) {
        if self.pending_dialogue_initialization_schedule.is_some()
            && self.rom_startup_timing()
            && self.original_timing_main_loop_iteration_returned_to_wait()
        {
            // The wire proves this Module0E iteration returns to its main
            // wait with a completed suffix: the ROM's Text_Initialize ran
            // only its cheap first incremental piece before returning, and
            // the long suspension begins on the NEXT iteration. Keep the
            // schedule armed for that iteration instead of suspending this
            // one (route host 154788, the post-game-over dialogue).
            return;
        }
        if let Some((
            prefix_nmi_crossings,
            caller_nmi_crossings,
            following_main_nmi_uses_host_animated_bg_operands,
        )) = self
            .pending_dialogue_initialization_schedule
            .take()
            .filter(|_| self.rom_startup_timing())
        {
            assert_ne!(prefix_nmi_crossings, 0);
            if let Some(GameWorkContinuation::FinishItemReceiptGraphics {
                continuation:
                    continuation @ ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { .. },
            }) = self.game_execution_scheduler.current_work()
            {
                // The atomic item-receipt decompression tail holds only the
                // NMI latch — its caller already returned and its remaining
                // held vblank uploads ride the dialogue-initialization
                // window's own held slices (route host 158014, the pendant
                // receipt's message). Retire the one scheduler slot for the
                // dialogue suspension; the enemy-drop sound retire is the
                // tail's only remaining semantic effect.
                self.game_execution_scheduler.finish_work();
                if matches!(
                    continuation,
                    ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { gfx: 0x22, .. }
                ) {
                    self.retire_enemy_drop_item_graphics_sound_effect_2();
                }
            }
            self.normal_dialogue_following_main_nmi_uses_host_animated_bg_operands =
                following_main_nmi_uses_host_animated_bg_operands;
            self.game_execution_scheduler
                .schedule_cpu_timed_work_returning_on_later_host(
                    GameWorkContinuation::FinishDialogueInitializationPrefix {
                        caller_nmi_crossings,
                    },
                    prefix_nmi_crossings,
                );
            // This slice and every held one until the completion slice keep
            // the host-boundary OAM shadow on screen (see the staging fn).
            self.stage_dialogue_initialization_obj_scanout();
            return;
        }
        self.complete_text_initialization_prefix();
        self.complete_text_initialization_suffix();
    }

    pub(super) fn complete_text_initialization_prefix(&mut self) {
        if self.game_state.frame.main_module == 20 {
            self.ResetHUDPalettes4and5();
        }
        self.Attract_DecompressStoryGFX();
        self.text_initialize_module_state_prefix();
    }

    /// The opening attract sequence has already reset its HUD palette and
    /// stages story-GFX decompression on the following NMI slice. Publish only
    /// the text state here so its BG3 update remains independently timed.
    pub(super) fn complete_text_initialization_state_prefix(&mut self) {
        self.text_initialize_module_state_prefix();
    }

    pub(super) fn Text_Initialize_initModuleStateLoop(&mut self) {
        self.text_initialize_module_state_prefix();
        self.complete_text_initialization_suffix();
    }

    fn text_initialize_module_state_prefix(&mut self) {
        // C copies all 32 bytes of TEXT_INITIALIZATION_DATA into the message-state struct at
        // TEXT_MSGBOX_TOPLEFT_COPY (0x1cd0..0x1cf0). init_msgbox_state_from only models a
        // subset, leaving unmodeled bytes (notably DIALOGUE_MSG_SRC_OFFS 0x1cdd-0x1cde, a dead
        // message-DMA-pointer scratch) stale. Mirror C's raw copy so those bytes match too;
        // the native fields re-project their (identical) values afterward.
        self.ram[crate::game_state::constants::TEXT_MSGBOX_TOPLEFT_COPY
            ..crate::game_state::constants::TEXT_MSGBOX_TOPLEFT_COPY
                + TEXT_INITIALIZATION_DATA.len()]
            .copy_from_slice(&TEXT_INITIALIZATION_DATA);
        self.messaging_state_mut()
            .init_msgbox_state_from(&TEXT_INITIALIZATION_DATA);
        self.clear_bg3_vwf_glyph_runs();
        self.Text_InitVwfState();
        // A fresh message's render machine cannot inherit the previous
        // message's suspended-glyph CPU model: the C main thread is inside
        // Text_Initialize here, not the VWF glyph loop. A stale hold routed
        // the initialization's caller-return host into the VWF terminal
        // branch (route host 103733).
        self.dialogue_vwf_handler_completed_at_endpoint = false;
        self.dialogue_fast_forward_hold_active = false;
        self.dialogue_fast_forward_hold_pending = false;
        self.RenderText_SetDefaultWindowPosition();
        self.messaging_state_mut().set_text_tilemap_cur(0x3980);
    }

    pub(super) fn complete_text_initialization_suffix(&mut self) {
        self.Text_LoadCharacterBuffer();
        self.finish_text_initialization_after_character_buffer();
    }

    pub(super) fn prepare_text_character_buffer_for_carry(&mut self) {
        let encoded_len = self.current_encoded_dialogue_len();
        self.Text_LoadCharacterBuffer();
        self.messaging_state_mut()
            .set_dialogue_msg_read_pos(encoded_len);
    }

    pub(super) fn complete_text_initialization_carry_suffix(&mut self) {
        self.messaging_state_mut().clear_dialogue_msg_read_pos();
        self.finish_text_initialization_after_character_buffer();
    }

    fn finish_text_initialization_after_character_buffer(&mut self) {
        self.clear_messaging_render_buffer_range(0x7e0);
        self.set_pending_nmi_subroutine(2);
        self.set_core_update_disable_flag(2);
    }

    fn current_encoded_dialogue_len(&self) -> u16 {
        let Some(dialogue_blk) = self.asset_memblk(94, self.dialogue_blk_index) else {
            return 0;
        };
        let dialogue = find_index_in_memblk(dialogue_blk, 1);
        let text_index = self.game_state.messaging.dialogue_message_index.value() as usize;
        (find_index_in_memblk(dialogue, text_index).ptr.len() as u16)
            .min(ROM_TEXT_DECODE_FIRST_SLICE_CURSOR)
    }

    pub(super) fn Text_InitVwfState(&mut self) {
        self.set_vwf_current_line(0);
        self.clear_vwf_next_line_request();
        self.clear_vwf_glyph_cursor();
        self.set_vwf_line_render_offset(0);
    }

    pub(super) fn Text_DecodeCmd(&self, a: u8, src: &[u8]) -> u32 {
        let (param, cmd, multibyte) = self.text_decode_cmd(a, src.first().copied());
        ((param as u32) << 6) | ((cmd as u32) << 1) | u32::from(multibyte)
    }

    fn text_decode_cmd(&self, a: u8, next: Option<u8>) -> (u8, u8, bool) {
        let decoded = crate::dialogue_ir::decode_dialogue_byte(self.dialogue_flags, a, next);
        (decoded.param, decoded.command, decoded.multibyte)
    }

    pub(super) fn Text_LoadCharacterBuffer(&mut self) {
        let Some(dialogue_blk) = self.asset_memblk(94, self.dialogue_blk_index) else {
            return;
        };
        let dictionary = find_index_in_memblk(dialogue_blk, 0).ptr.to_vec();
        let dialogue = find_index_in_memblk(dialogue_blk, 1).ptr.to_vec();
        let text_index = self.game_state.messaging.dialogue_message_index.value() as usize;
        let text_str = find_index_in_memblk(MemBlk { ptr: &dialogue }, text_index)
            .ptr
            .to_vec();

        let mut src = 0usize;
        let mut decoded = Vec::new();
        // C's Text_WritePlayerName writes all 6 name chars (including trailing 0x59 blanks)
        // to the buffer, then returns a pointer advanced by only the *effective* length
        // (trimming trailing blanks). Subsequent text overwrites from effective_len onward,
        // leaving any blanks beyond that position in the buffer. We track (name_start,
        // name_end_6) for each NAME command so we can re-inject those trailing blanks
        // after the loop exactly as C leaves them.
        let mut name_ranges: Vec<(usize, usize)> = Vec::new(); // (effective_end, full_end)
        while src < text_str.len() {
            let c = text_str[src];
            src += 1;
            if c >= TEXT_DICT_BASE {
                let blk = find_index_in_memblk(
                    MemBlk { ptr: &dictionary },
                    (c - TEXT_DICT_BASE) as usize,
                );
                decoded.extend_from_slice(blk.ptr);
                continue;
            }
            let (param, cmd, multibyte) = self.text_decode_cmd(c, text_str.get(src).copied());
            match cmd {
                TEXT_CMD_NAME => {
                    // C writes all 6 name chars then advances dst by effective_len only.
                    // We write all 6 to decoded, truncate to effective_len, and record the
                    // range [effective_end, full_end=effective_end+(6-effective_len)] so we
                    // can re-inject the trailing blanks after the loop (matching C).
                    let effective_len = self.text_write_player_name_vec_full(&mut decoded);
                    let full_end = decoded.len(); // effective_len + 6
                    let effective_end = full_end - (6 - effective_len);
                    decoded.truncate(effective_end);
                    name_ranges.push((effective_end, full_end));
                }
                TEXT_CMD_WINDOW => self.messaging_state_mut().set_text_render_state(param),
                TEXT_CMD_NUMBER => {
                    let v = self
                        .game_state
                        .messaging
                        .dialogue_number
                        .packed_digits((param >> 1) as usize);
                    decoded.push(0x34 + if param & 1 != 0 { v >> 4 } else { v & 0x0f });
                }
                TEXT_CMD_POSITION => {
                    self.messaging_state_mut()
                        .set_text_msgbox_topleft(TEXT_POSITIONS[param as usize & 1]);
                }
                TEXT_CMD_COLOR => {
                    let value = ((0x387f & 0xe300) | 0x180) | (((param as u16) << 10) & 0x3c00);
                    self.messaging_state_mut().set_text_tilemap_cur(value);
                }
                _ => {
                    decoded.push(c);
                    if multibyte {
                        if let Some(next) = text_str.get(src) {
                            decoded.push(*next);
                        }
                    }
                }
            }
            if multibyte {
                src += 1;
            }
        }
        decoded.push(0x7f);
        // Re-inject trailing 0x59 blanks from player-name writes that were not overwritten
        // by subsequent text. In C, Text_WritePlayerName writes all 6 chars at p[0..6] and
        // returns p+effective_len; positions [effective_len..6] remain as 0x59 unless later
        // text overwrites them. We reproduce that by appending the "orphaned" blanks now.
        for (_effective_end, full_end) in name_ranges {
            let leftover = full_end.saturating_sub(decoded.len());
            for _ in 0..leftover {
                decoded.push(0x59);
            }
        }
        self.messaging_text_mut().load_decoded_dialogue(&decoded);
        self.messaging_state_mut().clear_dialogue_msg_read_pos();
    }

    pub(super) fn Text_WritePlayerName(&mut self, dst: usize) -> usize {
        let mut decoded = Vec::new();
        self.text_write_player_name_vec(&mut decoded);
        let len = self
            .messaging_text_mut()
            .write_decoded_text_at(dst, &decoded);
        dst + len
    }

    /// Build the 6-char player name buffer from SRAM (all 6 entries, including trailing 0x59
    /// blanks). Returns the *effective* length (trailing blanks trimmed) so the caller can
    /// replicate C's behaviour: C writes all 6 chars at p[0..6] then returns p+effective_len.
    fn text_write_player_name_vec_full(&self, decoded: &mut Vec<u8>) -> usize {
        let slot = self.selected_save_slot_byte();
        let offs = (((slot >> 1) as isize) - 1) * 0x500;
        let start = 0x3d9isize + offs;
        let mut name = [0u8; 6];
        for (i, ch) in name.iter_mut().enumerate() {
            let p = start + (i as isize) * 2;
            let a = if p >= 0 && (p as usize) + 1 < self.sram.len() {
                read_le_u16(&self.sram, p as usize)
            } else {
                0
            };
            *ch = self.Text_FilterPlayerNameCharacters((a & 0x0f | (a >> 1) & 0xf0) as u8);
        }
        // Write all 6 chars (C writes p[0..6] unconditionally)
        decoded.extend_from_slice(&name);
        // Compute effective length (trailing 0x59 blanks trimmed) so caller knows where to
        // truncate decoded and where subsequent text should continue from.
        let mut effective_len = name.len();
        while effective_len != 0 && name[effective_len - 1] == 0x59 {
            effective_len -= 1;
        }
        effective_len
    }

    pub(crate) fn text_write_player_name_vec(&self, decoded: &mut Vec<u8>) {
        let slot = self.selected_save_slot_byte();
        let offs = (((slot >> 1) as isize) - 1) * 0x500;
        let start = 0x3d9isize + offs;
        let mut name = [0u8; 6];
        for (i, ch) in name.iter_mut().enumerate() {
            let p = start + (i as isize) * 2;
            let a = if p >= 0 && (p as usize) + 1 < self.sram.len() {
                read_le_u16(&self.sram, p as usize)
            } else {
                0
            };
            *ch = self.Text_FilterPlayerNameCharacters((a & 0x0f | (a >> 1) & 0xf0) as u8);
        }
        let mut len = name.len();
        while len != 0 && name[len - 1] == 0x59 {
            len -= 1;
        }
        decoded.extend_from_slice(&name[..len]);
    }

    pub(super) fn Text_FilterPlayerNameCharacters(&self, mut a: u8) -> u8 {
        if a >= 0x5f {
            if a >= 0x76 {
                a = a.wrapping_sub(0x42);
            } else if a == 0x5f {
                a = 8;
            } else if a == 0x60 {
                a = 0x22;
            } else if a == 0x61 {
                a = 0x3e;
            }
        }
        a
    }

    pub(super) fn Text_Render(&mut self) {
        match self.game_state.messaging.runtime.text_render_state() {
            0 => self.RenderText_Draw_Border(),
            1 => self.RenderText_Draw_BorderIncremental(),
            2 => self.RenderText_Draw_CharacterTilemap(),
            3 => self.RenderText_Draw_MessageCharacters(),
            4 => self.RenderText_Draw_Finish(),
            _ => {}
        }
    }

    pub(super) fn RenderText_Draw_Border(&mut self) {
        self.RenderText_DrawBorderInitialize();
        let mut d = self.RenderText_DrawBorderRow(0x1002, 0);
        for _ in 0..6 {
            d = self.RenderText_DrawBorderRow(d, 6);
        }
        self.RenderText_DrawBorderRow(d, 12);
        self.set_bg_vram_load_mode(1);
        self.messaging_state_mut().set_text_render_state(2);
    }

    pub(super) fn RenderText_Draw_BorderIncremental(&mut self) {
        self.set_bg_vram_load_mode(1);
        let mut a = self.game_state.messaging.runtime.text_incremental_state();
        let d = 0x1002;
        if a != 0 {
            a = if a < 7 { 1 } else { 2 };
        }
        match a {
            0 => {
                self.RenderText_DrawBorderInitialize();
                self.RenderText_DrawBorderRow(d, 0);
                self.messaging_state_mut()
                    .increment_text_incremental_state();
            }
            1 => {
                self.RenderText_DrawBorderRow(d, 6);
                self.messaging_state_mut()
                    .increment_text_incremental_state();
            }
            2 => {
                self.messaging_state_mut().set_text_render_state(2);
                self.RenderText_DrawBorderRow(d, 12);
                self.messaging_state_mut()
                    .increment_text_incremental_state();
            }
            _ => {}
        }
    }

    pub(super) fn RenderText_Draw_CharacterTilemap(&mut self) {
        self.Text_BuildCharacterTilemap();
    }

    pub(super) fn RenderText_Draw_MessageCharacters(&mut self) {
        // The ROM's RenderText_Draw_MessageCharacters runs the complete
        // command/glyph step in the caller, then unconditionally publishes
        // the VWF update through NMI.  In particular, its opening attract
        // text path does not split a glyph across synthetic host work slices.
        // Keeping that artificial budget delayed the first story glyph by one
        // display boundary, leaving the Triforce caption partially absent.
        let outcome = self.render_text_draw_message_characters_slice();
        let yielded_midline = matches!(outcome, VwfCpuSliceOutcome::InterruptedMidGlyph { .. });
        let yielded_to_authority = matches!(outcome, VwfCpuSliceOutcome::AuthorityBoundaryReached);
        let yielded = yielded_midline || yielded_to_authority;
        let caller_suffix_crosses_vblank = !yielded_midline
            && !yielded_to_authority
            && self.dialogue_scroll_cpu_is_idle()
            && outcome.caller_suffix_crosses_vblank();
        // A long scroll remains inside RenderText_Draw_Scroll; its dedicated
        // pre-main scheduler owns both the preceding NMI publication and the
        // eventual handler epilogue. Ordinary commands still finish here.
        if !yielded && self.dialogue_scroll_cpu_is_idle() {
            self.finish_dialogue_character_render_call();
            if caller_suffix_crosses_vblank {
                self.schedule_pre_main_caller_continuation(
                    PreMainCallerContinuation::DialogueVwfReturn,
                );
            }
        }
    }

    fn render_text_draw_message_characters_slice(&mut self) -> VwfCpuSliceOutcome {
        let outcome = self.render_text_draw_message_characters();
        let retain_incomplete_click = match outcome {
            VwfCpuSliceOutcome::InterruptedMidGlyph {
                retain_incomplete_click,
            } => retain_incomplete_click,
            VwfCpuSliceOutcome::AuthorityBoundaryReached => false,
            VwfCpuSliceOutcome::HandlerComplete { .. } => false,
        };
        let yielded_midline = matches!(outcome, VwfCpuSliceOutcome::InterruptedMidGlyph { .. });
        let yielded_to_authority = matches!(outcome, VwfCpuSliceOutcome::AuthorityBoundaryReached);
        let yielded = yielded_midline || yielded_to_authority;
        let caller_suffix_crosses_vblank = !yielded_midline
            && !yielded_to_authority
            && self.dialogue_scroll_cpu_is_idle()
            && outcome.caller_suffix_crosses_vblank();
        if yielded_midline
            && vwf_interrupted_click_marks_boundary(
                self.dialogue_vwf_glyph_cpu_phase,
                retain_incomplete_click,
            )
        {
            self.zelda_mark_vwf_glyph_tone_crossed_vblank_with_retention(retain_incomplete_click);
        }
        // A mid-line yield models Snes9x returning at vblank while the 65816 PC
        // is still inside VWF_RenderCharacter. The ROM has not reached the
        // handler epilogue yet, so $17/$0710 remain zero: NMI performs normal
        // core maintenance and does not publish the unfinished text buffer.
        // The next host slice resumes only this interrupted main-thread work.
        self.dialogue_fast_forward_hold_pending = yielded || caller_suffix_crosses_vblank;
        outcome
    }

    /// Run only the interruptible character handler to one authoritative
    /// decoder endpoint. Character epilogue, Module0E scroll copies, and the
    /// ZeldaRunGameLoop common suffix remain owned by the eventual terminal
    /// source host.
    pub(super) fn advance_suspended_vwf_to_authoritative_endpoint(
        &mut self,
        target_read_position: u16,
        source_current_glyph_started: bool,
    ) -> SuspendedVwfEndpointTransition {
        let start_read_position = self.game_state.messaging.runtime.dialogue_msg_read_pos();
        let decoded_text_len = self.game_state.messaging.decoded_text.as_slice().len();
        assert!(
            usize::from(start_read_position) < decoded_text_len
                && usize::from(target_read_position) < decoded_text_len,
            "a suspended VWF endpoint transition escaped the decoded message buffer",
        );
        assert!(
            self.frame_hosts_resident_render_text(),
            "a suspended VWF endpoint transition escaped Module0E/Module1B RenderText: {:?}",
            self.game_state.frame,
        );
        assert_eq!(
            self.game_state.messaging.runtime.module(),
            1,
            "a suspended VWF endpoint transition lost the active Text_Render module",
        );
        assert_eq!(
            self.game_state.messaging.runtime.text_render_state(),
            3,
            "a suspended VWF endpoint transition lost its character-render handler",
        );
        assert!(
            !self.dialogue_fast_forward_hold_pending,
            "a suspended VWF endpoint transition overlapped another pending hold",
        );
        let scheduler_before = self.game_execution_scheduler;
        let suffix_before = self.pending_main_loop_common_suffix;
        let expected_gates_before = self.original_timing_expected_nmi_update_gates.clone();
        let nmi_ownership_before = (
            self.original_timing_nmi_publication_pending,
            self.original_timing_pending_nmi_update_gate,
            self.game_state.display.nmi_update_is_latched(),
            self.game_state.display.pending_nmi_subroutine,
            self.game_state.display.core_update_disable_flag,
            self.main_loop_sprite_preparation_completed,
        );
        let assert_architectural_ownership_unchanged = |state: &ZeldaState| {
            assert_eq!(
                state.game_execution_scheduler, scheduler_before,
                "a VWF endpoint transition cannot schedule translated work",
            );
            assert_eq!(
                state.pending_main_loop_common_suffix, suffix_before,
                "a VWF endpoint transition cannot consume its caller suffix",
            );
            assert_eq!(
                state.original_timing_expected_nmi_update_gates, expected_gates_before,
                "a VWF endpoint transition cannot consume external NMI gate authority",
            );
            assert_eq!(
                (
                    state.original_timing_nmi_publication_pending,
                    state.original_timing_pending_nmi_update_gate,
                    state.game_state.display.nmi_update_is_latched(),
                    state.game_state.display.pending_nmi_subroutine,
                    state.game_state.display.core_update_disable_flag,
                    state.main_loop_sprite_preparation_completed,
                ),
                nmi_ownership_before,
                "a VWF endpoint transition cannot publish architectural NMI or suffix state",
            );
        };
        if start_read_position > target_read_position {
            // A fresh module iteration renders natively at its own budget and
            // may legitimately lead the wire's decoder; an endpoint at or
            // behind the native cursor is therefore an already-satisfied
            // source statement (route host 37226). Translated bookkeeping is
            // not source authority either way: normalize the target without
            // replaying or rewinding any native VWF command.
            self.dialogue_live_message_read_position_target = None;
            assert!(
                !source_current_glyph_started,
                "native VWF execution advanced beyond a source-started current glyph",
            );
            assert_architectural_ownership_unchanged(self);
            return SuspendedVwfEndpointTransition {
                start_read_position,
                target_read_position,
                slice_count: 0,
                current_glyph_started: false,
            };
        }

        self.dialogue_fast_forward_hold_active = true;
        self.dialogue_live_message_read_position_target = Some(target_read_position);
        self.dialogue_vwf_handler_completed_at_endpoint = false;
        if start_read_position == target_read_position {
            if source_current_glyph_started {
                self.resume_current_vwf_glyph_after_committed_prefix();
                self.dialogue_fast_forward_hold_pending = true;
            } else {
                self.dialogue_live_message_read_position_target = None;
            }
            assert_architectural_ownership_unchanged(self);
            return SuspendedVwfEndpointTransition {
                start_read_position,
                target_read_position,
                slice_count: 0,
                current_glyph_started: source_current_glyph_started,
            };
        }
        let mut slice_count = 0u32;
        loop {
            let cursor_before = self.game_state.messaging.runtime.dialogue_msg_read_pos();
            assert!(
                cursor_before < target_read_position,
                "native VWF execution skipped its authoritative decoder endpoint",
            );
            let phase_before = self.dialogue_vwf_glyph_cpu_phase;
            let wait_before = self.game_state.messaging.runtime.text_wait_countdown();
            let scroll_nibble_before = self
                .game_state
                .messaging
                .dialogue_source_offset
                .bank_offset_low_nibble();
            let progress_before = (
                target_read_position - cursor_before,
                if phase_before.is_ready() {
                    u64::MAX
                } else {
                    u64::from(phase_before.remaining_master_cycles())
                },
            );
            slice_count = slice_count
                .checked_add(1)
                .expect("native VWF endpoint transition exceeded its structural progress bound");
            let outcome = self.render_text_draw_message_characters_slice();
            let cursor_after = self.game_state.messaging.runtime.dialogue_msg_read_pos();
            match outcome {
                VwfCpuSliceOutcome::InterruptedMidGlyph { .. } => {
                    // These raster-budget yields are translated timing shadows
                    // inside the source endpoint receipt. Fold them without
                    // publishing an NMI, handler epilogue, or caller suffix;
                    // the strictly decreasing measure below proves progress.
                    assert!(
                        self.dialogue_fast_forward_hold_pending,
                        "an interrupted VWF endpoint slice omitted its pending hold",
                    );
                    let phase_after = self.dialogue_vwf_glyph_cpu_phase;
                    let progress_after = (
                        target_read_position.checked_sub(cursor_after).expect(
                            "native VWF execution skipped past its authoritative decoder endpoint",
                        ),
                        if phase_after.is_ready() {
                            u64::MAX
                        } else {
                            u64::from(phase_after.remaining_master_cycles())
                        },
                    );
                    assert!(
                        progress_after < progress_before,
                        "an interrupted VWF endpoint slice did not decrease its decoder/glyph progress measure",
                    );
                    self.dialogue_fast_forward_hold_pending = false;
                }
                VwfCpuSliceOutcome::AuthorityBoundaryReached => {
                    assert_eq!(
                        cursor_after, target_read_position,
                        "native VWF execution stopped at the wrong authoritative endpoint",
                    );
                    assert!(
                        self.dialogue_fast_forward_hold_pending,
                        "the authoritative VWF endpoint omitted its suspended hold marker",
                    );
                    if source_current_glyph_started {
                        self.resume_current_vwf_glyph_after_committed_prefix();
                    }
                    assert_architectural_ownership_unchanged(self);
                    return SuspendedVwfEndpointTransition {
                        start_read_position,
                        target_read_position,
                        slice_count,
                        current_glyph_started: source_current_glyph_started,
                    };
                }
                VwfCpuSliceOutcome::HandlerComplete { .. } => {
                    // A multi-frame command (a WAIT countdown, line pacing)
                    // returns the handler between hardware frames; the ROM
                    // crossed those frames before publishing this endpoint.
                    // Re-enter the handler as the following frame's call,
                    // requiring decoder or countdown progress so a stuck
                    // command cannot loop forever (route host 46822).
                    let wait_after = self.game_state.messaging.runtime.text_wait_countdown();
                    let scroll_nibble_after = self
                        .game_state
                        .messaging
                        .dialogue_source_offset
                        .bank_offset_low_nibble();
                    assert!(
                        cursor_after > cursor_before
                            || wait_after != wait_before
                            || scroll_nibble_after != scroll_nibble_before,
                        "native VWF handler stalled before authoritative endpoint {target_read_position:#x}: cursor={cursor_after:#x} byte={:#x} next={:?} wait={wait_after} line_speed={} scroll_idle={} stops_flag={} scroll_speed={} host={}",
                        self.game_state
                            .messaging
                            .decoded_text
                            .byte(usize::from(cursor_after)),
                        self.game_state
                            .messaging
                            .decoded_text
                            .next_byte(usize::from(cursor_after)),
                        self.game_state.messaging.runtime.vwf_line_speed_cur(),
                        self.dialogue_scroll_cpu_is_idle(),
                        self.dialogue_vwf_completion_stops_before_scroll,
                        self.game_state.messaging.runtime.dialogue_scroll_speed(),
                        self.frame_ctr_dbg,
                    );
                }
            }
        }
    }

    /// Align a decoder endpoint which returned from inside the current
    /// `VWF_RenderSingle` body. The source has already committed the function
    /// prefix, so establish the matching native continuation without replaying
    /// that prefix when the drawing body resumes on the next host.
    fn resume_current_vwf_glyph_after_committed_prefix(&mut self) {
        let read_pos = self.game_state.messaging.runtime.dialogue_msg_read_pos() as usize;
        let c = self.game_state.messaging.decoded_text.byte(read_pos);
        let (param, cmd, multibyte) = self.text_decode_cmd(
            c,
            self.game_state.messaging.decoded_text.next_byte(read_pos),
        );
        assert_eq!(cmd, TEXT_CMD_IS_LETTER);
        assert!(!multibyte);
        assert!(
            self.game_state.messaging.runtime.vwf_line_speed_cur() < 2,
            "a source-started VWF glyph retained an unexpired line delay",
        );
        let width = self.dialogue_glyph_width(param);
        let glyph_cursor = vwf_glyph_cursor_after_pending_line_transition(
            self.game_state.messaging.vwf_render.glyph_cursor_usize(),
            self.game_state.messaging.vwf_render.current_line(),
            self.game_state.messaging.vwf_render.next_line_requested() != 0,
        );
        let x = self.vwf_glyph_advance_prefix_sum(glyph_cursor);
        let drawing_master_cycles = vwf_render_glyph_drawing_master_cycles(width, x);
        match self.dialogue_vwf_glyph_cpu_phase {
            VwfGlyphCpuPhase::Ready => {
                self.begin_vwf_glyph(param);
                self.dialogue_vwf_glyph_cpu_phase = VwfGlyphCpuPhase::Drawing {
                    remaining_master_cycles: drawing_master_cycles,
                };
            }
            VwfGlyphCpuPhase::PreparingDrawing { .. } | VwfGlyphCpuPhase::Drawing { .. } => {}
            VwfGlyphCpuPhase::Entering { .. } => {
                panic!("native VWF execution had not committed a source-started glyph prefix")
            }
        }
    }

    /// Finish only the interruptible character handler which remains live
    /// after a source endpoint host returned. Any translated mid-glyph raster
    /// budgets are timing shadows inside the later typed caller-return
    /// authority; caller epilogue, Module0E scroll writes, and the outer
    /// ZeldaRunGameLoop suffix remain outside this transition.
    pub(super) fn advance_suspended_vwf_to_handler_completion(
        &mut self,
    ) -> SuspendedVwfCompletionTransition {
        let start_read_position = self.game_state.messaging.runtime.dialogue_msg_read_pos();
        let decoded_text_len = self.game_state.messaging.decoded_text.as_slice().len();
        assert!(
            usize::from(start_read_position) < decoded_text_len,
            "a suspended VWF completion escaped the decoded message buffer",
        );
        assert!(
            self.frame_hosts_resident_render_text(),
            "a suspended VWF completion escaped Module0E/Module1B RenderText: {:?}",
            self.game_state.frame,
        );
        assert_eq!(
            self.game_state.messaging.runtime.module(),
            1,
            "a suspended VWF completion lost the active Text_Render module",
        );
        assert_eq!(
            self.game_state.messaging.runtime.text_render_state(),
            3,
            "a suspended VWF completion lost its character-render handler",
        );
        assert!(
            self.dialogue_fast_forward_hold_active,
            "a suspended VWF completion lost its translated caller hold",
        );
        assert!(
            !self.dialogue_fast_forward_hold_pending,
            "a suspended VWF completion overlapped another pending hold",
        );
        assert_eq!(
            self.dialogue_live_message_read_position_target, None,
            "a suspended VWF completion retained an unconsumed endpoint target",
        );

        let scheduler_before = self.game_execution_scheduler;
        let suffix_before = self.pending_main_loop_common_suffix;
        let expected_gates_before = self.original_timing_expected_nmi_update_gates.clone();
        let nmi_ownership_before = (
            self.original_timing_nmi_publication_pending,
            self.original_timing_pending_nmi_update_gate,
            self.game_state.display.nmi_update_is_latched(),
            self.game_state.display.pending_nmi_subroutine,
            self.game_state.display.core_update_disable_flag,
            self.main_loop_sprite_preparation_completed,
        );
        let assert_architectural_ownership_unchanged = |state: &ZeldaState| {
            assert_eq!(
                state.game_execution_scheduler, scheduler_before,
                "a suspended VWF completion cannot schedule translated work",
            );
            assert_eq!(
                state.pending_main_loop_common_suffix, suffix_before,
                "a suspended VWF completion cannot consume its caller suffix",
            );
            assert_eq!(
                state.original_timing_expected_nmi_update_gates, expected_gates_before,
                "a suspended VWF completion cannot consume external NMI gate authority",
            );
            assert_eq!(
                (
                    state.original_timing_nmi_publication_pending,
                    state.original_timing_pending_nmi_update_gate,
                    state.game_state.display.nmi_update_is_latched(),
                    state.game_state.display.pending_nmi_subroutine,
                    state.game_state.display.core_update_disable_flag,
                    state.main_loop_sprite_preparation_completed,
                ),
                nmi_ownership_before,
                "a suspended VWF completion cannot publish architectural NMI or suffix state",
            );
        };

        if std::mem::take(&mut self.dialogue_vwf_handler_completed_at_endpoint) {
            // The endpoint host's final command already returned the ROM
            // handler; this terminal host runs only the caller suffix.
            assert_architectural_ownership_unchanged(self);
            return SuspendedVwfCompletionTransition {
                start_read_position,
                end_read_position: start_read_position,
                slice_count: 0,
                // The typed common-suffix completion of this terminal host
                // owns the caller suffix; report the translated hold as-is.
                caller_suffix_crossed_vblank: self.dialogue_fast_forward_hold_pending,
                begins_message_line_scroll: false,
            };
        }
        let mut slice_count = 0u32;
        self.dialogue_vwf_completion_stops_before_scroll = true;
        loop {
            let cursor_before = self.game_state.messaging.runtime.dialogue_msg_read_pos();
            assert!(
                usize::from(cursor_before) < decoded_text_len,
                "a suspended VWF completion advanced beyond the decoded message buffer",
            );
            let phase_before = self.dialogue_vwf_glyph_cpu_phase;
            let progress_before = (
                decoded_text_len - usize::from(cursor_before),
                if phase_before.is_ready() {
                    u64::MAX
                } else {
                    u64::from(phase_before.remaining_master_cycles())
                },
            );
            slice_count = slice_count
                .checked_add(1)
                .expect("a suspended VWF completion exceeded its structural progress bound");
            let outcome = self.render_text_draw_message_characters_slice();
            let cursor_after = self.game_state.messaging.runtime.dialogue_msg_read_pos();
            match outcome {
                VwfCpuSliceOutcome::InterruptedMidGlyph { .. } => {
                    assert!(
                        self.dialogue_fast_forward_hold_pending,
                        "an interrupted VWF completion slice omitted its pending hold",
                    );
                    let phase_after = self.dialogue_vwf_glyph_cpu_phase;
                    let progress_after = (
                        decoded_text_len
                            .checked_sub(usize::from(cursor_after))
                            .expect(
                            "a suspended VWF completion skipped beyond its decoded message buffer",
                        ),
                        if phase_after.is_ready() {
                            u64::MAX
                        } else {
                            u64::from(phase_after.remaining_master_cycles())
                        },
                    );
                    assert!(
                        usize::from(cursor_after) < decoded_text_len
                            && progress_after < progress_before,
                        "an interrupted VWF completion slice did not advance its decoder/glyph progress measure",
                    );
                    self.dialogue_fast_forward_hold_pending = false;
                }
                VwfCpuSliceOutcome::AuthorityBoundaryReached => {
                    panic!("a suspended VWF completion reached an unclaimed decoder endpoint")
                }
                outcome @ VwfCpuSliceOutcome::HandlerComplete { .. } => {
                    let caller_suffix_crossed_vblank = outcome.caller_suffix_crosses_vblank();
                    assert_eq!(
                        self.dialogue_fast_forward_hold_pending, caller_suffix_crossed_vblank,
                        "a suspended VWF completion disagreed with its translated caller-suffix hold",
                    );
                    assert_architectural_ownership_unchanged(self);
                    self.dialogue_vwf_completion_stops_before_scroll = false;
                    return SuspendedVwfCompletionTransition {
                        start_read_position,
                        end_read_position: cursor_after,
                        slice_count,
                        caller_suffix_crossed_vblank,
                        begins_message_line_scroll: !self.dialogue_scroll_cpu_is_idle(),
                    };
                }
            }
        }
    }

    pub(super) fn finish_dialogue_character_render_call(&mut self) {
        self.set_pending_nmi_subroutine(2);
        self.set_core_update_disable_flag(2);
    }

    /// Runs the interruptible 65816 message loop up to this display boundary
    /// and reports whether vblank owns the glyph loop or its caller suffix.
    fn render_text_draw_message_characters(&mut self) -> VwfCpuSliceOutcome {
        let debug_vwf_budget = debug_vwf_budget_for_frame(self.frame_ctr_dbg);
        let resuming = self.dialogue_fast_forward_hold_active;
        if !resuming {
            // A fresh handler call starts a new ROM RenderText slice; an
            // earlier endpoint's handler-complete mark cannot describe it
            // (route host 180562 followed the 180560 endpoint).
            self.dialogue_vwf_handler_completed_at_endpoint = false;
        }
        let handler_entry_glyph_phase = self.dialogue_vwf_glyph_cpu_phase;
        let caller_suffix_master_cycles = if resuming
            && matches!(
                self.dialogue_vwf_glyph_cpu_phase,
                VwfGlyphCpuPhase::PreparingDrawing { .. }
            ) {
            VWF_PREPARING_DRAWING_CALLER_SUFFIX_MASTER_CYCLES
        } else {
            VWF_CALLER_SUFFIX_MASTER_CYCLES
        };
        let current_line = self.game_state.messaging.vwf_render.current_line();
        let entry_phase = if resuming {
            VwfHandlerEntryPhase::OrdinaryModuleIteration
        } else {
            std::mem::take(&mut self.dialogue_vwf_handler_entry_phase)
        };
        let mut cycles_left = vwf_render_loop_cycle_budget(resuming, current_line, entry_phase);
        let mut frame_advance: u16 = 0;
        let mut midline_yield = false;
        let mut authority_boundary_reached = false;
        let mut retain_incomplete_click = false;
        loop {
            let read_pos = self.game_state.messaging.runtime.dialogue_msg_read_pos() as usize;
            let c = self.game_state.messaging.decoded_text.byte(read_pos);
            let (param, cmd, multibyte) = self.text_decode_cmd(
                c,
                self.game_state.messaging.decoded_text.next_byte(read_pos),
            );
            if cmd == TEXT_CMD_SCROLL && self.dialogue_vwf_completion_stops_before_scroll {
                // The typed suffix-completed terminal proves the RenderText
                // caller returned to ZeldaRunGameLoop; a begun scroll copy
                // cannot return. Yield with the scroll command unconsumed —
                // the next module iteration owns the scroll start (route
                // host 20765).
                break;
            }
            let mut command_done = false;
            let mut restart_if_zero_speed = false;
            match cmd {
                TEXT_CMD_IS_LETTER => {
                    if self.game_state.messaging.runtime.vwf_line_speed_cur() >= 2 {
                        self.messaging_state_mut().decrement_vwf_line_speed_cur();
                    } else {
                        let width = self.dialogue_glyph_width(param);
                        let fast_forward =
                            self.game_state.messaging.runtime.vwf_line_speed_cur() == 0;
                        if fast_forward {
                            // VWF_RenderSingle applies $0720 before reading
                            // vwf_arr[i]. Time the first glyph from that line's
                            // reset cursor, not the stale prior-line cursor.
                            let glyph_cursor = vwf_glyph_cursor_after_pending_line_transition(
                                self.game_state.messaging.vwf_render.glyph_cursor_usize(),
                                self.game_state.messaging.vwf_render.current_line(),
                                self.game_state.messaging.vwf_render.next_line_requested() != 0,
                            );
                            let x = self.vwf_glyph_advance_prefix_sum(glyph_cursor);
                            let drawing_master_cycles =
                                vwf_render_glyph_drawing_master_cycles(width, x);
                            if debug_vwf_budget {
                                eprintln!(
                                    "vwf_glyph host={} read_pos={:#x} code={:#x} width={} cursor={} line_x={} cycles_left={} phase={:?} drawing_cycles={}",
                                    self.frame_ctr_dbg,
                                    read_pos,
                                    param,
                                    width,
                                    glyph_cursor,
                                    x,
                                    cycles_left,
                                    self.dialogue_vwf_glyph_cpu_phase,
                                    drawing_master_cycles,
                                );
                            }
                            let cycles_before_advance = cycles_left;
                            let advance = self
                                .dialogue_vwf_glyph_cpu_phase
                                .advance(cycles_left, drawing_master_cycles);
                            self.dialogue_vwf_glyph_cpu_phase = advance.next_phase;
                            cycles_left -= advance.consumed_master_cycles;
                            if advance.entered_function {
                                // A near-boundary semantic entry can land on
                                // the far side of vblank in the measured ROM
                                // trace even though the coarse loop budget has
                                // just enough room for the click prefix. This
                                // flag belongs to the newest click candidate;
                                // begin_vwf_glyph replaces that candidate too.
                                retain_incomplete_click =
                                    vwf_new_glyph_click_requires_boundary_retention(
                                        cycles_before_advance,
                                        handler_entry_glyph_phase,
                                    );
                                self.begin_vwf_glyph(param);
                            }
                            if !advance.completed {
                                midline_yield = true;
                                break;
                            }
                        } else {
                            self.begin_vwf_glyph(param);
                        }
                        frame_advance = frame_advance.saturating_add(u16::from(width));
                        self.complete_vwf_glyph(param, read_pos as u16);
                        command_done = true;
                        if fast_forward {
                            cycles_left =
                                cycles_left.saturating_sub(VWF_GLYPH_TRANSITION_MASTER_CYCLES);
                            restart_if_zero_speed = true;
                        }
                    }
                }
                TEXT_CMD_NEXT_PIC => {
                    if self.game_state.frame.main_module == 20 {
                        self.PaletteFilterHistory();
                        command_done = self.game_state.display.palette_filter.countdown() == 0;
                    } else {
                        command_done = true;
                    }
                }
                TEXT_CMD_SCROLL_SPD => {
                    self.messaging_state_mut().set_dialogue_scroll_speed(param);
                    command_done = true;
                }
                TEXT_CMD_SCROLL => command_done = self.RenderText_Draw_Scroll(cycles_left),
                TEXT_CMD_1 | TEXT_CMD_2 | TEXT_CMD_3 => {
                    let idx = (cmd - TEXT_CMD_1) as usize;
                    self.set_vwf_current_line(VWF_ROW_POSITIONS[idx]);
                    self.request_vwf_next_line(1);
                    command_done = true;
                }
                TEXT_CMD_WAIT => {
                    let wait = if self.game_state.player.follower_link.joypad1l_last() & 0x80 != 0 {
                        1
                    } else {
                        self.game_state.messaging.runtime.text_wait_countdown()
                    };
                    match wait {
                        0 => self.messaging_state_mut().set_text_wait_countdown(
                            TEXT_WAIT_DURATIONS[param as usize].wrapping_sub(1),
                        ),
                        1 => {
                            self.messaging_state_mut().clear_text_wait_countdown();
                            command_done = true;
                        }
                        _ => self
                            .messaging_state_mut()
                            .set_text_wait_countdown(wait.wrapping_sub(1)),
                    }
                }
                TEXT_CMD_SOUND => {
                    self.set_sound_effect_2(param);
                    command_done = true;
                }
                TEXT_CMD_SPEED => {
                    self.messaging_state_mut().set_vwf_line_speed(param);
                    self.messaging_state_mut().set_vwf_line_speed_cur(param);
                    command_done = true;
                }
                TEXT_CMD_CHOOSE => self.RenderText_Draw_Choose2LowOr3(),
                TEXT_CMD_ITEM => self.RenderText_Draw_ChooseItem(),
                TEXT_CMD_SELCHG => self.RenderText_Draw_Choose2HiOr3(),
                TEXT_CMD_CHOOSE3 => self.RenderText_Draw_Choose3(),
                TEXT_CMD_CHOOSE2 => self.RenderText_Draw_Choose1Or2(),
                TEXT_CMD_WAITKEY | TEXT_CMD_END_MESSAGE => {
                    if std::env::var_os("ZELDA3_DEBUG_VWF_WAIT").is_some() {
                        eprintln!(
                            "[VWF-WAIT] host={} cmd={} read_pos={read_pos} target={:?} countdown2={} filtered={:#04x}/{:#04x} state={}",
                            self.frame_ctr_dbg,
                            if cmd == TEXT_CMD_WAITKEY {
                                "WAITKEY"
                            } else {
                                "END"
                            },
                            self.dialogue_live_message_read_position_target,
                            self.game_state.messaging.runtime.text_wait_countdown2(),
                            self.game_state.player.follower_link.filtered_joypad_h(),
                            self.game_state.player.follower_link.filtered_joypad_l(),
                            self.game_state.messaging.runtime.text_render_state(),
                        );
                    }
                    if self
                        .dialogue_live_message_read_position_target
                        .is_some_and(|target| usize::from(target) > read_pos)
                    {
                        // Endpoint catch-up: the published decoder endpoint
                        // past this command proves the ROM's countdown and
                        // keypress already elapsed on the preceding hosts
                        // (route host 46822).
                        self.messaging_state_mut().set_text_wait_countdown2(28);
                        command_done = cmd == TEXT_CMD_WAITKEY;
                        if cmd == TEXT_CMD_END_MESSAGE {
                            self.messaging_state_mut().set_text_render_state(4);
                        }
                    } else if self.game_state.messaging.runtime.text_wait_countdown2() != 0 {
                        self.messaging_state_mut().decrement_text_wait_countdown2();
                        if self.game_state.messaging.runtime.text_wait_countdown2() == 1 {
                            self.set_sound_effect_2(36);
                        }
                    } else if (self.game_state.player.follower_link.filtered_joypad_h()
                        | self.game_state.player.follower_link.filtered_joypad_l())
                        & if cmd == TEXT_CMD_WAITKEY { 0xc0 } else { 0xff }
                        != 0
                    {
                        self.messaging_state_mut().set_text_wait_countdown2(28);
                        command_done = cmd == TEXT_CMD_WAITKEY;
                        if cmd == TEXT_CMD_END_MESSAGE {
                            self.messaging_state_mut().set_text_render_state(4);
                        }
                    }
                }
                _ => {
                    panic!("RenderText_Draw_MessageCharacters unsupported cmd {cmd} param {param}")
                }
            }
            if command_done {
                self.messaging_state_mut().set_dialogue_msg_read_pos(
                    (read_pos as u16).wrapping_add(1 + u16::from(multibyte)),
                );
                if self.dialogue_live_message_read_position_target
                    == Some(self.game_state.messaging.runtime.dialogue_msg_read_pos())
                {
                    authority_boundary_reached = true;
                    // A command which does not restart the glyph loop returns
                    // the ROM handler here; the endpoint therefore also marks
                    // the handler complete (its caller suffix is what the
                    // following host still owes).
                    self.dialogue_vwf_handler_completed_at_endpoint = !restart_if_zero_speed;
                    break;
                }
            }
            if !restart_if_zero_speed {
                break;
            }
        }
        if !midline_yield {
            self.dialogue_vwf_glyph_cpu_phase = VwfGlyphCpuPhase::Ready;
        }
        if debug_vwf_budget {
            let cursor = self.game_state.messaging.vwf_render.glyph_cursor_usize();
            let arrval = self.vwf_glyph_advance_prefix_sum(cursor);
            eprintln!(
                "vwf_cycles host={} read_pos={:#x} frame_advance={} glyph_cursor={} line_x={} cycles_left={} cycle_debt={} glyph_phase={:?} midline_yield={} resumed={} entry_phase={entry_phase:?} suffix_threshold={} suffix_crosses_vblank={}",
                self.frame_ctr_dbg,
                self.game_state.messaging.runtime.dialogue_msg_read_pos(),
                frame_advance,
                cursor,
                arrval,
                cycles_left,
                self.dialogue_vwf_glyph_cpu_phase.remaining_master_cycles(),
                self.dialogue_vwf_glyph_cpu_phase,
                midline_yield,
                resuming,
                caller_suffix_master_cycles,
                !midline_yield && cycles_left < caller_suffix_master_cycles,
            );
        }
        if midline_yield {
            VwfCpuSliceOutcome::InterruptedMidGlyph {
                retain_incomplete_click,
            }
        } else if authority_boundary_reached {
            VwfCpuSliceOutcome::AuthorityBoundaryReached
        } else {
            VwfCpuSliceOutcome::HandlerComplete {
                master_cycles_before_vblank: cycles_left,
                caller_suffix_master_cycles,
            }
        }
    }

    pub(super) fn RenderText_Draw_Finish(&mut self) {
        self.RenderText_DrawBorderInitialize();
        let top_left = self.game_state.messaging.runtime.text_msgbox_topleft_copy();
        self.write_vram_upload_buffer_word(0, top_left.swap_bytes());
        self.write_vram_upload_buffer_word(2, 0x2e42);
        self.write_vram_upload_buffer_word(4, 0x387f);
        self.write_vram_upload_buffer_word(6, 0xffff);
        self.set_bg_vram_load_mode(1);
        self.messaging_state_mut().clear_module();
        self.set_submodule(0);
        let saved_module = self.game_state.frame.saved_module_for_menu;
        self.set_main_module(saved_module);
    }

    /// Width in pixels of dialogue glyph `c` (VWF proportional-font advance),
    /// from font memblk 95 index 1 — the same table `VWF_RenderSingle` uses.
    fn dialogue_glyph_width(&self, c: u8) -> u8 {
        self.asset_memblk(95, self.dialogue_font_blk_index)
            .map(|font| {
                find_index_in_memblk(font, 1)
                    .ptr
                    .get(c as usize)
                    .copied()
                    .unwrap_or(0)
            })
            .unwrap_or(0)
    }

    fn begin_vwf_glyph(&mut self, c: u8) {
        if c != 0x59 {
            self.set_sound_effect_2(12);
        }
        // Capture at the click itself. By the later vblank marker, gameplay
        // may already have cleared $012f even though the interrupted ROM call
        // still owns the earlier APUI03 value. Glyph $59 performs no new
        // write, so it intentionally captures the retained latch too.
        self.zelda_prepare_vwf_glyph_tone_boundary_marker();
        let speed = self.game_state.messaging.runtime.vwf_line_speed();
        self.messaging_state_mut().set_vwf_line_speed_cur(speed);
        // ROM VWF_RenderSingle applies the pending line transition before it
        // reads vwf_arr[i] and enters the interruptible pixel loops. Make that
        // state visible at the same function-entry boundary; delaying it until
        // the drawing completed made resumed slices reuse the prior-line
        // cursor when estimating their remaining work.
        if self.game_state.messaging.vwf_render.next_line_requested() != 0 {
            let line = (self.game_state.messaging.vwf_render.current_line() >> 1) as usize;
            self.set_vwf_line_render_offset(VWF_RENDER_CHARACTER_RENDER_POS[line]);
            self.set_vwf_glyph_cursor(VWF_RENDER_CHARACTER_LINE_POSITIONS[line]);
            self.clear_vwf_next_line_request();
        }
    }

    fn complete_vwf_glyph(&mut self, c: u8, dialogue_offset: u16) {
        let Some(dialogue_font) = self.asset_memblk(95, self.dialogue_font_blk_index) else {
            return;
        };
        let font_data = find_index_in_memblk(dialogue_font, 0).ptr.to_vec();
        let widths = find_index_in_memblk(dialogue_font, 1).ptr.to_vec();
        self.zelda_complete_vwf_glyph_boundary_marker();
        let width = widths.get(c as usize).copied().unwrap_or(0);
        assert!(width <= 8);
        let i = self.game_state.messaging.vwf_render.glyph_cursor_usize();
        self.increment_vwf_glyph_cursor();
        // C: arrval = vwf_arr[i]; vwf_arr[i + 1] = arrval + width (vwf_arr = raw g_ram).
        let arrval = self.vwf_glyph_advance_prefix_sum(i);
        self.set_vwf_next_glyph_advance_prefix_sum(i, arrval.wrapping_add(width));
        let r10 = ((c as usize & 0x70) * 2) + (c as usize & 0x0f);
        let r0 = arrval as usize * 2;
        let line_ptr = self.game_state.messaging.vwf_render.line_render_offset() as usize;
        self.record_bg3_vwf_glyph_run(c, arrval, line_ptr, width, dialogue_offset);
        self.messaging_vwf_render_half(&font_data, r10, r0, line_ptr, width);
        self.messaging_vwf_render_half(&font_data, r10 + 16, r0, line_ptr + 0x150, width);
    }

    fn messaging_vwf_render_half(
        &mut self,
        font_data: &[u8],
        r10: usize,
        r0: usize,
        line_ptr: usize,
        width: u8,
    ) {
        let mut src = r10 * 16;
        for i in (0..16).step_by(2) {
            if src + 1 >= font_data.len() {
                return;
            }
            let mut r4 = u16::from_le_bytes([font_data[src], font_data[src + 1]]);
            src += 2;
            let y_base = r0 + line_ptr;
            let mut x = (y_base & 0xff0) + i;
            let mut y = (y_base >> 1) & 7;
            let mut r3 = width;
            while r3 != 0 {
                if r4 & 0x0080 != 0 {
                    self.xor_messaging_render_buffer_mask(x, VWF_RENDER_CHARACTER_SET_MASKS[y]);
                } else {
                    self.clear_messaging_render_buffer_mask(x, VWF_RENDER_CHARACTER_SET_MASKS[y]);
                }
                if r4 & 0x8000 != 0 {
                    self.xor_messaging_render_buffer_mask(x + 1, VWF_RENDER_CHARACTER_SET_MASKS[y]);
                } else {
                    self.clear_messaging_render_buffer_mask(
                        x + 1,
                        VWF_RENDER_CHARACTER_SET_MASKS[y],
                    );
                }
                r4 = (r4 & !0x8080) << 1;
                r3 -= 1;
                y += 1;
                if y == 8 {
                    break;
                }
            }
            x += 16;
            if r4 != 0 {
                self.set_messaging_render_buffer_word_at_byte_offset(x, r4);
            }
        }
    }

    pub(super) fn RenderText_Draw_Choose2LowOr3(&mut self) {
        self.RenderText_Draw_Choose2(1);
    }

    pub(super) fn RenderText_Draw_ChooseItem(&mut self) {
        if self.game_state.messaging.runtime.text_wait_countdown2() != 0 {
            self.messaging_state_mut().decrement_text_wait_countdown2();
            if self.game_state.messaging.runtime.text_wait_countdown2() == 1 {
                self.RenderText_FindYItem_Next();
            }
        } else if (self.game_state.player.follower_link.filtered_joypad_h()
            | self.game_state.player.follower_link.filtered_joypad_l())
            & 0xc0
            != 0
        {
            self.messaging_state_mut().set_text_render_state(4);
        } else {
            if self.game_state.player.follower_link.filtered_joypad_h() & 5 != 0 {
                self.multiselect_choice_mut().increment_value();
            } else if self.game_state.player.follower_link.filtered_joypad_h() & 10 != 0 {
                self.multiselect_choice_mut().decrement_value();
                self.RenderText_FindYItem_Previous();
                self.RenderText_Refresh();
                return;
            }
            self.RenderText_FindYItem_Next();
            self.RenderText_Refresh();
        }
    }

    pub(super) fn RenderText_FindYItem_Previous(&mut self) {
        loop {
            let mut x = self.multiselect_choice().value();
            if (x as i8) < 0 {
                self.multiselect_choice_mut().set_value(31);
                x = 31;
            }
            // Raw RAM (not bounded inventory_item) — same fix as RenderText_FindYItem_Next.
            if x != 15
                && (self.ram[LINK_ITEM_BOW + x as usize] != 0
                    || (x == 32 && self.ram[LINK_ITEM_BOW + x as usize + 1] != 0))
            {
                break;
            }
            self.multiselect_choice_mut().decrement_value();
        }
        self.RenderText_DrawSelectedYItem();
    }

    pub(super) fn RenderText_FindYItem_Next(&mut self) {
        loop {
            let mut x = self.multiselect_choice().value();
            if x >= 32 {
                self.multiselect_choice_mut().set_value(0);
                x = 0;
            }
            // C reads raw inventory bytes ram[LINK_ITEM_BOW + x] for x up to 32. The native
            // item_slots model only covers 28 slots, so inventory_item(x) returns 0 for x>=28 —
            // which made this scan skip owned items 28-31 and wrap to 0 (NEW landed on item 0, OLD
            // on item 28), cascading the menu/text-render divergence. Read raw RAM to match C.
            if x != 15
                && (self.ram[LINK_ITEM_BOW + x as usize] != 0
                    || (x == 32 && self.ram[LINK_ITEM_BOW + x as usize + 1] != 0))
            {
                break;
            }
            self.multiselect_choice_mut().increment_value();
        }
        self.RenderText_DrawSelectedYItem();
    }

    pub(super) fn RenderText_DrawSelectedYItem(&mut self) {
        let item = self.multiselect_choice().value();
        // Raw RAM (not inventory_item, which is bounded to 28 slots) — matches C and covers the
        // multiselect index range 0..=32 (see RenderText_FindYItem_Next).
        let variant = if item == 3 || item == 32 {
            1
        } else {
            self.ram[LINK_ITEM_BOW + item as usize] as usize
        };
        let p = self.hud_get_item_box_table(item)[variant];
        self.set_vwf_tile_word_at_byte_offset(0x0c2, p[0]);
        self.set_vwf_tile_word_at_byte_offset(0x0c4, p[1]);
        self.set_vwf_tile_word_at_byte_offset(0x0ec, p[2]);
        self.set_vwf_tile_word_at_byte_offset(0x0ee, p[3]);
    }

    pub(super) fn RenderText_Draw_Choose2HiOr3(&mut self) {
        self.RenderText_Draw_Choose2(11);
    }

    fn RenderText_Draw_Choose2(&mut self, message_base: u16) {
        if self.game_state.messaging.runtime.text_wait_countdown2() != 0 {
            self.messaging_state_mut().decrement_text_wait_countdown2();
            if self.game_state.messaging.runtime.text_wait_countdown2() == 1 {
                self.set_sound_effect_2(36);
            }
        } else if (self.game_state.player.follower_link.filtered_joypad_h()
            | self.game_state.player.follower_link.filtered_joypad_l())
            & 0xc0
            != 0
        {
            self.set_sound_effect_1(43);
            self.messaging_state_mut().set_text_render_state(4);
        } else if self.game_state.player.follower_link.filtered_joypad_h() & 12 != 0 {
            let t = if self.game_state.player.follower_link.filtered_joypad_h() & 8 != 0 {
                0
            } else {
                1
            };
            if self.multiselect_choice().value() == t {
                return;
            }
            self.multiselect_choice_mut().set_value(t);
            self.set_sound_effect_2(32);
            self.dialogue_message_index_mut()
                .set_value(message_base + u16::from(t));
            self.Text_LoadCharacterBuffer();
            self.Text_InitVwfState();
        }
    }

    pub(super) fn RenderText_Draw_Choose3(&mut self) {
        if self.game_state.messaging.runtime.text_wait_countdown2() != 0 {
            self.messaging_state_mut().decrement_text_wait_countdown2();
            if self.game_state.messaging.runtime.text_wait_countdown2() == 1 {
                self.set_sound_effect_2(36);
            }
            return;
        }
        let y = (self.game_state.player.follower_link.filtered_joypad_l() & 0xc0)
            | self.game_state.player.follower_link.filtered_joypad_h();
        if y & 0xd0 != 0 {
            self.set_sound_effect_1(43);
            self.messaging_state_mut().set_text_render_state(4);
        } else if y & 12 != 0 {
            let mut choice = self.multiselect_choice().value();
            choice = if y & 8 != 0 {
                if choice == 0 {
                    2
                } else {
                    choice - 1
                }
            } else if choice == 2 {
                0
            } else {
                choice + 1
            };
            self.multiselect_choice_mut().set_value(choice);
            self.set_sound_effect_2(32);
            self.dialogue_message_index_mut()
                .set_value(u16::from(choice) + 6);
            self.Text_LoadCharacterBuffer();
            self.Text_InitVwfState();
        }
    }

    pub(super) fn RenderText_Draw_Choose1Or2(&mut self) {
        if self.game_state.messaging.runtime.text_wait_countdown2() != 0 {
            self.messaging_state_mut().decrement_text_wait_countdown2();
            if self.game_state.messaging.runtime.text_wait_countdown2() == 1 {
                self.set_sound_effect_2(36);
            }
            return;
        }
        let y = (self.game_state.player.follower_link.filtered_joypad_l() & 0xc0)
            | self.game_state.player.follower_link.filtered_joypad_h();
        if y & 0xd0 != 0 {
            self.set_sound_effect_1(43);
            self.messaging_state_mut().set_text_render_state(4);
        } else if y & 12 != 0 {
            let t = if y & 8 != 0 { 0 } else { 1 };
            if self.multiselect_choice().value() == t {
                return;
            }
            self.multiselect_choice_mut().set_value(t);
            self.set_sound_effect_2(32);
            self.dialogue_message_index_mut()
                .set_value(u16::from(t) + 9);
            self.Text_LoadCharacterBuffer();
            self.Text_InitVwfState();
        }
    }

    pub(super) fn begin_live_dialogue_scroll_after_vwf_endpoint(&mut self) {
        let read_pos = self.game_state.messaging.runtime.dialogue_msg_read_pos() as usize;
        let (_, command, _) = self.text_decode_cmd(
            self.game_state.messaging.decoded_text.byte(read_pos),
            self.game_state.messaging.decoded_text.next_byte(read_pos),
        );
        assert_eq!(
            command, TEXT_CMD_SCROLL,
            "source scroll entry did not follow the native VWF decoder endpoint"
        );
        assert!(self.dialogue_scroll_cpu_is_idle());
        assert!(self.dialogue_live_message_read_position_target.is_none());
        assert!(
            !self.RenderText_Draw_Scroll(0),
            "a suspended source scroll entry cannot complete the text line"
        );
        assert!(!self.dialogue_scroll_cpu_is_idle());
    }

    pub(super) fn RenderText_Draw_Scroll(&mut self, cycles_before_vblank: u32) -> bool {
        // One source call drains at most `scroll_speed + 1` pixel passes.
        // Live timing publishes the actual copy boundaries: ordinary entry
        // can span 2+2+1 passes, while entry after a VWF glyph can span 2+3.
        // The caller stays suspended until the source RTS, independently of
        // the number of copies completed in any particular host.
        let group = u16::from(self.game_state.messaging.runtime.dialogue_scroll_speed()) + 1;
        let nibble_before = u16::from(
            self.game_state
                .messaging
                .dialogue_source_offset
                .bank_offset_low_nibble()
                & 0x0f,
        );
        let remaining_in_line = 16u16.saturating_sub(nibble_before);
        if let Some(progress) = self.take_original_timing_dialogue_scroll_progress(true) {
            if self.rom_startup_timing() && self.triforce_room_poly_thread_is_active() {
                self.triforce_room_scroll_this_iteration = true;
            }
            let copies = u16::from(progress.completed_pixel_passes);
            assert!(
                copies <= group.min(remaining_in_line),
                "source scroll copies exceed the native call's remaining work"
            );
            if !progress.returned {
                self.begin_dialogue_scroll(
                    DialogueTextGeneration::PublishedDisplay,
                    DialogueScrollCompletionTiming::AfterReturnBoundary,
                );
            }
            let line_completed = self.render_text_scroll_pixels(copies);
            return line_completed && progress.returned;
        }
        assert!(
            !matches!(
                self.original_timing_owner,
                crate::zelda_rtl::OriginalTimingOwnerState::Live
            ),
            "live dialogue scroll entry requires its source copy/return receipt"
        );
        if self.rom_startup_timing() && self.triforce_room_poly_thread_is_active() {
            // Under the Triforce room's V-IRQ thread the main loop owns only
            // the lines from the IRQ to vblank; the whole call runs here and
            // Module19 holds the iteration by wire (see
            // `TriforceRoomLoadStep::Case9Scroll`).
            self.triforce_room_scroll_this_iteration = true;
            return self.render_text_scroll_pixels(group.min(remaining_in_line));
        }
        if group != 5 {
            // Only scroll speed 4 has oracle-verified lag timing; other
            // speeds keep the single-frame drain until ground truth is
            // captured for them (ZELDA3_SNES9X_VRAM_TRACE on 0x1cdf).
            return self.render_text_scroll_pixels(group.min(remaining_in_line));
        }
        if remaining_in_line < group {
            // Cheap completing call: the last pixel(s) of the line fit in a
            // normal frame with no lag.
            return self.render_text_scroll_pixels(remaining_in_line);
        }
        if self.dialogue_live_message_read_position_target.is_some() {
            // Endpoint catch-up: the wire's decoder already crossed this
            // scroll's lag frames on the preceding hosts; drain the whole
            // remaining line synchronously so the native cursor can reach
            // the published endpoint (route host 46822).
            return self.render_text_scroll_pixels(remaining_in_line);
        }
        // Copy slices retain the published display. CPU/vblank headroom only
        // decides whether the caller finishes before the next boundary or
        // requires the measured return-only continuation.
        let completion_timing =
            DialogueScrollCompletionTiming::at_scroll_entry(cycles_before_vblank);
        if std::env::var_os("ZELDA3_DEBUG_SCROLL_RETAIN").is_some() {
            eprintln!(
                "scroll_schedule host={} headroom={} timing={completion_timing:?}",
                self.frame_ctr_dbg, cycles_before_vblank,
            );
        }
        self.begin_dialogue_scroll(DialogueTextGeneration::PublishedDisplay, completion_timing);
        let command_done = self.render_text_scroll_pixels(2);
        debug_assert!(!command_done);
        // Phase 2 is the remaining three copy passes. Phase 1 is the
        // post-vblank caller suffix; it performs no further pixel copies.
        false
    }

    pub(super) fn dialogue_long_scroll_starts_this_frame(&self) -> bool {
        if !self.dialogue_scroll_cpu_is_idle()
            || self.game_state.frame.main_module != 0x0e
            || self.game_state.frame.submodule != 2
            || self.game_state.messaging.runtime.text_render_state() != 3
            || self.game_state.messaging.runtime.dialogue_scroll_speed() != 4
        {
            return false;
        }
        let read_pos = self.game_state.messaging.runtime.dialogue_msg_read_pos() as usize;
        let decoded = crate::dialogue_ir::decode_dialogue_byte(
            0,
            self.game_state.messaging.decoded_text.byte(read_pos),
            self.game_state.messaging.decoded_text.next_byte(read_pos),
        );
        if decoded.command != TEXT_CMD_SCROLL {
            return false;
        }
        let nibble = u16::from(
            self.game_state
                .messaging
                .dialogue_source_offset
                .bank_offset_low_nibble()
                & 0x0f,
        );
        16u16.saturating_sub(nibble) >= 5
    }

    pub(super) fn render_text_scroll_pixels(&mut self, pixels: u16) -> bool {
        for _ in 0..pixels {
            for i in (0..0x7e0).step_by(16) {
                for j in 0..7 {
                    let value = self
                        .game_state
                        .messaging
                        .render_buffer
                        .word_at_byte_offset(i + (j + 1) * 2);
                    self.set_messaging_render_buffer_word_at_byte_offset(i + j * 2, value);
                }
                let value = self
                    .game_state
                    .messaging
                    .render_buffer
                    .word_at_byte_offset(i + 168 * 2);
                self.set_messaging_render_buffer_word_at_byte_offset(i + 7 * 2, value);
            }
            for i in (0x34f..=0x3ef).step_by(8) {
                self.set_messaging_render_buffer_word_at_byte_offset(i * 2, 0);
            }
            self.scroll_bg3_vwf_glyph_runs_up_one_pixel();
            let source_bank_offset = self
                .dialogue_source_offset_mut()
                .increment_bank_offset_low_nibble();
            if source_bank_offset & 0x0f == 0 {
                self.set_vwf_current_line(4);
                self.request_vwf_next_line(1);
                return true;
            }
        }
        false
    }

    pub(super) fn RenderText_SetDefaultWindowPosition(&mut self) {
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        let flag = usize::from(y < 0x78);
        self.messaging_state_mut()
            .set_text_msgbox_topleft(TEXT_POSITIONS[flag]);
    }

    pub(super) fn RenderText_DrawBorderInitialize(&mut self) {
        let top_left = self.game_state.messaging.runtime.text_msgbox_topleft();
        self.messaging_state_mut()
            .set_text_msgbox_topleft_copy(top_left);
    }

    pub(super) fn RenderText_DrawBorderRow(&mut self, mut d: usize, y: usize) -> usize {
        let y = y >> 1;
        let top_left = self.game_state.messaging.runtime.text_msgbox_topleft_copy();
        self.write_vram_upload_absolute_word(d, top_left.swap_bytes());
        d += 2;
        self.messaging_state_mut()
            .set_text_msgbox_topleft_copy(top_left.wrapping_add(0x20));
        self.write_vram_upload_absolute_word(d, 0x2f00);
        d += 2;
        self.write_vram_upload_absolute_word(d, TEXT_BORDER_TILES[y]);
        d += 2;
        for _ in 0..22 {
            self.write_vram_upload_absolute_word(d, TEXT_BORDER_TILES[y + 1]);
            d += 2;
        }
        self.write_vram_upload_absolute_word(d, TEXT_BORDER_TILES[y + 2]);
        d += 2;
        self.write_vram_upload_absolute_word(d, 0xffff);
        d
    }

    pub(super) fn Text_BuildCharacterTilemap(&mut self) {
        let mut tile = self.game_state.messaging.runtime.text_tilemap_cur();
        for i in 0..126 {
            self.set_vwf_tile_word_at_byte_offset(i * 2, tile);
            tile = tile.wrapping_add(1);
        }
        self.messaging_state_mut().set_text_tilemap_cur(tile);
        self.RenderText_Refresh();
        self.messaging_state_mut().increment_text_render_state();
    }

    pub(super) fn RenderText_Refresh(&mut self) {
        self.RenderText_DrawBorderInitialize();
        let top_left = self
            .game_state
            .messaging
            .runtime
            .text_msgbox_topleft_copy()
            .wrapping_add(0x21);
        self.messaging_state_mut()
            .set_text_msgbox_topleft_copy(top_left);
        let mut d = 0x1002usize;
        let mut s = 0usize; // offset into VWF_TILE_BUFFER
        for _ in 0..6 {
            let row_top_left = self.game_state.messaging.runtime.text_msgbox_topleft_copy();
            self.write_vram_upload_absolute_word(d, row_top_left.swap_bytes());
            d += 2;
            self.messaging_state_mut()
                .set_text_msgbox_topleft_copy(row_top_left.wrapping_add(0x20));
            self.write_vram_upload_absolute_word(d, 0x2900);
            d += 2;
            for _ in 0..21 {
                let tile = self
                    .game_state
                    .messaging
                    .vwf_render
                    .tile_word_at_byte_offset(s);
                self.write_vram_upload_absolute_word(d, tile);
                d += 2;
                s += 2;
            }
        }
        self.write_vram_upload_absolute_word(d, 0xffff);
        self.set_bg_vram_load_mode(1);
    }

    pub(super) fn Text_GenerateMessagePointers(&mut self) {
        let Some(dialogue_blk) = self.asset_memblk(94, self.dialogue_blk_index) else {
            return;
        };
        let dialogue = find_index_in_memblk(dialogue_blk, 1).ptr.to_vec();
        let mut p = 0x1c8000u32;
        for i in 0..398 {
            if i == 359 {
                p = 0x0edf40;
            }
            self.messaging_text_mut().set_dialogue_pointer(i, p);
            let entry = find_index_in_memblk(MemBlk { ptr: &dialogue }, i);
            p = p.wrapping_add(entry.ptr.len() as u32 + 1);
        }
    }

    pub(super) fn Death_PlayerSwoon(&mut self) {
        let mut k = self.game_state.player.follower_link.item_action_step_var() as usize;
        self.follower_link_state_mut()
            .decrement_y_button_action_timer();
        if (self.game_state.player.follower_link.y_button_action_timer() as i8) < 0 {
            k += 1;
            if k == 15 {
                return;
            }
            if k == 14 {
                self.increment_submodule();
            }
            self.follower_link_state_mut()
                .set_item_action_step_var(k as u8);
            self.follower_link_state_mut()
                .set_y_button_action_step(DEATH_ANIM_CTR0[k]);
            self.follower_link_state_mut()
                .set_y_button_action_timer(DEATH_ANIM_CTR1[k]);
        }
        if k != 13 || self.game_state.player.follower_link.visibility_status() == 12 {
            return;
        }
        let y = (self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(16)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2()))
            as u8;
        let x = (self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_add(7)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2()))
            as u8;
        let flags = DEATH_SPR_FLAGS
            [self.game_state.player.follower_link.lower_level_state() as usize & 1]
            | 2;
        self.set_oam_plain(0x74, x, y, 0xaa, flags, 2);
    }

    pub(super) fn Death_PrepFaint(&mut self) {
        self.follower_link_state_mut().set_facing(2);
        self.follower_link_state_mut().set_faint_animation_active(1);
        self.follower_link_state_mut().clear_item_action_step_var();
        self.follower_link_state_mut().set_y_button_action_step(0);
        self.follower_link_state_mut().set_y_button_action_timer(5);
        {
            let mut resources = self.player_resources_mut();
            resources.set_heart_filler(0);
            resources.set_current_health(0);
        }
        self.link_reset_properties_c();
        self.follower_link_state_mut()
            .clear_somaria_platform_state();
        self.follower_link_state_mut()
            .clear_water_ripple_or_grass_state();
        self.follower_link_state_mut().clear_bunny_mirror();
        self.follower_link_state_mut().clear_defense_flags();
        self.follower_link_state_mut().clear_ancilla_pickup_flag();
        self.follower_link_state_mut().clear_auxiliary_state();
        self.follower_link_state_mut().set_incapacitated_timer(0);
        self.follower_link_state_mut().clear_given_damage();
        self.follower_link_state_mut().clear_transforming();
        self.follower_link_state_mut().set_speed_setting(0);
        self.follower_link_state_mut()
            .clear_transform_poof_need_and_temp_bunny_timer();
        if self.game_state.inventory.items.has_moon_pearl() {
            self.follower_link_state_mut().clear_bunny_body_state();
        }
        if self
            .game_state
            .enhanced_features
            .has(PLAYER_RESET_MISC_BUG_FIXES_FLAG)
        {
            self.LoadActualGearPalettes();
        }
        let sfx = 0x27 | self.link_calculate_sfx_pan();
        self.set_sound_effect_1(sfx);
        for i in 0..4 {
            if self.game_state.inventory.items.bottle(i) == 6 {
                return;
            }
        }
        self.dungeon_object_tracking_mut()
            .clear_changeable_object_index(0);
        self.dungeon_object_tracking_mut()
            .clear_changeable_object_index(1);
    }

    pub(super) fn DisplaySelectMenu(&mut self) {
        self.multiselect_choice_mut().save_backup();
        self.dialogue_message_index_mut().set_value(0x0186);
        let bak = self.game_state.frame.main_module;
        self.main_show_text_message();
        self.set_main_module(bak);
        self.set_subsubmodule(0);
        self.set_submodule(11);
        self.save_main_module_for_menu();
        self.set_main_module(14);
    }
}

impl ZeldaState {
    pub(super) fn world_map_load_light_world_map(&mut self) {
        self.world_map_fill_tilemap_with_ef();
        self.set_main_screen_layers(0x11);
        self.set_sub_screen_layers(0);
        self.transfer_mode7_characters();
        self.world_map_setup_hdma();
        self.load_overworld_map_palette();
        self.load_actual_gear_palettes();
        self.increment_cgram_update_flag();
        self.set_pending_nmi_subroutine(7);
        self.set_screen_brightness(0);
        self.increment_core_update_disable_flag();
        self.increment_overworld_map_state();
    }

    pub(super) fn world_map_fill_tilemap_with_ef(&mut self) {
        for i in 0..0x4000 {
            self.ppu.vram[i] = (self.ppu.vram[i] & 0xff00) | 0x00ef;
        }
    }

    pub(super) fn transfer_mode7_characters(&mut self) {
        if let Some(gfx) = self.asset_raw(66).map(Vec::from) {
            for i in 0..0x4000.min(gfx.len()).min(self.ppu.vram.len()) {
                self.ppu.vram[i] = (self.ppu.vram[i] & 0x00ff) | ((gfx[i] as u16) << 8);
            }
        }
    }

    pub(super) fn did_press_button_for_map(&self) -> bool {
        if self.game_state.world.transient.hud_cur_item_x() != 0 {
            self.game_state.player.follower_link.filtered_joypad_h() & 0x20 != 0
        } else {
            self.game_state.player.follower_link.filtered_joypad_l() & 0x40 != 0
        }
    }
}
