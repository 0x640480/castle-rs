//! Generator for the `e7` (yh / hg / Fg) behavioral-events blob.
//!
//! The output is a fixed 396 hex chars: a 6-char header, 183 custom 1-byte
//! floats (`Lf`), 11 log-compressed counters (`Fg`), and a 2-char Fg length.

use rand::Rng;

use crate::error::{Error, Result};

/// Number of `hg` float values.
pub const HG_LEN: usize = 183;
/// Number of `Fg` integer counters.
pub const FG_LEN: usize = 11;

/// Default `yh` bool vector, derived from a captured session.
pub const DEFAULT_YH_BOOLS: [bool; 15] = [
    false, true, true, false, true, true, false, true, false, false, false, false, false, false,
    false,
];
/// Default `Fg` counters, derived from a captured session.
pub const DEFAULT_FG_INTS: [i64; FG_LEN] = [34, 2, 2, 2, 0, 4, 0, 0, 6, 0, 0];

/// Inputs to [`fresh_events_hex`]. [`FreshOptions::default`] reproduces the
/// captured-session baseline used by the bundled profile.
#[derive(Clone, Debug)]
pub struct FreshOptions {
    /// 15 bools packed into the header.
    pub yh_bools: Vec<bool>,
    /// 183 floats; `0.0` (or a set `hg_nil_mask` entry) encodes to a zero byte.
    pub hg_floats: Vec<f64>,
    /// 183 flags; `true` treats the matching `hg_floats` entry as absent.
    pub hg_nil_mask: Vec<bool>,
    /// 11 counters; `fg_ints[0]` takes a random walk during minting.
    pub fg_ints: Vec<i64>,
}

impl Default for FreshOptions {
    fn default() -> Self {
        Self {
            yh_bools: DEFAULT_YH_BOOLS.to_vec(),
            hg_floats: vec![0.0; HG_LEN],
            hg_nil_mask: vec![true; HG_LEN],
            fg_ints: DEFAULT_FG_INTS.to_vec(),
        }
    }
}

/// The custom float layout `(e << mant_bits) | m`, used by `gl`.
fn float_encode(exp_bits: i32, mant_bits: i32, x: f64) -> i64 {
    if x == 0.0 {
        return 0;
    }
    let mut x = x.abs();
    let mut e: i32 = 0;
    while x >= 2.0 {
        x /= 2.0;
        e += 1;
    }
    while x < 1.0 {
        x *= 2.0;
        e -= 1;
    }
    let max_e = (1 << exp_bits) - 1;
    if e > max_e {
        e = max_e;
    }
    let mut m: i64 = 0;
    x -= 1.0;
    for _ in 0..mant_bits {
        x *= 2.0;
        let bit = x as i64;
        m = (m << 1) | bit;
        x -= bit as f64;
    }
    ((e as i64) << mant_bits) | m
}

/// Encodes a single `hg` value as a 1-byte custom float. A nil/zero slot is 0.
fn gl(x: f64, nil_float: bool) -> i64 {
    if nil_float || x == 0.0 {
        return 0;
    }
    if x <= 15.0 {
        return 0x40 | float_encode(2, 4, x + 1.0);
    }
    let v = if x > 65536.0 { 65536.0 } else { x };
    0x80 | float_encode(4, 3, v - 14.0)
}

/// Scales an `Fg` byte counter: identity below 128, log-compressed above.
fn ga(n: i64) -> i64 {
    if n < 128 {
        return if n < 0 { 0 } else { n };
    }
    let log_h = (n as f64).ln() * std::f64::consts::LOG2_E;
    let out = ((log_h - 7.0) * -31.75 + 128.0) as i64;
    out.clamp(0, 255)
}

/// Packs the first 16 bools MSB-first into an integer.
fn arr_to_16_bits(bools: &[bool]) -> i64 {
    let mut v: i64 = 0;
    for i in 0..16 {
        let b = i64::from(i < bools.len() && bools[i]);
        v = (v << 1) | b;
    }
    v
}

/// Builds a fresh e7-events hex blob (396 hex chars).
///
/// `fg_ints[0]` takes a random walk of ±5 (clamped at zero), the only
/// RNG-driven field. All other values are deterministic from `opts`.
pub fn fresh_events_hex(opts: &FreshOptions, rng: &mut impl Rng) -> Result<String> {
    check_len("YHBools", opts.yh_bools.len(), 15)?;
    check_len("HGFloats", opts.hg_floats.len(), HG_LEN)?;
    check_len("HGNilMask", opts.hg_nil_mask.len(), HG_LEN)?;
    check_len("FGInts", opts.fg_ints.len(), FG_LEN)?;

    let mut fg = opts.fg_ints.clone();
    fg[0] += rng.gen_range(0i64..11) - 5;
    if fg[0] < 0 {
        fg[0] = 0;
    }

    let yh_hash = arr_to_16_bits(&opts.yh_bools);
    let header_int = (6i64 << 20) | (2i64 << 16) | (yh_hash & 0xFFFF);
    let header_hex = format!("{:06x}", header_int & 0xFF_FFFF);

    let mut body = String::with_capacity(396);
    body.push_str(&header_hex);
    for (i, &x) in opts.hg_floats.iter().enumerate() {
        body.push_str(&format!("{:02x}", gl(x, opts.hg_nil_mask[i]) as u8));
    }
    for &n in &fg {
        body.push_str(&format!("{:02x}", ga(n) as u8));
    }
    body.push_str(&format!("{:02x}", FG_LEN as u8));
    Ok(body)
}

fn check_len(field: &'static str, got: usize, expected: usize) -> Result<()> {
    if got != expected {
        return Err(Error::BadVecLen {
            field,
            expected,
            got,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn default_blob_is_396_chars() {
        let mut rng = StdRng::seed_from_u64(1);
        let hex = fresh_events_hex(&FreshOptions::default(), &mut rng).unwrap();
        assert_eq!(hex.len(), 396);
        // Header is fixed for the default yh vector: (6<<20)|(2<<16)|0x6d00.
        assert_eq!(&hex[..6], "626d00");
    }
}
