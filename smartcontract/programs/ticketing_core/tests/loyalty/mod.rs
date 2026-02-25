use solana_sdk::signature::{Keypair, Signer};

use ticketing_core::instructions::{event::EventInput, ticket_class::TicketClassInput};

use crate::common::{
    event_pda, fetch_loyalty_ledger, ix_accrue_points, ix_buy_ticket, ix_check_in_ticket,
    ix_create_event, ix_create_organizer, ix_create_ticket_class, ix_initialize_protocol,
    ix_redeem_points, ix_set_event_loyalty_multiplier, ix_set_global_loyalty_multiplier,
    loyalty_ledger_pda, organizer_pda, protocol_config_pda, send_ix, setup, ticket_class_pda,
    ticket_pda, wallet_purchase_counter_pda,
};

struct Fixture {
    protocol_config: solana_sdk::pubkey::Pubkey,
    organizer_profile: solana_sdk::pubkey::Pubkey,
    event_account: solana_sdk::pubkey::Pubkey,
    ticket_class: solana_sdk::pubkey::Pubkey,
    ticket: solana_sdk::pubkey::Pubkey,
    loyalty_ledger: solana_sdk::pubkey::Pubkey,
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
            "https://org/loyalty".to_string(),
            organizer_wallet.pubkey(),
        ),
        &[&organizer_authority],
    )
    .await;

    let event_id = 1_414;
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
                title: "Loyalty Event".to_string(),
                venue: "Arena".to_string(),
                start_ts: 4_502_444_800,
                end_ts: 4_502_448_400,
                sales_start_ts: 4_502_430_000,
                lock_ts: 4_502_440_000,
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

    let (loyalty_ledger, _) = loyalty_ledger_pda(buyer.pubkey());

    (
        ctx,
        Fixture {
            protocol_config,
            organizer_profile,
            event_account,
            ticket_class,
            ticket,
            loyalty_ledger,
            organizer_authority,
            buyer,
        },
    )
}

#[tokio::test]
async fn purchase_accrual_creates_wallet_ledger() {
    let (mut ctx, fixture) = setup_fixture().await;

    let accrue_ix = ix_accrue_points(
        ctx.payer.pubkey(),
        fixture.organizer_authority.pubkey(),
        fixture.buyer.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.ticket,
        fixture.loyalty_ledger,
        1,
        1,
        1,
        100,
        0,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        accrue_ix,
        &[&fixture.organizer_authority],
    )
    .await;

    let ledger = fetch_loyalty_ledger(&mut ctx.banks_client, fixture.loyalty_ledger).await;
    assert_eq!(ledger.wallet, fixture.buyer.pubkey());
    assert_eq!(ledger.total_accrued_points, 100);
    assert_eq!(ledger.available_points, 100);
}

#[tokio::test]
async fn global_and_event_multipliers_apply_to_attendance_accrual() {
    let (mut ctx, fixture) = setup_fixture().await;

    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_set_global_loyalty_multiplier(ctx.payer.pubkey(), fixture.protocol_config, 20_000),
        &[],
    )
    .await;

    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_set_event_loyalty_multiplier(
            fixture.organizer_authority.pubkey(),
            fixture.organizer_profile,
            fixture.event_account,
            15_000,
        ),
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
            "G1".to_string(),
        ),
        &[&fixture.organizer_authority],
    )
    .await;

    let accrue_ix = ix_accrue_points(
        ctx.payer.pubkey(),
        fixture.organizer_authority.pubkey(),
        fixture.buyer.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.ticket,
        fixture.loyalty_ledger,
        1,
        1,
        2,
        100,
        0,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        accrue_ix,
        &[&fixture.organizer_authority],
    )
    .await;

    let ledger = fetch_loyalty_ledger(&mut ctx.banks_client, fixture.loyalty_ledger).await;
    assert_eq!(ledger.total_accrued_points, 300);
    assert_eq!(ledger.available_points, 300);
}

#[tokio::test]
async fn hold_duration_accrual_and_redeem_reduce_available_points() {
    let (mut ctx, fixture) = setup_fixture().await;

    let accrue_ix = ix_accrue_points(
        ctx.payer.pubkey(),
        fixture.organizer_authority.pubkey(),
        fixture.buyer.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.ticket_class,
        fixture.ticket,
        fixture.loyalty_ledger,
        1,
        1,
        3,
        10,
        3,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        accrue_ix,
        &[&fixture.organizer_authority],
    )
    .await;

    send_ix(
        &mut ctx.banks_client,
        &fixture.buyer,
        ctx.last_blockhash,
        ix_redeem_points(
            fixture.buyer.pubkey(),
            fixture.protocol_config,
            fixture.loyalty_ledger,
            12,
            "EARLY_ACCESS".to_string(),
            fixture.event_account,
        ),
        &[],
    )
    .await;

    let ledger = fetch_loyalty_ledger(&mut ctx.banks_client, fixture.loyalty_ledger).await;
    assert_eq!(ledger.total_accrued_points, 30);
    assert_eq!(ledger.total_redeemed_points, 12);
    assert_eq!(ledger.available_points, 18);
}
