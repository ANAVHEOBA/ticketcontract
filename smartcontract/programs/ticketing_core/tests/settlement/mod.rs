use solana_sdk::signature::{Keypair, Signer};

use ticketing_core::{
    instructions::event::EventInput,
    state::{FinancingLifecycleStatus, FinancingOfferInput},
};

use crate::common::{
    disbursement_record_pda, event_pda, fetch_financing_offer, fetch_settlement_ledger,
    financing_offer_pda, get_lamports, ix_accept_financing_offer, ix_create_event,
    ix_create_financing_offer, ix_create_organizer, ix_disburse_advance, ix_finalize_settlement,
    ix_initialize_protocol, ix_register_protocol_vaults, ix_settle_primary_revenue,
    ix_settle_resale_revenue, organizer_pda, protocol_config_pda, send_ix, send_ix_result,
    settlement_ledger_pda, setup,
};

struct Fixture {
    protocol_config: solana_sdk::pubkey::Pubkey,
    organizer_profile: solana_sdk::pubkey::Pubkey,
    event_account: solana_sdk::pubkey::Pubkey,
    financing_offer: solana_sdk::pubkey::Pubkey,
    settlement_ledger: solana_sdk::pubkey::Pubkey,
    organizer_authority: Keypair,
    organizer_wallet: Keypair,
    fee_vault: Keypair,
    financier_wallet: Keypair,
    royalty_vault: Keypair,
    other_vault: Keypair,
    revenue_source: Keypair,
}

fn financing_terms() -> FinancingOfferInput {
    FinancingOfferInput {
        advance_amount_lamports: 1_000_000_000,
        fee_bps: 1_000,
        repayment_cap_lamports: 1_100_000_000,
        schedule_start_ts: 4_302_000_000,
        schedule_interval_secs: 86_400,
        schedule_installments: 2,
    }
}

fn reference(tag: u8) -> [u8; 16] {
    [tag; 16]
}

async fn setup_fixture() -> (solana_program_test::ProgramTestContext, Fixture) {
    let mut ctx = setup().await;

    let organizer_authority = Keypair::new();
    let organizer_wallet = Keypair::new();
    let treasury_vault = Keypair::new();
    let fee_vault = Keypair::new();
    let financier_wallet = Keypair::new();
    let royalty_vault = Keypair::new();
    let other_vault = Keypair::new();
    let revenue_source = Keypair::new();
    let underwriter = Keypair::new();

    for wallet in [
        &organizer_authority,
        &organizer_wallet,
        &treasury_vault,
        &fee_vault,
        &financier_wallet,
        &royalty_vault,
        &other_vault,
        &revenue_source,
        &underwriter,
    ] {
        let ix = solana_sdk::system_instruction::transfer(
            &ctx.payer.pubkey(),
            &wallet.pubkey(),
            8_000_000_000,
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
            treasury_vault.pubkey(),
            fee_vault.pubkey(),
            250,
            10,
        ),
        &[],
    )
    .await;

    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_register_protocol_vaults(
            ctx.payer.pubkey(),
            protocol_config,
            treasury_vault.pubkey(),
            fee_vault.pubkey(),
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
            "https://org/settlement".to_string(),
            organizer_wallet.pubkey(),
        ),
        &[&organizer_authority],
    )
    .await;

    let event_id = 9001;
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
                title: "Settlement Event".to_string(),
                venue: "Venue".to_string(),
                start_ts: 4_302_444_800,
                end_ts: 4_302_448_400,
                sales_start_ts: 4_302_400_000,
                lock_ts: 4_302_430_000,
                capacity: 5_000,
            },
        ),
        &[&organizer_authority],
    )
    .await;

    let (financing_offer, _) = financing_offer_pda(event_account);
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_create_financing_offer(
            ctx.payer.pubkey(),
            underwriter.pubkey(),
            protocol_config,
            organizer_profile,
            event_account,
            financing_offer,
            financing_terms(),
        ),
        &[&underwriter],
    )
    .await;
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_accept_financing_offer(
            organizer_authority.pubkey(),
            protocol_config,
            organizer_profile,
            event_account,
            financing_offer,
            true,
        ),
        &[&organizer_authority],
    )
    .await;

    let (record_1, _) = disbursement_record_pda(financing_offer, 1);
    send_ix(
        &mut ctx.banks_client,
        &underwriter,
        ctx.last_blockhash,
        ix_disburse_advance(
            underwriter.pubkey(),
            protocol_config,
            organizer_profile,
            event_account,
            financing_offer,
            record_1,
            organizer_wallet.pubkey(),
            financing_terms().advance_amount_lamports,
            reference(9),
        ),
        &[],
    )
    .await;

    let (settlement_ledger, _) = settlement_ledger_pda(event_account);
    (
        ctx,
        Fixture {
            protocol_config,
            organizer_profile,
            event_account,
            financing_offer,
            settlement_ledger,
            organizer_authority,
            organizer_wallet,
            fee_vault,
            financier_wallet,
            royalty_vault,
            other_vault,
            revenue_source,
        },
    )
}

