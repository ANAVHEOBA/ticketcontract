use anchor_lang::prelude::*;

use crate::{error::TicketingError, state::ProtocolConfig};

pub fn enforce_privileged_signoff(
    protocol_config: &Account<'_, ProtocolConfig>,
    admin: Pubkey,
    remaining_accounts: &[AccountInfo<'_>],
) -> Result<()> {
    require_keys_eq!(protocol_config.admin, admin, TicketingError::Unauthorized);

    if !protocol_config.multisig_enabled {
        return Ok(());
    }

    let threshold = protocol_config.multisig_threshold;
    require!(
        threshold > 0 && threshold <= 3,
        TicketingError::InvalidMultisigConfig
    );

    let signers = [
        protocol_config.multisig_signer_1,
        protocol_config.multisig_signer_2,
        protocol_config.multisig_signer_3,
    ];
    let configured_count = signers.iter().filter(|k| **k != Pubkey::default()).count();
    require!(
        configured_count >= usize::from(threshold),
        TicketingError::InvalidMultisigConfig
    );

    let mut approved = 0usize;
    for signer in signers {
        if signer == Pubkey::default() {
            continue;
        }
        if signer == admin {
            approved += 1;
            continue;
        }
        let present = remaining_accounts
            .iter()
            .any(|acc| acc.is_signer && acc.key() == signer);
        if present {
            approved += 1;
        }
    }

    require!(
        approved >= usize::from(threshold),
        TicketingError::MultisigSignoffMissing
    );
    Ok(())
}
