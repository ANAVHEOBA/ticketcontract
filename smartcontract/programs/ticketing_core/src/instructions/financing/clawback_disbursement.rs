use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer};

use crate::{
    constants::{
        SEED_DISBURSEMENT_RECORD, SEED_EVENT, SEED_FINANCING_OFFER, SEED_ORGANIZER,
        SEED_PROTOCOL_CONFIG,
    },
    error::TicketingError,
    events::FinancingClawbackExecuted,
    state::{
        DisbursementRecord, EventAccount, FinancingLifecycleStatus, FinancingOffer,
        OrganizerProfile, ProtocolConfig,
    },
};

pub fn clawback_disbursement(
    ctx: Context<ClawbackDisbursement>,
    disbursement_index: u16,
) -> Result<()> {
    require!(
        !ctx.accounts.protocol_config.is_paused,
        TicketingError::ProtocolPaused
    );

    let offer = &mut ctx.accounts.financing_offer;
    require!(offer.financing_frozen, TicketingError::FinancingFrozen);
    require!(offer.clawback_allowed, TicketingError::ClawbackNotAllowed);

    let record = &mut ctx.accounts.disbursement_record;
    require!(
        record.disbursement_index == disbursement_index,
        TicketingError::Unauthorized
    );
    require!(
        !record.clawed_back,
        TicketingError::DisbursementAlreadyClawedBack
    );

    transfer_lamports(
        &ctx.accounts.system_program,
        &ctx.accounts.organizer_payout_wallet.to_account_info(),
        &ctx.accounts.treasury_vault.to_account_info(),
        record.amount_lamports,
    )?;

    let now = Clock::get()?.unix_timestamp;
    record.clawed_back = true;
    record.clawed_back_at = now;

    offer.total_disbursed_lamports = offer
        .total_disbursed_lamports
        .checked_sub(record.amount_lamports)
        .ok_or(TicketingError::MathOverflow)?;
    offer.status = if offer.total_disbursed_lamports == 0 {
        FinancingLifecycleStatus::Accepted
    } else if offer.total_disbursed_lamports < offer.advance_amount_lamports {
        FinancingLifecycleStatus::PartiallyDisbursed
    } else {
        FinancingLifecycleStatus::Disbursed
    };
    offer.updated_at = now;

    emit!(FinancingClawbackExecuted {
        event: ctx.accounts.event_account.key(),
        organizer: ctx.accounts.organizer_profile.key(),
        financing_offer: offer.key(),
        disbursement_record: record.key(),
        admin: ctx.accounts.admin.key(),
        treasury_vault: ctx.accounts.treasury_vault.key(),
        amount_lamports: record.amount_lamports,
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
#[instruction(disbursement_index: u16)]
pub struct ClawbackDisbursement<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        seeds = [SEED_PROTOCOL_CONFIG],
        bump = protocol_config.bump,
        constraint = protocol_config.admin == admin.key() @ TicketingError::Unauthorized,
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
        seeds = [SEED_FINANCING_OFFER, event_account.key().as_ref()],
        bump = financing_offer.bump,
        constraint = financing_offer.event == event_account.key() @ TicketingError::Unauthorized,
        constraint = financing_offer.organizer == organizer_profile.key() @ TicketingError::Unauthorized,
    )]
    pub financing_offer: Account<'info, FinancingOffer>,
    #[account(
        mut,
        seeds = [SEED_DISBURSEMENT_RECORD, financing_offer.key().as_ref(), &disbursement_index.to_le_bytes()],
        bump = disbursement_record.bump,
        constraint = disbursement_record.financing_offer == financing_offer.key() @ TicketingError::Unauthorized,
    )]
    pub disbursement_record: Account<'info, DisbursementRecord>,
    #[account(mut, address = organizer_profile.payout_wallet @ TicketingError::Unauthorized)]
    pub organizer_payout_wallet: Signer<'info>,
    #[account(mut, constraint = treasury_vault.key() == protocol_config.treasury_vault @ TicketingError::Unauthorized)]
    pub treasury_vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}
