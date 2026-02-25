use solana_sdk::signature::{Keypair, Signer};

use ticketing_core::{instructions::event::EventInput, state::EventStatus};

use crate::common::{
    event_pda, fetch_event_account, ix_cancel_event, ix_close_event, ix_create_event,
    ix_create_organizer, ix_freeze_event, ix_initialize_protocol, ix_update_event, organizer_pda,
    protocol_config_pda, send_ix, send_ix_result, setup,
};

async fn setup_organizer(
    ctx: &mut solana_program_test::ProgramTestContext,
    authority: &Keypair,
) -> (solana_sdk::pubkey::Pubkey, solana_sdk::pubkey::Pubkey) {
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

    let (organizer_profile, _) = organizer_pda(authority.pubkey());
    let create_org_ix = ix_create_organizer(
        ctx.payer.pubkey(),
        authority.pubkey(),
        protocol_config,
        organizer_profile,
        "https://org/events".to_string(),
        authority.pubkey(),
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        create_org_ix,
        &[authority],
    )
    .await;

    (protocol_config, organizer_profile)
}

#[tokio::test]
async fn create_event_stores_event_metadata() {
    let mut ctx = setup().await;
    let authority = Keypair::new();
    let (protocol_config, organizer_profile) = setup_organizer(&mut ctx, &authority).await;

    let event_id = 1;
    let (event_account, _) = event_pda(organizer_profile, event_id);
    let input = EventInput {
        title: "Dev Summit".to_string(),
        venue: "Hall A".to_string(),
        start_ts: 4_102_444_800,
        end_ts: 4_102_448_400,
        sales_start_ts: 4_102_430_000,
        lock_ts: 4_102_440_000,
        capacity: 1500,
    };

    let create_event_ix = ix_create_event(
        ctx.payer.pubkey(),
        authority.pubkey(),
        protocol_config,
        organizer_profile,
        event_account,
        event_id,
        input.clone(),
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        create_event_ix,
        &[&authority],
    )
    .await;

    let event = fetch_event_account(&mut ctx.banks_client, event_account).await;
    assert_eq!(event.organizer, organizer_profile);
    assert_eq!(event.event_id, event_id);
    assert_eq!(event.title, input.title);
    assert_eq!(event.venue, input.venue);
    assert_eq!(event.start_ts, input.start_ts);
    assert_eq!(event.end_ts, input.end_ts);
    assert_eq!(event.sales_start_ts, input.sales_start_ts);
    assert_eq!(event.lock_ts, input.lock_ts);
    assert_eq!(event.capacity, input.capacity);
    assert_eq!(event.status, EventStatus::Draft);
}

#[tokio::test]
async fn update_event_respects_lock_time() {
    let mut ctx = setup().await;
    let authority = Keypair::new();
    let (protocol_config, organizer_profile) = setup_organizer(&mut ctx, &authority).await;

    let event_id = 2;
    let (event_account, _) = event_pda(organizer_profile, event_id);
    let base = EventInput {
        title: "Base".to_string(),
        venue: "V1".to_string(),
        start_ts: 4_102_444_800,
        end_ts: 4_102_448_400,
        sales_start_ts: 4_102_430_000,
        lock_ts: 4_102_440_000,
        capacity: 100,
    };
    let create_ix = ix_create_event(
        ctx.payer.pubkey(),
        authority.pubkey(),
        protocol_config,
        organizer_profile,
        event_account,
        event_id,
        base,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        create_ix,
        &[&authority],
    )
    .await;

    let update_ok = EventInput {
        title: "Updated".to_string(),
        venue: "V2".to_string(),
        start_ts: 4_102_444_900,
        end_ts: 4_102_448_900,
        sales_start_ts: 4_102_430_100,
        lock_ts: 4_102_440_100,
        capacity: 120,
    };
    let update_ix = ix_update_event(
        authority.pubkey(),
        organizer_profile,
        event_account,
        update_ok.clone(),
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        update_ix,
        &[&authority],
    )
    .await;

    let updated = fetch_event_account(&mut ctx.banks_client, event_account).await;
    assert_eq!(updated.title, update_ok.title);

    let locked_event_id = 3;
    let (locked_event, _) = event_pda(organizer_profile, locked_event_id);
    let locked_input = EventInput {
        title: "Locked".to_string(),
        venue: "V3".to_string(),
        start_ts: 200,
        end_ts: 300,
        sales_start_ts: 100,
        lock_ts: 1,
        capacity: 50,
    };
    let create_locked_ix = ix_create_event(
        ctx.payer.pubkey(),
        authority.pubkey(),
        protocol_config,
        organizer_profile,
        locked_event,
        locked_event_id,
        locked_input,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        create_locked_ix,
        &[&authority],
    )
    .await;

    let locked_update_ix = ix_update_event(
        authority.pubkey(),
        organizer_profile,
        locked_event,
        update_ok,
    );
    let err = send_ix_result(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        locked_update_ix,
        &[&authority],
    )
    .await;
    assert!(err.is_err());
}

