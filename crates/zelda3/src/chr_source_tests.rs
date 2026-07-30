//! Milestone 1 tests: per-VRAM-slot logical CHR source bookkeeping populates
//! when graphics are written to VRAM (BG / sprite via do3->4, Link via DMA),
//! and is purely additive (does not change the VRAM bytes written).

use super::*;
use crate::chr_source::{
    LogicalChrSrc, VramChrSourceTable, CHR_KIND_BG, CHR_KIND_BG_STREAM, CHR_KIND_LINK,
    CHR_KIND_NONE, CHR_KIND_SPRITE,
};

#[test]
fn table_starts_empty() {
    let state = ZeldaState::new();
    // Every slot defaults to "none" before any CHR upload (RED baseline).
    assert_eq!(state.vram_chr_source().get(0x200), LogicalChrSrc::default());
    assert_eq!(
        state.vram_chr_preview_source().get(0x200),
        LogicalChrSrc::default()
    );
    assert_eq!(state.vram_chr_source().get(0x480).kind, CHR_KIND_NONE);
    assert_eq!(state.vram_chr_source().as_slice().len(), 0x800);
    assert_eq!(state.vram_chr_preview_source().as_slice().len(), 0x800);
}

#[test]
fn logical_chr_src_converts_to_renderer_source_tuple() {
    let src = LogicalChrSrc {
        kind: CHR_KIND_SPRITE,
        pack: 94,
        tile_off: 12,
    };

    assert_eq!(<(u8, u16, u16)>::from(src), (CHR_KIND_SPRITE, 94, 12));
}

#[test]
fn copying_a_vram_word_range_publishes_every_touched_chr_slot() {
    let mut captured = VramChrSourceTable::new();
    let mut following = VramChrSourceTable::new();
    captured.record_tiles(0x4000, 4, CHR_KIND_LINK, 7);
    following.record_tiles(0x4000, 4, CHR_KIND_LINK, 9);

    following.copy_word_range_from(&captured, 0x4008, 16);

    assert_eq!(following.get(0x400).pack, 7);
    assert_eq!(following.get(0x401).pack, 7);
    assert_eq!(following.get(0x402).pack, 9);
}

#[test]
fn bg_chr_upload_keeps_raw_preview_source_after_render_source_is_hashed() {
    let mut state = ZeldaState::new();
    let data = vec![0u8; 0x600];

    let before = state.ppu.vram.clone();
    state.do3_to_4_low_to_vram(0x2000, &data, CHR_KIND_BG, 5);

    // BG CHR region: word 0x2000 / 16 = slot 0x200. The render source is
    // content-hashed because generic BG pack/off keys are non-injective across
    // conversion modes and themes.
    let s0 = state.vram_chr_source().get(0x200);
    assert_eq!(s0.kind, CHR_KIND_BG_STREAM);

    let s10 = state.vram_chr_source().get(0x200 + 10);
    assert_eq!(s10.kind, CHR_KIND_BG_STREAM);

    // The preview source remains the raw logical pack/off for palette usage and
    // modder-facing atlas organization.
    let preview0 = state.vram_chr_preview_source().get(0x200);
    assert_eq!(preview0.kind, CHR_KIND_BG);
    assert_eq!(preview0.pack, 5);
    assert_eq!(preview0.tile_off, 0);

    let preview10 = state.vram_chr_preview_source().get(0x200 + 10);
    assert_eq!(preview10.kind, CHR_KIND_BG);
    assert_eq!(preview10.pack, 5);
    assert_eq!(preview10.tile_off, 10);

    // do3 writes exactly 64 tiles -> slots 0x200..0x240; 0x240 stays untouched.
    assert_eq!(state.vram_chr_source().get(0x23f).kind, CHR_KIND_BG_STREAM);
    assert_eq!(state.vram_chr_source().get(0x240).kind, CHR_KIND_NONE);

    // The bookkeeping must NOT change the VRAM bytes (here data is zeros so the
    // low planes stay zero); confirm the write footprint is byte-identical to a
    // run that produced the same VRAM (sanity: vram changed only where do3 wrote).
    // Region outside [0x2000, 0x2400) is unchanged.
    assert_eq!(&state.ppu.vram[0x2400..], &before[0x2400..]);
}

#[test]
fn sprite_chr_upload_keeps_raw_preview_source_after_render_source_is_hashed() {
    let mut state = ZeldaState::new();
    let data = vec![0u8; 0x600];

    // Sprite CHR region: word 0x4800 / 16 = slot 0x480 (>= 0x400).
    state.do3_to_4_high_to_vram(0x4800, &data, CHR_KIND_SPRITE, 94);

    let s = state.vram_chr_source().get(0x480);
    assert_eq!(s.kind, CHR_KIND_BG_STREAM);

    let preview = state.vram_chr_preview_source().get(0x480);
    assert_eq!(preview.kind, CHR_KIND_SPRITE);
    assert_eq!(preview.pack, 94);
    assert_eq!(preview.tile_off, 0);
    assert_eq!(
        state.vram_chr_preview_source().get(0x480 + 0x3f).kind,
        CHR_KIND_SPRITE
    );
    assert_eq!(
        state.vram_chr_source().get(0x480 + 0x3f).kind,
        CHR_KIND_BG_STREAM
    );
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

#[test]
fn record_tiles_from_keeps_global_tile_off_across_partial_chunks() {
    // The per-frame incremental sprite upload streams a 64-tile subset to VRAM 16
    // tiles at a time; each chunk records `tile_off = base_off + t` so the key is
    // identical to a single full-subset tag. Two adjacent chunks of subset pack 19:
    let mut t = VramChrSourceTable::new();
    // chunk k=8: VRAM page 0x5800 (slot 0x580), base_off 0.
    t.record_tiles_from(0x5800, 16, CHR_KIND_SPRITE, 19, 0);
    // chunk k=11: VRAM page 0x5b00 (slot 0x5b0), base_off 48.
    t.record_tiles_from(0x5b00, 16, CHR_KIND_SPRITE, 19, 48);

    // slot 0x58c (the previously pack=0-colliding slot) → pack 19, tile_off 12.
    let a = t.get(0x58c);
    assert_eq!((a.kind, a.pack, a.tile_off), (CHR_KIND_SPRITE, 19, 12));
    // slot 0x5b4 → pack 19, tile_off 48 + 4 = 52.
    let b = t.get(0x5b4);
    assert_eq!((b.kind, b.pack, b.tile_off), (CHR_KIND_SPRITE, 19, 52));
}
