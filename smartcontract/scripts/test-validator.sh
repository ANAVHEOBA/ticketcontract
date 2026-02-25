#!/usr/bin/env bash
set -euo pipefail

LEDGER_DIR="${SOLANA_TEST_LEDGER:-/tmp/ticketing-core-ledger}"
RPC_PORT="${SOLANA_TEST_RPC_PORT:-8899}"

echo "[validator] ledger=$LEDGER_DIR rpc_port=$RPC_PORT"
exec solana-test-validator \
  --reset \
  --ledger "$LEDGER_DIR" \
  --rpc-port "$RPC_PORT" \
  --faucet-port "$((RPC_PORT + 1))"
