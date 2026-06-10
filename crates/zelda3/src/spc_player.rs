//! Audio sequence player ported from `src/spc_player.c`.

#![allow(non_camel_case_types, non_snake_case)]

use std::ptr;

use snes::apu::DspState;

pub type uint8 = u8;
pub type uint16 = u16;
pub type uint8_t = u8;

const V0VOLL: uint8 = 0x00;
const V0VOLR: uint8 = 0x01;
const V0PITCHL: uint8 = 0x02;
const V0PITCHH: uint8 = 0x03;
const V0SRCN: uint8 = 0x04;
const V0ADSR1: uint8 = 0x05;
const V0ADSR2: uint8 = 0x06;
const V0GAIN: uint8 = 0x07;
const MVOLL: uint8 = 0x0c;
const MVOLR: uint8 = 0x1c;
const EVOLL: uint8 = 0x2c;
const EVOLR: uint8 = 0x3c;
const KON: uint8 = 0x4c;
const KOF: uint8 = 0x5c;
const FLG: uint8 = 0x6c;
const EFB: uint8 = 0x0d;
const PMON: uint8 = 0x2d;
const NON: uint8 = 0x3d;
const EON: uint8 = 0x4d;
const DIR: uint8 = 0x5d;
const ESA: uint8 = 0x6d;
const EDL: uint8 = 0x7d;
const FIR0: uint8 = 0x0f;

#[derive(Clone)]
#[repr(C)]
pub struct DspRegWriteHistory {
    pub count: usize,
    pub addr: [uint8; 256],
    pub val: [uint8; 256],
    pub sample_offset: [i32; 256],
    pub timer_cycles: [uint8; 256],
}

impl Default for DspRegWriteHistory {
    fn default() -> Self {
        Self {
            count: 0,
            addr: [0; 256],
            val: [0; 256],
            sample_offset: [0; 256],
            timer_cycles: [0; 256],
        }
    }
}

#[derive(Clone)]
#[repr(C)]
pub struct Dsp {
    pub sampleOffset: i32,
    ram: *mut uint8,
    core: DspState,
}

impl Default for Dsp {
    fn default() -> Self {
        Self {
            sampleOffset: 0,
            ram: ptr::null_mut(),
            core: DspState::default(),
        }
    }
}

#[repr(C)]
pub struct Apu {
    pub ram: [uint8; 65536],
    pub hist: DspRegWriteHistory,
}

impl Default for Apu {
    fn default() -> Self {
        Self {
            ram: [0; 65536],
            hist: DspRegWriteHistory::default(),
        }
    }
}

#[derive(Clone, Default)]
#[repr(C)]
pub struct Channel {
    pub pattern_order_ptr_for_chan: uint16,
    pub note_ticks_left: uint8,
    pub note_keyoff_ticks_left: uint8,
    pub subroutine_num_loops: uint8,
    pub volume_fade_ticks: uint8,
    pub pan_num_ticks: uint8,
    pub pitch_slide_length: uint8,
    pub pitch_slide_delay_left: uint8,
    pub vibrato_hold_count: uint8,
    pub vib_depth: uint8,
    pub tremolo_hold_count: uint8,
    pub tremolo_depth: uint8,
    pub vibrato_change_count: uint8,
    pub note_length: uint8,
    pub note_gate_off_fixedpt: uint8,
    pub channel_volume_master: uint8,
    pub instrument_id: uint8,
    pub instrument_pitch_base: uint16,
    pub saved_pattern_ptr: uint16,
    pub pattern_start_ptr: uint16,
    pub pitch_envelope_num_ticks: uint8,
    pub pitch_envelope_delay: uint8,
    pub pitch_envelope_direction: uint8,
    pub pitch_envelope_slide_value: uint8,
    pub vibrato_count: uint8,
    pub vibrato_rate: uint8,
    pub vibrato_delay_ticks: uint8,
    pub vibrato_fade_num_ticks: uint8,
    pub vibrato_fade_add_per_tick: uint8,
    pub vibrato_depth_target: uint8,
    pub tremolo_count: uint8,
    pub tremolo_rate: uint8,
    pub tremolo_delay_ticks: uint8,
    pub channel_transposition: uint8,
    pub channel_volume: uint16,
    pub volume_fade_addpertick: uint16,
    pub volume_fade_target: uint8,
    pub final_volume: uint8,
    pub pan_value: uint16,
    pub pan_add_per_tick: uint16,
    pub pan_target_value: uint8,
    pub pan_flag_with_phase_invert: uint8,
    pub pitch: uint16,
    pub pitch_add_per_tick: uint16,
    pub pitch_target: uint8,
    pub fine_tune: uint8,
    pub sfx_sound_ptr: uint16,
    pub sfx_which_sound: uint8,
    pub sfx_arr_countdown: uint8,
    pub sfx_note_length_left: uint8,
    pub sfx_note_length: uint8,
    pub sfx_pan: uint8,
    pub index: uint8,
}

#[derive(Clone)]
#[repr(C)]
pub struct SpcPlayer {
    pub reg_write_history: *mut DspRegWriteHistory,
    pub timer_cycles: uint8,
    pub dsp: *mut Dsp,
    pub new_value_from_snes: [uint8; 4],
    pub port_to_snes: [uint8; 4],
    pub last_value_from_snes: [uint8; 4],
    pub counter_sf0c: uint8,
    pub _always_zero: uint16,
    pub temp_accum: uint16,
    pub ttt: uint8,
    pub did_affect_volumepitch_flag: uint8,
    pub addr0: uint16,
    pub addr1: uint16,
    pub lfsr_value: uint16,
    pub is_chan_on: uint8,
    pub fast_forward: uint8,
    pub sfx_start_arg_pan: uint8,
    pub sfx_sound_ptr_cur: uint16,
    pub music_ptr_toplevel: uint16,
    pub block_count: uint8,
    pub sfx_timer_accum: uint8,
    pub chn: uint8,
    pub key_ON: uint8,
    pub key_OFF: uint8,
    pub cur_chan_bit: uint8,
    pub reg_FLG: uint8,
    pub reg_NON: uint8,
    pub reg_EON: uint8,
    pub reg_PMON: uint8,
    pub echo_stored_time: uint8,
    pub echo_parameter_EDL: uint8,
    pub reg_EFB: uint8,
    pub global_transposition: uint8,
    pub main_tempo_accum: uint8,
    pub tempo: uint16,
    pub tempo_fade_num_ticks: uint8,
    pub tempo_fade_final: uint8,
    pub tempo_fade_add: uint16,
    pub master_volume: uint16,
    pub master_volume_fade_ticks: uint8,
    pub master_volume_fade_target: uint8,
    pub master_volume_fade_add_per_tick: uint16,
    pub vol_dirty: uint8,
    pub percussion_base_id: uint8,
    pub echo_volume_left: uint16,
    pub echo_volume_right: uint16,
    pub echo_volume_fade_add_left: uint16,
    pub echo_volume_fade_add_right: uint16,
    pub echo_volume_fade_ticks: uint8,
    pub echo_volume_fade_target_left: uint8,
    pub echo_volume_fade_target_right: uint8,
    pub sfx_channel_index: uint8,
    pub current_bit: uint8,
    pub dsp_register_index: uint8,
    pub echo_channels: uint8,
    pub byte_3C4: uint8,
    pub byte_3C5: uint8,
    pub echo_fract_incr: uint8,
    pub sfx_channel_index2: uint8,
    pub sfx_channel_bit: uint8,
    pub pause_music_ctr: uint8,
    pub port2_active: uint8,
    pub port2_current_bit: uint8,
    pub port3_active: uint8,
    pub port3_current_bit: uint8,
    pub port1_active: uint8,
    pub port1_current_bit: uint8,
    pub byte_3E1: uint8,
    pub sfx_play_echo_flag: uint8,
    pub sfx_channels_echo_mask2: uint8,
    pub port1_counter: uint8,
    pub channel_67_volume: uint8,
    pub cutk_always_zero: uint8,
    pub last_written_edl: uint8,
    pub input_ports: [uint8; 4],
    pub channel: [Channel; 8],
    pub ram: [uint8; 65536],
}

impl Default for SpcPlayer {
    fn default() -> Self {
        Self {
            reg_write_history: ptr::null_mut(),
            timer_cycles: 0,
            dsp: ptr::null_mut(),
            new_value_from_snes: [0; 4],
            port_to_snes: [0; 4],
            last_value_from_snes: [0; 4],
            counter_sf0c: 0,
            _always_zero: 0,
            temp_accum: 0,
            ttt: 0,
            did_affect_volumepitch_flag: 0,
            addr0: 0,
            addr1: 0,
            lfsr_value: 0,
            is_chan_on: 0,
            fast_forward: 0,
            sfx_start_arg_pan: 0,
            sfx_sound_ptr_cur: 0,
            music_ptr_toplevel: 0,
            block_count: 0,
            sfx_timer_accum: 0,
            chn: 0,
            key_ON: 0,
            key_OFF: 0,
            cur_chan_bit: 0,
            reg_FLG: 0,
            reg_NON: 0,
            reg_EON: 0,
            reg_PMON: 0,
            echo_stored_time: 0,
            echo_parameter_EDL: 0,
            reg_EFB: 0,
            global_transposition: 0,
            main_tempo_accum: 0,
            tempo: 0,
            tempo_fade_num_ticks: 0,
            tempo_fade_final: 0,
            tempo_fade_add: 0,
            master_volume: 0,
            master_volume_fade_ticks: 0,
            master_volume_fade_target: 0,
            master_volume_fade_add_per_tick: 0,
            vol_dirty: 0,
            percussion_base_id: 0,
            echo_volume_left: 0,
            echo_volume_right: 0,
            echo_volume_fade_add_left: 0,
            echo_volume_fade_add_right: 0,
            echo_volume_fade_ticks: 0,
            echo_volume_fade_target_left: 0,
            echo_volume_fade_target_right: 0,
            sfx_channel_index: 0,
            current_bit: 0,
            dsp_register_index: 0,
            echo_channels: 0,
            byte_3C4: 0,
            byte_3C5: 0,
            echo_fract_incr: 0,
            sfx_channel_index2: 0,
            sfx_channel_bit: 0,
            pause_music_ctr: 0,
            port2_active: 0,
            port2_current_bit: 0,
            port3_active: 0,
            port3_current_bit: 0,
            port1_active: 0,
            port1_current_bit: 0,
            byte_3E1: 0,
            sfx_play_echo_flag: 0,
            sfx_channels_echo_mask2: 0,
            port1_counter: 0,
            channel_67_volume: 0,
            cutk_always_zero: 0,
            last_written_edl: 0,
            input_ports: [0; 4],
            channel: std::array::from_fn(|i| Channel {
                index: i as uint8,
                ..Channel::default()
            }),
            ram: [0; 65536],
        }
    }
}

fn word(ram: &[uint8; 65536], addr: usize) -> uint16 {
    ram[addr & 0xffff] as uint16 | ((ram[(addr + 1) & 0xffff] as uint16) << 8)
}

fn hi(x: uint16) -> uint8 {
    (x >> 8) as uint8
}

fn set_hi(x: &mut uint16, v: uint8) {
    *x = (*x & 0x00ff) | ((v as uint16) << 8);
}

fn write_byte(ram: &mut [uint8; 65536], addr: usize, value: uint8) {
    ram[addr & 0xffff] = value;
}

fn read_byte(ram: &[uint8; 65536], addr: usize) -> uint8 {
    ram[addr & 0xffff]
}

fn write_word(ram: &mut [uint8; 65536], addr: usize, value: uint16) {
    ram[addr & 0xffff] = value as uint8;
    ram[(addr + 1) & 0xffff] = (value >> 8) as uint8;
}

fn read_word(ram: &[uint8; 65536], addr: usize) -> uint16 {
    word(ram, addr)
}

fn copy_channel_variables_to_ram(ram: &mut [uint8; 65536], c: &Channel, i: usize) {
    macro_rules! b {
        ($field:ident, $addr:expr) => {
            write_byte(ram, $addr + i * 2, c.$field)
        };
    }
    macro_rules! w {
        ($field:ident, $addr:expr) => {
            write_word(ram, ($addr & 0x7fff) + i * 2, c.$field)
        };
    }

    w!(pattern_order_ptr_for_chan, 0x8030usize);
    b!(note_ticks_left, 0x70usize);
    b!(note_keyoff_ticks_left, 0x71usize);
    b!(subroutine_num_loops, 0x80usize);
    b!(volume_fade_ticks, 0x90usize);
    b!(pan_num_ticks, 0x91usize);
    b!(pitch_slide_length, 0xa0usize);
    b!(pitch_slide_delay_left, 0xa1usize);
    b!(vibrato_hold_count, 0xb0usize);
    b!(vib_depth, 0xb1usize);
    b!(tremolo_hold_count, 0xc0usize);
    b!(tremolo_depth, 0xc1usize);
    b!(vibrato_change_count, 0x100usize);
    b!(note_length, 0x200usize);
    b!(note_gate_off_fixedpt, 0x201usize);
    b!(channel_volume_master, 0x210usize);
    b!(instrument_id, 0x211usize);
    w!(instrument_pitch_base, 0x8220usize);
    w!(saved_pattern_ptr, 0x8230usize);
    w!(pattern_start_ptr, 0x8240usize);
    b!(pitch_envelope_num_ticks, 0x280usize);
    b!(pitch_envelope_delay, 0x281usize);
    b!(pitch_envelope_direction, 0x290usize);
    b!(pitch_envelope_slide_value, 0x291usize);
    b!(vibrato_count, 0x2a0usize);
    b!(vibrato_rate, 0x2a1usize);
    b!(vibrato_delay_ticks, 0x2b0usize);
    b!(vibrato_fade_num_ticks, 0x2b1usize);
    b!(vibrato_fade_add_per_tick, 0x2c0usize);
    b!(vibrato_depth_target, 0x2c1usize);
    b!(tremolo_count, 0x2d0usize);
    b!(tremolo_rate, 0x2d1usize);
    b!(tremolo_delay_ticks, 0x2e0usize);
    b!(channel_transposition, 0x2f0usize);
    w!(channel_volume, 0x8300usize);
    w!(volume_fade_addpertick, 0x8310usize);
    b!(volume_fade_target, 0x320usize);
    b!(final_volume, 0x321usize);
    w!(pan_value, 0x8330usize);
    w!(pan_add_per_tick, 0x8340usize);
    b!(pan_target_value, 0x350usize);
    b!(pan_flag_with_phase_invert, 0x351usize);
    w!(pitch, 0x8360usize);
    w!(pitch_add_per_tick, 0x8370usize);
    b!(pitch_target, 0x380usize);
    b!(fine_tune, 0x381usize);
    w!(sfx_sound_ptr, 0x8390usize);
    b!(sfx_which_sound, 0x3a0usize);
    b!(sfx_arr_countdown, 0x3a1usize);
    b!(sfx_note_length_left, 0x3b0usize);
    b!(sfx_note_length, 0x3b1usize);
    b!(sfx_pan, 0x3d0usize);
}

