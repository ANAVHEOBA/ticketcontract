use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct LoyaltyLedger {
    pub bump: u8,
    pub schema_version: u16,
    pub deprecated_layout_version: u16,
    pub replacement_account: Pubkey,
    pub deprecated_at: i64,
    pub wallet: Pubkey,
    pub total_accrued_points: u64,
    pub total_redeemed_points: u64,
    pub available_points: u64,
    pub last_event: Pubkey,
    pub last_reason: u8,
    pub last_accrued_at: i64,
    pub last_redeemed_at: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

impl LoyaltyLedger {
    pub fn mark_layout_deprecated(
        &mut self,
        deprecated_layout_version: u16,
        replacement_account: Pubkey,
        now: i64,
    ) {
        self.deprecated_layout_version = deprecated_layout_version;
        self.replacement_account = replacement_account;
        self.deprecated_at = now;
    }
}
