use anchor_lang::prelude::*;

use crate::{
    constants::{SEED_EVENT, SEED_FINANCING_OFFER, SEED_ORGANIZER, SEED_PROTOCOL_CONFIG},
    error::TicketingError,
    events::FinancingOfferCreated,
    state::{
        EventAccount, FinancingLifecycleStatus, FinancingOffer, FinancingOfferInput,
        OrganizerProfile, ProtocolConfig,
    },
};

pub fn create_financing_offer(
    ctx: Context<CreateFinancingOffer>,
    input: FinancingOfferInput,
) -> Result<()> {
    require!(
        !ctx.accounts.protocol_config.is_paused,
        TicketingError::ProtocolPaused
    );
    validate_financing_input(&input)?;

    let authority = ctx.accounts.authority.key();

    let now = Clock::get()?.unix_timestamp;
    let offer = &mut ctx.accounts.financing_offer;
    if offer.created_at == 0 {
        offer.bump = ctx.bumps.financing_offer;
        offer.event = ctx.accounts.event_account.key();
        offer.organizer = ctx.accounts.organizer_profile.key();
        offer.created_at = now;
    } else {
        require!(!offer.terms_locked, TicketingError::FinancingTermsLocked);
        let is_admin = authority == ctx.accounts.protocol_config.admin;
        let is_organizer = authority == ctx.accounts.organizer_profile.authority;
        let is_authority = authority == offer.offer_authority;
        require!(
            is_admin || is_organizer || is_authority,
            TicketingError::Unauthorized
        );
    }

    offer.offer_authority = authority;
    offer.advance_amount_lamports = input.advance_amount_lamports;
    offer.fee_bps = input.fee_bps;
    offer.repayment_cap_lamports = input.repayment_cap_lamports;
    offer.schedule_start_ts = input.schedule_start_ts;
    offer.schedule_interval_secs = input.schedule_interval_secs;
    offer.schedule_installments = input.schedule_installments;
    offer.max_disbursements = input.schedule_installments;
    offer.status = FinancingLifecycleStatus::Draft;
    offer.terms_locked = false;
    offer.financing_frozen = false;
    offer.clawback_allowed = false;
    offer.freeze_reason_code = 0;
    offer.accepted_by = Pubkey::default();
    offer.accepted_at = 0;
    offer.rejected_by = Pubkey::default();
    offer.rejected_at = 0;
    offer.total_disbursed_lamports = 0;
    offer.disbursement_count = 0;
    offer.disbursed_at = 0;
    offer.updated_at = now;

    emit!(FinancingOfferCreated {
        event: ctx.accounts.event_account.key(),
        organizer: ctx.accounts.organizer_profile.key(),
        financing_offer: offer.key(),
        authority,
        advance_amount_lamports: input.advance_amount_lamports,
        fee_bps: input.fee_bps,
        repayment_cap_lamports: input.repayment_cap_lamports,
    });

    Ok(())
}

fn validate_financing_input(input: &FinancingOfferInput) -> Result<()> {
    require!(
        input.advance_amount_lamports > 0,
        TicketingError::InvalidFinancingAdvanceAmount
    );
    require!(input.fee_bps <= 10_000, TicketingError::InvalidFeeBps);
    require!(
        input.repayment_cap_lamports >= input.advance_amount_lamports,
        TicketingError::InvalidRepaymentCap
    );
    require!(
        input.schedule_interval_secs > 0 && input.schedule_installments > 0,
        TicketingError::InvalidFinancingSchedule
    );

    Ok(())
}

#[derive(Accounts)]
pub struct CreateFinancingOffer<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
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
        init_if_needed,
        payer = payer,
        space = 8 + FinancingOffer::INIT_SPACE,
        seeds = [SEED_FINANCING_OFFER, event_account.key().as_ref()],
        bump,
    )]
    pub financing_offer: Account<'info, FinancingOffer>,
    pub system_program: Program<'info, System>,
}
