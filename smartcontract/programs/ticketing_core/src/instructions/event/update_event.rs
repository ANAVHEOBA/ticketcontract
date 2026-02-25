use anchor_lang::prelude::*;

use crate::{
    constants::SEED_ORGANIZER,
    error::TicketingError,
    state::{EventAccount, EventStatus, OrganizerProfile},
};

use super::create_event::{validate_event_input, EventInput};

pub fn update_event(ctx: Context<UpdateEvent>, input: EventInput) -> Result<()> {
    validate_event_input(&input)?;

    let now = Clock::get()?.unix_timestamp;
    let event = &mut ctx.accounts.event_account;

    require!(
        event.status == EventStatus::Draft,
        TicketingError::InvalidEventStatusTransition
    );
    require!(now < event.lock_ts, TicketingError::EventUpdateLocked);

    event.title = input.title;
    event.venue = input.venue;
    event.start_ts = input.start_ts;
    event.end_ts = input.end_ts;
    event.sales_start_ts = input.sales_start_ts;
    event.lock_ts = input.lock_ts;
    event.capacity = input.capacity;
    event.updated_at = now;

    Ok(())
}

#[derive(Accounts)]
pub struct UpdateEvent<'info> {
    pub authority: Signer<'info>,
    #[account(
        seeds = [SEED_ORGANIZER, authority.key().as_ref()],
        bump = organizer_profile.bump,
        constraint = organizer_profile.authority == authority.key() @ TicketingError::Unauthorized,
    )]
    pub organizer_profile: Account<'info, OrganizerProfile>,
    #[account(
        mut,
        constraint = event_account.organizer == organizer_profile.key() @ TicketingError::Unauthorized,
    )]
    pub event_account: Account<'info, EventAccount>,
}
