use anchor_lang::prelude::*;

use crate::{
    constants::{SEED_EVENT, SEED_ORGANIZER, SEED_PROTOCOL_CONFIG},
    error::TicketingError,
    events::EventComplianceFlagsUpdated,
    state::{EventAccount, OrganizerProfile, ProtocolConfig},
};

pub fn set_event_restrictions(
    ctx: Context<SetEventRestrictions>,
    restriction_flags: u32,
    decision_code: u16,
) -> Result<()> {
    let authority = ctx.accounts.authority.key();
    let organizer_authority = ctx.accounts.organizer_profile.authority;
    let protocol_admin = ctx.accounts.protocol_config.admin;
    require!(
        authority == organizer_authority || authority == protocol_admin,
        TicketingError::Unauthorized
    );

    let now = Clock::get()?.unix_timestamp;
    ctx.accounts.event_account.compliance_restriction_flags = restriction_flags;
    ctx.accounts.event_account.updated_at = now;

    let _ = decision_code;
    emit!(EventComplianceFlagsUpdated {
        event: ctx.accounts.event_account.key(),
        organizer: ctx.accounts.organizer_profile.key(),
        authority,
        restriction_flags,
        at: now,
    });
    Ok(())
}

#[derive(Accounts)]
pub struct SetEventRestrictions<'info> {
    pub authority: Signer<'info>,
    #[account(
        seeds = [SEED_PROTOCOL_CONFIG],
        bump = protocol_config.bump,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,
    #[account(
        seeds = [SEED_ORGANIZER, organizer_profile.authority.as_ref()],
        bump = organizer_profile.bump,
    )]
    pub organizer_profile: Account<'info, OrganizerProfile>,
    #[account(
        mut,
        seeds = [SEED_EVENT, organizer_profile.key().as_ref(), &event_account.event_id.to_le_bytes()],
        bump = event_account.bump,
        constraint = event_account.organizer == organizer_profile.key() @ TicketingError::Unauthorized,
    )]
    pub event_account: Account<'info, EventAccount>,
}