fn copy_channel_variables_from_ram(ram: &[uint8; 65536], c: &mut Channel, i: usize) {
    macro_rules! b {
        ($field:ident, $addr:expr) => {
            c.$field = read_byte(ram, $addr + i * 2)
        };
    }
    macro_rules! w {
        ($field:ident, $addr:expr) => {
            c.$field = read_word(ram, ($addr & 0x7fff) + i * 2)
        };
    }

    w!(pattern_order_ptr_for_chan, 0x8030usize);
    b!(note_ticks_left, 0x70usize);
    b!(note_keyoff_ticks_left, 0x71usize);
    b!(subroutine_num_loops, 0x80usize);
    b!(volume_fade_ticks, 0x90usize);
    b!(pan_num_ticks, 0x91usize);
    b!(pitch_slide_length, 0xa0usize);
    b!(pitch_slide_delay_left, 0xa1usize);
    b!(vibrato_hold_count, 0xb0usize);
    b!(vib_depth, 0xb1usize);
    b!(tremolo_hold_count, 0xc0usize);
    b!(tremolo_depth, 0xc1usize);
    b!(vibrato_change_count, 0x100usize);
    b!(note_length, 0x200usize);
    b!(note_gate_off_fixedpt, 0x201usize);
    b!(channel_volume_master, 0x210usize);
    b!(instrument_id, 0x211usize);
    w!(instrument_pitch_base, 0x8220usize);
    w!(saved_pattern_ptr, 0x8230usize);
    w!(pattern_start_ptr, 0x8240usize);
    b!(pitch_envelope_num_ticks, 0x280usize);
    b!(pitch_envelope_delay, 0x281usize);
    b!(pitch_envelope_direction, 0x290usize);
    b!(pitch_envelope_slide_value, 0x291usize);
    b!(vibrato_count, 0x2a0usize);
    b!(vibrato_rate, 0x2a1usize);
    b!(vibrato_delay_ticks, 0x2b0usize);
    b!(vibrato_fade_num_ticks, 0x2b1usize);
    b!(vibrato_fade_add_per_tick, 0x2c0usize);
    b!(vibrato_depth_target, 0x2c1usize);
    b!(tremolo_count, 0x2d0usize);
    b!(tremolo_rate, 0x2d1usize);
    b!(tremolo_delay_ticks, 0x2e0usize);
    b!(channel_transposition, 0x2f0usize);
    w!(channel_volume, 0x8300usize);
    w!(volume_fade_addpertick, 0x8310usize);
    b!(volume_fade_target, 0x320usize);
    b!(final_volume, 0x321usize);
    w!(pan_value, 0x8330usize);
    w!(pan_add_per_tick, 0x8340usize);
    b!(pan_target_value, 0x350usize);
    b!(pan_flag_with_phase_invert, 0x351usize);
    w!(pitch, 0x8360usize);
    w!(pitch_add_per_tick, 0x8370usize);
    b!(pitch_target, 0x380usize);
    b!(fine_tune, 0x381usize);
    w!(sfx_sound_ptr, 0x8390usize);
    b!(sfx_which_sound, 0x3a0usize);
    b!(sfx_arr_countdown, 0x3a1usize);
    b!(sfx_note_length_left, 0x3b0usize);
    b!(sfx_note_length, 0x3b1usize);
    b!(sfx_pan, 0x3d0usize);
}

fn copy_player_variables_to_ram(p: &mut SpcPlayer) {
    p.ram[0x0000..0x0004].copy_from_slice(&p.new_value_from_snes);
    p.ram[0x0004..0x0008].copy_from_slice(&p.port_to_snes);
    p.ram[0x0008..0x000c].copy_from_slice(&p.last_value_from_snes);
    write_byte(&mut p.ram, 0x000c, p.counter_sf0c);
    write_word(&mut p.ram, 0x000e, p._always_zero);
    write_word(&mut p.ram, 0x0010, p.temp_accum);
    write_byte(&mut p.ram, 0x0012, p.ttt);
    write_byte(&mut p.ram, 0x0013, p.did_affect_volumepitch_flag);
    write_word(&mut p.ram, 0x0014, p.addr0);
    write_word(&mut p.ram, 0x0016, p.addr1);
    write_word(&mut p.ram, 0x0018, p.lfsr_value);
    write_byte(&mut p.ram, 0x001a, p.is_chan_on);
    write_byte(&mut p.ram, 0x001b, p.fast_forward);
    write_byte(&mut p.ram, 0x0020, p.sfx_start_arg_pan);
    write_word(&mut p.ram, 0x002c, p.sfx_sound_ptr_cur);
    write_word(&mut p.ram, 0x0040, p.music_ptr_toplevel);
    write_byte(&mut p.ram, 0x0042, p.block_count);
    write_byte(&mut p.ram, 0x0043, p.sfx_timer_accum);
    write_byte(&mut p.ram, 0x0044, p.chn);
    write_byte(&mut p.ram, 0x0045, p.key_ON);
    write_byte(&mut p.ram, 0x0046, p.key_OFF);
    write_byte(&mut p.ram, 0x0047, p.cur_chan_bit);
    write_byte(&mut p.ram, 0x0048, p.reg_FLG);
    write_byte(&mut p.ram, 0x0049, p.reg_NON);
    write_byte(&mut p.ram, 0x004a, p.reg_EON);
    write_byte(&mut p.ram, 0x004b, p.reg_PMON);
    write_byte(&mut p.ram, 0x004c, p.echo_stored_time);
    write_byte(&mut p.ram, 0x004d, p.echo_parameter_EDL);
    write_byte(&mut p.ram, 0x004e, p.reg_EFB);
    write_byte(&mut p.ram, 0x0050, p.global_transposition);
    write_byte(&mut p.ram, 0x0051, p.main_tempo_accum);
    write_word(&mut p.ram, 0x0052, p.tempo);
    write_byte(&mut p.ram, 0x0054, p.tempo_fade_num_ticks);
    write_byte(&mut p.ram, 0x0055, p.tempo_fade_final);
    write_word(&mut p.ram, 0x0056, p.tempo_fade_add);
    write_word(&mut p.ram, 0x0058, p.master_volume);
    write_byte(&mut p.ram, 0x005a, p.master_volume_fade_ticks);
    write_byte(&mut p.ram, 0x005b, p.master_volume_fade_target);
    write_word(&mut p.ram, 0x005c, p.master_volume_fade_add_per_tick);
    write_byte(&mut p.ram, 0x005e, p.vol_dirty);
    write_byte(&mut p.ram, 0x005f, p.percussion_base_id);
    write_word(&mut p.ram, 0x0060, p.echo_volume_left);
    write_word(&mut p.ram, 0x0062, p.echo_volume_right);
    write_word(&mut p.ram, 0x0064, p.echo_volume_fade_add_left);
    write_word(&mut p.ram, 0x0066, p.echo_volume_fade_add_right);
    write_byte(&mut p.ram, 0x0068, p.echo_volume_fade_ticks);
    write_byte(&mut p.ram, 0x0069, p.echo_volume_fade_target_left);
    write_byte(&mut p.ram, 0x006a, p.echo_volume_fade_target_right);
    write_byte(&mut p.ram, 0x03c0, p.sfx_channel_index);
    write_byte(&mut p.ram, 0x03c1, p.current_bit);
    write_byte(&mut p.ram, 0x03c2, p.dsp_register_index);
    write_byte(&mut p.ram, 0x03c3, p.echo_channels);
    write_byte(&mut p.ram, 0x03c4, p.byte_3C4);
    write_byte(&mut p.ram, 0x03c5, p.byte_3C5);
    write_byte(&mut p.ram, 0x03c7, p.echo_fract_incr);
    write_byte(&mut p.ram, 0x03c8, p.sfx_channel_index2);
    write_byte(&mut p.ram, 0x03c9, p.sfx_channel_bit);
    write_byte(&mut p.ram, 0x03ca, p.pause_music_ctr);
    write_byte(&mut p.ram, 0x03cb, p.port2_active);
    write_byte(&mut p.ram, 0x03cc, p.port2_current_bit);
    write_byte(&mut p.ram, 0x03cd, p.port3_active);
    write_byte(&mut p.ram, 0x03ce, p.port3_current_bit);
    write_byte(&mut p.ram, 0x03cf, p.port1_active);
    write_byte(&mut p.ram, 0x03e0, p.port1_current_bit);
    write_byte(&mut p.ram, 0x03e1, p.byte_3E1);
    write_byte(&mut p.ram, 0x03e2, p.sfx_play_echo_flag);
    write_byte(&mut p.ram, 0x03e3, p.sfx_channels_echo_mask2);
    write_byte(&mut p.ram, 0x03e4, p.port1_counter);
    write_byte(&mut p.ram, 0x03e5, p.channel_67_volume);
    write_byte(&mut p.ram, 0x03ff, p.cutk_always_zero);
}

fn copy_player_variables_from_ram(p: &mut SpcPlayer) {
    p.new_value_from_snes
        .copy_from_slice(&p.ram[0x0000..0x0004]);
    p.port_to_snes.copy_from_slice(&p.ram[0x0004..0x0008]);
    p.last_value_from_snes
        .copy_from_slice(&p.ram[0x0008..0x000c]);
    p.counter_sf0c = read_byte(&p.ram, 0x000c);
    p._always_zero = read_word(&p.ram, 0x000e);
    p.temp_accum = read_word(&p.ram, 0x0010);
    p.ttt = read_byte(&p.ram, 0x0012);
    p.did_affect_volumepitch_flag = read_byte(&p.ram, 0x0013);
    p.addr0 = read_word(&p.ram, 0x0014);
    p.addr1 = read_word(&p.ram, 0x0016);
    p.lfsr_value = read_word(&p.ram, 0x0018);
    p.is_chan_on = read_byte(&p.ram, 0x001a);
    p.fast_forward = read_byte(&p.ram, 0x001b);
    p.sfx_start_arg_pan = read_byte(&p.ram, 0x0020);
    p.sfx_sound_ptr_cur = read_word(&p.ram, 0x002c);
    p.music_ptr_toplevel = read_word(&p.ram, 0x0040);
    p.block_count = read_byte(&p.ram, 0x0042);
    p.sfx_timer_accum = read_byte(&p.ram, 0x0043);
    p.chn = read_byte(&p.ram, 0x0044);
    p.key_ON = read_byte(&p.ram, 0x0045);
    p.key_OFF = read_byte(&p.ram, 0x0046);
    p.cur_chan_bit = read_byte(&p.ram, 0x0047);
    p.reg_FLG = read_byte(&p.ram, 0x0048);
    p.reg_NON = read_byte(&p.ram, 0x0049);
    p.reg_EON = read_byte(&p.ram, 0x004a);
    p.reg_PMON = read_byte(&p.ram, 0x004b);
    p.echo_stored_time = read_byte(&p.ram, 0x004c);
    p.echo_parameter_EDL = read_byte(&p.ram, 0x004d);
    p.reg_EFB = read_byte(&p.ram, 0x004e);
    p.global_transposition = read_byte(&p.ram, 0x0050);
    p.main_tempo_accum = read_byte(&p.ram, 0x0051);
    p.tempo = read_word(&p.ram, 0x0052);
    p.tempo_fade_num_ticks = read_byte(&p.ram, 0x0054);
    p.tempo_fade_final = read_byte(&p.ram, 0x0055);
    p.tempo_fade_add = read_word(&p.ram, 0x0056);
    p.master_volume = read_word(&p.ram, 0x0058);
    p.master_volume_fade_ticks = read_byte(&p.ram, 0x005a);
    p.master_volume_fade_target = read_byte(&p.ram, 0x005b);
    p.master_volume_fade_add_per_tick = read_word(&p.ram, 0x005c);
    p.vol_dirty = read_byte(&p.ram, 0x005e);
    p.percussion_base_id = read_byte(&p.ram, 0x005f);
    p.echo_volume_left = read_word(&p.ram, 0x0060);
    p.echo_volume_right = read_word(&p.ram, 0x0062);
    p.echo_volume_fade_add_left = read_word(&p.ram, 0x0064);
    p.echo_volume_fade_add_right = read_word(&p.ram, 0x0066);
    p.echo_volume_fade_ticks = read_byte(&p.ram, 0x0068);
    p.echo_volume_fade_target_left = read_byte(&p.ram, 0x0069);
    p.echo_volume_fade_target_right = read_byte(&p.ram, 0x006a);
    p.sfx_channel_index = read_byte(&p.ram, 0x03c0);
    p.current_bit = read_byte(&p.ram, 0x03c1);
    p.dsp_register_index = read_byte(&p.ram, 0x03c2);
    p.echo_channels = read_byte(&p.ram, 0x03c3);
    p.byte_3C4 = read_byte(&p.ram, 0x03c4);
    p.byte_3C5 = read_byte(&p.ram, 0x03c5);
    p.echo_fract_incr = read_byte(&p.ram, 0x03c7);
    p.sfx_channel_index2 = read_byte(&p.ram, 0x03c8);
    p.sfx_channel_bit = read_byte(&p.ram, 0x03c9);
    p.pause_music_ctr = read_byte(&p.ram, 0x03ca);
    p.port2_active = read_byte(&p.ram, 0x03cb);
    p.port2_current_bit = read_byte(&p.ram, 0x03cc);
    p.port3_active = read_byte(&p.ram, 0x03cd);
    p.port3_current_bit = read_byte(&p.ram, 0x03ce);
    p.port1_active = read_byte(&p.ram, 0x03cf);
    p.port1_current_bit = read_byte(&p.ram, 0x03e0);
    p.byte_3E1 = read_byte(&p.ram, 0x03e1);
    p.sfx_play_echo_flag = read_byte(&p.ram, 0x03e2);
    p.sfx_channels_echo_mask2 = read_byte(&p.ram, 0x03e3);
    p.port1_counter = read_byte(&p.ram, 0x03e4);
    p.channel_67_volume = read_byte(&p.ram, 0x03e5);
    p.cutk_always_zero = read_byte(&p.ram, 0x03ff);
}

