use anchor_lang::prelude::*;

use crate::{
    constants::{
        MAX_TICKET_METADATA_URI_LEN, SEED_EVENT, SEED_ORGANIZER, SEED_PROTOCOL_CONFIG, SEED_TICKET,
        SEED_TICKET_CLASS,
    },
    error::TicketingError,
    events::TicketMetadataUpdated,
    state::{EventAccount, OrganizerProfile, ProtocolConfig, Ticket, TicketClass},
};

pub fn set_ticket_metadata(
    ctx: Context<SetTicketMetadata>,
    _class_id: u16,
    _ticket_id: u32,
    metadata_uri: String,
    metadata_version: u16,
) -> Result<()> {
    require!(
        !ctx.accounts.protocol_config.is_paused,
        TicketingError::ProtocolPaused
    );
    require!(
        metadata_uri.len() <= MAX_TICKET_METADATA_URI_LEN,
        TicketingError::InvalidTicketMetadataUriLength
    );

    let authority = ctx.accounts.authority.key();
    let is_admin = authority == ctx.accounts.protocol_config.admin;
    let is_organizer = authority == ctx.accounts.organizer_profile.authority;
    require!(is_admin || is_organizer, TicketingError::Unauthorized);

    let now = Clock::get()?.unix_timestamp;
    let ticket = &mut ctx.accounts.ticket;
    ticket.metadata_uri = metadata_uri.clone();
    ticket.metadata_version = metadata_version;
    ticket.metadata_updated_at = now;

    emit!(TicketMetadataUpdated {
        event: ctx.accounts.event_account.key(),
        ticket_class: ctx.accounts.ticket_class.key(),
        ticket: ticket.key(),
        metadata_uri,
        metadata_version,
        authority,
        at: now,
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(class_id: u16, ticket_id: u32)]
pub struct SetTicketMetadata<'info> {
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
        mut,
        seeds = [SEED_TICKET, event_account.key().as_ref(), &class_id.to_le_bytes(), &ticket_id.to_le_bytes()],
        bump = ticket.bump,
        constraint = ticket.event == event_account.key() @ TicketingError::Unauthorized,
        constraint = ticket.ticket_class == ticket_class.key() @ TicketingError::Unauthorized,
        constraint = ticket.ticket_id == ticket_id @ TicketingError::InvalidTicketId,
    )]
    pub ticket: Account<'info, Ticket>,
}
