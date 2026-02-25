use anchor_lang::prelude::*;

use crate::{
    constants::{MAX_TRUST_FLAG_BITS, SEED_PROTOCOL_CONFIG, SEED_TRUST_SIGNAL},
    error::TicketingError,
    events::TrustSignalUpdated,
    state::{ProtocolConfig, TrustSignal},
};

pub fn flag_abuse(
    ctx: Context<FlagAbuse>,
    flag_bits: u32,
    event: Pubkey,
    ticket: Pubkey,
) -> Result<()> {
    require!(flag_bits > 0, TicketingError::InvalidTrustFlagBits);
    require!(
        flag_bits <= MAX_TRUST_FLAG_BITS,
        TicketingError::InvalidTrustFlagBits
    );

    let now = Clock::get()?.unix_timestamp;
    let signal = &mut ctx.accounts.trust_signal;
    if signal.wallet == Pubkey::default() {
        signal.bump = ctx.bumps.trust_signal;
        signal.wallet = ctx.accounts.wallet.key();
        signal.schema_version = 1;
        signal.total_tickets_purchased = 0;
        signal.attendance_eligible_count = 0;
        signal.attendance_attended_count = 0;
        signal.abuse_flags = 0;
        signal.abuse_incidents = 0;
        signal.last_event = Pubkey::default();
        signal.last_ticket = Pubkey::default();
        signal.created_at = now;
    }

    let new_bits = flag_bits & !signal.abuse_flags;
    signal.abuse_flags |= flag_bits;
    signal.abuse_incidents = signal
        .abuse_incidents
        .checked_add(new_bits.count_ones() as u16)
        .ok_or(TicketingError::MathOverflow)?;
    signal.last_event = event;
    signal.last_ticket = ticket;
    signal.updated_at = now;

    emit!(TrustSignalUpdated {
        wallet: signal.wallet,
        trust_signal: signal.key(),
        event,
        ticket,
        schema_version: signal.schema_version,
        update_type: 3,
        total_tickets_purchased: signal.total_tickets_purchased,
        attendance_eligible_count: signal.attendance_eligible_count,
        attendance_attended_count: signal.attendance_attended_count,
        abuse_flags: signal.abuse_flags,
        abuse_incidents: signal.abuse_incidents,
        at: now,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct FlagAbuse<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    pub wallet: SystemAccount<'info>,
    #[account(
        seeds = [SEED_PROTOCOL_CONFIG],
        bump = protocol_config.bump,
        constraint = protocol_config.admin == admin.key() @ TicketingError::Unauthorized,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,
    #[account(
        init_if_needed,
        payer = admin,
        space = 8 + TrustSignal::INIT_SPACE,
        seeds = [SEED_TRUST_SIGNAL, wallet.key().as_ref()],
        bump,
    )]
    pub trust_signal: Account<'info, TrustSignal>,
    pub system_program: Program<'info, System>,
}
