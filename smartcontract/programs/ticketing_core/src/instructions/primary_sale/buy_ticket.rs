use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer};

use crate::{
    constants::{
        SEED_EVENT, SEED_ORGANIZER, SEED_PROTOCOL_CONFIG, SEED_TICKET, SEED_TICKET_CLASS,
        SEED_WALLET_PURCHASE_COUNTER,
    },
    error::TicketingError,
    events::TicketPurchased,
    state::{
        EventAccount, EventStatus, OrganizerProfile, ProtocolConfig, Ticket, TicketClass,
        TicketStatus, WalletPurchaseCounter,
    },
};

pub fn buy_ticket(
    ctx: Context<BuyTicket>,
    _class_id: u16,
    ticket_id: u32,
    expected_price_lamports: u64,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let protocol_config = &ctx.accounts.protocol_config;
    let ticket_class = &mut ctx.accounts.ticket_class;

    require!(!protocol_config.is_paused, TicketingError::ProtocolPaused);
    require!(
        ctx.accounts.event_account.status == EventStatus::Draft
            || ctx.accounts.event_account.status == EventStatus::Frozen,
        TicketingError::InvalidEventStatusTransition
    );
    require!(
        now >= ticket_class.sale_start_ts && now <= ticket_class.sale_end_ts,
        TicketingError::SaleWindowNotActive
    );
    require!(
        ticket_class.remaining_supply > 0,
        TicketingError::InsufficientRemainingSupply
    );
    require!(
        expected_price_lamports == ticket_class.face_price_lamports,
        TicketingError::InvalidTicketPrice
    );
    require!(
        ticket_id == ticket_class.sold_supply + 1,
        TicketingError::InvalidTicketId
    );

    let counter = &mut ctx.accounts.wallet_purchase_counter;
    if counter.wallet == Pubkey::default() {
        counter.bump = ctx.bumps.wallet_purchase_counter;
        counter.event = ctx.accounts.event_account.key();
        counter.ticket_class = ticket_class.key();
        counter.wallet = ctx.accounts.buyer.key();
        counter.purchased_count = 0;
    }

    require!(
        counter.purchased_count < ticket_class.per_wallet_limit,
        TicketingError::PurchaseLimitExceeded
    );

    let total = expected_price_lamports;
    let protocol_fee =
        (u128::from(total) * u128::from(protocol_config.protocol_fee_bps) / 10_000u128) as u64;
    let stakeholder_fee =
        (u128::from(total) * u128::from(ticket_class.stakeholder_bps) / 10_000u128) as u64;

    if ticket_class.stakeholder_bps > 0 {
        require!(
            ctx.accounts.stakeholder_wallet.key() == ticket_class.stakeholder_wallet,
            TicketingError::InvalidStakeholderWallet
        );
    }

    let fees = protocol_fee
        .checked_add(stakeholder_fee)
        .ok_or(TicketingError::MathOverflow)?;
    require!(fees <= total, TicketingError::MathOverflow);

    let organizer_amount = total
        .checked_sub(fees)
        .ok_or(TicketingError::MathOverflow)?;

    transfer_lamports(
        &ctx.accounts.system_program,
        &ctx.accounts.buyer.to_account_info(),
        &ctx.accounts.protocol_fee_vault.to_account_info(),
        protocol_fee,
    )?;

    if stakeholder_fee > 0 {
        transfer_lamports(
            &ctx.accounts.system_program,
            &ctx.accounts.buyer.to_account_info(),
            &ctx.accounts.stakeholder_wallet.to_account_info(),
            stakeholder_fee,
        )?;
    }

    transfer_lamports(
        &ctx.accounts.system_program,
        &ctx.accounts.buyer.to_account_info(),
        &ctx.accounts.organizer_payout_wallet.to_account_info(),
        organizer_amount,
    )?;

    ticket_class.sold_supply = ticket_class
        .sold_supply
        .checked_add(1)
        .ok_or(TicketingError::MathOverflow)?;
    ticket_class.remaining_supply = ticket_class
        .remaining_supply
        .checked_sub(1)
        .ok_or(TicketingError::MathOverflow)?;
    ticket_class.updated_at = now;

    counter.purchased_count = counter
        .purchased_count
        .checked_add(1)
        .ok_or(TicketingError::MathOverflow)?;
    counter.updated_at = now;

    let ticket = &mut ctx.accounts.ticket;
    ticket.bump = ctx.bumps.ticket;
    ticket.event = ctx.accounts.event_account.key();
    ticket.ticket_class = ticket_class.key();
    ticket.owner = ctx.accounts.buyer.key();
    ticket.buyer = ctx.accounts.buyer.key();
    ticket.ticket_id = ticket_id;
    ticket.status = TicketStatus::Active;
    ticket.paid_amount_lamports = total;
    ticket.is_comp = false;
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
    ticket.purchase_trust_recorded = false;
    ticket.attendance_trust_recorded = false;

    emit!(TicketPurchased {
        event: ctx.accounts.event_account.key(),
        ticket_class: ticket_class.key(),
        ticket: ticket.key(),
        buyer: ctx.accounts.buyer.key(),
        owner: ctx.accounts.buyer.key(),
        ticket_id,
        amount_paid: total,
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
pub struct BuyTicket<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
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
        payer = buyer,
        space = 8 + Ticket::INIT_SPACE,
        seeds = [SEED_TICKET, event_account.key().as_ref(), &class_id.to_le_bytes(), &ticket_id.to_le_bytes()],
        bump,
    )]
    pub ticket: Account<'info, Ticket>,
    #[account(
        init_if_needed,
        payer = buyer,
        space = 8 + WalletPurchaseCounter::INIT_SPACE,
        seeds = [SEED_WALLET_PURCHASE_COUNTER, event_account.key().as_ref(), ticket_class.key().as_ref(), buyer.key().as_ref()],
        bump,
    )]
    pub wallet_purchase_counter: Account<'info, WalletPurchaseCounter>,
    #[account(mut, constraint = protocol_fee_vault.key() == protocol_config.fee_vault @ TicketingError::Unauthorized)]
    pub protocol_fee_vault: SystemAccount<'info>,
    #[account(mut, constraint = organizer_payout_wallet.key() == organizer_profile.payout_wallet @ TicketingError::Unauthorized)]
    pub organizer_payout_wallet: SystemAccount<'info>,
    /// CHECK: validated at runtime against `ticket_class.stakeholder_wallet` when `stakeholder_bps > 0`.
    #[account(mut)]
    pub stakeholder_wallet: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}
