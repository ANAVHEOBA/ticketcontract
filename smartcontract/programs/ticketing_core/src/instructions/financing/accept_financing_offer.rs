use anchor_lang::prelude::*;

use crate::{
    constants::{SEED_EVENT, SEED_FINANCING_OFFER, SEED_ORGANIZER, SEED_PROTOCOL_CONFIG},
    error::TicketingError,
    events::FinancingOfferDecisioned,
    state::{
        EventAccount, FinancingLifecycleStatus, FinancingOffer, OrganizerProfile, ProtocolConfig,
    },
};

pub fn accept_financing_offer(ctx: Context<AcceptFinancingOffer>, accept: bool) -> Result<()> {
    require!(
        !ctx.accounts.protocol_config.is_paused,
        TicketingError::ProtocolPaused
    );

    let now = Clock::get()?.unix_timestamp;
    let authority = ctx.accounts.authority.key();
    let offer = &mut ctx.accounts.financing_offer;

    if accept {
        require!(!offer.terms_locked, TicketingError::FinancingTermsLocked);
        offer.status = FinancingLifecycleStatus::Accepted;
        offer.terms_locked = true;
        offer.accepted_by = authority;
        offer.accepted_at = now;
        offer.rejected_by = Pubkey::default();
        offer.rejected_at = 0;
    } else {
        require!(!offer.terms_locked, TicketingError::FinancingTermsLocked);
        offer.status = FinancingLifecycleStatus::Rejected;
        offer.terms_locked = false;
        offer.rejected_by = authority;
        offer.rejected_at = now;
    }
    offer.updated_at = now;

    emit!(FinancingOfferDecisioned {
        event: ctx.accounts.event_account.key(),
        organizer: ctx.accounts.organizer_profile.key(),
        financing_offer: offer.key(),
        authority,
        accepted: accept,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct AcceptFinancingOffer<'info> {
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
