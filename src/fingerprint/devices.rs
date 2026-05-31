//! The bundled device catalog and helpers for loading custom ones.

use std::path::Path;
use std::sync::LazyLock;

use rand::Rng;

use super::Fingerprint;
use crate::error::Result;

const BUNDLED_DEVICES_JSON: &str = include_str!("devices.json");

static BUNDLED: LazyLock<Vec<Fingerprint>> = LazyLock::new(|| {
    let devices: Vec<Fingerprint> =
        serde_json::from_str(BUNDLED_DEVICES_JSON).expect("bundled devices.json is malformed");
    assert!(!devices.is_empty(), "bundled devices.json has no entries");
    devices
});

/// The parsed bundled fingerprint catalog.
pub fn bundled_devices() -> &'static [Fingerprint] {
    &BUNDLED
}

/// The first bundled fingerprint (Chrome 148 on macOS).
pub fn chrome_148_macos() -> &'static Fingerprint {
    &BUNDLED[0]
}

/// A uniformly random entry from the bundled catalog.
pub fn random_bundled_device(rng: &mut impl Rng) -> &'static Fingerprint {
    &BUNDLED[rng.gen_range(0..BUNDLED.len())]
}

/// Loads a custom catalog (a JSON array of `Fingerprint` records) from disk.
pub fn load_devices(path: impl AsRef<Path>) -> Result<Vec<Fingerprint>> {
    let raw = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw)?)
}
