use anchor_lang::prelude::*;

use crate::{
    constants::{
        LOYALTY_LEDGER_SCHEMA_VERSION, MAX_LOYALTY_MULTIPLIER_BPS, SEED_EVENT,
        SEED_LOYALTY_LEDGER, SEED_ORGANIZER, SEED_PROTOCOL_CONFIG, SEED_TICKET, SEED_TICKET_CLASS,
    },
    error::TicketingError,
    events::{LoyaltyEventMultiplierUpdated, LoyaltyGlobalMultiplierUpdated, LoyaltyPointsAccrued},
    state::{
        EventAccount, EventStatus, LoyaltyLedger, OrganizerProfile, ProtocolConfig, Ticket,
        TicketClass,
    },
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum LoyaltyAccrualReason {
    Purchase = 1,
    Attendance = 2,
    HoldDuration = 3,
}

impl LoyaltyAccrualReason {
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            1 => Ok(Self::Purchase),
            2 => Ok(Self::Attendance),
            3 => Ok(Self::HoldDuration),
            _ => err!(TicketingError::InvalidLoyaltyReason),
        }
    }
}

pub fn set_global_loyalty_multiplier(
    ctx: Context<SetGlobalLoyaltyMultiplier>,
    multiplier_bps: u16,
) -> Result<()> {
    require!(
        multiplier_bps > 0 && multiplier_bps <= MAX_LOYALTY_MULTIPLIER_BPS,
        TicketingError::InvalidLoyaltyMultiplier
    );

    let now = Clock::get()?.unix_timestamp;
    let protocol_config = &mut ctx.accounts.protocol_config;
    protocol_config.loyalty_multiplier_bps = multiplier_bps;
    protocol_config.updated_at = now;

    emit!(LoyaltyGlobalMultiplierUpdated {
        admin: ctx.accounts.admin.key(),
        multiplier_bps,
        at: now,
    });

    Ok(())
}

pub fn set_event_loyalty_multiplier(
    ctx: Context<SetEventLoyaltyMultiplier>,
    multiplier_bps: u16,
) -> Result<()> {
    require!(
        multiplier_bps > 0 && multiplier_bps <= MAX_LOYALTY_MULTIPLIER_BPS,
        TicketingError::InvalidLoyaltyMultiplier
    );
    require!(
        ctx.accounts.event_account.status == EventStatus::Draft
            || ctx.accounts.event_account.status == EventStatus::Frozen,
        TicketingError::InvalidEventStatusTransition
    );

    let now = Clock::get()?.unix_timestamp;
    let event_account = &mut ctx.accounts.event_account;
    event_account.loyalty_multiplier_bps = multiplier_bps;
    event_account.updated_at = now;

    emit!(LoyaltyEventMultiplierUpdated {
        organizer: ctx.accounts.authority.key(),
        event: event_account.key(),
        multiplier_bps,
        at: now,
    });

    Ok(())
}

