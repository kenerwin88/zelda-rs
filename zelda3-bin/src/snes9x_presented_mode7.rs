//! Replaceable presentation-domain adapter for completed Mode 7 transforms.
//!
//! Pinned Snes9x exposes its private per-scanline PPU result through a debug
//! field. This module validates and converts it into Zelda's typed receipt;
//! CPU addresses, raster counters, and emulator ownership stop here.

use crate::libretro_core::LibretroCore;
use zelda3::PresentedMode7Transform;

pub(crate) fn snes9x_presented_mode7_transform(
    oracle: &LibretroCore,
) -> Result<Option<PresentedMode7Transform>, String> {
    decode_presented_mode7_transform(|line, field| oracle.debug_scanline_mode7_value(line, field))
}

fn decode_presented_mode7_transform(
    mut value: impl FnMut(i32, i32) -> Option<i32>,
) -> Result<Option<PresentedMode7Transform>, String> {
    match value(0, 0) {
        None | Some(-1) => return Ok(None),
        Some(_) => {}
    }
    let scanlines = (0..PresentedMode7Transform::VISIBLE_LINES)
        .map(|line| {
            let mut transform = [0; PresentedMode7Transform::FIELD_COUNT];
            for (field, output) in transform.iter_mut().enumerate() {
                let raw = value(line as i32, field as i32).ok_or_else(|| {
                    format!("presented Mode 7 line {line} field {field} is unavailable")
                })?;
                *output = i16::try_from(raw).map_err(|_| {
                    format!("presented Mode 7 line {line} field {field} is invalid: {raw}")
                })?;
            }
            Ok(transform)
        })
        .collect::<Result<Vec<_>, String>>()?;
    PresentedMode7Transform::new(scanlines)
        .map(Some)
        .ok_or_else(|| "presented Mode 7 receipt has an invalid line count".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoder_requires_one_complete_signed_transform_raster() {
        let receipt = decode_presented_mode7_transform(|line, field| Some(line * 8 + field - 900))
            .unwrap()
            .unwrap();
        assert_eq!(
            receipt.scanlines()[3],
            [-876, -875, -874, -873, -872, -871, -870, -869]
        );
        assert!(decode_presented_mode7_transform(|_, _| None)
            .unwrap()
            .is_none());
        assert!(decode_presented_mode7_transform(|_, _| Some(-1))
            .unwrap()
            .is_none());
        assert!(decode_presented_mode7_transform(|line, field| {
            (!(line == 7 && field == 5)).then_some(0)
        })
        .unwrap_err()
        .contains("line 7 field 5 is unavailable"));
        assert!(decode_presented_mode7_transform(|line, field| {
            Some(if line == 9 && field == 2 { 0x1_0000 } else { 0 })
        })
        .unwrap_err()
        .contains("line 9 field 2 is invalid"));
    }
}
