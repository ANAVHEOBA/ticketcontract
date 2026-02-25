use anchor_lang::prelude::*;

pub fn derive_correlation_id(
    primary: &Pubkey,
    secondary: &Pubkey,
    at: i64,
    discriminator: u16,
) -> [u8; 16] {
    let mut out = [0u8; 16];
    let a = primary.to_bytes();
    let b = secondary.to_bytes();
    let t = at.to_le_bytes();
    let d = discriminator.to_le_bytes();

    for i in 0..16 {
        out[i] = a[i] ^ a[i + 16] ^ b[i] ^ b[i + 16] ^ t[i % 8] ^ d[i % 2];
    }
    out
}
