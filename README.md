# castle-token

Rust generator for the `X-CRS-Req-Token` header minted by Castle's web SDK.
The header carries an encrypted device fingerprint plus a behavioral signal
that Castle's risk engine validates server-side before letting requests through
to login / account endpoints.

The wire format is universal across Castle web customers — point the library at
a different deployment by supplying its public key (`pk_…`) and integration
group ID. HTTP-free.

XXTEA is hand-rolled for exact output control; everything else leans on
standard crates (`serde`, `base64`, `hex`, `rand`, `clap`, `thiserror`,
`murmurhash3`).

## Quick start

```rust
use castle_token::fingerprint;
use castle_token::token::{mint_fresh_default, MintOptions};

let cuid = hex::encode(rand::random::<[u8; 16]>());
let token = mint_fresh_default(&MintOptions {
    cuid: &cuid,
    fingerprint: fingerprint::chrome_148_macos(),
    init_time_ms: None, // None => now - 4000ms
    pk: "pk_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
    ig: 225,
    now_ms: None,       // None => current system time
})?;
// send `token` as the X-CRS-Req-Token request header
```

You supply:

- `cuid` — the 32-hex `__cuid` cookie value (16 random bytes). Whatever you mint
  with must also live in your cookie jar as `__cuid` when the request goes out.
- `fingerprint` — a typed browser identity. One is bundled
  (`fingerprint::chrome_148_macos()`); load more with
  `fingerprint::load_devices(path)`.
- `pk` / `ig` — your deployment's public key and integration group ID, both
  visible in any captured token from your site.

`fp_lists` and `ce` are rendered fresh per call from the fingerprint's typed
traits. Per-mint variation (the per-slot XXTEA ciphertexts keyed on
`init_time_ms`, the time tokens, the mask byte, and one e7 counter) is driven by
the supplied RNG; pass your own via [`token::mint_fresh`] for reproducible
output in tests.

## CLI

```sh
cargo run -- \
    --cuid 00112233445566778899aabbccddeeff \
    --pk pk_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx \
    --ig 225
```

`--cuid` defaults to a fresh random value and `--init-time-ms` to `now-4000ms`;
`--pk` and `--ig` are required.

## Modules

| Module | Purpose |
|---|---|
| `token` | Mint the `X-CRS-Req-Token` (public entry point). |
| `fingerprint` | Typed browser-identity bundle + per-slot fp_lists encoders. Embeds `devices.json`. |
| `ce` | Typed encoder/decoder for the `ce` event-stream blob. |
| `events` | `e7` (yh / hg / Fg) events generator. |
| `codec` | base64url + MurmurHash3 x86_32 + `n_hex` helper. |
| `xxtea` | XXTEA encrypt/decrypt with Castle's universal outer key. |

## Tests

`cargo test` runs golden vectors that lock the wire format —
the XXTEA / MurmurHash3 / base64url primitives, the `encode_token` SHA-256 gate,
the `encode_fp` / `encode_ce` fingerprint goldens — plus an end-to-end decode
round-trip that reverses a minted token and checks the embedded fields.
