//! Libretro (Snes9x) core FFI loader, capture plumbing, and env callbacks.

use crate::*;

use std::env;
use std::ffi::{CStr, CString};
use std::fs;
use std::os::raw::{c_char, c_int, c_uint, c_void};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::AtomicBool;
use std::sync::{Mutex, OnceLock};

pub(crate) const RETRO_MEMORY_SAVE_RAM: c_uint = 0;
pub(crate) const RETRO_MEMORY_RTC: c_uint = 1;
pub(crate) const RETRO_MEMORY_SYSTEM_RAM: c_uint = 2;
pub(crate) const RETRO_MEMORY_VIDEO_RAM: c_uint = 3;

#[derive(Clone, Copy, Default)]
// Mirrors the libretro C ABI struct; unread fields must stay for layout.
#[allow(dead_code)]
pub(crate) struct RetroGameGeometry {
    pub(crate) base_width: c_uint,
    pub(crate) base_height: c_uint,
    pub(crate) max_width: c_uint,
    pub(crate) max_height: c_uint,
    pub(crate) aspect_ratio: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub(crate) struct RetroSystemTiming {
    pub(crate) fps: f64,
    pub(crate) sample_rate: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub(crate) struct RetroSystemAvInfo {
    pub(crate) geometry: RetroGameGeometry,
    pub(crate) timing: RetroSystemTiming,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct RetroSystemInfo {
    pub(crate) library_name: *const c_char,
    pub(crate) library_version: *const c_char,
    pub(crate) valid_extensions: *const c_char,
    pub(crate) need_fullpath: bool,
    pub(crate) block_extract: bool,
}

impl Default for RetroSystemInfo {
    fn default() -> Self {
        Self {
            library_name: std::ptr::null(),
            library_version: std::ptr::null(),
            valid_extensions: std::ptr::null(),
            need_fullpath: false,
            block_extract: false,
        }
    }
}

#[repr(C)]
pub(crate) struct RetroGameInfo {
    pub(crate) path: *const c_char,
    pub(crate) data: *const c_void,
    pub(crate) size: usize,
    pub(crate) meta: *const c_char,
}

#[repr(C)]
pub(crate) struct RetroVariable {
    pub(crate) key: *const c_char,
    pub(crate) value: *const c_char,
}

#[derive(Default)]
pub(crate) struct LibretroCapture {
    pub(crate) audio: Vec<i16>,
    pub(crate) video: Vec<u8>,
    pub(crate) video_width: u32,
    pub(crate) video_height: u32,
    pub(crate) video_pitch: usize,
    pub(crate) pixel_format: u32,
}

pub(crate) static LIBRETRO_CAPTURE: OnceLock<Mutex<LibretroCapture>> = OnceLock::new();
pub(crate) static LIBRETRO_INPUT_STATE: OnceLock<Mutex<u16>> = OnceLock::new();
pub(crate) static LIBRETRO_CAPTURE_ENABLED: AtomicBool = AtomicBool::new(true);
pub(crate) static LIBRETRO_SYSTEM_DIR: OnceLock<CString> = OnceLock::new();
pub(crate) static LIBRETRO_SAVE_DIR: OnceLock<CString> = OnceLock::new();

pub(crate) struct LibretroFrame {
    pub(crate) audio: Vec<i16>,
    pub(crate) video: Vec<u8>,
    pub(crate) video_width: u32,
    pub(crate) video_height: u32,
    pub(crate) video_pitch: usize,
    pub(crate) pixel_format: u32,
}

pub(crate) struct LibretroCore {
    pub(crate) handle: *mut c_void,
    pub(crate) retro_deinit: unsafe extern "C" fn(),
    pub(crate) retro_run: unsafe extern "C" fn(),
    pub(crate) retro_unload_game: unsafe extern "C" fn(),
    pub(crate) retro_get_memory_data: unsafe extern "C" fn(c_uint) -> *mut c_void,
    pub(crate) retro_get_memory_size: unsafe extern "C" fn(c_uint) -> usize,
    pub(crate) retro_serialize_size: unsafe extern "C" fn() -> usize,
    pub(crate) retro_serialize: unsafe extern "C" fn(*mut c_void, usize) -> bool,
    pub(crate) retro_unserialize: unsafe extern "C" fn(*const c_void, usize) -> bool,
    pub(crate) debug_trace_count: Option<unsafe extern "C" fn() -> c_int>,
    pub(crate) debug_trace_value: Option<unsafe extern "C" fn(c_int, c_int) -> c_int>,
    pub(crate) debug_write_count: Option<unsafe extern "C" fn() -> c_int>,
    pub(crate) debug_write_value: Option<unsafe extern "C" fn(c_int, c_int) -> c_int>,
    pub(crate) debug_write_cycle: Option<unsafe extern "C" fn(c_int) -> u64>,
    pub(crate) debug_frame_first_output_cycle: Option<unsafe extern "C" fn() -> u64>,
    pub(crate) debug_frame_last_output_cycle: Option<unsafe extern "C" fn() -> u64>,
    pub(crate) debug_frame_output_count: Option<unsafe extern "C" fn() -> c_int>,
    pub(crate) debug_dsp_sample_count: Option<unsafe extern "C" fn() -> c_int>,
    pub(crate) debug_dsp_field_count: Option<unsafe extern "C" fn() -> c_int>,
    pub(crate) debug_dsp_sample_value: Option<unsafe extern "C" fn(c_int, c_int) -> c_int>,
    pub(crate) debug_dsp_register_write_count: Option<unsafe extern "C" fn() -> c_int>,
    pub(crate) debug_dsp_register_write_value: Option<unsafe extern "C" fn(c_int, c_int) -> c_int>,
    pub(crate) debug_apu_port_write_count: Option<unsafe extern "C" fn() -> c_int>,
    pub(crate) debug_apu_port_write_value: Option<unsafe extern "C" fn(c_int, c_int) -> c_int>,
    pub(crate) debug_smp_output_port_write_count: Option<unsafe extern "C" fn() -> c_int>,
    pub(crate) debug_smp_output_port_write_value:
        Option<unsafe extern "C" fn(c_int, c_int) -> c_int>,
    pub(crate) debug_smp_output_port_write_cycle: Option<unsafe extern "C" fn(c_int) -> u64>,
    pub(crate) debug_smp_instruction_count: Option<unsafe extern "C" fn() -> c_int>,
    pub(crate) debug_smp_instruction_value: Option<unsafe extern "C" fn(c_int, c_int) -> c_int>,
    pub(crate) debug_smp_instruction_cycle: Option<unsafe extern "C" fn(c_int) -> u64>,
    pub(crate) debug_ppu_value: Option<unsafe extern "C" fn(c_int, c_int) -> c_int>,
    pub(crate) debug_scanline_mode7_value: Option<unsafe extern "C" fn(c_int, c_int) -> c_int>,
    pub(crate) api_version: c_uint,
    pub(crate) library_name: String,
    pub(crate) library_version: String,
    pub(crate) av_info: RetroSystemAvInfo,
    pub(crate) geometry: RetroGameGeometry,
    pub(crate) _rom: Vec<u8>,
    pub(crate) _rom_path: CString,
}

impl LibretroCore {
    pub(crate) fn load(core_path: &str, rom_path: &str) -> Result<Self, String> {
        Self::load_with_sram(core_path, rom_path, None)
    }

    pub(crate) fn load_with_sram(
        core_path: &str,
        rom_path: &str,
        initial_sram: Option<&[u8]>,
    ) -> Result<Self, String> {
        let capture = LIBRETRO_CAPTURE.get_or_init(|| Mutex::new(LibretroCapture::default()));
        *capture.lock().map_err(|_| "capture lock poisoned")? = LibretroCapture::default();
        let input_state = LIBRETRO_INPUT_STATE.get_or_init(|| Mutex::new(0));
        *input_state.lock().map_err(|_| "input lock poisoned")? = 0;
        let save_dir = initialize_libretro_dirs()?;
        if let Some(sram) = initial_sram {
            let stem = Path::new(rom_path)
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("zelda3");
            let sram_path = save_dir.join(format!("{stem}.srm"));
            fs::write(&sram_path, sram)
                .map_err(|e| format!("failed to seed snes9x SRAM {}: {e}", sram_path.display()))?;
        }

        let core_path_c = CString::new(core_path).map_err(|e| e.to_string())?;
        let handle = unsafe { libc::dlopen(core_path_c.as_ptr(), libc::RTLD_NOW) };
        if handle.is_null() {
            return Err(dlerror_string());
        }

        unsafe {
            let retro_set_environment: unsafe extern "C" fn(
                extern "C" fn(c_uint, *mut c_void) -> bool,
            ) = load_symbol(handle, "retro_set_environment")?;
            let retro_set_video_refresh: unsafe extern "C" fn(
                extern "C" fn(*const c_void, c_uint, c_uint, usize),
            ) = load_symbol(handle, "retro_set_video_refresh")?;
            let retro_set_audio_sample: unsafe extern "C" fn(extern "C" fn(i16, i16)) =
                load_symbol(handle, "retro_set_audio_sample")?;
            let retro_set_audio_sample_batch: unsafe extern "C" fn(
                extern "C" fn(*const i16, usize) -> usize,
            ) = load_symbol(handle, "retro_set_audio_sample_batch")?;
            let retro_set_input_poll: unsafe extern "C" fn(extern "C" fn()) =
                load_symbol(handle, "retro_set_input_poll")?;
            let retro_set_input_state: unsafe extern "C" fn(
                extern "C" fn(c_uint, c_uint, c_uint, c_uint) -> i16,
            ) = load_symbol(handle, "retro_set_input_state")?;
            let retro_init: unsafe extern "C" fn() = load_symbol(handle, "retro_init")?;
            let retro_deinit: unsafe extern "C" fn() = load_symbol(handle, "retro_deinit")?;
            let retro_load_game: unsafe extern "C" fn(*const RetroGameInfo) -> bool =
                load_symbol(handle, "retro_load_game")?;
            let retro_unload_game: unsafe extern "C" fn() =
                load_symbol(handle, "retro_unload_game")?;
            let retro_run: unsafe extern "C" fn() = load_symbol(handle, "retro_run")?;
            let retro_get_system_av_info: unsafe extern "C" fn(*mut RetroSystemAvInfo) =
                load_symbol(handle, "retro_get_system_av_info")?;
            let retro_get_memory_data: unsafe extern "C" fn(c_uint) -> *mut c_void =
                load_symbol(handle, "retro_get_memory_data")?;
            let retro_get_memory_size: unsafe extern "C" fn(c_uint) -> usize =
                load_symbol(handle, "retro_get_memory_size")?;
            let retro_api_version: unsafe extern "C" fn() -> c_uint =
                load_symbol(handle, "retro_api_version")?;
            let retro_get_system_info: unsafe extern "C" fn(*mut RetroSystemInfo) =
                load_symbol(handle, "retro_get_system_info")?;
            let retro_serialize_size: unsafe extern "C" fn() -> usize =
                load_symbol(handle, "retro_serialize_size")?;
            let retro_serialize: unsafe extern "C" fn(*mut c_void, usize) -> bool =
                load_symbol(handle, "retro_serialize")?;
            let retro_unserialize: unsafe extern "C" fn(*const c_void, usize) -> bool =
                load_symbol(handle, "retro_unserialize")?;
            let debug_trace_count = optional_symbol(handle, "zelda3_snes9x_debug_trace_count");
            let debug_trace_value = optional_symbol(handle, "zelda3_snes9x_debug_trace_value");
            let debug_write_count = optional_symbol(handle, "zelda3_snes9x_debug_write_count");
            let debug_write_value = optional_symbol(handle, "zelda3_snes9x_debug_write_value");
            let debug_write_cycle = optional_symbol(handle, "zelda3_snes9x_debug_write_cycle");
            let debug_frame_first_output_cycle =
                optional_symbol(handle, "zelda3_snes9x_debug_frame_first_output_cycle");
            let debug_frame_last_output_cycle =
                optional_symbol(handle, "zelda3_snes9x_debug_frame_last_output_cycle");
            let debug_frame_output_count =
                optional_symbol(handle, "zelda3_snes9x_debug_frame_output_count");
            let debug_dsp_sample_count =
                optional_symbol(handle, "zelda3_snes9x_debug_dsp_sample_count");
            let debug_dsp_field_count =
                optional_symbol(handle, "zelda3_snes9x_debug_dsp_field_count");
            let debug_dsp_sample_value =
                optional_symbol(handle, "zelda3_snes9x_debug_dsp_sample_value");
            let debug_dsp_register_write_count =
                optional_symbol(handle, "zelda3_snes9x_debug_dsp_write_count");
            let debug_dsp_register_write_value =
                optional_symbol(handle, "zelda3_snes9x_debug_dsp_write_value");
            let debug_apu_port_write_count =
                optional_symbol(handle, "zelda3_snes9x_debug_apu_port_write_count");
            let debug_apu_port_write_value =
                optional_symbol(handle, "zelda3_snes9x_debug_apu_port_write_value");
            let debug_smp_output_port_write_count =
                optional_symbol(handle, "zelda3_snes9x_debug_smp_output_port_write_count");
            let debug_smp_output_port_write_value =
                optional_symbol(handle, "zelda3_snes9x_debug_smp_output_port_write_value");
            let debug_smp_output_port_write_cycle =
                optional_symbol(handle, "zelda3_snes9x_debug_smp_output_port_write_cycle");
            let debug_smp_instruction_count =
                optional_symbol(handle, "zelda3_snes9x_debug_smp_instruction_count");
            let debug_smp_instruction_value =
                optional_symbol(handle, "zelda3_snes9x_debug_smp_instruction_value");
            let debug_smp_instruction_cycle =
                optional_symbol(handle, "zelda3_snes9x_debug_smp_instruction_cycle");
            let debug_ppu_value = optional_symbol(handle, "zelda3_snes9x_debug_ppu_value");
            let debug_scanline_mode7_value =
                optional_symbol(handle, "zelda3_snes9x_debug_scanline_mode7_value");

            let api_version = retro_api_version();
            if api_version != 1 {
                libc::dlclose(handle);
                return Err(format!(
                    "unsupported libretro API version {api_version}; expected 1"
                ));
            }
            let mut system_info = RetroSystemInfo::default();
            retro_get_system_info(&mut system_info);
            let library_name = optional_c_string(system_info.library_name, "unknown");
            let library_version = optional_c_string(system_info.library_version, "unknown");

            retro_set_environment(libretro_environment);
            retro_set_video_refresh(libretro_video_refresh);
            retro_set_audio_sample(libretro_audio_sample);
            retro_set_audio_sample_batch(libretro_audio_sample_batch);
            retro_set_input_poll(libretro_input_poll);
            retro_set_input_state(libretro_input_state);
            retro_init();

            let rom = fs::read(rom_path).map_err(|e| e.to_string())?;
            let rom_path_c = CString::new(rom_path).map_err(|e| e.to_string())?;
            let game_info = RetroGameInfo {
                path: rom_path_c.as_ptr(),
                data: rom.as_ptr().cast(),
                size: rom.len(),
                meta: std::ptr::null(),
            };
            if !retro_load_game(&game_info) {
                retro_deinit();
                libc::dlclose(handle);
                return Err("retro_load_game returned false".to_string());
            }
            if let Some(sram) = initial_sram {
                let size = retro_get_memory_size(RETRO_MEMORY_SAVE_RAM);
                let data = retro_get_memory_data(RETRO_MEMORY_SAVE_RAM);
                if size != 0 && !data.is_null() {
                    let destination = std::slice::from_raw_parts_mut(data.cast::<u8>(), size);
                    destination.fill(0);
                    let count = destination.len().min(sram.len());
                    destination[..count].copy_from_slice(&sram[..count]);
                }
            }

            let mut av_info = RetroSystemAvInfo::default();
            retro_get_system_av_info(&mut av_info);
            Ok(Self {
                handle,
                retro_deinit,
                retro_run,
                retro_unload_game,
                retro_get_memory_data,
                retro_get_memory_size,
                retro_serialize_size,
                retro_serialize,
                retro_unserialize,
                debug_trace_count,
                debug_trace_value,
                debug_write_count,
                debug_write_value,
                debug_write_cycle,
                debug_frame_first_output_cycle,
                debug_frame_last_output_cycle,
                debug_frame_output_count,
                debug_dsp_sample_count,
                debug_dsp_field_count,
                debug_dsp_sample_value,
                debug_dsp_register_write_count,
                debug_dsp_register_write_value,
                debug_apu_port_write_count,
                debug_apu_port_write_value,
                debug_smp_output_port_write_count,
                debug_smp_output_port_write_value,
                debug_smp_output_port_write_cycle,
                debug_smp_instruction_count,
                debug_smp_instruction_value,
                debug_smp_instruction_cycle,
                debug_ppu_value,
                debug_scanline_mode7_value,
                api_version,
                library_name,
                library_version,
                av_info,
                geometry: av_info.geometry,
                _rom: rom,
                _rom_path: rom_path_c,
            })
        }
    }

    pub(crate) fn run_frame(&mut self) -> LibretroFrame {
        self.run_frame_with_input(0)
    }

    pub(crate) fn run_frame_with_input(&mut self, input: u16) -> LibretroFrame {
        if let Some(input_state) = LIBRETRO_INPUT_STATE.get() {
            if let Ok(mut input_state) = input_state.lock() {
                *input_state = input;
            }
        }
        if let Some(capture) = LIBRETRO_CAPTURE.get() {
            if let Ok(mut capture) = capture.lock() {
                capture.audio.clear();
            }
        }
        unsafe { (self.retro_run)() };
        LIBRETRO_CAPTURE
            .get()
            .and_then(|capture| {
                capture.lock().ok().map(|capture| LibretroFrame {
                    audio: capture.audio.clone(),
                    video: capture.video.clone(),
                    video_width: capture.video_width,
                    video_height: capture.video_height,
                    video_pitch: capture.video_pitch,
                    pixel_format: capture.pixel_format,
                })
            })
            .unwrap_or_else(|| LibretroFrame {
                audio: Vec::new(),
                video: Vec::new(),
                video_width: 0,
                video_height: 0,
                video_pitch: 0,
                pixel_format: 0,
            })
    }

    pub(crate) fn run_frame_discard_with_input(&mut self, input: u16) {
        if let Some(input_state) = LIBRETRO_INPUT_STATE.get() {
            if let Ok(mut input_state) = input_state.lock() {
                *input_state = input;
            }
        }
        unsafe { (self.retro_run)() };
    }

    pub(crate) fn memory_size(&self, id: c_uint) -> usize {
        unsafe { (self.retro_get_memory_size)(id) }
    }

    pub(crate) fn memory_bytes(&self, id: c_uint) -> Option<&[u8]> {
        let size = self.memory_size(id);
        if size == 0 {
            return None;
        }
        let ptr = unsafe { (self.retro_get_memory_data)(id) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { std::slice::from_raw_parts(ptr.cast::<u8>(), size) })
        }
    }

    pub(crate) fn replace_memory(
        &mut self,
        id: c_uint,
        bytes: &[u8],
        label: &str,
    ) -> Result<(), String> {
        let size = self.memory_size(id);
        let ptr = unsafe { (self.retro_get_memory_data)(id) };
        if size == 0 || ptr.is_null() {
            return Err(format!("{} did not expose {label}", self.library_name));
        }
        if size != bytes.len() {
            return Err(format!(
                "{label} restore has {} bytes, but {} exposes {size}",
                bytes.len(),
                self.library_name
            ));
        }
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr.cast::<u8>(), size);
        }
        Ok(())
    }

