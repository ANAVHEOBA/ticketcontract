use anchor_lang::{prelude::Pubkey, AccountDeserialize, InstructionData, ToAccountMetas};
use solana_program_test::{BanksClient, BanksClientError, ProgramTest, ProgramTestContext};
use solana_sdk::{
    hash::Hash,
    instruction::Instruction,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

use ticketing_core::{
    accounts, instruction,
    instructions::event::EventInput,
    instructions::ticket_class::TicketClassInput,
    state::{
        ComplianceRegistry, DisbursementRecord, EventAccount, FinancingOffer, FinancingOfferInput,
        Listing, LoyaltyLedger, OrganizerOperator, OrganizerProfile, ProtocolConfig, ResalePolicy,
        ResalePolicyInput, RoleBinding, SettlementLedger, Ticket, TicketClass, TrustSignal,
        VaultAccount, WalletPurchaseCounter,
    },
    ID,
};

pub fn protocol_config_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"protocol-config"], &ID)
}

pub fn organizer_pda(authority: Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"organizer", authority.as_ref()], &ID)
}

pub fn organizer_operator_pda(organizer: Pubkey, operator: Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"organizer-operator", organizer.as_ref(), operator.as_ref()],
        &ID,
    )
}

pub fn role_binding_pda(target: Pubkey, role: u8, subject: Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"role-binding", target.as_ref(), &[role], subject.as_ref()],
        &ID,
    )
}

pub fn event_pda(organizer: Pubkey, event_id: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"event", organizer.as_ref(), &event_id.to_le_bytes()],
        &ID,
    )
}

pub fn ticket_class_pda(event: Pubkey, class_id: u16) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"ticket-class", event.as_ref(), &class_id.to_le_bytes()],
        &ID,
    )
}

pub fn ticket_pda(event: Pubkey, class_id: u16, ticket_id: u32) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"ticket",
            event.as_ref(),
            &class_id.to_le_bytes(),
            &ticket_id.to_le_bytes(),
        ],
        &ID,
    )
}

pub fn wallet_purchase_counter_pda(
    event: Pubkey,
    ticket_class: Pubkey,
    wallet: Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"wallet-purchase-counter",
            event.as_ref(),
            ticket_class.as_ref(),
            wallet.as_ref(),
        ],
        &ID,
    )
}

pub fn resale_policy_pda(event: Pubkey, class_id: u16) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"resale-policy", event.as_ref(), &class_id.to_le_bytes()],
        &ID,
    )
}

pub fn listing_pda(ticket: Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"listing", ticket.as_ref()], &ID)
}

pub fn financing_offer_pda(event: Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"financing-offer", event.as_ref()], &ID)
}

pub fn disbursement_record_pda(financing_offer: Pubkey, disbursement_index: u16) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"disbursement-record",
            financing_offer.as_ref(),
            &disbursement_index.to_le_bytes(),
        ],
        &ID,
    )
}

pub fn settlement_ledger_pda(event: Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"settlement-ledger", event.as_ref()], &ID)
}

pub fn loyalty_ledger_pda(wallet: Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"loyalty-ledger", wallet.as_ref()], &ID)
}

pub fn trust_signal_pda(wallet: Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"trust-signal", wallet.as_ref()], &ID)
}

pub fn compliance_registry_pda(target: Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"compliance-registry", target.as_ref()], &ID)
}

pub fn vault_state_pda(kind: u8, parent: Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"vault", b"state", &[kind], parent.as_ref()], &ID)
}

pub fn vault_funds_pda(kind: u8, parent: Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"vault", b"funds", &[kind], parent.as_ref()], &ID)
}

pub async fn setup() -> ProgramTestContext {
    let deploy_dir =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/deploy");
    std::env::set_var("SBF_OUT_DIR", deploy_dir);

    let mut program_test = ProgramTest::default();
    program_test.prefer_bpf(true);
    program_test.add_program("ticketing_core", ID, None);
    program_test.start_with_context().await
}

pub async fn send_ix(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: Hash,
    ix: Instruction,
    extra_signers: &[&Keypair],
) {
    send_ix_result(banks_client, payer, recent_blockhash, ix, extra_signers)
        .await
        .unwrap();
}

pub async fn send_ix_result(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: Hash,
    ix: Instruction,
    extra_signers: &[&Keypair],
) -> Result<(), BanksClientError> {
    let mut all_signers = vec![payer];
    all_signers.extend_from_slice(extra_signers);
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &all_signers,
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await
}

pub async fn fetch_protocol_config(
    banks_client: &mut BanksClient,
    protocol_config: Pubkey,
) -> ProtocolConfig {
    let account = banks_client
        .get_account(protocol_config)
        .await
        .unwrap()
        .expect("protocol config account missing");

    let mut data = account.data.as_slice();
    ProtocolConfig::try_deserialize(&mut data).unwrap()
}

pub async fn fetch_organizer_profile(
    banks_client: &mut BanksClient,
    organizer_profile: Pubkey,
) -> OrganizerProfile {
    let account = banks_client
        .get_account(organizer_profile)
        .await
        .unwrap()
        .expect("organizer profile account missing");

    let mut data = account.data.as_slice();
    OrganizerProfile::try_deserialize(&mut data).unwrap()
}

pub async fn fetch_organizer_operator(
    banks_client: &mut BanksClient,
    organizer_operator: Pubkey,
) -> OrganizerOperator {
    let account = banks_client
        .get_account(organizer_operator)
        .await
        .unwrap()
        .expect("organizer operator account missing");

    let mut data = account.data.as_slice();
    OrganizerOperator::try_deserialize(&mut data).unwrap()
}

