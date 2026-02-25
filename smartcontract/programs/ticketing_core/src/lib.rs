use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod events;
pub mod instructions;
pub mod math;
pub mod migrations;
pub mod state;
pub mod utils;
pub mod validation;

pub(crate) use instructions::checkin::{
    __client_accounts_check_in_ticket, __client_accounts_set_check_in_policy,
};
pub(crate) use instructions::compliance::{
    __client_accounts_set_event_restrictions, __client_accounts_upsert_registry_entry,
};
pub(crate) use instructions::disputes::{
    __client_accounts_flag_dispute, __client_accounts_refund_ticket,
};
pub(crate) use instructions::event::{
    __client_accounts_cancel_event, __client_accounts_close_event, __client_accounts_create_event,
    __client_accounts_freeze_event, __client_accounts_pause_event, __client_accounts_update_event,
};
pub(crate) use instructions::financing::{
    __client_accounts_accept_financing_offer, __client_accounts_clawback_disbursement,
    __client_accounts_create_financing_offer, __client_accounts_disburse_advance,
    __client_accounts_set_financing_freeze,
};
pub(crate) use instructions::governance::{
    __client_accounts_grant_role, __client_accounts_revoke_role, __client_accounts_rotate_authority,
};
pub(crate) use instructions::loyalty::{
    __client_accounts_accrue_points, __client_accounts_redeem_points,
    __client_accounts_set_event_loyalty_multiplier,
    __client_accounts_set_global_loyalty_multiplier,
};
pub(crate) use instructions::organizer::{
    __client_accounts_create_organizer, __client_accounts_set_organizer_compliance_flags,
    __client_accounts_set_organizer_operator, __client_accounts_set_organizer_status,
    __client_accounts_update_organizer,
};
pub(crate) use instructions::primary_sale::{
    __client_accounts_buy_ticket, __client_accounts_issue_comp_ticket,
};
pub(crate) use instructions::resale::{
    __client_accounts_buy_resale_ticket, __client_accounts_cancel_listing,
    __client_accounts_expire_listing, __client_accounts_list_ticket,
    __client_accounts_set_resale_policy,
};
pub(crate) use instructions::settlement::{
    __client_accounts_finalize_settlement, __client_accounts_settle_primary_revenue,
    __client_accounts_settle_resale_revenue,
};
pub(crate) use instructions::ticket_class::{
    __client_accounts_create_ticket_class, __client_accounts_reserve_inventory,
    __client_accounts_update_ticket_class,
};
pub(crate) use instructions::ticket_state::{
    __client_accounts_set_ticket_metadata, __client_accounts_transition_ticket_status,
};
pub(crate) use instructions::treasury::{
    __client_accounts_initialize_vault, __client_accounts_snapshot_vault,
    __client_accounts_withdraw_vault,
};
pub(crate) use instructions::trust_signal::{
    __client_accounts_flag_abuse, __client_accounts_record_attendance_input,
    __client_accounts_record_purchase_input, __client_accounts_set_schema_version,
};
pub(crate) use instructions::{
    __client_accounts_accept_upgrade_authority_handoff,
    __client_accounts_emergency_admin_action,
    __client_accounts_initialize_protocol, __client_accounts_pause_protocol,
    __client_accounts_register_protocol_vaults, __client_accounts_set_protocol_authorities,
    __client_accounts_set_protocol_config, __client_accounts_set_protocol_governance,
};

