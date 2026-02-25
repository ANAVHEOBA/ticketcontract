use anchor_lang::prelude::*;

use crate::{
    constants::{MAX_MAX_TICKETS_PER_WALLET, MAX_PROTOCOL_FEE_BPS, SEED_PROTOCOL_CONFIG},
    error::TicketingError,
    events::{ProtocolAuthoritiesUpdated, ProtocolConfigUpdated, ProtocolVaultsUpdated},
    state::ProtocolConfig,
};

pub fn set_protocol_config(
    ctx: Context<SetProtocolConfig>,
    protocol_fee_bps: u16,
    max_tickets_per_wallet: u16,
) -> Result<()> {
    require!(
        protocol_fee_bps <= MAX_PROTOCOL_FEE_BPS,
        TicketingError::InvalidFeeBps
    );
    require!(
        max_tickets_per_wallet > 0 && max_tickets_per_wallet <= MAX_MAX_TICKETS_PER_WALLET,
        TicketingError::InvalidMaxTicketsPerWallet
    );

    let protocol_config = &mut ctx.accounts.protocol_config;
    protocol_config.protocol_fee_bps = protocol_fee_bps;
    protocol_config.max_tickets_per_wallet = max_tickets_per_wallet;
    protocol_config.updated_at = Clock::get()?.unix_timestamp;

    emit!(ProtocolConfigUpdated {
        admin: ctx.accounts.admin.key(),
        protocol_fee_bps,
        max_tickets_per_wallet,
        is_paused: protocol_config.is_paused,
    });

    Ok(())
}

pub fn set_protocol_authorities(
    ctx: Context<SetProtocolAuthorities>,
    new_admin: Pubkey,
    new_upgrade_authority: Pubkey,
) -> Result<()> {
    require!(
        new_admin != Pubkey::default(),
        TicketingError::InvalidAuthority
    );
    require!(
        new_upgrade_authority != Pubkey::default(),
        TicketingError::InvalidAuthority
    );

    let protocol_config = &mut ctx.accounts.protocol_config;
    let old_admin = protocol_config.admin;
    protocol_config.admin = new_admin;
    protocol_config.upgrade_authority = new_upgrade_authority;
    protocol_config.updated_at = Clock::get()?.unix_timestamp;

    emit!(ProtocolAuthoritiesUpdated {
        old_admin,
        new_admin,
        new_upgrade_authority,
    });

    Ok(())
}

pub fn register_protocol_vaults(
    ctx: Context<RegisterProtocolVaults>,
    treasury_vault: Pubkey,
    fee_vault: Pubkey,
) -> Result<()> {
    let protocol_config = &mut ctx.accounts.protocol_config;
    protocol_config.treasury_vault = treasury_vault;
    protocol_config.fee_vault = fee_vault;
    protocol_config.updated_at = Clock::get()?.unix_timestamp;

    emit!(ProtocolVaultsUpdated {
        admin: ctx.accounts.admin.key(),
        treasury_vault,
        fee_vault,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct SetProtocolConfig<'info> {
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
pub struct RegisterProtocolVaults<'info> {
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
pub struct SetProtocolAuthorities<'info> {
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [SEED_PROTOCOL_CONFIG],
        bump = protocol_config.bump,
        constraint = protocol_config.admin == admin.key() @ TicketingError::Unauthorized,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,
}
