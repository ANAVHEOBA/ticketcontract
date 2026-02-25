#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

./scripts/idl-sync.sh

echo "[sanity] validating IDL against backend/frontend call paths"
node ./app/idl-sanity.js

echo "[sanity] done"