use instructions::checkin::check_in_ticket::{CheckInTicket, SetCheckInPolicy};
use instructions::compliance::set_event_restrictions::SetEventRestrictions;
use instructions::compliance::upsert_registry_entry::UpsertRegistryEntry;
use instructions::disputes::flag_dispute::FlagDispute;
use instructions::disputes::refund_ticket::RefundTicket;
use instructions::event::cancel_event::{CancelEvent, CloseEvent};
use instructions::event::create_event::{CreateEvent, EventInput};
use instructions::event::freeze_event::FreezeEvent;
use instructions::event::pause_event::PauseEvent;
use instructions::event::update_event::UpdateEvent;
use instructions::financing::accept_financing_offer::AcceptFinancingOffer;
use instructions::financing::clawback_disbursement::ClawbackDisbursement;
use instructions::financing::create_financing_offer::CreateFinancingOffer;
use instructions::financing::disburse_advance::DisburseAdvance;
use instructions::financing::set_financing_freeze::SetFinancingFreeze;
use instructions::governance::grant_role::GrantRole;
use instructions::governance::revoke_role::RevokeRole;
use instructions::governance::rotate_authority::RotateAuthority;
use instructions::loyalty::accrue_points::{
    AccruePoints, SetEventLoyaltyMultiplier, SetGlobalLoyaltyMultiplier,
};
use instructions::loyalty::redeem_points::RedeemPoints;
use instructions::organizer::create_organizer::CreateOrganizer;
use instructions::organizer::set_operator::SetOrganizerOperator;
use instructions::organizer::update_organizer::{
    SetOrganizerComplianceFlags, SetOrganizerStatus, UpdateOrganizer,
};
use instructions::primary_sale::buy_ticket::BuyTicket;
use instructions::primary_sale::issue_comp_ticket::IssueCompTicket;
use instructions::protocol::initialize_protocol::InitializeProtocol;
use instructions::protocol::pause_protocol::PauseProtocol;
use instructions::protocol::governance_hooks::{
    AcceptUpgradeAuthorityHandoff, EmergencyAdminAction, SetProtocolGovernance,
};
use instructions::protocol::set_protocol_config::{
    RegisterProtocolVaults, SetProtocolAuthorities, SetProtocolConfig,
};
use instructions::resale::buy_resale_ticket::BuyResaleTicket;
use instructions::resale::cancel_listing::CancelListing;
use instructions::resale::expire_listing::ExpireListing;
use instructions::resale::list_ticket::ListTicket;
use instructions::resale::set_resale_policy::SetResalePolicy;
use instructions::settlement::finalize_settlement::FinalizeSettlement;
use instructions::settlement::settle_primary_revenue::SettlePrimaryRevenue;
use instructions::settlement::settle_resale_revenue::SettleResaleRevenue;
use instructions::ticket_class::create_ticket_class::{CreateTicketClass, TicketClassInput};
use instructions::ticket_class::reserve_inventory::ReserveInventory;
use instructions::ticket_class::update_ticket_class::UpdateTicketClass;
use instructions::ticket_state::set_ticket_metadata::SetTicketMetadata;
use instructions::ticket_state::transition_ticket_status::TransitionTicketStatus;
use instructions::treasury::initialize_vault::InitializeVault;
use instructions::treasury::snapshot_vault::SnapshotVault;
use instructions::treasury::withdraw_vault::WithdrawVault;
use instructions::trust_signal::flag_abuse::FlagAbuse;
use instructions::trust_signal::record_attendance_input::RecordAttendanceInput;
use instructions::trust_signal::record_purchase_input::RecordPurchaseInput;
use instructions::trust_signal::set_schema_version::SetSchemaVersion;
use state::FinancingOfferInput;
use state::ResalePolicyInput;

declare_id!("DyHzPALx4rqgj8X6tycKxFA8KyGscBJ38xdVpCeSL8ej");

#[program]
pub mod ticketing_core {
    use super::*;

    pub fn initialize_protocol(
        ctx: Context<InitializeProtocol>,
        admin: Pubkey,
        upgrade_authority: Pubkey,
        treasury_vault: Pubkey,
        fee_vault: Pubkey,
        protocol_fee_bps: u16,
        max_tickets_per_wallet: u16,
    ) -> Result<()> {
        instructions::protocol::initialize_protocol::initialize_protocol(
            ctx,
            admin,
            upgrade_authority,
            treasury_vault,
            fee_vault,
            protocol_fee_bps,
            max_tickets_per_wallet,
        )
    }

    pub fn set_protocol_config(
        ctx: Context<SetProtocolConfig>,
        protocol_fee_bps: u16,
        max_tickets_per_wallet: u16,
    ) -> Result<()> {
        instructions::protocol::set_protocol_config::set_protocol_config(
            ctx,
            protocol_fee_bps,
            max_tickets_per_wallet,
        )
    }

    pub fn register_protocol_vaults(
        ctx: Context<RegisterProtocolVaults>,
        treasury_vault: Pubkey,
        fee_vault: Pubkey,
    ) -> Result<()> {
        instructions::protocol::set_protocol_config::register_protocol_vaults(
            ctx,
            treasury_vault,
            fee_vault,
        )
    }

    pub fn set_protocol_authorities(
        ctx: Context<SetProtocolAuthorities>,
        new_admin: Pubkey,
        new_upgrade_authority: Pubkey,
    ) -> Result<()> {
        instructions::protocol::set_protocol_config::set_protocol_authorities(
            ctx,
            new_admin,
            new_upgrade_authority,
        )
    }

