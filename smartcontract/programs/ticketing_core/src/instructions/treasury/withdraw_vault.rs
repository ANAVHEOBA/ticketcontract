use anchor_lang::{
    prelude::*,
    solana_program::{program::invoke_signed, system_instruction},
};

use crate::{
    constants::{
        ROLE_ORGANIZER_ADMIN, ROLE_PROTOCOL_ADMIN, ROLE_SCOPE_ORGANIZER, ROLE_SCOPE_PROTOCOL,
        SEED_ROLE_BINDING, SEED_VAULT, VAULT_KIND_EVENT, VAULT_KIND_FINANCING,
        VAULT_KIND_ORGANIZER, VAULT_KIND_PROTOCOL,
    },
    error::TicketingError,
    events::VaultWithdrawn,
    state::{RoleBinding, VaultAccount},
    validation::access::role_is_active,
};

const VAULT_STATE_TAG: &[u8] = b"state";
const VAULT_FUNDS_TAG: &[u8] = b"funds";

pub fn withdraw_vault(
    ctx: Context<WithdrawVault>,
    kind: u8,
    parent: Pubkey,
    amount_lamports: u64,
) -> Result<()> {
    require!(amount_lamports > 0, TicketingError::InvalidSettlementAmount);
    require!(
        kind == VAULT_KIND_PROTOCOL
            || kind == VAULT_KIND_ORGANIZER
            || kind == VAULT_KIND_EVENT
            || kind == VAULT_KIND_FINANCING,
        TicketingError::InvalidVaultKind
    );

    let vault_state = &mut ctx.accounts.vault_state;
    require!(vault_state.kind == kind, TicketingError::InvalidVaultKind);
    require_keys_eq!(
        vault_state.parent,
        parent,
        TicketingError::InvalidVaultParent
    );
    require_keys_eq!(
        vault_state.vault,
        ctx.accounts.vault.key(),
        TicketingError::InvalidVaultParent
    );

    let authority = ctx.accounts.authority.key();
    require!(
        is_withdraw_authorized(authority, vault_state, &ctx.accounts.role_binding)?,
        TicketingError::Unauthorized
    );

    let vault_balance = ctx.accounts.vault.get_lamports();
    require!(
        vault_balance >= amount_lamports,
        TicketingError::InsufficientVaultBalance
    );

    let transfer_ix = system_instruction::transfer(
        &ctx.accounts.vault.key(),
        &ctx.accounts.destination.key(),
        amount_lamports,
    );
    let signer_seeds: &[&[u8]] = &[
        SEED_VAULT,
        VAULT_FUNDS_TAG,
        &[kind],
        parent.as_ref(),
        &[vault_state.vault_bump],
    ];
    invoke_signed(
        &transfer_ix,
        &[
            ctx.accounts.vault.to_account_info(),
            ctx.accounts.destination.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        &[signer_seeds],
    )?;

    let now = Clock::get()?.unix_timestamp;
    vault_state.total_outflow_lamports = vault_state
        .total_outflow_lamports
        .checked_add(amount_lamports)
        .ok_or(TicketingError::MathOverflow)?;
    vault_state.last_recorded_balance_lamports = ctx.accounts.vault.get_lamports();
    vault_state.updated_at = now;

    emit!(VaultWithdrawn {
        vault: ctx.accounts.vault.key(),
        kind,
        parent,
        destination: ctx.accounts.destination.key(),
        authority,
        amount_lamports,
        balance_lamports: vault_state.last_recorded_balance_lamports,
        total_outflow_lamports: vault_state.total_outflow_lamports,
        at: now,
    });

    Ok(())
}

fn is_withdraw_authorized(
    authority: Pubkey,
    vault_state: &VaultAccount,
    role_binding: &UncheckedAccount<'_>,
) -> Result<bool> {
    if authority == vault_state.authority {
        return Ok(true);
    }

    let (scope, role) = match vault_state.kind {
        VAULT_KIND_PROTOCOL | VAULT_KIND_FINANCING => (ROLE_SCOPE_PROTOCOL, ROLE_PROTOCOL_ADMIN),
        VAULT_KIND_ORGANIZER | VAULT_KIND_EVENT => (ROLE_SCOPE_ORGANIZER, ROLE_ORGANIZER_ADMIN),
        _ => return err!(TicketingError::InvalidVaultKind),
    };

    let (expected_role_binding, _) = Pubkey::find_program_address(
        &[
            SEED_ROLE_BINDING,
            vault_state.controller.as_ref(),
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
        &binding,
        role,
        scope,
        vault_state.controller,
        authority,
        now,
    ))
}

#[derive(Accounts)]
#[instruction(kind: u8, parent: Pubkey)]
pub struct WithdrawVault<'info> {
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [SEED_VAULT, VAULT_STATE_TAG, &[kind], parent.as_ref()],
        bump = vault_state.bump,
    )]
    pub vault_state: Account<'info, VaultAccount>,
    #[account(
        mut,
        seeds = [SEED_VAULT, VAULT_FUNDS_TAG, &[kind], parent.as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>,
    #[account(mut)]
    pub destination: SystemAccount<'info>,
    /// CHECK: optional role-binding PDA validated in handler for delegated access.
    pub role_binding: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}
