use solana_sdk::signature::{Keypair, Signer};

use crate::common::{
    fetch_protocol_config, ix_initialize_protocol, ix_pause_protocol, ix_register_protocol_vaults,
    ix_set_protocol_authorities, ix_set_protocol_config, protocol_config_pda, send_ix,
    send_ix_result, setup,
};

#[tokio::test]
async fn initialize_protocol_sets_expected_state() {
    let mut ctx = setup().await;
    let admin = ctx.payer.pubkey();
    let (protocol_config, _) = protocol_config_pda();

    let upgrade_authority = Keypair::new().pubkey();
    let treasury_vault = Keypair::new().pubkey();
    let fee_vault = Keypair::new().pubkey();

    let ix = ix_initialize_protocol(
        ctx.payer.pubkey(),
        admin,
        protocol_config,
        upgrade_authority,
        treasury_vault,
        fee_vault,
        250,
        8,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix,
        &[],
    )
    .await;

    let state = fetch_protocol_config(&mut ctx.banks_client, protocol_config).await;
    assert_eq!(state.admin, admin);
    assert_eq!(state.upgrade_authority, upgrade_authority);
    assert_eq!(state.treasury_vault, treasury_vault);
    assert_eq!(state.fee_vault, fee_vault);
    assert_eq!(state.protocol_fee_bps, 250);
    assert_eq!(state.max_tickets_per_wallet, 8);
    assert!(!state.is_paused);
}

#[tokio::test]
async fn initialize_protocol_rejects_invalid_limits() {
    let mut ctx = setup().await;
    let admin = ctx.payer.pubkey();
    let (protocol_config, _) = protocol_config_pda();

    let bad_fee_ix = ix_initialize_protocol(
        ctx.payer.pubkey(),
        admin,
        protocol_config,
        Keypair::new().pubkey(),
        Keypair::new().pubkey(),
        Keypair::new().pubkey(),
        10_001,
        8,
    );
    let err = send_ix_result(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        bad_fee_ix,
        &[],
    )
    .await;
    assert!(err.is_err());

    let bad_limit_ix = ix_initialize_protocol(
        ctx.payer.pubkey(),
        admin,
        protocol_config,
        Keypair::new().pubkey(),
        Keypair::new().pubkey(),
        Keypair::new().pubkey(),
        200,
        0,
    );
    let err = send_ix_result(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        bad_limit_ix,
        &[],
    )
    .await;
    assert!(err.is_err());
}

#[tokio::test]
async fn admin_can_update_config_vaults_pause_and_authorities() {
    let mut ctx = setup().await;
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

    let ix = ix_set_protocol_config(admin, protocol_config, 375, 12);
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix,
        &[],
    )
    .await;

    let new_treasury = Keypair::new().pubkey();
    let new_fee = Keypair::new().pubkey();
    let ix = ix_register_protocol_vaults(admin, protocol_config, new_treasury, new_fee);
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix,
        &[],
    )
    .await;

    let ix = ix_pause_protocol(admin, protocol_config, true);
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix,
        &[],
    )
    .await;

    let mut state = fetch_protocol_config(&mut ctx.banks_client, protocol_config).await;
    assert_eq!(state.protocol_fee_bps, 375);
    assert_eq!(state.max_tickets_per_wallet, 12);
    assert_eq!(state.treasury_vault, new_treasury);
    assert_eq!(state.fee_vault, new_fee);
    assert!(state.is_paused);

    let new_admin_kp = Keypair::new();
    let airdrop_ix = solana_sdk::system_instruction::transfer(
        &ctx.payer.pubkey(),
        &new_admin_kp.pubkey(),
        2_000_000_000,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        airdrop_ix,
        &[],
    )
    .await;

    let ix = ix_set_protocol_authorities(
        admin,
        protocol_config,
        new_admin_kp.pubkey(),
        Keypair::new().pubkey(),
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix,
        &[],
    )
    .await;

    state = fetch_protocol_config(&mut ctx.banks_client, protocol_config).await;
    assert_eq!(state.admin, new_admin_kp.pubkey());

    let unauthorized_ix = ix_pause_protocol(admin, protocol_config, false);
    let unauthorized = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[unauthorized_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer],
        ctx.last_blockhash,
    );
    let err = ctx.banks_client.process_transaction(unauthorized).await;
    assert!(err.is_err());

    let pause_with_new_admin = ix_pause_protocol(new_admin_kp.pubkey(), protocol_config, false);
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        pause_with_new_admin,
        &[&new_admin_kp],
    )
    .await;

    state = fetch_protocol_config(&mut ctx.banks_client, protocol_config).await;
    assert!(!state.is_paused);
}

#[tokio::test]
async fn unauthorized_admin_calls_fail() {
    let mut ctx = setup().await;
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

    let fake_admin = Keypair::new();
    let airdrop_ix = solana_sdk::system_instruction::transfer(
        &ctx.payer.pubkey(),
        &fake_admin.pubkey(),
        1_000_000_000,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        airdrop_ix,
        &[],
    )
    .await;

    let set_cfg_ix = ix_set_protocol_config(fake_admin.pubkey(), protocol_config, 100, 2);
    let err = send_ix_result(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        set_cfg_ix,
        &[&fake_admin],
    )
    .await;
    assert!(err.is_err());

    let pause_ix = ix_pause_protocol(fake_admin.pubkey(), protocol_config, true);
    let err = send_ix_result(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        pause_ix,
        &[&fake_admin],
    )
    .await;
    assert!(err.is_err());
}