fn dsp_ram_slice<'a>(dsp: &Dsp) -> Option<&'a mut [uint8]> {
    if dsp.ram.is_null() {
        None
    } else {
        Some(unsafe { std::slice::from_raw_parts_mut(dsp.ram, 65536) })
    }
}

fn dsp_write_impl(dsp: *mut Dsp, reg: uint8_t, value: uint8) {
    if let Some(dsp) = unsafe { dsp.as_mut() } {
        if let Some(ram) = dsp_ram_slice(dsp) {
            dsp.core.write(reg & 0x7f, value, ram);
        }
        dsp.sampleOffset = dsp.core.sample_offset as i32;
    }
}

fn dsp_reset_impl(dsp: *mut Dsp) {
    if let Some(dsp) = unsafe { dsp.as_mut() } {
        dsp.core.reset();
        dsp.sampleOffset = dsp.core.sample_offset as i32;
    }
}

fn dsp_cycle_impl(dsp: *mut Dsp) {
    if let Some(dsp) = unsafe { dsp.as_mut() } {
        if let Some(ram) = dsp_ram_slice(dsp) {
            dsp.core.cycle(ram);
        }
        dsp.sampleOffset = dsp.core.sample_offset as i32;
    }
}

fn dsp_init_impl(ram: *mut uint8) -> *mut Dsp {
    let mut dsp = Dsp::default();
    dsp.ram = ram;
    Box::into_raw(Box::new(dsp))
}

pub fn dsp_get_samples(
    dsp: *mut Dsp,
    sample_data: &mut [i16],
    samples_per_frame: usize,
    channels: usize,
) {
    if let Some(dsp) = unsafe { dsp.as_mut() } {
        dsp.core
            .get_samples(sample_data, samples_per_frame, channels);
        dsp.sampleOffset = dsp.core.sample_offset as i32;
    } else {
        let count = samples_per_frame.saturating_mul(channels);
        for sample in sample_data.iter_mut().take(count) {
            *sample = 0;
        }
    }
}

pub fn spc_player_save_ram(p: *const SpcPlayer) -> [uint8; 65536] {
    if let Some(p_ref) = unsafe { p.as_ref() } {
        p_ref.ram
    } else {
        [0; 65536]
    }
}

pub fn spc_player_load_ram(p: *mut SpcPlayer, ram: &[uint8]) {
    if let Some(p_ref) = unsafe { p.as_mut() } {
        let len = ram.len().min(p_ref.ram.len());
        p_ref.ram[..len].copy_from_slice(&ram[..len]);
        if len < p_ref.ram.len() {
            p_ref.ram[len..].fill(0);
        }
        if let Some(dsp) = unsafe { p_ref.dsp.as_mut() } {
            dsp.ram = p_ref.ram.as_mut_ptr();
        }
    }
}

pub fn spc_player_save_dsp_c_saveload(p: *const SpcPlayer) -> Vec<uint8> {
    unsafe { p.as_ref() }
        .and_then(|p_ref| unsafe { p_ref.dsp.as_ref() })
        .map(|dsp| dsp.core.save_c_saveload())
        .unwrap_or_else(|| vec![0; snes::apu::DSP_SAVELOAD_SIZE])
}

pub fn spc_player_load_dsp_c_saveload(p: *mut SpcPlayer, data: &[uint8]) -> Result<(), String> {
    let Some(p_ref) = (unsafe { p.as_mut() }) else {
        return Ok(());
    };
    let Some(dsp) = (unsafe { p_ref.dsp.as_mut() }) else {
        return Ok(());
    };
    dsp.core.load_c_saveload(data)?;
    dsp.ram = p_ref.ram.as_mut_ptr();
    dsp.sampleOffset = dsp.core.sample_offset as i32;
    Ok(())
}

fn not_implemented() -> ! {
    // Mirrors C spc_player.c Not_Implemented(), which is an assert(0) fallback
    // for unsupported SPC commands and unreachable music/SFX control paths.
    panic!("Not Implemented");
}

fn dsp_write(p: *mut SpcPlayer, reg: uint8_t, value: uint8) {
    let p = unsafe { &mut *p };
    if let Some(hist) = unsafe { p.reg_write_history.as_mut() } {
        if hist.count < 256 {
            hist.addr[hist.count] = reg;
            hist.val[hist.count] = value;
            hist.sample_offset[hist.count] = if p.dsp.is_null() {
                0
            } else {
                unsafe { (*p.dsp).sampleOffset }
            };
            hist.timer_cycles[hist.count] = p.timer_cycles;
            hist.count += 1;
        }
    }
    if !p.dsp.is_null() {
        dsp_write_impl(p.dsp, reg, value);
    }
}

fn spc_div_helper(a: i32, b: uint8) -> uint16 {
    let org_a = a;
    let a = if a & 0x100 != 0 { -a } else { a };
    let q = if b != 0 { (a & 0xff) / b as i32 } else { 0xff };
    let r = if b != 0 {
        (a & 0xff) % b as i32
    } else {
        a & 0xff
    };
    let t = (q << 8)
        + if b != 0 {
            ((r << 8) / b as i32) & 0xff
        } else {
            0xff
        };
    if org_a & 0x100 != 0 {
        0u16.wrapping_sub(t as u16)
    } else {
        t as uint16
    }
}

fn chan_do_any_fade(p: *mut uint16, add: uint16, target: uint8, cont: uint8) {
    unsafe {
        if cont == 0 {
            *p = (target as uint16) << 8;
        } else {
            *p = (*p).wrapping_add(add);
        }
    }
}

fn setup_echo_parameter_edl(p: *mut SpcPlayer, mut a: uint8) {
    let p_ref = unsafe { &mut *p };
    p_ref.echo_parameter_EDL = a;
    if a != p_ref.last_written_edl {
        a = (p_ref.last_written_edl & 0xf) ^ 0xff;
        if p_ref.echo_stored_time & 0x80 != 0 {
            a = a.wrapping_add(p_ref.echo_stored_time);
        }
        p_ref.echo_stored_time = a;
        dsp_write(p, EON, 0);
        dsp_write(p, EFB, 0);
        dsp_write(p, EVOLR, 0);
        dsp_write(p, EVOLL, 0);
        dsp_write(p, FLG, p_ref.reg_FLG | 0x20);
        p_ref.last_written_edl = p_ref.echo_parameter_EDL;
        dsp_write(p, EDL, p_ref.echo_parameter_EDL);
    }
    dsp_write(
        p,
        ESA,
        ((p_ref.echo_parameter_EDL.wrapping_mul(8)) ^ 0xff).wrapping_add(0xd1),
    );
}

fn write_volume_to_dsp(p: *mut SpcPlayer, c: *mut Channel, mut volume: uint16) {
    static SPC_VOLUME_CURVE: [uint8; 22] = [
        0, 1, 3, 7, 13, 21, 30, 41, 52, 66, 81, 94, 103, 110, 115, 119, 122, 124, 125, 126, 127,
        127,
    ];
    let p_ref = unsafe { &mut *p };
    let c_ref = unsafe { &mut *c };
    if p_ref.is_chan_on & p_ref.cur_chan_bit != 0 {
        return;
    }
    for i in 0..2u8 {
        let j = (volume >> 8) as usize;
        let mut t = if j >= 21 {
            let a = p_ref.ram[j + 0x1178];
            a.wrapping_add(
                (((p_ref.ram[j + 0x1179].wrapping_sub(a)) as uint16 * (volume as uint8) as uint16)
                    >> 8) as uint8,
            )
        } else {
            let a = SPC_VOLUME_CURVE[j];
            a.wrapping_add(
                (((SPC_VOLUME_CURVE[j + 1].wrapping_sub(a)) as uint16
                    * (volume as uint8) as uint16)
                    >> 8) as uint8,
            )
        };
        t = (((t as uint16) * c_ref.final_volume as uint16) >> 8) as uint8;
        if (c_ref.pan_flag_with_phase_invert << i) & 0x80 != 0 {
            t = 0u8.wrapping_sub(t);
        }
        dsp_write(p, V0VOLL + i + c_ref.index * 16, t);
        volume = 0x1400u16.wrapping_sub(volume);
    }
}

fn write_pitch(p: *mut SpcPlayer, c: *mut Channel, mut pitch: uint16) {
    static SPC_BASE_NOTE_FREQUENCIES: [uint16; 13] = [
        2143, 2270, 2405, 2548, 2700, 2860, 3030, 3211, 3402, 3604, 3818, 4045, 4286,
    ];
    if (pitch >> 8) >= 0x34 {
        pitch = pitch.wrapping_add((pitch >> 8) - 0x34);
    } else if (pitch >> 8) < 0x13 {
        let delta = ((pitch >> 8).wrapping_sub(0x13).wrapping_mul(2) as uint8) as uint16;
        pitch = pitch.wrapping_add(delta).wrapping_sub(256);
    }
    let pp = ((pitch >> 8) & 0x7f) as uint8;
    let mut q = pp / 12;
    let r = (pp % 12) as usize;
    let mut t = SPC_BASE_NOTE_FREQUENCIES[r].wrapping_add(
        (((SPC_BASE_NOTE_FREQUENCIES[r + 1] - SPC_BASE_NOTE_FREQUENCIES[r]) as uint8 as uint16)
            * (pitch as uint8 as uint16))
            >> 8,
    );
    t = t.wrapping_mul(2);
    while q != 6 {
        t >>= 1;
        q = q.wrapping_add(1);
    }
    let c_ref = unsafe { &mut *c };
    t = (((c_ref.instrument_pitch_base as u32) * (t as u32)) >> 8) as uint16;
    let p_ref = unsafe { &mut *p };
    if p_ref.cur_chan_bit & p_ref.is_chan_on == 0 {
        let reg = c_ref.index * 16;
        dsp_write(p, reg + V0PITCHL, t as uint8);
        dsp_write(p, reg + V0PITCHH, (t >> 8) as uint8);
    }
}

fn music_reset_chan(p: *mut SpcPlayer) {
    let p_ref = unsafe { &mut *p };
    p_ref.cur_chan_bit = 0x80;
    for i in (0..8).rev() {
        let c = &mut p_ref.channel[i];
        set_hi(&mut c.channel_volume, 0xff);
        c.pan_flag_with_phase_invert = 10;
        c.pan_value = 10 << 8;
        c.instrument_id = 0;
        c.fine_tune = 0;
        c.channel_transposition = 0;
        c.pitch_envelope_num_ticks = 0;
        c.vib_depth = 0;
        c.tremolo_depth = 0;
        p_ref.cur_chan_bit >>= 1;
    }
    p_ref.master_volume_fade_ticks = 0;
    p_ref.echo_volume_fade_ticks = 0;
    p_ref.tempo_fade_num_ticks = 0;
    p_ref.global_transposition = 0;
    p_ref.block_count = 0;
    p_ref.percussion_base_id = 0;
    set_hi(&mut p_ref.master_volume, 0xc0);
    set_hi(&mut p_ref.tempo, 0x20);
}

fn channel_set_instrument(p: *mut SpcPlayer, c: *mut Channel, mut instrument: uint8) {
    let p_ref = unsafe { &mut *p };
    let c_ref = unsafe { &mut *c };
    c_ref.instrument_id = instrument;
    if instrument & 0x80 != 0 {
        instrument = instrument
            .wrapping_add(54)
            .wrapping_add(p_ref.percussion_base_id);
    }
    let ip = instrument as usize * 6 + 0x3d00;
    if p_ref.is_chan_on & p_ref.cur_chan_bit != 0 {
        return;
    }
    let reg = c_ref.index * 16;
    if p_ref.ram[ip] & 0x80 != 0 {
        p_ref.reg_FLG = (p_ref.reg_FLG & 0x20) | (p_ref.ram[ip] & 0x1f);
        p_ref.reg_NON |= p_ref.cur_chan_bit;
        dsp_write(p, reg + V0SRCN, 0);
    } else {
        dsp_write(p, reg + V0SRCN, p_ref.ram[ip]);
    }
    dsp_write(p, reg + V0ADSR1, p_ref.ram[ip + 1]);
    dsp_write(p, reg + V0ADSR2, p_ref.ram[ip + 2]);
    dsp_write(p, reg + V0GAIN, p_ref.ram[ip + 3]);
    c_ref.instrument_pitch_base =
        ((p_ref.ram[ip + 4] as uint16) << 8) | p_ref.ram[ip + 5] as uint16;
}

fn compute_pitch_add(c: *mut Channel, pitch: uint8) {
    let c_ref = unsafe { &mut *c };
    c_ref.pitch_target = pitch & 0x7f;
    c_ref.pitch_add_per_tick = spc_div_helper(
        c_ref.pitch_target as i32 - (c_ref.pitch >> 8) as i32,
        c_ref.pitch_slide_length,
    );
}

