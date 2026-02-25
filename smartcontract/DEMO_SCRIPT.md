# Demo Script (Hackathon)

## Goal
Show full ticket lifecycle + financing + settlement on a single deterministic flow.

## Commands
1. `./scripts/build.sh`
2. `./scripts/idl-client-sanity.sh`
3. `cargo test -p ticketing_core --test integration end_to_end:: -- --nocapture`

## What the E2E Test Demonstrates
- Protocol + organizer + event + ticket class initialization
- Primary purchase with payment routing
- Ticket metadata update
- Resale policy set, listing, and secondary purchase
- Check-in and loyalty accrual
- Financing offer create/accept + advance disbursement
- Revenue waterfall settlement + financing finalization
- Pause gate blocking new financing mutation

## Success Criteria
- Test exits `ok` with no failed assertions.
- Logs show all critical instruction invocations in sequence.
