#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "[demo] running hardening pipeline"
./scripts/build.sh
./scripts/idl-client-sanity.sh
cargo test -p ticketing_core --test integration end_to_end:: -- --nocapture

echo "[demo] completed"
