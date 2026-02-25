use anchor_lang::prelude::*;

use crate::{
    constants::{SEED_EVENT, SEED_ORGANIZER, SEED_TICKET_CLASS},
    error::TicketingError,
    state::{EventAccount, EventStatus, OrganizerProfile, TicketClass},
};

use super::create_ticket_class::{validate_ticket_class_input, TicketClassInput};

pub fn update_ticket_class(
    ctx: Context<UpdateTicketClass>,
    _class_id: u16,
    input: TicketClassInput,
) -> Result<()> {
    validate_ticket_class_input(&input)?;

    require!(
        ctx.accounts.event_account.status == EventStatus::Draft
            || ctx.accounts.event_account.status == EventStatus::Frozen,
        TicketingError::InvalidEventStatusTransition
    );

    let now = Clock::get()?.unix_timestamp;
    let ticket_class = &mut ctx.accounts.ticket_class;

    require!(
        now < ticket_class.sale_start_ts,
        TicketingError::SalesAlreadyStarted
    );

    let reserved_plus_sold = input
        .reserved_supply
        .checked_add(ticket_class.sold_supply)
        .ok_or(TicketingError::MathOverflow)?;
    require!(
        reserved_plus_sold <= input.total_supply,
        TicketingError::InvalidTicketClassSupply
    );

    let remaining_supply = input
        .total_supply
        .checked_sub(reserved_plus_sold)
        .ok_or(TicketingError::MathOverflow)?;

    ticket_class.name = input.name;
    ticket_class.total_supply = input.total_supply;
    ticket_class.reserved_supply = input.reserved_supply;
    ticket_class.remaining_supply = remaining_supply;
    ticket_class.face_price_lamports = input.face_price_lamports;
    ticket_class.sale_start_ts = input.sale_start_ts;
    ticket_class.sale_end_ts = input.sale_end_ts;
    ticket_class.per_wallet_limit = input.per_wallet_limit;
    ticket_class.is_transferable = input.is_transferable;
    ticket_class.is_resale_enabled = input.is_resale_enabled;
    ticket_class.stakeholder_wallet = input.stakeholder_wallet;
    ticket_class.stakeholder_bps = input.stakeholder_bps;
    ticket_class.updated_at = now;

    Ok(())
}

#[derive(Accounts)]
#[instruction(class_id: u16)]
pub struct UpdateTicketClass<'info> {
    pub authority: Signer<'info>,
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
        seeds = [SEED_TICKET_CLASS, event_account.key().as_ref(), &class_id.to_le_bytes()],
        bump = ticket_class.bump,
        constraint = ticket_class.event == event_account.key() @ TicketingError::Unauthorized,
        constraint = ticket_class.class_id == class_id @ TicketingError::Unauthorized,
    )]
    pub ticket_class: Account<'info, TicketClass>,
}
