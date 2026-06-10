//! Config/keymap ports from `src/config.c`.

#![allow(non_snake_case)]

use std::ffi::CStr;
use std::fs;
use std::path::{Path, PathBuf};

use crate::util::{
    NextDelim, NextLineStripComments, NextPossiblyQuotedString, ReplaceFilenameWithNewPath,
    SplitKeyValue, StringEqualsNoCase, StringStartsWithNoCase,
};

pub type SdlKeycode = i32;
pub type SdlKeymod = u16;

const SDLK_SCANCODE_MASK: SdlKeycode = 1 << 30;
const SDLK_UNKNOWN: SdlKeycode = 0;
const SDLK_RETURN: SdlKeycode = 13;
const SDLK_TAB: SdlKeycode = 9;
const SDLK_F1: SdlKeycode = SDLK_SCANCODE_MASK | 58;
const SDLK_F2: SdlKeycode = SDLK_SCANCODE_MASK | 59;
const SDLK_F3: SdlKeycode = SDLK_SCANCODE_MASK | 60;
const SDLK_F4: SdlKeycode = SDLK_SCANCODE_MASK | 61;
const SDLK_F5: SdlKeycode = SDLK_SCANCODE_MASK | 62;
const SDLK_F6: SdlKeycode = SDLK_SCANCODE_MASK | 63;
const SDLK_F7: SdlKeycode = SDLK_SCANCODE_MASK | 64;
const SDLK_F8: SdlKeycode = SDLK_SCANCODE_MASK | 65;
const SDLK_F9: SdlKeycode = SDLK_SCANCODE_MASK | 66;
const SDLK_F10: SdlKeycode = SDLK_SCANCODE_MASK | 67;
const SDLK_UP: SdlKeycode = SDLK_SCANCODE_MASK | 82;
const SDLK_DOWN: SdlKeycode = SDLK_SCANCODE_MASK | 81;
const SDLK_LEFT: SdlKeycode = SDLK_SCANCODE_MASK | 80;
const SDLK_RIGHT: SdlKeycode = SDLK_SCANCODE_MASK | 79;
const SDLK_LCTRL: SdlKeycode = SDLK_SCANCODE_MASK | 224;
const SDLK_LSHIFT: SdlKeycode = SDLK_SCANCODE_MASK | 225;
const SDLK_LALT: SdlKeycode = SDLK_SCANCODE_MASK | 226;
const SDLK_RCTRL: SdlKeycode = SDLK_SCANCODE_MASK | 228;
const SDLK_RSHIFT: SdlKeycode = SDLK_SCANCODE_MASK | 229;
const SDLK_RALT: SdlKeycode = SDLK_SCANCODE_MASK | 230;

pub const KMOD_SHIFT: SdlKeymod = 0x0003;
pub const KMOD_CTRL: SdlKeymod = 0x00c0;
pub const KMOD_ALT: SdlKeymod = 0x0300;

const KEY_MODIFIER_SCAN_CODE: u16 = 0x0200;
const KEY_MODIFIER_ALT: u16 = 0x0400;
const KEY_MODIFIER_SHIFT: u16 = 0x0800;
const KEY_MODIFIER_CTRL: u16 = 0x1000;

const FEATURE_EXTEND_SCREEN64: u32 = 1;
const FEATURE_SWITCH_LR: u32 = 2;
const FEATURE_TURN_WHILE_DASHING: u32 = 4;
const FEATURE_MIRROR_TO_DARKWORLD: u32 = 8;
const FEATURE_COLLECT_ITEMS_WITH_SWORD: u32 = 16;
const FEATURE_BREAK_POTS_WITH_SWORD: u32 = 32;
const FEATURE_DISABLE_LOW_HEALTH_BEEP: u32 = 64;
const FEATURE_SKIP_INTRO_ON_KEYPRESS: u32 = 128;
const FEATURE_SHOW_MAX_ITEMS_IN_YELLOW: u32 = 256;
const FEATURE_MORE_ACTIVE_BOMBS: u32 = 512;
const FEATURE_WIDESCREEN_VISUAL_FIXES: u32 = 1024;
const FEATURE_CARRY_MORE_RUPEES: u32 = 2048;
const FEATURE_MISC_BUG_FIXES: u32 = 4096;
const FEATURE_CANCEL_BIRD_TRAVEL: u32 = 8192;
const FEATURE_GAME_CHANGING_BUG_FIXES: u32 = 16384;
const FEATURE_SWITCH_LR_LIMIT: u32 = 32768;
const FEATURE_DIM_FLASHES: u32 = 65536;

pub const KEY_COMMAND_NULL: u16 = 0;
pub const KEY_COMMAND_CONTROLS: u16 = 1;
pub const KEY_COMMAND_CONTROLS_LAST: u16 = KEY_COMMAND_CONTROLS + 11;
pub const KEY_COMMAND_LOAD: u16 = KEY_COMMAND_CONTROLS_LAST + 1;
pub const KEY_COMMAND_LOAD_LAST: u16 = KEY_COMMAND_LOAD + 19;
pub const KEY_COMMAND_SAVE: u16 = KEY_COMMAND_LOAD_LAST + 1;
pub const KEY_COMMAND_SAVE_LAST: u16 = KEY_COMMAND_SAVE + 19;
pub const KEY_COMMAND_REPLAY: u16 = KEY_COMMAND_SAVE_LAST + 1;
pub const KEY_COMMAND_REPLAY_LAST: u16 = KEY_COMMAND_REPLAY + 19;
pub const KEY_COMMAND_LOAD_REF: u16 = KEY_COMMAND_REPLAY_LAST + 1;
pub const KEY_COMMAND_LOAD_REF_LAST: u16 = KEY_COMMAND_LOAD_REF + 19;
pub const KEY_COMMAND_REPLAY_REF: u16 = KEY_COMMAND_LOAD_REF_LAST + 1;
pub const KEY_COMMAND_REPLAY_REF_LAST: u16 = KEY_COMMAND_REPLAY_REF + 19;
pub const KEY_COMMAND_CHEAT_LIFE: u16 = KEY_COMMAND_REPLAY_REF_LAST + 1;
pub const KEY_COMMAND_CHEAT_KEYS: u16 = KEY_COMMAND_CHEAT_LIFE + 1;
pub const KEY_COMMAND_CHEAT_EQUIPMENT: u16 = KEY_COMMAND_CHEAT_KEYS + 1;
pub const KEY_COMMAND_CHEAT_WALK_THROUGH_WALLS: u16 = KEY_COMMAND_CHEAT_EQUIPMENT + 1;
pub const KEY_COMMAND_CLEAR_KEY_LOG: u16 = KEY_COMMAND_CHEAT_WALK_THROUGH_WALLS + 1;
pub const KEY_COMMAND_STOP_REPLAY: u16 = KEY_COMMAND_CLEAR_KEY_LOG + 1;
pub const KEY_COMMAND_FULLSCREEN: u16 = KEY_COMMAND_STOP_REPLAY + 1;
pub const KEY_COMMAND_RESET: u16 = KEY_COMMAND_FULLSCREEN + 1;
pub const KEY_COMMAND_PAUSE: u16 = KEY_COMMAND_RESET + 1;
pub const KEY_COMMAND_PAUSE_DIMMED: u16 = KEY_COMMAND_PAUSE + 1;
pub const KEY_COMMAND_TURBO: u16 = KEY_COMMAND_PAUSE_DIMMED + 1;
pub const KEY_COMMAND_REPLAY_TURBO: u16 = KEY_COMMAND_TURBO + 1;
pub const KEY_COMMAND_WINDOW_BIGGER: u16 = KEY_COMMAND_REPLAY_TURBO + 1;
pub const KEY_COMMAND_WINDOW_SMALLER: u16 = KEY_COMMAND_WINDOW_BIGGER + 1;
pub const KEY_COMMAND_DISPLAY_PERF: u16 = KEY_COMMAND_WINDOW_SMALLER + 1;
pub const KEY_COMMAND_TOGGLE_RENDERER: u16 = KEY_COMMAND_DISPLAY_PERF + 1;
pub const KEY_COMMAND_VOLUME_UP: u16 = KEY_COMMAND_TOGGLE_RENDERER + 1;
pub const KEY_COMMAND_VOLUME_DOWN: u16 = KEY_COMMAND_VOLUME_UP + 1;
pub const KEY_COMMAND_TOTAL: usize = KEY_COMMAND_VOLUME_DOWN as usize + 1;

