#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO_DIR="$(cd "$ROOT_DIR/.." && pwd)"

if [[ -x "$REPO_DIR/smartcontract/scripts/deploy-devnet.sh" ]]; then
  echo "Deploying smart contract to devnet..."
  (cd "$REPO_DIR/smartcontract" && ./scripts/deploy-devnet.sh)
else
  echo "Smart contract deploy script not found at smartcontract/scripts/deploy-devnet.sh"
  exit 1
fi

echo "Backend deploy prerequisites ready."
echo "Set PROGRAM_ID and start backend with devnet env:"
echo "  cargo run"
