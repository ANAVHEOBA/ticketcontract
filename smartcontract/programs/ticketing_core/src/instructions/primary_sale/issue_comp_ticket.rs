use anchor_lang::prelude::*;

use crate::{
    constants::{
        SEED_EVENT, SEED_ORGANIZER, SEED_PROTOCOL_CONFIG, SEED_TICKET, SEED_TICKET_CLASS,
        TICKET_ACCOUNT_SCHEMA_VERSION,
    },
    error::TicketingError,
    events::TicketCompIssued,
    state::{
        EventAccount, EventStatus, OrganizerProfile, ProtocolConfig, Ticket, TicketClass,
        TicketStatus,
    },
    validation::invariants::assert_event_not_paused,
};

pub fn issue_comp_ticket(
    ctx: Context<IssueCompTicket>,
    class_id: u16,
    ticket_id: u32,
) -> Result<()> {
    let _ = class_id;
    require!(
        !ctx.accounts.protocol_config.is_paused,
        TicketingError::ProtocolPaused
    );
    assert_event_not_paused(&ctx.accounts.event_account)?;
    require!(
        ctx.accounts.event_account.status == EventStatus::Draft
            || ctx.accounts.event_account.status == EventStatus::Frozen,
        TicketingError::InvalidEventStatusTransition
    );

    let issuer = ctx.accounts.issuer.key();
    let is_admin = issuer == ctx.accounts.protocol_config.admin;
    let is_organizer = issuer == ctx.accounts.organizer_profile.authority;
    require!(is_admin || is_organizer, TicketingError::Unauthorized);

    let ticket_class = &mut ctx.accounts.ticket_class;
    require!(
        ticket_class.remaining_supply > 0,
        TicketingError::InsufficientRemainingSupply
    );
    require!(
        ticket_id == ticket_class.sold_supply + 1,
        TicketingError::InvalidTicketId
    );

    let now = Clock::get()?.unix_timestamp;

    ticket_class.sold_supply = ticket_class
        .sold_supply
        .checked_add(1)
        .ok_or(TicketingError::MathOverflow)?;
    ticket_class.remaining_supply = ticket_class
        .remaining_supply
        .checked_sub(1)
        .ok_or(TicketingError::MathOverflow)?;
    ticket_class.updated_at = now;

    let ticket = &mut ctx.accounts.ticket;
    ticket.bump = ctx.bumps.ticket;
    ticket.schema_version = TICKET_ACCOUNT_SCHEMA_VERSION;
    ticket.deprecated_layout_version = 0;
    ticket.replacement_account = Pubkey::default();
    ticket.deprecated_at = 0;
    ticket.event = ctx.accounts.event_account.key();
    ticket.ticket_class = ticket_class.key();
    ticket.owner = ctx.accounts.recipient.key();
    ticket.buyer = ctx.accounts.recipient.key();
    ticket.ticket_id = ticket_id;
    ticket.status = TicketStatus::Active;
    ticket.paid_amount_lamports = 0;
    ticket.is_comp = true;
    ticket.created_at = now;
    ticket.status_updated_at = now;
    ticket.checked_in_at = 0;
    ticket.last_check_in_at = 0;
    ticket.check_in_count = 0;
    ticket.last_check_in_gate_id = String::new();
    ticket.refunded_at = 0;
    ticket.refund_source = 0;
    ticket.refund_amount_lamports = 0;
    ticket.invalidated_at = 0;
    ticket.is_disputed = false;
    ticket.is_chargeback = false;
    ticket.disputed_at = 0;
    ticket.dispute_reason_code = 0;
    ticket.dispute_updated_at = 0;
    ticket.metadata_uri = String::new();
    ticket.metadata_version = 0;
    ticket.metadata_updated_at = 0;
    ticket.transfer_count = 0;
    ticket.last_transfer_at = 0;
    ticket.compliance_decision_code = 0;
    ticket.compliance_checked_at = 0;
    ticket.purchase_trust_recorded = false;
    ticket.attendance_trust_recorded = false;

    emit!(TicketCompIssued {
        event: ctx.accounts.event_account.key(),
        ticket_class: ticket_class.key(),
        ticket: ticket.key(),
        recipient: ctx.accounts.recipient.key(),
        issuer,
        ticket_id,
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(class_id: u16, ticket_id: u32)]
pub struct IssueCompTicket<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub issuer: Signer<'info>,
    pub recipient: SystemAccount<'info>,
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
        mut,
        seeds = [SEED_TICKET_CLASS, event_account.key().as_ref(), &class_id.to_le_bytes()],
        bump = ticket_class.bump,
        constraint = ticket_class.event == event_account.key() @ TicketingError::Unauthorized,
        constraint = ticket_class.class_id == class_id @ TicketingError::Unauthorized,
    )]
    pub ticket_class: Account<'info, TicketClass>,
    #[account(
        init,
        payer = payer,
        space = 8 + Ticket::INIT_SPACE,
        seeds = [SEED_TICKET, event_account.key().as_ref(), &class_id.to_le_bytes(), &ticket_id.to_le_bytes()],
        bump,
    )]
    pub ticket: Account<'info, Ticket>,
    pub system_program: Program<'info, System>,
}
