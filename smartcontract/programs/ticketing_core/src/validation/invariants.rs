use anchor_lang::prelude::*;

use crate::{error::TicketingError, state::EventAccount};

pub fn assert_event_not_paused(event: &EventAccount) -> Result<()> {
    require!(!event.is_paused, TicketingError::EventPaused);
    Ok(())
}

pub fn assert_account_size(account_info: &AccountInfo<'_>, expected_data_len: usize) -> Result<()> {
    require!(
        account_info.data_len() == expected_data_len,
        TicketingError::AccountSizeMismatch
    );
    Ok(())
}

pub fn assert_rent_exempt(account_info: &AccountInfo<'_>) -> Result<()> {
    let rent = Rent::get()?;
    require!(
        rent.is_exempt(account_info.lamports(), account_info.data_len()),
        TicketingError::AccountNotRentExempt
    );
    Ok(())
}
