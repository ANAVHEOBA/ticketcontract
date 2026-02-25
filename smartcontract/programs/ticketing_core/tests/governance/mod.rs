use solana_sdk::signature::{Keypair, Signer};

use ticketing_core::{
    constants::{ROLE_PROTOCOL_ADMIN, ROLE_SCANNER, ROLE_SCOPE_ORGANIZER, ROLE_SCOPE_PROTOCOL},
    instructions::{event::EventInput, ticket_class::TicketClassInput},
};

use crate::common::{
    event_pda, fetch_role_binding, fetch_ticket, ix_buy_ticket, ix_check_in_ticket,
    ix_create_event, ix_create_organizer, ix_create_ticket_class, ix_grant_role,
    ix_initialize_protocol, ix_revoke_role, organizer_pda, protocol_config_pda, role_binding_pda,
    send_ix, send_ix_result, setup, ticket_class_pda, ticket_pda, wallet_purchase_counter_pda,
};

#[tokio::test]
async fn protocol_admin_can_grant_and_revoke_protocol_admin_role() {
    let mut ctx = setup().await;
    let delegate = Keypair::new();
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

    let (role_binding, _) =
        role_binding_pda(protocol_config, ROLE_PROTOCOL_ADMIN, delegate.pubkey());
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_grant_role(
            ctx.payer.pubkey(),
            ctx.payer.pubkey(),
            delegate.pubkey(),
            protocol_config,
            protocol_config,
            protocol_config,
            role_binding,
            ROLE_PROTOCOL_ADMIN,
            ROLE_SCOPE_PROTOCOL,
            0,
            0,
        ),
        &[],
    )
    .await;

    let role = fetch_role_binding(&mut ctx.banks_client, role_binding).await;
    assert!(role.active);
    assert_eq!(role.role, ROLE_PROTOCOL_ADMIN);
    assert_eq!(role.scope, ROLE_SCOPE_PROTOCOL);
    assert_eq!(role.subject, delegate.pubkey());
    assert_eq!(role.target, protocol_config);

    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_revoke_role(
            ctx.payer.pubkey(),
            delegate.pubkey(),
            protocol_config,
            protocol_config,
            protocol_config,
            role_binding,
            ROLE_PROTOCOL_ADMIN,
            ROLE_SCOPE_PROTOCOL,
            11,
        ),
        &[],
    )
    .await;

    let revoked = fetch_role_binding(&mut ctx.banks_client, role_binding).await;
    assert!(!revoked.active);
    assert!(revoked.revoked_at > 0);
}

#[tokio::test]
async fn organizer_scanner_role_grant_enables_checkin_until_revoked() {
    let mut ctx = setup().await;
    let organizer_authority = Keypair::new();
    let scanner = Keypair::new();
    let buyer = Keypair::new();
    let organizer_wallet = Keypair::new();
    let fee_vault = Keypair::new();

    for kp in [
        &organizer_authority,
        &scanner,
        &buyer,
        &organizer_wallet,
        &fee_vault,
    ] {
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
            "https://org/roles".to_string(),
            organizer_wallet.pubkey(),
        ),
        &[&organizer_authority],
    )
    .await;

    let event_id = 9101;
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
                title: "Governance Event".to_string(),
                venue: "Venue".to_string(),
                start_ts: 4_602_444_800,
                end_ts: 4_602_448_400,
                sales_start_ts: 4_602_430_000,
                lock_ts: 4_602_440_000,
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
                total_supply: 5,
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

    let (scanner_role, _) = role_binding_pda(organizer_profile, ROLE_SCANNER, scanner.pubkey());
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_grant_role(
            organizer_authority.pubkey(),
            ctx.payer.pubkey(),
            scanner.pubkey(),
            organizer_profile,
            protocol_config,
            organizer_profile,
            scanner_role,
            ROLE_SCANNER,
            ROLE_SCOPE_ORGANIZER,
            0,
            0,
        ),
        &[&organizer_authority],
    )
    .await;

    send_ix(
        &mut ctx.banks_client,
        &scanner,
        ctx.last_blockhash,
        ix_check_in_ticket(
            scanner.pubkey(),
            protocol_config,
            organizer_profile,
            event_account,
            ticket_class,
            ticket,
            scanner_role,
            class_id,
            ticket_id,
            "SCAN-1".to_string(),
        ),
        &[],
    )
    .await;

    let checked_ticket = fetch_ticket(&mut ctx.banks_client, ticket).await;
    assert_eq!(checked_ticket.check_in_count, 1);

    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        ix_revoke_role(
            organizer_authority.pubkey(),
            scanner.pubkey(),
            organizer_profile,
            protocol_config,
            organizer_profile,
            scanner_role,
            ROLE_SCANNER,
            ROLE_SCOPE_ORGANIZER,
            22,
        ),
        &[&organizer_authority],
    )
    .await;

    let ticket_id_2 = 2;
    let (ticket_2, _) = ticket_pda(event_account, class_id, ticket_id_2);
    let (counter_2, _) = wallet_purchase_counter_pda(event_account, ticket_class, buyer.pubkey());
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
            ticket_2,
            counter_2,
            fee_vault.pubkey(),
            organizer_wallet.pubkey(),
            Keypair::new().pubkey(),
            class_id,
            ticket_id_2,
            1_000_000_000,
        ),
        &[],
    )
    .await;

    let err = send_ix_result(
        &mut ctx.banks_client,
        &scanner,
        ctx.last_blockhash,
        ix_check_in_ticket(
            scanner.pubkey(),
            protocol_config,
            organizer_profile,
            event_account,
            ticket_class,
            ticket_2,
            scanner_role,
            class_id,
            ticket_id_2,
            "SCAN-2".to_string(),
        ),
        &[],
    )
    .await;
    assert!(err.is_err());
}