fn pitch_slide_to_note_check(p: *mut SpcPlayer, c: *mut Channel) {
    let p_ref = unsafe { &mut *p };
    let c_ref = unsafe { &mut *c };
    if c_ref.pitch_slide_length != 0 || p_ref.ram[c_ref.pattern_order_ptr_for_chan as usize] != 0xf9
    {
        return;
    }
    if p_ref.cur_chan_bit & p_ref.is_chan_on != 0 {
        c_ref.pattern_order_ptr_for_chan = c_ref.pattern_order_ptr_for_chan.wrapping_add(4);
        return;
    }
    c_ref.pattern_order_ptr_for_chan = c_ref.pattern_order_ptr_for_chan.wrapping_add(1);
    c_ref.pitch_slide_delay_left = p_ref.ram[c_ref.pattern_order_ptr_for_chan as usize];
    c_ref.pattern_order_ptr_for_chan = c_ref.pattern_order_ptr_for_chan.wrapping_add(1);
    c_ref.pitch_slide_length = p_ref.ram[c_ref.pattern_order_ptr_for_chan as usize];
    c_ref.pattern_order_ptr_for_chan = c_ref.pattern_order_ptr_for_chan.wrapping_add(1);
    let pitch = p_ref.ram[c_ref.pattern_order_ptr_for_chan as usize]
        .wrapping_add(p_ref.global_transposition)
        .wrapping_add(c_ref.channel_transposition);
    c_ref.pattern_order_ptr_for_chan = c_ref.pattern_order_ptr_for_chan.wrapping_add(1);
    compute_pitch_add(c, pitch);
}

fn handle_effect(p: *mut SpcPlayer, c: *mut Channel, effect: uint8) {
    const SPC_EFFECT_ARGUMENT_BYTE_LENGTHS: [uint8; 27] = [
        1, 1, 2, 3, 0, 1, 2, 1, 2, 1, 1, 3, 0, 1, 2, 3, 1, 3, 3, 0, 1, 3, 0, 3, 3, 3, 1,
    ];
    let p_ref = unsafe { &mut *p };
    let c_ref = unsafe { &mut *c };
    let arg = if SPC_EFFECT_ARGUMENT_BYTE_LENGTHS[(effect - 0xe0) as usize] != 0 {
        let v = p_ref.ram[c_ref.pattern_order_ptr_for_chan as usize];
        c_ref.pattern_order_ptr_for_chan = c_ref.pattern_order_ptr_for_chan.wrapping_add(1);
        v
    } else {
        0
    };
    match effect {
        0xe0 => channel_set_instrument(p, c, arg),
        0xe1 => {
            c_ref.pan_flag_with_phase_invert = arg;
            c_ref.pan_value = ((arg & 0x1f) as uint16) << 8;
        }
        0xe2 => {
            c_ref.pan_num_ticks = arg;
            c_ref.pan_target_value = p_ref.ram[c_ref.pattern_order_ptr_for_chan as usize];
            c_ref.pattern_order_ptr_for_chan = c_ref.pattern_order_ptr_for_chan.wrapping_add(1);
            c_ref.pan_add_per_tick = spc_div_helper(
                c_ref.pan_target_value as i32 - (c_ref.pan_value >> 8) as i32,
                arg,
            );
        }
        0xe3 => {
            c_ref.vibrato_delay_ticks = arg;
            c_ref.vibrato_rate = p_ref.ram[c_ref.pattern_order_ptr_for_chan as usize];
            c_ref.pattern_order_ptr_for_chan = c_ref.pattern_order_ptr_for_chan.wrapping_add(1);
            c_ref.vibrato_depth_target = p_ref.ram[c_ref.pattern_order_ptr_for_chan as usize];
            c_ref.vib_depth = c_ref.vibrato_depth_target;
            c_ref.pattern_order_ptr_for_chan = c_ref.pattern_order_ptr_for_chan.wrapping_add(1);
            c_ref.vibrato_fade_num_ticks = 0;
        }
        0xe4 => {
            c_ref.vibrato_depth_target = 0;
            c_ref.vib_depth = 0;
            c_ref.vibrato_fade_num_ticks = 0;
        }
        0xe5 => {
            if p_ref.pause_music_ctr == 0 && p_ref.byte_3E1 == 0 {
                p_ref.master_volume = (arg as uint16) << 8;
            }
        }
        0xe6 => {
            p_ref.master_volume_fade_ticks = arg;
            p_ref.master_volume_fade_target = p_ref.ram[c_ref.pattern_order_ptr_for_chan as usize];
            c_ref.pattern_order_ptr_for_chan = c_ref.pattern_order_ptr_for_chan.wrapping_add(1);
            p_ref.master_volume_fade_add_per_tick = spc_div_helper(
                p_ref.master_volume_fade_target as i32 - (p_ref.master_volume >> 8) as i32,
                arg,
            );
        }
        0xe7 => p_ref.tempo = (arg as uint16) << 8,
        0xe8 => {
            p_ref.tempo_fade_num_ticks = arg;
            p_ref.tempo_fade_final = p_ref.ram[c_ref.pattern_order_ptr_for_chan as usize];
            c_ref.pattern_order_ptr_for_chan = c_ref.pattern_order_ptr_for_chan.wrapping_add(1);
            p_ref.tempo_fade_add = spc_div_helper(
                p_ref.tempo_fade_final as i32 - (p_ref.tempo >> 8) as i32,
                arg,
            );
        }
        0xe9 => p_ref.global_transposition = arg,
        0xea => c_ref.channel_transposition = arg,
        0xeb => {
            c_ref.tremolo_delay_ticks = arg;
            c_ref.tremolo_rate = p_ref.ram[c_ref.pattern_order_ptr_for_chan as usize];
            c_ref.pattern_order_ptr_for_chan = c_ref.pattern_order_ptr_for_chan.wrapping_add(1);
            c_ref.tremolo_depth = p_ref.ram[c_ref.pattern_order_ptr_for_chan as usize];
            c_ref.pattern_order_ptr_for_chan = c_ref.pattern_order_ptr_for_chan.wrapping_add(1);
        }
        0xec => c_ref.tremolo_depth = 0,
        0xed => c_ref.channel_volume = (arg as uint16) << 8,
        0xee => {
            c_ref.volume_fade_ticks = arg;
            c_ref.volume_fade_target = p_ref.ram[c_ref.pattern_order_ptr_for_chan as usize];
            c_ref.pattern_order_ptr_for_chan = c_ref.pattern_order_ptr_for_chan.wrapping_add(1);
            c_ref.volume_fade_addpertick = spc_div_helper(
                c_ref.volume_fade_target as i32 - (c_ref.channel_volume >> 8) as i32,
                arg,
            );
        }
        0xef => {
            c_ref.pattern_start_ptr =
                ((p_ref.ram[c_ref.pattern_order_ptr_for_chan as usize] as uint16) << 8)
                    | arg as uint16;
            c_ref.pattern_order_ptr_for_chan = c_ref.pattern_order_ptr_for_chan.wrapping_add(1);
            c_ref.subroutine_num_loops = p_ref.ram[c_ref.pattern_order_ptr_for_chan as usize];
            c_ref.pattern_order_ptr_for_chan = c_ref.pattern_order_ptr_for_chan.wrapping_add(1);
            c_ref.saved_pattern_ptr = c_ref.pattern_order_ptr_for_chan;
            c_ref.pattern_order_ptr_for_chan = c_ref.pattern_start_ptr;
        }
        0xf0 => {
            c_ref.vibrato_fade_num_ticks = arg;
            c_ref.vibrato_fade_add_per_tick = if arg != 0 {
                c_ref.vib_depth / arg
            } else {
                0xff
            };
        }
        0xf4 => c_ref.fine_tune = arg,
        0xf5 => {
            p_ref.reg_EON = arg;
            p_ref.echo_channels = arg;
            p_ref.echo_volume_left =
                (p_ref.ram[c_ref.pattern_order_ptr_for_chan as usize] as uint16) << 8;
            c_ref.pattern_order_ptr_for_chan = c_ref.pattern_order_ptr_for_chan.wrapping_add(1);
            p_ref.echo_volume_right =
                (p_ref.ram[c_ref.pattern_order_ptr_for_chan as usize] as uint16) << 8;
            c_ref.pattern_order_ptr_for_chan = c_ref.pattern_order_ptr_for_chan.wrapping_add(1);
            p_ref.reg_FLG &= !0x20;
        }
        0xf6 => {
            p_ref.echo_volume_left = 0;
            p_ref.echo_volume_right = 0;
            p_ref.reg_FLG |= 0x20;
        }
        0xf7 => {
            static SPC_ECHO_FIR_FILTER_PRESETS: [i8; 32] = [
                127, 0, 0, 0, 0, 0, 0, 0, 88, -65, -37, -16, -2, 7, 12, 12, 12, 33, 43, 43, 19, -2,
                -13, -7, 52, 51, 0, -39, -27, 1, -4, -21,
            ];
            setup_echo_parameter_edl(p, arg);
            p_ref.reg_EFB = p_ref.ram[c_ref.pattern_order_ptr_for_chan as usize];
            c_ref.pattern_order_ptr_for_chan = c_ref.pattern_order_ptr_for_chan.wrapping_add(1);
            let ep = p_ref.ram[c_ref.pattern_order_ptr_for_chan as usize] as usize * 8;
            c_ref.pattern_order_ptr_for_chan = c_ref.pattern_order_ptr_for_chan.wrapping_add(1);
            for i in 0..8 {
                dsp_write(
                    p,
                    FIR0 + i * 16,
                    SPC_ECHO_FIR_FILTER_PRESETS[ep + i as usize] as uint8,
                );
            }
        }
        0xf8 => {
            p_ref.echo_volume_fade_ticks = arg;
            p_ref.echo_volume_fade_target_left =
                p_ref.ram[c_ref.pattern_order_ptr_for_chan as usize];
            c_ref.pattern_order_ptr_for_chan = c_ref.pattern_order_ptr_for_chan.wrapping_add(1);
            p_ref.echo_volume_fade_target_right =
                p_ref.ram[c_ref.pattern_order_ptr_for_chan as usize];
            c_ref.pattern_order_ptr_for_chan = c_ref.pattern_order_ptr_for_chan.wrapping_add(1);
            p_ref.echo_volume_fade_add_left = spc_div_helper(
                p_ref.echo_volume_fade_target_left as i32 - (p_ref.echo_volume_left >> 8) as i32,
                arg,
            );
            p_ref.echo_volume_fade_add_right = spc_div_helper(
                p_ref.echo_volume_fade_target_right as i32 - (p_ref.echo_volume_right >> 8) as i32,
                arg,
            );
        }
        0xf9 => {
            c_ref.pitch_slide_delay_left = arg;
            c_ref.pitch_slide_length = p_ref.ram[c_ref.pattern_order_ptr_for_chan as usize];
            c_ref.pattern_order_ptr_for_chan = c_ref.pattern_order_ptr_for_chan.wrapping_add(1);
            let pitch = p_ref.ram[c_ref.pattern_order_ptr_for_chan as usize]
                .wrapping_add(p_ref.global_transposition)
                .wrapping_add(c_ref.channel_transposition);
            c_ref.pattern_order_ptr_for_chan = c_ref.pattern_order_ptr_for_chan.wrapping_add(1);
            compute_pitch_add(c, pitch);
        }
        0xfa => p_ref.percussion_base_id = arg,
        _ => not_implemented(),
    }
}

fn want_write_kof(p: *mut SpcPlayer, c: *mut Channel) -> bool {
    const SPC_EFFECT_ARGUMENT_BYTE_LENGTHS: [uint8; 27] = [
        1, 1, 2, 3, 0, 1, 2, 1, 2, 1, 1, 3, 0, 1, 2, 3, 1, 3, 3, 0, 1, 3, 0, 3, 3, 3, 1,
    ];
    let p_ref = unsafe { &mut *p };
    let c_ref = unsafe { &mut *c };
    let mut loops = c_ref.subroutine_num_loops;
    let mut ptr = c_ref.pattern_order_ptr_for_chan as usize;
    loop {
        let mut cmd = p_ref.ram[ptr];
        ptr += 1;
        if cmd == 0 {
            if loops == 0 {
                return true;
            }
            loops = loops.wrapping_sub(1);
            ptr = if loops == 0 {
                c_ref.saved_pattern_ptr
            } else {
                c_ref.pattern_start_ptr
            } as usize;
        } else {
            while cmd & 0x80 == 0 {
                cmd = p_ref.ram[ptr];
                ptr += 1;
            }
            if cmd == 0xc8 {
                return false;
            }
            if cmd == 0xef {
                ptr = word(&p_ref.ram, ptr) as usize;
            } else if cmd >= 0xe0 {
                let idx = (cmd - 0xe0) as usize;
                // C indexes kEffectByteLength directly for effect opcodes
                // e0..fa; this is the Rust guard for corrupt/out-of-range data.
                let Some(&len) = SPC_EFFECT_ARGUMENT_BYTE_LENGTHS.get(idx) else {
                    panic!(
                        "invalid SPC effect {:02x} in WantWriteKof ptr={:04x} loops={} chan={}",
                        cmd, ptr, loops, c_ref.index
                    );
                };
                ptr += len as usize;
            } else {
                return true;
            }
        }
    }
}

fn handle_tremolo(_p: *mut SpcPlayer, _c: *mut Channel) {
    not_implemented();
}

fn calc_vibrato_add_pitch(p: *mut SpcPlayer, c: *mut Channel, pitch: uint16, value: uint8) {
    let c_ref = unsafe { &mut *c };
    let mut t = (value as i32) << 2;
    if t & 0x100 != 0 {
        t ^= 0xff;
    }
    let r = if c_ref.vib_depth >= 0xf1 {
        ((t as uint8 as uint16) * (c_ref.vib_depth & 0xf) as uint16) as i32
    } else {
        (((t as uint8 as uint16) * c_ref.vib_depth as uint16) >> 8) as i32
    };
    let adjusted = if value & 0x80 != 0 {
        pitch.wrapping_sub(r as uint16)
    } else {
        pitch.wrapping_add(r as uint16)
    };
    write_pitch(p, c, adjusted);
}

