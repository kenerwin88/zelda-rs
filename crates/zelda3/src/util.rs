//! Utility ports from `src/util.c`.

#![allow(non_camel_case_types, non_snake_case)]

use std::fmt;
use std::fs;
use std::path::Path;

use crate::types::MemBlk;

#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ByteArray {
    pub data: Vec<u8>,
}

impl ByteArray {
    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }
}

pub fn NextDelim<'a>(s: &mut Option<&'a str>, sep: char) -> Option<&'a str> {
    let r = s.take()?;
    let r = r.trim_start_matches([' ', '\t']);
    if let Some((left, right)) = r.split_once(sep) {
        *s = Some(right);
        Some(left)
    } else {
        *s = None;
        Some(r)
    }
}

fn ToLower(a: u8) -> u8 {
    a + ((a >= b'A' && a <= b'Z') as u8) * 32
}

pub fn StringEqualsNoCase(a: &str, b: &str) -> bool {
    let mut a = a.bytes();
    let mut b = b.bytes();
    loop {
        let aa = a.next().map(ToLower).unwrap_or(0);
        let bb = b.next().map(ToLower).unwrap_or(0);
        if aa != bb {
            return false;
        }
        if aa == 0 {
            return true;
        }
    }
}

pub fn StringStartsWithNoCase<'a>(a: &'a str, b: &str) -> Option<&'a str> {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let mut i = 0;
    loop {
        if i == b_bytes.len() {
            return a.get(i..);
        }
        if i == a_bytes.len() || ToLower(a_bytes[i]) != ToLower(b_bytes[i]) {
            return None;
        }
        i += 1;
    }
}

pub fn ReadWholeFile(name: impl AsRef<Path>) -> Option<(Vec<u8>, usize)> {
    let mut buffer = fs::read(name).ok()?;
    let length = buffer.len();
    buffer.push(0);
    Some((buffer, length))
}

pub fn NextLineStripComments<'a>(s: &mut Option<&'a str>) -> Option<&'a str> {
    let p = s.take()?;
    let (line, rest) = if let Some((line, rest)) = p.split_once('\n') {
        (line, Some(rest))
    } else {
        (p, None)
    };
    *s = rest;

    let line = line.split_once('#').map_or(line, |(line, _)| line);
    Some(
        line.trim_end_matches(['\r', ' ', '\t'])
            .trim_start_matches([' ', '\t']),
    )
}

pub fn NextPossiblyQuotedString<'a>(s: &mut &'a str) -> &'a str {
    let r = s.trim_start_matches([' ', '\t']);
    let (result, mut rest) = if let Some(quoted) = r.strip_prefix('"') {
        if let Some((value, tail)) = quoted.split_once('"') {
            (value, tail)
        } else {
            (quoted, "")
        }
    } else {
        let split = r.find([' ', '\t']).map_or(r.len(), |split| split);
        (&r[..split], &r[split..])
    };
    rest = rest.trim_start_matches([' ', '\t']);
    *s = rest;
    result
}

pub fn ReplaceFilenameWithNewPath(old_path: &str, new_path: &str) -> String {
    let split = old_path.rfind(['/', '\\']).map_or(0, |index| index + 1);
    let mut result = String::with_capacity(split + new_path.len());
    result.push_str(&old_path[..split]);
    result.push_str(new_path);
    result
}

pub fn SplitKeyValue(p: &str) -> Option<(&str, &str)> {
    let (key, value) = p.split_once('=')?;
    let key = key.trim_end_matches([' ', '\t']);
    let value = value.trim_start_matches([' ', '\t']);
    Some((key, value))
}

pub fn SkipPrefix<'a>(big: &'a str, little: &str) -> Option<&'a str> {
    big.strip_prefix(little)
}

pub fn StrSet(rv: &mut String, s: &str) {
    rv.clear();
    rv.push_str(s);
}

pub fn StrFmt(args: fmt::Arguments<'_>) -> String {
    let formatted = args.to_string();
    if formatted.len() >= 4096 {
        panic!("vsnprintf failed");
    }
    formatted
}

pub fn ByteArray_Resize(arr: &mut ByteArray, new_size: usize) {
    if new_size > arr.data.capacity() {
        let minsize = arr.data.capacity() + (arr.data.capacity() >> 1) + 8;
        let capacity = if new_size < minsize {
            minsize
        } else {
            new_size
        };
        arr.data.reserve_exact(capacity - arr.data.capacity());
    }
    arr.data.resize(new_size, 0);
}

pub fn ByteArray_Destroy(arr: &mut ByteArray) {
    arr.data.clear();
    arr.data.shrink_to_fit();
}

pub fn ByteArray_AppendData(arr: &mut ByteArray, data: &[u8]) {
    let old_size = arr.data.len();
    ByteArray_Resize(arr, old_size + data.len());
    arr.data[old_size..].copy_from_slice(data);
}

