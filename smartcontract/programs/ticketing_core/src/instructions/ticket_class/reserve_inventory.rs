use anchor_lang::prelude::*;

use crate::{
    constants::{SEED_EVENT, SEED_ORGANIZER, SEED_TICKET_CLASS},
    error::TicketingError,
    state::{EventAccount, EventStatus, OrganizerProfile, TicketClass},
};

pub fn reserve_inventory(
    ctx: Context<ReserveInventory>,
    _class_id: u16,
    amount: u32,
) -> Result<()> {
    require!(amount > 0, TicketingError::InvalidReserveAmount);
    require!(
        ctx.accounts.event_account.status == EventStatus::Draft
            || ctx.accounts.event_account.status == EventStatus::Frozen,
        TicketingError::InvalidEventStatusTransition
    );

    let ticket_class = &mut ctx.accounts.ticket_class;

    let new_reserved = ticket_class
        .reserved_supply
        .checked_add(amount)
        .ok_or(TicketingError::MathOverflow)?;

    let used_supply = new_reserved
        .checked_add(ticket_class.sold_supply)
        .ok_or(TicketingError::MathOverflow)?;

    require!(
        used_supply <= ticket_class.total_supply,
        TicketingError::InsufficientRemainingSupply
    );

    ticket_class.reserved_supply = new_reserved;
    ticket_class.remaining_supply = ticket_class
        .total_supply
        .checked_sub(used_supply)
        .ok_or(TicketingError::MathOverflow)?;
    ticket_class.updated_at = Clock::get()?.unix_timestamp;

    Ok(())
}

#[derive(Accounts)]
#[instruction(class_id: u16)]
pub struct ReserveInventory<'info> {
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
