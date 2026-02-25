use anchor_lang::prelude::*;

use crate::{
    constants::{
        SEED_EVENT, SEED_ORGANIZER, SEED_PROTOCOL_CONFIG, SEED_TICKET, SEED_TICKET_CLASS,
        SEED_TRUST_SIGNAL,
    },
    error::TicketingError,
    events::TrustSignalUpdated,
    state::{EventAccount, OrganizerProfile, ProtocolConfig, Ticket, TicketClass, TrustSignal},
};

pub fn record_purchase_input(
    ctx: Context<RecordPurchaseInput>,
    _class_id: u16,
    _ticket_id: u32,
) -> Result<()> {
    require!(
        !ctx.accounts.protocol_config.is_paused,
        TicketingError::ProtocolPaused
    );

    let ticket = &mut ctx.accounts.ticket;
    require!(
        !ticket.purchase_trust_recorded,
        TicketingError::TrustSignalAlreadyRecorded
    );

    let now = Clock::get()?.unix_timestamp;
    let signal = &mut ctx.accounts.trust_signal;
    if signal.wallet == Pubkey::default() {
        signal.bump = ctx.bumps.trust_signal;
        signal.wallet = ctx.accounts.wallet.key();
        signal.schema_version = 1;
        signal.total_tickets_purchased = 0;
        signal.attendance_eligible_count = 0;
        signal.attendance_attended_count = 0;
        signal.abuse_flags = 0;
        signal.abuse_incidents = 0;
        signal.last_event = Pubkey::default();
        signal.last_ticket = Pubkey::default();
        signal.created_at = now;
    }
    require!(
        signal.last_ticket != ticket.key(),
        TicketingError::TrustSignalAlreadyRecorded
    );

    signal.total_tickets_purchased = signal
        .total_tickets_purchased
        .checked_add(1)
        .ok_or(TicketingError::MathOverflow)?;
    signal.last_event = ctx.accounts.event_account.key();
    signal.last_ticket = ticket.key();
    signal.updated_at = now;

    ticket.purchase_trust_recorded = true;

    emit!(TrustSignalUpdated {
        wallet: signal.wallet,
        trust_signal: signal.key(),
        event: ctx.accounts.event_account.key(),
        ticket: ticket.key(),
        schema_version: signal.schema_version,
        update_type: 1,
        total_tickets_purchased: signal.total_tickets_purchased,
        attendance_eligible_count: signal.attendance_eligible_count,
        attendance_attended_count: signal.attendance_attended_count,
        abuse_flags: signal.abuse_flags,
        abuse_incidents: signal.abuse_incidents,
        at: now,
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(class_id: u16, ticket_id: u32)]
pub struct RecordPurchaseInput<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub authority: Signer<'info>,
    pub wallet: SystemAccount<'info>,
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
        constraint = ticket.buyer == wallet.key() @ TicketingError::Unauthorized,
    )]
    pub ticket: Account<'info, Ticket>,
    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + TrustSignal::INIT_SPACE,
        seeds = [SEED_TRUST_SIGNAL, wallet.key().as_ref()],
        bump,
    )]
    pub trust_signal: Account<'info, TrustSignal>,
    pub system_program: Program<'info, System>,
}
