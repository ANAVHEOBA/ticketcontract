# Smart Contract Features by Module

## 1. Global Config Module
- Initialize protocol config account.
- Set protocol admin and upgrade authority constraints.
- Register treasury/fee vaults.
- Configure protocol-level fees and limits.
- Pause/unpause protocol actions by flag.

## 2. Organizer Module
- Create organizer profile account.
- Update organizer metadata and payout wallet.
- Set organizer status (active, suspended).
- Delegate organizer operators with scoped permissions.
- Store compliance flags for gating privileged actions.

## 3. Event Module
- Create event account linked to organizer.
- Store event metadata (title, venue, timestamp window, capacity).
- Update mutable event fields before lock time.
- Freeze event config before sales start.
- Cancel/close event with state transitions.

## 4. Ticket Class Module
- Create ticket classes per event (GA/VIP/custom).
- Set inventory, face price, sale window, purchase limits.
- Set class-level transferability and resale flags.
- Reserve allocations (partners, team, guest list).
- Track remaining supply counters safely.

## 5. Primary Sale Module
- Purchase/mint ticket with payment validation.
- Enforce sale window, inventory, per-wallet limits.
- Route payment splits (organizer/protocol/other stakeholders).
- Emit purchase/mint events for indexing.
- Support admin/organizer comp issuance paths.

## 6. Ticket State Module
- Maintain canonical on-chain ticket state.
- Track owner, class, status, and lifecycle timestamps.
- Support status transitions (active, checked-in, refunded, invalidated).
- Prevent illegal transitions via strict checks.
- Support ticket metadata pointer/version fields.

## 7. Transfer & Resale Policy Module
- Set event/class resale policy accounts.
- Enforce max markup on secondary sales.
- Enforce royalty distribution on resale.
- Enforce whitelist/blacklist recipient rules.
- Enforce transfer cooldowns and max transfer count.
- Enforce optional time-based transfer locks (e.g., close to event start).

## 8. Secondary Sale Execution Module
- List ticket for resale with asking price and expiry.
- Execute resale with policy + payment checks.
- Atomically transfer ownership and distribute proceeds.
- Cancel/expire listing safely.
- Emit resale settlement events.

## 9. Financing Terms Module
- Create financing offer account for an event.
- Store offer terms (advance amount, fee, repayment cap, schedule).
- Organizer accepts/rejects financing terms on-chain.
- Lock financing terms after acceptance.
- Track financing lifecycle status.

## 10. Disbursement Module
- Disburse advance funds to organizer vault/wallet based on accepted terms.
- Record disbursement amount, timestamp, and reference IDs.
- Enforce one-time or tranche-based disbursement constraints.
- Support authorized clawback/freeze conditions per protocol rules.

## 11. Revenue Waterfall Module
- Route primary + secondary revenues through waterfall.
- Priority 1: financier repayment until cap reached.
- Priority 2: organizer payout.
- Priority 3: royalties/protocol fees/other shares.
- Track cumulative paid amounts per beneficiary.
- Mark financing as settled when obligations are completed.

## 12. Check-In Module
- Check in ticket by authorized scanner/operator.
- Enforce single-use entry semantics.
- Support optional re-entry policy flags.
- Record check-in timestamp and gate identifier.
- Emit attendance events for downstream analytics.

## 13. Refund & Chargeback Control Module
- Trigger refunds under allowed conditions.
- Reverse ownership/state with consistent accounting.
- Apply refund source rules (organizer vault, escrow, reserve).
- Mark disputed/chargeback-linked tickets for risk handling.
- Prevent transfer/resale for disputed states.

## 14. Loyalty Points Module
- Create per-wallet loyalty ledger PDA.
- Accrue points on purchase, attendance, and hold duration.
- Burn/redeem points for perks entitlements.
- Support event-specific and global point multipliers.
- Emit loyalty events for client display.

## 15. Trust Signal Module
- Store immutable behavior counters per wallet (attendance rate inputs, abuse flags).
- Update trust signals only by authorized program paths.
- Version trust-signal schema for future model compatibility.
- Expose compact trust data accounts for off-chain scorers.

## 16. Access Control & Roles Module
- Define role accounts (protocol admin, organizer admin, operator, scanner).
- Enforce role-based instruction guards.
- Support time-bound role grants and revocations.
- Record role change events for auditability.

## 17. Treasury & Vault Module
- Manage PDAs for protocol, organizer, event, and financing vaults.
- Enforce signer seeds and vault authority checks.
- Support controlled withdrawals with role constraints.
- Maintain vault balance accounting snapshots.

## 18. Compliance Guardrails Module
- Maintain allowlist/denylist registries (wallets/entities).
- Gate high-risk instructions behind compliance checks.
- Enforce jurisdiction/event-level restriction flags.
- Record compliance decision codes on relevant instructions.

## 19. Security & Safety Module
- Reentrancy-safe execution patterns for CPIs and transfers.
- Overflow/underflow-safe arithmetic on all balances/counters.
- Deterministic account sizing and rent checks.
- Circuit-breaker pause at protocol/event levels.
- Idempotency guards for critical settlement paths.

## 20. Event Emission & Audit Module
- Emit structured events for all critical state transitions.
- Include correlation IDs for cross-service reconciliation.
- Emit financial ledger events for each distribution leg.
- Maintain on-chain audit references for governance actions.

## 21. Upgrade & Governance Hooks Module
- Controlled upgrade authority handoff process.
- Optional multisig gating for privileged instructions.
- Timelock hooks for sensitive config changes.
- Emergency admin procedures with full event logging.

## 22. Account Schema & Versioning Module
- Version all major accounts (event, ticket, policy, financing, loyalty).
- Provide migration-safe schema evolution strategy.
- Backward-compatible deserialization for old versions.
- Explicit deprecation paths for replaced account layouts.



• Deployment is done and verified on devnet.

  - PROGRAM_ID: Evb4oQnwYDZhabE5E5tuJPg2yvCMGgwMRgkxzzaUw5Rv
  - Deploy tx signature: 3br7WfL2SXaTxxbrHERbeir7C6v2dLkdmQeeT7cJu8Xepvdk7VXKFZJnMZjdPYZ8WtbWQpxcjRjJvitikF3xLmUQ
  - IDL account: 4YNrpoWStQZHyDLd9fX36FgUGu13RaHdqb7m9PrMS2jJ
  - Verified on-chain owner: BPFLoaderUpgradeab1e11111111111111111111111
  - Last deployed slot: 444799412
