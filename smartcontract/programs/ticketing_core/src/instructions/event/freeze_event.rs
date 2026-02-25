use anchor_lang::prelude::*;

use crate::{
    constants::SEED_ORGANIZER,
    error::TicketingError,
    events::EventStateTransitioned,
    state::{EventAccount, EventStatus, OrganizerProfile},
    utils::correlation::derive_correlation_id,
};

pub fn freeze_event(ctx: Context<FreezeEvent>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let event = &mut ctx.accounts.event_account;
    let old_status = event.status as u8;

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
