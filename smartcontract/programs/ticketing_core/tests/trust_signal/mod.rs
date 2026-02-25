use solana_sdk::signature::{Keypair, Signer};

use ticketing_core::instructions::{event::EventInput, ticket_class::TicketClassInput};

use crate::common::{
    event_pda, fetch_ticket, fetch_trust_signal, ix_buy_ticket, ix_check_in_ticket,
    ix_create_event, ix_create_organizer, ix_create_ticket_class, ix_flag_trust_abuse,
    ix_initialize_protocol, ix_record_attendance_input, ix_record_purchase_input,
    ix_set_trust_signal_schema_version, organizer_pda, protocol_config_pda, send_ix,
    send_ix_result, setup, ticket_class_pda, ticket_pda, trust_signal_pda,
    wallet_purchase_counter_pda,
};

struct Fixture {
    protocol_config: solana_sdk::pubkey::Pubkey,
    organizer_profile: solana_sdk::pubkey::Pubkey,
    event_account: solana_sdk::pubkey::Pubkey,
    ticket_class: solana_sdk::pubkey::Pubkey,
    ticket: solana_sdk::pubkey::Pubkey,
    trust_signal: solana_sdk::pubkey::Pubkey,
    organizer_authority: Keypair,
    buyer: Keypair,
}

async fn setup_fixture() -> (solana_program_test::ProgramTestContext, Fixture) {
    let mut ctx = setup().await;

    let organizer_authority = Keypair::new();
    let buyer = Keypair::new();
    let organizer_wallet = Keypair::new();
    let fee_vault = Keypair::new();

    for kp in [&organizer_authority, &buyer, &organizer_wallet, &fee_vault] {
        let ix = solana_sdk::system_instruction::transfer(
            &ctx.payer.pubkey(),
            &kp.pubkey(),
            5_000_000_000,
        );
        send_ix(
            &mut ctx.banks_client,
            &ctx.payer,
            ctx.last_blockhash,
            ix,
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
            "https://org/trust".to_string(),
            organizer_wallet.pubkey(),
        ),
        &[&organizer_authority],
    )
    .await;

    let event_id = 1_515;
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
                title: "Trust Event".to_string(),
                venue: "Venue".to_string(),
                start_ts: 4_602_444_800,
                end_ts: 4_602_448_400,
                sales_start_ts: 4_602_430_000,
                lock_ts: 4_602_440_000,
                capacity: 200,
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

    let ticket_id = 1;
    let (ticket, _) = ticket_pda(event_account, class_id, ticket_id);
    let (counter, _) = wallet_purchase_counter_pda(event_account, ticket_class, buyer.pubkey());
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
            counter,
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

    let (trust_signal, _) = trust_signal_pda(buyer.pubkey());

    (
        ctx,
        Fixture {
            protocol_config,
            organizer_profile,
            event_account,
            ticket_class,
            ticket,
            trust_signal,
            organizer_authority,
            buyer,
        },
    )
}

#[tokio::test]
async fn authorized_paths_record_purchase_and_attendance_once() {
    let (mut ctx, fixture) = setup_fixture().await;

    let purchase_ix = ix_record_purchase_input(
        ctx.payer.pubkey(),
        fixture.organizer_authority.pubkey(),
        fixture.buyer.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.ticket,
        fixture.trust_signal,
        1,
        1,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        purchase_ix,
        &[&fixture.organizer_authority],
    )
    .await;

    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_check_in_ticket(
            fixture.organizer_authority.pubkey(),
            fixture.protocol_config,
            fixture.organizer_profile,
            fixture.event_account,
            fixture.ticket_class,
            fixture.ticket,
            fixture.protocol_config,
            1,
            1,
            "A".to_string(),
        ),
        &[&fixture.organizer_authority],
    )
    .await;

    let attendance_ix = ix_record_attendance_input(
        ctx.payer.pubkey(),
        fixture.organizer_authority.pubkey(),
        fixture.buyer.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.ticket,
        fixture.trust_signal,
        1,
        1,
        true,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        attendance_ix,
        &[&fixture.organizer_authority],
    )
    .await;

    let signal = fetch_trust_signal(&mut ctx.banks_client, fixture.trust_signal).await;
    assert_eq!(signal.schema_version, 1);
    assert_eq!(signal.total_tickets_purchased, 1);
    assert_eq!(signal.attendance_eligible_count, 1);
    assert_eq!(signal.attendance_attended_count, 1);

    let ticket = fetch_ticket(&mut ctx.banks_client, fixture.ticket).await;
    assert!(ticket.purchase_trust_recorded);
    assert!(ticket.attendance_trust_recorded);

    let duplicate_purchase = ix_record_purchase_input(
        ctx.payer.pubkey(),
        fixture.organizer_authority.pubkey(),
        fixture.buyer.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.ticket,
        fixture.trust_signal,
        1,
        1,
    );
    ctx.last_blockhash = ctx.banks_client.get_latest_blockhash().await.unwrap();
    let err = send_ix_result(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        duplicate_purchase,
        &[&fixture.organizer_authority],
    )
    .await;
    assert!(err.is_err());
}

