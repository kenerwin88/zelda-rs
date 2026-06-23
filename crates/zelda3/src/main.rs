//! Helpers whose C source lives in `src/main.c`.

#![allow(non_camel_case_types)]

use std::ffi::CStr;
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use snes::{consts::PPU_EXTRA_LEFT_RIGHT, load_rom as snes_load_rom, ppu::PpuRenderFlags, Snes};

use crate::config::{
    config_value_path, parse_config_file_context, ConfigContext, GAMEPAD_BUTTON_A,
    GAMEPAD_BUTTON_B, GAMEPAD_BUTTON_BACK, GAMEPAD_BUTTON_COUNT, GAMEPAD_BUTTON_DPAD_DOWN,
    GAMEPAD_BUTTON_DPAD_LEFT, GAMEPAD_BUTTON_DPAD_RIGHT, GAMEPAD_BUTTON_DPAD_UP,
    GAMEPAD_BUTTON_GUIDE, GAMEPAD_BUTTON_L1, GAMEPAD_BUTTON_L2, GAMEPAD_BUTTON_L3,
    GAMEPAD_BUTTON_R1, GAMEPAD_BUTTON_R2, GAMEPAD_BUTTON_R3, GAMEPAD_BUTTON_START,
    GAMEPAD_BUTTON_X, GAMEPAD_BUTTON_Y, KEY_COMMAND_CLEAR_KEY_LOG, KEY_COMMAND_CONTROLS_LAST,
    KEY_COMMAND_DISPLAY_PERF, KEY_COMMAND_FULLSCREEN, KEY_COMMAND_LOAD_LAST,
    KEY_COMMAND_LOAD_REF_LAST, KEY_COMMAND_PAUSE, KEY_COMMAND_PAUSE_DIMMED,
    KEY_COMMAND_REPLAY_LAST, KEY_COMMAND_REPLAY_REF_LAST, KEY_COMMAND_REPLAY_TURBO,
    KEY_COMMAND_RESET, KEY_COMMAND_SAVE_LAST, KEY_COMMAND_STOP_REPLAY, KEY_COMMAND_TOGGLE_RENDERER,
    KEY_COMMAND_TURBO, KEY_COMMAND_VOLUME_DOWN, KEY_COMMAND_VOLUME_UP, KEY_COMMAND_WINDOW_BIGGER,
    KEY_COMMAND_WINDOW_SMALLER,
};
use crate::types::MemBlk;
use crate::util::{find_index_in_memblk, ApplyBps};
use crate::zelda_cpu_infra::{patch_rom_owned, LockstepOracle, OracleError};
use crate::zelda_rtl::ZeldaState;

const MAX_WINDOW_SCALE: i32 = 10;
const SDL_MIX_MAXVOLUME: i32 = 128;
const SDL_CONTROLLER_BUTTON_A: i32 = 0;
const SDL_CONTROLLER_BUTTON_B: i32 = 1;
const SDL_CONTROLLER_BUTTON_X: i32 = 2;
const SDL_CONTROLLER_BUTTON_Y: i32 = 3;
const SDL_CONTROLLER_BUTTON_BACK: i32 = 4;
const SDL_CONTROLLER_BUTTON_GUIDE: i32 = 5;
const SDL_CONTROLLER_BUTTON_START: i32 = 6;
const SDL_CONTROLLER_BUTTON_LEFTSTICK: i32 = 7;
const SDL_CONTROLLER_BUTTON_RIGHTSTICK: i32 = 8;
const SDL_CONTROLLER_BUTTON_LEFTSHOULDER: i32 = 9;
const SDL_CONTROLLER_BUTTON_RIGHTSHOULDER: i32 = 10;
const SDL_CONTROLLER_BUTTON_DPAD_UP: i32 = 11;
const SDL_CONTROLLER_BUTTON_DPAD_DOWN: i32 = 12;
const SDL_CONTROLLER_BUTTON_DPAD_LEFT: i32 = 13;
const SDL_CONTROLLER_BUTTON_DPAD_RIGHT: i32 = 14;
const SDL_CONTROLLER_AXIS_LEFTX: i32 = 0;
const SDL_CONTROLLER_AXIS_LEFTY: i32 = 1;
const SDL_CONTROLLER_AXIS_TRIGGERLEFT: i32 = 4;
const SDL_CONTROLLER_AXIS_TRIGGERRIGHT: i32 = 5;
const DEFAULT_AUDIO_FREQUENCY: i32 = 44100;
const DEFAULT_AUDIO_CHANNELS: i32 = 2;
const FEATURE_DIM_FLASHES: u32 = 65536;
const WINDOW_RESIZE_BORDER: i32 = 20;
const KEYBOARD_COMMAND_BUTTON_REMAP: [u8; 13] = [0, 4, 5, 6, 7, 2, 3, 8, 0, 9, 1, 10, 11];
const GAMEPAD_STICK_SEGMENT_BUTTONS: [i32; 8] = [
    1 << 4,
    1 << 4 | 1 << 7,
    1 << 7,
    1 << 7 | 1 << 5,
    1 << 5,
    1 << 5 | 1 << 6,
    1 << 6,
    1 << 6 | 1 << 4,
];
const NUMBER_OF_ASSETS: usize = 165;
const ASSETS_SIGNATURE: [u8; 48] = [
    90, 101, 108, 100, 97, 51, 95, 118, 48, 32, 32, 32, 32, 32, 10, 0, 27, 174, 233, 45, 74, 174,
    252, 50, 49, 27, 153, 197, 27, 43, 216, 197, 132, 101, 173, 169, 36, 108, 15, 155, 176, 169,
    57, 131, 174, 101, 51, 207,
];

