use anchor_lang::{prelude::*, Discriminator};

use crate::{
    constants::{
        ROLE_SCOPE_ORGANIZER, ROLE_SCOPE_PROTOCOL, SEED_ORGANIZER, SEED_PROTOCOL_CONFIG,
        SEED_ROLE_BINDING,
    },
    error::TicketingError,
    events::{GovernanceAuditReferenceStored, RoleRevoked},
    state::{OrganizerProfile, ProtocolConfig, RoleBinding},
    utils::correlation::derive_correlation_id,
};

pub fn revoke_role(ctx: Context<RevokeRole>, role: u8, scope: u8, reason_code: u16) -> Result<()> {
    require!(
        ctx.accounts.role_binding.role == role && ctx.accounts.role_binding.scope == scope,
        TicketingError::InvalidRole
    );
    require_keys_eq!(
        ctx.accounts.role_binding.subject,
        ctx.accounts.subject.key(),
        TicketingError::Unauthorized
    );

    let revoker = ctx.accounts.revoker.key();
    let protocol_admin = ctx.accounts.protocol_config.admin;
    if scope == ROLE_SCOPE_PROTOCOL {
        require_keys_eq!(revoker, protocol_admin, TicketingError::Unauthorized);
    } else if scope == ROLE_SCOPE_ORGANIZER {
        if revoker != protocol_admin {
            let (expected_profile, _) =
                Pubkey::find_program_address(&[SEED_ORGANIZER, revoker.as_ref()], &crate::ID);
            require_keys_eq!(
                ctx.accounts.organizer_profile.key(),
                expected_profile,
                TicketingError::Unauthorized
            );
            let data = ctx.accounts.organizer_profile.try_borrow_data()?;
            let mut data_slice: &[u8] = &data;
            require!(
                data_slice.len() >= 8 && &data_slice[..8] == OrganizerProfile::DISCRIMINATOR,
                TicketingError::Unauthorized
            );
            let organizer_profile = OrganizerProfile::try_deserialize(&mut data_slice)?;
            require_keys_eq!(
                organizer_profile.authority,
                revoker,
                TicketingError::Unauthorized
            );
        }
        require_keys_eq!(
            ctx.accounts.role_binding.target,
            ctx.accounts.target.key(),
            TicketingError::Unauthorized
        );
    } else {
        return err!(TicketingError::InvalidRoleScope);
    }

    let now = Clock::get()?.unix_timestamp;
    let role_binding = &mut ctx.accounts.role_binding;
    require!(role_binding.active, TicketingError::RoleNotActive);
    role_binding.active = false;
    role_binding.revoked_at = now;
    let correlation_id = derive_correlation_id(
        &role_binding.target,
        &role_binding.subject,
        now,
        u16::from(role),
    );
    let audit_reference =
        derive_correlation_id(&role_binding.key(), &revoker, now, u16::from(reason_code));
    role_binding.last_audit_reference = audit_reference;
    role_binding.last_correlation_id = correlation_id;
    role_binding.updated_at = now;

    emit!(RoleRevoked {
        role_binding: role_binding.key(),
        role,
        scope,
        target: role_binding.target,
        subject: role_binding.subject,
        revoker,
        reason_code,
        audit_reference,
        correlation_id,
        at: now,
    });

    emit!(GovernanceAuditReferenceStored {
        role_binding: role_binding.key(),
        target: role_binding.target,
        subject: role_binding.subject,
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
pub struct RevokeRole<'info> {
    pub revoker: Signer<'info>,
    pub subject: SystemAccount<'info>,
    /// CHECK: target account key is validated against role binding.
    pub target: UncheckedAccount<'info>,
    #[account(
        seeds = [SEED_PROTOCOL_CONFIG],
        bump = protocol_config.bump,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,
    /// CHECK: organizer-scope revocation validates this as revoker organizer PDA.
    pub organizer_profile: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [SEED_ROLE_BINDING, target.key().as_ref(), &[role], subject.key().as_ref()],
        bump = role_binding.bump,
    )]
    pub role_binding: Account<'info, RoleBinding>,
}
