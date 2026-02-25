use solana_sdk::signature::{Keypair, Signer};

use ticketing_core::{
    instructions::{event::EventInput, ticket_class::TicketClassInput},
    state::ResalePolicyInput,
};

use crate::common::{
    event_pda, fetch_listing, fetch_resale_policy, fetch_ticket, get_lamports,
    ix_buy_resale_ticket, ix_buy_ticket, ix_cancel_listing, ix_create_event, ix_create_organizer,
    ix_create_ticket_class, ix_initialize_protocol, ix_list_ticket, ix_set_resale_policy,
    listing_pda, organizer_pda, protocol_config_pda, resale_policy_pda, send_ix, send_ix_result,
    setup, ticket_class_pda, ticket_pda, wallet_purchase_counter_pda,
};

struct ResaleFixture {
    protocol_config: solana_sdk::pubkey::Pubkey,
    organizer_profile: solana_sdk::pubkey::Pubkey,
    event_account: solana_sdk::pubkey::Pubkey,
    ticket_class: solana_sdk::pubkey::Pubkey,
    ticket: solana_sdk::pubkey::Pubkey,
    listing: solana_sdk::pubkey::Pubkey,
    resale_policy: solana_sdk::pubkey::Pubkey,
    organizer_authority: Keypair,
    seller: Keypair,
    royalty_vault: Keypair,
}

