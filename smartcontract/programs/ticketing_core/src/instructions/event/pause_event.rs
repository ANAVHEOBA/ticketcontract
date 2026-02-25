use anchor_lang::prelude::*;

use crate::{
    constants::SEED_ORGANIZER,
    error::TicketingError,
    events::EventStateTransitioned,
    state::{EventAccount, OrganizerProfile},
    utils::correlation::derive_correlation_id,
};

pub fn pause_event(ctx: Context<PauseEvent>, is_paused: bool) -> Result<()> {
    let event = &mut ctx.accounts.event_account;
    require!(
        event.organizer == ctx.accounts.organizer_profile.key(),
        TicketingError::Unauthorized
    );

    let old_status = event.status as u8;
    let new_status = event.status as u8;
    let now = Clock::get()?.unix_timestamp;
    event.is_paused = is_paused;
    event.updated_at = now;
    let correlation_id = derive_correlation_id(
        &event.key(),
        &ctx.accounts.authority.key(),
        now,
        0x1201,
    );
    emit!(EventStateTransitioned {
        event: event.key(),
        organizer: event.organizer,
        authority: ctx.accounts.authority.key(),
        old_status,
        new_status,
        is_paused: event.is_paused,
        correlation_id,
        at: now,
    });
    Ok(())
}

#[derive(Accounts)]
pub struct PauseEvent<'info> {
    pub authority: Signer<'info>,
    #[account(
        seeds = [SEED_ORGANIZER, authority.key().as_ref()],
        bump = organizer_profile.bump,
        constraint = organizer_profile.authority == authority.key() @ TicketingError::Unauthorized,
    )]
    pub organizer_profile: Account<'info, OrganizerProfile>,
    #[account(mut)]
    pub event_account: Account<'info, EventAccount>,
}