fn handle_pan_and_sweep(p: *mut SpcPlayer, c: *mut Channel) {
    let p_ref = unsafe { &mut *p };
    let c_ref = unsafe { &mut *c };
    p_ref.did_affect_volumepitch_flag = 0;
    if c_ref.tremolo_depth != 0 {
        c_ref.tremolo_hold_count = c_ref.tremolo_delay_ticks;
        handle_tremolo(p, c);
    }
    let mut volume = c_ref.pan_value;
    if c_ref.pan_num_ticks != 0 {
        p_ref.did_affect_volumepitch_flag = 0x80;
        volume = volume.wrapping_add(
            (((p_ref.main_tempo_accum as i16 as i32) * (c_ref.pan_add_per_tick as i16 as i32))
                / 256) as uint16,
        );
    }
    if p_ref.did_affect_volumepitch_flag != 0 {
        write_volume_to_dsp(p, c, volume);
    }
    p_ref.did_affect_volumepitch_flag = 0;
    let mut pitch = c_ref.pitch;
    if c_ref.pitch_slide_length != 0 && c_ref.pitch_slide_delay_left == 0 {
        p_ref.did_affect_volumepitch_flag |= 0x80;
        pitch = pitch.wrapping_add(
            (((p_ref.main_tempo_accum as i16 as i32) * (c_ref.pitch_add_per_tick as i16 as i32))
                / 256) as uint16,
        );
    }
    if c_ref.vib_depth != 0 && c_ref.vibrato_delay_ticks == c_ref.vibrato_hold_count {
        calc_vibrato_add_pitch(
            p,
            c,
            pitch,
            ((p_ref.main_tempo_accum as uint16 * c_ref.vibrato_rate as uint16 >> 8) as uint8)
                .wrapping_add(c_ref.vibrato_count),
        );
        return;
    }
    if p_ref.did_affect_volumepitch_flag != 0 {
        write_pitch(p, c, pitch);
    }
}

fn handle_note_tick(p: *mut SpcPlayer, c: *mut Channel) {
    let p_ref = unsafe { &mut *p };
    let c_ref = unsafe { &mut *c };
    if c_ref.note_keyoff_ticks_left != 0 {
        c_ref.note_keyoff_ticks_left = c_ref.note_keyoff_ticks_left.wrapping_sub(1);
        if (c_ref.note_keyoff_ticks_left == 0 || c_ref.note_ticks_left == 2)
            && want_write_kof(p, c)
            && p_ref.cur_chan_bit & p_ref.is_chan_on == 0
        {
            dsp_write(p, KOF, p_ref.cur_chan_bit);
        }
    }
    p_ref.did_affect_volumepitch_flag = 0;
    if c_ref.pitch_slide_length != 0 {
        if c_ref.pitch_slide_delay_left != 0 {
            c_ref.pitch_slide_delay_left = c_ref.pitch_slide_delay_left.wrapping_sub(1);
        } else if p_ref.is_chan_on & p_ref.cur_chan_bit == 0 {
            p_ref.did_affect_volumepitch_flag = 0x80;
            c_ref.pitch_slide_length = c_ref.pitch_slide_length.wrapping_sub(1);
            chan_do_any_fade(
                &mut c_ref.pitch,
                c_ref.pitch_add_per_tick,
                c_ref.pitch_target,
                c_ref.pitch_slide_length,
            );
        }
    }
    let pitch = c_ref.pitch;
    if c_ref.vib_depth != 0 {
        if c_ref.vibrato_delay_ticks == c_ref.vibrato_hold_count {
            if c_ref.vibrato_change_count == c_ref.vibrato_fade_num_ticks {
                c_ref.vib_depth = c_ref.vibrato_depth_target;
            } else {
                c_ref.vib_depth = if c_ref.vibrato_change_count == 0 {
                    0
                } else {
                    c_ref.vib_depth
                }
                .wrapping_add(c_ref.vibrato_fade_add_per_tick);
                c_ref.vibrato_change_count = c_ref.vibrato_change_count.wrapping_add(1);
            }
            c_ref.vibrato_count = c_ref.vibrato_count.wrapping_add(c_ref.vibrato_rate);
            calc_vibrato_add_pitch(p, c, pitch, c_ref.vibrato_count);
            return;
        }
        c_ref.vibrato_hold_count = c_ref.vibrato_hold_count.wrapping_add(1);
    }
    if p_ref.did_affect_volumepitch_flag != 0 {
        write_pitch(p, c, pitch);
    }
}

pub fn calc_final_volume(p: *mut SpcPlayer, c: *mut Channel, vol: uint8) {
    let p_ref = unsafe { &mut *p };
    let c_ref = unsafe { &mut *c };
    let mut t = ((p_ref.master_volume >> 8) * vol as uint16) >> 8;
    t = (t * c_ref.channel_volume_master as uint16) >> 8;
    t = (t * (c_ref.channel_volume >> 8)) >> 8;
    c_ref.final_volume = ((t * t) >> 8) as uint8;
}

pub fn calc_tremolo(_p: *mut SpcPlayer, _c: *mut Channel) {
    not_implemented();
}

fn chan_handle_tick(p: *mut SpcPlayer, c: *mut Channel) {
    let p_ref = unsafe { &mut *p };
    let c_ref = unsafe { &mut *c };
    if c_ref.volume_fade_ticks != 0 {
        c_ref.volume_fade_ticks = c_ref.volume_fade_ticks.wrapping_sub(1);
        p_ref.vol_dirty |= p_ref.cur_chan_bit;
        chan_do_any_fade(
            &mut c_ref.channel_volume,
            c_ref.volume_fade_addpertick,
            c_ref.volume_fade_target,
            1,
        );
    }
    if c_ref.tremolo_depth != 0 {
        if c_ref.tremolo_delay_ticks == c_ref.tremolo_hold_count {
            p_ref.vol_dirty |= p_ref.cur_chan_bit;
            if c_ref.tremolo_count & 0x80 != 0 && c_ref.tremolo_depth == 0xff {
                c_ref.tremolo_count = 0x80;
            } else {
                c_ref.tremolo_count = c_ref.tremolo_count.wrapping_add(c_ref.tremolo_rate);
            }
            calc_tremolo(p, c);
        } else {
            c_ref.tremolo_hold_count = c_ref.tremolo_hold_count.wrapping_add(1);
            calc_final_volume(p, c, 0xff);
        }
    } else {
        calc_final_volume(p, c, 0xff);
    }
    if c_ref.pan_num_ticks != 0 {
        c_ref.pan_num_ticks = c_ref.pan_num_ticks.wrapping_sub(1);
        p_ref.vol_dirty |= p_ref.cur_chan_bit;
        chan_do_any_fade(
            &mut c_ref.pan_value,
            c_ref.pan_add_per_tick,
            c_ref.pan_target_value,
            1,
        );
    }
    if p_ref.vol_dirty & p_ref.cur_chan_bit != 0 {
        write_volume_to_dsp(p, c, c_ref.pan_value);
    }
}

fn port0_handle_music(p: *mut SpcPlayer) {
    const NOTE_VOL: [uint8; 16] = [
        25, 50, 76, 101, 114, 127, 140, 152, 165, 178, 191, 203, 216, 229, 242, 252,
    ];
    const NOTE_GATE_OFF_PCT: [uint8; 8] = [50, 101, 127, 152, 178, 203, 229, 252];

    let a = unsafe { (*p).new_value_from_snes[0] };

    if a == 0xff {
        not_implemented();
    }
    if a == 0xf1 {
        let p_ref = unsafe { &mut *p };
        p_ref.master_volume_fade_ticks = 0x80;
        p_ref.pause_music_ctr = 0x80;
        p_ref.master_volume_fade_target = 0;
        p_ref.master_volume_fade_add_per_tick =
            spc_div_helper(0 - (p_ref.master_volume >> 8) as i32, 0x80);
    } else if a == 0xf2 {
        let p_ref = unsafe { &mut *p };
        if p_ref.byte_3E1 != 0 {
            return;
        }
        p_ref.byte_3E1 = hi(p_ref.master_volume);
        set_hi(&mut p_ref.master_volume, 0x70);
    } else if a == 0xf3 {
        let p_ref = unsafe { &mut *p };
        if p_ref.byte_3E1 == 0 {
            return;
        }
        let volume = p_ref.byte_3E1;
        set_hi(&mut p_ref.master_volume, volume);
        p_ref.byte_3E1 = 0;
    } else if a == 0xf0 {
        let p_ref = unsafe { &mut *p };
        p_ref.key_OFF = p_ref.is_chan_on ^ 0xff;
        p_ref.port_to_snes[0] = 0;
        p_ref.cur_chan_bit = 0;
        return;
    } else if a != 0 {
        let p_ref = unsafe { &mut *p };
        p_ref.pause_music_ctr = 0;
        p_ref.byte_3E1 = 0;
        p_ref.port_to_snes[0] = a;
        p_ref.music_ptr_toplevel = word(&p_ref.ram, 0xd000 + (a as usize - 1) * 2);
        p_ref.counter_sf0c = 2;
        p_ref.key_OFF |= p_ref.is_chan_on ^ 0xff;
        return;
    }

    if unsafe { (*p).port_to_snes[0] } == 0 {
        return;
    }
    let mut load_next_phrase = false;
    {
        let p_ref = unsafe { &mut *p };
        if p_ref.pause_music_ctr != 0 {
            p_ref.pause_music_ctr = p_ref.pause_music_ctr.wrapping_sub(1);
            if p_ref.pause_music_ctr == 0 {
                p_ref.key_OFF = p_ref.is_chan_on ^ 0xff;
                p_ref.port_to_snes[0] = 0;
                p_ref.cur_chan_bit = 0;
                return;
            }
        }
        if p_ref.counter_sf0c != 0 {
            p_ref.counter_sf0c = p_ref.counter_sf0c.wrapping_sub(1);
            if p_ref.counter_sf0c != 0 {
                music_reset_chan(p);
                return;
            }
            load_next_phrase = true;
        }
    }

    'next_phrase: loop {
        if !load_next_phrase {
            // Continue the current phrase.
        } else {
            loop {
                let t = {
                    let p_ref = unsafe { &mut *p };
                    let t = word(&p_ref.ram, p_ref.music_ptr_toplevel as usize);
                    p_ref.music_ptr_toplevel = p_ref.music_ptr_toplevel.wrapping_add(2);
                    t
                };
                if hi(t) != 0 {
                    let mut ptr = t;
                    for i in 0..8 {
                        unsafe {
                            (*p).channel[i].pattern_order_ptr_for_chan =
                                word(&(*p).ram, ptr as usize);
                        }
                        ptr = ptr.wrapping_add(2);
                    }

                    let mut bit = 1u8;
                    for i in 0..8 {
                        unsafe {
                            (*p).cur_chan_bit = bit;
                            let c = &mut (*p).channel[i] as *mut Channel;
                            if hi((*c).pattern_order_ptr_for_chan) != 0 && (*c).instrument_id == 0 {
                                channel_set_instrument(p, c, 0);
                            }
                            (*c).subroutine_num_loops = 0;
                            (*c).volume_fade_ticks = 0;
                            (*c).pan_num_ticks = 0;
                            (*c).note_ticks_left = 1;
                        }
                        bit = bit.wrapping_shl(1);
                    }
                    unsafe {
                        (*p).counter_sf0c = 0;
                    }
                    break;
                }
                if t == 0 {
                    let p_ref = unsafe { &mut *p };
                    p_ref.key_OFF = p_ref.is_chan_on ^ 0xff;
                    p_ref.port_to_snes[0] = 0;
                    p_ref.cur_chan_bit = 0;
                    return;
                }
                if t == 0x80 {
                    unsafe { (*p).fast_forward = 0x80 };
                } else if t == 0x81 {
                    unsafe { (*p).fast_forward = 0 };
                } else {
                    let p_ref = unsafe { &mut *p };
                    p_ref.block_count = p_ref.block_count.wrapping_sub(1);
                    if (p_ref.block_count as i8) < 0 {
                        p_ref.block_count = t as uint8;
                    }
                    let next = word(&p_ref.ram, p_ref.music_ptr_toplevel as usize);
                    p_ref.music_ptr_toplevel = p_ref.music_ptr_toplevel.wrapping_add(2);
                    if p_ref.block_count != 0 {
                        p_ref.music_ptr_toplevel = next;
                    }
                }
            }
        }

        unsafe { (*p).vol_dirty = 0 };
        let mut bit = 1u8;
        for i in 0..8 {
            unsafe {
                (*p).cur_chan_bit = bit;
                let c = &mut (*p).channel[i] as *mut Channel;
                if hi((*c).pattern_order_ptr_for_chan) == 0 {
                    bit = bit.wrapping_shl(1);
                    continue;
                }
                (*c).note_ticks_left = (*c).note_ticks_left.wrapping_sub(1);
                if (*c).note_ticks_left == 0 {
                    loop {
                        let mut cmd = (*p).ram[(*c).pattern_order_ptr_for_chan as usize];
                        (*c).pattern_order_ptr_for_chan =
                            (*c).pattern_order_ptr_for_chan.wrapping_add(1);
                        if cmd == 0 {
                            if (*c).subroutine_num_loops == 0 {
                                load_next_phrase = true;
                                continue 'next_phrase;
                            }
                            (*c).subroutine_num_loops = (*c).subroutine_num_loops.wrapping_sub(1);
                            (*c).pattern_order_ptr_for_chan = if (*c).subroutine_num_loops == 0 {
                                (*c).saved_pattern_ptr
                            } else {
                                (*c).pattern_start_ptr
                            };
                            continue;
                        }
                        if cmd & 0x80 == 0 {
                            (*c).note_length = cmd;
                            cmd = (*p).ram[(*c).pattern_order_ptr_for_chan as usize];
                            (*c).pattern_order_ptr_for_chan =
                                (*c).pattern_order_ptr_for_chan.wrapping_add(1);
                            if cmd & 0x80 == 0 {
                                (*c).note_gate_off_fixedpt =
                                    NOTE_GATE_OFF_PCT[((cmd >> 4) & 7) as usize];
                                (*c).channel_volume_master = NOTE_VOL[(cmd & 0xf) as usize];
                                cmd = (*p).ram[(*c).pattern_order_ptr_for_chan as usize];
                                (*c).pattern_order_ptr_for_chan =
                                    (*c).pattern_order_ptr_for_chan.wrapping_add(1);
                            }
                        }
                        if cmd >= 0xe0 {
                            handle_effect(p, c, cmd);
                            continue;
                        }
                        if (*p).fast_forward == 0 && ((*p).is_chan_on & (*p).cur_chan_bit) == 0 {
                            play_note(p, c, cmd);
                        }
                        (*c).note_ticks_left = (*c).note_length;
                        let t = ((*c).note_ticks_left as uint16
                            * (*c).note_gate_off_fixedpt as uint16)
                            >> 8;
                        (*c).note_keyoff_ticks_left = if t != 0 { t as uint8 } else { 1 };
                        pitch_slide_to_note_check(p, c);
                        break;
                    }
                } else if (*p).fast_forward == 0 {
                    handle_note_tick(p, c);
                    pitch_slide_to_note_check(p, c);
                }
            }
            bit = bit.wrapping_shl(1);
        }
        unsafe {
            (*p).cur_chan_bit = 0;
        }

        {
            let p_ref = unsafe { &mut *p };
            if p_ref.tempo_fade_num_ticks != 0 {
                p_ref.tempo_fade_num_ticks = p_ref.tempo_fade_num_ticks.wrapping_sub(1);
                p_ref.tempo = if p_ref.tempo_fade_num_ticks == 0 {
                    (p_ref.tempo_fade_final as uint16) << 8
                } else {
                    p_ref.tempo.wrapping_add(p_ref.tempo_fade_add)
                };
            }
            if p_ref.echo_volume_fade_ticks != 0 {
                p_ref.echo_volume_left = p_ref
                    .echo_volume_left
                    .wrapping_add(p_ref.echo_volume_fade_add_left);
                p_ref.echo_volume_right = p_ref
                    .echo_volume_right
                    .wrapping_add(p_ref.echo_volume_fade_add_right);
                p_ref.echo_volume_fade_ticks = p_ref.echo_volume_fade_ticks.wrapping_sub(1);
                if p_ref.echo_volume_fade_ticks == 0 {
                    p_ref.echo_volume_left = (p_ref.echo_volume_fade_target_left as uint16) << 8;
                    p_ref.echo_volume_right = (p_ref.echo_volume_fade_target_right as uint16) << 8;
                }
            }
            if p_ref.master_volume_fade_ticks != 0 {
                p_ref.master_volume_fade_ticks = p_ref.master_volume_fade_ticks.wrapping_sub(1);
                p_ref.master_volume = if p_ref.master_volume_fade_ticks == 0 {
                    (p_ref.master_volume_fade_target as uint16) << 8
                } else {
                    p_ref
                        .master_volume
                        .wrapping_add(p_ref.master_volume_fade_add_per_tick)
                };
                p_ref.vol_dirty = 0xff;
            }
        }

        let mut bit = 1u8;
        for i in 0..8 {
            unsafe {
                (*p).cur_chan_bit = bit;
                let c = &mut (*p).channel[i] as *mut Channel;
                if hi((*c).pattern_order_ptr_for_chan) != 0 {
                    chan_handle_tick(p, c);
                }
            }
            bit = bit.wrapping_shl(1);
        }
        unsafe {
            (*p).cur_chan_bit = 0;
        }
        break;
    }
}

