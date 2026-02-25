use anchor_lang::prelude::*;

use crate::{
    constants::SEED_ORGANIZER,
    error::TicketingError,
    state::{EventAccount, EventStatus, OrganizerProfile},
};

pub fn cancel_event(ctx: Context<CancelEvent>) -> Result<()> {
    let event = &mut ctx.accounts.event_account;
    require!(
        event.status == EventStatus::Draft || event.status == EventStatus::Frozen,
        TicketingError::InvalidEventStatusTransition
    );

    event.status = EventStatus::Cancelled;
    event.updated_at = Clock::get()?.unix_timestamp;

    Ok(())
}

pub fn close_event(ctx: Context<CloseEvent>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let event = &mut ctx.accounts.event_account;

    let can_close = event.status == EventStatus::Cancelled || now >= event.end_ts;
    require!(can_close, TicketingError::InvalidEventStatusTransition);

    event.status = EventStatus::Closed;
    event.updated_at = now;

    Ok(())
}

#[derive(Accounts)]
pub struct CancelEvent<'info> {
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

#[derive(Accounts)]
pub struct CloseEvent<'info> {
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
