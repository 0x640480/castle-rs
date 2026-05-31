//! Time-token construction and the nibble-level hex XOR / rotate helpers.
//!
//! These operate on hex *strings* one nibble (one hex character) at a time —
//! never byte-level — because the token format XORs odd-length hex runs (e.g. the
//! 7-char `time_hex[1..]`). Converting to byte-level XOR would corrupt those.

use crate::codec::n_hex;

/// Formats `value` as exactly `n` lowercase hex digits, masked to `n*4` bits.
pub fn n_digit_hex(value: i64, n: usize) -> String {
    n_hex(value as u64, n)
}

/// XORs each hex nibble in `data` against the corresponding nibble in `key`
/// (cycling), returning lowercase hex. An empty key returns `data` unchanged.
pub fn xor_stream_hex(data: &str, key: &str) -> String {
    if key.is_empty() {
        return data.to_string();
    }
    let key = key.as_bytes();
    let mut out = String::with_capacity(data.len());
    for (i, c) in data.bytes().enumerate() {
        let d = (c as char).to_digit(16).unwrap_or(0);
        let k = (key[i % key.len()] as char).to_digit(16).unwrap_or(0);
        out.push(char::from_digit(d ^ k, 16).unwrap());
    }
    out
}

/// Returns `s[n%len..] + s[..n%len]`. An empty string returns empty.
pub fn rotate_string(s: &str, n: i64) -> String {
    if s.is_empty() {
        return String::new();
    }
    let len = s.len() as i64;
    let mut shift = n % len;
    if shift < 0 {
        shift += len;
    }
    let shift = shift as usize;
    format!("{}{}", &s[shift..], &s[..shift])
}

/// XORs `data` against the first `key_length` chars of `key`, rotated by the
/// integer value of the single hex digit `rotate_n_hex`.
pub fn xor_with_rotated_key(
    data: &str,
    key: &str,
    key_length: usize,
    rotate_n_hex: &str,
) -> String {
    let key_part = if key.len() > key_length {
        &key[..key_length]
    } else {
        key
    };
    let n = i64::from(u8::from_str_radix(rotate_n_hex, 16).unwrap_or(0));
    xor_stream_hex(data, &rotate_string(key_part, n))
}

/// Encodes `(t/1000 - 1535000000)` as 8 big-endian hex chars. The offset is the
/// epoch baseline (Aug 23, 2018 UTC).
fn encode_time_hex(t: i64) -> String {
    let time_diff = (t / 1000 - 1_535_000_000) as u32;
    format!("{time_diff:08x}")
}

/// Builds a 12-hex time token; used for both `time_token_v0` (per-mint clock)
/// and `time_token_v1` (cached init time). `random_value` is a nibble (0–15).
pub fn make_time_token(t: i64, random_value: u32) -> String {
    let random_hex = format!("{random_value:x}");
    let time_hex = encode_time_hex(t);
    let mod1000_hex = n_digit_hex(t % 1000, 4);
    let part1 = xor_stream_hex(&time_hex[1..], &random_hex);
    let part2 = xor_stream_hex(&mod1000_hex, &random_hex);
    format!("{part1}{random_hex}{}{random_hex}", &part2[1..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn n_digit_hex_cases() {
        assert_eq!(n_digit_hex(0xab, 2), "ab");
        assert_eq!(n_digit_hex(0xab, 4), "00ab");
        assert_eq!(n_digit_hex(0x12345, 4), "2345");
        assert_eq!(n_digit_hex(0, 4), "0000");
        assert_eq!(n_digit_hex(0xff, 2), "ff");
    }

    #[test]
    fn rotate_string_cases() {
        assert_eq!(rotate_string("abcdef", 2), "cdefab");
        assert_eq!(rotate_string("abcd", 5), "bcda");
        assert_eq!(rotate_string("", 3), "");
    }

    #[test]
    fn xor_stream_hex_cases() {
        assert_eq!(xor_stream_hex("a", "b"), "1");
        assert_eq!(xor_stream_hex("aaaa", "12"), "b8b8");
    }

    #[test]
    fn time_token_is_12_chars() {
        assert_eq!(make_time_token(1_700_000_000_000, 7).len(), 12);
    }
}
