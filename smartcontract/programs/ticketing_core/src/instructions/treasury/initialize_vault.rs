use anchor_lang::{
    prelude::*,
    solana_program::{program::invoke_signed, system_instruction},
};

use crate::{
    constants::{
        ROLE_ORGANIZER_ADMIN, ROLE_PROTOCOL_ADMIN, ROLE_SCOPE_ORGANIZER, ROLE_SCOPE_PROTOCOL,
        SEED_PROTOCOL_CONFIG, SEED_ROLE_BINDING, SEED_VAULT, VAULT_KIND_EVENT,
        VAULT_KIND_FINANCING, VAULT_KIND_ORGANIZER, VAULT_KIND_PROTOCOL,
    },
    error::TicketingError,
    events::VaultInitialized,
    state::{
        EventAccount, FinancingOffer, OrganizerProfile, ProtocolConfig, RoleBinding, VaultAccount,
    },
    validation::access::role_is_active,
};

const VAULT_STATE_TAG: &[u8] = b"state";
const VAULT_FUNDS_TAG: &[u8] = b"funds";

pub fn initialize_vault(ctx: Context<InitializeVault>, kind: u8, parent: Pubkey) -> Result<()> {
    validate_kind(kind)?;

    let protocol_config = &ctx.accounts.protocol_config;
    let authority = ctx.accounts.authority.key();
    let (controller, primary_authority, scope, role) = match kind {
        VAULT_KIND_PROTOCOL => {
            require_keys_eq!(
                parent,
                protocol_config.key(),
                TicketingError::InvalidVaultParent
            );
            (
                protocol_config.key(),
                protocol_config.admin,
                ROLE_SCOPE_PROTOCOL,
                ROLE_PROTOCOL_ADMIN,
            )
        }
        VAULT_KIND_ORGANIZER => {
            let organizer_key = ctx.accounts.organizer_profile.key();
            let organizer: OrganizerProfile =
                deserialize_checked_account(&ctx.accounts.organizer_profile)?;
            require_keys_eq!(parent, organizer_key, TicketingError::InvalidVaultParent);
            (
                organizer_key,
                organizer.authority,
                ROLE_SCOPE_ORGANIZER,
                ROLE_ORGANIZER_ADMIN,
            )
        }
        VAULT_KIND_EVENT => {
            let organizer_key = ctx.accounts.organizer_profile.key();
            let event_key = ctx.accounts.event_account.key();
            let organizer: OrganizerProfile =
                deserialize_checked_account(&ctx.accounts.organizer_profile)?;
            let event: EventAccount = deserialize_checked_account(&ctx.accounts.event_account)?;
            require_keys_eq!(parent, event_key, TicketingError::InvalidVaultParent);
            require_keys_eq!(
                event.organizer,
                organizer_key,
                TicketingError::InvalidVaultParent
            );
            (
                organizer_key,
                organizer.authority,
                ROLE_SCOPE_ORGANIZER,
                ROLE_ORGANIZER_ADMIN,
            )
        }
        VAULT_KIND_FINANCING => {
            let financing_key = ctx.accounts.financing_offer.key();
            let _financing: FinancingOffer =
                deserialize_checked_account(&ctx.accounts.financing_offer)?;
            require_keys_eq!(parent, financing_key, TicketingError::InvalidVaultParent);
            (
                protocol_config.key(),
                protocol_config.admin,
                ROLE_SCOPE_PROTOCOL,
                ROLE_PROTOCOL_ADMIN,
            )
        }
        _ => return err!(TicketingError::InvalidVaultKind),
    };

    require!(
        is_authorized_with_role(
            authority,
            primary_authority,
            controller,
            role,
            scope,
            &ctx.accounts.role_binding
        )?,
        TicketingError::Unauthorized
    );

    let now = Clock::get()?.unix_timestamp;
    if ctx.accounts.vault.get_lamports() == 0 {
        let rent_lamports = Rent::get()?.minimum_balance(0);
        let create_ix = system_instruction::create_account(
            &ctx.accounts.payer.key(),
            &ctx.accounts.vault.key(),
            rent_lamports,
            0,
            &anchor_lang::system_program::ID,
        );
        let signer_seeds: &[&[u8]] = &[
            SEED_VAULT,
            VAULT_FUNDS_TAG,
            &[kind],
            parent.as_ref(),
            &[ctx.bumps.vault],
        ];
        invoke_signed(
            &create_ix,
            &[
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.vault.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[signer_seeds],
        )?;
    }

    let vault_state = &mut ctx.accounts.vault_state;
    vault_state.bump = ctx.bumps.vault_state;
    vault_state.vault_bump = ctx.bumps.vault;
    vault_state.kind = kind;
    vault_state.parent = parent;
    vault_state.vault = ctx.accounts.vault.key();
    vault_state.controller = controller;
    vault_state.authority = primary_authority;
    vault_state.last_recorded_balance_lamports = ctx.accounts.vault.get_lamports();
    vault_state.total_inflow_lamports = 0;
    vault_state.total_outflow_lamports = 0;
    vault_state.created_at = now;
    vault_state.updated_at = now;

    emit!(VaultInitialized {
        vault: ctx.accounts.vault.key(),
        kind,
        parent,
        controller,
        authority: primary_authority,
        at: now,
    });

    Ok(())
}

fn deserialize_checked_account<T: AccountDeserialize + Owner>(
    account: &UncheckedAccount<'_>,
) -> Result<T> {
    require_keys_eq!(
        *account.owner,
        T::owner(),
        TicketingError::InvalidVaultParent
    );
    let data = account.try_borrow_data()?;
    let mut data_slice: &[u8] = &data;
    T::try_deserialize(&mut data_slice).map_err(|_| error!(TicketingError::InvalidVaultParent))
}

fn validate_kind(kind: u8) -> Result<()> {
    require!(
        kind == VAULT_KIND_PROTOCOL
            || kind == VAULT_KIND_ORGANIZER
            || kind == VAULT_KIND_EVENT
            || kind == VAULT_KIND_FINANCING,
        TicketingError::InvalidVaultKind
    );
    Ok(())
}

fn is_authorized_with_role(
    authority: Pubkey,
    primary_authority: Pubkey,
    controller: Pubkey,
    role: u8,
    scope: u8,
    role_binding: &UncheckedAccount<'_>,
) -> Result<bool> {
    if authority == primary_authority {
        return Ok(true);
    }

    let (expected_role_binding, _) = Pubkey::find_program_address(
        &[
            SEED_ROLE_BINDING,
            controller.as_ref(),
            &[role],
            authority.as_ref(),
        ],
        &crate::ID,
    );
    if role_binding.key() != expected_role_binding {
        return Ok(false);
    }

    let data = role_binding.try_borrow_data()?;
    let mut data_slice: &[u8] = &data;
    if data_slice.len() < 8 || &data_slice[..8] != RoleBinding::DISCRIMINATOR {
        return Ok(false);
    }
    let binding = RoleBinding::try_deserialize(&mut data_slice)?;
    let now = Clock::get()?.unix_timestamp;
    Ok(role_is_active(
        &binding, role, scope, controller, authority, now,
    ))
}

#[derive(Accounts)]
#[instruction(kind: u8, parent: Pubkey)]
pub struct InitializeVault<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub authority: Signer<'info>,
    #[account(
        seeds = [SEED_PROTOCOL_CONFIG],
        bump = protocol_config.bump,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,
    /// CHECK: decoded only for organizer/event vault kinds.
    pub organizer_profile: UncheckedAccount<'info>,
    /// CHECK: decoded only for event vault kind.
    pub event_account: UncheckedAccount<'info>,
    /// CHECK: decoded only for financing vault kind.
    pub financing_offer: UncheckedAccount<'info>,
    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + VaultAccount::INIT_SPACE,
        seeds = [SEED_VAULT, VAULT_STATE_TAG, &[kind], parent.as_ref()],
        bump,
    )]
    pub vault_state: Account<'info, VaultAccount>,
    #[account(mut, seeds = [SEED_VAULT, VAULT_FUNDS_TAG, &[kind], parent.as_ref()], bump)]
    /// CHECK: validated as funds PDA; created via invoke_signed when missing.
    pub vault: UncheckedAccount<'info>,
    /// CHECK: optional role-binding PDA validated in handler when authority differs from owner.
    pub role_binding: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}
