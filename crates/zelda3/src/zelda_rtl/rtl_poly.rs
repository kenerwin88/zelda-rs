//! `ZeldaState` methods split out of `zelda_rtl.rs` by topic (poly).
//! Mechanical move: bodies are unchanged; private methods became
//! `pub(super)` so the parent module and its siblings can call them.

use super::*;

/// Lag-free: the cartridge's preemptive polygon thread (the crystal
/// maiden's and the Triforce room's). Each frame the NMI handler returns into
/// the thread, which keeps the CPU until the V-IRQ hands it back to main
/// (`$00:822D` / `$00:831F` swap the stack with `$1F0A`); main then runs to the
/// next NMI, and an iteration that needs more than one frame lags, giving the
/// thread another phase. The native worker runs the thread's own code
/// instruction by instruction through exactly those phases.
#[derive(Clone, Debug)]
pub(crate) struct LagFreePolyThread {
    worker: crate::cycle_models::poly_worker::PolyWorker,
    context: crate::cycle_models::nmi_stack::NativeNmiContext,
    field: u64,
    hdma: Option<LagFreePolyHdma>,
}

/// Lag-free: the thread's view of the HDMA channels, which steal bus time
/// from it on every visible line. HDMA keeps running through main's lines,
/// and each thread NMI turns it off (`$00:8135`) and back on (`$00:82C4`,
/// [`POLY_THREAD_HDMA_ENABLE_BEFORE_RTI`] before its RTI). A long NMI (the
/// bitmap upload) therefore re-enables it after the field's line-0
/// initialisation, and the channels resume from their previous cursors.
/// Transfers only advance cursors; nothing is written.
#[derive(Clone, Debug)]
pub(crate) struct LagFreePolyHdma {
    timing: snes::HdmaTimingState,
    /// The field whose per-line events run next, and the next such line.
    field: u64,
    line: u16,
    /// WRAM at the phase's start, for table bytes.
    ram: Vec<u8>,
}

/// `STA $420C` at `$00:82C4` to the thread's first instruction: the store's
/// own write cycle, then REP/TSC/TAX/LDA/TCS/STX/PLB/PLD/PLY/PLX/PLA/RTI.
const POLY_THREAD_HDMA_ENABLE_BEFORE_RTI: u64 = 368;

impl LagFreePolyHdma {
    /// One HDMA event: its bus stall, with the cursors advanced.
    pub(crate) fn run(&mut self, event: snes::CpuBusEvent) -> u32 {
        let ram = &self.ram;
        let cycles = self.timing.stall(event, |address| hdma_table_byte(ram, address)).unwrap_or(0);
        match event {
            snes::CpuBusEvent::HdmaInit => self.line = 0,
            snes::CpuBusEvent::HdmaStart => self.line += 1,
            snes::CpuBusEvent::WramRefresh => {}
        }
        cycles
    }
}

fn hdma_table_byte(ram: &[u8], address: u32) -> Option<u8> {
    let bank = (address >> 16) as u8;
    let offset = address as u16;
    if bank == 0x7e || bank == 0x7f {
        return ram.get(usize::from(bank - 0x7e) * 0x10000 + usize::from(offset)).copied();
    }
    if (bank & 0x7f) < 0x40 && offset < 0x2000 {
        return ram.get(usize::from(offset)).copied();
    }
    ZeldaState::native_spotlight_table_byte(address)
        .or_else(|| ZeldaState::native_hdma_descriptor_byte(address))
}

impl Default for LagFreePolyThread {
    fn default() -> Self {
        // Polyhedral_InitializeThread's bootstrap frame ($1F32): DB $09,
        // D $1F00, A/X/Y 0, P $30, PC $09:F81D, popped to SP $1F3E.
        Self {
            worker: Default::default(),
            context: crate::cycle_models::nmi_stack::NativeNmiContext {
                a: 0,
                x: 0,
                y: 0,
                d: 0x1f00,
                db: 0x09,
                bank: 0x09,
                pc: 0xf81d,
                sp: 0x1f3e,
                p: 0x30,
            },
            field: 0,
            hdma: None,
        }
    }
}

/// A frame (NTSC, non-interlaced) in master cycles.
const POLY_THREAD_FRAME_MASTER_CYCLES: u64 = 357_368;
/// The NMI's acceptance on line 225 (raster cycle 86 on the route's ledger)
/// and the V-IRQ handler (acceptance to the stack swap's RTI).
const POLY_THREAD_NMI_RASTER_CYCLE: u16 = 86;
const POLY_THREAD_IRQ_HANDLER_MASTER_CYCLES: u64 = 742;
/// `NMI_UpdateIRQGFX` ($00:9347), which the thread's NMI path runs and the
/// NMI cost model does not charge: LDA $1F0C/BEQ taken/RTS (96), or the
/// bitmap DMA's setup with its system-bank register stores (544).
const POLY_THREAD_NMI_UPDATE_IRQ_GFX_MASTER_CYCLES: u64 = 96;
const POLY_THREAD_NMI_UPDATE_IRQ_GFX_UPLOAD_MASTER_CYCLES: u64 = 544;
/// A lag frame's NMI (main unfinished): the thread path alone, and with the
/// bitmap upload.
const POLY_THREAD_LAG_NMI_MASTER_CYCLES: u64 = 3_284;
const POLY_THREAD_LAG_NMI_UPLOAD_MASTER_CYCLES: u64 = 20_622;

#[cfg(test)]
mod phase_tests {
    use super::*;

    #[test]
    fn main_bus_work_preserves_worker_phase_across_dma_partition() {
        // $420B stalls main while the CPU bus transfers the bytes. Splitting
        // a fixed amount of main bus work between instructions and DMA must
        // not move the $012A activation across a different NMI.
        for prefix in [0, 900_000] {
            for suffix in [400_000, 900_000] {
                let run = |dma: bool| {
                    let mut game = ZeldaState::new();
                    game.set_lag_free(true);
                    game.lag_free_frame_master_start = crate::cycle_ledger::master();
                    game.native_begin_main_cost();
                    let before_dma = if dma { prefix / 2 } else { 0 };
                    crate::cycle_ledger::charge(prefix - before_dma);
                    game.native_note_main_dma(before_dma);
                    game.activate_nmi_thread();
                    game.set_vertical_irq_trigger(144);
                    let after_dma = if dma { suffix / 2 } else { 0 };
                    crate::cycle_ledger::charge(suffix - after_dma);
                    game.native_note_main_dma(after_dma);
                    game.native_finish_main_cost();
                    game.lag_free_poly_thread_lag_phases();
                    assert_eq!(game.ram[0x1f0c], 0, "GO is clear; no publication");
                    let thread = game.lag_free_poly_thread.unwrap();
                    (thread.field, thread.context, thread.worker)
                };
                assert_eq!(run(false), run(true), "prefix={prefix}, suffix={suffix}");
            }
        }
    }
}

