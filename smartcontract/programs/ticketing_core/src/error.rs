use anchor_lang::prelude::*;

#[error_code]
pub enum TicketingError {
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Invalid fee bps")]
    InvalidFeeBps,
    #[msg("Invalid max tickets per wallet")]
    InvalidMaxTicketsPerWallet,
    #[msg("Invalid authority")]
    InvalidAuthority,
    #[msg("Invalid metadata URI length")]
    InvalidMetadataUriLength,
    #[msg("Protocol is paused")]
    ProtocolPaused,
    #[msg("Invalid organizer status")]
    InvalidOrganizerStatus,
    #[msg("Organizer is suspended")]
    OrganizerSuspended,
    #[msg("Invalid event title length")]
    InvalidEventTitleLength,
    #[msg("Invalid event venue length")]
    InvalidEventVenueLength,
    #[msg("Invalid event time window")]
    InvalidEventTimeWindow,
    #[msg("Invalid event capacity")]
    InvalidEventCapacity,
    #[msg("Event update lock has passed")]
    EventUpdateLocked,
    #[msg("Sales already started")]
    SalesAlreadyStarted,
    #[msg("Invalid event status transition")]
    InvalidEventStatusTransition,
    #[msg("Invalid ticket class name length")]
    InvalidTicketClassNameLength,
    #[msg("Invalid ticket class supply")]
    InvalidTicketClassSupply,
    #[msg("Invalid ticket class sale window")]
    InvalidTicketClassSaleWindow,
    #[msg("Invalid ticket class purchase limit")]
    InvalidTicketClassPurchaseLimit,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Invalid reserve amount")]
    InvalidReserveAmount,
    #[msg("Insufficient remaining supply")]
    InsufficientRemainingSupply,
    #[msg("Sale window not active")]
    SaleWindowNotActive,
    #[msg("Purchase limit exceeded")]
    PurchaseLimitExceeded,
    #[msg("Invalid ticket price")]
    InvalidTicketPrice,
    #[msg("Invalid ticket id")]
    InvalidTicketId,
    #[msg("Invalid stakeholder wallet")]
    InvalidStakeholderWallet,
    #[msg("Illegal ticket status transition")]
    IllegalTicketStatusTransition,
    #[msg("Invalid ticket status")]
    InvalidTicketStatus,
    #[msg("Invalid ticket metadata URI length")]
    InvalidTicketMetadataUriLength,
    #[msg("Resale is disabled")]
    ResaleDisabled,
    #[msg("Transfer is not allowed")]
    TransferNotAllowed,
    #[msg("Listing is not active")]
    ListingNotActive,
    #[msg("Invalid resale price")]
    InvalidResalePrice,
    #[msg("Listing exceeds max markup")]
    ListingExceedsMaxMarkup,
    #[msg("Recipient is not in the whitelist")]
    RecipientNotWhitelisted,
    #[msg("Recipient is blacklisted")]
    RecipientBlacklisted,
    #[msg("Transfer cooldown is active")]
    TransferCooldownActive,
    #[msg("Max transfer count exceeded")]
    MaxTransferCountExceeded,
    #[msg("Transfer is locked close to event start")]
    TransferLockedByEventStart,
    #[msg("Invalid listing expiry")]
    InvalidListingExpiry,
    #[msg("Listing has expired")]
    ListingExpired,
    #[msg("Listing is not yet expired")]
    ListingNotYetExpired,
    #[msg("Invalid financing advance amount")]
    InvalidFinancingAdvanceAmount,
    #[msg("Invalid repayment cap")]
    InvalidRepaymentCap,
    #[msg("Invalid financing schedule")]
    InvalidFinancingSchedule,
    #[msg("Invalid financing status")]
    InvalidFinancingStatus,
    #[msg("Financing terms are locked")]
    FinancingTermsLocked,
    #[msg("Financing offer has not been accepted")]
    FinancingOfferNotAccepted,
    #[msg("Financing advance already disbursed")]
    FinancingAlreadyDisbursed,
    #[msg("Financing is currently frozen")]
    FinancingFrozen,
    #[msg("Invalid disbursement amount")]
    InvalidDisbursementAmount,
    #[msg("Disbursement limit exceeded")]
    DisbursementLimitExceeded,
    #[msg("Invalid disbursement reference")]
    InvalidDisbursementReference,
    #[msg("Clawback is not allowed")]
    ClawbackNotAllowed,
    #[msg("Disbursement already clawed back")]
    DisbursementAlreadyClawedBack,
    #[msg("Invalid waterfall bps")]
    InvalidWaterfallBps,
    #[msg("Invalid settlement amount")]
    InvalidSettlementAmount,
    #[msg("Financing obligations not completed")]
    FinancingNotSettled,
    #[msg("Invalid gate identifier")]
    InvalidGateIdentifier,
    #[msg("Ticket already checked in")]
    TicketAlreadyCheckedIn,
    #[msg("Re-entry is not allowed")]
    ReentryNotAllowed,
    #[msg("Re-entry limit exceeded")]
    ReentryLimitExceeded,
    #[msg("Invalid refund amount")]
    InvalidRefundAmount,
    #[msg("Refund is not allowed for this ticket")]
    RefundNotAllowed,
    #[msg("Invalid refund source")]
    InvalidRefundSource,
    #[msg("Ticket is marked disputed")]
    TicketDisputed,
    #[msg("Chargeback requires dispute flag")]
    ChargebackRequiresDispute,
    #[msg("Invalid loyalty points amount")]
    InvalidLoyaltyPoints,
    #[msg("Invalid loyalty accrual reason")]
    InvalidLoyaltyReason,
    #[msg("Invalid loyalty multiplier")]
    InvalidLoyaltyMultiplier,
    #[msg("Insufficient loyalty points")]
    InsufficientLoyaltyPoints,
    #[msg("Invalid loyalty perk code")]
    InvalidLoyaltyPerkCode,
    #[msg("Trust signal already recorded for this ticket")]
    TrustSignalAlreadyRecorded,
    #[msg("Invalid trust signal flag bits")]
    InvalidTrustFlagBits,
    #[msg("Invalid trust signal schema version")]
    InvalidTrustSchemaVersion,
}
