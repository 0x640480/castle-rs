//! Generator for the `X-CRS-Req-Token` header minted by Castle's web SDK.
//!
//! The header carries an encrypted device fingerprint plus a behavioral signal
//! that Castle's risk engine validates server-side. This crate reproduces the
//! token wire format byte-for-byte: a fingerprint is rendered through the
//! per-slot encoders, framed with the `ce` and e7-events blobs, XOR-layered,
//! XXTEA-encrypted, and base64url-encoded.
//!
//! See [`token::mint_fresh`] for the high-level entry point.

pub mod ce;
pub mod codec;
pub mod error;
pub mod events;
pub mod fingerprint;
pub mod token;
pub mod xxtea;

pub use error::{Error, Result};