#[tokio::test]
async fn admin_can_flag_abuse_and_upgrade_schema_version_only_forward() {
    let (mut ctx, fixture) = setup_fixture().await;

    let purchase_ix = ix_record_purchase_input(
        ctx.payer.pubkey(),
        fixture.organizer_authority.pubkey(),
        fixture.buyer.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.ticket,
        fixture.trust_signal,
        1,
        1,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        purchase_ix,
        &[&fixture.organizer_authority],
    )
    .await;

    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_flag_trust_abuse(
            ctx.payer.pubkey(),
            fixture.buyer.pubkey(),
            fixture.protocol_config,
            fixture.trust_signal,
            0b0011,
            fixture.event_account,
            fixture.ticket,
        ),
        &[],
    )
    .await;

    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_flag_trust_abuse(
            ctx.payer.pubkey(),
            fixture.buyer.pubkey(),
            fixture.protocol_config,
            fixture.trust_signal,
            0b0001,
            fixture.event_account,
            fixture.ticket,
        ),
        &[],
    )
    .await;

    let signal = fetch_trust_signal(&mut ctx.banks_client, fixture.trust_signal).await;
    assert_eq!(signal.abuse_flags, 0b0011);
    assert_eq!(signal.abuse_incidents, 2);

    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_set_trust_signal_schema_version(
            ctx.payer.pubkey(),
            fixture.protocol_config,
            fixture.trust_signal,
            2,
        ),
        &[],
    )
    .await;

    let upgraded = fetch_trust_signal(&mut ctx.banks_client, fixture.trust_signal).await;
    assert_eq!(upgraded.schema_version, 2);

    let downgrade_ix = ix_set_trust_signal_schema_version(
        ctx.payer.pubkey(),
        fixture.protocol_config,
        fixture.trust_signal,
        1,
    );
    let err = send_ix_result(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        downgrade_ix,
        &[],
    )
    .await;
    assert!(err.is_err());
}

#[tokio::test]
async fn unauthorized_paths_cannot_update_trust_signal() {
    let (mut ctx, fixture) = setup_fixture().await;

    let fake_authority = Keypair::new();
    let fund_ix = solana_sdk::system_instruction::transfer(
        &ctx.payer.pubkey(),
        &fake_authority.pubkey(),
        1_000_000_000,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        fund_ix,
        &[],
    )
    .await;

    let unauthorized_purchase = ix_record_purchase_input(
        ctx.payer.pubkey(),
        fake_authority.pubkey(),
        fixture.buyer.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.ticket,
        fixture.trust_signal,
        1,
        1,
    );
    let err = send_ix_result(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        unauthorized_purchase,
        &[&fake_authority],
    )
    .await;
    assert!(err.is_err());

    let fake_admin = Keypair::new();
    let fund_ix = solana_sdk::system_instruction::transfer(
        &ctx.payer.pubkey(),
        &fake_admin.pubkey(),
        1_000_000_000,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        fund_ix,
        &[],
    )
    .await;

    let unauthorized_abuse = ix_flag_trust_abuse(
        fake_admin.pubkey(),
        fixture.buyer.pubkey(),
        fixture.protocol_config,
        fixture.trust_signal,
        1,
        fixture.event_account,
        fixture.ticket,
    );
    let err = send_ix_result(
        &mut ctx.banks_client,
        &fake_admin,
        ctx.last_blockhash,
        unauthorized_abuse,
        &[],
    )
    .await;
    assert!(err.is_err());
}
