use anchor_lang::prelude::*;

use crate::{
    constants::SEED_PROTOCOL_CONFIG,
    error::TicketingError,
    events::TrustSignalSchemaVersionUpdated,
    state::{ProtocolConfig, TrustSignal},
};

pub fn set_schema_version(ctx: Context<SetSchemaVersion>, new_schema_version: u16) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let signal = &mut ctx.accounts.trust_signal;
    let old_schema_version = signal.schema_version;

    require!(
        new_schema_version > 0,
        TicketingError::InvalidTrustSchemaVersion
    );
    require!(
        new_schema_version > old_schema_version,
        TicketingError::InvalidTrustSchemaVersion
    );

    signal.schema_version = new_schema_version;
    signal.updated_at = now;

    emit!(TrustSignalSchemaVersionUpdated {
        admin: ctx.accounts.admin.key(),
        wallet: signal.wallet,
        trust_signal: signal.key(),
        old_schema_version,
        new_schema_version,
        at: now,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct SetSchemaVersion<'info> {
    pub admin: Signer<'info>,
    #[account(
        seeds = [SEED_PROTOCOL_CONFIG],
        bump = protocol_config.bump,
        constraint = protocol_config.admin == admin.key() @ TicketingError::Unauthorized,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,
    #[account(mut)]
    pub trust_signal: Account<'info, TrustSignal>,
}