pub enum SDL_Window {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SDL_Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SDL_HitTestResult {
    SDL_HITTEST_NORMAL = 0,
    SDL_HITTEST_DRAGGABLE = 1,
    SDL_HITTEST_RESIZE_TOPLEFT = 2,
    SDL_HITTEST_RESIZE_TOP = 3,
    SDL_HITTEST_RESIZE_TOPRIGHT = 4,
    SDL_HITTEST_RESIZE_RIGHT = 5,
    SDL_HITTEST_RESIZE_BOTTOMRIGHT = 6,
    SDL_HITTEST_RESIZE_BOTTOM = 7,
    SDL_HITTEST_RESIZE_BOTTOMLEFT = 8,
    SDL_HITTEST_RESIZE_LEFT = 9,
}

#[derive(Clone, Debug)]
struct MainState {
    paused: bool,
    turbo: bool,
    replay_turbo: bool,
    cursor: bool,
    current_window_scale: i32,
    gamepad_buttons: i32,
    input1_state: i32,
    display_perf: bool,
    curr_fps: i32,
    perf_history: [f32; 64],
    perf_history_pos: usize,
    perf_average: f32,
    ppu_render_flags: i32,
    snes_width: i32,
    snes_height: i32,
    sdl_audio_mixer_volume: i32,
    gamepad_modifiers: u32,
    gamepad_last_cmd: [u16; GAMEPAD_BUTTON_COUNT],
    video_buffer: Vec<u8>,
    audio_buffer: Vec<u8>,
    audio_buffer_cur: usize,
    audio_buffer_end: usize,
    frames_per_block: i32,
    audio_channels: u8,
    axis_last_gamepad_id: i32,
    axis_last_x: i32,
    axis_last_y: i32,
    assets: Vec<Vec<u8>>,
    config_context: ConfigContext,
}

impl Default for MainState {
    fn default() -> Self {
        Self {
            paused: false,
            turbo: false,
            replay_turbo: true,
            cursor: true,
            current_window_scale: 2,
            gamepad_buttons: 0,
            input1_state: 0,
            display_perf: false,
            curr_fps: 0,
            perf_history: [0.0; 64],
            perf_history_pos: 0,
            perf_average: 0.0,
            ppu_render_flags: 0,
            snes_width: 256,
            snes_height: 224,
            sdl_audio_mixer_volume: SDL_MIX_MAXVOLUME,
            gamepad_modifiers: 0,
            gamepad_last_cmd: [0; GAMEPAD_BUTTON_COUNT],
            video_buffer: Vec::new(),
            audio_buffer: Vec::new(),
            audio_buffer_cur: 0,
            audio_buffer_end: 0,
            frames_per_block: 0,
            audio_channels: 2,
            axis_last_gamepad_id: 0,
            axis_last_x: 0,
            axis_last_y: 0,
            assets: Vec::new(),
            config_context: parse_config_file_context(None),
        }
    }
}

fn state() -> &'static Mutex<MainState> {
    static STATE: OnceLock<Mutex<MainState>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(MainState::default()))
}

pub fn die(error: *const i8) -> ! {
    let msg = cstr_to_string(error).unwrap_or_else(|| "unknown error".to_string());
    eprintln!("Error: {msg}");
    std::process::exit(1);
}

pub fn change_window_scale(scale_step: i32) {
    let mut s = state().lock().unwrap();
    let new_scale = (s.current_window_scale + scale_step).clamp(1, MAX_WINDOW_SCALE);
    s.current_window_scale = new_scale;
    s.snes_width = s.snes_width.max(1);
    s.snes_height = s.snes_height.max(1);
}