pub fn accrue_points(
    ctx: Context<AccruePoints>,
    _class_id: u16,
    _ticket_id: u32,
    reason: u8,
    base_points: u64,
    hold_duration_days: u16,
) -> Result<()> {
    require!(
        !ctx.accounts.protocol_config.is_paused,
        TicketingError::ProtocolPaused
    );
    require!(base_points > 0, TicketingError::InvalidLoyaltyPoints);

    let reason = LoyaltyAccrualReason::from_u8(reason)?;
    let ticket = &ctx.accounts.ticket;
    let wallet = ctx.accounts.wallet.key();
    match reason {
        LoyaltyAccrualReason::Purchase => {
            require!(ticket.buyer == wallet, TicketingError::Unauthorized);
        }
        LoyaltyAccrualReason::Attendance => {
            require!(
                ticket.owner == wallet && ticket.checked_in_at > 0,
                TicketingError::InvalidLoyaltyPoints
            );
        }
        LoyaltyAccrualReason::HoldDuration => {
            require!(
                ticket.owner == wallet && hold_duration_days > 0,
                TicketingError::InvalidLoyaltyPoints
            );
        }
    }

    let adjusted_base = if reason == LoyaltyAccrualReason::HoldDuration {
        base_points
            .checked_mul(u64::from(hold_duration_days))
            .ok_or(TicketingError::MathOverflow)?
    } else {
        base_points
    };

    let global_multiplier = ctx.accounts.protocol_config.loyalty_multiplier_bps;
    let event_multiplier = ctx.accounts.event_account.loyalty_multiplier_bps;
    require!(
        global_multiplier > 0
            && global_multiplier <= MAX_LOYALTY_MULTIPLIER_BPS
            && event_multiplier > 0
            && event_multiplier <= MAX_LOYALTY_MULTIPLIER_BPS,
        TicketingError::InvalidLoyaltyMultiplier
    );

    let points_credited = (u128::from(adjusted_base)
        .checked_mul(u128::from(global_multiplier))
        .ok_or(TicketingError::MathOverflow)?
        .checked_mul(u128::from(event_multiplier))
        .ok_or(TicketingError::MathOverflow)?
        / 10_000u128
        / 10_000u128) as u64;
    require!(points_credited > 0, TicketingError::InvalidLoyaltyPoints);

    let now = Clock::get()?.unix_timestamp;
    let ledger = &mut ctx.accounts.loyalty_ledger;
    if ledger.wallet == Pubkey::default() {
        ledger.bump = ctx.bumps.loyalty_ledger;
        ledger.schema_version = LOYALTY_LEDGER_SCHEMA_VERSION;
        ledger.deprecated_layout_version = 0;
        ledger.replacement_account = Pubkey::default();
        ledger.deprecated_at = 0;
        ledger.wallet = wallet;
        ledger.total_accrued_points = 0;
        ledger.total_redeemed_points = 0;
        ledger.available_points = 0;
        ledger.last_event = Pubkey::default();
        ledger.last_reason = 0;
        ledger.last_accrued_at = 0;
        ledger.last_redeemed_at = 0;
        ledger.created_at = now;
    }

    ledger.total_accrued_points = ledger
        .total_accrued_points
        .checked_add(points_credited)
        .ok_or(TicketingError::MathOverflow)?;
    ledger.available_points = ledger
        .available_points
        .checked_add(points_credited)
        .ok_or(TicketingError::MathOverflow)?;
    ledger.last_event = ctx.accounts.event_account.key();
    ledger.last_reason = reason as u8;
    ledger.last_accrued_at = now;
    ledger.updated_at = now;

    emit!(LoyaltyPointsAccrued {
        wallet,
        loyalty_ledger: ledger.key(),
        event: ctx.accounts.event_account.key(),
        ticket: ticket.key(),
        reason: reason as u8,
        base_points: adjusted_base,
        points_credited,
        global_multiplier_bps: global_multiplier,
        event_multiplier_bps: event_multiplier,
        available_points: ledger.available_points,
        total_accrued_points: ledger.total_accrued_points,
        at: now,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct SetGlobalLoyaltyMultiplier<'info> {
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [SEED_PROTOCOL_CONFIG],
        bump = protocol_config.bump,
        constraint = protocol_config.admin == admin.key() @ TicketingError::Unauthorized,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,
}

#[derive(Accounts)]
pub struct SetEventLoyaltyMultiplier<'info> {
    pub authority: Signer<'info>,
    #[account(
        seeds = [SEED_ORGANIZER, authority.key().as_ref()],
        bump = organizer_profile.bump,
        constraint = organizer_profile.authority == authority.key() @ TicketingError::Unauthorized,
    )]
    pub organizer_profile: Account<'info, OrganizerProfile>,
    #[account(
        mut,
        seeds = [SEED_EVENT, organizer_profile.key().as_ref(), &event_account.event_id.to_le_bytes()],
        bump = event_account.bump,
        constraint = event_account.organizer == organizer_profile.key() @ TicketingError::Unauthorized,
    )]
    pub event_account: Account<'info, EventAccount>,
}

#[derive(Accounts)]
#[instruction(class_id: u16, ticket_id: u32)]
pub struct AccruePoints<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub authority: Signer<'info>,
    pub wallet: SystemAccount<'info>,
    #[account(
        seeds = [SEED_PROTOCOL_CONFIG],
        bump = protocol_config.bump,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,
    #[account(
        seeds = [SEED_ORGANIZER, authority.key().as_ref()],
        bump = organizer_profile.bump,
        constraint = organizer_profile.authority == authority.key() @ TicketingError::Unauthorized,
    )]
    pub organizer_profile: Account<'info, OrganizerProfile>,
    #[account(
        seeds = [SEED_EVENT, organizer_profile.key().as_ref(), &event_account.event_id.to_le_bytes()],
        bump = event_account.bump,
        constraint = event_account.organizer == organizer_profile.key() @ TicketingError::Unauthorized,
    )]
    pub event_account: Account<'info, EventAccount>,
    #[account(
        seeds = [SEED_TICKET_CLASS, event_account.key().as_ref(), &class_id.to_le_bytes()],
        bump = ticket_class.bump,
        constraint = ticket_class.event == event_account.key() @ TicketingError::Unauthorized,
        constraint = ticket_class.class_id == class_id @ TicketingError::Unauthorized,
    )]
    pub ticket_class: Account<'info, TicketClass>,
    #[account(
        seeds = [SEED_TICKET, event_account.key().as_ref(), &class_id.to_le_bytes(), &ticket_id.to_le_bytes()],
        bump = ticket.bump,
        constraint = ticket.event == event_account.key() @ TicketingError::Unauthorized,
        constraint = ticket.ticket_class == ticket_class.key() @ TicketingError::Unauthorized,
        constraint = ticket.ticket_id == ticket_id @ TicketingError::InvalidTicketId,
    )]
    pub ticket: Account<'info, Ticket>,
    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + LoyaltyLedger::INIT_SPACE,
        seeds = [SEED_LOYALTY_LEDGER, wallet.key().as_ref()],
        bump,
    )]
    pub loyalty_ledger: Account<'info, LoyaltyLedger>,
    pub system_program: Program<'info, System>,
}
