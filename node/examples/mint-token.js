// Mint an `X-Castle-Request-Token` header value with castle-token.
//
//   npm install castle-token
//   node examples/mint-token.js

const crypto = require('node:crypto')
const ct = require('castle-token')

// Your Castle deployment's parameters — all visible in any token captured from
// the target site (DevTools -> Network -> the X-Castle-Request-Token header):
const PK = 'pk_xPQ5kRvjnzuTy24zZtig3eNMzspdJS92' // public key
const IG = 225 // integration group id
const HOSTNAME = 'id.example.com' // window.location.hostname — the site the token is for

// The `__cuid` cookie value (32 hex chars = 16 random bytes). Whatever you mint
// with must also be in your cookie jar as `__cuid` when the request goes out.
const cuid = crypto.randomBytes(16).toString('hex')

// Required args only — uses the bundled Chrome 148 / macOS fingerprint.
const basic = ct.mintToken(cuid, PK, IG, HOSTNAME)
console.log(`basic token      : ${basic.slice(0, 48)}... (${basic.length} chars)`)

// A German session with per-mint jitter, so no two tokens are identical.
// Positional optionals: (…, fingerprint?, localeProfile?, jitter?) — pass
// `undefined` to keep the default fingerprint.
const de = ct.LocaleProfile.preset('de-DE')
const a = ct.mintToken(cuid, PK, IG, HOSTNAME, undefined, de, true)
const b = ct.mintToken(cuid, PK, IG, HOSTNAME, undefined, de, true)
console.log(`localized+jitter : ${a.slice(0, 48)}... (${a.length} chars)`)
if (a === b) throw new Error('jitter should make every mint unique')

// Use it as the request header (Castle's default name; some deployments rename
// it, e.g. Fanatics sends it as X-CRS-Req-Token):
//
//   await fetch('https://id.example.com/...', {
//     headers: { 'X-Castle-Request-Token': basic, cookie: `__cuid=${cuid}` },
//   })
console.log('\nSend the value as the `X-Castle-Request-Token` request header.')
