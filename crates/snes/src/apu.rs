//! APU/SPC/DSP register surface. Port of `zelda3/snes/apu.c`,
//! `zelda3/snes/spc.c`, and the Rust-side `zelda3/snes/dsp.c` audio core.

use crate::cycle_spc700::{Smp, SmpBus, SmpCoroutineState, SmpState};
pub use crate::cycle_spc700::{
    SmpMicroStepResult, Snes9xSmpCoroutineCheckpoint, UnsupportedSmpMicroStep,
};

const BOOT_ROM: [u8; 0x40] = [
    0xcd, 0xef, 0xbd, 0xe8, 0x00, 0xc6, 0x1d, 0xd0, 0xfc, 0x8f, 0xaa, 0xf4, 0x8f, 0xbb, 0xf5, 0x78,
    0xcc, 0xf4, 0xd0, 0xfb, 0x2f, 0x19, 0xeb, 0xf4, 0xd0, 0xfc, 0x7e, 0xf4, 0xd0, 0x0b, 0xe4, 0xf5,
    0xcb, 0xf4, 0xd7, 0x00, 0xfc, 0xd0, 0xf3, 0xab, 0x01, 0x10, 0xef, 0x7e, 0xf4, 0x10, 0xeb, 0xba,
    0xf6, 0xda, 0x00, 0xba, 0xf4, 0xc4, 0xf4, 0xdd, 0x5d, 0xd0, 0xdb, 0x1f, 0x00, 0x00, 0xc0, 0xff,
];

const SPC_CYCLES_PER_OPCODE: [u8; 256] = [
    2, 8, 4, 5, 3, 4, 3, 6, 2, 6, 5, 4, 5, 4, 6, 8, 2, 8, 4, 5, 4, 5, 5, 6, 5, 5, 6, 5, 2, 2, 4, 6,
    2, 8, 4, 5, 3, 4, 3, 6, 2, 6, 5, 4, 5, 4, 5, 4, 2, 8, 4, 5, 4, 5, 5, 6, 5, 5, 6, 5, 2, 2, 3, 8,
    2, 8, 4, 5, 3, 4, 3, 6, 2, 6, 4, 4, 5, 4, 6, 6, 2, 8, 4, 5, 4, 5, 5, 6, 5, 5, 4, 5, 2, 2, 4, 3,
    2, 8, 4, 5, 3, 4, 3, 6, 2, 6, 4, 4, 5, 4, 5, 5, 2, 8, 4, 5, 4, 5, 5, 6, 5, 5, 5, 5, 2, 2, 3, 6,
    2, 8, 4, 5, 3, 4, 3, 6, 2, 6, 5, 4, 5, 2, 4, 5, 2, 8, 4, 5, 4, 5, 5, 6, 5, 5, 5, 5, 2, 2, 12,
    5, 2, 8, 4, 5, 3, 4, 3, 6, 2, 6, 4, 4, 5, 2, 4, 4, 2, 8, 4, 5, 4, 5, 5, 6, 5, 5, 5, 5, 2, 2, 3,
    4, 2, 8, 4, 5, 4, 5, 4, 7, 2, 5, 6, 4, 5, 2, 4, 9, 2, 8, 4, 5, 5, 6, 6, 7, 4, 5, 5, 5, 2, 2, 6,
    3, 2, 8, 4, 5, 3, 4, 3, 6, 2, 4, 5, 3, 4, 3, 4, 3, 2, 8, 4, 5, 4, 5, 5, 6, 3, 4, 5, 4, 2, 2, 4,
    3,
];

const DSP_RATE_VALUES: [u16; 32] = [
    0, 2048, 1536, 1280, 1024, 768, 640, 512, 384, 320, 256, 192, 160, 128, 96, 80, 64, 48, 40, 32,
    24, 20, 16, 12, 10, 8, 6, 5, 4, 3, 2, 1,
];

const DSP_GAUSS_VALUES: [i32; 512] = [
    0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000,
    0x000, 0x000, 0x000, 0x001, 0x001, 0x001, 0x001, 0x001, 0x001, 0x001, 0x001, 0x001, 0x001,
    0x001, 0x002, 0x002, 0x002, 0x002, 0x002, 0x002, 0x002, 0x003, 0x003, 0x003, 0x003, 0x003,
    0x004, 0x004, 0x004, 0x004, 0x004, 0x005, 0x005, 0x005, 0x005, 0x006, 0x006, 0x006, 0x006,
    0x007, 0x007, 0x007, 0x008, 0x008, 0x008, 0x009, 0x009, 0x009, 0x00A, 0x00A, 0x00A, 0x00B,
    0x00B, 0x00B, 0x00C, 0x00C, 0x00D, 0x00D, 0x00E, 0x00E, 0x00F, 0x00F, 0x00F, 0x010, 0x010,
    0x011, 0x011, 0x012, 0x013, 0x013, 0x014, 0x014, 0x015, 0x015, 0x016, 0x017, 0x017, 0x018,
    0x018, 0x019, 0x01A, 0x01B, 0x01B, 0x01C, 0x01D, 0x01D, 0x01E, 0x01F, 0x020, 0x020, 0x021,
    0x022, 0x023, 0x024, 0x024, 0x025, 0x026, 0x027, 0x028, 0x029, 0x02A, 0x02B, 0x02C, 0x02D,
    0x02E, 0x02F, 0x030, 0x031, 0x032, 0x033, 0x034, 0x035, 0x036, 0x037, 0x038, 0x03A, 0x03B,
    0x03C, 0x03D, 0x03E, 0x040, 0x041, 0x042, 0x043, 0x045, 0x046, 0x047, 0x049, 0x04A, 0x04C,
    0x04D, 0x04E, 0x050, 0x051, 0x053, 0x054, 0x056, 0x057, 0x059, 0x05A, 0x05C, 0x05E, 0x05F,
    0x061, 0x063, 0x064, 0x066, 0x068, 0x06A, 0x06B, 0x06D, 0x06F, 0x071, 0x073, 0x075, 0x076,
    0x078, 0x07A, 0x07C, 0x07E, 0x080, 0x082, 0x084, 0x086, 0x089, 0x08B, 0x08D, 0x08F, 0x091,
    0x093, 0x096, 0x098, 0x09A, 0x09C, 0x09F, 0x0A1, 0x0A3, 0x0A6, 0x0A8, 0x0AB, 0x0AD, 0x0AF,
    0x0B2, 0x0B4, 0x0B7, 0x0BA, 0x0BC, 0x0BF, 0x0C1, 0x0C4, 0x0C7, 0x0C9, 0x0CC, 0x0CF, 0x0D2,
    0x0D4, 0x0D7, 0x0DA, 0x0DD, 0x0E0, 0x0E3, 0x0E6, 0x0E9, 0x0EC, 0x0EF, 0x0F2, 0x0F5, 0x0F8,
    0x0FB, 0x0FE, 0x101, 0x104, 0x107, 0x10B, 0x10E, 0x111, 0x114, 0x118, 0x11B, 0x11E, 0x122,
    0x125, 0x129, 0x12C, 0x130, 0x133, 0x137, 0x13A, 0x13E, 0x141, 0x145, 0x148, 0x14C, 0x150,
    0x153, 0x157, 0x15B, 0x15F, 0x162, 0x166, 0x16A, 0x16E, 0x172, 0x176, 0x17A, 0x17D, 0x181,
    0x185, 0x189, 0x18D, 0x191, 0x195, 0x19A, 0x19E, 0x1A2, 0x1A6, 0x1AA, 0x1AE, 0x1B2, 0x1B7,
    0x1BB, 0x1BF, 0x1C3, 0x1C8, 0x1CC, 0x1D0, 0x1D5, 0x1D9, 0x1DD, 0x1E2, 0x1E6, 0x1EB, 0x1EF,
    0x1F3, 0x1F8, 0x1FC, 0x201, 0x205, 0x20A, 0x20F, 0x213, 0x218, 0x21C, 0x221, 0x226, 0x22A,
    0x22F, 0x233, 0x238, 0x23D, 0x241, 0x246, 0x24B, 0x250, 0x254, 0x259, 0x25E, 0x263, 0x267,
    0x26C, 0x271, 0x276, 0x27B, 0x280, 0x284, 0x289, 0x28E, 0x293, 0x298, 0x29D, 0x2A2, 0x2A6,
    0x2AB, 0x2B0, 0x2B5, 0x2BA, 0x2BF, 0x2C4, 0x2C9, 0x2CE, 0x2D3, 0x2D8, 0x2DC, 0x2E1, 0x2E6,
    0x2EB, 0x2F0, 0x2F5, 0x2FA, 0x2FF, 0x304, 0x309, 0x30E, 0x313, 0x318, 0x31D, 0x322, 0x326,
    0x32B, 0x330, 0x335, 0x33A, 0x33F, 0x344, 0x349, 0x34E, 0x353, 0x357, 0x35C, 0x361, 0x366,
    0x36B, 0x370, 0x374, 0x379, 0x37E, 0x383, 0x388, 0x38C, 0x391, 0x396, 0x39B, 0x39F, 0x3A4,
    0x3A9, 0x3AD, 0x3B2, 0x3B7, 0x3BB, 0x3C0, 0x3C5, 0x3C9, 0x3CE, 0x3D2, 0x3D7, 0x3DC, 0x3E0,
    0x3E5, 0x3E9, 0x3ED, 0x3F2, 0x3F6, 0x3FB, 0x3FF, 0x403, 0x408, 0x40C, 0x410, 0x415, 0x419,
    0x41D, 0x421, 0x425, 0x42A, 0x42E, 0x432, 0x436, 0x43A, 0x43E, 0x442, 0x446, 0x44A, 0x44E,
    0x452, 0x455, 0x459, 0x45D, 0x461, 0x465, 0x468, 0x46C, 0x470, 0x473, 0x477, 0x47A, 0x47E,
    0x481, 0x485, 0x488, 0x48C, 0x48F, 0x492, 0x496, 0x499, 0x49C, 0x49F, 0x4A2, 0x4A6, 0x4A9,
    0x4AC, 0x4AF, 0x4B2, 0x4B5, 0x4B7, 0x4BA, 0x4BD, 0x4C0, 0x4C3, 0x4C5, 0x4C8, 0x4CB, 0x4CD,
    0x4D0, 0x4D2, 0x4D5, 0x4D7, 0x4D9, 0x4DC, 0x4DE, 0x4E0, 0x4E3, 0x4E5, 0x4E7, 0x4E9, 0x4EB,
    0x4ED, 0x4EF, 0x4F1, 0x4F3, 0x4F5, 0x4F6, 0x4F8, 0x4FA, 0x4FB, 0x4FD, 0x4FF, 0x500, 0x502,
    0x503, 0x504, 0x506, 0x507, 0x508, 0x50A, 0x50B, 0x50C, 0x50D, 0x50E, 0x50F, 0x510, 0x511,
    0x511, 0x512, 0x513, 0x514, 0x514, 0x515, 0x516, 0x516, 0x517, 0x517, 0x517, 0x518, 0x518,
    0x518, 0x518, 0x518, 0x519, 0x519,
];

const DSP_MVOLL: u8 = 0x0c;
const DSP_EFB: u8 = 0x0d;
const DSP_FIR0: u8 = 0x0f;
const DSP_MVOLR: u8 = 0x1c;
const DSP_EVOLL: u8 = 0x2c;
const DSP_PMON: u8 = 0x2d;
const DSP_EVOLR: u8 = 0x3c;
const DSP_NON: u8 = 0x3d;
const DSP_KON: u8 = 0x4c;
const DSP_EON: u8 = 0x4d;
const DSP_KOF: u8 = 0x5c;
const DSP_DIR: u8 = 0x5d;
const DSP_FLG: u8 = 0x6c;
const DSP_ESA: u8 = 0x6d;
const DSP_ENDX: u8 = 0x7c;
const DSP_EDL: u8 = 0x7d;
const DSP_SAMPLE_BUFFER_LEN: usize = 534 * 2;
// S-DSP power-on register image used by Snes9x 1.63's SPC_DSP core. The
// Zelda driver intentionally reads the previous EDL value during echo setup,
// so these externally visible reset bytes are part of program state rather
// than renderer-specific implementation detail.
const DSP_POWER_ON_REGISTERS: [u8; 0x80] = [
    0x45, 0x8b, 0x5a, 0x9a, 0xe4, 0x82, 0x1b, 0x78, 0x00, 0x00, 0xaa, 0x96, 0x89, 0x0e, 0xe0, 0x80,
    0x2a, 0x49, 0x3d, 0xba, 0x14, 0xa0, 0xac, 0xc5, 0x00, 0x00, 0x51, 0xbb, 0x9c, 0x4e, 0x7b, 0xff,
    0xf4, 0xfd, 0x57, 0x32, 0x37, 0xd9, 0x42, 0x22, 0x00, 0x00, 0x5b, 0x3c, 0x9f, 0x1b, 0x87, 0x9a,
    0x6f, 0x27, 0xaf, 0x7b, 0xe5, 0x68, 0x0a, 0xd9, 0x00, 0x00, 0x9a, 0xc5, 0x9c, 0x4e, 0x7b, 0xff,
    0xea, 0x21, 0x78, 0x4f, 0xdd, 0xed, 0x24, 0x14, 0x00, 0x00, 0x77, 0xb1, 0xd1, 0x36, 0xc1, 0x67,
    0x52, 0x57, 0x46, 0x3d, 0x59, 0xf4, 0x87, 0xa4, 0x00, 0x00, 0x7e, 0x44, 0x00, 0x4e, 0x7b, 0xff,
    0x75, 0xf5, 0x06, 0x97, 0x10, 0xc3, 0x24, 0xbb, 0x00, 0x00, 0x7b, 0x7a, 0xe0, 0x60, 0x12, 0x0f,
    0xf7, 0x74, 0x1c, 0xe5, 0x39, 0x3d, 0x73, 0xc1, 0x00, 0x00, 0x7a, 0xb3, 0xff, 0x4e, 0x7b, 0xff,
];
pub const DSP_SAVELOAD_SIZE: usize = 3024;
pub const SPC_SAVELOAD_SIZE: usize = 15;
pub const APU_SAVELOAD_PREFIX_SIZE: usize = 65_576;

fn clip_i16(value: i32) -> i32 {
    value.clamp(-0x8000, 0x7fff)
}

fn clip_i16_cast(value: i32) -> i32 {
    value as i16 as i32
}

fn clip_15(value: i32) -> i16 {
    (((value & 0x7fff) << 1) as i16 >> 1) as i16
}

/// Apply the SNES DSP's four-tap Gaussian interpolation to four consecutive
/// decoded BRR samples. `offset` is the eight-bit fractional sample position.
pub fn dsp_gaussian_interpolate(oldest: i16, older: i16, old: i16, new: i16, offset: u8) -> i16 {
    let offset = usize::from(offset);
    let mut out = (DSP_GAUSS_VALUES[0xff - offset] * i32::from(oldest)) >> 11;
    out += (DSP_GAUSS_VALUES[0x1ff - offset] * i32::from(older)) >> 11;
    out += (DSP_GAUSS_VALUES[0x100 + offset] * i32::from(old)) >> 11;
    out = clip_i16_cast(out);
    out += (DSP_GAUSS_VALUES[offset] * i32::from(new)) >> 11;
    (clip_i16(out) & !1) as i16
}

fn dsp_exp_decrease_gain(gain: u16) -> u16 {
    let step = (((gain as i32 - 1) >> 8) + 1) as i32;
    (gain as i32 - step) as u16
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct DspChannel {
    pub pitch: u16,
    pub pitch_counter: u16,
    pub pitch_modulation: bool,
    pub decode_buffer: [i16; 19],
    pub srcn: u8,
    pub decode_offset: u16,
    pub previous_flags: u8,
    pub old: i16,
    pub older: i16,
    pub use_noise: bool,
    pub adsr_rates: [u16; 4],
    pub rate_counter: u16,
    pub adsr_state: u8,
    pub sustain_level: u16,
    pub use_gain: bool,
    pub gain_mode: u8,
    pub direct_gain: bool,
    pub gain_value: u16,
    pub gain: u16,
    pub key_on: bool,
    pub key_off: bool,
    pub sample_out: i16,
    pub volume_l: i8,
    pub volume_r: i8,
    pub echo_enable: bool,
}

impl Default for DspChannel {
    fn default() -> Self {
        Self {
            pitch: 0,
            pitch_counter: 0,
            pitch_modulation: false,
            decode_buffer: [0; 19],
            srcn: 0,
            decode_offset: 0,
            previous_flags: 0,
            old: 0,
            older: 0,
            use_noise: false,
            adsr_rates: [0; 4],
            rate_counter: 0,
            adsr_state: 0,
            sustain_level: 0,
            use_gain: false,
            gain_mode: 0,
            direct_gain: false,
            gain_value: 0,
            gain: 0,
            key_on: false,
            key_off: false,
            sample_out: 0,
            volume_l: 0,
            volume_r: 0,
            echo_enable: false,
        }
    }
}

impl DspChannel {
    fn save_c_saveload(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), 86);
        put_u16(out, 0, self.pitch);
        put_u16(out, 2, self.pitch_counter);
        out[4] = self.pitch_modulation as u8;
        for (i, &value) in self.decode_buffer.iter().enumerate() {
            put_i16(out, 6 + i * 2, value);
        }
        out[44] = self.srcn;
        put_u16(out, 46, self.decode_offset);
        out[48] = self.previous_flags;
        put_i16(out, 50, self.old);
        put_i16(out, 52, self.older);
        out[54] = self.use_noise as u8;
        for (i, &value) in self.adsr_rates.iter().enumerate() {
            put_u16(out, 56 + i * 2, value);
        }
        put_u16(out, 64, self.rate_counter);
        out[66] = self.adsr_state;
        put_u16(out, 68, self.sustain_level);
        out[70] = self.use_gain as u8;
        out[71] = self.gain_mode;
        out[72] = self.direct_gain as u8;
        put_u16(out, 74, self.gain_value);
        put_u16(out, 76, self.gain);
        out[78] = self.key_on as u8;
        out[79] = self.key_off as u8;
        put_i16(out, 80, self.sample_out);
        out[82] = self.volume_l as u8;
        out[83] = self.volume_r as u8;
        out[84] = self.echo_enable as u8;
    }

    fn load_c_saveload(&mut self, data: &[u8]) {
        debug_assert_eq!(data.len(), 86);
        self.pitch = get_u16(data, 0);
        self.pitch_counter = get_u16(data, 2);
        self.pitch_modulation = data[4] != 0;
        for (i, value) in self.decode_buffer.iter_mut().enumerate() {
            *value = get_i16(data, 6 + i * 2);
        }
        self.srcn = data[44];
        self.decode_offset = get_u16(data, 46);
        self.previous_flags = data[48];
        self.old = get_i16(data, 50);
        self.older = get_i16(data, 52);
        self.use_noise = data[54] != 0;
        for (i, value) in self.adsr_rates.iter_mut().enumerate() {
            *value = get_u16(data, 56 + i * 2);
        }
        self.rate_counter = get_u16(data, 64);
        self.adsr_state = data[66];
        self.sustain_level = get_u16(data, 68);
        self.use_gain = data[70] != 0;
        self.gain_mode = data[71];
        self.direct_gain = data[72] != 0;
        self.gain_value = get_u16(data, 74);
        self.gain = get_u16(data, 76);
        self.key_on = data[78] != 0;
        self.key_off = data[79] != 0;
        self.sample_out = get_i16(data, 80);
        self.volume_l = data[82] as i8;
        self.volume_r = data[83] as i8;
        self.echo_enable = data[84] != 0;
    }
}

