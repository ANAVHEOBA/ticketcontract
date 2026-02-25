use anchor_lang::prelude::*;

use crate::{
    constants::SEED_ORGANIZER,
    error::TicketingError,
    state::{EventAccount, EventStatus, OrganizerProfile},
};

pub fn freeze_event(ctx: Context<FreezeEvent>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let event = &mut ctx.accounts.event_account;

    require!(
        event.status == EventStatus::Draft,
        TicketingError::InvalidEventStatusTransition
    );
    require!(
        now < event.sales_start_ts,
        TicketingError::SalesAlreadyStarted
    );

    event.status = EventStatus::Frozen;
    event.updated_at = now;

    Ok(())
}

#[derive(Accounts)]
pub struct FreezeEvent<'info> {
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
