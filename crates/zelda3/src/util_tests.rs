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