    pub(crate) fn serialize_state(&self) -> Result<Vec<u8>, String> {
        let mut state = Vec::new();
        self.serialize_state_into(&mut state)?;
        Ok(state)
    }

    pub(crate) fn serialize_state_into(&self, state: &mut Vec<u8>) -> Result<(), String> {
        let size = unsafe { (self.retro_serialize_size)() };
        if size == 0 {
            return Err(format!(
                "{} {} does not support libretro serialization",
                self.library_name, self.library_version
            ));
        }
        state.resize(size, 0);
        if !unsafe { (self.retro_serialize)(state.as_mut_ptr().cast(), state.len()) } {
            return Err(format!(
                "{} {} retro_serialize returned false",
                self.library_name, self.library_version
            ));
        }
        Ok(())
    }

    pub(crate) fn unserialize_state(&mut self, state: &[u8]) -> Result<(), String> {
        if !unsafe { (self.retro_unserialize)(state.as_ptr().cast(), state.len()) } {
            return Err(format!(
                "{} {} retro_unserialize returned false for {} bytes",
                self.library_name,
                self.library_version,
                state.len()
            ));
        }
        Ok(())
    }

    pub(crate) fn debug_dsp_trace(&self) -> Option<Vec<Vec<i32>>> {
        let (Some(count), Some(value)) = (self.debug_trace_count, self.debug_trace_value) else {
            return None;
        };
        let count = unsafe { count() }.max(0);
        Some(
            (0..count)
                .map(|sample| {
                    (0..118)
                        .map(|field| unsafe { value(sample, field) })
                        .collect()
                })
                .collect(),
        )
    }

