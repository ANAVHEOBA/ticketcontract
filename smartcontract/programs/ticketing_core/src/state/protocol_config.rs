use anchor_lang::prelude::*;

#[account]
pub struct ProtocolConfig {
    pub bump: u8,
    pub admin: Pubkey,
    pub upgrade_authority: Pubkey,
    pub pending_upgrade_authority: Pubkey,
    pub upgrade_handoff_started_at: i64,
    pub upgrade_handoff_eta: i64,
    pub timelock_delay_secs: i64,
    pub pending_protocol_fee_bps: u16,
    pub pending_max_tickets_per_wallet: u16,
    pub config_change_eta: i64,
    pub multisig_enabled: bool,
    pub multisig_threshold: u8,
    pub multisig_signer_1: Pubkey,
    pub multisig_signer_2: Pubkey,
    pub multisig_signer_3: Pubkey,
    pub emergency_admin: Pubkey,
    pub emergency_action_nonce: u64,
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
    pub const INIT_SPACE: usize = 1
        + (32 * 9)
        + (8 * 6)
        + (2 * 4)
        + 1
        + 1
        + 1
        + 8
        + 8;
}
