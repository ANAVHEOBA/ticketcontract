use anchor_lang::prelude::*;

use crate::{
    constants::{MAX_ORGANIZER_METADATA_URI_LEN, SEED_ORGANIZER, SEED_PROTOCOL_CONFIG},
    error::TicketingError,
    state::{OrganizerProfile, OrganizerStatus, ProtocolConfig},
};

pub fn update_organizer(
    ctx: Context<UpdateOrganizer>,
    metadata_uri: String,
    payout_wallet: Pubkey,
) -> Result<()> {
    require!(
        metadata_uri.len() <= MAX_ORGANIZER_METADATA_URI_LEN,
        TicketingError::InvalidMetadataUriLength
    );

    let organizer = &mut ctx.accounts.organizer_profile;
    organizer.metadata_uri = metadata_uri;
    organizer.payout_wallet = payout_wallet;
    organizer.updated_at = Clock::get()?.unix_timestamp;

    Ok(())
}

pub fn set_organizer_status(ctx: Context<SetOrganizerStatus>, status: u8) -> Result<()> {
    let organizer = &mut ctx.accounts.organizer_profile;
    organizer.status = match status {
        1 => OrganizerStatus::Active,
        2 => OrganizerStatus::Suspended,
        _ => return err!(TicketingError::InvalidOrganizerStatus),
    };
    organizer.updated_at = Clock::get()?.unix_timestamp;

    Ok(())
}

pub fn set_organizer_compliance_flags(
    ctx: Context<SetOrganizerComplianceFlags>,
    compliance_flags: u32,
) -> Result<()> {
    let organizer = &mut ctx.accounts.organizer_profile;
    organizer.compliance_flags = compliance_flags;
    organizer.updated_at = Clock::get()?.unix_timestamp;

    Ok(())
}

#[derive(Accounts)]
pub struct UpdateOrganizer<'info> {
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [SEED_ORGANIZER, authority.key().as_ref()],
        bump = organizer_profile.bump,
        constraint = organizer_profile.authority == authority.key() @ TicketingError::Unauthorized,
    )]
    pub organizer_profile: Account<'info, OrganizerProfile>,
}

#[derive(Accounts)]
pub struct SetOrganizerStatus<'info> {
    pub admin: Signer<'info>,
    #[account(
        seeds = [SEED_PROTOCOL_CONFIG],
        bump = protocol_config.bump,
        constraint = protocol_config.admin == admin.key() @ TicketingError::Unauthorized,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,
    #[account(mut)]
    pub organizer_profile: Account<'info, OrganizerProfile>,
}

#[derive(Accounts)]
pub struct SetOrganizerComplianceFlags<'info> {
    pub admin: Signer<'info>,
    #[account(
        seeds = [SEED_PROTOCOL_CONFIG],
        bump = protocol_config.bump,
        constraint = protocol_config.admin == admin.key() @ TicketingError::Unauthorized,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,
    #[account(mut)]
    pub organizer_profile: Account<'info, OrganizerProfile>,
}
