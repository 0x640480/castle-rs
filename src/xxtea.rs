//! XXTEA block cipher with Castle's universal outer key.
//!
//! Operates on little-endian `u32` words. All arithmetic wraps at the u32
//! boundary.

const DELTA: u32 = 0x9E37_79B9;

/// Castle's XXTEA key for the outer (whole-payload) layer.
pub const CASTLE_OUTER_KEY: [u32; 4] = [0x4567_8907, 0xE7F2_A9B0, 0x0D0D_0D0D, 0xA3B5_C5D6];

/// Packs bytes as little-endian `u32`s, right-padding the final word with zeros.
pub fn to_u32_array(data: &[u8]) -> Vec<u32> {
    let n = data.len().div_ceil(4);
    let mut out = Vec::with_capacity(n);
    for r in 0..n {
        let mut b = [0u8; 4];
        let start = r * 4;
        let end = (start + 4).min(data.len());
        b[..end - start].copy_from_slice(&data[start..end]);
        out.push(u32::from_le_bytes(b));
    }
    out
}

/// Unpacks `u32`s back to little-endian bytes (4 bytes per word).
pub fn from_u32_array(arr: &[u32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(arr.len() * 4);
    for &v in arr {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

/// XXTEA mixing function: `((P1 + P2) ^ P3)`.
///
/// The parentheses are explicit: the two additions bind before the final XOR.
fn mx(z: u32, y: u32, sum: u32, k: u32) -> u32 {
    let p1 = (z >> 5) ^ (y << 2);
    let p2 = (y >> 3) ^ (z << 4);
    let p3 = (sum ^ y).wrapping_add(k ^ z);
    p1.wrapping_add(p2) ^ p3
}

/// Encrypts `data` (zero-padded to a 4-byte boundary). Returns the encrypted
/// word array; use [`from_u32_array`] for the bytes. Inputs of fewer than two
/// words are returned unchanged.
pub fn encrypt(data: &[u8], key: &[u32; 4]) -> Vec<u32> {
    let mut v = to_u32_array(data);
    if v.len() < 2 {
        return v;
    }
    let n = v.len() - 1;
    let rounds = 6 + 52 / (n + 1);

    let mut sum: u32 = 0;
    let mut z = v[n];
    for _ in 0..rounds {
        sum = sum.wrapping_add(DELTA);
        let e = (sum >> 2) & 3;
        for p in 0..n {
            let y = v[p + 1];
            v[p] = v[p].wrapping_add(mx(z, y, sum, key[((p as u32 & 3) ^ e) as usize]));
            z = v[p];
        }
        let y = v[0];
        v[n] = v[n].wrapping_add(mx(z, y, sum, key[((n as u32 & 3) ^ e) as usize]));
        z = v[n];
    }
    v
}

/// Reverses [`encrypt`].
pub fn decrypt(v: &[u32], key: &[u32; 4]) -> Vec<u32> {
    let mut out = v.to_vec();
    if out.len() < 2 {
        return out;
    }
    let n = out.len() - 1;
    let rounds = (6 + 52 / (n + 1)) as u32;
    let mut sum = rounds.wrapping_mul(DELTA);

    for _ in 0..rounds {
        let e = (sum >> 2) & 3;
        let y = out[0];
        let mut z = out[n - 1];
        out[n] = out[n].wrapping_sub(mx(z, y, sum, key[((n as u32 & 3) ^ e) as usize]));
        for p in (1..n).rev() {
            z = out[p - 1];
            let y = out[p + 1];
            out[p] = out[p].wrapping_sub(mx(z, y, sum, key[((p as u32 & 3) ^ e) as usize]));
        }
        z = out[n];
        let y = out[1];
        out[0] = out[0].wrapping_sub(mx(z, y, sum, key[e as usize]));
        sum = sum.wrapping_sub(DELTA);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_unpack() {
        let input = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let arr = to_u32_array(&input);
        assert_eq!(arr, vec![0x0403_0201, 0x0807_0605]);
        assert_eq!(from_u32_array(&arr), input);
    }

    #[test]
    fn pack_zero_padding() {
        let arr = to_u32_array(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE]);
        assert_eq!(arr, vec![0xDDCC_BBAA, 0x0000_00EE]);
    }

    #[test]
    fn mx_against_reference() {
        assert_eq!(
            mx(0x1234_5678, 0x9ABC_DEF0, 0x9E37_79B9, 0x4567_8907),
            0xC1AA_5319
        );
    }

    #[test]
    fn encrypt_against_reference() {
        let plain = b"the quick brown fox jumps over the lazy dog 1234567890 abcdef!!\x00";
        assert_eq!(plain.len(), 64);
        let enc = encrypt(plain, &CASTLE_OUTER_KEY);
        let head = [0x8abc_ba83, 0x57a9_6688, 0xdf8f_3e87, 0x7aeb_d459];
        let tail = [0x99e6_a3d6, 0x95f1_b117, 0x58de_7c47, 0x37ef_1be8];
        assert_eq!(&enc[..4], &head);
        assert_eq!(&enc[enc.len() - 4..], &tail);
    }

    #[test]
    fn round_trip() {
        let plain = b"the quick brown fox jumps over the lazy dog 1234567890 abcdef!!\x00";
        let enc = encrypt(plain, &CASTLE_OUTER_KEY);
        let dec = decrypt(&enc, &CASTLE_OUTER_KEY);
        assert_eq!(from_u32_array(&dec), plain);
    }
}
