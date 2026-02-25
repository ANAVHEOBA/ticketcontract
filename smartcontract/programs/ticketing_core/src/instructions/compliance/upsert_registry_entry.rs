use anchor_lang::{prelude::*, Discriminator};

use crate::{
    constants::{
        COMPLIANCE_LIST_ALLOW, COMPLIANCE_LIST_DENY, COMPLIANCE_LIST_ENTITY_DENY,
        COMPLIANCE_SCOPE_EVENT, COMPLIANCE_SCOPE_PROTOCOL, MAX_COMPLIANCE_REGISTRY_ENTRIES,
        SEED_COMPLIANCE_REGISTRY, SEED_EVENT, SEED_ORGANIZER, SEED_PROTOCOL_CONFIG,
    },
    error::TicketingError,
    events::ComplianceRegistryUpdated,
    state::{ComplianceRegistry, EventAccount, OrganizerProfile, ProtocolConfig},
};

pub fn upsert_registry_entry(
    ctx: Context<UpsertRegistryEntry>,
    scope: u8,
    target: Pubkey,
    subject: Pubkey,
    list_type: u8,
    is_allowed: bool,
    decision_code: u16,
) -> Result<()> {
    require!(
        scope == COMPLIANCE_SCOPE_PROTOCOL || scope == COMPLIANCE_SCOPE_EVENT,
        TicketingError::InvalidComplianceScope
    );
    require!(
        list_type == COMPLIANCE_LIST_ALLOW
            || list_type == COMPLIANCE_LIST_DENY
            || list_type == COMPLIANCE_LIST_ENTITY_DENY,
        TicketingError::InvalidComplianceListType
    );

    let protocol_config = &ctx.accounts.protocol_config;
    if scope == COMPLIANCE_SCOPE_PROTOCOL {
        require_keys_eq!(target, protocol_config.key(), TicketingError::Unauthorized);
        require_keys_eq!(
            ctx.accounts.authority.key(),
            protocol_config.admin,
            TicketingError::Unauthorized
        );
    } else {
        require!(
            is_event_scope_authorized(
                &ctx.accounts.authority,
                &ctx.accounts.protocol_config,
                &ctx.accounts.organizer_profile,
                &ctx.accounts.event_account,
                target
            )?,
            TicketingError::Unauthorized
        );
    }

    let now = Clock::get()?.unix_timestamp;
    let registry = &mut ctx.accounts.compliance_registry;
    if registry.target == Pubkey::default() {
        registry.bump = ctx.bumps.compliance_registry;
        registry.scope = scope;
        registry.target = target;
        registry.allowlist = Vec::new();
        registry.denylist = Vec::new();
        registry.entity_denylist = Vec::new();
    } else {
        require!(
            registry.scope == scope,
            TicketingError::InvalidComplianceScope
        );
        require_keys_eq!(registry.target, target, TicketingError::Unauthorized);
    }

    let entries = match list_type {
        COMPLIANCE_LIST_ALLOW => &mut registry.allowlist,
        COMPLIANCE_LIST_DENY => &mut registry.denylist,
        _ => &mut registry.entity_denylist,
    };
    if is_allowed {
        if !entries.iter().any(|k| *k == subject) {
            require!(
                entries.len() < MAX_COMPLIANCE_REGISTRY_ENTRIES,
                TicketingError::ComplianceRegistryFull
            );
            entries.push(subject);
        }
    } else {
        entries.retain(|k| *k != subject);
    }
    registry.updated_at = now;

    emit!(ComplianceRegistryUpdated {
        registry: registry.key(),
        scope,
        target,
        subject,
        list_type,
        is_allowed,
        decision_code,
        authority: ctx.accounts.authority.key(),
        at: now,
    });

    Ok(())
}

fn is_event_scope_authorized(
    authority: &Signer<'_>,
    protocol_config: &Account<'_, ProtocolConfig>,
    organizer_profile: &UncheckedAccount<'_>,
    event_account: &UncheckedAccount<'_>,
    target: Pubkey,
) -> Result<bool> {
    if authority.key() == protocol_config.admin {
        return Ok(true);
    }
    let organizer_data = organizer_profile.try_borrow_data()?;
    let mut organizer_slice: &[u8] = &organizer_data;
    if organizer_slice.len() < 8 || &organizer_slice[..8] != OrganizerProfile::DISCRIMINATOR {
        return Ok(false);
    }
    let organizer = OrganizerProfile::try_deserialize(&mut organizer_slice)?;
    let (expected_organizer, _) =
        Pubkey::find_program_address(&[SEED_ORGANIZER, authority.key().as_ref()], &crate::ID);
    if organizer_profile.key() != expected_organizer || organizer.authority != authority.key() {
        return Ok(false);
    }

    let event_data = event_account.try_borrow_data()?;
    let mut event_slice: &[u8] = &event_data;
    if event_slice.len() < 8 || &event_slice[..8] != EventAccount::DISCRIMINATOR {
        return Ok(false);
    }
    let event = EventAccount::try_deserialize(&mut event_slice)?;
    let (expected_event, _) = Pubkey::find_program_address(
        &[
            SEED_EVENT,
            organizer_profile.key().as_ref(),
            &event.event_id.to_le_bytes(),
        ],
        &crate::ID,
    );
    Ok(event_account.key() == expected_event
        && event.organizer == organizer_profile.key()
        && target == event_account.key())
}

#[derive(Accounts)]
#[instruction(scope: u8, target: Pubkey)]
pub struct UpsertRegistryEntry<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub authority: Signer<'info>,
    #[account(
        seeds = [SEED_PROTOCOL_CONFIG],
        bump = protocol_config.bump,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,
    /// CHECK: only deserialized and validated when using event scope.
    pub organizer_profile: UncheckedAccount<'info>,
    /// CHECK: only deserialized and validated when using event scope.
    pub event_account: UncheckedAccount<'info>,
    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + ComplianceRegistry::INIT_SPACE,
        seeds = [SEED_COMPLIANCE_REGISTRY, target.as_ref()],
        bump,
    )]
    pub compliance_registry: Account<'info, ComplianceRegistry>,
    pub system_program: Program<'info, System>,
}
