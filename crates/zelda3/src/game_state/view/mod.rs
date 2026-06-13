//! Byte-backed typed views over game state.

use crate::types::{read_le_u16, write_le_u16};

fn copy_word(ram: &mut [u8], dst: usize, src: usize) {
    let value = read_le_u16(ram, src);
    write_le_u16(ram, dst, value);
}

use crate::game_state::constants::*;

mod effects;
mod player;
mod poly;
mod raw;
mod sprites;
mod world;

pub(crate) use effects::*;
pub(crate) use player::*;
pub(crate) use poly::*;
pub(crate) use raw::*;
pub(crate) use sprites::*;
pub(crate) use world::*;

fn byte(ram: &[u8], offset: usize) -> u8 {
    ram.get(offset).copied().unwrap_or(0)
}

fn word(ram: &[u8], offset: usize) -> u16 {
    if offset + 1 < ram.len() {
        read_le_u16(ram, offset)
    } else {
        0
    }
}

fn packed_position(ram: &[u8], low_offset: usize, high_offset: usize) -> u16 {
    u16::from(byte(ram, low_offset)) | (u16::from(byte(ram, high_offset)) << 8)
}

fn write_position(ram: &mut [u8], low_offset: usize, high_offset: usize, value: u16) {
    ram[low_offset] = value as u8;
    ram[high_offset] = (value >> 8) as u8;
}

fn move_axis24(
    ram: &mut [u8],
    subpixel_offset: usize,
    low_offset: usize,
    high_offset: usize,
    velocity_offset: usize,
) {
    let pos = u32::from(ram[subpixel_offset])
        | (u32::from(ram[low_offset]) << 8)
        | (u32::from(ram[high_offset]) << 16);
    let delta = ((ram[velocity_offset] as i8 as i32) << 4) as u32;
    let moved = pos.wrapping_add(delta);
    ram[subpixel_offset] = moved as u8;
    ram[low_offset] = (moved >> 8) as u8;
    ram[high_offset] = (moved >> 16) as u8;
}

fn move_axis16(ram: &mut [u8], subpixel_offset: usize, offset: usize, velocity_offset: usize) {
    let pos = (u16::from(ram[offset]) << 8) | u16::from(ram[subpixel_offset]);
    let delta = ((ram[velocity_offset] as i8 as i32) << 4) as u16;
    let moved = pos.wrapping_add(delta);
    ram[subpixel_offset] = moved as u8;
    ram[offset] = (moved >> 8) as u8;
}

fn move_link_axis_by_velocity(
    ram: &mut [u8],
    subpixel_offset: usize,
    coord_offset: usize,
    velocity: u8,
) -> u16 {
    let pos = u32::from(ram[subpixel_offset]) | (u32::from(read_le_u16(ram, coord_offset)) << 8);
    let delta = ((velocity as i8 as i32) << 4) as u32;
    let moved = pos.wrapping_add(delta);
    ram[subpixel_offset] = moved as u8;
    write_le_u16(ram, coord_offset, (moved >> 8) as u16);
    (moved >> 8) as u16
}

fn move_link_axis_by_subpixel_delta(
    ram: &mut [u8],
    subpixel_offset: usize,
    coord_offset: usize,
    delta: u16,
) -> u16 {
    let pos = u32::from(ram[subpixel_offset]) | (u32::from(read_le_u16(ram, coord_offset)) << 8);
    let moved = pos.wrapping_add(delta as i16 as i32 as u32);
    ram[subpixel_offset] = moved as u8;
    write_le_u16(ram, coord_offset, (moved >> 8) as u16);
    (moved >> 8) as u16
}
