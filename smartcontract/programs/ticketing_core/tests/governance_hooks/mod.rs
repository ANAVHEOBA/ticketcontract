use solana_sdk::{
    instruction::AccountMeta,
    signature::{Keypair, Signer},
};

use crate::common::{
    fetch_protocol_config, ix_accept_upgrade_authority_handoff, ix_begin_upgrade_authority_handoff,
    ix_emergency_rotate_admin, ix_execute_protocol_config_change, ix_initialize_protocol,
    ix_pause_protocol, ix_queue_protocol_config_change, ix_set_multisig_config, ix_set_protocol_config,
    ix_set_timelock_delay, protocol_config_pda, send_ix, send_ix_result, setup,
};

#[tokio::test]
async fn upgrade_handoff_requires_pending_authority_acceptance() {
    let mut ctx = setup().await;
    let (protocol_config, _) = protocol_config_pda();
    let old_upgrade = Keypair::new().pubkey();
    let new_upgrade = Keypair::new();
    let wrong = Keypair::new();

    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_initialize_protocol(
            ctx.payer.pubkey(),
            ctx.payer.pubkey(),
            protocol_config,
            old_upgrade,
            Keypair::new().pubkey(),
            Keypair::new().pubkey(),
            250,
            8,
        ),
        &[],
    )
    .await;

    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_begin_upgrade_authority_handoff(
            ctx.payer.pubkey(),
            protocol_config,
            new_upgrade.pubkey(),
        ),
        &[],
    )
    .await;

    let wrong_accept = send_ix_result(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_accept_upgrade_authority_handoff(wrong.pubkey(), protocol_config),
        &[&wrong],
    )
    .await;
    assert!(wrong_accept.is_err());

    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_accept_upgrade_authority_handoff(new_upgrade.pubkey(), protocol_config),
        &[&new_upgrade],
    )
    .await;

    let state = fetch_protocol_config(&mut ctx.banks_client, protocol_config).await;
    assert_eq!(state.upgrade_authority, new_upgrade.pubkey());
    assert_eq!(state.pending_upgrade_authority, solana_sdk::pubkey::Pubkey::default());
}

#[tokio::test]
async fn optional_multisig_gates_privileged_calls() {
    let mut ctx = setup().await;
    let (protocol_config, _) = protocol_config_pda();
    let cosigner = Keypair::new();

    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        solana_sdk::system_instruction::transfer(
            &ctx.payer.pubkey(),
            &cosigner.pubkey(),
            2_000_000_000,
        ),
        &[],
    )
    .await;

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
            Keypair::new().pubkey(),
            250,
            8,
        ),
        &[],
    )
    .await;

    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_set_multisig_config(
            ctx.payer.pubkey(),
            protocol_config,
            true,
            2,
            ctx.payer.pubkey(),
            cosigner.pubkey(),
            solana_sdk::pubkey::Pubkey::default(),
        ),
        &[],
    )
    .await;

    let denied = send_ix_result(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_pause_protocol(ctx.payer.pubkey(), protocol_config, true),
        &[],
    )
    .await;
    assert!(denied.is_err());

    let mut pause_ix = ix_pause_protocol(ctx.payer.pubkey(), protocol_config, true);
    pause_ix
        .accounts
        .push(AccountMeta::new_readonly(cosigner.pubkey(), true));
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        pause_ix,
        &[&cosigner],
    )
    .await;

    let state = fetch_protocol_config(&mut ctx.banks_client, protocol_config).await;
    assert!(state.is_paused);
    assert!(state.multisig_enabled);
}

#[tokio::test]
async fn timelock_queue_and_execute_hooks_are_enforced() {
    let mut ctx = setup().await;
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
            Keypair::new().pubkey(),
            250,
            8,
        ),
        &[],
    )
    .await;

    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_set_timelock_delay(ctx.payer.pubkey(), protocol_config, 300),
        &[],
    )
    .await;

    let direct_change = send_ix_result(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_set_protocol_config(ctx.payer.pubkey(), protocol_config, 375, 10),
        &[],
    )
    .await;
    assert!(direct_change.is_err());

    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_queue_protocol_config_change(ctx.payer.pubkey(), protocol_config, 375, 10),
        &[],
    )
    .await;

    let early_execute = send_ix_result(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_execute_protocol_config_change(ctx.payer.pubkey(), protocol_config),
        &[],
    )
    .await;
    assert!(early_execute.is_err());
}

#[tokio::test]
async fn emergency_admin_can_rotate_admin_when_paused() {
    let mut ctx = setup().await;
    let (protocol_config, _) = protocol_config_pda();
    let new_admin = Keypair::new();
    let new_emergency = Keypair::new();

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
            Keypair::new().pubkey(),
            250,
            8,
        ),
        &[],
    )
    .await;

    let unpaused_attempt = send_ix_result(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_emergency_rotate_admin(
            ctx.payer.pubkey(),
            protocol_config,
            new_admin.pubkey(),
            new_emergency.pubkey(),
            9001,
        ),
        &[],
    )
    .await;
    assert!(unpaused_attempt.is_err());

    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_pause_protocol(ctx.payer.pubkey(), protocol_config, true),
        &[],
    )
    .await;

    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_emergency_rotate_admin(
            ctx.payer.pubkey(),
            protocol_config,
            new_admin.pubkey(),
            new_emergency.pubkey(),
            9002,
        ),
        &[],
    )
    .await;

    let state = fetch_protocol_config(&mut ctx.banks_client, protocol_config).await;
    assert_eq!(state.admin, new_admin.pubkey());
    assert_eq!(state.emergency_admin, new_emergency.pubkey());
    assert_eq!(state.emergency_action_nonce, 1);
}
