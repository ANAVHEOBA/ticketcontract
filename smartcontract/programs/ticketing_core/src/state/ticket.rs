use anchor_lang::prelude::*;

use crate::{
    constants::{MAX_GATE_IDENTIFIER_LEN, MAX_TICKET_METADATA_URI_LEN},
    error::TicketingError,
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
#[repr(u8)]
pub enum TicketStatus {
    Active = 1,
    CheckedIn = 2,
    Refunded = 3,
    Invalidated = 4,
}

impl TicketStatus {
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            1 => Ok(Self::Active),
            2 => Ok(Self::CheckedIn),
            3 => Ok(Self::Refunded),
            4 => Ok(Self::Invalidated),
            _ => err!(TicketingError::InvalidTicketStatus),
        }
    }

    pub fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Active, Self::CheckedIn)
                | (Self::Active, Self::Refunded)
                | (Self::Active, Self::Invalidated)
        )
    }
}

#[account]
#[derive(InitSpace)]
pub struct Ticket {
    pub bump: u8,
    pub event: Pubkey,
    pub ticket_class: Pubkey,
    pub owner: Pubkey,
    pub buyer: Pubkey,
    pub ticket_id: u32,
    pub status: TicketStatus,
    pub paid_amount_lamports: u64,
    pub is_comp: bool,
    pub created_at: i64,
    pub status_updated_at: i64,
    pub checked_in_at: i64,
    pub last_check_in_at: i64,
    pub check_in_count: u16,
    #[max_len(MAX_GATE_IDENTIFIER_LEN)]
    pub last_check_in_gate_id: String,
    pub refunded_at: i64,
    pub refund_source: u8,
    pub refund_amount_lamports: u64,
    pub invalidated_at: i64,
    pub is_disputed: bool,
    pub is_chargeback: bool,
    pub disputed_at: i64,
    pub dispute_reason_code: u16,
    pub dispute_updated_at: i64,
    #[max_len(MAX_TICKET_METADATA_URI_LEN)]
    pub metadata_uri: String,
    pub metadata_version: u16,
    pub metadata_updated_at: i64,
    pub transfer_count: u16,
    pub last_transfer_at: i64,
    pub purchase_trust_recorded: bool,
    pub attendance_trust_recorded: bool,
}

#[account]
#[derive(InitSpace)]
pub struct WalletPurchaseCounter {
    pub bump: u8,
    pub event: Pubkey,
    pub ticket_class: Pubkey,
    pub wallet: Pubkey,
    pub purchased_count: u16,
    pub updated_at: i64,
}
