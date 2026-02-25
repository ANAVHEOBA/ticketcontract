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
    pub schema_version: u16,
    pub deprecated_layout_version: u16,
    pub replacement_account: Pubkey,
    pub deprecated_at: i64,
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
    pub compliance_restriction_flags: u32,
    pub is_paused: bool,
    pub status: EventStatus,
    pub created_at: i64,
    pub updated_at: i64,
}

impl EventAccount {
    pub const INIT_SPACE: usize = 1
        + 2
        + 2
        + 32
        + 8
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
        + 4
        + 1
        + 1
        + 8
        + 8;

    pub fn mark_layout_deprecated(&mut self, deprecated_layout_version: u16, replacement_account: Pubkey, now: i64) {
        self.deprecated_layout_version = deprecated_layout_version;
        self.replacement_account = replacement_account;
        self.deprecated_at = now;
    }
}
