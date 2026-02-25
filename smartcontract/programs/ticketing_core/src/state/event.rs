use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum EventStatus {
    Draft = 1,
    Frozen = 2,
    Cancelled = 3,
    Closed = 4,
}

#[account]
pub struct EventAccount {
    pub bump: u8,
    pub organizer: Pubkey,
    pub event_id: u64,
    pub title: String,
    pub venue: String,
    pub start_ts: i64,
    pub end_ts: i64,
    pub sales_start_ts: i64,
    pub lock_ts: i64,
    pub capacity: u32,
    pub loyalty_multiplier_bps: u16,
    pub status: EventStatus,
    pub created_at: i64,
    pub updated_at: i64,
}

impl EventAccount {
    pub const INIT_SPACE: usize = 1
        + 32
        + 8
        + (4 + crate::constants::MAX_EVENT_TITLE_LEN)
        + (4 + crate::constants::MAX_EVENT_VENUE_LEN)
        + 8
        + 8
        + 8
        + 8
        + 4
        + 2
        + 1
        + 8
        + 8;
}
