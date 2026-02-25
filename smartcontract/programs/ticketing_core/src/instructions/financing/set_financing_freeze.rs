use anchor_lang::prelude::*;

use crate::{
    constants::{SEED_EVENT, SEED_FINANCING_OFFER, SEED_ORGANIZER, SEED_PROTOCOL_CONFIG},
    error::TicketingError,
    events::FinancingFreezeUpdated,
    state::{EventAccount, FinancingOffer, OrganizerProfile, ProtocolConfig},
};

pub fn set_financing_freeze(
    ctx: Context<SetFinancingFreeze>,
    is_frozen: bool,
    reason_code: u16,
    clawback_allowed: bool,
) -> Result<()> {
    require!(
        !ctx.accounts.protocol_config.is_paused,
        TicketingError::ProtocolPaused
    );

    let admin = ctx.accounts.admin.key();
    let now = Clock::get()?.unix_timestamp;
    let offer = &mut ctx.accounts.financing_offer;

    offer.financing_frozen = is_frozen;
    offer.clawback_allowed = if is_frozen { clawback_allowed } else { false };
    offer.freeze_reason_code = if is_frozen { reason_code } else { 0 };
    offer.updated_at = now;

    emit!(FinancingFreezeUpdated {
        event: ctx.accounts.event_account.key(),
        organizer: ctx.accounts.organizer_profile.key(),
        financing_offer: offer.key(),
        admin,
        is_frozen,
        reason_code: offer.freeze_reason_code,
        clawback_allowed: offer.clawback_allowed,
        at: now,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct SetFinancingFreeze<'info> {
    pub admin: Signer<'info>,
    #[account(
        seeds = [SEED_PROTOCOL_CONFIG],
        bump = protocol_config.bump,
        constraint = protocol_config.admin == admin.key() @ TicketingError::Unauthorized,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,
    #[account(
        seeds = [SEED_ORGANIZER, organizer_profile.authority.as_ref()],
        bump = organizer_profile.bump,
    )]
    pub organizer_profile: Account<'info, OrganizerProfile>,
    #[account(
        seeds = [SEED_EVENT, organizer_profile.key().as_ref(), &event_account.event_id.to_le_bytes()],
        bump = event_account.bump,
        constraint = event_account.organizer == organizer_profile.key() @ TicketingError::Unauthorized,
    )]
    pub event_account: Account<'info, EventAccount>,
    #[account(
        mut,
        seeds = [SEED_FINANCING_OFFER, event_account.key().as_ref()],
        bump = financing_offer.bump,
        constraint = financing_offer.event == event_account.key() @ TicketingError::Unauthorized,
        constraint = financing_offer.organizer == organizer_profile.key() @ TicketingError::Unauthorized,
    )]
    pub financing_offer: Account<'info, FinancingOffer>,
}