pub fn hit_test_callback(
    _win: *mut SDL_Window,
    pt: *const SDL_Point,
    _data: *mut std::ffi::c_void,
) -> SDL_HitTestResult {
    let s = state().lock().unwrap();
    let point = unsafe { pt.as_ref().copied().unwrap_or_default() };
    let w = s.snes_width * s.current_window_scale;
    let h = s.snes_height * s.current_window_scale;
    if point.y < WINDOW_RESIZE_BORDER {
        if point.x < WINDOW_RESIZE_BORDER {
            SDL_HitTestResult::SDL_HITTEST_RESIZE_TOPLEFT
        } else if point.x >= w - WINDOW_RESIZE_BORDER {
            SDL_HitTestResult::SDL_HITTEST_RESIZE_TOPRIGHT
        } else {
            SDL_HitTestResult::SDL_HITTEST_RESIZE_TOP
        }
    } else if point.y >= h - WINDOW_RESIZE_BORDER {
        if point.x < WINDOW_RESIZE_BORDER {
            SDL_HitTestResult::SDL_HITTEST_RESIZE_BOTTOMLEFT
        } else if point.x >= w - WINDOW_RESIZE_BORDER {
            SDL_HitTestResult::SDL_HITTEST_RESIZE_BOTTOMRIGHT
        } else {
            SDL_HitTestResult::SDL_HITTEST_RESIZE_BOTTOM
        }
    } else if point.x < WINDOW_RESIZE_BORDER {
        SDL_HitTestResult::SDL_HITTEST_RESIZE_LEFT
    } else if point.x >= w - WINDOW_RESIZE_BORDER {
        SDL_HitTestResult::SDL_HITTEST_RESIZE_RIGHT
    } else {
        SDL_HitTestResult::SDL_HITTEST_NORMAL
    }
}

pub fn draw_ppu_frame_with_perf(game: &mut ZeldaState) {
    let (snes_width, snes_height, display_perf, render_flags) = {
        let s = state().lock().unwrap();
        (
            s.snes_width,
            s.snes_height,
            s.display_perf,
            PpuRenderFlags::from_bits_truncate(s.ppu_render_flags as u32),
        )
    };

    let render_scale = game.ppu.current_render_scale(render_flags);
    let mut pixel_ptr: *mut u8 = std::ptr::null_mut();
    let mut pitch = 0i32;
    sdl_renderer_begin_draw(
        snes_width * render_scale,
        snes_height * render_scale,
        &mut pixel_ptr,
        &mut pitch,
    );

    if !pixel_ptr.is_null() && pitch > 0 {
        let pixel_len =
            (pitch as usize).saturating_mul((snes_height * render_scale).max(0) as usize);
        let pixels = unsafe { std::slice::from_raw_parts_mut(pixel_ptr, pixel_len) };
        let display_perf_title = {
            let s = state().lock().unwrap();
            s.config_context.config.display_perf_title
        };
        if display_perf || display_perf_title {
            let before = Instant::now();
            game.zelda_draw_display_frame(pixels, pitch as usize, render_flags);
            let elapsed = before.elapsed().as_secs_f32();
            let curr_fps = update_perf_counter(elapsed);
            unsafe {
                render_number(
                    pixel_ptr.add((pitch * render_scale) as usize),
                    pitch as usize,
                    curr_fps,
                    render_scale == 4,
                );
            }
        } else {
            game.zelda_draw_display_frame(pixels, pitch as usize, render_flags);
        }
    }
    sdl_renderer_end_draw();
}

fn update_perf_counter(elapsed_seconds: f32) -> i32 {
    let mut s = state().lock().unwrap();
    let v = if elapsed_seconds > 0.0 {
        1.0 / elapsed_seconds
    } else {
        0.0
    };
    let pos = s.perf_history_pos;
    s.perf_average += v - s.perf_history[pos];
    s.perf_history[pos] = v;
    s.perf_history_pos = (pos + 1) & 63;
    s.curr_fps = (s.perf_average * (1.0 / 64.0)) as i32;
    s.curr_fps
}

pub fn audio_callback(_userdata: *mut std::ffi::c_void, stream: *mut u8, len: i32) {
    if len <= 0 {
        return;
    }
    if !stream.is_null() {
        unsafe { std::ptr::write_bytes(stream, 0, len as usize) };
    }
}

pub fn audio_callback_for_game(game: &mut ZeldaState, stream: *mut u8, len: i32) {
    if len <= 0 {
        return;
    }
    let mut s = state().lock().unwrap();
    let mut remaining = len as usize;
    let mut out = stream;
    while remaining != 0 {
        if s.audio_buffer_end == s.audio_buffer_cur {
            let channels = s.audio_channels.max(1) as usize;
            let frames = s.frames_per_block.max(1) as usize;
            let sample_count = frames.saturating_mul(channels);
            let mut samples = vec![0i16; sample_count];
            game.zelda_render_audio(&mut samples, frames as i32, channels as i32);
            s.audio_buffer.resize(sample_count * 2, 0);
            for (chunk, sample) in s.audio_buffer.chunks_exact_mut(2).zip(samples) {
                chunk.copy_from_slice(&sample.to_le_bytes());
            }
            s.audio_buffer_cur = 0;
            s.audio_buffer_end = s.audio_buffer.len();
        }
        let available = s
            .audio_buffer_end
            .saturating_sub(s.audio_buffer_cur)
            .min(remaining);
        if available != 0 && !out.is_null() {
            unsafe {
                copy_audio_chunk(
                    out,
                    &s.audio_buffer[s.audio_buffer_cur..s.audio_buffer_cur + available],
                    s.sdl_audio_mixer_volume,
                );
                out = out.add(available);
            }
        }
        s.audio_buffer_cur += available;
        remaining -= available;
    }
    drop(s);
    game.zelda_discard_unused_audio_frames();
}