    pub(crate) fn debug_dsp_writes(&self) -> Option<Vec<LibretroDspWrite>> {
        let (Some(count), Some(value)) = (self.debug_write_count, self.debug_write_value) else {
            return None;
        };
        let count = unsafe { count() }.max(0);
        Some(
            (0..count)
                .map(|write| LibretroDspWrite {
                    register: unsafe { value(write, 0) },
                    value: unsafe { value(write, 1) },
                    legacy_sample_offset: unsafe { value(write, 2) },
                    phase: unsafe { value(write, 3) },
                    sfx_voice_mask: unsafe { value(write, 4) },
                    absolute_cycle: self.debug_write_cycle.map(|cycle| unsafe { cycle(write) }),
                })
                .collect(),
        )
    }

    pub(crate) fn debug_dsp_frame_clock(&self) -> Option<LibretroDspFrameClock> {
        let (Some(first), Some(last), Some(count)) = (
            self.debug_frame_first_output_cycle,
            self.debug_frame_last_output_cycle,
            self.debug_frame_output_count,
        ) else {
            return None;
        };
        Some(LibretroDspFrameClock {
            first_output_cycle: unsafe { first() },
            last_output_cycle: unsafe { last() },
            output_count: unsafe { count() }.max(0),
        })
    }

    pub(crate) fn debug_dsp_samples(&self) -> Option<Vec<LibretroDspSample>> {
        let (Some(count), Some(field_count), Some(value)) = (
            self.debug_dsp_sample_count,
            self.debug_dsp_field_count,
            self.debug_dsp_sample_value,
        ) else {
            return None;
        };
        if unsafe { field_count() } != 217 {
            return None;
        }
        let count = unsafe { count() }.max(0);
        Some(
            (0..count)
                .map(|sample| {
                    let field = |field| unsafe { value(sample, field) };
                    LibretroDspSample {
                        output: [field(0), field(1)],
                        counter: field(2),
                        every_other_sample: field(3),
                        kon: field(4),
                        new_kon: field(5),
                        latched_koff: field(6),
                        phase: field(7),
                        koff_register: field(8),
                        kon_register: field(9),
                        flags_register: field(10),
                        pipeline_output: field(11),
                        pitch_modulation: field(12),
                        noise_enable: field(13),
                        echo_enable: field(14),
                        source_directory_page: field(15),
                        echo_filtered: [field(128), field(129)],
                        echo_write: [field(130), field(131)],
                        echo_offset: field(132),
                        echo_length: field(133),
                        echo_history_position: field(134),
                        echo_history: [
                            std::array::from_fn(|tap| field(135 + tap as i32)),
                            std::array::from_fn(|tap| field(143 + tap as i32)),
                        ],
                        raw_main: [field(151), field(152)],
                        voice_output: std::array::from_fn(|voice| {
                            [field(153 + voice as i32 * 2), field(154 + voice as i32 * 2)]
                        }),
                        voice_pipeline_output: std::array::from_fn(|voice| {
                            field(169 + voice as i32)
                        }),
                        interpolation: (0..8)
                            .map(|voice| {
                                let base = 177 + voice * 5;
                                LibretroDspInterpolation {
                                    offset: field(base),
                                    samples: [
                                        field(base + 1),
                                        field(base + 2),
                                        field(base + 3),
                                        field(base + 4),
                                    ],
                                }
                            })
                            .collect(),
                        voices: (0..8)
                            .map(|voice| {
                                let base = 16 + voice * 14;
                                LibretroDspVoice {
                                    envelope: field(base),
                                    hidden_envelope: field(base + 1),
                                    envelope_mode: field(base + 2),
                                    key_on_delay: field(base + 3),
                                    interpolation_position: field(base + 4),
                                    brr_address: field(base + 5),
                                    brr_offset: field(base + 6),
                                    envelope_register_output: field(base + 7),
                                    volume_left: field(base + 8),
                                    volume_right: field(base + 9),
                                    pitch: field(base + 10),
                                    adsr0: field(base + 11),
                                    adsr1: field(base + 12),
                                    gain: field(base + 13),
                                }
                            })
                            .collect(),
                    }
                })
                .collect(),
        )
    }

