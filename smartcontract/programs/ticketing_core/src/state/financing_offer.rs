use anchor_lang::prelude::*;

use crate::error::TicketingError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
#[repr(u8)]
pub enum FinancingLifecycleStatus {
    Draft = 1,
    Accepted = 2,
    Rejected = 3,
    Disbursed = 4,
    PartiallyDisbursed = 5,
    Settled = 6,
}

impl FinancingLifecycleStatus {
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            1 => Ok(Self::Draft),
            2 => Ok(Self::Accepted),
            3 => Ok(Self::Rejected),
            4 => Ok(Self::Disbursed),
            5 => Ok(Self::PartiallyDisbursed),
            6 => Ok(Self::Settled),
            _ => err!(TicketingError::InvalidFinancingStatus),
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, InitSpace)]
pub struct FinancingOfferInput {
    pub advance_amount_lamports: u64,
    pub fee_bps: u16,
    pub repayment_cap_lamports: u64,
    pub schedule_start_ts: i64,
    pub schedule_interval_secs: i64,
    pub schedule_installments: u16,
}

#[account]
#[derive(InitSpace)]
pub struct FinancingOffer {
    pub bump: u8,
    pub schema_version: u16,
    pub deprecated_layout_version: u16,
    pub replacement_account: Pubkey,
    pub deprecated_at: i64,
    pub event: Pubkey,
    pub organizer: Pubkey,
    pub offer_authority: Pubkey,
    pub advance_amount_lamports: u64,
    pub fee_bps: u16,
    pub repayment_cap_lamports: u64,
    pub schedule_start_ts: i64,
    pub schedule_interval_secs: i64,
    pub schedule_installments: u16,
    pub max_disbursements: u16,
    pub status: FinancingLifecycleStatus,
    pub terms_locked: bool,
    pub financing_frozen: bool,
    pub clawback_allowed: bool,
    pub freeze_reason_code: u16,
    pub accepted_by: Pubkey,
    pub accepted_at: i64,
    pub rejected_by: Pubkey,
    pub rejected_at: i64,
    pub total_disbursed_lamports: u64,
    pub disbursement_count: u16,
    pub disbursed_at: i64,
    pub compliance_decision_code: u16,
    pub compliance_checked_at: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

impl FinancingOffer {
    pub fn mark_layout_deprecated(
        &mut self,
        deprecated_layout_version: u16,
        replacement_account: Pubkey,
        now: i64,
    ) {
        self.deprecated_layout_version = deprecated_layout_version;
        self.replacement_account = replacement_account;
        self.deprecated_at = now;
    }
}