unsafe fn copy_audio_chunk(dst: *mut u8, src: &[u8], volume: i32) {
    unsafe {
        if volume >= SDL_MIX_MAXVOLUME {
            std::ptr::copy_nonoverlapping(src.as_ptr(), dst, src.len());
            return;
        }
        std::ptr::write_bytes(dst, 0, src.len());
        let volume = volume.clamp(0, SDL_MIX_MAXVOLUME);
        for (i, sample) in src.chunks_exact(2).enumerate() {
            let value = i16::from_le_bytes([sample[0], sample[1]]) as i32;
            let mixed =
                (value * volume / SDL_MIX_MAXVOLUME).clamp(i16::MIN as i32, i16::MAX as i32);
            let bytes = (mixed as i16).to_le_bytes();
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), dst.add(i * 2), 2);
        }
        if src.len() & 1 != 0 {
            *dst.add(src.len() - 1) = src[src.len() - 1];
        }
    }
}

pub fn configure_runtime_from_config(game: &mut ZeldaState) {
    let mut s = state().lock().unwrap();
    let cfg = s.config_context.config.clone();
    game.ppu.extra_left_right =
        (cfg.extended_aspect_ratio as usize).min(PPU_EXTRA_LEFT_RIGHT) as u8;
    game.wanted_zelda_features = cfg.features0;
    s.snes_width = cfg.extended_aspect_ratio as i32 * 2 + 256;
    s.snes_height = if cfg.extend_y { 240 } else { 224 };
    s.ppu_render_flags = 0;
    if cfg.new_renderer {
        s.ppu_render_flags |= PpuRenderFlags::NEW_RENDERER.bits() as i32;
    }
    if cfg.enhanced_mode7 {
        s.ppu_render_flags |= PpuRenderFlags::MODE7_4X4.bits() as i32;
    }
    if cfg.extend_y {
        s.ppu_render_flags |= PpuRenderFlags::HEIGHT_240.bits() as i32;
    }
    if cfg.no_sprite_limits {
        s.ppu_render_flags |= PpuRenderFlags::NO_SPRITE_LIMITS.bits() as i32;
    }
    s.current_window_scale = if cfg.window_scale == 0 {
        2
    } else {
        (cfg.window_scale as i32).min(MAX_WINDOW_SCALE)
    };
    let mut audio_freq = cfg.audio_freq as i32;
    if !(11025..=48000).contains(&audio_freq) {
        audio_freq = DEFAULT_AUDIO_FREQUENCY;
    }
    let mut audio_channels = cfg.audio_channels as i32;
    if !(1..=2).contains(&audio_channels) {
        audio_channels = DEFAULT_AUDIO_CHANNELS;
    }
    s.audio_channels = audio_channels as u8;
    s.frames_per_block = (534 * audio_freq) / 32000;
    let audio_buffer_len = (s.frames_per_block as usize)
        .saturating_mul(s.audio_channels as usize)
        .saturating_mul(2);
    s.audio_buffer.resize(audio_buffer_len, 0);
    s.audio_buffer_cur = 0;
    s.audio_buffer_end = 0;
    game.zelda_configure_audio(
        audio_freq as u32,
        cfg.msuvolume,
        cfg.resume_msu,
        cfg.msu_path.clone(),
    );
    drop(s);
    game.zelda_enable_msu(cfg.enable_msu);
    game.zelda_set_language(cfg.language.as_deref());
}

fn apply_feature_asset_patches() {
    let mut s = state().lock().unwrap();
    if s.config_context.config.features0 & FEATURE_DIM_FLASHES == 0 {
        return;
    }
    if let Some(asset) = s.assets.get_mut(79) {
        write_asset_u16(asset, 0x484, 0x70);
        write_asset_u16(asset, 0x485, 0x95);
        write_asset_u16(asset, 0x486, 0x57);
    }
}

fn write_asset_u16(asset: &mut [u8], index: usize, value: u16) {
    let offset = index.saturating_mul(2);
    if let Some(slot) = asset.get_mut(offset..offset + 2) {
        slot.copy_from_slice(&value.to_le_bytes());
    }
}

