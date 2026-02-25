use anchor_lang::{AnchorSerialize, Discriminator};
use solana_sdk::signature::{Keypair, Signer};

use ticketing_core::{
    constants::{
        EVENT_ACCOUNT_SCHEMA_VERSION, FINANCING_OFFER_SCHEMA_VERSION, LOYALTY_LEDGER_SCHEMA_VERSION,
        RESALE_POLICY_SCHEMA_VERSION, SCHEMA_VERSION_V0, TICKET_ACCOUNT_SCHEMA_VERSION,
    },
    instructions::{event::EventInput, ticket_class::TicketClassInput},
    migrations::{
        deserialize_event_account_compat, deserialize_financing_offer_compat,
        deserialize_loyalty_ledger_compat, deserialize_resale_policy_compat,
        deserialize_ticket_compat, EventAccountV0, FinancingOfferV0, LoyaltyLedgerV0,
        ResalePolicyV0, TicketV0,
    },
    state::{FinancingOfferInput, ResalePolicyInput},
};

use crate::common::{
    event_pda, fetch_event_account, fetch_financing_offer, fetch_loyalty_ledger,
    fetch_resale_policy, fetch_ticket, financing_offer_pda, ix_accrue_points, ix_buy_ticket,
    ix_create_event, ix_create_financing_offer, ix_create_organizer, ix_create_ticket_class,
    ix_initialize_protocol, ix_set_resale_policy, loyalty_ledger_pda, organizer_pda,
    protocol_config_pda, resale_policy_pda, send_ix, setup, ticket_class_pda, ticket_pda,
    wallet_purchase_counter_pda,
};

fn encode_legacy_account<TLegacy: AnchorSerialize, TCurrent: Discriminator>(legacy: &TLegacy) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(TCurrent::DISCRIMINATOR);
    legacy.serialize(&mut data).unwrap();
    data
}

struct Fixture {
    event_account: solana_sdk::pubkey::Pubkey,
    ticket: solana_sdk::pubkey::Pubkey,
    resale_policy: solana_sdk::pubkey::Pubkey,
    financing_offer: solana_sdk::pubkey::Pubkey,
    loyalty_ledger: solana_sdk::pubkey::Pubkey,
}