pub async fn fetch_role_binding(
    banks_client: &mut BanksClient,
    role_binding: Pubkey,
) -> RoleBinding {
    let account = banks_client
        .get_account(role_binding)
        .await
        .unwrap()
        .expect("role binding account missing");

    let mut data = account.data.as_slice();
    RoleBinding::try_deserialize(&mut data).unwrap()
}

pub async fn fetch_vault_account(
    banks_client: &mut BanksClient,
    vault_state: Pubkey,
) -> VaultAccount {
    let account = banks_client
        .get_account(vault_state)
        .await
        .unwrap()
        .expect("vault state account missing");

    let mut data = account.data.as_slice();
    VaultAccount::try_deserialize(&mut data).unwrap()
}

#[allow(dead_code)]
pub async fn fetch_compliance_registry(
    banks_client: &mut BanksClient,
    compliance_registry: Pubkey,
) -> ComplianceRegistry {
    let account = banks_client
        .get_account(compliance_registry)
        .await
        .unwrap()
        .expect("compliance registry account missing");

    let mut data = account.data.as_slice();
    ComplianceRegistry::try_deserialize(&mut data).unwrap()
}

pub async fn fetch_event_account(
    banks_client: &mut BanksClient,
    event_account: Pubkey,
) -> EventAccount {
    let account = banks_client
        .get_account(event_account)
        .await
        .unwrap()
        .expect("event account missing");

    let mut data = account.data.as_slice();
    EventAccount::try_deserialize(&mut data).unwrap()
}

pub async fn fetch_ticket_class(
    banks_client: &mut BanksClient,
    ticket_class: Pubkey,
) -> TicketClass {
    let account = banks_client
        .get_account(ticket_class)
        .await
        .unwrap()
        .expect("ticket class account missing");

    let mut data = account.data.as_slice();
    TicketClass::try_deserialize(&mut data).unwrap()
}

pub async fn fetch_ticket(banks_client: &mut BanksClient, ticket: Pubkey) -> Ticket {
    let account = banks_client
        .get_account(ticket)
        .await
        .unwrap()
        .expect("ticket account missing");

    let mut data = account.data.as_slice();
    Ticket::try_deserialize(&mut data).unwrap()
}

pub async fn fetch_wallet_purchase_counter(
    banks_client: &mut BanksClient,
    counter: Pubkey,
) -> WalletPurchaseCounter {
    let account = banks_client
        .get_account(counter)
        .await
        .unwrap()
        .expect("wallet purchase counter account missing");

    let mut data = account.data.as_slice();
    WalletPurchaseCounter::try_deserialize(&mut data).unwrap()
}

pub async fn fetch_resale_policy(
    banks_client: &mut BanksClient,
    resale_policy: Pubkey,
) -> ResalePolicy {
    let account = banks_client
        .get_account(resale_policy)
        .await
        .unwrap()
        .expect("resale policy account missing");

    let mut data = account.data.as_slice();
    ResalePolicy::try_deserialize(&mut data).unwrap()
}

pub async fn fetch_listing(banks_client: &mut BanksClient, listing: Pubkey) -> Listing {
    let account = banks_client
        .get_account(listing)
        .await
        .unwrap()
        .expect("listing account missing");

    let mut data = account.data.as_slice();
    Listing::try_deserialize(&mut data).unwrap()
}

pub async fn fetch_financing_offer(
    banks_client: &mut BanksClient,
    financing_offer: Pubkey,
) -> FinancingOffer {
    let account = banks_client
        .get_account(financing_offer)
        .await
        .unwrap()
        .expect("financing offer account missing");

    let mut data = account.data.as_slice();
    FinancingOffer::try_deserialize(&mut data).unwrap()
}

pub async fn fetch_disbursement_record(
    banks_client: &mut BanksClient,
    disbursement_record: Pubkey,
) -> DisbursementRecord {
    let account = banks_client
        .get_account(disbursement_record)
        .await
        .unwrap()
        .expect("disbursement record account missing");

    let mut data = account.data.as_slice();
    DisbursementRecord::try_deserialize(&mut data).unwrap()
}

pub async fn fetch_settlement_ledger(
    banks_client: &mut BanksClient,
    settlement_ledger: Pubkey,
) -> SettlementLedger {
    let account = banks_client
        .get_account(settlement_ledger)
        .await
        .unwrap()
        .expect("settlement ledger account missing");

    let mut data = account.data.as_slice();
    SettlementLedger::try_deserialize(&mut data).unwrap()
}

pub async fn fetch_loyalty_ledger(
    banks_client: &mut BanksClient,
    loyalty_ledger: Pubkey,
) -> LoyaltyLedger {
    let account = banks_client
        .get_account(loyalty_ledger)
        .await
        .unwrap()
        .expect("loyalty ledger account missing");

    let mut data = account.data.as_slice();
    LoyaltyLedger::try_deserialize(&mut data).unwrap()
}

pub async fn fetch_trust_signal(
    banks_client: &mut BanksClient,
    trust_signal: Pubkey,
) -> TrustSignal {
    let account = banks_client
        .get_account(trust_signal)
        .await
        .unwrap()
        .expect("trust signal account missing");

    let mut data = account.data.as_slice();
    TrustSignal::try_deserialize(&mut data).unwrap()
}

