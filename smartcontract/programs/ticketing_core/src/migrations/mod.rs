use anchor_lang::{prelude::*, Discriminator};

use crate::{
    constants::SCHEMA_VERSION_V0,
    error::TicketingError,
    state::{
        EventAccount, EventStatus, FinancingLifecycleStatus, FinancingOffer, LoyaltyLedger,
        ResalePolicy, Ticket, TicketStatus,
    },
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct EventAccountV0 {
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
    pub compliance_restriction_flags: u32,
    pub is_paused: bool,
    pub status: EventStatus,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct TicketV0 {
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
    pub metadata_uri: String,
    pub metadata_version: u16,
    pub metadata_updated_at: i64,
    pub transfer_count: u16,
    pub last_transfer_at: i64,
    pub compliance_decision_code: u16,
    pub compliance_checked_at: i64,
    pub purchase_trust_recorded: bool,
    pub attendance_trust_recorded: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct ResalePolicyV0 {
    pub bump: u8,
    pub event: Pubkey,
    pub ticket_class: Pubkey,
    pub class_id: u16,
    pub max_markup_bps: u16,
    pub royalty_bps: u16,
    pub royalty_vault: Pubkey,
    pub transfer_cooldown_secs: i64,
    pub max_transfer_count: u16,
    pub transfer_lock_before_event_secs: i64,
    pub whitelist: Vec<Pubkey>,
    pub blacklist: Vec<Pubkey>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct FinancingOfferV0 {
    pub bump: u8,
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

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct LoyaltyLedgerV0 {
    pub bump: u8,
    pub wallet: Pubkey,
    pub total_accrued_points: u64,
    pub total_redeemed_points: u64,
    pub available_points: u64,
    pub last_event: Pubkey,
    pub last_reason: u8,
    pub last_accrued_at: i64,
    pub last_redeemed_at: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

fn payload_without_discriminator<T: Discriminator>(raw_data: &[u8]) -> &[u8] {
    if raw_data.len() >= T::DISCRIMINATOR.len()
        && &raw_data[..T::DISCRIMINATOR.len()] == T::DISCRIMINATOR
    {
        &raw_data[T::DISCRIMINATOR.len()..]
    } else {
        raw_data
    }
}

pub fn deserialize_event_account_compat(raw_data: &[u8]) -> Result<EventAccount> {
    let payload = payload_without_discriminator::<EventAccount>(raw_data);
    if let Ok(account) = EventAccount::try_from_slice(payload) {
        return Ok(account);
    }
    if let Ok(v0) = EventAccountV0::try_from_slice(payload) {
        return Ok(EventAccount {
            bump: v0.bump,
            schema_version: SCHEMA_VERSION_V0,
            deprecated_layout_version: 0,
            replacement_account: Pubkey::default(),
            deprecated_at: 0,
            organizer: v0.organizer,
            event_id: v0.event_id,
            title: v0.title,
            venue: v0.venue,
            start_ts: v0.start_ts,
            end_ts: v0.end_ts,
            sales_start_ts: v0.sales_start_ts,
            lock_ts: v0.lock_ts,
            capacity: v0.capacity,
            loyalty_multiplier_bps: v0.loyalty_multiplier_bps,
            compliance_restriction_flags: v0.compliance_restriction_flags,
            is_paused: v0.is_paused,
            status: v0.status,
            created_at: v0.created_at,
            updated_at: v0.updated_at,
        });
    }
    err!(TicketingError::AccountSizeMismatch)
}

pub fn deserialize_ticket_compat(raw_data: &[u8]) -> Result<Ticket> {
    let payload = payload_without_discriminator::<Ticket>(raw_data);
    if let Ok(account) = Ticket::try_from_slice(payload) {
        return Ok(account);
    }
    if let Ok(v0) = TicketV0::try_from_slice(payload) {
        return Ok(Ticket {
            bump: v0.bump,
            schema_version: SCHEMA_VERSION_V0,
            deprecated_layout_version: 0,
            replacement_account: Pubkey::default(),
            deprecated_at: 0,
            event: v0.event,
            ticket_class: v0.ticket_class,
            owner: v0.owner,
            buyer: v0.buyer,
            ticket_id: v0.ticket_id,
            status: v0.status,
            paid_amount_lamports: v0.paid_amount_lamports,
            is_comp: v0.is_comp,
            created_at: v0.created_at,
            status_updated_at: v0.status_updated_at,
            checked_in_at: v0.checked_in_at,
            last_check_in_at: v0.last_check_in_at,
            check_in_count: v0.check_in_count,
            last_check_in_gate_id: v0.last_check_in_gate_id,
            refunded_at: v0.refunded_at,
            refund_source: v0.refund_source,
            refund_amount_lamports: v0.refund_amount_lamports,
            invalidated_at: v0.invalidated_at,
            is_disputed: v0.is_disputed,
            is_chargeback: v0.is_chargeback,
            disputed_at: v0.disputed_at,
            dispute_reason_code: v0.dispute_reason_code,
            dispute_updated_at: v0.dispute_updated_at,
            metadata_uri: v0.metadata_uri,
            metadata_version: v0.metadata_version,
            metadata_updated_at: v0.metadata_updated_at,
            transfer_count: v0.transfer_count,
            last_transfer_at: v0.last_transfer_at,
            compliance_decision_code: v0.compliance_decision_code,
            compliance_checked_at: v0.compliance_checked_at,
            purchase_trust_recorded: v0.purchase_trust_recorded,
            attendance_trust_recorded: v0.attendance_trust_recorded,
        });
    }
    err!(TicketingError::AccountSizeMismatch)
}

pub fn deserialize_resale_policy_compat(raw_data: &[u8]) -> Result<ResalePolicy> {
    let payload = payload_without_discriminator::<ResalePolicy>(raw_data);
    if let Ok(account) = ResalePolicy::try_from_slice(payload) {
        return Ok(account);
    }
    if let Ok(v0) = ResalePolicyV0::try_from_slice(payload) {
        return Ok(ResalePolicy {
            bump: v0.bump,
            schema_version: SCHEMA_VERSION_V0,
            deprecated_layout_version: 0,
            replacement_account: Pubkey::default(),
            deprecated_at: 0,
            event: v0.event,
            ticket_class: v0.ticket_class,
            class_id: v0.class_id,
            max_markup_bps: v0.max_markup_bps,
            royalty_bps: v0.royalty_bps,
            royalty_vault: v0.royalty_vault,
            transfer_cooldown_secs: v0.transfer_cooldown_secs,
            max_transfer_count: v0.max_transfer_count,
            transfer_lock_before_event_secs: v0.transfer_lock_before_event_secs,
            whitelist: v0.whitelist,
            blacklist: v0.blacklist,
            created_at: v0.created_at,
            updated_at: v0.updated_at,
        });
    }
    err!(TicketingError::AccountSizeMismatch)
}

pub fn deserialize_financing_offer_compat(raw_data: &[u8]) -> Result<FinancingOffer> {
    let payload = payload_without_discriminator::<FinancingOffer>(raw_data);
    if let Ok(account) = FinancingOffer::try_from_slice(payload) {
        return Ok(account);
    }
    if let Ok(v0) = FinancingOfferV0::try_from_slice(payload) {
        return Ok(FinancingOffer {
            bump: v0.bump,
            schema_version: SCHEMA_VERSION_V0,
            deprecated_layout_version: 0,
            replacement_account: Pubkey::default(),
            deprecated_at: 0,
            event: v0.event,
            organizer: v0.organizer,
            offer_authority: v0.offer_authority,
            advance_amount_lamports: v0.advance_amount_lamports,
            fee_bps: v0.fee_bps,
            repayment_cap_lamports: v0.repayment_cap_lamports,
            schedule_start_ts: v0.schedule_start_ts,
            schedule_interval_secs: v0.schedule_interval_secs,
            schedule_installments: v0.schedule_installments,
            max_disbursements: v0.max_disbursements,
            status: v0.status,
            terms_locked: v0.terms_locked,
            financing_frozen: v0.financing_frozen,
            clawback_allowed: v0.clawback_allowed,
            freeze_reason_code: v0.freeze_reason_code,
            accepted_by: v0.accepted_by,
            accepted_at: v0.accepted_at,
            rejected_by: v0.rejected_by,
            rejected_at: v0.rejected_at,
            total_disbursed_lamports: v0.total_disbursed_lamports,
            disbursement_count: v0.disbursement_count,
            disbursed_at: v0.disbursed_at,
            compliance_decision_code: v0.compliance_decision_code,
            compliance_checked_at: v0.compliance_checked_at,
            created_at: v0.created_at,
            updated_at: v0.updated_at,
        });
    }
    err!(TicketingError::AccountSizeMismatch)
}

pub fn deserialize_loyalty_ledger_compat(raw_data: &[u8]) -> Result<LoyaltyLedger> {
    let payload = payload_without_discriminator::<LoyaltyLedger>(raw_data);
    if let Ok(account) = LoyaltyLedger::try_from_slice(payload) {
        return Ok(account);
    }
    if let Ok(v0) = LoyaltyLedgerV0::try_from_slice(payload) {
        return Ok(LoyaltyLedger {
            bump: v0.bump,
            schema_version: SCHEMA_VERSION_V0,
            deprecated_layout_version: 0,
            replacement_account: Pubkey::default(),
            deprecated_at: 0,
            wallet: v0.wallet,
            total_accrued_points: v0.total_accrued_points,
            total_redeemed_points: v0.total_redeemed_points,
            available_points: v0.available_points,
            last_event: v0.last_event,
            last_reason: v0.last_reason,
            last_accrued_at: v0.last_accrued_at,
            last_redeemed_at: v0.last_redeemed_at,
            created_at: v0.created_at,
            updated_at: v0.updated_at,
        });
    }
    err!(TicketingError::AccountSizeMismatch)
}
