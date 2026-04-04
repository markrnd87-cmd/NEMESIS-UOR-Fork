//! Content address computation via FNV-1a hash.
//!
//! Deterministic, computed at macro expansion time, stored as `u64`
//! in the emitted const.

/// FNV-1a offset basis for 64-bit.
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;

/// FNV-1a prime for 64-bit.
const FNV_PRIME: u64 = 0x0100_0000_01b3;

/// Compute the FNV-1a hash of a byte slice.
pub fn fnv1a(data: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET;
    for &byte in data {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Compute the content address of a surface syntax string.
pub fn content_address(surface: &str) -> u64 {
    fnv1a(surface.as_bytes())
}
