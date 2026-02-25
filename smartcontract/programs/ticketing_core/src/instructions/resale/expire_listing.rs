use anchor_lang::prelude::*;

use crate::{
    constants::{
        SEED_EVENT, SEED_LISTING, SEED_ORGANIZER, SEED_PROTOCOL_CONFIG, SEED_TICKET,
        SEED_TICKET_CLASS,
    },
    error::TicketingError,
    events::ListingExpired,
    state::{EventAccount, Listing, OrganizerProfile, ProtocolConfig, Ticket, TicketClass},
};

pub fn expire_listing(ctx: Context<ExpireListing>, _class_id: u16, _ticket_id: u32) -> Result<()> {
    require!(
        !ctx.accounts.protocol_config.is_paused,
        TicketingError::ProtocolPaused
    );

    let now = Clock::get()?.unix_timestamp;
    let listing = &mut ctx.accounts.listing;
    require!(listing.is_active, TicketingError::ListingNotActive);
    require!(
        now > listing.expires_at,
        TicketingError::ListingNotYetExpired
    );

    listing.is_active = false;
    listing.close_reason = 3;
    listing.closed_at = now;
    listing.updated_at = now;

    emit!(ListingExpired {
        event: ctx.accounts.event_account.key(),
        ticket_class: ctx.accounts.ticket_class.key(),
        ticket: ctx.accounts.ticket.key(),
        listing: listing.key(),
        seller: listing.seller,
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(class_id: u16, ticket_id: u32)]
pub struct ExpireListing<'info> {
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
        seeds = [SEED_EVENT, organizer_profile.key().as_ref(), &event_account.event_id.to_le_bytes()],
        bump = event_account.bump,
        constraint = event_account.organizer == organizer_profile.key() @ TicketingError::Unauthorized,
    )]
    pub event_account: Account<'info, EventAccount>,
    #[account(
        seeds = [SEED_TICKET_CLASS, event_account.key().as_ref(), &class_id.to_le_bytes()],
        bump = ticket_class.bump,
        constraint = ticket_class.event == event_account.key() @ TicketingError::Unauthorized,
        constraint = ticket_class.class_id == class_id @ TicketingError::Unauthorized,
    )]
    pub ticket_class: Account<'info, TicketClass>,
    #[account(
        seeds = [SEED_TICKET, event_account.key().as_ref(), &class_id.to_le_bytes(), &ticket_id.to_le_bytes()],
        bump = ticket.bump,
        constraint = ticket.event == event_account.key() @ TicketingError::Unauthorized,
        constraint = ticket.ticket_class == ticket_class.key() @ TicketingError::Unauthorized,
        constraint = ticket.ticket_id == ticket_id @ TicketingError::InvalidTicketId,
    )]
    pub ticket: Account<'info, Ticket>,
    #[account(
        mut,
        seeds = [SEED_LISTING, ticket.key().as_ref()],
        bump = listing.bump,
        constraint = listing.ticket == ticket.key() @ TicketingError::Unauthorized,
    )]
    pub listing: Account<'info, Listing>,
}