pub fn ByteArray_AppendByte(arr: &mut ByteArray, v: u8) {
    let old_size = arr.data.len();
    ByteArray_Resize(arr, old_size + 1);
    arr.data[old_size] = v;
}

pub fn FindIndexInMemblk(data: MemBlk<'_>, i: usize) -> MemBlk<'_> {
    let bytes = data.ptr;
    if bytes.len() < 2 {
        return MemBlk { ptr: &[] };
    }

    let end = bytes.len() - 2;
    let mut mx = read_u16(bytes, end) as usize;
    let (left_off, right_off) = if mx < 8192 {
        if i > mx || mx * 2 > end {
            return MemBlk { ptr: &[] };
        }
        let left = if i == 0 {
            mx * 2
        } else {
            mx * 2 + read_u16(bytes, i * 2 - 2) as usize
        };
        let right = if i == mx {
            end
        } else {
            mx * 2 + read_u16(bytes, i * 2) as usize
        };
        (left, right)
    } else {
        mx -= 8192;
        if i > mx || mx * 4 > end {
            return MemBlk { ptr: &[] };
        }
        let left = if i == 0 {
            mx * 4
        } else {
            mx * 4 + read_u32(bytes, i * 4 - 4) as usize
        };
        let right = if i == mx {
            end
        } else {
            mx * 4 + read_u32(bytes, i * 4) as usize
        };
        (left, right)
    };

    if left_off > right_off || right_off > end {
        return MemBlk { ptr: &[] };
    }
    MemBlk {
        ptr: &bytes[left_off..right_off],
    }
}

fn BpsDecodeInt(src: &mut &[u8]) -> Option<u64> {
    let mut data = 0u64;
    let mut shift = 1u64;
    loop {
        let (&x, rest) = src.split_first()?;
        *src = rest;
        data = data.wrapping_add(((x & 0x7f) as u64).wrapping_mul(shift));
        if x & 0x80 != 0 {
            break;
        }
        shift <<= 7;
        data = data.wrapping_add(shift);
    }
    Some(data)
}

const CRC32_POLYNOMIAL: u32 = 0xedb88320;

fn crc32_impl(data: &[u8]) -> u32 {
    let mut crc = 0xffffffffu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            crc = (crc >> 1) ^ ((crc & 1).wrapping_mul(CRC32_POLYNOMIAL));
        }
    }
    crc ^ 0xffffffff
}

pub fn ApplyBps(src: &[u8], bps: &[u8]) -> Option<Vec<u8>> {
    if bps.len() < 16 {
        return None;
    }
    let bps_end = bps.len() - 12;

    if bps.get(..4) != Some(b"BPS1") {
        return None;
    }
    if crc32_impl(src) != read_u32(bps, bps_end) {
        return None;
    }
    if crc32_impl(&bps[..bps.len() - 4]) != read_u32(bps, bps_end + 8) {
        return None;
    }

    let mut stream = &bps[4..bps_end];
    let src_size = BpsDecodeInt(&mut stream)? as usize;
    let dst_size = BpsDecodeInt(&mut stream)? as usize;
    let _meta_size = BpsDecodeInt(&mut stream)? as usize;

    let mut output_offset = 0usize;
    let mut source_relative_offset = 0isize;
    let mut target_relative_offset = 0isize;
    if src_size != src.len() {
        return None;
    }
    let mut dst = vec![0; dst_size];

    while !stream.is_empty() {
        let mut cmd = BpsDecodeInt(&mut stream)?;
        let mut length = (cmd >> 2) as usize + 1;
        match cmd & 3 {
            0 => {
                while length != 0 {
                    if output_offset >= src.len() || output_offset >= dst.len() {
                        return None;
                    }
                    dst[output_offset] = src[output_offset];
                    output_offset += 1;
                    length -= 1;
                }
            }
            1 => {
                if stream.len() < length || output_offset + length > dst.len() {
                    return None;
                }
                let end = output_offset + length;
                dst[output_offset..end].copy_from_slice(&stream[..length]);
                stream = &stream[length..];
                output_offset = end;
            }
            2 => {
                cmd = BpsDecodeInt(&mut stream)?;
                let delta = (cmd >> 1) as isize;
                source_relative_offset += if cmd & 1 != 0 { -delta } else { delta };
                while length != 0 {
                    let source = source_relative_offset as usize;
                    if source >= src.len() || output_offset >= dst.len() {
                        return None;
                    }
                    dst[output_offset] = src[source];
                    source_relative_offset += 1;
                    output_offset += 1;
                    length -= 1;
                }
            }
            _ => {
                cmd = BpsDecodeInt(&mut stream)?;
                let delta = (cmd >> 1) as isize;
                target_relative_offset += if cmd & 1 != 0 { -delta } else { delta };
                while length != 0 {
                    let target = target_relative_offset as usize;
                    if target >= dst.len() || output_offset >= dst.len() {
                        return None;
                    }
                    dst[output_offset] = dst[target];
                    target_relative_offset += 1;
                    output_offset += 1;
                    length -= 1;
                }
            }
        }
    }

    if dst_size != output_offset {
        return None;
    }
    if crc32_impl(&dst) != read_u32(bps, bps_end + 4) {
        return None;
    }
    Some(dst)
}

