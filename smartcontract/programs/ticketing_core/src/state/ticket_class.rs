use anchor_lang::prelude::*;

#[account]
pub struct TicketClass {
    pub bump: u8,
    pub event: Pubkey,
    pub class_id: u16,
    pub name: String,
    pub total_supply: u32,
    pub reserved_supply: u32,
    pub sold_supply: u32,
    pub refunded_supply: u32,
    pub remaining_supply: u32,
    pub face_price_lamports: u64,
    pub sale_start_ts: i64,
    pub sale_end_ts: i64,
    pub per_wallet_limit: u16,
    pub is_transferable: bool,
    pub is_resale_enabled: bool,
    pub allow_reentry: bool,
    pub max_reentries: u8,
    pub stakeholder_wallet: Pubkey,
    pub stakeholder_bps: u16,
    pub created_at: i64,
    pub updated_at: i64,
}

impl TicketClass {
    pub const INIT_SPACE: usize = 1
        + 32
        + 2
        + (4 + crate::constants::MAX_TICKET_CLASS_NAME_LEN)
        + 4
        + 4
        + 4
        + 4
        + 4
        + 8
        + 8
        + 8
        + 2
        + 1
        + 1
        + 1
        + 1
        + 32
        + 2
        + 8
        + 8;
}
