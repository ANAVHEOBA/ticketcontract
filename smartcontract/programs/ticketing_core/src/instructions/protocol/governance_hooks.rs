use anchor_lang::prelude::*;

use crate::{
    constants::{MAX_MAX_TICKETS_PER_WALLET, MAX_PROTOCOL_FEE_BPS, SEED_PROTOCOL_CONFIG},
    error::TicketingError,
    events::{
        ProtocolConfigChangeExecuted, ProtocolConfigChangeQueued, ProtocolEmergencyAdminAction,
        ProtocolMultisigConfigUpdated, ProtocolTimelockUpdated, ProtocolUpgradeAuthorityAccepted,
        ProtocolUpgradeHandoffStarted,
    },
    instructions::protocol::hooks::enforce_privileged_signoff,
    state::ProtocolConfig,
};

pub fn set_multisig_config(
    ctx: Context<SetProtocolGovernance>,
    enabled: bool,
    threshold: u8,
    signer_1: Pubkey,
    signer_2: Pubkey,
    signer_3: Pubkey,
) -> Result<()> {
    enforce_privileged_signoff(
        &ctx.accounts.protocol_config,
        ctx.accounts.admin.key(),
        ctx.remaining_accounts,
    )?;

    if enabled {
        require!(
            threshold > 0 && threshold <= 3,
            TicketingError::InvalidMultisigConfig
        );
        let configured = [signer_1, signer_2, signer_3]
            .iter()
            .filter(|k| **k != Pubkey::default())
            .count();
        require!(
            configured >= usize::from(threshold),
            TicketingError::InvalidMultisigConfig
        );
    }

    let now = Clock::get()?.unix_timestamp;
    let cfg = &mut ctx.accounts.protocol_config;
    cfg.multisig_enabled = enabled;
    cfg.multisig_threshold = if enabled { threshold } else { 0 };
    cfg.multisig_signer_1 = signer_1;
    cfg.multisig_signer_2 = signer_2;
    cfg.multisig_signer_3 = signer_3;
    cfg.updated_at = now;

    emit!(ProtocolMultisigConfigUpdated {
        admin: ctx.accounts.admin.key(),
        enabled,
        threshold: cfg.multisig_threshold,
        signer_1,
        signer_2,
        signer_3,
        at: now,
    });
    Ok(())
}

pub fn set_timelock_delay(
    ctx: Context<SetProtocolGovernance>,
    timelock_delay_secs: i64,
) -> Result<()> {
    require!(timelock_delay_secs >= 0, TicketingError::InvalidAuthority);
    enforce_privileged_signoff(
        &ctx.accounts.protocol_config,
        ctx.accounts.admin.key(),
        ctx.remaining_accounts,
    )?;

    let now = Clock::get()?.unix_timestamp;
    let cfg = &mut ctx.accounts.protocol_config;
    cfg.timelock_delay_secs = timelock_delay_secs;
    cfg.updated_at = now;

    emit!(ProtocolTimelockUpdated {
        admin: ctx.accounts.admin.key(),
        timelock_delay_secs,
        at: now,
    });
    Ok(())
}

pub fn queue_protocol_config_change(
    ctx: Context<SetProtocolGovernance>,
    pending_protocol_fee_bps: u16,
    pending_max_tickets_per_wallet: u16,
) -> Result<()> {
    require!(
        pending_protocol_fee_bps <= MAX_PROTOCOL_FEE_BPS,
        TicketingError::InvalidFeeBps
    );
    require!(
        pending_max_tickets_per_wallet > 0
            && pending_max_tickets_per_wallet <= MAX_MAX_TICKETS_PER_WALLET,
        TicketingError::InvalidMaxTicketsPerWallet
    );
    enforce_privileged_signoff(
        &ctx.accounts.protocol_config,
        ctx.accounts.admin.key(),
        ctx.remaining_accounts,
    )?;

    let now = Clock::get()?.unix_timestamp;
    let cfg = &mut ctx.accounts.protocol_config;
    let eta = now
        .checked_add(cfg.timelock_delay_secs)
        .ok_or(TicketingError::MathOverflow)?;
    cfg.pending_protocol_fee_bps = pending_protocol_fee_bps;
    cfg.pending_max_tickets_per_wallet = pending_max_tickets_per_wallet;
    cfg.config_change_eta = eta;
    cfg.updated_at = now;

    emit!(ProtocolConfigChangeQueued {
        admin: ctx.accounts.admin.key(),
        pending_protocol_fee_bps,
        pending_max_tickets_per_wallet,
        execute_at: eta,
        at: now,
    });
    Ok(())
}

