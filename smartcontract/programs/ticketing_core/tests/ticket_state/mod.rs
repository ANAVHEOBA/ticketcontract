use solana_sdk::signature::{Keypair, Signer};

use ticketing_core::{
    instructions::{event::EventInput, ticket_class::TicketClassInput},
    state::TicketStatus,
};

use crate::common::{
    event_pda, fetch_ticket, ix_buy_ticket, ix_create_event, ix_create_organizer,
    ix_create_ticket_class, ix_initialize_protocol, ix_set_ticket_metadata,
    ix_transition_ticket_status, organizer_pda, protocol_config_pda, send_ix, send_ix_result,
    setup, ticket_class_pda, ticket_pda, wallet_purchase_counter_pda,
};

struct TicketFixture {
    protocol_config: solana_sdk::pubkey::Pubkey,
    organizer_profile: solana_sdk::pubkey::Pubkey,
    event_account: solana_sdk::pubkey::Pubkey,
    ticket_class: solana_sdk::pubkey::Pubkey,
    ticket: solana_sdk::pubkey::Pubkey,
    organizer_authority: Keypair,
}

async fn setup_ticket_fixture() -> (solana_program_test::ProgramTestContext, TicketFixture) {
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
    let init_ix = ix_initialize_protocol(
        ctx.payer.pubkey(),
        ctx.payer.pubkey(),
        protocol_config,
        Keypair::new().pubkey(),
        Keypair::new().pubkey(),
        fee_vault.pubkey(),
        500,
        8,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        init_ix,
        &[],
    )
    .await;

    let (organizer_profile, _) = organizer_pda(organizer_authority.pubkey());
    let create_org_ix = ix_create_organizer(
        ctx.payer.pubkey(),
        organizer_authority.pubkey(),
        protocol_config,
        organizer_profile,
        "https://org/ticket-state".to_string(),
        organizer_wallet.pubkey(),
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        create_org_ix,
        &[&organizer_authority],
    )
    .await;

    let event_id = 707;
    let (event_account, _) = event_pda(organizer_profile, event_id);
    let create_event_ix = ix_create_event(
        ctx.payer.pubkey(),
        organizer_authority.pubkey(),
        protocol_config,
        organizer_profile,
        event_account,
        event_id,
        EventInput {
            title: "Ticket State Event".to_string(),
            venue: "Venue".to_string(),
            start_ts: 4_102_444_800,
            end_ts: 4_102_448_400,
            sales_start_ts: 4_102_430_000,
            lock_ts: 4_102_440_000,
            capacity: 1000,
        },
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        create_event_ix,
        &[&organizer_authority],
    )
    .await;

    let class_id = 1;
    let (ticket_class, _) = ticket_class_pda(event_account, class_id);
    let create_class_ix = ix_create_ticket_class(
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
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        create_class_ix,
        &[&organizer_authority],
    )
    .await;

    let ticket_id = 1;
    let (ticket, _) = ticket_pda(event_account, class_id, ticket_id);
    let (counter, _) = wallet_purchase_counter_pda(event_account, ticket_class, buyer.pubkey());
    let buy_ix = ix_buy_ticket(
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
    );
    send_ix(
        &mut ctx.banks_client,
        &buyer,
        ctx.last_blockhash,
        buy_ix,
        &[],
    )
    .await;

    (
        ctx,
        TicketFixture {
            protocol_config,
            organizer_profile,
            event_account,
            ticket_class,
            ticket,
            organizer_authority,
        },
    )
}

#[tokio::test]
async fn ticket_defaults_are_initialized_on_mint() {
    let (mut ctx, fixture) = setup_ticket_fixture().await;
    let ticket = fetch_ticket(&mut ctx.banks_client, fixture.ticket).await;

    assert_eq!(ticket.status, TicketStatus::Active);
    assert!(ticket.created_at > 0);
    assert_eq!(ticket.status_updated_at, ticket.created_at);
    assert_eq!(ticket.checked_in_at, 0);
    assert_eq!(ticket.refunded_at, 0);
    assert_eq!(ticket.invalidated_at, 0);
    assert_eq!(ticket.metadata_uri, "");
    assert_eq!(ticket.metadata_version, 0);
    assert_eq!(ticket.metadata_updated_at, 0);
}

#[tokio::test]
async fn valid_transition_updates_lifecycle_timestamp() {
    let (mut ctx, fixture) = setup_ticket_fixture().await;
    let class_id = 1;
    let ticket_id = 1;

    let transition_ix = ix_transition_ticket_status(
        fixture.organizer_authority.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.ticket,
        class_id,
        ticket_id,
        TicketStatus::CheckedIn as u8,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        transition_ix,
        &[&fixture.organizer_authority],
    )
    .await;

    let ticket = fetch_ticket(&mut ctx.banks_client, fixture.ticket).await;
    assert_eq!(ticket.status, TicketStatus::CheckedIn);
    assert!(ticket.status_updated_at > 0);
    assert!(ticket.checked_in_at > 0);
}

#[tokio::test]
async fn illegal_and_unknown_status_transitions_fail() {
    let (mut ctx, fixture) = setup_ticket_fixture().await;
    let class_id = 1;
    let ticket_id = 1;

    let to_checked_in = ix_transition_ticket_status(
        fixture.organizer_authority.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.ticket,
        class_id,
        ticket_id,
        TicketStatus::CheckedIn as u8,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        to_checked_in,
        &[&fixture.organizer_authority],
    )
    .await;

    let illegal_transition = ix_transition_ticket_status(
        fixture.organizer_authority.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.ticket,
        class_id,
        ticket_id,
        TicketStatus::Refunded as u8,
    );
    let illegal_err = send_ix_result(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        illegal_transition,
        &[&fixture.organizer_authority],
    )
    .await;
    assert!(illegal_err.is_err());

    let invalid_status = ix_transition_ticket_status(
        fixture.organizer_authority.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.ticket,
        class_id,
        ticket_id,
        99,
    );
    let invalid_err = send_ix_result(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        invalid_status,
        &[&fixture.organizer_authority],
    )
    .await;
    assert!(invalid_err.is_err());
}

#[tokio::test]
async fn metadata_pointer_and_version_can_be_updated_with_auth() {
    let (mut ctx, fixture) = setup_ticket_fixture().await;
    let class_id = 1;
    let ticket_id = 1;

    let set_metadata = ix_set_ticket_metadata(
        fixture.organizer_authority.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.ticket,
        class_id,
        ticket_id,
        "https://kyd.example/ticket/1".to_string(),
        2,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        set_metadata,
        &[&fixture.organizer_authority],
    )
    .await;

    let ticket = fetch_ticket(&mut ctx.banks_client, fixture.ticket).await;
    assert_eq!(ticket.metadata_uri, "https://kyd.example/ticket/1");
    assert_eq!(ticket.metadata_version, 2);
    assert!(ticket.metadata_updated_at > 0);

    let random_authority = Keypair::new();
    let unauthorized_set = ix_set_ticket_metadata(
        random_authority.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.ticket,
        class_id,
        ticket_id,
        "https://evil.example".to_string(),
        3,
    );
    let unauthorized_err = send_ix_result(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        unauthorized_set,
        &[&random_authority],
    )
    .await;
    assert!(unauthorized_err.is_err());

    let oversized_set = ix_set_ticket_metadata(
        ctx.payer.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.ticket,
        class_id,
        ticket_id,
        "a".repeat(300),
        4,
    );
    let oversized_err = send_ix_result(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        oversized_set,
        &[],
    )
    .await;
    assert!(oversized_err.is_err());
}
