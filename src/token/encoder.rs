//! The outer token encoder: XOR layers, XXTEA, framing, and base64url.

use super::time_token::{n_digit_hex, xor_stream_hex, xor_with_rotated_key};
use crate::codec;
use crate::error::{Error, Result};
use crate::xxtea;

/// Pre-prepared fields for [`encode_token`].
pub struct EncodeTokenInput<'a> {
    /// Random mask byte, 0..=255.
    pub random_byte: u32,
    /// 12 hex chars — time token for the init timestamp.
    pub time_token_v1: &'a str,
    /// 4 hex chars — `(num_parts << 11) | ig`.
    pub one_field: &'a str,
    /// 64 hex chars — encoded public key.
    pub pk_encoded: &'a str,
    /// 32 hex chars — the `__cuid` cookie value.
    pub cuid: &'a str,
    /// 12 hex chars — time token used as the inner-XOR key.
    pub time_token_v0: &'a str,
    /// `fp_lists + ce_len + ce + events + 'ff'`.
    pub inner_payload_plain: &'a str,
}

/// Assembles the `X-CRS-Req-Token` from already-prepared fields:
///
/// ```text
/// xor1  = v0 + xor_with_rotated(inner, v0[:4], v0[3])
/// outer = xor_with_rotated(xor1, cuid[:8], cuid[9])
/// v3    = v1 + one_field + pk_encoded + cuid + outer
/// ct    = XXTEA(v3, CASTLE_OUTER_KEY)
/// f3    = '0d' + length_delta(2) + ct.hex()
/// final = rand_byte(2) + xor_stream(f3 + len(f3), rand_byte)
/// token = b64url_no_pad(hex_decode(final))
/// ```
pub fn encode_token(input: &EncodeTokenInput) -> Result<String> {
    let v0 = input.time_token_v0;
    let inner_xored = xor_with_rotated_key(input.inner_payload_plain, v0, 4, &v0[3..4]);
    let xor1 = format!("{v0}{inner_xored}");

    if input.cuid.len() < 10 {
        return Err(Error::CuidTooShort(input.cuid.len()));
    }
    let outer = xor_with_rotated_key(&xor1, input.cuid, 8, &input.cuid[9..10]);

    let v3_hex = format!(
        "{}{}{}{}{}",
        input.time_token_v1, input.one_field, input.pk_encoded, input.cuid, outer
    );
    let v3_bytes = hex::decode(&v3_hex)?;

    let enc_u32 = xxtea::encrypt(&v3_bytes, &xxtea::CASTLE_OUTER_KEY);
    let enc_bytes = xxtea::from_u32_array(&enc_u32);
    let xxtea_hex = hex::encode(&enc_bytes);

    let length_delta = enc_bytes.len() - v3_bytes.len();
    let f3_value = format!("0d{}{xxtea_hex}", n_digit_hex(length_delta as i64, 2));

    let rand_hex = n_digit_hex(i64::from(input.random_byte), 2);
    let value_with_len = format!("{f3_value}{}", n_digit_hex(f3_value.len() as i64, 2));
    let final_hex = format!("{rand_hex}{}", xor_stream_hex(&value_with_len, &rand_hex));

    let raw = hex::decode(&final_hex)?;
    Ok(codec::b64url_encode(&raw))
}

/// Hex-encodes each char of `pk[3..]`. `pk` must be `pk_` followed by 32 chars
/// (35 total). Returns 64 hex chars.
pub fn encode_pk(pk: &str) -> Result<String> {
    if pk.len() != 35 || !pk.starts_with("pk_") {
        return Err(Error::InvalidPk(pk.len()));
    }
    Ok(hex::encode(&pk.as_bytes()[3..]))
}

/// Encodes `((num_parts << 11) | ig)` as 4 hex chars.
pub fn encode_one_field(num_parts: i64, ig: i64) -> String {
    let val = (num_parts << 11) | ig;
    format!("{:04x}", val as u16)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    const TEST_PK: &str = "pk_xPQ5kRvjnzuTy24zZtig3eNMzspdJS92";
    const TEST_IG: i64 = 225;

    #[test]
    fn encode_token_against_reference() {
        let inner_plain = format!("00000002aabb{}ff", "00".repeat(198));
        let pk_encoded = encode_pk(TEST_PK).unwrap();
        let one_field = encode_one_field(5, TEST_IG);

        let got = encode_token(&EncodeTokenInput {
            random_byte: 0x42,
            time_token_v1: "aabbccddeeff",
            one_field: &one_field,
            pk_encoded: &pk_encoded,
            cuid: "00112233445566778899aabbccddeeff",
            time_token_v0: "102030405060",
            inner_payload_plain: &inner_plain,
        })
        .unwrap();

        assert_eq!(got.len(), 363);
        let sum = Sha256::digest(got.as_bytes());
        assert_eq!(
            hex::encode(sum),
            "9f43ea555bc670d43fb89839db780c254b31baaee255cdff8e56420f705385ee"
        );
    }

    #[test]
    fn encode_one_field_value() {
        assert_eq!(encode_one_field(5, TEST_IG), "28e1");
    }

    #[test]
    fn encode_pk_value() {
        let got = encode_pk(TEST_PK).unwrap();
        assert_eq!(got, hex::encode("xPQ5kRvjnzuTy24zZtig3eNMzspdJS92"));
        assert_eq!(got.len(), 64);
    }

    #[test]
    fn encode_pk_rejects_bad_input() {
        assert!(encode_pk("nope").is_err());
        assert!(encode_pk("xx_xPQ5kRvjnzuTy24zZtig3eNMzspdJS92").is_err());
    }
}
