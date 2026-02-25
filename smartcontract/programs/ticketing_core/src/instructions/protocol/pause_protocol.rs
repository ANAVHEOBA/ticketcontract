use anchor_lang::prelude::*;

use crate::{
    constants::SEED_PROTOCOL_CONFIG, error::TicketingError, events::ProtocolConfigUpdated,
    instructions::protocol::hooks::enforce_privileged_signoff,
    state::ProtocolConfig,
};

pub fn pause_protocol(ctx: Context<PauseProtocol>, is_paused: bool) -> Result<()> {
    enforce_privileged_signoff(
        &ctx.accounts.protocol_config,
        ctx.accounts.admin.key(),
        ctx.remaining_accounts,
    )?;

    let protocol_config = &mut ctx.accounts.protocol_config;
    protocol_config.is_paused = is_paused;
    protocol_config.updated_at = Clock::get()?.unix_timestamp;

    emit!(ProtocolConfigUpdated {
        admin: ctx.accounts.admin.key(),
        protocol_fee_bps: protocol_config.protocol_fee_bps,
        max_tickets_per_wallet: protocol_config.max_tickets_per_wallet,
        is_paused,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct PauseProtocol<'info> {
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [SEED_PROTOCOL_CONFIG],
        bump = protocol_config.bump,
        constraint = protocol_config.admin == admin.key() @ TicketingError::Unauthorized,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,
}
