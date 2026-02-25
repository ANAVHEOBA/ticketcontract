use anchor_lang::prelude::*;

use crate::{
    constants::{
        SEED_EVENT, SEED_LISTING, SEED_ORGANIZER, SEED_PROTOCOL_CONFIG, SEED_RESALE_POLICY,
        SEED_TICKET, SEED_TICKET_CLASS,
    },
    error::TicketingError,
    events::TicketListed,
    state::{
        EventAccount, Listing, OrganizerProfile, ProtocolConfig, ResalePolicy, Ticket, TicketClass,
        TicketStatus,
    },
};

pub fn list_ticket(
    ctx: Context<ListTicket>,
    _class_id: u16,
    _ticket_id: u32,
    price_lamports: u64,
    expires_at: i64,
) -> Result<()> {
    require!(
        !ctx.accounts.protocol_config.is_paused,
        TicketingError::ProtocolPaused
    );
    require!(
        ctx.accounts.ticket_class.is_resale_enabled,
        TicketingError::ResaleDisabled
    );
    require!(
        ctx.accounts.ticket_class.is_transferable,
        TicketingError::TransferNotAllowed
    );
    require!(price_lamports > 0, TicketingError::InvalidResalePrice);
    let now = Clock::get()?.unix_timestamp;
    require!(expires_at > now, TicketingError::InvalidListingExpiry);

    let ticket = &ctx.accounts.ticket;
    require!(
        ticket.owner == ctx.accounts.seller.key(),
        TicketingError::Unauthorized
    );
    require!(
        ticket.status == TicketStatus::Active,
        TicketingError::IllegalTicketStatusTransition
    );
    require!(
        !ticket.is_disputed && !ticket.is_chargeback,
        TicketingError::TicketDisputed
    );

    let max_allowed = (u128::from(ctx.accounts.ticket_class.face_price_lamports)
        * u128::from(10_000u16 + ctx.accounts.resale_policy.max_markup_bps)
        / 10_000u128) as u64;
    require!(
        price_lamports <= max_allowed,
        TicketingError::ListingExceedsMaxMarkup
    );

    let listing = &mut ctx.accounts.listing;
    if listing.created_at == 0 {
        listing.bump = ctx.bumps.listing;
        listing.event = ctx.accounts.event_account.key();
        listing.ticket_class = ctx.accounts.ticket_class.key();
        listing.ticket = ticket.key();
        listing.created_at = now;
    }
    require!(!listing.is_active, TicketingError::ListingNotActive);

    listing.seller = ctx.accounts.seller.key();
    listing.price_lamports = price_lamports;
    listing.expires_at = expires_at;
    listing.is_active = true;
    listing.close_reason = 0;
    listing.closed_at = 0;
    listing.updated_at = now;

    emit!(TicketListed {
        event: listing.event,
        ticket_class: listing.ticket_class,
        ticket: listing.ticket,
        listing: listing.key(),
        seller: listing.seller,
        price_lamports,
        expires_at,
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(class_id: u16, ticket_id: u32)]
pub struct ListTicket<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,
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
        seeds = [SEED_RESALE_POLICY, event_account.key().as_ref(), &class_id.to_le_bytes()],
        bump = resale_policy.bump,
        constraint = resale_policy.ticket_class == ticket_class.key() @ TicketingError::Unauthorized,
    )]
    pub resale_policy: Account<'info, ResalePolicy>,
    #[account(
        mut,
        seeds = [SEED_TICKET, event_account.key().as_ref(), &class_id.to_le_bytes(), &ticket_id.to_le_bytes()],
        bump = ticket.bump,
        constraint = ticket.event == event_account.key() @ TicketingError::Unauthorized,
        constraint = ticket.ticket_class == ticket_class.key() @ TicketingError::Unauthorized,
        constraint = ticket.ticket_id == ticket_id @ TicketingError::InvalidTicketId,
    )]
    pub ticket: Account<'info, Ticket>,
    #[account(
        init_if_needed,
        payer = seller,
        space = 8 + Listing::INIT_SPACE,
        seeds = [SEED_LISTING, ticket.key().as_ref()],
        bump,
    )]
    pub listing: Account<'info, Listing>,
    pub system_program: Program<'info, System>,
}
