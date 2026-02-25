use solana_sdk::signature::{Keypair, Signer};

use ticketing_core::{
    instructions::{event::EventInput, ticket_class::TicketClassInput},
    state::ResalePolicyInput,
};

use crate::common::{
    event_pda, fetch_listing, fetch_ticket, get_lamports, ix_buy_resale_ticket, ix_buy_ticket,
    ix_create_event, ix_create_organizer, ix_create_ticket_class, ix_expire_listing,
    ix_initialize_protocol, ix_list_ticket, ix_set_resale_policy, listing_pda, organizer_pda,
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
    seller: Keypair,
    royalty_vault: Keypair,
}

async fn setup_fixture(
    transfer_lock_before_event_secs: i64,
) -> (solana_program_test::ProgramTestContext, Fixture) {
    let mut ctx = setup().await;
    let organizer_authority = Keypair::new();
    let seller = Keypair::new();
    let organizer_wallet = Keypair::new();
    let fee_vault = Keypair::new();
    let royalty_vault = Keypair::new();

    for kp in [
        &organizer_authority,
        &seller,
        &organizer_wallet,
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
        "https://org/sec-sale".to_string(),
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

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let event_id = 909;
    let (event_account, _) = event_pda(organizer_profile, event_id);
    let create_event_ix = ix_create_event(
        ctx.payer.pubkey(),
        organizer_authority.pubkey(),
        protocol_config,
        organizer_profile,
        event_account,
        event_id,
        EventInput {
            title: "Secondary Sale Event".to_string(),
            venue: "Arena".to_string(),
            start_ts: now + 7_200,
            end_ts: now + 10_800,
            sales_start_ts: now - 3_600,
            lock_ts: now + 3_600,
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
    let (counter, _) = wallet_purchase_counter_pda(event_account, ticket_class, seller.pubkey());
    let buy_primary_ix = ix_buy_ticket(
        seller.pubkey(),
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
        &seller,
        ctx.last_blockhash,
        buy_primary_ix,
        &[],
    )
    .await;

    let (resale_policy, _) = resale_policy_pda(event_account, class_id);
    let policy_ix = ix_set_resale_policy(
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
            transfer_lock_before_event_secs,
            whitelist: vec![],
            blacklist: vec![],
        },
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        policy_ix,
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
            seller,
            royalty_vault,
        },
    )
}

#[tokio::test]
async fn list_with_expiry_and_execute_resale_atomically() {
    let (mut ctx, fixture) = setup_fixture(0).await;
    let class_id = 1;
    let ticket_id = 1;
    let buyer = Keypair::new();
    let fund_ix = solana_sdk::system_instruction::transfer(
        &ctx.payer.pubkey(),
        &buyer.pubkey(),
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

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let list_ix = ix_list_ticket(
        fixture.seller.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.resale_policy,
        fixture.ticket,
        fixture.listing,
        class_id,
        ticket_id,
        1_100_000_000,
        now + 3_600,
    );
    send_ix(
        &mut ctx.banks_client,
        &fixture.seller,
        ctx.last_blockhash,
        list_ix,
        &[],
    )
    .await;

    let listing_before = fetch_listing(&mut ctx.banks_client, fixture.listing).await;
    assert!(listing_before.is_active);
    assert_eq!(listing_before.price_lamports, 1_100_000_000);

    let seller_before = get_lamports(&mut ctx.banks_client, fixture.seller.pubkey()).await;
    let royalty_before = get_lamports(&mut ctx.banks_client, fixture.royalty_vault.pubkey()).await;

    let buy_ix = ix_buy_resale_ticket(
        buyer.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.resale_policy,
        fixture.ticket,
        fixture.listing,
        fixture.seller.pubkey(),
        fixture.royalty_vault.pubkey(),
        class_id,
        ticket_id,
        1_100_000_000,
    );
    send_ix(
        &mut ctx.banks_client,
        &buyer,
        ctx.last_blockhash,
        buy_ix,
        &[],
    )
    .await;

    let ticket = fetch_ticket(&mut ctx.banks_client, fixture.ticket).await;
    let listing_after = fetch_listing(&mut ctx.banks_client, fixture.listing).await;
    let seller_after = get_lamports(&mut ctx.banks_client, fixture.seller.pubkey()).await;
    let royalty_after = get_lamports(&mut ctx.banks_client, fixture.royalty_vault.pubkey()).await;

    assert_eq!(ticket.owner, buyer.pubkey());
    assert_eq!(ticket.transfer_count, 1);
    assert!(!listing_after.is_active);
    assert_eq!(listing_after.close_reason, 2);
    assert_eq!(royalty_after - royalty_before, 110_000_000);
    assert_eq!(seller_after - seller_before, 990_000_000);
}

#[tokio::test]
async fn expired_listing_rejects_buy_and_can_be_expired() {
    let (mut ctx, fixture) = setup_fixture(0).await;
    let class_id = 1;
    let ticket_id = 1;
    let buyer = Keypair::new();
    let fund_ix = solana_sdk::system_instruction::transfer(
        &ctx.payer.pubkey(),
        &buyer.pubkey(),
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

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let list_ix = ix_list_ticket(
        fixture.seller.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.resale_policy,
        fixture.ticket,
        fixture.listing,
        class_id,
        ticket_id,
        1_050_000_000,
        now - 10,
    );
    let list_err = send_ix_result(
        &mut ctx.banks_client,
        &fixture.seller,
        ctx.last_blockhash,
        list_ix,
        &[],
    )
    .await;
    assert!(list_err.is_err());

    let valid_list_ix = ix_list_ticket(
        fixture.seller.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.resale_policy,
        fixture.ticket,
        fixture.listing,
        class_id,
        ticket_id,
        1_050_000_000,
        now + 1,
    );
    send_ix(
        &mut ctx.banks_client,
        &fixture.seller,
        ctx.last_blockhash,
        valid_list_ix,
        &[],
    )
    .await;
    ctx.warp_to_slot(5_000).unwrap();

    let buy_ix = ix_buy_resale_ticket(
        buyer.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.resale_policy,
        fixture.ticket,
        fixture.listing,
        fixture.seller.pubkey(),
        fixture.royalty_vault.pubkey(),
        class_id,
        ticket_id,
        1_050_000_000,
    );
    let buy_err = send_ix_result(
        &mut ctx.banks_client,
        &buyer,
        ctx.last_blockhash,
        buy_ix,
        &[],
    )
    .await;
    assert!(buy_err.is_err());

    let expire_ix = ix_expire_listing(
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.ticket,
        fixture.listing,
        class_id,
        ticket_id,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        expire_ix,
        &[],
    )
    .await;

    let listing = fetch_listing(&mut ctx.banks_client, fixture.listing).await;
    assert!(!listing.is_active);
    assert_eq!(listing.close_reason, 3);
}

#[tokio::test]
async fn time_based_transfer_lock_blocks_execution() {
    let (mut ctx, fixture) = setup_fixture(50_000).await;
    let class_id = 1;
    let ticket_id = 1;
    let buyer = Keypair::new();
    let fund_ix = solana_sdk::system_instruction::transfer(
        &ctx.payer.pubkey(),
        &buyer.pubkey(),
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

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let list_ix = ix_list_ticket(
        fixture.seller.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.resale_policy,
        fixture.ticket,
        fixture.listing,
        class_id,
        ticket_id,
        1_050_000_000,
        now + 3_600,
    );
    send_ix(
        &mut ctx.banks_client,
        &fixture.seller,
        ctx.last_blockhash,
        list_ix,
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
        fixture.seller.pubkey(),
        fixture.royalty_vault.pubkey(),
        class_id,
        ticket_id,
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
}
