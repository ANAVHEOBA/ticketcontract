use anchor_lang::prelude::*;

use crate::{
    constants::{SEED_EVENT, SEED_ORGANIZER, SEED_PROTOCOL_CONFIG, SEED_TICKET, SEED_TICKET_CLASS},
    error::TicketingError,
    events::TicketDisputeFlagged,
    state::{EventAccount, OrganizerProfile, ProtocolConfig, Ticket, TicketClass, TicketStatus},
};

pub fn flag_dispute(
    ctx: Context<FlagDispute>,
    _class_id: u16,
    _ticket_id: u32,
    is_disputed: bool,
    is_chargeback: bool,
    reason_code: u16,
) -> Result<()> {
    require!(
        !ctx.accounts.protocol_config.is_paused,
        TicketingError::ProtocolPaused
    );
    require!(
        !is_chargeback || is_disputed,
        TicketingError::ChargebackRequiresDispute
    );

    let authority = ctx.accounts.authority.key();
    let is_admin = authority == ctx.accounts.protocol_config.admin;
    let is_organizer = authority == ctx.accounts.organizer_profile.authority;
    require!(is_admin || is_organizer, TicketingError::Unauthorized);

    let now = Clock::get()?.unix_timestamp;
    let ticket = &mut ctx.accounts.ticket;
    ticket.is_disputed = is_disputed;
    ticket.is_chargeback = is_chargeback;
    if is_disputed && ticket.disputed_at == 0 {
        ticket.disputed_at = now;
    }
    if !is_disputed {
        ticket.disputed_at = 0;
    }
    ticket.dispute_reason_code = reason_code;
    ticket.dispute_updated_at = now;

    if is_chargeback && ticket.status == TicketStatus::Active {
        ticket.status = TicketStatus::Invalidated;
        ticket.status_updated_at = now;
        if ticket.invalidated_at == 0 {
            ticket.invalidated_at = now;
        }
    }

    emit!(TicketDisputeFlagged {
        event: ctx.accounts.event_account.key(),
        ticket_class: ctx.accounts.ticket_class.key(),
        ticket: ticket.key(),
        authority,
        is_disputed,
        is_chargeback,
        reason_code,
        at: now,
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(class_id: u16, ticket_id: u32)]
pub struct FlagDispute<'info> {
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