pub fn execute_protocol_config_change(ctx: Context<SetProtocolGovernance>) -> Result<()> {
    enforce_privileged_signoff(
        &ctx.accounts.protocol_config,
        ctx.accounts.admin.key(),
        ctx.remaining_accounts,
    )?;

    let now = Clock::get()?.unix_timestamp;
    let cfg = &mut ctx.accounts.protocol_config;
    require!(cfg.config_change_eta > 0, TicketingError::NoPendingConfigChange);
    require!(now >= cfg.config_change_eta, TicketingError::TimelockNotReady);

    cfg.protocol_fee_bps = cfg.pending_protocol_fee_bps;
    cfg.max_tickets_per_wallet = cfg.pending_max_tickets_per_wallet;
    cfg.config_change_eta = 0;
    cfg.updated_at = now;

    emit!(ProtocolConfigChangeExecuted {
        admin: ctx.accounts.admin.key(),
        protocol_fee_bps: cfg.protocol_fee_bps,
        max_tickets_per_wallet: cfg.max_tickets_per_wallet,
        at: now,
    });
    Ok(())
}

pub fn begin_upgrade_authority_handoff(
    ctx: Context<SetProtocolGovernance>,
    pending_upgrade_authority: Pubkey,
) -> Result<()> {
    require!(
        pending_upgrade_authority != Pubkey::default(),
        TicketingError::InvalidAuthority
    );
    enforce_privileged_signoff(
        &ctx.accounts.protocol_config,
        ctx.accounts.admin.key(),
        ctx.remaining_accounts,
    )?;

    let now = Clock::get()?.unix_timestamp;
    let cfg = &mut ctx.accounts.protocol_config;
    let eta = now
        .checked_add(cfg.timelock_delay_secs)
        .ok_or(TicketingError::MathOverflow)?;
    cfg.pending_upgrade_authority = pending_upgrade_authority;
    cfg.upgrade_handoff_started_at = now;
    cfg.upgrade_handoff_eta = eta;
    cfg.updated_at = now;

    emit!(ProtocolUpgradeHandoffStarted {
        admin: ctx.accounts.admin.key(),
        current_upgrade_authority: cfg.upgrade_authority,
        pending_upgrade_authority,
        ready_at: eta,
        at: now,
    });
    Ok(())
}

pub fn accept_upgrade_authority_handoff(ctx: Context<AcceptUpgradeAuthorityHandoff>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let cfg = &mut ctx.accounts.protocol_config;
    require!(
        cfg.pending_upgrade_authority != Pubkey::default(),
        TicketingError::NoUpgradeHandoffInProgress
    );
    require!(
        ctx.accounts.pending_upgrade_authority.key() == cfg.pending_upgrade_authority,
        TicketingError::InvalidPendingUpgradeAuthority
    );
    require!(now >= cfg.upgrade_handoff_eta, TicketingError::UpgradeHandoffNotReady);

    let previous_upgrade_authority = cfg.upgrade_authority;
    cfg.upgrade_authority = cfg.pending_upgrade_authority;
    cfg.pending_upgrade_authority = Pubkey::default();
    cfg.upgrade_handoff_started_at = 0;
    cfg.upgrade_handoff_eta = 0;
    cfg.updated_at = now;

    emit!(ProtocolUpgradeAuthorityAccepted {
        previous_upgrade_authority,
        new_upgrade_authority: cfg.upgrade_authority,
        accepted_by: ctx.accounts.pending_upgrade_authority.key(),
        at: now,
    });
    Ok(())
}

pub fn emergency_rotate_admin(
    ctx: Context<EmergencyAdminAction>,
    new_admin: Pubkey,
    new_emergency_admin: Pubkey,
    reason_code: u16,
) -> Result<()> {
    require!(new_admin != Pubkey::default(), TicketingError::InvalidAuthority);
    require!(
        new_emergency_admin != Pubkey::default(),
        TicketingError::InvalidAuthority
    );

    let now = Clock::get()?.unix_timestamp;
    let cfg = &mut ctx.accounts.protocol_config;
    require!(cfg.is_paused, TicketingError::EmergencyRequiresPausedProtocol);

    let old_admin = cfg.admin;
    cfg.admin = new_admin;
    cfg.emergency_admin = new_emergency_admin;
    cfg.emergency_action_nonce = cfg
        .emergency_action_nonce
        .checked_add(1)
        .ok_or(TicketingError::MathOverflow)?;
    cfg.updated_at = now;

    emit!(ProtocolEmergencyAdminAction {
        emergency_admin: ctx.accounts.emergency_admin.key(),
        old_admin,
        new_admin,
        new_emergency_admin,
        nonce: cfg.emergency_action_nonce,
        reason_code,
        at: now,
    });
    Ok(())
}

#[derive(Accounts)]
pub struct SetProtocolGovernance<'info> {
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
pub struct AcceptUpgradeAuthorityHandoff<'info> {
    pub pending_upgrade_authority: Signer<'info>,
    #[account(
        mut,
        seeds = [SEED_PROTOCOL_CONFIG],
        bump = protocol_config.bump,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,
}

#[derive(Accounts)]
pub struct EmergencyAdminAction<'info> {
    pub emergency_admin: Signer<'info>,
    #[account(
        mut,
        seeds = [SEED_PROTOCOL_CONFIG],
        bump = protocol_config.bump,
        constraint = protocol_config.emergency_admin == emergency_admin.key() @ TicketingError::Unauthorized,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,
}
