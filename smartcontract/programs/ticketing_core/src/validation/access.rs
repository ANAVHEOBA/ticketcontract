use anchor_lang::prelude::*;

use crate::{error::TicketingError, state::RoleBinding};

pub fn role_is_active(
    role_binding: &RoleBinding,
    role: u8,
    scope: u8,
    target: Pubkey,
    subject: Pubkey,
    now: i64,
) -> bool {
    if !role_binding.active
        || role_binding.role != role
        || role_binding.scope != scope
        || role_binding.target != target
        || role_binding.subject != subject
    {
        return false;
    }

    if now < role_binding.starts_at {
        return false;
    }

    role_binding.expires_at == 0 || now <= role_binding.expires_at
}

pub fn require_role_active(
    role_binding: &RoleBinding,
    role: u8,
    scope: u8,
    target: Pubkey,
    subject: Pubkey,
    now: i64,
) -> Result<()> {
    require!(
        role_is_active(role_binding, role, scope, target, subject, now),
        TicketingError::RoleNotActive
    );
    Ok(())
}
