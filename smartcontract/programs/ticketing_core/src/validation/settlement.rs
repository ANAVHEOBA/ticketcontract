use anchor_lang::prelude::*;

use crate::{
    error::TicketingError,
    state::SettlementLedger,
};

pub fn assert_waterfall_bps(protocol_bps: u16, royalty_bps: u16, other_bps: u16) -> Result<()> {
    let total = u32::from(protocol_bps)
        .checked_add(u32::from(royalty_bps))
        .and_then(|v| v.checked_add(u32::from(other_bps)))
        .ok_or(TicketingError::MathOverflow)?;
    require!(total <= 10_000, TicketingError::InvalidWaterfallBps);
    Ok(())
}

pub fn assert_settlement_reference(reference_id: &[u8; 16]) -> Result<()> {
    require!(
        !reference_id.iter().all(|byte| *byte == 0),
        TicketingError::InvalidSettlementReference
    );
    Ok(())
}

pub fn try_idempotent_replay(ledger: &SettlementLedger, reference_id: &[u8; 16]) -> bool {
    ledger.last_settlement_reference == *reference_id
}

pub fn begin_settlement(ledger: &mut SettlementLedger) -> Result<()> {
    require!(!ledger.is_settling, TicketingError::SettlementInProgress);
    ledger.is_settling = true;
    Ok(())
}

pub fn finish_settlement(ledger: &mut SettlementLedger, reference_id: [u8; 16]) {
    ledger.is_settling = false;
    ledger.last_settlement_reference = reference_id;
}
