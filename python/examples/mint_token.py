"""Mint an `X-Castle-Request-Token` header value with castle-token.

    pip install castle-token        # or, from source: cd python && maturin develop
    python examples/mint_token.py
"""

import secrets

import castle_token as ct

# Your Castle deployment's parameters — all visible in any token captured from
# the target site (DevTools -> Network -> the X-Castle-Request-Token header):
PK = "pk_xPQ5kRvjnzuTy24zZtig3eNMzspdJS92"  # public key
IG = 225  # integration group id
HOSTNAME = "id.example.com"  # window.location.hostname — the site the token is for

# The `__cuid` cookie value (32 hex chars = 16 random bytes). Whatever you mint
# with must also be in your cookie jar as `__cuid` when the request goes out.
cuid = secrets.token_hex(16)


def basic() -> str:
    """Required args only — uses the bundled Chrome 148 / macOS fingerprint."""
    return ct.mint_token(cuid=cuid, pk=PK, ig=IG, hostname=HOSTNAME)


def localized() -> str:
    """A German session with per-mint jitter, so no two tokens are identical."""
    return ct.mint_token(
        cuid=cuid,
        pk=PK,
        ig=IG,
        hostname=HOSTNAME,
        locale_profile=ct.LocaleProfile.preset("de-DE"),
        jitter=True,
    )


def main() -> None:
    token = basic()
    print(f"basic token      : {token[:48]}... ({len(token)} chars)")

    a, b = localized(), localized()
    print(f"localized+jitter : {a[:48]}... ({len(a)} chars)")
    assert a != b, "jitter should make every mint unique"

    # Use it as the request header (Castle's default name; some deployments
    # rename it, e.g. Fanatics sends it as X-CRS-Req-Token):
    #
    #   import requests
    #   requests.get(
    #       "https://id.example.com/...",
    #       headers={"X-Castle-Request-Token": token},
    #       cookies={"__cuid": cuid},
    #   )
    print("\nSend the value as the `X-Castle-Request-Token` request header.")


if __name__ == "__main__":
    main()
