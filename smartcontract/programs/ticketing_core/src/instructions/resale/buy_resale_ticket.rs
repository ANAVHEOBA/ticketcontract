use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer};

use crate::{
    constants::{
        SEED_EVENT, SEED_LISTING, SEED_ORGANIZER, SEED_PROTOCOL_CONFIG, SEED_RESALE_POLICY,
        SEED_TICKET, SEED_TICKET_CLASS,
    },
    error::TicketingError,
    events::{ResaleSettlement, TicketResold},
    state::{
        EventAccount, Listing, OrganizerProfile, ProtocolConfig, ResalePolicy, Ticket, TicketClass,
        TicketStatus,
    },
};

pub fn buy_resale_ticket(
    ctx: Context<BuyResaleTicket>,
    _class_id: u16,
    _ticket_id: u32,
    max_price_lamports: u64,
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

    let listing = &mut ctx.accounts.listing;
    let ticket = &mut ctx.accounts.ticket;
    let policy = &ctx.accounts.resale_policy;
    let now = Clock::get()?.unix_timestamp;

    require!(listing.is_active, TicketingError::ListingNotActive);
    require!(now <= listing.expires_at, TicketingError::ListingExpired);
    require!(
        ticket.status == TicketStatus::Active,
        TicketingError::IllegalTicketStatusTransition
    );
    require!(
        !ticket.is_disputed && !ticket.is_chargeback,
        TicketingError::TicketDisputed
    );
    require!(
        listing.seller != ctx.accounts.buyer.key(),
        TicketingError::Unauthorized
    );
    require!(
        listing.price_lamports <= max_price_lamports,
        TicketingError::InvalidResalePrice
    );

    let max_allowed = (u128::from(ctx.accounts.ticket_class.face_price_lamports)
        * u128::from(10_000u16 + policy.max_markup_bps)
        / 10_000u128) as u64;
    require!(
        listing.price_lamports <= max_allowed,
        TicketingError::ListingExceedsMaxMarkup
    );

    if !policy.whitelist.is_empty() {
        require!(
            policy
                .whitelist
                .iter()
                .any(|k| k == &ctx.accounts.buyer.key()),
            TicketingError::RecipientNotWhitelisted
        );
    }
    require!(
        !policy
            .blacklist
            .iter()
            .any(|k| k == &ctx.accounts.buyer.key()),
        TicketingError::RecipientBlacklisted
    );

    if policy.transfer_cooldown_secs > 0 && ticket.last_transfer_at > 0 {
        let min_next = ticket
            .last_transfer_at
            .checked_add(policy.transfer_cooldown_secs)
            .ok_or(TicketingError::MathOverflow)?;
        require!(now >= min_next, TicketingError::TransferCooldownActive);
    }

    if policy.max_transfer_count > 0 {
        require!(
            ticket.transfer_count < policy.max_transfer_count,
            TicketingError::MaxTransferCountExceeded
        );
    }

    if policy.transfer_lock_before_event_secs > 0 {
        let lock_start = ctx
            .accounts
            .event_account
            .start_ts
            .checked_sub(policy.transfer_lock_before_event_secs)
            .ok_or(TicketingError::MathOverflow)?;
        require!(now < lock_start, TicketingError::TransferLockedByEventStart);
    }

    let price = listing.price_lamports;
    let royalty = (u128::from(price) * u128::from(policy.royalty_bps) / 10_000u128) as u64;
    let seller_amount = price
        .checked_sub(royalty)
        .ok_or(TicketingError::MathOverflow)?;

    transfer_lamports(
        &ctx.accounts.system_program,
        &ctx.accounts.buyer.to_account_info(),
        &ctx.accounts.seller_wallet.to_account_info(),
        seller_amount,
    )?;
    transfer_lamports(
        &ctx.accounts.system_program,
        &ctx.accounts.buyer.to_account_info(),
        &ctx.accounts.royalty_vault.to_account_info(),
        royalty,
    )?;

    ticket.owner = ctx.accounts.buyer.key();
    ticket.transfer_count = ticket
        .transfer_count
        .checked_add(1)
        .ok_or(TicketingError::MathOverflow)?;
    ticket.last_transfer_at = now;
    ticket.status_updated_at = now;

    listing.is_active = false;
    listing.close_reason = 2;
    listing.closed_at = now;
    listing.updated_at = now;

    emit!(TicketResold {
        event: ctx.accounts.event_account.key(),
        ticket_class: ctx.accounts.ticket_class.key(),
        ticket: ticket.key(),
        listing: listing.key(),
        seller: listing.seller,
        buyer: ctx.accounts.buyer.key(),
        price_lamports: price,
        royalty_amount: royalty,
        seller_amount,
    });

    emit!(ResaleSettlement {
        event: ctx.accounts.event_account.key(),
        ticket_class: ctx.accounts.ticket_class.key(),
        ticket: ticket.key(),
        listing: listing.key(),
        price_lamports: price,
        seller_amount,
        royalty_amount: royalty,
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
pub struct BuyResaleTicket<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(
        seeds = [SEED_PROTOCOL_CONFIG],
        bump = protocol_config.bump,
    )]
    pub protocol_config: Box<Account<'info, ProtocolConfig>>,
    #[account(
        seeds = [SEED_ORGANIZER, organizer_profile.authority.as_ref()],
        bump = organizer_profile.bump,
    )]
    pub organizer_profile: Box<Account<'info, OrganizerProfile>>,
    #[account(
        seeds = [SEED_EVENT, organizer_profile.key().as_ref(), &event_account.event_id.to_le_bytes()],
        bump = event_account.bump,
        constraint = event_account.organizer == organizer_profile.key() @ TicketingError::Unauthorized,
    )]
    pub event_account: Box<Account<'info, EventAccount>>,
    #[account(
        seeds = [SEED_TICKET_CLASS, event_account.key().as_ref(), &class_id.to_le_bytes()],
        bump = ticket_class.bump,
        constraint = ticket_class.event == event_account.key() @ TicketingError::Unauthorized,
        constraint = ticket_class.class_id == class_id @ TicketingError::Unauthorized,
    )]
    pub ticket_class: Box<Account<'info, TicketClass>>,
    #[account(
        seeds = [SEED_RESALE_POLICY, event_account.key().as_ref(), &class_id.to_le_bytes()],
        bump = resale_policy.bump,
        constraint = resale_policy.ticket_class == ticket_class.key() @ TicketingError::Unauthorized,
    )]
    pub resale_policy: Box<Account<'info, ResalePolicy>>,
    #[account(
        mut,
        seeds = [SEED_TICKET, event_account.key().as_ref(), &class_id.to_le_bytes(), &ticket_id.to_le_bytes()],
        bump = ticket.bump,
        constraint = ticket.event == event_account.key() @ TicketingError::Unauthorized,
        constraint = ticket.ticket_class == ticket_class.key() @ TicketingError::Unauthorized,
        constraint = ticket.ticket_id == ticket_id @ TicketingError::InvalidTicketId,
    )]
    pub ticket: Box<Account<'info, Ticket>>,
    #[account(
        mut,
        seeds = [SEED_LISTING, ticket.key().as_ref()],
        bump = listing.bump,
        constraint = listing.ticket == ticket.key() @ TicketingError::Unauthorized,
    )]
    pub listing: Box<Account<'info, Listing>>,
    #[account(mut, constraint = seller_wallet.key() == listing.seller @ TicketingError::Unauthorized)]
    pub seller_wallet: SystemAccount<'info>,
    #[account(mut, constraint = royalty_vault.key() == resale_policy.royalty_vault @ TicketingError::Unauthorized)]
    pub royalty_vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}
