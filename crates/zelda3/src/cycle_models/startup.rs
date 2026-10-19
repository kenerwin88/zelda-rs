//! Reset caller after the native SPC upload, through first NMI acceptance.
//! Host returns are hardware vblank events, not a calibrated frame delay.

use super::{main_wait::MainWait, startup_memory::StartupMemoryWork, native_work::WorkBus, startup_bus::{CallerBus, StartupAccess}};
use snes::{
    cpu_timeline::CpuSynchronousTimelineEvent, CpuMasterTimeline,
    SNES9X_NMI_ACCEPTANCE_DELAY_MASTER_CYCLES,
};

#[derive(Clone, Debug, PartialEq, Eq)]
enum Phase {
    CallMemory,
    Memory(StartupMemoryWork),
    LoadEnable,
    StoreEnable,
    Wait(MainWait),
    Accepted(MainWait),
    WaitHandler(MainWait),
    Complete,
    ModuleEntered,
    IntroHalfSlotEntered,
    IntroHalfSlotReturned,
    IntroSetupReturned,
    IntroFirstClearEntered,
    IntroClearActive,
    IntroClearAccepted,
    IntroClearReturned,
    IntroPackingReturned,
    IntroSourcesReturned,
    IntroTailReturned,
    IntroTextReturned,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cycle_models::nmi_stack::{NativeNmiContext, ZERO};
    use snes::{CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition,
        RomCpuTimingProbe, Snes, SourcePpuReadState};

