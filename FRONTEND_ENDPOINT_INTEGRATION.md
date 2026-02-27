# Frontend Endpoint Integration Status

Legend:
- `[x]` Integrated in frontend flow
- `[ ]` Not integrated yet
- `[~]` Partially integrated / fallback-based

## Auth & Access
- `[x]` `POST /api/v1/auth/nonce`
- `[x]` `POST /api/v1/auth/verify`
- `[x]` `POST /api/v1/auth/provider/verify` (Google + wallet flow)
- `[ ]` `GET /api/v1/auth/me`
- `[ ]` `GET /api/v1/auth/organizers/{organizer_id}/access`

## Relay / Sponsored Gas
- `[x]` `POST /api/v1/relay/submit`
  - Used for create organizer, create event, create ticket class, buy ticket.

## Events
- `[x]` `GET /api/v1/events` (dashboard listing)
- `[x]` `GET /api/v1/events/{event_id}` (event detail tries API first, then local fallback if indexer/DB is behind)
- `[x]` `POST /api/v1/events` (direct create path integrated with fallback to sponsored relay)
- `[x]` `POST /api/v1/events/simulate`
- `[x]` `POST /api/v1/events/update`
- `[x]` `POST /api/v1/events/update/simulate`
- `[x]` `POST /api/v1/events/freeze`
- `[x]` `POST /api/v1/events/freeze/simulate`
- `[x]` `POST /api/v1/events/cancel`
- `[x]` `POST /api/v1/events/cancel/simulate`
- `[x]` `POST /api/v1/events/pause`
- `[x]` `POST /api/v1/events/pause/simulate`
- `[x]` `POST /api/v1/events/close`
- `[x]` `POST /api/v1/events/close/simulate`
- `[x]` `POST /api/v1/events/restrictions`
- `[x]` `POST /api/v1/events/restrictions/simulate`
- `[x]` `POST /api/v1/events/loyalty-multiplier`
- `[x]` `POST /api/v1/events/loyalty-multiplier/simulate`

## Ticket Classes
- `[x]` `GET /api/v1/ticket-classes` (event detail class selector)
- `[x]` `POST /api/v1/ticket-classes` (direct create path integrated; create flow still has relay fallback)
- `[x]` `POST /api/v1/ticket-classes/simulate`
- `[x]` `POST /api/v1/ticket-classes/update`
- `[x]` `POST /api/v1/ticket-classes/update/simulate`
- `[x]` `POST /api/v1/ticket-classes/reserve`
- `[x]` `POST /api/v1/ticket-classes/reserve/simulate`
- `[x]` `GET /api/v1/ticket-classes/{class_id}`
- `[x]` `GET /api/v1/ticket-classes/{class_id}/analytics`

## Primary Sale
- `[x]` `POST /api/v1/primary-sale/buy` (event detail write path now calls endpoint first; relay fallback kept for compatibility)
- `[x]` `POST /api/v1/primary-sale/buy/simulate`
- `[x]` `POST /api/v1/primary-sale/comp`
- `[x]` `POST /api/v1/primary-sale/comp/simulate`

## Ticket State
- `[x]` `GET /api/v1/tickets/{ticket_id}`
- `[x]` `POST /api/v1/tickets/metadata` (frontend calls `/tickets/metadata` with fallback to legacy `/ticket-state/metadata`)
- `[x]` `POST /api/v1/tickets/metadata/simulate` (same legacy fallback behavior)
- `[x]` `POST /api/v1/tickets/status` (frontend calls `/tickets/status` with fallback to legacy `/ticket-state/transition`)
- `[x]` `POST /api/v1/tickets/status/simulate` (wired; backend may still expose legacy simulate route depending on version)

## Organizer
- `[x]` `POST /api/v1/organizers` (create flow now calls direct endpoint first, then relay fallback if unavailable)
- `[x]` `POST /api/v1/organizers/simulate`
- `[x]` `POST /api/v1/organizers/update`
- `[x]` `POST /api/v1/organizers/update/simulate`
- `[x]` `POST /api/v1/organizers/status`
- `[x]` `POST /api/v1/organizers/status/simulate`
- `[x]` `POST /api/v1/organizers/compliance-flags`
- `[x]` `POST /api/v1/organizers/compliance-flags/simulate`
- `[x]` `POST /api/v1/organizers/operators`
- `[x]` `POST /api/v1/organizers/operators/simulate`
- `[x]` `GET /api/v1/organizers/{organizer_id}`