fn asl(p: *mut uint8) -> uint8 {
    unsafe {
        let old = *p;
        *p = (*p).wrapping_shl(1);
        old >> 7
    }
}

fn sfx_turn_off_channel(p: *mut SpcPlayer, c: *mut Channel) {
    let p_ref = unsafe { &mut *p };
    let c_ref = unsafe { &mut *c };
    c_ref.sfx_which_sound = 0;
    p_ref.is_chan_on &= !p_ref.current_bit;
    p_ref.port1_active &= !p_ref.current_bit;
    p_ref.port2_active &= !p_ref.current_bit;
    p_ref.port3_active &= !p_ref.current_bit;
    channel_set_instrument(p, c, c_ref.instrument_id);
    if p_ref.echo_channels & p_ref.current_bit != 0 && p_ref.reg_EON & p_ref.current_bit == 0 {
        p_ref.reg_EON |= p_ref.current_bit;
        dsp_write(p, EON, p_ref.reg_EON);
        p_ref.sfx_channels_echo_mask2 &= !p_ref.current_bit;
    }
}

fn write_key_on(p: *mut SpcPlayer, bit: uint8) {
    dsp_write(p, KOF, 0);
    dsp_write(p, KON, bit);
}

fn play_note(p: *mut SpcPlayer, c: *mut Channel, mut note: uint8) {
    let p_ref = unsafe { &mut *p };
    let c_ref = unsafe { &mut *c };
    if note >= 0xca {
        channel_set_instrument(p, c, note);
        note = 0xa4;
    }
    if note >= 0xc8 || p_ref.is_chan_on & p_ref.cur_chan_bit != 0 {
        return;
    }
    c_ref.pitch = ((note & 0x7f)
        .wrapping_add(p_ref.global_transposition)
        .wrapping_add(c_ref.channel_transposition) as uint16)
        << 8
        | c_ref.fine_tune as uint16;
    c_ref.vibrato_count = c_ref.vibrato_fade_num_ticks << 7;
    c_ref.vibrato_hold_count = 0;
    c_ref.vibrato_change_count = 0;
    c_ref.tremolo_count = 0;
    c_ref.tremolo_hold_count = 0;
    p_ref.vol_dirty |= p_ref.cur_chan_bit;
    p_ref.key_ON |= p_ref.cur_chan_bit;
    c_ref.pitch_slide_length = c_ref.pitch_envelope_num_ticks;
    if c_ref.pitch_slide_length != 0 {
        c_ref.pitch_slide_delay_left = c_ref.pitch_envelope_delay;
        if c_ref.pitch_envelope_direction == 0 {
            c_ref.pitch = c_ref
                .pitch
                .wrapping_sub((c_ref.pitch_envelope_slide_value as uint16) << 8);
        }
        compute_pitch_add(
            c,
            (c_ref.pitch >> 8).wrapping_add(c_ref.pitch_envelope_slide_value as uint16) as uint8,
        );
    }
    write_pitch(p, c, c_ref.pitch);
}

fn sfx_maybe_disable_echo(p: *mut SpcPlayer) {
    let p_ref = unsafe { &mut *p };
    if p_ref.port_to_snes[0] & 0x10 == 0 || p_ref.current_bit & p_ref.sfx_channels_echo_mask2 != 0 {
        if p_ref.current_bit & p_ref.reg_EON != 0 {
            p_ref.reg_EON ^= p_ref.current_bit;
            dsp_write(p, EON, p_ref.reg_EON);
        }
    }
}

fn sfx_channel_tick(p: *mut SpcPlayer, c: *mut Channel, is_continue: bool) {
    let p_ref = unsafe { &mut *p };
    let c_ref = unsafe { &mut *c };
    if is_continue {
        sfx_maybe_disable_echo(p);
        p_ref.sfx_channel_index = c_ref.index * 2;
        p_ref.sfx_sound_ptr_cur = c_ref.sfx_sound_ptr;
        c_ref.sfx_note_length_left = c_ref.sfx_note_length_left.wrapping_sub(1);
        if c_ref.sfx_note_length_left != 0 {
            sfx_note_continue(p, c);
            return;
        }
        p_ref.sfx_sound_ptr_cur = p_ref.sfx_sound_ptr_cur.wrapping_add(1);
    }

    loop {
        p_ref.dsp_register_index = p_ref.sfx_channel_index * 8;
        let mut cmd = p_ref.ram[p_ref.sfx_sound_ptr_cur as usize];
        if cmd == 0 {
            sfx_turn_off_channel(p, c);
            return;
        }

        if cmd & 0x80 == 0 {
            c_ref.sfx_note_length = cmd;
            p_ref.sfx_sound_ptr_cur = p_ref.sfx_sound_ptr_cur.wrapping_add(1);
            cmd = p_ref.ram[p_ref.sfx_sound_ptr_cur as usize];
            if cmd & 0x80 == 0 {
                if p_ref.port1_active & p_ref.current_bit != 0 {
                    if cmd == 0 || p_ref.channel_67_volume == 0 {
                        let volume = cmd;
                        dsp_write(p, p_ref.dsp_register_index + V0VOLL, cmd);
                        p_ref.sfx_sound_ptr_cur = p_ref.sfx_sound_ptr_cur.wrapping_add(1);
                        cmd = p_ref.ram[p_ref.sfx_sound_ptr_cur as usize];
                        if cmd & 0x80 != 0 {
                            dsp_write(p, p_ref.dsp_register_index + V0VOLR, volume);
                        } else {
                            dsp_write(p, p_ref.dsp_register_index + V0VOLR, cmd);
                            p_ref.sfx_sound_ptr_cur = p_ref.sfx_sound_ptr_cur.wrapping_add(1);
                            cmd = p_ref.ram[p_ref.sfx_sound_ptr_cur as usize];
                        }
                    } else {
                        p_ref.sfx_sound_ptr_cur = p_ref.sfx_sound_ptr_cur.wrapping_add(1);
                        cmd = p_ref.ram[p_ref.sfx_sound_ptr_cur as usize];
                    }
                } else {
                    c_ref.final_volume = cmd.wrapping_mul(2);
                    c_ref.pan_flag_with_phase_invert = 10;
                    let pan = if p_ref.sfx_start_arg_pan & 0x80 != 0 {
                        16
                    } else if p_ref.sfx_start_arg_pan & 0x40 != 0 {
                        4
                    } else {
                        10
                    };
                    write_volume_to_dsp(p, c, pan << 8);
                    p_ref.sfx_sound_ptr_cur = p_ref.sfx_sound_ptr_cur.wrapping_add(1);
                    cmd = p_ref.ram[p_ref.sfx_sound_ptr_cur as usize];
                }
            }
        }

        if cmd == 0xe0 {
            p_ref.sfx_sound_ptr_cur = p_ref.sfx_sound_ptr_cur.wrapping_add(1);
            let ip = 0x3e00 + p_ref.ram[p_ref.sfx_sound_ptr_cur as usize] as usize * 9;
            let reg = c_ref.index * 16;
            dsp_write(p, reg + V0VOLL, p_ref.ram[ip]);
            dsp_write(p, reg + V0VOLR, p_ref.ram[ip + 1]);
            dsp_write(p, reg + V0PITCHL, p_ref.ram[ip + 2]);
            dsp_write(p, reg + V0PITCHH, p_ref.ram[ip + 3]);
            dsp_write(p, reg + V0SRCN, p_ref.ram[ip + 4]);
            dsp_write(p, reg + V0ADSR1, p_ref.ram[ip + 5]);
            dsp_write(p, reg + V0ADSR2, p_ref.ram[ip + 6]);
            dsp_write(p, reg + V0GAIN, p_ref.ram[ip + 7]);
            c_ref.instrument_pitch_base = (p_ref.ram[ip + 8] as uint16) << 8;
            p_ref.sfx_sound_ptr_cur = p_ref.sfx_sound_ptr_cur.wrapping_add(1);
        } else if cmd == 0xf9 || cmd == 0xf1 {
            if cmd == 0xf9 {
                p_ref.sfx_sound_ptr_cur = p_ref.sfx_sound_ptr_cur.wrapping_add(1);
                play_note(p, c, p_ref.ram[p_ref.sfx_sound_ptr_cur as usize]);
                write_key_on(p, p_ref.current_bit);
            }
            p_ref.sfx_sound_ptr_cur = p_ref.sfx_sound_ptr_cur.wrapping_add(1);
            c_ref.pitch_slide_delay_left = p_ref.ram[p_ref.sfx_sound_ptr_cur as usize];
            p_ref.sfx_sound_ptr_cur = p_ref.sfx_sound_ptr_cur.wrapping_add(1);
            c_ref.pitch_slide_length = p_ref.ram[p_ref.sfx_sound_ptr_cur as usize];
            p_ref.sfx_sound_ptr_cur = p_ref.sfx_sound_ptr_cur.wrapping_add(1);
            compute_pitch_add(c, p_ref.ram[p_ref.sfx_sound_ptr_cur as usize]);
            c_ref.sfx_note_length_left = c_ref.sfx_note_length;
            sfx_note_continue(p, c);
            break;
        } else if cmd == 0xff {
            c_ref.sfx_sound_ptr = word(
                &p_ref.ram,
                0x17c0 + (c_ref.sfx_which_sound as usize - 1) * 2,
            );
            p_ref.sfx_sound_ptr_cur = c_ref.sfx_sound_ptr;
        } else {
            play_note(p, c, cmd);
            write_key_on(p, p_ref.current_bit);
            c_ref.sfx_note_length_left = c_ref.sfx_note_length;
            sfx_note_continue(p, c);
            break;
        }
    }
    c_ref.sfx_sound_ptr = p_ref.sfx_sound_ptr_cur;
}