#[tokio::test]
async fn waterfall_routes_primary_and_marks_financing_settled() {
    let (mut ctx, fixture) = setup_fixture().await;

    let gross = 1_300_000_000u64;
    let protocol_bps = 500u16;
    let royalty_bps = 500u16;
    let other_bps = 500u16;
    let protocol_expected = gross * u64::from(protocol_bps) / 10_000;
    let royalty_expected = gross * u64::from(royalty_bps) / 10_000;
    let other_expected = gross * u64::from(other_bps) / 10_000;
    let pool_p1_p2 = gross - protocol_expected - royalty_expected - other_expected;
    let financier_expected = 1_100_000_000u64;
    let organizer_expected = pool_p1_p2 - financier_expected;

    let financier_before =
        get_lamports(&mut ctx.banks_client, fixture.financier_wallet.pubkey()).await;
    let organizer_before =
        get_lamports(&mut ctx.banks_client, fixture.organizer_wallet.pubkey()).await;
    let fee_before = get_lamports(&mut ctx.banks_client, fixture.fee_vault.pubkey()).await;
    let royalty_before = get_lamports(&mut ctx.banks_client, fixture.royalty_vault.pubkey()).await;
    let other_before = get_lamports(&mut ctx.banks_client, fixture.other_vault.pubkey()).await;

    send_ix(
        &mut ctx.banks_client,
        &fixture.revenue_source,
        ctx.last_blockhash,
        ix_settle_primary_revenue(
            fixture.revenue_source.pubkey(),
            fixture.organizer_authority.pubkey(),
            fixture.protocol_config,
            fixture.organizer_profile,
            fixture.event_account,
            fixture.financing_offer,
            fixture.settlement_ledger,
            fixture.financier_wallet.pubkey(),
            fixture.organizer_wallet.pubkey(),
            fixture.fee_vault.pubkey(),
            fixture.royalty_vault.pubkey(),
            fixture.other_vault.pubkey(),
            gross,
            protocol_bps,
            royalty_bps,
            other_bps,
        ),
        &[&fixture.organizer_authority],
    )
    .await;

    let financier_after =
        get_lamports(&mut ctx.banks_client, fixture.financier_wallet.pubkey()).await;
    let organizer_after =
        get_lamports(&mut ctx.banks_client, fixture.organizer_wallet.pubkey()).await;
    let fee_after = get_lamports(&mut ctx.banks_client, fixture.fee_vault.pubkey()).await;
    let royalty_after = get_lamports(&mut ctx.banks_client, fixture.royalty_vault.pubkey()).await;
    let other_after = get_lamports(&mut ctx.banks_client, fixture.other_vault.pubkey()).await;
    assert_eq!(financier_after - financier_before, financier_expected);
    assert_eq!(organizer_after - organizer_before, organizer_expected);
    assert_eq!(fee_after - fee_before, protocol_expected);
    assert_eq!(royalty_after - royalty_before, royalty_expected);
    assert_eq!(other_after - other_before, other_expected);

    let ledger = fetch_settlement_ledger(&mut ctx.banks_client, fixture.settlement_ledger).await;
    assert_eq!(ledger.cumulative_primary_routed_lamports, gross);
    assert_eq!(
        ledger.cumulative_financier_paid_lamports,
        financier_expected
    );
    assert!(ledger.financing_settled);
    let financing_offer =
        fetch_financing_offer(&mut ctx.banks_client, fixture.financing_offer).await;
    assert_eq!(financing_offer.status, FinancingLifecycleStatus::Settled);
}

