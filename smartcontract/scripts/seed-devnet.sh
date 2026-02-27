#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

ADMIN_KEYPAIR="${ADMIN_KEYPAIR:-$HOME/.config/solana/id.json}"
CLUSTER="${CLUSTER:-https://api.devnet.solana.com}"

echo "[seed] cluster=$CLUSTER admin_keypair=$ADMIN_KEYPAIR"
echo "[seed] this script funds local signer and verifies program visibility"

solana config set --url "$CLUSTER" >/dev/null
solana airdrop 2 --keypair "$ADMIN_KEYPAIR" --url "$CLUSTER" || true
solana balance --keypair "$ADMIN_KEYPAIR" --url "$CLUSTER"
solana program show Evb4oQnwYDZhabE5E5tuJPg2yvCMGgwMRgkxzzaUw5Rv --url "$CLUSTER"

echo "[seed] complete"