fn reset_audio_buffer() {
    let mut s = state().lock().unwrap();
    let channels = s.audio_channels.max(1) as usize;
    let frames = s.frames_per_block.max(1) as usize;
    s.audio_buffer.resize(frames * channels * 2, 0);
    s.audio_buffer_cur = 0;
    s.audio_buffer_end = 0;
}

fn sdl_renderer_init(_window: *mut SDL_Window) -> bool {
    true
}

fn sdl_renderer_begin_draw(width: i32, height: i32, pixels: *mut *mut u8, pitch: *mut i32) {
    let mut s = state().lock().unwrap();
    let width = width.max(0) as usize;
    let height = height.max(0) as usize;
    let p = width.saturating_mul(4);
    s.video_buffer.resize(p.saturating_mul(height), 0);
    if !pixels.is_null() {
        unsafe { *pixels = s.video_buffer.as_mut_ptr() };
    }
    if !pitch.is_null() {
        unsafe { *pitch = p as i32 };
    }
}

fn sdl_renderer_end_draw() {}

pub fn main(argc: i32, argv: *mut *mut i8) -> i32 {
    let config_file = parse_main_config_file(argc, argv);
    if config_file.is_none() {
        switch_directory();
    }
    {
        let mut s = state().lock().unwrap();
        s.config_context = parse_config_file_context(config_file.as_deref());
    }
    load_assets();
    load_link_graphics();
    apply_feature_asset_patches();
    let mut game = ZeldaState::new();
    configure_runtime_from_config(&mut game);
    reset_audio_buffer();
    0
}

fn render_digit(dst: *mut u8, pitch: usize, digit: i32, color: u32, big: bool) {
    static FONT: [u8; 100] = [
        0x1c, 0x36, 0x63, 0x63, 0x63, 0x63, 0x63, 0x63, 0x36, 0x1c, 0x18, 0x1c, 0x1e, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x7e, 0x3e, 0x63, 0x60, 0x30, 0x18, 0x0c, 0x06, 0x03, 0x63, 0x7f,
        0x3e, 0x63, 0x60, 0x60, 0x3c, 0x60, 0x60, 0x60, 0x63, 0x3e, 0x30, 0x38, 0x3c, 0x36, 0x33,
        0x7f, 0x30, 0x30, 0x30, 0x78, 0x7f, 0x03, 0x03, 0x03, 0x3f, 0x60, 0x60, 0x60, 0x63, 0x3e,
        0x1c, 0x06, 0x03, 0x03, 0x3f, 0x63, 0x63, 0x63, 0x63, 0x3e, 0x7f, 0x63, 0x60, 0x60, 0x30,
        0x18, 0x0c, 0x0c, 0x0c, 0x0c, 0x3e, 0x63, 0x63, 0x63, 0x3e, 0x63, 0x63, 0x63, 0x63, 0x3e,
        0x3e, 0x63, 0x63, 0x63, 0x7e, 0x60, 0x60, 0x60, 0x30, 0x1e,
    ];
    if dst.is_null() || !(0..=9).contains(&digit) {
        return;
    }
    let p = &FONT[digit as usize * 10..digit as usize * 10 + 10];
    unsafe {
        for (y, mut v) in p.iter().copied().enumerate() {
            let row = dst.add(y * pitch * if big { 2 } else { 1 });
            let mut x = 0usize;
            while v != 0 {
                if v & 1 != 0 {
                    let pos = if big { x * 2 } else { x };
                    (row.add(pos * 4) as *mut u32).write_unaligned(color);
                    if big {
                        (row.add((pos + 1) * 4) as *mut u32).write_unaligned(color);
                        (row.add(pitch + pos * 4) as *mut u32).write_unaligned(color);
                        (row.add(pitch + (pos + 1) * 4) as *mut u32).write_unaligned(color);
                    }
                }
                x += 1;
                v >>= 1;
            }
        }
    }
}

fn render_number(dst: *mut u8, pitch: usize, n: i32, big: bool) {
    let text = n.to_string();
    for (i, ch) in text.bytes().enumerate() {
        if ch.is_ascii_digit() {
            let ofs = ((2 + i * 8) * 4) << usize::from(big);
            unsafe {
                render_digit(
                    dst.add((pitch + (2 + i * 8) * 4 + 4) << usize::from(big)),
                    pitch,
                    (ch - b'0') as i32,
                    0x404040,
                    big,
                );
                render_digit(dst.add(ofs), pitch, (ch - b'0') as i32, 0xffffff, big);
            }
        }
    }
}

fn handle_command(j: u32, pressed: bool) {
    let mut s = state().lock().unwrap();
    if j <= KEY_COMMAND_CONTROLS_LAST as u32 {
        let bit = 1 << KEYBOARD_COMMAND_BUTTON_REMAP[j as usize];
        if pressed {
            s.input1_state |= bit;
        } else {
            s.input1_state &= !bit;
        }
        return;
    }
    if j == KEY_COMMAND_TURBO as u32 {
        s.turbo = pressed;
        return;
    }
    drop(s);
    handle_command_locked(j, pressed);
}

