use anchor_lang::prelude::*;

use crate::{
    constants::{
        MAX_TICKET_CLASS_NAME_LEN, SEED_EVENT, SEED_ORGANIZER, SEED_PROTOCOL_CONFIG,
        SEED_TICKET_CLASS,
    },
    error::TicketingError,
    state::{EventAccount, EventStatus, OrganizerProfile, ProtocolConfig, TicketClass},
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TicketClassInput {
    pub name: String,
    pub total_supply: u32,
    pub reserved_supply: u32,
    pub face_price_lamports: u64,
    pub sale_start_ts: i64,
    pub sale_end_ts: i64,
    pub per_wallet_limit: u16,
    pub is_transferable: bool,
    pub is_resale_enabled: bool,
    pub stakeholder_wallet: Pubkey,
    pub stakeholder_bps: u16,
}

pub fn create_ticket_class(
    ctx: Context<CreateTicketClass>,
    class_id: u16,
    input: TicketClassInput,
) -> Result<()> {
    validate_ticket_class_input(&input)?;

    require!(
        !ctx.accounts.protocol_config.is_paused,
        TicketingError::ProtocolPaused
    );
    require!(
        ctx.accounts.event_account.status == EventStatus::Draft
            || ctx.accounts.event_account.status == EventStatus::Frozen,
        TicketingError::InvalidEventStatusTransition
    );

    let remaining_supply = input
        .total_supply
        .checked_sub(input.reserved_supply)
        .ok_or(TicketingError::InvalidTicketClassSupply)?;

    let now = Clock::get()?.unix_timestamp;
    let ticket_class = &mut ctx.accounts.ticket_class;
    ticket_class.bump = ctx.bumps.ticket_class;
    ticket_class.event = ctx.accounts.event_account.key();
    ticket_class.class_id = class_id;
    ticket_class.name = input.name;
    ticket_class.total_supply = input.total_supply;
    ticket_class.reserved_supply = input.reserved_supply;
    ticket_class.sold_supply = 0;
    ticket_class.refunded_supply = 0;
    ticket_class.remaining_supply = remaining_supply;
    ticket_class.face_price_lamports = input.face_price_lamports;
    ticket_class.sale_start_ts = input.sale_start_ts;
    ticket_class.sale_end_ts = input.sale_end_ts;
    ticket_class.per_wallet_limit = input.per_wallet_limit;
    ticket_class.is_transferable = input.is_transferable;
    ticket_class.is_resale_enabled = input.is_resale_enabled;
    ticket_class.allow_reentry = false;
    ticket_class.max_reentries = 0;
    ticket_class.stakeholder_wallet = input.stakeholder_wallet;
    ticket_class.stakeholder_bps = input.stakeholder_bps;
    ticket_class.created_at = now;
    ticket_class.updated_at = now;

    Ok(())
}

pub fn validate_ticket_class_input(input: &TicketClassInput) -> Result<()> {
    require!(
        !input.name.is_empty() && input.name.len() <= MAX_TICKET_CLASS_NAME_LEN,
        TicketingError::InvalidTicketClassNameLength
    );
    require!(
        input.total_supply > 0,
        TicketingError::InvalidTicketClassSupply
    );
    require!(
        input.reserved_supply <= input.total_supply,
        TicketingError::InvalidTicketClassSupply
    );
    require!(
        input.sale_start_ts < input.sale_end_ts,
        TicketingError::InvalidTicketClassSaleWindow
    );
    require!(
        input.per_wallet_limit > 0,
        TicketingError::InvalidTicketClassPurchaseLimit
    );
    require!(
        input.stakeholder_bps <= 10_000,
        TicketingError::InvalidFeeBps
    );

    Ok(())
}

#[derive(Accounts)]
#[instruction(class_id: u16)]
pub struct CreateTicketClass<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub authority: Signer<'info>,
    #[account(
        seeds = [SEED_PROTOCOL_CONFIG],
        bump = protocol_config.bump,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,
    #[account(
        seeds = [SEED_ORGANIZER, authority.key().as_ref()],
        bump = organizer_profile.bump,
        constraint = organizer_profile.authority == authority.key() @ TicketingError::Unauthorized,
    )]
    pub organizer_profile: Account<'info, OrganizerProfile>,
    #[account(
        seeds = [SEED_EVENT, organizer_profile.key().as_ref(), &event_account.event_id.to_le_bytes()],
        bump = event_account.bump,
        constraint = event_account.organizer == organizer_profile.key() @ TicketingError::Unauthorized,
    )]
    pub event_account: Account<'info, EventAccount>,
    #[account(
        init,
        payer = payer,
        space = 8 + TicketClass::INIT_SPACE,
        seeds = [SEED_TICKET_CLASS, event_account.key().as_ref(), &class_id.to_le_bytes()],
        bump,
    )]
    pub ticket_class: Account<'info, TicketClass>,
    pub system_program: Program<'info, System>,
}
