use std::error::Error;
use std::fs;
use std::io::BufWriter;
use std::path::Path;

pub(crate) fn write_argb_frame_png(
    path: &Path,
    frame: &[u8],
    width: u32,
    height: u32,
) -> Result<(), Box<dyn Error>> {
    let mut rgba = Vec::with_capacity(frame.len());
    for pixel in frame.chunks_exact(4) {
        rgba.push(pixel[2]);
        rgba.push(pixel[1]);
        rgba.push(pixel[0]);
        rgba.push(0xff);
    }
    let file = fs::File::create(path)?;
    let writer = BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut png = encoder.write_header()?;
    png.write_image_data(&rgba)?;
    Ok(())
}

/// Encode palette-slot indices as a viewable indexed PNG grid.
///
/// The renderer reads the raw index values; this viewing palette is only for
/// external tools and human inspection.
pub(crate) fn write_assets_index_png(
    path: &str,
    bin: &[u8],
    cell_count: usize,
) -> Result<(), Box<dyn Error>> {
    const ASSETS_PNG_COLUMNS: usize = 128;

    let cols = ASSETS_PNG_COLUMNS;
    let rows = cell_count.div_ceil(cols).max(1);
    let img_w = cols * 8;
    let img_h = rows * 8;
    let mut pixels = vec![0u8; img_w * img_h];
    for cell in 0..cell_count {
        let cx = (cell % cols) * 8;
        let cy = (cell / cols) * 8;
        for py in 0..8 {
            for px in 0..8 {
                pixels[(cy + py) * img_w + (cx + px)] = bin[cell * 64 + py * 8 + px];
            }
        }
    }

    let mut palette = vec![0u8; 32 * 3];
    for i in 1..32usize {
        let t = (i as u8).wrapping_mul(37);
        palette[i * 3] = t;
        palette[i * 3 + 1] = t.wrapping_mul(2).wrapping_add(48);
        palette[i * 3 + 2] = 255u8.wrapping_sub(t);
    }
    let mut trns = vec![255u8; 32];
    trns[0] = 0;

    let file = fs::File::create(path)?;
    let writer = BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, img_w as u32, img_h as u32);
    encoder.set_color(png::ColorType::Indexed);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_palette(palette);
    encoder.set_trns(trns);
    let mut png = encoder.write_header()?;
    png.write_image_data(&pixels)?;
    Ok(())
}

pub(crate) fn write_rgba_frame_png(
    path: &Path,
    rgba: &[u8],
    width: u32,
    height: u32,
) -> Result<(), Box<dyn Error>> {
    let file = fs::File::create(path)?;
    let writer = BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut png = encoder.write_header()?;
    png.write_image_data(rgba)?;
    Ok(())
}

pub(crate) fn decode_rgba_png(path: &Path) -> Option<(Vec<u8>, u32, u32)> {
    let file = fs::File::open(path).ok()?;
    let decoder = png::Decoder::new(std::io::BufReader::new(file));
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;
    let bytes = &buf[..info.buffer_size()];
    let rgba = match (info.color_type, info.bit_depth) {
        (png::ColorType::Rgba, png::BitDepth::Eight) => bytes.to_vec(),
        (png::ColorType::Rgb, png::BitDepth::Eight) => {
            let mut rgba = Vec::with_capacity((info.width * info.height * 4) as usize);
            for rgb in bytes.chunks_exact(3) {
                rgba.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 0xff]);
            }
            rgba
        }
        (color, depth) => {
            eprintln!(
                "{}: unsupported PNG format {color:?}/{depth:?}",
                path.display()
            );
            return None;
        }
    };
    Some((rgba, info.width, info.height))
}

#[cfg(test)]
mod tests {
    use super::{
        decode_rgba_png, write_argb_frame_png, write_assets_index_png, write_rgba_frame_png,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_png(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "zelda3-rs-image-output-{}-{unique}-{name}.png",
            std::process::id()
        ))
    }

    #[test]
    fn writes_and_decodes_rgba_png() {
        let path = temp_png("rgba");
        let rgba = vec![1, 2, 3, 255, 4, 5, 6, 128];

        write_rgba_frame_png(&path, &rgba, 2, 1).unwrap();
        let decoded = decode_rgba_png(&path).unwrap();
        let _ = fs::remove_file(&path);

        assert_eq!(decoded, (rgba, 2, 1));
    }

    #[test]
    fn writes_argb_frame_as_rgba_png() {
        let path = temp_png("argb");
        let argb = vec![10, 20, 30, 255, 40, 50, 60, 255];

        write_argb_frame_png(&path, &argb, 2, 1).unwrap();
        let decoded = decode_rgba_png(&path).unwrap();
        let _ = fs::remove_file(&path);

        assert_eq!(decoded, (vec![30, 20, 10, 255, 60, 50, 40, 255], 2, 1));
    }

    #[test]
    fn writes_assets_index_png_grid_dimensions() {
        let path = temp_png("assets-index");
        let cells = vec![0u8; 129 * 64];

        write_assets_index_png(path.to_str().unwrap(), &cells, 129).unwrap();
        let file = fs::File::open(&path).unwrap();
        let decoder = png::Decoder::new(std::io::BufReader::new(file));
        let reader = decoder.read_info().unwrap();
        let info = reader.info();
        let _ = fs::remove_file(&path);

        assert_eq!((info.width, info.height), (1024, 16));
        assert_eq!(info.color_type, png::ColorType::Indexed);
    }
}
