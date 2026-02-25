use solana_sdk::signature::{Keypair, Signer};

use ticketing_core::{
    instructions::event::EventInput,
    state::{FinancingLifecycleStatus, FinancingOfferInput},
};

use crate::common::{
    disbursement_record_pda, event_pda, fetch_disbursement_record, fetch_financing_offer,
    financing_offer_pda, get_lamports, ix_accept_financing_offer, ix_clawback_disbursement,
    ix_create_event, ix_create_financing_offer, ix_create_organizer, ix_disburse_advance,
    ix_initialize_protocol, ix_set_financing_freeze, organizer_pda, protocol_config_pda, send_ix,
    send_ix_result, setup,
};

struct Fixture {
    protocol_config: solana_sdk::pubkey::Pubkey,
    organizer_profile: solana_sdk::pubkey::Pubkey,
    event_account: solana_sdk::pubkey::Pubkey,
    financing_offer: solana_sdk::pubkey::Pubkey,
    organizer_authority: Keypair,
    organizer_wallet: Keypair,
}

async fn setup_fixture() -> (solana_program_test::ProgramTestContext, Fixture) {
    let mut ctx = setup().await;
    let organizer_authority = Keypair::new();
    let organizer_wallet = Keypair::new();

    for kp in [&organizer_authority, &organizer_wallet] {
        let fund = solana_sdk::system_instruction::transfer(
            &ctx.payer.pubkey(),
            &kp.pubkey(),
            5_000_000_000,
        );
        send_ix(
            &mut ctx.banks_client,
            &ctx.payer,
            ctx.last_blockhash,
            fund,
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
        Keypair::new().pubkey(),
        250,
        10,
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
        "https://org/financing".to_string(),
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

    let event_id = 7001;
    let (event_account, _) = event_pda(organizer_profile, event_id);
    let create_event_ix = ix_create_event(
        ctx.payer.pubkey(),
        organizer_authority.pubkey(),
        protocol_config,
        organizer_profile,
        event_account,
        event_id,
        EventInput {
            title: "Financing Event".to_string(),
            venue: "Venue".to_string(),
            start_ts: 4_302_444_800,
            end_ts: 4_302_448_400,
            sales_start_ts: 4_302_400_000,
            lock_ts: 4_302_430_000,
            capacity: 2_000,
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

    let (financing_offer, _) = financing_offer_pda(event_account);

    (
        ctx,
        Fixture {
            protocol_config,
            organizer_profile,
            event_account,
            financing_offer,
            organizer_authority,
            organizer_wallet,
        },
    )
}

fn sample_terms() -> FinancingOfferInput {
    FinancingOfferInput {
        advance_amount_lamports: 1_500_000_000,
        fee_bps: 800,
        repayment_cap_lamports: 1_800_000_000,
        schedule_start_ts: 4_302_000_000,
        schedule_interval_secs: 86_400,
        schedule_installments: 6,
    }
}

fn reference(tag: u8) -> [u8; 16] {
    [tag; 16]
}

#[tokio::test]
async fn create_financing_offer_stores_terms() {
    let (mut ctx, fixture) = setup_fixture().await;
    let create_ix = ix_create_financing_offer(
        ctx.payer.pubkey(),
        ctx.payer.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.financing_offer,
        sample_terms(),
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        create_ix,
        &[],
    )
    .await;

    let offer = fetch_financing_offer(&mut ctx.banks_client, fixture.financing_offer).await;
    assert_eq!(offer.status, FinancingLifecycleStatus::Draft);
    assert!(!offer.terms_locked);
    assert_eq!(offer.advance_amount_lamports, 1_500_000_000);
    assert_eq!(offer.fee_bps, 800);
    assert_eq!(offer.repayment_cap_lamports, 1_800_000_000);
    assert_eq!(offer.schedule_interval_secs, 86_400);
    assert_eq!(offer.schedule_installments, 6);
}

#[tokio::test]
async fn organizer_accepts_offer_and_terms_lock() {
    let (mut ctx, fixture) = setup_fixture().await;
    let create_ix = ix_create_financing_offer(
        ctx.payer.pubkey(),
        ctx.payer.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.financing_offer,
        sample_terms(),
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        create_ix,
        &[],
    )
    .await;

    let accept_ix = ix_accept_financing_offer(
        fixture.organizer_authority.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.financing_offer,
        true,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        accept_ix,
        &[&fixture.organizer_authority],
    )
    .await;

    let offer = fetch_financing_offer(&mut ctx.banks_client, fixture.financing_offer).await;
    assert_eq!(offer.status, FinancingLifecycleStatus::Accepted);
    assert!(offer.terms_locked);
    assert_eq!(offer.accepted_by, fixture.organizer_authority.pubkey());

    let relist_ix = ix_create_financing_offer(
        ctx.payer.pubkey(),
        ctx.payer.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.financing_offer,
        FinancingOfferInput {
            advance_amount_lamports: 1,
            ..sample_terms()
        },
    );
    let err = send_ix_result(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        relist_ix,
        &[],
    )
    .await;
    assert!(err.is_err());
}

#[tokio::test]
async fn organizer_can_reject_offer_on_chain() {
    let (mut ctx, fixture) = setup_fixture().await;
    let create_ix = ix_create_financing_offer(
        ctx.payer.pubkey(),
        ctx.payer.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.financing_offer,
        sample_terms(),
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        create_ix,
        &[],
    )
    .await;

    let reject_ix = ix_accept_financing_offer(
        fixture.organizer_authority.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.financing_offer,
        false,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        reject_ix,
        &[&fixture.organizer_authority],
    )
    .await;

    let offer = fetch_financing_offer(&mut ctx.banks_client, fixture.financing_offer).await;
    assert_eq!(offer.status, FinancingLifecycleStatus::Rejected);
    assert!(!offer.terms_locked);
    assert_eq!(offer.rejected_by, fixture.organizer_authority.pubkey());
}

#[tokio::test]
async fn disburse_requires_acceptance_and_tracks_lifecycle() {
    let (mut ctx, fixture) = setup_fixture().await;
    let underwriter = Keypair::new();
    let fund_underwriter = solana_sdk::system_instruction::transfer(
        &ctx.payer.pubkey(),
        &underwriter.pubkey(),
        5_000_000_000,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        fund_underwriter,
        &[],
    )
    .await;

    let create_ix = ix_create_financing_offer(
        ctx.payer.pubkey(),
        underwriter.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.financing_offer,
        sample_terms(),
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        create_ix,
        &[&underwriter],
    )
    .await;

    let (record_1, _) = disbursement_record_pda(fixture.financing_offer, 1);
    let disburse_ix = ix_disburse_advance(
        underwriter.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.financing_offer,
        record_1,
        fixture.organizer_wallet.pubkey(),
        sample_terms().advance_amount_lamports,
        reference(1),
    );
    let err = send_ix_result(
        &mut ctx.banks_client,
        &underwriter,
        ctx.last_blockhash,
        disburse_ix.clone(),
        &[],
    )
    .await;
    assert!(err.is_err());

    let accept_ix = ix_accept_financing_offer(
        fixture.organizer_authority.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.financing_offer,
        true,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        accept_ix,
        &[&fixture.organizer_authority],
    )
    .await;

    let accepted_offer =
        fetch_financing_offer(&mut ctx.banks_client, fixture.financing_offer).await;
    assert_eq!(accepted_offer.status, FinancingLifecycleStatus::Accepted);
    assert!(accepted_offer.terms_locked);
    ctx.last_blockhash = ctx.banks_client.get_latest_blockhash().await.unwrap();

    let before = get_lamports(&mut ctx.banks_client, fixture.organizer_wallet.pubkey()).await;
    let (record_1, _) = disbursement_record_pda(fixture.financing_offer, 1);
    let disburse_ix = ix_disburse_advance(
        underwriter.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.financing_offer,
        record_1,
        fixture.organizer_wallet.pubkey(),
        sample_terms().advance_amount_lamports,
        reference(1),
    );
    send_ix(
        &mut ctx.banks_client,
        &underwriter,
        ctx.last_blockhash,
        disburse_ix,
        &[],
    )
    .await;
    let after = get_lamports(&mut ctx.banks_client, fixture.organizer_wallet.pubkey()).await;

    assert_eq!(after - before, sample_terms().advance_amount_lamports);
    let offer = fetch_financing_offer(&mut ctx.banks_client, fixture.financing_offer).await;
    assert_eq!(offer.status, FinancingLifecycleStatus::Disbursed);
    assert_ne!(offer.disbursed_at, 0);
    assert_eq!(offer.disbursement_count, 1);
    let record = fetch_disbursement_record(&mut ctx.banks_client, record_1).await;
    assert_eq!(
        record.amount_lamports,
        sample_terms().advance_amount_lamports
    );
    assert_eq!(record.reference_id, reference(1));
}

#[tokio::test]
async fn tranche_constraints_and_one_time_mode_enforced() {
    let (mut ctx, fixture) = setup_fixture().await;
    let underwriter = Keypair::new();
    let fund_underwriter = solana_sdk::system_instruction::transfer(
        &ctx.payer.pubkey(),
        &underwriter.pubkey(),
        5_000_000_000,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        fund_underwriter,
        &[],
    )
    .await;

    let one_time_terms = FinancingOfferInput {
        schedule_installments: 1,
        ..sample_terms()
    };
    let create_ix = ix_create_financing_offer(
        ctx.payer.pubkey(),
        underwriter.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.financing_offer,
        one_time_terms.clone(),
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        create_ix,
        &[&underwriter],
    )
    .await;

    let accept_ix = ix_accept_financing_offer(
        fixture.organizer_authority.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.financing_offer,
        true,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        accept_ix,
        &[&fixture.organizer_authority],
    )
    .await;

    let (record_1, _) = disbursement_record_pda(fixture.financing_offer, 1);
    let partial_ix = ix_disburse_advance(
        underwriter.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.financing_offer,
        record_1,
        fixture.organizer_wallet.pubkey(),
        100_000_000,
        reference(3),
    );
    let err = send_ix_result(
        &mut ctx.banks_client,
        &underwriter,
        ctx.last_blockhash,
        partial_ix,
        &[],
    )
    .await;
    assert!(err.is_err());
}

#[tokio::test]
async fn admin_can_freeze_and_clawback_recorded_disbursement() {
    let (mut ctx, fixture) = setup_fixture().await;
    let underwriter = Keypair::new();
    let treasury_vault = Keypair::new();
    let fund_underwriter = solana_sdk::system_instruction::transfer(
        &ctx.payer.pubkey(),
        &underwriter.pubkey(),
        5_000_000_000,
    );
    let fund_treasury = solana_sdk::system_instruction::transfer(
        &ctx.payer.pubkey(),
        &treasury_vault.pubkey(),
        1_000_000_000,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        fund_underwriter,
        &[],
    )
    .await;
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        fund_treasury,
        &[],
    )
    .await;

    let register_vaults_ix = crate::common::ix_register_protocol_vaults(
        ctx.payer.pubkey(),
        fixture.protocol_config,
        treasury_vault.pubkey(),
        Keypair::new().pubkey(),
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        register_vaults_ix,
        &[],
    )
    .await;

    let create_ix = ix_create_financing_offer(
        ctx.payer.pubkey(),
        underwriter.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.financing_offer,
        sample_terms(),
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        create_ix,
        &[&underwriter],
    )
    .await;
    let accept_ix = ix_accept_financing_offer(
        fixture.organizer_authority.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.financing_offer,
        true,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        accept_ix,
        &[&fixture.organizer_authority],
    )
    .await;

    let disburse_amount = 400_000_000;
    let (record_1, _) = disbursement_record_pda(fixture.financing_offer, 1);
    let disburse_ix = ix_disburse_advance(
        underwriter.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.financing_offer,
        record_1,
        fixture.organizer_wallet.pubkey(),
        disburse_amount,
        reference(4),
    );
    send_ix(
        &mut ctx.banks_client,
        &underwriter,
        ctx.last_blockhash,
        disburse_ix,
        &[],
    )
    .await;

    let freeze_ix = ix_set_financing_freeze(
        ctx.payer.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.financing_offer,
        true,
        77,
        true,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        freeze_ix,
        &[],
    )
    .await;

    let treasury_before = get_lamports(&mut ctx.banks_client, treasury_vault.pubkey()).await;
    let organizer_before =
        get_lamports(&mut ctx.banks_client, fixture.organizer_wallet.pubkey()).await;
    let clawback_ix = ix_clawback_disbursement(
        ctx.payer.pubkey(),
        fixture.protocol_config,
        fixture.organizer_profile,
        fixture.event_account,
        fixture.financing_offer,
        record_1,
        fixture.organizer_wallet.pubkey(),
        treasury_vault.pubkey(),
        1,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        clawback_ix,
        &[&fixture.organizer_wallet],
    )
    .await;

    let treasury_after = get_lamports(&mut ctx.banks_client, treasury_vault.pubkey()).await;
    let organizer_after =
        get_lamports(&mut ctx.banks_client, fixture.organizer_wallet.pubkey()).await;
    assert_eq!(treasury_after - treasury_before, disburse_amount);
    assert_eq!(organizer_before - organizer_after, disburse_amount);

    let record = fetch_disbursement_record(&mut ctx.banks_client, record_1).await;
    assert!(record.clawed_back);
}