fn sfx_note_continue(p: *mut SpcPlayer, c: *mut Channel) {
    let p_ref = unsafe { &mut *p };
    let c_ref = unsafe { &mut *c };
    p_ref.did_affect_volumepitch_flag = 0;
    if c_ref.pitch_slide_length != 0 {
        p_ref.did_affect_volumepitch_flag = 0x80;
        c_ref.pitch_slide_length = c_ref.pitch_slide_length.wrapping_sub(1);
        chan_do_any_fade(
            &mut c_ref.pitch,
            c_ref.pitch_add_per_tick,
            c_ref.pitch_target,
            c_ref.pitch_slide_length,
        );
        p_ref.cur_chan_bit = 0;
        write_pitch(p, c, c_ref.pitch);
    } else if c_ref.sfx_note_length_left == 2 {
        dsp_write(p, KOF, p_ref.current_bit);
    }
}

fn port1_play_inner(p: *mut SpcPlayer) {
    let p_ref = unsafe { &mut *p };
    p_ref.port1_counter = 0;
    p_ref.channel[7].sfx_which_sound = p_ref.new_value_from_snes[1];
    p_ref.channel[7].sfx_arr_countdown = 3;
    p_ref.channel[7].pitch_envelope_num_ticks = 0;
    p_ref.port1_active = 0x80;
    p_ref.is_chan_on |= 0x80;
    dsp_write(p, KOF, 0x80);
    p_ref.new_value_from_snes[1] = p_ref.ram[0x1800 + p_ref.new_value_from_snes[1] as usize - 1];
    if p_ref.new_value_from_snes[1] == 0 {
        return;
    }
    p_ref.channel[6].sfx_which_sound = p_ref.new_value_from_snes[1];
    p_ref.channel[6].sfx_arr_countdown = 3;
    p_ref.channel[6].pitch_envelope_num_ticks = 0;
    p_ref.port1_active = 0x40;
    p_ref.is_chan_on |= 0x40;
    dsp_write(p, KOF, 0x40);
    p_ref.port1_active = 0xc0;
    p_ref.sfx_channels_echo_mask2 |= 0xc0;
    p_ref.port2_active &= 0x3f;
    p_ref.port3_active &= 0x3f;
}

fn port1_start_new_sound(p: *mut SpcPlayer) {
    let p_ref = unsafe { &mut *p };
    if p_ref.port1_counter != 0 {
        p_ref.port1_counter = p_ref.port1_counter.wrapping_sub(1);
        if p_ref.port1_counter == 0 {
            p_ref.new_value_from_snes[1] = 5;
            port1_play_inner(p);
            p_ref.new_value_from_snes[1] = 0;
            return;
        }
        p_ref.channel_67_volume = p_ref.port1_counter >> 1;
        dsp_write(p, V0VOLL + 7 * 16, p_ref.channel_67_volume);
        dsp_write(p, V0VOLR + 7 * 16, p_ref.channel_67_volume);
        dsp_write(p, V0VOLL + 6 * 16, p_ref.channel_67_volume);
        dsp_write(p, V0VOLR + 6 * 16, p_ref.channel_67_volume);
    }
    p_ref.port1_current_bit = p_ref.port1_active;
    if p_ref.port1_current_bit == 0 {
        return;
    }
    p_ref.current_bit = 0x80;
    for i in (5..8).rev() {
        let c = &mut p_ref.channel[i] as *mut Channel;
        if asl(&mut p_ref.port1_current_bit) != 0 {
            p_ref.sfx_channel_index = unsafe { (*c).index } * 2;
            p_ref.dsp_register_index = unsafe { (*c).index } * 16;
            p_ref.sfx_start_arg_pan = unsafe { (*c).sfx_pan };
            if unsafe { (*c).sfx_arr_countdown } == 0 {
                if unsafe { (*c).sfx_which_sound } != 0 {
                    sfx_channel_tick(p, c, true);
                }
            } else {
                p_ref.sfx_channel_index = unsafe { (*c).index } * 2;
                unsafe {
                    (*c).sfx_arr_countdown = (*c).sfx_arr_countdown.wrapping_sub(1);
                    if (*c).sfx_arr_countdown == 0 {
                        (*c).sfx_sound_ptr =
                            word(&p_ref.ram, 0x17c0 + ((*c).sfx_which_sound as usize - 1) * 2);
                        p_ref.sfx_sound_ptr_cur = (*c).sfx_sound_ptr;
                        sfx_channel_tick(p, c, false);
                    }
                }
            }
        }
        p_ref.current_bit >>= 1;
    }
}

fn port1_handle_cmd(p: *mut SpcPlayer) {
    let p_ref = unsafe { &mut *p };
    let a = p_ref.new_value_from_snes[1];
    if a & 0x80 == 0 {
        if a != 0 {
            p_ref.port_to_snes[1] = a;
            if a != 5 || p_ref.port1_active != 0 {
                port1_play_inner(p);
            }
        }
    } else {
        p_ref.port_to_snes[1] = a;
        if p_ref.port1_active != 0 {
            p_ref.port1_counter = 0x78;
        }
    }
}

fn port2_start_new_sound(p: *mut SpcPlayer) {
    let p_ref = unsafe { &mut *p };
    p_ref.port2_current_bit = p_ref.port2_active;
    if p_ref.port2_current_bit == 0 {
        return;
    }
    p_ref.current_bit = 0x80;
    for i in (0..8).rev() {
        let c = &mut p_ref.channel[i] as *mut Channel;
        if asl(&mut p_ref.port2_current_bit) != 0 {
            p_ref.sfx_channel_index = unsafe { (*c).index } * 2;
            p_ref.dsp_register_index = unsafe { (*c).index } * 16;
            p_ref.sfx_start_arg_pan = unsafe { (*c).sfx_pan };
            if unsafe { (*c).sfx_arr_countdown } == 0 {
                if unsafe { (*c).sfx_which_sound } != 0 {
                    sfx_channel_tick(p, c, true);
                }
            } else {
                p_ref.sfx_channel_index = unsafe { (*c).index } * 2;
                unsafe {
                    (*c).sfx_arr_countdown = (*c).sfx_arr_countdown.wrapping_sub(1);
                    if (*c).sfx_arr_countdown == 0 {
                        (*c).sfx_sound_ptr =
                            word(&p_ref.ram, 0x1820 + ((*c).sfx_which_sound as usize - 1) * 2);
                        p_ref.sfx_sound_ptr_cur = (*c).sfx_sound_ptr;
                        sfx_channel_tick(p, c, false);
                    }
                }
            }
        }
        p_ref.current_bit >>= 1;
    }
}

fn sfx_allocate_chan(
    p: *mut SpcPlayer,
    active_mask: uint8,
    value_from_snes: uint8,
) -> *mut Channel {
    let p_ref = unsafe { &mut *p };
    p_ref.current_bit = 0x80;
    for i in (0..8).rev() {
        let c = &mut p_ref.channel[i] as *mut Channel;
        let c_ref = unsafe { &mut *c };
        if active_mask & p_ref.current_bit != 0
            && (c_ref.sfx_which_sound as uint16 + c_ref.sfx_pan as uint16)
                == value_from_snes as uint16
        {
            p_ref.sfx_channel_index2 = c_ref.index * 2;
            p_ref.sfx_channel_index = p_ref.sfx_channel_index2;
            p_ref.sfx_channel_bit = p_ref.current_bit;
            p_ref.is_chan_on |= p_ref.current_bit;
            if p_ref.sfx_play_echo_flag != 0 {
                p_ref.sfx_channels_echo_mask2 |= p_ref.current_bit;
            }
            sfx_maybe_disable_echo(p);
            return c;
        }
        p_ref.current_bit >>= 1;
    }

    p_ref.current_bit = 0x80;
    for i in (0..8).rev() {
        if p_ref.is_chan_on & p_ref.current_bit == 0 {
            let c = &mut p_ref.channel[i] as *mut Channel;
            p_ref.sfx_channel_index2 = p_ref.channel[i].index * 2;
            p_ref.sfx_channel_index = p_ref.sfx_channel_index2;
            p_ref.sfx_channel_bit = p_ref.current_bit;
            p_ref.is_chan_on |= p_ref.current_bit;
            if p_ref.sfx_play_echo_flag != 0 {
                p_ref.sfx_channels_echo_mask2 |= p_ref.current_bit;
            }
            sfx_maybe_disable_echo(p);
            return c;
        }
        p_ref.current_bit >>= 1;
    }
    // C Port3_AllocateChan marks this path assert(0) unreachable.
    panic!("unreachable");
}

fn port2_allocate_chan(p: *mut SpcPlayer) -> *mut Channel {
    let p_ref = unsafe { &mut *p };
    let value = p_ref.new_value_from_snes[2];
    p_ref.sfx_play_echo_flag = p_ref.ram[0x18dd + ((value & 0x3f) as usize) - 1];
    sfx_allocate_chan(p, p_ref.port2_active, value)
}

fn port2_handle_cmd(p: *mut SpcPlayer) {
    let p_ref = unsafe { &mut *p };
    while p_ref.new_value_from_snes[2] != 0 && p_ref.is_chan_on != 0xff {
        let c = port2_allocate_chan(p);
        let c_ref = unsafe { &mut *c };
        c_ref.sfx_pan = p_ref.new_value_from_snes[2] & 0xc0;
        c_ref.sfx_which_sound = p_ref.new_value_from_snes[2] & 0x3f;
        c_ref.sfx_arr_countdown = 3;
        c_ref.pitch_envelope_num_ticks = 0;
        p_ref.port2_active |= p_ref.current_bit;
        dsp_write(p, KOF, p_ref.current_bit);
        p_ref.new_value_from_snes[2] = p_ref.ram[0x189e + c_ref.sfx_which_sound as usize - 1];
    }
}

fn port3_start_new_sound(p: *mut SpcPlayer) {
    let p_ref = unsafe { &mut *p };
    p_ref.port3_current_bit = p_ref.port3_active;
    if p_ref.port3_current_bit == 0 {
        return;
    }
    p_ref.current_bit = 0x80;
    for i in (0..8).rev() {
        let c = &mut p_ref.channel[i] as *mut Channel;
        if asl(&mut p_ref.port3_current_bit) != 0 {
            p_ref.sfx_channel_index = unsafe { (*c).index } * 2;
            p_ref.dsp_register_index = unsafe { (*c).index } * 16;
            p_ref.sfx_start_arg_pan = unsafe { (*c).sfx_pan };
            if unsafe { (*c).sfx_arr_countdown } == 0 {
                if unsafe { (*c).sfx_which_sound } != 0 {
                    sfx_channel_tick(p, c, true);
                }
            } else {
                p_ref.sfx_channel_index = unsafe { (*c).index } * 2;
                unsafe {
                    (*c).sfx_arr_countdown = (*c).sfx_arr_countdown.wrapping_sub(1);
                    if (*c).sfx_arr_countdown == 0 {
                        (*c).sfx_sound_ptr =
                            word(&p_ref.ram, 0x191c + ((*c).sfx_which_sound as usize - 1) * 2);
                        p_ref.sfx_sound_ptr_cur = (*c).sfx_sound_ptr;
                        sfx_channel_tick(p, c, false);
                    }
                }
            }
        }
        p_ref.current_bit >>= 1;
    }
}

fn port3_allocate_chan(p: *mut SpcPlayer) -> *mut Channel {
    let p_ref = unsafe { &mut *p };
    let value = p_ref.new_value_from_snes[3];
    p_ref.sfx_play_echo_flag = p_ref.ram[0x19d8 + (value & 0x3f) as usize];
    sfx_allocate_chan(p, p_ref.port3_active, value)
}

fn port3_handle_cmd(p: *mut SpcPlayer) {
    let p_ref = unsafe { &mut *p };
    while p_ref.new_value_from_snes[3] != 0 && p_ref.is_chan_on != 0xff {
        let c = port3_allocate_chan(p);
        let c_ref = unsafe { &mut *c };
        c_ref.sfx_pan = p_ref.new_value_from_snes[3] & 0xc0;
        c_ref.sfx_which_sound = p_ref.new_value_from_snes[3] & 0x3f;
        c_ref.sfx_arr_countdown = 3;
        c_ref.pitch_envelope_num_ticks = 0;
        p_ref.port3_active |= p_ref.current_bit;
        dsp_write(p, KOF, p_ref.current_bit);
        p_ref.new_value_from_snes[3] = p_ref.ram[0x199a + c_ref.sfx_which_sound as usize - 1];
    }
}

fn read_port_from_snes(p: *mut SpcPlayer, port: i32) {
    let p_ref = unsafe { &mut *p };
    let port = port as usize;
    let old = p_ref.last_value_from_snes[port];
    p_ref.last_value_from_snes[port] = p_ref.input_ports[port];
    p_ref.new_value_from_snes[port] = if p_ref.input_ports[port] != old {
        p_ref.input_ports[port]
    } else {
        0
    };
}

fn spc_loop_part1(p: *mut SpcPlayer) {
    let p_ref = unsafe { &mut *p };
    dsp_write(p, KOF, p_ref.key_OFF);
    dsp_write(p, PMON, p_ref.reg_PMON);
    dsp_write(p, NON, p_ref.reg_NON);
    dsp_write(p, KOF, 0);
    dsp_write(p, KON, p_ref.key_ON);
    if p_ref.echo_stored_time & 0x80 == 0 {
        dsp_write(p, FLG, p_ref.reg_FLG);
        if p_ref.echo_stored_time == p_ref.echo_parameter_EDL {
            dsp_write(p, EON, p_ref.reg_EON);
            dsp_write(p, EFB, p_ref.reg_EFB);
            dsp_write(p, EVOLR, (p_ref.echo_volume_right >> 8) as uint8);
            dsp_write(p, EVOLL, (p_ref.echo_volume_left >> 8) as uint8);
        }
    }
    p_ref.key_OFF = 0;
    p_ref.key_ON = 0;
}

