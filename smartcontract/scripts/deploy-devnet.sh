#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

PROGRAM_KEYPAIR="${PROGRAM_KEYPAIR:-$ROOT_DIR/target/deploy/ticketing_core-keypair.json}"
PROGRAM_ID="${PROGRAM_ID:-Evb4oQnwYDZhabE5E5tuJPg2yvCMGgwMRgkxzzaUw5Rv}"

echo "[deploy] checking solana CLI context"
solana config get
solana address

echo "[deploy] ensuring build artifacts"
./scripts/build.sh

echo "[deploy] deploying to devnet with keypair=$PROGRAM_KEYPAIR"
anchor deploy \
  --program-name ticketing_core \
  --provider.cluster devnet \
  --program-keypair "$PROGRAM_KEYPAIR"

echo "[deploy] verifying program exists on devnet"
solana program show "$PROGRAM_ID" --url https://api.devnet.solana.com

echo "[deploy] syncing IDL artifact"
./scripts/idl-sync.sh

echo "[deploy] done"
