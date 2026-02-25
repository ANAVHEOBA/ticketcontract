use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer};

use crate::{
    constants::{
        SEED_DISBURSEMENT_RECORD, SEED_EVENT, SEED_FINANCING_OFFER, SEED_ORGANIZER,
        SEED_PROTOCOL_CONFIG,
    },
    error::TicketingError,
    events::{FinancingAdvanceDisbursed, FinancingDisbursementRecorded},
    state::{
        DisbursementRecord, EventAccount, FinancingLifecycleStatus, FinancingOffer,
        OrganizerProfile, ProtocolConfig,
    },
    validation::invariants::assert_event_not_paused,
};

pub fn disburse_advance(
    ctx: Context<DisburseAdvance>,
    amount_lamports: u64,
    reference_id: [u8; 16],
) -> Result<()> {
    require!(
        !ctx.accounts.protocol_config.is_paused,
        TicketingError::ProtocolPaused
    );
    assert_event_not_paused(&ctx.accounts.event_account)?;

    let disburser = ctx.accounts.disburser.key();
    let offer = &mut ctx.accounts.financing_offer;
    let is_offer_authority = disburser == offer.offer_authority;
    let is_admin = disburser == ctx.accounts.protocol_config.admin;
    require!(is_offer_authority || is_admin, TicketingError::Unauthorized);
    require!(
        (offer.status == FinancingLifecycleStatus::Accepted
            || offer.status == FinancingLifecycleStatus::PartiallyDisbursed)
            && offer.terms_locked,
        TicketingError::FinancingOfferNotAccepted
    );
    require!(!offer.financing_frozen, TicketingError::FinancingFrozen);
    require!(
        !reference_id.iter().all(|b| *b == 0),
        TicketingError::InvalidDisbursementReference
    );
    require!(
        amount_lamports > 0,
        TicketingError::InvalidDisbursementAmount
    );
    require!(
        offer.disbursement_count < offer.max_disbursements,
        TicketingError::DisbursementLimitExceeded
    );

    let remaining = offer
        .advance_amount_lamports
        .checked_sub(offer.total_disbursed_lamports)
        .ok_or(TicketingError::MathOverflow)?;
    require!(
        amount_lamports <= remaining,
        TicketingError::InvalidDisbursementAmount
    );
    if offer.max_disbursements == 1 {
        require!(
            amount_lamports == offer.advance_amount_lamports,
            TicketingError::InvalidDisbursementAmount
        );
    }

    let next_index = offer
        .disbursement_count
        .checked_add(1)
        .ok_or(TicketingError::MathOverflow)?;
    transfer_lamports(
        &ctx.accounts.system_program,
        &ctx.accounts.disburser.to_account_info(),
        &ctx.accounts.organizer_payout_wallet.to_account_info(),
        amount_lamports,
    )?;

    let now = Clock::get()?.unix_timestamp;
    offer.total_disbursed_lamports = offer
        .total_disbursed_lamports
        .checked_add(amount_lamports)
        .ok_or(TicketingError::MathOverflow)?;
    offer.disbursement_count = next_index;
    if offer.disbursed_at == 0 {
        offer.disbursed_at = now;
    }
    offer.status = if offer.total_disbursed_lamports == offer.advance_amount_lamports {
        FinancingLifecycleStatus::Disbursed
    } else {
        FinancingLifecycleStatus::PartiallyDisbursed
    };
    offer.updated_at = now;

    let record = &mut ctx.accounts.disbursement_record;
    record.bump = ctx.bumps.disbursement_record;
    record.financing_offer = offer.key();
    record.disbursement_index = next_index;
    record.amount_lamports = amount_lamports;
    record.executed_by = disburser;
    record.destination_wallet = ctx.accounts.organizer_payout_wallet.key();
    record.reference_id = reference_id;
    record.executed_at = now;
    record.clawed_back = false;
    record.clawed_back_at = 0;

    emit!(FinancingAdvanceDisbursed {
        event: ctx.accounts.event_account.key(),
        organizer: ctx.accounts.organizer_profile.key(),
        financing_offer: offer.key(),
        disburser,
        organizer_payout_wallet: ctx.accounts.organizer_payout_wallet.key(),
        amount_lamports,
    });

    emit!(FinancingDisbursementRecorded {
        event: ctx.accounts.event_account.key(),
        organizer: ctx.accounts.organizer_profile.key(),
        financing_offer: offer.key(),
        disbursement_record: record.key(),
        disbursement_index: next_index,
        amount_lamports,
        reference_id,
        disburser,
        destination_wallet: ctx.accounts.organizer_payout_wallet.key(),
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
pub struct DisburseAdvance<'info> {
    #[account(mut)]
    pub disburser: Signer<'info>,
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
        seeds = [SEED_FINANCING_OFFER, event_account.key().as_ref()],
        bump = financing_offer.bump,
        constraint = financing_offer.event == event_account.key() @ TicketingError::Unauthorized,
        constraint = financing_offer.organizer == organizer_profile.key() @ TicketingError::Unauthorized,
    )]
    pub financing_offer: Account<'info, FinancingOffer>,
    #[account(
        init,
        payer = disburser,
        space = 8 + DisbursementRecord::INIT_SPACE,
        seeds = [
            SEED_DISBURSEMENT_RECORD,
            financing_offer.key().as_ref(),
            &(financing_offer.disbursement_count + 1).to_le_bytes(),
        ],
        bump,
    )]
    pub disbursement_record: Account<'info, DisbursementRecord>,
    #[account(mut, constraint = organizer_payout_wallet.key() == organizer_profile.payout_wallet @ TicketingError::Unauthorized)]
    pub organizer_payout_wallet: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}
