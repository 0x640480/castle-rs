//! Parts 8 & 9 (wire ids 8, 9): additional UA-Client-Hints / WebGL / platform
//! detail — GPU vendor & renderer, the full user agent, architecture/bitness/
//! model, Chrome full version, and the UA-CH brand list.
//!
//! Unlike parts 0/4/7 these have no bespoke per-slot encoders; each slot is
//! reproduced generically via [`ExtraSlot`]. Case-4 slots carry an XXTEA frame
//! keyed on `init_time`, so they store the **plaintext** and re-encrypt per
//! mint; every other case is init-independent and replays its raw payload.

use serde::{Deserialize, Serialize};

use super::super::encode::{case4, tag};

/// One part-8/9 slot.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ExtraSlot {
    /// Slot index within the part.
    pub index: u8,
    /// Encoding case (1–7).
    pub case: u8,
    /// Case 4: the plaintext, re-encrypted per mint. Other cases: the raw
    /// payload hex appended after the tag byte (init-independent).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub data: String,
}

/// Encodes a part-8/9 slot list at the given `init_time`.
pub(super) fn encode(slots: &[ExtraSlot], init: i64) -> Vec<String> {
    slots
        .iter()
        .map(|s| {
            if s.case == 4 {
                case4(i64::from(s.index), &s.data, init)
            } else {
                tag(i64::from(s.index), i64::from(s.case)) + &s.data
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fingerprint::chrome_148_macos;

    // Captured from a real clean macOS Chrome 148 X-Castle-Request-Token.
    const INIT: i64 = 1_780_365_919_859;
    const PART8: &str = "05300c080fbe443f8f0143fb14080abc2f502e01c08e1d162c0c302179a3bb397395a70e91063700000000005c3f2b000000000000450054084a2651ba1d485bfa5a626d0075007d008704008c00919e0aa58000ad8000b9c42461893581a9c0f7283403593e0fc9e745dbc20c7391c3b54faa1613f93f7c2d762404adbfcc045b5d0000d444c81103ef3ee440a34a291c1014fc1a8897c9c222d831e0b3486973ef0909d8698aa885f4129f613c3a02b690d3f04ff9e0994d9bfc00b0c212e0db0c6513a7267e886d84";
    const PART9: &str = "05010c045b5d0000121a240877bfb92e66af60ae2a4c7862b676f8b6962d3265432ce7470a5746126adf5a91d97872a11a3405b3d0a112963ce89f4fc7620c008193b35326e024835d11055d9b64d29f579b12cc7dd5f3a614f22f1f66113d340b4802f354df82d244509f0d322f326797961668a78720264af1e72e66e3bdaa829b03454e1160f3aafebb21db6d6a64087f537e57614e25ee6c0461726d007404363400007c044e4100008408cd301b1bcdcc2c0f8c10899523d539000e8b73de32f595eab3779494a4a4df9f180da117ea0cf3112573f4de89b27cea394ebe63074c95dd35ce3e3e80f84a9ddb5203913d02df6cd90f59c5ac13c99c660320aff29bf7cb511ec06c0178077c10fe6e3fd67d98b0c176e1ece4245808bff5a2d3de66fcb44f4f9e8d3d102b2832ed598309e98e14bb2efe8989d4c46a74b34fb6bb573727e6c6d57d06d1d89379cf508d0cad8717f3b27a89551b4099";

    #[test]
    fn bundled_parts_8_9_reproduce_real_token() {
        let fp = chrome_148_macos();
        assert_eq!(encode(&fp.part8, INIT).concat(), PART8, "part 8 mismatch");
        assert_eq!(encode(&fp.part9, INIT).concat(), PART9, "part 9 mismatch");
    }
}