async fn setup_fixture() -> (solana_program_test::ProgramTestContext, Fixture) {
    let mut ctx = setup().await;
    let organizer_authority = Keypair::new();
    let buyer = Keypair::new();
    let organizer_wallet = Keypair::new();
    let fee_vault = Keypair::new();
    let royalty_vault = Keypair::new();

    for kp in [
        &organizer_authority,
        &buyer,
        &organizer_wallet,
        &fee_vault,
        &royalty_vault,
    ] {
        let fund_ix = solana_sdk::system_instruction::transfer(
            &ctx.payer.pubkey(),
            &kp.pubkey(),
            5_000_000_000,
        );
        send_ix(
            &mut ctx.banks_client,
            &ctx.payer,
            ctx.last_blockhash,
            fund_ix,
            &[],
        )
        .await;
    }

    let (protocol_config, _) = protocol_config_pda();
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_initialize_protocol(
            ctx.payer.pubkey(),
            ctx.payer.pubkey(),
            protocol_config,
            Keypair::new().pubkey(),
            Keypair::new().pubkey(),
            fee_vault.pubkey(),
            500,
            8,
        ),
        &[],
    )
    .await;

    let (organizer_profile, _) = organizer_pda(organizer_authority.pubkey());
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_create_organizer(
            ctx.payer.pubkey(),
            organizer_authority.pubkey(),
            protocol_config,
            organizer_profile,
            "https://org/schema".to_string(),
            organizer_wallet.pubkey(),
        ),
        &[&organizer_authority],
    )
    .await;

    let event_id = 5050;
    let class_id = 1u16;
    let ticket_id = 1u32;

    let (event_account, _) = event_pda(organizer_profile, event_id);
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_create_event(
            ctx.payer.pubkey(),
            organizer_authority.pubkey(),
            protocol_config,
            organizer_profile,
            event_account,
            event_id,
            EventInput {
                title: "Schema Event".to_string(),
                venue: "Hall 7".to_string(),
                start_ts: 4_802_444_800,
                end_ts: 4_802_448_400,
                sales_start_ts: 4_802_430_000,
                lock_ts: 4_802_440_000,
                capacity: 150,
            },
        ),
        &[&organizer_authority],
    )
    .await;

    let (ticket_class, _) = ticket_class_pda(event_account, class_id);
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_create_ticket_class(
            ctx.payer.pubkey(),
            organizer_authority.pubkey(),
            protocol_config,
            organizer_profile,
            event_account,
            ticket_class,
            class_id,
            TicketClassInput {
                name: "GA".to_string(),
                total_supply: 20,
                reserved_supply: 0,
                face_price_lamports: 1_000_000_000,
                sale_start_ts: 0,
                sale_end_ts: i64::MAX,
                per_wallet_limit: 2,
                is_transferable: true,
                is_resale_enabled: true,
                stakeholder_wallet: Keypair::new().pubkey(),
                stakeholder_bps: 0,
            },
        ),
        &[&organizer_authority],
    )
    .await;

    let (ticket, _) = ticket_pda(event_account, class_id, ticket_id);
    let (wallet_counter, _) = wallet_purchase_counter_pda(event_account, ticket_class, buyer.pubkey());
    send_ix(
        &mut ctx.banks_client,
        &buyer,
        ctx.last_blockhash,
        ix_buy_ticket(
            buyer.pubkey(),
            protocol_config,
            organizer_profile,
            event_account,
            ticket_class,
            ticket,
            wallet_counter,
            fee_vault.pubkey(),
            organizer_wallet.pubkey(),
            Keypair::new().pubkey(),
            class_id,
            ticket_id,
            1_000_000_000,
        ),
        &[],
    )
    .await;

    let (resale_policy, _) = resale_policy_pda(event_account, class_id);
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_set_resale_policy(
            ctx.payer.pubkey(),
            organizer_authority.pubkey(),
            protocol_config,
            organizer_profile,
            event_account,
            ticket_class,
            resale_policy,
            class_id,
            ResalePolicyInput {
                max_markup_bps: 1_500,
                royalty_bps: 1_000,
                royalty_vault: royalty_vault.pubkey(),
                transfer_cooldown_secs: 0,
                max_transfer_count: 5,
                transfer_lock_before_event_secs: 0,
                whitelist: vec![],
                blacklist: vec![],
            },
        ),
        &[&organizer_authority],
    )
    .await;

    let (financing_offer, _) = financing_offer_pda(event_account);
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_create_financing_offer(
            ctx.payer.pubkey(),
            ctx.payer.pubkey(),
            protocol_config,
            organizer_profile,
            event_account,
            financing_offer,
            FinancingOfferInput {
                advance_amount_lamports: 1_000_000_000,
                fee_bps: 700,
                repayment_cap_lamports: 1_250_000_000,
                schedule_start_ts: 4_802_000_000,
                schedule_interval_secs: 86_400,
                schedule_installments: 4,
            },
        ),
        &[],
    )
    .await;

    let (loyalty_ledger, _) = loyalty_ledger_pda(buyer.pubkey());
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_accrue_points(
            ctx.payer.pubkey(),
            organizer_authority.pubkey(),
            buyer.pubkey(),
            protocol_config,
            organizer_profile,
            event_account,
            ticket_class,
            ticket,
            loyalty_ledger,
            class_id,
            ticket_id,
            1,
            100,
            0,
        ),
        &[&organizer_authority],
    )
    .await;

    (
        ctx,
        Fixture {
            event_account,
            ticket,
            resale_policy,
            financing_offer,
            loyalty_ledger,
        },
    )
}

