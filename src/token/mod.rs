//! Minting the `X-CRS-Req-Token` and its building blocks.

mod encoder;
mod inner_payload;
mod time_token;

pub use encoder::{encode_one_field, encode_pk, encode_token, EncodeTokenInput};
pub use inner_payload::{assemble_inner_payload, E7_TOTAL_LEN_HEX};
pub use time_token::{
    make_time_token, n_digit_hex, rotate_string, xor_stream_hex, xor_with_rotated_key,
};

use std::time::{SystemTime, UNIX_EPOCH};

use rand::Rng;

use crate::error::Result;
use crate::events::{self, FreshOptions};
use crate::fingerprint::Fingerprint;

/// Inputs to [`mint_fresh`].
pub struct MintOptions<'a> {
    /// The 32-hex `__cuid` cookie value (must also live in your cookie jar).
    pub cuid: &'a str,
    /// Browser identity used to render fp_lists and the `ce` blob.
    pub fingerprint: &'a Fingerprint,
    /// Init timestamp in ms. `None` means `now - 4000`.
    pub init_time_ms: Option<i64>,
    /// Castle deployment public key, format `pk_<32 chars>`.
    pub pk: &'a str,
    /// Castle deployment integration group ID.
    pub ig: i64,
    /// Clock override in ms. `None` means the current system time.
    pub now_ms: Option<i64>,
}

/// Produces an `X-CRS-Req-Token` from a site fingerprint.
///
/// fp_lists and `ce` are rendered fresh per call from the fingerprint's typed
/// traits. Per-mint variation (the per-slot XXTEA ciphertexts keyed on `init_time_ms`,
/// time-token nibbles, the mask byte, and one e7 counter) comes from `rng`.
pub fn mint_fresh(opts: &MintOptions, rng: &mut impl Rng) -> Result<String> {
    let now = opts.now_ms.unwrap_or_else(now_ms);
    let init_time = opts.init_time_ms.unwrap_or(now - 4000);
    // Equivalent to `new Date(init_time).getUTCMinutes()` for non-negative ms.
    let utc_minutes = (init_time / 60_000) % 60;

    let fp_lists_hex = opts.fingerprint.encode_fp(init_time, utc_minutes);
    let ce_hex = opts.fingerprint.encode_ce()?;
    let e7_hex = events::fresh_events_hex(&FreshOptions::default(), rng)?;
    let inner_plain = assemble_inner_payload(&fp_lists_hex, &ce_hex, &e7_hex)?;

    let time_token_v1 = make_time_token(init_time, rng.gen_range(0u32..16));
    let time_token_v0 = make_time_token(now, rng.gen_range(0u32..16));

    let pk_encoded = encode_pk(opts.pk)?;
    let one_field = encode_one_field(5, opts.ig);

    encode_token(&EncodeTokenInput {
        random_byte: rng.gen_range(0u32..256),
        time_token_v1: &time_token_v1,
        one_field: &one_field,
        pk_encoded: &pk_encoded,
        cuid: opts.cuid,
        time_token_v0: &time_token_v0,
        inner_payload_plain: &inner_plain,
    })
}

/// [`mint_fresh`] using the thread-local RNG.
pub fn mint_fresh_default(opts: &MintOptions) -> Result<String> {
    mint_fresh(opts, &mut rand::thread_rng())
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fingerprint::chrome_148_macos;
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn mint_fresh_end_to_end() {
        let mut rng = StdRng::seed_from_u64(1);
        let tok = mint_fresh(
            &MintOptions {
                cuid: "00112233445566778899aabbccddeeff",
                fingerprint: chrome_148_macos(),
                init_time_ms: Some(1_700_000_000_000),
                pk: "pk_xPQ5kRvjnzuTy24zZtig3eNMzspdJS92",
                ig: 225,
                now_ms: Some(1_700_000_999_000),
            },
            &mut rng,
        )
        .unwrap();

        assert!(!tok.is_empty());
        assert!(
            tok.bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_'),
            "non-base64url char in token"
        );
        assert!(tok.len() >= 1000, "token suspiciously short: {}", tok.len());
    }
}
