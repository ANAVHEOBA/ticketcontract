use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer};

use crate::{
    constants::{
        SEED_EVENT, SEED_FINANCING_OFFER, SEED_ORGANIZER, SEED_PROTOCOL_CONFIG,
        SEED_SETTLEMENT_LEDGER,
    },
    error::TicketingError,
    events::{FinancingSettled, RevenueWaterfallSettled},
    state::{
        EventAccount, FinancingLifecycleStatus, FinancingOffer, OrganizerProfile, ProtocolConfig,
        SettlementLedger,
    },
};

pub fn settle_primary_revenue(
    ctx: Context<SettlePrimaryRevenue>,
    gross_revenue_lamports: u64,
    protocol_bps: u16,
    royalty_bps: u16,
    other_bps: u16,
) -> Result<()> {
    settle_revenue_waterfall(
        ctx.accounts.into(),
        gross_revenue_lamports,
        protocol_bps,
        royalty_bps,
        other_bps,
        true,
    )
}

pub(crate) fn settle_revenue_waterfall<'info>(
    mut accounts: SettleRevenueAccounts<'_, 'info>,
    gross_revenue_lamports: u64,
    protocol_bps: u16,
    royalty_bps: u16,
    other_bps: u16,
    is_primary: bool,
) -> Result<()> {
    require!(
        !accounts.protocol_config.is_paused,
        TicketingError::ProtocolPaused
    );
    require!(
        accounts.organizer_authority.key() == accounts.organizer_profile.authority,
        TicketingError::Unauthorized
    );
    require!(
        gross_revenue_lamports > 0,
        TicketingError::InvalidSettlementAmount
    );

    let total_priority_three_bps = u32::from(protocol_bps)
        .checked_add(u32::from(royalty_bps))
        .and_then(|v| v.checked_add(u32::from(other_bps)))
        .ok_or(TicketingError::MathOverflow)?;
    require!(
        total_priority_three_bps <= 10_000,
        TicketingError::InvalidWaterfallBps
    );

    let protocol_amount =
        prorata_amount(gross_revenue_lamports, protocol_bps).ok_or(TicketingError::MathOverflow)?;
    let royalty_amount =
        prorata_amount(gross_revenue_lamports, royalty_bps).ok_or(TicketingError::MathOverflow)?;
    let other_amount =
        prorata_amount(gross_revenue_lamports, other_bps).ok_or(TicketingError::MathOverflow)?;

    let priority_three_total = protocol_amount
        .checked_add(royalty_amount)
        .and_then(|v| v.checked_add(other_amount))
        .ok_or(TicketingError::MathOverflow)?;
    require!(
        priority_three_total <= gross_revenue_lamports,
        TicketingError::MathOverflow
    );

    let priority_one_two_pool = gross_revenue_lamports
        .checked_sub(priority_three_total)
        .ok_or(TicketingError::MathOverflow)?;

    let repayment_remaining = accounts
        .financing_offer
        .repayment_cap_lamports
        .checked_sub(
            accounts
                .settlement_ledger
                .cumulative_financier_paid_lamports,
        )
        .unwrap_or(0);
    let financier_amount = core::cmp::min(priority_one_two_pool, repayment_remaining);
    let organizer_amount = priority_one_two_pool
        .checked_sub(financier_amount)
        .ok_or(TicketingError::MathOverflow)?;

    transfer_lamports(
        accounts.system_program,
        accounts.revenue_source.to_account_info(),
        accounts.financier_wallet.to_account_info(),
        financier_amount,
    )?;
    transfer_lamports(
        accounts.system_program,
        accounts.revenue_source.to_account_info(),
        accounts.organizer_payout_wallet.to_account_info(),
        organizer_amount,
    )?;
    transfer_lamports(
        accounts.system_program,
        accounts.revenue_source.to_account_info(),
        accounts.protocol_fee_vault.to_account_info(),
        protocol_amount,
    )?;
    transfer_lamports(
        accounts.system_program,
        accounts.revenue_source.to_account_info(),
        accounts.royalty_vault.to_account_info(),
        royalty_amount,
    )?;
    transfer_lamports(
        accounts.system_program,
        accounts.revenue_source.to_account_info(),
        accounts.other_vault.to_account_info(),
        other_amount,
    )?;

    let now = Clock::get()?.unix_timestamp;
    let ledger = &mut accounts.settlement_ledger;
    if ledger.created_at == 0 {
        let (_, bump) = Pubkey::find_program_address(
            &[
                SEED_SETTLEMENT_LEDGER,
                accounts.event_account.key().as_ref(),
            ],
            &crate::ID,
        );
        ledger.bump = bump;
        ledger.event = accounts.event_account.key();
        ledger.organizer = accounts.organizer_profile.key();
        ledger.financing_offer = accounts.financing_offer.key();
        ledger.created_at = now;
    }

    if is_primary {
        ledger.cumulative_primary_routed_lamports = ledger
            .cumulative_primary_routed_lamports
            .checked_add(gross_revenue_lamports)
            .ok_or(TicketingError::MathOverflow)?;
    } else {
        ledger.cumulative_secondary_routed_lamports = ledger
            .cumulative_secondary_routed_lamports
            .checked_add(gross_revenue_lamports)
            .ok_or(TicketingError::MathOverflow)?;
    }
    ledger.cumulative_financier_paid_lamports = ledger
        .cumulative_financier_paid_lamports
        .checked_add(financier_amount)
        .ok_or(TicketingError::MathOverflow)?;
    ledger.cumulative_organizer_paid_lamports = ledger
        .cumulative_organizer_paid_lamports
        .checked_add(organizer_amount)
        .ok_or(TicketingError::MathOverflow)?;
    ledger.cumulative_protocol_paid_lamports = ledger
        .cumulative_protocol_paid_lamports
        .checked_add(protocol_amount)
        .ok_or(TicketingError::MathOverflow)?;
    ledger.cumulative_royalty_paid_lamports = ledger
        .cumulative_royalty_paid_lamports
        .checked_add(royalty_amount)
        .ok_or(TicketingError::MathOverflow)?;
    ledger.cumulative_other_paid_lamports = ledger
        .cumulative_other_paid_lamports
        .checked_add(other_amount)
        .ok_or(TicketingError::MathOverflow)?;
    ledger.updated_at = now;

    let obligations_completed = ledger.cumulative_financier_paid_lamports
        >= accounts.financing_offer.repayment_cap_lamports;
    if obligations_completed {
        ledger.financing_settled = true;
        if ledger.settled_at == 0 {
            ledger.settled_at = now;
        }
        accounts.financing_offer.status = FinancingLifecycleStatus::Settled;
        accounts.financing_offer.updated_at = now;

        emit!(FinancingSettled {
            event: accounts.event_account.key(),
            organizer: accounts.organizer_profile.key(),
            financing_offer: accounts.financing_offer.key(),
            settlement_ledger: ledger.key(),
            settled_at: ledger.settled_at,
        });
    }

    emit!(RevenueWaterfallSettled {
        event: accounts.event_account.key(),
        organizer: accounts.organizer_profile.key(),
        financing_offer: accounts.financing_offer.key(),
        settlement_ledger: ledger.key(),
        source_wallet: accounts.revenue_source.key(),
        gross_revenue_lamports,
        primary_revenue: is_primary,
        financier_amount,
        organizer_amount,
        protocol_amount,
        royalty_amount,
        other_amount,
        at: now,
    });

    Ok(())
}