pub const OUTPUT_METHOD_SDL: u8 = 0;
pub const OUTPUT_METHOD_SDL_SOFTWARE: u8 = 1;
pub const OUTPUT_METHOD_OPENGL: u8 = 2;
pub const OUTPUT_METHOD_OPENGL_ES: u8 = 3;

pub const MSU_FEATURE_MSU: u8 = 1;
pub const MSU_FEATURE_MSU_DELUXE: u8 = 2;
pub const MSU_FEATURE_OPUZ: u8 = 4;

pub const GAMEPAD_BUTTON_INVALID: i32 = -1;
pub const GAMEPAD_BUTTON_A: usize = 0;
pub const GAMEPAD_BUTTON_B: usize = 1;
pub const GAMEPAD_BUTTON_X: usize = 2;
pub const GAMEPAD_BUTTON_Y: usize = 3;
pub const GAMEPAD_BUTTON_BACK: usize = 4;
pub const GAMEPAD_BUTTON_GUIDE: usize = 5;
pub const GAMEPAD_BUTTON_START: usize = 6;
pub const GAMEPAD_BUTTON_L3: usize = 7;
pub const GAMEPAD_BUTTON_R3: usize = 8;
pub const GAMEPAD_BUTTON_L1: usize = 9;
pub const GAMEPAD_BUTTON_R1: usize = 10;
pub const GAMEPAD_BUTTON_DPAD_UP: usize = 11;
pub const GAMEPAD_BUTTON_DPAD_DOWN: usize = 12;
pub const GAMEPAD_BUTTON_DPAD_LEFT: usize = 13;
pub const GAMEPAD_BUTTON_DPAD_RIGHT: usize = 14;
pub const GAMEPAD_BUTTON_L2: usize = 15;
pub const GAMEPAD_BUTTON_R2: usize = 16;
pub const GAMEPAD_BUTTON_COUNT: usize = 17;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub window_width: i32,
    pub window_height: i32,
    pub enhanced_mode7: bool,
    pub new_renderer: bool,
    pub ignore_aspect_ratio: bool,
    pub fullscreen: u8,
    pub window_scale: u8,
    pub enable_audio: bool,
    pub linear_filtering: bool,
    pub output_method: u8,
    pub audio_freq: u16,
    pub audio_channels: u8,
    pub audio_samples: u16,
    pub autosave: bool,
    pub extended_aspect_ratio: u8,
    pub extend_y: bool,
    pub no_sprite_limits: bool,
    pub display_perf_title: bool,
    pub enable_msu: u8,
    pub resume_msu: bool,
    pub disable_frame_delay: bool,
    pub msuvolume: u8,
    pub features0: u32,
    pub link_graphics: Option<String>,
    pub shader: Option<String>,
    pub msu_path: Option<String>,
    pub language: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            window_width: 0,
            window_height: 0,
            enhanced_mode7: false,
            new_renderer: false,
            ignore_aspect_ratio: false,
            fullscreen: 0,
            window_scale: 0,
            enable_audio: false,
            linear_filtering: false,
            output_method: OUTPUT_METHOD_SDL,
            audio_freq: 0,
            audio_channels: 0,
            audio_samples: 0,
            autosave: false,
            extended_aspect_ratio: 0,
            extend_y: false,
            no_sprite_limits: false,
            display_perf_title: false,
            enable_msu: 0,
            resume_msu: false,
            disable_frame_delay: false,
            msuvolume: 0,
            features0: 0,
            link_graphics: None,
            shader: None,
            msu_path: None,
            language: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct KeyNameId {
    name: &'static str,
    id: u16,
    size: u16,
}

const KEY_NAME_ID: [KeyNameId; 25] = [
    KeyNameId {
        name: "Null",
        id: KEY_COMMAND_NULL,
        size: 65535,
    },
    key_range("Controls", KEY_COMMAND_CONTROLS, KEY_COMMAND_CONTROLS_LAST),
    key_range("Load", KEY_COMMAND_LOAD, KEY_COMMAND_LOAD_LAST),
    key_range("Save", KEY_COMMAND_SAVE, KEY_COMMAND_SAVE_LAST),
    key_range("Replay", KEY_COMMAND_REPLAY, KEY_COMMAND_REPLAY_LAST),
    key_range("LoadRef", KEY_COMMAND_LOAD_REF, KEY_COMMAND_LOAD_REF_LAST),
    key_range(
        "ReplayRef",
        KEY_COMMAND_REPLAY_REF,
        KEY_COMMAND_REPLAY_REF_LAST,
    ),
    key_single("CheatLife", KEY_COMMAND_CHEAT_LIFE),
    key_single("CheatKeys", KEY_COMMAND_CHEAT_KEYS),
    key_single("CheatEquipment", KEY_COMMAND_CHEAT_EQUIPMENT),
    key_single(
        "CheatWalkThroughWalls",
        KEY_COMMAND_CHEAT_WALK_THROUGH_WALLS,
    ),
    key_single("ClearKeyLog", KEY_COMMAND_CLEAR_KEY_LOG),
    key_single("StopReplay", KEY_COMMAND_STOP_REPLAY),
    key_single("Fullscreen", KEY_COMMAND_FULLSCREEN),
    key_single("Reset", KEY_COMMAND_RESET),
    key_single("Pause", KEY_COMMAND_PAUSE),
    key_single("PauseDimmed", KEY_COMMAND_PAUSE_DIMMED),
    key_single("Turbo", KEY_COMMAND_TURBO),
    key_single("ReplayTurbo", KEY_COMMAND_REPLAY_TURBO),
    key_single("WindowBigger", KEY_COMMAND_WINDOW_BIGGER),
    key_single("WindowSmaller", KEY_COMMAND_WINDOW_SMALLER),
    key_single("VolumeUp", KEY_COMMAND_VOLUME_UP),
    key_single("VolumeDown", KEY_COMMAND_VOLUME_DOWN),
    key_single("DisplayPerf", KEY_COMMAND_DISPLAY_PERF),
    key_single("ToggleRenderer", KEY_COMMAND_TOGGLE_RENDERER),
];

