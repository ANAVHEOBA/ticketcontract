use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer};

use crate::{
    constants::{SEED_EVENT, SEED_ORGANIZER, SEED_PROTOCOL_CONFIG, SEED_TICKET, SEED_TICKET_CLASS},
    error::TicketingError,
    events::TicketRefunded,
    state::{EventAccount, OrganizerProfile, ProtocolConfig, Ticket, TicketClass, TicketStatus},
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum RefundSource {
    OrganizerVault = 1,
    EscrowVault = 2,
    ReserveVault = 3,
}

impl RefundSource {
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            1 => Ok(Self::OrganizerVault),
            2 => Ok(Self::EscrowVault),
            3 => Ok(Self::ReserveVault),
            _ => err!(TicketingError::InvalidRefundSource),
        }
    }
}

pub fn refund_ticket(
    ctx: Context<RefundTicket>,
    _class_id: u16,
    _ticket_id: u32,
    amount_lamports: u64,
    source: u8,
) -> Result<()> {
    require!(
        !ctx.accounts.protocol_config.is_paused,
        TicketingError::ProtocolPaused
    );

    let authority = ctx.accounts.authority.key();
    let is_admin = authority == ctx.accounts.protocol_config.admin;
    let is_organizer = authority == ctx.accounts.organizer_profile.authority;
    require!(is_admin || is_organizer, TicketingError::Unauthorized);

    let ticket = &mut ctx.accounts.ticket;
    require!(
        ticket.status == TicketStatus::Active,
        TicketingError::RefundNotAllowed
    );
    require!(
        amount_lamports > 0 && amount_lamports <= ticket.paid_amount_lamports,
        TicketingError::InvalidRefundAmount
    );

    let refund_source = RefundSource::from_u8(source)?;
    let source_account = match refund_source {
        RefundSource::OrganizerVault => {
            require_keys_eq!(
                ctx.accounts.organizer_vault.key(),
                ctx.accounts.organizer_profile.payout_wallet,
                TicketingError::InvalidRefundSource
            );
            ctx.accounts.organizer_vault.to_account_info()
        }
        RefundSource::EscrowVault => {
            require_keys_eq!(
                ctx.accounts.escrow_vault.key(),
                ctx.accounts.protocol_config.treasury_vault,
                TicketingError::InvalidRefundSource
            );
            ctx.accounts.escrow_vault.to_account_info()
        }
        RefundSource::ReserveVault => {
            require_keys_eq!(
                ctx.accounts.reserve_vault.key(),
                ctx.accounts.protocol_config.fee_vault,
                TicketingError::InvalidRefundSource
            );
            ctx.accounts.reserve_vault.to_account_info()
        }
    };

    transfer_lamports(
        &ctx.accounts.system_program,
        &source_account,
        &ctx.accounts.refund_recipient.to_account_info(),
        amount_lamports,
    )?;

    let now = Clock::get()?.unix_timestamp;
    ticket.status = TicketStatus::Refunded;
    ticket.status_updated_at = now;
    if ticket.refunded_at == 0 {
        ticket.refunded_at = now;
        ctx.accounts.ticket_class.refunded_supply = ctx
            .accounts
            .ticket_class
            .refunded_supply
            .checked_add(1)
            .ok_or(TicketingError::MathOverflow)?;
    }
    ticket.owner = ticket.buyer;
    ticket.refund_source = source;
    ticket.refund_amount_lamports = amount_lamports;
    ticket.dispute_updated_at = now;

    emit!(TicketRefunded {
        event: ctx.accounts.event_account.key(),
        ticket_class: ctx.accounts.ticket_class.key(),
        ticket: ticket.key(),
        authority,
        recipient: ctx.accounts.refund_recipient.key(),
        amount_lamports,
        source,
        at: now,
    });

    Ok(())
}

fn transfer_lamports<'info>(
    system_program: &Program<'info, System>,
    from: &AccountInfo<'info>,
    to: &AccountInfo<'info>,
    amount: u64,
) -> Result<()> {
    if amount == 0 {
        return Ok(());
    }

    let cpi_accounts = Transfer {
        from: from.clone(),
        to: to.clone(),
    };
    let cpi_ctx = CpiContext::new(system_program.to_account_info(), cpi_accounts);
    system_program::transfer(cpi_ctx, amount)
}

#[derive(Accounts)]
#[instruction(class_id: u16, ticket_id: u32)]
pub struct RefundTicket<'info> {
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
        mut,
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
    #[account(mut, constraint = refund_recipient.key() == ticket.owner @ TicketingError::Unauthorized)]
    pub refund_recipient: SystemAccount<'info>,
    #[account(mut)]
    pub organizer_vault: Signer<'info>,
    #[account(mut)]
    pub escrow_vault: Signer<'info>,
    #[account(mut)]
    pub reserve_vault: Signer<'info>,
    pub system_program: Program<'info, System>,
}
