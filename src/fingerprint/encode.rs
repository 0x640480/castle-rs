//! Low-level fp_lists encoding primitives: slot tags, the seven encoding
//! "cases", bit/hash packers, the per-slot XXTEA frame, and list framing.

use crate::xxtea;

pub use crate::codec::n_hex;

// Per-slot XXTEA key-derivation constants. The per-slot key is
// `[slot_index, init_time_ms & 0xFFFFFFFF, tr0, tr1]`.
const TR0: u32 = 4_219_861_129;
const TR1: u32 = 2_113_960_264;

/// Wire IDs for the five fp_lists parts, in encoded order.
pub const PART_IDS: [i64; 5] = [0, 4, 7, 8, 9];

fn jr_key(index: u32, init_time_ms: i64) -> [u32; 4] {
    [index, init_time_ms as u32, TR0, TR1]
}

/// 1-byte slot+case tag as 2 hex chars: `(index & 31) << 3 | (case & 7)`.
pub fn tag(index: i64, field_case: i64) -> String {
    let v = ((index as u64 & 31) << 3) | (field_case as u64 & 7);
    n_hex(v, 2)
}

/// Case 3: a 1-byte payload (small int / bitmask).
pub fn case3(index: i64, input: i64) -> String {
    tag(index, 3) + &n_hex(input as u64, 2)
}

/// Case 4: XXTEA-encrypt `input` under the per-slot key, length-prefixed.
pub fn case4(index: i64, input: &str, init_time_ms: i64) -> String {
    let enc = time_index_encrypt(index as u32, input, init_time_ms);
    tag(index, 4) + &n_hex(enc.len() as u64, 2) + &hex::encode(&enc)
}

/// Case 5: a 1-byte int (`<= 127`) or a 2-byte varint (high bit marks 2 bytes).
pub fn case5(index: i64, input: i64) -> String {
    let v = input & 0x7FFF;
    if v > 127 {
        tag(index, 5) + &n_hex(((1 << 15) | v) as u64, 4)
    } else {
        tag(index, 5) + &n_hex(input as u64 & 0xFF, 2)
    }
}

/// Case 6: a 1-byte fixed-point value, `round(input * 10)`.
pub fn case6(index: i64, input: f64) -> String {
    let v = round_scaled(input, 10.0);
    tag(index, 6) + &n_hex(v as u64, 2)
}

/// Case 7: a pre-encoded hex payload (caller owns the shape).
pub fn case7(index: i64, hex_payload: &str) -> String {
    tag(index, 7) + hex_payload
}

/// Cases 1 and 2: tag only, no payload.
pub fn case_empty(index: i64, field_case: i64) -> String {
    tag(index, field_case)
}

/// Index of `value` in `arr`, or `-1` if absent.
pub fn index_of(arr: &[&str], value: &str) -> i64 {
    arr.iter()
        .position(|&v| v == value)
        .map_or(-1, |i| i as i64)
}

/// Case 3 on a LUT hit (`lut_index >= 0`), else Case 4 on the literal.
pub fn encode_optional_index(index: i64, lut_index: i64, value: &str, init_time_ms: i64) -> String {
    if lut_index == -1 {
        case4(index, value, init_time_ms)
    } else {
        case3(index, lut_index)
    }
}

/// Packs bools into hex: a 1-byte length prefix (`count & 0xFF`) followed by
/// 24-bit MSB-first chunks (the final partial chunk sized by `ceil(bits/8)`).
pub fn bits_to_hex(bits: &[bool]) -> String {
    let mut out = n_hex((bits.len() & 0xFF) as u64, 2);
    const CHUNK: usize = 24;
    let mut i = 0;
    while i < bits.len() {
        let end = (i + CHUNK).min(bits.len());
        let mut v: u64 = 0;
        for &b in &bits[i..end] {
            v = (v << 1) | u64::from(b);
        }
        let num_bytes = (end - i).div_ceil(8);
        out += &n_hex(v, num_bytes * 2);
        i += CHUNK;
    }
    out
}

/// Packs two ints as a 4-hex compressed form when equal (high bit set) or as
/// two 4-hex values otherwise.
pub fn pack15_16(v1: i64, v2: i64) -> String {
    let a = v1 & 0x7FFF;
    let b = v2 & 0xFFFF;
    if a == b {
        n_hex((a | 0x8000) as u64, 4)
    } else {
        n_hex(a as u64, 4) + &n_hex(b as u64, 4)
    }
}

/// Length-prefixed XXTEA frame: `byte_len_of(len)(1B) + len(1B) + ciphertext`.
pub fn encode_xxtea_frame(index: i64, data: &str, init_time_ms: i64) -> String {
    let enc = time_index_encrypt(index as u32, data, init_time_ms);
    let byte_len = byte_length_of(enc.len() as u64);
    n_hex(byte_len as u64, 2) + &n_hex(enc.len() as u64, 2) + &hex::encode(&enc)
}

/// Hashes the sorted, joined string set and returns `count(1B) + hash(4B)`.
pub fn mmh3_string_set_hex(strs: &[String]) -> String {
    let mut sorted: Vec<&str> = strs.iter().map(String::as_str).collect();
    sorted.sort_unstable();
    let joined: String = sorted.concat();
    let h = crate::codec::mmh3_hash(joined.as_bytes());
    n_hex(strs.len() as u64, 2) + &n_hex(h as u32 as u64, 8)
}

/// Emits a length-prefix then `n` 12-hex fixed-point entries
/// (`"0000" + round(v * 1000)` as 8 hex). Used by slots 7/12 and 7/16.
pub fn arr_12_dig_hex_seq(values: &[f64]) -> String {
    let mut out = n_hex(values.len() as u64, 2);
    for &v in values {
        let mut scaled = round_scaled(v, 1000.0);
        if scaled < 0 {
            scaled = 0;
        }
        out += "0000";
        out += &n_hex(scaled as u64, 8);
    }
    out
}

/// Wraps each part with `((wire_id * 2) & 0xFFF) << 4 | (count & 0x1F)` and
/// concatenates entries verbatim.
pub fn encode_lists(parts: &[Vec<String>; 5]) -> String {
    let mut out = String::new();
    for (i, list) in parts.iter().enumerate() {
        let header = (((PART_IDS[i] * 2) & 0xFFF) as u64) << 4 | (list.len() as u64 & 0x1F);
        out += &n_hex(header, 4);
        for item in list {
            out += item;
        }
    }
    out
}

/// `round(x * scale)` with away-from-zero rounding, then truncation toward zero.
pub fn round_scaled(x: f64, scale: f64) -> i64 {
    if x < 0.0 {
        (x * scale - 0.5) as i64
    } else {
        (x * scale + 0.5) as i64
    }
}

fn time_index_encrypt(index: u32, input: &str, init_time_ms: i64) -> Vec<u8> {
    let enc = xxtea::encrypt(input.as_bytes(), &jr_key(index, init_time_ms));
    xxtea::from_u32_array(&enc)
}

fn byte_length_of(mut n: u64) -> usize {
    let mut out = 0;
    while n != 0 {
        n >>= 8;
        out += 1;
    }
    out
}