#[tokio::test]
async fn major_accounts_are_versioned_and_not_deprecated_on_create() {
    let (mut ctx, fixture) = setup_fixture().await;

    let event = fetch_event_account(&mut ctx.banks_client, fixture.event_account).await;
    assert_eq!(event.schema_version, EVENT_ACCOUNT_SCHEMA_VERSION);
    assert_eq!(event.deprecated_layout_version, 0);
    assert_eq!(event.deprecated_at, 0);

    let ticket = fetch_ticket(&mut ctx.banks_client, fixture.ticket).await;
    assert_eq!(ticket.schema_version, TICKET_ACCOUNT_SCHEMA_VERSION);
    assert_eq!(ticket.deprecated_layout_version, 0);
    assert_eq!(ticket.deprecated_at, 0);

    let policy = fetch_resale_policy(&mut ctx.banks_client, fixture.resale_policy).await;
    assert_eq!(policy.schema_version, RESALE_POLICY_SCHEMA_VERSION);
    assert_eq!(policy.deprecated_layout_version, 0);
    assert_eq!(policy.deprecated_at, 0);

    let offer = fetch_financing_offer(&mut ctx.banks_client, fixture.financing_offer).await;
    assert_eq!(offer.schema_version, FINANCING_OFFER_SCHEMA_VERSION);
    assert_eq!(offer.deprecated_layout_version, 0);
    assert_eq!(offer.deprecated_at, 0);

    let ledger = fetch_loyalty_ledger(&mut ctx.banks_client, fixture.loyalty_ledger).await;
    assert_eq!(ledger.schema_version, LOYALTY_LEDGER_SCHEMA_VERSION);
    assert_eq!(ledger.deprecated_layout_version, 0);
    assert_eq!(ledger.deprecated_at, 0);
}

