use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct TrustSignal {
    pub bump: u8,
    pub wallet: Pubkey,
    pub schema_version: u16,
    pub total_tickets_purchased: u32,
    pub attendance_eligible_count: u32,
    pub attendance_attended_count: u32,
    pub abuse_flags: u32,
    pub abuse_incidents: u16,
    pub last_event: Pubkey,
    pub last_ticket: Pubkey,
    pub created_at: i64,
    pub updated_at: i64,
}
