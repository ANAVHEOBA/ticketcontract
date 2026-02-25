use anchor_lang::prelude::*;

#[event]
pub struct ProtocolInitialized {
    pub admin: Pubkey,
    pub upgrade_authority: Pubkey,
    pub treasury_vault: Pubkey,
    pub fee_vault: Pubkey,
}

#[event]
pub struct ProtocolConfigUpdated {
    pub admin: Pubkey,
    pub protocol_fee_bps: u16,
    pub max_tickets_per_wallet: u16,
    pub is_paused: bool,
}

#[event]
pub struct ProtocolAuthoritiesUpdated {
    pub old_admin: Pubkey,
    pub new_admin: Pubkey,
    pub new_upgrade_authority: Pubkey,
}

#[event]
pub struct ProtocolVaultsUpdated {
    pub admin: Pubkey,
    pub treasury_vault: Pubkey,
    pub fee_vault: Pubkey,
}

#[event]
pub struct ProtocolUpgradeHandoffStarted {
    pub admin: Pubkey,
    pub current_upgrade_authority: Pubkey,
    pub pending_upgrade_authority: Pubkey,
    pub ready_at: i64,
    pub at: i64,
}

#[event]
pub struct ProtocolUpgradeAuthorityAccepted {
    pub previous_upgrade_authority: Pubkey,
    pub new_upgrade_authority: Pubkey,
    pub accepted_by: Pubkey,
    pub at: i64,
}

#[event]
pub struct ProtocolMultisigConfigUpdated {
    pub admin: Pubkey,
    pub enabled: bool,
    pub threshold: u8,
    pub signer_1: Pubkey,
    pub signer_2: Pubkey,
    pub signer_3: Pubkey,
    pub at: i64,
}

#[event]
pub struct ProtocolTimelockUpdated {
    pub admin: Pubkey,
    pub timelock_delay_secs: i64,
    pub at: i64,
}

#[event]
pub struct ProtocolConfigChangeQueued {
    pub admin: Pubkey,
    pub pending_protocol_fee_bps: u16,
    pub pending_max_tickets_per_wallet: u16,
    pub execute_at: i64,
    pub at: i64,
}

#[event]
pub struct ProtocolConfigChangeExecuted {
    pub admin: Pubkey,
    pub protocol_fee_bps: u16,
    pub max_tickets_per_wallet: u16,
    pub at: i64,
}

#[event]
pub struct ProtocolEmergencyAdminAction {
    pub emergency_admin: Pubkey,
    pub old_admin: Pubkey,
    pub new_admin: Pubkey,
    pub new_emergency_admin: Pubkey,
    pub nonce: u64,
    pub reason_code: u16,
    pub at: i64,
}

#[event]
pub struct TicketPurchased {
    pub event: Pubkey,
    pub ticket_class: Pubkey,
    pub ticket: Pubkey,
    pub buyer: Pubkey,
    pub owner: Pubkey,
    pub ticket_id: u32,
    pub amount_paid: u64,
}

#[event]
pub struct TicketCompIssued {
    pub event: Pubkey,
    pub ticket_class: Pubkey,
    pub ticket: Pubkey,
    pub recipient: Pubkey,
    pub issuer: Pubkey,
    pub ticket_id: u32,
}

#[event]
pub struct TicketStatusTransitioned {
    pub event: Pubkey,
    pub ticket_class: Pubkey,
    pub ticket: Pubkey,
    pub old_status: u8,
    pub new_status: u8,
    pub authority: Pubkey,
    pub at: i64,
}

#[event]
pub struct TicketMetadataUpdated {
    pub event: Pubkey,
    pub ticket_class: Pubkey,
    pub ticket: Pubkey,
    pub metadata_uri: String,
    pub metadata_version: u16,
    pub authority: Pubkey,
    pub at: i64,
}

#[event]
pub struct ResalePolicySet {
    pub event: Pubkey,
    pub ticket_class: Pubkey,
    pub authority: Pubkey,
}

#[event]
pub struct TicketListed {
    pub event: Pubkey,
    pub ticket_class: Pubkey,
    pub ticket: Pubkey,
    pub listing: Pubkey,
    pub seller: Pubkey,
    pub price_lamports: u64,
    pub expires_at: i64,
}

#[event]
pub struct TicketResold {
    pub event: Pubkey,
    pub ticket_class: Pubkey,
    pub ticket: Pubkey,
    pub listing: Pubkey,
    pub seller: Pubkey,
    pub buyer: Pubkey,
    pub price_lamports: u64,
    pub royalty_amount: u64,
    pub seller_amount: u64,
}

#[event]
pub struct ResaleSettlement {
    pub event: Pubkey,
    pub ticket_class: Pubkey,
    pub ticket: Pubkey,
    pub listing: Pubkey,
    pub price_lamports: u64,
    pub seller_amount: u64,
    pub royalty_amount: u64,
}

#[event]
pub struct ListingCanceled {
    pub event: Pubkey,
    pub ticket_class: Pubkey,
    pub ticket: Pubkey,
    pub listing: Pubkey,
    pub seller: Pubkey,
    pub reason: u8,
}