impl ZeldaState {
    /// CPU-only display evidence at a polygon NMI, including the live upload
    /// destination rather than just the worker's possibly partial bitmap.
    pub(crate) fn debug_poly_nmi_state(&self, kind: &str, uploaded: bool) {
        let Some(path) = crate::debug_env::var_os!("ZELDA3_DEBUG_POLY_NMI_STATE") else { return };
        use std::io::Write;
        let ram = self.wram_image();
        let cursor = self.lag_free_poly_thread.as_ref().map(|thread| {
            let c = &thread.context;
            [thread.field, u64::from(c.bank) << 16 | u64::from(c.pc),
                u64::from(c.a), u64::from(c.x), u64::from(c.y), u64::from(c.sp),
                u64::from(c.d), u64::from(c.db), u64::from(c.p)]
        });
        let row = format!(
            "{{\"host\":{},\"kind\":\"{}\",\"module\":{:?},\"work\":{:?},\"bitmap\":{:?},\"vram\":{:?},\"cgram\":{:?},\"oam\":{:?},\"mode\":{},\"brightness\":{},\"fixed_color\":{:?},\"worker_cursor\":{},\"uploaded\":{}}}",
            self.frame_ctr_dbg, kind, &ram[0x10..0x14], &ram[0x1f00..0x2000],
            &ram[0xe800..0xf000], &self.ppu.vram[0x5800..0x5c00],
            self.ppu.cgram, self.ppu.oam, self.ppu.mode, self.ppu.brightness,
            [self.ppu.fixed_color_r, self.ppu.fixed_color_g, self.ppu.fixed_color_b],
            cursor.map_or_else(|| "null".to_owned(), |c| format!("{c:?}")),
            uploaded);
        let mut file = std::fs::OpenOptions::new().create(true).append(true).open(path)
            .expect("open polygon NMI evidence");
        writeln!(file, "{row}").expect("write polygon NMI evidence");
    }

    /// Fixed logical work quanta for the one cold boot intro. No bus price or
    /// raster position participates in this scheduler.
    pub(super) fn lag_free_intro_poly_before_main(&mut self) {
        let Some(iteration) = self.lag_free_intro_schedule else { return };
        let Some((expected, (before, _))) = super::intro_poly_schedule::work(iteration) else {
            self.lag_free_intro_schedule = None;
            return;
        };
        let actual = (self.game_state.frame.main_module, self.game_state.frame.submodule);
        if actual != expected {
            // A skip leaves the fixed boot sequence; ordinary lagless rules
            // own the outgoing scene from this point onward.
            self.lag_free_intro_schedule = None;
            self.lag_free_poly_thread = None;
            return;
        }
        self.lag_free_intro_poly_work(before);
    }

    pub(super) fn lag_free_intro_poly_after_main(&mut self) {
        if self.lag_free_intro_after_ran { return; }
        self.lag_free_intro_after_ran = true;
        let Some(iteration) = self.lag_free_intro_schedule else { return };
        if let Some((expected, _)) = super::intro_poly_schedule::work(iteration + 1) {
            let actual = (self.game_state.frame.main_module, self.game_state.frame.submodule);
            if actual != expected {
                self.lag_free_intro_schedule = None;
                self.lag_free_poly_thread = None;
                return;
            }
        }
        let (_, (_, after)) = super::intro_poly_schedule::work(iteration).unwrap();
        self.lag_free_intro_poly_work(after);
    }

    pub(super) fn finish_lag_free_intro_iteration(&mut self) {
        if let Some(iteration) = self.lag_free_intro_schedule.as_mut() {
            *iteration += 1;
            if super::intro_poly_schedule::work(*iteration).is_none() {
                self.lag_free_intro_schedule = None;
                self.lag_free_poly_thread = None;
            }
        }
    }

    fn lag_free_intro_poly_work(&mut self, units: u16) {
        if units == 0 || !self.game_state.display.nmi_thread_active { return; }
        let mut thread = self.lag_free_poly_thread.take().unwrap_or_default();
        // The worker needs a bus cursor for instruction semantics. Elapsed
        // cycles and bus events select neither work nor publication here.
        let mut clock = snes::CpuMasterTimeline::new(0, snes::CpuBusWorkload::default(), snes::CpuFieldTiming::NON_INTERLACE_EVEN);
        clock.begin_synchronous_timeline().expect("intro owns its native work cursor");
        let complete = units == super::intro_poly_schedule::COMPLETE;
        crate::cycle_ledger::muted(|| {
            for _ in 0..if complete { 30_000 } else { usize::from(units) } {
                if complete && self.ram[0x1f00] == 0 && thread.context.pc == 0xf81d { break; }
                self.step_native_poly_worker_with_vblank(&mut thread.worker, &mut thread.context, &mut clock, |_, _| {});
            }
        });
        assert!(!complete || (self.ram[0x1f00] == 0 && thread.context.pc == 0xf81d), "intro render must finish its native publication");
        self.lag_free_poly_thread = Some(thread);
    }

    /// Lag-free: this frame's main-loop cost, ROM-equivalent: the cycle
    /// ledger from the frame's start to its NMI's acceptance (the NMI's own
    /// charges are not main's) plus the glyph renderer's modelled cost.
    pub fn lag_free_main_master(&self) -> u64 {
        let end = self
            .ledger_master_at_nmi_acceptance
            .filter(|&at| at >= self.lag_free_frame_master_start)
            .unwrap_or_else(crate::cycle_ledger::master);
        end.saturating_sub(self.lag_free_frame_master_start) + self.lag_free_vwf_master
    }

    /// Lag-free and a preemptive polygon thread running (the crystal
    /// maidens' and the Triforce room's): the native worker owns the thread
    /// and runs it through the cartridge's own schedule, its lag frames
    /// derived from main's cycle-ledger cost. Every crystal cutscene and the
    /// Triforce room match the ROM ledger iteration for iteration.
    pub(super) fn lag_free_native_poly_thread(&self) -> bool {
        self.lag_free
            && (self.dungeon_poly_thread_is_active() || self.triforce_room_poly_thread_is_active())
    }

    /// The NMI's acceptance and the V-IRQ deadline that ends the thread's
    /// phase in thread field `field`.
    fn poly_thread_phase_window(&self, field: u64) -> (u64, u64) {
        let timing = snes::CpuFieldTiming::NON_INTERLACE_EVEN;
        let base = snes::CpuMasterTimeline::new(0, snes::CpuBusWorkload::default(), timing);
        let nmi_at = base.master_cycles_at_raster(
            field,
            snes::CpuRasterPosition::new(snes::NMI_SCANLINE as u16, POLY_THREAD_NMI_RASTER_CYCLE),
        );
        let at_nmi = snes::CpuMasterTimeline::new(nmi_at, snes::CpuBusWorkload::default(), timing);
        let virq = u16::from(self.game_state.display.vertical_irq_trigger);
        let deadline = snes::cpu_timeline::vertical_irq_deadline(&at_nmi, virq, false)
            .map_or(nmi_at, |deadline| deadline.master_cycles());
        (nmi_at, deadline)
    }

    /// Main's work per frame while the thread runs, in the cycle ledger's
    /// instruction cycles: the frame less the thread's NMI-to-V-IRQ share and
    /// the V-IRQ handler, less the bus stalls on main's lines (the WRAM
    /// refresh every line, HDMA every visible line).
    fn poly_thread_main_master_cycles_per_frame(&self) -> u64 {
        let field = self.lag_free_poly_thread.as_ref().map_or(0, |thread| thread.field);
        let (nmi_at, deadline) = self.poly_thread_phase_window(field);
        let (next_nmi, _) = self.poly_thread_phase_window(field + 1);
        let wall = (next_nmi - nmi_at).saturating_sub(deadline - nmi_at + POLY_THREAD_IRQ_HANDLER_MASTER_CYCLES);
        let lines = u64::from(snes::NMI_SCANLINE).saturating_sub(u64::from(self.game_state.display.vertical_irq_trigger));
        wall.saturating_sub(lines * self.poly_thread_line_stall_master_cycles())
    }

