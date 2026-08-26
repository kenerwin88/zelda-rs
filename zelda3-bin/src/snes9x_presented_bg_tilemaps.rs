//! Replaceable presentation-domain adapter for BG name-table generations.
//!
//! Pinned Snes9x exposes private scanout evidence through two debug fields.
//! This module validates and converts it into Zelda's typed receipt. CPU,
//! raster, DMA, and emulator-cache details stop at this boundary.

use crate::libretro_core::LibretroCore;
use zelda3::{PresentedBgTilemapLayer, PresentedBgTilemaps};

const META_FIELD: i32 = 41;
const WORD_FIELD: i32 = 42;
const SCHEMA_VERSION: i32 = 1;
const MAX_WORDS_PER_LAYER: usize = 4096;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LayerSignature {
    word_address: u16,
    size: u8,
    word_count: usize,
    hash: u32,
}

#[derive(Default)]
pub(crate) struct PresentedBgTilemapCache {
    signature: Option<[LayerSignature; PresentedBgTilemaps::LAYER_COUNT]>,
    receipt: Option<PresentedBgTilemaps>,
}

pub(crate) fn snes9x_presented_bg_tilemaps(
    oracle: &LibretroCore,
    cache: &mut PresentedBgTilemapCache,
) -> Result<Option<PresentedBgTilemaps>, String> {
    decode_presented_bg_tilemaps(|field, index| oracle.debug_ppu_value(field, index), cache)
}

fn decode_presented_bg_tilemaps(
    mut value: impl FnMut(i32, i32) -> Option<i32>,
    cache: &mut PresentedBgTilemapCache,
) -> Result<Option<PresentedBgTilemaps>, String> {
    let schema = match value(META_FIELD, 0) {
        None | Some(-1) => return Ok(None),
        // A scanout can legitimately span more than one BG-map generation.
        // Such a frame has no single whole-map semantic receipt; leave the
        // domain unavailable instead of exposing a synthetic generation.
        Some(-2) => return Ok(None),
        Some(schema) => schema,
    };
    if schema != SCHEMA_VERSION {
        return Err(format!(
            "presented BG tilemap schema is {schema}, expected {SCHEMA_VERSION}"
        ));
    }

    let mut signatures = Vec::with_capacity(PresentedBgTilemaps::LAYER_COUNT);
    for layer in 0..PresentedBgTilemaps::LAYER_COUNT {
        let base = 1 + layer * 5;
        let word_address = read_meta(&mut value, base, layer, "word address")?;
        let size = read_meta(&mut value, base + 1, layer, "size")?;
        let word_count = read_meta(&mut value, base + 2, layer, "word count")?;
        let hash_low = read_meta(&mut value, base + 3, layer, "hash low")?;
        let hash_high = read_meta(&mut value, base + 4, layer, "hash high")?;
        let hash = u32::try_from(hash_low)
            .ok()
            .filter(|&part| part <= 0xffff)
            .zip(u32::try_from(hash_high).ok().filter(|&part| part <= 0xffff))
            .map(|(low, high)| low | (high << 16))
            .ok_or_else(|| format!("presented BG{layer} tilemap hash is invalid"))?;
        let word_address = u16::try_from(word_address)
            .ok()
            .filter(|&address| address < 0x8000)
            .ok_or_else(|| {
                format!("presented BG{layer} tilemap word address is invalid: {word_address}")
            })?;
        let size = u8::try_from(size)
            .ok()
            .filter(|&size| size < 4)
            .ok_or_else(|| format!("presented BG{layer} tilemap size is invalid: {size}"))?;
        let expected_words = 1024usize << size.count_ones();
        let word_count = usize::try_from(word_count)
            .ok()
            .filter(|&count| count == expected_words && count <= MAX_WORDS_PER_LAYER)
            .ok_or_else(|| {
                format!(
                    "presented BG{layer} tilemap word count is {word_count}, expected {expected_words}"
                )
            })?;
        signatures.push(LayerSignature {
            word_address,
            size,
            word_count,
            hash,
        });
    }
    let signatures: [LayerSignature; PresentedBgTilemaps::LAYER_COUNT] = signatures
        .try_into()
        .map_err(|_| "presented BG tilemap metadata omitted a layer".to_string())?;
    if cache.signature == Some(signatures) {
        return cache
            .receipt
            .clone()
            .map(Some)
            .ok_or_else(|| "presented BG tilemap cache lost its receipt".to_string());
    }

    let mut layers = Vec::with_capacity(PresentedBgTilemaps::LAYER_COUNT);
    for (layer, signature) in signatures.iter().enumerate() {
        let words = (0..signature.word_count)
            .map(|offset| {
                let index = layer * MAX_WORDS_PER_LAYER + offset;
                let word = value(WORD_FIELD, index as i32).ok_or_else(|| {
                    format!("presented BG{layer} tilemap word {offset} is unavailable")
                })?;
                u16::try_from(word).map_err(|_| {
                    format!("presented BG{layer} tilemap word {offset} is invalid: {word}")
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        layers.push(
            PresentedBgTilemapLayer::new(
                layer as u8,
                signature.word_address,
                signature.size & 1 != 0,
                signature.size & 2 != 0,
                words,
            )
            .ok_or_else(|| format!("presented BG{layer} tilemap receipt has an invalid shape"))?,
        );
    }
    let receipt = PresentedBgTilemaps::new(layers)
        .ok_or_else(|| "presented BG tilemap receipt is incomplete".to_string())?;
    cache.signature = Some(signatures);
    cache.receipt = Some(receipt.clone());
    Ok(Some(receipt))
}

fn read_meta(
    value: &mut impl FnMut(i32, i32) -> Option<i32>,
    index: usize,
    layer: usize,
    name: &str,
) -> Result<i32, String> {
    match value(META_FIELD, index as i32) {
        Some(-2) => Err("presented BG tilemaps changed during the completed scanout".to_string()),
        Some(value) => Ok(value),
        None => Err(format!("presented BG{layer} tilemap {name} is unavailable")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoder_validates_all_layers_and_reuses_unchanged_generation() {
        use std::cell::Cell;

        let mut cache = PresentedBgTilemapCache::default();
        let word_reads = Cell::new(0);
        let mut values = |field: i32, index: i32| match field {
            META_FIELD if index == 0 => Some(SCHEMA_VERSION),
            META_FIELD => {
                let field = (index - 1) % 5;
                let layer = (index - 1) / 5;
                Some(match field {
                    0 => layer * 0x400,
                    1 => 0,
                    2 => 0x400,
                    3 => 0x1000 + layer,
                    4 => 0x2000 + layer,
                    _ => unreachable!(),
                })
            }
            WORD_FIELD => {
                word_reads.set(word_reads.get() + 1);
                Some(index & 0xffff)
            }
            _ => None,
        };

        assert!(decode_presented_bg_tilemaps(&mut values, &mut cache)
            .unwrap()
            .is_some());
        assert_eq!(word_reads.get(), 4 * 0x400);
        assert!(decode_presented_bg_tilemaps(&mut values, &mut cache)
            .unwrap()
            .is_some());
        assert_eq!(
            word_reads.get(),
            4 * 0x400,
            "unchanged metadata/hash must not re-read every map word"
        );
    }

    #[test]
    fn decoder_omits_nonuniform_scanout_instead_of_inventing_a_generation() {
        let mut cache = PresentedBgTilemapCache::default();
        let receipt = decode_presented_bg_tilemaps(
            |field, index| (field == META_FIELD && index == 0).then_some(-2),
            &mut cache,
        )
        .unwrap();
        assert!(receipt.is_none());
    }
}