pub async fn get_lamports(banks_client: &mut BanksClient, key: Pubkey) -> u64 {
    banks_client
        .get_account(key)
        .await
        .unwrap()
        .map(|a| a.lamports)
        .unwrap_or(0)
}

pub fn ix_initialize_protocol(
    payer: Pubkey,
    admin: Pubkey,
    protocol_config: Pubkey,
    upgrade_authority: Pubkey,
    treasury_vault: Pubkey,
    fee_vault: Pubkey,
    protocol_fee_bps: u16,
    max_tickets_per_wallet: u16,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::InitializeProtocol {
            payer,
            protocol_config,
            system_program: solana_sdk::system_program::id(),
        }
        .to_account_metas(None),
        data: instruction::InitializeProtocol {
            admin,
            upgrade_authority,
            treasury_vault,
            fee_vault,
            protocol_fee_bps,
            max_tickets_per_wallet,
        }
        .data(),
    }
}

pub fn ix_set_protocol_config(
    admin: Pubkey,
    protocol_config: Pubkey,
    protocol_fee_bps: u16,
    max_tickets_per_wallet: u16,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::SetProtocolConfig {
            admin,
            protocol_config,
        }
        .to_account_metas(None),
        data: instruction::SetProtocolConfig {
            protocol_fee_bps,
            max_tickets_per_wallet,
        }
        .data(),
    }
}

pub fn ix_register_protocol_vaults(
    admin: Pubkey,
    protocol_config: Pubkey,
    treasury_vault: Pubkey,
    fee_vault: Pubkey,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::RegisterProtocolVaults {
            admin,
            protocol_config,
        }
        .to_account_metas(None),
        data: instruction::RegisterProtocolVaults {
            treasury_vault,
            fee_vault,
        }
        .data(),
    }
}

pub fn ix_set_protocol_authorities(
    admin: Pubkey,
    protocol_config: Pubkey,
    new_admin: Pubkey,
    new_upgrade_authority: Pubkey,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::SetProtocolAuthorities {
            admin,
            protocol_config,
        }
        .to_account_metas(None),
        data: instruction::SetProtocolAuthorities {
            new_admin,
            new_upgrade_authority,
        }
        .data(),
    }
}

pub fn ix_pause_protocol(admin: Pubkey, protocol_config: Pubkey, is_paused: bool) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::PauseProtocol {
            admin,
            protocol_config,
        }
        .to_account_metas(None),
        data: instruction::PauseProtocol { is_paused }.data(),
    }
}

pub fn ix_set_multisig_config(
    admin: Pubkey,
    protocol_config: Pubkey,
    enabled: bool,
    threshold: u8,
    signer_1: Pubkey,
    signer_2: Pubkey,
    signer_3: Pubkey,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::SetProtocolGovernance {
            admin,
            protocol_config,
        }
        .to_account_metas(None),
        data: instruction::SetMultisigConfig {
            enabled,
            threshold,
            signer_1,
            signer_2,
            signer_3,
        }
        .data(),
    }
}

pub fn ix_set_timelock_delay(
    admin: Pubkey,
    protocol_config: Pubkey,
    timelock_delay_secs: i64,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::SetProtocolGovernance {
            admin,
            protocol_config,
        }
        .to_account_metas(None),
        data: instruction::SetTimelockDelay { timelock_delay_secs }.data(),
    }
}

pub fn ix_queue_protocol_config_change(
    admin: Pubkey,
    protocol_config: Pubkey,
    pending_protocol_fee_bps: u16,
    pending_max_tickets_per_wallet: u16,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::SetProtocolGovernance {
            admin,
            protocol_config,
        }
        .to_account_metas(None),
        data: instruction::QueueProtocolConfigChange {
            pending_protocol_fee_bps,
            pending_max_tickets_per_wallet,
        }
        .data(),
    }
}

pub fn ix_execute_protocol_config_change(admin: Pubkey, protocol_config: Pubkey) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::SetProtocolGovernance {
            admin,
            protocol_config,
        }
        .to_account_metas(None),
        data: instruction::ExecuteProtocolConfigChange {}.data(),
    }
}

pub fn ix_begin_upgrade_authority_handoff(
    admin: Pubkey,
    protocol_config: Pubkey,
    pending_upgrade_authority: Pubkey,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::SetProtocolGovernance {
            admin,
            protocol_config,
        }
        .to_account_metas(None),
        data: instruction::BeginUpgradeAuthorityHandoff {
            pending_upgrade_authority,
        }
        .data(),
    }
}

pub fn ix_accept_upgrade_authority_handoff(
    pending_upgrade_authority: Pubkey,
    protocol_config: Pubkey,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::AcceptUpgradeAuthorityHandoff {
            pending_upgrade_authority,
            protocol_config,
        }
        .to_account_metas(None),
        data: instruction::AcceptUpgradeAuthorityHandoff {}.data(),
    }
}

pub fn ix_emergency_rotate_admin(
    emergency_admin: Pubkey,
    protocol_config: Pubkey,
    new_admin: Pubkey,
    new_emergency_admin: Pubkey,
    reason_code: u16,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::EmergencyAdminAction {
            emergency_admin,
            protocol_config,
        }
        .to_account_metas(None),
        data: instruction::EmergencyRotateAdmin {
            new_admin,
            new_emergency_admin,
            reason_code,
        }
        .data(),
    }
}