    /// V-IRQ worker owns the CPU after NMI returns until its programmed
    /// deadline; the IRQ handler returns it to main. No worker instructions
    /// are added to main's cost counter.
    pub(super) fn native_pacing_worker_share(&self, handler: u64) -> u64 {
        // $00:8144..8149 tests $012A before NMI_SwitchThread $00:822D;
        // Interrupt_IRQ $00:82D8 returns CPU ownership at the programmed
        // V-IRQ even when Polyhedral_RunThread $09:F81D is waiting on GO.
        // Module routing does not disable that hardware ownership.
        if !self.lag_free || !self.game_state.display.nmi_thread_active { return 0; }
        let field = self.lag_free_poly_thread.as_ref().map_or(0, |thread| thread.field);
        let (nmi, deadline) = self.poly_thread_phase_window(field);
        if self.native_pacing.worker_after_irq {
            // $00:8324..832A swaps to the worker when IRQ arrives before
            // the first threaded NMI. That NMI swaps back to main.
            // IRQ executes inside this IRQ-to-NMI interval. Adding its
            // handler again would count the same ownership twice.
            return POLY_THREAD_FRAME_MASTER_CYCLES - (deadline - nmi);
        }
        (deadline - nmi).saturating_sub(handler) + POLY_THREAD_IRQ_HANDLER_MASTER_CYCLES
    }

    /// Bus stalls per visible line: the WRAM refresh and the enabled HDMA
    /// channels' transfers.
    fn poly_thread_line_stall_master_cycles(&self) -> u64 {
        u64::from(snes::WRAM_REFRESH_STALL_MASTER_CYCLES) + u64::from(self.native_hdma_scanline_stall_master_cycles())
    }

