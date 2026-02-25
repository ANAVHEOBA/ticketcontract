use anchor_lang::prelude::*;

use crate::{
    constants::{
        MAX_EVENT_TITLE_LEN, MAX_EVENT_VENUE_LEN, SEED_EVENT, SEED_ORGANIZER, SEED_PROTOCOL_CONFIG,
    },
    error::TicketingError,
    state::{EventAccount, EventStatus, OrganizerProfile, OrganizerStatus, ProtocolConfig},
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct EventInput {
    pub title: String,
    pub venue: String,
    pub start_ts: i64,
    pub end_ts: i64,
    pub sales_start_ts: i64,
    pub lock_ts: i64,
    pub capacity: u32,
}

pub fn create_event(ctx: Context<CreateEvent>, event_id: u64, input: EventInput) -> Result<()> {
    validate_event_input(&input)?;

    require!(
        !ctx.accounts.protocol_config.is_paused,
        TicketingError::ProtocolPaused
    );
    require!(
        ctx.accounts.organizer_profile.status == OrganizerStatus::Active,
        TicketingError::OrganizerSuspended
    );

    let now = Clock::get()?.unix_timestamp;
    let event = &mut ctx.accounts.event_account;
    event.bump = ctx.bumps.event_account;
    event.organizer = ctx.accounts.organizer_profile.key();
    event.event_id = event_id;
    event.title = input.title;
    event.venue = input.venue;
    event.start_ts = input.start_ts;
    event.end_ts = input.end_ts;
    event.sales_start_ts = input.sales_start_ts;
    event.lock_ts = input.lock_ts;
    event.capacity = input.capacity;
    event.loyalty_multiplier_bps = 10_000;
    event.status = EventStatus::Draft;
    event.created_at = now;
    event.updated_at = now;

    Ok(())
}

pub fn validate_event_input(input: &EventInput) -> Result<()> {
    require!(
        input.title.len() <= MAX_EVENT_TITLE_LEN,
        TicketingError::InvalidEventTitleLength
    );
    require!(
        input.venue.len() <= MAX_EVENT_VENUE_LEN,
        TicketingError::InvalidEventVenueLength
    );
    require!(input.capacity > 0, TicketingError::InvalidEventCapacity);
    require!(
        input.start_ts < input.end_ts
            && input.sales_start_ts <= input.start_ts
            && input.lock_ts <= input.start_ts,
        TicketingError::InvalidEventTimeWindow
    );

    Ok(())
}

#[derive(Accounts)]
#[instruction(event_id: u64)]
pub struct CreateEvent<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub authority: Signer<'info>,
    #[account(
        seeds = [SEED_PROTOCOL_CONFIG],
        bump = protocol_config.bump,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,
    #[account(
        seeds = [SEED_ORGANIZER, authority.key().as_ref()],
        bump = organizer_profile.bump,
        constraint = organizer_profile.authority == authority.key() @ TicketingError::Unauthorized,
    )]
    pub organizer_profile: Account<'info, OrganizerProfile>,
    #[account(
        init,
        payer = payer,
        space = 8 + EventAccount::INIT_SPACE,
        seeds = [SEED_EVENT, organizer_profile.key().as_ref(), &event_id.to_le_bytes()],
        bump,
    )]
    pub event_account: Account<'info, EventAccount>,
    pub system_program: Program<'info, System>,
}
