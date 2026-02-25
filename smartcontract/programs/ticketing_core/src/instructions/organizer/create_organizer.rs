use anchor_lang::prelude::*;

use crate::{
    constants::{MAX_ORGANIZER_METADATA_URI_LEN, SEED_ORGANIZER, SEED_PROTOCOL_CONFIG},
    error::TicketingError,
    state::{OrganizerProfile, OrganizerStatus, ProtocolConfig},
};

pub fn create_organizer(
    ctx: Context<CreateOrganizer>,
    metadata_uri: String,
    payout_wallet: Pubkey,
) -> Result<()> {
    require!(
        metadata_uri.len() <= MAX_ORGANIZER_METADATA_URI_LEN,
        TicketingError::InvalidMetadataUriLength
    );
    require!(
        !ctx.accounts.protocol_config.is_paused,
        TicketingError::ProtocolPaused
    );

    let now = Clock::get()?.unix_timestamp;
    let organizer = &mut ctx.accounts.organizer_profile;
    organizer.bump = ctx.bumps.organizer_profile;
    organizer.authority = ctx.accounts.authority.key();
    organizer.payout_wallet = payout_wallet;
    organizer.status = OrganizerStatus::Active;
    organizer.compliance_flags = 0;
    organizer.metadata_uri = metadata_uri;
    organizer.created_at = now;
    organizer.updated_at = now;

    Ok(())
}

#[derive(Accounts)]
pub struct CreateOrganizer<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub authority: Signer<'info>,
    #[account(
        seeds = [SEED_PROTOCOL_CONFIG],
        bump = protocol_config.bump,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,
    #[account(
        init,
        payer = payer,
        space = 8 + OrganizerProfile::INIT_SPACE,
        seeds = [SEED_ORGANIZER, authority.key().as_ref()],
        bump,
    )]
    pub organizer_profile: Account<'info, OrganizerProfile>,
    pub system_program: Program<'info, System>,
}