    pub(crate) fn debug_dsp_register_writes(&self) -> Option<Vec<LibretroDspRegisterWrite>> {
        let (Some(count), Some(value)) = (
            self.debug_dsp_register_write_count,
            self.debug_dsp_register_write_value,
        ) else {
            return None;
        };
        let count = unsafe { count() }.max(0);
        Some(
            (0..count)
                .map(|write| LibretroDspRegisterWrite {
                    register: unsafe { value(write, 0) },
                    value: unsafe { value(write, 1) },
                    output_sample: unsafe { value(write, 2) },
                    dsp_phase: unsafe { value(write, 3) },
                })
                .collect(),
        )
    }

    pub(crate) fn debug_apu_port_writes(&self) -> Option<Vec<LibretroApuPortWrite>> {
        let (Some(count), Some(value)) = (
            self.debug_apu_port_write_count,
            self.debug_apu_port_write_value,
        ) else {
            return None;
        };
        let count = unsafe { count() }.max(0);
        Some(
            (0..count)
                .map(|write| LibretroApuPortWrite {
                    port: unsafe { value(write, 0) },
                    value: unsafe { value(write, 1) },
                    output_sample: unsafe { value(write, 2) },
                    v_counter: unsafe { value(write, 3) },
                    cpu_cycle: unsafe { value(write, 4) },
                    program_counter: unsafe { value(write, 5) },
                    apu_cycle_before: unsafe { value(write, 6) },
                    apu_cycle_after: unsafe { value(write, 7) },
                    smp_clock_before: unsafe { value(write, 8) },
                    smp_clock_after: unsafe { value(write, 9) },
                    smp_pc_before: unsafe { value(write, 14) },
                    smp_pc_after: unsafe { value(write, 15) },
                    smp_opcode_before: unsafe { value(write, 16) },
                    smp_opcode_after: unsafe { value(write, 17) },
                    is_read: unsafe { value(write, 20) } != 0,
                })
                .collect(),
        )
    }

