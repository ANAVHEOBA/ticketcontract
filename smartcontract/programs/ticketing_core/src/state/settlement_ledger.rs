use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct SettlementLedger {
    pub bump: u8,
    pub event: Pubkey,
    pub organizer: Pubkey,
    pub financing_offer: Pubkey,
    pub cumulative_primary_routed_lamports: u64,
    pub cumulative_secondary_routed_lamports: u64,
    pub cumulative_financier_paid_lamports: u64,
    pub cumulative_organizer_paid_lamports: u64,
    pub cumulative_protocol_paid_lamports: u64,
    pub cumulative_royalty_paid_lamports: u64,
    pub cumulative_other_paid_lamports: u64,
    pub financing_settled: bool,
    pub settled_at: i64,
    pub created_at: i64,
    pub updated_at: i64,
}
