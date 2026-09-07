//! Helpers shared by the fixture-driven unit tests.

/// Decode the base64 payloads carried by the Snes9x JSONL fixtures.
pub(crate) fn decode_base64_bytes(encoded: &str) -> Vec<u8> {
    assert_eq!(encoded.len() % 4, 0);
    let value = |byte: u8| match byte {
        b'A'..=b'Z' => byte - b'A',
        b'a'..=b'z' => byte - b'a' + 26,
        b'0'..=b'9' => byte - b'0' + 52,
        b'+' => 62,
        b'/' => 63,
        b'=' => 0,
        _ => panic!("invalid fixture base64 digit"),
    };
    let mut decoded = Vec::with_capacity(encoded.len() / 4 * 3);
    for chunk in encoded.as_bytes().chunks_exact(4) {
        let bits = u32::from(value(chunk[0])) << 18
            | u32::from(value(chunk[1])) << 12
            | u32::from(value(chunk[2])) << 6
            | u32::from(value(chunk[3]));
        decoded.push((bits >> 16) as u8);
        if chunk[2] != b'=' {
            decoded.push((bits >> 8) as u8);
        }
        if chunk[3] != b'=' {
            decoded.push(bits as u8);
        }
    }
    decoded
}