pub fn ix_create_organizer(
    payer: Pubkey,
    authority: Pubkey,
    protocol_config: Pubkey,
    organizer_profile: Pubkey,
    metadata_uri: String,
    payout_wallet: Pubkey,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::CreateOrganizer {
            payer,
            authority,
            protocol_config,
            organizer_profile,
            system_program: solana_sdk::system_program::id(),
        }
        .to_account_metas(None),
        data: instruction::CreateOrganizer {
            metadata_uri,
            payout_wallet,
        }
        .data(),
    }
}

pub fn ix_update_organizer(
    authority: Pubkey,
    organizer_profile: Pubkey,
    metadata_uri: String,
    payout_wallet: Pubkey,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::UpdateOrganizer {
            authority,
            organizer_profile,
        }
        .to_account_metas(None),
        data: instruction::UpdateOrganizer {
            metadata_uri,
            payout_wallet,
        }
        .data(),
    }
}

pub fn ix_set_organizer_status(
    admin: Pubkey,
    protocol_config: Pubkey,
    organizer_profile: Pubkey,
    status: u8,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::SetOrganizerStatus {
            admin,
            protocol_config,
            organizer_profile,
        }
        .to_account_metas(None),
        data: instruction::SetOrganizerStatus { status }.data(),
    }
}

pub fn ix_set_organizer_compliance_flags(
    admin: Pubkey,
    protocol_config: Pubkey,
    organizer_profile: Pubkey,
    compliance_flags: u32,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::SetOrganizerComplianceFlags {
            admin,
            protocol_config,
            organizer_profile,
        }
        .to_account_metas(None),
        data: instruction::SetOrganizerComplianceFlags { compliance_flags }.data(),
    }
}

pub fn ix_set_organizer_operator(
    payer: Pubkey,
    authority: Pubkey,
    operator: Pubkey,
    organizer_profile: Pubkey,
    organizer_operator: Pubkey,
    permissions: u32,
    active: bool,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::SetOrganizerOperator {
            payer,
            authority,
            operator,
            organizer_profile,
            organizer_operator,
            system_program: solana_sdk::system_program::id(),
        }
        .to_account_metas(None),
        data: instruction::SetOrganizerOperator {
            permissions,
            active,
        }
        .data(),
    }
}

pub fn ix_set_check_in_policy(
    authority: Pubkey,
    protocol_config: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
    ticket_class: Pubkey,
    class_id: u16,
    allow_reentry: bool,
    max_reentries: u8,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::SetCheckInPolicy {
            authority,
            protocol_config,
            organizer_profile,
            event_account,
            ticket_class,
        }
        .to_account_metas(None),
        data: instruction::SetCheckInPolicy {
            class_id,
            allow_reentry,
            max_reentries,
        }
        .data(),
    }
}

pub fn ix_check_in_ticket(
    scanner: Pubkey,
    protocol_config: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
    ticket_class: Pubkey,
    ticket: Pubkey,
    organizer_operator: Pubkey,
    class_id: u16,
    ticket_id: u32,
    gate_identifier: String,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::CheckInTicket {
            scanner,
            protocol_config,
            organizer_profile,
            event_account,
            ticket_class,
            ticket,
            organizer_operator,
            role_binding: organizer_operator,
        }
        .to_account_metas(None),
        data: instruction::CheckInTicket {
            class_id,
            ticket_id,
            gate_identifier,
        }
        .data(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn ix_grant_role(
    granter: Pubkey,
    payer: Pubkey,
    subject: Pubkey,
    target: Pubkey,
    protocol_config: Pubkey,
    organizer_profile: Pubkey,
    role_binding: Pubkey,
    role: u8,
    scope: u8,
    starts_at: i64,
    expires_at: i64,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::GrantRole {
            granter,
            payer,
            subject,
            target,
            protocol_config,
            organizer_profile,
            role_binding,
            system_program: solana_sdk::system_program::id(),
        }
        .to_account_metas(None),
        data: instruction::GrantRole {
            role,
            scope,
            starts_at,
            expires_at,
        }
        .data(),
    }
}

pub fn ix_revoke_role(
    revoker: Pubkey,
    subject: Pubkey,
    target: Pubkey,
    protocol_config: Pubkey,
    organizer_profile: Pubkey,
    role_binding: Pubkey,
    role: u8,
    scope: u8,
    reason_code: u16,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::RevokeRole {
            revoker,
            subject,
            target,
            protocol_config,
            organizer_profile,
            role_binding,
        }
        .to_account_metas(None),
        data: instruction::RevokeRole {
            role,
            scope,
            reason_code,
        }
        .data(),
    }
}

pub fn ix_create_event(
    payer: Pubkey,
    authority: Pubkey,
    protocol_config: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
    event_id: u64,
    input: EventInput,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::CreateEvent {
            payer,
            authority,
            protocol_config,
            organizer_profile,
            event_account,
            system_program: solana_sdk::system_program::id(),
        }
        .to_account_metas(None),
        data: instruction::CreateEvent { event_id, input }.data(),
    }
}

pub fn ix_update_event(
    authority: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
    input: EventInput,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::UpdateEvent {
            authority,
            organizer_profile,
            event_account,
        }
        .to_account_metas(None),
        data: instruction::UpdateEvent { input }.data(),
    }
}

pub fn ix_freeze_event(
    authority: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::FreezeEvent {
            authority,
            organizer_profile,
            event_account,
        }
        .to_account_metas(None),
        data: instruction::FreezeEvent {}.data(),
    }
}

pub fn ix_pause_event(
    authority: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
    is_paused: bool,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::PauseEvent {
            authority,
            organizer_profile,
            event_account,
        }
        .to_account_metas(None),
        data: instruction::PauseEvent { is_paused }.data(),
    }
}

