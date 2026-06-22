const { test } = require('node:test')
const assert = require('node:assert')

// `require('..')` resolves the package main (index.js) — the same entry point a
// consumer hits with `require('castle-token')`.
const ct = require('..')

const CUID = '00112233445566778899aabbccddeeff'
const PK = 'pk_xPQ5kRvjnzuTy24zZtig3eNMzspdJS92'
const IG = 225
const HOST = 'id.fanatics.com'

const isValidToken = (t) =>
  typeof t === 'string' && t.length >= 1000 && /^[A-Za-z0-9_-]+$/.test(t)

test('mint with required args only', () => {
  const tok = ct.mintToken(CUID, PK, IG, HOST)
  assert.ok(isValidToken(tok), `unexpected token (len=${tok && tok.length})`)
})

test('fingerprint, locale profile, and jitter', () => {
  const fp = ct.Fingerprint.chrome148Macos()
  assert.strictEqual(fp.uaPlatform, 'macOS')

  const prof = ct.LocaleProfile.preset('de-DE')
  assert.strictEqual(prof.locale, 'de-DE')
  assert.deepStrictEqual(prof.languages, ['de-DE', 'de'])

  const t1 = ct.mintToken(CUID, PK, IG, HOST, fp, prof, true)
  const t2 = ct.mintToken(CUID, PK, IG, HOST, fp, prof, true)
  assert.ok(isValidToken(t1) && isValidToken(t2))
  assert.notStrictEqual(t1, t2) // per-mint randomness + jitter
})

test('locale preset shorthands and custom builder', () => {
  assert.strictEqual(ct.LocaleProfile.jaJp().locale, 'ja-JP')
  // en-CA has no built-in date format -> the core returns None -> throws.
  assert.throws(() =>
    ct.LocaleProfile.create('en-CA', 'America/Toronto', 300, 60, ['en-CA', 'en']),
  )
})

test('invalid input throws', () => {
  assert.throws(() => ct.LocaleProfile.preset('zz-ZZ'))
  assert.throws(() => ct.mintToken(CUID, 'nope', IG, HOST))
})

test('random bundled device', () => {
  const fp = ct.randomBundledDevice()
  assert.ok(typeof fp.userAgent === 'string' && fp.userAgent.length > 0)
})
