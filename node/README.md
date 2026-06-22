# castle-token (Node.js)

Native Node.js bindings ([napi-rs](https://napi.rs)) for the Rust
[`castle-token`](https://github.com/0x640480/castle-rs) crate — generate the
`X-Castle-Request-Token` header minted by Castle's web SDK. The byte-exact wire
format lives in the Rust core; this is a thin addon (no WASM, no port).

```sh
npm install castle-token
```
A prebuilt `.node` is selected automatically via per-platform
`optionalDependencies` (Linux gnu/musl x64 & arm64, macOS x64 & arm64, Windows
x64 & arm64) — no Rust toolchain needed.

```js
const ct = require('castle-token')

const token = ct.mintToken(
  '00112233445566778899aabbccddeeff', // cuid
  'pk_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx', // pk
  225, // ig
  'id.example.com', // hostname (the site the token is for)
  ct.Fingerprint.chrome148Macos(), // fingerprint (optional; this is the default)
  ct.LocaleProfile.preset('de-DE'), // localeProfile (optional)
  true, // jitter (optional)
)
// send `token` as the X-Castle-Request-Token request header
```

`mintToken(cuid, pk, ig, hostname, fingerprint?, localeProfile?, jitter?, initTimeMs?, nowMs?) -> string`

- `Fingerprint.chrome148Macos()`, `loadDevices(path)`, `randomBundledDevice()`
- `LocaleProfile.preset(tag)` / `.enUs()` / `.deDe()` / … / `.create(...)`
- Invalid input throws an `Error`.

## Example

A runnable script is in [`examples/mint-token.js`](examples/mint-token.js) —
`node examples/mint-token.js`.

## Build from source

```sh
npm install
npm run build   # napi build --platform --release → index.js + index.d.ts + *.node
npm test        # node --test
```
