"""Python bindings for castle-token — generate the X-Castle-Request-Token header.

The wire-format logic lives in the Rust `castle-token` crate; this package is a
thin PyO3 wrapper. See the compiled extension `castle_token._castle_token`.
"""

from ._castle_token import (
    CastleError,
    Fingerprint,
    LocaleProfile,
    load_devices,
    mint_token,
    random_bundled_device,
)

__all__ = [
    "CastleError",
    "Fingerprint",
    "LocaleProfile",
    "load_devices",
    "mint_token",
    "random_bundled_device",
]
