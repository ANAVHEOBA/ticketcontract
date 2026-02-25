use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Listing {
    pub bump: u8,
    pub event: Pubkey,
    pub ticket_class: Pubkey,
    pub ticket: Pubkey,
    pub seller: Pubkey,
    pub price_lamports: u64,
    pub expires_at: i64,
    pub is_active: bool,
    pub close_reason: u8,
    pub created_at: i64,
    pub closed_at: i64,
    pub updated_at: i64,
}
