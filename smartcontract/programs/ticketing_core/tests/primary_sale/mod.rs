use solana_sdk::signature::{Keypair, Signer};

use ticketing_core::instructions::{event::EventInput, ticket_class::TicketClassInput};

use crate::common::{
    event_pda, fetch_ticket, fetch_ticket_class, fetch_wallet_purchase_counter, get_lamports,
    ix_buy_ticket, ix_create_event, ix_create_organizer, ix_create_ticket_class,
    ix_initialize_protocol, ix_issue_comp_ticket, organizer_pda, protocol_config_pda, send_ix,
    send_ix_result, setup, ticket_class_pda, ticket_pda, wallet_purchase_counter_pda,
};

struct SaleFixture {
    protocol_config: solana_sdk::pubkey::Pubkey,
    organizer_profile: solana_sdk::pubkey::Pubkey,
    event_account: solana_sdk::pubkey::Pubkey,
    ticket_class: solana_sdk::pubkey::Pubkey,
    organizer_wallet: Keypair,
    fee_vault: Keypair,
    stakeholder_wallet: Keypair,
}

async fn setup_sale_fixture(
    ctx: &mut solana_program_test::ProgramTestContext,
    organizer_authority: &Keypair,
    protocol_fee_bps: u16,
    class: TicketClassInput,
) -> SaleFixture {
    let organizer_wallet = Keypair::new();
    let fee_vault = Keypair::new();
    let stakeholder_wallet = Keypair::new();

    for kp in [
        &organizer_authority,
        &organizer_wallet,
        &fee_vault,
        &stakeholder_wallet,
    ] {
        let ix = solana_sdk::system_instruction::transfer(
            &ctx.payer.pubkey(),
            &kp.pubkey(),
            2_000_000_000,
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
        protocol_fee_bps,
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
        "https://org/sale".to_string(),
        organizer_wallet.pubkey(),
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        create_org_ix,
        &[organizer_authority],
    )
    .await;

    let event_id = 99;
    let (event_account, _) = event_pda(organizer_profile, event_id);
    let create_event_ix = ix_create_event(
        ctx.payer.pubkey(),
        organizer_authority.pubkey(),
        protocol_config,
        organizer_profile,
        event_account,
        event_id,
        EventInput {
            title: "Primary Sale Event".to_string(),
            venue: "Main Hall".to_string(),
            start_ts: 4_102_444_800,
            end_ts: 4_102_448_400,
            sales_start_ts: 4_102_430_000,
            lock_ts: 4_102_440_000,
            capacity: 5000,
        },
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        create_event_ix,
        &[organizer_authority],
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
        class,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        create_class_ix,
        &[organizer_authority],
    )
    .await;

    SaleFixture {
        protocol_config,
        organizer_profile,
        event_account,
        ticket_class,
        organizer_wallet,
        fee_vault,
        stakeholder_wallet,
    }
}

#[tokio::test]
async fn buy_ticket_enforces_limits_and_records_ticket() {
    let mut ctx = setup().await;
    let organizer_authority = Keypair::new();
    let buyer = Keypair::new();
    let fund_buyer = solana_sdk::system_instruction::transfer(
        &ctx.payer.pubkey(),
        &buyer.pubkey(),
        5_000_000_000,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        fund_buyer,
        &[],
    )
    .await;

    let fixture = setup_sale_fixture(
        &mut ctx,
        &organizer_authority,
        500,
        TicketClassInput {
            name: "GA".to_string(),
            total_supply: 10,
            reserved_supply: 1,
            face_price_lamports: 1_000_000_000,
            sale_start_ts: 0,
            sale_end_ts: i64::MAX,
            per_wallet_limit: 1,
            is_transferable: true,
            is_resale_enabled: true,
            stakeholder_wallet: Keypair::new().pubkey(),
            stakeholder_bps: 0,
        },
    )
    .await;

    let class_id = 1;
    let ticket_id = 1;
    let (ticket, _) = ticket_pda(fixture.event_account, class_id, ticket_id);
    let (counter, _) =
        wallet_purchase_counter_pda(fixture.event_account, fixture.ticket_class, buyer.pubkey());

    let buy_ix = ix_buy_ticket(
        buyer.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        ticket,
        counter,
        fixture.fee_vault.pubkey(),
        fixture.organizer_wallet.pubkey(),
        fixture.stakeholder_wallet.pubkey(),
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

    let ticket_state = fetch_ticket(&mut ctx.banks_client, ticket).await;
    assert_eq!(ticket_state.owner, buyer.pubkey());
    assert_eq!(ticket_state.ticket_id, 1);
    assert!(!ticket_state.is_comp);

    let counter_state = fetch_wallet_purchase_counter(&mut ctx.banks_client, counter).await;
    assert_eq!(counter_state.purchased_count, 1);

    let class_state = fetch_ticket_class(&mut ctx.banks_client, fixture.ticket_class).await;
    assert_eq!(class_state.sold_supply, 1);
    assert_eq!(class_state.remaining_supply, 8);

    let second_ticket_id = 2;
    let (ticket2, _) = ticket_pda(fixture.event_account, class_id, second_ticket_id);
    let second_buy_ix = ix_buy_ticket(
        buyer.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        ticket2,
        counter,
        fixture.fee_vault.pubkey(),
        fixture.organizer_wallet.pubkey(),
        fixture.stakeholder_wallet.pubkey(),
        class_id,
        second_ticket_id,
        1_000_000_000,
    );
    let err = send_ix_result(
        &mut ctx.banks_client,
        &buyer,
        ctx.last_blockhash,
        second_buy_ix,
        &[],
    )
    .await;
    assert!(err.is_err());
}

#[tokio::test]
async fn buy_ticket_routes_protocol_and_stakeholder_splits() {
    let mut ctx = setup().await;
    let organizer_authority = Keypair::new();
    let buyer = Keypair::new();
    let fund_buyer = solana_sdk::system_instruction::transfer(
        &ctx.payer.pubkey(),
        &buyer.pubkey(),
        5_000_000_000,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        fund_buyer,
        &[],
    )
    .await;

    let stakeholder = Keypair::new();
    let fund_stakeholder = solana_sdk::system_instruction::transfer(
        &ctx.payer.pubkey(),
        &stakeholder.pubkey(),
        1_000_000,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        fund_stakeholder,
        &[],
    )
    .await;

    let fixture = setup_sale_fixture(
        &mut ctx,
        &organizer_authority,
        500,
        TicketClassInput {
            name: "VIP".to_string(),
            total_supply: 20,
            reserved_supply: 0,
            face_price_lamports: 1_000_000_000,
            sale_start_ts: 0,
            sale_end_ts: i64::MAX,
            per_wallet_limit: 2,
            is_transferable: true,
            is_resale_enabled: true,
            stakeholder_wallet: stakeholder.pubkey(),
            stakeholder_bps: 1000,
        },
    )
    .await;

    let fee_before = get_lamports(&mut ctx.banks_client, fixture.fee_vault.pubkey()).await;
    let org_before = get_lamports(&mut ctx.banks_client, fixture.organizer_wallet.pubkey()).await;
    let stakeholder_before = get_lamports(&mut ctx.banks_client, stakeholder.pubkey()).await;

    let class_id = 1;
    let ticket_id = 1;
    let (ticket, _) = ticket_pda(fixture.event_account, class_id, ticket_id);
    let (counter, _) =
        wallet_purchase_counter_pda(fixture.event_account, fixture.ticket_class, buyer.pubkey());

    let buy_ix = ix_buy_ticket(
        buyer.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        ticket,
        counter,
        fixture.fee_vault.pubkey(),
        fixture.organizer_wallet.pubkey(),
        stakeholder.pubkey(),
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

    let fee_after = get_lamports(&mut ctx.banks_client, fixture.fee_vault.pubkey()).await;
    let org_after = get_lamports(&mut ctx.banks_client, fixture.organizer_wallet.pubkey()).await;
    let stakeholder_after = get_lamports(&mut ctx.banks_client, stakeholder.pubkey()).await;

    assert_eq!(fee_after - fee_before, 50_000_000);
    assert_eq!(stakeholder_after - stakeholder_before, 100_000_000);
    assert_eq!(org_after - org_before, 850_000_000);
}

#[tokio::test]
async fn sale_window_and_price_validation_enforced() {
    let mut ctx = setup().await;
    let organizer_authority = Keypair::new();
    let buyer = Keypair::new();
    let fund_buyer = solana_sdk::system_instruction::transfer(
        &ctx.payer.pubkey(),
        &buyer.pubkey(),
        5_000_000_000,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        fund_buyer,
        &[],
    )
    .await;

    let fixture = setup_sale_fixture(
        &mut ctx,
        &organizer_authority,
        300,
        TicketClassInput {
            name: "EARLY".to_string(),
            total_supply: 5,
            reserved_supply: 0,
            face_price_lamports: 900_000_000,
            sale_start_ts: 4_202_430_000,
            sale_end_ts: 4_202_440_000,
            per_wallet_limit: 2,
            is_transferable: true,
            is_resale_enabled: true,
            stakeholder_wallet: Keypair::new().pubkey(),
            stakeholder_bps: 0,
        },
    )
    .await;

    let class_id = 1;
    let ticket_id = 1;
    let (ticket, _) = ticket_pda(fixture.event_account, class_id, ticket_id);
    let (counter, _) =
        wallet_purchase_counter_pda(fixture.event_account, fixture.ticket_class, buyer.pubkey());

    let bad_window_ix = ix_buy_ticket(
        buyer.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        ticket,
        counter,
        fixture.fee_vault.pubkey(),
        fixture.organizer_wallet.pubkey(),
        fixture.stakeholder_wallet.pubkey(),
        class_id,
        ticket_id,
        900_000_000,
    );
    let err = send_ix_result(
        &mut ctx.banks_client,
        &buyer,
        ctx.last_blockhash,
        bad_window_ix,
        &[],
    )
    .await;
    assert!(err.is_err());
}

#[tokio::test]
async fn comp_issuance_allowed_for_organizer_and_admin() {
    let mut ctx = setup().await;
    let organizer_authority = Keypair::new();
    let recipient = Keypair::new();
    let fund_recipient = solana_sdk::system_instruction::transfer(
        &ctx.payer.pubkey(),
        &recipient.pubkey(),
        1_000_000,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        fund_recipient,
        &[],
    )
    .await;

    let fixture = setup_sale_fixture(
        &mut ctx,
        &organizer_authority,
        250,
        TicketClassInput {
            name: "COMP".to_string(),
            total_supply: 5,
            reserved_supply: 0,
            face_price_lamports: 1_000_000_000,
            sale_start_ts: 4_102_430_000,
            sale_end_ts: 4_102_444_000,
            per_wallet_limit: 2,
            is_transferable: true,
            is_resale_enabled: true,
            stakeholder_wallet: Keypair::new().pubkey(),
            stakeholder_bps: 0,
        },
    )
    .await;

    let class_id = 1;

    let (ticket1, _) = ticket_pda(fixture.event_account, class_id, 1);
    let organizer_comp_ix = ix_issue_comp_ticket(
        ctx.payer.pubkey(),
        organizer_authority.pubkey(),
        recipient.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        ticket1,
        class_id,
        1,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        organizer_comp_ix,
        &[&organizer_authority],
    )
    .await;

    let t1 = fetch_ticket(&mut ctx.banks_client, ticket1).await;
    assert!(t1.is_comp);
    assert_eq!(t1.owner, recipient.pubkey());

    let (ticket2, _) = ticket_pda(fixture.event_account, class_id, 2);
    let admin_comp_ix = ix_issue_comp_ticket(
        ctx.payer.pubkey(),
        ctx.payer.pubkey(),
        recipient.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        ticket2,
        class_id,
        2,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        admin_comp_ix,
        &[],
    )
    .await;

    let t2 = fetch_ticket(&mut ctx.banks_client, ticket2).await;
    assert!(t2.is_comp);

    let unauthorized = Keypair::new();
    let fund_unauth = solana_sdk::system_instruction::transfer(
        &ctx.payer.pubkey(),
        &unauthorized.pubkey(),
        1_000_000_000,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        fund_unauth,
        &[],
    )
    .await;

    let (ticket3, _) = ticket_pda(fixture.event_account, class_id, 3);
    let bad_comp_ix = ix_issue_comp_ticket(
        ctx.payer.pubkey(),
        unauthorized.pubkey(),
        recipient.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        ticket3,
        class_id,
        3,
    );
    let err = send_ix_result(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        bad_comp_ix,
        &[&unauthorized],
    )
    .await;
    assert!(err.is_err());
}