    /// One thread phase: from the NMI handler's return, `nmi_master_cycles`
    /// after its acceptance, to the V-IRQ that hands the CPU back to main.
    pub(super) fn lag_free_run_poly_thread_phase(&mut self, nmi_master_cycles: u64) {
        let mut thread = self.lag_free_poly_thread.take().unwrap_or_default();
        let (nmi_at, deadline) = self.poly_thread_phase_window(thread.field);
        let mut hdma = thread.hdma.take().unwrap_or_else(|| LagFreePolyHdma {
            timing: snes::HdmaTimingState::new(&self.dma),
            field: thread.field,
            line: 0,
            ram: Vec::new(),
        });
        // A long NMI can return inside a bus stall (the HDMA initialisation
        // at the next field's first line); the thread resumes when it ends.
        let mut start = nmi_at + nmi_master_cycles;
        self.lag_free_poly_hdma_through_nmi(&mut hdma, thread.field, start);
        let mut clock = loop {
            let mut clock = snes::CpuMasterTimeline::new(
                start,
                snes::CpuBusWorkload::with_dynamic_hdma(),
                snes::CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            match clock.begin_synchronous_timeline() {
                Ok(()) => break clock,
                // Inside a bus stall the thread resumes when it ends; on a
                // render event (no bus time) the event is retired before the
                // thread's first instruction, as Snes9x drains due events.
                Err(snes::cpu_timeline::CpuSynchronousTimelineStartError::AmbiguousEventState { .. }
                    | snes::cpu_timeline::CpuSynchronousTimelineStartError::AmbiguousRenderEvent { .. }) => start += 1,
                Err(error) => panic!("a fresh polygon-thread clock claims its synchronous timeline: {error:?}"),
            }
        };
        crate::cycle_ledger::muted(|| {
            let mut irq_masked = thread.context.p & 4 != 0;
            let step_trace = crate::debug_env::var_os!("ZELDA3_DEBUG_POLY_STEPS").is_some();
            while clock.clock_master_cycles() < deadline || irq_masked {
                let (pc, before) = ((u32::from(thread.context.bank) << 16) | u32::from(thread.context.pc), clock.clock_master_cycles());
                let retired = self.step_native_poly_worker_with_hdma(&mut thread.worker, &mut thread.context, &mut clock, &mut hdma);
                if step_trace {
                    eprintln!("[POLY-STEP] host={} pc={pc:06x} start={} end={}", self.frame_ctr_dbg,
                        before - nmi_at, clock.clock_master_cycles() - nmi_at);
                }
                irq_masked = retired.irq_masked;
            }
        });
        if crate::debug_env::var_os!("ZELDA3_DEBUG_POLY").is_some() {
            eprintln!("[POLY-PHASE] host={} field={} nmi_cost={nmi_master_cycles} start={} budget={} ran={} pc={:02x}{:04x} go={} upload={}",
                self.frame_ctr_dbg, thread.field, start - nmi_at, deadline.saturating_sub(start),
                clock.clock_master_cycles() - start, thread.context.bank, thread.context.pc,
                self.ram[0x1f00], self.ram[0x1f0c]);
        }
        thread.field += 1;
        thread.hdma = Some(hdma);
        self.lag_free_poly_thread = Some(thread);
    }

    /// Bring the thread's HDMA view from its last event up to `start`, the
    /// thread's first instruction after the NMI of `field`: main's remaining
    /// lines of `field`, the NMI's disable, and its re-enable relative to
    /// the next field's line-0 initialisation and first lines.
    fn lag_free_poly_hdma_through_nmi(&self, hdma: &mut LagFreePolyHdma, field: u64, start: u64) {
        use snes::{CpuBusEvent, CpuRasterPosition, cpu_timeline::{HDMA_INIT_CYCLE, HDMA_START_CYCLE}};
        let base = snes::CpuMasterTimeline::new(0, snes::CpuBusWorkload::default(), snes::CpuFieldTiming::NON_INTERLACE_EVEN);
        let at = |field: u64, line: u16, cycle: u32| base.master_cycles_at_raster(field, CpuRasterPosition::new(line, cycle as u16));
        // The HDMA timing bus keeps a detached cartridge image. Its reads
        // cannot consult exclusive owners' poisoned backing bytes.
        self.wram_image().as_ref().clone_into(&mut hdma.ram);
        hdma.timing.adopt_registers(&self.dma);
        let enabled = self.game_state.display.hdma_enable_mask;
        if hdma.field == field {
            while u32::from(hdma.line) < snes::NMI_SCANLINE {
                hdma.run(CpuBusEvent::HdmaStart);
            }
        }
        hdma.timing.set_enable_mask(0);
        hdma.field = field + 1;
        hdma.line = 0;
        let enable_at = start.saturating_sub(POLY_THREAD_HDMA_ENABLE_BEFORE_RTI);
        let init_at = at(field + 1, 0, HDMA_INIT_CYCLE);
        if init_at >= start {
            hdma.timing.set_enable_mask(enabled);
            return;
        }
        if enable_at < init_at {
            hdma.timing.set_enable_mask(enabled);
        }
        hdma.run(CpuBusEvent::HdmaInit);
        hdma.timing.set_enable_mask(enabled);
        while u32::from(hdma.line) < snes::NMI_SCANLINE && at(field + 1, hdma.line, HDMA_START_CYCLE) < start {
            hdma.run(CpuBusEvent::HdmaStart);
        }
    }

    /// The thread's phase before this frame's main: after the NMI that closed
    /// the previous iteration.
    pub(super) fn lag_free_poly_thread_leading_phase(&mut self) {
        let uploaded = std::mem::take(&mut self.lag_free_nmi_uploaded_poly);
        let field = self.lag_free_poly_thread.as_ref().map_or(0, |thread| thread.field);
        let nmi = self.poly_thread_nmi_wall_master_cycles(field, Some(uploaded));
        self.lag_free_run_poly_thread_phase(nmi);
    }

    /// The NMI handler's wall time from its acceptance in thread field
    /// `field`: its modelled cost (instructions and DMA) and
    /// `NMI_UpdateIRQGFX` on the thread's path (`None` before the thread
    /// runs), stretched by the WRAM refresh of every line it spans.
    fn poly_thread_nmi_wall_master_cycles(&self, field: u64, uploaded: Option<bool>) -> u64 {
        let (nmi_at, _) = self.poly_thread_phase_window(field);
        // The phase window's NMI point follows the hardware entry, which the
        // handler's cost includes.
        // Presentation-only held NMIs follow the native closing callback;
        // the source worker resumes through the ordinary closing handler.
        let work = self.native_previous_closing_handler()
                .saturating_sub(crate::zelda_rtl::nmi::NATIVE_NMI_ENTRY_MASTER_CYCLES)
            + match uploaded {
                Some(true) => POLY_THREAD_NMI_UPDATE_IRQ_GFX_UPLOAD_MASTER_CYCLES,
                Some(false) => POLY_THREAD_NMI_UPDATE_IRQ_GFX_MASTER_CYCLES,
                None => 0,
            };
        let mut clock = snes::CpuMasterTimeline::new(
            nmi_at,
            snes::CpuBusWorkload::default(),
            snes::CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        clock.advance_work_unbounded(u32::try_from(work).unwrap_or(u32::MAX));
        clock.clock_master_cycles() - nmi_at
    }

    /// Main's store of the thread's go byte (`$1F00` = 1, the step it hands
    /// the thread). Lag-free, the frame's main cost at the first store from
    /// 0 decides which lag frames' thread phases still find the thread idle.
    pub(crate) fn mark_poly_thread_go(&mut self) {
        if self.lag_free
            && self.lag_free_go_master.is_none()
            && self.ram[crate::game_state::constants::INTRO_DID_RUN_STEP] == 0
        {
            self.lag_free_go_master = Some(
                crate::cycle_ledger::master().saturating_sub(self.lag_free_frame_master_start)
                    + self.lag_free_vwf_master + self.native_cost_main_dma(),
            );
        }
        self.attract_scene_mut().mark_intro_did_run_step();
    }

    /// After this frame's main: the frames it would have taken beyond the
    /// first are lag frames, each a thread phase after a lag NMI. Main owns
    /// the whole frame (less the NMI) until it activates the thread; only the
    /// NMIs after that return into the thread.
    pub(crate) fn lag_free_poly_thread_lag_phases(&mut self) {
        self.lag_free_lag_phases_run = true;
        // Main's $420B transfers own the CPU bus too; their bytes were
        // recorded by the existing DMA owner, outside instruction prices.
        let cost = self.lag_free_main_master() + self.native_cost_main_dma();
        self.lag_free_last_main_master = cost;
        let threaded = self.poly_thread_main_master_cycles_per_frame().max(1);
        let activation = self.lag_free_thread_activation_master.take();
        let mut phases = 0;
        let mut phase_plan: Vec<bool> = Vec::new();
        let mut done = 0u64;
        let mut active = activation.is_none();
        loop {
            let field = self.lag_free_poly_thread.as_ref().map_or(0, |thread| thread.field);
            // $00:813A skips NMI_DoUpdates while main is unfinished. Only
            // the opening NMI runs the ordinary closing handler; following
            // unthreaded lag NMIs use its latched, update-free path.
            let handler = if done == 0 {
                self.poly_thread_nmi_wall_master_cycles(field, None)
            } else {
                self.native_cost_lag_handler().saturating_sub(
                    crate::zelda_rtl::nmi::NATIVE_NMI_ENTRY_MASTER_CYCLES)
            };
            let unthreaded = (POLY_THREAD_FRAME_MASTER_CYCLES - handler)
                .saturating_sub(u64::from(snes::NMI_SCANLINE) * self.poly_thread_line_stall_master_cycles());
            done += if active { threaded } else { unthreaded };
            if activation.is_some_and(|at| done >= at) {
                active = true;
            }
            if done >= cost {
                break;
            }
            if active {
                // A lag NMI before main stored the go byte finds the thread
                // still waiting for it.
                let idle = self.lag_free_go_master.is_some_and(|go| done < go);
                phase_plan.push(idle);
                phases += 1;
            }
        }
        if crate::debug_env::var_os!("ZELDA3_DEBUG_POLY").is_some() {
            eprintln!("[POLY-LAGFREE] host={} cost={cost} activation={activation:?} go_at={:?} phases={phases} go={} upload={}",
                self.frame_ctr_dbg, self.lag_free_go_master, self.ram[0x1f00], self.ram[0x1f0c]);
        }
        for idle in phase_plan {
            if idle {
                // The thread waits for the go byte, but HDMA still runs.
                if let Some(mut thread) = self.lag_free_poly_thread.take() {
                    if let Some(mut hdma) = thread.hdma.take() {
                        let (nmi_at, deadline) = self.poly_thread_phase_window(thread.field);
                        self.lag_free_poly_hdma_through_nmi(&mut hdma, thread.field, nmi_at + POLY_THREAD_LAG_NMI_MASTER_CYCLES);
                        let base = snes::CpuMasterTimeline::new(0, snes::CpuBusWorkload::default(), snes::CpuFieldTiming::NON_INTERLACE_EVEN);
                        while u32::from(hdma.line) < snes::NMI_SCANLINE
                            && base.master_cycles_at_raster(hdma.field, snes::CpuRasterPosition::new(hdma.line,
                                snes::cpu_timeline::HDMA_START_CYCLE as u16)) < deadline
                        {
                            hdma.run(snes::CpuBusEvent::HdmaStart);
                        }
                        thread.hdma = Some(hdma);
                    }
                    thread.field += 1;
                    self.lag_free_poly_thread = Some(thread);
                }
                continue;
            }
            let nmi = if self.ram[crate::game_state::constants::NMI_FLAG_UPDATE_POLYHEDRAL] != 0 {
                self.lag_free_poly_lag_nmi_upload();
                POLY_THREAD_LAG_NMI_UPLOAD_MASTER_CYCLES
            } else {
                POLY_THREAD_LAG_NMI_MASTER_CYCLES
            };
            self.lag_free_run_poly_thread_phase(nmi);
        }
    }

    /// A lag NMI's `NMI_UpdateIRQGFX`: the finished bitmap goes to VRAM
    /// $5800 and `$1F0C` clears, so the thread starts its next render and
    /// the lag NMIs after it only swap.
    fn lag_free_poly_lag_nmi_upload(&mut self) {
        let bitmap = match self.poly_completed_upload.take() {
            Some((_, completed)) => completed,
            None => self.polyhedral_tile_buffer().to_vec(),
        };
        for i in 0..0x400 {
            self.ppu.vram[0x5800 + i] = u16::from_le_bytes([bitmap[i * 2], bitmap[i * 2 + 1]]);
        }
        self.clear_pending_polyhedral_update();
        self.debug_poly_nmi_state("lag-upload", true);
    }

    pub fn intro_poly_upload_delay(&self) -> u8 {
        self.intro_poly_upload_delay
    }

    pub fn debug_nmi_poly_upload_deferred(&self) -> u8 {
        self.nmi_poly_upload_deferred
    }

    pub fn debug_nmi_poly_upload_started(&self) -> bool {
        self.nmi_poly_upload_started
    }

    pub fn debug_snes9x_poly_scheduler_counter(&self) -> u8 {
        self.snes9x_poly_scheduler_counter
    }

    pub fn debug_last_poly_work(&self) -> PolyWorkMetrics {
        self.last_poly_work
    }

    pub(crate) fn request_polyhedral_nmi_update(&mut self) {
        self.display_core_mut().request_polyhedral_nmi_update();
    }

    pub(crate) fn clear_pending_polyhedral_update(&mut self) {
        self.display_core_mut().clear_pending_polyhedral_update();
    }

    pub(crate) fn polyhedral_tile_buffer(&self) -> &[u8] {
        self.game_state.display.polyhedral_tile_buffer(&self.ram)
    }

    pub(super) fn selected_intro_poly_display_buffer(&self) -> Vec<u16> {
        if env::var_os("ZELDA3_INTRO_POLY_PRESENT_OBJ_LATCH").is_some() {
            if let Some(latched_vram) = self.ppu.obj_vram_latch.as_deref() {
                if latched_vram.len() >= 0x5c00 {
                    return latched_vram[0x5800..0x5c00].to_vec();
                }
            }
        }
        let diagnostic_lag = env::var("ZELDA3_INTRO_POLY_PRESENT_HISTORY_LAG")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(0);
        if diagnostic_lag != 0 {
            if let Some((_, vram, _)) = self
                .intro_poly_vram_history
                .len()
                .checked_sub(1 + diagnostic_lag)
                .and_then(|index| self.intro_poly_vram_history.get(index))
            {
                return vram.clone();
            }
        }
        self.ppu.vram[0x5800..0x5c00].to_vec()
    }

    /// The crystal maiden's IRQ poly thread: active from her Sprite_AB
    /// activation through her dialogue (Module0E keeps calling Sprite_Main,
    /// which re-arms the thread) until the cutscene deactivates it.
    pub(super) fn dungeon_poly_thread_is_active(&self) -> bool {
        self.game_state.display.nmi_thread_active
            && matches!(self.game_state.frame.main_module, 0x07 | 0x0e)
            && !self.triforce_room_poly_thread_is_active()
    }

    /// `Module19_TriforceRoom`'s V-IRQ thread (and the same thread while
    /// its Triforce message runs under module $0E with $19 saved as the
    /// return module).
    pub(crate) fn triforce_room_poly_thread_is_active(&self) -> bool {
        self.game_state.display.nmi_thread_active
            && (matches!(self.game_state.frame.main_module, 0x19 | 0x1a)
                || (self.game_state.frame.main_module == 0x0e
                    && self.game_state.frame.saved_module_for_menu == 0x19))
    }

    /// Whether a preemptive (V-IRQ switched) polyhedral thread is live under
    /// original timing: the dungeon crystal maiden or the Triforce room.
    pub(super) fn preemptive_poly_thread_is_active(&self) -> bool {
        self.rom_startup_timing
            && (self.dungeon_poly_thread_is_active() || self.triforce_room_poly_thread_is_active())
    }

    /// Called at every NMI entry: measure this NMI handler's duration to its
    /// thread swap on the RAM the handler will see, so the next host's thread
    /// slot starts at the right raster position.
    pub(super) fn stash_preemptive_poly_thread_nmi_swap(&mut self) {
        if !self.preemptive_poly_thread_is_active() {
            self.poly_next_host_swap_master = None;
            return;
        }
        // The shadow's slot pairing consumes this stash one host later, so
        // the swap-duration bookkeeping keeps attributing the upload to the
        // NMI of the completing host (the calibration of the dialogue-phase
        // cadence, hosts 414033-416096, rests on that pairing); only the
        // uploaded VRAM content is deferred to the next host's NMI. A deferred
        // upload executing at that next NMI therefore does not count again.
        let upload = self.game_state.display.has_pending_polyhedral_update()
            && self
                .poly_completed_upload
                .as_ref()
                .is_none_or(|(host, _)| *host >= self.frame_ctr_dbg);
        // The NMI this trailing handler models is normally accepted at the
        // end of the current host's run (the last `NmiAccepted` of its
        // receipts). When the run boundary falls just before the acceptance,
        // the receipt moves to the head of the next host's vector and the
        // slot consumer reads it there.
        let wire_latch = self
            .original_timing_semantic_receipts
            .as_ref()
            .and_then(|receipts| {
                receipts
                    .semantic()
                    .iter()
                    .rev()
                    .find_map(|receipt| match receipt {
                        OriginalTimingSemanticReceipt::NmiAccepted(gate) => {
                            Some(matches!(gate, NmiUpdateGate::LatchHeld))
                        }
                        _ => None,
                    })
            });
        let latch = wire_latch;
        self.poly_next_host_nmi_state = Some((latch, upload));
        self.poly_next_host_swap_master = None;
        if crate::debug_env::var_os!("ZELDA3_DEBUG_POLY").is_some() {
            eprintln!(
                "[POLY-STASH] host={} latch={:?} upload={}",
                self.frame_ctr_dbg, latch, upload,
            );
        }
    }

    /// Start the shadow render of the frame the ROM thread begins this host.
    /// Must run before the native rasterizer mutates the poly runtime state.
    pub(super) fn begin_poly_shadow_frame(&mut self) {
        const THREAD_WORK_RANGE: std::ops::Range<usize> = 0x1f10..0x2000;
        const THREAD_BITMAP_RANGE: std::ops::Range<usize> = 0xe800..0xf000;
        let timing_dma = self.dma_with_native_hdma_enable();
        let mut ram_image = self.wram_image().into_owned();
        if let Some(saved) = self.triforce_poly_shadow_ram.as_ref() {
            ram_image[THREAD_WORK_RANGE].copy_from_slice(&saved[THREAD_WORK_RANGE]);
            ram_image[THREAD_BITMAP_RANGE].copy_from_slice(&saved[THREAD_BITMAP_RANGE]);
        }
        self.poly_shadow_run = RomCpuTimingRun::new(
            &self.rom,
            &ram_image,
            &self.sram,
            &self.ppu,
            &timing_dma,
            self.zelda_audio_apu_output_ports(),
            TRIFORCE_ROOM_POLY_RENDER_CPU_CHECKPOINT,
        )
        .ok();
        self.poly_shadow_master = 0;
        self.poly_shadow_hosts = 0;
    }

    /// Lag-free: the frames this iteration would have taken on the cartridge
    /// while a preemptive (V-IRQ) polygon thread runs, from its ROM-equivalent
    /// CPU cost. Main gets 233.5k-241k master cycles of each frame then (the
    /// range that reproduces the frame count of every crystal-sequence
    /// iteration on the route: the activations, the dialogue's opening and
    /// its three-frame line scrolls); each frame beyond the first gives the
    /// thread one more slice, which a lag-free iteration has to credit
    /// explicitly.
    /// Lag-free: the last frame's ROM-equivalent main cost (cycle ledger plus
    /// the glyph renderer's modelled cost), set while a polygon thread runs.
    pub fn lag_free_last_main_master(&self) -> u64 {
        self.lag_free_last_main_master
    }

    /// Lag-free: this frame's glyph-renderer cost, which the cycle ledger
    /// does not carry (the renderer prices itself with its own model).
    pub fn lag_free_vwf_master(&self) -> u64 {
        self.lag_free_vwf_master
    }

    /// The last NMI handler's modelled cost (ledger plus DMA bus time).
    pub fn last_nmi_handler_master_cycles(&self) -> Option<u32> {
        self.last_nmi_handler_master_cycles
    }

    pub(super) fn zelda_run_poly_loop(&mut self) {
        if self.lag_free_intro_schedule.is_some() {
            return;
        }
        if self.lag_free_native_poly_thread() {
            self.lag_free_poly_thread_leading_phase();
            return;
        }
        /// Crystal scanlines the maiden's IRQ poly thread rasterizes per host.
        const DUNGEON_POLY_THREAD_SCANLINES_PER_HOST: u32 = 32;
        /// Per-frame setup of the crystal thread, in scanline equivalents,
        /// while Module07 runs the cutscene.
        const DUNGEON_POLY_THREAD_SETUP_SCANLINES: u32 = 67;
        /// Through the maiden's Module0E dialogue the thread's per-host share
        /// in estimated 65816 cycles and the affine per-frame offset (fit on
        /// the oracle's step cadence at route hosts 414033-414115).
        const DUNGEON_POLY_THREAD_DIALOGUE_BUDGET_CYCLES: i64 = 7_572;
        const DUNGEON_POLY_THREAD_DIALOGUE_SETUP_CYCLES: i64 = -20_641;
        // The C port interleaves one poly step per main-loop step
        // (`intro_did_run_step`). The ROM's V-IRQ threads (dungeon crystal
        // maiden, Triforce room) start their next frame at the next host's
        // swap whether or not the main iteration is held by the wire (oracle
        // 415132: the thread restarts at V=257 while the Module0E iteration
        // is still suspended); they wait only for the NMI upload of the
        // finished frame.
        // `intro_did_run_step` IS the ROM thread's go byte $1F00: the thread
        // loop ($09:F81D) spins until the main thread sets it (the crystal
        // maiden's `Sprite_AB_CrystalMaiden` at V≈65 of its main slot, once
        // per completed frame) and the NMI has cleared the upload byte $1F0C.
        // A bitmap completed in an earlier host's slot is uploaded by THIS
        // host's NMI, which precedes this host's slot chronologically: the
        // thread may start its next frame in this slot (oracle run 413573:
        // upload at its NMI, `$09:F825` restart at V=261 of the same run).
        let pending_upload_precedes_this_slot = self
            .poly_completed_upload
            .as_ref()
            .is_some_and(|(host, _)| *host < self.frame_ctr_dbg);
        // Lag-free, the thread runs only while the NMI hands it the CPU
        // ($012A): leaving the intro mid-render (save-and-quit back to the
        // file select) must not finish that render into the menu's frame.
        let can_run_poly = self.game_state.ending.attract_scene.intro_did_run_step() != 0
            && (!self.lag_free || self.game_state.display.nmi_thread_active)
            && (!self.game_state.display.has_pending_polyhedral_update()
                || pending_upload_precedes_this_slot);
        let live_timing = matches!(self.original_timing_owner, OriginalTimingOwnerState::Live);
        let source_started_preemptive_render =
            self.take_original_timing_preemptive_polyhedral_render_started();
        if source_started_preemptive_render {
            assert!(
                self.preemptive_poly_thread_is_active(),
                "source started a preemptive poly render outside the native preemptive-thread phase",
            );
            assert!(
                can_run_poly,
                "source started a preemptive poly render while the native go/upload gates rejected it",
            );
        }
        if self.preemptive_poly_thread_is_active() {
            // A run's leading acceptance (before any handler completion)
            // belongs to the handler that begins this run's slot; a trailing
            // acceptance (no completion after it) belongs to the next run's.
            let gates = self
                .original_timing_semantic_receipts
                .as_ref()
                .map(|receipts| {
                    let held = |receipt: &OriginalTimingSemanticReceipt| match receipt {
                        OriginalTimingSemanticReceipt::NmiAccepted(gate) => {
                            Some(matches!(gate, NmiUpdateGate::LatchHeld))
                        }
                        _ => None,
                    };
                    let completed = |receipt: &OriginalTimingSemanticReceipt| {
                        matches!(receipt, OriginalTimingSemanticReceipt::NmiHandlerCompleted)
                    };
                    let semantic = receipts.semantic();
                    let leading = semantic
                        .iter()
                        .take_while(|receipt| !completed(receipt))
                        .find_map(held);
                    let trailing = semantic
                        .iter()
                        .rev()
                        .take_while(|receipt| !completed(receipt))
                        .find_map(held);
                    (leading, trailing)
                });
            self.poly_receipt_gates_prev = self.poly_receipt_gates_cur.take();
            self.poly_receipt_gates_cur = gates;
        } else {
            self.poly_receipt_gates_prev = None;
            self.poly_receipt_gates_cur = None;
        }
        if self.dungeon_poly_thread_is_active()
            && crate::debug_env::var_os!("ZELDA3_DEBUG_POLY").is_some()
        {
            eprintln!(
                "[POLY] host={} entry: did_run_step={} pending_update={} in_flight={} hold_frames={} startup_hold={:?} maiden_ai={}",
                self.frame_ctr_dbg,
                self.game_state.ending.attract_scene.intro_did_run_step(),
                self.game_state.display.has_pending_polyhedral_update(),
                self.poly_job_in_flight,
                self.poly_job_hold_frames,
                self.poly_dungeon_thread_startup_hold,
                self.sprite_slot_view(15).ai_state(),
            );
        }
        if can_run_poly {
            let frame = self.game_state.frame;
            // The crystal maiden's cutscene (Sprite_AB, dungeon module) runs
            // the same IRQ-driven poly worker as the title triforce; its
            // frame cost gates the cutscene sprite the same way (route host
            // 413549: the ROM's first poly frame held the maiden four hosts).
            let dungeon_poly_thread = self.dungeon_poly_thread_is_active();
            // The Triforce room's V-IRQ thread renders the zooming Triforce
            // while Module19 runs the text, sprites, and LinkOam each frame:
            // the oracle's zoom steps land every host whose render costs
            // less than one host's thread share and every second host
            // otherwise (route hosts 1557810-1558053: single-host steps at
            // config1 $cf/$cd/$cb, $c1/$bf/$bd, $a1/$9f/$9d = the frames
            // under ~25.2k estimated cycles; 44.9k-cycle frames still take
            // two hosts).
            let triforce_room_poly_thread = self.triforce_room_poly_thread_is_active();
            const TRIFORCE_ROOM_POLY_THREAD_HOST_CYCLES: u32 = 25_200;
            if !self.game_state.display.nmi_thread_active {
                self.poly_dungeon_thread_startup_hold = None;
                self.poly_dungeon_frames_rendered = 0;
                self.triforce_poly_shadow_ram = None;
                self.poly_shadow_run = None;
                self.poly_shadow_hosts = 0;
                self.poly_shadow_master = 0;
                self.poly_completed_upload = None;
            }
            // Lag-free frames pace every thread by its render cost as the ROM
            // timing does; the incremental ROM-CPU shadow needs the cartridge.
            // Lag-free, any active thread is paced by its render cost: a
            // module that ends the thread (the intro's exit to the file
            // select) still gives it only its phase before clearing $012A.
            let use_timed_worker = (self.rom_startup_timing || self.lag_free)
                && (rom_intro_poly_thread_is_active(frame.main_module, frame.submodule)
                    || dungeon_poly_thread
                    || triforce_room_poly_thread
                    || (self.lag_free && self.game_state.display.nmi_thread_active));
            let incremental_shadow = !self.lag_free
                && (dungeon_poly_thread || triforce_room_poly_thread)
                && std::env::var("ZELDA3_POLY_SHADOW_INCREMENTAL").map_or(true, |v| v != "0");
            if incremental_shadow {
                if live_timing && self.poly_job_in_flight {
                    assert!(
                        !source_started_preemptive_render,
                        "source started a preemptive poly render while the native ROM-CPU shadow was still in flight",
                    );
                }
                if live_timing && !self.poly_job_in_flight && !source_started_preemptive_render {
                    return;
                }
                if live_timing && !self.poly_job_in_flight {
                    self.poly_dungeon_thread_startup_hold = None;
                } else if dungeon_poly_thread && !self.poly_job_in_flight {
                    // The ROM's `CrystalCutscene_InitializePolyhedral` sets the go byte
                    // four source hosts after `Polyhedral_InitializeThread`
                    // (oracle 500942 → go-byte write late in 500946). The
                    // render begins only when the next NMI swaps to the thread
                    // (source 500947 `$09:F825`), which is debug host +6.
                    let hold = self.poly_dungeon_thread_startup_hold.get_or_insert(6);
                    let elapsed = self
                        .frame_ctr_dbg
                        .saturating_sub(self.poly_dungeon_activation_host);
                    if u32::from(*hold) > elapsed {
                        return;
                    }
                    *hold = 0;
                }
                if !self.poly_job_in_flight {
                    // The ROM thread reads the frame's config at its start;
                    // capture that state before the native rasterizer advances it.
                    self.begin_poly_shadow_frame();
                    self.poly_run_frame();
                    if dungeon_poly_thread {
                        self.poly_dungeon_frames_rendered =
                            self.poly_dungeon_frames_rendered.saturating_add(1);
                    }
                    self.poly_job_in_flight = true;
                    self.poly_job_hold_frames = 0;
                }
                if !self.advance_poly_shadow_host() {
                    return;
                }
                if crate::debug_env::var_os!("ZELDA3_DEBUG_POLY").is_some() {
                    eprintln!(
                        "[POLY-FRAME] host={} module={:02x} config1={} scanlines={} hosts={} master={}",
                        self.frame_ctr_dbg,
                        frame.main_module,
                        self.game_state.poly.runtime.config1(),
                        self.last_poly_work.scanlines,
                        self.poly_shadow_hosts,
                        self.poly_shadow_master,
                    );
                }
                self.poly_job_in_flight = false;
            } else if use_timed_worker {
                if dungeon_poly_thread && crate::debug_env::var_os!("ZELDA3_DEBUG_POLY").is_some() {
                    eprintln!(
                        "[POLY] host={} dungeon thread: startup_hold={:?} in_flight={} hold_frames={} rendered={} pending_update={}",
                        self.frame_ctr_dbg,
                        self.poly_dungeon_thread_startup_hold,
                        self.poly_job_in_flight,
                        self.poly_job_hold_frames,
                        self.poly_dungeon_frames_rendered,
                        self.game_state.display.has_pending_polyhedral_update(),
                    );
                }
                // Lag-free, the ROM's startup hosts are lag NMIs of the
                // activating iteration: the thread starts at once.
                if dungeon_poly_thread && !self.poly_job_in_flight && !self.lag_free {
                    // The maiden activates the thread inside Sprite_Main (route
                    // host 413550, counted by `activate_nmi_thread`); the IRQ
                    // thread's first slice, which writes the crystal config,
                    // runs four hosts later (413554). The activation host's
                    // own lane may bypass this loop, so the hold counts hosts
                    // from the activation itself.
                    let hold = self.poly_dungeon_thread_startup_hold.get_or_insert(4);
                    let elapsed = self
                        .frame_ctr_dbg
                        .saturating_sub(self.poly_dungeon_activation_host);
                    if u32::from(*hold) > elapsed {
                        return;
                    }
                    *hold = 0;
                }
                if !self.poly_job_in_flight {
                    // Measure the ROM thread's exact render cost before the
                    // native rasterizer mutates the frame state.
                    let dungeon_shadow_render = dungeon_poly_thread
                        && std::env::var("ZELDA3_POLY_DUNGEON_SHADOW").map_or(true, |v| v != "0");
                    let triforce_render_hosts =
                        if triforce_room_poly_thread || dungeon_shadow_render {
                            self.rom_triforce_poly_render_hosts()
                        } else {
                            None
                        };
                    self.poly_run_frame();
                    self.poly_job_hold_frames = if dungeon_poly_thread {
                        // The thread rasterizes 32 crystal scanlines per host
                        // after a fixed per-frame setup measured in scanline
                        // equivalents: 67 while Module07 runs the cutscene
                        // (every one of the 86 oracle frames at route hosts
                        // 413554-414023: 4 hosts at 38-60 scanlines, 5 at
                        // 64-92, 6 at 94-106, 7 at 130-138) and 56 through
                        // the maiden's Module0E dialogue, whose lighter main
                        // iteration leaves the thread about a third of a host
                        // more per frame (414033-416096: 5 hosts at ~100
                        // scanlines, 6 at ~134, 7 at 138).
                        self.poly_dungeon_frames_rendered =
                            self.poly_dungeon_frames_rendered.saturating_add(1);
                        let scanlines = self.last_poly_work.scanlines;
                        let slices = if let Some((hosts, master)) = triforce_render_hosts {
                            if crate::debug_env::var_os!("ZELDA3_DEBUG_POLY").is_some() {
                                eprintln!(
                                    "[POLY-DUNG] host={} module={:02x} config1={} shadow_master={} scanlines={} hosts={}",
                                    self.frame_ctr_dbg,
                                    frame.main_module,
                                    self.game_state.poly.runtime.config1(),
                                    master,
                                    scanlines,
                                    hosts,
                                );
                            }
                            hosts.clamp(1, 16)
                        } else if frame.main_module == 0x0e {
                            // Through the dialogue the oracle's frame lengths
                            // separate by the estimated cycle cost rather than
                            // by scanlines (137-scanline frames complete in 6
                            // hosts at 63.3k cycles, 138-scanline ones in 7 at
                            // 66.7k). `ZELDA3_POLY_DIALOGUE_BUDGET=B,S`
                            // overrides the calibration.
                            static DIALOGUE_BUDGET: std::sync::OnceLock<(i64, i64)> =
                                std::sync::OnceLock::new();
                            let (budget, setup) = *DIALOGUE_BUDGET.get_or_init(|| {
                                std::env::var("ZELDA3_POLY_DIALOGUE_BUDGET")
                                    .ok()
                                    .and_then(|value| {
                                        let (b, s) = value.split_once(',')?;
                                        Some((b.trim().parse().ok()?, s.trim().parse().ok()?))
                                    })
                                    .unwrap_or((
                                        DUNGEON_POLY_THREAD_DIALOGUE_BUDGET_CYCLES,
                                        DUNGEON_POLY_THREAD_DIALOGUE_SETUP_CYCLES,
                                    ))
                            });
                            let cost = (i64::from(self.last_poly_work.estimated_65816_cycles())
                                + setup)
                                .max(1);
                            ((cost + budget - 1) / budget).clamp(1, 16) as u8
                        } else {
                            (scanlines + DUNGEON_POLY_THREAD_SETUP_SCANLINES)
                                .div_ceil(DUNGEON_POLY_THREAD_SCANLINES_PER_HOST)
                                .clamp(1, 16) as u8
                        };
                        self.poly_dungeon_current_frame_slices = slices;
                        slices - 1
                    } else if triforce_room_poly_thread {
                        // The thread owns the scanlines from the NMI's return
                        // to the V-IRQ line each host; the exact render cost
                        // decides how many of those slots the frame needs.
                        // Calibrated on route hosts 1557810-1558053: the
                        // oracle's single-host renders cost at most 188,016
                        // master cycles in the shadow and its two-host ones
                        // at least 190,392 (about 139 scanlines of thread time
                        // between the NMI's return and the V-IRQ at line 144).
                        let hosts = match triforce_render_hosts {
                            Some((hosts, _)) => hosts.clamp(1, 16),
                            None => self
                                .last_poly_work
                                .estimated_65816_cycles()
                                .div_ceil(TRIFORCE_ROOM_POLY_THREAD_HOST_CYCLES)
                                .clamp(1, 16) as u8,
                        };
                        if crate::debug_env::var_os!("ZELDA3_DEBUG_POLY").is_some() {
                            eprintln!(
                                "[POLY-TRI] host={} config1={} shadow_master={:?} estimate={} hosts={}",
                                self.frame_ctr_dbg,
                                self.game_state.poly.runtime.config1(),
                                triforce_render_hosts.map(|(_, master)| master),
                                self.last_poly_work.estimated_65816_cycles(),
                                hosts,
                            );
                        }
                        hosts - 1
                    } else {
                        self.last_poly_work.worker_frames() - 1
                    };
                    self.poly_job_in_flight = true;
                    if crate::debug_env::var_os!("ZELDA3_DEBUG_POLY").is_some() {
                        eprintln!(
                            "[POLY] host={} module={:02x}/{:02x} cycles={} worker_frames={} config1={:#x} metrics={:?}",
                            self.frame_ctr_dbg,
                            frame.main_module,
                            frame.submodule,
                            self.last_poly_work.estimated_65816_cycles(),
                            self.last_poly_work.worker_frames(),
                            self.game_state.poly.runtime.config1(),
                            self.last_poly_work,
                        );
                    }
                }
                if self.poly_job_hold_frames != 0 {
                    self.poly_job_hold_frames -= 1;
                    return;
                }
                self.poly_job_in_flight = false;
            } else {
                self.poly_run_frame();
                if crate::debug_env::var_os!("ZELDA3_DEBUG_POLY").is_some() {
                    eprintln!(
                        "[POLY] host={} untimed frame: module={:02x}/{:02x} config1={} scanlines={} cycles={} worker_frames={} work={:?}",
                        self.frame_ctr_dbg,
                        frame.main_module,
                        frame.submodule,
                        self.game_state.poly.runtime.config1(),
                        self.last_poly_work.scanlines,
                        self.last_poly_work.estimated_65816_cycles(),
                        self.last_poly_work.worker_frames(),
                        self.last_poly_work,
                    );
                }
            }
            self.attract_scene_mut().clear_intro_did_run_step();
            self.request_polyhedral_nmi_update();
            if incremental_shadow {
                self.poly_completed_upload = Some((
                    self.frame_ctr_dbg,
                    self.ram[POLYHEDRAL_BUFFER..POLYHEDRAL_BUFFER + 0x800].to_vec(),
                ));
            }
        }
    }

    pub(super) fn clear_poly_thread_work_area(&mut self) {
        SystemWorkArea::clear_poly_thread_work_area(&mut self.ram);
        // The clear zeros the poly thread work area (0x1f00-0x1fff) directly in RAM, but the
        // PolyState native model (num_vertices at 0x1f3f, projected vertices, face coords, raster
        // edges) still holds the previous polyhedron's values. Without resyncing, a stale poly
        // field re-stamps RAM (e.g. num_vertices reverts the just-cleared 0x1f3f), leaving the
        // poly scratch a frame out of phase with the old clone (f465536).
        self.game_state.poly = crate::game_state::PolyState::load_from_ram(&self.ram);
    }

    /// Publish one initializer store or MVN byte. Adopt only the native
    /// owners sharing this work area; never project their old values over an
    /// interrupted clear or the newly copied thread stack.
    pub(super) fn write_native_intro_poly_state(&mut self, address: usize, value: u16, width: u8) {
        assert!(matches!(width,1|2));
        let end=address+usize::from(width);
        assert!((0x1e00..=0x1e70).contains(&address) && end<=0x1e70
            || (0x1f00..0x2000).contains(&address) && end<=0x2000);
        self.ram[address..end].copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
        if address<0x1f00 {
            if address<0x1e0a && end>0x1e08 {
                self.game_state.ending.intro_scene.sprite_oam_cursor=crate::types::read_le_u16(&self.ram,crate::game_state::constants::INTRO_SPRITE_ALLOC);
            }
            self.game_state.ending.intro_actors=crate::game_state::IntroActorState::load_from_ram(&self.ram);
            self.game_state.ending.attract_scene=crate::game_state::AttractSceneState::load_from_ram(&self.ram);
        } else {
            self.game_state.poly=crate::game_state::PolyState::load_from_ram(&self.ram);
            if address==0x1f00 {
                self.game_state.ending.attract_scene.intro_did_run_step=self.ram[crate::game_state::constants::INTRO_DID_RUN_STEP];
            }
            if address<0x1f0c && end>0x1f0a {
                self.set_nmi_thread_stack_pointer(crate::types::read_le_u16(&self.ram,crate::game_state::constants::POLY_THREAD_STACK));
            }
            if address<=0x1f0c && end>0x1f0c {
                assert_eq!(self.ram[crate::game_state::constants::NMI_FLAG_UPDATE_POLYHEDRAL],0,"initializer must clear the pending poly upload");
                self.clear_pending_polyhedral_update();
            }
        }
    }

    pub(super) fn write_poly_thread_init_bytes(&mut self) {
        SystemWorkArea::write_poly_thread_bootstrap_bytes(&mut self.ram);
    }
}
