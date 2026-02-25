# Security Review Checklist

## Authority Edges
- Protocol admin-only actions enforced by PDA constraints and explicit `Unauthorized` checks.
- Organizer authority required for organizer/event-scoped mutations.
- Governance hooks support multisig signoff and optional timelock queue/execute path.
- Upgrade handoff requires pending authority acceptance (no unilateral take-over).

## Vault Movement Controls
- Primary sale routes lamports to protocol fee, organizer payout, and optional stakeholder wallet with explicit wallet checks.
- Resale routes lamports to seller and royalty vault with policy-based constraints.
- Financing disbursement enforces accepted terms, authority check, amount bounds, and disbursement count cap.
- Settlement enforces waterfall math checks, settlement idempotency reference, and per-leg transfer accounting.

## Pause / Circuit Breaker
- Protocol pause blocks core mutation paths (ticket sale/resale/financing etc.).
- Event pause checked in event-scoped execution paths.
- Emergency admin procedures require paused protocol and emit events.

## Bypass Attempt Coverage
- Timelock bypass attempt covered by governance hook tests (`direct set_protocol_config` when timelock active).
- Multisig bypass attempt covered by governance hook tests (missing cosigner fails).
- Unauthorized privileged calls covered in global config/organizer/governance tests.
- Settlement replay guarded by idempotent settlement reference checks.

## Remaining Risk Items (Track Before Mainnet)
- Add fuzz/property tests around watermark and distribution arithmetic.
- Add explicit CPI allowlist policy if introducing external token/oracle CPIs.
- Add formal account migration instructions for live upgrades (compat reads are present now).
- Add off-chain signer policy for operational key rotation and emergency playbooks.