pub fn ix_cancel_event(
    authority: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::CancelEvent {
            authority,
            organizer_profile,
            event_account,
        }
        .to_account_metas(None),
        data: instruction::CancelEvent {}.data(),
    }
}

pub fn ix_close_event(
    authority: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::CloseEvent {
            authority,
            organizer_profile,
            event_account,
        }
        .to_account_metas(None),
        data: instruction::CloseEvent {}.data(),
    }
}

pub fn ix_create_ticket_class(
    payer: Pubkey,
    authority: Pubkey,
    protocol_config: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
    ticket_class: Pubkey,
    class_id: u16,
    input: TicketClassInput,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::CreateTicketClass {
            payer,
            authority,
            protocol_config,
            organizer_profile,
            event_account,
            ticket_class,
            system_program: solana_sdk::system_program::id(),
        }
        .to_account_metas(None),
        data: instruction::CreateTicketClass { class_id, input }.data(),
    }
}

pub fn ix_update_ticket_class(
    authority: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
    ticket_class: Pubkey,
    class_id: u16,
    input: TicketClassInput,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::UpdateTicketClass {
            authority,
            organizer_profile,
            event_account,
            ticket_class,
        }
        .to_account_metas(None),
        data: instruction::UpdateTicketClass { class_id, input }.data(),
    }
}

pub fn ix_reserve_inventory(
    authority: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
    ticket_class: Pubkey,
    class_id: u16,
    amount: u32,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::ReserveInventory {
            authority,
            organizer_profile,
            event_account,
            ticket_class,
        }
        .to_account_metas(None),
        data: instruction::ReserveInventory { class_id, amount }.data(),
    }
}

pub fn ix_buy_ticket(
    buyer: Pubkey,
    protocol_config: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
    ticket_class: Pubkey,
    ticket: Pubkey,
    wallet_purchase_counter: Pubkey,
    protocol_fee_vault: Pubkey,
    organizer_payout_wallet: Pubkey,
    stakeholder_wallet: Pubkey,
    class_id: u16,
    ticket_id: u32,
    expected_price_lamports: u64,
) -> Instruction {
    let (compliance_registry, _) = compliance_registry_pda(event_account);
    Instruction {
        program_id: ID,
        accounts: accounts::BuyTicket {
            buyer,
            protocol_config,
            organizer_profile,
            event_account,
            ticket_class,
            ticket,
            wallet_purchase_counter,
            protocol_fee_vault,
            organizer_payout_wallet,
            stakeholder_wallet,
            compliance_registry,
            system_program: solana_sdk::system_program::id(),
        }
        .to_account_metas(None),
        data: instruction::BuyTicket {
            class_id,
            ticket_id,
            expected_price_lamports,
        }
        .data(),
    }
}

pub fn ix_issue_comp_ticket(
    payer: Pubkey,
    issuer: Pubkey,
    recipient: Pubkey,
    protocol_config: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
    ticket_class: Pubkey,
    ticket: Pubkey,
    class_id: u16,
    ticket_id: u32,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::IssueCompTicket {
            payer,
            issuer,
            recipient,
            protocol_config,
            organizer_profile,
            event_account,
            ticket_class,
            ticket,
            system_program: solana_sdk::system_program::id(),
        }
        .to_account_metas(None),
        data: instruction::IssueCompTicket {
            class_id,
            ticket_id,
        }
        .data(),
    }
}

pub fn ix_transition_ticket_status(
    authority: Pubkey,
    protocol_config: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
    ticket_class: Pubkey,
    ticket: Pubkey,
    class_id: u16,
    ticket_id: u32,
    next_status: u8,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::TransitionTicketStatus {
            authority,
            protocol_config,
            organizer_profile,
            event_account,
            ticket_class,
            ticket,
        }
        .to_account_metas(None),
        data: instruction::TransitionTicketStatus {
            class_id,
            ticket_id,
            next_status,
        }
        .data(),
    }
}

pub fn ix_set_ticket_metadata(
    authority: Pubkey,
    protocol_config: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
    ticket_class: Pubkey,
    ticket: Pubkey,
    class_id: u16,
    ticket_id: u32,
    metadata_uri: String,
    metadata_version: u16,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::SetTicketMetadata {
            authority,
            protocol_config,
            organizer_profile,
            event_account,
            ticket_class,
            ticket,
        }
        .to_account_metas(None),
        data: instruction::SetTicketMetadata {
            class_id,
            ticket_id,
            metadata_uri,
            metadata_version,
        }
        .data(),
    }
}

pub fn ix_set_resale_policy(
    payer: Pubkey,
    authority: Pubkey,
    protocol_config: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
    ticket_class: Pubkey,
    resale_policy: Pubkey,
    class_id: u16,
    input: ResalePolicyInput,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::SetResalePolicy {
            payer,
            authority,
            protocol_config,
            organizer_profile,
            event_account,
            ticket_class,
            resale_policy,
            system_program: solana_sdk::system_program::id(),
        }
        .to_account_metas(None),
        data: instruction::SetResalePolicy { class_id, input }.data(),
    }
}

