use anchor_lang::prelude::*;

use crate::{
    constants::{
        SEED_VAULT, VAULT_KIND_EVENT, VAULT_KIND_FINANCING, VAULT_KIND_ORGANIZER,
        VAULT_KIND_PROTOCOL,
    },
    error::TicketingError,
    events::VaultSnapshotRecorded,
    state::VaultAccount,
};

const VAULT_STATE_TAG: &[u8] = b"state";
const VAULT_FUNDS_TAG: &[u8] = b"funds";

pub fn snapshot_vault(ctx: Context<SnapshotVault>, kind: u8, parent: Pubkey) -> Result<()> {
    require!(
        kind == VAULT_KIND_PROTOCOL
            || kind == VAULT_KIND_ORGANIZER
            || kind == VAULT_KIND_EVENT
            || kind == VAULT_KIND_FINANCING,
        TicketingError::InvalidVaultKind
    );

    let vault_state = &mut ctx.accounts.vault_state;
    require_keys_eq!(
        vault_state.parent,
        parent,
        TicketingError::InvalidVaultParent
    );
    require!(vault_state.kind == kind, TicketingError::InvalidVaultKind);
    require_keys_eq!(
        vault_state.vault,
        ctx.accounts.vault.key(),
        TicketingError::InvalidVaultParent
    );

    let balance = ctx.accounts.vault.get_lamports();
    if balance >= vault_state.last_recorded_balance_lamports {
        let delta = balance
            .checked_sub(vault_state.last_recorded_balance_lamports)
            .ok_or(TicketingError::MathOverflow)?;
        vault_state.total_inflow_lamports = vault_state
            .total_inflow_lamports
            .checked_add(delta)
            .ok_or(TicketingError::MathOverflow)?;
    } else {
        let delta = vault_state
            .last_recorded_balance_lamports
            .checked_sub(balance)
            .ok_or(TicketingError::MathOverflow)?;
        vault_state.total_outflow_lamports = vault_state
            .total_outflow_lamports
            .checked_add(delta)
            .ok_or(TicketingError::MathOverflow)?;
    }
    vault_state.last_recorded_balance_lamports = balance;
    vault_state.updated_at = Clock::get()?.unix_timestamp;

    emit!(VaultSnapshotRecorded {
        vault: ctx.accounts.vault.key(),
        kind,
        parent,
        balance_lamports: balance,
        total_inflow_lamports: vault_state.total_inflow_lamports,
        total_outflow_lamports: vault_state.total_outflow_lamports,
        at: vault_state.updated_at,
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(kind: u8, parent: Pubkey)]
pub struct SnapshotVault<'info> {
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
}