    pub(crate) fn debug_smp_instructions(&self) -> Option<Vec<LibretroSmpInstruction>> {
        let (Some(count), Some(value), Some(cycle)) = (
            self.debug_smp_instruction_count,
            self.debug_smp_instruction_value,
            self.debug_smp_instruction_cycle,
        ) else {
            return None;
        };
        let count = unsafe { count() }.max(0);
        Some(
            (0..count)
                .map(|instruction| LibretroSmpInstruction {
                    absolute_cycle: unsafe { cycle(instruction) },
                    program_counter: unsafe { value(instruction, 0) },
                    opcode: unsafe { value(instruction, 1) },
                    a: unsafe { value(instruction, 2) },
                    x: unsafe { value(instruction, 3) },
                    y: unsafe { value(instruction, 4) },
                    stack_pointer: unsafe { value(instruction, 5) },
                    status: unsafe { value(instruction, 6) },
                    timer0_stage1: unsafe { value(instruction, 7) },
                    timer0_stage2: unsafe { value(instruction, 8) },
                    timer0_stage3: unsafe { value(instruction, 9) },
                    output_sample: unsafe { value(instruction, 10) },
                    dsp_phase: unsafe { value(instruction, 11) },
                    smp_clock: unsafe { value(instruction, 12) },
                    direct_page_0_11: std::array::from_fn(|offset| unsafe {
                        value(instruction, 13 + offset as i32)
                    }),
                    boundary_opcode_cycle: unsafe { value(instruction, 25) },
                    op_step_calls: unsafe { value(instruction, 26) },
                    max_continuation_opcode_cycle: unsafe { value(instruction, 27) },
                })
                .collect(),
        )
    }

