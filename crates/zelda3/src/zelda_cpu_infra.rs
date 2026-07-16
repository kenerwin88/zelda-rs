//! Lockstep oracle harness, ported from `src/zelda_cpu_infra.c`.

#![allow(non_snake_case)]

use std::cell::RefCell;
use std::error::Error;
use std::fmt;
use std::slice;

use snes::{cpu_run_opcode, Cart, LoadRomError, Snes};

use crate::game_state::constants::{RUN_MAIN_THREAD, RUN_POLY_THREAD};
use crate::game_state::{
    AncillaSlotSnapshot, DisplayState, FrameState, PlayerSnapshotState, SpriteSlotSnapshot,
    WorldLocationState, WorldState,
};
use crate::types::{read_le_u16, write_le_u16};
use crate::zelda_rtl::ZeldaState;
use crate::OverworldMap16LoadState;

pub const RUN_MAIN: u8 = 1;
pub const RUN_POLY: u8 = 2;

const MAX_ORIG_LOOP_OPCODES: usize = 2_000_000;

#[derive(Debug)]
pub enum OracleError {
    LoadRom(LoadRomError),
    PatchRom(String),
    LoadAssets(String),
    LoadSram(String),
    StepLimitExceeded { pc: u32, limit: usize },
    Diverged(ComparisonReport),
}

impl fmt::Display for OracleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OracleError::LoadRom(err) => write!(f, "failed to load ROM: {err}"),
            OracleError::PatchRom(err) => write!(f, "failed to patch oracle ROM: {err}"),
            OracleError::LoadAssets(err) => write!(f, "failed to load assets: {err}"),
            OracleError::LoadSram(err) => write!(f, "failed to load SRAM: {err}"),
            OracleError::StepLimitExceeded { pc, limit } => {
                write!(f, "SNES oracle did not reach a frame checkpoint within {limit} opcodes (pc=${pc:06X})")
            }
            OracleError::Diverged(report) => write!(f, "{report}"),
        }
    }
}

impl Error for OracleError {}

