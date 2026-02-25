use anchor_lang::prelude::*;

use crate::{
    constants::{
        SEED_EVENT, SEED_FINANCING_OFFER, SEED_ORGANIZER, SEED_PROTOCOL_CONFIG,
        SEED_SETTLEMENT_LEDGER,
    },
    error::TicketingError,
    state::{EventAccount, FinancingOffer, OrganizerProfile, ProtocolConfig, SettlementLedger},
};

use super::settle_primary_revenue::{settle_revenue_waterfall, SettleRevenueAccounts};

pub fn settle_resale_revenue(
    ctx: Context<SettleResaleRevenue>,
    gross_revenue_lamports: u64,
    protocol_bps: u16,
    royalty_bps: u16,
    other_bps: u16,
    settlement_reference: [u8; 16],
) -> Result<()> {
    settle_revenue_waterfall(
        ctx.accounts.into(),
        gross_revenue_lamports,
        protocol_bps,
        royalty_bps,
        other_bps,
        settlement_reference,
        false,
    )
}

#[derive(Accounts)]
pub struct SettleResaleRevenue<'info> {
    #[account(mut)]
    pub revenue_source: Signer<'info>,
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
        constraint = financing_offer.terms_locked @ TicketingError::FinancingOfferNotAccepted,
    )]
    pub financing_offer: Account<'info, FinancingOffer>,
    #[account(
        init_if_needed,
        payer = revenue_source,
        space = 8 + SettlementLedger::INIT_SPACE,
        seeds = [SEED_SETTLEMENT_LEDGER, event_account.key().as_ref()],
        bump,
    )]
    pub settlement_ledger: Account<'info, SettlementLedger>,
    #[account(mut)]
    pub financier_wallet: SystemAccount<'info>,
    #[account(mut, address = organizer_profile.payout_wallet @ TicketingError::Unauthorized)]
    pub organizer_payout_wallet: SystemAccount<'info>,
    #[account(mut, address = protocol_config.fee_vault @ TicketingError::Unauthorized)]
    pub protocol_fee_vault: SystemAccount<'info>,
    #[account(mut)]
    pub royalty_vault: SystemAccount<'info>,
    #[account(mut)]
    pub other_vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'a, 'info> From<&'a mut SettleResaleRevenue<'info>> for SettleRevenueAccounts<'a, 'info> {
    fn from(value: &'a mut SettleResaleRevenue<'info>) -> Self {
        Self {
            revenue_source: &mut value.revenue_source,
            organizer_authority: &value.organizer_authority,
            protocol_config: &value.protocol_config,
            organizer_profile: &value.organizer_profile,
            event_account: &value.event_account,
            financing_offer: &mut value.financing_offer,
            settlement_ledger: &mut value.settlement_ledger,
            financier_wallet: &mut value.financier_wallet,
            organizer_payout_wallet: &mut value.organizer_payout_wallet,
            protocol_fee_vault: &mut value.protocol_fee_vault,
            royalty_vault: &mut value.royalty_vault,
            other_vault: &mut value.other_vault,
            system_program: &value.system_program,
        }
    }
}