    pub(crate) fn debug_smp_output_port_writes(&self) -> Option<Vec<LibretroSmpOutputPortWrite>> {
        let (Some(count), Some(value), Some(cycle)) = (
            self.debug_smp_output_port_write_count,
            self.debug_smp_output_port_write_value,
            self.debug_smp_output_port_write_cycle,
        ) else {
            return None;
        };
        let count = unsafe { count() }.max(0);
        Some(
            (0..count)
                .map(|write| LibretroSmpOutputPortWrite {
                    absolute_cycle: unsafe { cycle(write) },
                    port: unsafe { value(write, 0) },
                    value: unsafe { value(write, 1) },
                    origin_pc: unsafe { value(write, 2) },
                    opcode: unsafe { value(write, 3) },
                    opcode_cycle: unsafe { value(write, 4) },
                    v_counter: unsafe { value(write, 5) },
                    cpu_cycle: unsafe { value(write, 6) },
                    cpu_program_counter: unsafe { value(write, 7) },
                    cpu_reference_time: unsafe { value(write, 8) },
                    cpu_remainder: unsafe { value(write, 9) },
                    smp_clock: unsafe { value(write, 10) },
                    next_pc: unsafe { value(write, 11) },
                    dsp_clock: unsafe { value(write, 12) },
                    dsp_phase: unsafe { value(write, 13) },
                    output_sample: unsafe { value(write, 14) },
                })
                .collect(),
        )
    }

