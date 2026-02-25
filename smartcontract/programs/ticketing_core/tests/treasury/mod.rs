use solana_sdk::signature::{Keypair, Signer};

use ticketing_core::constants::{
    ROLE_ORGANIZER_ADMIN, ROLE_SCOPE_ORGANIZER, VAULT_KIND_ORGANIZER, VAULT_KIND_PROTOCOL,
};

use crate::common::{
    fetch_vault_account, ix_create_organizer, ix_grant_role, ix_initialize_protocol,
    ix_initialize_vault, ix_snapshot_vault, ix_withdraw_vault, organizer_pda, protocol_config_pda,
    role_binding_pda, send_ix, send_ix_result, setup, vault_funds_pda, vault_state_pda,
};

#[tokio::test]
async fn protocol_vault_snapshot_and_admin_withdraw_work() {
    let mut ctx = setup().await;
    let fee_vault = Keypair::new();
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

    let (vault_state, _) = vault_state_pda(VAULT_KIND_PROTOCOL, protocol_config);
    let (vault, _) = vault_funds_pda(VAULT_KIND_PROTOCOL, protocol_config);
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_initialize_vault(
            ctx.payer.pubkey(),
            ctx.payer.pubkey(),
            protocol_config,
            protocol_config,
            protocol_config,
            protocol_config,
            vault_state,
            vault,
            protocol_config,
            VAULT_KIND_PROTOCOL,
            protocol_config,
        ),
        &[],
    )
    .await;

    let fund_vault =
        solana_sdk::system_instruction::transfer(&ctx.payer.pubkey(), &vault, 2_000_000_000);
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        fund_vault,
        &[],
    )
    .await;

    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_snapshot_vault(vault_state, vault, VAULT_KIND_PROTOCOL, protocol_config),
        &[],
    )
    .await;

    let state_after_snapshot = fetch_vault_account(&mut ctx.banks_client, vault_state).await;
    assert!(state_after_snapshot.total_inflow_lamports >= 2_000_000_000);

    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_withdraw_vault(
            ctx.payer.pubkey(),
            vault_state,
            vault,
            fee_vault.pubkey(),
            protocol_config,
            VAULT_KIND_PROTOCOL,
            protocol_config,
            1_000_000_000,
        ),
        &[],
    )
    .await;

    let state_after_withdraw = fetch_vault_account(&mut ctx.banks_client, vault_state).await;
    assert!(state_after_withdraw.total_outflow_lamports >= 1_000_000_000);
}

#[tokio::test]
async fn organizer_admin_role_can_withdraw_organizer_vault() {
    let mut ctx = setup().await;
    let organizer_authority = Keypair::new();
    let organizer_wallet = Keypair::new();
    let delegate = Keypair::new();

    for kp in [&organizer_authority, &organizer_wallet, &delegate] {
        let ix = solana_sdk::system_instruction::transfer(
            &ctx.payer.pubkey(),
            &kp.pubkey(),
            2_000_000_000,
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
            Keypair::new().pubkey(),
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
            "https://org/vault".to_string(),
            organizer_wallet.pubkey(),
        ),
        &[&organizer_authority],
    )
    .await;

    let (vault_state, _) = vault_state_pda(VAULT_KIND_ORGANIZER, organizer_profile);
    let (vault, _) = vault_funds_pda(VAULT_KIND_ORGANIZER, organizer_profile);
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_initialize_vault(
            ctx.payer.pubkey(),
            organizer_authority.pubkey(),
            protocol_config,
            organizer_profile,
            protocol_config,
            protocol_config,
            vault_state,
            vault,
            protocol_config,
            VAULT_KIND_ORGANIZER,
            organizer_profile,
        ),
        &[&organizer_authority],
    )
    .await;

    let fund_vault =
        solana_sdk::system_instruction::transfer(&ctx.payer.pubkey(), &vault, 1_000_000_000);
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        fund_vault,
        &[],
    )
    .await;
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_snapshot_vault(vault_state, vault, VAULT_KIND_ORGANIZER, organizer_profile),
        &[],
    )
    .await;

    let (organizer_admin_role, _) =
        role_binding_pda(organizer_profile, ROLE_ORGANIZER_ADMIN, delegate.pubkey());
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_grant_role(
            organizer_authority.pubkey(),
            ctx.payer.pubkey(),
            delegate.pubkey(),
            organizer_profile,
            protocol_config,
            organizer_profile,
            organizer_admin_role,
            ROLE_ORGANIZER_ADMIN,
            ROLE_SCOPE_ORGANIZER,
            0,
            0,
        ),
        &[&organizer_authority],
    )
    .await;

    let unauthorized = send_ix_result(
        &mut ctx.banks_client,
        &delegate,
        ctx.last_blockhash,
        ix_withdraw_vault(
            delegate.pubkey(),
            vault_state,
            vault,
            organizer_wallet.pubkey(),
            protocol_config,
            VAULT_KIND_ORGANIZER,
            organizer_profile,
            100_000_000,
        ),
        &[],
    )
    .await;
    assert!(unauthorized.is_err());

    send_ix(
        &mut ctx.banks_client,
        &delegate,
        ctx.last_blockhash,
        ix_withdraw_vault(
            delegate.pubkey(),
            vault_state,
            vault,
            organizer_wallet.pubkey(),
            organizer_admin_role,
            VAULT_KIND_ORGANIZER,
            organizer_profile,
            100_000_000,
        ),
        &[],
    )
    .await;

    let state = fetch_vault_account(&mut ctx.banks_client, vault_state).await;
    assert!(state.total_outflow_lamports >= 100_000_000);
}
