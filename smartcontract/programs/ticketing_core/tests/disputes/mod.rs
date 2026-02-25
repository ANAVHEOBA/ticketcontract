use solana_sdk::signature::{Keypair, Signer};

use ticketing_core::{
    instructions::{event::EventInput, ticket_class::TicketClassInput},
    state::{ResalePolicyInput, TicketStatus},
};

use crate::common::{
    event_pda, fetch_ticket, get_lamports, ix_buy_resale_ticket, ix_buy_ticket, ix_create_event,
    ix_create_organizer, ix_create_ticket_class, ix_flag_dispute, ix_initialize_protocol,
    ix_list_ticket, ix_refund_ticket, ix_set_resale_policy, listing_pda, organizer_pda,
    protocol_config_pda, resale_policy_pda, send_ix, send_ix_result, setup, ticket_class_pda,
    ticket_pda, wallet_purchase_counter_pda,
};

struct Fixture {
    protocol_config: solana_sdk::pubkey::Pubkey,
    organizer_profile: solana_sdk::pubkey::Pubkey,
    event_account: solana_sdk::pubkey::Pubkey,
    ticket_class: solana_sdk::pubkey::Pubkey,
    ticket: solana_sdk::pubkey::Pubkey,
    listing: solana_sdk::pubkey::Pubkey,
    resale_policy: solana_sdk::pubkey::Pubkey,
    organizer_authority: Keypair,
    owner: Keypair,
    organizer_wallet: Keypair,
    treasury_vault: Keypair,
    fee_vault: Keypair,
    royalty_vault: Keypair,
}

