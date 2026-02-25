use anchor_lang::{prelude::*, Discriminator};

use crate::{
    constants::{
        MAX_GATE_IDENTIFIER_LEN, OPERATOR_PERMISSION_CHECKIN, SEED_EVENT, SEED_ORGANIZER,
        SEED_ORGANIZER_OPERATOR, SEED_PROTOCOL_CONFIG, SEED_TICKET, SEED_TICKET_CLASS,
    },
    error::TicketingError,
    events::{CheckInPolicyUpdated, TicketAttendanceRecorded},
    state::{
        EventAccount, EventStatus, OrganizerOperator, OrganizerProfile, ProtocolConfig, Ticket,
        TicketClass, TicketStatus,
    },
};

pub fn set_checkin_policy(
    ctx: Context<SetCheckInPolicy>,
    allow_reentry: bool,
    max_reentries: u8,
) -> Result<()> {
    require!(
        !ctx.accounts.protocol_config.is_paused,
        TicketingError::ProtocolPaused
    );
    require!(
        ctx.accounts.event_account.status == EventStatus::Draft
            || ctx.accounts.event_account.status == EventStatus::Frozen,
        TicketingError::InvalidEventStatusTransition
    );
    require!(
        allow_reentry || max_reentries == 0,
        TicketingError::ReentryNotAllowed
    );

    let now = Clock::get()?.unix_timestamp;
    let ticket_class = &mut ctx.accounts.ticket_class;
    ticket_class.allow_reentry = allow_reentry;
    ticket_class.max_reentries = max_reentries;
    ticket_class.updated_at = now;

    emit!(CheckInPolicyUpdated {
        event: ctx.accounts.event_account.key(),
        ticket_class: ticket_class.key(),
        authority: ctx.accounts.authority.key(),
        allow_reentry,
        max_reentries,
        at: now,
    });

    Ok(())
}

pub fn check_in_ticket(
    ctx: Context<CheckInTicket>,
    _class_id: u16,
    _ticket_id: u32,
    gate_identifier: String,
) -> Result<()> {
    require!(
        !ctx.accounts.protocol_config.is_paused,
        TicketingError::ProtocolPaused
    );
    require!(
        ctx.accounts.event_account.status == EventStatus::Draft
            || ctx.accounts.event_account.status == EventStatus::Frozen,
        TicketingError::InvalidEventStatusTransition
    );
    require!(
        !gate_identifier.is_empty() && gate_identifier.len() <= MAX_GATE_IDENTIFIER_LEN,
        TicketingError::InvalidGateIdentifier
    );
    require!(
        is_authorized_scanner(
            &ctx.accounts.scanner.to_account_info(),
            &ctx.accounts.organizer_profile,
            &ctx.accounts.organizer_operator,
        )?,
        TicketingError::Unauthorized
    );

    let now = Clock::get()?.unix_timestamp;
    let ticket_class = &ctx.accounts.ticket_class;
    let ticket = &mut ctx.accounts.ticket;
    require!(
        ticket.status != TicketStatus::Refunded && ticket.status != TicketStatus::Invalidated,
        TicketingError::IllegalTicketStatusTransition
    );

    let is_reentry = ticket.check_in_count > 0;
    if is_reentry {
        require!(
            ticket_class.allow_reentry,
            TicketingError::TicketAlreadyCheckedIn
        );
        let reentries_used = ticket
            .check_in_count
            .checked_sub(1)
            .ok_or(TicketingError::MathOverflow)?;
        require!(
            reentries_used < u16::from(ticket_class.max_reentries),
            TicketingError::ReentryLimitExceeded
        );
    }

    ticket.status = TicketStatus::CheckedIn;
    ticket.status_updated_at = now;
    if ticket.checked_in_at == 0 {
        ticket.checked_in_at = now;
    }
    ticket.last_check_in_at = now;
    ticket.last_check_in_gate_id = gate_identifier.clone();
    ticket.check_in_count = ticket
        .check_in_count
        .checked_add(1)
        .ok_or(TicketingError::MathOverflow)?;

    emit!(TicketAttendanceRecorded {
        event: ctx.accounts.event_account.key(),
        ticket_class: ticket_class.key(),
        ticket: ticket.key(),
        owner: ticket.owner,
        scanner: ctx.accounts.scanner.key(),
        gate_identifier,
        check_in_count: ticket.check_in_count,
        is_reentry,
        at: now,
    });

    Ok(())
}

fn is_authorized_scanner(
    scanner: &AccountInfo<'_>,
    organizer_profile: &Account<'_, OrganizerProfile>,
    organizer_operator: &AccountInfo<'_>,
) -> Result<bool> {
    if scanner.key() == organizer_profile.authority {
        return Ok(true);
    }

    let (expected_operator, _) = Pubkey::find_program_address(
        &[
            SEED_ORGANIZER_OPERATOR,
            organizer_profile.key().as_ref(),
            scanner.key().as_ref(),
        ],
        &crate::ID,
    );
    require_keys_eq!(
        organizer_operator.key(),
        expected_operator,
        TicketingError::Unauthorized
    );

    let data = organizer_operator.try_borrow_data()?;
    let mut data_slice: &[u8] = &data;
    if data_slice.len() < 8 || &data_slice[..8] != OrganizerOperator::DISCRIMINATOR {
        return Ok(false);
    }
    let operator = OrganizerOperator::try_deserialize(&mut data_slice)?;
    let has_permission = (operator.permissions & OPERATOR_PERMISSION_CHECKIN) != 0;

    Ok(operator.active && operator.operator == scanner.key() && has_permission)
}

#[derive(Accounts)]
#[instruction(class_id: u16)]
pub struct SetCheckInPolicy<'info> {
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
        seeds = [SEED_TICKET_CLASS, event_account.key().as_ref(), &class_id.to_le_bytes()],
        bump = ticket_class.bump,
        constraint = ticket_class.event == event_account.key() @ TicketingError::Unauthorized,
        constraint = ticket_class.class_id == class_id @ TicketingError::Unauthorized,
    )]
    pub ticket_class: Account<'info, TicketClass>,
}

#[derive(Accounts)]
#[instruction(class_id: u16, ticket_id: u32)]
pub struct CheckInTicket<'info> {
    pub scanner: Signer<'info>,
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
    /// CHECK: runtime-validated PDA and deserialized only when scanner != organizer authority.
    pub organizer_operator: UncheckedAccount<'info>,
}
