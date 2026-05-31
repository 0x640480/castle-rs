//! Assembly of the pre-XOR inner payload consumed by [`super::encode_token`].

use crate::error::{Error, Result};
use crate::events::{FG_LEN, HG_LEN};

const E7_HEADER_LEN_HEX: usize = 6;
const FG_LEN_BYTE_HEX: usize = 2;

/// Total length of a valid e7-events hex blob (396 chars).
pub const E7_TOTAL_LEN_HEX: usize = E7_HEADER_LEN_HEX + HG_LEN * 2 + FG_LEN * 2 + FG_LEN_BYTE_HEX;

/// Concatenates fp_lists, the `ce` blob (with its 2-byte length), the e7 events,
/// and the `ff` terminator into the pre-XOR inner payload.
pub fn assemble_inner_payload(
    fp_lists_hex: &str,
    ce_hex: &str,
    e7_events_hex: &str,
) -> Result<String> {
    let ce_byte_count = ce_hex.len() / 2;
    if ce_byte_count > 0xFFFF {
        return Err(Error::CeTooLong(ce_byte_count));
    }
    if e7_events_hex.len() != E7_TOTAL_LEN_HEX {
        return Err(Error::BadEventsLen {
            expected: E7_TOTAL_LEN_HEX,
            got: e7_events_hex.len(),
        });
    }
    Ok(format!(
        "{fp_lists_hex}{ce_byte_count:04x}{ce_hex}{e7_events_hex}ff"
    ))
}
