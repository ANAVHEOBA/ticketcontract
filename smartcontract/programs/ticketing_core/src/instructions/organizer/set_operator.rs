use anchor_lang::prelude::*;

use crate::{
    constants::{SEED_ORGANIZER, SEED_ORGANIZER_OPERATOR},
    error::TicketingError,
    state::{OrganizerOperator, OrganizerProfile},
};

pub fn set_organizer_operator(
    ctx: Context<SetOrganizerOperator>,
    permissions: u32,
    active: bool,
) -> Result<()> {
    let operator = &mut ctx.accounts.organizer_operator;
    operator.bump = ctx.bumps.organizer_operator;
    operator.organizer = ctx.accounts.organizer_profile.key();
    operator.operator = ctx.accounts.operator.key();
    operator.permissions = permissions;
    operator.active = active;
    operator.updated_at = Clock::get()?.unix_timestamp;

    Ok(())
}

#[derive(Accounts)]
pub struct SetOrganizerOperator<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub authority: Signer<'info>,
    pub operator: SystemAccount<'info>,
    #[account(
        seeds = [SEED_ORGANIZER, authority.key().as_ref()],
        bump = organizer_profile.bump,
        constraint = organizer_profile.authority == authority.key() @ TicketingError::Unauthorized,
    )]
    pub organizer_profile: Account<'info, OrganizerProfile>,
    #[account(
        init,
        payer = payer,
        space = 8 + OrganizerOperator::INIT_SPACE,
        seeds = [SEED_ORGANIZER_OPERATOR, organizer_profile.key().as_ref(), operator.key().as_ref()],
        bump,
    )]
    pub organizer_operator: Account<'info, OrganizerOperator>,
    pub system_program: Program<'info, System>,
}
