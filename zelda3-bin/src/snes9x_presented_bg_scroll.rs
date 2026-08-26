//! Replaceable presentation-domain adapter for completed BG scroll registers.
//!
//! Pinned Snes9x exposes private scanout state through a debug field. This
//! module validates and converts it into Zelda's typed receipt; CPU, raster,
//! and emulator provenance stop at this boundary.

use crate::libretro_core::LibretroCore;
use zelda3::PresentedBgScroll;

const BG_H_SCROLL_FIELD: i32 = 33;
const BG_V_SCROLL_FIELD: i32 = 34;

pub(crate) fn snes9x_presented_bg_scroll(
    oracle: &LibretroCore,
) -> Result<Option<PresentedBgScroll>, String> {
    decode_presented_bg_scroll(|field, index| oracle.debug_ppu_value(field, index))
}

fn decode_presented_bg_scroll(
    mut value: impl FnMut(i32, i32) -> Option<i32>,
) -> Result<Option<PresentedBgScroll>, String> {
    match value(BG_H_SCROLL_FIELD, 0) {
        None | Some(-1) => return Ok(None),
        Some(_) => {}
    }
    let scanlines = (0..PresentedBgScroll::VISIBLE_LINES)
        .map(|line| {
            let mut scanline = [[0; 2]; PresentedBgScroll::LAYER_COUNT];
            for (layer, offsets) in scanline.iter_mut().enumerate() {
                let index = (line * PresentedBgScroll::LAYER_COUNT + layer) as i32;
                let mut read = |field, axis| -> Result<u16, String> {
                    let raw = value(field, index).ok_or_else(|| {
                        format!(
                            "presented BG scroll line {line} layer {layer} {axis} is unavailable"
                        )
                    })?;
                    let raw = u16::try_from(raw).map_err(|_| {
                        format!(
                            "presented BG scroll line {line} layer {layer} {axis} is invalid: {raw}"
                        )
                    })?;
                    // Pinned Snes9x stores `PPU.BG.VOffset + 1` in LineData
                    // before rendering. Zelda's renderer owns the identical
                    // SNES fetch +1 itself, so its typed register input is the
                    // source LineData value normalized back to VOffset.
                    Ok(if field == BG_V_SCROLL_FIELD {
                        raw.wrapping_sub(1)
                    } else {
                        raw
                    })
                };
                *offsets = [read(BG_H_SCROLL_FIELD, "H")?, read(BG_V_SCROLL_FIELD, "V")?];
            }
            Ok(scanline)
        })
        .collect::<Result<Vec<_>, String>>()?;
    PresentedBgScroll::new(scanlines)
        .map(Some)
        .ok_or_else(|| "presented BG scroll receipt has an invalid line count".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoder_requires_one_complete_scanline_raster() {
        let receipt = decode_presented_bg_scroll(|field, index| {
            Some(
                (if field == BG_H_SCROLL_FIELD {
                    0x1000
                } else {
                    0x2000
                }) + index,
            )
        })
        .unwrap();
        assert_eq!(receipt.unwrap().scanlines()[3][2], [0x100e, 0x200d]);
        assert!(decode_presented_bg_scroll(|_, _| None).unwrap().is_none());
        assert!(decode_presented_bg_scroll(|_, _| Some(-1))
            .unwrap()
            .is_none());
        assert!(decode_presented_bg_scroll(|field, index| {
            (!(field == BG_V_SCROLL_FIELD && index == 5)).then_some(index)
        })
        .unwrap_err()
        .contains("line 1 layer 1 V is unavailable"));
        assert!(decode_presented_bg_scroll(|field, index| {
            Some(if field == BG_H_SCROLL_FIELD && index == 7 {
                -2
            } else {
                index
            })
        })
        .unwrap_err()
        .contains("line 1 layer 3 H is invalid"));
    }
}