fn handle_command_locked(j: u32, pressed: bool) {
    if !pressed {
        return;
    }
    let mut s = state().lock().unwrap();
    if j <= KEY_COMMAND_LOAD_LAST as u32 {
        return;
    } else if j <= KEY_COMMAND_SAVE_LAST as u32 {
        return;
    } else if j <= KEY_COMMAND_REPLAY_LAST as u32 {
        return;
    } else if j <= KEY_COMMAND_LOAD_REF_LAST as u32 {
        return;
    } else if j <= KEY_COMMAND_REPLAY_REF_LAST as u32 {
        return;
    }
    match j as u16 {
        KEY_COMMAND_CLEAR_KEY_LOG | KEY_COMMAND_STOP_REPLAY => {}
        KEY_COMMAND_FULLSCREEN => s.cursor = !s.cursor,
        KEY_COMMAND_RESET => {}
        KEY_COMMAND_PAUSE | KEY_COMMAND_PAUSE_DIMMED => s.paused = !s.paused,
        KEY_COMMAND_REPLAY_TURBO => s.replay_turbo = !s.replay_turbo,
        KEY_COMMAND_WINDOW_BIGGER => {
            drop(s);
            change_window_scale(1);
        }
        KEY_COMMAND_WINDOW_SMALLER => {
            drop(s);
            change_window_scale(-1);
        }
        KEY_COMMAND_DISPLAY_PERF => s.display_perf = !s.display_perf,
        KEY_COMMAND_TOGGLE_RENDERER => s.ppu_render_flags ^= 1,
        KEY_COMMAND_VOLUME_UP => {
            drop(s);
            handle_volume_adjustment(1);
        }
        KEY_COMMAND_VOLUME_DOWN => {
            drop(s);
            handle_volume_adjustment(-1);
        }
        _ => {}
    }
}

fn handle_input(key_code: i32, key_mod: i32, pressed: bool) {
    let cmd = {
        let s = state().lock().unwrap();
        s.config_context
            .find_cmd_for_sdl_key(key_code, key_mod as u16)
    };
    if cmd != 0 {
        handle_command(cmd as u32, pressed);
    }
}

fn remap_sdl_button(button: i32) -> i32 {
    match button {
        SDL_CONTROLLER_BUTTON_A => GAMEPAD_BUTTON_A as i32,
        SDL_CONTROLLER_BUTTON_B => GAMEPAD_BUTTON_B as i32,
        SDL_CONTROLLER_BUTTON_X => GAMEPAD_BUTTON_X as i32,
        SDL_CONTROLLER_BUTTON_Y => GAMEPAD_BUTTON_Y as i32,
        SDL_CONTROLLER_BUTTON_BACK => GAMEPAD_BUTTON_BACK as i32,
        SDL_CONTROLLER_BUTTON_GUIDE => GAMEPAD_BUTTON_GUIDE as i32,
        SDL_CONTROLLER_BUTTON_START => GAMEPAD_BUTTON_START as i32,
        SDL_CONTROLLER_BUTTON_LEFTSTICK => GAMEPAD_BUTTON_L3 as i32,
        SDL_CONTROLLER_BUTTON_RIGHTSTICK => GAMEPAD_BUTTON_R3 as i32,
        SDL_CONTROLLER_BUTTON_LEFTSHOULDER => GAMEPAD_BUTTON_L1 as i32,
        SDL_CONTROLLER_BUTTON_RIGHTSHOULDER => GAMEPAD_BUTTON_R1 as i32,
        SDL_CONTROLLER_BUTTON_DPAD_UP => GAMEPAD_BUTTON_DPAD_UP as i32,
        SDL_CONTROLLER_BUTTON_DPAD_DOWN => GAMEPAD_BUTTON_DPAD_DOWN as i32,
        SDL_CONTROLLER_BUTTON_DPAD_LEFT => GAMEPAD_BUTTON_DPAD_LEFT as i32,
        SDL_CONTROLLER_BUTTON_DPAD_RIGHT => GAMEPAD_BUTTON_DPAD_RIGHT as i32,
        _ => -1,
    }
}

fn handle_gamepad_input(button: i32, pressed: bool) {
    if button < 0 || button as usize >= GAMEPAD_BUTTON_COUNT {
        return;
    }
    let mut s = state().lock().unwrap();
    let bit = 1u32 << button;
    if (s.gamepad_modifiers & bit != 0) == pressed {
        return;
    }
    s.gamepad_modifiers ^= bit;
    if pressed {
        s.gamepad_last_cmd[button as usize] = s
            .config_context
            .find_cmd_for_gamepad_button(button as usize, s.gamepad_modifiers)
            as u16;
    }
    let cmd = s.gamepad_last_cmd[button as usize];
    drop(s);
    if cmd != 0 {
        handle_command(cmd as u32, pressed);
    }
}

