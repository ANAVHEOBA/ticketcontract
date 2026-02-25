use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum OrganizerStatus {
    Active = 1,
    Suspended = 2,
}

#[account]
pub struct OrganizerProfile {
    pub bump: u8,
    pub authority: Pubkey,
    pub payout_wallet: Pubkey,
    pub status: OrganizerStatus,
    pub compliance_flags: u32,
    pub metadata_uri: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl OrganizerProfile {
    pub const INIT_SPACE: usize =
        1 + 32 + 32 + 1 + 4 + 4 + crate::constants::MAX_ORGANIZER_METADATA_URI_LEN + 8 + 8;
}

#[account]
pub struct OrganizerOperator {
    pub bump: u8,
    pub organizer: Pubkey,
    pub operator: Pubkey,
    pub permissions: u32,
    pub active: bool,
    pub updated_at: i64,
}

impl OrganizerOperator {
    pub const INIT_SPACE: usize = 1 + 32 + 32 + 4 + 1 + 8;
}