async fn setup_resale_fixture() -> (solana_program_test::ProgramTestContext, ResaleFixture) {
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
        "https://org/resale".to_string(),
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
    let event_id = 808;
    let (event_account, _) = event_pda(organizer_profile, event_id);
    let create_event_ix = ix_create_event(
        ctx.payer.pubkey(),
        organizer_authority.pubkey(),
        protocol_config,
        organizer_profile,
        event_account,
        event_id,
        EventInput {
            title: "Resale Event".to_string(),
            venue: "Arena".to_string(),
            start_ts: now + 7_200,
            end_ts: now + 10_800,
            sales_start_ts: now - 3_600,
            lock_ts: now + 3_600,
            capacity: 2000,
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
            name: "VIP".to_string(),
            total_supply: 50,
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
    let (listing, _) = listing_pda(ticket);

    (
        ctx,
        ResaleFixture {
            protocol_config,
            organizer_profile,
            event_account,
            ticket_class,
            ticket,
            listing,
            resale_policy,
            organizer_authority,
            seller,
            royalty_vault,
        },
    )
}

fn base_policy(royalty_vault: solana_sdk::pubkey::Pubkey) -> ResalePolicyInput {
    ResalePolicyInput {
        max_markup_bps: 2_000,
        royalty_bps: 1_000,
        royalty_vault,
        transfer_cooldown_secs: 0,
        max_transfer_count: 10,
        transfer_lock_before_event_secs: 0,
        whitelist: vec![],
        blacklist: vec![],
    }
}

#[tokio::test]
async fn set_policy_and_markup_are_enforced() {
    let (mut ctx, fixture) = setup_resale_fixture().await;
    let class_id = 1;
    let ticket_id = 1;

    let set_policy_ix = ix_set_resale_policy(
        ctx.payer.pubkey(),
        fixture.organizer_authority.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.resale_policy,
        class_id,
        base_policy(fixture.royalty_vault.pubkey()),
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        set_policy_ix,
        &[&fixture.organizer_authority],
    )
    .await;

    let policy = fetch_resale_policy(&mut ctx.banks_client, fixture.resale_policy).await;
    assert_eq!(policy.max_markup_bps, 2_000);
    assert_eq!(policy.royalty_bps, 1_000);

    let overpriced_list_ix = ix_list_ticket(
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
        1_500_000_000,
        i64::MAX,
    );
    let overpriced_err = send_ix_result(
        &mut ctx.banks_client,
        &fixture.seller,
        ctx.last_blockhash,
        overpriced_list_ix,
        &[],
    )
    .await;
    assert!(overpriced_err.is_err());

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
        1_200_000_000,
        i64::MAX,
    );
    send_ix(
        &mut ctx.banks_client,
        &fixture.seller,
        ctx.last_blockhash,
        valid_list_ix,
        &[],
    )
    .await;

    let listing = fetch_listing(&mut ctx.banks_client, fixture.listing).await;
    assert!(listing.is_active);
    assert_eq!(listing.price_lamports, 1_200_000_000);
}

#[tokio::test]
async fn royalty_and_recipient_rules_are_enforced() {
    let (mut ctx, fixture) = setup_resale_fixture().await;
    let class_id = 1;
    let ticket_id = 1;
    let allowed_buyer = Keypair::new();
    let blocked_buyer = Keypair::new();
    for kp in [&allowed_buyer, &blocked_buyer] {
        let ix = solana_sdk::system_instruction::transfer(
            &ctx.payer.pubkey(),
            &kp.pubkey(),
            4_000_000_000,
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

    let mut policy = base_policy(fixture.royalty_vault.pubkey());
    policy.whitelist = vec![allowed_buyer.pubkey()];
    policy.blacklist = vec![blocked_buyer.pubkey()];
    let set_policy_ix = ix_set_resale_policy(
        ctx.payer.pubkey(),
        fixture.organizer_authority.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.resale_policy,
        class_id,
        policy,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        set_policy_ix,
        &[&fixture.organizer_authority],
    )
    .await;

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
        i64::MAX,
    );
    send_ix(
        &mut ctx.banks_client,
        &fixture.seller,
        ctx.last_blockhash,
        list_ix,
        &[],
    )
    .await;

    let blocked_buy_ix = ix_buy_resale_ticket(
        blocked_buyer.pubkey(),
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
    let blocked_err = send_ix_result(
        &mut ctx.banks_client,
        &blocked_buyer,
        ctx.last_blockhash,
        blocked_buy_ix,
        &[],
    )
    .await;
    assert!(blocked_err.is_err());

    let seller_before = get_lamports(&mut ctx.banks_client, fixture.seller.pubkey()).await;
    let royalty_before = get_lamports(&mut ctx.banks_client, fixture.royalty_vault.pubkey()).await;

    let allowed_buy_ix = ix_buy_resale_ticket(
        allowed_buyer.pubkey(),
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
        &allowed_buyer,
        ctx.last_blockhash,
        allowed_buy_ix,
        &[],
    )
    .await;

    let seller_after = get_lamports(&mut ctx.banks_client, fixture.seller.pubkey()).await;
    let royalty_after = get_lamports(&mut ctx.banks_client, fixture.royalty_vault.pubkey()).await;
    assert_eq!(royalty_after - royalty_before, 110_000_000);
    assert_eq!(seller_after - seller_before, 990_000_000);

    let ticket = fetch_ticket(&mut ctx.banks_client, fixture.ticket).await;
    assert_eq!(ticket.owner, allowed_buyer.pubkey());
}

#[tokio::test]
async fn cooldown_and_max_transfer_count_are_enforced() {
    let (mut ctx, fixture) = setup_resale_fixture().await;
    let class_id = 1;
    let ticket_id = 1;
    let buyer_one = Keypair::new();
    let buyer_two = Keypair::new();
    for kp in [&buyer_one, &buyer_two] {
        let ix = solana_sdk::system_instruction::transfer(
            &ctx.payer.pubkey(),
            &kp.pubkey(),
            4_000_000_000,
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

    let mut cooldown_policy = base_policy(fixture.royalty_vault.pubkey());
    cooldown_policy.transfer_cooldown_secs = i64::MAX;
    let set_policy_ix = ix_set_resale_policy(
        ctx.payer.pubkey(),
        fixture.organizer_authority.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.resale_policy,
        class_id,
        cooldown_policy,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        set_policy_ix,
        &[&fixture.organizer_authority],
    )
    .await;

    let first_list = ix_list_ticket(
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
        i64::MAX,
    );
    send_ix(
        &mut ctx.banks_client,
        &fixture.seller,
        ctx.last_blockhash,
        first_list,
        &[],
    )
    .await;

    let first_buy = ix_buy_resale_ticket(
        buyer_one.pubkey(),
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
    send_ix(
        &mut ctx.banks_client,
        &buyer_one,
        ctx.last_blockhash,
        first_buy,
        &[],
    )
    .await;

    let relist = ix_list_ticket(
        buyer_one.pubkey(),
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
        i64::MAX,
    );
    send_ix(
        &mut ctx.banks_client,
        &buyer_one,
        ctx.last_blockhash,
        relist,
        &[],
    )
    .await;

    let cooldown_blocked = ix_buy_resale_ticket(
        buyer_two.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.resale_policy,
        fixture.ticket,
        fixture.listing,
        buyer_one.pubkey(),
        fixture.royalty_vault.pubkey(),
        class_id,
        ticket_id,
        1_050_000_000,
    );
    let cooldown_err = send_ix_result(
        &mut ctx.banks_client,
        &buyer_two,
        ctx.last_blockhash,
        cooldown_blocked,
        &[],
    )
    .await;
    assert!(cooldown_err.is_err());

    let mut max_transfer_policy = base_policy(fixture.royalty_vault.pubkey());
    max_transfer_policy.max_transfer_count = 1;
    let set_max_policy_ix = ix_set_resale_policy(
        ctx.payer.pubkey(),
        fixture.organizer_authority.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.resale_policy,
        class_id,
        max_transfer_policy,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        set_max_policy_ix,
        &[&fixture.organizer_authority],
    )
    .await;

    let max_transfer_blocked = ix_buy_resale_ticket(
        buyer_two.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.resale_policy,
        fixture.ticket,
        fixture.listing,
        buyer_one.pubkey(),
        fixture.royalty_vault.pubkey(),
        class_id,
        ticket_id,
        1_050_000_000,
    );
    let max_transfer_err = send_ix_result(
        &mut ctx.banks_client,
        &buyer_two,
        ctx.last_blockhash,
        max_transfer_blocked,
        &[],
    )
    .await;
    assert!(max_transfer_err.is_err());
}

#[tokio::test]
async fn transfer_time_lock_and_listing_cancel_are_enforced() {
    let (mut ctx, fixture) = setup_resale_fixture().await;
    let class_id = 1;
    let ticket_id = 1;
    let buyer = Keypair::new();
    let ix = solana_sdk::system_instruction::transfer(
        &ctx.payer.pubkey(),
        &buyer.pubkey(),
        4_000_000_000,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix,
        &[],
    )
    .await;

    let mut lock_policy = base_policy(fixture.royalty_vault.pubkey());
    lock_policy.transfer_lock_before_event_secs = 50_000;
    let set_policy_ix = ix_set_resale_policy(
        ctx.payer.pubkey(),
        fixture.organizer_authority.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.resale_policy,
        class_id,
        lock_policy,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        set_policy_ix,
        &[&fixture.organizer_authority],
    )
    .await;

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
        i64::MAX,
    );
    send_ix(
        &mut ctx.banks_client,
        &fixture.seller,
        ctx.last_blockhash,
        list_ix,
        &[],
    )
    .await;

    let locked_buy_ix = ix_buy_resale_ticket(
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
    let locked_err = send_ix_result(
        &mut ctx.banks_client,
        &buyer,
        ctx.last_blockhash,
        locked_buy_ix,
        &[],
    )
    .await;
    assert!(locked_err.is_err());

    let cancel_ix = ix_cancel_listing(
        fixture.seller.pubkey(),
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
        &fixture.seller,
        ctx.last_blockhash,
        cancel_ix,
        &[],
    )
    .await;

    let listing = fetch_listing(&mut ctx.banks_client, fixture.listing).await;
    assert!(!listing.is_active);
}
