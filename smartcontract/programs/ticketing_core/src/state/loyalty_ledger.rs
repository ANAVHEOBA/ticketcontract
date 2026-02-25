use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct LoyaltyLedger {
    pub bump: u8,
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