## Secondary Sale
- `[x]` `POST /api/v1/secondary-sale/list`
- `[x]` `POST /api/v1/secondary-sale/list/simulate`
- `[x]` `POST /api/v1/secondary-sale/buy`
- `[x]` `POST /api/v1/secondary-sale/buy/simulate`
- `[x]` `POST /api/v1/secondary-sale/cancel`
- `[x]` `POST /api/v1/secondary-sale/cancel/simulate`
- `[x]` `POST /api/v1/secondary-sale/expire`
- `[x]` `POST /api/v1/secondary-sale/expire/simulate`
- `[x]` `GET /api/v1/secondary-sale/listings/{listing_id}`

## Resale Policy
- `[x]` `POST /api/v1/resale-policy`
- `[x]` `POST /api/v1/resale-policy/simulate`
- `[x]` `POST /api/v1/resale-policy/recommendation`
- `[x]` `POST /api/v1/resale-policy/validate`
- `[x]` `GET /api/v1/resale-policy/{policy_id}` (UI attempts path route first; local backend may still require query fallback read)

## Financing
- `[x]` `POST /api/v1/financing/offers`
- `[x]` `POST /api/v1/financing/offers/simulate`
- `[x]` `POST /api/v1/financing/offers/accept`
- `[x]` `POST /api/v1/financing/offers/accept/simulate`
- `[x]` `POST /api/v1/financing/offers/reject`
- `[x]` `POST /api/v1/financing/offers/reject/simulate`
- `[x]` `POST /api/v1/financing/disburse`
- `[x]` `POST /api/v1/financing/disburse/simulate`
- `[x]` `POST /api/v1/financing/clawback`
- `[x]` `POST /api/v1/financing/clawback/simulate`
- `[x]` `POST /api/v1/financing/freeze`
- `[x]` `POST /api/v1/financing/freeze/simulate`
- `[x]` `GET /api/v1/financing/offers/{offer_id}`

## Settlement
- `[x]` `POST /api/v1/settlement/primary`
- `[x]` `POST /api/v1/settlement/primary/simulate`
- `[x]` `POST /api/v1/settlement/resale`
- `[x]` `POST /api/v1/settlement/resale/simulate`
- `[x]` `POST /api/v1/settlement/finalize`
- `[x]` `POST /api/v1/settlement/finalize/simulate`
- `[~]` `GET /api/v1/settlement/{settlement_ref}` (frontend call wired; backend route may be unavailable in current build)

## Check-in
- `[x]` `POST /api/v1/checkin/policy`
- `[x]` `POST /api/v1/checkin/ticket`
- `[~]` `POST /api/v1/checkin/gate-response` (frontend call wired; backend route may be unavailable in current build)

## Disputes / Refund
- `[x]` `POST /api/v1/disputes/refund`
- `[x]` `POST /api/v1/disputes/flag`
- `[x]` `POST /api/v1/disputes/chargeback`
- `[x]` `GET /api/v1/disputes/queue`

## Loyalty + Trust
- `[x]` `POST /api/v1/loyalty/accrue`
- `[x]` `POST /api/v1/loyalty/redeem`
- `[x]` `POST /api/v1/trust/purchase`
- `[x]` `POST /api/v1/trust/attendance`
- `[x]` `POST /api/v1/trust/abuse`
- `[x]` `POST /api/v1/trust/schema-version`
- `[x]` `GET /api/v1/loyalty`
- `[x]` `GET /api/v1/trust/signals`

## Underwriting / Resale Compiler
- `[~]` `POST /api/v1/underwriting/score` (frontend calls score path first, then falls back to `/underwriting/financing/proposal`)
- `[x]` `POST /api/v1/resale-compiler/simulate`

## Indexer / KPI Reads
- `[x]` `GET /api/v1/indexer/status`
- `[~]` `GET /api/v1/kpis/event-sales` (frontend supports both `/kpis/event-sales/{event_id}` and checklist-style query path)
- `[~]` `GET /api/v1/kpis/fan-quality` (frontend call wired; backend route may be unavailable in current build)
- `[x]` `GET /api/v1/kpis/financing-cash`

## Ops / Admin / Delivery
- `[x]` `GET /api/v1/ops/metrics`
- `[x]` `GET /api/v1/ops/alerts`
- `[x]` `GET /api/v1/ops/audit-logs`
- `[x]` `GET /api/v1/docs/openapi.yaml`
- `[~]` `GET /api/v1/docs/postman` (frontend tries alias path then falls back to `/docs/postman_collection.json`)
- `[~]` `GET /api/v1/docs/bruno` (frontend tries alias path then falls back to `/docs/bruno_collection.json`)

## Current Frontend Feature State (High-Level)
- `[x]` Sign-in (Google + Wallet)
- `[x]` Dashboard event cards
- `[x]` Event detail page
- `[x]` Buy ticket with sponsored gas (relayer)
- `[x]` Tx status states + explorer links
- `[x]` Purchase receipt panel
- `[x]` My Tickets page (on-chain ticket account read)
- `[~]` DB/indexer-backed event reads (still inconsistent in local dev)