    #[test]
    fn returned_wait_matches_rom_with_saved_branch_flags_and_live_latch() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc");
        let Ok(rom) = std::fs::read(path) else { return };
        for wait in [MainWait::LoadLatch, MainWait::TestLatch { value: 0 }, MainWait::TestLatch { value: 1 }] {
            for latch in [1, 0xff] {
                for (v, h) in [(252, 184), (252, 530), (261, 1350)] {
                    for odd in [false, true] {
                        let saved = match wait { MainWait::TestLatch { value } => value, _ => 0 };
                        let mut context = NativeNmiContext { a: 0x5a00 | u16::from(saved), x: 23, y: 47,
                            d: 0, db: 0, bank: 0, pc: wait.next_pc() as u16, sp: 0x1ff,
                            p: 0x30 | if saved == 0 { ZERO } else { 0 } };
                        let mut machine = Snes::new();
                        machine.cart.load(CartType::LoRom, &rom, 0x2000);
                        machine.ram[0x12] = latch;
                        machine.cpu.a = context.a; machine.cpu.x = context.x; machine.cpu.y = context.y;
                        machine.cpu.pc = context.pc; machine.cpu.sp = context.sp;
                        machine.cpu.e = false; machine.cpu.unpack_flags(context.p);
                        let ram = machine.ram.clone();
                        let mut clock = CpuMasterTimeline::at_raster(0, CpuRasterPosition::new(v, h),
                            CpuBusWorkload::default(), CpuFieldTiming::non_interlace(odd));
                        let mut source = RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset()).unwrap();
                        while source.snes().cpu.pc != 0x8038 { source.step().unwrap(); }
                        clock.begin_synchronous_timeline().unwrap();
                        let mut startup = Startup::after_upload(&clock);
                        startup.phase = Phase::Accepted(wait);
                        startup.context = Some(context);
                        let cycles = startup.return_from_nmi(&mut clock, &mut context, &ram, |_, _| 0);
                        assert_eq!(cycles, match wait { MainWait::LoadLatch => 40,
                            MainWait::TestLatch { value: 0 } => 62, _ => 16 });
                        let cpu = &source.snes().cpu;
                        assert_eq!((context.a, context.x, context.y, context.sp, context.pc, context.p),
                            (cpu.a, cpu.x, cpu.y, cpu.sp, cpu.pc, cpu.pack_flags()));
                        assert_eq!(clock, *source.timeline());
                        assert_eq!(ram, source.snes().ram);
                        assert_eq!(startup.main_caller_context(), Some(context));
                        startup.complete_externally();
                        assert!(startup.main_caller_context().is_none());
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StartupStep {
    Instruction,
    ClearWindow,
    MemoryComplete,
    HostReturn,
    AcceptNmi,
    AcceptIrq(super::nmi_stack::NativeNmiContext,u64),
    RunNmi,
    MainReady,
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum SelectedWorkerStep {
    Instruction(super::nmi_stack::NativeNmiContext),
    AcceptNmi(super::nmi_stack::NativeNmiContext),
    RunNmi(super::nmi_stack::NativeNmiContext),
    HostReturn,
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
struct SelectedWorker {
    context:super::nmi_stack::NativeNmiContext,
    nmi_accepted:bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum IntroClearBoundary {
    HostReturn,
    AcceptNmi(super::nmi_stack::NativeNmiContext),
    AcceptIrq(super::nmi_stack::NativeNmiContext,u64),
    Returned,
    Packed,
    Sources,
    Tail,
    TextReturned,
    PaletteCleared,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum IntroGraphics {
    Overworld(super::overworld_module::OverworldModule),
    Properties(super::overworld_properties_caller::OverworldPropertiesCaller),
    Overlays(super::overworld_overlays_caller::OverworldOverlaysCaller),
    ScreenBuild(super::overworld_screen_build::OverworldScreenBuild),
    Closing(super::spotlight_caller_return::ClosingSpotlight),
    Opening(super::spotlight_caller_return::OpeningSpotlight),
    Dungeon(super::dungeon_module::DungeonModule),
    LoadFile(super::load_file::LoadFile),
    Items(super::item_graphics_caller::ItemGraphics),
    Follower(super::follower_graphics::FollowerGraphics),
    Darkening(super::intro_darkening::IntroDarkening),
    Poly(super::intro_poly_init::IntroPolyInit),
    Animation(super::intro_animation::IntroAnimation),
    Title(super::intro_title::IntroTitle),
    FileSelect(super::file_select_init::FileSelectInit),
    FileMenu(super::file_select_erase::FileSelectErase),
    FileStripes(super::file_select_stripes::FileSelectStripes),
    FileActive(super::file_select_main::FileSelectMain),
    KillFile(super::kill_file::KillFile),
    NameSetup(super::name_file_setup::NameFileSetup),
    NameActive(super::name_file_active::NameFileActive),
}
impl IntroGraphics {
    fn complete(&self) -> bool {
        match self { Self::Overworld(work) => work.complete(), Self::Opening(work) => work.complete(), Self::ScreenBuild(work) => work.complete(), Self::Overlays(work) => work.complete(), Self::Properties(work) => work.complete(), Self::Closing(work) => work.complete(), Self::Dungeon(work) => work.complete(), Self::LoadFile(work) => work.complete(), Self::Items(work) => work.complete(), Self::Follower(work) => work.complete(), Self::Darkening(work)=>work.complete(), Self::Poly(work)=>work.complete(), Self::Animation(work)=>work.complete(), Self::Title(work)=>work.complete(), Self::FileSelect(work)=>work.complete(), Self::FileMenu(work)=>work.complete(), Self::FileStripes(work)=>work.complete(), Self::FileActive(work)=>work.complete(),Self::KillFile(work)=>work.complete(),Self::NameSetup(work)=>work.complete(),Self::NameActive(work)=>work.complete() }
    }
    fn routine(&self) -> u32 {
        match self { Self::Overworld(work) => work.routine(), Self::Opening(_) => 0x029ad7, Self::ScreenBuild(_) => 0x0283bf, Self::Overlays(_) => 0x0283bf, Self::Properties(_) => 0x0283bf, Self::Closing(_) => 0x029982, Self::Dungeon(_) => 0x0287a2, Self::LoadFile(work) => work.routine(), Self::Items(work) => work.routine(), Self::Follower(work) => work.routine(), Self::Darkening(work)=>work.routine(), Self::Poly(work)=>work.routine(), Self::Animation(work)=>work.routine(), Self::Title(work)=>work.routine(), Self::FileSelect(work)=>work.routine(), Self::FileMenu(work)=>work.routine(), Self::FileStripes(work)=>work.routine(), Self::FileActive(work)=>work.routine(),Self::KillFile(work)=>work.routine(),Self::NameSetup(work)=>work.routine(),Self::NameActive(work)=>work.routine() }
    }
    fn step(&mut self, cpu: &mut super::nmi_stack::NativeNmiContext, clock: &mut CpuMasterTimeline, dma: &mut snes::DmaState,
        bus: impl WorkBus) -> Option<u64> {
        if let Self::ScreenBuild(work) = self {
            assert!(!work.event_overlay_pending(), "native screen build event overlay requires its child owner");
            return work.step(cpu, clock, super::native_hdma::HdmaBus { dma, bus });
        }
        if let Self::Overlays(work) = self {
            return work.step(cpu, clock, super::native_hdma::HdmaBus { dma, bus });
        }
        if let Self::Properties(work) = self {
            return work.step(cpu, clock, super::native_hdma::HdmaBus { dma, bus });
        }
        if let Self::LoadFile(work) = self { return work.step(cpu, clock, dma, bus); }
        if let Self::Overworld(work) = self {
            return work.step(cpu, clock, super::native_hdma::HdmaBus { dma, bus });
        }
        if let Self::Dungeon(work) = self {
            return work.step(cpu, clock, super::native_hdma::HdmaBus { dma, bus });
        }
        if let Self::Opening(work) = self {
            return work.step(cpu, clock, super::native_hdma::HdmaBus { dma, bus }).map(|(cycles, _)| cycles);
        }
        if let Self::Closing(work) = self {
            return work.step(cpu, clock, super::native_hdma::HdmaBus { dma, bus }).map(|(cycles, _)| cycles);
        }
        match self { Self::Overworld(_) => unreachable!(), Self::Opening(_) => unreachable!(), Self::ScreenBuild(_) => unreachable!(), Self::Overlays(_) => unreachable!(), Self::Properties(_) => unreachable!(), Self::Closing(work) => work.step(cpu, clock, bus).map(|(cycles, _)| cycles), Self::Dungeon(work) => work.step(cpu, clock, bus), Self::LoadFile(_) => unreachable!(), Self::Items(work) => work.step(cpu, clock, bus), Self::Follower(work) => work.step(cpu, clock, bus),Self::Darkening(work)=>work.step(cpu,clock,dma,bus),Self::Poly(work)=>work.step(cpu,clock,bus),Self::Animation(work)=>work.step(cpu,clock,bus),Self::Title(work)=>work.step(cpu,clock,bus),Self::FileSelect(work)=>work.step(cpu,clock,bus),Self::FileMenu(work)=>work.step(cpu,clock,dma,bus),Self::FileStripes(work)=>work.step(cpu,clock,bus),Self::FileActive(work)=>work.step(cpu,clock,bus),Self::KillFile(work)=>work.step(cpu,clock,bus),Self::NameSetup(work)=>work.step(cpu,clock,dma,bus),Self::NameActive(work)=>work.step(cpu,clock,bus) }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Startup {
    phase: Phase,
    next_host: u64,
    host_return_pending: bool,
    nmi_enabled: bool,
    nmi_deadline: Option<u64>,
    vertical_irq: Option<super::poly_interrupt::PolyIrqProgramming>,
    selected_worker: Option<SelectedWorker>,
    context: Option<super::nmi_stack::NativeNmiContext>,
    intro_clear: super::intro_clear::IntroClear,
    intro_text: Option<super::text_pointers::TextPointers>,
    intro_text_streams: Option<[Vec<u8>;2]>,
    intro_graphics: Option<IntroGraphics>,
    intro_packing: super::oam_pack::CallerPacking,
    intro_sources: super::sprite_dma::SourceWords,
    intro_source_master_cycles: u64,
    intro_tail: super::sprite_dma::MainSpriteTail,
    interrupted_clear: Option<super::nmi_stack::NativeNmiContext>,
    interrupted_wait: Option<super::nmi_stack::NativeNmiContext>,
}

impl Startup {
    pub(crate) fn from_retained_main_wait(context: super::nmi_stack::NativeNmiContext,
        clock: &CpuMasterTimeline, host: u64) -> Self {
        assert_eq!((context.bank, context.sp, context.db, context.d, context.p & 0x38),
            (0, 0x1ff, 0, 0, 0x30));
        let wait = match context.pc {
            0x8034 => MainWait::LoadLatch,
            0x8036 => MainWait::TestLatch { value: context.a as u8 },
            _ => panic!("retained caller is outside main wait"),
        };
        let mut startup = Self::after_upload(clock);
        startup.phase = Phase::Wait(wait);
        startup.context = Some(context);
        startup.next_host = host;
        startup.nmi_enabled = true;
        // This producer stopped at the actual NMI acceptance boundary.
        // Keep that boundary on its existing timeline; do not re-seed H=12.
        startup.nmi_deadline = Some(clock.clock_master_cycles());
        startup
    }
    #[cfg(test)]
    pub(crate) fn from_main_caller_for_test(context: super::nmi_stack::NativeNmiContext, clock: &CpuMasterTimeline) -> Self {
        assert_eq!((context.bank, context.pc, context.sp), (0, 0x8038, 0x1ff));
        let mut startup = Self::after_upload(clock);
        startup.phase = Phase::Complete;
        startup.context = Some(context);
        startup.nmi_enabled = true;
        startup
    }
    pub(crate) fn after_upload(clock: &CpuMasterTimeline) -> Self {
        Self {
            phase: Phase::CallMemory,
            next_host: clock.field_index() + u64::from(clock.raster_position().scanline() >= 225),
            host_return_pending: false,
            nmi_enabled: false,
            nmi_deadline: None,
            vertical_irq: None,
            selected_worker: None,
            context: None,
            intro_clear: super::intro_clear::IntroClear::default(),
            intro_text: None,
            intro_text_streams: None,
            intro_graphics: None,
            intro_packing: Default::default(),
            intro_sources: Default::default(),
            intro_source_master_cycles: 0,
            intro_tail: Default::default(),
            interrupted_clear: None,
            interrupted_wait: None,
        }
    }

    pub(crate) fn pending(&self) -> bool {
        !matches!(self.phase, Phase::Complete | Phase::ModuleEntered | Phase::IntroHalfSlotEntered | Phase::IntroHalfSlotReturned | Phase::IntroSetupReturned | Phase::IntroFirstClearEntered | Phase::IntroClearActive | Phase::IntroClearAccepted | Phase::IntroClearReturned | Phase::IntroPackingReturned | Phase::IntroSourcesReturned | Phase::IntroTailReturned | Phase::IntroTextReturned)
    }
    pub(crate) fn ready_in_host(&self, host: u64) -> bool {
        host >= self.next_host
    }
    pub(crate) fn owns_cpu_clock(&self) -> bool {
        self.phase != Phase::Complete || self.context.is_some()
    }
    pub(crate) fn begins_memory_call(&self) -> bool {
        self.phase == Phase::CallMemory
    }
    pub(crate) fn memory_initialization_pending(&self) -> bool {
        matches!(self.phase, Phase::CallMemory | Phase::Memory(_))
    }
    pub(crate) fn complete_externally(&mut self) {
        self.intro_graphics=None;
        self.intro_text=None;
        self.intro_text_streams=None;
        self.phase = Phase::Complete;
        self.context = None;
    }

    pub(crate) fn main_caller_context(&self) -> Option<super::nmi_stack::NativeNmiContext> {
        if self.phase == Phase::Complete { self.context } else { None }
    }

    pub(crate) fn enter_module(&mut self, context: super::nmi_stack::NativeNmiContext) {
        assert!(self.main_caller_context().is_some(), "native main prefix was already consumed");
        self.phase = Phase::ModuleEntered;
        self.context = Some(context);
    }

    #[cfg(test)]
    pub(crate) fn retained_context(&self)->Option<super::nmi_stack::NativeNmiContext> {self.selected_worker.map(|w|w.context).or(self.context)}

    pub(crate) fn entered_module_context(&self) -> Option<super::nmi_stack::NativeNmiContext> {
        if self.phase == Phase::ModuleEntered { self.context } else { None }
    }

    pub(crate) fn enter_intro_half_slot(&mut self, context: super::nmi_stack::NativeNmiContext) {
        assert!(self.entered_module_context().is_some(), "native intro setup was already consumed");
        self.phase = Phase::IntroHalfSlotEntered;
        self.context = Some(context);
    }

    pub(crate) fn intro_half_slot_context(&self) -> Option<super::nmi_stack::NativeNmiContext> {
        if self.phase == Phase::IntroHalfSlotEntered { self.context } else { None }
    }

    pub(crate) fn return_intro_half_slot(&mut self, context: super::nmi_stack::NativeNmiContext) {
        assert!(self.intro_half_slot_context().is_some(), "native half-slot was already consumed");
        self.phase = Phase::IntroHalfSlotReturned;
        self.context = Some(context);
    }

    pub(crate) fn intro_setup_after_half_slot_context(&self) -> Option<super::nmi_stack::NativeNmiContext> {
        if self.phase == Phase::IntroHalfSlotReturned { self.context } else { None }
    }

    pub(crate) fn return_intro_setup(&mut self, context: super::nmi_stack::NativeNmiContext) {
        assert!(self.intro_setup_after_half_slot_context().is_some(), "native setup suffix was already consumed");
        self.phase = Phase::IntroSetupReturned;
        self.context = Some(context);
    }

    pub(crate) fn intro_after_setup_context(&self) -> Option<super::nmi_stack::NativeNmiContext> {
        if self.phase == Phase::IntroSetupReturned { self.context } else { None }
    }

    pub(crate) fn enter_intro_clear(&mut self, context: super::nmi_stack::NativeNmiContext) {
        self.intro_graphics=None;
        assert!(self.intro_after_setup_context().is_some() || self.entered_module_context().is_some(),
            "native intro caller was already consumed");
        assert_eq!((context.bank, context.pc), (12, 0xc1a0));
        self.intro_text=None;
        self.intro_text_streams=None;
        self.intro_clear = Default::default();
        self.intro_packing = Default::default();
        self.intro_sources = Default::default();
        self.intro_source_master_cycles = 0;
        self.intro_tail = Default::default();
        assert!(self.interrupted_clear.is_none());
        self.phase = Phase::IntroFirstClearEntered;
        self.context = Some(context);
    }

    pub(crate) fn enter_intro_text(&mut self, context: super::nmi_stack::NativeNmiContext, streams: [Vec<u8>;2]) {
        self.intro_graphics=None;
        assert!(self.entered_module_context().is_some());
        assert_eq!((context.bank,context.pc),(2,0x8116));
        self.intro_text=Some(Default::default());
        self.intro_text_streams=Some(streams);
        // The prior WRAM-clear iteration already consumed these continuations.
        // This module call owns a fresh shared suffix after its palette RTL.
        self.intro_packing=Default::default();
        self.intro_sources=Default::default();
        self.intro_source_master_cycles=0;
        self.intro_tail=Default::default();
        self.context=Some(context);
        self.phase=Phase::IntroClearActive;
        self.host_return_pending=false;
        self.nmi_deadline=None;
    }
    pub(crate) fn begin_intro_palette_clear(&mut self) {
        assert_eq!(self.phase,Phase::IntroTextReturned);
        self.intro_text.as_mut().unwrap().begin_palette_clear();
        self.phase=Phase::IntroClearActive;
    }
    pub(crate) fn enter_intro_items(&mut self,context:super::nmi_stack::NativeNmiContext) {
        assert_eq!((context.bank,context.pc),(0,0xd231));
        self.enter_intro_graphics(context,IntroGraphics::Items(Default::default()));
    }
    pub(crate) fn enter_intro_follower(&mut self,context:super::nmi_stack::NativeNmiContext) {
        assert_eq!((context.bank,context.pc),(0,0xd423));
        self.enter_intro_graphics(context,IntroGraphics::Follower(Default::default()));
    }
    pub(crate) fn enter_intro_darkening(&mut self,context:super::nmi_stack::NativeNmiContext) {
        assert_eq!((context.bank,context.pc),(12,0xc1f5));
        self.enter_intro_graphics(context,IntroGraphics::Darkening(Default::default()));
    }
    pub(crate) fn enter_intro_poly(&mut self,context:super::nmi_stack::NativeNmiContext) {
        assert_eq!((context.bank,context.pc),(12,0xc33c));
        self.enter_intro_graphics(context,IntroGraphics::Poly(Default::default()));
    }
    pub(crate) fn enter_intro_animation(&mut self,context:super::nmi_stack::NativeNmiContext) {
        assert_eq!((context.bank,context.pc),(12,0xc404));
        self.enter_intro_graphics(context,IntroGraphics::Animation(Default::default()));
    }
    pub(crate) fn enter_intro_title(&mut self,context:super::nmi_stack::NativeNmiContext) {
        assert_eq!(context.bank,12);
        self.enter_intro_graphics(context,IntroGraphics::Title(super::intro_title::IntroTitle::new(context.pc)));
    }
    pub(crate) fn enter_opening(&mut self,context:super::nmi_stack::NativeNmiContext) {
        assert_eq!((context.bank,context.pc,context.sp),(2,0x9ad7,0x1fc));
        self.enter_intro_graphics(context,IntroGraphics::Opening(super::spotlight_caller_return::OpeningSpotlight::from_module_entry()));
    }
    pub(crate) fn opening_active(&self)->bool {
        matches!(self.intro_graphics,Some(IntroGraphics::Opening(_)))
    }
    pub(crate) fn enter_screen_build(&mut self,context:super::nmi_stack::NativeNmiContext) {
        assert_eq!((context.bank,context.pc,context.sp),(2,0x83bf,0x1fc));
        self.enter_intro_graphics(context,IntroGraphics::ScreenBuild(Default::default()));
    }
    pub(crate) fn screen_build_active(&self)->bool {
        matches!(self.intro_graphics,Some(IntroGraphics::ScreenBuild(_)))
    }
    pub(crate) fn enter_overlays(&mut self, context: super::nmi_stack::NativeNmiContext) {
        assert_eq!((context.bank, context.pc, context.sp), (2, 0x83bf, 0x1fc));
        self.enter_intro_graphics(context, IntroGraphics::Overlays(Default::default()));
    }
    pub(crate) fn overlays_active(&self) -> bool {
        matches!(self.intro_graphics, Some(IntroGraphics::Overlays(_)))
    }
    pub(crate) fn enter_properties(&mut self, context: super::nmi_stack::NativeNmiContext) {
        assert_eq!((context.bank, context.pc, context.sp), (2, 0x83bf, 0x1fc));
        self.enter_intro_graphics(context, IntroGraphics::Properties(Default::default()));
    }
    pub(crate) fn properties_active(&self) -> bool {
        matches!(self.intro_graphics, Some(IntroGraphics::Properties(_)))
    }
    pub(crate) fn enter_load_file(&mut self, context: super::nmi_stack::NativeNmiContext) {
        assert_eq!((context.bank, context.pc), (2, 0x8136));
        self.enter_intro_graphics(context, IntroGraphics::LoadFile(Default::default()));
    }
    pub(crate) fn load_file_active(&self) -> bool {
        matches!(self.intro_graphics, Some(IntroGraphics::LoadFile(_)))
    }
    pub(crate) fn enter_overworld(&mut self, context: super::nmi_stack::NativeNmiContext) {
        assert_eq!((context.bank, context.pc, context.sp), (2, 0xa475, 0x1fc));
        self.enter_intro_graphics(context, IntroGraphics::Overworld(Default::default()));
    }
    pub(crate) fn overworld_active(&self) -> bool {
        matches!(self.intro_graphics, Some(IntroGraphics::Overworld(_)))
    }
    pub(crate) fn enter_dungeon(&mut self, context: super::nmi_stack::NativeNmiContext) {
        assert_eq!((context.bank, context.pc), (2, 0x87a2));
        self.enter_intro_graphics(context, IntroGraphics::Dungeon(Default::default()));
    }
    pub(crate) fn dungeon_active(&self) -> bool {
        matches!(self.intro_graphics, Some(IntroGraphics::Dungeon(_)))
    }
    pub(crate) fn enter_closing(&mut self, context: super::nmi_stack::NativeNmiContext) {
        assert_eq!((context.bank, context.pc, context.sp), (2, 0x9982, 0x1fc));
        self.enter_intro_graphics(context, IntroGraphics::Closing(
            super::spotlight_caller_return::ClosingSpotlight::from_module_entry()));
    }
    pub(crate) fn closing_active(&self) -> bool {
        matches!(self.intro_graphics, Some(IntroGraphics::Closing(_)))
    }
    pub(crate) fn closing_entry_graphics_field(&self) -> Option<u64> {
        match &self.intro_graphics {
            Some(IntroGraphics::Closing(work)) => work.entry_graphics_field(),
            _ => None,
        }
    }
    pub(crate) fn intro_title_active(&self)->bool {matches!(self.intro_graphics,Some(IntroGraphics::Title(_)))}
    pub(crate) fn file_menu_active(&self)->bool {matches!(self.intro_graphics,Some(IntroGraphics::FileActive(_)|IntroGraphics::KillFile(_)))}
    pub(crate) fn name_file_active(&self)->bool {matches!(self.intro_graphics,Some(IntroGraphics::NameSetup(_)|IntroGraphics::NameActive(_)))}
    pub(crate) fn enter_file_select(&mut self,context:super::nmi_stack::NativeNmiContext) {
        assert_eq!(context.bank,12);
        let work=match context.pc {
            0xcda1=>IntroGraphics::FileSelect(Default::default()),
            0xcdf2=>IntroGraphics::FileMenu(Default::default()),
            0xcdf9=>IntroGraphics::FileMenu(super::file_select_erase::FileSelectErase::erase_triforce()),
            0xcebd=>IntroGraphics::FileActive(Default::default()),
            0xd49a=>IntroGraphics::FileActive(super::file_select_main::FileSelectMain::kill_file_setup()),
            0xd49f|0xd4b1=>IntroGraphics::KillFile(super::kill_file::KillFile::new(context.pc)),
            0xda4d=>IntroGraphics::NameActive(Default::default()),
            0xd89c|0xd911|0xd928=>IntroGraphics::NameSetup(super::name_file_setup::NameFileSetup::new(context.pc)),
            0xce53|0xcea5|0xceb1=>IntroGraphics::FileStripes(super::file_select_stripes::FileSelectStripes::new(context.pc)),
            _=>unreachable!(),
        };
        self.enter_intro_graphics(context,work);
    }
    fn enter_intro_graphics(&mut self,context:super::nmi_stack::NativeNmiContext,work:IntroGraphics) {
        assert!(self.entered_module_context().is_some());
        self.intro_graphics=Some(work);
        self.intro_text=None;self.intro_text_streams=None;
        self.intro_packing=Default::default();self.intro_sources=Default::default();
        self.intro_source_master_cycles=0;self.intro_tail=Default::default();
        self.context=Some(context);self.phase=Phase::IntroClearActive;
        self.host_return_pending=false;self.nmi_deadline=None;
    }
    pub(crate) fn intro_graphics_active(&self)->bool {self.intro_graphics.is_some()}
    pub(crate) fn begin_intro_palette_loads(&mut self) {
        assert_eq!(self.phase,Phase::IntroTextReturned);
        self.intro_text.as_mut().unwrap().begin_palette_loads();
        self.phase=Phase::IntroClearActive;
    }
    pub(crate) fn intro_text_streams(&self)->Option<&[Vec<u8>;2]> {self.intro_text_streams.as_ref()}
    pub(crate) fn intro_text_returned_context(&self)->Option<super::nmi_stack::NativeNmiContext> {
        if self.phase==Phase::IntroTextReturned {self.context} else {None}
    }

    pub(crate) fn intro_first_clear_context(&self) -> Option<super::nmi_stack::NativeNmiContext> {
        if self.phase == Phase::IntroFirstClearEntered { self.context } else { None }
    }

    pub(crate) fn begin_intro_clear(&mut self) {
        assert!(self.intro_first_clear_context().is_some());
        self.phase = Phase::IntroClearActive;
        self.host_return_pending = false;
        self.nmi_deadline = None;
    }

    pub(crate) fn intro_clear_pending(&self) -> bool {
        matches!(self.phase, Phase::IntroClearActive | Phase::IntroClearAccepted)
    }

    #[cfg(test)]
    pub(crate) fn intro_clear_context(&self) -> Option<super::nmi_stack::NativeNmiContext> {
        if self.intro_clear_pending() { self.context } else { None }
    }

    pub(crate) fn intro_clear_returned_context(&self) -> Option<super::nmi_stack::NativeNmiContext> {
        if self.phase == Phase::IntroClearReturned { self.context } else { None }
    }

    pub(crate) fn begin_intro_packing(&mut self) {
        assert_eq!(self.phase, Phase::IntroClearReturned);
        self.phase = Phase::IntroClearActive;
    }

    pub(crate) fn intro_sprite_prefix_master_cycles(&self) -> u64 {
        if let Some(IntroGraphics::Closing(work)) = &self.intro_graphics {
            return work.sprite_preparation_master_cycles();
        }
        if let Some(IntroGraphics::Opening(work)) = &self.intro_graphics {
            return work.sprite_preparation_master_cycles();
        }
        self.intro_packing.body_master_cycles() + self.intro_source_master_cycles + self.intro_tail.body_master_cycles()
    }

    pub(crate) fn begin_intro_sources(&mut self) {
        assert_eq!(self.phase, Phase::IntroPackingReturned);
        self.phase = Phase::IntroClearActive;
    }

    pub(crate) fn intro_sources_returned_context(&self) -> Option<super::nmi_stack::NativeNmiContext> {
        if self.phase == Phase::IntroSourcesReturned { self.context } else { None }
    }

    pub(crate) fn begin_intro_tail(&mut self) {
        assert_eq!(self.phase,Phase::IntroSourcesReturned);
        self.phase=Phase::IntroClearActive;
    }

    pub(crate) fn intro_tail_returned_context(&self)->Option<super::nmi_stack::NativeNmiContext> {
        if self.phase==Phase::IntroTailReturned {self.context} else {None}
    }

    pub(crate) fn resume_main_wait_after_sprite_tail(&mut self) {
        assert_eq!(self.phase, Phase::IntroTailReturned);
        let context = self.context.unwrap();
        assert_eq!((context.bank, context.pc), (0, 0x8034));
        self.phase = Phase::Wait(MainWait::default());
    }

    fn note_wait_vblank(&mut self, event: Option<(u64, u64)>) {
        if let Some((field, at)) = event {
            self.host_return_pending = true;
            self.next_host = field + 1;
            if self.nmi_enabled {
                self.nmi_deadline = Some(at + u64::from(SNES9X_NMI_ACCEPTANCE_DELAY_MASTER_CYCLES));
            }
        }
    }

    /// NMITIMEN and VTIMEL/H remain armed after a thread's software flag clears.
    /// TIMEUP acknowledgement advances the hardware compare to the next field.
    pub(crate) fn retain_vertical_irq(&mut self,mut programming:super::poly_interrupt::PolyIrqProgramming,clock:&CpuMasterTimeline) {
        assert_eq!(programming.nmitimen&0x30,0x20);
        programming.irq_deadline=snes::cpu_timeline::vertical_irq_deadline(clock,programming.vertical_timer,false).map(|at|at.master_cycles());
        self.vertical_irq=Some(programming);
    }
    pub(crate) fn irq_deadline(&self)->Option<u64> {
        self.vertical_irq.and_then(|p|p.irq_deadline)
    }
    pub(crate) fn install_vertical_irq(&mut self,programming:super::poly_interrupt::PolyIrqProgramming) {
        assert_eq!(programming.nmitimen&0x30,0x20);
        self.vertical_irq=Some(programming);
    }
    fn due_irq(&self,clock:&CpuMasterTimeline)->Option<u64> {
        let deadline=self.vertical_irq?.irq_deadline?;
        (clock.clock_master_cycles()>=deadline && self.context?.p&4==0).then_some(deadline)
    }
    pub(crate) fn return_from_main_irq(&mut self,context:super::nmi_stack::NativeNmiContext,clock:&CpuMasterTimeline,nmitimen:Option<u8>) {
        if Some(context)!=self.context {
            // The software flag can reactivate the worker between NMI and
            // V-IRQ. Follow the thread selected by RTI's actual stack frame.
            assert_eq!((context.bank,context.d),(9,0x1f00),"IRQ selected an unowned CPU context");
            assert!(self.selected_worker.is_none());
            self.selected_worker=Some(SelectedWorker{context,nmi_accepted:false});
        }
        if let Some(value)=nmitimen {
            assert_eq!(value,0x81,"ordinary IRQ disables the V timer");
            let programming=self.vertical_irq.as_mut().expect("IRQ needs live hardware programming");
            programming.nmitimen=value;programming.irq_deadline=None;
        } else {
            self.retain_vertical_irq(self.vertical_irq.expect("IRQ needs live hardware programming"),clock);
        }
    }
    pub(crate) fn worker_selected(&self)->bool {self.selected_worker.is_some()}
    pub(crate) fn selected_worker_step(&mut self,clock:&CpuMasterTimeline)->SelectedWorkerStep {
        let worker=self.selected_worker.expect("selected worker context");
        if self.nmi_deadline.is_some_and(|at|clock.clock_master_cycles()>=at) {
            assert!(!worker.nmi_accepted);
            return SelectedWorkerStep::AcceptNmi(worker.context);
        }
        if std::mem::take(&mut self.host_return_pending) {return SelectedWorkerStep::HostReturn;}
        if worker.nmi_accepted {SelectedWorkerStep::RunNmi(worker.context)}
        else {SelectedWorkerStep::Instruction(worker.context)}
    }
    pub(crate) fn retire_selected_worker(&mut self,context:super::nmi_stack::NativeNmiContext,vblank:Option<(u64,u64)>) {
        let worker=self.selected_worker.as_mut().unwrap();assert!(!worker.nmi_accepted);
        worker.context=context;self.note_wait_vblank(vblank);
    }
    pub(crate) fn accept_selected_worker_nmi(&mut self,context:super::nmi_stack::NativeNmiContext) {
        let worker=self.selected_worker.as_mut().unwrap();assert!(!worker.nmi_accepted);
        worker.context=context;worker.nmi_accepted=true;self.nmi_deadline=None;
    }
    pub(crate) fn return_selected_worker_nmi(&mut self,context:super::nmi_stack::NativeNmiContext,programming:super::poly_interrupt::PolyIrqProgramming) {
        assert!(self.selected_worker.unwrap().nmi_accepted);
        assert_eq!(Some(context),self.context,"worker NMI did not restore the suspended main continuation");
        self.selected_worker=None;
        // This timer has just been programmed, not acknowledged. Its current
        // deadline must remain eligible even when it selects the other thread.
        self.vertical_irq=Some(programming);
    }

    pub(crate) fn intro_packing_returned_context(&self) -> Option<super::nmi_stack::NativeNmiContext> {
        if self.phase == Phase::IntroPackingReturned { self.context } else { None }
    }

    pub(crate) fn intro_clear_accepted_context(&self) -> Option<super::nmi_stack::NativeNmiContext> {
        if self.phase == Phase::IntroClearAccepted { self.context } else { None }
    }

    pub(crate) fn accept_intro_clear_interrupt(&mut self, context: super::nmi_stack::NativeNmiContext) {
        assert_eq!(self.phase, Phase::IntroClearActive);
        self.interrupted_clear = self.context;
        self.context = Some(context);
        self.phase = Phase::IntroClearAccepted;
        self.nmi_deadline = None;
    }

    pub(crate) fn resume_intro_clear(&mut self, context: super::nmi_stack::NativeNmiContext) {
        assert_eq!(self.phase, Phase::IntroClearAccepted);
        assert_eq!(Some(context), self.interrupted_clear.take(), "NMI changed its interrupted clear context");
        self.context = Some(context);
        self.phase = Phase::IntroClearActive;
        self.host_return_pending = false;
    }

    pub(crate) fn advance_intro_clear(&mut self, clock: &mut CpuMasterTimeline, dma: &mut snes::DmaState,
        ppu: &mut Option<super::native_ppu::NativePpuState>,
        mut bus: impl FnMut(u64, StartupAccess) -> u16) -> IntroClearBoundary {
        assert_eq!(self.phase, Phase::IntroClearActive);
        loop {
            if self.nmi_deadline.is_some_and(|at| clock.clock_master_cycles() >= at) {
                return IntroClearBoundary::AcceptNmi(self.context.unwrap());
            }
            if let Some(deadline)=self.due_irq(clock) {return IntroClearBoundary::AcceptIrq(self.context.unwrap(),deadline);}
            if std::mem::take(&mut self.host_return_pending) { return IntroClearBoundary::HostReturn; }
            let mut vblank = None;
            if let Some(graphics)=self.intro_graphics.as_mut().filter(|work|!work.complete()) {
                if matches!(graphics, IntroGraphics::Closing(_)|IntroGraphics::Opening(_))
                    && (self.context.unwrap().bank, self.context.unwrap().pc) == (0, 0x805a) {
                    bus(clock.clock_master_cycles(), StartupAccess::EnterSpritePreparation);
                }
                let routine=graphics.routine();
                let palette_source=if matches!(graphics,IntroGraphics::Poly(_)) && routine==0x0cc3bd {
                    super::intro_palettes::PaletteSource::Asset
                } else if matches!(graphics,IntroGraphics::Title(work) if work.restoring_palette()) {
                    super::intro_palettes::PaletteSource::RestoreAdditive
                } else if let IntroGraphics::LoadFile(work)=graphics {
                    work.palette_source(self.context.as_ref().unwrap())
                } else if let IntroGraphics::FileSelect(work)=graphics {
                    work.palette_source(self.context.as_ref().unwrap())
                } else if let IntroGraphics::NameSetup(work)=graphics {
                    work.palette_source()
                } else if let IntroGraphics::FileMenu(work)=graphics {
                    work.palette_source()
                } else {super::intro_palettes::PaletteSource::Constant};
                let mut nmitimen = None;
                let mut hdma = None;
                let owner = match graphics {
                    IntroGraphics::ScreenBuild(_) => super::startup_bus::CallerOwner::ScreenBuild,
                    IntroGraphics::Overlays(_) => super::startup_bus::CallerOwner::Overlays,
                    IntroGraphics::Properties(work) => super::startup_bus::CallerOwner::Properties(work.active_child()),
                    IntroGraphics::LoadFile(work) if work.opening_player_active() => super::startup_bus::CallerOwner::OpeningPlayer,
                    IntroGraphics::LoadFile(work) if work.sprite_records_active() => super::startup_bus::CallerOwner::SpriteRecords,
                    IntroGraphics::LoadFile(work) if work.sprite_reset_parent_active() => super::startup_bus::CallerOwner::SpriteResetParent,
                    IntroGraphics::LoadFile(work) if work.sprite_cache_active() => super::startup_bus::CallerOwner::SpriteCache,
                    IntroGraphics::LoadFile(work) if work.sprite_reset_suffix_active() => super::startup_bus::CallerOwner::SpriteResetSuffix,
                    IntroGraphics::LoadFile(work) if work.sprite_disable_active() => super::startup_bus::CallerOwner::SpriteDisable,
                    IntroGraphics::LoadFile(work) if work.follower_initialization_active() => super::startup_bus::CallerOwner::FollowerInitialization,
                    IntroGraphics::LoadFile(work) if work.room_entry_cache_active() => super::startup_bus::CallerOwner::RoomEntryCache,
                    IntroGraphics::LoadFile(work) if work.interactive_cleanup_active() => super::startup_bus::CallerOwner::InteractiveCleanup,
                    IntroGraphics::LoadFile(work) if work.visited_active() => super::startup_bus::CallerOwner::DungeonVisited,
                    IntroGraphics::LoadFile(work) if work.attributes_active() => super::startup_bus::CallerOwner::DungeonAttributes,
                    IntroGraphics::LoadFile(work) if work.room_upload_active() => super::startup_bus::CallerOwner::DungeonRoomUpload,
                    IntroGraphics::LoadFile(work) if work.door_state_active() => super::startup_bus::CallerOwner::DungeonDoorState,
                    IntroGraphics::LoadFile(work) if work.saved_objects_active() => super::startup_bus::CallerOwner::DungeonSavedObjects,
                    IntroGraphics::LoadFile(work) if work.object_tiles_active() => super::startup_bus::CallerOwner::DungeonObjectTiles,
                    IntroGraphics::LoadFile(work) if work.object_parser_active() => super::startup_bus::CallerOwner::DungeonObjectParser,
                    IntroGraphics::LoadFile(work) if work.floors_active() => super::startup_bus::CallerOwner::DungeonFloors,
                    IntroGraphics::LoadFile(work) if work.room_parent_active() => super::startup_bus::CallerOwner::DungeonRoomParent,
                    IntroGraphics::LoadFile(work) if work.header_active() => super::startup_bus::CallerOwner::DungeonHeader,
                    IntroGraphics::LoadFile(work) if work.entrance_active() => super::startup_bus::CallerOwner::DungeonEntrance,
                    IntroGraphics::LoadFile(work) if work.parent_active() => super::startup_bus::CallerOwner::LoadFileParent,
                    _ => super::startup_bus::CallerOwner::Startup,
                };
                let cycles=graphics.step(self.context.as_mut().unwrap(),clock,dma, super::native_ppu::NativePpuBus::new(ppu, CallerBus {
                    callback: &mut bus, palette: palette_source, vblank: &mut vblank,
                    owner,
                    nmitimen: &mut nmitimen, hdma: &mut hdma, in_vblank: false,
                })).expect("completed graphics must return to its caller");
                let enable_in_vblank = nmitimen.is_some_and(|(value, blank)| value & 0x80 != 0 && !self.nmi_enabled && blank);
                if let Some((value, _)) = nmitimen {
                    assert_eq!(value & 0x30, 0, "load-game caller does not program a vertical IRQ");
                    self.nmi_enabled = value & 0x80 != 0;
                    self.vertical_irq = None;
                }
                if let Some(mask) = hdma {
                    for (i, channel) in dma.channel.iter_mut().enumerate() {
                        channel.hdma_active = mask & (1 << i) != 0;
                    }
                }
                crate::cycle_ledger::charge_routine(routine,cycles);
                let complete=graphics.complete();
                let closing=matches!(graphics, IntroGraphics::Closing(_)|IntroGraphics::Opening(_));
                let dma_end=match graphics {IntroGraphics::LoadFile(work)=>work.take_dma_end(),IntroGraphics::Darkening(work)=>work.take_dma_end(),IntroGraphics::FileMenu(work)=>work.take_dma_end(),IntroGraphics::NameSetup(work)=>work.take_dma_end(),_=>None};
                self.note_wait_vblank(vblank);
                if enable_in_vblank {
                    // Source write_reg(4200) defers the enable edge until the
                    // current CPU instruction has retired, then adds six clocks.
                    self.nmi_deadline = Some(clock.clock_master_cycles() + 6);
                }
                if let Some(end)=dma_end.filter(|_|self.nmi_deadline.is_some()) {
                    self.nmi_deadline=Some(end+u64::from(snes::SNES9X_NMI_GENERAL_DMA_DELAY_MASTER_CYCLES));
                }
                if complete {
                    if closing {
                        self.phase = Phase::IntroTailReturned;
                        return IntroClearBoundary::Tail;
                    }
                    self.phase=Phase::IntroClearReturned;return IntroClearBoundary::Returned;
                }
                continue;
            }
            if let Some(text)=self.intro_text.as_mut().filter(|text|!text.caller_complete()) {
                let routine=text.routine();
                let palette_source=text.palette_source(self.context.as_ref().unwrap());
                let cycles=text.step(self.context.as_mut().unwrap(),clock,|at,access| {
                    if let super::intro_clear::Access::Vblank{field,at}=access {vblank=Some((field,at));}
                    bus(at,StartupAccess::Memory(access,palette_source,super::startup_bus::CallerOwner::Startup))
                }).expect("completed text scan must hand off its caller");
                crate::cycle_ledger::charge_routine(routine,cycles);
                let complete=text.complete();
                let palette=text.palette_clear_complete();
                let caller=text.caller_complete();
                self.note_wait_vblank(vblank);
                if complete {
                    if caller {
                        self.phase=Phase::IntroClearReturned;
                        return IntroClearBoundary::Returned;
                    }
                    self.phase=Phase::IntroTextReturned;
                    return if palette {IntroClearBoundary::PaletteCleared} else {IntroClearBoundary::TextReturned};
                }
                continue;
            }
            let packing = self.intro_clear.complete() || self.intro_text.as_ref().is_some_and(|text|text.caller_complete())
                || self.intro_graphics.as_ref().is_some_and(|work|work.complete());
            let sources = packing && self.intro_packing.complete();
            let tail = sources && self.intro_sources.is_complete();
            let mut routine = if !packing { 0x0c_c1a0 }
                else if self.intro_packing.pending_call() { 0x00_8034 } else { 0x00_85fc };
            let cycles = if packing {
                let mut nmitimen = None;
                let mut hdma = None;
                let bus = super::native_hdma::HdmaBus {
                    dma,
                    bus: super::native_ppu::NativePpuBus::new(ppu, CallerBus {
                        callback: &mut bus,
                        palette: super::intro_palettes::PaletteSource::Constant,
                        owner: super::startup_bus::CallerOwner::Startup,
                        vblank: &mut vblank, nmitimen: &mut nmitimen, hdma: &mut hdma,
                        in_vblank: false,
                    }),
                };
                if tail {
                    self.intro_tail.step(self.context.as_mut().unwrap(), clock, bus)
                        .map(|(cycles, owner)| { routine = owner; cycles })
                } else if sources {
                    self.intro_sources.step_clocked(self.context.as_mut().unwrap(), clock, bus)
                } else {
                    self.intro_packing.step_with_bus(self.context.as_mut().unwrap(), clock, bus)
                }
            } else {
                self.intro_clear.step(self.context.as_mut().unwrap(), clock, |at, access| {
                    if let super::intro_clear::Access::Vblank { field, at } = access { vblank = Some((field, at)); }
                    bus(at,StartupAccess::Memory(access,super::intro_palettes::PaletteSource::Constant,super::startup_bus::CallerOwner::Startup))
                })
            };
            if let Some(cycles) = cycles {
                crate::cycle_ledger::charge_routine(routine, cycles);
                if sources && !tail { self.intro_source_master_cycles += cycles; }
            }
            if let Some((field, at)) = vblank {
                self.next_host = field + 1;
                self.host_return_pending = true;
                self.nmi_deadline = Some(at + u64::from(SNES9X_NMI_ACCEPTANCE_DELAY_MASTER_CYCLES));
            }
            if tail && self.intro_tail.complete() {
                self.phase=Phase::IntroTailReturned;
                return IntroClearBoundary::Tail;
            }
            if !tail && sources && self.intro_sources.is_complete() {
                self.phase = Phase::IntroSourcesReturned;
                return IntroClearBoundary::Sources;
            }
            if !sources && packing && self.intro_packing.complete() {
                self.phase = Phase::IntroPackingReturned;
                return IntroClearBoundary::Packed;
            }
            if !packing && self.intro_clear.complete() {
                self.phase = Phase::IntroClearReturned;
                return IntroClearBoundary::Returned;
            }
        }
    }

    pub(crate) fn accepted_context(&self) -> super::nmi_stack::NativeNmiContext {
        assert!(matches!(self.phase, Phase::Accepted(_)));
        self.context.expect("native initializer did not return its registers")
    }

    pub(crate) fn accept_wait_interrupt(&mut self, entered: super::nmi_stack::NativeNmiContext) {
        let Phase::Accepted(wait) = self.phase else { panic!("wait interrupt without acceptance") };
        self.interrupted_wait = self.context;
        self.context = Some(entered);
        self.phase = Phase::WaitHandler(wait);
    }

    pub(crate) fn wait_handler_context(&self) -> super::nmi_stack::NativeNmiContext {
        assert!(matches!(self.phase, Phase::WaitHandler(_)));
        self.context.unwrap()
    }

    #[cfg(test)]
    pub(crate) fn waiting_context(&self) -> Option<super::nmi_stack::NativeNmiContext> {
        if matches!(self.phase, Phase::Wait(_)) { self.context } else { None }
    }

    pub(crate) fn step(
        &mut self,
        clock: &mut CpuMasterTimeline,
        ram: &mut [u8],
        sram: &mut [u8],
        ppu: &mut super::native_ppu::NativePpuState,
        mut device_event: impl FnMut(CpuSynchronousTimelineEvent, &CpuMasterTimeline, &[u8], &mut super::native_ppu::NativePpuState) -> u32,
    ) -> StartupStep {
        // Snes9x checks the interrupt deadline before its SCAN_KEYS host exit.
        if self
            .nmi_deadline
            .is_some_and(|at| clock.clock_master_cycles() >= at)
        {
            let Phase::Wait(wait) = self.phase else {
                panic!("startup NMI outside main wait")
            };
            self.phase = Phase::Accepted(wait);
            self.context.as_mut().unwrap().pc = wait.next_pc() as u16;
            self.nmi_deadline = None;
            return StartupStep::AcceptNmi;
        }
        if matches!(self.phase,Phase::Wait(_)) {
            if let Some(deadline)=self.due_irq(clock) {return StartupStep::AcceptIrq(self.context.unwrap(),deadline);}
        }
        if std::mem::take(&mut self.host_return_pending) {
            return StartupStep::HostReturn;
        }
        if matches!(self.phase, Phase::WaitHandler(_)) {
            return StartupStep::RunNmi;
        }
        if let Phase::Wait(wait) = &mut self.phase {
            let mut vblank = None;
            wait.step_clocked(self.context.as_mut().unwrap(), clock, |_, access| {
                match access {
                    super::main_wait::WaitAccess::ReadLatch => ram[0x12],
                    super::main_wait::WaitAccess::Vblank { field, at } => {
                        vblank = Some((field, at));
                        0
                    }
                }
            }, |event, clock| device_event(event, clock, ram, ppu));
            ppu.synchronize(clock);
            self.note_wait_vblank(vblank);
            if matches!(self.phase,Phase::Wait(MainWait::Complete)) {
                self.phase=Phase::Complete;
                return StartupStep::MainReady;
            }
            return StartupStep::Instruction;
        }
        let mut result = StartupStep::Instruction;
        let cycles = match &mut self.phase {
            Phase::CallMemory => {
                ram[0x1fe..0x200].copy_from_slice(&0x802eu16.to_le_bytes());
                self.phase = Phase::Memory(StartupMemoryWork::default());
                46
            }
            Phase::Memory(work) => {
                if work.clears_window_mask() {
                    result = StartupStep::ClearWindow;
                }
                let Some(cycles) = work.step(ram, sram) else {
                    self.context = Some(work.returned_registers());
                    self.phase = Phase::LoadEnable;
                    return StartupStep::MemoryComplete;
                };
                cycles
            }
            Phase::LoadEnable => {
                let context = self.context.as_mut().unwrap();
                assert_eq!((context.bank, context.pc), (0, 0x802f));
                context.load_a_byte(0x81);
                context.pc = 0x8031;
                self.phase = Phase::StoreEnable;
                16
            }
            Phase::StoreEnable => {
                // The reset caller writes $81 to NMITIMEN after initialization.
                // If this work ever returns in vblank, its register-edge NMI
                // needs its own delayed installation; don't silently use the
                // ordinary HMax deadline for that unsupported handoff.
                assert!(
                    clock.raster_position().scanline() < 225,
                    "native startup NMI enable during vblank needs register-edge continuation"
                );
                self.nmi_enabled = true;
                let context = self.context.as_mut().unwrap();
                assert_eq!((context.bank, context.pc), (0, 0x8031));
                context.pc = 0x8034;
                self.phase = Phase::Wait(MainWait::default());
                30
            }
            Phase::Wait(_) => unreachable!("wait has its own bus owner"),
            Phase::Accepted(_) | Phase::WaitHandler(_) | Phase::Complete | Phase::ModuleEntered | Phase::IntroHalfSlotEntered | Phase::IntroHalfSlotReturned | Phase::IntroSetupReturned | Phase::IntroFirstClearEntered | Phase::IntroClearActive | Phase::IntroClearAccepted | Phase::IntroClearReturned | Phase::IntroPackingReturned | Phase::IntroSourcesReturned | Phase::IntroTailReturned | Phase::IntroTextReturned => panic!("startup instruction after NMI handoff"),
        };
        if result == StartupStep::ClearWindow {
            // STZ $212E is a word store: each six-clock PPU byte has its own
            // semantic boundary. In particular HMax may separate the writes.
            clock.advance_synchronous_pcbase_opcode_fetch(8);
            self.advance_reset_clock(clock, ppu, 16);
            ppu.write_clocked(clock, 0x212e, 0);
            self.advance_reset_clock(clock, ppu, 6);
            ppu.write_clocked(clock, 0x212f, 0);
            self.advance_reset_clock(clock, ppu, 6);
        } else {
            self.advance_reset_clock(clock, ppu, cycles);
        }
        result
    }

    fn advance_reset_clock(&mut self, clock: &mut CpuMasterTimeline,
        ppu: &mut super::native_ppu::NativePpuState, cycles: u32) {
        clock
            .advance_synchronous_after_semantics_with(cycles, |event, _| {
                if let CpuSynchronousTimelineEvent::HMax {
                    completed_field_index,
                    completed_scanline: 224,
                    event_timestamp,
                    ..
                } = event
                {
                    self.host_return_pending = true;
                    self.next_host = completed_field_index + 1;
                    if self.nmi_enabled {
                        self.nmi_deadline = Some(
                            event_timestamp.master_cycles()
                                + u64::from(SNES9X_NMI_ACCEPTANCE_DELAY_MASTER_CYCLES),
                        );
                    }
                }
                Ok::<u32, String>(0)
            })
            .expect("native reset caller clock");
        ppu.synchronize(clock);
    }

    /// Resume the saved wait instruction on the same clock after actual RTI.
    /// A suspended BEQ uses its pre-NMI flags; only LDA rereads the live latch.
    pub(crate) fn return_from_nmi(&mut self, clock: &mut CpuMasterTimeline,
        context: &mut super::nmi_stack::NativeNmiContext, ram: &[u8],
        mut device_event: impl FnMut(CpuSynchronousTimelineEvent, &CpuMasterTimeline) -> u32) -> u32 {
        let (Phase::Accepted(mut wait) | Phase::WaitHandler(mut wait)) = self.phase else {
            panic!("startup return without accepted NMI")
        };
        if let Some(interrupted) = self.interrupted_wait.take() {
            assert_eq!(*context, interrupted, "native NMI changed its interrupted wait context");
        }
        assert_ne!(ram[0x12], 0, "first NMI did not release main wait");
        assert_eq!((context.bank, context.pc, context.d, context.p & 0x30),
            (0, wait.next_pc() as u16, 0, 0x30));
        let mut cycles = 0;
        while wait != MainWait::Complete {
            let mut vblank = None;
            cycles += wait.step_clocked(context, clock, |_, access| match access {
                super::main_wait::WaitAccess::ReadLatch => ram[0x12],
                super::main_wait::WaitAccess::Vblank { field, at } => {
                    vblank = Some((field, at));
                    0
                }
            }, &mut device_event);
            self.note_wait_vblank(vblank);
        }
        self.context = Some(*context);
        self.phase = Phase::Complete;
        cycles
    }

}
