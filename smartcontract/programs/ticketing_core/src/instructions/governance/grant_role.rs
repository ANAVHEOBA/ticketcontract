use anchor_lang::{prelude::*, Discriminator};

use crate::{
    constants::{
        ROLE_OPERATOR, ROLE_ORGANIZER_ADMIN, ROLE_PROTOCOL_ADMIN, ROLE_SCANNER,
        ROLE_SCOPE_ORGANIZER, ROLE_SCOPE_PROTOCOL, SEED_ORGANIZER, SEED_PROTOCOL_CONFIG,
        SEED_ROLE_BINDING,
    },
    error::TicketingError,
    events::{GovernanceAuditReferenceStored, RoleGranted},
    state::{OrganizerProfile, ProtocolConfig, RoleBinding},
    utils::correlation::derive_correlation_id,
};

pub fn grant_role(
    ctx: Context<GrantRole>,
    role: u8,
    scope: u8,
    starts_at: i64,
    expires_at: i64,
) -> Result<()> {
    validate_role(scope, role)?;
    require!(
        expires_at == 0 || expires_at >= starts_at,
        TicketingError::InvalidRoleGrantWindow
    );

    let granter = ctx.accounts.granter.key();
    let protocol_config = &ctx.accounts.protocol_config;
    if scope == ROLE_SCOPE_PROTOCOL {
        require_keys_eq!(
            ctx.accounts.target.key(),
            protocol_config.key(),
            TicketingError::Unauthorized
        );
        require_keys_eq!(granter, protocol_config.admin, TicketingError::Unauthorized);
    } else {
        require!(
            is_organizer_authorized(
                granter,
                protocol_config.admin,
                &ctx.accounts.target,
                &ctx.accounts.organizer_profile
            )?,
            TicketingError::Unauthorized
        );
    }

    let now = Clock::get()?.unix_timestamp;
    let role_binding = &mut ctx.accounts.role_binding;
    role_binding.bump = ctx.bumps.role_binding;
    role_binding.role = role;
    role_binding.scope = scope;
    role_binding.active = true;
    role_binding.target = ctx.accounts.target.key();
    role_binding.subject = ctx.accounts.subject.key();
    role_binding.granted_by = granter;
    role_binding.starts_at = starts_at;
    role_binding.expires_at = expires_at;
    role_binding.revoked_at = 0;
    let correlation_id = derive_correlation_id(
        &role_binding.target,
        &role_binding.subject,
        now,
        u16::from(role),
    );
    let audit_reference =
        derive_correlation_id(&role_binding.key(), &granter, now, u16::from(scope));
    role_binding.last_audit_reference = audit_reference;
    role_binding.last_correlation_id = correlation_id;
    if role_binding.created_at == 0 {
        role_binding.created_at = now;
    }
    role_binding.updated_at = now;

    emit!(RoleGranted {
        role_binding: role_binding.key(),
        role,
        scope,
        target: role_binding.target,
        subject: role_binding.subject,
        granter,
        starts_at,
        expires_at,
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

fn validate_role(scope: u8, role: u8) -> Result<()> {
    match scope {
        ROLE_SCOPE_PROTOCOL => {
            require!(role == ROLE_PROTOCOL_ADMIN, TicketingError::InvalidRole)
        }
        ROLE_SCOPE_ORGANIZER => require!(
            role == ROLE_ORGANIZER_ADMIN || role == ROLE_OPERATOR || role == ROLE_SCANNER,
            TicketingError::InvalidRole
        ),
        _ => return err!(TicketingError::InvalidRoleScope),
    }
    Ok(())
}

fn is_organizer_authorized(
    granter: Pubkey,
    protocol_admin: Pubkey,
    target: &UncheckedAccount<'_>,
    organizer_profile: &UncheckedAccount<'_>,
) -> Result<bool> {
    if granter == protocol_admin {
        return Ok(true);
    }

    let (expected_profile, _) =
        Pubkey::find_program_address(&[SEED_ORGANIZER, granter.as_ref()], &crate::ID);
    if organizer_profile.key() != expected_profile || target.key() != organizer_profile.key() {
        return Ok(false);
    }
    let data = organizer_profile.try_borrow_data()?;
    let mut data_slice: &[u8] = &data;
    if data_slice.len() < 8 || &data_slice[..8] != OrganizerProfile::DISCRIMINATOR {
        return Ok(false);
    }
    let profile = OrganizerProfile::try_deserialize(&mut data_slice)?;
    Ok(profile.authority == granter)
}

#[derive(Accounts)]
#[instruction(role: u8, scope: u8)]
pub struct GrantRole<'info> {
    #[account(mut)]
    pub granter: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub subject: SystemAccount<'info>,
    /// CHECK: target account key is verified against scope-specific rules.
    pub target: UncheckedAccount<'info>,
    #[account(
        seeds = [SEED_PROTOCOL_CONFIG],
        bump = protocol_config.bump,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,
    /// CHECK: organizer scope validates this as PDA for `granter` when needed.
    pub organizer_profile: UncheckedAccount<'info>,
    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + RoleBinding::INIT_SPACE,
        seeds = [SEED_ROLE_BINDING, target.key().as_ref(), &[role], subject.key().as_ref()],
        bump,
    )]
    pub role_binding: Account<'info, RoleBinding>,
    pub system_program: Program<'info, System>,
}
