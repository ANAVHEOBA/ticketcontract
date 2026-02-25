use anchor_lang::prelude::*;

#[account]
pub struct ProtocolConfig {
    pub bump: u8,
    pub admin: Pubkey,
    pub upgrade_authority: Pubkey,
    pub treasury_vault: Pubkey,
    pub fee_vault: Pubkey,
    pub protocol_fee_bps: u16,
    pub loyalty_multiplier_bps: u16,
    pub max_tickets_per_wallet: u16,
    pub is_paused: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

impl ProtocolConfig {
    pub const INIT_SPACE: usize = 1 + (32 * 4) + 2 + 2 + 2 + 1 + 8 + 8;
}
