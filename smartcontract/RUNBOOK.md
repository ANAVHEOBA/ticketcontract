# Ticketing Core Runbook

## 1) Prerequisites
- `solana`, `anchor`, `cargo`, `node` available in `PATH`
- wallet keypair at `~/.config/solana/id.json`
- from project root: `/home/a/ticketcontract/smartcontract`

## 2) Local Hardening Pipeline
1. `./scripts/build.sh`
2. `./scripts/idl-client-sanity.sh`
3. `./scripts/test.sh`

## 3) Local End-to-End Demo
1. Start validator in another terminal:
   - `./scripts/test-validator.sh`
2. Run full flow test:
   - `cargo test -p ticketing_core --test integration end_to_end:: -- --nocapture`

## 4) Devnet Deploy
1. Verify cluster and signer:
   - `solana config set --url https://api.devnet.solana.com`
   - `solana address`
2. Deploy and verify:
   - `./scripts/deploy-devnet.sh`
3. Optional seed/funding helper:
   - `./scripts/seed-devnet.sh`

## 5) Artifacts
- Program binary: `target/deploy/ticketing_core.so`
- IDL source: `target/idl/ticketing_core.json`
- IDL synced for clients: `app/idl/ticketing_core.json`
