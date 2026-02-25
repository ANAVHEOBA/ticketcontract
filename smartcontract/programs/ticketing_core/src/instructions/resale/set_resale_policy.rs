use anchor_lang::prelude::*;

use crate::{
    constants::{
        MAX_RESALE_RECIPIENT_LIST_LEN, RESALE_POLICY_SCHEMA_VERSION, SEED_EVENT, SEED_ORGANIZER,
        SEED_PROTOCOL_CONFIG, SEED_RESALE_POLICY, SEED_TICKET_CLASS,
    },
    error::TicketingError,
    events::ResalePolicySet,
    state::{
        EventAccount, OrganizerProfile, ProtocolConfig, ResalePolicy, ResalePolicyInput,
        TicketClass,
    },
};

pub fn set_resale_policy(
    ctx: Context<SetResalePolicy>,
    class_id: u16,
    input: ResalePolicyInput,
) -> Result<()> {
    require!(
        !ctx.accounts.protocol_config.is_paused,
        TicketingError::ProtocolPaused
    );
    require!(
        input.max_markup_bps <= 10_000,
        TicketingError::InvalidFeeBps
    );
    require!(input.royalty_bps <= 10_000, TicketingError::InvalidFeeBps);
    require!(
        input.whitelist.len() <= MAX_RESALE_RECIPIENT_LIST_LEN,
        TicketingError::InvalidResalePrice
    );
    require!(
        input.blacklist.len() <= MAX_RESALE_RECIPIENT_LIST_LEN,
        TicketingError::InvalidResalePrice
    );
    require!(
        input.transfer_cooldown_secs >= 0 && input.transfer_lock_before_event_secs >= 0,
        TicketingError::InvalidResalePrice
    );
    require!(
        ctx.accounts.ticket_class.is_resale_enabled,
        TicketingError::ResaleDisabled
    );

    let authority = ctx.accounts.authority.key();
    let is_admin = authority == ctx.accounts.protocol_config.admin;
    let is_organizer = authority == ctx.accounts.organizer_profile.authority;
    require!(is_admin || is_organizer, TicketingError::Unauthorized);

    let now = Clock::get()?.unix_timestamp;
    let policy = &mut ctx.accounts.resale_policy;
    if policy.created_at == 0 {
        policy.bump = ctx.bumps.resale_policy;
        policy.schema_version = RESALE_POLICY_SCHEMA_VERSION;
        policy.deprecated_layout_version = 0;
        policy.replacement_account = Pubkey::default();
        policy.deprecated_at = 0;
        policy.event = ctx.accounts.event_account.key();
        policy.ticket_class = ctx.accounts.ticket_class.key();
        policy.class_id = class_id;
        policy.created_at = now;
    }

    policy.max_markup_bps = input.max_markup_bps;
    policy.royalty_bps = input.royalty_bps;
    policy.royalty_vault = input.royalty_vault;
    policy.transfer_cooldown_secs = input.transfer_cooldown_secs;
    policy.max_transfer_count = input.max_transfer_count;
    policy.transfer_lock_before_event_secs = input.transfer_lock_before_event_secs;
    policy.whitelist = input.whitelist;
    policy.blacklist = input.blacklist;
    policy.updated_at = now;

    emit!(ResalePolicySet {
        event: ctx.accounts.event_account.key(),
        ticket_class: ctx.accounts.ticket_class.key(),
        authority,
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(class_id: u16)]
pub struct SetResalePolicy<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub authority: Signer<'info>,
    #[account(
        seeds = [SEED_PROTOCOL_CONFIG],
        bump = protocol_config.bump,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,
    #[account(
        seeds = [SEED_ORGANIZER, organizer_profile.authority.as_ref()],
        bump = organizer_profile.bump,
    )]
    pub organizer_profile: Account<'info, OrganizerProfile>,
    #[account(
        seeds = [SEED_EVENT, organizer_profile.key().as_ref(), &event_account.event_id.to_le_bytes()],
        bump = event_account.bump,
        constraint = event_account.organizer == organizer_profile.key() @ TicketingError::Unauthorized,
    )]
    pub event_account: Account<'info, EventAccount>,
    #[account(
        seeds = [SEED_TICKET_CLASS, event_account.key().as_ref(), &class_id.to_le_bytes()],
        bump = ticket_class.bump,
        constraint = ticket_class.event == event_account.key() @ TicketingError::Unauthorized,
        constraint = ticket_class.class_id == class_id @ TicketingError::Unauthorized,
    )]
    pub ticket_class: Account<'info, TicketClass>,
    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + ResalePolicy::INIT_SPACE,
        seeds = [SEED_RESALE_POLICY, event_account.key().as_ref(), &class_id.to_le_bytes()],
        bump,
    )]
    pub resale_policy: Account<'info, ResalePolicy>,
    pub system_program: Program<'info, System>,
}
