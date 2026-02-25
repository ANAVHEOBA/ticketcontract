use anchor_lang::prelude::*;

use crate::{
    constants::SEED_ORGANIZER,
    error::TicketingError,
    events::EventStateTransitioned,
    state::{EventAccount, EventStatus, OrganizerProfile},
    utils::correlation::derive_correlation_id,
};

pub fn cancel_event(ctx: Context<CancelEvent>) -> Result<()> {
    let event = &mut ctx.accounts.event_account;
    let old_status = event.status as u8;
    require!(
        event.status == EventStatus::Draft || event.status == EventStatus::Frozen,
        TicketingError::InvalidEventStatusTransition
    );

    let now = Clock::get()?.unix_timestamp;
    event.status = EventStatus::Cancelled;
    event.updated_at = now;
    let correlation_id = derive_correlation_id(
        &event.key(),
        &ctx.accounts.authority.key(),
        now,
        event.status as u16,
    );
    emit!(EventStateTransitioned {
        event: event.key(),
        organizer: event.organizer,
        authority: ctx.accounts.authority.key(),
        old_status,
        new_status: event.status as u8,
        is_paused: event.is_paused,
        correlation_id,
        at: now,
    });

    Ok(())
}

pub fn close_event(ctx: Context<CloseEvent>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let event = &mut ctx.accounts.event_account;
    let old_status = event.status as u8;

    let can_close = event.status == EventStatus::Cancelled || now >= event.end_ts;
    require!(can_close, TicketingError::InvalidEventStatusTransition);

    event.status = EventStatus::Closed;
    event.updated_at = now;
    let correlation_id = derive_correlation_id(
        &event.key(),
        &ctx.accounts.authority.key(),
        now,
        event.status as u16,
    );
    emit!(EventStateTransitioned {
        event: event.key(),
        organizer: event.organizer,
        authority: ctx.accounts.authority.key(),
        old_status,
        new_status: event.status as u8,
        is_paused: event.is_paused,
        correlation_id,
        at: now,
    });

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
