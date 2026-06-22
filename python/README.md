# castle-token (Python)

Python bindings for the Rust [`castle-token`](https://github.com/0x640480/castle-rs)
crate — generate the `X-Castle-Request-Token` header minted by Castle's web SDK.
The byte-exact wire format lives in the Rust core; this is a thin PyO3 wrapper.

```sh
pip install castle-token
```
Prebuilt `abi3` wheels are published for Linux (manylinux + musllinux, x86-64 &
arm64), macOS (x86-64 & arm64), and Windows (x64) — one wheel covers every
CPython ≥ 3.9, so most installs need no Rust toolchain.

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

## Build from source

```sh
cd python && maturin develop      # or: pip install .
```

## Releasing to PyPI (maintainers)

Wheels + sdist are built and published by
[`.github/workflows/python-release.yml`](../.github/workflows/python-release.yml)
on a `vX.Y.Z` tag, via **PyPI Trusted Publishing** (OIDC — no API token stored).

One-time setup at <https://pypi.org/manage/account/publishing/> — add a *pending*
publisher (works before the project exists):

| Field | Value |
|---|---|
| PyPI project name | `castle-token` |
| Owner | `0x640480` |
| Repository | `castle-rs` |
| Workflow name | `python-release.yml` |
| Environment | `pypi` |

Then create a GitHub environment named `pypi` (repo **Settings → Environments**);
add manual-approval reviewers there if you want a release gate.

To cut a release: bump `version` in [`python/Cargo.toml`](Cargo.toml) (the wheel
version is `dynamic`, read from it), commit, then:

```sh
git tag vX.Y.Z && git push origin vX.Y.Z
```

`workflow_dispatch` runs the same build matrix without publishing, for a dry run.
