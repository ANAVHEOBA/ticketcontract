#!/usr/bin/env bash
set -euo pipefail

# Create a new relayer wallet and move SOL from an old keypair.
# Usage:
#   SOURCE_KEYPAIR=/home/a/.config/solana/id.json \
#   TRANSFER_SOL=2 \
#   ./scripts/rotate-relayer-wallet.sh
#
# Optional:
#   DEST_KEYPAIR=/home/a/.config/solana/relayer.json
#   CLUSTER_URL=https://api.devnet.solana.com
#   KEEP_MIN_SOL=0.01

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

SOURCE_KEYPAIR="${SOURCE_KEYPAIR:-}"
DEST_KEYPAIR="${DEST_KEYPAIR:-$HOME/.config/solana/relayer.json}"
CLUSTER_URL="${CLUSTER_URL:-https://api.devnet.solana.com}"
TRANSFER_SOL="${TRANSFER_SOL:-}"
KEEP_MIN_SOL="${KEEP_MIN_SOL:-0.01}"

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing command: $1"
    exit 1
  }
}

require_cmd solana
require_cmd solana-keygen
require_cmd awk

if [[ -z "$SOURCE_KEYPAIR" ]]; then
  echo "SOURCE_KEYPAIR is required"
  echo "Example: SOURCE_KEYPAIR=/home/a/.config/solana/id.json TRANSFER_SOL=2 ./scripts/rotate-relayer-wallet.sh"
  exit 1
fi

if [[ ! -f "$SOURCE_KEYPAIR" ]]; then
  echo "SOURCE_KEYPAIR not found: $SOURCE_KEYPAIR"
  exit 1
fi

mkdir -p "$(dirname "$DEST_KEYPAIR")"
if [[ -f "$DEST_KEYPAIR" ]]; then
  echo "DEST_KEYPAIR already exists: $DEST_KEYPAIR"
  echo "Refusing to overwrite existing keypair."
  exit 1
fi

echo "Creating new relayer wallet at: $DEST_KEYPAIR"
solana-keygen new --no-bip39-passphrase --silent --outfile "$DEST_KEYPAIR" >/dev/null

SOURCE_PUBKEY="$(solana-keygen pubkey "$SOURCE_KEYPAIR")"
DEST_PUBKEY="$(solana-keygen pubkey "$DEST_KEYPAIR")"

echo "Source: $SOURCE_PUBKEY"
echo "Relayer: $DEST_PUBKEY"

SOURCE_BALANCE_SOL="$(solana balance "$SOURCE_PUBKEY" --url "$CLUSTER_URL" | awk '{print $1}')"
echo "Source balance: ${SOURCE_BALANCE_SOL} SOL"

if [[ -z "$TRANSFER_SOL" ]]; then
  TRANSFER_SOL="$(awk -v bal="$SOURCE_BALANCE_SOL" -v keep="$KEEP_MIN_SOL" 'BEGIN { v=bal-keep; if (v<0) v=0; printf "%.9f", v }')"
fi

if awk -v amt="$TRANSFER_SOL" 'BEGIN { exit !(amt <= 0) }'; then
  echo "Nothing to transfer (TRANSFER_SOL=$TRANSFER_SOL)."
  echo "Relayer keypair created, but not funded."
  exit 0
fi

echo "Transferring $TRANSFER_SOL SOL to relayer..."
solana transfer "$DEST_PUBKEY" "$TRANSFER_SOL" \
  --from "$SOURCE_KEYPAIR" \
  --allow-unfunded-recipient \
  --url "$CLUSTER_URL" \
  --no-wait

echo
echo "Done."
echo "Set these env vars:"
echo "  ANCHOR_WALLET=$DEST_KEYPAIR"
echo "  RELAYER_WALLET_PATH=$DEST_KEYPAIR"
echo "  RELAYER_PUBKEY=$DEST_PUBKEY"
echo
echo "Verify balances:"
echo "  solana balance $SOURCE_PUBKEY --url $CLUSTER_URL"
echo "  solana balance $DEST_PUBKEY --url $CLUSTER_URL"