fn handle_volume_adjustment(volume_adjustment: i32) {
    let mut s = state().lock().unwrap();
    s.sdl_audio_mixer_volume = (s.sdl_audio_mixer_volume
        + volume_adjustment * (SDL_MIX_MAXVOLUME >> 4))
        .clamp(0, SDL_MIX_MAXVOLUME);
    println!("[SDL mixer volume]={}", s.sdl_audio_mixer_volume);
}

fn approximate_atan2(y: f32, x: f32) -> f32 {
    let sign_mask = 0x80000000u32;
    let b = 0.596227f32;
    let ux_s = sign_mask & x.to_bits();
    let uy_s = sign_mask & y.to_bits();
    let q = ((ux_s ^ sign_mask) & uy_s) >> 29 | ux_s >> 30;
    let mut bxy_a = b * x * y;
    if bxy_a < 0.0 {
        bxy_a = -bxy_a;
    }
    let num = bxy_a + y * y;
    let atan_1q = num / (x * x + bxy_a + num + 0.000001);
    let uatan_2q = (ux_s ^ uy_s) | atan_1q.to_bits();
    q as f32 + f32::from_bits(uatan_2q)
}

fn handle_gamepad_axis_input(gamepad_id: i32, axis: i32, value: i32) {
    if axis == SDL_CONTROLLER_AXIS_LEFTX || axis == SDL_CONTROLLER_AXIS_LEFTY {
        let mut s = state().lock().unwrap();
        if s.axis_last_gamepad_id != gamepad_id {
            if value > -16000 && value < 16000 {
                return;
            }
            s.axis_last_gamepad_id = gamepad_id;
            s.axis_last_x = 0;
            s.axis_last_y = 0;
        }
        if axis == SDL_CONTROLLER_AXIS_LEFTX {
            s.axis_last_x = value;
        } else {
            s.axis_last_y = value;
        }
        let mut buttons = 0;
        if s.axis_last_x * s.axis_last_x + s.axis_last_y * s.axis_last_y >= 10000 * 10000 {
            let angle =
                (approximate_atan2(s.axis_last_y as f32, s.axis_last_x as f32) * 64.0 + 0.5) as u8;
            buttons = GAMEPAD_STICK_SEGMENT_BUTTONS
                [((angle.wrapping_add(16).wrapping_add(64)) >> 5) as usize];
        }
        s.gamepad_buttons = buttons;
    } else if axis == SDL_CONTROLLER_AXIS_TRIGGERLEFT || axis == SDL_CONTROLLER_AXIS_TRIGGERRIGHT {
        if value < 12000 || value >= 16000 {
            handle_gamepad_input(
                if axis == SDL_CONTROLLER_AXIS_TRIGGERLEFT {
                    GAMEPAD_BUTTON_L2 as i32
                } else {
                    GAMEPAD_BUTTON_R2 as i32
                },
                value >= 12000,
            );
        }
    }
}

fn load_rom(filename: *const i8) -> bool {
    let Some(path) = cstr_to_string(filename) else {
        return false;
    };
    let Ok(rom) = std::fs::read(path) else {
        return false;
    };
    let Ok(patched_rom) = patch_rom_owned(&rom) else {
        return false;
    };
    let mut snes = Snes::new();
    snes_load_rom(&mut snes, &patched_rom).is_ok()
}

fn parse_link_graphics(file: *mut u8, length: usize) -> bool {
    if file.is_null() || length < 27 {
        return false;
    }
    let bytes = unsafe { std::slice::from_raw_parts(file, length) };
    if &bytes[0..4] != b"ZSPR" {
        return false;
    }
    let pixel_offs = read_u32_c(bytes, 9) as usize;
    let pixel_length = read_u16_c(bytes, 13) as usize;
    let palette_offs = read_u32_c(bytes, 15) as usize;
    let palette_length = read_u16_c(bytes, 19) as usize;
    if pixel_offs + pixel_length > length
        || palette_offs + palette_length > length
        || pixel_length != 0x7000
    {
        return false;
    }

    let mut s = state().lock().unwrap();
    if s.assets.get(57).map(Vec::len) != Some(0x7000) || s.assets.get(81).map(Vec::len) != Some(150)
    {
        return false;
    }
    s.assets[57].copy_from_slice(&bytes[pixel_offs..pixel_offs + 0x7000]);
    if palette_length >= 120 {
        s.assets[81][..120].copy_from_slice(&bytes[palette_offs..palette_offs + 120]);
    }
    if palette_length >= 124 && s.assets[81].len() >= 124 {
        s.assets[81][120..124].copy_from_slice(&bytes[palette_offs + 120..palette_offs + 124]);
    }
    true
}

