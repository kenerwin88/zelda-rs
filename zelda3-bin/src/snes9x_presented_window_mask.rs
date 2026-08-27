//! Replaceable presentation-domain adapter for completed window boundaries.
//!
//! Pinned Snes9x exposes private scanout state through a debug field. This
//! module validates and converts it into Zelda's typed receipt; HDMA tables,
//! raster counters, and emulator provenance stop at this boundary.

use crate::libretro_core::LibretroCore;
use zelda3::PresentedWindowMask;

const WINDOW_FIELD_BASE: i32 = 8;

pub(crate) fn snes9x_presented_window_mask(
    oracle: &LibretroCore,
) -> Result<Option<PresentedWindowMask>, String> {
    decode_presented_window_mask(
        |line, field| oracle.debug_scanline_mode7_value(line, field),
        |screen| oracle.debug_ppu_value(43, screen),
        |index| oracle.debug_ppu_value(44, index),
    )
}

fn decode_presented_window_mask(
    mut value: impl FnMut(i32, i32) -> Option<i32>,
    mut screen_mask_value: impl FnMut(i32) -> Option<i32>,
    mut layer_predicate_value: impl FnMut(i32) -> Option<i32>,
) -> Result<Option<PresentedWindowMask>, String> {
    match value(0, WINDOW_FIELD_BASE) {
        None | Some(-1) => return Ok(None),
        Some(_) => {}
    }
    let scanlines = (0..PresentedWindowMask::VISIBLE_LINES)
        .map(|line| {
            let mut scanline = [[0; 2]; PresentedWindowMask::WINDOW_COUNT];
            for (window, bounds) in scanline.iter_mut().enumerate() {
                for (side, bound) in bounds.iter_mut().enumerate() {
                    let field = WINDOW_FIELD_BASE + (window * 2 + side) as i32;
                    let raw = value(line as i32, field).ok_or_else(|| {
                        format!(
                            "presented window mask line {line} window {window} side {side} is unavailable"
                        )
                    })?;
                    *bound = u8::try_from(raw).map_err(|_| {
                        format!(
                            "presented window mask line {line} window {window} side {side} is invalid: {raw}"
                        )
                    })?;
                }
            }
            Ok(scanline)
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut screen_windowed_layers = [0u8; 2];
    for screen in 0..2 {
        let raw = screen_mask_value(screen as i32)
            .ok_or_else(|| format!("presented window screen {screen} layer mask is unavailable"))?;
        for line in 1..PresentedWindowMask::VISIBLE_LINES {
            let index = (line * 2 + screen) as i32;
            let line_mask = screen_mask_value(index).ok_or_else(|| {
                format!("presented window screen {screen} layer mask is unavailable on line {line}")
            })?;
            if line_mask != raw {
                return Err(format!(
                    "presented window screen {screen} layer mask changes on line {line}: {raw} -> {line_mask}"
                ));
            }
        }
        let mask = u8::try_from(raw)
            .ok()
            .filter(|mask| mask & !0x3f == 0)
            .ok_or_else(|| {
                format!("presented window screen {screen} layer mask is invalid: {raw}")
            })?;
        screen_windowed_layers[screen] = mask;
    }
    let mut layer_predicates = [0u8; 6];
    for (layer, predicate) in layer_predicates.iter_mut().enumerate() {
        let raw = layer_predicate_value(layer as i32).ok_or_else(|| {
            format!("presented window predicate for layer {layer} is unavailable")
        })?;
        for line in 1..PresentedWindowMask::VISIBLE_LINES {
            let index = (line * 6 + layer) as i32;
            let line_predicate = layer_predicate_value(index).ok_or_else(|| {
                format!(
                    "presented window predicate for layer {layer} is unavailable on line {line}"
                )
            })?;
            if line_predicate != raw {
                return Err(format!(
                    "presented window predicate for layer {layer} changes on line {line}: {raw} -> {line_predicate}"
                ));
            }
        }
        if !(0..=0x3f).contains(&raw) {
            return Err(format!(
                "presented window predicate for layer {layer} is invalid: {raw}"
            ));
        }
        if raw & 0x30 != 0 {
            return Err(format!(
                "presented window predicate for layer {layer} uses unsupported non-OR overlap logic: {}",
                raw >> 4,
            ));
        }
        *predicate = raw as u8 & 0x0f;
    }
    PresentedWindowMask::new(scanlines, screen_windowed_layers, layer_predicates)
        .map(Some)
        .ok_or_else(|| "presented window mask receipt has an invalid line count".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoder_requires_one_complete_window_raster() {
        let receipt = decode_presented_window_mask(
            |line, field| Some((line + field - WINDOW_FIELD_BASE) & 0xff),
            |_| Some(0),
            |index| Some(if index % 6 == 1 { 3 } else { 0 }),
        )
        .unwrap()
        .unwrap();
        assert_eq!(receipt.scanlines()[3], [[3, 4], [5, 6]]);
        assert_eq!(receipt.screen_windowed_layers(), [0, 0]);
        assert_eq!(receipt.layer_predicates(), [0, 3, 0, 0, 0, 0]);
        assert!(
            decode_presented_window_mask(|_, _| None, |_| None, |_| None)
                .unwrap()
                .is_none()
        );
        assert!(
            decode_presented_window_mask(|_, _| Some(-1), |_| None, |_| None)
                .unwrap()
                .is_none()
        );
        assert!(decode_presented_window_mask(
            |line, field| (!(line == 5 && field == 10)).then_some(0),
            |_| Some(0),
            |_| Some(0),
        )
        .unwrap_err()
        .contains("line 5 window 1 side 0 is unavailable"));
        assert!(decode_presented_window_mask(
            |line, field| Some(if line == 7 && field == 11 { 256 } else { 0 }),
            |_| Some(0),
            |_| Some(0),
        )
        .unwrap_err()
        .contains("line 7 window 1 side 1 is invalid"));

        let receipt = decode_presented_window_mask(
            |_, _| Some(0),
            |index| Some(if index & 1 == 0 { 0x16 } else { 0 }),
            |index| Some(if index % 6 == 4 { 3 } else { 0 }),
        )
        .unwrap()
        .unwrap();
        assert_eq!(receipt.screen_windowed_layers(), [0x16, 0]);
        assert_eq!(receipt.layer_predicates(), [0, 0, 0, 0, 3, 0]);

        assert!(decode_presented_window_mask(
            |_, _| Some(0),
            |_| Some(0),
            |index| Some(if index % 6 == 2 { 0x10 } else { 0 }),
        )
        .unwrap_err()
        .contains("non-OR overlap logic"));
    }
}
