//! End-to-end self-consistency check: mint a token, then fully reverse the
//! pipeline (base64url -> XOR -> XXTEA decrypt -> XOR layers) and confirm the
//! deterministic fields embedded in the token are exactly what went in.
//!
//! This exercises the whole stack against itself; the RNG-free `encode_token`
//! SHA-256 unit test separately pins byte-identity to the golden vector.

use castle_token::token::{
    encode_one_field, encode_pk, mint_fresh, xor_stream_hex, xor_with_rotated_key, MintOptions,
};
use castle_token::{codec, fingerprint, xxtea};
use rand::{rngs::StdRng, SeedableRng};

const CUID: &str = "00112233445566778899aabbccddeeff";
const PK: &str = "pk_xPQ5kRvjnzuTy24zZtig3eNMzspdJS92";
const IG: i64 = 225;
const INIT: i64 = 1_700_000_000_000;
const NOW: i64 = 1_700_000_999_000;

#[test]
fn minted_token_decodes_to_expected_fields() {
    let mut rng = StdRng::seed_from_u64(42);
    let fp = fingerprint::chrome_148_macos();
    let token = mint_fresh(
        &MintOptions {
            cuid: CUID,
            fingerprint: fp,
            init_time_ms: Some(INIT),
            pk: PK,
            ig: IG,
            now_ms: Some(NOW),
            hostname: None,
            locale_profile: None,
            jitter: false,
        },
        &mut rng,
    )
    .unwrap();

    // Reverse: base64url -> hex -> strip the random mask (XOR is self-inverse).
    let final_hex = hex::encode(codec::b64url_decode(&token).unwrap());
    let rand_hex = &final_hex[..2];
    let value_with_len = xor_stream_hex(&final_hex[2..], rand_hex);

    // value_with_len = f3 + len(f3); f3 = "0d" + len_delta(2) + xxtea_hex.
    let f3 = &value_with_len[..value_with_len.len() - 2];
    let len_delta = usize::from_str_radix(&f3[2..4], 16).unwrap();
    let xxtea_hex = &f3[4..];

    // Decrypt the outer XXTEA layer and drop the block padding.
    let enc_u32 = xxtea::to_u32_array(&hex::decode(xxtea_hex).unwrap());
    let v3_bytes = xxtea::from_u32_array(&xxtea::decrypt(&enc_u32, &xxtea::CASTLE_OUTER_KEY));
    let v3_hex = hex::encode(&v3_bytes[..v3_bytes.len() - len_delta]);

    // v3 = v1(12) + one_field(4) + pk(64) + cuid(32) + outer.
    let one_field = &v3_hex[12..16];
    let pk_encoded = &v3_hex[16..80];
    let cuid = &v3_hex[80..112];
    let outer = &v3_hex[112..];

    assert_eq!(cuid, CUID, "recovered cuid");
    assert_eq!(one_field, encode_one_field(5, IG), "recovered one_field");
    assert_eq!(pk_encoded, encode_pk(PK).unwrap(), "recovered pk_encoded");

    // Reverse both XOR layers to recover the inner payload.
    let xor1 = xor_with_rotated_key(outer, cuid, 8, &cuid[9..10]);
    let v0 = &xor1[..12];
    let inner_xored = &xor1[12..];
    let inner_payload_plain = xor_with_rotated_key(inner_xored, v0, 4, &v0[3..4]);

    // The inner payload begins with fp_lists; it must match encode_fp exactly.
    let utc_minutes = (INIT / 60_000) % 60;
    let fp_lists = fp.encode_fp(INIT, utc_minutes);
    assert!(
        inner_payload_plain.starts_with(&fp_lists),
        "recovered inner payload does not start with the expected fp_lists"
    );
    // ...and ends with the 'ff' terminator.
    assert!(inner_payload_plain.ends_with("ff"), "missing ff terminator");
}
