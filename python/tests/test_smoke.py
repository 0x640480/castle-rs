import pytest

import castle_token as ct

CUID = "00112233445566778899aabbccddeeff"
PK = "pk_xPQ5kRvjnzuTy24zZtig3eNMzspdJS92"
IG = 225
HOST = "id.fanatics.com"


def _valid(tok: str) -> bool:
    return len(tok) >= 1000 and all(c.isalnum() or c in "-_" for c in tok)


def test_mint_basic():
    tok = ct.mint_token(cuid=CUID, pk=PK, ig=IG, hostname=HOST)
    assert _valid(tok)


def test_fingerprint_locale_and_jitter():
    fp = ct.Fingerprint.chrome_148_macos()
    assert fp.ua_platform == "macOS"

    prof = ct.LocaleProfile.preset("de-DE")
    assert prof.locale == "de-DE"
    assert prof.languages == ["de-DE", "de"]

    t1 = ct.mint_token(
        cuid=CUID, pk=PK, ig=IG, hostname=HOST,
        fingerprint=fp, locale_profile=prof, jitter=True,
    )
    t2 = ct.mint_token(
        cuid=CUID, pk=PK, ig=IG, hostname=HOST,
        fingerprint=fp, locale_profile=prof, jitter=True,
    )
    assert _valid(t1) and _valid(t2)
    assert t1 != t2  # per-mint randomness + jitter


def test_locale_new_unknown_raises():
    # en-CA has no built-in date format -> Rust returns None -> CastleError.
    with pytest.raises(ct.CastleError):
        ct.LocaleProfile.new("en-CA", "America/Toronto", 300, 60, ["en-CA", "en"])


def test_preset_unknown_raises():
    with pytest.raises(ct.CastleError):
        ct.LocaleProfile.preset("zz-ZZ")


def test_bad_pk_raises():
    with pytest.raises(ct.CastleError):
        ct.mint_token(cuid=CUID, pk="nope", ig=IG, hostname=HOST)


def test_random_bundled_device():
    fp = ct.random_bundled_device()
    assert isinstance(fp.user_agent, str) and fp.user_agent