const fn key_range(name: &'static str, first: u16, last: u16) -> KeyNameId {
    KeyNameId {
        name,
        id: first,
        size: last - first + 1,
    }
}

const fn key_single(name: &'static str, id: u16) -> KeyNameId {
    KeyNameId { name, id, size: 1 }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct KeyMapHashEnt {
    key: u16,
    cmd: u16,
    next: u16,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct GamepadMapEnt {
    modifiers: u32,
    cmd: u16,
    next: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigContext {
    pub config: Config,
    keymap_hash_first: [u16; 255],
    keymap_hash: Vec<KeyMapHashEnt>,
    has_keynameid: [bool; KEY_NAME_ID.len()],
    joymap_first: [u16; GAMEPAD_BUTTON_COUNT],
    joymap_ents: Vec<GamepadMapEnt>,
    has_joypad_controls: bool,
}

impl Default for ConfigContext {
    fn default() -> Self {
        Self {
            config: Config::default(),
            keymap_hash_first: [0; 255],
            keymap_hash: Vec::new(),
            has_keynameid: [false; KEY_NAME_ID.len()],
            joymap_first: [0; GAMEPAD_BUTTON_COUNT],
            joymap_ents: Vec::new(),
            has_joypad_controls: false,
        }
    }
}

const fn remap_sdl_keycode(key: SdlKeycode) -> u16 {
    let scancode = if key & SDLK_SCANCODE_MASK != 0 {
        KEY_MODIFIER_SCAN_CODE
    } else {
        0
    };
    scancode | ((key as u16) & (KEY_MODIFIER_SCAN_CODE - 1))
}

const fn key(key: SdlKeycode) -> u16 {
    remap_sdl_keycode(key)
}

const fn shift(key: SdlKeycode) -> u16 {
    remap_sdl_keycode(key) | KEY_MODIFIER_SHIFT
}

const fn alt(key: SdlKeycode) -> u16 {
    remap_sdl_keycode(key) | KEY_MODIFIER_ALT
}

const fn ctrl(key: SdlKeycode) -> u16 {
    remap_sdl_keycode(key) | KEY_MODIFIER_CTRL
}

const N: u16 = 0;

const DEFAULT_KBD_CONTROLS: [u16; KEY_COMMAND_TOTAL] = {
    let mut a = [0; KEY_COMMAND_TOTAL];
    a[KEY_COMMAND_CONTROLS as usize] = key(SDLK_UP);
    a[KEY_COMMAND_CONTROLS as usize + 1] = key(SDLK_DOWN);
    a[KEY_COMMAND_CONTROLS as usize + 2] = key(SDLK_LEFT);
    a[KEY_COMMAND_CONTROLS as usize + 3] = key(SDLK_RIGHT);
    a[KEY_COMMAND_CONTROLS as usize + 4] = key(SDLK_RSHIFT);
    a[KEY_COMMAND_CONTROLS as usize + 5] = key(SDLK_RETURN);
    a[KEY_COMMAND_CONTROLS as usize + 6] = key(b'x' as SdlKeycode);
    a[KEY_COMMAND_CONTROLS as usize + 7] = key(b'z' as SdlKeycode);
    a[KEY_COMMAND_CONTROLS as usize + 8] = key(b's' as SdlKeycode);
    a[KEY_COMMAND_CONTROLS as usize + 9] = key(b'a' as SdlKeycode);
    a[KEY_COMMAND_CONTROLS as usize + 10] = key(b'c' as SdlKeycode);
    a[KEY_COMMAND_CONTROLS as usize + 11] = key(b'v' as SdlKeycode);

    let fkeys = [
        SDLK_F1, SDLK_F2, SDLK_F3, SDLK_F4, SDLK_F5, SDLK_F6, SDLK_F7, SDLK_F8, SDLK_F9, SDLK_F10,
    ];
    let mut i = 0;
    while i < 10 {
        a[KEY_COMMAND_LOAD as usize + i] = key(fkeys[i]);
        a[KEY_COMMAND_SAVE as usize + i] = shift(fkeys[i]);
        a[KEY_COMMAND_REPLAY as usize + i] = ctrl(fkeys[i]);
        i += 1;
    }

    a[KEY_COMMAND_CHEAT_LIFE as usize] = key(b'w' as SdlKeycode);
    a[KEY_COMMAND_CHEAT_KEYS as usize] = key(b'o' as SdlKeycode);
    a[KEY_COMMAND_CHEAT_EQUIPMENT as usize] = shift(b'w' as SdlKeycode);
    a[KEY_COMMAND_CHEAT_WALK_THROUGH_WALLS as usize] = ctrl(b'e' as SdlKeycode);
    a[KEY_COMMAND_CLEAR_KEY_LOG as usize] = key(b'k' as SdlKeycode);
    a[KEY_COMMAND_STOP_REPLAY as usize] = key(b'l' as SdlKeycode);
    a[KEY_COMMAND_FULLSCREEN as usize] = alt(SDLK_RETURN);
    a[KEY_COMMAND_RESET as usize] = ctrl(b'r' as SdlKeycode);
    a[KEY_COMMAND_PAUSE as usize] = shift(b'p' as SdlKeycode);
    a[KEY_COMMAND_PAUSE_DIMMED as usize] = key(b'p' as SdlKeycode);
    a[KEY_COMMAND_TURBO as usize] = key(SDLK_TAB);
    a[KEY_COMMAND_REPLAY_TURBO as usize] = key(b't' as SdlKeycode);
    a[KEY_COMMAND_WINDOW_BIGGER as usize] = N;
    a[KEY_COMMAND_WINDOW_SMALLER as usize] = N;
    a[KEY_COMMAND_DISPLAY_PERF as usize] = key(b'f' as SdlKeycode);
    a[KEY_COMMAND_TOGGLE_RENDERER as usize] = key(b'r' as SdlKeycode);
    a
};

const DEFAULT_GAMEPAD_CMDS: [usize; 12] = [
    GAMEPAD_BUTTON_DPAD_UP,
    GAMEPAD_BUTTON_DPAD_DOWN,
    GAMEPAD_BUTTON_DPAD_LEFT,
    GAMEPAD_BUTTON_DPAD_RIGHT,
    GAMEPAD_BUTTON_BACK,
    GAMEPAD_BUTTON_START,
    GAMEPAD_BUTTON_B,
    GAMEPAD_BUTTON_A,
    GAMEPAD_BUTTON_Y,
    GAMEPAD_BUTTON_X,
    GAMEPAD_BUTTON_L1,
    GAMEPAD_BUTTON_R1,
];

fn cstr_to_string(s: *const i8) -> String {
    if s.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(s).to_string_lossy().into_owned() }
    }
}

impl ConfigContext {
    fn key_map_hash_add(&mut self, key: u16, cmd: u16) -> bool {
        // C grows keymap_hash in 256-entry chunks and only dies at that
        // boundary, after the size has crossed 10000.
        if (self.keymap_hash.len() & 0xff) == 0 && self.keymap_hash.len() > 10000 {
            panic!("Too many keys");
        }
        let j = key as usize % 255;
        let i = self.keymap_hash.len();
        self.keymap_hash.push(KeyMapHashEnt { key, cmd, next: 0 });

        let mut cur = self.keymap_hash_first[j];
        while cur != 0 {
            let ent = self.keymap_hash[(cur - 1) as usize];
            if ent.key == key {
                return false;
            }
            cur = ent.next;
        }

        let mut slot = &mut self.keymap_hash_first[j];
        while *slot != 0 {
            let next_index = (*slot - 1) as usize;
            slot = &mut self.keymap_hash[next_index].next;
        }
        *slot = i as u16 + 1;
        true
    }

    fn key_map_hash_find(&self, key: u16) -> i32 {
        let mut i = self.keymap_hash_first[key as usize % 255];
        while i != 0 {
            let ent = self.keymap_hash[(i - 1) as usize];
            if ent.key == key {
                return ent.cmd as i32;
            }
            i = ent.next;
        }
        0
    }

    pub fn find_cmd_for_sdl_key(&self, code: SdlKeycode, keymod: SdlKeymod) -> i32 {
        if code & !(SDLK_SCANCODE_MASK | 0x01ff) != 0 {
            return 0;
        }
        let mut key = 0;
        if code != SDLK_LALT && code != SDLK_RALT {
            key |= if keymod & KMOD_ALT != 0 {
                KEY_MODIFIER_ALT
            } else {
                0
            };
        }
        if code != SDLK_LCTRL && code != SDLK_RCTRL {
            key |= if keymod & KMOD_CTRL != 0 {
                KEY_MODIFIER_CTRL
            } else {
                0
            };
        }
        if code != SDLK_LSHIFT && code != SDLK_RSHIFT {
            key |= if keymod & KMOD_SHIFT != 0 {
                KEY_MODIFIER_SHIFT
            } else {
                0
            };
        }
        key |= remap_sdl_keycode(code);
        self.key_map_hash_find(key)
    }

    fn parse_key_array(&mut self, value: &str, mut cmd: u16, size: u16) {
        let mut value = Some(value);
        let mut i = 0;
        while i < size {
            let Some(mut s) = NextDelim(&mut value, ',') else {
                break;
            };
            if !s.is_empty() {
                let mut key_with_mod = 0;
                loop {
                    if let Some(rest) = StringStartsWithNoCase(s, "Shift+") {
                        key_with_mod |= KEY_MODIFIER_SHIFT;
                        s = rest;
                    } else if let Some(rest) = StringStartsWithNoCase(s, "Ctrl+") {
                        key_with_mod |= KEY_MODIFIER_CTRL;
                        s = rest;
                    } else if let Some(rest) = StringStartsWithNoCase(s, "Alt+") {
                        key_with_mod |= KEY_MODIFIER_ALT;
                        s = rest;
                    } else {
                        break;
                    }
                }
                let key = sdl_get_key_from_name(s);
                if key != SDLK_UNKNOWN {
                    self.key_map_hash_add(key_with_mod | remap_sdl_keycode(key), cmd);
                }
            }
            i += 1;
            if cmd != 0 {
                cmd += 1;
            }
        }
    }

    fn gamepad_map_add(&mut self, button: usize, modifiers: u32, cmd: u16) {
        // C checks this limit only when joymap_size is on its allocation
        // boundary, not before every inserted entry.
        if (self.joymap_ents.len() & 0xff) == 0 && self.joymap_ents.len() > 1000 {
            panic!("Too many joypad keys");
        }
        let cb = count_bits32(modifiers);
        let mut entries = Vec::new();
        let mut e = self.joymap_first[button];
        while e != 0 {
            let ent = self.joymap_ents[(e - 1) as usize];
            entries.push((e, ent));
            e = ent.next;
        }
        let insert_before = entries
            .iter()
            .position(|(_, ent)| cb >= count_bits32(ent.modifiers))
            .unwrap_or(entries.len());

        let new_index = self.joymap_ents.len() as u16 + 1;
        let next = entries
            .get(insert_before)
            .map(|(index, _)| *index)
            .unwrap_or(0);
        self.joymap_ents.push(GamepadMapEnt {
            modifiers,
            cmd,
            next,
        });

        if insert_before == 0 {
            self.joymap_first[button] = new_index;
        } else {
            let prev = entries[insert_before - 1].0;
            self.joymap_ents[(prev - 1) as usize].next = new_index;
        }
    }

    pub fn find_cmd_for_gamepad_button(&self, button: usize, modifiers: u32) -> i32 {
        let mut e = self.joymap_first[button];
        while e != 0 {
            let ent = self.joymap_ents[(e - 1) as usize];
            if modifiers & ent.modifiers == ent.modifiers {
                return ent.cmd as i32;
            }
            e = ent.next;
        }
        0
    }

    fn parse_gamepad_array(&mut self, value: &str, mut cmd: u16, size: u16) {
        let mut value = Some(value);
        let mut i = 0;
        while i < size {
            let Some(s) = NextDelim(&mut value, ',') else {
                break;
            };
            if !s.is_empty() {
                let mut modifiers = 0;
                let mut ss = s;
                loop {
                    let button = parse_gamepad_button_name_str(&mut ss);
                    if button == GAMEPAD_BUTTON_INVALID {
                        break;
                    }
                    ss = ss.trim_start_matches([' ', '\t']);
                    if let Some(rest) = ss.strip_prefix('+') {
                        ss = rest;
                        modifiers |= 1 << button;
                    } else if ss.is_empty() {
                        self.gamepad_map_add(button as usize, modifiers, cmd);
                        break;
                    } else {
                        break;
                    }
                }
            }
            i += 1;
            if cmd != 0 {
                cmd += 1;
            }
        }
    }

    fn register_default_keys(&mut self) {
        for (i, item) in KEY_NAME_ID.iter().enumerate().skip(1) {
            if !self.has_keynameid[i] {
                let mut k = item.id;
                for _ in 0..item.size {
                    self.key_map_hash_add(DEFAULT_KBD_CONTROLS[k as usize], k);
                    k += 1;
                }
            }
        }
        if !self.has_joypad_controls {
            for (i, &button) in DEFAULT_GAMEPAD_CMDS.iter().enumerate() {
                self.gamepad_map_add(button, 0, KEY_COMMAND_CONTROLS + i as u16);
            }
        }
    }

    fn handle_ini_config(&mut self, section: i32, key: &str, value: &str) -> bool {
        if section == 0 {
            for (i, item) in KEY_NAME_ID.iter().enumerate() {
                if StringEqualsNoCase(key, item.name) {
                    self.has_keynameid[i] = true;
                    self.parse_key_array(value, item.id, item.size);
                    return true;
                }
            }
        } else if section == 5 {
            for (i, item) in KEY_NAME_ID.iter().enumerate() {
                if StringEqualsNoCase(key, item.name) {
                    if i == 1 {
                        self.has_joypad_controls = true;
                    }
                    self.parse_gamepad_array(value, item.id, item.size);
                    return true;
                }
            }
        } else if section == 1 {
            if StringEqualsNoCase(key, "WindowSize") {
                if StringEqualsNoCase(value, "Auto") {
                    self.config.window_width = 0;
                    self.config.window_height = 0;
                    return true;
                }
                let mut parts = Some(value);
                while let Some(s) = NextDelim(&mut parts, 'x') {
                    if self.config.window_width == 0 {
                        self.config.window_width = parse_i32(s);
                    } else {
                        self.config.window_height = parse_i32(s);
                        return true;
                    }
                }
            } else if StringEqualsNoCase(key, "EnhancedMode7") {
                return parse_bool_str(value, Some(&mut self.config.enhanced_mode7));
            } else if StringEqualsNoCase(key, "NewRenderer") {
                return parse_bool_str(value, Some(&mut self.config.new_renderer));
            } else if StringEqualsNoCase(key, "IgnoreAspectRatio") {
                return parse_bool_str(value, Some(&mut self.config.ignore_aspect_ratio));
            } else if StringEqualsNoCase(key, "Fullscreen") {
                self.config.fullscreen = parse_i32(value) as u8;
                return true;
            } else if StringEqualsNoCase(key, "WindowScale") {
                self.config.window_scale = parse_i32(value) as u8;
                return true;
            } else if StringEqualsNoCase(key, "OutputMethod") {
                self.config.output_method = if StringEqualsNoCase(value, "SDL-Software") {
                    OUTPUT_METHOD_SDL_SOFTWARE
                } else if StringEqualsNoCase(value, "OpenGL") {
                    OUTPUT_METHOD_OPENGL
                } else if StringEqualsNoCase(value, "OpenGL ES") {
                    OUTPUT_METHOD_OPENGL_ES
                } else {
                    OUTPUT_METHOD_SDL
                };
                return true;
            } else if StringEqualsNoCase(key, "LinearFiltering") {
                return parse_bool_str(value, Some(&mut self.config.linear_filtering));
            } else if StringEqualsNoCase(key, "NoSpriteLimits") {
                return parse_bool_str(value, Some(&mut self.config.no_sprite_limits));
            } else if StringEqualsNoCase(key, "LinkGraphics") {
                self.config.link_graphics = Some(value.to_string());
                return true;
            } else if StringEqualsNoCase(key, "Shader") {
                self.config.shader = if value.is_empty() {
                    None
                } else {
                    Some(value.to_string())
                };
                return true;
            } else if StringEqualsNoCase(key, "DimFlashes") {
                return parse_bool_bit_str(value, &mut self.config.features0, FEATURE_DIM_FLASHES);
            }
        } else if section == 2 {
            if StringEqualsNoCase(key, "EnableAudio") {
                return parse_bool_str(value, Some(&mut self.config.enable_audio));
            } else if StringEqualsNoCase(key, "AudioFreq") {
                self.config.audio_freq = parse_i32(value) as u16;
                return true;
            } else if StringEqualsNoCase(key, "AudioChannels") {
                self.config.audio_channels = parse_i32(value) as u8;
                return true;
            } else if StringEqualsNoCase(key, "AudioSamples") {
                self.config.audio_samples = parse_i32(value) as u16;
                return true;
            } else if StringEqualsNoCase(key, "EnableMSU") {
                if StringEqualsNoCase(value, "opuz") {
                    self.config.enable_msu = MSU_FEATURE_OPUZ;
                } else if StringEqualsNoCase(value, "deluxe") {
                    self.config.enable_msu = MSU_FEATURE_MSU_DELUXE;
                } else if StringEqualsNoCase(value, "deluxe-opuz") {
                    self.config.enable_msu = MSU_FEATURE_MSU_DELUXE | MSU_FEATURE_OPUZ;
                } else {
                    let mut enabled = self.config.enable_msu != 0;
                    if !parse_bool_str(value, Some(&mut enabled)) {
                        return false;
                    }
                    self.config.enable_msu = u8::from(enabled);
                }
                return true;
            } else if StringEqualsNoCase(key, "MSUPath") {
                self.config.msu_path = Some(value.to_string());
                return true;
            } else if StringEqualsNoCase(key, "MSUVolume") {
                self.config.msuvolume = parse_i32(value) as u8;
                return true;
            } else if StringEqualsNoCase(key, "ResumeMSU") {
                return parse_bool_str(value, Some(&mut self.config.resume_msu));
            }
        } else if section == 3 {
            if StringEqualsNoCase(key, "Autosave") {
                self.config.autosave = parse_i32(value) != 0;
                return true;
            } else if StringEqualsNoCase(key, "ExtendedAspectRatio") {
                let mut value = Some(value);
                let mut h = 224u32;
                let mut nospr = false;
                let mut novis = false;
                while let Some(s) = NextDelim(&mut value, ',') {
                    if s == "extend_y" {
                        h = 240;
                        self.config.extend_y = true;
                    } else if s == "16:9" {
                        self.config.extended_aspect_ratio = ((h * 16 / 9 - 256) / 2) as u8;
                    } else if s == "16:10" {
                        self.config.extended_aspect_ratio = ((h * 16 / 10 - 256) / 2) as u8;
                    } else if s == "18:9" {
                        self.config.extended_aspect_ratio = ((h * 18 / 9 - 256) / 2) as u8;
                    } else if s == "4:3" {
                        self.config.extended_aspect_ratio = 0;
                    } else if s == "unchanged_sprites" {
                        nospr = true;
                    } else if s == "no_visual_fixes" {
                        novis = true;
                    } else {
                        return false;
                    }
                }
                if self.config.extended_aspect_ratio != 0 && !nospr {
                    self.config.features0 |= FEATURE_EXTEND_SCREEN64;
                }
                if self.config.extended_aspect_ratio != 0 && !novis {
                    self.config.features0 |= FEATURE_WIDESCREEN_VISUAL_FIXES;
                }
                return true;
            } else if StringEqualsNoCase(key, "DisplayPerfInTitle") {
                return parse_bool_str(value, Some(&mut self.config.display_perf_title));
            } else if StringEqualsNoCase(key, "DisableFrameDelay") {
                return parse_bool_str(value, Some(&mut self.config.disable_frame_delay));
            } else if StringEqualsNoCase(key, "Language") {
                self.config.language = Some(value.to_string());
                return true;
            }
        } else if section == 4 {
            let mask = if StringEqualsNoCase(key, "ItemSwitchLR") {
                Some(FEATURE_SWITCH_LR)
            } else if StringEqualsNoCase(key, "ItemSwitchLRLimit") {
                Some(FEATURE_SWITCH_LR_LIMIT)
            } else if StringEqualsNoCase(key, "TurnWhileDashing") {
                Some(FEATURE_TURN_WHILE_DASHING)
            } else if StringEqualsNoCase(key, "MirrorToDarkworld") {
                Some(FEATURE_MIRROR_TO_DARKWORLD)
            } else if StringEqualsNoCase(key, "CollectItemsWithSword") {
                Some(FEATURE_COLLECT_ITEMS_WITH_SWORD)
            } else if StringEqualsNoCase(key, "BreakPotsWithSword") {
                Some(FEATURE_BREAK_POTS_WITH_SWORD)
            } else if StringEqualsNoCase(key, "DisableLowHealthBeep") {
                Some(FEATURE_DISABLE_LOW_HEALTH_BEEP)
            } else if StringEqualsNoCase(key, "SkipIntroOnKeypress") {
                Some(FEATURE_SKIP_INTRO_ON_KEYPRESS)
            } else if StringEqualsNoCase(key, "ShowMaxItemsInYellow") {
                Some(FEATURE_SHOW_MAX_ITEMS_IN_YELLOW)
            } else if StringEqualsNoCase(key, "MoreActiveBombs") {
                Some(FEATURE_MORE_ACTIVE_BOMBS)
            } else if StringEqualsNoCase(key, "CarryMoreRupees") {
                Some(FEATURE_CARRY_MORE_RUPEES)
            } else if StringEqualsNoCase(key, "MiscBugFixes") {
                Some(FEATURE_MISC_BUG_FIXES)
            } else if StringEqualsNoCase(key, "GameChangingBugFixes") {
                Some(FEATURE_GAME_CHANGING_BUG_FIXES)
            } else if StringEqualsNoCase(key, "CancelBirdTravel") {
                Some(FEATURE_CANCEL_BIRD_TRAVEL)
            } else {
                None
            };
            if let Some(mask) = mask {
                return parse_bool_bit_str(value, &mut self.config.features0, mask);
            }
        }
        false
    }

    fn parse_one_config_file(&mut self, filename: &str, depth: i32) -> bool {
        let Ok(mut bytes) = fs::read(filename) else {
            return false;
        };
        if let Some(nul) = bytes.iter().position(|&b| b == 0) {
            bytes.truncate(nul);
        }
        let filedata: String = bytes.iter().map(|&b| char::from(b)).collect();
        let mut filedata_ref = Some(filedata.as_ref());
        let mut section = -2;
        let mut lineno = 1;
        while let Some(p) = NextLineStripComments(&mut filedata_ref) {
            let current_lineno = lineno;
            lineno += 1;
            if p.is_empty() {
                continue;
            }
            if p.starts_with('[') {
                section = get_ini_section_str(p);
                if section < 0 {
                    eprintln!("{filename}:{current_lineno}: Invalid .ini section {p}");
                }
            } else if let Some(include) =
                p.strip_prefix('!').and_then(|s| s.strip_prefix("include "))
            {
                let mut include = include;
                let new_filename =
                    ReplaceFilenameWithNewPath(filename, NextPossiblyQuotedString(&mut include));
                if depth > 10 || !self.parse_one_config_file(&new_filename, depth + 1) {
                    eprintln!("Warning: Unable to read {new_filename}");
                }
            } else if section == -2 {
                eprintln!("{filename}:{current_lineno}: Expecting [section]");
            } else {
                if let Some((key, value)) = SplitKeyValue(p) {
                    if section >= 0
                        && !self.handle_ini_config(section, key, value.trim_end_matches('\0'))
                    {
                        eprintln!("{filename}:{current_lineno}: Can't parse '{p}'");
                    }
                } else {
                    eprintln!("{filename}:{current_lineno}: Expecting 'key=value'");
                }
            }
        }
        true
    }

    pub fn parse_config_file(&mut self, filename: Option<&str>) {
        self.config.msuvolume = 100;
        if filename.is_some() || !self.parse_one_config_file("zelda3.user.ini", 0) {
            let filename = filename.unwrap_or("zelda3.ini");
            self.parse_one_config_file(filename, 0);
        }
        self.register_default_keys();
    }
}

fn count_bits32(mut n: u32) -> i32 {
    let mut count = 0;
    while n != 0 {
        count += 1;
        n &= n - 1;
    }
    count
}

fn parse_gamepad_button_name_str(value: &mut &str) -> i32 {
    const NAMES: [(&str, usize); 19] = [
        ("Back", GAMEPAD_BUTTON_BACK),
        ("Guide", GAMEPAD_BUTTON_GUIDE),
        ("Start", GAMEPAD_BUTTON_START),
        ("L3", GAMEPAD_BUTTON_L3),
        ("R3", GAMEPAD_BUTTON_R3),
        ("L1", GAMEPAD_BUTTON_L1),
        ("R1", GAMEPAD_BUTTON_R1),
        ("DpadUp", GAMEPAD_BUTTON_DPAD_UP),
        ("DpadDown", GAMEPAD_BUTTON_DPAD_DOWN),
        ("DpadLeft", GAMEPAD_BUTTON_DPAD_LEFT),
        ("DpadRight", GAMEPAD_BUTTON_DPAD_RIGHT),
        ("L2", GAMEPAD_BUTTON_L2),
        ("R2", GAMEPAD_BUTTON_R2),
        ("Lb", GAMEPAD_BUTTON_L1),
        ("Rb", GAMEPAD_BUTTON_R1),
        ("A", GAMEPAD_BUTTON_A),
        ("B", GAMEPAD_BUTTON_B),
        ("X", GAMEPAD_BUTTON_X),
        ("Y", GAMEPAD_BUTTON_Y),
    ];
    for (name, id) in NAMES {
        if let Some(rest) = StringStartsWithNoCase(value, name) {
            *value = rest;
            return id as i32;
        }
    }
    GAMEPAD_BUTTON_INVALID
}

fn get_ini_section_str(s: &str) -> i32 {
    if StringEqualsNoCase(s, "[KeyMap]") {
        0
    } else if StringEqualsNoCase(s, "[Graphics]") {
        1
    } else if StringEqualsNoCase(s, "[Sound]") {
        2
    } else if StringEqualsNoCase(s, "[General]") {
        3
    } else if StringEqualsNoCase(s, "[Features]") {
        4
    } else if StringEqualsNoCase(s, "[GamepadMap]") {
        5
    } else {
        -1
    }
}

pub fn parse_bool(value: *const i8, result: *mut bool) -> bool {
    let mut tmp = false;
    if !parse_bool_str(&cstr_to_string(value), Some(&mut tmp)) {
        return false;
    }
    if !result.is_null() {
        unsafe { *result = tmp };
        true
    } else {
        tmp
    }
}

pub fn parse_bool_str(value: &str, result: Option<&mut bool>) -> bool {
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    let tail = &value[1..];
    let mut rv = false;
    match first | 32 {
        b'0' => {
            if !tail.is_empty() {
                return false;
            }
        }
        b'f' => {
            if !StringEqualsNoCase(tail, "alse") {
                return false;
            }
        }
        b'n' => {
            if !StringEqualsNoCase(tail, "o") {
                return false;
            }
        }
        b'o' => {
            rv = tail.bytes().next().map(|b| b | 32) == Some(b'n');
            if !StringEqualsNoCase(tail, if rv { "n" } else { "ff" }) {
                return false;
            }
        }
        b'1' => {
            rv = true;
            if !tail.is_empty() {
                return false;
            }
        }
        b'y' => {
            rv = true;
            if !StringEqualsNoCase(tail, "es") {
                return false;
            }
        }
        b't' => {
            rv = true;
            if !StringEqualsNoCase(tail, "rue") {
                return false;
            }
        }
        _ => return false,
    }
    if let Some(result) = result {
        *result = rv;
        true
    } else {
        rv
    }
}

fn parse_bool_bit_str(value: &str, data: &mut u32, mask: u32) -> bool {
    let mut tmp = false;
    if !parse_bool_str(value, Some(&mut tmp)) {
        return false;
    }
    *data = (*data & !mask) | if tmp { mask } else { 0 };
    true
}

// Names returned by SDL2 2.32.10's SDL_GetScancodeName that
// SDL_GetKeyFromName accepts as non-printable or scancode-backed keys.
const SDL_KEY_NAME_MAP: &[(&str, SdlKeycode)] = &[
    ("Return", 13),
    ("Escape", 27),
    ("Backspace", 8),
    ("Tab", 9),
    ("Space", 32),
    ("CapsLock", 1073741881),
    ("F1", 1073741882),
    ("F2", 1073741883),
    ("F3", 1073741884),
    ("F4", 1073741885),
    ("F5", 1073741886),
    ("F6", 1073741887),
    ("F7", 1073741888),
    ("F8", 1073741889),
    ("F9", 1073741890),
    ("F10", 1073741891),
    ("F11", 1073741892),
    ("F12", 1073741893),
    ("PrintScreen", 1073741894),
    ("ScrollLock", 1073741895),
    ("Pause", 1073741896),
    ("Insert", 1073741897),
    ("Home", 1073741898),
    ("PageUp", 1073741899),
    ("Delete", 127),
    ("End", 1073741901),
    ("PageDown", 1073741902),
    ("Right", 1073741903),
    ("Left", 1073741904),
    ("Down", 1073741905),
    ("Up", 1073741906),
    ("Numlock", 1073741907),
    ("Keypad /", 1073741908),
    ("Keypad *", 1073741909),
    ("Keypad -", 1073741910),
    ("Keypad +", 1073741911),
    ("Keypad Enter", 1073741912),
    ("Keypad 1", 1073741913),
    ("Keypad 2", 1073741914),
    ("Keypad 3", 1073741915),
    ("Keypad 4", 1073741916),
    ("Keypad 5", 1073741917),
    ("Keypad 6", 1073741918),
    ("Keypad 7", 1073741919),
    ("Keypad 8", 1073741920),
    ("Keypad 9", 1073741921),
    ("Keypad 0", 1073741922),
    ("Keypad .", 1073741923),
    ("Application", 1073741925),
    ("Power", 1073741926),
    ("Keypad =", 1073741927),
    ("F13", 1073741928),
    ("F14", 1073741929),
    ("F15", 1073741930),
    ("F16", 1073741931),
    ("F17", 1073741932),
    ("F18", 1073741933),
    ("F19", 1073741934),
    ("F20", 1073741935),
    ("F21", 1073741936),
    ("F22", 1073741937),
    ("F23", 1073741938),
    ("F24", 1073741939),
    ("Execute", 1073741940),
    ("Help", 1073741941),
    ("Menu", 1073741942),
    ("Select", 1073741943),
    ("Stop", 1073741944),
    ("Again", 1073741945),
    ("Undo", 1073741946),
    ("Cut", 1073741947),
    ("Copy", 1073741948),
    ("Paste", 1073741949),
    ("Find", 1073741950),
    ("Mute", 1073741951),
    ("VolumeUp", 1073741952),
    ("VolumeDown", 1073741953),
    ("Keypad ,", 1073741957),
    ("Keypad = (AS400)", 1073741958),
    ("AltErase", 1073741977),
    ("SysReq", 1073741978),
    ("Cancel", 1073741979),
    ("Clear", 1073741980),
    ("Prior", 1073741981),
    ("Separator", 1073741983),
    ("Out", 1073741984),
    ("Oper", 1073741985),
    ("Clear / Again", 1073741986),
    ("CrSel", 1073741987),
    ("ExSel", 1073741988),
    ("Keypad 00", 1073742000),
    ("Keypad 000", 1073742001),
    ("ThousandsSeparator", 1073742002),
    ("DecimalSeparator", 1073742003),
    ("CurrencyUnit", 1073742004),
    ("CurrencySubUnit", 1073742005),
    ("Keypad (", 1073742006),
    ("Keypad )", 1073742007),
    ("Keypad {", 1073742008),
    ("Keypad }", 1073742009),
    ("Keypad Tab", 1073742010),
    ("Keypad Backspace", 1073742011),
    ("Keypad A", 1073742012),
    ("Keypad B", 1073742013),
    ("Keypad C", 1073742014),
    ("Keypad D", 1073742015),
    ("Keypad E", 1073742016),
    ("Keypad F", 1073742017),
    ("Keypad XOR", 1073742018),
    ("Keypad ^", 1073742019),
    ("Keypad %", 1073742020),
    ("Keypad <", 1073742021),
    ("Keypad >", 1073742022),
    ("Keypad &", 1073742023),
    ("Keypad &&", 1073742024),
    ("Keypad |", 1073742025),
    ("Keypad ||", 1073742026),
    ("Keypad :", 1073742027),
    ("Keypad #", 1073742028),
    ("Keypad Space", 1073742029),
    ("Keypad @", 1073742030),
    ("Keypad !", 1073742031),
    ("Keypad MemStore", 1073742032),
    ("Keypad MemRecall", 1073742033),
    ("Keypad MemClear", 1073742034),
    ("Keypad MemAdd", 1073742035),
    ("Keypad MemSubtract", 1073742036),
    ("Keypad MemMultiply", 1073742037),
    ("Keypad MemDivide", 1073742038),
    ("Keypad +/-", 1073742039),
    ("Keypad Clear", 1073742040),
    ("Keypad ClearEntry", 1073742041),
    ("Keypad Binary", 1073742042),
    ("Keypad Octal", 1073742043),
    ("Keypad Decimal", 1073742044),
    ("Keypad Hexadecimal", 1073742045),
    ("Left Ctrl", 1073742048),
    ("Left Shift", 1073742049),
    ("Left Alt", 1073742050),
    ("Left GUI", 1073742051),
    ("Right Ctrl", 1073742052),
    ("Right Shift", 1073742053),
    ("Right Alt", 1073742054),
    ("Right GUI", 1073742055),
    ("ModeSwitch", 1073742081),
    ("AudioNext", 1073742082),
    ("AudioPrev", 1073742083),
    ("AudioStop", 1073742084),
    ("AudioPlay", 1073742085),
    ("AudioMute", 1073742086),
    ("MediaSelect", 1073742087),
    ("WWW", 1073742088),
    ("Mail", 1073742089),
    ("Calculator", 1073742090),
    ("Computer", 1073742091),
    ("AC Search", 1073742092),
    ("AC Home", 1073742093),
    ("AC Back", 1073742094),
    ("AC Forward", 1073742095),
    ("AC Stop", 1073742096),
    ("AC Refresh", 1073742097),
    ("AC Bookmarks", 1073742098),
    ("BrightnessDown", 1073742099),
    ("BrightnessUp", 1073742100),
    ("DisplaySwitch", 1073742101),
    ("KBDIllumToggle", 1073742102),
    ("KBDIllumDown", 1073742103),
    ("KBDIllumUp", 1073742104),
    ("Eject", 1073742105),
    ("Sleep", 1073742106),
    ("App1", 1073742107),
    ("App2", 1073742108),
    ("AudioRewind", 1073742109),
    ("AudioFastForward", 1073742110),
    ("SoftLeft", 1073742111),
    ("SoftRight", 1073742112),
    ("Call", 1073742113),
    ("EndCall", 1073742114),
];

fn sdl_get_key_from_name(s: &str) -> SdlKeycode {
    if s.len() == 1 {
        return s.as_bytes()[0].to_ascii_lowercase() as SdlKeycode;
    }
    for &(name, key) in SDL_KEY_NAME_MAP {
        if StringEqualsNoCase(s, name) {
            return key;
        }
    }
    SDLK_UNKNOWN
}

fn parse_i32(s: &str) -> i32 {
    let mut bytes = s.as_bytes();
    while let Some((&b, rest)) = bytes.split_first() {
        if !b.is_ascii_whitespace() {
            break;
        }
        bytes = rest;
    }

    let mut sign = 1i64;
    if let Some((&b, rest)) = bytes.split_first() {
        if b == b'-' {
            sign = -1;
            bytes = rest;
        } else if b == b'+' {
            bytes = rest;
        }
    }

    let mut seen_digit = false;
    let mut value = 0i64;
    for &b in bytes {
        if !b.is_ascii_digit() {
            break;
        }
        seen_digit = true;
        value = value.wrapping_mul(10).wrapping_add(i64::from(b - b'0'));
    }

    if seen_digit {
        value.wrapping_mul(sign) as i32
    } else {
        0
    }
}

pub fn parse_config_file_context(filename: Option<&str>) -> ConfigContext {
    let mut ctx = ConfigContext::default();
    ctx.parse_config_file(filename);
    ctx
}

pub fn parse_config_file_from_path(path: impl AsRef<Path>) -> ConfigContext {
    parse_config_file_context(path.as_ref().to_str())
}

pub fn config_value_bytes(value: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(value.len());
    for ch in value.chars() {
        let code = ch as u32;
        if code <= 0xff {
            bytes.push(code as u8);
        } else {
            let mut buf = [0; 4];
            bytes.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
        }
    }
    bytes
}

#[cfg(unix)]
pub fn config_value_path(value: &str) -> PathBuf {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    PathBuf::from(OsString::from_vec(config_value_bytes(value)))
}

#[cfg(not(unix))]
pub fn config_value_path(value: &str) -> PathBuf {
    PathBuf::from(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bool_parser_matches_c_truth_table() {
        let mut out = false;
        assert!(parse_bool_str("true", Some(&mut out)));
        assert!(out);
        assert!(parse_bool_str("off", Some(&mut out)));
        assert!(!out);
        assert!(parse_bool_str("yes", None));
        assert!(!parse_bool_str("maybe", Some(&mut out)));
        assert!(!parse_bool_str("10", Some(&mut out)));
    }

    #[test]
    fn integer_parser_accepts_c_numeric_prefixes() {
        assert_eq!(parse_i32("320abc"), 320);
        assert_eq!(parse_i32("  -12x"), -12);
        assert_eq!(parse_i32("+77junk"), 77);
        assert_eq!(parse_i32("x77"), 0);
    }

    #[test]
    fn config_value_bytes_preserve_parser_byte_identity() {
        let value: String = [b'm', b's', b'u', b'/', 0x80, b'.', b'p', b'c', b'm']
            .into_iter()
            .map(char::from)
            .collect();
        assert_eq!(
            config_value_bytes(&value),
            vec![b'm', b's', b'u', b'/', 0x80, b'.', b'p', b'c', b'm']
        );
    }

    #[test]
    fn default_keyboard_and_modified_lookup_match_c_shape() {
        let mut ctx = ConfigContext::default();
        ctx.parse_config_file(Some("__missing_config__"));

        assert_eq!(
            ctx.find_cmd_for_sdl_key(SDLK_UP, 0),
            KEY_COMMAND_CONTROLS as i32
        );
        assert_eq!(
            ctx.find_cmd_for_sdl_key(SDLK_RETURN, KMOD_ALT),
            KEY_COMMAND_FULLSCREEN as i32
        );
        assert_eq!(
            ctx.find_cmd_for_sdl_key(b'r' as SdlKeycode, KMOD_CTRL),
            KEY_COMMAND_RESET as i32
        );
        assert_eq!(
            ctx.find_cmd_for_sdl_key(SDLK_RSHIFT, KMOD_SHIFT),
            KEY_COMMAND_CONTROLS as i32 + 4
        );
    }

    #[test]
    fn gamepad_modifier_entries_precede_less_specific_entries() {
        let mut ctx = ConfigContext::default();
        ctx.parse_gamepad_array("L1+A,A", KEY_COMMAND_CONTROLS, 2);
        assert_eq!(
            ctx.find_cmd_for_gamepad_button(GAMEPAD_BUTTON_A, 1 << GAMEPAD_BUTTON_L1),
            KEY_COMMAND_CONTROLS as i32
        );
        assert_eq!(
            ctx.find_cmd_for_gamepad_button(GAMEPAD_BUTTON_A, 0),
            KEY_COMMAND_CONTROLS as i32 + 1
        );
    }

    #[test]
    fn sdl_key_name_resolver_matches_sdl_names() {
        assert_eq!(sdl_get_key_from_name("Return"), SDLK_RETURN);
        assert_eq!(sdl_get_key_from_name("Enter"), SDLK_UNKNOWN);
        assert_eq!(sdl_get_key_from_name("F24"), 1073741939);
        assert_eq!(sdl_get_key_from_name("Keypad Enter"), 1073741912);
        assert_eq!(sdl_get_key_from_name("Left Ctrl"), SDLK_LCTRL);
        assert_eq!(sdl_get_key_from_name("Right Shift"), SDLK_RSHIFT);
        assert_eq!(sdl_get_key_from_name("a"), b'a' as SdlKeycode);
        assert_eq!(sdl_get_key_from_name("A"), b'a' as SdlKeycode);
    }

    #[test]
    fn config_file_parses_sections_and_includes() {
        let dir =
            std::env::temp_dir().join(format!("zelda3-rs-config-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let child = dir.join("child.ini");
        let root = dir.join("root.ini");
        fs::write(
            &child,
            "[Features]\nItemSwitchLR = true\n[GamepadMap]\nControls = L1+A,A\n",
        )
        .unwrap();
        fs::write(
            &root,
            "!include child.ini\n[Graphics]\nWindowSize=320x240\nOutputMethod=OpenGL ES\nShader=\n[Sound]\nEnableMSU=deluxe-opuz\nMSUVolume=77\n[General]\nExtendedAspectRatio=extend_y,16:9\nLanguage=en\n[KeyMap]\nReset=Ctrl+r\n",
        )
        .unwrap();

        let mut ctx = ConfigContext::default();
        ctx.parse_config_file(Some(root.to_str().unwrap()));
        assert_eq!(ctx.config.window_width, 320);
        assert_eq!(ctx.config.window_height, 240);
        assert_eq!(ctx.config.output_method, OUTPUT_METHOD_OPENGL_ES);
        assert_eq!(ctx.config.shader, None);
        assert_eq!(
            ctx.config.enable_msu,
            MSU_FEATURE_MSU_DELUXE | MSU_FEATURE_OPUZ
        );
        assert_eq!(ctx.config.msuvolume, 77);
        assert!(ctx.config.extend_y);
        assert_ne!(ctx.config.features0 & FEATURE_SWITCH_LR, 0);
        assert_eq!(ctx.config.language.as_deref(), Some("en"));
        assert_eq!(
            ctx.find_cmd_for_sdl_key(b'r' as SdlKeycode, KMOD_CTRL),
            KEY_COMMAND_RESET as i32
        );
        assert_eq!(
            ctx.find_cmd_for_gamepad_button(GAMEPAD_BUTTON_A, 1 << GAMEPAD_BUTTON_L1),
            KEY_COMMAND_CONTROLS as i32
        );

        fs::remove_dir_all(&dir).unwrap();
    }
}
