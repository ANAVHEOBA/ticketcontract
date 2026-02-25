use solana_sdk::signature::{Keypair, Signer};

use ticketing_core::state::OrganizerStatus;

use crate::common::{
    fetch_organizer_operator, fetch_organizer_profile, ix_create_organizer, ix_initialize_protocol,
    ix_pause_protocol, ix_set_organizer_compliance_flags, ix_set_organizer_operator,
    ix_set_organizer_status, ix_update_organizer, organizer_operator_pda, organizer_pda,
    protocol_config_pda, send_ix, send_ix_result, setup,
};

async fn init_protocol(ctx: &mut solana_program_test::ProgramTestContext) {
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
}

#[tokio::test]
async fn create_and_update_organizer_profile() {
    let mut ctx = setup().await;
    init_protocol(&mut ctx).await;

    let authority = Keypair::new();
    let fund_authority = solana_sdk::system_instruction::transfer(
        &ctx.payer.pubkey(),
        &authority.pubkey(),
        1_000_000_000,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        fund_authority,
        &[],
    )
    .await;

    let (protocol_config, _) = protocol_config_pda();
    let (organizer_profile, _) = organizer_pda(authority.pubkey());

    let create_ix = ix_create_organizer(
        ctx.payer.pubkey(),
        authority.pubkey(),
        protocol_config,
        organizer_profile,
        "https://org/one".to_string(),
        authority.pubkey(),
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        create_ix,
        &[&authority],
    )
    .await;

    let mut organizer = fetch_organizer_profile(&mut ctx.banks_client, organizer_profile).await;
    assert_eq!(organizer.authority, authority.pubkey());
    assert_eq!(organizer.payout_wallet, authority.pubkey());
    assert_eq!(organizer.status, OrganizerStatus::Active);
    assert_eq!(organizer.compliance_flags, 0);
    assert_eq!(organizer.metadata_uri, "https://org/one");

    let new_payout = Keypair::new().pubkey();
    let update_ix = ix_update_organizer(
        authority.pubkey(),
        organizer_profile,
        "https://org/two".to_string(),
        new_payout,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        update_ix,
        &[&authority],
    )
    .await;

    organizer = fetch_organizer_profile(&mut ctx.banks_client, organizer_profile).await;
    assert_eq!(organizer.metadata_uri, "https://org/two");
    assert_eq!(organizer.payout_wallet, new_payout);
}

#[tokio::test]
async fn admin_can_set_status_and_compliance_flags() {
    let mut ctx = setup().await;
    init_protocol(&mut ctx).await;

    let authority = Keypair::new();
    let fund_authority = solana_sdk::system_instruction::transfer(
        &ctx.payer.pubkey(),
        &authority.pubkey(),
        1_000_000_000,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        fund_authority,
        &[],
    )
    .await;

    let admin = ctx.payer.pubkey();
    let (protocol_config, _) = protocol_config_pda();
    let (organizer_profile, _) = organizer_pda(authority.pubkey());

    let create_ix = ix_create_organizer(
        ctx.payer.pubkey(),
        authority.pubkey(),
        protocol_config,
        organizer_profile,
        "https://org/admin".to_string(),
        authority.pubkey(),
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        create_ix,
        &[&authority],
    )
    .await;

    let set_status_ix = ix_set_organizer_status(admin, protocol_config, organizer_profile, 2);
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        set_status_ix,
        &[],
    )
    .await;

    let set_flags_ix =
        ix_set_organizer_compliance_flags(admin, protocol_config, organizer_profile, 0b1011);
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        set_flags_ix,
        &[],
    )
    .await;

    let organizer = fetch_organizer_profile(&mut ctx.banks_client, organizer_profile).await;
    assert_eq!(organizer.status, OrganizerStatus::Suspended);
    assert_eq!(organizer.compliance_flags, 0b1011);
}