pub fn ix_list_ticket(
    seller: Pubkey,
    protocol_config: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
    ticket_class: Pubkey,
    resale_policy: Pubkey,
    ticket: Pubkey,
    listing: Pubkey,
    class_id: u16,
    ticket_id: u32,
    price_lamports: u64,
    expires_at: i64,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::ListTicket {
            seller,
            protocol_config,
            organizer_profile,
            event_account,
            ticket_class,
            resale_policy,
            ticket,
            listing,
            system_program: solana_sdk::system_program::id(),
        }
        .to_account_metas(None),
        data: instruction::ListTicket {
            class_id,
            ticket_id,
            price_lamports,
            expires_at,
        }
        .data(),
    }
}

pub fn ix_buy_resale_ticket(
    buyer: Pubkey,
    protocol_config: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
    ticket_class: Pubkey,
    resale_policy: Pubkey,
    ticket: Pubkey,
    listing: Pubkey,
    seller_wallet: Pubkey,
    royalty_vault: Pubkey,
    class_id: u16,
    ticket_id: u32,
    max_price_lamports: u64,
) -> Instruction {
    let (compliance_registry, _) = compliance_registry_pda(event_account);
    Instruction {
        program_id: ID,
        accounts: accounts::BuyResaleTicket {
            buyer,
            protocol_config,
            organizer_profile,
            event_account,
            ticket_class,
            resale_policy,
            ticket,
            listing,
            seller_wallet,
            royalty_vault,
            compliance_registry,
            system_program: solana_sdk::system_program::id(),
        }
        .to_account_metas(None),
        data: instruction::BuyResaleTicket {
            class_id,
            ticket_id,
            max_price_lamports,
        }
        .data(),
    }
}

pub fn ix_cancel_listing(
    authority: Pubkey,
    protocol_config: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
    ticket_class: Pubkey,
    ticket: Pubkey,
    listing: Pubkey,
    class_id: u16,
    ticket_id: u32,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::CancelListing {
            authority,
            protocol_config,
            organizer_profile,
            event_account,
            ticket_class,
            ticket,
            listing,
        }
        .to_account_metas(None),
        data: instruction::CancelListing {
            class_id,
            ticket_id,
        }
        .data(),
    }
}

pub fn ix_expire_listing(
    protocol_config: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
    ticket_class: Pubkey,
    ticket: Pubkey,
    listing: Pubkey,
    class_id: u16,
    ticket_id: u32,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::ExpireListing {
            protocol_config,
            organizer_profile,
            event_account,
            ticket_class,
            ticket,
            listing,
        }
        .to_account_metas(None),
        data: instruction::ExpireListing {
            class_id,
            ticket_id,
        }
        .data(),
    }
}

pub fn ix_create_financing_offer(
    payer: Pubkey,
    authority: Pubkey,
    protocol_config: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
    financing_offer: Pubkey,
    input: FinancingOfferInput,
) -> Instruction {
    let (compliance_registry, _) = compliance_registry_pda(event_account);
    Instruction {
        program_id: ID,
        accounts: accounts::CreateFinancingOffer {
            payer,
            authority,
            protocol_config,
            organizer_profile,
            event_account,
            financing_offer,
            compliance_registry,
            system_program: solana_sdk::system_program::id(),
        }
        .to_account_metas(None),
        data: instruction::CreateFinancingOffer { input }.data(),
    }
}

pub fn ix_accept_financing_offer(
    authority: Pubkey,
    protocol_config: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
    financing_offer: Pubkey,
    accept: bool,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::AcceptFinancingOffer {
            authority,
            protocol_config,
            organizer_profile,
            event_account,
            financing_offer,
        }
        .to_account_metas(None),
        data: instruction::AcceptFinancingOffer { accept }.data(),
    }
}

pub fn ix_disburse_advance(
    disburser: Pubkey,
    protocol_config: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
    financing_offer: Pubkey,
    disbursement_record: Pubkey,
    organizer_payout_wallet: Pubkey,
    amount_lamports: u64,
    reference_id: [u8; 16],
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::DisburseAdvance {
            disburser,
            protocol_config,
            organizer_profile,
            event_account,
            financing_offer,
            disbursement_record,
            organizer_payout_wallet,
            system_program: solana_sdk::system_program::id(),
        }
        .to_account_metas(None),
        data: instruction::DisburseAdvance {
            amount_lamports,
            reference_id,
        }
        .data(),
    }
}

pub fn ix_set_financing_freeze(
    admin: Pubkey,
    protocol_config: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
    financing_offer: Pubkey,
    is_frozen: bool,
    reason_code: u16,
    clawback_allowed: bool,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::SetFinancingFreeze {
            admin,
            protocol_config,
            organizer_profile,
            event_account,
            financing_offer,
        }
        .to_account_metas(None),
        data: instruction::SetFinancingFreeze {
            is_frozen,
            reason_code,
            clawback_allowed,
        }
        .data(),
    }
}

