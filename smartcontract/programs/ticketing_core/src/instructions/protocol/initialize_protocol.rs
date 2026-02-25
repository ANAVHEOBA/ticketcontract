use anchor_lang::prelude::*;

use crate::{
    constants::{MAX_MAX_TICKETS_PER_WALLET, MAX_PROTOCOL_FEE_BPS, SEED_PROTOCOL_CONFIG},
    error::TicketingError,
    events::ProtocolInitialized,
    state::ProtocolConfig,
};

pub fn initialize_protocol(
    ctx: Context<InitializeProtocol>,
    admin: Pubkey,
    upgrade_authority: Pubkey,
    treasury_vault: Pubkey,
    fee_vault: Pubkey,
    protocol_fee_bps: u16,
    max_tickets_per_wallet: u16,
) -> Result<()> {
    require!(admin != Pubkey::default(), TicketingError::InvalidAuthority);
    require!(
        upgrade_authority != Pubkey::default(),
        TicketingError::InvalidAuthority
    );
    require!(
        protocol_fee_bps <= MAX_PROTOCOL_FEE_BPS,
        TicketingError::InvalidFeeBps
    );
    require!(
        max_tickets_per_wallet > 0 && max_tickets_per_wallet <= MAX_MAX_TICKETS_PER_WALLET,
        TicketingError::InvalidMaxTicketsPerWallet
    );

    let now = Clock::get()?.unix_timestamp;
    let protocol_config = &mut ctx.accounts.protocol_config;
    protocol_config.bump = ctx.bumps.protocol_config;
    protocol_config.admin = admin;
    protocol_config.upgrade_authority = upgrade_authority;
    protocol_config.pending_upgrade_authority = Pubkey::default();
    protocol_config.upgrade_handoff_started_at = 0;
    protocol_config.upgrade_handoff_eta = 0;
    protocol_config.timelock_delay_secs = 0;
    protocol_config.pending_protocol_fee_bps = protocol_fee_bps;
    protocol_config.pending_max_tickets_per_wallet = max_tickets_per_wallet;
    protocol_config.config_change_eta = 0;
    protocol_config.multisig_enabled = false;
    protocol_config.multisig_threshold = 0;
    protocol_config.multisig_signer_1 = Pubkey::default();
    protocol_config.multisig_signer_2 = Pubkey::default();
    protocol_config.multisig_signer_3 = Pubkey::default();
    protocol_config.emergency_admin = admin;
    protocol_config.emergency_action_nonce = 0;
    protocol_config.treasury_vault = treasury_vault;
    protocol_config.fee_vault = fee_vault;
    protocol_config.protocol_fee_bps = protocol_fee_bps;
    protocol_config.loyalty_multiplier_bps = 10_000;
    protocol_config.max_tickets_per_wallet = max_tickets_per_wallet;
    protocol_config.is_paused = false;
    protocol_config.created_at = now;
    protocol_config.updated_at = now;

    emit!(ProtocolInitialized {
        admin,
        upgrade_authority,
        treasury_vault,
        fee_vault,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct InitializeProtocol<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + ProtocolConfig::INIT_SPACE,
        seeds = [SEED_PROTOCOL_CONFIG],
        bump
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,
    pub system_program: Program<'info, System>,
}
