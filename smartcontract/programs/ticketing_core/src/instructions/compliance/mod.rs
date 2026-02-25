pub mod set_event_restrictions;
pub mod upsert_registry_entry;

pub use set_event_restrictions::*;
pub use upsert_registry_entry::*;

use anchor_lang::{prelude::*, Discriminator};

use crate::{
    constants::{
        COMPLIANCE_DECISION_ALLOW, COMPLIANCE_DECISION_BLOCK_FINANCING,
        COMPLIANCE_DECISION_BLOCK_PRIMARY, COMPLIANCE_DECISION_BLOCK_RESALE,
        COMPLIANCE_DECISION_DENYLISTED, COMPLIANCE_DECISION_ENTITY_DENIED,
        COMPLIANCE_DECISION_JURISDICTION_RESTRICTED, COMPLIANCE_DECISION_MISSING_ALLOWLIST,
        COMPLIANCE_FLAG_BLOCK_FINANCING, COMPLIANCE_FLAG_BLOCK_PRIMARY,
        COMPLIANCE_FLAG_BLOCK_RESALE, COMPLIANCE_FLAG_JURISDICTION_RESTRICTED,
        COMPLIANCE_FLAG_REQUIRE_ALLOWLIST, COMPLIANCE_FLOW_FINANCING, COMPLIANCE_FLOW_PRIMARY_SALE,
        COMPLIANCE_FLOW_RESALE,
    },
    state::ComplianceRegistry,
};

pub fn evaluate_compliance(
    compliance_registry: &UncheckedAccount<'_>,
    flow: u8,
    event_restriction_flags: u32,
    wallet: Pubkey,
    entity: Pubkey,
) -> Result<u16> {
    if flow == COMPLIANCE_FLOW_PRIMARY_SALE
        && (event_restriction_flags & COMPLIANCE_FLAG_BLOCK_PRIMARY) != 0
    {
        return Ok(COMPLIANCE_DECISION_BLOCK_PRIMARY);
    }
    if flow == COMPLIANCE_FLOW_RESALE
        && (event_restriction_flags & COMPLIANCE_FLAG_BLOCK_RESALE) != 0
    {
        return Ok(COMPLIANCE_DECISION_BLOCK_RESALE);
    }
    if flow == COMPLIANCE_FLOW_FINANCING
        && (event_restriction_flags & COMPLIANCE_FLAG_BLOCK_FINANCING) != 0
    {
        return Ok(COMPLIANCE_DECISION_BLOCK_FINANCING);
    }

    let data = compliance_registry.try_borrow_data()?;
    let mut data_slice: &[u8] = &data;
    if data_slice.len() < 8 || &data_slice[..8] != ComplianceRegistry::DISCRIMINATOR {
        return Ok(COMPLIANCE_DECISION_ALLOW);
    }
    let registry = ComplianceRegistry::try_deserialize(&mut data_slice)?;

    if registry.denylist.iter().any(|x| *x == wallet) {
        return Ok(COMPLIANCE_DECISION_DENYLISTED);
    }
    if registry.entity_denylist.iter().any(|x| *x == entity) {
        return Ok(COMPLIANCE_DECISION_ENTITY_DENIED);
    }
    if (event_restriction_flags & COMPLIANCE_FLAG_REQUIRE_ALLOWLIST) != 0
        && !registry.allowlist.iter().any(|x| *x == wallet)
    {
        return Ok(COMPLIANCE_DECISION_MISSING_ALLOWLIST);
    }
    if (event_restriction_flags & COMPLIANCE_FLAG_JURISDICTION_RESTRICTED) != 0
        && !registry.allowlist.iter().any(|x| *x == wallet)
    {
        return Ok(COMPLIANCE_DECISION_JURISDICTION_RESTRICTED);
    }

    Ok(COMPLIANCE_DECISION_ALLOW)
}
