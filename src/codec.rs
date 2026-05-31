//! Encoding helpers shared by the encoders: base64url, MurmurHash3 x86_32,
//! and the `n_hex` lowercase-hex formatter.

use base64::Engine;

/// Formats `v` as exactly `n` lowercase hex digits, zero-padded.
///
/// For `n < 16` the value is first masked to `n*4` bits. For `n >= 16` no mask
/// is applied: `1u64 << 64` would panic, so the special case is load-bearing,
/// not an optimization.
pub fn n_hex(v: u64, n: usize) -> String {
    if n >= 16 {
        format!("{v:0n$x}")
    } else {
        let mask = (1u64 << (4 * n)) - 1;
        format!("{:0n$x}", v & mask)
    }
}

/// Base64url with `=` padding stripped (the unpadded URL-safe alphabet).
pub fn b64url_encode(data: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(data)
}

/// Base64url decode tolerant of missing padding.
pub fn b64url_decode(s: &str) -> Result<Vec<u8>, base64::DecodeError> {
    // URL_SAFE_NO_PAD decodes unpadded input; it also tolerates input that was
    // padded once the padding is stripped, so normalize by removing any '='.
    let trimmed = s.trim_end_matches('=');
    base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(trimmed)
}

/// MurmurHash3 x86_32 of `data` with seed 0, returned as a signed `i32`
/// (Castle's convention).
///
/// Used by the fingerprint encoder to hash MIME-type, plugin, and
/// navigator-property sets. Delegates to the `murmurhash3` crate — seed-0
/// MurmurHash3 x86_32 is a single well-defined function, so the output is
/// byte-identical to the canonical algorithm (gated by the fingerprint golden tests).
pub fn mmh3_hash(data: &[u8]) -> i32 {
    murmurhash3::murmurhash3_x86_32(data, 0) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn n_hex_basics() {
        assert_eq!(n_hex(0xab, 2), "ab");
        assert_eq!(n_hex(0xab, 4), "00ab");
        assert_eq!(n_hex(0x12345, 4), "2345"); // truncated by mask
        assert_eq!(n_hex(0, 4), "0000");
    }

    #[test]
    fn mmh3_empty_is_zero() {
        assert_eq!(mmh3_hash(&[]), 0);
    }

    #[test]
    fn b64url_known_vector() {
        let enc = b64url_encode(&[0x00, 0xff, 0xfe]);
        assert_eq!(enc, "AP_-");
        let dec = b64url_decode("AP_-").unwrap();
        assert_eq!(dec, vec![0x00, 0xff, 0xfe]);
    }
}