fn prorata_amount(total: u64, bps: u16) -> Option<u64> {
    (u128::from(total) * u128::from(bps) / 10_000u128)
        .try_into()
        .ok()
}

fn transfer_lamports<'info>(
    system_program: &Program<'info, System>,
    from: AccountInfo<'info>,
    to: AccountInfo<'info>,
    amount: u64,
) -> Result<()> {
    if amount == 0 {
        return Ok(());
    }

    let cpi_accounts = Transfer { from, to };
    let cpi_ctx = CpiContext::new(system_program.to_account_info(), cpi_accounts);
    system_program::transfer(cpi_ctx, amount)
}

#[derive(Accounts)]
pub struct SettlePrimaryRevenue<'info> {
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

pub struct SettleRevenueAccounts<'a, 'info> {
    pub revenue_source: &'a mut Signer<'info>,
    pub organizer_authority: &'a Signer<'info>,
    pub protocol_config: &'a Account<'info, ProtocolConfig>,
    pub organizer_profile: &'a Account<'info, OrganizerProfile>,
    pub event_account: &'a Account<'info, EventAccount>,
    pub financing_offer: &'a mut Account<'info, FinancingOffer>,
    pub settlement_ledger: &'a mut Account<'info, SettlementLedger>,
    pub financier_wallet: &'a mut SystemAccount<'info>,
    pub organizer_payout_wallet: &'a mut SystemAccount<'info>,
    pub protocol_fee_vault: &'a mut SystemAccount<'info>,
    pub royalty_vault: &'a mut SystemAccount<'info>,
    pub other_vault: &'a mut SystemAccount<'info>,
    pub system_program: &'a Program<'info, System>,
}

impl<'a, 'info> From<&'a mut SettlePrimaryRevenue<'info>> for SettleRevenueAccounts<'a, 'info> {
    fn from(value: &'a mut SettlePrimaryRevenue<'info>) -> Self {
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