#[tokio::test]
async fn compat_deserializers_support_legacy_v0_layouts() {
    let (mut ctx, fixture) = setup_fixture().await;

    let event = fetch_event_account(&mut ctx.banks_client, fixture.event_account).await;
    let legacy_event = EventAccountV0 {
        bump: event.bump,
        organizer: event.organizer,
        event_id: event.event_id,
        title: event.title.clone(),
        venue: event.venue.clone(),
        start_ts: event.start_ts,
        end_ts: event.end_ts,
        sales_start_ts: event.sales_start_ts,
        lock_ts: event.lock_ts,
        capacity: event.capacity,
        loyalty_multiplier_bps: event.loyalty_multiplier_bps,
        compliance_restriction_flags: event.compliance_restriction_flags,
        is_paused: event.is_paused,
        status: event.status,
        created_at: event.created_at,
        updated_at: event.updated_at,
    };
    let raw_event = encode_legacy_account::<_, ticketing_core::state::EventAccount>(&legacy_event);
    let mut compat_event = deserialize_event_account_compat(&raw_event).unwrap();
    assert_eq!(compat_event.schema_version, SCHEMA_VERSION_V0);
    compat_event.mark_layout_deprecated(SCHEMA_VERSION_V0, fixture.event_account, 111);
    assert_eq!(compat_event.deprecated_layout_version, SCHEMA_VERSION_V0);
    assert_eq!(compat_event.replacement_account, fixture.event_account);
    assert_eq!(compat_event.deprecated_at, 111);

    let ticket = fetch_ticket(&mut ctx.banks_client, fixture.ticket).await;
    let legacy_ticket = TicketV0 {
        bump: ticket.bump,
        event: ticket.event,
        ticket_class: ticket.ticket_class,
        owner: ticket.owner,
        buyer: ticket.buyer,
        ticket_id: ticket.ticket_id,
        status: ticket.status,
        paid_amount_lamports: ticket.paid_amount_lamports,
        is_comp: ticket.is_comp,
        created_at: ticket.created_at,
        status_updated_at: ticket.status_updated_at,
        checked_in_at: ticket.checked_in_at,
        last_check_in_at: ticket.last_check_in_at,
        check_in_count: ticket.check_in_count,
        last_check_in_gate_id: ticket.last_check_in_gate_id.clone(),
        refunded_at: ticket.refunded_at,
        refund_source: ticket.refund_source,
        refund_amount_lamports: ticket.refund_amount_lamports,
        invalidated_at: ticket.invalidated_at,
        is_disputed: ticket.is_disputed,
        is_chargeback: ticket.is_chargeback,
        disputed_at: ticket.disputed_at,
        dispute_reason_code: ticket.dispute_reason_code,
        dispute_updated_at: ticket.dispute_updated_at,
        metadata_uri: ticket.metadata_uri.clone(),
        metadata_version: ticket.metadata_version,
        metadata_updated_at: ticket.metadata_updated_at,
        transfer_count: ticket.transfer_count,
        last_transfer_at: ticket.last_transfer_at,
        compliance_decision_code: ticket.compliance_decision_code,
        compliance_checked_at: ticket.compliance_checked_at,
        purchase_trust_recorded: ticket.purchase_trust_recorded,
        attendance_trust_recorded: ticket.attendance_trust_recorded,
    };
    let raw_ticket = encode_legacy_account::<_, ticketing_core::state::Ticket>(&legacy_ticket);
    let compat_ticket = deserialize_ticket_compat(&raw_ticket).unwrap();
    assert_eq!(compat_ticket.schema_version, SCHEMA_VERSION_V0);
    assert_eq!(compat_ticket.owner, ticket.owner);

    let policy = fetch_resale_policy(&mut ctx.banks_client, fixture.resale_policy).await;
    let legacy_policy = ResalePolicyV0 {
        bump: policy.bump,
        event: policy.event,
        ticket_class: policy.ticket_class,
        class_id: policy.class_id,
        max_markup_bps: policy.max_markup_bps,
        royalty_bps: policy.royalty_bps,
        royalty_vault: policy.royalty_vault,
        transfer_cooldown_secs: policy.transfer_cooldown_secs,
        max_transfer_count: policy.max_transfer_count,
        transfer_lock_before_event_secs: policy.transfer_lock_before_event_secs,
        whitelist: policy.whitelist.clone(),
        blacklist: policy.blacklist.clone(),
        created_at: policy.created_at,
        updated_at: policy.updated_at,
    };
    let raw_policy =
        encode_legacy_account::<_, ticketing_core::state::ResalePolicy>(&legacy_policy);
    let compat_policy = deserialize_resale_policy_compat(&raw_policy).unwrap();
    assert_eq!(compat_policy.schema_version, SCHEMA_VERSION_V0);
    assert_eq!(compat_policy.class_id, policy.class_id);

    let offer = fetch_financing_offer(&mut ctx.banks_client, fixture.financing_offer).await;
    let legacy_offer = FinancingOfferV0 {
        bump: offer.bump,
        event: offer.event,
        organizer: offer.organizer,
        offer_authority: offer.offer_authority,
        advance_amount_lamports: offer.advance_amount_lamports,
        fee_bps: offer.fee_bps,
        repayment_cap_lamports: offer.repayment_cap_lamports,
        schedule_start_ts: offer.schedule_start_ts,
        schedule_interval_secs: offer.schedule_interval_secs,
        schedule_installments: offer.schedule_installments,
        max_disbursements: offer.max_disbursements,
        status: offer.status,
        terms_locked: offer.terms_locked,
        financing_frozen: offer.financing_frozen,
        clawback_allowed: offer.clawback_allowed,
        freeze_reason_code: offer.freeze_reason_code,
        accepted_by: offer.accepted_by,
        accepted_at: offer.accepted_at,
        rejected_by: offer.rejected_by,
        rejected_at: offer.rejected_at,
        total_disbursed_lamports: offer.total_disbursed_lamports,
        disbursement_count: offer.disbursement_count,
        disbursed_at: offer.disbursed_at,
        compliance_decision_code: offer.compliance_decision_code,
        compliance_checked_at: offer.compliance_checked_at,
        created_at: offer.created_at,
        updated_at: offer.updated_at,
    };
    let raw_offer =
        encode_legacy_account::<_, ticketing_core::state::FinancingOffer>(&legacy_offer);
    let compat_offer = deserialize_financing_offer_compat(&raw_offer).unwrap();
    assert_eq!(compat_offer.schema_version, SCHEMA_VERSION_V0);
    assert_eq!(
        compat_offer.advance_amount_lamports,
        offer.advance_amount_lamports
    );

    let ledger = fetch_loyalty_ledger(&mut ctx.banks_client, fixture.loyalty_ledger).await;
    let legacy_ledger = LoyaltyLedgerV0 {
        bump: ledger.bump,
        wallet: ledger.wallet,
        total_accrued_points: ledger.total_accrued_points,
        total_redeemed_points: ledger.total_redeemed_points,
        available_points: ledger.available_points,
        last_event: ledger.last_event,
        last_reason: ledger.last_reason,
        last_accrued_at: ledger.last_accrued_at,
        last_redeemed_at: ledger.last_redeemed_at,
        created_at: ledger.created_at,
        updated_at: ledger.updated_at,
    };
    let raw_ledger =
        encode_legacy_account::<_, ticketing_core::state::LoyaltyLedger>(&legacy_ledger);
    let compat_ledger = deserialize_loyalty_ledger_compat(&raw_ledger).unwrap();
    assert_eq!(compat_ledger.schema_version, SCHEMA_VERSION_V0);
    assert_eq!(compat_ledger.wallet, ledger.wallet);
}