#[event]
pub struct ListingExpired {
    pub event: Pubkey,
    pub ticket_class: Pubkey,
    pub ticket: Pubkey,
    pub listing: Pubkey,
    pub seller: Pubkey,
}

#[event]
pub struct FinancingOfferCreated {
    pub event: Pubkey,
    pub organizer: Pubkey,
    pub financing_offer: Pubkey,
    pub authority: Pubkey,
    pub advance_amount_lamports: u64,
    pub fee_bps: u16,
    pub repayment_cap_lamports: u64,
}

#[event]
pub struct FinancingOfferDecisioned {
    pub event: Pubkey,
    pub organizer: Pubkey,
    pub financing_offer: Pubkey,
    pub authority: Pubkey,
    pub accepted: bool,
}

#[event]
pub struct FinancingAdvanceDisbursed {
    pub event: Pubkey,
    pub organizer: Pubkey,
    pub financing_offer: Pubkey,
    pub disburser: Pubkey,
    pub organizer_payout_wallet: Pubkey,
    pub amount_lamports: u64,
}

#[event]
pub struct FinancingDisbursementRecorded {
    pub event: Pubkey,
    pub organizer: Pubkey,
    pub financing_offer: Pubkey,
    pub disbursement_record: Pubkey,
    pub disbursement_index: u16,
    pub amount_lamports: u64,
    pub reference_id: [u8; 16],
    pub disburser: Pubkey,
    pub destination_wallet: Pubkey,
    pub at: i64,
}

#[event]
pub struct FinancingFreezeUpdated {
    pub event: Pubkey,
    pub organizer: Pubkey,
    pub financing_offer: Pubkey,
    pub admin: Pubkey,
    pub is_frozen: bool,
    pub reason_code: u16,
    pub clawback_allowed: bool,
    pub at: i64,
}

#[event]
pub struct FinancingClawbackExecuted {
    pub event: Pubkey,
    pub organizer: Pubkey,
    pub financing_offer: Pubkey,
    pub disbursement_record: Pubkey,
    pub admin: Pubkey,
    pub treasury_vault: Pubkey,
    pub amount_lamports: u64,
    pub at: i64,
}

#[event]
pub struct RevenueWaterfallSettled {
    pub event: Pubkey,
    pub organizer: Pubkey,
    pub financing_offer: Pubkey,
    pub settlement_ledger: Pubkey,
    pub source_wallet: Pubkey,
    pub gross_revenue_lamports: u64,
    pub primary_revenue: bool,
    pub financier_amount: u64,
    pub organizer_amount: u64,
    pub protocol_amount: u64,
    pub royalty_amount: u64,
    pub other_amount: u64,
    pub correlation_id: [u8; 16],
    pub at: i64,
}

#[event]
pub struct FinancingSettled {
    pub event: Pubkey,
    pub organizer: Pubkey,
    pub financing_offer: Pubkey,
    pub settlement_ledger: Pubkey,
    pub correlation_id: [u8; 16],
    pub settled_at: i64,
}

#[event]
pub struct FinancialDistributionLegSettled {
    pub settlement_ledger: Pubkey,
    pub source_wallet: Pubkey,
    pub destination_wallet: Pubkey,
    pub leg_type: u8,
    pub amount_lamports: u64,
    pub correlation_id: [u8; 16],
    pub at: i64,
}

#[event]
pub struct CheckInPolicyUpdated {
    pub event: Pubkey,
    pub ticket_class: Pubkey,
    pub authority: Pubkey,
    pub allow_reentry: bool,
    pub max_reentries: u8,
    pub at: i64,
}

#[event]
pub struct TicketAttendanceRecorded {
    pub event: Pubkey,
    pub ticket_class: Pubkey,
    pub ticket: Pubkey,
    pub owner: Pubkey,
    pub scanner: Pubkey,
    pub gate_identifier: String,
    pub check_in_count: u16,
    pub is_reentry: bool,
    pub at: i64,
}

#[event]
pub struct TicketRefunded {
    pub event: Pubkey,
    pub ticket_class: Pubkey,
    pub ticket: Pubkey,
    pub authority: Pubkey,
    pub recipient: Pubkey,
    pub amount_lamports: u64,
    pub source: u8,
    pub at: i64,
}

#[event]
pub struct TicketDisputeFlagged {
    pub event: Pubkey,
    pub ticket_class: Pubkey,
    pub ticket: Pubkey,
    pub authority: Pubkey,
    pub is_disputed: bool,
    pub is_chargeback: bool,
    pub reason_code: u16,
    pub at: i64,
}

#[event]
pub struct LoyaltyPointsAccrued {
    pub wallet: Pubkey,
    pub loyalty_ledger: Pubkey,
    pub event: Pubkey,
    pub ticket: Pubkey,
    pub reason: u8,
    pub base_points: u64,
    pub points_credited: u64,
    pub global_multiplier_bps: u16,
    pub event_multiplier_bps: u16,
    pub available_points: u64,
    pub total_accrued_points: u64,
    pub at: i64,
}

