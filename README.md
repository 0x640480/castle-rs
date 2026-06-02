# castle-rs

Rust generator for the `X-Castle-Request-Token` header minted by Castle's web SDK.
The header carries an encrypted device fingerprint plus a behavioral signal
that Castle's risk engine validates server-side before letting requests through
to login / account endpoints.

The wire format is universal across Castle web customers — point the library at
a different deployment by supplying its public key (`pk_…`) and integration
group ID.

Castle's default header name is `X-Castle-Request-Token`; individual deployments
may rename it (Fanatics, for example, sends it as `X-CRS-Req-Token`) — send the
token under whatever header your target site uses.

## Quick start

```rust
use castle_token::fingerprint;
use castle_token::token::{mint_fresh_default, MintOptions};

let cuid = hex::encode(rand::random::<[u8; 16]>());
let token = mint_fresh_default(&MintOptions {
    cuid: &cuid,
    fingerprint: fingerprint::chrome_148_macos(),
    init_time_ms: None,   // None => now - 4000ms
    pk: "pk_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
    ig: 225,
    now_ms: None,            // None => current system time
    hostname: "id.fanatics.com", // required: the site the token is for
    locale_profile: None,    // None => the fingerprint's bundled locale
    jitter: false,           // true => vary per-session signals each mint
})?;
// send `token` as the X-Castle-Request-Token request header
```

You supply:

- `cuid` — the 32-hex `__cuid` cookie value (16 random bytes). Whatever you mint
  with must also live in your cookie jar as `__cuid` when the request goes out.
- `fingerprint` — a typed browser identity. One is bundled
  (`fingerprint::chrome_148_macos()`); load more with
  `fingerprint::load_devices(path)`.
- `pk` / `ig` — your deployment's public key and integration group ID, both
  visible in any captured token from your site.
- `hostname` — `window.location.hostname`, the site the token is for. It is
  deliberately not bundled in the fingerprint, so every mint must supply it.

`fp_lists` and `ce` are rendered fresh per call from the fingerprint's typed
traits. `fp_lists` spans five parts — basic browser traits (part 0), runtime /
UA-data probes (parts 4 and 7), and UA-Client-Hints / WebGL / platform detail
(parts 8 and 9); the part-8/9 case-4 fields are XXTEA frames re-encrypted on
every mint. Per-mint variation (the per-slot XXTEA ciphertexts keyed on
`init_time_ms`, the time tokens, the mask byte, and one e7 counter) is driven by
the supplied RNG; pass your own via [`token::mint_fresh`] for reproducible
output in tests.

## Site, locale & jitter

**`hostname` is required** — `window.location.hostname` is the site the token is
for, and it is deliberately not part of the bundled fingerprint, so every mint
supplies it. The fingerprint is otherwise a single captured session (an en-US
user); two more `MintOptions` fields adapt the rest, both opt-in and defaulting
to "off", which reproduces the fingerprint verbatim:

- **`locale_profile`** — a coherent geo/locale bundle (IANA time zone + offset,
  `navigator.language(s)`, `Intl` locale, voice language, and the derived
  `localeDateString`). Use a preset or build your own:

  ```rust
  use castle_token::fingerprint::LocaleProfile;

  let de = LocaleProfile::de_de();          // or ::en_us/_gb, ::fr_fr, ::it_it,
                                            // ::es_es, ::ja_jp, or ::preset("de-DE")
  // ...then in MintOptions: locale_profile: Some(&de)
  let custom = LocaleProfile::new(
      "en-CA", "America/Toronto", 300, 60,
      vec!["en-CA".into(), "en".into(), "fr-CA".into()],
  ); // None if the locale's date format isn't built in — set the field directly
  ```

- **`jitter`** — when `true`, the per-session timing/behavioral signals
  (navigation timing, JS heap usage, render latency, canvas-perf ratio, and the
  `ce` event timings) are varied each mint so no two tokens are byte-identical,
  while every device-identity field stays fixed and consistent. Variation is
  drawn from the supplied RNG, so a fixed seed is reproducible.

## CLI

```sh
cargo run -- \
    --cuid 00112233445566778899aabbccddeeff \
    --pk pk_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx \
    --ig 225 \
    --hostname id.fanatics.com \
    --locale de-DE \
    --jitter
```

`--cuid` defaults to a fresh random value and `--init-time-ms` to `now-4000ms`;
`--pk`, `--ig`, and `--hostname` are required. `--locale` (one of the
[`LocaleProfile`] presets) and `--jitter` are optional and map to the
`MintOptions` fields above.

## Modules

| Module | Purpose |
|---|---|
| `token` | Mint the `X-Castle-Request-Token` (public entry point). |
| `fingerprint` | Typed browser-identity bundle + per-slot fp_lists encoders (parts 0/4/7 plus the `ExtraSlot` parts 8/9), `LocaleProfile` presets, and the per-mint jitter pass. Embeds `devices.json`. |
| `ce` | Typed encoder/decoder for the header-less `ce` event-stream blob. |
| `events` | `e7` (yh / hg / Fg) events generator. |
| `codec` | base64url + MurmurHash3 x86_32 + `n_hex` helper. |
| `xxtea` | XXTEA encrypt/decrypt with Castle's universal outer key. |

## Tests

`cargo test` runs golden vectors that lock the wire format —
the XXTEA / MurmurHash3 / base64url primitives, the `encode_token` SHA-256 gate,
the `encode_fp` / `encode_ce` fingerprint goldens, and a real-token vector that
reproduces parts 8/9 byte-for-byte — plus an end-to-end decode round-trip that
reverses a minted token and checks the embedded fields.