    pub fn pause_protocol(ctx: Context<PauseProtocol>, is_paused: bool) -> Result<()> {
        instructions::protocol::pause_protocol::pause_protocol(ctx, is_paused)
    }

    pub fn set_multisig_config(
        ctx: Context<SetProtocolGovernance>,
        enabled: bool,
        threshold: u8,
        signer_1: Pubkey,
        signer_2: Pubkey,
        signer_3: Pubkey,
    ) -> Result<()> {
        instructions::protocol::governance_hooks::set_multisig_config(
            ctx, enabled, threshold, signer_1, signer_2, signer_3,
        )
    }

    pub fn set_timelock_delay(
        ctx: Context<SetProtocolGovernance>,
        timelock_delay_secs: i64,
    ) -> Result<()> {
        instructions::protocol::governance_hooks::set_timelock_delay(ctx, timelock_delay_secs)
    }

    pub fn queue_protocol_config_change(
        ctx: Context<SetProtocolGovernance>,
        pending_protocol_fee_bps: u16,
        pending_max_tickets_per_wallet: u16,
    ) -> Result<()> {
        instructions::protocol::governance_hooks::queue_protocol_config_change(
            ctx,
            pending_protocol_fee_bps,
            pending_max_tickets_per_wallet,
        )
    }

    pub fn execute_protocol_config_change(ctx: Context<SetProtocolGovernance>) -> Result<()> {
        instructions::protocol::governance_hooks::execute_protocol_config_change(ctx)
    }

    pub fn begin_upgrade_authority_handoff(
        ctx: Context<SetProtocolGovernance>,
        pending_upgrade_authority: Pubkey,
    ) -> Result<()> {
        instructions::protocol::governance_hooks::begin_upgrade_authority_handoff(
            ctx,
            pending_upgrade_authority,
        )
    }

    pub fn accept_upgrade_authority_handoff(
        ctx: Context<AcceptUpgradeAuthorityHandoff>,
    ) -> Result<()> {
        instructions::protocol::governance_hooks::accept_upgrade_authority_handoff(ctx)
    }

    pub fn emergency_rotate_admin(
        ctx: Context<EmergencyAdminAction>,
        new_admin: Pubkey,
        new_emergency_admin: Pubkey,
        reason_code: u16,
    ) -> Result<()> {
        instructions::protocol::governance_hooks::emergency_rotate_admin(
            ctx,
            new_admin,
            new_emergency_admin,
            reason_code,
        )
    }

    pub fn create_organizer(
        ctx: Context<CreateOrganizer>,
        metadata_uri: String,
        payout_wallet: Pubkey,
    ) -> Result<()> {
        instructions::organizer::create_organizer::create_organizer(
            ctx,
            metadata_uri,
            payout_wallet,
        )
    }

    pub fn update_organizer(
        ctx: Context<UpdateOrganizer>,
        metadata_uri: String,
        payout_wallet: Pubkey,
    ) -> Result<()> {
        instructions::organizer::update_organizer::update_organizer(
            ctx,
            metadata_uri,
            payout_wallet,
        )
    }

    pub fn set_organizer_status(ctx: Context<SetOrganizerStatus>, status: u8) -> Result<()> {
        instructions::organizer::update_organizer::set_organizer_status(ctx, status)
    }

    pub fn set_organizer_compliance_flags(
        ctx: Context<SetOrganizerComplianceFlags>,
        compliance_flags: u32,
    ) -> Result<()> {
        instructions::organizer::update_organizer::set_organizer_compliance_flags(
            ctx,
            compliance_flags,
        )
    }

    pub fn set_organizer_operator(
        ctx: Context<SetOrganizerOperator>,
        permissions: u32,
        active: bool,
    ) -> Result<()> {
        instructions::organizer::set_operator::set_organizer_operator(ctx, permissions, active)
    }

    pub fn set_check_in_policy(
        ctx: Context<SetCheckInPolicy>,
        class_id: u16,
        allow_reentry: bool,
        max_reentries: u8,
    ) -> Result<()> {
        let _ = class_id;
        instructions::checkin::check_in_ticket::set_checkin_policy(
            ctx,
            allow_reentry,
            max_reentries,
        )
    }

