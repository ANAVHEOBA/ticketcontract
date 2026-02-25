use anchor_lang::prelude::*;

#[account]
pub struct RoleBinding {
    pub bump: u8,
    pub role: u8,
    pub scope: u8,
    pub active: bool,
    pub target: Pubkey,
    pub subject: Pubkey,
    pub granted_by: Pubkey,
    pub starts_at: i64,
    pub expires_at: i64,
    pub revoked_at: i64,
    pub last_audit_reference: [u8; 16],
    pub last_correlation_id: [u8; 16],
    pub created_at: i64,
    pub updated_at: i64,
}

impl RoleBinding {
    pub const INIT_SPACE: usize = 1 + 1 + 1 + 1 + 32 + 32 + 32 + 8 + 8 + 8 + 16 + 16 + 8 + 8;
}
