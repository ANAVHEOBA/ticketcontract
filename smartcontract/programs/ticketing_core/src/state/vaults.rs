use anchor_lang::prelude::*;

#[account]
pub struct VaultAccount {
    pub bump: u8,
    pub vault_bump: u8,
    pub kind: u8,
    pub parent: Pubkey,
    pub vault: Pubkey,
    pub controller: Pubkey,
    pub authority: Pubkey,
    pub last_recorded_balance_lamports: u64,
    pub total_inflow_lamports: u64,
    pub total_outflow_lamports: u64,
    pub created_at: i64,
    pub updated_at: i64,
}

impl VaultAccount {
    pub const INIT_SPACE: usize = 1 + 1 + 1 + 32 + 32 + 32 + 32 + 8 + 8 + 8 + 8 + 8;
}
