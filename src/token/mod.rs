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
use crate::fingerprint::{Fingerprint, LocaleProfile};

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
    /// Override `window.location.hostname` (the site the token is for).
    /// `None` keeps the fingerprint's value.
    pub hostname: Option<&'a str>,
    /// Override the geo/locale profile. `None` keeps the fingerprint's values.
    pub locale_profile: Option<&'a LocaleProfile>,
    /// When `true`, jitter the per-session timing/behavioral signals so each
    /// minted token differs (see [`Fingerprint::jittered`]). `false` reproduces
    /// the fingerprint verbatim.
    pub jitter: bool,
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

    // Build the effective fingerprint: caller context (deterministic) then the
    // jitter pass (the first RNG draws). With no context and `jitter` off this
    // is a verbatim clone, so the output is byte-identical to the bundled fp.
    let mut fp = opts.fingerprint.clone();
    if let Some(profile) = opts.locale_profile {
        fp = fp.with_locale_profile(profile);
    }
    if let Some(hostname) = opts.hostname {
        fp = fp.with_hostname(hostname);
    }
    if opts.jitter {
        fp = fp.jittered(rng);
    }

    let fp_lists_hex = fp.encode_fp(init_time, utc_minutes);
    let ce_hex = fp.encode_ce()?;
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
                hostname: None,
                locale_profile: None,
                jitter: false,
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

    fn opts<'a>(fp: &'a Fingerprint, hostname: Option<&'a str>, jitter: bool) -> MintOptions<'a> {
        MintOptions {
            cuid: "00112233445566778899aabbccddeeff",
            fingerprint: fp,
            init_time_ms: Some(1_700_000_000_000),
            pk: "pk_xPQ5kRvjnzuTy24zZtig3eNMzspdJS92",
            ig: 225,
            now_ms: Some(1_700_000_999_000),
            hostname,
            locale_profile: None,
            jitter,
        }
    }

    #[test]
    fn context_and_jitter_change_the_token() {
        let fp = chrome_148_macos();

        // Hostname override changes the token (and the no-op default matches).
        let plain = mint_fresh(&opts(fp, None, false), &mut StdRng::seed_from_u64(1)).unwrap();
        let plain2 = mint_fresh(&opts(fp, None, false), &mut StdRng::seed_from_u64(1)).unwrap();
        assert_eq!(plain, plain2, "no-context mint is deterministic for a seed");
        let other_site =
            mint_fresh(&opts(fp, Some("login.example.com"), false), &mut StdRng::seed_from_u64(1))
                .unwrap();
        assert_ne!(plain, other_site, "hostname override must change the token");

        // Jitter varies output across seeds but is reproducible per seed.
        let j1 = mint_fresh(&opts(fp, None, true), &mut StdRng::seed_from_u64(1)).unwrap();
        let j1_again = mint_fresh(&opts(fp, None, true), &mut StdRng::seed_from_u64(1)).unwrap();
        let j2 = mint_fresh(&opts(fp, None, true), &mut StdRng::seed_from_u64(2)).unwrap();
        assert_eq!(j1, j1_again, "same seed → same jittered token");
        assert_ne!(j1, j2, "different seed → different jittered token");
        assert_ne!(plain, j1, "jitter must change the token vs. the verbatim mint");
    }
}