    pub fn check_in_ticket(
        ctx: Context<CheckInTicket>,
        class_id: u16,
        ticket_id: u32,
        gate_identifier: String,
    ) -> Result<()> {
        instructions::checkin::check_in_ticket::check_in_ticket(
            ctx,
            class_id,
            ticket_id,
            gate_identifier,
        )
    }

    pub fn set_global_loyalty_multiplier(
        ctx: Context<SetGlobalLoyaltyMultiplier>,
        multiplier_bps: u16,
    ) -> Result<()> {
        instructions::loyalty::accrue_points::set_global_loyalty_multiplier(ctx, multiplier_bps)
    }

    pub fn set_event_loyalty_multiplier(
        ctx: Context<SetEventLoyaltyMultiplier>,
        multiplier_bps: u16,
    ) -> Result<()> {
        instructions::loyalty::accrue_points::set_event_loyalty_multiplier(ctx, multiplier_bps)
    }

    pub fn accrue_points(
        ctx: Context<AccruePoints>,
        class_id: u16,
        ticket_id: u32,
        reason: u8,
        base_points: u64,
        hold_duration_days: u16,
    ) -> Result<()> {
        instructions::loyalty::accrue_points::accrue_points(
            ctx,
            class_id,
            ticket_id,
            reason,
            base_points,
            hold_duration_days,
        )
    }

    pub fn redeem_points(
        ctx: Context<RedeemPoints>,
        points_to_burn: u64,
        perk_code: String,
        event: Pubkey,
    ) -> Result<()> {
        instructions::loyalty::redeem_points::redeem_points(ctx, points_to_burn, perk_code, event)
    }

    pub fn record_purchase_input(
        ctx: Context<RecordPurchaseInput>,
        class_id: u16,
        ticket_id: u32,
    ) -> Result<()> {
        instructions::trust_signal::record_purchase_input::record_purchase_input(
            ctx, class_id, ticket_id,
        )
    }

    pub fn record_attendance_input(
        ctx: Context<RecordAttendanceInput>,
        class_id: u16,
        ticket_id: u32,
        did_attend: bool,
    ) -> Result<()> {
        instructions::trust_signal::record_attendance_input::record_attendance_input(
            ctx, class_id, ticket_id, did_attend,
        )
    }

    pub fn flag_trust_abuse(
        ctx: Context<FlagAbuse>,
        flag_bits: u32,
        event: Pubkey,
        ticket: Pubkey,
    ) -> Result<()> {
        instructions::trust_signal::flag_abuse::flag_abuse(ctx, flag_bits, event, ticket)
    }

    pub fn set_trust_signal_schema_version(
        ctx: Context<SetSchemaVersion>,
        new_schema_version: u16,
    ) -> Result<()> {
        instructions::trust_signal::set_schema_version::set_schema_version(ctx, new_schema_version)
    }

    pub fn upsert_registry_entry(
        ctx: Context<UpsertRegistryEntry>,
        scope: u8,
        target: Pubkey,
        subject: Pubkey,
        list_type: u8,
        is_allowed: bool,
        decision_code: u16,
    ) -> Result<()> {
        instructions::compliance::upsert_registry_entry::upsert_registry_entry(
            ctx,
            scope,
            target,
            subject,
            list_type,
            is_allowed,
            decision_code,
        )
    }

    pub fn set_event_restrictions(
        ctx: Context<SetEventRestrictions>,
        restriction_flags: u32,
        decision_code: u16,
    ) -> Result<()> {
        instructions::compliance::set_event_restrictions::set_event_restrictions(
            ctx,
            restriction_flags,
            decision_code,
        )
    }

    pub fn grant_role(
        ctx: Context<GrantRole>,
        role: u8,
        scope: u8,
        starts_at: i64,
        expires_at: i64,
    ) -> Result<()> {
        instructions::governance::grant_role::grant_role(ctx, role, scope, starts_at, expires_at)
    }

    pub fn revoke_role(
        ctx: Context<RevokeRole>,
        role: u8,
        scope: u8,
        reason_code: u16,
    ) -> Result<()> {
        instructions::governance::revoke_role::revoke_role(ctx, role, scope, reason_code)
    }

    pub fn rotate_authority(
        ctx: Context<RotateAuthority>,
        role: u8,
        scope: u8,
        starts_at: i64,
        expires_at: i64,
    ) -> Result<()> {
        instructions::governance::rotate_authority::rotate_authority(
            ctx, role, scope, starts_at, expires_at,
        )
    }

