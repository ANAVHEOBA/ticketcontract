use anchor_lang::prelude::*;

use crate::{
    constants::{
        SEED_EVENT, SEED_FINANCING_OFFER, SEED_ORGANIZER, SEED_PROTOCOL_CONFIG,
        SEED_SETTLEMENT_LEDGER,
    },
    error::TicketingError,
    events::FinancingSettled,
    state::{
        EventAccount, FinancingLifecycleStatus, FinancingOffer, OrganizerProfile, ProtocolConfig,
        SettlementLedger,
    },
};

pub fn finalize_settlement(ctx: Context<FinalizeSettlement>) -> Result<()> {
    require!(
        !ctx.accounts.protocol_config.is_paused,
        TicketingError::ProtocolPaused
    );

    let ledger = &mut ctx.accounts.settlement_ledger;
    let financing_offer = &mut ctx.accounts.financing_offer;
    require!(
        ledger.cumulative_financier_paid_lamports >= financing_offer.repayment_cap_lamports,
        TicketingError::FinancingNotSettled
    );

    let now = Clock::get()?.unix_timestamp;
    ledger.financing_settled = true;
    if ledger.settled_at == 0 {
        ledger.settled_at = now;
    }
    ledger.updated_at = now;
    financing_offer.status = FinancingLifecycleStatus::Settled;
    financing_offer.updated_at = now;

    emit!(FinancingSettled {
        event: ctx.accounts.event_account.key(),
        organizer: ctx.accounts.organizer_profile.key(),
        financing_offer: financing_offer.key(),
        settlement_ledger: ledger.key(),
        settled_at: ledger.settled_at,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct FinalizeSettlement<'info> {
    pub organizer_authority: Signer<'info>,
    #[account(
        seeds = [SEED_PROTOCOL_CONFIG],
        bump = protocol_config.bump,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,
    #[account(
        seeds = [SEED_ORGANIZER, organizer_authority.key().as_ref()],
        bump = organizer_profile.bump,
        constraint = organizer_profile.authority == organizer_authority.key() @ TicketingError::Unauthorized,
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
        seeds = [SEED_SETTLEMENT_LEDGER, event_account.key().as_ref()],
        bump = settlement_ledger.bump,
        constraint = settlement_ledger.event == event_account.key() @ TicketingError::Unauthorized,
        constraint = settlement_ledger.financing_offer == financing_offer.key() @ TicketingError::Unauthorized,
    )]
    pub settlement_ledger: Account<'info, SettlementLedger>,
}
