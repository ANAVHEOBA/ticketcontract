use solana_sdk::signature::{Keypair, Signer};

use ticketing_core::{instructions::event::EventInput, state::FinancingOfferInput};

use crate::common::{
    disbursement_record_pda, event_pda, fetch_settlement_ledger, financing_offer_pda, get_lamports,
    ix_accept_financing_offer, ix_create_event, ix_create_financing_offer, ix_create_organizer,
    ix_disburse_advance, ix_initialize_protocol, ix_pause_event, ix_register_protocol_vaults,
    ix_settle_primary_revenue, organizer_pda, protocol_config_pda, send_ix, send_ix_result,
    settlement_ledger_pda, setup,
};

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

fn settlement_ref(tag: u8) -> [u8; 16] {
    [tag; 16]
}

#[tokio::test]
async fn settlement_reference_is_idempotent() {
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
        send_ix(
            &mut ctx.banks_client,
            &ctx.payer,
            ctx.last_blockhash,
            solana_sdk::system_instruction::transfer(
                &ctx.payer.pubkey(),
                &wallet.pubkey(),
                8_000_000_000,
            ),
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
            "https://org/security".to_string(),
            organizer_wallet.pubkey(),
        ),
        &[&organizer_authority],
    )
    .await;

    let event_id = 9901;
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
                title: "Security Event".to_string(),
                venue: "Venue".to_string(),
                start_ts: 4_402_444_800,
                end_ts: 4_402_448_400,
                sales_start_ts: 4_402_400_000,
                lock_ts: 4_402_430_000,
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

    let (record, _) = disbursement_record_pda(financing_offer, 1);
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
            record,
            organizer_wallet.pubkey(),
            financing_terms().advance_amount_lamports,
            [77u8; 16],
        ),
        &[],
    )
    .await;

    let (settlement_ledger, _) = settlement_ledger_pda(event_account);
    let gross = 1_300_000_000u64;
    let reference = settlement_ref(21);

    let financier_before =
        get_lamports(&mut ctx.banks_client, financier_wallet.pubkey()).await;
    send_ix(
        &mut ctx.banks_client,
        &revenue_source,
        ctx.last_blockhash,
        ix_settle_primary_revenue(
            revenue_source.pubkey(),
            organizer_authority.pubkey(),
            protocol_config,
            organizer_profile,
            event_account,
            financing_offer,
            settlement_ledger,
            financier_wallet.pubkey(),
            organizer_wallet.pubkey(),
            fee_vault.pubkey(),
            royalty_vault.pubkey(),
            other_vault.pubkey(),
            gross,
            500,
            500,
            500,
            reference,
        ),
        &[&organizer_authority],
    )
    .await;
    let financier_after_first =
        get_lamports(&mut ctx.banks_client, financier_wallet.pubkey()).await;

    send_ix(
        &mut ctx.banks_client,
        &revenue_source,
        ctx.last_blockhash,
        ix_settle_primary_revenue(
            revenue_source.pubkey(),
            organizer_authority.pubkey(),
            protocol_config,
            organizer_profile,
            event_account,
            financing_offer,
            settlement_ledger,
            financier_wallet.pubkey(),
            organizer_wallet.pubkey(),
            fee_vault.pubkey(),
            royalty_vault.pubkey(),
            other_vault.pubkey(),
            gross,
            500,
            500,
            500,
            reference,
        ),
        &[&organizer_authority],
    )
    .await;
    let financier_after_second =
        get_lamports(&mut ctx.banks_client, financier_wallet.pubkey()).await;

    assert!(financier_after_first > financier_before);
    assert_eq!(financier_after_second, financier_after_first);

    let ledger = fetch_settlement_ledger(&mut ctx.banks_client, settlement_ledger).await;
    assert_eq!(ledger.cumulative_primary_routed_lamports, gross);
    assert_eq!(ledger.last_settlement_reference, reference);
}

#[tokio::test]
async fn paused_event_blocks_settlement_flow() {
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
        send_ix(
            &mut ctx.banks_client,
            &ctx.payer,
            ctx.last_blockhash,
            solana_sdk::system_instruction::transfer(
                &ctx.payer.pubkey(),
                &wallet.pubkey(),
                8_000_000_000,
            ),
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
            "https://org/security-2".to_string(),
            organizer_wallet.pubkey(),
        ),
        &[&organizer_authority],
    )
    .await;

    let event_id = 9902;
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
                title: "Paused Security Event".to_string(),
                venue: "Venue".to_string(),
                start_ts: 4_402_444_800,
                end_ts: 4_402_448_400,
                sales_start_ts: 4_402_400_000,
                lock_ts: 4_402_430_000,
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

    let (record, _) = disbursement_record_pda(financing_offer, 1);
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
            record,
            organizer_wallet.pubkey(),
            financing_terms().advance_amount_lamports,
            [88u8; 16],
        ),
        &[],
    )
    .await;

    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_pause_event(
            organizer_authority.pubkey(),
            organizer_profile,
            event_account,
            true,
        ),
        &[&organizer_authority],
    )
    .await;

    let (settlement_ledger, _) = settlement_ledger_pda(event_account);
    let settle_err = send_ix_result(
        &mut ctx.banks_client,
        &revenue_source,
        ctx.last_blockhash,
        ix_settle_primary_revenue(
            revenue_source.pubkey(),
            organizer_authority.pubkey(),
            protocol_config,
            organizer_profile,
            event_account,
            financing_offer,
            settlement_ledger,
            financier_wallet.pubkey(),
            organizer_wallet.pubkey(),
            fee_vault.pubkey(),
            royalty_vault.pubkey(),
            other_vault.pubkey(),
            200_000_000,
            250,
            250,
            250,
            settlement_ref(22),
        ),
        &[&organizer_authority],
    )
    .await;
    assert!(settle_err.is_err());
}