impl From<LoadRomError> for OracleError {
    fn from(value: LoadRomError) -> Self {
        OracleError::LoadRom(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Region {
    Wram,
    Sram,
    Vram,
    Cgram,
    Oam,
    PpuRegs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Difference {
    pub region: Region,
    pub offset: usize,
    pub mine: u16,
    pub theirs: u16,
    pub previous: u16,
    pub width: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComparisonReport {
    pub frame: u32,
    pub differences: Vec<Difference>,
    pub total_wram: usize,
    pub total_sram: usize,
    pub total_vram: usize,
    pub total_cgram: usize,
    pub total_oam: usize,
    pub total_ppu_regs: usize,
}

impl ComparisonReport {
    pub fn is_empty(&self) -> bool {
        self.total_wram == 0
            && self.total_sram == 0
            && self.total_vram == 0
            && self.total_cgram == 0
            && self.total_oam == 0
            && self.total_ppu_regs == 0
    }
}

impl fmt::Display for ComparisonReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "lockstep divergence at frame {}: WRAM={} SRAM={} VRAM={} CGRAM={} OAM={} PPU_REGS={}",
            self.frame,
            self.total_wram,
            self.total_sram,
            self.total_vram,
            self.total_cgram,
            self.total_oam,
            self.total_ppu_regs
        )?;
        for diff in self.differences.iter().take(32) {
            write!(
                f,
                "\n  {:?} ${:06X}: mine={:0width$X} theirs={:0width$X} prev={:0width$X}",
                diff.region,
                diff.offset,
                diff.mine,
                diff.theirs,
                diff.previous,
                width = (diff.width as usize) * 2
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SemanticSnapshot {
    pub frame: SemanticFrame,
    pub player: SemanticPlayer,
    pub world: SemanticWorld,
    pub ppu: SemanticPpu,
    pub sprites: Vec<SemanticSpriteSlot>,
    pub ancillas: Vec<SemanticAncillaSlot>,
}

impl fmt::Display for SemanticSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "main={:02X}/sub={:02X}/subsub={:02X} room={:04X} screen={:02X} link=({:04X},{:04X},z={:04X}) dir={:02X}/face={:02X} handler={:02X}/aux={:02X} bg1=({:04X},{:04X}) bg2=({:04X},{:04X}) ppu=mode:{}/bright:{:02X}/blank:{} sprites={} ancillas={}",
            self.frame.main_module,
            self.frame.submodule,
            self.frame.subsubmodule,
            self.world.dungeon_room,
            self.world.overworld_screen,
            self.player.x,
            self.player.y,
            self.player.z,
            self.player.direction,
            self.player.facing,
            self.player.handler_state,
            self.player.auxiliary_state,
            self.world.bg1_x,
            self.world.bg1_y,
            self.world.bg2_x,
            self.world.bg2_y,
            self.ppu.mode,
            self.ppu.brightness,
            self.ppu.forced_blank,
            self.sprites.len(),
            self.ancillas.len(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SemanticFrame {
    pub main_module: u8,
    pub submodule: u8,
    pub subsubmodule: u8,
    pub nmi_thread_active: bool,
    pub selected_run_thread: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SemanticPlayer {
    pub x: u16,
    pub y: u16,
    pub z: u16,
    pub x_velocity: u8,
    pub y_velocity: u8,
    pub z_velocity: u8,
    pub direction: u8,
    pub last_direction: u8,
    pub facing: u8,
    pub handler_state: u8,
    pub auxiliary_state: u8,
    pub current_health: u8,
    pub magic_power: u8,
    pub equipped_item: u8,
    pub item_in_hand: u8,
    pub current_item_y: u8,
    pub current_item_active: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SemanticWorld {
    pub dungeon_room: u16,
    pub overworld_screen: u8,
    pub overworld_area: u16,
    pub transition_direction: u8,
    pub overlay_index: u8,
    pub map16_load_src: u16,
    pub map16_load_dst: u16,
    pub map16_load_y_unit: u16,
    pub bg1_x: u16,
    pub bg1_y: u16,
    pub bg2_x: u16,
    pub bg2_y: u16,
    pub camera_x: u16,
    pub camera_y: u16,
    pub rng_seed: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SemanticPpu {
    pub screen_enabled: [u8; 4],
    pub screen_windowed: [u8; 4],
    pub mosaic_enabled: u8,
    pub mosaic_size: u8,
    pub math_enabled: u8,
    pub fixed_color: [u8; 3],
    pub forced_blank: bool,
    pub brightness: u8,
    pub mode: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SemanticSpriteSlot {
    pub slot: u8,
    pub sprite_type: u8,
    pub state: u8,
    pub x: u16,
    pub y: u16,
    pub x_velocity: u8,
    pub y_velocity: u8,
    pub ai_state: u8,
    pub delay_main: u8,
    pub health: u8,
    pub hit_timer: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SemanticAncillaSlot {
    pub slot: u8,
    pub ancilla_type: u8,
    pub x: u16,
    pub y: u16,
    pub x_velocity: u8,
    pub y_velocity: u8,
    pub item_to_link: u8,
    pub timer: u8,
    pub direction: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SemanticDifference {
    pub field: String,
    pub mine: String,
    pub theirs: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SemanticComparisonReport {
    pub frame: u32,
    pub differences: Vec<SemanticDifference>,
}

impl SemanticComparisonReport {
    pub fn is_empty(&self) -> bool {
        self.differences.is_empty()
    }
}

impl fmt::Display for SemanticComparisonReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return write!(f, "semantic state matches at frame {}", self.frame);
        }
        write!(
            f,
            "semantic divergence at frame {}: {} changed field(s)",
            self.frame,
            self.differences.len()
        )?;
        for diff in self.differences.iter().take(16) {
            write!(
                f,
                "\n  {}: mine={} theirs={}",
                diff.field, diff.mine, diff.theirs
            )?;
        }
        Ok(())
    }
}

#[derive(Clone)]
struct Snapshot {
    a: u16,
    x: u16,
    y: u16,
    sp: u16,
    dp: u16,
    pc: u16,
    k: u8,
    db: u8,
    flags: u8,
    ram: Vec<u8>,
    sram: Vec<u8>,
    vram: Vec<u16>,
    cgram: Vec<u16>,
    oam: Vec<u16>,
    ppu_regs: Vec<u8>,
    semantic: SemanticSnapshot,
}

impl Snapshot {
    fn from_snes(snes: &Snes) -> Self {
        let mut ram = snes.ram.clone();
        copy_within_clone(&mut ram, 0x1b00, 0x1dba0, 224 * 2);
        let ppu_regs = ppu_visible_regs(&snes.ppu);
        let semantic = semantic_snapshot_from_parts(&ram, &ppu_regs);
        Self {
            a: snes.cpu.a,
            x: snes.cpu.x,
            y: snes.cpu.y,
            sp: snes.cpu.sp,
            dp: snes.cpu.dp,
            pc: snes.cpu.pc,
            k: snes.cpu.k,
            db: snes.cpu.db,
            flags: snes.cpu.pack_flags(),
            ram,
            sram: snes.cart.ram.clone(),
            vram: snes.ppu.vram.clone(),
            cgram: snes.ppu.cgram.clone(),
            oam: snes.ppu.oam.clone(),
            ppu_regs,
            semantic,
        }
    }

    fn from_game(game: &ZeldaState) -> Self {
        let mut ram = game.ram.clone();
        copy_within_clone(&mut ram, 0x1dba0, 0x1b00, 224 * 2);
        let ppu_regs = ppu_visible_regs(&game.ppu);
        let semantic = semantic_snapshot_from_game(game, &ram, &ppu_regs);
        Self {
            a: 0,
            x: 0,
            y: 0,
            sp: 0,
            dp: 0,
            pc: 0,
            k: 0,
            db: 0,
            flags: 0,
            ram,
            sram: game.sram.clone(),
            vram: game.ppu.vram.clone(),
            cgram: game.ppu.cgram.clone(),
            oam: game.ppu.oam.clone(),
            ppu_regs,
            semantic,
        }
    }

    fn restore_snes(&self, snes: &mut Snes) {
        snes.cpu.a = self.a;
        snes.cpu.x = self.x;
        snes.cpu.y = self.y;
        snes.cpu.sp = self.sp;
        snes.cpu.dp = self.dp;
        snes.cpu.pc = self.pc;
        snes.cpu.k = self.k;
        snes.cpu.db = self.db;
        snes.cpu.unpack_flags(self.flags);
        snes.ram.copy_from_slice(&self.ram);
        let sram_len = snes.cart.ram.len().min(self.sram.len());
        snes.cart.ram[..sram_len].copy_from_slice(&self.sram[..sram_len]);
        snes.ppu.vram.copy_from_slice(&self.vram);
        snes.ppu.cgram.copy_from_slice(&self.cgram);
        snes.ppu.oam.copy_from_slice(&self.oam);
    }

    fn restore_game(&self, game: &mut ZeldaState) {
        game.ram.copy_from_slice(&self.ram);
        game.sync_native_game_state_from_ram();
        let sram_len = game.sram.len();
        game.sram.copy_from_slice(&self.sram[..sram_len]);
        game.ppu.vram.copy_from_slice(&self.vram);
        game.ppu.cgram.copy_from_slice(&self.cgram);
        game.ppu.oam.copy_from_slice(&self.oam);
    }

    fn semantic(&self) -> SemanticSnapshot {
        self.semantic.clone()
    }
}

fn ppu_visible_regs(ppu: &snes::PpuState) -> Vec<u8> {
    let mut out = Vec::with_capacity(128);
    out.push(ppu.extra_left_cur);
    out.push(ppu.extra_right_cur);
    out.push(ppu.extra_left_right);
    out.push(ppu.extra_bottom_cur);
    out.extend_from_slice(&ppu.screen_enabled);
    out.extend_from_slice(&ppu.screen_windowed);
    out.push(ppu.mosaic_enabled);
    out.push(ppu.mosaic_size);
    out.extend_from_slice(&ppu.obj_tile_adr1.to_le_bytes());
    out.extend_from_slice(&ppu.obj_tile_adr2.to_le_bytes());
    out.push(ppu.obj_size);
    out.push(ppu.window1_left);
    out.push(ppu.window1_right);
    out.push(ppu.window2_left);
    out.push(ppu.window2_right);
    out.extend_from_slice(&ppu.windowsel.to_le_bytes());
    out.push(ppu.clip_mode);
    out.push(ppu.prevent_math_mode);
    out.push(ppu.add_subscreen as u8);
    out.push(ppu.subtract_color as u8);
    out.push(ppu.half_color as u8);
    out.push(ppu.math_enabled);
    out.push(ppu.fixed_color_r);
    out.push(ppu.fixed_color_g);
    out.push(ppu.fixed_color_b);
    out.push(ppu.forced_blank as u8);
    out.push(ppu.brightness);
    out.push(ppu.mode);
    for bg in &ppu.bg_layer {
        out.extend_from_slice(&bg.h_scroll.to_le_bytes());
        out.extend_from_slice(&bg.v_scroll.to_le_bytes());
        out.push(bg.tilemap_wider as u8);
        out.push(bg.tilemap_higher as u8);
        out.extend_from_slice(&bg.tilemap_adr.to_le_bytes());
        out.extend_from_slice(&bg.tile_adr.to_le_bytes());
    }
    out.push(ppu.scroll_prev);
    out.push(ppu.scroll_prev2);
    if ppu.mode & 7 == 7 {
        for value in ppu.m7_matrix {
            out.extend_from_slice(&value.to_le_bytes());
        }
        out.push(ppu.m7_prev);
        out.push(ppu.m7_large_field as u8);
        out.push(ppu.m7_char_fill as u8);
        out.push(ppu.m7_x_flip as u8);
        out.push(ppu.m7_y_flip as u8);
        out.push(ppu.m7_ext_bg_always_zero as u8);
    }
    out
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct LockstepOracle {
    pub snes: Snes,
    pub game: ZeldaState,
}

impl LockstepOracle {
    pub fn new() -> Self {
        Self {
            snes: Snes::new(),
            game: ZeldaState::new(),
        }
    }

    pub fn load_sram(&mut self, sram: &[u8]) -> Result<(), OracleError> {
        let expected = self.game.sram.len();
        if sram.len() < expected {
            return Err(OracleError::LoadSram(format!(
                "expected at least {} bytes, got {}",
                expected,
                sram.len()
            )));
        }
        let sram = &sram[..expected];
        self.game.sram.copy_from_slice(sram);
        self.snes.cart.ram.copy_from_slice(sram);
        Ok(())
    }

    pub fn sync_oracle_from_game(&mut self) {
        self.emu_synchronize_whole_state();
    }

    pub fn sync_game_from_oracle(&mut self) {
        self.game.ppu = self.snes.ppu.clone();
        self.game.dma = self.snes.dma.clone();
        self.game.ram.copy_from_slice(&self.snes.ram);
        self.game.sync_native_game_state_from_ram();
        self.game.sram.copy_from_slice(&self.snes.cart.ram);
    }

    pub fn compare_current(&self) -> Result<(), OracleError> {
        let before = Snapshot::from_snes(&self.snes);
        let mine = Snapshot::from_game(&self.game);
        let theirs = Snapshot::from_snes(&self.snes);
        let report = compare_snapshots_eq(self.snes.frames, mine, theirs, &before);
        if report.is_empty() {
            Ok(())
        } else {
            Err(OracleError::Diverged(report))
        }
    }

    pub fn compare_current_non_render_state(&self) -> Result<(), OracleError> {
        let before = Snapshot::from_snes(&self.snes);
        let mine = Snapshot::from_game(&self.game);
        let theirs = Snapshot::from_snes(&self.snes);
        let report =
            compare_snapshots_eq_with_options(self.snes.frames, mine, theirs, &before, true);
        if report.is_empty() {
            Ok(())
        } else {
            Err(OracleError::Diverged(report))
        }
    }

    pub fn compare_current_with_graduated_semantics(&self) -> Result<(), OracleError> {
        let before = Snapshot::from_snes(&self.snes);
        let mine = Snapshot::from_game(&self.game);
        let theirs = Snapshot::from_snes(&self.snes);
        let report =
            compare_snapshots_eq_with_graduated_semantics(self.snes.frames, mine, theirs, &before);
        if report.is_empty() {
            Ok(())
        } else {
            Err(OracleError::Diverged(report))
        }
    }

    pub fn semantic_snapshot_pair(&self) -> (SemanticSnapshot, SemanticSnapshot) {
        (
            Snapshot::from_game(&self.game).semantic(),
            Snapshot::from_snes(&self.snes).semantic(),
        )
    }

    pub fn semantic_game_snapshot(&self) -> SemanticSnapshot {
        Snapshot::from_game(&self.game).semantic()
    }

    pub fn compare_current_semantic(&self) -> SemanticComparisonReport {
        let (mine, theirs) = self.semantic_snapshot_pair();
        compare_semantic_snapshots(self.snes.frames, &mine, &theirs)
    }

    pub fn run_frame_with_compare(
        &mut self,
        input_state: u16,
        run_what: u8,
    ) -> Result<(), OracleError> {
        self.emu_run_frame_with_compare(input_state, run_what)
    }

    pub fn emu_run_frame_with_compare(
        &mut self,
        input_state: u16,
        run_what: u8,
    ) -> Result<(), OracleError> {
        let before = Snapshot::from_snes(&self.snes);
        let mine_before = Snapshot::from_game(&self.game);
        let theirs_before = Snapshot::from_snes(&self.snes);
        let report = compare_snapshots_eq(self.snes.frames, mine_before, theirs_before, &before);
        if !report.is_empty() {
            return Err(OracleError::Diverged(report));
        }

        self.run_oracle_frame(input_state, run_what)?;
        let theirs = Snapshot::from_snes(&self.snes);

        self.game.run_frame_internal(input_state, run_what);
        let mine = Snapshot::from_game(&self.game);

        let report = compare_snapshots_eq(self.snes.frames, mine, theirs, &before);
        if report.is_empty() {
            Ok(())
        } else {
            Err(OracleError::Diverged(report))
        }
    }

    pub fn run_oracle_frame(&mut self, input_state: u16, run_what: u8) -> Result<(), OracleError> {
        self.snes.input1.current_state = input_state;
        run_emulated_snes_frame_checked(&mut self.snes, run_what)?;
        self.snes.frames = self.snes.frames.wrapping_add(1);
        Ok(())
    }

    pub fn measure_poly_frame_cycles(&mut self, game: &ZeldaState) -> Result<u64, OracleError> {
        self.game = game.clone();
        self.emu_synchronize_whole_state();
        self.snes.cpu.sp = 0x1f3e;
        self.snes.cpu.pc = 0xf81d;
        self.snes.cpu.db = 9;
        self.snes.cpu.k = 9;
        self.snes.cpu.dp = 0x1f00;
        self.snes.cpu.a = 0;
        self.snes.cpu.x = 0;
        self.snes.cpu.y = 0;
        self.snes.cpu.e = false;
        self.snes.cpu.irq_wanted = false;
        self.snes.cpu.nmi_wanted = false;
        self.snes.cpu.waiting = false;
        self.snes.cpu.stopped = false;
        self.snes.cpu.unpack_flags(0x30);

        let mut cycles = 0u64;
        for loops in 0..MAX_ORIG_LOOP_OPCODES {
            cycles = cycles.saturating_add(u64::from(cpu_run_opcode(&mut self.snes)));
            while self.snes.dma.dma_busy {
                self.snes.dma_do();
            }
            let pc = ((self.snes.cpu.k as u32) << 16) | self.snes.cpu.pc as u32;
            if pc == 0x09f81d && loops >= 10 {
                return Ok(cycles);
            }
        }

        let pc = ((self.snes.cpu.k as u32) << 16) | self.snes.cpu.pc as u32;
        Err(OracleError::StepLimitExceeded {
            pc,
            limit: MAX_ORIG_LOOP_OPCODES,
        })
    }

    pub fn emu_synchronize_whole_state(&mut self) {
        self.snes.ppu = self.game.ppu.clone();
        self.snes.dma = self.game.dma.clone();
        self.snes.ram.copy_from_slice(&self.game.ram);
        self.snes.cart.ram.copy_from_slice(&self.game.sram);

        if self.game.ram[0xadc] == 0 && self.game.ram[0xadd] == 0 {
            self.snes.cpu_seed_reset_vector();
        }
    }

    pub fn run_emulated_func(
        &mut self,
        pc: u32,
        a: u16,
        x: u16,
        y: u16,
        mf: bool,
        xf: bool,
        b: i32,
        whatflags: i32,
    ) -> Result<(), OracleError> {
        self.snes.debug_cycles = true;
        let result = self.run_emulated_func_silent(pc, a, x, y, mf, xf, b, whatflags | 2);
        self.snes.debug_cycles = false;
        result
    }

    pub fn run_emulated_func_silent(
        &mut self,
        pc: u32,
        a: u16,
        x: u16,
        y: u16,
        mf: bool,
        xf: bool,
        b: i32,
        whatflags: i32,
    ) -> Result<(), OracleError> {
        self.emu_synchronize_whole_state();
        if whatflags & 2 != 0 {
            self.snes.ram[0x1ffff] = 0x67;
        }

        let org_sp = self.snes.cpu.sp;
        let org_pc = self.snes.cpu.pc;
        let org_b = self.snes.cpu.db;
        let org_dp = self.snes.cpu.dp;
        if b != -1 {
            self.snes.cpu.db = if b >= 0 { b as u8 } else { (pc >> 16) as u8 };
        }
        if b == -3 {
            self.snes.cpu.dp = 0x1f00;
        }

        self.snes.cpu.a = a;
        self.snes.cpu.x = x;
        self.snes.cpu.y = y;
        self.snes.cpu.sp_breakpoint = self.snes.cpu.sp;
        self.snes.cpu.k = (pc >> 16) as u8;
        self.snes.cpu.pc = pc as u16;
        self.snes.cpu.mf = mf;
        self.snes.cpu.xf = xf;

        for _ in 0..MAX_ORIG_LOOP_OPCODES {
            if self.snes.debug_cycles {
                println!("{}", self.snes.cpu_trace_line());
            }
            let _ = cpu_run_opcode(&mut self.snes);
            while self.snes.dma.dma_busy {
                self.snes.dma_do();
            }
            if self.snes.cpu.sp_breakpoint == 0 {
                self.snes.cpu.dp = org_dp;
                self.snes.cpu.sp = org_sp;
                self.snes.cpu.db = org_b;
                self.snes.cpu.pc = org_pc;
                self.sync_game_from_oracle();
                return Ok(());
            }
        }

        let pc = ((self.snes.cpu.k as u32) << 16) | self.snes.cpu.pc as u32;
        self.snes.cpu.dp = org_dp;
        self.snes.cpu.sp = org_sp;
        self.snes.cpu.db = org_b;
        self.snes.cpu.pc = org_pc;
        Err(OracleError::StepLimitExceeded {
            pc,
            limit: MAX_ORIG_LOOP_OPCODES,
        })
    }

    pub fn emu_initialize_owned(rom: &[u8]) -> Result<Self, OracleError> {
        let mut oracle = Self::new();
        oracle.load_rom(rom)?;
        oracle.sync_game_from_oracle();
        Ok(oracle)
    }
}

impl Default for LockstepOracle {
    fn default() -> Self {
        Self::new()
    }
}

thread_local! {
    static GLOBAL_ORACLE: RefCell<Option<LockstepOracle>> = const { RefCell::new(None) };
}

pub fn emu_initialize(data: *mut u8, size: usize) -> bool {
    if data.is_null() {
        return false;
    }
    let rom = unsafe { slice::from_raw_parts(data.cast_const(), size) };
    match LockstepOracle::emu_initialize_owned(rom) {
        Ok(oracle) => {
            GLOBAL_ORACLE.with(|slot| *slot.borrow_mut() = Some(oracle));
            true
        }
        Err(_) => false,
    }
}

pub fn emu_run_frame_with_compare(input_state: u16, run_what: i32) {
    GLOBAL_ORACLE.with(|slot| {
        if let Some(oracle) = slot.borrow_mut().as_mut() {
            oracle
                .emu_run_frame_with_compare(input_state, run_what as u8)
                .expect("oracle frame diverged");
        }
    });
}

fn emu_synchronize_whole_state() {
    GLOBAL_ORACLE.with(|slot| {
        if let Some(oracle) = slot.borrow_mut().as_mut() {
            oracle.emu_synchronize_whole_state();
        }
    });
}

pub fn get_ptr(addr: u32) -> u8 {
    GLOBAL_ORACLE.with(|slot| {
        slot.borrow()
            .as_ref()
            .and_then(|oracle| get_ptr_ref(&oracle.snes.cart, addr).copied())
            .unwrap_or(0)
    })
}

pub fn get_cart_ram_ptr(addr: u32) -> u8 {
    GLOBAL_ORACLE.with(|slot| {
        slot.borrow()
            .as_ref()
            .and_then(|oracle| get_cart_ram_ptr_ref(&oracle.snes.cart, addr).copied())
            .unwrap_or(0)
    })
}

fn make_snapshot(s: *mut Snapshot) {
    if s.is_null() {
        return;
    }
    GLOBAL_ORACLE.with(|slot| {
        if let Some(oracle) = slot.borrow().as_ref() {
            unsafe {
                *s = Snapshot::from_snes(&oracle.snes);
            }
        }
    });
}

fn make_my_snapshot(s: *mut Snapshot) {
    if s.is_null() {
        return;
    }
    GLOBAL_ORACLE.with(|slot| {
        if let Some(oracle) = slot.borrow().as_ref() {
            unsafe {
                *s = Snapshot::from_game(&oracle.game);
            }
        }
    });
}

fn restore_snapshot(s: *mut Snapshot) {
    if s.is_null() {
        return;
    }
    GLOBAL_ORACLE.with(|slot| {
        if let Some(oracle) = slot.borrow_mut().as_mut() {
            unsafe {
                (*s).restore_snes(&mut oracle.snes);
            }
        }
    });
}

fn restore_my_snapshot(s: *mut Snapshot) {
    if s.is_null() {
        return;
    }
    GLOBAL_ORACLE.with(|slot| {
        if let Some(oracle) = slot.borrow_mut().as_mut() {
            unsafe {
                (*s).restore_game(&mut oracle.game);
            }
        }
    });
}

fn verify_snapshots_eq(b: *mut Snapshot, a: *mut Snapshot, prev: *mut Snapshot) {
    if b.is_null() || a.is_null() || prev.is_null() {
        return;
    }
    let report = unsafe { compare_snapshots_eq(0, (*b).clone(), (*a).clone(), &*prev) };
    assert!(report.is_empty(), "{report}");
}

pub fn rom_byte(cart: &Cart, addr: u32) -> u8 {
    get_ptr_ref(cart, addr).copied().unwrap_or(0)
}

#[rustfmt::skip]
pub fn run_emulated_func(pc: u32, a: u16, x: u16, y: u16, mf: bool, xf: bool, b: i32, whatflags: i32) {
    GLOBAL_ORACLE.with(|slot| {
        if let Some(oracle) = slot.borrow_mut().as_mut() {
            oracle
                .run_emulated_func(pc, a, x, y, mf, xf, b, whatflags)
                .expect("emulated function failed");
        }
    });
}

#[rustfmt::skip]
pub fn run_emulated_func_silent(pc: u32, a: u16, x: u16, y: u16, mf: bool, xf: bool, b: i32, whatflags: i32) {
    GLOBAL_ORACLE.with(|slot| {
        if let Some(oracle) = slot.borrow_mut().as_mut() {
            oracle
                .run_emulated_func_silent(pc, a, x, y, mf, xf, b, whatflags)
                .expect("emulated function failed");
        }
    });
}

fn run_emulated_snes_frame(snes: &mut Snes, run_what: i32) {
    run_emulated_snes_frame_checked(snes, run_what as u8).expect("emulated SNES frame failed");
}

pub fn run_orig_asm_code_one_loop(snes: &mut Snes) {
    run_orig_asm_code_one_loop_checked(snes).expect("original ASM loop failed");
}

fn patch_rom(rom: *mut u8) {
    if rom.is_null() {
        return;
    }
    let bytes = unsafe { slice::from_raw_parts(rom.cast_const(), 0x100000) };
    let patched = patch_rom_owned(bytes).expect("ROM patch failed");
    unsafe {
        std::ptr::copy_nonoverlapping(patched.as_ptr(), rom, patched.len());
    }
}

fn patch_rom_bp(rom: *mut u8, addr: u32) {
    with_rom_slice(rom, |rom| patch_rom_bp_checked(rom, addr));
}

fn patch_rom_byte(rom: *mut u8, addr: u32, old_value: u8, value: u8) {
    with_rom_slice(rom, |rom| {
        let idx = lorom_index(addr);
        assert_eq!(rom[idx], old_value);
        patch_rom_byte_checked(rom, addr, value)
    });
}

fn patch_rom_word(rom: *mut u8, addr: u32, old_value: u16, value: u16) {
    with_rom_slice(rom, |rom| {
        let idx = lorom_index(addr);
        assert_eq!(read_le_u16(rom, idx), old_value);
        patch_rom_word_checked(rom, addr, value)
    });
}

fn patch_rom_array(rom: *mut u8, addr: u32, values: *const u8, n: i32) {
    if values.is_null() || n <= 0 {
        return;
    }
    let values = unsafe { slice::from_raw_parts(values, n as usize) };
    with_rom_slice(rom, |rom| patch_rom_array_checked(rom, addr, values));
}

fn with_rom_slice<F>(rom: *mut u8, f: F)
where
    F: FnOnce(&mut [u8]) -> Result<(), OracleError>,
{
    if rom.is_null() {
        return;
    }
    let rom = unsafe { slice::from_raw_parts_mut(rom, 0x100000) };
    f(rom).expect("ROM patch failed");
}

pub fn hooked_function_rts(_is_long: i32) {
    GLOBAL_CALLING_ASM_FROM_C
        .with(|calling| hooked_function_rts_checked(&mut calling.borrow_mut()));
}

thread_local! {
    static GLOBAL_CALLING_ASM_FROM_C: RefCell<bool> = const { RefCell::new(false) };
}

fn run_emulated_snes_frame_checked(snes: &mut Snes, run_what: u8) -> Result<(), OracleError> {
    if snes.cpu.pc == 0x8000 && snes.cpu.k == 0 {
        run_orig_asm_code_one_loop_checked(snes)?;
        snes.ram[0x12] = 1;
        write_le_u16(&mut snes.ram, 0x0ae0, 0xb280);
        write_le_u16(&mut snes.ram, 0x0ae2, 0xb280 + 0x60);
    }

    if run_what & RUN_POLY != 0 {
        snes.cpu.sp = 0x1f3e;
        snes.cpu.pc = 0xf81d;
        snes.cpu.db = 9;
        snes.cpu.k = 9;
        snes.cpu.dp = 0x1f00;
        run_orig_asm_code_one_loop_checked(snes)?;
    }

    if run_what & RUN_MAIN != 0 {
        snes.cpu.sp = 0x01ff;
        snes.cpu.pc = 0x8034;
        snes.cpu.k = 0;
        snes.cpu.dp = 0;
        snes.cpu.db = 0;
        run_orig_asm_code_one_loop_checked(snes)?;
    }

    snes.do_auto_joypad();

    if snes.ram[0x0add] == 0 {
        write_le_u16(&mut snes.ram, 0x0adc, 0xa680);
    }

    snes.write(0x004300, 0x01);
    snes.write(0x004301, 0x18);

    snes.cpu.sp = 0x01ff;
    snes.cpu.pc = 0x80d9;
    snes.cpu.k = 0;
    snes.cpu.dp = 0;
    snes.cpu.db = 0;
    run_orig_asm_code_one_loop_checked(snes)
}

pub fn get_ptr_ref(cart: &Cart, addr: u32) -> Option<&u8> {
    if cart.rom.is_empty() {
        return None;
    }
    let idx = lorom_index(addr) & (cart.rom.len() - 1);
    cart.rom.get(idx)
}

pub fn get_cart_ram_ptr_ref(cart: &Cart, addr: u32) -> Option<&u8> {
    cart.ram.get(addr as usize)
}

pub(crate) fn patch_rom_owned(rom: &[u8]) -> Result<Vec<u8>, OracleError> {
    // The C oracle patches the real 1 MiB Zelda ROM before loading it.
    // Keep tiny synthetic test ROMs unpatched so unit tests can use minimal
    // images without carrying the full Zelda address map.
    if rom.len() < 0x100000 {
        return Ok(rom.to_vec());
    }

    let mut patched = rom.to_vec();

    checked_copy_within(&mut patched, 0x36436, 0x36434, 7)?;
    write_direct(&mut patched, 0x3643b, 0xb0)?;
    write_direct(&mut patched, 0x3643c, 0x40 - 7)?;

    write_direct_bytes(
        &mut patched,
        0x17f6e,
        &[
            0xc0, 0x00, 0x20, 0x90, 0x03, 0xa9, 0x00, 0x00, 0x9d, 0x00, 0x05, 0x60,
        ],
    )?;
    for &offset in &[0x174a7, 0x174b5, 0x173dd, 0x173ef] {
        write_direct_bytes(&mut patched, offset, &[0x20, 0x6e, 0xff])?;
    }

    write_direct_bytes(
        &mut patched,
        0x6ffc1,
        &[
            0xad, 0xa1, 0x0f, 0x18, 0x65, 0x1a, 0x4a, 0xb0, 0x02, 0x49, 0xb8, 0x8d, 0xa1, 0x0f,
            0x18, 0x6b,
        ],
    )?;
    write_direct_bytes(&mut patched, 0x6ba71, &[0x4c, 0xc1, 0xff])?;

    write_direct_bytes(
        &mut patched,
        0x6ffd8,
        &[
            0xa5, 0x00, 0x48, 0xa5, 0x02, 0x48, 0x22, 0x5c, 0xad, 0x02, 0xc2, 0x30, 0x68, 0x85,
            0x02, 0x68, 0x85, 0x00, 0x6b,
        ],
    )?;
    write_direct_bytes(&mut patched, 0xdc0f2, &[0xd8, 0xff, 0x0d])?;

    for &offset in &[0x2dec7, 0x4be5e, 0xd79a4, 0xf0a46, 0xf0a52, 0x4a966] {
        write_direct(&mut patched, offset, 0)?;
    }
    write_direct(&mut patched, 0xef9b9, 0xb9)?;
    write_direct_bytes(&mut patched, 0xdf107, &[0xa2, 0x03, 0x6b])?;

    for &addr in &[
        0x1de0e5, 0x6d0b6, 0x6d0c6, 0x1d8f29, 0x1ddbd3, 0x1df856, 0x1e88da, 0x06ed0b, 0x1dc812,
        0x9b46c, 0x9b478, 0x9b468, 0x9b46a, 0x9b474, 0x9b476, 0x9b60c, 0x8f708, 0x1dcdeb, 0x7b269,
    ] {
        patch_rom_bp_checked(&mut patched, addr)?;
    }

    checked_copy_within(&mut patched, 0x332b7, 0x332b8, 4)?;
    write_direct(&mut patched, 0x332b7, 0xfa)?;

    write_direct_bytes(&mut patched, 0x443fe, &[0x48, 0x06])?;
    write_direct_bytes(&mut patched, 0x44607, &[0x48, 0x06])?;
    write_direct_bytes(&mut patched, 0x49d0c, &[0xda, 0xfa])?;
    write_direct_bytes(&mut patched, 0x49d0f, &[0xda, 0xfa])?;
    write_direct(&mut patched, 0x888, 0x60)?;

    patch_rom_word_expected_checked(&mut patched, 0x2c7e5 + 1, 0x1df, 0x1cf)?;

    for &addr in &[0x9816c, 0xffdeb, 0xffdee, 0xffdf7, 0xffdfa] {
        patch_rom_byte_expected_checked(&mut patched, addr, 0xd2, 0xcf)?;
    }

    for &addr in &[
        0x1cfc6, 0x1d29d, 0x89794, 0x897a3, 0x8a0a1, 0x8edca, 0x99aa6,
    ] {
        patch_rom_word_expected_checked(&mut patched, addr + 1, 0x3b6, 0x728)?;
    }
    for &addr in &[0x89797, 0x897a6] {
        patch_rom_word_expected_checked(&mut patched, addr + 1, 0x3b7, 0x729)?;
    }
    for &addr in &[0x1cfd7, 0x1d2ae, 0x8a099, 0x8edc5, 0x99aa1] {
        patch_rom_word_expected_checked(&mut patched, addr + 1, 0x3ba, 0x732)?;
    }
    for &addr in &[0x1cfb2, 0x1d2ba, 0x8a0b7] {
        patch_rom_word_expected_checked(&mut patched, addr + 1, 0x3be, 0x73c)?;
    }
    for &addr in &[0x89fb9, 0x89fc0, 0x98157, 0x99c49] {
        patch_rom_word_expected_checked(&mut patched, addr + 1, 0x3c0, 0x741)?;
    }
    for &addr in &[
        0x89fc3, 0x89fc6, 0x8a0ae, 0x8ab7c, 0x8aba7, 0x8abb6, 0x8ae92, 0x8bae2, 0x8baff, 0x8f429,
        0x98148, 0x98e0a, 0x98ebc, 0x9920a, 0x9931e, 0x9987f, 0x99c44,
    ] {
        patch_rom_word_expected_checked(&mut patched, addr + 1, 0x3c2, 0x746)?;
    }
    for &addr in &[0x9816e, 0xffde0, 0xffde7] {
        patch_rom_word_expected_checked(&mut patched, addr + 1, 0x3e1, 0x74b)?;
    }

    patch_rom_word_expected_checked(&mut patched, 0xddfac + 1, 0xfa85, 0xfa70)?;
    patch_rom_word_expected_checked(&mut patched, 0xe589, 0xe772, 0xe852)?;
    patch_rom_array_checked(
        &mut patched,
        0xe852,
        &[0xc0, 0x0c, 0xb0, 0x02, 0xa0, 0x0c, 0x4c, 0x72, 0xe7],
    )?;

    Ok(patched)
}

fn lorom_index(addr: u32) -> usize {
    (((addr >> 16) << 15) | (addr & 0x7fff)) as usize
}

fn write_direct(rom: &mut [u8], offset: usize, value: u8) -> Result<(), OracleError> {
    let slot = rom
        .get_mut(offset)
        .ok_or_else(|| OracleError::PatchRom(format!("offset ${offset:06X} out of range")))?;
    *slot = value;
    Ok(())
}

fn write_direct_bytes(rom: &mut [u8], offset: usize, values: &[u8]) -> Result<(), OracleError> {
    let end = offset + values.len();
    let dst = rom.get_mut(offset..end).ok_or_else(|| {
        OracleError::PatchRom(format!("range ${offset:06X}..${end:06X} out of range"))
    })?;
    dst.copy_from_slice(values);
    Ok(())
}

fn checked_copy_within(
    rom: &mut [u8],
    src: usize,
    dst: usize,
    len: usize,
) -> Result<(), OracleError> {
    if src + len > rom.len() || dst + len > rom.len() {
        return Err(OracleError::PatchRom(format!(
            "memmove ${src:06X} -> ${dst:06X} len {len} out of range"
        )));
    }
    rom.copy_within(src..src + len, dst);
    Ok(())
}

fn patch_rom_bp_checked(rom: &mut [u8], addr: u32) -> Result<(), OracleError> {
    write_direct(rom, lorom_index(addr), 0)
}

fn patch_rom_byte_checked(rom: &mut [u8], addr: u32, value: u8) -> Result<(), OracleError> {
    write_direct(rom, lorom_index(addr), value)
}

fn patch_rom_byte_expected_checked(
    rom: &mut [u8],
    addr: u32,
    old_value: u8,
    value: u8,
) -> Result<(), OracleError> {
    let idx = lorom_index(addr);
    let actual = *rom
        .get(idx)
        .ok_or_else(|| OracleError::PatchRom(format!("offset ${idx:06X} out of range")))?;
    if actual != old_value {
        return Err(OracleError::PatchRom(format!(
            "byte at ${addr:06X} was ${actual:02X}, expected ${old_value:02X}"
        )));
    }
    patch_rom_byte_checked(rom, addr, value)
}

fn patch_rom_word_checked(rom: &mut [u8], addr: u32, value: u16) -> Result<(), OracleError> {
    let idx = lorom_index(addr);
    let end = idx + 2;
    let dst = rom
        .get_mut(idx..end)
        .ok_or_else(|| OracleError::PatchRom(format!("word at ${addr:06X} out of range")))?;
    dst.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

fn patch_rom_word_expected_checked(
    rom: &mut [u8],
    addr: u32,
    old_value: u16,
    value: u16,
) -> Result<(), OracleError> {
    let idx = lorom_index(addr);
    let end = idx + 2;
    let src = rom
        .get(idx..end)
        .ok_or_else(|| OracleError::PatchRom(format!("word at ${addr:06X} out of range")))?;
    let actual = read_le_u16(src, 0);
    if actual != old_value {
        return Err(OracleError::PatchRom(format!(
            "word at ${addr:06X} was ${actual:04X}, expected ${old_value:04X}"
        )));
    }
    patch_rom_word_checked(rom, addr, value)
}

fn patch_rom_array_checked(
    rom: &mut [u8],
    mut addr: u32,
    values: &[u8],
) -> Result<(), OracleError> {
    for &value in values {
        patch_rom_byte_checked(rom, addr, value)?;
        addr += 1;
    }
    Ok(())
}

fn hooked_function_rts_checked(calling_asm_from_c: &mut bool) {
    if *calling_asm_from_c {
        *calling_asm_from_c = false;
        return;
    }
    // C HookedFunctionRts expects this path to be reached only while returning
    // from an ASM call made by C.
    debug_assert!(false, "unexpected RTS hook");
}

fn run_orig_asm_code_one_loop_checked(snes: &mut Snes) -> Result<(), OracleError> {
    snes.cpu.a = 0;
    snes.cpu.x = 0;
    snes.cpu.y = 0;
    snes.cpu.e = false;
    snes.cpu.irq_wanted = false;
    snes.cpu.nmi_wanted = false;
    snes.cpu.waiting = false;
    snes.cpu.stopped = false;
    snes.cpu.unpack_flags(0x30);

    for loops in 0..MAX_ORIG_LOOP_OPCODES {
        let _ = cpu_run_opcode(snes);
        while snes.dma.dma_busy {
            snes.dma_do();
        }

        let pc = ((snes.cpu.k as u32) << 16) | snes.cpu.pc as u32;
        if pc == 0x008034 || (pc == 0x09f81d && loops >= 10) || pc == 0x008225 || pc == 0x0082d2 {
            return Ok(());
        }
    }

    let pc = ((snes.cpu.k as u32) << 16) | snes.cpu.pc as u32;
    Err(OracleError::StepLimitExceeded {
        pc,
        limit: MAX_ORIG_LOOP_OPCODES,
    })
}

fn compare_snapshots_eq(
    frame: u32,
    mine: Snapshot,
    theirs: Snapshot,
    previous: &Snapshot,
) -> ComparisonReport {
    compare_snapshots_eq_with_options(frame, mine, theirs, previous, false)
}

fn compare_snapshots_eq_with_graduated_semantics(
    frame: u32,
    mut mine: Snapshot,
    theirs: Snapshot,
    previous: &Snapshot,
) -> ComparisonReport {
    let semantic_report = compare_semantic_snapshots(frame, &mine.semantic, &theirs.semantic);
    if semantic_report.is_empty()
        && same_graduated_map16_load_semantics(&mine.semantic, &theirs.semantic)
    {
        copy_graduated_map16_load_bytes(&mut mine, &theirs);
    }
    compare_snapshots_eq_with_options(frame, mine, theirs, previous, false)
}

fn same_graduated_map16_load_semantics(mine: &SemanticSnapshot, theirs: &SemanticSnapshot) -> bool {
    mine.world.map16_load_src == theirs.world.map16_load_src
        && mine.world.map16_load_dst == theirs.world.map16_load_dst
        && mine.world.map16_load_y_unit == theirs.world.map16_load_y_unit
}

fn copy_graduated_map16_load_bytes(mine: &mut Snapshot, theirs: &Snapshot) {
    for offset in 0x84..=0x89 {
        mine.ram[offset] = theirs.ram[offset];
    }
}

fn semantic_snapshot_from_parts(ram: &[u8], ppu_regs: &[u8]) -> SemanticSnapshot {
    let frame = FrameState::load_from_ram(ram);
    let display = DisplayState::load_from_ram(ram);
    let player = PlayerSnapshotState::load_from_ram(ram);
    let world_location = WorldLocationState::load_from_ram(ram);
    let world = WorldState::load_from_ram(ram);
    let map16_load = OverworldMap16LoadState::load_from_ram(ram);
    SemanticSnapshot {
        frame: SemanticFrame {
            main_module: frame.main_module,
            submodule: frame.submodule,
            subsubmodule: frame.subsubmodule,
            nmi_thread_active: display.nmi_thread_active,
            selected_run_thread: if display.nmi_thread_uses_poly_stack() {
                RUN_POLY_THREAD
            } else {
                RUN_MAIN_THREAD
            },
        },
        player: SemanticPlayer {
            x: player.x,
            y: player.y,
            z: player.z,
            x_velocity: player.x_velocity,
            y_velocity: player.y_velocity,
            z_velocity: player.z_velocity,
            direction: player.direction,
            last_direction: player.last_direction,
            facing: player.facing,
            handler_state: player.handler_state,
            auxiliary_state: player.auxiliary_state,
            current_health: player.current_health,
            magic_power: player.magic_power,
            equipped_item: player.equipped_item,
            item_in_hand: player.item_in_hand,
            current_item_y: player.current_item_y,
            current_item_active: player.current_item_active,
        },
        world: SemanticWorld {
            dungeon_room: world_location.dungeon_room(),
            overworld_screen: world_location.overworld_screen_index(),
            overworld_area: world.region.overworld_area(),
            transition_direction: world.overworld.transition.direction_enum(),
            overlay_index: world.region.overlay_index(),
            map16_load_src: map16_load.src_off,
            map16_load_dst: map16_load.dst_off,
            map16_load_y_unit: map16_load.y_unit,
            bg1_x: read_le_u16(ram, crate::game_state::constants::BG1_X_SCROLL),
            bg1_y: read_le_u16(ram, crate::game_state::constants::BG1_Y_SCROLL),
            bg2_x: read_le_u16(ram, crate::game_state::constants::BG2_X_SCROLL),
            bg2_y: read_le_u16(ram, crate::game_state::constants::BG2_Y_SCROLL),
            camera_x: world.camera_boundaries.camera_x_coord_scroll_low(),
            camera_y: world.camera_boundaries.camera_y_coord_scroll_low(),
            rng_seed: world.region.rng_seed(),
        },
        ppu: SemanticPpu {
            screen_enabled: ppu_array::<4>(ppu_regs, 4),
            screen_windowed: ppu_array::<4>(ppu_regs, 8),
            mosaic_enabled: ppu_byte(ppu_regs, 12),
            mosaic_size: ppu_byte(ppu_regs, 13),
            math_enabled: ppu_byte(ppu_regs, 30),
            fixed_color: ppu_array::<3>(ppu_regs, 31),
            forced_blank: ppu_byte(ppu_regs, 34) != 0,
            brightness: ppu_byte(ppu_regs, 35),
            mode: ppu_byte(ppu_regs, 36),
        },
        sprites: semantic_sprite_slots(ram),
        ancillas: semantic_ancilla_slots(ram),
    }
}

fn semantic_snapshot_from_game(game: &ZeldaState, ram: &[u8], ppu_regs: &[u8]) -> SemanticSnapshot {
    let mut snapshot = semantic_snapshot_from_parts(ram, ppu_regs);
    snapshot.frame = semantic_frame_from_native_game(game);
    snapshot.world = semantic_world_from_native_game(game);
    snapshot
}

fn semantic_frame_from_native_game(game: &ZeldaState) -> SemanticFrame {
    let frame = &game.game_state.frame;
    let display = &game.game_state.display;
    SemanticFrame {
        main_module: frame.main_module,
        submodule: frame.submodule,
        subsubmodule: frame.subsubmodule,
        nmi_thread_active: display.nmi_thread_active,
        selected_run_thread: if display.nmi_thread_uses_poly_stack() {
            RUN_POLY_THREAD
        } else {
            RUN_MAIN_THREAD
        },
    }
}

fn semantic_world_from_native_game(game: &ZeldaState) -> SemanticWorld {
    let location = &game.game_state.world.location;
    let region = &game.game_state.world.region;
    let map16 = game.game_state.world.overworld.map16.active_load;
    SemanticWorld {
        dungeon_room: location.dungeon_room(),
        overworld_screen: location.overworld_screen_index(),
        overworld_area: region.overworld_area(),
        transition_direction: game.transition_direction_enum(),
        overlay_index: region.overlay_index(),
        map16_load_src: map16.src_off,
        map16_load_dst: map16.dst_off,
        map16_load_y_unit: map16.y_unit,
        bg1_x: game.game_state.display.ppu_scroll_copy.bg1_h_copy2(),
        bg1_y: game.game_state.display.ppu_scroll_copy.bg1_v_copy2(),
        bg2_x: game.game_state.display.ppu_scroll_copy.bg2_h_copy2(),
        bg2_y: game.game_state.display.ppu_scroll_copy.bg2_v_copy2(),
        camera_x: game
            .game_state
            .world
            .camera_boundaries
            .camera_x_coord_scroll_low(),
        camera_y: game
            .game_state
            .world
            .camera_boundaries
            .camera_y_coord_scroll_low(),
        rng_seed: region.rng_seed(),
    }
}

fn compare_semantic_snapshots(
    frame: u32,
    mine: &SemanticSnapshot,
    theirs: &SemanticSnapshot,
) -> SemanticComparisonReport {
    let mut report = SemanticComparisonReport {
        frame,
        differences: Vec::new(),
    };
    push_semantic_diff(
        &mut report,
        "frame.main_module",
        &mine.frame.main_module,
        &theirs.frame.main_module,
    );
    push_semantic_diff(
        &mut report,
        "frame.submodule",
        &mine.frame.submodule,
        &theirs.frame.submodule,
    );
    push_semantic_diff(
        &mut report,
        "frame.subsubmodule",
        &mine.frame.subsubmodule,
        &theirs.frame.subsubmodule,
    );
    push_semantic_diff(
        &mut report,
        "frame.nmi_thread_active",
        &mine.frame.nmi_thread_active,
        &theirs.frame.nmi_thread_active,
    );
    push_semantic_diff(
        &mut report,
        "frame.selected_run_thread",
        &mine.frame.selected_run_thread,
        &theirs.frame.selected_run_thread,
    );
    push_semantic_diff(&mut report, "player.x", &mine.player.x, &theirs.player.x);
    push_semantic_diff(&mut report, "player.y", &mine.player.y, &theirs.player.y);
    push_semantic_diff(&mut report, "player.z", &mine.player.z, &theirs.player.z);
    push_semantic_diff(
        &mut report,
        "player.x_velocity",
        &mine.player.x_velocity,
        &theirs.player.x_velocity,
    );
    push_semantic_diff(
        &mut report,
        "player.y_velocity",
        &mine.player.y_velocity,
        &theirs.player.y_velocity,
    );
    push_semantic_diff(
        &mut report,
        "player.z_velocity",
        &mine.player.z_velocity,
        &theirs.player.z_velocity,
    );
    push_semantic_diff(
        &mut report,
        "player.direction",
        &mine.player.direction,
        &theirs.player.direction,
    );
    push_semantic_diff(
        &mut report,
        "player.last_direction",
        &mine.player.last_direction,
        &theirs.player.last_direction,
    );
    push_semantic_diff(
        &mut report,
        "player.facing",
        &mine.player.facing,
        &theirs.player.facing,
    );
    push_semantic_diff(
        &mut report,
        "player.handler_state",
        &mine.player.handler_state,
        &theirs.player.handler_state,
    );
    push_semantic_diff(
        &mut report,
        "player.auxiliary_state",
        &mine.player.auxiliary_state,
        &theirs.player.auxiliary_state,
    );
    push_semantic_diff(
        &mut report,
        "player.current_health",
        &mine.player.current_health,
        &theirs.player.current_health,
    );
    push_semantic_diff(
        &mut report,
        "player.magic_power",
        &mine.player.magic_power,
        &theirs.player.magic_power,
    );
    push_semantic_diff(
        &mut report,
        "player.equipped_item",
        &mine.player.equipped_item,
        &theirs.player.equipped_item,
    );
    push_semantic_diff(
        &mut report,
        "player.item_in_hand",
        &mine.player.item_in_hand,
        &theirs.player.item_in_hand,
    );
    push_semantic_diff(
        &mut report,
        "player.current_item_y",
        &mine.player.current_item_y,
        &theirs.player.current_item_y,
    );
    push_semantic_diff(
        &mut report,
        "player.current_item_active",
        &mine.player.current_item_active,
        &theirs.player.current_item_active,
    );
    push_semantic_diff(
        &mut report,
        "world.dungeon_room",
        &mine.world.dungeon_room,
        &theirs.world.dungeon_room,
    );
    push_semantic_diff(
        &mut report,
        "world.overworld_screen",
        &mine.world.overworld_screen,
        &theirs.world.overworld_screen,
    );
    push_semantic_diff(
        &mut report,
        "world.overworld_area",
        &mine.world.overworld_area,
        &theirs.world.overworld_area,
    );
    push_semantic_diff(
        &mut report,
        "world.transition_direction",
        &mine.world.transition_direction,
        &theirs.world.transition_direction,
    );
    push_semantic_diff(
        &mut report,
        "world.overlay_index",
        &mine.world.overlay_index,
        &theirs.world.overlay_index,
    );
    push_semantic_diff(
        &mut report,
        "world.map16_load_src",
        &mine.world.map16_load_src,
        &theirs.world.map16_load_src,
    );
    push_semantic_diff(
        &mut report,
        "world.map16_load_dst",
        &mine.world.map16_load_dst,
        &theirs.world.map16_load_dst,
    );
    push_semantic_diff(
        &mut report,
        "world.map16_load_y_unit",
        &mine.world.map16_load_y_unit,
        &theirs.world.map16_load_y_unit,
    );
    push_semantic_diff(
        &mut report,
        "world.bg1_x",
        &mine.world.bg1_x,
        &theirs.world.bg1_x,
    );
    push_semantic_diff(
        &mut report,
        "world.bg1_y",
        &mine.world.bg1_y,
        &theirs.world.bg1_y,
    );
    push_semantic_diff(
        &mut report,
        "world.bg2_x",
        &mine.world.bg2_x,
        &theirs.world.bg2_x,
    );
    push_semantic_diff(
        &mut report,
        "world.bg2_y",
        &mine.world.bg2_y,
        &theirs.world.bg2_y,
    );
    push_semantic_diff(
        &mut report,
        "world.camera_x",
        &mine.world.camera_x,
        &theirs.world.camera_x,
    );
    push_semantic_diff(
        &mut report,
        "world.camera_y",
        &mine.world.camera_y,
        &theirs.world.camera_y,
    );
    push_semantic_diff(
        &mut report,
        "world.rng_seed",
        &mine.world.rng_seed,
        &theirs.world.rng_seed,
    );
    push_semantic_diff(
        &mut report,
        "ppu.screen_enabled",
        &mine.ppu.screen_enabled,
        &theirs.ppu.screen_enabled,
    );
    push_semantic_diff(
        &mut report,
        "ppu.screen_windowed",
        &mine.ppu.screen_windowed,
        &theirs.ppu.screen_windowed,
    );
    push_semantic_diff(
        &mut report,
        "ppu.mosaic_enabled",
        &mine.ppu.mosaic_enabled,
        &theirs.ppu.mosaic_enabled,
    );
    push_semantic_diff(
        &mut report,
        "ppu.mosaic_size",
        &mine.ppu.mosaic_size,
        &theirs.ppu.mosaic_size,
    );
    push_semantic_diff(
        &mut report,
        "ppu.math_enabled",
        &mine.ppu.math_enabled,
        &theirs.ppu.math_enabled,
    );
    push_semantic_diff(
        &mut report,
        "ppu.fixed_color",
        &mine.ppu.fixed_color,
        &theirs.ppu.fixed_color,
    );
    push_semantic_diff(
        &mut report,
        "ppu.forced_blank",
        &mine.ppu.forced_blank,
        &theirs.ppu.forced_blank,
    );
    push_semantic_diff(
        &mut report,
        "ppu.brightness",
        &mine.ppu.brightness,
        &theirs.ppu.brightness,
    );
    push_semantic_diff(&mut report, "ppu.mode", &mine.ppu.mode, &theirs.ppu.mode);
    push_semantic_diff(&mut report, "sprites", &mine.sprites, &theirs.sprites);
    push_semantic_diff(&mut report, "ancillas", &mine.ancillas, &theirs.ancillas);
    report
}

fn push_semantic_diff<T: fmt::Debug + PartialEq>(
    report: &mut SemanticComparisonReport,
    field: &'static str,
    mine: &T,
    theirs: &T,
) {
    if mine != theirs {
        report.differences.push(SemanticDifference {
            field: field.to_string(),
            mine: format!("{mine:?}"),
            theirs: format!("{theirs:?}"),
        });
    }
}

fn semantic_sprite_slots(ram: &[u8]) -> Vec<SemanticSpriteSlot> {
    let mut slots = Vec::new();
    for slot in 0..16 {
        let slot = SpriteSlotSnapshot::load_from_ram(ram, slot);
        if !slot.is_active() {
            continue;
        }
        slots.push(SemanticSpriteSlot {
            slot: slot.slot,
            sprite_type: slot.sprite_type,
            state: slot.state,
            x: slot.x,
            y: slot.y,
            x_velocity: slot.x_velocity,
            y_velocity: slot.y_velocity,
            ai_state: slot.ai_state,
            delay_main: slot.delay_main,
            health: slot.health,
            hit_timer: slot.hit_timer,
        });
    }
    slots
}

fn semantic_ancilla_slots(ram: &[u8]) -> Vec<SemanticAncillaSlot> {
    let mut slots = Vec::new();
    for slot in 0..10 {
        let slot = AncillaSlotSnapshot::load_from_ram(ram, slot);
        if !slot.is_active() {
            continue;
        }
        slots.push(SemanticAncillaSlot {
            slot: slot.slot,
            ancilla_type: slot.ancilla_type,
            x: slot.x,
            y: slot.y,
            x_velocity: slot.x_velocity,
            y_velocity: slot.y_velocity,
            item_to_link: slot.item_to_link,
            timer: slot.timer,
            direction: slot.direction,
        });
    }
    slots
}

fn ppu_byte(ppu_regs: &[u8], offset: usize) -> u8 {
    ppu_regs.get(offset).copied().unwrap_or(0)
}

fn ppu_array<const N: usize>(ppu_regs: &[u8], offset: usize) -> [u8; N] {
    let mut out = [0; N];
    let Some(src) = ppu_regs.get(offset..offset + N) else {
        return out;
    };
    out.copy_from_slice(src);
    out
}

fn compare_snapshots_eq_with_options(
    frame: u32,
    mut mine: Snapshot,
    mut theirs: Snapshot,
    previous: &Snapshot,
    skip_ppu_regs: bool,
) -> ComparisonReport {
    normalize_snapshots(&mut mine, &mut theirs);

    let mut report = ComparisonReport {
        frame,
        differences: Vec::new(),
        total_wram: 0,
        total_sram: 0,
        total_vram: 0,
        total_cgram: 0,
        total_oam: 0,
        total_ppu_regs: 0,
    };

    compare_bytes(
        Region::Wram,
        &mine.ram,
        &theirs.ram,
        &previous.ram,
        128,
        &mut report.total_wram,
        &mut report.differences,
    );
    compare_bytes(
        Region::Sram,
        &mine.sram,
        &theirs.sram,
        &previous.sram,
        128,
        &mut report.total_sram,
        &mut report.differences,
    );
    compare_words(
        Region::Vram,
        &mine.vram,
        &theirs.vram,
        &previous.vram,
        16,
        &mut report.total_vram,
        &mut report.differences,
    );
    compare_words(
        Region::Cgram,
        &mine.cgram,
        &theirs.cgram,
        &previous.cgram,
        32,
        &mut report.total_cgram,
        &mut report.differences,
    );
    compare_words(
        Region::Oam,
        &mine.oam,
        &theirs.oam,
        &previous.oam,
        32,
        &mut report.total_oam,
        &mut report.differences,
    );
    if !skip_ppu_regs {
        compare_bytes(
            Region::PpuRegs,
            &mine.ppu_regs,
            &theirs.ppu_regs,
            &previous.ppu_regs,
            64,
            &mut report.total_ppu_regs,
            &mut report.differences,
        );
    }

    report
}

fn normalize_snapshots(mine: &mut Snapshot, theirs: &mut Snapshot) {
    mine.ram[..16].copy_from_slice(&theirs.ram[..16]);
    for &offset in &[
        0x0fa1, 0x0072, 0x0073, 0x0074, 0x0075, 0x00b7, 0x00b8, 0x00b9, 0x00ba, 0x00bb, 0x00bd,
        0x00be, 0x00c8, 0x00c9, 0x00ca, 0x00cb, 0x00cc, 0x00cd, 0x00a0, 0x0128, 0x0463,
    ] {
        mine.ram[offset] = theirs.ram[offset];
    }

    let word_1f0a = read_le_u16(&mine.ram, 0x1f0a);
    write_le_u16(&mut theirs.ram, 0x1f0a, word_1f0a);

    copy_between(&mut mine.ram, &theirs.ram, 0x1f0d, 0x1f0d, 0x3f - 0x0d);
    copy_between(&mut mine.ram, &theirs.ram, 0x0138, 0x0138, 256 - 0x38);

    copy_between(&mut theirs.ram, &mine.ram, 0x1cc0, 0x1cc0, 2);
    copy_between(&mut theirs.ram, &mine.ram, 0x1dd60, 0x1dd60, 16 * 2);
    copy_between(&mut theirs.ram, &mine.ram, 0x1db20, 0x1db20, 64 * 2);
    theirs.ram[0x0654] = mine.ram[0x0654];
    copy_between(&mut theirs.ram, &mine.ram, 0x1cdd, 0x1cdd, 2);
}

fn compare_bytes(
    region: Region,
    mine: &[u8],
    theirs: &[u8],
    previous: &[u8],
    max_reported: usize,
    total: &mut usize,
    differences: &mut Vec<Difference>,
) {
    let mut i = 0;
    while i < mine.len() {
        if mine[i] != theirs[i] {
            *total += 1;
            if differences.len() < max_reported {
                if i & 1 == 0 && i + 1 < mine.len() && mine[i + 1] != theirs[i + 1] {
                    differences.push(Difference {
                        region,
                        offset: i,
                        mine: read_le_u16(mine, i),
                        theirs: read_le_u16(theirs, i),
                        previous: read_le_u16_padded(previous, i),
                        width: 2,
                    });
                    *total += 1;
                    i += 2;
                    continue;
                }
                differences.push(Difference {
                    region,
                    offset: i,
                    mine: mine[i] as u16,
                    theirs: theirs[i] as u16,
                    previous: previous.get(i).copied().unwrap_or(0) as u16,
                    width: 1,
                });
            }
        }
        i += 1;
    }
}

fn read_le_u16_padded(bytes: &[u8], offset: usize) -> u16 {
    let lo = bytes.get(offset).copied().unwrap_or(0) as u16;
    let hi = bytes.get(offset + 1).copied().unwrap_or(0) as u16;
    lo | (hi << 8)
}

fn compare_words(
    region: Region,
    mine: &[u16],
    theirs: &[u16],
    previous: &[u16],
    max_reported: usize,
    total: &mut usize,
    differences: &mut Vec<Difference>,
) {
    for (i, ((&mine_word, &theirs_word), &previous_word)) in
        mine.iter().zip(theirs).zip(previous).enumerate()
    {
        if mine_word != theirs_word {
            *total += 1;
            if differences.len() < max_reported {
                differences.push(Difference {
                    region,
                    offset: i,
                    mine: mine_word,
                    theirs: theirs_word,
                    previous: previous_word,
                    width: 2,
                });
            }
        }
    }
}

fn copy_within_clone(bytes: &mut [u8], src: usize, dst: usize, len: usize) {
    let tmp = bytes[src..src + len].to_vec();
    bytes[dst..dst + len].copy_from_slice(&tmp);
}

fn copy_between(dst: &mut [u8], src: &[u8], dst_start: usize, src_start: usize, len: usize) {
    dst[dst_start..dst_start + len].copy_from_slice(&src[src_start..src_start + len]);
}

#[cfg(test)]
#[path = "zelda_cpu_infra_tests.rs"]
mod tests;