#[tokio::test]
async fn freeze_cancel_close_event_transitions() {
    let mut ctx = setup().await;
    let authority = Keypair::new();
    let (protocol_config, organizer_profile) = setup_organizer(&mut ctx, &authority).await;

    let event_id = 4;
    let (event_account, _) = event_pda(organizer_profile, event_id);
    let input = EventInput {
        title: "Flow".to_string(),
        venue: "Arena".to_string(),
        start_ts: 4_102_444_800,
        end_ts: 4_102_448_400,
        sales_start_ts: 4_102_430_000,
        lock_ts: 4_102_440_000,
        capacity: 999,
    };
    let create_ix = ix_create_event(
        ctx.payer.pubkey(),
        authority.pubkey(),
        protocol_config,
        organizer_profile,
        event_account,
        event_id,
        input,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        create_ix,
        &[&authority],
    )
    .await;

    let freeze_ix = ix_freeze_event(authority.pubkey(), organizer_profile, event_account);
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        freeze_ix,
        &[&authority],
    )
    .await;

    let frozen = fetch_event_account(&mut ctx.banks_client, event_account).await;
    assert_eq!(frozen.status, EventStatus::Frozen);

    let cancel_ix = ix_cancel_event(authority.pubkey(), organizer_profile, event_account);
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        cancel_ix,
        &[&authority],
    )
    .await;

    let cancelled = fetch_event_account(&mut ctx.banks_client, event_account).await;
    assert_eq!(cancelled.status, EventStatus::Cancelled);

    let close_ix = ix_close_event(authority.pubkey(), organizer_profile, event_account);
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        close_ix,
        &[&authority],
    )
    .await;

    let closed = fetch_event_account(&mut ctx.banks_client, event_account).await;
    assert_eq!(closed.status, EventStatus::Closed);
}

#[tokio::test]
async fn event_transition_guards_enforced() {
    let mut ctx = setup().await;
    let authority = Keypair::new();
    let (protocol_config, organizer_profile) = setup_organizer(&mut ctx, &authority).await;

    let started_event_id = 5;
    let (started_event, _) = event_pda(organizer_profile, started_event_id);
    let started_input = EventInput {
        title: "Started".to_string(),
        venue: "Past".to_string(),
        start_ts: 100,
        end_ts: 200,
        sales_start_ts: 1,
        lock_ts: 1,
        capacity: 50,
    };
    let create_started_ix = ix_create_event(
        ctx.payer.pubkey(),
        authority.pubkey(),
        protocol_config,
        organizer_profile,
        started_event,
        started_event_id,
        started_input,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        create_started_ix,
        &[&authority],
    )
    .await;

    let freeze_started_ix = ix_freeze_event(authority.pubkey(), organizer_profile, started_event);
    let err = send_ix_result(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        freeze_started_ix,
        &[&authority],
    )
    .await;
    assert!(err.is_err());

    let future_event_id = 6;
    let (future_event, _) = event_pda(organizer_profile, future_event_id);
    let future_input = EventInput {
        title: "Future".to_string(),
        venue: "Main".to_string(),
        start_ts: 4_102_444_800,
        end_ts: 4_102_448_400,
        sales_start_ts: 4_102_430_000,
        lock_ts: 4_102_440_000,
        capacity: 500,
    };
    let create_future_ix = ix_create_event(
        ctx.payer.pubkey(),
        authority.pubkey(),
        protocol_config,
        organizer_profile,
        future_event,
        future_event_id,
        future_input,
    );
    send_ix(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        create_future_ix,
        &[&authority],
    )
    .await;

    let close_future_ix = ix_close_event(authority.pubkey(), organizer_profile, future_event);
    let err = send_ix_result(
        &mut ctx.banks_client,
        &ctx.payer,
        ctx.last_blockhash,
        close_future_ix,
        &[&authority],
    )
    .await;
    assert!(err.is_err());
}
