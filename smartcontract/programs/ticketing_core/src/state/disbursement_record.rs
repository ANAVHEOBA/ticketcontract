use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct DisbursementRecord {
    pub bump: u8,
    pub financing_offer: Pubkey,
    pub disbursement_index: u16,
    pub amount_lamports: u64,
    pub executed_by: Pubkey,
    pub destination_wallet: Pubkey,
    pub reference_id: [u8; 16],
    pub executed_at: i64,
    pub clawed_back: bool,
    pub clawed_back_at: i64,
}
