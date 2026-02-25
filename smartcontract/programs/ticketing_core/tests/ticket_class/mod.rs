use solana_sdk::signature::{Keypair, Signer};

use ticketing_core::instructions::{event::EventInput, ticket_class::TicketClassInput};

use crate::common::{
    event_pda, fetch_ticket_class, ix_create_event, ix_create_organizer, ix_create_ticket_class,
    ix_initialize_protocol, ix_reserve_inventory, ix_update_ticket_class, organizer_pda,
    protocol_config_pda, send_ix, send_ix_result, setup, ticket_class_pda,
};

async fn setup_event(
    ctx: &mut solana_program_test::ProgramTestContext,
    authority: &Keypair,
    event_id: u64,
) -> (
    solana_sdk::pubkey::Pubkey,
    solana_sdk::pubkey::Pubkey,
    solana_sdk::pubkey::Pubkey,
) {
    let fund_authority = solana_sdk::system_instruction::transfer(
        &ctx.payer.pubkey(),
        &authority.pubkey(),
        1_000_000_000,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        fund_authority,
        &[],
    )
    .await;

    let admin = ctx.payer.pubkey();
    let (protocol_config, _) = protocol_config_pda();
    let init_ix = ix_initialize_protocol(
        ctx.payer.pubkey(),
        admin,
        protocol_config,
        Keypair::new().pubkey(),
        Keypair::new().pubkey(),
        Keypair::new().pubkey(),
        250,
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

    let (organizer_profile, _) = organizer_pda(authority.pubkey());
    let create_org_ix = ix_create_organizer(
        ctx.payer.pubkey(),
        authority.pubkey(),
        protocol_config,
        organizer_profile,
        "https://org/ticket-class".to_string(),
        authority.pubkey(),
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        create_org_ix,
        &[authority],
    )
    .await;

    let (event_account, _) = event_pda(organizer_profile, event_id);
    let create_event_ix = ix_create_event(
        ctx.payer.pubkey(),
        authority.pubkey(),
        protocol_config,
        organizer_profile,
        event_account,
        event_id,
        EventInput {
            title: "TC Event".to_string(),
            venue: "Main".to_string(),
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
        &[authority],
    )
    .await;

    (protocol_config, organizer_profile, event_account)
}

#[tokio::test]
async fn create_ticket_class_sets_inventory_and_flags() {
    let mut ctx = setup().await;
    let authority = Keypair::new();
    let (protocol_config, organizer_profile, event_account) =
        setup_event(&mut ctx, &authority, 11).await;

    let class_id = 1;
    let (ticket_class, _) = ticket_class_pda(event_account, class_id);
    let input = TicketClassInput {
        name: "GA".to_string(),
        total_supply: 1000,
        reserved_supply: 100,
        face_price_lamports: 1_000_000_000,
        sale_start_ts: 4_102_430_000,
        sale_end_ts: 4_102_444_000,
        per_wallet_limit: 4,
        is_transferable: true,
        is_resale_enabled: true,
        stakeholder_wallet: solana_sdk::pubkey::Pubkey::default(),
        stakeholder_bps: 0,
    };

    let ix = ix_create_ticket_class(
        ctx.payer.pubkey(),
        authority.pubkey(),
        protocol_config,
        organizer_profile,
        event_account,
        ticket_class,
        class_id,
        input,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix,
        &[&authority],
    )
    .await;

    let state = fetch_ticket_class(&mut ctx.banks_client, ticket_class).await;
    assert_eq!(state.name, "GA");
    assert_eq!(state.total_supply, 1000);
    assert_eq!(state.reserved_supply, 100);
    assert_eq!(state.sold_supply, 0);
    assert_eq!(state.remaining_supply, 900);
    assert_eq!(state.per_wallet_limit, 4);
    assert!(state.is_transferable);
    assert!(state.is_resale_enabled);
}

#[tokio::test]
async fn update_ticket_class_updates_config_before_sales_start() {
    let mut ctx = setup().await;
    let authority = Keypair::new();
    let (protocol_config, organizer_profile, event_account) =
        setup_event(&mut ctx, &authority, 12).await;

    let class_id = 2;
    let (ticket_class, _) = ticket_class_pda(event_account, class_id);
    let create_ix = ix_create_ticket_class(
        ctx.payer.pubkey(),
        authority.pubkey(),
        protocol_config,
        organizer_profile,
        event_account,
        ticket_class,
        class_id,
        TicketClassInput {
            name: "VIP".to_string(),
            total_supply: 200,
            reserved_supply: 20,
            face_price_lamports: 2_000_000_000,
            sale_start_ts: 4_102_430_000,
            sale_end_ts: 4_102_444_000,
            per_wallet_limit: 2,
            is_transferable: true,
            is_resale_enabled: false,
            stakeholder_wallet: solana_sdk::pubkey::Pubkey::default(),
            stakeholder_bps: 0,
        },
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        create_ix,
        &[&authority],
    )
    .await;

    let update_ix = ix_update_ticket_class(
        authority.pubkey(),
        organizer_profile,
        event_account,
        ticket_class,
        class_id,
        TicketClassInput {
            name: "VIP-PLUS".to_string(),
            total_supply: 250,
            reserved_supply: 50,
            face_price_lamports: 2_500_000_000,
            sale_start_ts: 4_102_430_500,
            sale_end_ts: 4_102_444_500,
            per_wallet_limit: 3,
            is_transferable: false,
            is_resale_enabled: false,
            stakeholder_wallet: solana_sdk::pubkey::Pubkey::default(),
            stakeholder_bps: 0,
        },
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        update_ix,
        &[&authority],
    )
    .await;

    let state = fetch_ticket_class(&mut ctx.banks_client, ticket_class).await;
    assert_eq!(state.name, "VIP-PLUS");
    assert_eq!(state.total_supply, 250);
    assert_eq!(state.reserved_supply, 50);
    assert_eq!(state.remaining_supply, 200);
    assert_eq!(state.per_wallet_limit, 3);
    assert!(!state.is_transferable);
}

#[tokio::test]
async fn reserve_inventory_updates_remaining_and_prevents_overreserve() {
    let mut ctx = setup().await;
    let authority = Keypair::new();
    let (protocol_config, organizer_profile, event_account) =
        setup_event(&mut ctx, &authority, 13).await;

    let class_id = 3;
    let (ticket_class, _) = ticket_class_pda(event_account, class_id);
    let create_ix = ix_create_ticket_class(
        ctx.payer.pubkey(),
        authority.pubkey(),
        protocol_config,
        organizer_profile,
        event_account,
        ticket_class,
        class_id,
        TicketClassInput {
            name: "TEAM".to_string(),
            total_supply: 100,
            reserved_supply: 10,
            face_price_lamports: 500_000_000,
            sale_start_ts: 4_102_430_000,
            sale_end_ts: 4_102_444_000,
            per_wallet_limit: 5,
            is_transferable: true,
            is_resale_enabled: true,
            stakeholder_wallet: solana_sdk::pubkey::Pubkey::default(),
            stakeholder_bps: 0,
        },
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        create_ix,
        &[&authority],
    )
    .await;

    let reserve_ix = ix_reserve_inventory(
        authority.pubkey(),
        organizer_profile,
        event_account,
        ticket_class,
        class_id,
        15,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        reserve_ix,
        &[&authority],
    )
    .await;

    let state = fetch_ticket_class(&mut ctx.banks_client, ticket_class).await;
    assert_eq!(state.reserved_supply, 25);
    assert_eq!(state.remaining_supply, 75);

    let over_ix = ix_reserve_inventory(
        authority.pubkey(),
        organizer_profile,
        event_account,
        ticket_class,
        class_id,
        80,
    );
    let err = send_ix_result(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        over_ix,
        &[&authority],
    )
    .await;
    assert!(err.is_err());
}

#[tokio::test]
async fn ticket_class_constraints_enforced() {
    let mut ctx = setup().await;
    let authority = Keypair::new();
    let other = Keypair::new();
    let fund_other = solana_sdk::system_instruction::transfer(
        &ctx.payer.pubkey(),
        &other.pubkey(),
        1_000_000_000,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        fund_other,
        &[],
    )
    .await;

    let (protocol_config, organizer_profile, event_account) =
        setup_event(&mut ctx, &authority, 14).await;

    let class_id = 4;
    let (ticket_class, _) = ticket_class_pda(event_account, class_id);

    let bad_create_ix = ix_create_ticket_class(
        ctx.payer.pubkey(),
        authority.pubkey(),
        protocol_config,
        organizer_profile,
        event_account,
        ticket_class,
        class_id,
        TicketClassInput {
            name: "BAD".to_string(),
            total_supply: 10,
            reserved_supply: 11,
            face_price_lamports: 1,
            sale_start_ts: 10,
            sale_end_ts: 20,
            per_wallet_limit: 1,
            is_transferable: true,
            is_resale_enabled: true,
            stakeholder_wallet: solana_sdk::pubkey::Pubkey::default(),
            stakeholder_bps: 0,
        },
    );
    let err = send_ix_result(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        bad_create_ix,
        &[&authority],
    )
    .await;
    assert!(err.is_err());

    let create_ix = ix_create_ticket_class(
        ctx.payer.pubkey(),
        authority.pubkey(),
        protocol_config,
        organizer_profile,
        event_account,
        ticket_class,
        class_id,
        TicketClassInput {
            name: "OK".to_string(),
            total_supply: 10,
            reserved_supply: 1,
            face_price_lamports: 1,
            sale_start_ts: 4_102_430_000,
            sale_end_ts: 4_102_440_000,
            per_wallet_limit: 1,
            is_transferable: true,
            is_resale_enabled: true,
            stakeholder_wallet: solana_sdk::pubkey::Pubkey::default(),
            stakeholder_bps: 0,
        },
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        create_ix,
        &[&authority],
    )
    .await;

    let unauthorized_reserve_ix = ix_reserve_inventory(
        other.pubkey(),
        organizer_profile,
        event_account,
        ticket_class,
        class_id,
        1,
    );
    let err = send_ix_result(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        unauthorized_reserve_ix,
        &[&other],
    )
    .await;
    assert!(err.is_err());
}
