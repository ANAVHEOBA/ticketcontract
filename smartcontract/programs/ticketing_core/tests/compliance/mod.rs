use solana_sdk::signature::{Keypair, Signer};

use ticketing_core::{
    constants::{COMPLIANCE_FLAG_BLOCK_FINANCING, COMPLIANCE_LIST_DENY, COMPLIANCE_SCOPE_EVENT},
    instructions::{event::EventInput, ticket_class::TicketClassInput},
};

use crate::common::{
    compliance_registry_pda, event_pda, financing_offer_pda, ix_buy_ticket, ix_create_event,
    ix_create_financing_offer, ix_create_organizer, ix_create_ticket_class, ix_initialize_protocol,
    ix_set_event_restrictions, ix_upsert_registry_entry, organizer_pda, protocol_config_pda,
    send_ix, send_ix_result, setup, ticket_class_pda, ticket_pda, wallet_purchase_counter_pda,
};

#[tokio::test]
async fn denylisted_wallet_cannot_buy_ticket() {
    let mut ctx = setup().await;
    let organizer_authority = Keypair::new();
    let buyer = Keypair::new();
    let organizer_wallet = Keypair::new();
    let fee_vault = Keypair::new();

    for kp in [&organizer_authority, &buyer, &organizer_wallet, &fee_vault] {
        send_ix(
            &mut ctx.banks_client,
            &ctx.payer,
            ctx.last_blockhash,
            solana_sdk::system_instruction::transfer(
                &ctx.payer.pubkey(),
                &kp.pubkey(),
                3_000_000_000,
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
            "https://org/compliance".to_string(),
            organizer_wallet.pubkey(),
        ),
        &[&organizer_authority],
    )
    .await;

    let event_id = 4401;
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
                title: "Compliance Event".to_string(),
                venue: "Venue".to_string(),
                start_ts: 4_602_444_800,
                end_ts: 4_602_448_400,
                sales_start_ts: 0,
                lock_ts: 4_602_440_000,
                capacity: 100,
            },
        ),
        &[&organizer_authority],
    )
    .await;

    let class_id = 1;
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
                total_supply: 10,
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

    let (compliance_registry, _) = compliance_registry_pda(event_account);
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_upsert_registry_entry(
            ctx.payer.pubkey(),
            organizer_authority.pubkey(),
            protocol_config,
            organizer_profile,
            event_account,
            compliance_registry,
            COMPLIANCE_SCOPE_EVENT,
            event_account,
            buyer.pubkey(),
            COMPLIANCE_LIST_DENY,
            true,
            8001,
        ),
        &[&organizer_authority],
    )
    .await;

    let (ticket, _) = ticket_pda(event_account, class_id, 1);
    let (counter, _) = wallet_purchase_counter_pda(event_account, ticket_class, buyer.pubkey());
    let err = send_ix_result(
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
            counter,
            fee_vault.pubkey(),
            organizer_wallet.pubkey(),
            Keypair::new().pubkey(),
            class_id,
            1,
            1_000_000_000,
        ),
        &[],
    )
    .await;
    assert!(err.is_err());
}

#[tokio::test]
async fn financing_block_flag_prevents_offer_creation() {
    let mut ctx = setup().await;
    let organizer_authority = Keypair::new();
    let organizer_wallet = Keypair::new();
    let fee_vault = Keypair::new();

    for kp in [&organizer_authority, &organizer_wallet, &fee_vault] {
        send_ix(
            &mut ctx.banks_client,
            &ctx.payer,
            ctx.last_blockhash,
            solana_sdk::system_instruction::transfer(
                &ctx.payer.pubkey(),
                &kp.pubkey(),
                3_000_000_000,
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
            "https://org/compliance-2".to_string(),
            organizer_wallet.pubkey(),
        ),
        &[&organizer_authority],
    )
    .await;

    let event_id = 4402;
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
                title: "Compliance Event 2".to_string(),
                venue: "Venue".to_string(),
                start_ts: 4_602_444_800,
                end_ts: 4_602_448_400,
                sales_start_ts: 0,
                lock_ts: 4_602_440_000,
                capacity: 100,
            },
        ),
        &[&organizer_authority],
    )
    .await;

    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_set_event_restrictions(
            organizer_authority.pubkey(),
            protocol_config,
            organizer_profile,
            event_account,
            COMPLIANCE_FLAG_BLOCK_FINANCING,
            8101,
        ),
        &[&organizer_authority],
    )
    .await;

    let (financing_offer, _) = financing_offer_pda(event_account);
    let err = send_ix_result(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_create_financing_offer(
            ctx.payer.pubkey(),
            organizer_authority.pubkey(),
            protocol_config,
            organizer_profile,
            event_account,
            financing_offer,
            ticketing_core::state::FinancingOfferInput {
                advance_amount_lamports: 1_000_000_000,
                fee_bps: 300,
                repayment_cap_lamports: 1_200_000_000,
                schedule_start_ts: 4_602_430_000,
                schedule_interval_secs: 86_400,
                schedule_installments: 4,
            },
        ),
        &[&organizer_authority],
    )
    .await;
    assert!(err.is_err());
}
