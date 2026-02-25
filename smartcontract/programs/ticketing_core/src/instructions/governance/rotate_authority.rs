use anchor_lang::prelude::*;

use crate::{
    constants::SEED_ROLE_BINDING,
    error::TicketingError,
    events::{GovernanceAuditReferenceStored, RoleAuthorityRotated},
    state::RoleBinding,
    utils::correlation::derive_correlation_id,
};

pub fn rotate_authority(
    ctx: Context<RotateAuthority>,
    role: u8,
    scope: u8,
    starts_at: i64,
    expires_at: i64,
) -> Result<()> {
    require!(
        expires_at == 0 || expires_at >= starts_at,
        TicketingError::InvalidRoleGrantWindow
    );
    require!(
        ctx.accounts.old_role_binding.active,
        TicketingError::RoleNotActive
    );
    require!(
        ctx.accounts.old_role_binding.role == role && ctx.accounts.old_role_binding.scope == scope,
        TicketingError::InvalidRole
    );
    require_keys_eq!(
        ctx.accounts.old_role_binding.target,
        ctx.accounts.target.key(),
        TicketingError::Unauthorized
    );
    require_keys_eq!(
        ctx.accounts.old_role_binding.granted_by,
        ctx.accounts.authority.key(),
        TicketingError::Unauthorized
    );

    let now = Clock::get()?.unix_timestamp;
    let old_role_binding = &mut ctx.accounts.old_role_binding;
    old_role_binding.active = false;
    old_role_binding.revoked_at = now;
    let correlation_id = derive_correlation_id(
        &ctx.accounts.target.key(),
        &ctx.accounts.new_subject.key(),
        now,
        u16::from(role),
    );
    let audit_reference = derive_correlation_id(
        &old_role_binding.key(),
        &ctx.accounts.new_role_binding.key(),
        now,
        u16::from(scope),
    );
    old_role_binding.last_audit_reference = audit_reference;
    old_role_binding.last_correlation_id = correlation_id;
    old_role_binding.updated_at = now;

    let new_role_binding = &mut ctx.accounts.new_role_binding;
    new_role_binding.bump = ctx.bumps.new_role_binding;
    new_role_binding.role = role;
    new_role_binding.scope = scope;
    new_role_binding.active = true;
    new_role_binding.target = ctx.accounts.target.key();
    new_role_binding.subject = ctx.accounts.new_subject.key();
    new_role_binding.granted_by = ctx.accounts.authority.key();
    new_role_binding.starts_at = starts_at;
    new_role_binding.expires_at = expires_at;
    new_role_binding.revoked_at = 0;
    new_role_binding.last_audit_reference = audit_reference;
    new_role_binding.last_correlation_id = correlation_id;
    if new_role_binding.created_at == 0 {
        new_role_binding.created_at = now;
    }
    new_role_binding.updated_at = now;

    emit!(RoleAuthorityRotated {
        old_role_binding: old_role_binding.key(),
        new_role_binding: new_role_binding.key(),
        role,
        scope,
        target: ctx.accounts.target.key(),
        old_subject: old_role_binding.subject,
        new_subject: new_role_binding.subject,
        authority: ctx.accounts.authority.key(),
        audit_reference,
        correlation_id,
        at: now,
    });

    emit!(GovernanceAuditReferenceStored {
        role_binding: new_role_binding.key(),
        target: new_role_binding.target,
        subject: new_role_binding.subject,
        role,
        scope,
        audit_reference,
        correlation_id,
        at: now,
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(role: u8)]
pub struct RotateAuthority<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: key-only role target.
    pub target: UncheckedAccount<'info>,
    pub new_subject: SystemAccount<'info>,
    #[account(
        mut,
        seeds = [SEED_ROLE_BINDING, target.key().as_ref(), &[role], old_role_binding.subject.as_ref()],
        bump = old_role_binding.bump,
    )]
    pub old_role_binding: Account<'info, RoleBinding>,
    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + RoleBinding::INIT_SPACE,
        seeds = [SEED_ROLE_BINDING, target.key().as_ref(), &[role], new_subject.key().as_ref()],
        bump,
    )]
    pub new_role_binding: Account<'info, RoleBinding>,
    pub system_program: Program<'info, System>,
}