    pub fn initialize_vault(ctx: Context<InitializeVault>, kind: u8, parent: Pubkey) -> Result<()> {
        instructions::treasury::initialize_vault::initialize_vault(ctx, kind, parent)
    }

    pub fn snapshot_vault(ctx: Context<SnapshotVault>, kind: u8, parent: Pubkey) -> Result<()> {
        instructions::treasury::snapshot_vault::snapshot_vault(ctx, kind, parent)
    }

    pub fn withdraw_vault(
        ctx: Context<WithdrawVault>,
        kind: u8,
        parent: Pubkey,
        amount_lamports: u64,
    ) -> Result<()> {
        instructions::treasury::withdraw_vault::withdraw_vault(ctx, kind, parent, amount_lamports)
    }

    pub fn refund_ticket(
        ctx: Context<RefundTicket>,
        class_id: u16,
        ticket_id: u32,
        amount_lamports: u64,
        source: u8,
    ) -> Result<()> {
        instructions::disputes::refund_ticket::refund_ticket(
            ctx,
            class_id,
            ticket_id,
            amount_lamports,
            source,
        )
    }

    pub fn flag_dispute(
        ctx: Context<FlagDispute>,
        class_id: u16,
        ticket_id: u32,
        is_disputed: bool,
        is_chargeback: bool,
        reason_code: u16,
    ) -> Result<()> {
        instructions::disputes::flag_dispute::flag_dispute(
            ctx,
            class_id,
            ticket_id,
            is_disputed,
            is_chargeback,
            reason_code,
        )
    }

    pub fn create_event(ctx: Context<CreateEvent>, event_id: u64, input: EventInput) -> Result<()> {
        instructions::event::create_event::create_event(ctx, event_id, input)
    }

    pub fn update_event(ctx: Context<UpdateEvent>, input: EventInput) -> Result<()> {
        instructions::event::update_event::update_event(ctx, input)
    }

    pub fn freeze_event(ctx: Context<FreezeEvent>) -> Result<()> {
        instructions::event::freeze_event::freeze_event(ctx)
    }

    pub fn pause_event(ctx: Context<PauseEvent>, is_paused: bool) -> Result<()> {
        instructions::event::pause_event::pause_event(ctx, is_paused)
    }

    pub fn cancel_event(ctx: Context<CancelEvent>) -> Result<()> {
        instructions::event::cancel_event::cancel_event(ctx)
    }

    pub fn close_event(ctx: Context<CloseEvent>) -> Result<()> {
        instructions::event::cancel_event::close_event(ctx)
    }

    pub fn create_ticket_class(
        ctx: Context<CreateTicketClass>,
        class_id: u16,
        input: TicketClassInput,
    ) -> Result<()> {
        instructions::ticket_class::create_ticket_class::create_ticket_class(ctx, class_id, input)
    }

    pub fn update_ticket_class(
        ctx: Context<UpdateTicketClass>,
        class_id: u16,
        input: TicketClassInput,
    ) -> Result<()> {
        instructions::ticket_class::update_ticket_class::update_ticket_class(ctx, class_id, input)
    }

    pub fn reserve_inventory(
        ctx: Context<ReserveInventory>,
        class_id: u16,
        amount: u32,
    ) -> Result<()> {
        instructions::ticket_class::reserve_inventory::reserve_inventory(ctx, class_id, amount)
    }

    pub fn buy_ticket(
        ctx: Context<BuyTicket>,
        class_id: u16,
        ticket_id: u32,
        expected_price_lamports: u64,
    ) -> Result<()> {
        instructions::primary_sale::buy_ticket::buy_ticket(
            ctx,
            class_id,
            ticket_id,
            expected_price_lamports,
        )
    }

    pub fn issue_comp_ticket(
        ctx: Context<IssueCompTicket>,
        class_id: u16,
        ticket_id: u32,
    ) -> Result<()> {
        instructions::primary_sale::issue_comp_ticket::issue_comp_ticket(ctx, class_id, ticket_id)
    }

    pub fn set_resale_policy(
        ctx: Context<SetResalePolicy>,
        class_id: u16,
        input: ResalePolicyInput,
    ) -> Result<()> {
        instructions::resale::set_resale_policy::set_resale_policy(ctx, class_id, input)
    }

    pub fn list_ticket(
        ctx: Context<ListTicket>,
        class_id: u16,
        ticket_id: u32,
        price_lamports: u64,
        expires_at: i64,
    ) -> Result<()> {
        instructions::resale::list_ticket::list_ticket(
            ctx,
            class_id,
            ticket_id,
            price_lamports,
            expires_at,
        )
    }

