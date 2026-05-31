//! The single error type returned across the crate.

/// Errors produced while minting or decoding a Castle token.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The `__cuid` cookie value is shorter than the 10 hex chars the outer
    /// XOR layer indexes into.
    #[error("cuid too short: {0} (need >=10)")]
    CuidTooShort(usize),

    /// The public key was not `pk_` followed by exactly 32 characters.
    #[error("pk must be 'pk_' + 32 chars, got {0} chars")]
    InvalidPk(usize),

    /// The encoded `ce` blob exceeds the 16-bit length field.
    #[error("ce too long: {0} bytes")]
    CeTooLong(usize),

    /// The e7 events blob was not the expected fixed length.
    #[error("e7 events hex must be {expected} hex chars, got {got}")]
    BadEventsLen { expected: usize, got: usize },

    /// A `ce` event's registry-byte length did not match its class handler.
    #[error("ce class {class} expects {expected}B registry, got {got}B")]
    BadRegistryLen {
        class: u8,
        expected: usize,
        got: usize,
    },

    /// A vector that must have a fixed length did not.
    #[error("{field} must have {expected} elements, got {got}")]
    BadVecLen {
        field: &'static str,
        expected: usize,
        got: usize,
    },

    /// A `ce` blob could not be decoded back into typed events.
    #[error("ce decode: {0}")]
    CeDecode(String),

    /// Failed to decode a hex string.
    #[error("invalid hex: {0}")]
    Hex(#[from] hex::FromHexError),

    /// Failed to (de)serialize a device catalog.
    #[error("devices json: {0}")]
    Json(#[from] serde_json::Error),

    /// Failed to read a device catalog from disk.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

/// Crate-wide result alias.
pub type Result<T> = std::result::Result<T, Error>;