#[tokio::test]
async fn resale_routing_updates_secondary_counters_after_settlement() {
    let (mut ctx, fixture) = setup_fixture().await;

    send_ix(
        &mut ctx.banks_client,
        &fixture.revenue_source,
        ctx.last_blockhash,
        ix_settle_primary_revenue(
            fixture.revenue_source.pubkey(),
            fixture.organizer_authority.pubkey(),
            fixture.protocol_config,
            fixture.organizer_profile,
            fixture.event_account,
            fixture.financing_offer,
            fixture.settlement_ledger,
            fixture.financier_wallet.pubkey(),
            fixture.organizer_wallet.pubkey(),
            fixture.fee_vault.pubkey(),
            fixture.royalty_vault.pubkey(),
            fixture.other_vault.pubkey(),
            1_300_000_000,
            500,
            500,
            500,
        ),
        &[&fixture.organizer_authority],
    )
    .await;

    let financier_before =
        get_lamports(&mut ctx.banks_client, fixture.financier_wallet.pubkey()).await;
    let organizer_before =
        get_lamports(&mut ctx.banks_client, fixture.organizer_wallet.pubkey()).await;
    let gross_resale = 200_000_000u64;
    send_ix(
        &mut ctx.banks_client,
        &fixture.revenue_source,
        ctx.last_blockhash,
        ix_settle_resale_revenue(
            fixture.revenue_source.pubkey(),
            fixture.organizer_authority.pubkey(),
            fixture.protocol_config,
            fixture.organizer_profile,
            fixture.event_account,
            fixture.financing_offer,
            fixture.settlement_ledger,
            fixture.financier_wallet.pubkey(),
            fixture.organizer_wallet.pubkey(),
            fixture.fee_vault.pubkey(),
            fixture.royalty_vault.pubkey(),
            fixture.other_vault.pubkey(),
            gross_resale,
            300,
            400,
            200,
        ),
        &[&fixture.organizer_authority],
    )
    .await;

    let financier_after =
        get_lamports(&mut ctx.banks_client, fixture.financier_wallet.pubkey()).await;
    let organizer_after =
        get_lamports(&mut ctx.banks_client, fixture.organizer_wallet.pubkey()).await;
    assert_eq!(financier_after - financier_before, 0);
    assert!(organizer_after > organizer_before);

    let ledger = fetch_settlement_ledger(&mut ctx.banks_client, fixture.settlement_ledger).await;
    assert_eq!(ledger.cumulative_secondary_routed_lamports, gross_resale);
    assert!(ledger.financing_settled);
}

#[tokio::test]
async fn finalize_settlement_requires_obligation_completion() {
    let (mut ctx, fixture) = setup_fixture().await;

    send_ix(
        &mut ctx.banks_client,
        &fixture.revenue_source,
        ctx.last_blockhash,
        ix_settle_primary_revenue(
            fixture.revenue_source.pubkey(),
            fixture.organizer_authority.pubkey(),
            fixture.protocol_config,
            fixture.organizer_profile,
            fixture.event_account,
            fixture.financing_offer,
            fixture.settlement_ledger,
            fixture.financier_wallet.pubkey(),
            fixture.organizer_wallet.pubkey(),
            fixture.fee_vault.pubkey(),
            fixture.royalty_vault.pubkey(),
            fixture.other_vault.pubkey(),
            500_000_000,
            0,
            0,
            0,
        ),
        &[&fixture.organizer_authority],
    )
    .await;

    let finalize_err = send_ix_result(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_finalize_settlement(
            fixture.organizer_authority.pubkey(),
            fixture.protocol_config,
            fixture.organizer_profile,
            fixture.event_account,
            fixture.financing_offer,
            fixture.settlement_ledger,
        ),
        &[&fixture.organizer_authority],
    )
    .await;
    assert!(finalize_err.is_err());

    send_ix(
        &mut ctx.banks_client,
        &fixture.revenue_source,
        ctx.last_blockhash,
        ix_settle_primary_revenue(
            fixture.revenue_source.pubkey(),
            fixture.organizer_authority.pubkey(),
            fixture.protocol_config,
            fixture.organizer_profile,
            fixture.event_account,
            fixture.financing_offer,
            fixture.settlement_ledger,
            fixture.financier_wallet.pubkey(),
            fixture.organizer_wallet.pubkey(),
            fixture.fee_vault.pubkey(),
            fixture.royalty_vault.pubkey(),
            fixture.other_vault.pubkey(),
            700_000_000,
            0,
            0,
            0,
        ),
        &[&fixture.organizer_authority],
    )
    .await;

    let ledger_after_second =
        fetch_settlement_ledger(&mut ctx.banks_client, fixture.settlement_ledger).await;
    let cap = financing_terms().repayment_cap_lamports;
    if ledger_after_second.cumulative_financier_paid_lamports < cap {
        let top_up = cap - ledger_after_second.cumulative_financier_paid_lamports;
        send_ix(
            &mut ctx.banks_client,
            &fixture.revenue_source,
            ctx.last_blockhash,
            ix_settle_primary_revenue(
                fixture.revenue_source.pubkey(),
                fixture.organizer_authority.pubkey(),
                fixture.protocol_config,
                fixture.organizer_profile,
                fixture.event_account,
                fixture.financing_offer,
                fixture.settlement_ledger,
                fixture.financier_wallet.pubkey(),
                fixture.organizer_wallet.pubkey(),
                fixture.fee_vault.pubkey(),
                fixture.royalty_vault.pubkey(),
                fixture.other_vault.pubkey(),
                top_up,
                0,
                0,
                0,
            ),
            &[&fixture.organizer_authority],
        )
        .await;
    }

    let settled_ledger =
        fetch_settlement_ledger(&mut ctx.banks_client, fixture.settlement_ledger).await;
    assert!(settled_ledger.financing_settled);
    assert!(
        settled_ledger.cumulative_financier_paid_lamports
            >= financing_terms().repayment_cap_lamports
    );
    let financing_offer =
        fetch_financing_offer(&mut ctx.banks_client, fixture.financing_offer).await;
    assert_eq!(financing_offer.status, FinancingLifecycleStatus::Settled);
}