pub fn find_index_in_memblk(data: MemBlk<'_>, i: usize) -> MemBlk<'_> {
    FindIndexInMemblk(data, i)
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bps_encode_int(mut data: u64, out: &mut Vec<u8>) {
        loop {
            let x = data & 0x7f;
            data >>= 7;
            if data == 0 {
                out.push((x | 0x80) as u8);
                break;
            }
            out.push(x as u8);
            data -= 1;
        }
    }

    fn source_read_bps(src: &[u8]) -> Vec<u8> {
        let mut bps = Vec::new();
        bps.extend_from_slice(b"BPS1");
        bps_encode_int(src.len() as u64, &mut bps);
        bps_encode_int(src.len() as u64, &mut bps);
        bps_encode_int(0, &mut bps);
        bps_encode_int(((src.len() as u64 - 1) << 2) | 0, &mut bps);
        bps.extend_from_slice(&crc32_impl(src).to_le_bytes());
        bps.extend_from_slice(&crc32_impl(src).to_le_bytes());
        let patch_crc = crc32_impl(&bps);
        bps.extend_from_slice(&patch_crc.to_le_bytes());
        bps
    }

    #[test]
    fn string_helpers_match_c_shape() {
        assert!(StringEqualsNoCase("Zelda", "zelDA"));
        assert!(!StringEqualsNoCase("Zelda", "zelda3"));
        assert_eq!(
            StringStartsWithNoCase("Dialogue.en", "dialogue"),
            Some(".en")
        );
        assert_eq!(SkipPrefix("abc", "ab"), Some("c"));
        assert_eq!(SkipPrefix("abc", "ac"), None);
    }

    #[test]
    fn tokenizers_match_c_shape() {
        let mut comma = Some(" \talpha,beta");
        assert_eq!(NextDelim(&mut comma, ','), Some("alpha"));
        assert_eq!(comma, Some("beta"));
        assert_eq!(NextDelim(&mut comma, ','), Some("beta"));
        assert_eq!(comma, None);

        let mut lines = Some("  key = value \t# comment\r\n next");
        assert_eq!(NextLineStripComments(&mut lines), Some("key = value"));
        assert_eq!(lines, Some(" next"));

        let mut words = " \t\"two words\" tail";
        assert_eq!(NextPossiblyQuotedString(&mut words), "two words");
        assert_eq!(words, "tail");
        assert_eq!(SplitKeyValue("key \t= \tvalue"), Some(("key", "value")));
        assert_eq!(
            ReplaceFilenameWithNewPath("dir\\old.sfc", "new.sfc"),
            "dir\\new.sfc"
        );
    }

    #[test]
    fn read_whole_file_zero_terminates_like_c() {
        let path = std::env::temp_dir().join(format!("zelda3-rs-util-{}", std::process::id()));
        std::fs::write(&path, b"abc").unwrap();
        let (data, length) = ReadWholeFile(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        assert_eq!(length, 3);
        assert_eq!(data, b"abc\0");
    }

    #[test]
    fn byte_array_helpers_match_c_growth() {
        let mut arr = ByteArray::default();
        ByteArray_AppendByte(&mut arr, 1);
        ByteArray_AppendData(&mut arr, &[2, 3]);
        assert_eq!(arr.data, [1, 2, 3]);
        assert!(arr.capacity() >= 8);
        ByteArray_Resize(&mut arr, 1);
        assert_eq!(arr.data, [1]);
        ByteArray_Destroy(&mut arr);
        assert_eq!(arr.size(), 0);
    }

    #[test]
    fn find_index_in_16_bit_memblk() {
        let mut data = Vec::new();
        data.extend_from_slice(&[1, 0, 3, 0]);
        data.extend_from_slice(b"abbccc");
        data.extend_from_slice(&2u16.to_le_bytes());

        let first = FindIndexInMemblk(MemBlk { ptr: &data }, 0);
        let second = FindIndexInMemblk(MemBlk { ptr: &data }, 1);
        let third = FindIndexInMemblk(MemBlk { ptr: &data }, 2);

        assert_eq!(first.ptr, b"a");
        assert_eq!(second.ptr, b"bb");
        assert_eq!(third.ptr, b"ccc");
    }

    #[test]
    fn bps_decode_source_read_patch() {
        let src = b"zelda";
        let bps = source_read_bps(src);
        assert_eq!(ApplyBps(src, &bps).as_deref(), Some(&src[..]));
        assert_eq!(ApplyBps(b"wrong", &bps), None);
    }

    #[test]
    fn bps_int_and_crc32_match_known_values() {
        let mut stream = [0x80].as_slice();
        assert_eq!(BpsDecodeInt(&mut stream), Some(0));
        assert_eq!(crc32_impl(b"123456789"), 0xcbf43926);
    }
}
