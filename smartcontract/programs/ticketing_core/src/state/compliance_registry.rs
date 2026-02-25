use anchor_lang::prelude::*;

use crate::constants::MAX_COMPLIANCE_REGISTRY_ENTRIES;

#[account]
#[derive(InitSpace)]
pub struct ComplianceRegistry {
    pub bump: u8,
    pub scope: u8,
    pub target: Pubkey,
    #[max_len(MAX_COMPLIANCE_REGISTRY_ENTRIES)]
    pub allowlist: Vec<Pubkey>,
    #[max_len(MAX_COMPLIANCE_REGISTRY_ENTRIES)]
    pub denylist: Vec<Pubkey>,
    #[max_len(MAX_COMPLIANCE_REGISTRY_ENTRIES)]
    pub entity_denylist: Vec<Pubkey>,
    pub updated_at: i64,
}
