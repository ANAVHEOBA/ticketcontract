use solana_sdk::signature::{Keypair, Signer};

use ticketing_core::{
    instructions::{event::EventInput, ticket_class::TicketClassInput},
    state::TicketStatus,
};

use crate::common::{
    event_pda, fetch_ticket, ix_buy_ticket, ix_check_in_ticket, ix_create_event,
    ix_create_organizer, ix_create_ticket_class, ix_initialize_protocol, ix_set_check_in_policy,
    ix_set_organizer_operator, organizer_operator_pda, organizer_pda, protocol_config_pda, send_ix,
    send_ix_result, setup, ticket_class_pda, ticket_pda, wallet_purchase_counter_pda,
};

struct Fixture {
    protocol_config: solana_sdk::pubkey::Pubkey,
    organizer_profile: solana_sdk::pubkey::Pubkey,
    event_account: solana_sdk::pubkey::Pubkey,
    ticket_class: solana_sdk::pubkey::Pubkey,
    ticket: solana_sdk::pubkey::Pubkey,
    organizer_authority: Keypair,
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
            "https://org/checkin".to_string(),
            organizer_wallet.pubkey(),
        ),
        &[&organizer_authority],
    )
    .await;

    let event_id = 808;
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
                title: "CheckIn Event".to_string(),
                venue: "Venue".to_string(),
                start_ts: 4_202_444_800,
                end_ts: 4_202_448_400,
                sales_start_ts: 4_202_430_000,
                lock_ts: 4_202_440_000,
                capacity: 500,
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

    (
        ctx,
        Fixture {
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
async fn authority_can_check_in_and_records_gate_timestamp() {
    let (mut ctx, fixture) = setup_fixture().await;
    let ix = ix_check_in_ticket(
        fixture.organizer_authority.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.ticket,
        fixture.protocol_config,
        1,
        1,
        "GATE-A".to_string(),
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix,
        &[&fixture.organizer_authority],
    )
    .await;

    let ticket = fetch_ticket(&mut ctx.banks_client, fixture.ticket).await;
    assert_eq!(ticket.status, TicketStatus::CheckedIn);
    assert!(ticket.checked_in_at > 0);
    assert!(ticket.last_check_in_at > 0);
    assert_eq!(ticket.last_check_in_gate_id, "GATE-A");
    assert_eq!(ticket.check_in_count, 1);
}

#[tokio::test]
async fn single_use_entry_is_enforced_by_default() {
    let (mut ctx, fixture) = setup_fixture().await;
    let first = ix_check_in_ticket(
        fixture.organizer_authority.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.ticket,
        fixture.protocol_config,
        1,
        1,
        "GATE-A".to_string(),
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        first,
        &[&fixture.organizer_authority],
    )
    .await;

    let second = ix_check_in_ticket(
        fixture.organizer_authority.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.ticket,
        fixture.protocol_config,
        1,
        1,
        "GATE-B".to_string(),
    );
    let err = send_ix_result(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        second,
        &[&fixture.organizer_authority],
    )
    .await;
    assert!(err.is_err());
}

#[tokio::test]
async fn operator_with_permission_can_check_in() {
    let (mut ctx, fixture) = setup_fixture().await;
    let operator = Keypair::new();
    let fund_ix = solana_sdk::system_instruction::transfer(
        &ctx.payer.pubkey(),
        &operator.pubkey(),
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

    let (organizer_operator, _) =
        organizer_operator_pda(fixture.organizer_profile, operator.pubkey());
    let set_operator = ix_set_organizer_operator(
        ctx.payer.pubkey(),
        fixture.organizer_authority.pubkey(),
        operator.pubkey(),
        fixture.organizer_profile,
        organizer_operator,
        ticketing_core::constants::OPERATOR_PERMISSION_CHECKIN,
        true,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        set_operator,
        &[&fixture.organizer_authority],
    )
    .await;

    let checkin = ix_check_in_ticket(
        operator.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.ticket,
        organizer_operator,
        1,
        1,
        "GATE-OP".to_string(),
    );
    send_ix(
        &mut ctx.banks_client,
        &operator,
        ctx.last_blockhash,
        checkin,
        &[],
    )
    .await;

    let ticket = fetch_ticket(&mut ctx.banks_client, fixture.ticket).await;
    assert_eq!(ticket.check_in_count, 1);
    assert_eq!(ticket.last_check_in_gate_id, "GATE-OP");
}

#[tokio::test]
async fn reentry_policy_allows_limited_reentry() {
    let (mut ctx, fixture) = setup_fixture().await;

    let set_policy = ix_set_check_in_policy(
        fixture.organizer_authority.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        1,
        true,
        1,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        set_policy,
        &[&fixture.organizer_authority],
    )
    .await;

    for gate in ["GATE-A", "GATE-B"] {
        let ix = ix_check_in_ticket(
            fixture.organizer_authority.pubkey(),
            fixture.protocol_config,
            fixture.organizer_profile,
            fixture.event_account,
            fixture.ticket_class,
            fixture.ticket,
            fixture.protocol_config,
            1,
            1,
            gate.to_string(),
        );
        send_ix(
            &mut ctx.banks_client,
            &ctx.payer,
            ctx.last_blockhash,
            ix,
            &[&fixture.organizer_authority],
        )
        .await;
    }

    let third = ix_check_in_ticket(
        fixture.organizer_authority.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.ticket,
        fixture.protocol_config,
        1,
        1,
        "GATE-C".to_string(),
    );
    let err = send_ix_result(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        third,
        &[&fixture.organizer_authority],
    )
    .await;
    assert!(err.is_err());

    let ticket = fetch_ticket(&mut ctx.banks_client, fixture.ticket).await;
    assert_eq!(ticket.check_in_count, 2);
    assert_eq!(ticket.last_check_in_gate_id, "GATE-B");
}