    pub fn buy_resale_ticket(
        ctx: Context<BuyResaleTicket>,
        class_id: u16,
        ticket_id: u32,
        max_price_lamports: u64,
    ) -> Result<()> {
        instructions::resale::buy_resale_ticket::buy_resale_ticket(
            ctx,
            class_id,
            ticket_id,
            max_price_lamports,
        )
    }

    pub fn cancel_listing(
        ctx: Context<CancelListing>,
        class_id: u16,
        ticket_id: u32,
    ) -> Result<()> {
        instructions::resale::cancel_listing::cancel_listing(ctx, class_id, ticket_id)
    }

    pub fn expire_listing(
        ctx: Context<ExpireListing>,
        class_id: u16,
        ticket_id: u32,
    ) -> Result<()> {
        instructions::resale::expire_listing::expire_listing(ctx, class_id, ticket_id)
    }

    pub fn transition_ticket_status(
        ctx: Context<TransitionTicketStatus>,
        class_id: u16,
        ticket_id: u32,
        next_status: u8,
    ) -> Result<()> {
        instructions::ticket_state::transition_ticket_status::transition_ticket_status(
            ctx,
            class_id,
            ticket_id,
            next_status,
        )
    }

    pub fn set_ticket_metadata(
        ctx: Context<SetTicketMetadata>,
        class_id: u16,
        ticket_id: u32,
        metadata_uri: String,
        metadata_version: u16,
    ) -> Result<()> {
        instructions::ticket_state::set_ticket_metadata::set_ticket_metadata(
            ctx,
            class_id,
            ticket_id,
            metadata_uri,
            metadata_version,
        )
    }

    pub fn create_financing_offer(
        ctx: Context<CreateFinancingOffer>,
        input: FinancingOfferInput,
    ) -> Result<()> {
        instructions::financing::create_financing_offer::create_financing_offer(ctx, input)
    }

    pub fn accept_financing_offer(ctx: Context<AcceptFinancingOffer>, accept: bool) -> Result<()> {
        instructions::financing::accept_financing_offer::accept_financing_offer(ctx, accept)
    }

    pub fn disburse_advance(
        ctx: Context<DisburseAdvance>,
        amount_lamports: u64,
        reference_id: [u8; 16],
    ) -> Result<()> {
        instructions::financing::disburse_advance::disburse_advance(
            ctx,
            amount_lamports,
            reference_id,
        )
    }

    pub fn set_financing_freeze(
        ctx: Context<SetFinancingFreeze>,
        is_frozen: bool,
        reason_code: u16,
        clawback_allowed: bool,
    ) -> Result<()> {
        instructions::financing::set_financing_freeze::set_financing_freeze(
            ctx,
            is_frozen,
            reason_code,
            clawback_allowed,
        )
    }

    pub fn clawback_disbursement(
        ctx: Context<ClawbackDisbursement>,
        disbursement_index: u16,
    ) -> Result<()> {
        instructions::financing::clawback_disbursement::clawback_disbursement(
            ctx,
            disbursement_index,
        )
    }

    pub fn settle_primary_revenue(
        ctx: Context<SettlePrimaryRevenue>,
        gross_revenue_lamports: u64,
        protocol_bps: u16,
        royalty_bps: u16,
        other_bps: u16,
        settlement_reference: [u8; 16],
    ) -> Result<()> {
        instructions::settlement::settle_primary_revenue::settle_primary_revenue(
            ctx,
            gross_revenue_lamports,
            protocol_bps,
            royalty_bps,
            other_bps,
            settlement_reference,
        )
    }

    pub fn settle_resale_revenue(
        ctx: Context<SettleResaleRevenue>,
        gross_revenue_lamports: u64,
        protocol_bps: u16,
        royalty_bps: u16,
        other_bps: u16,
        settlement_reference: [u8; 16],
    ) -> Result<()> {
        instructions::settlement::settle_resale_revenue::settle_resale_revenue(
            ctx,
            gross_revenue_lamports,
            protocol_bps,
            royalty_bps,
            other_bps,
            settlement_reference,
        )
    }

    pub fn finalize_settlement(ctx: Context<FinalizeSettlement>) -> Result<()> {
        instructions::settlement::finalize_settlement::finalize_settlement(ctx)
    }
}