fn load_link_graphics() {
    let link_graphics = {
        let s = state().lock().unwrap();
        s.config_context.config.link_graphics.clone()
    };
    let Some(link_graphics) = link_graphics else {
        return;
    };
    eprintln!("Loading Link Graphics: {link_graphics}");
    let Ok(mut file) = std::fs::read(config_value_path(&link_graphics)) else {
        panic!("Unable to load file");
    };
    if !parse_link_graphics(file.as_mut_ptr(), file.len()) {
        panic!("Unable to load file");
    }
}

fn load_assets() {
    let data = match std::fs::read("zelda3_assets.dat") {
        Ok(data) => data,
        Err(_) => {
            let Ok(bps) = std::fs::read("zelda3_assets.bps") else {
                panic!(
                    "Failed to read zelda3_assets.dat. Please see the README for information about how you get this file."
                );
            };
            let Ok(bps_src) = std::fs::read("zelda3.sfc") else {
                panic!("Missing file: zelda3.sfc");
            };
            ApplyBps(&bps_src, &bps).unwrap_or_else(|| {
                panic!(
                    "Unable to apply zelda3_assets.bps. Please make sure you got the right version of 'zelda3.sfc'"
                )
            })
        }
    };
    let assets = parse_assets_file(&data).unwrap_or_else(|| panic!("Invalid assets file"));
    state().lock().unwrap().assets = assets;
}

fn switch_directory() {
    let Ok(mut dir) = std::env::current_dir() else {
        return;
    };
    for step in 0..3 {
        if dir.join("zelda3.ini").is_file() {
            if step != 0 {
                let _ = std::env::set_current_dir(dir);
            }
            return;
        }
        if !dir.pop() {
            return;
        }
    }
}

fn parse_main_config_file(argc: i32, argv: *mut *mut i8) -> Option<String> {
    if argc < 3 || argv.is_null() {
        return None;
    }
    let args = unsafe { std::slice::from_raw_parts(argv, argc as usize) };
    let first = cstr_to_string(args.get(1).copied().unwrap_or(std::ptr::null_mut()));
    if first.as_deref() != Some("--config") {
        return None;
    }
    cstr_to_string(args.get(2).copied().unwrap_or(std::ptr::null_mut()))
}

pub fn find_in_asset_array(asset: i32, idx: i32) -> MemBlk<'static> {
    let s = state().lock().unwrap();
    let data = s
        .assets
        .get(asset as usize)
        .cloned()
        .unwrap_or_default()
        .into_boxed_slice();
    let leaked: &'static [u8] = Box::leak(data);
    find_index_in_memblk(MemBlk { ptr: leaked }, idx as usize)
}

impl LockstepOracle {
    pub fn load_rom(&mut self, rom: &[u8]) -> Result<(), OracleError> {
        let patched_rom = patch_rom_owned(rom)?;
        snes_load_rom(&mut self.snes, &patched_rom)?;
        self.game.set_rom(rom);
        self.snes.cpu_seed_reset_vector();
        Ok(())
    }

    pub fn load_assets(&mut self, assets: &[u8]) -> Result<(), OracleError> {
        self.game
            .set_assets(assets)
            .map_err(OracleError::LoadAssets)
    }
}

fn cstr_to_string(ptr: *const i8) -> Option<String> {
    if ptr.is_null() {
        None
    } else {
        unsafe { CStr::from_ptr(ptr) }
            .to_str()
            .ok()
            .map(ToOwned::to_owned)
    }
}

fn parse_assets_file(data: &[u8]) -> Option<Vec<Vec<u8>>> {
    if data.len() < 88 + NUMBER_OF_ASSETS * 4
        || data.get(..ASSETS_SIGNATURE.len()) != Some(&ASSETS_SIGNATURE)
        || read_u32_c(data, 80) as usize != NUMBER_OF_ASSETS
    {
        return None;
    }
    let mut offset = 88 + NUMBER_OF_ASSETS * 4 + read_u32_c(data, 84) as usize;
    let mut out = Vec::with_capacity(NUMBER_OF_ASSETS);
    for i in 0..NUMBER_OF_ASSETS {
        let size = read_u32_c(data, 88 + i * 4) as usize;
        offset = (offset + 3) & !3;
        if offset + size > data.len() {
            return None;
        }
        out.push(data[offset..offset + size].to_vec());
        offset += size;
    }
    Some(out)
}

fn read_u16_c(bytes: &[u8], offset: usize) -> u16 {
    bytes
        .get(offset..offset + 2)
        .map(|s| u16::from_le_bytes([s[0], s[1]]))
        .unwrap_or(0)
}

fn read_u32_c(bytes: &[u8], offset: usize) -> u32 {
    bytes
        .get(offset..offset + 4)
        .map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
        .unwrap_or(0)
}

#[allow(dead_code)]
fn _path_exists(path: &Path) -> bool {
    path.exists()
}