pub fn ix_clawback_disbursement(
    admin: Pubkey,
    protocol_config: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
    financing_offer: Pubkey,
    disbursement_record: Pubkey,
    organizer_payout_wallet: Pubkey,
    treasury_vault: Pubkey,
    disbursement_index: u16,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::ClawbackDisbursement {
            admin,
            protocol_config,
            organizer_profile,
            event_account,
            financing_offer,
            disbursement_record,
            organizer_payout_wallet,
            treasury_vault,
            system_program: solana_sdk::system_program::id(),
        }
        .to_account_metas(None),
        data: instruction::ClawbackDisbursement { disbursement_index }.data(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn ix_settle_primary_revenue(
    revenue_source: Pubkey,
    organizer_authority: Pubkey,
    protocol_config: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
    financing_offer: Pubkey,
    settlement_ledger: Pubkey,
    financier_wallet: Pubkey,
    organizer_payout_wallet: Pubkey,
    protocol_fee_vault: Pubkey,
    royalty_vault: Pubkey,
    other_vault: Pubkey,
    gross_revenue_lamports: u64,
    protocol_bps: u16,
    royalty_bps: u16,
    other_bps: u16,
    settlement_reference: [u8; 16],
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::SettlePrimaryRevenue {
            revenue_source,
            organizer_authority,
            protocol_config,
            organizer_profile,
            event_account,
            financing_offer,
            settlement_ledger,
            financier_wallet,
            organizer_payout_wallet,
            protocol_fee_vault,
            royalty_vault,
            other_vault,
            system_program: solana_sdk::system_program::id(),
        }
        .to_account_metas(None),
        data: instruction::SettlePrimaryRevenue {
            gross_revenue_lamports,
            protocol_bps,
            royalty_bps,
            other_bps,
            settlement_reference,
        }
        .data(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn ix_settle_resale_revenue(
    revenue_source: Pubkey,
    organizer_authority: Pubkey,
    protocol_config: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
    financing_offer: Pubkey,
    settlement_ledger: Pubkey,
    financier_wallet: Pubkey,
    organizer_payout_wallet: Pubkey,
    protocol_fee_vault: Pubkey,
    royalty_vault: Pubkey,
    other_vault: Pubkey,
    gross_revenue_lamports: u64,
    protocol_bps: u16,
    royalty_bps: u16,
    other_bps: u16,
    settlement_reference: [u8; 16],
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::SettleResaleRevenue {
            revenue_source,
            organizer_authority,
            protocol_config,
            organizer_profile,
            event_account,
            financing_offer,
            settlement_ledger,
            financier_wallet,
            organizer_payout_wallet,
            protocol_fee_vault,
            royalty_vault,
            other_vault,
            system_program: solana_sdk::system_program::id(),
        }
        .to_account_metas(None),
        data: instruction::SettleResaleRevenue {
            gross_revenue_lamports,
            protocol_bps,
            royalty_bps,
            other_bps,
            settlement_reference,
        }
        .data(),
    }
}

pub fn ix_finalize_settlement(
    organizer_authority: Pubkey,
    protocol_config: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
    financing_offer: Pubkey,
    settlement_ledger: Pubkey,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::FinalizeSettlement {
            organizer_authority,
            protocol_config,
            organizer_profile,
            event_account,
            financing_offer,
            settlement_ledger,
        }
        .to_account_metas(None),
        data: instruction::FinalizeSettlement {}.data(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn ix_refund_ticket(
    authority: Pubkey,
    protocol_config: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
    ticket_class: Pubkey,
    ticket: Pubkey,
    refund_recipient: Pubkey,
    organizer_vault: Pubkey,
    escrow_vault: Pubkey,
    reserve_vault: Pubkey,
    class_id: u16,
    ticket_id: u32,
    amount_lamports: u64,
    source: u8,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::RefundTicket {
            authority,
            protocol_config,
            organizer_profile,
            event_account,
            ticket_class,
            ticket,
            refund_recipient,
            organizer_vault,
            escrow_vault,
            reserve_vault,
            system_program: solana_sdk::system_program::id(),
        }
        .to_account_metas(None),
        data: instruction::RefundTicket {
            class_id,
            ticket_id,
            amount_lamports,
            source,
        }
        .data(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn ix_flag_dispute(
    authority: Pubkey,
    protocol_config: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
    ticket_class: Pubkey,
    ticket: Pubkey,
    class_id: u16,
    ticket_id: u32,
    is_disputed: bool,
    is_chargeback: bool,
    reason_code: u16,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::FlagDispute {
            authority,
            protocol_config,
            organizer_profile,
            event_account,
            ticket_class,
            ticket,
        }
        .to_account_metas(None),
        data: instruction::FlagDispute {
            class_id,
            ticket_id,
            is_disputed,
            is_chargeback,
            reason_code,
        }
        .data(),
    }
}

pub fn ix_set_global_loyalty_multiplier(
    admin: Pubkey,
    protocol_config: Pubkey,
    multiplier_bps: u16,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::SetGlobalLoyaltyMultiplier {
            admin,
            protocol_config,
        }
        .to_account_metas(None),
        data: instruction::SetGlobalLoyaltyMultiplier { multiplier_bps }.data(),
    }
}

pub fn ix_set_event_loyalty_multiplier(
    authority: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
    multiplier_bps: u16,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::SetEventLoyaltyMultiplier {
            authority,
            organizer_profile,
            event_account,
        }
        .to_account_metas(None),
        data: instruction::SetEventLoyaltyMultiplier { multiplier_bps }.data(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn ix_accrue_points(
    payer: Pubkey,
    authority: Pubkey,
    wallet: Pubkey,
    protocol_config: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
    ticket_class: Pubkey,
    ticket: Pubkey,
    loyalty_ledger: Pubkey,
    class_id: u16,
    ticket_id: u32,
    reason: u8,
    base_points: u64,
    hold_duration_days: u16,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::AccruePoints {
            payer,
            authority,
            wallet,
            protocol_config,
            organizer_profile,
            event_account,
            ticket_class,
            ticket,
            loyalty_ledger,
            system_program: solana_sdk::system_program::id(),
        }
        .to_account_metas(None),
        data: instruction::AccruePoints {
            class_id,
            ticket_id,
            reason,
            base_points,
            hold_duration_days,
        }
        .data(),
    }
}

pub fn ix_redeem_points(
    wallet: Pubkey,
    protocol_config: Pubkey,
    loyalty_ledger: Pubkey,
    points_to_burn: u64,
    perk_code: String,
    event: Pubkey,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::RedeemPoints {
            wallet,
            protocol_config,
            loyalty_ledger,
        }
        .to_account_metas(None),
        data: instruction::RedeemPoints {
            points_to_burn,
            perk_code,
            event,
        }
        .data(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn ix_record_purchase_input(
    payer: Pubkey,
    authority: Pubkey,
    wallet: Pubkey,
    protocol_config: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
    ticket_class: Pubkey,
    ticket: Pubkey,
    trust_signal: Pubkey,
    class_id: u16,
    ticket_id: u32,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::RecordPurchaseInput {
            payer,
            authority,
            wallet,
            protocol_config,
            organizer_profile,
            event_account,
            ticket_class,
            ticket,
            trust_signal,
            system_program: solana_sdk::system_program::id(),
        }
        .to_account_metas(None),
        data: instruction::RecordPurchaseInput {
            class_id,
            ticket_id,
        }
        .data(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn ix_record_attendance_input(
    payer: Pubkey,
    authority: Pubkey,
    wallet: Pubkey,
    protocol_config: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
    ticket_class: Pubkey,
    ticket: Pubkey,
    trust_signal: Pubkey,
    class_id: u16,
    ticket_id: u32,
    did_attend: bool,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::RecordAttendanceInput {
            payer,
            authority,
            wallet,
            protocol_config,
            organizer_profile,
            event_account,
            ticket_class,
            ticket,
            trust_signal,
            system_program: solana_sdk::system_program::id(),
        }
        .to_account_metas(None),
        data: instruction::RecordAttendanceInput {
            class_id,
            ticket_id,
            did_attend,
        }
        .data(),
    }
}

pub fn ix_flag_trust_abuse(
    admin: Pubkey,
    wallet: Pubkey,
    protocol_config: Pubkey,
    trust_signal: Pubkey,
    flag_bits: u32,
    event: Pubkey,
    ticket: Pubkey,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::FlagAbuse {
            admin,
            wallet,
            protocol_config,
            trust_signal,
            system_program: solana_sdk::system_program::id(),
        }
        .to_account_metas(None),
        data: instruction::FlagTrustAbuse {
            flag_bits,
            event,
            ticket,
        }
        .data(),
    }
}

pub fn ix_set_trust_signal_schema_version(
    admin: Pubkey,
    protocol_config: Pubkey,
    trust_signal: Pubkey,
    new_schema_version: u16,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::SetSchemaVersion {
            admin,
            protocol_config,
            trust_signal,
        }
        .to_account_metas(None),
        data: instruction::SetTrustSignalSchemaVersion { new_schema_version }.data(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn ix_initialize_vault(
    payer: Pubkey,
    authority: Pubkey,
    protocol_config: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
    financing_offer: Pubkey,
    vault_state: Pubkey,
    vault: Pubkey,
    role_binding: Pubkey,
    kind: u8,
    parent: Pubkey,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::InitializeVault {
            payer,
            authority,
            protocol_config,
            organizer_profile,
            event_account,
            financing_offer,
            vault_state,
            vault,
            role_binding,
            system_program: solana_sdk::system_program::id(),
        }
        .to_account_metas(None),
        data: instruction::InitializeVault { kind, parent }.data(),
    }
}

pub fn ix_snapshot_vault(
    vault_state: Pubkey,
    vault: Pubkey,
    kind: u8,
    parent: Pubkey,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::SnapshotVault { vault_state, vault }.to_account_metas(None),
        data: instruction::SnapshotVault { kind, parent }.data(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn ix_withdraw_vault(
    authority: Pubkey,
    vault_state: Pubkey,
    vault: Pubkey,
    destination: Pubkey,
    role_binding: Pubkey,
    kind: u8,
    parent: Pubkey,
    amount_lamports: u64,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::WithdrawVault {
            authority,
            vault_state,
            vault,
            destination,
            role_binding,
            system_program: solana_sdk::system_program::id(),
        }
        .to_account_metas(None),
        data: instruction::WithdrawVault {
            kind,
            parent,
            amount_lamports,
        }
        .data(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn ix_upsert_registry_entry(
    payer: Pubkey,
    authority: Pubkey,
    protocol_config: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
    compliance_registry: Pubkey,
    scope: u8,
    target: Pubkey,
    subject: Pubkey,
    list_type: u8,
    is_allowed: bool,
    decision_code: u16,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::UpsertRegistryEntry {
            payer,
            authority,
            protocol_config,
            organizer_profile,
            event_account,
            compliance_registry,
            system_program: solana_sdk::system_program::id(),
        }
        .to_account_metas(None),
        data: instruction::UpsertRegistryEntry {
            scope,
            target,
            subject,
            list_type,
            is_allowed,
            decision_code,
        }
        .data(),
    }
}

pub fn ix_set_event_restrictions(
    authority: Pubkey,
    protocol_config: Pubkey,
    organizer_profile: Pubkey,
    event_account: Pubkey,
    restriction_flags: u32,
    decision_code: u16,
) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts::SetEventRestrictions {
            authority,
            protocol_config,
            organizer_profile,
            event_account,
        }
        .to_account_metas(None),
        data: instruction::SetEventRestrictions {
            restriction_flags,
            decision_code,
        }
        .data(),
    }
}