#[tokio::test]
async fn authority_can_delegate_operator_with_permissions() {
    let mut ctx = setup().await;
    init_protocol(&mut ctx).await;

    let authority = Keypair::new();
    let operator = Keypair::new();
    let fund_authority = solana_sdk::system_instruction::transfer(
        &ctx.payer.pubkey(),
        &authority.pubkey(),
        1_000_000_000,
    );
    let fund_operator = solana_sdk::system_instruction::transfer(
        &ctx.payer.pubkey(),
        &operator.pubkey(),
        1_000_000_000,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        fund_authority,
        &[],
    )
    .await;
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        fund_operator,
        &[],
    )
    .await;

    let (protocol_config, _) = protocol_config_pda();
    let (organizer_profile, _) = organizer_pda(authority.pubkey());
    let create_ix = ix_create_organizer(
        ctx.payer.pubkey(),
        authority.pubkey(),
        protocol_config,
        organizer_profile,
        "https://org/operator".to_string(),
        authority.pubkey(),
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        create_ix,
        &[&authority],
    )
    .await;

    let (organizer_operator, _) = organizer_operator_pda(organizer_profile, operator.pubkey());
    let set_operator_ix = ix_set_organizer_operator(
        ctx.payer.pubkey(),
        authority.pubkey(),
        operator.pubkey(),
        organizer_profile,
        organizer_operator,
        0x00FF,
        true,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        set_operator_ix,
        &[&authority],
    )
    .await;

    let operator_state = fetch_organizer_operator(&mut ctx.banks_client, organizer_operator).await;
    assert_eq!(operator_state.organizer, organizer_profile);
    assert_eq!(operator_state.operator, operator.pubkey());
    assert_eq!(operator_state.permissions, 0x00FF);
    assert!(operator_state.active);
}

#[tokio::test]
async fn organizer_constraints_enforced() {
    let mut ctx = setup().await;
    init_protocol(&mut ctx).await;

    let authority = Keypair::new();
    let other = Keypair::new();
    let fund_authority = solana_sdk::system_instruction::transfer(
        &ctx.payer.pubkey(),
        &authority.pubkey(),
        1_000_000_000,
    );
    let fund_other = solana_sdk::system_instruction::transfer(
        &ctx.payer.pubkey(),
        &other.pubkey(),
        1_000_000_000,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        fund_authority,
        &[],
    )
    .await;
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        fund_other,
        &[],
    )
    .await;

    let admin = ctx.payer.pubkey();
    let (protocol_config, _) = protocol_config_pda();

    let pause_ix = ix_pause_protocol(admin, protocol_config, true);
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        pause_ix,
        &[],
    )
    .await;

    let (paused_organizer, _) = organizer_pda(authority.pubkey());
    let create_paused_ix = ix_create_organizer(
        ctx.payer.pubkey(),
        authority.pubkey(),
        protocol_config,
        paused_organizer,
        "https://org/paused".to_string(),
        authority.pubkey(),
    );
    let err = send_ix_result(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        create_paused_ix,
        &[&authority],
    )
    .await;
    assert!(err.is_err());

    let unpause_ix = ix_pause_protocol(admin, protocol_config, false);
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        unpause_ix,
        &[],
    )
    .await;

    let (organizer_profile, _) = organizer_pda(authority.pubkey());
    let create_ix = ix_create_organizer(
        ctx.payer.pubkey(),
        authority.pubkey(),
        protocol_config,
        organizer_profile,
        "https://org/live".to_string(),
        authority.pubkey(),
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        create_ix,
        &[&authority],
    )
    .await;

    let unauthorized_status_ix =
        ix_set_organizer_status(other.pubkey(), protocol_config, organizer_profile, 2);
    let err = send_ix_result(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        unauthorized_status_ix,
        &[&other],
    )
    .await;
    assert!(err.is_err());

    let unauthorized_update_ix = ix_update_organizer(
        other.pubkey(),
        organizer_profile,
        "https://org/bad".to_string(),
        other.pubkey(),
    );
    let err = send_ix_result(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        unauthorized_update_ix,
        &[&other],
    )
    .await;
    assert!(err.is_err());
}
