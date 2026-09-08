use std::io::Read;
use std::path::Path;

/// An indexed PNG whose unexpanded palette indices form an 8x8 tile grid.
pub(crate) struct IndexedTileSheet {
    pub(crate) width: usize,
    pub(crate) height: usize,
    indices: Vec<u8>,
}

impl IndexedTileSheet {
    // Callers open the file so their command-specific error context stays intact.
    pub(crate) fn decode(source: impl Read, path: &Path) -> Result<Self, String> {
        let decoder = png::Decoder::new(source);
        let mut reader = decoder
            .read_info()
            .map_err(|e| format!("failed to read PNG header {}: {e}", path.display()))?;
        let mut indices = vec![0u8; reader.output_buffer_size()];
        let info = reader
            .next_frame(&mut indices)
            .map_err(|e| format!("failed to decode {}: {e}", path.display()))?;
        if info.bit_depth != png::BitDepth::Eight || info.color_type != png::ColorType::Indexed {
            return Err(format!(
                "{}: expected 8-bit indexed PNG, got {:?}/{:?}",
                path.display(),
                info.color_type,
                info.bit_depth
            ));
        }
        let width = info.width as usize;
        let height = info.height as usize;
        if !width.is_multiple_of(8) || !height.is_multiple_of(8) {
            return Err(format!(
                "{}: PNG size {}x{} is not aligned to 8x8 cells",
                path.display(),
                info.width,
                info.height
            ));
        }
        indices.truncate(info.buffer_size());
        Ok(Self {
            width,
            height,
            indices,
        })
    }

    /// Manifest IDs run left to right, then top to bottom. Missing cells are
    /// reported to the caller, which decides whether to skip them or fail.
    pub(crate) fn cell(&self, id: usize) -> Option<[u8; 64]> {
        let cols = self.width / 8;
        let cx = (id % cols) * 8;
        let cy = (id / cols) * 8;
        if cy + 8 > self.height || cx + 8 > self.width {
            return None;
        }
        let mut pattern = [0u8; 64];
        for row in 0..8usize {
            let src = (cy + row) * self.width + cx;
            pattern[row * 8..row * 8 + 8].copy_from_slice(&self.indices[src..src + 8]);
        }
        Some(pattern)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png_bytes(
        width: u32,
        height: u32,
        color: png::ColorType,
        depth: png::BitDepth,
        data: &[u8],
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut bytes, width, height);
            encoder.set_color(color);
            encoder.set_depth(depth);
            if color == png::ColorType::Indexed {
                encoder.set_palette(vec![0; 256 * 3]);
            }
            encoder
                .write_header()
                .unwrap()
                .write_image_data(data)
                .unwrap();
        }
        bytes
    }

    #[test]
    fn cells_preserve_indices_and_manifest_grid_order() {
        // Every palette entry has the same RGB value; cell contents must retain
        // indices, not an expansion to palette colors.
        let pixels = (0..256).map(|value| value as u8).collect::<Vec<_>>();
        let png = png_bytes(
            16,
            16,
            png::ColorType::Indexed,
            png::BitDepth::Eight,
            &pixels,
        );
        let sheet = IndexedTileSheet::decode(png.as_slice(), Path::new("sheet.png")).unwrap();
        for (id, (left, top)) in [(0, 0), (8, 0), (0, 8), (8, 8)].into_iter().enumerate() {
            let expected = std::array::from_fn(|i| ((top + i / 8) * 16 + left + i % 8) as u8);
            assert_eq!(sheet.cell(id), Some(expected));
        }
        assert_eq!(sheet.cell(4), None);
    }

    #[test]
    fn rejects_unsupported_pixel_formats_and_unaligned_dimensions() {
        for (width, height, color, depth, pixels, expected) in [
            (
                8,
                8,
                png::ColorType::Rgb,
                png::BitDepth::Eight,
                vec![0; 192],
                "sheet.png: expected 8-bit indexed PNG, got Rgb/Eight",
            ),
            (
                8,
                8,
                png::ColorType::Indexed,
                png::BitDepth::Four,
                vec![0; 32],
                "sheet.png: expected 8-bit indexed PNG, got Indexed/Four",
            ),
            (
                7,
                8,
                png::ColorType::Indexed,
                png::BitDepth::Eight,
                vec![0; 56],
                "sheet.png: PNG size 7x8 is not aligned to 8x8 cells",
            ),
            (
                8,
                7,
                png::ColorType::Indexed,
                png::BitDepth::Eight,
                vec![0; 56],
                "sheet.png: PNG size 8x7 is not aligned to 8x8 cells",
            ),
        ] {
            let png = png_bytes(width, height, color, depth, &pixels);
            assert_eq!(
                IndexedTileSheet::decode(png.as_slice(), Path::new("sheet.png"))
                    .err()
                    .unwrap(),
                expected
            );
        }
    }

    #[test]
    fn corrupt_png_errors_retain_the_path_and_decode_stage() {
        let path = Path::new("broken.png");
        let error = IndexedTileSheet::decode(&b"not a PNG"[..], path)
            .err()
            .unwrap();
        assert!(error.starts_with("failed to read PNG header broken.png:"));

        let mut png = png_bytes(
            8,
            8,
            png::ColorType::Indexed,
            png::BitDepth::Eight,
            &[0; 64],
        );
        let data_start = png.windows(4).position(|bytes| bytes == b"IDAT").unwrap() + 4;
        png.truncate(data_start);
        let error = IndexedTileSheet::decode(png.as_slice(), path)
            .err()
            .unwrap();
        assert!(error.starts_with("failed to decode broken.png:"), "{error}");
    }
}
