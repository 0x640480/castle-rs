# castle-token (Python)

Python bindings for the Rust [`castle-token`](https://github.com/0x640480/castle-rs)
crate — generate the `X-Castle-Request-Token` header minted by Castle's web SDK.
The byte-exact wire format lives in the Rust core; this is a thin PyO3 wrapper.

```python
import castle_token as ct

token = ct.mint_token(
    cuid="00112233445566778899aabbccddeeff",
    pk="pk_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
    ig=225,
    hostname="id.example.com",          # the site the token is for (required)
    locale_profile=ct.LocaleProfile.preset("de-DE"),  # optional
    jitter=True,                        # optional: vary per-session signals
)
# send `token` as the X-Castle-Request-Token request header
```

- `mint_token(cuid, pk, ig, hostname, *, fingerprint=None, locale_profile=None, jitter=False, init_time_ms=None, now_ms=None) -> str`
- `Fingerprint.chrome_148_macos()`, `load_devices(path)`, `random_bundled_device()`
- `LocaleProfile.preset(tag)` / `.en_us()` / `.de_de()` / … / `.new(...)`
- Errors raise `CastleError`.