fn spc_loop_part2(p: *mut SpcPlayer, ticks: uint8) {
    let p_ref = unsafe { &mut *p };
    let t = p_ref.sfx_timer_accum as i32 + ticks.wrapping_mul(0x38) as i32;
    p_ref.sfx_timer_accum = t as uint8;
    if t >= 256 {
        port1_start_new_sound(p);
        port1_handle_cmd(p);
        read_port_from_snes(p, 1);
        port2_start_new_sound(p);
        port2_handle_cmd(p);
        read_port_from_snes(p, 2);
        port3_start_new_sound(p);
        port3_handle_cmd(p);
        read_port_from_snes(p, 3);
        if p_ref.echo_stored_time != p_ref.echo_parameter_EDL {
            p_ref.echo_fract_incr = p_ref.echo_fract_incr.wrapping_add(1);
            if p_ref.echo_fract_incr & 1 == 0 {
                p_ref.echo_stored_time = p_ref.echo_stored_time.wrapping_add(1);
            }
        }
    }
    let t = p_ref.main_tempo_accum as i32 + ticks.wrapping_mul(hi(p_ref.tempo)) as i32;
    p_ref.main_tempo_accum = t as uint8;
    if t >= 256 {
        port0_handle_music(p);
        read_port_from_snes(p, 0);
    } else if p_ref.port_to_snes[0] != 0 {
        p_ref.cur_chan_bit = 1;
        for i in 0..8 {
            let c = &mut p_ref.channel[i] as *mut Channel;
            if hi(unsafe { (*c).pattern_order_ptr_for_chan }) != 0 {
                handle_pan_and_sweep(p, c);
            }
            p_ref.cur_chan_bit = p_ref.cur_chan_bit.wrapping_shl(1);
        }
    }
}

fn interrupt_reset(p: *mut SpcPlayer) {
    let p_ref = unsafe { &mut *p };
    dsp_reset_impl(p_ref.dsp);
    let dsp = p_ref.dsp;
    let hist = p_ref.reg_write_history;
    *p_ref = SpcPlayer::default();
    p_ref.dsp = dsp;
    p_ref.reg_write_history = hist;
    for i in 0..8 {
        p_ref.channel[i].index = i as uint8;
    }
    setup_echo_parameter_edl(p, 1);
    p_ref.reg_FLG |= 0x20;
    dsp_write(p, MVOLL, 0x60);
    dsp_write(p, MVOLR, 0x60);
    dsp_write(p, DIR, 0x3c);
    set_hi(&mut p_ref.tempo, 16);
    p_ref.timer_cycles = 0;
}

pub fn spc_player_create() -> *mut SpcPlayer {
    let mut p = Box::new(SpcPlayer::default());
    p.dsp = dsp_init_impl(p.ram.as_mut_ptr());
    p.reg_write_history = ptr::null_mut();
    Box::into_raw(p)
}

pub fn spc_player_clone(p: *const SpcPlayer) -> *mut SpcPlayer {
    if p.is_null() {
        return spc_player_create();
    }
    let src = unsafe { &*p };
    let mut player = Box::new(src.clone());
    player.reg_write_history = ptr::null_mut();
    if !src.dsp.is_null() {
        let mut dsp = unsafe { (*src.dsp).clone() };
        dsp.ram = player.ram.as_mut_ptr();
        player.dsp = Box::into_raw(Box::new(dsp));
    } else {
        player.dsp = ptr::null_mut();
    }
    Box::into_raw(player)
}

pub fn spc_player_destroy(p: *mut SpcPlayer) {
    if let Some(player) = unsafe { p.as_mut() } {
        if !player.dsp.is_null() {
            unsafe {
                drop(Box::from_raw(player.dsp));
            }
            player.dsp = ptr::null_mut();
        }
    }
    if !p.is_null() {
        unsafe {
            drop(Box::from_raw(p));
        }
    }
}

pub fn spc_player_initialize(p: *mut SpcPlayer) {
    interrupt_reset(p);
    spc_loop_part1(p);
}

pub fn spc_player_copy_variables_to_ram(p: *mut SpcPlayer) {
    let Some(p_ref) = (unsafe { p.as_mut() }) else {
        return;
    };
    for i in 0..8 {
        copy_channel_variables_to_ram(&mut p_ref.ram, &p_ref.channel[i], i);
    }
    copy_player_variables_to_ram(p_ref);
}

pub fn spc_player_copy_variables_from_ram(p: *mut SpcPlayer) {
    let Some(p_ref) = (unsafe { p.as_mut() }) else {
        return;
    };
    for i in 0..8 {
        copy_channel_variables_from_ram(&p_ref.ram, &mut p_ref.channel[i], i);
    }
    copy_player_variables_from_ram(p_ref);
}

pub fn spc_player_generate_samples(p: *mut SpcPlayer) {
    let p_ref = unsafe { &mut *p };
    assert!(p_ref.timer_cycles <= 64);
    assert!(!p_ref.dsp.is_null());
    assert!(unsafe { (*p_ref.dsp).sampleOffset } <= 534);
    loop {
        if p_ref.timer_cycles >= 64 {
            spc_loop_part2(p, p_ref.timer_cycles >> 6);
            spc_loop_part1(p);
            p_ref.timer_cycles &= 63;
        }
        let sample_offset = unsafe { (*p_ref.dsp).sampleOffset };
        let mut n = 534 - sample_offset;
        if n > (64 - p_ref.timer_cycles) as i32 {
            n = (64 - p_ref.timer_cycles) as i32;
        }
        p_ref.timer_cycles = p_ref.timer_cycles.wrapping_add(n as uint8);
        for _ in 0..n {
            dsp_cycle_impl(p_ref.dsp);
        }
        if unsafe { (*p_ref.dsp).sampleOffset } == 534 {
            break;
        }
    }
}

pub fn spc_player_debug_summary(p: *const SpcPlayer) -> String {
    let Some(p_ref) = (unsafe { p.as_ref() }) else {
        return "spc=null".to_owned();
    };
    let mut active_patterns = 0usize;
    let mut note_ticks = [0u8; 4];
    let mut pattern_ptrs = [0u16; 4];
    let mut sfx_info = [(0u8, 0u8, 0u8, 0u16, 0u16, 0u8, 0u8); 2];
    for i in 0..8 {
        if hi(p_ref.channel[i].pattern_order_ptr_for_chan) != 0 {
            active_patterns += 1;
        }
        if i < 4 {
            note_ticks[i] = p_ref.channel[i].note_ticks_left;
            pattern_ptrs[i] = p_ref.channel[i].pattern_order_ptr_for_chan;
        } else if i >= 6 {
            let c = &p_ref.channel[i];
            sfx_info[i - 6] = (
                c.sfx_which_sound,
                c.sfx_note_length_left,
                c.instrument_id,
                c.sfx_sound_ptr,
                c.pitch,
                c.sfx_pan,
                c.pitch_slide_length,
            );
        }
    }
    let (sample_offset, sample_nonzero, kon, flg) = if let Some(dsp) = unsafe { p_ref.dsp.as_ref() }
    {
        (
            dsp.sampleOffset,
            dsp.core.sample_buffer.iter().any(|&sample| sample != 0),
            dsp.core.ram[KON as usize],
            dsp.core.ram[FLG as usize],
        )
    } else {
        (0, false, 0, 0)
    };
    format!(
        "spc(port={:02x?} new={:02x?} last={:02x?} ptr={:04x} ctr={} active={} key_on={:02x} key_off={:02x} is_on={:02x} vol={:04x} tempo={:04x} patterns={:04x?} ticks={:02x?} sfx67={:02x?} dsp_off={} dsp_nonzero={} dsp_kon={:02x} dsp_flg={:02x})",
        p_ref.port_to_snes,
        p_ref.new_value_from_snes,
        p_ref.last_value_from_snes,
        p_ref.music_ptr_toplevel,
        p_ref.counter_sf0c,
        active_patterns,
        p_ref.key_ON,
        p_ref.key_OFF,
        p_ref.is_chan_on,
        p_ref.master_volume,
        p_ref.tempo,
        pattern_ptrs,
        note_ticks,
        sfx_info,
        sample_offset,
        sample_nonzero,
        kon,
        flg,
    )
}

pub fn spc_player_upload(p: *mut SpcPlayer, mut data: *const uint8_t) {
    let p_ref = unsafe { &mut *p };
    dsp_write(p, EVOLL, 0);
    dsp_write(p, EVOLR, 0);
    dsp_write(p, KOF, 0xff);
    loop {
        let numbytes = unsafe { ptr::read_unaligned(data as *const uint16) };
        if numbytes == 0 {
            break;
        }
        let mut target = unsafe { ptr::read_unaligned(data.add(2) as *const uint16) };
        data = unsafe { data.add(4) };
        for _ in 0..numbytes {
            p_ref.ram[target as usize] = unsafe { *data };
            data = unsafe { data.add(1) };
            target = target.wrapping_add(1);
        }
    }
    p_ref.pause_music_ctr = 0;
    p_ref.port_to_snes[0] = 0;
    p_ref.port1_active = 0;
    p_ref.port2_active = 0;
    p_ref.port3_active = 0;
    p_ref.is_chan_on = 0;
    p_ref.input_ports = [0; 4];
}

pub fn compare_spc_impls(p: *mut SpcPlayer, p_org: *mut SpcPlayer, apu: *mut Apu) -> bool {
    if p.is_null() || p_org.is_null() || apu.is_null() {
        return false;
    }

    spc_player_copy_variables_to_ram(p);
    let p_ref = unsafe { &mut *p };
    let p_org_ref = unsafe { &*p_org };
    let apu_ref = unsafe { &mut *apu };

    p_ref.ram[0x18..0x1a].copy_from_slice(&apu_ref.ram[0x18..0x1a]);
    p_ref.ram[0x110..0x200].copy_from_slice(&apu_ref.ram[0x110..0x200]);
    p_ref.ram[0xf1..0x100].copy_from_slice(&apu_ref.ram[0xf1..0x100]);
    p_ref.ram[0x10..0x18].copy_from_slice(&apu_ref.ram[0x10..0x18]);
    p_ref.ram[0x44] = apu_ref.ram[0x44];

    let mut errors = 0usize;
    for i in 0..0xc000usize {
        if p_ref.ram[i] != apu_ref.ram[i] {
            if errors < 16 {
                if errors == 0 {
                    eprintln!("SPC RAM divergence:");
                }
                eprintln!(
                    "{i:04X}: {:02X} != {:02X} (mine, theirs) orig {:02X}",
                    p_ref.ram[i], apu_ref.ram[i], p_org_ref.ram[i]
                );
            }
            errors += 1;
        }
    }

    let mut hist = unsafe { p_ref.reg_write_history.as_mut() };
    let hist_count = hist.as_ref().map(|hist| hist.count).unwrap_or(0);
    let n = hist_count.max(apu_ref.hist.count);
    for i in 0..n {
        let mine = hist
            .as_ref()
            .and_then(|hist| (i < hist.count).then_some((hist.addr[i], hist.val[i])));
        let theirs =
            (i < apu_ref.hist.count).then_some((apu_ref.hist.addr[i], apu_ref.hist.val[i]));
        if mine != theirs {
            if errors == 0 {
                eprintln!("SPC DSP write divergence:");
            }
            match mine {
                Some((addr, val)) => eprint!("{i}: [{addr:02x}: {val:02x}]"),
                None => eprint!("{i}: [??: ??]"),
            }
            eprint!(" != ");
            match theirs {
                Some((addr, val)) => eprintln!("[{addr:02x}: {val:02x}]"),
                None => eprintln!("[??: ??]"),
            }
            errors += 1;
        }
    }

    if errors != 0 {
        eprintln!("Total {errors} SPC compare error(s)");
        return false;
    }

    if let Some(hist) = hist.as_mut() {
        hist.count = 0;
    }
    apu_ref.hist.count = 0;
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_variables_to_and_from_spc_ram_uses_c_addresses() {
        let p = spc_player_create();
        let p_ref = unsafe { &mut *p };
        p_ref.new_value_from_snes = [1, 2, 3, 4];
        p_ref.port_to_snes = [5, 6, 7, 8];
        p_ref.counter_sf0c = 0x9a;
        p_ref.tempo = 0x1234;
        p_ref.echo_volume_right = 0x4567;
        p_ref.port1_current_bit = 0x80;
        p_ref.channel[2].pattern_order_ptr_for_chan = 0xabcd;
        p_ref.channel[2].note_ticks_left = 0x44;
        p_ref.channel[2].instrument_pitch_base = 0x5678;
        p_ref.channel[2].pan_value = 0x1357;
        p_ref.channel[2].sfx_pan = 0xc0;

        spc_player_copy_variables_to_ram(p);
        assert_eq!(&p_ref.ram[0x0000..0x0004], &[1, 2, 3, 4]);
        assert_eq!(&p_ref.ram[0x0004..0x0008], &[5, 6, 7, 8]);
        assert_eq!(p_ref.ram[0x000c], 0x9a);
        assert_eq!(word(&p_ref.ram, 0x0052), 0x1234);
        assert_eq!(word(&p_ref.ram, 0x0062), 0x4567);
        assert_eq!(p_ref.ram[0x03e0], 0x80);
        assert_eq!(word(&p_ref.ram, 0x0030 + 2 * 2), 0xabcd);
        assert_eq!(p_ref.ram[0x0070 + 2 * 2], 0x44);
        assert_eq!(word(&p_ref.ram, 0x0220 + 2 * 2), 0x5678);
        assert_eq!(word(&p_ref.ram, 0x0330 + 2 * 2), 0x1357);
        assert_eq!(p_ref.ram[0x03d0 + 2 * 2], 0xc0);

        p_ref.ram[0x0052] = 0xef;
        p_ref.ram[0x0053] = 0xbe;
        p_ref.ram[0x0070 + 2 * 2] = 0x55;
        p_ref.ram[0x03d0 + 2 * 2] = 0x40;
        spc_player_copy_variables_from_ram(p);
        assert_eq!(p_ref.tempo, 0xbeef);
        assert_eq!(p_ref.channel[2].note_ticks_left, 0x55);
        assert_eq!(p_ref.channel[2].sfx_pan, 0x40);

        spc_player_destroy(p);
    }
}
