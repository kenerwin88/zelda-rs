//! Milestone 1 tests: per-VRAM-slot logical CHR source bookkeeping populates
//! when graphics are written to VRAM (BG / sprite via do3->4, Link via DMA),
//! and is purely additive (does not change the VRAM bytes written).

use super::*;
use crate::chr_source::{
    LogicalChrSrc, VramChrSourceTable, CHR_KIND_BG, CHR_KIND_LINK, CHR_KIND_NONE, CHR_KIND_SPRITE,
};

#[test]
fn table_starts_empty() {
    let state = ZeldaState::new();
    // Every slot defaults to "none" before any CHR upload (RED baseline).
    assert_eq!(state.vram_chr_source().get(0x200), LogicalChrSrc::default());
    assert_eq!(state.vram_chr_source().get(0x480).kind, CHR_KIND_NONE);
    assert_eq!(state.vram_chr_source().as_slice().len(), 0x800);
}

#[test]
fn do3_to_4_low_to_vram_records_bg_chr_source() {
    let mut state = ZeldaState::new();
    let data = vec![0u8; 0x600];

    let before = state.ppu.vram.clone();
    state.do3_to_4_low_to_vram(0x2000, &data, CHR_KIND_BG, 5);

    // BG CHR region: word 0x2000 / 16 = slot 0x200.
    let s0 = state.vram_chr_source().get(0x200);
    assert_eq!(s0.kind, CHR_KIND_BG);
    assert_eq!(s0.pack, 5);
    assert_eq!(s0.tile_off, 0);

    let s10 = state.vram_chr_source().get(0x200 + 10);
    assert_eq!(s10.kind, CHR_KIND_BG);
    assert_eq!(s10.pack, 5);
    assert_eq!(s10.tile_off, 10);

    // do3 writes exactly 64 tiles -> slots 0x200..0x240; 0x240 stays untouched.
    assert_eq!(state.vram_chr_source().get(0x23f).kind, CHR_KIND_BG);
    assert_eq!(state.vram_chr_source().get(0x240).kind, CHR_KIND_NONE);

    // The bookkeeping must NOT change the VRAM bytes (here data is zeros so the
    // low planes stay zero); confirm the write footprint is byte-identical to a
    // run that produced the same VRAM (sanity: vram changed only where do3 wrote).
    // Region outside [0x2000, 0x2400) is unchanged.
    assert_eq!(&state.ppu.vram[0x2400..], &before[0x2400..]);
}

#[test]
fn do3_to_4_high_to_vram_records_sprite_chr_source() {
    let mut state = ZeldaState::new();
    let data = vec![0u8; 0x600];

    // Sprite CHR region: word 0x4800 / 16 = slot 0x480 (>= 0x400).
    state.do3_to_4_high_to_vram(0x4800, &data, CHR_KIND_SPRITE, 94);

    let s = state.vram_chr_source().get(0x480);
    assert_eq!(s.kind, CHR_KIND_SPRITE);
    assert_eq!(s.pack, 94);
    assert_eq!(s.tile_off, 0);
    assert_eq!(state.vram_chr_source().get(0x480 + 0x3f).kind, CHR_KIND_SPRITE);
}

#[test]
fn record_words_tags_link_chr_region() {
    // The Link per-frame DMA path tags its VRAM slots kind=link with the active
    // link_dma_graphics_index as the pack. Exercise the recording helper the
    // way nmi_core_link_graphics_update calls it (len bytes -> len/2 words).
    let mut t = VramChrSourceTable::new();
    // Link body top at VRAM word 0x4000 (slot 0x400), 0x40 bytes = 0x20 words.
    t.record_words(0x4000, 0x40 / 2, CHR_KIND_LINK, 7);

    let s = t.get(0x400);
    assert_eq!(s.kind, CHR_KIND_LINK);
    assert_eq!(s.pack, 7);
    // 0x20 words = 2 tiles -> slots 0x400, 0x401.
    assert_eq!(t.get(0x401).kind, CHR_KIND_LINK);
    assert_eq!(t.get(0x402).kind, CHR_KIND_NONE);
}
