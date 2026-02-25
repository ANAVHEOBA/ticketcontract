use anchor_lang::prelude::*;

use crate::constants::MAX_RESALE_RECIPIENT_LIST_LEN;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct ResalePolicyInput {
    pub max_markup_bps: u16,
    pub royalty_bps: u16,
    pub royalty_vault: Pubkey,
    pub transfer_cooldown_secs: i64,
    pub max_transfer_count: u16,
    pub transfer_lock_before_event_secs: i64,
    #[max_len(MAX_RESALE_RECIPIENT_LIST_LEN)]
    pub whitelist: Vec<Pubkey>,
    #[max_len(MAX_RESALE_RECIPIENT_LIST_LEN)]
    pub blacklist: Vec<Pubkey>,
}

#[account]
#[derive(InitSpace)]
pub struct ResalePolicy {
    pub bump: u8,
    pub schema_version: u16,
    pub deprecated_layout_version: u16,
    pub replacement_account: Pubkey,
    pub deprecated_at: i64,
    pub event: Pubkey,
    pub ticket_class: Pubkey,
    pub class_id: u16,
    pub max_markup_bps: u16,
    pub royalty_bps: u16,
    pub royalty_vault: Pubkey,
    pub transfer_cooldown_secs: i64,
    pub max_transfer_count: u16,
    pub transfer_lock_before_event_secs: i64,
    #[max_len(MAX_RESALE_RECIPIENT_LIST_LEN)]
    pub whitelist: Vec<Pubkey>,
    #[max_len(MAX_RESALE_RECIPIENT_LIST_LEN)]
    pub blacklist: Vec<Pubkey>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl ResalePolicy {
    pub fn mark_layout_deprecated(
        &mut self,
        deprecated_layout_version: u16,
        replacement_account: Pubkey,
        now: i64,
    ) {
        self.deprecated_layout_version = deprecated_layout_version;
        self.replacement_account = replacement_account;
        self.deprecated_at = now;
    }
}