    pub(crate) fn debug_ppu_value(&self, field: i32, index: i32) -> Option<i32> {
        self.debug_ppu_value
            .map(|probe| unsafe { probe(field, index) })
    }

    pub(crate) fn debug_scanline_mode7_value(&self, line: i32, field: i32) -> Option<i32> {
        self.debug_scanline_mode7_value
            .map(|probe| unsafe { probe(line, field) })
    }
}

#[derive(serde::Serialize)]
pub(crate) struct LibretroDspWrite {
    pub(crate) register: i32,
    pub(crate) value: i32,
    pub(crate) legacy_sample_offset: i32,
    pub(crate) phase: i32,
    pub(crate) sfx_voice_mask: i32,
    pub(crate) absolute_cycle: Option<u64>,
}

#[derive(serde::Serialize)]
pub(crate) struct LibretroDspFrameClock {
    pub(crate) first_output_cycle: u64,
    pub(crate) last_output_cycle: u64,
    pub(crate) output_count: i32,
}

#[derive(serde::Serialize)]
pub(crate) struct LibretroDspSample {
    pub(crate) output: [i32; 2],
    pub(crate) counter: i32,
    pub(crate) every_other_sample: i32,
    pub(crate) kon: i32,
    pub(crate) new_kon: i32,
    pub(crate) latched_koff: i32,
    pub(crate) phase: i32,
    pub(crate) koff_register: i32,
    pub(crate) kon_register: i32,
    pub(crate) flags_register: i32,
    pub(crate) pipeline_output: i32,
    pub(crate) pitch_modulation: i32,
    pub(crate) noise_enable: i32,
    pub(crate) echo_enable: i32,
    pub(crate) source_directory_page: i32,
    pub(crate) echo_filtered: [i32; 2],
    pub(crate) echo_write: [i32; 2],
    pub(crate) echo_offset: i32,
    pub(crate) echo_length: i32,
    pub(crate) echo_history_position: i32,
    pub(crate) echo_history: [[i32; 8]; 2],
    pub(crate) raw_main: [i32; 2],
    pub(crate) voice_output: [[i32; 2]; 8],
    pub(crate) voice_pipeline_output: [i32; 8],
    pub(crate) interpolation: Vec<LibretroDspInterpolation>,
    pub(crate) voices: Vec<LibretroDspVoice>,
}

#[derive(serde::Serialize)]
pub(crate) struct LibretroDspInterpolation {
    pub(crate) offset: i32,
    pub(crate) samples: [i32; 4],
}