async fn setup_fixture() -> (solana_program_test::ProgramTestContext, Fixture) {
    let mut ctx = setup().await;
    let organizer_authority = Keypair::new();
    let owner = Keypair::new();
    let organizer_wallet = Keypair::new();
    let treasury_vault = Keypair::new();
    let fee_vault = Keypair::new();
    let royalty_vault = Keypair::new();

    for kp in [
        &organizer_authority,
        &owner,
        &organizer_wallet,
        &treasury_vault,
        &fee_vault,
        &royalty_vault,
    ] {
        let ix = solana_sdk::system_instruction::transfer(
            &ctx.payer.pubkey(),
            &kp.pubkey(),
            6_000_000_000,
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
        treasury_vault.pubkey(),
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
        "https://org/disputes".to_string(),
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

    let event_id = 1_313;
    let (event_account, _) = event_pda(organizer_profile, event_id);
    let create_event_ix = ix_create_event(
        ctx.payer.pubkey(),
        organizer_authority.pubkey(),
        protocol_config,
        organizer_profile,
        event_account,
        event_id,
        EventInput {
            title: "Disputes Event".to_string(),
            venue: "Hall".to_string(),
            start_ts: 4_302_444_800,
            end_ts: 4_302_448_400,
            sales_start_ts: 4_302_430_000,
            lock_ts: 4_302_440_000,
            capacity: 500,
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
            total_supply: 20,
            reserved_supply: 0,
            face_price_lamports: 1_000_000_000,
            sale_start_ts: 0,
            sale_end_ts: i64::MAX,
            per_wallet_limit: 5,
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
    let (counter, _) = wallet_purchase_counter_pda(event_account, ticket_class, owner.pubkey());
    let buy_ix = ix_buy_ticket(
        owner.pubkey(),
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
        &owner,
        ctx.last_blockhash,
        buy_ix,
        &[],
    )
    .await;

    let (resale_policy, _) = resale_policy_pda(event_account, class_id);
    let set_policy_ix = ix_set_resale_policy(
        ctx.payer.pubkey(),
        organizer_authority.pubkey(),
        protocol_config,
        organizer_profile,
        event_account,
        ticket_class,
        resale_policy,
        class_id,
        ResalePolicyInput {
            max_markup_bps: 2_000,
            royalty_bps: 1_000,
            royalty_vault: royalty_vault.pubkey(),
            transfer_cooldown_secs: 0,
            max_transfer_count: 10,
            transfer_lock_before_event_secs: 0,
            whitelist: vec![],
            blacklist: vec![],
        },
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        set_policy_ix,
        &[&organizer_authority],
    )
    .await;

    let (listing, _) = listing_pda(ticket);

    (
        ctx,
        Fixture {
            protocol_config,
            organizer_profile,
            event_account,
            ticket_class,
            ticket,
            listing,
            resale_policy,
            organizer_authority,
            owner,
            organizer_wallet,
            treasury_vault,
            fee_vault,
            royalty_vault,
        },
    )
}

#[tokio::test]
async fn organizer_can_refund_and_ticket_state_updates() {
    let (mut ctx, fixture) = setup_fixture().await;

    let owner_before = get_lamports(&mut ctx.banks_client, fixture.owner.pubkey()).await;
    let organizer_before =
        get_lamports(&mut ctx.banks_client, fixture.organizer_wallet.pubkey()).await;

    let refund_ix = ix_refund_ticket(
        fixture.organizer_authority.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.ticket,
        fixture.owner.pubkey(),
        fixture.organizer_wallet.pubkey(),
        fixture.treasury_vault.pubkey(),
        fixture.fee_vault.pubkey(),
        1,
        1,
        500_000_000,
        1,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        refund_ix,
        &[
            &fixture.organizer_authority,
            &fixture.organizer_wallet,
            &fixture.treasury_vault,
            &fixture.fee_vault,
        ],
    )
    .await;

    let ticket = fetch_ticket(&mut ctx.banks_client, fixture.ticket).await;
    assert_eq!(ticket.status, TicketStatus::Refunded);
    assert_eq!(ticket.owner, ticket.buyer);
    assert_eq!(ticket.refund_amount_lamports, 500_000_000);
    assert_eq!(ticket.refund_source, 1);
    assert!(ticket.refunded_at > 0);

    let owner_after = get_lamports(&mut ctx.banks_client, fixture.owner.pubkey()).await;
    let organizer_after =
        get_lamports(&mut ctx.banks_client, fixture.organizer_wallet.pubkey()).await;
    assert!(owner_after > owner_before);
    assert!(organizer_after < organizer_before);
}

#[tokio::test]
async fn disputed_ticket_cannot_be_listed_for_resale() {
    let (mut ctx, fixture) = setup_fixture().await;

    let flag_ix = ix_flag_dispute(
        fixture.organizer_authority.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.ticket,
        1,
        1,
        true,
        false,
        7,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        flag_ix,
        &[&fixture.organizer_authority],
    )
    .await;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let list_ix = ix_list_ticket(
        fixture.owner.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.resale_policy,
        fixture.ticket,
        fixture.listing,
        1,
        1,
        1_100_000_000,
        now + 3_600,
    );
    let err = send_ix_result(
        &mut ctx.banks_client,
        &fixture.owner,
        ctx.last_blockhash,
        list_ix,
        &[],
    )
    .await;
    assert!(err.is_err());
}

#[tokio::test]
async fn chargeback_flag_blocks_resale_execution() {
    let (mut ctx, fixture) = setup_fixture().await;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let list_ix = ix_list_ticket(
        fixture.owner.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.resale_policy,
        fixture.ticket,
        fixture.listing,
        1,
        1,
        1_050_000_000,
        now + 3_600,
    );
    send_ix(
        &mut ctx.banks_client,
        &fixture.owner,
        ctx.last_blockhash,
        list_ix,
        &[],
    )
    .await;

    let flag_ix = ix_flag_dispute(
        fixture.organizer_authority.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.ticket,
        1,
        1,
        true,
        true,
        19,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        flag_ix,
        &[&fixture.organizer_authority],
    )
    .await;

    let buyer = Keypair::new();
    let fund_ix = solana_sdk::system_instruction::transfer(
        &ctx.payer.pubkey(),
        &buyer.pubkey(),
        4_000_000_000,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        fund_ix,
        &[],
    )
    .await;

    let buy_ix = ix_buy_resale_ticket(
        buyer.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.resale_policy,
        fixture.ticket,
        fixture.listing,
        fixture.owner.pubkey(),
        fixture.royalty_vault.pubkey(),
        1,
        1,
        1_050_000_000,
    );
    let err = send_ix_result(
        &mut ctx.banks_client,
        &buyer,
        ctx.last_blockhash,
        buy_ix,
        &[],
    )
    .await;
    assert!(err.is_err());

    let ticket = fetch_ticket(&mut ctx.banks_client, fixture.ticket).await;
    assert!(ticket.is_chargeback);
    assert_eq!(ticket.status, TicketStatus::Invalidated);
}

#[tokio::test]
async fn invalid_refund_source_is_rejected() {
    let (mut ctx, fixture) = setup_fixture().await;

    let refund_ix = ix_refund_ticket(
        fixture.organizer_authority.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.ticket,
        fixture.owner.pubkey(),
        fixture.organizer_wallet.pubkey(),
        fixture.treasury_vault.pubkey(),
        fixture.fee_vault.pubkey(),
        1,
        1,
        250_000_000,
        4,
    );
    let err = send_ix_result(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        refund_ix,
        &[
            &fixture.organizer_authority,
            &fixture.organizer_wallet,
            &fixture.treasury_vault,
            &fixture.fee_vault,
        ],
    )
    .await;
    assert!(err.is_err());
}
