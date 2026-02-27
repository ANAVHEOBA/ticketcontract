# Backend Features (Ticketing + Financial Control Layer)

## Total Scope
- **22 backend feature modules** (grouped by core platform areas).
- For hackathon delivery, implement in phases: `P0 demo-critical`, `P1 strong`, `P2 stretch`.

## 1. Platform Foundation (P0)
- Service bootstrap/config (`env`, chain RPC, DB, signer config).
- Health/readiness endpoints.
- Structured logging + request tracing.
- Error mapping (chain errors -> API-safe responses).

## 2. Auth & Access (P0)
- Wallet-based auth (SIWS / signed nonce).
- Role model: protocol admin, organizer admin, operator, scanner, financier.
- API guards by role + organizer scope.

## 3. Chain Client Module (P0)
- Anchor client initialization.
- PDA derivation helpers.
- Transaction builder + submit + confirm wrapper.
- Simulation endpoint for preflight/debug.

## 4. Protocol Admin API (P1)
- Initialize protocol config.
- Pause/unpause protocol.
- Set protocol config/vaults/authorities.
- Governance hook calls (timelock, multisig config, upgrade handoff).

## 5. Organizer API (P0)
- Create organizer profile.
- Update metadata/payout wallet.
- Set organizer status + compliance flags.
- Delegate operator permissions.

## 6. Event API (P0)
- Create event.
- Update/freeze/cancel event.
- Event read endpoints with status and policy snapshots.

## 7. Ticket Class API (P0)
- Create/update ticket classes.
- Reserve inventory.
- Class analytics read (supply sold/remaining, pacing).

## 8. Primary Sale API (P0)
- Buy ticket orchestration.
- Comp issuance path.
- Purchase receipt payload (signature, ticket PDA, amounts).

## 9. Ticket State API (P1)
- Get ticket details/lifecycle state.
- Update metadata pointer/version.
- Controlled status transitions (admin/operator paths).

## 10. Resale Policy API (P0)
- Set/get policy per event/class.
- Optional policy recommendation write path.
- Policy validation helper endpoint.

## 11. Secondary Sale API (P0)
- List ticket.
- Buy resale ticket.
- Cancel/expire listing.

## 12. Financing API (P0)
- Create financing offers.
- Accept/reject financing terms.
- Disburse advance / clawback / freeze controls.

## 13. Settlement API (P0)
- Settle primary revenue.
- Settle resale revenue.
- Finalize settlement.
- Idempotent settlement reference handling.

## 14. Check-in API (P1)
- Set class check-in policy.
- Check-in ticket endpoint for scanner apps.
- Gate-level check-in response payloads.

## 15. Disputes/Refund API (P1)
- Refund ticket under allowed conditions.
- Flag dispute / chargeback.
- Query disputed ticket queues.

## 16. Loyalty + Trust API (P1)
- Accrue/redeem loyalty points.
- Record purchase/attendance trust signals.
- Trust schema/version admin hooks.

## 17. Indexer Worker (P0)
- Subscribe/poll program events + account changes.
- Persist canonical history into Postgres.
- Backfill mode from slot range.

## 18. Data Models & Storage (P0)
- Tables: organizers, events, classes, tickets, listings, financing, disbursements, settlements, loyalty, trust.
- Materialized views for dashboard KPIs.
- Migrations + seed scripts.

## 19. Underwriting Engine (P1)
- Rule-based risk scoring from indexed history.
- Financing term proposal (advance %, fee, cap, schedule).
- Explainability payload for organizer dashboard.

## 20. Resale Compiler / Simulator (P1)
- Simulate outcomes for policy candidates.
- Optimize for organizer goals (liquidity, fan fairness, royalty).
- Return recommended policy + confidence score.

## 21. Observability & Ops (P1)
- Metrics (tx success rate, confirmation latency, queue lag).
- Alerting thresholds (failed submissions, indexer lag, pause state).
- Admin audit logs (who triggered sensitive actions).

## 22. Delivery Tooling (P0)
- OpenAPI docs + Postman/Bruno collection.
- End-to-end backend integration test (mirrors smart contract E2E).
- Devnet deploy + smoke scripts.

---

## Hackathon Build Order
1. `P0`: modules 1,2,3,5,6,7,8,10,11,12,13,17,18,22.
2. `P1`: modules 4,9,14,15,16,19,20,21.
3. `P2`: advanced compliance/KYC adapters, oracle enrichments, model upgrades.

## Recommended Milestone Target
- **Week 1:** P0 write paths + indexer storage.
- **Week 2:** dashboard read APIs + underwriting/resale simulator basics.
- **Week 3:** reliability hardening + full demo flow + pitch metrics.
