use anchor_lang::prelude::*;

use crate::{
    constants::{MAX_PERK_CODE_LEN, SEED_LOYALTY_LEDGER, SEED_PROTOCOL_CONFIG},
    error::TicketingError,
    events::LoyaltyPointsRedeemed,
    state::{LoyaltyLedger, ProtocolConfig},
};

pub fn redeem_points(
    ctx: Context<RedeemPoints>,
    points_to_burn: u64,
    perk_code: String,
    event: Pubkey,
) -> Result<()> {
    require!(
        !ctx.accounts.protocol_config.is_paused,
        TicketingError::ProtocolPaused
    );
    require!(points_to_burn > 0, TicketingError::InvalidLoyaltyPoints);
    require!(
        !perk_code.is_empty() && perk_code.len() <= MAX_PERK_CODE_LEN,
        TicketingError::InvalidLoyaltyPerkCode
    );

    let now = Clock::get()?.unix_timestamp;
    let ledger = &mut ctx.accounts.loyalty_ledger;
    require!(
        ledger.available_points >= points_to_burn,
        TicketingError::InsufficientLoyaltyPoints
    );

    ledger.available_points = ledger
        .available_points
        .checked_sub(points_to_burn)
        .ok_or(TicketingError::MathOverflow)?;
    ledger.total_redeemed_points = ledger
        .total_redeemed_points
        .checked_add(points_to_burn)
        .ok_or(TicketingError::MathOverflow)?;
    ledger.last_redeemed_at = now;
    ledger.updated_at = now;

    emit!(LoyaltyPointsRedeemed {
        wallet: ctx.accounts.wallet.key(),
        loyalty_ledger: ledger.key(),
        event,
        points_burned: points_to_burn,
        perk_code,
        available_points: ledger.available_points,
        total_redeemed_points: ledger.total_redeemed_points,
        at: now,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct RedeemPoints<'info> {
    pub wallet: Signer<'info>,
    #[account(
        seeds = [SEED_PROTOCOL_CONFIG],
        bump = protocol_config.bump,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,
    #[account(
        mut,
        seeds = [SEED_LOYALTY_LEDGER, wallet.key().as_ref()],
        bump = loyalty_ledger.bump,
        constraint = loyalty_ledger.wallet == wallet.key() @ TicketingError::Unauthorized,
    )]
    pub loyalty_ledger: Account<'info, LoyaltyLedger>,
}
