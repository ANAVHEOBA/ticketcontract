use solana_sdk::signature::{Keypair, Signer};

use ticketing_core::{
    instructions::{event::EventInput, ticket_class::TicketClassInput},
    state::{FinancingLifecycleStatus, TicketStatus},
};

use crate::common::{
    disbursement_record_pda, event_pda, fetch_financing_offer, fetch_loyalty_ledger,
    fetch_settlement_ledger, fetch_ticket, fetch_ticket_class, financing_offer_pda, get_lamports,
    ix_accept_financing_offer, ix_accrue_points, ix_buy_resale_ticket, ix_buy_ticket,
    ix_check_in_ticket, ix_create_event, ix_create_financing_offer, ix_create_organizer,
    ix_create_ticket_class, ix_disburse_advance, ix_finalize_settlement, ix_initialize_protocol,
    ix_list_ticket, ix_pause_protocol, ix_set_resale_policy, ix_settle_primary_revenue,
    ix_set_ticket_metadata, listing_pda, loyalty_ledger_pda, organizer_pda, protocol_config_pda,
    resale_policy_pda, send_ix, send_ix_result, settlement_ledger_pda, setup, ticket_class_pda,
    ticket_pda, wallet_purchase_counter_pda,
};

#[tokio::test]
async fn full_revenue_and_ticket_lifecycle_flow() {
    let mut ctx = setup().await;

    let organizer_authority = Keypair::new();
    let organizer_wallet = Keypair::new();
    let fee_vault = Keypair::new();
    let treasury_vault = Keypair::new();
    let stakeholder_wallet = Keypair::new();
    let royalty_vault = Keypair::new();
    let other_vault = Keypair::new();
    let financier_wallet = Keypair::new();
    let underwriter = Keypair::new();
    let buyer_primary = Keypair::new();
    let buyer_resale = Keypair::new();
    let revenue_source = Keypair::new();

    for kp in [
        &organizer_authority,
        &organizer_wallet,
        &fee_vault,
        &treasury_vault,
        &stakeholder_wallet,
        &royalty_vault,
        &other_vault,
        &financier_wallet,
        &underwriter,
        &buyer_primary,
        &buyer_resale,
        &revenue_source,
    ] {
        send_ix(
            &mut ctx.banks_client,
            &ctx.payer,
            ctx.last_blockhash,
            solana_sdk::system_instruction::transfer(
                &ctx.payer.pubkey(),
                &kp.pubkey(),
                8_000_000_000,
            ),
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
            treasury_vault.pubkey(),
            fee_vault.pubkey(),
            500,
            10,
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
            "https://org/e2e".to_string(),
            organizer_wallet.pubkey(),
        ),
        &[&organizer_authority],
    )
    .await;

    let event_id = 4040u64;
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
                title: "Hackathon Main Event".to_string(),
                venue: "Hall A".to_string(),
                start_ts: 4_902_444_800,
                end_ts: 4_902_448_400,
                sales_start_ts: 0,
                lock_ts: 4_902_440_000,
                capacity: 10_000,
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
                name: "VIP".to_string(),
                total_supply: 100,
                reserved_supply: 0,
                face_price_lamports: 1_000_000_000,
                sale_start_ts: 0,
                sale_end_ts: i64::MAX,
                per_wallet_limit: 3,
                is_transferable: true,
                is_resale_enabled: true,
                stakeholder_wallet: stakeholder_wallet.pubkey(),
                stakeholder_bps: 500,
            },
        ),
        &[&organizer_authority],
    )
    .await;

    let (ticket, _) = ticket_pda(event_account, class_id, ticket_id);
    let (counter, _) =
        wallet_purchase_counter_pda(event_account, ticket_class, buyer_primary.pubkey());
    send_ix(
        &mut ctx.banks_client,
        &buyer_primary,
        ctx.last_blockhash,
        ix_buy_ticket(
            buyer_primary.pubkey(),
            protocol_config,
            organizer_profile,
            event_account,
            ticket_class,
            ticket,
            counter,
            fee_vault.pubkey(),
            organizer_wallet.pubkey(),
            stakeholder_wallet.pubkey(),
            class_id,
            ticket_id,
            1_000_000_000,
        ),
        &[],
    )
    .await;

    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_set_ticket_metadata(
            organizer_authority.pubkey(),
            protocol_config,
            organizer_profile,
            event_account,
            ticket_class,
            ticket,
            class_id,
            ticket_id,
            "ipfs://ticket/e2e/1".to_string(),
            1,
        ),
        &[&organizer_authority],
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
            ticketing_core::state::ResalePolicyInput {
                max_markup_bps: 2_000,
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

    let (listing, _) = listing_pda(ticket);
    send_ix(
        &mut ctx.banks_client,
        &buyer_primary,
        ctx.last_blockhash,
        ix_list_ticket(
            buyer_primary.pubkey(),
            protocol_config,
            organizer_profile,
            event_account,
            ticket_class,
            resale_policy,
            ticket,
            listing,
            class_id,
            ticket_id,
            1_100_000_000,
            i64::MAX,
        ),
        &[],
    )
    .await;

    send_ix(
        &mut ctx.banks_client,
        &buyer_resale,
        ctx.last_blockhash,
        ix_buy_resale_ticket(
            buyer_resale.pubkey(),
            protocol_config,
            organizer_profile,
            event_account,
            ticket_class,
            resale_policy,
            ticket,
            listing,
            buyer_primary.pubkey(),
            royalty_vault.pubkey(),
            class_id,
            ticket_id,
            1_100_000_000,
        ),
        &[],
    )
    .await;

    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_check_in_ticket(
            organizer_authority.pubkey(),
            protocol_config,
            organizer_profile,
            event_account,
            ticket_class,
            ticket,
            protocol_config,
            class_id,
            ticket_id,
            "GATE-1".to_string(),
        ),
        &[&organizer_authority],
    )
    .await;

    let (loyalty_ledger, _) = loyalty_ledger_pda(buyer_resale.pubkey());
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_accrue_points(
            ctx.payer.pubkey(),
            organizer_authority.pubkey(),
            buyer_resale.pubkey(),
            protocol_config,
            organizer_profile,
            event_account,
            ticket_class,
            ticket,
            loyalty_ledger,
            class_id,
            ticket_id,
            2,
            100,
            0,
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
            underwriter.pubkey(),
            protocol_config,
            organizer_profile,
            event_account,
            financing_offer,
            ticketing_core::state::FinancingOfferInput {
                advance_amount_lamports: 1_000_000_000,
                fee_bps: 700,
                repayment_cap_lamports: 1_200_000_000,
                schedule_start_ts: 4_902_000_000,
                schedule_interval_secs: 86_400,
                schedule_installments: 3,
            },
        ),
        &[&underwriter],
    )
    .await;

    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_accept_financing_offer(
            organizer_authority.pubkey(),
            protocol_config,
            organizer_profile,
            event_account,
            financing_offer,
            true,
        ),
        &[&organizer_authority],
    )
    .await;

    let (disbursement_record, _) = disbursement_record_pda(financing_offer, 1);
    send_ix(
        &mut ctx.banks_client,
        &underwriter,
        ctx.last_blockhash,
        ix_disburse_advance(
            underwriter.pubkey(),
            protocol_config,
            organizer_profile,
            event_account,
            financing_offer,
            disbursement_record,
            organizer_wallet.pubkey(),
            1_000_000_000,
            [1u8; 16],
        ),
        &[],
    )
    .await;

    let (settlement_ledger, _) = settlement_ledger_pda(event_account);
    send_ix(
        &mut ctx.banks_client,
        &revenue_source,
        ctx.last_blockhash,
        ix_settle_primary_revenue(
            revenue_source.pubkey(),
            organizer_authority.pubkey(),
            protocol_config,
            organizer_profile,
            event_account,
            financing_offer,
            settlement_ledger,
            financier_wallet.pubkey(),
            organizer_wallet.pubkey(),
            fee_vault.pubkey(),
            royalty_vault.pubkey(),
            other_vault.pubkey(),
            1_500_000_000,
            500,
            500,
            500,
            [9u8; 16],
        ),
        &[&organizer_authority],
    )
    .await;

    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_finalize_settlement(
            organizer_authority.pubkey(),
            protocol_config,
            organizer_profile,
            event_account,
            financing_offer,
            settlement_ledger,
        ),
        &[&organizer_authority],
    )
    .await;

    let ticket_after = fetch_ticket(&mut ctx.banks_client, ticket).await;
    assert_eq!(ticket_after.owner, buyer_resale.pubkey());
    assert_eq!(ticket_after.status, TicketStatus::CheckedIn);
    assert!(ticket_after.check_in_count >= 1);

    let class_after = fetch_ticket_class(&mut ctx.banks_client, ticket_class).await;
    assert_eq!(class_after.sold_supply, 1);
    assert_eq!(class_after.remaining_supply, 99);

    let offer_after = fetch_financing_offer(&mut ctx.banks_client, financing_offer).await;
    assert_eq!(offer_after.status, FinancingLifecycleStatus::Settled);
    assert_eq!(offer_after.total_disbursed_lamports, 1_000_000_000);

    let ledger_after = fetch_settlement_ledger(&mut ctx.banks_client, settlement_ledger).await;
    assert!(ledger_after.financing_settled);
    assert_eq!(ledger_after.cumulative_primary_routed_lamports, 1_500_000_000);
    assert!(ledger_after.cumulative_financier_paid_lamports >= 1_200_000_000);

    let loyalty_after = fetch_loyalty_ledger(&mut ctx.banks_client, loyalty_ledger).await;
    assert_eq!(loyalty_after.wallet, buyer_resale.pubkey());
    assert!(loyalty_after.available_points > 0);

    assert!(get_lamports(&mut ctx.banks_client, financier_wallet.pubkey()).await > 8_000_000_000);
    assert!(get_lamports(&mut ctx.banks_client, fee_vault.pubkey()).await > 8_000_000_000);
    assert!(get_lamports(&mut ctx.banks_client, royalty_vault.pubkey()).await > 8_000_000_000);

    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_pause_protocol(ctx.payer.pubkey(), protocol_config, true),
        &[],
    )
    .await;

    let paused_create_offer = send_ix_result(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_create_financing_offer(
            ctx.payer.pubkey(),
            underwriter.pubkey(),
            protocol_config,
            organizer_profile,
            event_account,
            financing_offer,
            ticketing_core::state::FinancingOfferInput {
                advance_amount_lamports: 10,
                fee_bps: 1,
                repayment_cap_lamports: 10,
                schedule_start_ts: 1,
                schedule_interval_secs: 1,
                schedule_installments: 1,
            },
        ),
        &[&underwriter],
    )
    .await;
    assert!(paused_create_offer.is_err());
}