#[event]
pub struct LoyaltyPointsRedeemed {
    pub wallet: Pubkey,
    pub loyalty_ledger: Pubkey,
    pub event: Pubkey,
    pub points_burned: u64,
    pub perk_code: String,
    pub available_points: u64,
    pub total_redeemed_points: u64,
    pub at: i64,
}

#[event]
pub struct LoyaltyGlobalMultiplierUpdated {
    pub admin: Pubkey,
    pub multiplier_bps: u16,
    pub at: i64,
}

#[event]
pub struct LoyaltyEventMultiplierUpdated {
    pub organizer: Pubkey,
    pub event: Pubkey,
    pub multiplier_bps: u16,
    pub at: i64,
}

#[event]
pub struct TrustSignalUpdated {
    pub wallet: Pubkey,
    pub trust_signal: Pubkey,
    pub event: Pubkey,
    pub ticket: Pubkey,
    pub schema_version: u16,
    pub update_type: u8,
    pub total_tickets_purchased: u32,
    pub attendance_eligible_count: u32,
    pub attendance_attended_count: u32,
    pub abuse_flags: u32,
    pub abuse_incidents: u16,
    pub at: i64,
}

#[event]
pub struct TrustSignalSchemaVersionUpdated {
    pub admin: Pubkey,
    pub wallet: Pubkey,
    pub trust_signal: Pubkey,
    pub old_schema_version: u16,
    pub new_schema_version: u16,
    pub at: i64,
}

#[event]
pub struct RoleGranted {
    pub role_binding: Pubkey,
    pub role: u8,
    pub scope: u8,
    pub target: Pubkey,
    pub subject: Pubkey,
    pub granter: Pubkey,
    pub starts_at: i64,
    pub expires_at: i64,
    pub audit_reference: [u8; 16],
    pub correlation_id: [u8; 16],
    pub at: i64,
}

#[event]
pub struct RoleRevoked {
    pub role_binding: Pubkey,
    pub role: u8,
    pub scope: u8,
    pub target: Pubkey,
    pub subject: Pubkey,
    pub revoker: Pubkey,
    pub reason_code: u16,
    pub audit_reference: [u8; 16],
    pub correlation_id: [u8; 16],
    pub at: i64,
}

#[event]
pub struct RoleAuthorityRotated {
    pub old_role_binding: Pubkey,
    pub new_role_binding: Pubkey,
    pub role: u8,
    pub scope: u8,
    pub target: Pubkey,
    pub old_subject: Pubkey,
    pub new_subject: Pubkey,
    pub authority: Pubkey,
    pub audit_reference: [u8; 16],
    pub correlation_id: [u8; 16],
    pub at: i64,
}

#[event]
pub struct GovernanceAuditReferenceStored {
    pub role_binding: Pubkey,
    pub target: Pubkey,
    pub subject: Pubkey,
    pub role: u8,
    pub scope: u8,
    pub audit_reference: [u8; 16],
    pub correlation_id: [u8; 16],
    pub at: i64,
}

#[event]
pub struct EventStateTransitioned {
    pub event: Pubkey,
    pub organizer: Pubkey,
    pub authority: Pubkey,
    pub old_status: u8,
    pub new_status: u8,
    pub is_paused: bool,
    pub correlation_id: [u8; 16],
    pub at: i64,
}

#[event]
pub struct EventMetadataUpdated {
    pub event: Pubkey,
    pub organizer: Pubkey,
    pub authority: Pubkey,
    pub title: String,
    pub venue: String,
    pub start_ts: i64,
    pub end_ts: i64,
    pub sales_start_ts: i64,
    pub lock_ts: i64,
    pub capacity: u32,
    pub correlation_id: [u8; 16],
    pub at: i64,
}

#[event]
pub struct VaultInitialized {
    pub vault: Pubkey,
    pub kind: u8,
    pub parent: Pubkey,
    pub controller: Pubkey,
    pub authority: Pubkey,
    pub at: i64,
}

#[event]
pub struct VaultSnapshotRecorded {
    pub vault: Pubkey,
    pub kind: u8,
    pub parent: Pubkey,
    pub balance_lamports: u64,
    pub total_inflow_lamports: u64,
    pub total_outflow_lamports: u64,
    pub at: i64,
}

#[event]
pub struct VaultWithdrawn {
    pub vault: Pubkey,
    pub kind: u8,
    pub parent: Pubkey,
    pub destination: Pubkey,
    pub authority: Pubkey,
    pub amount_lamports: u64,
    pub balance_lamports: u64,
    pub total_outflow_lamports: u64,
    pub at: i64,
}

#[event]
pub struct ComplianceRegistryUpdated {
    pub registry: Pubkey,
    pub scope: u8,
    pub target: Pubkey,
    pub subject: Pubkey,
    pub list_type: u8,
    pub is_allowed: bool,
    pub decision_code: u16,
    pub authority: Pubkey,
    pub at: i64,
}

#[event]
pub struct EventComplianceFlagsUpdated {
    pub event: Pubkey,
    pub organizer: Pubkey,
    pub authority: Pubkey,
    pub restriction_flags: u32,
    pub at: i64,
}
