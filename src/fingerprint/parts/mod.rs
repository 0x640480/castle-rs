//! The fp_lists part encoders, grouped by Castle's wire-part id.
//!
//! A token's fp_lists is five parts whose wire ids are non-contiguous —
//! `[0, 4, 7, 8, 9]` (see [`super::encode::PART_IDS`]); ids 1/2/3/5/6 are unused.
//! Parts 0, 4, and 7 have bespoke per-slot encoders ([`part0`], [`part4`],
//! [`part7`]); parts 8 and 9 share the generic [`extra`] / [`ExtraSlot`]
//! mechanism. The module names track the wire ids so they line up with the
//! `(part/slot)` notation in the slot doc comments and with real captures.

mod extra;
mod part0;
mod part4;
mod part7;

pub use extra::ExtraSlot;

use super::{encode, Fingerprint};

/// Renders the full fp_lists hex (parts 0/4/7/8/9) for `fp` at `init_time_ms`.
///
/// `utc_minutes` feeds slot 7/4; see [`super::Fingerprint::encode_fp`].
pub(super) fn fp_lists(fp: &Fingerprint, init_time_ms: i64, utc_minutes: i64) -> String {
    encode::encode_lists(&[
        part0::encode(fp, init_time_ms),
        part4::encode(fp, init_time_ms),
        part7::encode(fp, init_time_ms, utc_minutes),
        extra::encode(&fp.part8, init_time_ms),
        extra::encode(&fp.part9, init_time_ms),
    ])
}
