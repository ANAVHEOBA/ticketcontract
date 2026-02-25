# Hardening Report

## Scope
- End-to-end flow validation
- Account sizing / stack guard checks
- Failure-path and privilege guard verification
- IDL/client compatibility checks

## Checks Implemented
- `scripts/build.sh`
  - Runs `anchor build`
  - Fails if SBF stack overflow warning is detected (`Stack offset ... exceeded`)
- `scripts/test.sh`
  - Runs full integration suite + focused security/governance/e2e filters
- `scripts/idl-client-sanity.sh`
  - Rebuilds/syncs IDL
  - Validates required backend + frontend instruction paths exist in IDL

## Critical Tests Added
- `programs/ticketing_core/tests/end_to_end/mod.rs`
  - Primary sale -> resale -> check-in -> loyalty -> financing -> disbursement -> settlement -> finalization -> pause gate
- `programs/ticketing_core/tests/schema_versioning/mod.rs`
  - Schema version fields and backward-compatible deserialization for v0 layouts

## Known Residual Work Before Mainnet
- Add explicit migration instructions for in-place upgrades (compat readers are present).
- Add formal property/fuzz tests for settlement arithmetic.
- Add performance benchmark budget around max-size recipient/policy lists.