fn put_u16(out: &mut [u8], offset: usize, value: u16) {
    out[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn put_u32(out: &mut [u8], offset: usize, value: u32) {
    out[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn put_i16(out: &mut [u8], offset: usize, value: i16) {
    out[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn get_u16(data: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([data[offset], data[offset + 1]])
}

fn get_u32(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}

fn get_i16(data: &[u8], offset: usize) -> i16 {
    i16::from_le_bytes([data[offset], data[offset + 1]])
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DspState {
    pub ram: Vec<u8>,
    pub channel: [DspChannel; 8],
    pub dir_page: u16,
    pub even_cycle: bool,
    pub mute: bool,
    pub reset_flag: bool,
    pub master_volume_l: i8,
    pub master_volume_r: i8,
    pub noise_sample: i16,
    pub noise_rate: u16,
    pub noise_counter: u16,
    pub echo_writes: bool,
    pub echo_volume_l: i8,
    pub echo_volume_r: i8,
    pub feedback_volume: i8,
    pub echo_buffer_adr: u16,
    pub echo_delay: u16,
    pub echo_remain: u16,
    pub echo_buffer_index: u16,
    pub fir_buffer_index: u8,
    pub fir_values: [i8; 8],
    pub fir_buffer_l: [i16; 8],
    pub fir_buffer_r: [i16; 8],
    pub sample_buffer: Vec<i16>,
    pub sample_offset: u16,
    #[serde(skip, default)]
    key_on_pipeline: [u8; 8],
    #[serde(skip, default)]
    pub debug_voice_samples: [Vec<i16>; 8],
}

impl Default for DspState {
    fn default() -> Self {
        let mut dsp = Self {
            ram: vec![0; 0x80],
            channel: [DspChannel::default(); 8],
            dir_page: 0,
            even_cycle: false,
            mute: true,
            reset_flag: true,
            master_volume_l: 0,
            master_volume_r: 0,
            noise_sample: -0x4000,
            noise_rate: 0,
            noise_counter: 0,
            echo_writes: false,
            echo_volume_l: 0,
            echo_volume_r: 0,
            feedback_volume: 0,
            echo_buffer_adr: 0,
            echo_delay: 1,
            echo_remain: 1,
            echo_buffer_index: 0,
            fir_buffer_index: 0,
            fir_values: [0; 8],
            fir_buffer_l: [0; 8],
            fir_buffer_r: [0; 8],
            sample_buffer: vec![0; DSP_SAMPLE_BUFFER_LEN],
            sample_offset: 0,
            key_on_pipeline: [0; 8],
            debug_voice_samples: std::array::from_fn(|_| Vec::new()),
        };
        dsp.reset();
        dsp
    }
}

impl DspState {
    pub fn reset(&mut self) {
        self.ram.copy_from_slice(&DSP_POWER_ON_REGISTERS);
        self.channel = [DspChannel::default(); 8];
        self.dir_page = 0;
        self.even_cycle = false;
        self.mute = true;
        self.reset_flag = true;
        self.master_volume_l = 0;
        self.master_volume_r = 0;
        self.noise_sample = -0x4000;
        self.noise_rate = 0;
        self.noise_counter = 0;
        self.echo_writes = false;
        self.echo_volume_l = 0;
        self.echo_volume_r = 0;
        self.feedback_volume = 0;
        self.echo_buffer_adr = 0;
        self.echo_delay = 1;
        self.echo_remain = 1;
        self.echo_buffer_index = 0;
        self.fir_buffer_index = 0;
        self.fir_values = [0; 8];
        self.fir_buffer_l = [0; 8];
        self.fir_buffer_r = [0; 8];
        self.sample_buffer.fill(0);
        self.sample_offset = 0;
        self.key_on_pipeline = [0; 8];
    }

    pub fn save_c_saveload(&self) -> Vec<u8> {
        let mut out = vec![0; DSP_SAVELOAD_SIZE];
        out[0..0x80].copy_from_slice(&self.ram[..0x80]);
        for (i, channel) in self.channel.iter().enumerate() {
            channel.save_c_saveload(&mut out[128 + i * 86..128 + (i + 1) * 86]);
        }
        put_u16(&mut out, 816, self.dir_page);
        out[818] = self.even_cycle as u8;
        out[819] = self.mute as u8;
        out[820] = self.reset_flag as u8;
        out[821] = self.master_volume_l as u8;
        out[822] = self.master_volume_r as u8;
        put_i16(&mut out, 824, self.noise_sample);
        put_u16(&mut out, 826, self.noise_rate);
        put_u16(&mut out, 828, self.noise_counter);
        out[830] = self.echo_writes as u8;
        out[831] = self.echo_volume_l as u8;
        out[832] = self.echo_volume_r as u8;
        out[833] = self.feedback_volume as u8;
        put_u16(&mut out, 834, self.echo_buffer_adr);
        put_u16(&mut out, 836, self.echo_delay);
        put_u16(&mut out, 838, self.echo_remain);
        put_u16(&mut out, 840, self.echo_buffer_index);
        out[842] = self.fir_buffer_index;
        for (i, &value) in self.fir_values.iter().enumerate() {
            out[843 + i] = value as u8;
        }
        for (i, &value) in self.fir_buffer_l.iter().enumerate() {
            put_i16(&mut out, 852 + i * 2, value);
        }
        for (i, &value) in self.fir_buffer_r.iter().enumerate() {
            put_i16(&mut out, 868 + i * 2, value);
        }
        for (i, &value) in self
            .sample_buffer
            .iter()
            .take(DSP_SAMPLE_BUFFER_LEN)
            .enumerate()
        {
            put_i16(&mut out, 884 + i * 2, value);
        }
        put_u16(&mut out, 3020, self.sample_offset);
        out
    }

    pub fn load_c_saveload(&mut self, data: &[u8]) -> Result<(), String> {
        if data.len() != DSP_SAVELOAD_SIZE {
            return Err(format!(
                "invalid DSP saveload block: expected {DSP_SAVELOAD_SIZE}, got {}",
                data.len()
            ));
        }
        self.ram[..0x80].copy_from_slice(&data[0..0x80]);
        for i in 0..8 {
            self.channel[i].load_c_saveload(&data[128 + i * 86..128 + (i + 1) * 86]);
        }
        self.dir_page = get_u16(data, 816);
        self.even_cycle = data[818] != 0;
        self.mute = data[819] != 0;
        self.reset_flag = data[820] != 0;
        self.master_volume_l = data[821] as i8;
        self.master_volume_r = data[822] as i8;
        self.noise_sample = get_i16(data, 824);
        self.noise_rate = get_u16(data, 826);
        self.noise_counter = get_u16(data, 828);
        self.echo_writes = data[830] != 0;
        self.echo_volume_l = data[831] as i8;
        self.echo_volume_r = data[832] as i8;
        self.feedback_volume = data[833] as i8;
        self.echo_buffer_adr = get_u16(data, 834);
        self.echo_delay = get_u16(data, 836);
        self.echo_remain = get_u16(data, 838);
        self.echo_buffer_index = get_u16(data, 840);
        self.fir_buffer_index = data[842];
        for (i, value) in self.fir_values.iter_mut().enumerate() {
            *value = data[843 + i] as i8;
        }
        for (i, value) in self.fir_buffer_l.iter_mut().enumerate() {
            *value = get_i16(data, 852 + i * 2);
        }
        for (i, value) in self.fir_buffer_r.iter_mut().enumerate() {
            *value = get_i16(data, 868 + i * 2);
        }
        if self.sample_buffer.len() != DSP_SAMPLE_BUFFER_LEN {
            self.sample_buffer.resize(DSP_SAMPLE_BUFFER_LEN, 0);
        }
        for (i, value) in self
            .sample_buffer
            .iter_mut()
            .enumerate()
            .take(DSP_SAMPLE_BUFFER_LEN)
        {
            *value = get_i16(data, 884 + i * 2);
        }
        self.sample_offset = get_u16(data, 3020);
        Ok(())
    }

    pub fn read(&self, adr: u8) -> u8 {
        self.ram[adr as usize]
    }

    pub fn write(&mut self, adr: u8, val: u8, _apu_ram: &[u8]) {
        let ch = (adr >> 4) as usize;
        match adr {
            0x00 | 0x10 | 0x20 | 0x30 | 0x40 | 0x50 | 0x60 | 0x70 => {
                self.channel[ch].volume_l = val as i8;
            }
            0x01 | 0x11 | 0x21 | 0x31 | 0x41 | 0x51 | 0x61 | 0x71 => {
                self.channel[ch].volume_r = val as i8;
            }
            0x02 | 0x12 | 0x22 | 0x32 | 0x42 | 0x52 | 0x62 | 0x72 => {
                self.channel[ch].pitch = (self.channel[ch].pitch & 0x3f00) | val as u16;
            }
            0x03 | 0x13 | 0x23 | 0x33 | 0x43 | 0x53 | 0x63 | 0x73 => {
                self.channel[ch].pitch =
                    ((self.channel[ch].pitch & 0x00ff) | ((val as u16) << 8)) & 0x3fff;
            }
            0x04 | 0x14 | 0x24 | 0x34 | 0x44 | 0x54 | 0x64 | 0x74 => {
                self.channel[ch].srcn = val;
            }
            0x05 | 0x15 | 0x25 | 0x35 | 0x45 | 0x55 | 0x65 | 0x75 => {
                self.channel[ch].adsr_rates[0] = DSP_RATE_VALUES[((val & 0x0f) * 2 + 1) as usize];
                self.channel[ch].adsr_rates[1] =
                    DSP_RATE_VALUES[(((val & 0x70) >> 4) * 2 + 16) as usize];
                self.channel[ch].use_gain = val & 0x80 == 0;
            }
            0x06 | 0x16 | 0x26 | 0x36 | 0x46 | 0x56 | 0x66 | 0x76 => {
                self.channel[ch].adsr_rates[2] = DSP_RATE_VALUES[(val & 0x1f) as usize];
                self.channel[ch].sustain_level = ((((val & 0xe0) >> 5) as u16) + 1) * 0x100;
            }
            0x07 | 0x17 | 0x27 | 0x37 | 0x47 | 0x57 | 0x67 | 0x77 => {
                self.channel[ch].direct_gain = val & 0x80 == 0;
                if val & 0x80 != 0 {
                    self.channel[ch].gain_mode = (val & 0x60) >> 5;
                    self.channel[ch].adsr_rates[3] = DSP_RATE_VALUES[(val & 0x1f) as usize];
                } else {
                    self.channel[ch].gain_value = ((val & 0x7f) as u16) * 16;
                }
            }
            DSP_MVOLL => self.master_volume_l = val as i8,
            DSP_MVOLR => self.master_volume_r = val as i8,
            DSP_EVOLL => self.echo_volume_l = val as i8,
            DSP_EVOLR => self.echo_volume_r = val as i8,
            DSP_KON => {
                for i in 0..8 {
                    self.channel[i].key_on = val & (1 << i) != 0;
                    if self.channel[i].key_on {
                        // The S-DSP polls KON every other sample. The poll below
                        // starts the internal five-sample voice-start pipeline.
                    }
                }
            }
            DSP_KOF => {
                for i in 0..8 {
                    self.channel[i].key_off = val & (1 << i) != 0;
                    if self.channel[i].key_off {
                        self.channel[i].adsr_state = 4;
                    }
                }
            }
            DSP_FLG => {
                self.reset_flag = val & 0x80 != 0;
                self.mute = val & 0x40 != 0;
                self.echo_writes = val & 0x20 == 0;
                self.noise_rate = DSP_RATE_VALUES[(val & 0x1f) as usize];
            }
            DSP_ENDX => {
                self.ram[DSP_ENDX as usize] = 0;
                return;
            }
            DSP_EFB => self.feedback_volume = val as i8,
            DSP_PMON => {
                for i in 0..8 {
                    self.channel[i].pitch_modulation = val & (1 << i) != 0;
                }
            }
            DSP_NON => {
                for i in 0..8 {
                    self.channel[i].use_noise = val & (1 << i) != 0;
                }
            }
            DSP_EON => {
                for i in 0..8 {
                    self.channel[i].echo_enable = val & (1 << i) != 0;
                }
            }
            DSP_DIR => self.dir_page = (val as u16) << 8,
            DSP_ESA => self.echo_buffer_adr = (val as u16) << 8,
            DSP_EDL => {
                self.echo_delay = ((val & 0x0f) as u16) * 512;
                if self.echo_delay == 0 {
                    self.echo_delay = 1;
                }
            }
            DSP_FIR0 | 0x1f | 0x2f | 0x3f | 0x4f | 0x5f | 0x6f | 0x7f => {
                self.fir_values[ch] = val as i8;
            }
            _ => {}
        }
        self.ram[adr as usize] = val;
    }

    pub fn cycle(&mut self, apu_ram: &mut [u8]) {
        if self.sample_offset == 0 {
            for samples in &mut self.debug_voice_samples {
                samples.clear();
            }
        }
        let mut total_l = 0;
        let mut total_r = 0;
        for i in 0..8 {
            self.advance_key_on_pipeline(apu_ram, i);
            self.cycle_channel(apu_ram, i);
            self.debug_voice_samples[i].push(self.channel[i].sample_out);
            total_l += (self.channel[i].sample_out as i32 * self.channel[i].volume_l as i32) >> 6;
            total_r += (self.channel[i].sample_out as i32 * self.channel[i].volume_r as i32) >> 6;
            total_l = clip_i16(total_l);
            total_r = clip_i16(total_r);
        }
        total_l = (total_l * self.master_volume_l as i32) >> 7;
        total_r = (total_r * self.master_volume_r as i32) >> 7;
        total_l = clip_i16(total_l);
        total_r = clip_i16(total_r);
        self.handle_echo(apu_ram, &mut total_l, &mut total_r);
        if self.mute {
            total_l = 0;
            total_r = 0;
        }
        self.handle_noise();
        if (self.sample_offset as usize) < 534 {
            let offset = self.sample_offset as usize * 2;
            self.sample_buffer[offset] = total_l as i16;
            self.sample_buffer[offset + 1] = total_r as i16;
            self.sample_offset += 1;
        }
        if self.even_cycle {
            for ch in 0..8 {
                if self.channel[ch].key_on {
                    self.channel[ch].key_on = false;
                    self.key_on_pipeline[ch] = 5;
                }
            }
        }
        self.even_cycle = !self.even_cycle;
    }

    fn advance_key_on_pipeline(&mut self, apu_ram: &[u8], ch: usize) {
        let delay = &mut self.key_on_pipeline[ch];
        if *delay == 0 {
            return;
        }
        *delay -= 1;
        if *delay != 0 {
            return;
        }

        self.channel[ch].previous_flags = 0;
        let sample_pointer = self.dir_page.wrapping_add(4 * self.channel[ch].srcn as u16);
        self.channel[ch].decode_offset = apu_ram[sample_pointer as usize] as u16
            | ((apu_ram[sample_pointer.wrapping_add(1) as usize] as u16) << 8);
        self.channel[ch].decode_buffer = [0; 19];
        self.channel[ch].gain = 0;
        self.channel[ch].adsr_state = if self.channel[ch].use_gain { 3 } else { 0 };
    }

    fn handle_echo(&mut self, apu_ram: &mut [u8], output_l: &mut i32, output_r: &mut i32) {
        let adr = self
            .echo_buffer_adr
            .wrapping_add(self.echo_buffer_index.wrapping_mul(4));
        let adr_usize = adr as usize;
        let fir_index = self.fir_buffer_index as usize;
        self.fir_buffer_l[fir_index] = ((apu_ram[adr_usize] as u16
            | ((apu_ram[(adr.wrapping_add(1)) as usize] as u16) << 8))
            as i16)
            >> 1;
        self.fir_buffer_r[fir_index] = ((apu_ram[(adr.wrapping_add(2)) as usize] as u16
            | ((apu_ram[(adr.wrapping_add(3)) as usize] as u16) << 8))
            as i16)
            >> 1;

        let mut sum_l = 0;
        let mut sum_r = 0;
        for i in 0..8 {
            let idx = (fir_index + i + 1) & 7;
            sum_l += (self.fir_buffer_l[idx] as i32 * self.fir_values[i] as i32) >> 6;
            sum_r += (self.fir_buffer_r[idx] as i32 * self.fir_values[i] as i32) >> 6;
            if i == 6 {
                sum_l = clip_i16_cast(sum_l);
                sum_r = clip_i16_cast(sum_r);
            }
        }
        sum_l = clip_i16(sum_l);
        sum_r = clip_i16(sum_r);

        *output_l = clip_i16(*output_l + ((sum_l * self.echo_volume_l as i32) >> 7));
        *output_r = clip_i16(*output_r + ((sum_r * self.echo_volume_r as i32) >> 7));

        let mut in_l = 0;
        let mut in_r = 0;
        for i in 0..8 {
            if self.channel[i].echo_enable {
                in_l += (self.channel[i].sample_out as i32 * self.channel[i].volume_l as i32) >> 6;
                in_r += (self.channel[i].sample_out as i32 * self.channel[i].volume_r as i32) >> 6;
                in_l = clip_i16(in_l);
                in_r = clip_i16(in_r);
            }
        }
        in_l = clip_i16(in_l + ((sum_l * self.feedback_volume as i32) >> 7)) & 0xfffe;
        in_r = clip_i16(in_r + ((sum_r * self.feedback_volume as i32) >> 7)) & 0xfffe;
        if self.echo_writes {
            apu_ram[adr_usize] = in_l as u8;
            apu_ram[(adr.wrapping_add(1)) as usize] = (in_l >> 8) as u8;
            apu_ram[(adr.wrapping_add(2)) as usize] = in_r as u8;
            apu_ram[(adr.wrapping_add(3)) as usize] = (in_r >> 8) as u8;
        }

        self.fir_buffer_index = (self.fir_buffer_index + 1) & 7;
        self.echo_buffer_index = self.echo_buffer_index.wrapping_add(1);
        self.echo_remain = self.echo_remain.wrapping_sub(1);
        if self.echo_remain == 0 {
            self.echo_remain = self.echo_delay;
            self.echo_buffer_index = 0;
        }
    }

    fn cycle_channel(&mut self, apu_ram: &mut [u8], ch: usize) {
        let mut pitch = self.channel[ch].pitch;
        if ch > 0 && self.channel[ch].pitch_modulation {
            let factor = (self.channel[ch - 1].sample_out as i32 >> 4) + 0x400;
            let modulated = ((pitch as i32 * factor) >> 10) as u16;
            pitch = if modulated > 0x3fff {
                0x3fff
            } else {
                modulated
            };
        }
        let new_counter = self.channel[ch].pitch_counter as u32 + pitch as u32;
        if new_counter > 0xffff {
            self.decode_brr(apu_ram, ch);
        }
        self.channel[ch].pitch_counter = new_counter as u16;

        let mut sample = if self.channel[ch].use_noise {
            self.noise_sample
        } else {
            self.get_sample(
                ch,
                (self.channel[ch].pitch_counter >> 12) as usize,
                ((self.channel[ch].pitch_counter >> 4) & 0xff) as usize,
            )
        };

        if self.reset_flag {
            self.channel[ch].adsr_state = 4;
            self.channel[ch].gain = 0;
        }

        let doing_direct_gain = self.channel[ch].adsr_state != 4
            && self.channel[ch].use_gain
            && self.channel[ch].direct_gain;
        let rate = if self.channel[ch].adsr_state == 4 {
            0
        } else {
            self.channel[ch].adsr_rates[self.channel[ch].adsr_state as usize]
        };
        if self.channel[ch].adsr_state != 4 && !doing_direct_gain && rate != 0 {
            self.channel[ch].rate_counter = self.channel[ch].rate_counter.wrapping_add(1);
        }
        if self.channel[ch].adsr_state == 4
            || (!doing_direct_gain && self.channel[ch].rate_counter >= rate && rate != 0)
        {
            if self.channel[ch].adsr_state != 4 {
                self.channel[ch].rate_counter = 0;
            }
            self.handle_gain(ch);
        }
        if doing_direct_gain {
            self.channel[ch].gain = self.channel[ch].gain_value;
        }
        self.ram[(ch << 4) | 8] = (self.channel[ch].gain >> 4) as u8;
        sample = ((sample as i32 * self.channel[ch].gain as i32) >> 11) as i16;
        self.ram[(ch << 4) | 9] = (sample >> 7) as u8;
        self.channel[ch].sample_out = sample;
    }

    fn handle_gain(&mut self, ch: usize) {
        match self.channel[ch].adsr_state {
            0 => {
                let rate = self.channel[ch].adsr_rates[0];
                self.channel[ch].gain =
                    self.channel[ch]
                        .gain
                        .wrapping_add(if rate == 1 { 1024 } else { 32 });
                if self.channel[ch].gain >= 0x7e0 {
                    self.channel[ch].adsr_state = 1;
                }
                if self.channel[ch].gain > 0x7ff {
                    self.channel[ch].gain = 0x7ff;
                }
            }
            1 => {
                self.channel[ch].gain = dsp_exp_decrease_gain(self.channel[ch].gain);
                if self.channel[ch].gain < self.channel[ch].sustain_level {
                    self.channel[ch].adsr_state = 2;
                }
            }
            2 => {
                self.channel[ch].gain = dsp_exp_decrease_gain(self.channel[ch].gain);
            }
            3 => match self.channel[ch].gain_mode {
                0 => {
                    self.channel[ch].gain = self.channel[ch].gain.wrapping_sub(32);
                    if self.channel[ch].gain > 0x7ff {
                        self.channel[ch].gain = 0;
                    }
                }
                1 => {
                    self.channel[ch].gain = dsp_exp_decrease_gain(self.channel[ch].gain);
                }
                2 => {
                    self.channel[ch].gain = self.channel[ch].gain.wrapping_add(32);
                    if self.channel[ch].gain > 0x7ff {
                        self.channel[ch].gain = 0;
                    }
                }
                3 => {
                    self.channel[ch].gain = self.channel[ch]
                        .gain
                        .wrapping_add(if self.channel[ch].gain < 0x600 { 32 } else { 8 });
                    if self.channel[ch].gain > 0x7ff {
                        self.channel[ch].gain = 0;
                    }
                }
                _ => {}
            },
            4 => {
                self.channel[ch].gain = self.channel[ch].gain.wrapping_sub(8);
                if self.channel[ch].gain > 0x7ff {
                    self.channel[ch].gain = 0;
                }
            }
            _ => {}
        }
    }

    fn get_sample(&self, ch: usize, sample_num: usize, offset: usize) -> i16 {
        let news = self.channel[ch].decode_buffer[sample_num + 3] as i32;
        let olds = self.channel[ch].decode_buffer[sample_num + 2] as i32;
        let olders = self.channel[ch].decode_buffer[sample_num + 1] as i32;
        let oldests = self.channel[ch].decode_buffer[sample_num] as i32;
        dsp_gaussian_interpolate(
            oldests as i16,
            olders as i16,
            olds as i16,
            news as i16,
            offset as u8,
        )
    }

    fn decode_brr(&mut self, apu_ram: &[u8], ch: usize) {
        self.channel[ch].decode_buffer[0] = self.channel[ch].decode_buffer[16];
        self.channel[ch].decode_buffer[1] = self.channel[ch].decode_buffer[17];
        self.channel[ch].decode_buffer[2] = self.channel[ch].decode_buffer[18];
        if self.channel[ch].previous_flags == 1 || self.channel[ch].previous_flags == 3 {
            let sample_pointer = self.dir_page.wrapping_add(4 * self.channel[ch].srcn as u16);
            self.channel[ch].decode_offset = apu_ram[(sample_pointer.wrapping_add(2)) as usize]
                as u16
                | ((apu_ram[(sample_pointer.wrapping_add(3)) as usize] as u16) << 8);
            if self.channel[ch].previous_flags == 1 {
                self.channel[ch].adsr_state = 4;
                self.channel[ch].gain = 0;
            }
            self.ram[DSP_ENDX as usize] |= 1 << ch;
        }
        let header = apu_ram[self.channel[ch].decode_offset as usize];
        self.channel[ch].decode_offset = self.channel[ch].decode_offset.wrapping_add(1);
        let shift = header >> 4;
        let filter = (header & 0x0c) >> 2;
        self.channel[ch].previous_flags = header & 3;
        let mut cur_byte = 0;
        let mut old = self.channel[ch].old as i32;
        let mut older = self.channel[ch].older as i32;
        for i in 0..16 {
            let mut s;
            if i & 1 != 0 {
                s = (cur_byte & 0x0f) as i32;
            } else {
                cur_byte = apu_ram[self.channel[ch].decode_offset as usize];
                self.channel[ch].decode_offset = self.channel[ch].decode_offset.wrapping_add(1);
                s = (cur_byte >> 4) as i32;
            }
            if s > 7 {
                s -= 16;
            }
            if shift <= 0x0c {
                s = (s << shift) >> 1;
            } else {
                s = (s >> 3) << 12;
            }
            match filter {
                1 => s += old + (-old >> 4),
                2 => s += 2 * old + ((3 * -old) >> 5) - older + (older >> 4),
                3 => s += 2 * old + ((13 * -old) >> 6) - older + ((3 * older) >> 4),
                _ => {}
            }
            s = clip_i16(s);
            let clipped = clip_15(s);
            older = old;
            old = clipped as i32;
            self.channel[ch].decode_buffer[i + 3] = clipped;
        }
        self.channel[ch].older = older as i16;
        self.channel[ch].old = old as i16;
    }

    fn handle_noise(&mut self) {
        if self.noise_rate != 0 {
            self.noise_counter = self.noise_counter.wrapping_add(1);
        }
        if self.noise_counter >= self.noise_rate && self.noise_rate != 0 {
            let sample = self.noise_sample as i32;
            let bit = (sample & 1) ^ ((sample >> 1) & 1);
            self.noise_sample = clip_15(((sample >> 1) & 0x3fff) | (bit << 14));
            self.noise_counter = 0;
        }
    }

    pub fn get_samples(
        &mut self,
        sample_data: &mut [i16],
        samples_per_frame: usize,
        num_channels: usize,
    ) {
        if samples_per_frame == 0 {
            self.sample_offset = 0;
            return;
        }

        if samples_per_frame == 534 {
            if num_channels == 1 {
                for (sample, src) in sample_data
                    .iter_mut()
                    .take(samples_per_frame)
                    .zip(self.sample_buffer.chunks_exact(2))
                {
                    *sample = ((src[0] as i32 + src[1] as i32) >> 1) as i16;
                }
            } else {
                let count = samples_per_frame.saturating_mul(2).min(sample_data.len());
                sample_data[..count].copy_from_slice(&self.sample_buffer[..count]);
            }
            self.sample_offset = 0;
            return;
        }

        let adder = 534.0f32 / samples_per_frame as f32;
        let mut location = 0.0f32;
        if num_channels == 1 {
            for sample in sample_data.iter_mut().take(samples_per_frame) {
                let src = ((location as usize).min(533)) * 2;
                let left = self.sample_buffer[src];
                let right = self.sample_buffer[src + 1];
                *sample = ((left as i32 + right as i32) >> 1) as i16;
                location += adder;
            }
        } else {
            for i in 0..samples_per_frame {
                let dst = i * 2;
                if dst + 1 < sample_data.len() {
                    let src = ((location as usize).min(533)) * 2;
                    sample_data[dst] = self.sample_buffer[src];
                    sample_data[dst + 1] = self.sample_buffer[src + 1];
                }
                location += adder;
            }
        }
        self.sample_offset = 0;
    }

    /// Drain already-generated 32 kHz DSP samples without resampling them.
    ///
    /// Libretro frame callbacks alternate between 533 and 534 samples.  The
    /// older `get_samples` API always treats the internal buffer as a complete
    /// 534-sample block, so asking it for 533 samples drops one sample and
    /// breaks continuous waveform alignment.  This path preserves the DSP
    /// stream across arbitrary callback boundaries.
    pub fn drain_samples_exact(
        &mut self,
        sample_data: &mut [i16],
        samples: usize,
        num_channels: usize,
    ) -> Result<(), String> {
        let available = self.sample_offset as usize;
        if samples > available {
            return Err(format!(
                "requested {samples} exact DSP samples with only {available} available"
            ));
        }
        if num_channels != 1 && num_channels != 2 {
            return Err(format!(
                "exact DSP drain supports mono or stereo, got {num_channels} channels"
            ));
        }
        let required = samples.saturating_mul(num_channels);
        if sample_data.len() < required {
            return Err(format!(
                "exact DSP drain needs {required} output values, got {}",
                sample_data.len()
            ));
        }

        if num_channels == 1 {
            for (sample, src) in sample_data
                .iter_mut()
                .take(samples)
                .zip(self.sample_buffer.chunks_exact(2))
            {
                *sample = ((i32::from(src[0]) + i32::from(src[1])) >> 1) as i16;
            }
        } else {
            sample_data[..samples * 2].copy_from_slice(&self.sample_buffer[..samples * 2]);
        }

        let remaining = available - samples;
        self.sample_buffer
            .copy_within(samples * 2..available * 2, 0);
        self.sample_buffer[remaining * 2..available * 2].fill(0);
        self.sample_offset = remaining as u16;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub struct ApuTimer {
    pub cycles: u8,
    pub divider: u8,
    pub target: u8,
    pub counter: u8,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub struct SpcState {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub pc: u16,
    pub c: bool,
    pub z: bool,
    pub v: bool,
    pub n: bool,
    pub i: bool,
    pub h: bool,
    pub p: bool,
    pub b: bool,
    pub stopped: bool,
    pub cycles_used: u8,
}

impl SpcState {
    /// Byte layout used by C `spc_saveload`.
    pub fn save_c_saveload(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(SPC_SAVELOAD_SIZE);
        out.push(self.a);
        out.push(self.x);
        out.push(self.y);
        out.push(self.sp);
        out.extend_from_slice(&self.pc.to_le_bytes());
        out.push(self.c as u8);
        out.push(self.z as u8);
        out.push(self.v as u8);
        out.push(self.n as u8);
        out.push(self.i as u8);
        out.push(self.h as u8);
        out.push(self.p as u8);
        out.push(self.b as u8);
        out.push(self.stopped as u8);
        debug_assert_eq!(out.len(), SPC_SAVELOAD_SIZE);
        out
    }

    pub fn load_c_saveload(&mut self, data: &[u8]) -> Result<(), String> {
        if data.len() != SPC_SAVELOAD_SIZE {
            return Err(format!(
                "invalid SPC saveload size {}, expected {}",
                data.len(),
                SPC_SAVELOAD_SIZE
            ));
        }
        self.a = data[0];
        self.x = data[1];
        self.y = data[2];
        self.sp = data[3];
        self.pc = u16::from_le_bytes([data[4], data[5]]);
        self.c = data[6] != 0;
        self.z = data[7] != 0;
        self.v = data[8] != 0;
        self.n = data[9] != 0;
        self.i = data[10] != 0;
        self.h = data[11] != 0;
        self.p = data[12] != 0;
        self.b = data[13] != 0;
        self.stopped = data[14] != 0;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, serde::Serialize)]
pub struct SpcInstructionTrace {
    pub cycle: u32,
    pub pc: u16,
    pub opcode: u8,
    pub operands: [u8; 2],
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub p: bool,
    pub direct_page_0_3: [u8; 4],
    pub direct_page_8_11: [u8; 4],
    pub input_ports: [u8; 4],
    pub timer0_cycles: u8,
    pub timer0_divider: u8,
    pub timer0_counter: u8,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApuState {
    pub ram: Vec<u8>,
    pub dsp_regs: Vec<u8>,
    pub dsp: DspState,
    pub out_ports: [u8; 4],
    pub in_ports: [u8; 6],
    pub rom_readable: bool,
    pub dsp_adr: u8,
    pub cycles: u32,
    pub timer: [ApuTimer; 3],
    pub spc: SpcState,
    #[serde(skip)]
    smp_coroutine: SmpCoroutineState,
    pub cpu_cycles_left: u8,
    pub dsp_write_history: Vec<(u8, u8)>,
    #[serde(skip)]
    pub debug_dsp_write_trace: Option<Vec<(u32, u8, u8)>>,
    #[serde(skip)]
    pub debug_spc_instruction_trace: Option<Vec<SpcInstructionTrace>>,
    #[serde(skip)]
    scheduled_input_port_writes: Vec<(u32, u8, u8)>,
}

impl Default for ApuState {
    fn default() -> Self {
        Self {
            ram: vec![0; 0x10000],
            dsp_regs: vec![0; 0x80],
            dsp: DspState::default(),
            out_ports: [0; 4],
            in_ports: [0; 6],
            rom_readable: true,
            dsp_adr: 0,
            cycles: 0,
            timer: [ApuTimer::default(); 3],
            spc: SpcState::default(),
            smp_coroutine: SmpCoroutineState::default(),
            cpu_cycles_left: 7,
            dsp_write_history: Vec::new(),
            debug_dsp_write_trace: None,
            debug_spc_instruction_trace: None,
            scheduled_input_port_writes: Vec::new(),
        }
    }
}

impl ApuState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.rom_readable = true;
        self.spc_reset();
        self.dsp.reset();
        self.ram.fill(0);
        self.dsp_regs.clone_from(&self.dsp.ram);
        self.dsp_adr = 0;
        self.cycles = 0;
        self.in_ports = [0; 6];
        self.out_ports = [0; 4];
        self.timer = [ApuTimer::default(); 3];
        self.smp_coroutine = SmpCoroutineState::default();
        self.cpu_cycles_left = 7;
        self.dsp_write_history.clear();
        if let Some(trace) = self.debug_dsp_write_trace.as_mut() {
            trace.clear();
        }
        if let Some(trace) = self.debug_spc_instruction_trace.as_mut() {
            trace.clear();
        }
        self.scheduled_input_port_writes.clear();
    }

    /// Reset into pinned Snes9x's SMP coroutine state for an isolated timing
    /// shadow. The normal C-port `reset` remains unchanged.
    pub fn reset_snes9x_coroutine(&mut self) {
        self.reset();
        self.spc.sp = 0xef;
        self.spc.z = true;
        self.cpu_cycles_left = 0;
        self.smp_coroutine = SmpCoroutineState::enabled();
    }

    /// Capture the opt-in SMP continuation as a timing sidecar, separate from
    /// the stable `ApuState`/`Snes` machine serialization layout.
    pub fn capture_snes9x_coroutine_checkpoint(&self) -> Option<Snes9xSmpCoroutineCheckpoint> {
        self.smp_coroutine.checkpoint()
    }

    /// Restore a timing-sidecar continuation onto its matching APU machine
    /// state after cloning or deserializing that machine state.
    pub fn restore_snes9x_coroutine_checkpoint(
        &mut self,
        checkpoint: Snes9xSmpCoroutineCheckpoint,
    ) {
        self.smp_coroutine = SmpCoroutineState::restore(checkpoint);
    }

    /// Import the instruction-boundary SMP portion of a Snes9x 1.63 `SND`
    /// snapshot block. This is an offline parity diagnostic: it lets the Rust
    /// SPC700/timer core and Snes9x continue from identical state without
    /// importing Snes9x's DSP renderer.
    pub fn load_snes9x_1_63_smp_state(&mut self, snd: &[u8]) -> Result<(), String> {
        const RAM_BYTES: usize = 0x10000;
        const SMP_INT_COUNT: usize = 41;
        const SMP_BYTES: usize = RAM_BYTES + SMP_INT_COUNT * 4;
        if snd.len() < SMP_BYTES {
            return Err(format!(
                "Snes9x SND block has {} bytes, expected at least {SMP_BYTES}",
                snd.len()
            ));
        }

        self.ram[..RAM_BYTES].copy_from_slice(&snd[..RAM_BYTES]);
        let mut cursor = RAM_BYTES;
        let next = |cursor: &mut usize| {
            let value = u32::from_le_bytes(snd[*cursor..*cursor + 4].try_into().unwrap());
            *cursor += 4;
            value
        };

        let _clock = next(&mut cursor);
        let _opcode_number = next(&mut cursor);
        let opcode_cycle = next(&mut cursor);
        if opcode_cycle != 0 {
            return Err(format!(
                "Snes9x SMP snapshot is mid-instruction at opcode cycle {opcode_cycle}"
            ));
        }
        self.spc.pc = next(&mut cursor) as u16;
        self.spc.sp = next(&mut cursor) as u8;
        self.spc.a = next(&mut cursor) as u8;
        self.spc.x = next(&mut cursor) as u8;
        self.spc.y = next(&mut cursor) as u8;
        self.spc.n = next(&mut cursor) != 0;
        self.spc.v = next(&mut cursor) != 0;
        self.spc.p = next(&mut cursor) != 0;
        self.spc.b = next(&mut cursor) != 0;
        self.spc.h = next(&mut cursor) != 0;
        self.spc.i = next(&mut cursor) != 0;
        self.spc.z = next(&mut cursor) != 0;
        self.spc.c = next(&mut cursor) != 0;
        self.spc.stopped = false;
        self.spc.cycles_used = 0;
        self.smp_coroutine = SmpCoroutineState::default();
        self.rom_readable = next(&mut cursor) != 0;
        self.dsp_adr = next(&mut cursor) as u8;
        self.in_ports[4] = next(&mut cursor) as u8;
        self.in_ports[5] = next(&mut cursor) as u8;

        for (timer_index, frequency) in [128u8, 128, 16].into_iter().enumerate() {
            let enabled = next(&mut cursor) != 0;
            let target = next(&mut cursor) as u8;
            let stage1_ticks = next(&mut cursor) as u8;
            let stage2_ticks = next(&mut cursor) as u8;
            let stage3_ticks = next(&mut cursor) as u8;
            self.timer[timer_index] = ApuTimer {
                cycles: frequency.wrapping_sub(stage1_ticks),
                divider: stage2_ticks,
                target,
                counter: stage3_ticks,
                enabled,
            };
        }

        // Skip Snes9x's six opcode scratch registers. At an instruction
        // boundary they do not feed the next opcode.
        for _ in 0..6 {
            let _ = next(&mut cursor);
        }
        debug_assert_eq!(cursor, SMP_BYTES);

        self.out_ports.copy_from_slice(&self.ram[0xf4..0xf8]);
        self.in_ports[..4].fill(0);
        self.cycles = 0;
        self.cpu_cycles_left = 0;
        self.dsp_write_history.clear();
        self.debug_dsp_write_trace = Some(Vec::new());
        self.scheduled_input_port_writes.clear();
        if snd.len() >= SMP_BYTES + 0x80 {
            self.dsp_regs
                .copy_from_slice(&snd[SMP_BYTES..SMP_BYTES + 0x80]);
        }
        Ok(())
    }

    /// Byte layout used by C `apu_saveload` before nested DSP/SPC blocks.
    ///
    /// C serializes from `Apu.ram` through `cpuCyclesLeft`, including native
    /// struct padding before `cycles` and before the trailing history union.
    pub fn save_c_saveload_prefix(&self) -> Vec<u8> {
        let mut out = vec![0; APU_SAVELOAD_PREFIX_SIZE];
        out[..0x10000].copy_from_slice(&self.ram[..0x10000]);
        out[0x10000] = self.rom_readable as u8;
        out[0x10001] = self.dsp_adr;
        put_u32(&mut out, 0x10004, self.cycles);
        out[0x10008..0x1000e].copy_from_slice(&self.in_ports);
        out[0x1000e..0x10012].copy_from_slice(&self.out_ports);
        let mut pos = 0x10012;
        for timer in &self.timer {
            out[pos] = timer.cycles;
            out[pos + 1] = timer.divider;
            out[pos + 2] = timer.target;
            out[pos + 3] = timer.counter;
            out[pos + 4] = timer.enabled as u8;
            pos += 5;
        }
        out[0x10021] = self.cpu_cycles_left;
        out
    }

    pub fn load_c_saveload_prefix(&mut self, data: &[u8]) -> Result<(), String> {
        if data.len() != APU_SAVELOAD_PREFIX_SIZE {
            return Err(format!(
                "invalid APU saveload prefix size {}, expected {}",
                data.len(),
                APU_SAVELOAD_PREFIX_SIZE
            ));
        }
        if self.ram.len() != 0x10000 {
            self.ram.resize(0x10000, 0);
        }
        self.ram[..0x10000].copy_from_slice(&data[..0x10000]);
        self.rom_readable = data[0x10000] != 0;
        self.dsp_adr = data[0x10001];
        self.cycles = get_u32(data, 0x10004);
        self.in_ports.copy_from_slice(&data[0x10008..0x1000e]);
        self.out_ports.copy_from_slice(&data[0x1000e..0x10012]);
        let mut pos = 0x10012;
        for timer in &mut self.timer {
            timer.cycles = data[pos];
            timer.divider = data[pos + 1];
            timer.target = data[pos + 2];
            timer.counter = data[pos + 3];
            timer.enabled = data[pos + 4] != 0;
            pos += 5;
        }
        self.cpu_cycles_left = data[0x10021];
        // The C save format is instruction-boundary state and has no retained
        // Snes9x `op_step` continuation.
        self.smp_coroutine = SmpCoroutineState::default();
        Ok(())
    }

    pub fn cycle(&mut self) {
        self.cycle_inner(true);
    }

    /// Advance the SPC700 CPU and hardware timers without synthesizing a DSP
    /// sample. This is used by translated runtimes that need the original
    /// driver's instruction/poll cadence while a modern renderer owns audio
    /// production.
    pub fn cycle_without_dsp(&mut self) {
        self.cycle_inner(false);
    }

    /// Run one complete SPC700 instruction while advancing timers at each
    /// individual bus or internal cycle. Unlike `cycle_without_dsp`, opcode
    /// effects occur at their hardware-visible cycle within the instruction.
    pub fn run_cycle_sequenced_instruction_without_dsp(&mut self) -> u8 {
        assert!(
            self.smp_coroutine.is_idle(),
            "cannot run an atomic SPC instruction while a resumable instruction is in progress"
        );
        let direct_page_base = if self.spc.p { 0x100 } else { 0 };
        let direct_page_0_3 = self.ram[direct_page_base..direct_page_base + 4]
            .try_into()
            .expect("SPC direct page prefix is always four bytes");
        let direct_page_8_11 = self.ram[direct_page_base + 8..direct_page_base + 12]
            .try_into()
            .expect("SPC direct page command latches are always four bytes");
        if let Some(trace) = self.debug_spc_instruction_trace.as_mut() {
            trace.push(SpcInstructionTrace {
                cycle: self.cycles,
                pc: self.spc.pc,
                opcode: self.ram[self.spc.pc as usize],
                operands: [
                    self.ram[self.spc.pc.wrapping_add(1) as usize],
                    self.ram[self.spc.pc.wrapping_add(2) as usize],
                ],
                a: self.spc.a,
                x: self.spc.x,
                y: self.spc.y,
                sp: self.spc.sp,
                p: self.spc.p,
                direct_page_0_3,
                direct_page_8_11,
                input_ports: self.in_ports[..4]
                    .try_into()
                    .expect("SPC input port prefix is always four bytes"),
                timer0_cycles: self.timer[0].cycles,
                timer0_divider: self.timer[0].divider,
                timer0_counter: self.timer[0].counter,
            });
        }
        let state = self.cycle_sequenced_smp_state();
        let (state, cycles) = {
            let mut smp = Smp::new(self, state);
            let cycles = smp.run_instruction();
            (smp.state(), cycles)
        };
        self.apply_cycle_sequenced_smp_state(state);
        self.spc.cycles_used = cycles as u8;
        self.cpu_cycles_left = 0;
        cycles as u8
    }

    /// Run exactly one pinned-Snes9x SMP coroutine pseudo-op boundary.
    ///
    /// This is opt-in: the existing complete-instruction execution path is
    /// unchanged. An unsupported next opcode is reported without fetching it
    /// or advancing the SPC clock, so callers can stop before losing phase.
    pub fn run_snes9x_micro_step_without_dsp(
        &mut self,
    ) -> Result<SmpMicroStepResult, UnsupportedSmpMicroStep> {
        assert!(
            self.smp_coroutine.is_enabled(),
            "Snes9x micro-step execution requires reset_snes9x_coroutine"
        );
        let opcode = self.smp_coroutine.opcode().unwrap_or_else(|| {
            if self.rom_readable && self.spc.pc >= 0xffc0 {
                BOOT_ROM[usize::from(self.spc.pc - 0xffc0)]
            } else {
                self.ram[self.spc.pc as usize]
            }
        });
        if !Smp::supports_resumable_opcode(opcode) {
            return Err(UnsupportedSmpMicroStep {
                opcode,
                pc: self.spc.pc,
            });
        }

        let state = self.cycle_sequenced_smp_state();
        let mut coroutine = std::mem::take(&mut self.smp_coroutine);
        let start_cycles = self.cycles;
        let (state, result) = {
            let mut smp = Smp::new(self, state);
            let result = smp.run_resumable_micro_step(&mut coroutine);
            (smp.state(), result)
        };
        self.smp_coroutine = coroutine;
        let result = result.expect("resumable opcode was validated before its first bus cycle");
        self.apply_cycle_sequenced_smp_state(state);
        self.spc.cycles_used = self.cycles.wrapping_sub(start_cycles) as u8;
        self.cpu_cycles_left = 0;
        Ok(result)
    }

    fn cycle_sequenced_smp_state(&self) -> SmpState {
        SmpState {
            pc: self.spc.pc,
            a: self.spc.a,
            x: self.spc.x,
            y: self.spc.y,
            sp: self.spc.sp,
            c: self.spc.c,
            z: self.spc.z,
            h: self.spc.h,
            p: self.spc.p,
            v: self.spc.v,
            n: self.spc.n,
            i: self.spc.i,
            b: self.spc.b,
            stopped: self.spc.stopped,
        }
    }

    fn apply_cycle_sequenced_smp_state(&mut self, state: SmpState) {
        self.spc.pc = state.pc;
        self.spc.a = state.a;
        self.spc.x = state.x;
        self.spc.y = state.y;
        self.spc.sp = state.sp;
        self.spc.c = state.c;
        self.spc.z = state.z;
        self.spc.h = state.h;
        self.spc.p = state.p;
        self.spc.v = state.v;
        self.spc.n = state.n;
        self.spc.i = state.i;
        self.spc.b = state.b;
        self.spc.stopped = state.stopped;
    }

    /// Latch the host's four input ports immediately before the given local
    /// SMP cycle, including when it falls inside a multi-cycle instruction.
    pub fn schedule_input_port_write(&mut self, local_cycle: u32, ports: [u8; 4]) {
        self.schedule_input_port_writes(std::array::from_fn(|port| {
            (local_cycle, port as u8, ports[port])
        }));
    }

    /// Schedule one host-port write on the SPC hardware clock.
    pub fn schedule_input_port_event(&mut self, local_cycle: u32, port: u8, value: u8) {
        self.scheduled_input_port_writes
            .push((local_cycle, port & 3, value));
        self.scheduled_input_port_writes
            .sort_unstable_by_key(|event| event.0);
    }

    /// Schedule independent host-port writes on the SPC hardware clock.
    /// Events are applied inside instructions before a bus access on the same
    /// cycle, matching the CPU/APU coroutine boundary used by the console.
    pub fn schedule_input_port_writes(&mut self, writes: [(u32, u8, u8); 4]) {
        for (cycle, port, value) in writes {
            self.schedule_input_port_event(cycle, port, value);
        }
    }

    fn cycle_inner(&mut self, render_dsp: bool) {
        assert!(
            self.smp_coroutine.is_idle(),
            "cannot use the atomic SPC cycle path while a resumable instruction is in progress"
        );
        if self.cpu_cycles_left == 0 {
            self.cpu_cycles_left = self.spc_run_opcode();
        }
        self.cpu_cycles_left = self.cpu_cycles_left.wrapping_sub(1);

        self.advance_hardware_cycle(render_dsp);
    }

    fn advance_hardware_cycle(&mut self, render_dsp: bool) {
        while self
            .scheduled_input_port_writes
            .first()
            .is_some_and(|event| event.0 <= self.cycles)
        {
            let (_, port, value) = self.scheduled_input_port_writes.remove(0);
            self.in_ports[usize::from(port & 3)] = value;
        }

        if render_dsp && self.cycles & 0x1f == 0 {
            self.dsp.cycle(&mut self.ram);
        }

        for i in 0..3 {
            if self.timer[i].cycles == 0 {
                self.timer[i].cycles = if i == 2 { 16 } else { 128 };
            }
            self.timer[i].cycles = self.timer[i].cycles.wrapping_sub(1);
            if self.timer[i].cycles == 0 && self.timer[i].enabled {
                self.timer[i].divider = self.timer[i].divider.wrapping_add(1);
                if self.timer[i].divider == self.timer[i].target {
                    self.timer[i].divider = 0;
                    self.timer[i].counter = self.timer[i].counter.wrapping_add(1) & 0x0f;
                }
            }
        }

        self.cycles = self.cycles.wrapping_add(1);
    }

    fn spc_reset(&mut self) {
        self.spc = SpcState::default();
        self.spc.pc = self.spc_read_word(0xfffe, 0xffff);
    }

    fn spc_run_opcode(&mut self) -> u8 {
        self.spc.cycles_used = 0;
        if self.spc.stopped {
            return 1;
        }
        let opcode = self.spc_read_opcode();
        self.spc.cycles_used = SPC_CYCLES_PER_OPCODE[opcode as usize];
        self.spc_do_opcode(opcode);
        self.spc.cycles_used
    }

    fn spc_read_opcode(&mut self) -> u8 {
        let ret = self.cpu_read(self.spc.pc);
        self.spc.pc = self.spc.pc.wrapping_add(1);
        ret
    }

    fn spc_read_opcode_word(&mut self) -> u16 {
        let low = self.spc_read_opcode() as u16;
        low | ((self.spc_read_opcode() as u16) << 8)
    }

    fn spc_read_word(&mut self, adrl: u16, adrh: u16) -> u16 {
        self.cpu_read(adrl) as u16 | ((self.cpu_read(adrh) as u16) << 8)
    }

    fn spc_write_word(&mut self, adrl: u16, adrh: u16, value: u16) {
        self.cpu_write(adrl, value as u8);
        self.cpu_write(adrh, (value >> 8) as u8);
    }

    fn spc_get_flags(&self) -> u8 {
        ((self.spc.n as u8) << 7)
            | ((self.spc.v as u8) << 6)
            | ((self.spc.p as u8) << 5)
            | ((self.spc.b as u8) << 4)
            | ((self.spc.h as u8) << 3)
            | ((self.spc.i as u8) << 2)
            | ((self.spc.z as u8) << 1)
            | self.spc.c as u8
    }

    fn spc_set_flags(&mut self, val: u8) {
        self.spc.n = val & 0x80 != 0;
        self.spc.v = val & 0x40 != 0;
        self.spc.p = val & 0x20 != 0;
        self.spc.b = val & 0x10 != 0;
        self.spc.h = val & 0x08 != 0;
        self.spc.i = val & 0x04 != 0;
        self.spc.z = val & 0x02 != 0;
        self.spc.c = val & 0x01 != 0;
    }

    fn spc_pull_byte(&mut self) -> u8 {
        self.spc.sp = self.spc.sp.wrapping_add(1);
        self.cpu_read(0x100 | self.spc.sp as u16)
    }

    fn spc_push_byte(&mut self, value: u8) {
        self.cpu_write(0x100 | self.spc.sp as u16, value);
        self.spc.sp = self.spc.sp.wrapping_sub(1);
    }

    fn spc_pull_word(&mut self) -> u16 {
        let value = self.spc_pull_byte() as u16;
        value | ((self.spc_pull_byte() as u16) << 8)
    }

    fn spc_push_word(&mut self, value: u16) {
        self.spc_push_byte((value >> 8) as u8);
        self.spc_push_byte(value as u8);
    }

    fn spc_adr_dp(&mut self) -> u16 {
        self.spc_read_opcode() as u16 | ((self.spc.p as u16) << 8)
    }

    fn spc_adr_abs(&mut self) -> u16 {
        self.spc_read_opcode_word()
    }

    fn spc_adr_imm(&mut self) -> u16 {
        let ret = self.spc.pc;
        self.spc.pc = self.spc.pc.wrapping_add(1);
        ret
    }

    fn spc_adr_ind(&self) -> u16 {
        self.spc.x as u16 | ((self.spc.p as u16) << 8)
    }

    fn spc_adr_idx(&mut self) -> u16 {
        let pointer = self.spc_read_opcode().wrapping_add(self.spc.x) as u16;
        self.spc_read_word(
            pointer & 0xff | ((self.spc.p as u16) << 8),
            pointer.wrapping_add(1) & 0xff | ((self.spc.p as u16) << 8),
        )
    }

    fn spc_adr_dpx(&mut self) -> u16 {
        self.spc_read_opcode().wrapping_add(self.spc.x) as u16 | ((self.spc.p as u16) << 8)
    }

    fn spc_adr_dpy(&mut self) -> u16 {
        self.spc_read_opcode().wrapping_add(self.spc.y) as u16 | ((self.spc.p as u16) << 8)
    }

    fn spc_adr_abx(&mut self) -> u16 {
        self.spc_read_opcode_word().wrapping_add(self.spc.x as u16)
    }

    fn spc_adr_aby(&mut self) -> u16 {
        self.spc_read_opcode_word().wrapping_add(self.spc.y as u16)
    }

    fn spc_adr_idy(&mut self) -> u16 {
        let pointer = self.spc_read_opcode() as u16;
        let base = self.spc_read_word(
            pointer | ((self.spc.p as u16) << 8),
            pointer.wrapping_add(1) & 0xff | ((self.spc.p as u16) << 8),
        );
        base.wrapping_add(self.spc.y as u16)
    }

    fn spc_adr_dp_word(&mut self) -> (u16, u16) {
        let adr = self.spc_read_opcode() as u16;
        let low = adr | ((self.spc.p as u16) << 8);
        let high = adr.wrapping_add(1) & 0xff | ((self.spc.p as u16) << 8);
        (low, high)
    }

    fn spc_adr_dp_dp(&mut self) -> (u16, u16) {
        let src = self.spc_read_opcode() as u16 | ((self.spc.p as u16) << 8);
        let dst = self.spc_read_opcode() as u16 | ((self.spc.p as u16) << 8);
        (dst, src)
    }

    fn spc_adr_dp_imm(&mut self) -> (u16, u16) {
        let src = self.spc.pc;
        self.spc.pc = self.spc.pc.wrapping_add(1);
        let dst = self.spc_read_opcode() as u16 | ((self.spc.p as u16) << 8);
        (dst, src)
    }

    fn spc_adr_ind_ind(&mut self) -> (u16, u16) {
        let src = self.spc.y as u16 | ((self.spc.p as u16) << 8);
        let dst = self.spc.x as u16 | ((self.spc.p as u16) << 8);
        (dst, src)
    }

    fn spc_adr_ind_p(&mut self) -> u16 {
        let adr = self.spc.x as u16 | ((self.spc.p as u16) << 8);
        self.spc.x = self.spc.x.wrapping_add(1);
        adr
    }

    fn spc_adr_abs_bit(&mut self) -> (u16, u8) {
        let adr_bit = self.spc_read_opcode_word();
        (adr_bit & 0x1fff, (adr_bit >> 13) as u8)
    }

    fn spc_do_branch(&mut self, rel: u8, check: bool) {
        if check {
            self.spc.cycles_used = self.spc.cycles_used.wrapping_add(2);
            self.spc.pc = self.spc.pc.wrapping_add_signed(rel as i8 as i16);
        }
    }

    fn spc_set_zn(&mut self, value: u8) {
        self.spc.z = value == 0;
        self.spc.n = value & 0x80 != 0;
    }

    fn spc_set_zn_word(&mut self, value: u16) {
        self.spc.z = value == 0;
        self.spc.n = value & 0x8000 != 0;
    }

    fn spc_mov_a(&mut self, adr: u16) {
        self.spc.a = self.cpu_read(adr);
        self.spc_set_zn(self.spc.a);
    }

    fn spc_mov_x(&mut self, adr: u16) {
        self.spc.x = self.cpu_read(adr);
        self.spc_set_zn(self.spc.x);
    }

    fn spc_mov_y(&mut self, adr: u16) {
        self.spc.y = self.cpu_read(adr);
        self.spc_set_zn(self.spc.y);
    }

    fn spc_movs_a(&mut self, adr: u16) {
        let _ = self.cpu_read(adr);
        self.cpu_write(adr, self.spc.a);
    }

    fn spc_movsx(&mut self, adr: u16) {
        let _ = self.cpu_read(adr);
        self.cpu_write(adr, self.spc.x);
    }

    fn spc_movsy(&mut self, adr: u16) {
        let _ = self.cpu_read(adr);
        self.cpu_write(adr, self.spc.y);
    }

    fn spc_or(&mut self, adr: u16) {
        self.spc.a |= self.cpu_read(adr);
        self.spc_set_zn(self.spc.a);
    }

    fn spc_orm(&mut self, dst: u16, src: u16) {
        let value = self.cpu_read(src);
        let result = self.cpu_read(dst) | value;
        self.cpu_write(dst, result);
        self.spc_set_zn(result);
    }

    fn spc_and(&mut self, adr: u16) {
        self.spc.a &= self.cpu_read(adr);
        self.spc_set_zn(self.spc.a);
    }

    fn spc_andm(&mut self, dst: u16, src: u16) {
        let value = self.cpu_read(src);
        let result = self.cpu_read(dst) & value;
        self.cpu_write(dst, result);
        self.spc_set_zn(result);
    }

    fn spc_eor(&mut self, adr: u16) {
        self.spc.a ^= self.cpu_read(adr);
        self.spc_set_zn(self.spc.a);
    }

    fn spc_eorm(&mut self, dst: u16, src: u16) {
        let value = self.cpu_read(src);
        let result = self.cpu_read(dst) ^ value;
        self.cpu_write(dst, result);
        self.spc_set_zn(result);
    }

    fn spc_cmp_a(&mut self, adr: u16) {
        let value = self.cpu_read(adr) ^ 0xff;
        let result = self.spc.a as u16 + value as u16 + 1;
        self.spc.c = result > 0xff;
        self.spc_set_zn(result as u8);
    }

    fn spc_cmp_x(&mut self, adr: u16) {
        let value = self.cpu_read(adr) ^ 0xff;
        let result = self.spc.x as u16 + value as u16 + 1;
        self.spc.c = result > 0xff;
        self.spc_set_zn(result as u8);
    }

    fn spc_cmp_y(&mut self, adr: u16) {
        let value = self.cpu_read(adr) ^ 0xff;
        let result = self.spc.y as u16 + value as u16 + 1;
        self.spc.c = result > 0xff;
        self.spc_set_zn(result as u8);
    }

    fn spc_cmpm(&mut self, dst: u16, src: u16) {
        let value = self.cpu_read(src) ^ 0xff;
        let result = self.cpu_read(dst) as u16 + value as u16 + 1;
        self.spc.c = result > 0xff;
        self.spc_set_zn(result as u8);
    }

    fn spc_adc(&mut self, adr: u16) {
        let value = self.cpu_read(adr);
        let result = self.spc.a as u16 + value as u16 + self.spc.c as u16;
        self.spc.v =
            (self.spc.a & 0x80) == (value & 0x80) && (value & 0x80) != (result as u8 & 0x80);
        self.spc.h = ((self.spc.a & 0x0f) + (value & 0x0f) + self.spc.c as u8) > 0x0f;
        self.spc.c = result > 0xff;
        self.spc.a = result as u8;
        self.spc_set_zn(self.spc.a);
    }

    fn spc_adcm(&mut self, dst: u16, src: u16) {
        let value = self.cpu_read(src);
        let apply_on = self.cpu_read(dst);
        let result = apply_on as u16 + value as u16 + self.spc.c as u16;
        self.spc.v = (apply_on & 0x80) == (value & 0x80) && (value & 0x80) != (result as u8 & 0x80);
        self.spc.h = ((apply_on & 0x0f) + (value & 0x0f) + self.spc.c as u8) > 0x0f;
        self.spc.c = result > 0xff;
        self.cpu_write(dst, result as u8);
        self.spc_set_zn(result as u8);
    }

    fn spc_sbc(&mut self, adr: u16) {
        let value = self.cpu_read(adr) ^ 0xff;
        let result = self.spc.a as u16 + value as u16 + self.spc.c as u16;
        self.spc.v =
            (self.spc.a & 0x80) == (value & 0x80) && (value & 0x80) != (result as u8 & 0x80);
        self.spc.h = ((self.spc.a & 0x0f) + (value & 0x0f) + self.spc.c as u8) > 0x0f;
        self.spc.c = result > 0xff;
        self.spc.a = result as u8;
        self.spc_set_zn(self.spc.a);
    }

    fn spc_sbcm(&mut self, dst: u16, src: u16) {
        let value = self.cpu_read(src) ^ 0xff;
        let apply_on = self.cpu_read(dst);
        let result = apply_on as u16 + value as u16 + self.spc.c as u16;
        self.spc.v = (apply_on & 0x80) == (value & 0x80) && (value & 0x80) != (result as u8 & 0x80);
        self.spc.h = ((apply_on & 0x0f) + (value & 0x0f) + self.spc.c as u8) > 0x0f;
        self.spc.c = result > 0xff;
        self.cpu_write(dst, result as u8);
        self.spc_set_zn(result as u8);
    }

    fn spc_asl(&mut self, adr: u16) {
        let mut val = self.cpu_read(adr);
        self.spc.c = val & 0x80 != 0;
        val = val.wrapping_shl(1);
        self.cpu_write(adr, val);
        self.spc_set_zn(val);
    }

    fn spc_lsr(&mut self, adr: u16) {
        let mut val = self.cpu_read(adr);
        self.spc.c = val & 1 != 0;
        val >>= 1;
        self.cpu_write(adr, val);
        self.spc_set_zn(val);
    }

    fn spc_rol(&mut self, adr: u16) {
        let mut val = self.cpu_read(adr);
        let new_c = val & 0x80 != 0;
        val = val.wrapping_shl(1) | self.spc.c as u8;
        self.spc.c = new_c;
        self.cpu_write(adr, val);
        self.spc_set_zn(val);
    }

    fn spc_ror(&mut self, adr: u16) {
        let mut val = self.cpu_read(adr);
        let new_c = val & 1 != 0;
        val = (val >> 1) | ((self.spc.c as u8) << 7);
        self.spc.c = new_c;
        self.cpu_write(adr, val);
        self.spc_set_zn(val);
    }

    fn spc_inc(&mut self, adr: u16) {
        let value = self.cpu_read(adr).wrapping_add(1);
        self.cpu_write(adr, value);
        self.spc_set_zn(value);
    }

    fn spc_dec(&mut self, adr: u16) {
        let value = self.cpu_read(adr).wrapping_sub(1);
        self.cpu_write(adr, value);
        self.spc_set_zn(value);
    }

    fn spc_do_opcode(&mut self, opcode: u8) {
        match opcode {
            0x00 => {}
            0x01 | 0x11 | 0x21 | 0x31 | 0x41 | 0x51 | 0x61 | 0x71 | 0x81 | 0x91 | 0xa1 | 0xb1
            | 0xc1 | 0xd1 | 0xe1 | 0xf1 => {
                self.spc_push_word(self.spc.pc);
                let adr = 0xffdeu16.wrapping_sub(2 * ((opcode >> 4) as u16));
                self.spc.pc = self.spc_read_word(adr, adr + 1);
            }
            0x02 | 0x22 | 0x42 | 0x62 | 0x82 | 0xa2 | 0xc2 | 0xe2 => {
                let adr = self.spc_adr_dp();
                let bit = 1u8 << (opcode >> 5);
                let value = self.cpu_read(adr) | bit;
                self.cpu_write(adr, value);
            }
            0x03 | 0x23 | 0x43 | 0x63 | 0x83 | 0xa3 | 0xc3 | 0xe3 => {
                let adr = self.spc_adr_dp();
                let val = self.cpu_read(adr);
                let rel = self.spc_read_opcode();
                self.spc_do_branch(rel, val & (1 << (opcode >> 5)) != 0);
            }
            0x04 => {
                let adr = self.spc_adr_dp();
                self.spc_or(adr);
            }
            0x05 => {
                let adr = self.spc_adr_abs();
                self.spc_or(adr);
            }
            0x06 => {
                let adr = self.spc_adr_ind();
                self.spc_or(adr);
            }
            0x07 => {
                let adr = self.spc_adr_idx();
                self.spc_or(adr);
            }
            0x08 => {
                let adr = self.spc_adr_imm();
                self.spc_or(adr);
            }
            0x09 => {
                let (dst, src) = self.spc_adr_dp_dp();
                self.spc_orm(dst, src);
            }
            0x0a => {
                let (adr, bit) = self.spc_adr_abs_bit();
                self.spc.c |= (self.cpu_read(adr) >> bit) & 1 != 0;
            }
            0x0b => {
                let adr = self.spc_adr_dp();
                self.spc_asl(adr);
            }
            0x0c => {
                let adr = self.spc_adr_abs();
                self.spc_asl(adr);
            }
            0x0d => {
                self.spc_push_byte(self.spc_get_flags());
            }
            0x0e => {
                let adr = self.spc_adr_abs();
                let val = self.cpu_read(adr);
                let result = self.spc.a.wrapping_add(val ^ 0xff).wrapping_add(1);
                self.spc_set_zn(result);
                self.cpu_write(adr, val | self.spc.a);
            }
            0x0f => {
                self.spc_push_word(self.spc.pc);
                self.spc_push_byte(self.spc_get_flags());
                self.spc.i = false;
                self.spc.b = true;
                self.spc.pc = self.spc_read_word(0xffde, 0xffdf);
            }
            0x10 => {
                let rel = self.spc_read_opcode();
                self.spc_do_branch(rel, !self.spc.n);
            }
            0x12 | 0x32 | 0x52 | 0x72 | 0x92 | 0xb2 | 0xd2 | 0xf2 => {
                let adr = self.spc_adr_dp();
                let bit = 1u8 << (opcode >> 5);
                let value = self.cpu_read(adr) & !bit;
                self.cpu_write(adr, value);
            }
            0x13 | 0x33 | 0x53 | 0x73 | 0x93 | 0xb3 | 0xd3 | 0xf3 => {
                let adr = self.spc_adr_dp();
                let val = self.cpu_read(adr);
                let rel = self.spc_read_opcode();
                self.spc_do_branch(rel, val & (1 << (opcode >> 5)) == 0);
            }
            0x14 => {
                let adr = self.spc_adr_dpx();
                self.spc_or(adr);
            }
            0x15 => {
                let adr = self.spc_adr_abx();
                self.spc_or(adr);
            }
            0x16 => {
                let adr = self.spc_adr_aby();
                self.spc_or(adr);
            }
            0x17 => {
                let adr = self.spc_adr_idy();
                self.spc_or(adr);
            }
            0x18 => {
                let (dst, src) = self.spc_adr_dp_imm();
                self.spc_orm(dst, src);
            }
            0x19 => {
                let (dst, src) = self.spc_adr_ind_ind();
                self.spc_orm(dst, src);
            }
            0x1a => {
                let (low, high) = self.spc_adr_dp_word();
                let value = self.spc_read_word(low, high).wrapping_sub(1);
                self.spc_set_zn_word(value);
                self.spc_write_word(low, high, value);
            }
            0x1b => {
                let adr = self.spc_adr_dpx();
                self.spc_asl(adr);
            }
            0x1c => {
                self.spc.c = self.spc.a & 0x80 != 0;
                self.spc.a = self.spc.a.wrapping_shl(1);
                self.spc_set_zn(self.spc.a);
            }
            0x1d => {
                self.spc.x = self.spc.x.wrapping_sub(1);
                self.spc_set_zn(self.spc.x);
            }
            0x1e => {
                let adr = self.spc_adr_abs();
                self.spc_cmp_x(adr);
            }
            0x1f => {
                let pointer = self.spc_read_opcode_word();
                self.spc.pc = self.spc_read_word(
                    pointer.wrapping_add(self.spc.x as u16),
                    pointer.wrapping_add(self.spc.x as u16).wrapping_add(1),
                );
            }
            0x20 => self.spc.p = false,
            0x24 => {
                let adr = self.spc_adr_dp();
                self.spc_and(adr);
            }
            0x25 => {
                let adr = self.spc_adr_abs();
                self.spc_and(adr);
            }
            0x26 => {
                let adr = self.spc_adr_ind();
                self.spc_and(adr);
            }
            0x27 => {
                let adr = self.spc_adr_idx();
                self.spc_and(adr);
            }
            0x28 => {
                let adr = self.spc_adr_imm();
                self.spc_and(adr);
            }
            0x29 => {
                let (dst, src) = self.spc_adr_dp_dp();
                self.spc_andm(dst, src);
            }
            0x2a => {
                let (adr, bit) = self.spc_adr_abs_bit();
                self.spc.c |= !((self.cpu_read(adr) >> bit) & 1 != 0);
            }
            0x2b => {
                let adr = self.spc_adr_dp();
                self.spc_rol(adr);
            }
            0x2c => {
                let adr = self.spc_adr_abs();
                self.spc_rol(adr);
            }
            0x2d => self.spc_push_byte(self.spc.a),
            0x2e => {
                let adr = self.spc_adr_dp();
                let val = self.cpu_read(adr) ^ 0xff;
                let result = self.spc.a.wrapping_add(val).wrapping_add(1);
                let rel = self.spc_read_opcode();
                self.spc_do_branch(rel, result != 0);
            }
            0x2f => {
                let rel = self.spc_read_opcode();
                self.spc.pc = self.spc.pc.wrapping_add_signed(rel as i8 as i16);
            }
            0x30 => {
                let rel = self.spc_read_opcode();
                self.spc_do_branch(rel, self.spc.n);
            }
            0x34 => {
                let adr = self.spc_adr_dpx();
                self.spc_and(adr);
            }
            0x35 => {
                let adr = self.spc_adr_abx();
                self.spc_and(adr);
            }
            0x36 => {
                let adr = self.spc_adr_aby();
                self.spc_and(adr);
            }
            0x37 => {
                let adr = self.spc_adr_idy();
                self.spc_and(adr);
            }
            0x38 => {
                let (dst, src) = self.spc_adr_dp_imm();
                self.spc_andm(dst, src);
            }
            0x39 => {
                let (dst, src) = self.spc_adr_ind_ind();
                self.spc_andm(dst, src);
            }
            0x3a => {
                let (low, high) = self.spc_adr_dp_word();
                let value = self.spc_read_word(low, high).wrapping_add(1);
                self.spc_set_zn_word(value);
                self.spc_write_word(low, high, value);
            }
            0x3b => {
                let adr = self.spc_adr_dpx();
                self.spc_rol(adr);
            }
            0x3c => {
                let new_c = self.spc.a & 0x80 != 0;
                self.spc.a = self.spc.a.wrapping_shl(1) | self.spc.c as u8;
                self.spc.c = new_c;
                self.spc_set_zn(self.spc.a);
            }
            0x3d => {
                self.spc.x = self.spc.x.wrapping_add(1);
                self.spc_set_zn(self.spc.x);
            }
            0x3e => {
                let adr = self.spc_adr_dp();
                self.spc_cmp_x(adr);
            }
            0x3f => {
                let dst = self.spc_read_opcode_word();
                self.spc_push_word(self.spc.pc);
                self.spc.pc = dst;
            }
            0x40 => self.spc.p = true,
            0x44 => {
                let adr = self.spc_adr_dp();
                self.spc_eor(adr);
            }
            0x45 => {
                let adr = self.spc_adr_abs();
                self.spc_eor(adr);
            }
            0x46 => {
                let adr = self.spc_adr_ind();
                self.spc_eor(adr);
            }
            0x47 => {
                let adr = self.spc_adr_idx();
                self.spc_eor(adr);
            }
            0x48 => {
                let adr = self.spc_adr_imm();
                self.spc_eor(adr);
            }
            0x49 => {
                let (dst, src) = self.spc_adr_dp_dp();
                self.spc_eorm(dst, src);
            }
            0x4a => {
                let (adr, bit) = self.spc_adr_abs_bit();
                self.spc.c &= (self.cpu_read(adr) >> bit) & 1 != 0;
            }
            0x4b => {
                let adr = self.spc_adr_dp();
                self.spc_lsr(adr);
            }
            0x4c => {
                let adr = self.spc_adr_abs();
                self.spc_lsr(adr);
            }
            0x4d => self.spc_push_byte(self.spc.x),
            0x4e => {
                let adr = self.spc_adr_abs();
                let val = self.cpu_read(adr);
                let result = self.spc.a.wrapping_add(val ^ 0xff).wrapping_add(1);
                self.spc_set_zn(result);
                self.cpu_write(adr, val & !self.spc.a);
            }
            0x4f => {
                let dst = self.spc_read_opcode() as u16;
                self.spc_push_word(self.spc.pc);
                self.spc.pc = 0xff00 | dst;
            }
            0x50 => {
                let rel = self.spc_read_opcode();
                self.spc_do_branch(rel, !self.spc.v);
            }
            0x54 => {
                let adr = self.spc_adr_dpx();
                self.spc_eor(adr);
            }
            0x55 => {
                let adr = self.spc_adr_abx();
                self.spc_eor(adr);
            }
            0x56 => {
                let adr = self.spc_adr_aby();
                self.spc_eor(adr);
            }
            0x57 => {
                let adr = self.spc_adr_idy();
                self.spc_eor(adr);
            }
            0x58 => {
                let (dst, src) = self.spc_adr_dp_imm();
                self.spc_eorm(dst, src);
            }
            0x59 => {
                let (dst, src) = self.spc_adr_ind_ind();
                self.spc_eorm(dst, src);
            }
            0x5a => {
                let (low, high) = self.spc_adr_dp_word();
                let value = self.spc_read_word(low, high) ^ 0xffff;
                let ya = self.spc.a as u16 | ((self.spc.y as u16) << 8);
                let result = ya as u32 + value as u32 + 1;
                self.spc.c = result > 0xffff;
                self.spc.z = (result as u16) == 0;
                self.spc.n = result as u16 & 0x8000 != 0;
            }
            0x5b => {
                let adr = self.spc_adr_dpx();
                self.spc_lsr(adr);
            }
            0x5c => {
                self.spc.c = self.spc.a & 1 != 0;
                self.spc.a >>= 1;
                self.spc_set_zn(self.spc.a);
            }
            0x5d => {
                self.spc.x = self.spc.a;
                self.spc_set_zn(self.spc.x);
            }
            0x5e => {
                let adr = self.spc_adr_abs();
                self.spc_cmp_y(adr);
            }
            0x5f => {
                self.spc.pc = self.spc_read_opcode_word();
            }
            0x60 => self.spc.c = false,
            0x64 => {
                let adr = self.spc_adr_dp();
                self.spc_cmp_a(adr);
            }
            0x65 => {
                let adr = self.spc_adr_abs();
                self.spc_cmp_a(adr);
            }
            0x66 => {
                let adr = self.spc_adr_ind();
                self.spc_cmp_a(adr);
            }
            0x67 => {
                let adr = self.spc_adr_idx();
                self.spc_cmp_a(adr);
            }
            0x68 => {
                let adr = self.spc_adr_imm();
                self.spc_cmp_a(adr);
            }
            0x69 => {
                let (dst, src) = self.spc_adr_dp_dp();
                self.spc_cmpm(dst, src);
            }
            0x6a => {
                let (adr, bit) = self.spc_adr_abs_bit();
                self.spc.c &= !((self.cpu_read(adr) >> bit) & 1 != 0);
            }
            0x6b => {
                let adr = self.spc_adr_dp();
                self.spc_ror(adr);
            }
            0x6c => {
                let adr = self.spc_adr_abs();
                self.spc_ror(adr);
            }
            0x6d => self.spc_push_byte(self.spc.y),
            0x6e => {
                let adr = self.spc_adr_dp();
                let result = self.cpu_read(adr).wrapping_sub(1);
                self.cpu_write(adr, result);
                let rel = self.spc_read_opcode();
                self.spc_do_branch(rel, result != 0);
            }
            0x6f => self.spc.pc = self.spc_pull_word(),
            0x70 => {
                let rel = self.spc_read_opcode();
                self.spc_do_branch(rel, self.spc.v);
            }
            0x74 => {
                let adr = self.spc_adr_dpx();
                self.spc_cmp_a(adr);
            }
            0x75 => {
                let adr = self.spc_adr_abx();
                self.spc_cmp_a(adr);
            }
            0x76 => {
                let adr = self.spc_adr_aby();
                self.spc_cmp_a(adr);
            }
            0x77 => {
                let adr = self.spc_adr_idy();
                self.spc_cmp_a(adr);
            }
            0x78 => {
                let src = self.spc_adr_imm();
                let dst = self.spc_adr_dp();
                self.spc_cmpm(dst, src);
            }
            0x79 => {
                let (dst, src) = self.spc_adr_ind_ind();
                self.spc_cmpm(dst, src);
            }
            0x7a => {
                let (low, high) = self.spc_adr_dp_word();
                let value = self.spc_read_word(low, high);
                let ya = self.spc.a as u16 | ((self.spc.y as u16) << 8);
                let result = ya as u32 + value as u32;
                self.spc.v = (ya & 0x8000) == (value & 0x8000)
                    && (value & 0x8000) != (result as u16 & 0x8000);
                self.spc.h = ((ya & 0x0fff) + (value & 0x0fff) + 1) > 0x0fff;
                self.spc.c = result > 0xffff;
                self.spc_set_zn_word(result as u16);
                self.spc.a = result as u8;
                self.spc.y = (result >> 8) as u8;
            }
            0x7b => {
                let adr = self.spc_adr_dpx();
                self.spc_ror(adr);
            }
            0x7c => {
                let new_c = self.spc.a & 1 != 0;
                self.spc.a = (self.spc.a >> 1) | ((self.spc.c as u8) << 7);
                self.spc.c = new_c;
                self.spc_set_zn(self.spc.a);
            }
            0x7d => {
                self.spc.a = self.spc.x;
                self.spc_set_zn(self.spc.a);
            }
            0x7e => {
                let adr = self.spc_adr_dp();
                self.spc_cmp_y(adr);
            }
            0x7f => {
                let flags = self.spc_pull_byte();
                self.spc_set_flags(flags);
                self.spc.pc = self.spc_pull_word();
            }
            0x80 => self.spc.c = true,
            0x84 => {
                let adr = self.spc_adr_dp();
                self.spc_adc(adr);
            }
            0x85 => {
                let adr = self.spc_adr_abs();
                self.spc_adc(adr);
            }
            0x86 => {
                let adr = self.spc_adr_ind();
                self.spc_adc(adr);
            }
            0x87 => {
                let adr = self.spc_adr_idx();
                self.spc_adc(adr);
            }
            0x88 => {
                let adr = self.spc_adr_imm();
                self.spc_adc(adr);
            }
            0x89 => {
                let (dst, src) = self.spc_adr_dp_dp();
                self.spc_adcm(dst, src);
            }
            0x8a => {
                let (adr, bit) = self.spc_adr_abs_bit();
                self.spc.c ^= (self.cpu_read(adr) >> bit) & 1 != 0;
            }
            0x8b => {
                let adr = self.spc_adr_dp();
                self.spc_dec(adr);
            }
            0x8c => {
                let adr = self.spc_adr_abs();
                self.spc_dec(adr);
            }
            0x8d => {
                let adr = self.spc_adr_imm();
                self.spc_mov_y(adr);
            }
            0x8e => {
                let flags = self.spc_pull_byte();
                self.spc_set_flags(flags);
            }
            0x8f => {
                let src = self.spc_adr_imm();
                let dst = self.spc_adr_dp();
                let val = self.cpu_read(src);
                let _ = self.cpu_read(dst);
                self.cpu_write(dst, val);
            }
            0x90 => {
                let rel = self.spc_read_opcode();
                self.spc_do_branch(rel, !self.spc.c);
            }
            0x94 => {
                let adr = self.spc_adr_dpx();
                self.spc_adc(adr);
            }
            0x95 => {
                let adr = self.spc_adr_abx();
                self.spc_adc(adr);
            }
            0x96 => {
                let adr = self.spc_adr_aby();
                self.spc_adc(adr);
            }
            0x97 => {
                let adr = self.spc_adr_idy();
                self.spc_adc(adr);
            }
            0x98 => {
                let (dst, src) = self.spc_adr_dp_imm();
                self.spc_adcm(dst, src);
            }
            0x99 => {
                let (dst, src) = self.spc_adr_ind_ind();
                self.spc_adcm(dst, src);
            }
            0x9a => {
                let (low, high) = self.spc_adr_dp_word();
                let value = self.spc_read_word(low, high) ^ 0xffff;
                let ya = self.spc.a as u16 | ((self.spc.y as u16) << 8);
                let result = ya as u32 + value as u32 + 1;
                self.spc.v = (ya & 0x8000) == (value & 0x8000)
                    && (value & 0x8000) != (result as u16 & 0x8000);
                self.spc.h = ((ya & 0x0fff) + (value & 0x0fff) + 1) > 0x0fff;
                self.spc.c = result > 0xffff;
                self.spc_set_zn_word(result as u16);
                self.spc.a = result as u8;
                self.spc.y = (result >> 8) as u8;
            }
            0x9b => {
                let adr = self.spc_adr_dpx();
                self.spc_dec(adr);
            }
            0x9c => {
                self.spc.a = self.spc.a.wrapping_sub(1);
                self.spc_set_zn(self.spc.a);
            }
            0x9d => {
                self.spc.x = self.spc.sp;
                self.spc_set_zn(self.spc.x);
            }
            0x9e => {
                let value = self.spc.a as u16 | ((self.spc.y as u16) << 8);
                let mut result = 0xffffu16;
                let mut rem = self.spc.a;
                if self.spc.x != 0 {
                    result = value / self.spc.x as u16;
                    rem = (value % self.spc.x as u16) as u8;
                }
                self.spc.v = result > 0xff;
                self.spc.h = (self.spc.x & 0x0f) <= (self.spc.y & 0x0f);
                self.spc.a = result as u8;
                self.spc.y = rem;
                self.spc_set_zn(self.spc.a);
            }
            0x9f => {
                self.spc.a = (self.spc.a >> 4) | self.spc.a.wrapping_shl(4);
                self.spc_set_zn(self.spc.a);
            }
            0xa0 => self.spc.i = true,
            0xa4 => {
                let adr = self.spc_adr_dp();
                self.spc_sbc(adr);
            }
            0xa5 => {
                let adr = self.spc_adr_abs();
                self.spc_sbc(adr);
            }
            0xa6 => {
                let adr = self.spc_adr_ind();
                self.spc_sbc(adr);
            }
            0xa7 => {
                let adr = self.spc_adr_idx();
                self.spc_sbc(adr);
            }
            0xa8 => {
                let adr = self.spc_adr_imm();
                self.spc_sbc(adr);
            }
            0xa9 => {
                let (dst, src) = self.spc_adr_dp_dp();
                self.spc_sbcm(dst, src);
            }
            0xaa => {
                let (adr, bit) = self.spc_adr_abs_bit();
                self.spc.c = (self.cpu_read(adr) >> bit) & 1 != 0;
            }
            0xab => {
                let adr = self.spc_adr_dp();
                let value = self.cpu_read(adr).wrapping_add(1);
                self.cpu_write(adr, value);
                self.spc_set_zn(value);
            }
            0xac => {
                let adr = self.spc_adr_abs();
                self.spc_inc(adr);
            }
            0xad => {
                let adr = self.spc_adr_imm();
                self.spc_cmp_y(adr);
            }
            0xae => {
                self.spc.a = self.spc_pull_byte();
            }
            0xaf => {
                let adr = self.spc_adr_ind_p();
                self.cpu_write(adr, self.spc.a);
            }
            0xb0 => {
                let rel = self.spc_read_opcode();
                self.spc_do_branch(rel, self.spc.c);
            }
            0xb4 => {
                let adr = self.spc_adr_dpx();
                self.spc_sbc(adr);
            }
            0xb5 => {
                let adr = self.spc_adr_abx();
                self.spc_sbc(adr);
            }
            0xb6 => {
                let adr = self.spc_adr_aby();
                self.spc_sbc(adr);
            }
            0xb7 => {
                let adr = self.spc_adr_idy();
                self.spc_sbc(adr);
            }
            0xb8 => {
                let (dst, src) = self.spc_adr_dp_imm();
                self.spc_sbcm(dst, src);
            }
            0xb9 => {
                let (dst, src) = self.spc_adr_ind_ind();
                self.spc_sbcm(dst, src);
            }
            0xba => {
                let (low, high) = self.spc_adr_dp_word();
                let val = self.spc_read_word(low, high);
                self.spc.a = val as u8;
                self.spc.y = (val >> 8) as u8;
                self.spc_set_zn_word(val);
            }
            0xbb => {
                let adr = self.spc_adr_dpx();
                self.spc_inc(adr);
            }
            0xbc => {
                self.spc.a = self.spc.a.wrapping_add(1);
                self.spc_set_zn(self.spc.a);
            }
            0xbd => {
                self.spc.sp = self.spc.x;
            }
            0xbe => {
                if self.spc.a > 0x99 || !self.spc.c {
                    self.spc.a = self.spc.a.wrapping_sub(0x60);
                    self.spc.c = false;
                }
                if (self.spc.a & 0x0f) > 9 || !self.spc.h {
                    self.spc.a = self.spc.a.wrapping_sub(6);
                }
                self.spc_set_zn(self.spc.a);
            }
            0xbf => {
                let adr = self.spc_adr_ind_p();
                self.spc.a = self.cpu_read(adr);
                self.spc_set_zn(self.spc.a);
            }
            0xc0 => self.spc.i = false,
            0xc4 => {
                let adr = self.spc_adr_dp();
                self.spc_movs_a(adr);
            }
            0xc5 => {
                let adr = self.spc_adr_abs();
                self.spc_movs_a(adr);
            }
            0xc6 => {
                let adr = self.spc_adr_ind();
                self.spc_movs_a(adr);
            }
            0xc7 => {
                let adr = self.spc_adr_idx();
                self.spc_movs_a(adr);
            }
            0xc8 => {
                let adr = self.spc_adr_imm();
                self.spc_cmp_x(adr);
            }
            0xc9 => {
                let adr = self.spc_adr_abs();
                self.spc_movsx(adr);
            }
            0xca => {
                let (adr, bit) = self.spc_adr_abs_bit();
                let result = (self.cpu_read(adr) & !(1 << bit)) | ((self.spc.c as u8) << bit);
                self.cpu_write(adr, result);
            }
            0xcb => {
                let adr = self.spc_adr_dp();
                self.spc_movsy(adr);
            }
            0xcc => {
                let adr = self.spc_adr_abs();
                self.spc_movsy(adr);
            }
            0xcd => {
                let adr = self.spc_adr_imm();
                self.spc_mov_x(adr);
            }
            0xce => {
                self.spc.x = self.spc_pull_byte();
            }
            0xcf => {
                let result = self.spc.a as u16 * self.spc.y as u16;
                self.spc.a = result as u8;
                self.spc.y = (result >> 8) as u8;
                self.spc_set_zn(self.spc.y);
            }
            0xd0 => {
                let rel = self.spc_read_opcode();
                self.spc_do_branch(rel, !self.spc.z);
            }
            0xd4 => {
                let adr = self.spc_adr_dpx();
                self.spc_movs_a(adr);
            }
            0xd5 => {
                let adr = self.spc_adr_abx();
                self.spc_movs_a(adr);
            }
            0xd6 => {
                let adr = self.spc_adr_aby();
                self.spc_movs_a(adr);
            }
            0xd7 => {
                let adr = self.spc_adr_idy();
                self.spc_movs_a(adr);
            }
            0xd8 => {
                let adr = self.spc_adr_dp();
                self.spc_movsx(adr);
            }
            0xd9 => {
                let adr = self.spc_adr_dpy();
                self.spc_movsx(adr);
            }
            0xda => {
                let (low, high) = self.spc_adr_dp_word();
                let _ = self.cpu_read(low);
                self.spc_write_word(low, high, self.spc.a as u16 | ((self.spc.y as u16) << 8));
            }
            0xdb => {
                let adr = self.spc_adr_dpx();
                self.spc_movsy(adr);
            }
            0xdc => {
                self.spc.y = self.spc.y.wrapping_sub(1);
                self.spc_set_zn(self.spc.y);
            }
            0xdd => {
                self.spc.a = self.spc.y;
                self.spc_set_zn(self.spc.a);
            }
            0xde => {
                let adr = self.spc_adr_dpx();
                let val = self.cpu_read(adr) ^ 0xff;
                let result = self.spc.a.wrapping_add(val).wrapping_add(1);
                let rel = self.spc_read_opcode();
                self.spc_do_branch(rel, result != 0);
            }
            0xdf => {
                if self.spc.a > 0x99 || self.spc.c {
                    self.spc.a = self.spc.a.wrapping_add(0x60);
                    self.spc.c = true;
                }
                if (self.spc.a & 0x0f) > 9 || self.spc.h {
                    self.spc.a = self.spc.a.wrapping_add(6);
                }
                self.spc_set_zn(self.spc.a);
            }
            0xe0 => {
                self.spc.v = false;
                self.spc.h = false;
            }
            0xe4 => {
                let adr = self.spc_adr_dp();
                self.spc_mov_a(adr);
            }
            0xe5 => {
                let adr = self.spc_adr_abs();
                self.spc_mov_a(adr);
            }
            0xe6 => {
                let adr = self.spc_adr_ind();
                self.spc_mov_a(adr);
            }
            0xe7 => {
                let adr = self.spc_adr_idx();
                self.spc_mov_a(adr);
            }
            0xe8 => {
                let adr = self.spc_adr_imm();
                self.spc_mov_a(adr);
            }
            0xe9 => {
                let adr = self.spc_adr_abs();
                self.spc_mov_x(adr);
            }
            0xea => {
                let (adr, bit) = self.spc_adr_abs_bit();
                let result = self.cpu_read(adr) ^ (1 << bit);
                self.cpu_write(adr, result);
            }
            0xeb => {
                let adr = self.spc_adr_dp();
                self.spc_mov_y(adr);
            }
            0xec => {
                let adr = self.spc_adr_abs();
                self.spc_mov_y(adr);
            }
            0xed => self.spc.c = !self.spc.c,
            0xee => {
                self.spc.y = self.spc_pull_byte();
            }
            0xef => {
                self.spc.stopped = true;
            }
            0xf0 => {
                let rel = self.spc_read_opcode();
                self.spc_do_branch(rel, self.spc.z);
            }
            0xf4 => {
                let adr = self.spc_adr_dpx();
                self.spc_mov_a(adr);
            }
            0xf5 => {
                let adr = self.spc_adr_abx();
                self.spc_mov_a(adr);
            }
            0xf6 => {
                let adr = self.spc_adr_aby();
                self.spc_mov_a(adr);
            }
            0xf7 => {
                let adr = self.spc_adr_idy();
                self.spc_mov_a(adr);
            }
            0xf8 => {
                let adr = self.spc_adr_dp();
                self.spc_mov_x(adr);
            }
            0xf9 => {
                let adr = self.spc_adr_dpy();
                self.spc_mov_x(adr);
            }
            0xfa => {
                let (dst, src) = self.spc_adr_dp_dp();
                let val = self.cpu_read(src);
                self.cpu_write(dst, val);
            }
            0xfb => {
                let adr = self.spc_adr_dpx();
                self.spc_mov_y(adr);
            }
            0xfc => {
                self.spc.y = self.spc.y.wrapping_add(1);
                self.spc_set_zn(self.spc.y);
            }
            0xfd => {
                self.spc.y = self.spc.a;
                self.spc_set_zn(self.spc.y);
            }
            0xfe => {
                self.spc.y = self.spc.y.wrapping_sub(1);
                let rel = self.spc_read_opcode();
                self.spc_do_branch(rel, self.spc.y != 0);
            }
            0xff => {
                self.spc.stopped = true;
            }
        }
    }

    pub fn read_snes_port(&self, port: u8) -> u8 {
        self.out_ports[(port & 3) as usize]
    }

    pub fn write_snes_port(&mut self, port: u8, val: u8) {
        self.in_ports[(port & 3) as usize] = val;
    }

    pub fn cpu_read(&mut self, adr: u16) -> u8 {
        match adr {
            0xf0 | 0xf1 | 0xfa | 0xfb | 0xfc => 0,
            0xf2 => self.dsp_adr,
            0xf3 => self.dsp.read(self.dsp_adr & 0x7f),
            0xf4..=0xf9 => self.in_ports[(adr - 0xf4) as usize],
            0xfd..=0xff => {
                let i = (adr - 0xfd) as usize;
                let ret = self.timer[i].counter;
                self.timer[i].counter = 0;
                ret
            }
            _ if self.rom_readable && adr >= 0xffc0 => BOOT_ROM[(adr - 0xffc0) as usize],
            _ => self.ram[adr as usize],
        }
    }

    pub fn cpu_write(&mut self, adr: u16, val: u8) {
        match adr {
            0xf0 => {}
            0xf1 => {
                for i in 0..3 {
                    if !self.timer[i].enabled && val & (1 << i) != 0 {
                        self.timer[i].divider = 0;
                        self.timer[i].counter = 0;
                    }
                    self.timer[i].enabled = val & (1 << i) != 0;
                }
                if val & 0x10 != 0 {
                    self.in_ports[0] = 0;
                    self.in_ports[1] = 0;
                }
                if val & 0x20 != 0 {
                    self.in_ports[2] = 0;
                    self.in_ports[3] = 0;
                }
                self.rom_readable = val & 0x80 != 0;
            }
            0xf2 => self.dsp_adr = val,
            0xf3 => {
                if self.dsp_write_history.len() < 256 {
                    self.dsp_write_history.push((self.dsp_adr, val));
                }
                if let Some(trace) = self.debug_dsp_write_trace.as_mut() {
                    trace.push((self.cycles, self.dsp_adr, val));
                }
                if self.dsp_adr < 0x80 {
                    self.dsp.write(self.dsp_adr, val, &self.ram);
                    self.dsp_regs[self.dsp_adr as usize] = self.dsp.read(self.dsp_adr);
                }
            }
            0xf4..=0xf7 => self.out_ports[(adr - 0xf4) as usize] = val,
            0xf8..=0xf9 => self.in_ports[(adr - 0xf4) as usize] = val,
            0xfa..=0xfc => self.timer[(adr - 0xfa) as usize].target = val,
            _ => {}
        }
        self.ram[adr as usize] = val;
    }
}

impl SmpBus for ApuState {
    fn cycles(&mut self, cycles: i32) {
        for _ in 0..cycles {
            self.advance_hardware_cycle(false);
        }
    }

    fn read(&mut self, address: u16) -> u8 {
        self.cpu_read(address)
    }

    fn write(&mut self, address: u16, value: u8) {
        self.cpu_write(address, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_bootstrap_fixture::{
        cpu_apu_accesses, first_cc_output_port_writes, records, smp_instruction_boundaries,
        smp_output_port_writes, split_first_cc_cpu_accesses,
    };

    fn pinned_snes9x_bootstrap_fixture() -> Vec<serde_json::Value> {
        records()
    }

    fn advance_ipl_to_first_output_store(apu: &mut ApuState) {
        for _ in 0..1_000 {
            if apu.spc.pc == 0xffc9 {
                assert_eq!(apu.cycles, 2_394);
                return;
            }
            apu.run_cycle_sequenced_instruction_without_dsp();
        }
        panic!(
            "IPL did not reach its first MOV dp,#imm at cycle {}, PC ${:04x}",
            apu.cycles, apu.spc.pc
        );
    }

    #[test]
    fn scheduled_input_ports_latch_independently_inside_the_cycle_stream() {
        let mut apu = ApuState::new();
        apu.schedule_input_port_writes([(1, 0, 10), (3, 1, 20), (3, 2, 30), (5, 3, 40)]);

        apu.advance_hardware_cycle(false);
        assert_eq!(&apu.in_ports[..4], &[0, 0, 0, 0]);
        apu.advance_hardware_cycle(false);
        assert_eq!(&apu.in_ports[..4], &[10, 0, 0, 0]);
        apu.advance_hardware_cycle(false);
        apu.advance_hardware_cycle(false);
        assert_eq!(&apu.in_ports[..4], &[10, 20, 30, 0]);
        apu.advance_hardware_cycle(false);
        apu.advance_hardware_cycle(false);
        assert_eq!(&apu.in_ports[..4], &[10, 20, 30, 40]);
    }

    #[test]
    fn gaussian_interpolation_matches_snes9x_1_63_rounding() {
        assert_eq!(dsp_gaussian_interpolate(6482, 7612, 8672, 9666, 219), 8516);
        assert_eq!(dsp_gaussian_interpolate(6784, 2266, -1018, -3044, 184), -6);
    }

    #[test]
    fn exact_dsp_drain_preserves_callback_remainder() {
        let mut dsp = DspState::default();
        dsp.sample_buffer[..8].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        dsp.sample_offset = 4;
        let mut output = [0i16; 6];

        dsp.drain_samples_exact(&mut output, 3, 2).unwrap();

        assert_eq!(output, [1, 2, 3, 4, 5, 6]);
        assert_eq!(dsp.sample_offset, 1);
        assert_eq!(&dsp.sample_buffer[..2], &[7, 8]);
    }

    #[test]
    fn boot_rom_is_visible_until_control_clears_it() {
        let mut apu = ApuState::new();
        assert_eq!(apu.cpu_read(0xffc0), 0xcd);
        apu.cpu_write(0xf1, 0);
        assert_eq!(apu.cpu_read(0xffc0), 0);
    }

    #[test]
    fn legacy_apu_reset_retains_c_port_sp_and_flags() {
        let mut apu = ApuState::new();
        apu.reset();

        assert_eq!(apu.spc.pc, 0xffc0);
        assert_eq!(apu.spc.sp, 0);
        assert_eq!(
            (
                apu.spc.c, apu.spc.z, apu.spc.h, apu.spc.p, apu.spc.v, apu.spc.n, apu.spc.i,
                apu.spc.b,
            ),
            (false, false, false, false, false, false, false, false)
        );
        assert_eq!(apu.cpu_cycles_left, 7);
        assert_eq!(
            apu.spc.save_c_saveload(),
            vec![0, 0, 0, 0, 0xc0, 0xff, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(apu.save_c_saveload_prefix()[0x10021], 7);
    }

    #[test]
    fn opt_in_snes9x_coroutine_reset_is_source_exact() {
        let mut apu = ApuState::new();
        apu.reset_snes9x_coroutine();
        let fixture = pinned_snes9x_bootstrap_fixture();
        let reset = &fixture[1];

        assert_eq!(u64::from(apu.spc.pc), reset["pc"].as_u64().unwrap());
        assert_eq!(u64::from(apu.spc.sp), reset["sp"].as_u64().unwrap());
        assert_eq!(
            (
                apu.spc.c, apu.spc.z, apu.spc.h, apu.spc.p, apu.spc.v, apu.spc.n, apu.spc.i,
                apu.spc.b,
            ),
            (false, true, false, false, false, false, false, false)
        );
        assert_eq!(reset["status"], 0x02);
        assert_eq!(apu.cpu_cycles_left, 0);
    }

    #[test]
    fn resumable_ipl_stores_publish_on_exact_snes9x_cycles() {
        let mut apu = ApuState::new();
        apu.reset_snes9x_coroutine();
        advance_ipl_to_first_output_store(&mut apu);
        let fixture = pinned_snes9x_bootstrap_fixture();
        let writes = smp_output_port_writes(&fixture[2]);
        let aa_cycle = writes[0].absolute_cycle;
        let bb_cycle = writes[1].absolute_cycle;
        assert_eq!(writes[0].origin_pc, 0xffc9);
        assert_eq!(writes[0].opcode, 0x8f);
        assert_eq!(writes[0].port, 0);
        assert_eq!(writes[0].value, 0xaa);
        assert_eq!(writes[1].origin_pc, 0xffcc);
        assert_eq!(writes[1].opcode, 0x8f);
        assert_eq!(writes[1].port, 1);
        assert_eq!(writes[1].value, 0xbb);
        assert_eq!(apu.out_ports[..2], [0, 0]);

        assert_eq!(
            apu.run_snes9x_micro_step_without_dsp().unwrap(),
            SmpMicroStepResult::InProgress {
                opcode: 0x8f,
                opcode_cycle: 1,
            }
        );
        assert_eq!(apu.cycles, 2_397);
        assert_eq!(apu.out_ports[..2], [0, 0]);
        assert_eq!(
            apu.run_snes9x_micro_step_without_dsp().unwrap(),
            SmpMicroStepResult::InProgress {
                opcode: 0x8f,
                opcode_cycle: 2,
            }
        );
        assert_eq!(apu.cycles, 2_398);
        assert_eq!(apu.out_ports[..2], [0, 0]);
        assert_eq!(
            apu.run_snes9x_micro_step_without_dsp().unwrap(),
            SmpMicroStepResult::InstructionComplete { opcode: 0x8f }
        );
        assert_eq!(apu.cycles, aa_cycle);
        assert_eq!(apu.out_ports[..2], [0xaa, 0]);

        assert_eq!(
            apu.run_snes9x_micro_step_without_dsp().unwrap(),
            SmpMicroStepResult::InProgress {
                opcode: 0x8f,
                opcode_cycle: 1,
            }
        );
        assert_eq!(apu.cycles, 2_402);
        assert_eq!(apu.out_ports[..2], [0xaa, 0]);
        assert_eq!(
            apu.run_snes9x_micro_step_without_dsp().unwrap(),
            SmpMicroStepResult::InProgress {
                opcode: 0x8f,
                opcode_cycle: 2,
            }
        );
        assert_eq!(apu.cycles, 2_403);
        assert_eq!(apu.out_ports[..2], [0xaa, 0]);
        assert_eq!(
            apu.run_snes9x_micro_step_without_dsp().unwrap(),
            SmpMicroStepResult::InstructionComplete { opcode: 0x8f }
        );
        assert_eq!(apu.cycles, bb_cycle);
        assert_eq!(apu.out_ports[..2], [0xaa, 0xbb]);
    }

    #[test]
    fn resumable_only_cold_ipl_reaches_fixture_cc_handshake() {
        let fixture = pinned_snes9x_bootstrap_fixture();
        let bootstrap = &fixture[2];
        let all_expected_writes = smp_output_port_writes(bootstrap);
        let expected_writes = first_cc_output_port_writes(&all_expected_writes);
        let cc_cycle = expected_writes.last().unwrap().absolute_cycle;
        let expected_instructions = smp_instruction_boundaries(bootstrap)
            .into_iter()
            .take_while(|instruction| instruction.absolute_end_cycle <= cc_cycle)
            .map(|instruction| {
                (
                    instruction.origin_pc,
                    instruction.opcode,
                    instruction.absolute_start_cycle,
                    instruction.absolute_end_cycle,
                    instruction.op_step_calls,
                    instruction.max_continuation_opcode_cycle,
                )
            })
            .collect::<Vec<_>>();

        let mut apu = ApuState::new();
        apu.reset_snes9x_coroutine();
        let cpu_accesses = cpu_apu_accesses(bootstrap);
        let (_, handshake) = split_first_cc_cpu_accesses(&cpu_accesses);
        for event in handshake.iter().filter(|event| !event.is_read) {
            apu.schedule_input_port_event(event.apu_cycle_after, event.port, event.value);
        }

        let mut observed_writes = Vec::new();
        let mut observed_instructions = Vec::new();
        let mut instruction = None;
        let mut previous_ports = apu.out_ports;
        for _ in 0..5_000 {
            if apu.smp_coroutine.is_idle() {
                let origin_pc = apu.spc.pc;
                let opcode = apu.cpu_read(origin_pc);
                instruction = Some((origin_pc, opcode, apu.cycles, 0u8, 0u8));
            }
            let result = apu.run_snes9x_micro_step_without_dsp().unwrap();
            let (_, result_opcode_cycle) = match result {
                SmpMicroStepResult::InProgress {
                    opcode,
                    opcode_cycle,
                } => (opcode, opcode_cycle),
                SmpMicroStepResult::InstructionComplete { opcode } => (opcode, 0),
            };
            let active = instruction.as_mut().unwrap();
            active.3 += 1;
            active.4 = active.4.max(result_opcode_cycle);
            for port in 0..4 {
                if apu.out_ports[port] != previous_ports[port] {
                    observed_writes.push((
                        apu.cycles,
                        active.0,
                        match result {
                            SmpMicroStepResult::InProgress { opcode, .. }
                            | SmpMicroStepResult::InstructionComplete { opcode } => opcode,
                        },
                        port as u8,
                        apu.out_ports[port],
                    ));
                }
            }
            previous_ports = apu.out_ports;
            if matches!(result, SmpMicroStepResult::InstructionComplete { .. }) {
                let (origin_pc, opcode, start_cycle, op_step_calls, max_opcode_cycle) =
                    instruction.take().unwrap();
                observed_instructions.push((
                    origin_pc,
                    opcode,
                    start_cycle,
                    apu.cycles,
                    op_step_calls,
                    max_opcode_cycle,
                ));
            }
            if apu.out_ports[0] == 0xcc {
                assert_eq!(
                    result,
                    SmpMicroStepResult::InstructionComplete { opcode: 0xc4 }
                );
                break;
            }
        }

        assert_eq!(observed_instructions.len(), expected_instructions.len());
        assert_eq!(observed_instructions, expected_instructions);
        let expected = expected_writes
            .iter()
            .map(|event| {
                (
                    event.absolute_cycle,
                    event.origin_pc,
                    event.opcode,
                    event.port,
                    event.value,
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(observed_writes, expected);
        assert_eq!(apu.cycles, 2_461);
        assert_eq!(apu.spc.pc, 0xfff7);
        assert_eq!(apu.out_ports, [0xcc, 0xbb, 0, 0]);
    }

    #[test]
    fn resumable_ipl_uploads_zelda_driver_prefix_and_hands_off_to_0800() {
        // Actual Zelda ROM bytes at $99:8000, the start of the $0800 SPC
        // driver upload. Zero-filling the rest of its first transfer page keeps
        // the source bytes while deterministically exercising IPL page carry.
        const ZELDA_DRIVER_PREFIX: [u8; 4] = [0x20, 0xcd, 0xcf, 0xbd];
        let mut upload_page = [0; 0x100];
        upload_page[..ZELDA_DRIVER_PREFIX.len()].copy_from_slice(&ZELDA_DRIVER_PREFIX);

        let mut apu = ApuState::new();
        apu.reset_snes9x_coroutine();
        apu.write_snes_port(0, 0xcc);
        apu.write_snes_port(1, 1);
        apu.write_snes_port(2, 0x00);
        apu.write_snes_port(3, 0x08);

        let mut seen_opcodes = std::collections::BTreeSet::new();
        for _ in 0..5_000 {
            if apu.smp_coroutine.is_idle() {
                seen_opcodes.insert(apu.cpu_read(apu.spc.pc));
            }
            apu.run_snes9x_micro_step_without_dsp().unwrap();
            if apu.out_ports[0] == 0xcc && apu.spc.pc == 0xfff7 {
                break;
            }
        }
        assert_eq!(apu.out_ports[..2], [0xcc, 0xbb]);

        apu.write_snes_port(0, 0);
        apu.write_snes_port(1, upload_page[0]);
        let mut previous_ack = 0xcc;
        let mut terminal_sent = false;
        let mut active_instruction = None;
        let mut post_cc_shapes = std::collections::BTreeSet::new();
        for _ in 0..20_000 {
            if apu.smp_coroutine.is_idle() {
                let opcode = apu.cpu_read(apu.spc.pc);
                seen_opcodes.insert(opcode);
                active_instruction = Some((opcode, apu.cycles, 0u8, 0u8));
            }
            let result = apu.run_snes9x_micro_step_without_dsp().unwrap();
            let active = active_instruction.as_mut().unwrap();
            active.2 += 1;
            if let SmpMicroStepResult::InProgress { opcode_cycle, .. } = result {
                active.3 = active.3.max(opcode_cycle);
            }
            if matches!(result, SmpMicroStepResult::InstructionComplete { .. }) {
                let (opcode, start_cycle, calls, max_opcode_cycle) =
                    active_instruction.take().unwrap();
                post_cc_shapes.insert((opcode, apu.cycles - start_cycle, calls, max_opcode_cycle));
            }

            let ack = apu.out_ports[0];
            if ack != previous_ack {
                previous_ack = ack;
                let byte_index = usize::from(ack);
                if !terminal_sent && byte_index < upload_page.len() {
                    if byte_index + 1 < upload_page.len() {
                        apu.write_snes_port(1, upload_page[byte_index + 1]);
                        apu.write_snes_port(0, ack.wrapping_add(1));
                    } else {
                        // Skip one counter value to end the block, provide the
                        // terminal entry address, and set port1=0 so IPL exits.
                        apu.write_snes_port(0, ack.wrapping_add(2));
                        apu.write_snes_port(1, 0);
                        apu.write_snes_port(2, 0x00);
                        apu.write_snes_port(3, 0x08);
                        terminal_sent = true;
                    }
                }
            }
            if apu.spc.pc == 0x0800 {
                break;
            }
        }

        assert_eq!(&apu.ram[0x0800..0x0900], &upload_page);
        assert_eq!(apu.spc.pc, 0x0800);
        for opcode in [
            0xdd, 0x5d, 0xeb, 0x7e, 0xe4, 0xcb, 0xd7, 0xfc, 0xab, 0x10, 0x1f,
        ] {
            assert!(
                seen_opcodes.contains(&opcode),
                "IPL did not execute ${opcode:02x}"
            );
        }
        for shape in [
            (0xdd, 2, 1, 0),
            (0x5d, 2, 1, 0),
            (0xeb, 3, 2, 1),
            (0x7e, 3, 2, 1),
            (0xe4, 3, 2, 1),
            (0xcb, 4, 3, 2),
            (0xd7, 7, 5, 4),
            (0xfc, 2, 1, 0),
            (0xab, 4, 1, 0),
            (0x10, 2, 1, 0),
            (0x10, 4, 1, 0),
            (0x1f, 6, 1, 0),
        ] {
            assert!(
                post_cc_shapes.contains(&shape),
                "IPL did not execute source pseudo-step shape {shape:?}"
            );
        }
    }

    #[test]
    fn d7_indirect_upload_write_restores_mid_pseudo_case_scratch() {
        let mut apu = ApuState::new();
        apu.reset_snes9x_coroutine();
        apu.spc.pc = 0xffe2;
        apu.spc.a = 0x5a;
        apu.spc.y = 3;
        apu.ram[0] = 0x00;
        apu.ram[1] = 0x08;

        for expected_cycle in [1, 2, 3] {
            assert_eq!(
                apu.run_snes9x_micro_step_without_dsp().unwrap(),
                SmpMicroStepResult::InProgress {
                    opcode: 0xd7,
                    opcode_cycle: expected_cycle,
                }
            );
        }
        assert_eq!(apu.cycles, 5);

        let checkpoint = apu.capture_snes9x_coroutine_checkpoint().unwrap();
        let checkpoint: Snes9xSmpCoroutineCheckpoint =
            serde_json::from_slice(&serde_json::to_vec(&checkpoint).unwrap()).unwrap();
        let mut restored: ApuState =
            serde_json::from_slice(&serde_json::to_vec(&apu).unwrap()).unwrap();
        restored.restore_snes9x_coroutine_checkpoint(checkpoint);

        for resumed in [&mut apu, &mut restored] {
            assert_eq!(
                resumed.run_snes9x_micro_step_without_dsp().unwrap(),
                SmpMicroStepResult::InProgress {
                    opcode: 0xd7,
                    opcode_cycle: 4,
                }
            );
            assert_eq!(resumed.cycles, 6);
            assert_eq!(resumed.ram[0x0803], 0);
            assert_eq!(
                resumed.run_snes9x_micro_step_without_dsp().unwrap(),
                SmpMicroStepResult::InstructionComplete { opcode: 0xd7 }
            );
            assert_eq!(resumed.cycles, 7);
            assert_eq!(resumed.spc.pc, 0xffe4);
            assert_eq!(resumed.ram[0x0803], 0x5a);
        }
    }

    #[test]
    fn unsupported_resumable_opcode_does_not_advance_or_fetch() {
        let mut apu = ApuState::new();
        apu.reset_snes9x_coroutine();
        apu.rom_readable = false;
        apu.spc.pc = 0x0200;
        apu.ram[0x0200] = 0x00;

        let before = (apu.cycles, apu.spc.pc);
        assert_eq!(
            apu.run_snes9x_micro_step_without_dsp().unwrap_err(),
            UnsupportedSmpMicroStep {
                opcode: 0x00,
                pc: 0x0200,
            }
        );
        assert_eq!((apu.cycles, apu.spc.pc), before);
    }

    #[test]
    fn every_pending_snes9x_split_fails_before_fetch_or_state_advance() {
        // Exact members of the pinned 37-opcode split set whose source stages
        // have not yet been ported. Implemented non-store and store plans are
        // covered separately in `cycle_spc700` tests.
        for opcode in [
            0xaa, 0xbf, 0xca, 0xe5, 0xe6, 0xe7, 0xe9, 0xec, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9,
            0xfa, 0xfb,
        ] {
            let mut apu = ApuState::new();
            apu.reset_snes9x_coroutine();
            apu.rom_readable = false;
            apu.spc.pc = 0x0200;
            apu.ram[0x0200] = opcode;
            let machine_before = serde_json::to_vec(&apu).unwrap();
            let coroutine_before =
                serde_json::to_vec(&apu.capture_snes9x_coroutine_checkpoint().unwrap()).unwrap();

            assert_eq!(
                apu.run_snes9x_micro_step_without_dsp().unwrap_err(),
                UnsupportedSmpMicroStep { opcode, pc: 0x0200 }
            );
            assert_eq!(serde_json::to_vec(&apu).unwrap(), machine_before);
            assert_eq!(
                serde_json::to_vec(&apu.capture_snes9x_coroutine_checkpoint().unwrap()).unwrap(),
                coroutine_before
            );
        }
    }

    #[test]
    fn resumable_smp_state_survives_clone_and_serde_mid_instruction() {
        let mut apu = ApuState::new();
        apu.reset_snes9x_coroutine();
        advance_ipl_to_first_output_store(&mut apu);
        assert!(matches!(
            apu.run_snes9x_micro_step_without_dsp().unwrap(),
            SmpMicroStepResult::InProgress { opcode: 0x8f, .. }
        ));
        assert_eq!(apu.cycles, 2_397);

        let checkpoint = apu.capture_snes9x_coroutine_checkpoint().unwrap();
        let checkpoint_json = serde_json::to_vec(&checkpoint).unwrap();
        let checkpoint: Snes9xSmpCoroutineCheckpoint =
            serde_json::from_slice(&checkpoint_json).unwrap();

        let mut cloned = apu.clone();
        cloned.smp_coroutine = SmpCoroutineState::default();
        cloned.restore_snes9x_coroutine_checkpoint(checkpoint);

        let machine_json = serde_json::to_vec(&apu).unwrap();
        let mut restored: ApuState = serde_json::from_slice(&machine_json).unwrap();
        assert!(restored.capture_snes9x_coroutine_checkpoint().is_none());
        restored.restore_snes9x_coroutine_checkpoint(checkpoint);
        for resumed in [&mut cloned, &mut restored] {
            assert_eq!(
                resumed.run_snes9x_micro_step_without_dsp().unwrap(),
                SmpMicroStepResult::InProgress {
                    opcode: 0x8f,
                    opcode_cycle: 2,
                }
            );
            assert_eq!(resumed.cycles, 2_398);
            assert_eq!(resumed.out_ports[0], 0);
            assert_eq!(
                resumed.run_snes9x_micro_step_without_dsp().unwrap(),
                SmpMicroStepResult::InstructionComplete { opcode: 0x8f }
            );
            assert_eq!(resumed.cycles, 2_399);
            assert_eq!(resumed.out_ports[0], 0xaa);
        }
    }

    #[test]
    fn coroutine_sidecar_is_absent_from_apu_machine_serialization() {
        let ordinary = ApuState::new();
        let ordinary_json = serde_json::to_vec(&ordinary).unwrap();

        let mut with_sidecar = ordinary.clone();
        with_sidecar.smp_coroutine = SmpCoroutineState::enabled();
        let with_sidecar_json = serde_json::to_vec(&with_sidecar).unwrap();

        assert_eq!(with_sidecar_json, ordinary_json);
        assert!(!String::from_utf8(ordinary_json)
            .unwrap()
            .contains("smp_coroutine"));
    }

    #[test]
    fn timers_increment_and_clear_on_read() {
        let mut apu = ApuState::new();
        apu.cpu_write(0xfa, 1);
        apu.cpu_write(0xf1, 1);
        for _ in 0..128 {
            apu.cycle();
        }
        assert_eq!(apu.cpu_read(0xfd), 1);
        assert_eq!(apu.cpu_read(0xfd), 0);
    }

    #[test]
    fn timer_tick_is_visible_to_mmio_read_on_the_same_hardware_cycle() {
        let mut apu = ApuState::new();
        apu.rom_readable = false;
        apu.spc.pc = 0x0200;
        apu.ram[0x0200..0x0203].copy_from_slice(&[0xec, 0xfd, 0x00]);
        apu.timer[0] = ApuTimer {
            cycles: 4,
            divider: 0,
            target: 1,
            counter: 0,
            enabled: true,
        };

        apu.run_cycle_sequenced_instruction_without_dsp();

        assert_eq!(apu.spc.y, 1);
        assert_eq!(apu.timer[0].counter, 0);
    }

    #[test]
    fn timing_only_cycle_matches_spc_and_timers_without_rendering_dsp() {
        let mut full = ApuState::new();
        full.reset();
        let mut timing_only = full.clone();

        for _ in 0..1_000 {
            full.cycle();
            timing_only.cycle_without_dsp();
        }

        assert_eq!(timing_only.cycles, full.cycles);
        assert_eq!(timing_only.cpu_cycles_left, full.cpu_cycles_left);
        assert_eq!(
            timing_only.spc.save_c_saveload(),
            full.spc.save_c_saveload()
        );
        assert_eq!(timing_only.ram, full.ram);
        assert_eq!(timing_only.in_ports, full.in_ports);
        assert_eq!(timing_only.out_ports, full.out_ports);
        for (timing, rendered) in timing_only.timer.iter().zip(&full.timer) {
            assert_eq!(timing.cycles, rendered.cycles);
            assert_eq!(timing.divider, rendered.divider);
            assert_eq!(timing.target, rendered.target);
            assert_eq!(timing.counter, rendered.counter);
            assert_eq!(timing.enabled, rendered.enabled);
        }
        assert_eq!(timing_only.dsp.sample_offset, 0);
        assert!(full.dsp.sample_offset > 0);
    }

    #[test]
    fn imports_instruction_boundary_snes9x_smp_state() {
        const RAM_BYTES: usize = 0x10000;
        let mut snd = vec![0; RAM_BYTES + 41 * 4 + 0x80];
        snd[0x0800] = 0x20;
        snd[0xf4..0xf8].copy_from_slice(&[1, 2, 3, 4]);
        let mut cursor = RAM_BYTES;
        let mut push = |value: u32| {
            snd[cursor..cursor + 4].copy_from_slice(&value.to_le_bytes());
            cursor += 4;
        };
        push(0); // clock
        push(0xeb); // previous opcode number
        push(0); // instruction boundary
        push(0x0f9b); // pc
        push(0xcb); // sp
        push(0x20); // a
        push(4); // x
        push(10); // y
        for flag in [0, 0, 0, 0, 0, 0, 1, 0] {
            push(flag);
        }
        push(0); // IPL hidden
        push(0x11); // DSP address
        push(0xaa); // $f8
        push(0xbb); // $f9
        for value in [1, 16, 113, 8, 3, 0, 0, 113, 0, 0, 0, 0, 1, 0, 0] {
            push(value);
        }
        for _ in 0..6 {
            push(0);
        }
        drop(push);
        snd[RAM_BYTES + 41 * 4 + 0x4c] = 0x80;

        let mut apu = ApuState::new();
        apu.load_snes9x_1_63_smp_state(&snd).unwrap();

        assert_eq!(apu.spc.pc, 0x0f9b);
        assert_eq!(
            (apu.spc.a, apu.spc.x, apu.spc.y, apu.spc.sp),
            (0x20, 4, 10, 0xcb)
        );
        assert!(apu.spc.z);
        assert!(!apu.rom_readable);
        assert_eq!(apu.dsp_adr, 0x11);
        assert_eq!(apu.in_ports, [0, 0, 0, 0, 0xaa, 0xbb]);
        assert_eq!(apu.out_ports, [1, 2, 3, 4]);
        assert_eq!(apu.timer[0].cycles, 15);
        assert_eq!(apu.timer[0].divider, 8);
        assert_eq!(apu.timer[0].counter, 3);
        assert_eq!(apu.timer[2].cycles, 15);
        assert_eq!(apu.dsp_regs[0x4c], 0x80);
        assert_eq!(apu.cpu_cycles_left, 0);
    }

    #[test]
    fn dsp_write_records_register_value() {
        let mut apu = ApuState::new();
        apu.debug_dsp_write_trace = Some(Vec::new());
        apu.cpu_write(0xf2, 0x6c);
        apu.cpu_write(0xf3, 0x80);
        assert_eq!(apu.cpu_read(0xf3), 0x80);
        assert_eq!(apu.dsp_write_history, vec![(0x6c, 0x80)]);
        assert_eq!(apu.debug_dsp_write_trace, Some(vec![(0, 0x6c, 0x80)]));
    }

    #[test]
    fn dsp_key_on_waits_for_internal_pipeline() {
        let mut dsp = DspState::default();
        let mut apu_ram = vec![0u8; 0x10000];
        apu_ram[0] = 0x34;
        apu_ram[1] = 0x12;
        dsp.reset_flag = false;
        dsp.channel[0].decode_offset = 0xabcd;

        dsp.write(DSP_KON, 1, &apu_ram);
        assert!(dsp.channel[0].key_on);
        for _ in 0..6 {
            dsp.cycle(&mut apu_ram);
            assert_eq!(dsp.channel[0].decode_offset, 0xabcd);
        }

        dsp.cycle(&mut apu_ram);
        assert!(!dsp.channel[0].key_on);
        assert_eq!(dsp.channel[0].decode_offset, 0x1234);
    }

    #[test]
    fn dsp_c_saveload_layout_roundtrips_saved_fields() {
        let mut dsp = DspState::default();
        dsp.ram[0x4c] = 0x80;
        dsp.channel[3].pitch = 0x1234;
        dsp.channel[3].decode_buffer[18] = -1234;
        dsp.channel[3].volume_l = -17;
        dsp.dir_page = 0x3c00;
        dsp.master_volume_l = -64;
        dsp.noise_sample = -0x1234;
        dsp.echo_buffer_index = 0x4567;
        dsp.fir_values[7] = -8;
        dsp.fir_buffer_r[6] = -2222;
        dsp.sample_buffer[1067] = 12345;
        dsp.sample_offset = 321;

        let saved = dsp.save_c_saveload();
        assert_eq!(saved.len(), DSP_SAVELOAD_SIZE);
        assert_eq!(saved[0x4c], 0x80);
        assert_eq!(&saved[128 + 3 * 86..128 + 3 * 86 + 2], [0x34, 0x12]);
        assert_eq!(&saved[3020..3022], &321u16.to_le_bytes());

        let mut loaded = DspState::default();
        loaded.load_c_saveload(&saved).unwrap();
        assert_eq!(loaded.ram[0x4c], 0x80);
        assert_eq!(loaded.channel[3].pitch, 0x1234);
        assert_eq!(loaded.channel[3].decode_buffer[18], -1234);
        assert_eq!(loaded.channel[3].volume_l, -17);
        assert_eq!(loaded.dir_page, 0x3c00);
        assert_eq!(loaded.master_volume_l, -64);
        assert_eq!(loaded.noise_sample, -0x1234);
        assert_eq!(loaded.echo_buffer_index, 0x4567);
        assert_eq!(loaded.fir_values[7], -8);
        assert_eq!(loaded.fir_buffer_r[6], -2222);
        assert_eq!(loaded.sample_buffer[1067], 12345);
        assert_eq!(loaded.sample_offset, 321);
    }

    #[test]
    fn dsp_pitch_modulation_wraps_negative_product_like_c() {
        let mut dsp = DspState::default();
        let mut ram = vec![0; 0x10000];
        dsp.channel[0].sample_out = -0x8000;
        dsp.channel[1].pitch = 0x1000;
        dsp.channel[1].pitch_modulation = true;

        dsp.cycle_channel(&mut ram, 1);

        assert_eq!(dsp.channel[1].pitch_counter, 0x3fff);
    }

    #[test]
    fn snes_ports_cross_between_cpu_sides() {
        let mut apu = ApuState::new();
        apu.cpu_write(0xf4, 0x12);
        assert_eq!(apu.read_snes_port(0), 0x12);
        apu.write_snes_port(1, 0x34);
        assert_eq!(apu.cpu_read(0xf5), 0x34);
    }
}