#[derive(serde::Serialize)]
pub(crate) struct LibretroDspVoice {
    pub(crate) envelope: i32,
    pub(crate) hidden_envelope: i32,
    pub(crate) envelope_mode: i32,
    pub(crate) key_on_delay: i32,
    pub(crate) interpolation_position: i32,
    pub(crate) brr_address: i32,
    pub(crate) brr_offset: i32,
    pub(crate) envelope_register_output: i32,
    pub(crate) volume_left: i32,
    pub(crate) volume_right: i32,
    pub(crate) pitch: i32,
    pub(crate) adsr0: i32,
    pub(crate) adsr1: i32,
    pub(crate) gain: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub(crate) struct LibretroDspRegisterWrite {
    pub(crate) register: i32,
    pub(crate) value: i32,
    pub(crate) output_sample: i32,
    pub(crate) dsp_phase: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(crate) struct LibretroApuPortWrite {
    pub(crate) port: i32,
    pub(crate) value: i32,
    pub(crate) output_sample: i32,
    pub(crate) v_counter: i32,
    pub(crate) cpu_cycle: i32,
    pub(crate) program_counter: i32,
    pub(crate) apu_cycle_before: i32,
    pub(crate) apu_cycle_after: i32,
    pub(crate) smp_clock_before: i32,
    pub(crate) smp_clock_after: i32,
    pub(crate) smp_pc_before: i32,
    pub(crate) smp_pc_after: i32,
    pub(crate) smp_opcode_before: i32,
    pub(crate) smp_opcode_after: i32,
    pub(crate) is_read: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub(crate) struct LibretroSmpOutputPortWrite {
    pub(crate) absolute_cycle: u64,
    pub(crate) port: i32,
    pub(crate) value: i32,
    pub(crate) origin_pc: i32,
    pub(crate) opcode: i32,
    pub(crate) opcode_cycle: i32,
    pub(crate) v_counter: i32,
    pub(crate) cpu_cycle: i32,
    pub(crate) cpu_program_counter: i32,
    pub(crate) cpu_reference_time: i32,
    pub(crate) cpu_remainder: i32,
    pub(crate) smp_clock: i32,
    pub(crate) next_pc: i32,
    pub(crate) dsp_clock: i32,
    pub(crate) dsp_phase: i32,
    pub(crate) output_sample: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(crate) struct LibretroSmpInstruction {
    pub(crate) absolute_cycle: u64,
    pub(crate) program_counter: i32,
    pub(crate) opcode: i32,
    pub(crate) a: i32,
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) stack_pointer: i32,
    pub(crate) status: i32,
    pub(crate) timer0_stage1: i32,
    pub(crate) timer0_stage2: i32,
    pub(crate) timer0_stage3: i32,
    pub(crate) output_sample: i32,
    pub(crate) dsp_phase: i32,
    pub(crate) smp_clock: i32,
    pub(crate) direct_page_0_11: [i32; 12],
    pub(crate) boundary_opcode_cycle: i32,
    pub(crate) op_step_calls: i32,
    pub(crate) max_continuation_opcode_cycle: i32,
}

impl Drop for LibretroCore {
    fn drop(&mut self) {
        unsafe {
            (self.retro_unload_game)();
            (self.retro_deinit)();
            libc::dlclose(self.handle);
        }
    }
}

unsafe fn load_symbol<T: Copy>(handle: *mut c_void, name: &str) -> Result<T, String> {
    let name_c = CString::new(name).unwrap();
    let ptr = unsafe { libc::dlsym(handle, name_c.as_ptr()) };
    if ptr.is_null() {
        Err(dlerror_string())
    } else {
        Ok(unsafe { std::mem::transmute_copy(&ptr) })
    }
}

unsafe fn optional_symbol<T: Copy>(handle: *mut c_void, name: &str) -> Option<T> {
    let name_c = CString::new(name).unwrap();
    let ptr = unsafe { libc::dlsym(handle, name_c.as_ptr()) };
    (!ptr.is_null()).then(|| unsafe { std::mem::transmute_copy(&ptr) })
}

pub(crate) fn dlerror_string() -> String {
    let err = unsafe { libc::dlerror() };
    if err.is_null() {
        "unknown dynamic loader error".to_string()
    } else {
        unsafe { CStr::from_ptr(err) }
            .to_string_lossy()
            .into_owned()
    }
}

pub(crate) fn optional_c_string(value: *const c_char, fallback: &str) -> String {
    if value.is_null() {
        fallback.to_string()
    } else {
        unsafe { CStr::from_ptr(value) }
            .to_string_lossy()
            .into_owned()
    }
}

pub(crate) fn initialize_libretro_dirs() -> Result<PathBuf, String> {
    if let Some(save_dir) = LIBRETRO_SAVE_DIR.get() {
        return Ok(PathBuf::from(save_dir.to_string_lossy().into_owned()));
    }

    let save_dir = env::current_dir()
        .map_err(|e| e.to_string())?
        .join("target")
        .join("libretro-oracle-save")
        .join(process::id().to_string());
    fs::create_dir_all(&save_dir).map_err(|e| e.to_string())?;
    let save_dir_c =
        CString::new(save_dir.to_string_lossy().as_bytes()).map_err(|e| e.to_string())?;
    let _ = LIBRETRO_SYSTEM_DIR.set(save_dir_c.clone());
    let _ = LIBRETRO_SAVE_DIR.set(save_dir_c);
    Ok(save_dir)
}

pub(crate) fn libretro_memory_name(id: c_uint) -> &'static str {
    match id {
        RETRO_MEMORY_SAVE_RAM => "SAVE_RAM",
        RETRO_MEMORY_RTC => "RTC",
        RETRO_MEMORY_SYSTEM_RAM => "SYSTEM_RAM",
        RETRO_MEMORY_VIDEO_RAM => "VIDEO_RAM",
        _ => "UNKNOWN",
    }
}
